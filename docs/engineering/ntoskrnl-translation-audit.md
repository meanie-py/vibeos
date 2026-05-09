# ntoskrnl C→Rust Translation Audit

Last updated 2026-05-08. Seven milestones landed this session:

1. **The header-drop arc completed for the 8 shims.** All 8 active C
   shim files are now header-free (no `<ntoskrnl.h>`, no internal
   kernel headers).
2. **`kernel_c_helpers.c` retired.** Rust now supplies the canonical
   `PsProcessInfoClass` / `PsThreadInfoClass` symbols, replacing the
   `ps_i.h` re-include trick that lived in C since the start of the port.
3. **The cargo-driven `ntoskrnl.exe` build PoC for i386 boots to desktop
   and passes 7/7 smoke.** Cargo can now drive the final link without
   cmake; the remaining 30+ static libs are still cmake-built but
   that's the next thread of work, not a blocker.
4. **The cargo PoC compiles its own C shim layer via `cc-rs`.** The
   8 header-free shim translation units (and `io_tables.c` while it
   still existed; later retired in milestone 7) are now produced by
   cc-rs from `build.rs`, not pulled from cmake's `.obj` directory.
   Boot-equivalent. Pre-existing latent bug surfaced and fixed:
   `ntkrnlmp/CMakeLists.txt` was missing `io_rust_shims.c` and
   `kdbg_rust_shims.c` from `NTKRNLMP_PCH_SKIP_SOURCE` — only
   mattered once livecd actually tried to rebuild ntkrnlmp.
5. **The cargo PoC compiles its own asm layer via `cc-rs`.** All 8
   ntoskrnl asm files (`zw.S`, `fastinterlck_asm.S`, `ioport.S`,
   `ctxswitch.S`, `trap.s`, `usercall_asm.S`, `zeropage.S`,
   `prefetch.S`) compile via `cc-rs` with the same kernel-mode flags
   + DEFINES + asm-include dirs cmake uses. The `.text` and `.data`
   sections of every cc-rs-built `.o` are byte-identical to cmake's;
   only `.debug_info` strings differ (path-dependent). 7/7 smoke pass.
6. **The cargo PoC compiles its own resource (`.rc.res`) via
   `windres`.** The new `compile_resource()` helper in `build.rs`
   shells out to `i686-w64-mingw32-windres` with the same
   preprocessor + 136 `LANGUAGE_*` defines + `KERNEL_DEFINES` +
   include chain cmake uses. Output is **byte-identical** to cmake's
   `.rc.res` (61,116 bytes, `cmp` exit 0). The cargo PoC no longer
   pulls a single `.obj`/`.res` file from cmake's output dir — only
   the `.def` file and the ~30 sibling static libs.
7. **`io_tables.c` retired — last `<ntoskrnl.h>` `#include` in the
   active build is gone.** The 8 `Iop*Operation*` tables are now
   `#[no_mangle] pub static [u8/u32; N]` in
   `iomgr/io_tables.rs`, computed from `core::mem::size_of::<>()`
   against ntkrnl-sys bindgen types and `FILE_READ_*`/`FILE_WRITE_*`
   constants from `ntkrnl_sys::constants`. `verify_sizes.py`'s
   compile-time assertions guarantee the bindgen sizes match the C
   structs byte-for-byte. cmake i386 7/7, amd64 7/7, arm64 5/5,
   cargo PoC 7/7. Cargo PoC binary grew 4 KB (one page) for the new
   `.data` statics.

Treat any prior version of this document as suspect.

## Where we are

**~99.7% of the original ntoskrnl C body of work is now Rust.** The
active C in the default i386/amd64 build is **8 narrow shim files
totalling 1,037 LOC** — and **all 8 compile with only
`<stdarg.h>`/`<stddef.h>`/`<stdint.h>` + a small local typedef
block**, no `<ntoskrnl.h>` chain. **Zero source files in the active
build pull `<ntoskrnl.h>` anymore** as of 2026-05-08.

**Cargo-driven build for i386 works end-to-end.** `cargo +nightly
build -p ntoskrnl-cargo --target ntoskrnl-cargo/i686-kernel.json -Z
build-std=…` produces a 3.1 MB `ntoskrnl.exe` (vs. cmake's 4.5 MB)
that boots in QEMU and passes 7/7 smoke. The PoC reuses cmake's
`.obj` files for now; with the header drop complete, every shim is
ready to be compiled by `cc::Build::file()` from `build.rs` without
any `-I` machinery.

Counts (verified by `wc -l`):

| Bucket | LOC |
|---|---:|
| Original ntoskrnl C body of work (excl. shims) | 268,436 |
| Rust under `sdk/lib/rust/ntoskrnl-rust/src/` | 441,282 |
| Active C in default i386/amd64 build (excl. shims) | 0 |
| C shim layer (8 files in `PCH_SKIP_SOURCE`) | 1,037 |
| C data-table sister files | 0 (was 1; `io_tables.c` retired 2026-05-08) |

Shim layer files (LOC, header status). All shims are header-free on
every architecture they're compiled for:

| File | LOC | Headers | Role |
|---|---:|---|---|
| `ke/ke_rust_shims.c` | 251 | header-free ✓ | minimal trampoline |
| `ntprintf_trampoline.c` | 170 | header-free ✓ | variadic `va_arg` pull |
| `io/iomgr/io_rust_shims.c` | 154 | header-free ✓ | probes + variadic |
| `mm/mm_rust_shims.c` | 152 | header-free ✓ | ARM64-only PTE/IRQL debug |
| `kdbg/kdbg_rust_shims.c` | 91 | header-free ✓ | `KdbpPrint` + `KdbPromptStr` |
| `lto_workaround.c` | 87 | header-free ✓ | LLVM i386 fat-LTO isolation |
| `variadic_shims.c` | 77 | header-free ✓ | `KdpDprintf` + `RtlStringCbPrintfA` |
| `mm_lock_outlines.c` | 55 | header-free ✓ | C `__FILE__`/`__LINE__` capture |

The historical "sister file" pattern (small `.c` that re-includes
`<ntoskrnl.h>` to materialize a C-side data table whose entries
depend on `sizeof(KernelStruct)`) is now empty. `io_tables.c` was
the last instance and got translated to Rust on 2026-05-08
(`iomgr/io_tables.rs`) using `core::mem::size_of::<>()` against
ntkrnl-sys bindgen types — verified equivalent by `verify_sizes.py`.

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
| 8 shim files in `PCH_SKIP_SOURCE` | 1,037 | All header-free. See table above. |
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
driver. With the header-drop arc complete and the C shim compile
moved to cc-rs, the remaining items are: replace the asm + .res +
.def cmake outputs, and replace the ~30 static libs.

### 1. Cargo PoC follow-ups

In rough priority order:

1. **Generate `ks386.inc` ourselves.** cmake builds it via the
   `genincdata` host tool from `sdk/include/asm/genincdata.c`. The
   asm layer is now the only thing in the cargo PoC's compile path
   that pulls `build-i386/sdk/include/asm/` (a cmake-generated
   header). Reproducing that in `build.rs` would let us drop the
   `build-i386/sdk/include/asm` reference and free the cargo PoC of
   the `cmake .` configure step for headers entirely.
2. **Replicate per-arch.** Clone `i686-kernel.json` to `amd64-kernel.json`
   and `arm64-kernel.json`; the rest of the PoC structure is arch-agnostic.
3. **Audit `pefixup` / `fix_lld_forwarding.py`.** The smoke-passing
   PoC binary skipped both — strong sign that with the modern
   linker/Rust setup neither is strictly required for boot, but worth
   a careful audit (especially: does pefixup adjust the export table
   in ways that user-mode DLLs depend on?).
4. **Replace cmake-built static libs with cargo-built equivalents.**
   The PoC still links ~30 cmake-produced `.a` files (libcmlib,
   librtl, libhal, etc.). Each is its own subdirectory build; moving
   them to cargo crates would let us drop the cmake step entirely.
   Big lift; not blocking.

### 2. Re-probe the FIXME(amd64-misalign-231) workaround

`ex/exintrin.rs::ProbeForRead` carries a workaround that treats
non-{1,2,4,8,16} `Alignment` values as 1, citing a "consistent 0xE7
from a 16-byte UNICODE_STRING-style probe post-shell-ready on amd64."
This has the exact same root-cause smell as the just-fixed info-class
COUNT bug — past-end array reads producing junk Alignment values.
Worth re-probing whether the workaround is still load-bearing now
that `PS_PROCESS_INFO_CLASS_COUNT` and `PS_THREAD_INFO_CLASS_COUNT`
match the actual array sizes.

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
  (with the same custom target), compiles the 8 header-free C shim
  translation units and the 8 asm files (zw, fastinterlck_asm,
  ioport, ctxswitch, trap, usercall_asm, zeropage, prefetch) via
  `cc-rs`, builds `ntoskrnl.rc.res` via `windres`
  (`compile_resource()`), and emits `cargo:rustc-link-arg=`
  directives for: the resulting `libkernel_rust.a`, the per-shim/
  per-asm `.o` files, the per-build `.res` resource, the cmake-built
  `.def` file, the ~30 cmake-built static libs, the clang builtins
  archive, and the kernel-mode PE-COFF link flags. The
  `apply_kernel_base()` helper mirrors cmake's `DEFINES = …` and
  `FLAGS = …` lines exactly, `apply_kernel_includes()` mirrors
  `INCLUDES = …`, and the resulting `.text`/`.data` sections of every
  cc-rs-built `.o` (and the entire `.rc.res` blob) match cmake's
  byte-for-byte (only `.debug_info` strings differ in the C/asm
  objects, since they encode `__FILE__` paths).
- **`src/main.rs`** — `#![no_main]` shell with a stub
  `#[panic_handler]` and a `#[used]` anchor referencing `KiSystemStartup`.
- **`build-and-test.sh`** — wrapper script that refreshes cmake
  inputs, runs the cargo build, and (with `RUN_SMOKE=1`) swaps the
  cargo binary into `build-i386/` + reruns smoke.
- **Nightly toolchain required** for `-Z build-std`. The cmake-driven
  build still uses stable; `build.rs` sets `RUSTUP_TOOLCHAIN=nightly`
  for its sub-cargo invocation only.

Notable: the cargo binary is **smaller** (3.1 MB vs. cmake's 4.5 MB)
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
| **TOTAL active (i386/amd64)** | **~325** | **~441,300** |

## Verified must-stay set

These have been re-probed and **proven** to require either C-side
execution or a separate compile unit. Not candidates for further
elimination until the underlying constraint moves.

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
"rustc 1.92 ABI bug," the four HAL stdcall bridges, the three Phase 1
substantive translations
(`KiAttachProcess`/`KiThreadStartup`/`Ki386InitializeTss`), and most
recently the `kernel_c_helpers.c` "alignment regression" — which
turned out not to be a Rust-translation issue at all but a pre-existing
latent count bug in `ps/query.rs` that the C-side trick was
accidentally masking.

The pattern: the heavy lifting (callees in Rust, layout structs already
defined, helpers as private fns next to the consumer) is usually
already done in earlier passes, leaving the actual port as a
one-session task. And: the "Rust regression" framing is sometimes
itself wrong — the C trick may have been hiding an upstream bug that
only surfaces after the swap.

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
   Recent surface: `PS_{PROCESS,THREAD}_INFO_CLASS_COUNT` in
   `ps/query.rs` were transcribed from the Vista enum maxes (51/56)
   instead of NT 5.x values (47/28). The off-by-many indexed past
   the table end, reading garbage `AlignmentQUERY` bytes that tripped
   `ProbeForRead`'s misalign check. Bug was masked by C-side trailing
   bytes happening to be safe; surfaced when swapping the symbol
   supplier to Rust.
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
12. **PCH-skipped `static` re-include trick.** A C file that
    `#define static` + re-includes a header to externalize a `static`
    array (the original `kernel_c_helpers.c` pattern) only works when
    the file is PCH-SKIPPED. PCH-included siblings already pulled the
    header with the `static` qualifier, so the re-include creates a
    `static` vs non-static redefinition. Both former users of this
    pattern (`kernel_c_helpers.c` and `io_tables.c`) are now Rust;
    the pattern is gone from the active build. If a future shim
    needs it, either keep the file PCH-skipped (with `<ntoskrnl.h>`
    retained) or translate to Rust.

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

## How these numbers are computed

```bash
# Total ntoskrnl C body of work (excludes shims):
find ntoskrnl -name "*.c" \
  | grep -v -E '_rust_shims\.c|variadic_shims\.c|lto_workaround\.c|mm_lock_outlines\.c|ntprintf_trampoline\.c' \
  | xargs wc -l   # → 268,436

# Active C in default build (excludes shims):
grep -E '^\s+\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/.*\.c' ntoskrnl/ntos.cmake \
  | grep -v '^\s*#' \
  | grep -v -E '_rust_shims\.c|variadic_shims\.c|lto_workaround\.c|mm_lock_outlines\.c|ntprintf_trampoline\.c' \
  | sed 's|^\s*\${REACTOS_SOURCE_DIR}/||; s|).*||; s| .*||' \
  | sort -u | xargs wc -l   # → 0

# Shim layer (8 files, all PCH-skipped):
wc -l ntoskrnl/ke/ke_rust_shims.c \
      ntoskrnl/ntprintf_trampoline.c \
      ntoskrnl/io/iomgr/io_rust_shims.c \
      ntoskrnl/mm/mm_rust_shims.c \
      ntoskrnl/kdbg/kdbg_rust_shims.c \
      ntoskrnl/lto_workaround.c \
      ntoskrnl/variadic_shims.c \
      ntoskrnl/mm_lock_outlines.c   # → 1,037

# Rust LOC across ntoskrnl-rust/:
find sdk/lib/rust/ntoskrnl-rust/src -name "*.rs" | xargs wc -l   # → 441,282

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
