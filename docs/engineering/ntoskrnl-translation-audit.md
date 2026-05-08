# ntoskrnl C→Rust Translation Audit

Last updated 2026-05-08 (afternoon — trap-exit chain + HAL bridges
re-probe). Treat any prior version of this document as suspect — the
shim layer changed significantly between 2026-05-06 and 2026-05-08,
and the arm64 baseline was unblocked.

## Where we are

**~99.6% of the original ntoskrnl C body of work is now Rust.** The
active C in the default i386/amd64 build is down to **9 narrow shim
files totalling 1,023 LOC**, the smallest the surface has ever been.
Five entire shim files were retired between the last audit and now:
`mm_sysldr_shims.c`, `arm64_uart.c`, `kd_rust_shims.c`,
`cm_rust_shims.c`, and `kernel_c_helpers.c`'s legacy bulk
(it survives at 33 LOC purely as the `ps_i.h` re-include trick).

Counts (from `find … | xargs wc -l`):

| Bucket | LOC |
|---|---:|
| Original ntoskrnl C body of work (excl. shims) | ~268,600 |
| Rust under `sdk/lib/rust/ntoskrnl-rust/src/` | ~440,200 |
| Active C in default i386/amd64 build (excl. shims) | 0 |
| C shim layer (9 files in `ntos.cmake`) | **1,023** |

Shim layer files (LOC):

| File | LOC | Status |
|---|---:|---|
| `ke/ke_rust_shims.c` | 461 | residual "big trampoline" — TSS init + KiAttachProcess + KiThreadStartup + must-stays |
| `ntprintf_trampoline.c` | 170 | minimal-trampoline (variadic `va_arg` pull) |
| `variadic_shims.c` | 77 | minimal-trampoline (variadic) |
| `io/iomgr/io_rust_shims.c` | 70 | minimal-trampoline reached |
| `mm/mm_rust_shims.c` | 69 | minimal-trampoline reached on i386/amd64 |
| `kdbg/kdbg_rust_shims.c` | 62 | minimal-trampoline reached |
| `lto_workaround.c` | 45 | LLVM i386 fat-LTO isolation boundary |
| `mm_lock_outlines.c` | 36 | C `__FILE__`/`__LINE__` capture |
| `kernel_c_helpers.c` | 33 | `ps_i.h` re-include only |

The Rust LOC is higher than the original C because (a) bindgen unions
need explicit accessor paths, (b) every output-pointer write goes
through `write_volatile` or an `opaque_ptr` launder, (c) the
augment+isolated-rlib pattern means C trampolines + Rust impls coexist
for a shrinking set of fns. Net work-done metric is closer to 1:1.

**Sibling rlib count: 43.** Up from ~15 at the last audit.

Smoke status:
- **i386: 7/7 PASS**
- **amd64: 7/7 PASS** (red-screen BSOD fixed 2026-05-08 via
  `PsProcess/ThreadInfoClass` table-count correction in
  `6f3dd1ca` — bindgen's Vista enum max diverged from the C
  array length, so info-class probes overran into garbage bytes
  and bugchecked at `[WINLOGON] Calling StartRpcServer`.)
- **arm64: 5/5 PASS** (was 0/7 at last audit). The pre-existing
  baseline hang at `[HIVE] CmCheckRegistry done` resolved as we
  fixed five small bugs through 2026-05-07/08
  (`KeNumberProcessors` width, `_TYPE_OF_MEMORY` constants,
  PrototypePte template, ramdisk smoke-test wiring,
  `RAMDISK_CREATE_OPTIONS` bit layout). One open downstream
  issue — post-boot fastfat BUGCHECK with `STATUS_INSUFFICIENT_RESOURCES`
  — is tracked separately and does not block smoke.

## What's still C

| Bucket | LOC | Notes |
|---|---:|---|
| 9 shim files in `ntos.cmake` | 1,075 | Detailed table above. The "must-stay" subset is small and verified — most of the residual LOC is in `ke_rust_shims.c`. |
| `cmdata.c` Phase B holdouts | ~150 | `MmConsumedPoolPercentage` and a handful of inherent registry-structure tables. |

**Translation candidates with no Rust translation in tree: zero.** Every
C file the default i386/amd64 build pulls in either has its Rust
replacement active, or is a shim that augments a Rust impl living in a
sibling rlib.

The historical `cache/` directory (NEWCC alternate cache subsystem) is
gone as of 2026-05-06. The Rust CC subsystem (`ntoskrnl-rust/src/cc/`)
is the only cache implementation; if it needs work, the fix lands
directly in Rust, not via porting a parallel C version.

## What changed since 2026-05-06

The shim layer collapsed in two phases:

1. **Bulk dead-trampoline sweep (2026-05-06)** — `--allow-multiple-definition`
   was masking ~150 trampolines in `ke_rust_shims.c` / `mm_rust_shims.c`
   that lld was already discarding in favour of the `shim_exports.rs`
   versions. ~700 LOC dropped across mm/io/ke/cm. 7 of 8 mm-avl
   trampolines collapsed (only `RtlpInsertAvlTreeNode` keeps a guard).
   `kernel_c_helpers.c` AVL block + amd64 `KfRaiseIrql` + i386
   `InterlockedExchange` cdecl wrappers + Mm globals all dropped.

2. **Per-file "minimal trampoline" push (2026-05-06 → 2026-05-08)** — every
   non-`ke_rust_shims.c` file driven down to where headers are limited to
   `<ntdef.h>` + `<stdarg.h>` + `<stddef.h>` + `<stdint.h>`, no
   `<ntoskrnl.h>`, no FORCEINLINE bodies, no struct field accesses, no
   logic macros. `kernel_c_helpers.c` split into 5 focused
   trampolines (one per distinct must-stay reason: variadics,
   LTO workaround, `__FILE__`/`__LINE__`, `ps_i.h` re-include, and the
   PS-globals which moved out to `ntoskrnl-rust::ps::ps_globals`).
   `mm_sysldr_shims.c` translated and deleted entirely (-1.4K LOC of
   `MmLoadSystemImage` + dependency chain).

3. **`ke_rust_shims.c` trap-exit chain port + HAL re-probes
   (2026-05-08, afternoon)** — the audit's prior "multi-month" estimate
   for the i386 trap-exit chain turned out to be off by orders of
   magnitude. All three trap-exit FORCEINLINEs
   (`KiExitV86Trap`, `KiExitTrapDebugChecks`,
   `KiExitSystemCallDebugChecks`) plus the `KiDumpTrapFrame` debug
   helper translated to direct `ke-macros` exports in one session;
   513 → 461 LOC. The `_saved_*` rename machinery is gone for trap
   exits. Same session re-probed the audit's "rustc 1.92
   stdcall/fastcall ABI bug" — disasm of `KiExitV86Trap` /
   `KiEndInterrupt` shows correct fastcall (arg in `ecx`) and stdcall
   (`_X@N` decoration) emission, so the bug doesn't reproduce in
   1.92.0 (ded5c06cf) and `KiEndInterrupt` plus four HAL bridges
   (`HalQuery/Set/VectorToIDTEntry`, `KeGetTrapFramePc`) all promoted
   to bare-name `extern "system"` exports. One latent ABI bug
   surfaced and fixed (`kdb_cli.rs:188` had `KeGetTrapFramePc` in an
   `extern "C"` block — mismatch was hidden by the cdecl trampoline).

The "intentional-stay" classifications listed in earlier audits all
turned out to be tractable. As of this audit the only categories that
have been **proven** to require C are documented in the Verified
Must-Stay Set below — everything else, including categories that
*looked* unkillable a session or two ago, fell to one of the lessons
captured in the bottom half of this doc.

## Active in Rust (default build)

Counts are *active modules* in `mod.rs`. As of 2026-05-08, **every
Rust file in the tree is wired in** — the previously-parked
`mm/section_missing.rs` (1,554 LOC) plus its C source
`ntoskrnl/mm/section_missing_funcs.c` (3,458 LOC) were deleted after
audit confirmed every function it exported was already live in
`mm/section.rs`, `mm/section_pe.rs`, or `ARM3/section.rs`.

| Subsystem | Active mods | LOC active |
|---|---:|---:|
| KE (kernel) | 67 | ~54,100 |
| ARM3 (mm internals) | 25 | ~46,000 |
| iomgr | 26 | ~42,100 |
| EX (executive) | 32 | ~38,100 |
| SE (security) | 19 | ~35,200 |
| config (registry CM) | 25 | ~34,900 |
| pnpmgr | 20 (incl. arb) | ~26,400 |
| ps (process) | 21 | ~23,700 |
| ob (object) | 12 | ~22,600 |
| fsrtl | 18 | ~21,000 |
| kdbg (KDB) | 12 | ~13,800 |
| mm (mm internals) | 13 | ~11,900 |
| kd64 | 12 | ~12,100 |
| fstub | 5 | ~12,300 |
| cc (cache) | 8 | ~6,200 |
| dbgk / po / inbv / lpc / kd / wmi / vdm / arch dirs / rtl / vf / top-level | ~50 | ~30,000 |
| **TOTAL active (i386/amd64)** | **~325** | **~440,200 LOC** |

Parked translations: **0**. Every Rust file in tree is wired in.

## Forward path: cargo-driven build PoC

The strategic goal motivating shim minimization isn't the LOC count
itself — it's switching ntoskrnl from cmake to cargo as the build
driver. Cargo already produces the kernel staticlib
(`libkernel_rust.a`); we want cargo to also drive the final link,
eliminating the cmake step.

What this requires from the shim layer:

- **Headers limited to `<ntdef.h>` + `<stdarg.h>` + `<stddef.h>` + `<stdint.h>`.**
  cc-rs in a `build.rs` script can compile minimal-trampoline files
  cleanly. Files that pull `<ntoskrnl.h>` need a separate `cc::Build`
  invocation (or to be eliminated).
- **Variadic exports stay in C.** Stable Rust can't `#[no_mangle]` a
  C-variadic function, and ntoskrnl exports several
  (`KdbpPrint`, `KdpDprintf`, `RtlStringCbPrintfA`, KDBG ext printers,
  `RtlUnicodeStringPrintf_4wZ_shim`). cc-rs handles them fine — we
  just need them in their own minimal `.c` files. Done.
- **`ke_rust_shims.c` is the residual blocker.** Pulls full ddk for
  TSS+GDT+IDT init (`Ki386InitializeTss` + helpers, ~120 LOC),
  `KiAttachProcess` (~45 LOC, calls 2 FORCEINLINE dispatcher/APC
  release helpers via `_saved_*`), `KiThreadStartup` (~35 LOC,
  layout-sensitive context-switch return), KVM `KiSaveXState` /
  `KiRestoreXState` workaround (must-stay), and a few small must-stay
  trampolines (`Ke386Get{Global,Local}DescriptorTable` dual-decoration,
  `KiComputeIopmOffset` macro capture, ARM64 `KdbpPrint` variadic
  stub). The trap-exit chain port (2026-05-08) was the big residual —
  it's done. What's left is the substantive Phase 1 batch
  (KiAttachProcess + KiThreadStartup + Ki386InitializeTss) plus
  documented must-stays.
- **Resource compilation.** `.rc` files (versioninfo, message tables)
  via the `embed-resource` cargo crate.
- **PE-COFF link flags.** cargo `[profile.X]` + `-Clink-arg=` covers
  `-nostdlib`, `-fuse-ld=lld`, `--image-base=...`, `-entry=...`,
  `--subsystem=native:5.01`.
- **Host tools** (`pefixup`, `fix_lld_forwarding.py`) — build.rs scripts
  can compile/run as a pre-link step.
- **`--allow-multiple-definition` link flag.** Currently used to handle
  ABI-anchor-vs-Rust-impl symbol conflicts. Should be eliminable as
  trampolines fully delegate; tracking elimination is the audit's job.

Definition of done: `cargo build -p ntoskrnl --target <arch>` produces
a working `ntoskrnl.exe` for all 3 arches without invoking cmake, all
3 smoke tests pass on the cargo-driven binary, `tools/scan_ud2.py
--check` is clean, and the cmake path can be removed (or kept as a
parallel CI check).

## Verified must-stay set

These have been re-probed and **proven** to require the full kernel
header chain or a C compile. Not candidates for further elimination
until the underlying constraint moves.

- **Variadic exports** (`KdbpPrint`, `KdpDprintf`, `RtlStringCbPrintfA`,
  KDBG ext printers, `RtlUnicodeStringPrintf_4wZ_shim`,
  `ntprintf_trampoline.c`'s callees). Stable Rust limitation. Already
  isolated to minimal trampolines — fine.
- **`SepInitTokensShim`** (i386, in `lto_workaround.c`). LLVM fat-LTO
  miscompilation: `ud2` inserted after `@ObInitializeFastReference@8`.
  None of `opaque_ptr`, `compiler_fence(SeqCst)`, `core::hint::black_box`
  defeat it. C-side execution outside the Rust LTO scope is the only
  working workaround. Re-probed 2026-05-07.
- **`KiSaveXState` / `KiRestoreXState`** (amd64, in `ke_rust_shims.c`).
  KVM bug: `xrstors64` faults on StateSaveArea pages that look valid
  via PTE self-referencing. Force `fxsave64`/`fxrstor64` instead.
- **`MmLockSectionSegment` trio** (`mm_lock_outlines.c`). Uses
  `__FILE__`/`__LINE__` for call-site recording in the underlying
  `_Mm*` primitives.
- ~~**3 i386 trap-exit `_saved_*` outlines**~~ — CLOSED 2026-05-08.
  All three (`KiExitV86Trap`, `KiExitTrapDebugChecks`,
  `KiExitSystemCallDebugChecks`) translated to direct Rust impls in
  `ke-macros`, plus `KiDumpTrapFrame` debug helper. The "PSEH-
  intertwined with trap-entry chain" diagnosis was wrong — the C
  source `ke/i386/traphdlr.c` is already commented out in
  `ntos.cmake` and the FORCEINLINE bodies were straight imperative
  code (no PSEH machinery in Rust required). Rust callers in
  `ke/i386/traphdlr.rs` link directly to the new bare-name exports.
- ~~**rustc 1.92 stdcall/fastcall ABI bug**~~ — CLOSED 2026-05-08.
  Re-probed against rustc 1.92.0 (ded5c06cf, 2025-12-08); disasm of
  `KiExitV86Trap` and `KiEndInterrupt` shows correct fastcall (arg
  in `ecx`) and stdcall (`_X@N` decoration) emission. `KiEndInterrupt`
  + 4 HAL stdcall bridges (`HalQuerySystemInformation`,
  `HalSetSystemInformation`, `HalVectorToIDTEntry`,
  `KeGetTrapFramePc`) all promoted to bare-name `extern "system"`
  exports without any inline-asm workaround. The audit's prior
  classification was either fixed in 1.92.0 release or was
  misdiagnosed.
- **Rust dual-decoration constraint** (i386):
  `Ke386Get{Global,Local}DescriptorTable` has both an `extern "C"`
  cdecl caller (kdbg/kdb_cli.rs) and an `extern "system"` stdcall
  caller (ke/i386/cpu.rs) referencing the same Rust source name.
  Rust forbids two source-level fn definitions of the same name, so
  the cdecl impl lives bare-name in `ke-macros` and the stdcall
  `_X@4` decoration is supplied by a one-line C trampoline calling
  the cdecl `_rs_*` variant. ~10 LOC; not worth eliminating until
  one of the two callers normalizes.
- **`ps_i.h` re-include trick** in `kernel_c_helpers.c`. The C file
  re-includes `ps_i.h` with a `#pragma push/pop` of `static` and
  `_static_*` rename to externalize `PsProcessInfoClass[]` /
  `PsThreadInfoClass[]`. A parallel Rust translation exists at
  `ps/info_class.rs` (under `_rs_*` symbols), byte-equal to the C
  arrays — but renaming the Rust statics to drop the prefix
  reproducibly bugchecks i386 with `STATUS_DATATYPE_MISALIGNMENT`
  even though the bytes are identical (purely positional, .rdata
  offset-dependent). Not blocking; worth re-investigation.

## Stale-claim watch (carry forward)

The pattern from prior sessions, restated because it keeps paying off:

> Many "Rust crashes / hangs" excuses in shim or DISABLED docs predate
> the workspace `lto = false` + `codegen-units = 256` firewall, the
> `opaque_ptr` laundering discipline, and `verify_abi.py`. **Re-probe
> before treating any classification as a settled answer.**

Specific examples worth remembering:

- **kd64** ("recursive exception flood on amd64"): silently fixed
  before re-test. Just needed wiring.
- **lpc/connect** ("captures(none) cascade"): file already had the
  right `opaque_ptr` defenses.
- **pnpres** ("translated, never wired"): 2 small fixes, then 7/7.
- **iomgr/ramdisk on arm64** ("PE layout issue → 0x6B crash"): real
  fix was `_force_iomgr_symbols` to pull statics out of the staticlib
  archive, plus removing the cfg gate.
- **ke/amd64/usercall** ("return-address corruption BSOD"): real bug
  was importing `KeGdiFlushUserBatch` as `extern "system" fn ...`
  when it's a global `PGDI_BATCHFLUSH_ROUTINE` data symbol. Fix:
  import as `static mut` and call via `(KeGdiFlushUserBatch.unwrap())()`.
- **`IopInitializeDriverModule`** ("complex delegation"): real bug was
  an `extern "C"`/`extern "system"` ABI mismatch on
  `IopReadyDeviceObjects` in driver.rs that had been latent for months.
  amd64+arm64 link silently because NTAPI=cdecl there; i386 stdcall
  callee + cdecl call-site folded to ud2.
- **All 10 io captures(none) workarounds**: every one solved by the
  workspace LTO firewall. Not one needed structural changes.
- **`PsProcessInfoClass` table count** (this audit's reason for
  amd64 going red): bindgen `MaxProcessInfoClass = 51` (Vista)
  diverged from the C array length 48 (NT 5.x re-include). Hard-code
  the count to match the C array.
- **Trap-exit chain "multi-month project"** (2026-05-08): the audit
  estimated translating `KiExitV86Trap` + `KiExitTrapDebugChecks` +
  `KiExitSystemCallDebugChecks` would take months because they were
  "PSEH-intertwined with KiEoiHelper / KiDeliverApc / KfRaiseIrql /
  KiDumpTrapFrame / KeBugCheckEx interactions". Actual time: a few
  hours. Every callee was already in Rust (`KiEoiHelper` in
  `traphdlr.rs`, `KfRaiseIrql`/`KfLowerIrql` in shim_exports.rs,
  `KiUserTrap` in ntkrnl-sys, `KiDeliverApc` in `ke/apc.rs`); the C
  source `ke/i386/traphdlr.c` was already commented out so the only
  callers are Rust; and "PSEH-intertwined" was wrong — the
  FORCEINLINE bodies were straight imperative code with no SEH
  machinery. Lesson: estimate by inspecting what actually has to
  change, not by guessing from header comments.
- **`KiEndInterrupt` "rustc 1.92 ABI bug"** (2026-05-08): the audit
  said this had to stay in C because rustc 1.92 emits cdecl call
  sequences for stdcall/fastcall externs. Re-probed against 1.92.0
  release: disasm shows correct emission (`call @KfRaiseIrql@4`
  with arg in `ecx` for fastcall; `call _HalEndSystemInterrupt@8`
  with pushed args for stdcall). Either fixed in 1.92.0 or
  misdiagnosed. Translated cleanly.

When picking a parked file or shim to revive, the cheapest first step
is often: re-enable it, build, smoke. The bug may already be gone.

## Common bug patterns we've seen

Track these — they keep recurring across translations.

1. **NT_ERROR vs NT_SUCCESS.** `NT_ERROR(x)` is `(x >> 30) == 3`, NOT
   `x < 0` (that's `!NT_SUCCESS(x)` and includes warnings).
2. **Bindgen anonymous union offsets.** `__bindgen_anon_1` can point
   to the wrong union member. KTHREAD::ApcState lives inside an anon
   union on every arch.
3. **Wrong enum constants.** Match arms using sequential ordinals
   instead of actual enum values (DeviceProperty, KeyValueInformationClass).
4. **Bindgen flexible-array indexing.** Bindgen renders trailing
   `Foo[]` as `Foo: [T; 1]`; indexing past 0 panics on bounds-check.
   Take the array base via `addr_of!` once, use `.add(i)` from there.
5. **`extern "C"` vs `extern "system"` ABI mismatch.** On i386, NTAPI
   functions are stdcall; declaring them `extern "C"` produces a cdecl
   call-site that blows the stack frame. amd64+arm64 link silently.
   Run `verify_abi.py` or `nm` first when ud2 shows up after a call.
6. **Bindgen enum max ≠ C array length.** When bindgen emits a Vista+
   `MaxXxxInfoClass` and the C array uses an NT 5.x-only re-include,
   they diverge. Hard-code the count to match the C array.
7. **Wide string encoding.** `b"..."` byte strings used as UTF-16. Must
   be `[u16; N]` or use the `wstr!` macro.
8. **Static lifetime on inline arrays.** Inline `[u16; N]` arrays as
   temporaries → dangling pointers; declare as `static`.
9. **Bitfield bit-position errors.** `RAMDISK_CREATE_OPTIONS` (and
   similar union/bitfield C definitions) need the bit layout matched
   exactly; collapsing to a single `: ULONG` field and setting `=1`
   sets bit 0, not the named bit. Use explicit `1 << N` constants.
10. **i386 has 3 decorations, not 1.** Before deleting a "dead" shim,
    grep for `_X` (cdecl), `_X@N` (stdcall), AND `@X@N` (fastcall).

## Recipe for re-probing a shim

1. Read the in-source comment for the original symptom.
2. Grep for Rust callers across all 3 i386 decorations (`_X`,
   `_X@N`, `@X@N`).
3. If used: drop the C trampoline, declare the underlying Rust fn
   directly with the right ABI. Verify decoration via `nm`.
4. Build all 3 arches. `python3 tools/scan_ud2.py --check build-i386`
   is the canary — unexpected ud2 in a named function is the smoking
   gun.
5. Run smoke. Green on all 3 = ghost. Failure = either a real bug
   surfaced (ABI mismatch, missing volatile, etc.) or a genuine
   workaround that has to stay.

> When you see ud2 + folded success path on i386, run `nm` on the
> binary and check `verify_abi.py` BEFORE going down the captures(none)
> rabbit hole. Both produce the same disasm pattern; ABI is a
> 1-second check.

## Verification tools

| Tool | What it checks | Location |
|---|---|---|
| `verify_abi.py` | extern "C" vs extern "system" mismatch | `tools/rustconv/rustconv/verify_abi.py` |
| `verify_ffi.py` | ABI + params + C macros + flexible arrays | `tools/rustconv/rustconv/verify_ffi.py` |
| `verify_constants.py` | Constant values vs C headers | `tools/rustconv/rustconv/verify_constants.py` |
| `lint_kernel.py` | Unprotected output pointer writes | `tools/lint_kernel.py` |
| `scan_ud2.py` | LLVM miscompilation (ud2 in named functions) | `tools/scan_ud2.py` |

`nm` decoration check (1 second, faster than any of the above):

```
i686-w64-mingw32-nm build-i386/ntoskrnl/ntoskrnl.exe | grep <FunctionName>
```

`_X` = cdecl, `_X@N` = stdcall, `@X@N` = fastcall. If the Rust caller's
extern decl produces a different decoration than the symbol's actual
decoration, that's the bug.

## What this audit doesn't measure

- **Boot correctness.** A line-count metric says nothing about whether
  the Rust version actually works under load. Smoke is the source of
  truth; this audit is just the LOC ledger backing the smoke story.
- **Per-arch differentials.** i386 and amd64 are at parity. arm64 just
  unblocked from 0/7 → 5/5 — there are arm64-only translation gaps
  (e.g. `mm_rust_shims.c`'s `Arm64DumpPte` debug helper) and one open
  downstream issue (post-boot fastfat BUGCHECK) that aren't reflected
  in LOC counts.
- **Implicit shim debt.** Even at 1,023 LOC, much of
  `ke_rust_shims.c` pulls `<ntoskrnl.h>` for the substantive bodies
  (KiAttachProcess, KiThreadStartup, Ki386InitializeTss). Header-
  weight matters more than line-count for the cargo-driven build —
  trampolines that delegate to `_rs_*` impls in sibling rlibs are
  cheap to keep; the substantive bodies aren't.

## How these numbers are computed

```bash
# Total ntoskrnl C body of work (excludes shims; ntoskrnl/tests/ +
# ntoskrnl/cache/ both deleted earlier):
find ntoskrnl -name "*.c" \
  | grep -v -E '_rust_shims\.c|kernel_c_helpers\.c|variadic_shims\.c|lto_workaround\.c|mm_lock_outlines\.c|ntprintf_trampoline\.c' \
  | xargs wc -l   # → ~268,600

# Active C in default build (excludes shims):
grep -E '^\s+\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/.*\.c' ntoskrnl/ntos.cmake \
  | grep -v '^\s*#' \
  | grep -v -E '_rust_shims\.c|kernel_c_helpers\.c|variadic_shims\.c|lto_workaround\.c|mm_lock_outlines\.c|ntprintf_trampoline\.c' \
  | sed 's|^\s*\${REACTOS_SOURCE_DIR}/||; s|).*||; s| .*||' \
  | sort -u | xargs wc -l   # → 0

# Shim layer (active in build):
grep -E '^\s+\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/.*\.c' ntoskrnl/ntos.cmake \
  | grep -v '^\s*#' \
  | sed 's|^\s*\${REACTOS_SOURCE_DIR}/||; s|).*||; s| .*||' \
  | sort -u | xargs wc -l   # → 1,023

# Rust LOC across ntoskrnl-rust/:
find sdk/lib/rust/ntoskrnl-rust/src -name "*.rs" | xargs wc -l   # → ~440,200

# Sibling rlib count:
find sdk/lib/rust -maxdepth 2 -name "Cargo.toml" | wc -l   # → 43

# Parked Rust translations (file on disk, not in mod.rs):
for d in sdk/lib/rust/ntoskrnl-rust/src/*/mod.rs; do
  dir=$(dirname "$d")
  diff <(find "$dir" -maxdepth 1 -name "*.rs" ! -name mod.rs \
         | xargs -n1 basename | sed 's/\.rs$//' | sort -u) \
       <(grep -E '^pub mod ' "$d" | awk '{print $3}' | sed 's/;//' | sort -u) \
    | grep '^<' || true
done   # → empty
```
