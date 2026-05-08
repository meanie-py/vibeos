# Shim Elimination — Path to Cargo-Driven ntoskrnl

## Goal

Switch `ntoskrnl` from cmake to cargo as the build driver. cargo already
produces the kernel staticlib (`libkernel_rust.a`); we want cargo to
also drive the final link, eliminating the cmake step entirely.

## Required intermediate state: "minimal trampolines"

Cargo can compile C via `cc-rs` from a `build.rs` script, but only
cleanly when the C files have **minimal header dependencies**. The
target shape:

- Headers limited to `<ntdef.h>` + `<stdarg.h>` + `<stddef.h>` + `<stdint.h>`.
- No `#include <ntoskrnl.h>` (and no transitive ddk pulls).
- No FORCEINLINE bodies imported into the trampoline.
- No macro expansions that contain logic (HALDISPATCH dispatch, KGDT
  selectors, `KiComputeIopmOffset`, etc.).
- No struct field accesses requiring full kernel type definitions.
- Body is either pure `va_arg` pull or pure delegation: `_rs_X(args);`.

`ntoskrnl/ntprintf_trampoline.c` is the gold standard — pure `va_arg`
pull into the Rust ntprintf core, no kernel types, no struct fields,
no macros.

## Current state — per-file dependency audit

Header dependency proxies: `_saved_*` uses (FORCEINLINE captures via
Phase 1 rename trick), `->` field accesses (require full struct type),
unique uppercase function-like macros (logic macros, ignoring type
casts), intrinsics (`__sgdt`, `_fxsave64`, etc.).

| File | LOC | `_saved_` | `->` | macros | intrinsics |
|------|----:|----------:|-----:|-------:|-----------:|
| `ntprintf_trampoline.c` | 170 | 0 | 0 | 0 | 0 |
| `io_rust_shims.c` | 70 | 0 | 0 | 0 | 0 |
| `mm_rust_shims.c` | 69 | 0 | 10\* | 1\* | 0 |
| `kdbg_rust_shims.c` | 62 | 0 | 0 | 1\*\* | 0 |
| `kernel_c_helpers.c` | 435 | 1 | 42 | 5 | 0 |
| `ke_rust_shims.c` | 513 | 10 | 79 | 4 | 3 |

\* `mm_rust_shims.c` is at minimal-trampoline status on i386/amd64
(everything outside `#ifdef _M_ARM64` is gone). The remaining `->`
and macro are entirely inside the ARM64-only `Arm64DumpPte` PTE
bitfield dump — a pure debug helper that pokes the UART. Translating
it to Rust requires routing through bindgen's `__bindgen_anon_*`
union accessors; punted as a focused ARM64-only follow-up.

\*\* `kdbg_rust_shims.c`'s remaining macro is `RTL_CONSTANT_STRING`
used to const-init `KdbPromptStr: CSTRING`. Trivially translatable
to Rust (compile-time `pub static`) but low-priority since the file
only also contains the `KdbpPrint` variadic adapter (must stay).

`ntprintf_trampoline.c` is already minimal. Every other file pulls
`<ntoskrnl.h>` and uses substantial logic from it.

To reproduce the audit, comment-strip via `gcc -fpreprocessed -dD -E
-P -x c <file>` then count `_saved_`, `->`, and unique uppercase
function-like calls (filtering type-cast-style names like `ULONG(`,
`PVOID(`, `NULL(`).

## Attack order

Recommended sequence for reaching minimal-trampoline state:

1. **`io_rust_shims.c`** — already at 0 macros / 0 intrinsics.
   Fewest blockers; reaching minimal here is mostly probe-helper +
   small-body translations. Best file to validate the full
   minimal-trampoline → cargo-driven workflow on.
2. **`mm_rust_shims.c`** — 2 macros, both isolatable (`PAGED_CODE`,
   `ARM64_UART_TRACE_HEX`). 119 LOC; quick win.
3. **`kdbg_rust_shims.c`** — 2 intrinsics (`Ke386GetTr` `str` asm) +
   1 macro. KDBG traffic isn't on the smoke path, so regressions are
   only caught by manual KDBG verification.
4. **`kernel_c_helpers.c`** — 42 `->` accesses, 6 macros, 1 `_saved_`.
   Substantive — split into 3 minimal trampolines (variadics, LTO
   workaround, lock outlines) rather than one minimal file.
5. **`ke_rust_shims.c`** — 79 `->`, 10 `_saved_`, intrinsics, TSS init,
   trap-exit chain. Will not reach minimal-trampoline status without
   porting `KiExitV86Trap` + the trap-entry chain (a separate
   multi-month project). Treat as the residual "big trampoline" once
   the others are minimal.

Files 1-3 unblock the cargo-driven build PoC because they're cheap
enough to attempt in parallel; if all three reach minimal, that's
proof-of-concept that the cc-rs path works for non-variadic shims.

## Per-file path to minimal trampolines

### `mm_rust_shims.c` — DONE on i386/amd64 (audit-target reached)

Reduced from 119→69 LOC. On i386/amd64 the file post-preprocessing is
empty (the surviving body is entirely under `#ifdef _M_ARM64`).

Promoted in this pass:
- `MmWaitPageEvent` / `MmSubsectionBase` globals → `mminit.rs` as
  `#[no_mangle] pub static mut`.
- `ExAllocatePoolZero` stdcall variant → `mm-util-inlines` rlib as
  `pub unsafe extern "system" fn ExAllocatePoolZero`. Coexists with
  the cdecl `_ExAllocatePoolZero` from `shim_exports.rs` as a
  separate i386 decoration.
- `MmVerifyImageIsOkForMpUse` — deleted entirely. The Rust caller
  (`ex/init.rs`) has the `CONFIG_SMP` block commented out, so the
  function was unreachable in any current build.

Followup (ARM64-only): translate `Arm64DumpPte` / `MmDbgCheckIrql` /
`MmUartPuts` / `MmUartHex` to Rust. `MMPTE.u.Hard.X` bitfield reads
go through bindgen `__bindgen_anon_*` accessors. Pure debug code
(direct UART pokes — `DbgPrint` doesn't reach serial on ARM64), so
no boot-path implications.

### `io_rust_shims.c` — DONE (audit-target reached)

Reduced from 172→70 LOC; audit proxies are 0/0/0/0. The 8
non-variadic trampolines have been promoted: 3 translated to
`io-inlines` rlib (`HalInitPnpDriver_shim`,
`IopIsValidPhysicalDeviceObject`, `CmiGetNextResourceDescriptor`),
5 deleted as dead code (`HalGetDmaAdapter` — Rust callers go
through `HalDispatchTable` directly; `IopDeviceNodeSetFlag`,
`ObReferenceObject` — Rust impls already existed at the cdecl
decoration with identical bodies; the NTAPI/stdcall trampoline
variant on i386 was unused).

What stays:
- `RtlUnicodeStringPrintf_4wZ_shim` — variadic; can't be a Rust
  export on stable.
- `ProbeForWriteHandle` / `ProbeForReadLargeInteger` — kept here
  until the corresponding `common-inlines` impls grow the same
  `MmUserProbeAddress` overflow/upper-bound check (the C bodies
  raise `ExRaiseAccessViolation` on bad input; the Rust impls
  silently `core::ptr::read_volatile`).

Followup to reach truly minimal headers: drop `<ntoskrnl.h>` and
declare `MmUserProbeAddress` / `ExRaiseAccessViolation` /
`ProbeForWrite` directly. Currently still `#include <ntoskrnl.h>`
because the cargo-driven build hasn't been wired yet.

### `kdbg_rust_shims.c` — DONE (audit-target reached)

Reduced from 162→62 LOC; audit proxies are 0/0/1/0 (the lone macro
is `RTL_CONSTANT_STRING`, used to const-init the kdb prompt string).

Promoted in this pass:
- Globals (KdbCliCallbacks, mode flags, dmesg state, KSPIN_LOCK,
  bug-check requested, init file buffer) → `kdbg::tables` as
  `#[no_mangle] pub static mut`. ~17 storage-only definitions,
  layout-trivial across all 3 arches.
- `Ke386Get{Global,Local}DescriptorTable` (cdecl variants),
  `Ke386GetTr` → `ke-macros` rlib via `core::arch::asm!`. The
  stdcall variants (`_X@N`) already lived there from prior work;
  on i386 the cdecl + stdcall decorations coexist as separate
  symbols. amd64 versions added too: `KdbpCmdGdtLdtIdt` is exported
  unconditionally and references these.
- `KeGet/SetContextFrameRegister`,
  `KeGetTrapFrame{Frame,Stack}Register` → `ke-macros`. Plain
  `CONTEXT::{Ebp,Rbp}` / `KTRAP_FRAME::{Ebp,Rbp,TempEsp,HardwareEsp,
  Rsp}` field reads, both arches.

What stays:
- `KdbpPrint` — variadic.
- `KdbPromptStr` — `CSTRING` const-init via `RTL_CONSTANT_STRING`.
  Translatable but the const-init dance is low-value relative to
  the variadic constraint (file already has to stay for `KdbpPrint`).

### `kernel_c_helpers.c` — Pass 1 done (~3-5 sessions remaining)

PS globals + SharedUserData + GENERIC_MAPPING tables + PsNtDllPathName
moved to `ntoskrnl-rust::ps::ps_globals` (`#[no_mangle] pub static mut`,
~140 LOC of new Rust). Net file reduction was modest (490→435 LOC,
-55) because the heavy `->` and macro counts live in the variadic
functions, which stay in C.

**Latent bug surfaced.** `ntkrnl_sys::constants::PROCESS_ALL_ACCESS`
is the Vista+ value `... | 0xFFFF` = 0x001FFFFF, but the C kernel
build is NT 5.x where the macro evaluates to `... | 0xFFF` =
0x001F0FFF. While the C definition shadowed the Rust one (linker
chose first; `--allow-multiple-definition`), the bug was masked.
Direct Rust ownership exposes it: SMSS process-create access checks
fail and i386 hangs in the SAM/LSA wait loop. Worked around by
hard-coding the NT 5.x mask in `ps_globals.rs`. The constant in
`ntkrnl-sys` should be fixed at the source — separate task.

What still needs translating:
- `ExpKdbgExtFileCache` / `ExpKdbgExtDefWrites` — variadic; can be
  isolated to a minimal trampoline file when other prerequisites are
  ready.
- `ExpKdbgExtHandle` (140 LOC) — substantial port: walks
  `HandleTableListHead`, dereferences `OBJECT_HEADER`, branches on
  `IoFileObjectType` / `CmpKeyObjectType`. ~1 session.
- ARM64 `ARM64_UART_TRACE` / `ARM64_UART_TRACE_HEX` — translatable
  but not blocking i386/amd64 minimal-trampoline status.
- `PsProcessInfoClass` / `PsThreadInfoClass` re-include trick — the
  `#define static` rename pattern needs careful translation.

What stays:
- `KdpDprintf`, `RtlStringCbPrintfA` — variadic.
- `SepInitTokensShim` — verified LLVM i386 fat-LTO bug.
- `MmLockSectionSegment` trio — `__FILE__`/`__LINE__`.

After all of the above: split into 2-3 minimal trampolines per the
prior plan (variadic_shims.c, lto_workaround.c, mm_lock_outlines.c).

### `ke_rust_shims.c` — heaviest (~5+ sessions)

The single hardest file. The 14 `_saved_*` FORCEINLINE captures, 75
struct field accesses, 5 intrinsic uses, and 3 substantive bodies
(`KiAttachProcess`, `KiThreadStartup`, `Ki386InitializeTss`) all
require the full ddk header chain.

What needs translating:
- 4 stdcall-on-i386 ABI bridges (`Hal{Query,Set}SystemInformation`,
  `HalVectorToIDTEntry`, `KeGetTrapFramePc`) — already have `_rs_*`
  bodies in `ke-macros`; could be promoted to bare names with
  `#[no_mangle] pub unsafe extern "system" fn` IF the rustc 1.92
  stdcall ABI bug can be worked around. Currently bug-blocked.
- `KiAttachProcess` (~45 LOC) — accesses `KAPC_STATE`, calls
  `KiMoveApcState`, `KiSwapProcess`, `KiExitDispatcher`. 2 of those
  callees are FORCEINLINE (`KiReleaseDispatcherLockFromSynchLevel` +
  `KiReleaseApcLockFromSynchLevel`). Substantial port.
- `KiThreadStartup` — uses internal `KSWITCHFRAME_SHIM` /
  `KSTART_FRAME_SHIM` / `KUINIT_FRAME_SHIM` stack-frame layouts.
  Layout-sensitive.
- `Ki386InitializeTss` + 2 helpers (~117 LOC) — TSS+GDT+IDT init,
  inline asm (`ltr`, `movl ss`), many constants (`KGDT_TSS`,
  `I386_TSS`, `RPL_MASK`, `KGDT_DF_TSS`, `KGDT_NMI_TSS`).
- `KeRaiseIrqlToSynchLevel` — i386 `__imp__@0` IAT call (dllimport
  thunk). `extern "system" fn IMP_X` declaration in Rust should work.
- `KiComputeIopmOffset` — KTSS layout-dependent; `offset_of!` in Rust
  doesn't support runtime array indexing, so requires manual offset
  computation.

What stays:
- 3 trap-exit `_saved_*` outlines (`KiExitV86Trap`,
  `KiExitTrapDebugChecks`, `KiExitSystemCallDebugChecks`) — substantively
  intertwined with PSEH and KiEoiHelper / KiDeliverApc / KfRaiseIrql /
  KiDumpTrapFrame / KeBugCheckEx. Translation would require porting
  those callees first.
- `KiSaveXState` / `KiRestoreXState` — KVM xrstors64 workaround on
  amd64. Stays.
- `KiEndInterrupt` — re-probed 2026-05-07; rustc 1.92 emits cdecl call
  sequences for `extern "stdcall"` (`HalEndSystemInterrupt`) and
  `extern "fastcall"` (`KiEoiHelper`) declarations, breaking the link.
  C trampoline kept.
- ARM64 `KdbpPrint` empty stub — variadic.
- SSDT include — `napi.h` generates the syscall table. Stays.

After translation: file would still need full ddk for the trap-exit
outlines and TSS init. Probably never reaches "minimal trampoline"
unless the trap-exit chain itself is ported to Rust (which is its
own multi-month project).

## Cargo-driven build prerequisites

After all shim files reach minimal-trampoline status:

1. **C compilation**: use `cc-rs` in a `build.rs` script. Each .c file
   with minimal headers compiles standalone.
2. **Variadics**: ~5-10 minimal .c files will always remain (variadic
   exports aren't possible from stable Rust). cc-rs handles them fine.
3. **Resource compilation**: `.rc` files (versioninfo, message tables)
   currently use `windres` / `llvm-rc`. The `embed-resource` cargo
   crate handles this.
4. **Multi-arch dispatch**: cargo `--target i686-pc-windows-gnu` /
   `x86_64-pc-windows-gnu` / `aarch64-pc-windows-gnullvm` already
   works.
5. **PE-COFF link flags**: cargo `[profile.X]` + `-Clink-arg=` can pass
   `-nostdlib`, `-fuse-ld=lld`, `--image-base=...`, `-entry=...`,
   `--subsystem=native:5.01`, etc. Doable.
6. **Host build tools**: `pefixup`, `fix_lld_forwarding.py`, etc. —
   build.rs scripts can compile/run these as a pre-link step.
7. **`SepInitTokensShim` LLVM workaround**: must be compiled outside
   Rust LTO scope. Easy via separate `cc::Build` invocation in
   build.rs that produces an isolated static lib.
8. **`--allow-multiple-definition` link flag**: currently used to
   handle the ABI-anchor-vs-Rust-impl symbol conflict. Should be
   eliminable once trampolines are fully delegating.

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

## Verified must-stay set

These have been re-probed and **proven** to require the full kernel
header chain. Not candidates for further elimination.

- **Variadic exports** (`KdbpPrint`, `KdpDprintf`, `RtlStringCbPrintfA`,
  KDBG ext printers, `RtlUnicodeStringPrintf_4wZ_shim`) — Rust can't
  define C-variadic exports on stable. Variadic functions can be
  isolated to minimal trampolines (see `ntprintf_trampoline.c`).
- **`SepInitTokensShim`** (i386) — LLVM fat-LTO miscompilation:
  `ud2` inserted after `@ObInitializeFastReference@8` call. None of
  `opaque_ptr`, `compiler_fence(SeqCst)`, `core::hint::black_box`
  defeat it. C-side execution outside the Rust LTO scope is the only
  working workaround. Re-probed 2026-05-07.
- **`KiSaveXState` / `KiRestoreXState`** (amd64) — KVM bug: `xrstors64`
  faults on StateSaveArea pages that look valid via PTE
  self-referencing. Force `fxsave64`/`fxrstor64` instead.
- **`MmLockSectionSegment` trio** — uses `__FILE__`/`__LINE__` for
  call-site recording in the underlying `_Mm*` primitives.
- **3 i386 trap-exit `_saved_*` outlines** (`KiExitV86Trap`,
  `KiExitTrapDebugChecks`, `KiExitSystemCallDebugChecks`) — PSEH-
  intertwined with the trap-entry chain.
- **rustc 1.92 stdcall/fastcall ABI bug**: rustc emits cdecl call
  sequences for `extern "stdcall"`/`extern "fastcall"` calls to
  external symbols on i386, breaking the link against decorated
  symbols (`_X@N` / `@X@N`). Affects `KiEndInterrupt` translation.
  Workaround: keep C trampoline, OR use inline-asm to force the right
  calling convention.

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

## Lessons from prior translations

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
