# Resolved: Debug `mm/ARM3/virtual.rs` smoke regression

Resolution: i386 smoke is back to 7/7 with `virtual.rs` still active. The boot-blocking bug was a bad local Rust enum constant (`TableFoundNode = 0`) rather than an LLVM `ud2` cascade; the fixed code now uses the bindgen/C value (`1`). The original handoff notes are preserved below for context.

You are walking into a multi-arch ReactOS kernel port that compiles to PE-COFF for **i386, amd64, and arm64** via llvm-mingw. Background lives in `AGENTS.md` (project root) — read it first if you need orientation. The relevant excerpt: kernel subsystems are progressively translated from C to Rust under `sdk/lib/rust/ntoskrnl-rust/`, linked via the `kernel-rust` umbrella staticlib into `ntoskrnl.exe`. The QEMU smoke tests (`tests/qemu/test_smoke.py`) are the source of truth for whether the kernel boots. Translation philosophy: **never gate broken Rust off the build; never re-enable C to dodge a regression**. See `AGENTS.md` "Translation Philosophy" for full rules.

## What just happened

We re-enabled the parked Rust translation `sdk/lib/rust/ntoskrnl-rust/src/ARM3/virtual.rs` and replaced two stub functions with full translations:

- `NtAllocateVirtualMemory` — was `STATUS_NOT_IMPLEMENTED` stub, now full ~280 LOC translation of the C `NtAllocateVirtualMemory` (786 LOC).
- `NtFreeVirtualMemory` — same story, now full ~210 LOC translation of the C version (480 LOC).

The C source `ntoskrnl/mm/ARM3/virtual.c` has been removed from `ntoskrnl/ntos.cmake` (commented out). The Rust file is enabled in `sdk/lib/rust/ntoskrnl-rust/src/ARM3/mod.rs` for all three arches. A `_force_virtual_symbols` block was added to `sdk/lib/rust/ntoskrnl-rust/src/lib.rs` to defeat the rlib→staticlib DCE that strips `#[no_mangle]` exports (same pattern used for kd64 — see the `_force_kd64_symbols` block right above it as a reference).

The translation was done by hand (no rustconv API call). Approximate diff: 4 files, ~960 lines added.

## Original smoke status (clean rebuild — `--skip-clean-build` removed)

| Arch  | Result | Failure |
|-------|--------|---------|
| i386  | 0/7    | `BUGCHECK 0x64 (STATUS_CONFLICTING_ADDRESSES)` at `RtlpCreateUserStack → ZwAllocateVirtualMemory(MEM_COMMIT)`. Plus 2 ud2 in unrelated functions: `_PopSetSystemPowerState@8`, `_MmPageOutPhysicalAddress@4`. |
| amd64 | 0/7    | Boot stalls in HAL `x86bios.c:353 Invalid IO port read access (port: 0x3c0)` loop, never reaches SMSS. ~23.4 KB serial. |
| arm64 | 0/7    | Pre-existing baseline — hangs after `[HIVE] CmCheckRegistry done`, 13,449 bytes serial. **Not introduced by this change.** |

The earlier "i386 7/7 PASS" we observed mid-investigation was a `--skip-clean-build` cache flake. With cargo's incremental cache nuked, i386 deterministically fails as above.

## Original hypothesis (superseded)

This is the **LLVM CGU `captures(none)` cascade** documented in `docs/engineering/rust-kernel-patterns.md` (sections "Why write_volatile alone can't work", "Validated: opaque_ptr Inline Asm Barrier", "Current Workaround: CGU=256"). Summary:

- LLVM deduces `readnone captures(none)` on pointer parameters of `#[no_mangle] pub unsafe extern "system"` functions.
- Callers then pass `poison` instead of the real pointer.
- Stores through poison are UB → LLVM emits `ud2`.
- The cascade is **interprocedural** — adding code to the staticlib (like enabling `virtual.rs`) reshuffles LLVM's CGU partitioning and can expose latent ud2 in *unrelated* functions.

The two ud2 sites (`PopSetSystemPowerState`, `MmPageOutPhysicalAddress`) are in `po/` and `mm/` — both untouched by this change. Their appearance is the canonical cascade signature.

`STATUS_CONFLICTING_ADDRESSES` from `ZwAllocateVirtualMemory(MEM_COMMIT)` on a freshly-reserved range initially suggested `MiCheckForConflictingNode` was being called with poisoned arguments. The trace later showed the lookup was succeeding (`TableFoundNode == 1`) and Rust was comparing it against the wrong local constant (`0`).

CGU=256 already mitigates the cascade for the existing baseline, but enabling `virtual.rs` pushes the staticlib past whatever threshold causes new attribute chains to form.

## Files modified by this change

```
ntoskrnl/ntos.cmake                                       # virtual.c commented out
sdk/lib/rust/ntoskrnl-rust/src/ARM3/mod.rs                # virtual.rs uncommented (enabled for all 3 arches)
sdk/lib/rust/ntoskrnl-rust/src/ARM3/virtual.rs            # 2 stubs → full translations + 3 missing `mut`s
sdk/lib/rust/ntoskrnl-rust/src/lib.rs                     # added _force_virtual_symbols block
```

The diff is also saved in `git stash@{0}` (named `"virtual.rs port — stubs replaced, integrated but smoke-broken (LLVM ud2 cascade hypothesis)"`).

## Original task (completed)

Get the i386 smoke back to green (or at least back to baseline, which was 7/7 before this change). amd64 is allowed to stay at its pre-existing baseline (6/7 with `[EXPLORER] _tWinMain entered` missing — that's the documented taskbar regression, unrelated to translation work). arm64 is allowed to stay at its pre-existing baseline (0/7).

Do **not**:
- Revert `virtual.rs` to a stub.
- Re-enable `virtual.c` in `ntos.cmake` to dodge the failure.
- Delete the `virtual.rs` file.
- Add `#[cfg(target_arch = ...)]` gates to hide the failure on specific arches.

Do:
- Apply `opaque_ptr` barriers where they break the cascade. The barrier is at `sdk/lib/rust/ntkrnl-sys/src/lib.rs:50-65` — `pub unsafe fn opaque_ptr<T>(ptr: *mut T) -> *mut T`. The pattern is `let param = opaque_ptr(param);` as the first statement of every `#[no_mangle]` function. Most of `virtual.rs` already does this (see the existing functions like `NtProtectVirtualMemory` at line 2305 — every pointer param gets `opaque_ptr` immediately). Verify both new functions (`NtAllocateVirtualMemory`, `NtFreeVirtualMemory`) follow this pattern correctly.
- Investigate the ud2 sites with `python3 tools/scan_ud2.py build-i386` — the tool reports the exact functions and offsets. Cross-reference with `objdump -d build-i386/ntoskrnl/ntoskrnl.exe`.
- If `opaque_ptr` on the new functions doesn't clear the cascade, look at the *callers* of `NtAllocateVirtualMemory` / `NtFreeVirtualMemory` (Zw* wrappers in `ke_rust_shims.c`, the SSDT in `MainSSDT`, syscall dispatch). Also worth checking that the `_force_virtual_symbols` block in `lib.rs` references each function correctly — the array length is hardcoded to 33 + 1 (split because of an array-size constraint I hit during integration).
- If `opaque_ptr` doesn't help, the next-most-likely root cause is a real translation bug. Cross-check `NtAllocateVirtualMemory` against `ntoskrnl/mm/ARM3/virtual.c:4491-5276` and `NtFreeVirtualMemory` against `5283-5762` line by line. The translation matches C control flow but the SEH wrappers are dropped (DUMMY_PSEH means `_SEH2_EXCEPT` blocks are dead code on this build — the existing `NtProtectVirtualMemory` translation in this file shows the convention: `ProbeForWritePointer` + direct dereference, no SEH). Watch for: type width mismatches between bindgen `SIZE_T` (= `c_ulong`, u32 on i386, u64 on amd64) vs `base::SIZE_T` (= `usize`); the `MMVAD_LONG` / `EPROCESS` field accesses for `CommitCharge` / `VirtualSize` / `CommitChargePeak` (bindgen widths differ by arch, fixed via `as ntkrnl_sys::ddk::SIZE_T` casts).
- The ARM64-only "stale FreeLDR identity-mapping PTE clear" block from `virtual.c:4819-4869` was deliberately left out of the Rust translation (commented note in the file). ARM64 boot fails before reaching user-mode allocation anyway, so this won't affect smoke. Restore it from C source if/when arm64 boot is fixed past `CmCheckRegistry`.

## How to build + verify

From the repo root (`/home/curi/Projects/os-playground/vibeos`):

```bash
# Clean rebuild for i386 — DO NOT use --skip-clean-build for verification
rm -rf build-i386/rust-kernel-target build-i386/ntoskrnl/rust-kernel-target \
       build-i386/ntoskrnl/libkernel_rust.a build-i386/ntoskrnl/ntoskrnl.exe \
       build-i386/livecd.iso sdk/lib/rust/target
cd build-i386 && ninja livecd
cd ..

# Run i386 smoke (full)
.venv/bin/pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 600 --skip-clean-build

# Static checks (faster, no QEMU)
python3 tools/scan_ud2.py build-i386                          # Should report only known ud2
python3 tools/scan_ud2.py --check build-i386                  # Exit 1 if unexpected ud2
python3 tools/lint_kernel.py                                  # captures(none) vulnerable writes
bash tools/verify_sacred_primitives.sh build-i386             # IRQL/spinlock primitives intact
```

Tip: `tools/scan_ud2.py` runs in ~30s and tells you whether the cascade is still present without needing to boot. Use it as your inner-loop signal.

For amd64 + arm64, the pattern is the same with `build-amd64` / `build-arm64` and `--arch amd64` / `--arch arm64`. arm64 needs a ramdisk update before booting:

```bash
mcopy -o -i qemu/ramdisk.img build-arm64/ntoskrnl/ntoskrnl.exe ::/reactos/system32/ntoskrnl.exe
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img
```

## Acceptance criteria

- `python3 tools/scan_ud2.py --check build-i386` exits 0.
- `tests/qemu/test_smoke.py --arch i386 --timeout 600` (no `--skip-clean-build`) passes 7/7.
- amd64 smoke either passes 7/7 OR matches its documented baseline (6/7 with `[EXPLORER] _tWinMain entered` missing).
- arm64 smoke is unchanged from its documented baseline (0/7, hang at `CmCheckRegistry`).
- `virtual.rs`, `mod.rs`, `ntos.cmake`, `lib.rs` modifications preserved (no reverts to gate-off-and-fail-quietly).
- All three arches still build cleanly.

## Reference reading inside this repo

- `AGENTS.md` — project rules, translation philosophy, build commands.
- `docs/engineering/ntoskrnl-translation-audit.md` — current translation status; needs an update once you land.
- `docs/engineering/rust-kernel-patterns.md` — full LLVM cascade theory, `opaque_ptr` validation, RAII guards. Read sections "What Doesn't Work — Failed Approaches" and "Validated: opaque_ptr Inline Asm Barrier" before touching anything.
- `sdk/lib/rust/ntkrnl-sys/src/lib.rs:50-79` — `opaque_ptr` / `opaque_ptr_const` definitions.
- `tools/scan_ud2.py`, `tools/lint_kernel.py`, `tools/verify_sacred_primitives.sh` — your primary diagnostic tools.
- `sdk/lib/rust/ntoskrnl-rust/src/ARM3/virtual.rs` lines 2305-2402 (`NtProtectVirtualMemory`) — reference for the established convention this file uses for syscall translations (probe + capture + ObReferenceObjectByHandle + KeStackAttachProcess pattern, no SEH wrappers since DUMMY_PSEH).

## Things that will waste your time if you don't know them

- **Cargo's incremental cache masks regressions.** Always nuke `rust-kernel-target/` and `sdk/lib/rust/target` before considering any "fixed" claim verified.
- **`MEM_TOP_DOWN` is `ULONG_PTR` (usize) in `ntkrnl-sys::mm`** but the C macro it represents is a `ULONG`. The translation casts via `as ULONG` for the bitwise op. Don't "fix" this back to a usize comparison.
- **`set_CommitCharge` / `set_MemCommit` etc. take `_ as _`** — bindgen bitfield setters use `c_ulong`, which is `u32` on i386 and `u64` on amd64. Always cast via `as _` and let inference pick the per-arch width.
- **`ExFreePool` is `extern "stdcall"` on i386** — must come from `ntkrnl_sys::ddk::ExFreePool`, NOT a local `extern "C"` declaration. The current translation uses `use ntkrnl_sys::ddk::ExFreePool;` for this.
- **`size_of::<MMVAD_LONG>() as _`** matters because the various `Ps*Quota` and `Ex*Pool*` calls take bindgen `SIZE_T` (per-arch width), not `base::SIZE_T = usize`.
- **Don't run `cargo build` directly to "verify" anything.** The build runs in two layers (cargo workspace target + per-arch `build-<arch>/rust-kernel-target/`). Always go through `ninja livecd` (or `ninja ntoskrnl`) so both layers stay in sync.

## If you finish or get stuck

If you finish: update `docs/engineering/ntoskrnl-translation-audit.md` to reflect that `virtual.rs` is fully active across all 3 arches (drop it from the "Parked translations" table; bump the percentage). Suggested commit message: `mm/ARM3/virtual: enable Rust port and fix VAD lookup`.

If you get stuck after ≥30 min on the same hypothesis: write a `<200 word note in `docs/engineering/virtual-rs-codex-handoff.md`'s "Update from agent" section below describing what you tried, what scan_ud2 reported at each attempt, and what your current best guess is. Don't push speculative changes that don't pass the acceptance criteria.

## Update from agent

Resolved 2026-04-30. The i386 `STATUS_CONFLICTING_ADDRESSES` smoke failure was not caused by a new `ud2` cascade. A temporary serial trace showed the stack reserve succeeded for `0x30000..0x12ffff`, then the commit for `0x12e000..0x12ffff` failed with `MiCheckForConflictingNode` returning `1`. The root cause was a stale local Rust constant: `virtual.rs` defined `TableFoundNode` as `0`, but the C/bindgen enum value is `1`. Fixing the constant to use `ntkrnl_sys::ddk::_TABLE_SEARCH_RESULT_TableFoundNode` restores the VAD lookup path.

Also fixed while here: `_force_virtual_symbols` is no longer i386-gated in `lib.rs`, and a local `MI_IS_ROSMM_VAD` stub in `virtual.rs` was removed so the file uses the real helper from `ntkrnl_sys::mm`.

Verification: forced clean rebuild of the i386 Rust kernel archive, `ninja -C build-i386 livecd`, `python3 tools/scan_ud2.py --check build-i386`, `bash tools/verify_sacred_primitives.sh build-i386`, `git diff --check`, and full i386 smoke (`.venv/bin/python -m pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 600`) all passed. Screenshot `qemu/i386_smoke.png` shows the desktop/taskbar.
