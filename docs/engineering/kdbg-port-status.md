# kdbg/ Rust port — done (2026-04-28)

## TL;DR

7 Rust modules in `sdk/lib/rust/ntoskrnl-rust/src/kdbg/` replace
`ntoskrnl/kdbg/{kdbg,kdb,kdb_cli,kdb_cmdhist,kdb_expr,kdb_print,kdb_symbols}.c`
on i386 + amd64. **All 7/7 smoke tests pass on both arches** (i386 also passes
full clean-build cargo). ARM64 is unchanged (KDBG is off there).

## Inventory

| File | Rust LOC | Source LOC | Functions |
|------|----------|------------|-----------|
| kdbg.rs | 429 | 212 | 6 |
| kdb.rs | 1,784 | 1,682 | 23 (incl. KdbpDeleteBreakPoint, KdbpEnableBreakPoint) |
| kdb_cli.rs | 3,656 | 3,711 | 30 + 22 KdbpCmd* handlers |
| kdb_cmdhist.rs | 310 | 162 | 6 |
| kdb_expr.rs | 2,283 | 1,216 | 26 |
| kdb_print.rs | 413 | 201 | 7 |
| kdb_symbols.rs | 1,026 | 469 | 7 |
| **types.rs** | 80 | n/a | shared kdb-private types |
| **kdbg_rust_shims.c** | 539 | n/a | variadic + statics + intrinsics |

## Translation pipeline

**First attempt (failed):** `rustconv batch -m sonnet`. Output for kdb_cli.c
was 8.9K tokens (no truncation hit) but Sonnet stubbed all 22 `KdbpCmd*`
command handlers with `1u8`/`0u8` bodies instead of translating them. ~85%
of resulting kdb_cli.rs was C source embedded as comments.

**Second attempt (worked):** `rustconv translate` (function-by-function,
separate API call per function). Full-fidelity translations: 13.8K out
tokens for kdb (21 functions), 46.6K out tokens for kdb_cli (46 functions),
all real bodies. Two functions (`KdbpDeleteBreakPoint`,
`KdbpEnableBreakPoint`) skipped by topo sort due to a tree-sitter parse
glitch around `OPTIONAL` C annotation macros — translated by subagent.

## Architecture

### Symbol naming under #ifdef KDBG

`<ntoskrnl.h>` macro-renames the public KD-protocol entry points when KDBG
is on (so `KdSendPacket` → `KdbgSendPacket`, etc.). The Rust port mirrors
this:

- `kd/kdmain.rs` + `kd/kdio.rs` keep the canonical `KdD0Transition`,
  `KdSendPacket`, `KdReceivePacket`, etc. names.
- `kdbg/kdbg.rs` exports `KdbgD0Transition`, `KdbgSendPacket`,
  `KdbgReceivePacket` etc., which are the symbols that the rest of the
  kernel calls when KDBG is enabled.

### kdbg_rust_shims.c

C shim providing what Rust can't or shouldn't:

1. Variadic `KdbPrintf` (`__cdecl`) and `KdbpPrint` — Rust can't define
   C-variadic functions on stable.
2. Static command tables — `KdbDebuggerCommands` (with all 22 KdbpCmd*
   function pointers), `ComponentTable` (~150 DPFLTR_* entries),
   `KdbCliCallbacks`, `KdbPromptStr`. These reference Rust functions; the
   C array initializers are easier to write in C.
3. File-private statics from the original `kdb_cli.c`:
   `KdbUseIntelSyntax`, `KdbBreakOnModuleLoad`, dmesg ring state, term
   state, etc.
4. `KDB_*_TO_STRING` macro→function shims.
5. Out-of-line wrappers for FORCEINLINE intrinsics:
   `Ke386GetGlobalDescriptorTable`, `Ke386GetLocalDescriptorTable`,
   `Ke386GetTr`, `KeGetContextFrameRegister`, `KeSetContextFrameRegister`,
   `KeGetTrapFrameFrameRegister`, `KeGetTrapFrameStackRegister`.

### Shared kdb-private types

`kdbg/types.rs` holds the Rust translations of the types from
`ntoskrnl/kdbg/kdb.h`: `KDB_BREAKPOINT`, `KDB_BREAKPOINT_TYPE`,
`KDB_BREAKPOINT_DATA`, `PKDB_BREAKPOINT`, `KDB_KTRAP_FRAME`,
`PKDB_KTRAP_FRAME`, `KDB_ENTER_CONDITION`, `KDB_ACCESS_TYPE`,
`KD_CONTINUE_TYPE`. Imported by all kdbg files via `use super::types::*;`.

### Build gate

`pub mod kdbg;` is gated on `cfg(any(target_arch = "x86", target_arch = "x86_64"))`
in `ntoskrnl-rust/src/lib.rs` — it's compiled unconditionally on i386 and
amd64 (matching the C build's `if(KDBG AND NOT ARCH STREQUAL "arm64")`),
not exposed on ARM64.

## The load-bearing fix

The integration's hardest debug: 3 unexpected `ud2` instances in
`KdbpAttachToThread` on i386. Root cause was an `extern "C"` (cdecl)
declaration of `ObDereferenceObject` in kdb.rs while the actual
definition in `ob/obref.rs` is `extern "system"` (stdcall). The ABI
mismatch let LLVM detect inconsistency and replace the surrounding code
with `ud2` (the captures(none) pattern documented in
`MEMORY.md / feedback_cgu1.md`). Moving the decl to a separate
`extern "system"` block cleared all three.

## Smoke results

**i386:** 7/7 (incremental + full clean build)
**amd64:** 7/7 (incremental); the explorer-milestone regression that's
been on the board for weeks is gone — likely cleared incidentally by
this session's ABI fixes.
**ARM64:** unchanged (KDBG is off, kdbg/ files not compiled).

## Known followup

The bigger LLVM captures(none) cache-sensitivity issue
(`feedback_cgu1.md`) is still open — `--skip-clean-build` always works,
but full clean amd64 occasionally hits the KiSystemStartup ud2 cascade.
Adding ~9.6K LOC of Rust to the kernel may make that more visible. The
systemic fix (opaque_ptr barriers across all #[no_mangle] functions) is
tracked separately.
