# Shim Elimination — Status and Forward Path

## TL;DR (2026-05-09)

The shim-elimination arc and the cargo-driven build arc are **both
substantially done**. The cargo PoC has **zero `build-i386/*`
build-time dependencies** for headers/objects/.res/.def — every
shim, every asm file, the resource, every generated header, AND
the `.def` file (milestone 12, 2026-05-08) is produced in-process
from `build.rs`. The only remaining cmake outputs the cargo PoC
consumes are the ~30 sibling static libs (`librtl.a`, `libhal.a`,
etc.). Those are a separate arc.

The source-tree kernel includes are now down to **2 paths** (from
12 → 5 → 2 across three rounds on 2026-05-09):
- `ntoskrnl/include` — pulled only by 6 remaining asm files
- `sdk/include/crt` — universal `-isystem` mingw CRT replacement

`KERNEL_INCLUDES` was 25 entries when this arc started; survey-prune,
genincdata out-of-band, RC out-of-band, and 2 asm-→-`global_asm!`
translations took it to 2. The remaining gap (the 6 still-cmake-built
asm files) is covered by Forward Path § 1 — translate the rest of
the asm to `global_asm!` and `compile_asm()` goes away entirely.

## The cargo PoC's compile inputs (2026-05-08)

- **9 C shims** (`ntprintf_trampoline.c`, `variadic_shims.c`,
  `mm_lock_outlines.c`, `kdbg_rust_shims.c`, `io_rust_shims.c`,
  `mm_rust_shims.c`, `lto_workaround.c`, `ke_rust_shims.c`) —
  all header-free, only need `<stdarg.h>` / `<stddef.h>` /
  `<stdint.h>` + a small local NT typedef block. Compile via
  `cc-rs` from `build.rs` with no `-I` chain.
- **6 asm files** (`zw.S`, `fastinterlck_asm.S`, `ioport.S`,
  `ctxswitch.S`, `trap.s`, `usercall_asm.S`) — `#include
  <ks386.inc>` (staged from `data/ks386-i386.inc`) + `<asm.inc>`
  + `<internal/i386/asmmacro.S>` from the source tree. Compile
  via `cc-rs` with `-x assembler-with-cpp`. The 2 smallest of
  the original 8 (`zeropage.S`, `prefetch.S`) were translated to
  `core::arch::global_asm!` in milestone 14 (see
  `ntoskrnl-rust/src/{ke/i386/zeropage_asm.rs,rtl/prefetch_asm.rs}`).
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

The cargo PoC's `KERNEL_INCLUDES` const is now **2 source-tree
paths** (down from 25 → 12 → 5 → 2 across four surveys/refactors).

| Path | Pulled by | What for |
|------|-----------|----------|
| `ntoskrnl/include` | asm | `<internal/i386/asmmacro.S>`, `<sysfuncs.h>` (6 still-cmake-built asm files) |
| `sdk/include/crt` | all | `-isystem` of mingw user-mode CRT replacement |
| `sdk/include/asm` | asm | `<asm.inc>`, `<syscalls.inc>` (added per-asm-source via `compile_asm()`) |

Plus the in-process gen dirs (`xdk-gen/{ddk,psdk}`, `mc-gen`,
`asm-gen`, `spec-gen`).

8 paths used to be in this list and were dropped on 2026-05-09:
- ~~`sdk/include`~~, ~~`sdk/include/ddk`~~, ~~`sdk/include/ndk`~~,
  ~~`sdk/include/reactos`~~, ~~`sdk/lib/pseh/include`~~ (only
  `genincdata.c` pulled them; gone with milestone 13)
- ~~`ntoskrnl`~~, ~~`sdk/include/psdk`~~, ~~`sdk/include/vcruntime`~~
  (only RC pulled them; gone with milestone 15)

The remaining 2 source-tree kernel paths are pulled by the asm
files only. Translating the 6 remaining asm files to `global_asm!`
(see Forward Path § 1) eliminates `compile_asm()` entirely and drops
both `ntoskrnl/include` and `sdk/include/asm`.

## Forward path: eliminate the source-tree kernel includes

These are the next layers of "what's still C and asm." Each is a
real translation arc, not a build-system tweak.

### 1. Translate the remaining 6 asm files to Rust `global_asm!`

2 of 8 done (milestone 14: `zeropage.S`, `prefetch.S`). 6 remaining:
`zw.S` (30 LOC), `fastinterlck_asm.S` (124), `ioport.S` (210),
`ctxswitch.S` (244), `usercall_asm.S` (202), `trap.s` (257) — total
~1,070 LOC. All `#include <ks386.inc>` for kernel struct offsets
and most pull `<internal/i386/asmmacro.S>` for entry/exit macros.

Translating to `core::arch::global_asm!` is mechanical but
substantial:
- Each `#include <ks386.inc>` becomes inline `.equ` definitions
  emitted by Rust (probably const-evaluated from `ntkrnl-sys`
  bindgen offsets — same trick `io_tables.rs` uses).
- The asmmacro.S entry/exit chains are 200+ lines of complex
  preprocessor macros. They'd become Rust `macro_rules!` that
  expand to `global_asm!` literals.
- Some files (e.g. `trap.s`) interact tightly with `_SEH2_TRY`
  preamble fragments; those would need a Rust analog.

Once done, this drops 6 of the 11 source-tree paths
(`ntoskrnl/include`, `sdk/include/asm`, plus `sdk/include/ndk` if
the templates can be expressed without the full NDK pull) and
removes the cargo PoC's `compile_asm()` call entirely.

Smaller files first (`zeropage.S` is 14 bytes of `.text`,
`prefetch.S` is 16); `trap.s` last (the trap dispatch is the
deepest tie-in to the kernel layout).

### 2. ~~Translate `ntoskrnl.rc` to embedded Rust resources~~ (DONE out-of-band 2026-05-09)

`ntoskrnl.rc` is no longer compiled by the cargo PoC at all. Regen
moved out-of-band to `tools/gen_ntoskrnl_res.py`; the resulting
`data/ntoskrnl-i386.rc.res` (61,116 bytes, byte-identical to
cmake's) is checked in. Build-time `build.rs` just stages the
file. This dropped 3 source-tree paths from `KERNEL_INCLUDES`
(`ntoskrnl` root, `sdk/include/psdk`, `sdk/include/vcruntime`).

Re-run `python3 tools/gen_ntoskrnl_res.py` (or `--check`) when
version blocks (`common.ver`, `ntverp.h`) or bootanim resources
change. The script needs cmake-built `bugcodes.rc` from a prior
`ninja ntoskrnl`.

A future "fully-pure-Rust" version would translate the version
blocks to `#[link_section]` statics and `include_bytes!` the
bootanim BMPs, eliminating the cmake-build dep at script-runtime.
Only matters if version info or bootanim change often (they don't).

### 3. ~~Replace `genincdata.c` with a pure-Rust constant emitter~~ (PARTIAL — done out-of-band 2026-05-09)

`genincdata.c` is no longer compiled by the cargo PoC at all.
Regen moved out-of-band to `tools/gen_ks386_inc.py`; the resulting
`data/ks386-i386.inc` is checked in. Build-time `build.rs` just
stages the file. This dropped 5 source-tree paths from
`KERNEL_INCLUDES` (the deepest puller: `<ntndk.h>`,
`<arc/arc.h>`, the full ddk/ndk/pseh tree).

Re-run `python3 tools/gen_ks386_inc.py` (or `--check` to verify)
when kernel struct layouts or the templates change. The script
needs `build-i386/sdk/include/{ddk,psdk}` + `bugcodes.h` from a
prior `ninja ntoskrnl`.

A future "fully-pure-Rust" version of this would inline kernel-
struct offsets via `core::mem::offset_of!` against `ntkrnl-sys`
bindgen types directly (eliminating the cmake-build dependency
even at script-runtime). That requires either a host-buildable
view of `ntkrnl-sys` or const-evaluable struct introspection —
and only matters if the templates change often (they don't).
For now, the script is run on demand.

### What's left after items 1, 2, and the residual of 3

If items 1+2 land, the cargo PoC's `KERNEL_INCLUDES` would shrink
from 5 source-tree paths to 1 (`sdk/include/crt` — universal
mingw user-mode CRT replacement, comes for free with `-nostdinc`).
The PoC would compile and link the kernel with **only the source
tree under `sdk/lib/rust/`** plus the CRT shim — no
`sdk/include/{psdk,vcruntime}`, no `ntoskrnl/include/...`. Every
kernel struct offset would route through `ntkrnl-sys` bindgen.

That's the absolute minimum the C codebase can reach without
removing the C / asm / RC consumers entirely (which is a separate,
even bigger, "translate the kernel ENTIRELY to Rust" arc).

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
