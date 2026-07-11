# Shim Elimination — Status and Forward Path

> **ARCHIVED 2026-07-10** — the shim-elimination + cargo-driven-build arc is DONE (2026-05-09, post-milestone-16 — all 8 C shims header-free, KERNEL_INCLUDES deleted, zero build-dir header deps). The living as-built is `rust-in-kernel.md` + `build-system.md`.


## TL;DR (2026-05-09, post-milestone-16)

The shim-elimination arc and the cargo-driven build arc for the
**ntoskrnl source body** are **DONE**:

1. **All 8 C shims are header-free** (only `<stdarg.h>`/`<stddef.h>`/`<stdint.h>`).
2. **`KERNEL_INCLUDES` is deleted entirely** — every cargo-build
   compile input is self-contained or staged from a checked-in
   artifact. cargo PoC's only `-I` chain is `-isystem sdk/include/crt`
   (mingw user-mode CRT replacement) + clang's resource dir.
3. **Zero `build-i386/*` deps** for headers, objects, .res, or .def —
   every artifact comes from one of:
   - 3 in-process generators (`hpp`, `windmc`, `spec2def` — host C
     tools build.rs builds via `cc::Build::host()`)
   - 3 out-of-band Python scripts (`tools/gen_ks386_inc.py`,
     `tools/gen_ntoskrnl_res.py`, `tools/gen_preprocessed_asm.py`)
     producing checked-in artifacts under
     `sdk/lib/rust/ntoskrnl-cargo/data/`
   - cc-rs compiling 8 header-free C shims + 6 pre-processed
     `.s` files

The remaining cmake outputs the cargo PoC consumes are the ~30
**cmake-built static libs** (`librtl.a`, `libhal.a`, `libcmlib.a`,
…). That's its own translation arc — see
[`static-libs-translation-audit.md`](archive/static-libs-translation-audit.md).

## The cargo PoC's compile inputs (2026-05-09)

- **8 C shims** (`ntprintf_trampoline.c`, `variadic_shims.c`,
  `mm_lock_outlines.c`, `kdbg_rust_shims.c`, `io_rust_shims.c`,
  `mm_rust_shims.c`, `lto_workaround.c`, `ke_rust_shims.c`) —
  all header-free, only need `<stdarg.h>` / `<stddef.h>` /
  `<stdint.h>` + a small local NT typedef block. Compile via
  `cc-rs` from `build.rs` with no kernel `-I` chain.
- **6 pre-processed asm files** at `data/asm/*-i386.s` —
  fully self-contained (every `#include` expanded by
  `tools/gen_preprocessed_asm.py`); `compile_asm()` invokes cc-rs
  with NO kernel `-I` chain. The 2 smallest of the original 8
  (`zeropage.S`, `prefetch.S`) became `core::arch::global_asm!`
  blocks in `ntoskrnl-rust/src/{ke/i386/zeropage_asm.rs,rtl/prefetch_asm.rs}`
  (milestone 14).
- **`ntoskrnl.rc`** — checked in to `data/ntoskrnl-i386.rc.res`,
  regen out-of-band via `python3 tools/gen_ntoskrnl_res.py`. Build-
  time `build.rs` just stages the file. **`compile_resource()` and
  the 136-entry `LANGUAGES` const are no longer in the cargo PoC.**
- **`ks386.inc`** — checked in to `sdk/lib/rust/ntoskrnl-cargo/data/`.
  Regen out-of-band via `python3 tools/gen_ks386_inc.py` (compiles
  `genincdata.c` once, walks `.asmdef` COFF section, emits
  byte-identical to cmake's). Build-time `build.rs` just stages it
  into `OUT_DIR`. **`genincdata.c` is no longer compiled by the
  cargo PoC at all** — that was the deepest puller of the kernel
  `-I` chain.
- **`hpp` host tool** — built from source by `build.rs` via
  `cc::Build::host()`. Drives the 4 xdk header generations.

Every shim is `cc-rs`-compileable with no kernel `-I` (the file's
own local typedefs cover the symbol surface).

## The 8 + 1 retired

| File | Status |
|------|--------|
| `ntprintf_trampoline.c` (170 LOC) | header-free since the earliest pass (gold-standard variadic adapter) |
| `variadic_shims.c` (77) | header-free since 2026-05-07 |
| `mm_lock_outlines.c` (55) | header-free 2026-05-08 |
| `kdbg_rust_shims.c` (91) | header-free 2026-05-08 |
| `io_rust_shims.c` (154) | header-free 2026-05-08 |
| `mm_rust_shims.c` (152) | header-free 2026-05-08 (ARM64-only body) |
| `lto_workaround.c` (87) | header-free 2026-05-08 |
| `ke_rust_shims.c` (251) | header-free 2026-05-07 |
| ~~`kernel_c_helpers.c`~~ | **RETIRED 2026-05-08** — Rust `iomgr/io_tables.rs` and `ps/info_class.rs` |
| ~~`io_tables.c`~~ | **RETIRED 2026-05-08** — Rust `iomgr/io_tables.rs` |
| ~~`arm64_uart.c`~~ | retired 2026-05-07 (Rust UART traces) |

The "sister-file" pattern (small `.c` that pulls `<ntoskrnl.h>` to
host data tables) is **empty** as of 2026-05-08. Both former users
(`kernel_c_helpers.c` and `io_tables.c`) translated to Rust.

## Cargo-driven build status (2026-05-08)

Lives at `sdk/lib/rust/ntoskrnl-cargo/`. i386 boots to desktop and
passes 7/7 smoke. Architecture replication (amd64 + arm64) is the
remaining "Definition of done" gap.

What `build.rs` owns end-to-end:

| Step | Helper | Output | Byte-identical to cmake? |
|------|--------|--------|:--:|
| Build host hpp | `compile_hpp_host()` | `OUT_DIR/host-tools/hpp` | (cmake builds the same `.c`) |
| Generate xdk kernel headers | `generate_xdk_headers()` | `OUT_DIR/xdk-gen/{ddk,psdk}/*.h` | ✓ all 4 |
| Convert + run windmc | `generate_bugcodes_mc()` | `OUT_DIR/mc-gen/bugcodes.{h,rc,bin}` | ✓ all 3 |
| Stage checked-in `ks386.inc` | `stage_ks386_inc()` | `OUT_DIR/asm-gen/ks386.inc` | ✓ (regen via `tools/gen_ks386_inc.py`) |
| cc-rs C shim compile | `compile_shim()` (×8) | `OUT_DIR/shims/*.o` | ✓ `.text`/`.data` |
| cc-rs asm compile | `compile_asm()` (×6) | `OUT_DIR/shims/*.o` | ✓ `.text`/`.data` |
| Stage checked-in `.rc.res` | `stage_ntoskrnl_res()` | `OUT_DIR/shims/ntoskrnl.rc.res` | ✓ (regen via `tools/gen_ntoskrnl_res.py`) |
| Build host spec2def | `compile_spec2def_host()` | `OUT_DIR/host-tools/spec2def` | (cmake builds the same `.c`) |
| Run spec2def on `.spec` | `generate_def_file()` | `OUT_DIR/spec-gen/ntoskrnl.def` | ✓ |
| Final link | rustc | `target/.../ntoskrnl.exe` (3.1 MB) | smaller than cmake's 4.5 MB; smoke green |

What still comes from cmake (not in scope for this arc):

- The ~30 sibling static libs (`librtl.a`, `libhal.a`, `libcmlib.a`,
  `libreactos_*.a`, etc.). Each is its own subdirectory build.

## What still pulls the kernel `-I` chain

**Nothing.** `KERNEL_INCLUDES` was deleted in milestone 16. The cargo
PoC's only `-I` chain is `-isystem sdk/include/crt` (mingw user-mode
CRT replacement) + clang's resource dir, applied directly inside each
cc-rs caller.

Cmake originally needed 25 source-tree paths. Removal history:
- ~~10 dead-weight build-i386/* paths~~ (milestone 10 survey)
- ~~`build-i386/sdk/include/{ddk,psdk}`~~ (milestone 11: hpp self-host)
- ~~`build-i386/sdk/include/reactos/mc`~~ (milestone 9: windmc in-process)
- ~~`sdk/include`, `sdk/include/ddk`, `sdk/include/ndk`, `sdk/include/reactos`, `sdk/lib/pseh/include`~~ (milestone 13: genincdata out-of-band)
- ~~`ntoskrnl`, `sdk/include/psdk`, `sdk/include/vcruntime`~~ (milestone 15: RC out-of-band)
- ~~`ntoskrnl/include`, `sdk/include/asm`~~ (milestone 16: 6 asm files pre-processed out-of-band)

Net: every source-tree kernel header has been eliminated from the
cargo PoC's compile surface.

## Forward path: out of scope for this doc

Every source-tree kernel header is gone from the cargo PoC's compile
surface (milestones 9-16). What remains:

- **The ~30 cmake-built static libs** (`librtl.a`, `libhal.a`,
  `libcmlib.a`, …) — collectively ~165K LOC of C/asm. Tracked in
  [`static-libs-translation-audit.md`](archive/static-libs-translation-audit.md).
  The 17 already-Rust sibling rlibs in that list are mechanical
  build-system work; the 18 cmake-built C/asm libs are real
  translation arcs.
- **Per-arch replication** of the i386 cargo PoC for amd64 + arm64.
  The structure is arch-agnostic; mostly cloning
  `i686-kernel.json` and verifying the `.s` regen produces
  per-arch artifacts.
- **`pefixup` / `fix_lld_forwarding.py` audit** — the cargo PoC
  skips both and smoke passes; worth a careful audit before
  declaring the cmake path retirable.

**Out-of-band scripts to re-run when source changes:**
- `python3 tools/gen_ks386_inc.py` — regen `data/ks386-i386.inc`
  when kernel struct layouts or `ks{386,x}.template.h` change
- `python3 tools/gen_ntoskrnl_res.py` — regen
  `data/ntoskrnl-i386.rc.res` when version blocks or bootanim
  resources change
- `python3 tools/gen_preprocessed_asm.py` — regen `data/asm/*.s`
  when an asm source or its includes (`asm.inc`, `ks386.inc`,
  `asmmacro.S`, `syscalls.inc`, `sysfuncs.h`) change

Each script supports `--check` to verify the checked-in artifact
matches a fresh regen (useful as a CI gate).

**Possible future "fully-pure-Rust" refinements** (not blocking):
- Translate the `data/asm/*.s` pre-processed blobs to inline
  `core::arch::global_asm!(include_str!())` blocks (requires
  resolving GAS macro-redefinition conflicts in the integrated
  assembler, OR hand-translating asm.inc + asmmacro.S as Rust
  `macro_rules!`).
- Translate `ntoskrnl.rc`'s version blocks to `#[link_section]`
  Rust statics + `include_bytes!` the bootanim BMPs (eliminates
  cmake-built `bugcodes.rc` dep at script-runtime).
- Translate `genincdata.c` to `core::mem::offset_of!` against
  `ntkrnl-sys` bindgen types directly (eliminates cmake-built
  xdk-gen dep at script-runtime).

## What still has to stay C — verified must-stay set

These have been re-probed and proven to require C-side execution
or a separate compile unit. Re-probe before assuming any of them
have moved.

- **Variadic exports** (`KdbpPrint`, `KdpDprintf`, `RtlStringCbPrintfA`,
  KDBG ext printers, `RtlUnicodeStringPrintf_4wZ_shim`,
  `ntprintf_trampoline.c`'s callees). Stable Rust limitation;
  `#[no_mangle] extern "C-variadic"` is unstable. Already isolated
  to minimal trampolines (compiled by cc-rs) — fine.
- **`SepInitTokensShim`** (i386, in `lto_workaround.c`). LLVM fat-LTO
  miscompilation: `ud2` inserted after `@ObInitializeFastReference@8`.
  Stays C until either LLVM fixes the bug or we factor the call into
  an isolated cc-rs static lib (essentially what we do today).
- **`KiSaveXState` / `KiRestoreXState`** (amd64, in `ke_rust_shims.c`).
  KVM bug: `xrstors64` faults on StateSaveArea pages that look valid
  via PTE self-referencing. C trampoline forces `fxsave64`/`fxrstor64`.
- **`MmLockSectionSegment` trio** (`mm_lock_outlines.c`). Uses
  `__FILE__`/`__LINE__` for call-site recording. Rust's
  `core::file!()`/`line!()` would produce Rust source paths.
- **3 i386 trap-exit `_saved_*` outlines** (`KiExitV86Trap`,
  `KiExitTrapDebugChecks`, `KiExitSystemCallDebugChecks`). PSEH-
  intertwined with the trap-entry chain.
- **rustc 1.92 stdcall/fastcall ABI bug**. Affects `KiEndInterrupt`
  translation. Workaround: keep C trampoline OR force calling
  convention via inline asm.

## Patterns and tactics that worked

These came up repeatedly during the header-drop arc. Useful for
new C surface that creeps in (or the asm-→-Rust arc above).

### Local NT typedef block

Replaces `<ntoskrnl.h>` for files that just need primitive NT types.

```c
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

typedef void              VOID;
typedef void             *PVOID;
typedef long              LONG;
typedef unsigned long     ULONG;
typedef unsigned char     UCHAR;
typedef UCHAR             BOOLEAN;
typedef UCHAR             KIRQL;
typedef long              NTSTATUS;
typedef uintptr_t         ULONG_PTR;  /* uintptr_t — NOT uint64_t */

#define IN
#if defined(_M_IX86)
# define NTAPI    __attribute__((stdcall))
# define FASTCALL __attribute__((fastcall))
#else
# define NTAPI
# define FASTCALL
#endif

#define STATUS_SUCCESS              ((NTSTATUS)0x00000000L)
#define STATUS_INVALID_PARAMETER    ((NTSTATUS)0xC000000DL)
```

`ULONG_PTR = uintptr_t` (NOT `uint64_t`) — width matters on i386,
where the wrong width changes stdcall decoration.

### Rust helper for layout-dependent C accesses

Where a shim needs the address of an EPROCESS / KTHREAD field,
add a tiny Rust helper that returns the raw pointer. Example:
`_rs_PsCurrentProcessTokenAddr()` returns `&CurrentProcess->Token`
as `*mut PVOID`, keeping the EPROCESS layout entirely in Rust.

### `core::mem::size_of::<>()` for sizeof tables (replaces sister files)

When data tables need `sizeof(KernelStruct)` evaluation, the
historical pattern was a sister `.c` file that re-included
`<ntoskrnl.h>` (e.g. `io_tables.c`). The Rust replacement uses
`core::mem::size_of::<_FILE_FOO>()` against `ntkrnl-sys` bindgen
types — equivalence guaranteed by `verify_sizes.py`'s compile-
time assertions. See `iomgr/io_tables.rs` for the canonical
example.

### Failed pattern: PCH-included `static` re-include

The `#define static` + re-include trick (used historically by
`kernel_c_helpers.c`) **only works when the file is PCH-skipped**.
PCH-included siblings have already pulled the header with the
`static` qualifier, causing static-vs-non-static redefinition
errors. Both former users of this pattern are now Rust; if a
future shim needs it, either keep PCH-skipped (with
`<ntoskrnl.h>` retained) or translate to Rust.

## Verification tools

| Tool | What it checks | Location |
|------|---------------|----------|
| `verify_abi.py` | extern "C" vs extern "system" mismatch | `tools/rustconv/rustconv/verify_abi.py` |
| `verify_ffi.py` | ABI + params + C macros + flexible arrays | `tools/rustconv/rustconv/verify_ffi.py` |
| `verify_constants.py` | Constant values vs C headers | `tools/rustconv/rustconv/verify_constants.py` |
| `lint_kernel.py` | Unprotected output pointer writes | `tools/lint_kernel.py` |
| `scan_ud2.py` | LLVM miscompilation (ud2 in named functions) | `tools/scan_ud2.py` |

`nm` decoration check (1 second, faster than any of the above):

```
i686-w64-mingw32-nm build-i386/ntoskrnl/ntoskrnl.exe | grep <FunctionName>
```

`_X` = cdecl, `_X@N` = stdcall, `@X@N` = fastcall.

## Lessons from the header-drop arc

These keep recurring. Track them.

- **Captures(none) ghosts.** Many shim comments cite "LLVM cross-CGU
  captures(none) attribute propagation" as the reason a function
  must stay in C. The workspace `lto = false` + `codegen-units =
  256` firewall has been the default for months — most of these
  comments are stale. Re-probe.
- **ABI mismatch looks like captures(none).** ud2 + folded success
  path on i386 has the same disassembly pattern for both. Check
  ABI first via `nm` (1 second) before chasing captures(none)
  (minutes of failed write_volatile / opaque_ptr defenses).
- **Shims hide latent bugs.** The shim wraps the symptom; the
  underlying bug only surfaces when the call goes direct.
  `IopReadyDeviceObjects` had an `extern "C"`/`extern "system"`
  ABI mismatch in Rust that linked silently for months until
  `IopInitializeDriverModule` translation removed the C trampoline.
  More recently: the `kernel_c_helpers.c` "alignment regression"
  was a pre-existing `PS_*_INFO_CLASS_COUNT` off-by-many in
  `ps/query.rs` that the C `ps_i.h` re-include happened to mask
  via favorable trailing `.rdata` bytes.
- **Empty extern decls don't preserve the symbol.** The historical
  `extern "C" { static mut FOO: [T; 0]; }` pattern (kdbg
  ComponentTable / KdbDebuggerCommands) makes the optimizer fold
  all indexing to panics. Properly size the array in Rust.
- **i386 has 3 decorations, not 1.** Before deleting a "dead"
  shim, grep for `_X` (cdecl), `_X@N` (stdcall), AND `@X@N`
  (fastcall). Different Rust callers use different `extern "..."` ABIs.
- **`global_asm!` symbols need force-pulling.** Adding
  `core::arch::global_asm!(...)` in an rlib doesn't put the
  symbols in `libkernel_rust.a` unless something references them.
  The `_force_<crate>_symbols` pattern in `kernel-rust/src/lib.rs`
  keeps them alive. (Relevant for the asm-to-Rust arc above.)
- **PCH-skipped vs PCH-included is load-bearing.** A C file that
  uses the `#define static` re-include trick to externalise a
  `static` array ONLY works when the file is PCH-skipped.
- **`uintptr_t` not `uint64_t` for `ULONG_PTR`.** Width matters on
  i386 — getting it wrong changes stdcall decoration.
- **The "Rust translation regression" framing is sometimes wrong.**
  When a Rust translation that's "byte-identical" to a C
  predecessor crashes after a symbol-supplier swap, look upstream
  before blaming the translation. The C trick may have been
  accidentally masking a latent caller-side bug.
- **PCH_SKIP_SOURCE for ntkrnlmp is a separate list.** Adding a
  shim to `ntoskrnl/CMakeLists.txt` PCH_SKIP_SOURCE is not enough;
  `ntoskrnl/ntkrnlmp/CMakeLists.txt` has its own
  `NTKRNLMP_PCH_SKIP_SOURCE` that needs the same entries. Was a
  silent latent bug for io_rust_shims.c + kdbg_rust_shims.c — only
  surfaced when livecd actually rebuilt ntkrnlmp.

## Recipe for re-probing a shim

1. Read the in-source comment for the original symptom.
2. Grep for Rust callers across all 3 i386 decorations (`_X`,
   `_X@N`, `@X@N`).
3. If used: drop the C trampoline, declare the underlying Rust fn
   directly with the right ABI. Verify decoration via `nm`.
4. Build all 3 arches. `python3 tools/scan_ud2.py --check
   build-i386` is the canary.
5. Run smoke. Green on i386 + amd64 = ghost. Failure = either a
   real bug surfaced (ABI mismatch, missing volatile, etc.) or a
   genuine workaround that has to stay.

## Recipe for adding a fresh `cc-rs`-compileable shim

The pattern is settled — for a new shim that creeps in:

1. **Audit the symbol surface.** What types, functions, and
   globals does the file actually use? Most files use a tiny
   fraction of `<ntoskrnl.h>`.
2. **Add to `PCH_SKIP_SOURCE`** in BOTH `ntoskrnl/CMakeLists.txt`
   AND `ntoskrnl/ntkrnlmp/CMakeLists.txt`. The latter is easy to
   miss; was a real silent bug for io/kdbg shims.
3. **Replace `<ntoskrnl.h>` with local typedefs** (see "Local NT
   typedef block" above).
4. **Replace function-pulling macros with `extern` decls.**
   Verify decoration via `nm`.
5. **Replace `&Process->Field` with a `_rs_*` Rust helper.**
6. **Watch for "secret" data dependencies.** A `#include
   "internal/foo_i.h"` may be silently providing data tables —
   translate those to Rust statics next to their consumer (see
   `iomgr/io_tables.rs`).
7. **Add to the `header_free_shims` list in
   `sdk/lib/rust/ntoskrnl-cargo/build.rs`** — that's what makes
   cc-rs pick it up.
8. **Build all 3 arches + run all 3 smokes.** A change here can
   blow up just one arch (i386 ABI decoration mismatches that
   amd64 tolerates).
