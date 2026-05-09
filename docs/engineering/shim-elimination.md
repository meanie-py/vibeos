# Shim Elimination — Path to Cargo-Driven ntoskrnl

## Status (2026-05-08)

**The shim layer is ready for `cc-rs` compilation.** As of the
2026-05-08 round:

- All 8 active shim files are header-free. Compile with only
  `<stdarg.h>` / `<stddef.h>` / `<stdint.h>` plus a small local typedef
  block. No `<ntoskrnl.h>`, no internal kernel headers.
- `kernel_c_helpers.c` retired entirely. Rust supplies the canonical
  `PsProcessInfoClass` / `PsThreadInfoClass` linker symbols from
  `ntoskrnl-rust/src/ps/info_class.rs`.
- `arm64_uart.c` retired in 2026-05-07 (UART trace helpers moved to
  Rust under `cfg(target_arch = "aarch64")`).
- New sister-file pattern: `io/iomgr/io_tables.c` (19 LOC). Tiny
  PCH-included data-tables file that hosts `IopQueryOperationLength`
  + 7 sister arrays. Sister files are NOT shims — they're the safety
  valve for cases where data tables need `<ntoskrnl.h>` for `sizeof()`
  evaluation but the shims can't.

The cargo-driven `ntoskrnl.exe` build PoC for i386 boots to desktop
and passes 7/7 smoke. See `docs/engineering/ntoskrnl-translation-audit.md`
for current LOC ledger and forward path.

## Goal

Switch `ntoskrnl` from cmake to cargo as the build driver. cargo already
produces the kernel staticlib (`libkernel_rust.a`) and drives the final
link in the i386 PoC; we want cargo to also compile the shims (via
`cc-rs`) so the whole ntoskrnl build is one cargo invocation per arch.

## Required intermediate state — REACHED for all 8 shims

Cargo can compile C via `cc-rs` from a `build.rs` script, but only
cleanly when the C files have **minimal header dependencies**. Target
shape:

- Headers limited to `<stdarg.h>` + `<stddef.h>` + `<stdint.h>` + an
  in-file local typedef block for NT primitive types.
- No `#include <ntoskrnl.h>` (and no transitive ddk pulls).
- No FORCEINLINE bodies imported into the trampoline.
- No macro expansions that contain logic (HALDISPATCH dispatch, KGDT
  selectors, `KiComputeIopmOffset`, etc.) — inline the formula instead.
- No struct field accesses requiring full kernel type definitions.
  Where layout knowledge is needed, hide it behind a `_rs_*` Rust
  helper that returns a raw pointer (e.g. `_rs_PsCurrentProcessTokenAddr()`
  → `&CurrentProcess->Token`).
- Body is either pure `va_arg` pull or pure delegation: `_rs_X(args);`.

## Per-file status (2026-05-08)

All 8 shim files have reached header-free status. The dependency-proxy
table from earlier rounds (counting `_saved_*`, `->`, function-like
macros, intrinsics) is now uniformly `0/0/0/0` for every shim — they
no longer pull `<ntoskrnl.h>` so there's nothing to count.

| File | LOC | Status |
|------|----:|--------|
| `ntprintf_trampoline.c` | 170 | header-free since the earliest pass (gold-standard variadic adapter) |
| `variadic_shims.c` | 77 | header-free since 2026-05-07 |
| `mm_lock_outlines.c` | 55 | header-free 2026-05-08 (round 3) |
| `kdbg_rust_shims.c` | 91 | header-free 2026-05-08 (round 2) |
| `io_rust_shims.c` | 154 | header-free 2026-05-08 (round 2) |
| `mm_rust_shims.c` | 152 | header-free 2026-05-08 (round 2) |
| `lto_workaround.c` | 87 | header-free 2026-05-08 (round 3) |
| `ke_rust_shims.c` | 251 | header-free 2026-05-07 (round 1) |
| ~~`kernel_c_helpers.c`~~ | — | **RETIRED 2026-05-08**. Rust supplies `PsProcessInfoClass` / `PsThreadInfoClass` |
| ~~`arm64_uart.c`~~ | — | retired 2026-05-07 (Rust takes the UART traces) |

Sister data-table files (PCH-included, NOT in PCH_SKIP_SOURCE):

| File | LOC | Notes |
|------|----:|-------|
| `io/iomgr/io_tables.c` | 19 | Hosts 8 `Iop*Operation{Length,Access}` arrays via `#include <ntoskrnl.h>` + `#include "internal/io_i.h"`. Externalises symbols for Rust `iofunc.rs`. Created 2026-05-08 when extracting data tables out of `io_rust_shims.c` so the latter could go header-free. |

Sister files are not "shims" and don't need to be header-free —
they're tiny adapters that exist precisely BECAUSE we want the shims
header-free, by absorbing the kernel-header dependency in a separate
PCH-included compile unit.

## Round-by-round history

The header-drop arc completed in 4 rounds:

- **Round 1 (2026-05-07):** `ke_rust_shims.c`. Local NT typedefs +
  inlined `KiComputeIopmOffset` formula (`offsetof(KTSS, IoMaps) =
  0x68`, `sizeof(KTSS) = 8364`). Plus the substantive Phase 1
  translations (`KiAttachProcess`, `KiThreadStartup`, `Ki386InitializeTss`)
  that finished off the file's "haven't been ported" column.
- **Round 2 (2026-05-08):** `mm_rust_shims.c`, `io_rust_shims.c`,
  `kdbg_rust_shims.c`. Plus extracted `io_tables.c` (the
  `IopQueryOperationLength` etc. arrays were sneaking in through the
  io shim's old `internal/io_i.h` include).
- **Round 3 (2026-05-08):** `lto_workaround.c`, `mm_lock_outlines.c`.
  New `_rs_PsCurrentProcessTokenAddr` Rust helper hides EPROCESS
  layout from the C trampoline.
- **Round 4 (2026-05-08):** `kernel_c_helpers.c` retired. Discovered
  the "alignment regression" was actually a pre-existing latent bug
  in `ps/query.rs` — `PS_PROCESS_INFO_CLASS_COUNT` and
  `PS_THREAD_INFO_CLASS_COUNT` had been transcribed from Vista+
  enum maxes (51/56) instead of NT 5.x array sizes (47/28). Fixed
  the constants, renamed `_rs_PsProcessInfoClass` →
  `PsProcessInfoClass` (and same for thread), deleted the C file.

## Patterns and tactics that worked

These came up repeatedly. Useful for future header-drop work on any
new C surface that creeps in:

### Local NT typedef block

Replaces `<ntoskrnl.h>` for files that just need primitive NT types.
Example from a header-free shim:

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
typedef uintptr_t         ULONG_PTR;  /* uintptr_t — NOT uint64_t. Width matters on i386. */

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
/* etc. */
```

The `ULONG_PTR = uintptr_t` distinction matters: `uint64_t` is wrong
on i386 (would make NTAPI signatures balloon to wrong stdcall
decoration — `_X@16` instead of `_X@12` for a typical 3-arg fn).

### Rust helper for layout-dependent C accesses

Where a shim needs the address of an EPROCESS / KTHREAD field (the
classic case is `&CurrentProcess->Token`), don't pull the kernel
struct definitions into C — add a tiny Rust helper that returns the
raw pointer. Example: `_rs_PsCurrentProcessTokenAddr()` in
`shim_exports.rs` returns `&CurrentProcess->Token` as `*mut PVOID`,
keeping the EPROCESS layout entirely in Rust where it belongs.

### Sister-file pattern for data tables

When a shim was the accidental host of externally-needed data tables
(the `io_rust_shims.c` case with the 8 `Iop*Operation*` arrays from
`internal/io_i.h`), extract the tables to a tiny dedicated file
(`io_tables.c`, 19 LOC). The new file keeps `<ntoskrnl.h>` and is
PCH-included like a regular ntoskrnl source — NOT in
`PCH_SKIP_SOURCE`. This separates the "header-free shim" concern
from the "needs kernel struct sizes for `sizeof()`" concern.

The file count grows by one but the shim becomes header-free.

### Failed pattern: PCH-included `static` re-include

Tried this for `kernel_c_helpers.c` → `ps_tables.c` rename in round 4
and it didn't work. The trick is:

```c
#define PsProcessInfoClass _static_PsProcessInfoClass
#include <ntoskrnl.h>     /* pulls ps_i.h with `static const ... _static_PsProcessInfoClass[]` */
#undef PsProcessInfoClass
#pragma push_macro("static")
#define static
#include "internal/ps_i.h"  /* now WITHOUT static, externalising the symbol */
#pragma pop_macro("static")
```

This requires the file to be PCH-SKIPPED. PCH-included siblings have
already pulled `ps_i.h` (transitively via the precompiled
`<ntoskrnl.h>`) with the `static` qualifier intact, so the unconditional
re-include creates a static-vs-non-static redefinition. The PCH machinery
runs at PCH-build time, not at file-compile time, so there's no way to
redefine `static` early enough to affect the PCH-pulled inclusion.

Lesson: the `static` re-include trick is fundamentally tied to
PCH-skipping. If you want a tiny `<ntoskrnl.h>`-pulling adapter to live
alongside header-free shims, the data has to come through a non-trick
path (regular global definition in the file, which only works for
data that's NOT already declared `static` in a kernel header).

For `kernel_c_helpers.c`'s case, the answer was to translate the trick
to Rust and supply the symbols from there — which works because Rust
doesn't have C's TU-private vs externally-linkable wars.

## Definition of done

The cargo-driven build is "done" when:

- `cargo build -p ntoskrnl --target <arch>` produces a working
  `ntoskrnl.exe` for all 3 arches without invoking cmake.
- The 3 smoke tests pass (i386 + amd64 boot to `shell ready`,
  arm64 builds clean) on the cargo-driven binary.
- `tools/scan_ud2.py --check build-<arch>` is clean.
- The cmake build path can be removed (or kept as a parallel CI
  check), with no functionality regression.
- Variadic trampolines remain (~5-10 minimal `.c` files compiled by
  `cc-rs`) — that's expected, not a regression.

## What still has to stay C

These have been re-probed and **proven** to require C-side execution
or a separate compile unit. Not candidates for further elimination
until the underlying constraint moves.

- **Variadic exports** (`KdbpPrint`, `KdpDprintf`, `RtlStringCbPrintfA`,
  KDBG ext printers, `RtlUnicodeStringPrintf_4wZ_shim`) — Rust can't
  define C-variadic exports on stable. Variadic functions can be
  isolated to minimal trampolines (see `ntprintf_trampoline.c`).
- **`SepInitTokensShim`** (i386, in `lto_workaround.c`) — LLVM fat-LTO
  miscompilation: `ud2` inserted after `@ObInitializeFastReference@8`
  call. None of `opaque_ptr`, `compiler_fence(SeqCst)`,
  `core::hint::black_box` defeat it. C-side execution outside the
  Rust LTO scope is the only working workaround. Re-probed 2026-05-07.
- **`KiSaveXState` / `KiRestoreXState`** (amd64) — KVM bug: `xrstors64`
  faults on StateSaveArea pages that look valid via PTE
  self-referencing. Force `fxsave64`/`fxrstor64` instead.
- **`MmLockSectionSegment` trio** — uses `__FILE__`/`__LINE__` for
  call-site recording in the underlying `_Mm*` primitives. Rust's
  `core::file!()` / `core::line!()` would produce Rust source paths,
  not the C-side paths the diagnostics want.
- **3 i386 trap-exit `_saved_*` outlines** (`KiExitV86Trap`,
  `KiExitTrapDebugChecks`, `KiExitSystemCallDebugChecks`) — PSEH-
  intertwined with the trap-entry chain.
- **rustc 1.92 stdcall/fastcall ABI bug**: rustc emits cdecl call
  sequences for `extern "stdcall"`/`extern "fastcall"` calls to
  external symbols on i386, breaking the link against decorated
  symbols (`_X@N` / `@X@N`). Affects `KiEndInterrupt` translation.
  Workaround: keep C trampoline, OR use inline-asm to force the right
  calling convention.

## Cargo-driven build prerequisites

After all shim files reached header-free status (now done):

1. **C compilation**: use `cc-rs` in a `build.rs` script. Each shim
   compiles standalone — no `cc::Build::include()` needed.
2. **Variadics**: ~5-10 minimal `.c` files will always remain (variadic
   exports aren't possible from stable Rust). cc-rs handles them fine.
3. **`io_tables.c` (sister file)**: needs `cc::Build::include()` for
   `<ntoskrnl.h>`, OR translation to Rust. Pick whichever needs less
   plumbing for the cargo PoC.
4. **Resource compilation**: `.rc` files (versioninfo, message tables)
   currently use `windres` / `llvm-rc`. The `embed-resource` cargo
   crate handles this.
5. **Multi-arch dispatch**: cargo `--target i686-pc-windows-gnu` /
   `x86_64-pc-windows-gnu` / `aarch64-pc-windows-gnullvm` already
   works.
6. **PE-COFF link flags**: cargo `[profile.X]` + `-Clink-arg=` can pass
   `-nostdlib`, `-fuse-ld=lld`, `--image-base=...`, `-entry=...`,
   `--subsystem=native:5.01`, etc. Doable.
7. **Host build tools**: `pefixup`, `fix_lld_forwarding.py`, etc. The
   smoke-passing PoC binary skipped both — strong sign that with the
   modern linker/Rust setup neither is strictly required for boot, but
   worth a careful audit.
8. **`SepInitTokensShim` LLVM workaround**: must be compiled outside
   Rust LTO scope. Easy via separate `cc::Build` invocation in
   build.rs that produces an isolated static lib.
9. **`--allow-multiple-definition` link flag**: currently used to
   handle the ABI-anchor-vs-Rust-impl symbol conflict. Should be
   eliminable once trampolines are fully delegating.

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

`_X` = cdecl, `_X@N` = stdcall, `@X@N` = fastcall. If the Rust caller's
extern decl produces a different decoration than the symbol's actual
decoration, that's the bug.

## Lessons from the header-drop arc

These keep recurring; track them.

- **Captures(none) ghosts.** Many shim comments cite "LLVM cross-CGU
  captures(none) attribute propagation" as the reason a function must
  stay in C. The workspace `lto = false` + `codegen-units = 256`
  firewall has been the default for months — most of these comments
  are stale. Re-probe before assuming the workaround is still needed.
- **ABI mismatch looks like captures(none).** ud2 + folded success
  path on i386 has the same disassembly pattern for both. Check ABI
  first via `nm` (1 second) before chasing captures(none) (minutes
  of failed write_volatile / opaque_ptr defenses).
- **Shims hide latent bugs.** The shim wraps the symptom; the
  underlying bug only surfaces when the call goes direct.
  `IopReadyDeviceObjects` had an `extern "C"`/`extern "system"` ABI
  mismatch in Rust that linked silently for months until
  `IopInitializeDriverModule` translation removed the C trampoline.
  More recently: the `kernel_c_helpers.c` "alignment regression" was
  a pre-existing `PS_*_INFO_CLASS_COUNT` off-by-many in `ps/query.rs`
  that the C `ps_i.h` re-include happened to mask via favorable
  trailing `.rdata` bytes.
- **Empty extern decls don't preserve the symbol.** The historical
  `extern "C" { static mut FOO: [T; 0]; }` pattern (kdbg ComponentTable
  / KdbDebuggerCommands) makes the optimizer fold all indexing to
  panics. Properly size the array in Rust.
- **i386 has 3 decorations, not 1.** Before deleting a "dead" shim,
  grep for `_X` (cdecl), `_X@N` (stdcall), AND `@X@N` (fastcall).
  Different Rust callers use different `extern "..."` ABIs.
- **`global_asm!` symbols need force-pulling.** Adding `core::arch::
  global_asm!(...)` in an rlib doesn't put the symbols in
  `libkernel_rust.a` unless something references them. The
  `_force_<crate>_symbols` pattern in `kernel-rust/src/lib.rs` keeps
  them alive.
- **PCH-skipped vs PCH-included is load-bearing.** A C file that
  uses the `#define static` re-include trick to externalise a
  `static` array (the `kernel_c_helpers.c` pattern) ONLY works when
  the file is PCH-skipped. Trying to make it a PCH-included
  sister file fails with a static-vs-non-static redefinition error
  because the PCH already pulled the header. For PCH-included
  sister files, only data that's NOT already `static` in the
  upstream header can be externalised cleanly.
- **`uintptr_t` not `uint64_t` for `ULONG_PTR`.** Width matters on
  i386 — getting it wrong changes stdcall decoration (`_X@16` vs
  `_X@12` for a typical 3-arg fn) and breaks the link.
- **The "Rust translation regression" framing is sometimes wrong.**
  When a Rust translation that's "byte-identical" to a C
  predecessor crashes after a symbol-supplier swap, look upstream
  before blaming the translation. The C trick may have been
  accidentally masking a latent caller-side bug.

## Recipe for re-probing a shim

1. Read the in-source comment for the original symptom.
2. Grep for Rust callers across all 3 i386 decorations (`_X`,
   `_X@N`, `@X@N`).
3. If used: drop the C trampoline, declare the underlying Rust fn
   directly with the right ABI. Verify decoration via `nm`.
4. Build all 3 arches. `python3 tools/scan_ud2.py --check build-i386`
   is the canary — unexpected ud2 in a named function is the smoking
   gun.
5. Run smoke. Green on i386 + amd64 = ghost. Failure = either a real
   bug surfaced (ABI mismatch, missing volatile, etc.) or a genuine
   workaround that has to stay.

## Recipe for dropping `<ntoskrnl.h>` from a new C shim

For when a fresh shim creeps in and needs the header-drop treatment:

1. **Audit the symbol surface.** What types, functions, and globals
   does the file actually use? Grep callers, look at the function
   bodies. Most files use a tiny fraction of `<ntoskrnl.h>`.
2. **Add to `PCH_SKIP_SOURCE`** in both `ntoskrnl/CMakeLists.txt` and
   `ntoskrnl/ntkrnlmp/CMakeLists.txt`. Without this the PCH still
   forcibly injects `<ntoskrnl.h>` regardless of what the file's
   own `#include` lines say.
3. **Replace `<ntoskrnl.h>` with local typedefs.** Use `<stdarg.h>` /
   `<stddef.h>` / `<stdint.h>` plus a hand-written NT primitive types
   block (see "Local NT typedef block" above).
4. **Replace function-pulling macros with `extern` decls.** The
   underlying functions are usually exported by ntoskrnl-rust or live
   in a sibling rlib. Check decoration via `nm`.
5. **Replace `&Process->Field` with a `_rs_*` Rust helper.** Don't
   pull EPROCESS / KTHREAD layout into C.
6. **Watch for "secret" data dependencies.** A `#include
   "internal/foo_i.h"` may be silently providing data tables that
   Rust callers extern-link against. Extract those to a sister file
   (see `io_tables.c`).
7. **Build all 3 arches + run all 3 smokes.** A change here can blow
   up just one arch (e.g. i386 ABI decoration mismatches that amd64
   tolerates).

If all that succeeds you've got a `cc-rs`-compileable shim. If you
hit the `static` redefinition error from the previous section, the
file's data can't move to a sister file — translate the data to Rust
instead.
