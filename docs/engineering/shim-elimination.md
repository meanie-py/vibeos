# Shim Elimination — Status and Forward Path

## TL;DR (2026-05-08)

The shim-elimination arc and the cargo-driven build arc are **both
substantially done**. The cargo PoC has **zero `build-i386/*`
build-time dependencies** for headers/objects/.res/.def — every
shim, every asm file, the resource, and every generated header is
produced in-process from `build.rs`. The only remaining cmake
outputs the cargo PoC consumes are the ~30 sibling static libs
(`librtl.a`, `libhal.a`, etc.). Those are a separate arc.

The next axis of "elimination" is the **source-tree kernel
includes** — the 12 `KERNEL_INCLUDES` paths that survived the
build-i386 prune. Those are pulled by the asm files, the RC file,
and `genincdata.c`. Eliminating each requires translating its
consumer, not editing the include list.

## The cargo PoC's compile inputs (2026-05-08)

- **9 C shims** (`ntprintf_trampoline.c`, `variadic_shims.c`,
  `mm_lock_outlines.c`, `kdbg_rust_shims.c`, `io_rust_shims.c`,
  `mm_rust_shims.c`, `lto_workaround.c`, `ke_rust_shims.c`) —
  all header-free, only need `<stdarg.h>` / `<stddef.h>` /
  `<stdint.h>` + a small local NT typedef block. Compile via
  `cc-rs` from `build.rs` with no `-I` chain.
- **8 asm files** (`zw.S`, `fastinterlck_asm.S`, `ioport.S`,
  `ctxswitch.S`, `trap.s`, `usercall_asm.S`, `zeropage.S`,
  `prefetch.S`) — `#include <ks386.inc>` (in-process generated)
  + `<asm.inc>` + `<internal/i386/asmmacro.S>` from the source
  tree. Compile via `cc-rs` with `-x assembler-with-cpp`.
- **`ntoskrnl.rc`** — `#include <bugcodes.rc>` (in-process generated)
  + the `inbv/bootanim.rc` chain + version stuff. Compiled via
  `windres` from `build.rs`.
- **`genincdata.c`** — build-time tool that materializes asm
  constants. Compiled via cc-rs to a `.obj`; we walk its `.asmdef`
  COFF section in-process to emit `ks386.inc`.
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
| cc-rs `genincdata.c` + COFF parse | `generate_ks386_inc()` | `OUT_DIR/asm-gen/ks386.inc` | ✓ |
| cc-rs C shim compile | `compile_shim()` (×8) | `OUT_DIR/shims/*.o` | ✓ `.text`/`.data` |
| cc-rs asm compile | `compile_asm()` (×8) | `OUT_DIR/shims/*.o` | ✓ `.text`/`.data` |
| windres on `ntoskrnl.rc` | `compile_resource()` | `OUT_DIR/shims/ntoskrnl.rc.res` | ✓ |
| Final link | rustc | `target/.../ntoskrnl.exe` (3.1 MB) | smaller than cmake's 4.5 MB; smoke green |

What still comes from cmake (not in scope for this arc):

- The ~30 sibling static libs (`librtl.a`, `libhal.a`, `libcmlib.a`,
  `libreactos_*.a`, etc.). Each is its own subdirectory build.
- The `.def` file (cmake runs `spec2def.py` on `ntoskrnl.spec`).
  Trivial Python shell-out from build.rs would self-host this.

## What still pulls the kernel `-I` chain

The cargo PoC's `KERNEL_INCLUDES` const is now 12 source-tree paths
(down from 25 — survey 2026-05-08 dropped 13 dead-weight entries).
Cross-referenced via `clang -H` against the live compile inputs:

| Path | Pulled by | What for |
|------|-----------|----------|
| `ntoskrnl` | RC | `#include "inbv/bootanim.rc"` |
| `ntoskrnl/include` | asm | `<internal/i386/asmmacro.S>` chain |
| `sdk/include` | RC, genincdata | `<psdk/ntverp.h>` etc. |
| `sdk/include/crt` | all | `-isystem` of mingw user-mode CRT replacement |
| `sdk/include/ddk` | genincdata | source-tree ddk fragments (alongside hpp-generated headers) |
| `sdk/include/ndk` | genincdata | `<ntndk.h>` + arch-specific NDK |
| `sdk/include/psdk` | RC, genincdata | source-tree psdk (`winnt.h`, `windbgkd.h`) |
| `sdk/include/reactos` | genincdata | `<arc/arc.h>` chain |
| `sdk/include/vcruntime` | RC, genincdata | `<excpt.h>`, `<setjmp.h>`, mingw32 stubs |
| `sdk/lib/pseh/include` | genincdata | `<pseh/pseh2.h>` |
| `sdk/include/asm` | asm | `<asm.inc>` |

Plus the in-process gen dirs (`xdk-gen/{ddk,psdk}`, `mc-gen`,
`asm-gen`).

The 11 source-tree kernel paths above are not "build artifacts" —
they're the canonical kernel header surface that hand-written
asm + RC + the genincdata build-time tool consume. Eliminating
them requires eliminating the **consumer**, not editing the
include list.

## Forward path: eliminate the source-tree kernel includes

These are the next layers of "what's still C and asm." Each is a
real translation arc, not a build-system tweak.

### 1. Translate the 8 asm files to Rust `global_asm!`

Currently the heaviest source-tree-include consumer. The 8 files
total ~4,600 LOC of i386 trap-frame / context-switch / interlocked
asm, all `#include <ks386.inc>` for kernel struct offsets and
`<internal/i386/asmmacro.S>` for the entry/exit macros.

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

### 2. Translate `ntoskrnl.rc` to embedded Rust resources

Smaller scope than asm. The file pulls `<bugcodes.rc>` (already
in-process), `inbv/bootanim.rc` (a wrapper around per-language
binary blobs), and version info via `<common.ver>` /
`<ntverp.h>`.

Two paths:
- Use the `embed-resource` cargo crate, which would still
  windres-compile our generated `.rc` text. Doesn't actually
  eliminate the `-I` chain.
- Translate the RC tree to `#[link_section]` Rust statics for
  the version blocks + `include_bytes!` for the bootanim
  fragments. Drops `<winver.h>`, `<verrsrc.h>`, `<common.ver>`,
  `<ntverp.h>` — i.e., most of `sdk/include/psdk` for the RC
  consumer.

This eliminates the `compile_resource()` call entirely and drops
3-4 source-tree paths (RC was the biggest consumer of
`sdk/include/psdk` and `sdk/include/vcruntime`).

### 3. Replace `genincdata.c` with a pure-Rust constant emitter

`genincdata.c` is the build-time tool that turns kernel struct
offsets into asm `.equ` symbols. It pulls the deepest header
chain (`<ntndk.h>`, `<arc/arc.h>`, the full ddk/ndk tree).

Once asm is translated to `global_asm!` (item 1), `ks386.inc` is
no longer needed at all — Rust can inline the offsets directly
via `core::mem::offset_of!` against `ntkrnl-sys` bindgen types,
exactly the way `io_tables.rs` already does for the IO tables.

Drops `generate_ks386_inc()` + the `genincdata.c` compile
entirely. Removes `<arc/arc.h>` and the rest of the
heaviest-pulling chain from the cargo PoC's compile surface.

### What's left after items 1–3

If all three land, the cargo PoC's `KERNEL_INCLUDES` would shrink
from 12 source-tree paths to 0. The PoC would compile and link
the kernel with **only the source tree under `sdk/lib/rust/`** —
no `sdk/include/...`, no `ntoskrnl/include/...`. Every kernel
struct offset would route through `ntkrnl-sys` bindgen.

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
