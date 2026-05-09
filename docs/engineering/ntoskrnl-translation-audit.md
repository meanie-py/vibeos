# ntoskrnl C→Rust Translation Audit

Last updated 2026-05-08. Two strategic milestones landed this round:
the substantive Phase 1 translations (`KiAttachProcess`,
`KiThreadStartup`, `Ki386InitializeTss`) emptied the "haven't been
ported yet" column from `ke_rust_shims.c`, and the cargo-driven
`ntoskrnl.exe` build PoC for i386 boots to desktop and passes 7/7
smoke. Treat any prior version of this document as suspect.

## Where we are

**~99.7% of the original ntoskrnl C body of work is now Rust.** The
active C in the default i386/amd64 build is **9 narrow shim files
totalling 813 LOC** — the smallest the surface has ever been. Of
those 9, **3 are fully header-free** (compile with only
`<stdarg.h>`/`<stddef.h>`/`<stdint.h>` + a small local typedef block):
`ke/ke_rust_shims.c`, `ntprintf_trampoline.c`, `variadic_shims.c`.
The other 6 still pull `<ntoskrnl.h>` and a few internal headers.
Cleaning them up the same way is the next milestone — see "Forward
path" below.

**Cargo-driven build for i386 works end-to-end.** `cargo +nightly
build -p ntoskrnl-cargo --target ntoskrnl-cargo/i686-kernel.json -Z
build-std=…` produces a 3.1 MB `ntoskrnl.exe` (vs. cmake's 4.6 MB)
that boots in QEMU and passes 7/7 smoke. The PoC reuses cmake's `.obj`
files for now; future iterations will compile shims via cc-rs once
the remaining 6 shims drop their kernel-header dependencies.

Counts (verified by `wc -l`):

| Bucket | LOC |
|---|---:|
| Original ntoskrnl C body of work (excl. shims) | 268,436 |
| Rust under `sdk/lib/rust/ntoskrnl-rust/src/` | 441,262 |
| Active C in default i386/amd64 build (excl. shims) | 0 |
| C shim layer (9 files in `ntos.cmake`) | **813** |

Shim layer files (LOC, header status):

| File | LOC | Headers | Status |
|---|---:|---|---|
| `ke/ke_rust_shims.c` | 251 | header-free ✓ | minimal-trampoline |
| `ntprintf_trampoline.c` | 170 | header-free ✓ | variadic `va_arg` pull |
| `variadic_shims.c` | 77 | header-free ✓ | variadic |
| `io/iomgr/io_rust_shims.c` | 70 | `<ntoskrnl.h>` + `<ntstrsafe.h>` + `<debug.h>` + `internal/io_i.h` | minimal-trampoline |
| `mm/mm_rust_shims.c` | 69 | `<ntoskrnl.h>` + `<debug.h>` + `mm/ARM3/miarm.h` + `internal/arch/mm.h` | minimal-trampoline |
| `kdbg/kdbg_rust_shims.c` | 62 | `<ntoskrnl.h>` + `kdb.h` + `kd/kdterminal.h` | minimal-trampoline |
| `lto_workaround.c` | 45 | `<ntoskrnl.h>` + `<debug.h>` | LLVM i386 fat-LTO isolation |
| `mm_lock_outlines.c` | 36 | `<ntoskrnl.h>` + `<debug.h>` + `mm/ARM3/miarm.h` | C `__FILE__`/`__LINE__` capture |
| `kernel_c_helpers.c` | 33 | `<ntoskrnl.h>` + `<debug.h>` + `internal/ps_i.h` | `ps_i.h` re-include only |

Rust LOC is higher than original C because (a) bindgen unions need
explicit accessor paths, (b) every output-pointer write goes through
`write_volatile` or an `opaque_ptr` launder, (c) the augment+isolated-
rlib pattern means C trampolines + Rust impls coexist for a shrinking
set of fns. Net work-done metric is closer to 1:1.

**Sibling rlib count: 45.** Up from ~15 a few sessions ago.

Smoke status:
- **i386: 7/7 PASS** (cmake build); cargo-built binary also boots, smoke green.
- **amd64: 7/7 PASS** (cmake build).
- **arm64: 5/5 PASS** (cmake build). One open downstream issue —
  post-boot fastfat BUGCHECK with `STATUS_INSUFFICIENT_RESOURCES` —
  is tracked separately and does not block smoke.

## What's still C

| Bucket | LOC | Notes |
|---|---:|---|
| 9 shim files in `ntos.cmake` | 813 | Detailed table above. |
| `cmdata.c` Phase B holdouts | ~150 | `MmConsumedPoolPercentage` and a handful of inherent registry-structure tables. |

**Translation candidates with no Rust translation in tree: zero.**
Every C file the default i386/amd64 build pulls in either has its Rust
replacement active, or is a shim that augments a Rust impl living in a
sibling rlib.

The historical `cache/` directory (NEWCC alternate cache subsystem) is
gone as of 2026-05-06; the Rust CC subsystem (`ntoskrnl-rust/src/cc/`)
is the only cache implementation now.

## Forward path

The strategic goal motivating shim minimization isn't the LOC count
itself — it's switching ntoskrnl from cmake to cargo as the build
driver. The cargo PoC has landed; the remaining work splits into two
threads.

### 1. Drop `<ntoskrnl.h>` and the heavy internal headers from the remaining 6 shims (next)

This is the prerequisite for "compile shims via cc-rs cleanly." We
explicitly **don't** want to pile cmake-equivalent `cc::Build::include()`
calls into the build.rs — the right move is to make the shims
header-free in the same way `ke_rust_shims.c` was made header-free
(local typedef block + inlined small constants/macros). `cc::Build::
include()` should be a last resort, used only when a header pull is
genuinely small (`<stdarg.h>`, `<stddef.h>`, etc.).

Per-file plan, in rough order of how big the dependency surface is:

- **`io/iomgr/io_rust_shims.c`** (70 LOC, 4 includes). Real deps to
  audit: which Iop* / file-object / IRP types and macros are touched.
  May need a minimal local mirror of a few struct accessors.
- **`mm/mm_rust_shims.c`** (69 LOC, 4 includes). Pulls `miarm.h`
  (Mi macros) and `arch/mm.h`. Heaviest internal-header pull.
- **`kdbg/kdbg_rust_shims.c`** (62 LOC, 3 includes). KDB-specific
  types. Probably tractable.
- **`lto_workaround.c`** (45 LOC, 2 includes). Just pulls
  `<ntoskrnl.h>` for `SepInitTokensShim`'s declared signature. Small.
- **`mm_lock_outlines.c`** (36 LOC, 3 includes). `__FILE__`/`__LINE__`
  capture for MM lock instrumentation. Small surface.
- **`kernel_c_helpers.c`** (33 LOC, 3 includes). The `ps_i.h`
  re-include trick to externalise `PsProcessInfoClass[]` /
  `PsThreadInfoClass[]`. May need to be replaced with a Rust-side
  table generator (similar pattern to `gen_ssdt.py` for SSDT).

Once these 6 are header-free, every shim file can be compiled by
`cc::Build::file()` without any `-I` machinery, and we can move shim
compilation into the cargo build.

### 2. Cargo PoC follow-ups

In rough priority order:

1. **Compile C shims via cc-rs** in `build.rs`. Trivial for the 3 already
   header-free; gated on threads (1) for the other 6.
2. **Resource compilation.** `embed-resource` cargo crate, or shell
   out to `windres` from build.rs.
3. **Replicate per-arch.** Clone `i686-kernel.json` to `amd64-kernel.json`
   and `arm64-kernel.json`; the rest of the PoC structure is arch-agnostic.
4. **Audit `pefixup` / `fix_lld_forwarding.py`.** The smoke-passing
   PoC binary skipped both — strong sign that with the modern
   linker/Rust setup neither is strictly required for boot, but worth
   a careful audit (especially: does pefixup adjust the export table
   in ways that user-mode DLLs depend on?).
5. **Replace cmake-built static libs with cargo-built equivalents.**
   The PoC still links ~30 cmake-produced `.a` files (libcmlib,
   librtl, libhal, etc.). Each is its own subdirectory build; moving
   them to cargo crates would let us drop the cmake step entirely.
   Big lift; not blocking.

### Definition of done

`cargo build -p ntoskrnl --target <arch>` produces a working
`ntoskrnl.exe` for all 3 arches without invoking cmake, all 3 smoke
tests pass on the cargo-driven binary, `tools/scan_ud2.py --check` is
clean, and the cmake path can be removed (or kept as a parallel CI
check).

## Cargo PoC, as it stands

Lives at `sdk/lib/rust/ntoskrnl-cargo/`. Run via:

```bash
RUN_SMOKE=1 ./sdk/lib/rust/ntoskrnl-cargo/build-and-test.sh
```

Components:

- **`Cargo.toml`** — `[[bin]]` named `ntoskrnl`. No Rust deps —
  `kernel-rust` is `crate-type = ["staticlib"]` so build.rs links it
  by absolute path.
- **`i686-kernel.json`** — custom target spec forked from
  `i686-pc-windows-gnu` with the user-mode CRT bits stripped:
  `late-link-args` (no `-lmsvcrt` / `-lgcc_eh` / etc.),
  `pre-link-objects` and `post-link-objects` (no `rsbegin.o`/`rsend.o`),
  `crt-objects-fallback` removed. `rustc-abi` set to `x86-softfloat`
  so build-std's compiler_builtins doesn't insist on SSE.
- **`build.rs`** — invokes `cargo build -p kernel-rust` as a sub-step
  (with the same custom target), then emits `cargo:rustc-link-arg=`
  directives for: the resulting `libkernel_rust.a`, the 18 cmake-
  built `.obj` files (8 asm + 9 C shims + 1 resource), the `.def`
  file, the ~30 cmake-built static libs, the clang builtins archive,
  and the kernel-mode PE-COFF link flags.
- **`src/main.rs`** — `#![no_main]` shell with a stub
  `#[panic_handler]` and a `#[used]` anchor referencing `KiSystemStartup`.
- **`build-and-test.sh`** — wrapper script that refreshes cmake
  inputs, runs the cargo build, and (with `RUN_SMOKE=1`) swaps the
  cargo binary into `build-i386/` + reruns smoke.
- **Nightly toolchain required** for `-Z build-std`. The cmake-driven
  build still uses stable; `build.rs` sets `RUSTUP_TOOLCHAIN=nightly`
  for its sub-cargo invocation only.

Notable: the cargo binary is **smaller** (3.1 MB vs. cmake's 4.6 MB)
because `--gc-sections` runs more aggressively without the cmake-side
intermediate static libs accumulating unreferenced helpers.

## Active in Rust (default build)

Counts are *active modules* in `mod.rs`. As of 2026-05-08, every
Rust file in the tree is wired in — parked translations: **0**.

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
| **TOTAL active (i386/amd64)** | **~325** | **~441,200** |

## Verified must-stay set

These have been re-probed and **proven** to require the full kernel
header chain or a C compile. Not candidates for further elimination
until the underlying constraint moves.

- **Variadic exports** (`KdbpPrint`, `KdpDprintf`, `RtlStringCbPrintfA`,
  KDBG ext printers, `RtlUnicodeStringPrintf_4wZ_shim`,
  `ntprintf_trampoline.c`'s callees). Stable Rust limitation; cargo's
  `#[no_mangle] extern "C-variadic"` is unstable. Already isolated to
  minimal trampolines — fine.
- **`SepInitTokensShim`** (i386, in `lto_workaround.c`). LLVM fat-LTO
  miscompilation: `ud2` inserted after `@ObInitializeFastReference@8`.
  None of `opaque_ptr`, `compiler_fence(SeqCst)`, `core::hint::black_box`
  defeat it. C-side execution outside the Rust LTO scope is the only
  working workaround.
- **`KiSaveXState` / `KiRestoreXState`** (amd64, in `ke_rust_shims.c`).
  KVM bug: `xrstors64` faults on StateSaveArea pages that look valid
  via PTE self-referencing. Force `fxsave64`/`fxrstor64` instead.
- **`MmLockSectionSegment` trio** (`mm_lock_outlines.c`). Uses
  `__FILE__`/`__LINE__` for call-site recording in the underlying
  `_Mm*` primitives.
- **Rust dual-decoration constraint** (i386):
  `Ke386Get{Global,Local}DescriptorTable` has both an `extern "C"`
  cdecl caller (kdbg/kdb_cli.rs) and an `extern "system"` stdcall
  caller (ke/i386/cpu.rs) referencing the same Rust source name.
  Rust forbids two source-level fn definitions of the same name, so
  the cdecl impl lives bare-name in `ke-macros` and the stdcall
  `_X@4` decoration is supplied by a one-line C trampoline calling
  the cdecl `_rs_*` variant.
- **`ps_i.h` re-include trick** in `kernel_c_helpers.c`. The C file
  re-includes `ps_i.h` with a `#pragma push/pop` of `static` and
  `_static_*` rename to externalise `PsProcessInfoClass[]` /
  `PsThreadInfoClass[]`. A parallel Rust translation exists at
  `ps/info_class.rs` (under `_rs_*` symbols), byte-equal to the C
  arrays — but renaming the Rust statics to drop the prefix
  reproducibly bugchecks i386 with `STATUS_DATATYPE_MISALIGNMENT`
  even though the bytes are identical (purely positional, .rdata
  offset-dependent). Not blocking; worth re-investigation.

## Re-probing pattern (carry forward)

Many "Rust crashes / hangs" excuses in shim or DISABLED docs predate
the workspace `lto = false` + `codegen-units = 256` firewall, the
`opaque_ptr` laundering discipline, and `verify_abi.py`. **Re-probe
before treating any classification as a settled answer.**

When picking a parked file, deferred translation, or stay-in-C
classification to revive, the cheapest first step is often: re-enable
it, build, smoke. The bug may already be gone. Recent examples that
fell to a re-probe: every io captures(none) workaround, the trap-exit
chain ("multi-month" estimate that took hours), the `KiEndInterrupt`
"rustc 1.92 ABI bug," the four HAL stdcall bridges, and most recently
the three Phase 1 substantive translations
(`KiAttachProcess`/`KiThreadStartup`/`Ki386InitializeTss`).

The pattern: the heavy lifting (callees in Rust, layout structs already
defined, helpers as private fns next to the consumer) is usually
already done in earlier passes, leaving the actual port as a
one-session task.

## Common bug patterns

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
11. **ud2 + folded success path.** Identical symptom for
    `captures(none)` AND `extern "C"` / `extern "system"` ABI
    mismatch on i386. Check ABI first via `nm` or `verify_abi.py` —
    1-second check vs. minutes of captures(none) defenses.

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
- **Per-arch differentials.** i386 and amd64 are at parity. arm64 is
  green at boot smoke; one open downstream issue (post-boot fastfat
  BUGCHECK).
- **The remaining 6 header-pulling shims.** They compile and link
  fine — the "header debt" is a property of the cargo-driven build
  PoC's prerequisites, not of correctness. See "Forward path" §1.

## How these numbers are computed

```bash
# Total ntoskrnl C body of work (excludes shims):
find ntoskrnl -name "*.c" \
  | grep -v -E '_rust_shims\.c|kernel_c_helpers\.c|variadic_shims\.c|lto_workaround\.c|mm_lock_outlines\.c|ntprintf_trampoline\.c' \
  | xargs wc -l   # → 268,436

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
  | sort -u | xargs wc -l   # → 813

# Rust LOC across ntoskrnl-rust/:
find sdk/lib/rust/ntoskrnl-rust/src -name "*.rs" | xargs wc -l   # → 441,262

# Sibling rlib count:
find sdk/lib/rust -maxdepth 2 -name "Cargo.toml" | wc -l   # → 45

# Parked Rust translations (file on disk, not in mod.rs):
for d in sdk/lib/rust/ntoskrnl-rust/src/*/mod.rs; do
  dir=$(dirname "$d")
  diff <(find "$dir" -maxdepth 1 -name "*.rs" ! -name mod.rs \
         | xargs -n1 basename | sed 's/\.rs$//' | sort -u) \
       <(grep -E '^pub mod ' "$d" | awk '{print $3}' | sed 's/;//' | sort -u) \
    | grep '^<' || true
done   # → empty
```
