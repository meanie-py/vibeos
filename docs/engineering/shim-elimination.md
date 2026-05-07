# Shim Elimination Plan

Last updated 2026-05-06.

## Status snapshot

The kernel has **~7 `*_rust_shims.c` / `kernel_c_helpers.c` / trampoline
files**, totaling ~7,200 LOC. Most of the LOC is now one-line `_rs_*`
delegations into Rust impls living in sibling rlibs under `sdk/lib/rust/`,
not real C work — but the C symbol stays alive as an ABI anchor that Rust
+ C callers both resolve to. The shim layer is a *bridge* now, not a
backlog of untranslated work.

The originally-claimed "must-stay" set keeps shrinking on every re-probe.
The settled must-stay categories are short:

- **Variadic trampolines** — Rust can't safely cross the `va_arg` boundary,
  so the C `va_arg` pull stays. Bodies have collapsed to ~6 lines that
  hand a typed slot array to the Rust `ntprintf` / `ntprintf_w` core.
- **A handful of LLVM/KVM workarounds with documented evidence** —
  `KiSaveXState`/`KiRestoreXState` on amd64 (KVM, not LLVM),
  `SepInitTokensShim` on i386 (fat-LTO ud2). These have specific repros.
- **`__FILE__`/`__LINE__` macros** that need C preprocessor info.
- **Some i386 register/V86/segment shims** — `Ke386Set*Reg`, `KiEnterTrap`,
  `Ki386InitializeTss`, FXSAVE+MXCSR sanitization. Probably some of these
  could move to `core::arch::asm!`, but trap-entry ones are intertwined
  with PSEH and may be cheaper to leave alone.
- ~~**C `#include <avlsupp.c>` AVL bridges**~~ — CLOSED 2026-05-06.
  `mm-avl` rlib (sdk/lib/rust/mm-avl) handles the MMADDRESS_NODE flavor;
  the avlsupp.c-include trick is gone.
- **KDBG enum-name-to-string lookup tables** — pure debugger output
  formatting, no Rust value-add.

Everything else is in principle eliminable. The history below has lessons
about which "must-stay" claims have repeatedly turned out wrong on
re-probe.

## Pattern that makes this work

**Augment, don't replace.** When a C `_shims.c` file's body translates to
Rust, keep the C symbol alive as a one-line trampoline that delegates to
the Rust impl. The Rust impl lives in a sibling rlib (`sdk/lib/rust/<name>-inlines/`
or `<name>-macros/`), separate from `ntoskrnl-rust`. Existing C callers and
Rust `extern { fn X }` callers don't change — they still resolve to the
same C symbol. Only the C body collapses.

Why a sibling rlib instead of moving the impl into ntoskrnl-rust? Because
adding a `#[no_mangle] extern "C" fn` to `ntoskrnl-rust` cascades ud2 into
unrelated functions in the same crate (rustc cross-CGU attribute deduction
re-shuffles when new symbols appear). The crate boundary cuts that
deduction wire. Workspace settings `lto = false` + `codegen-units = 256`
plus `opaque_ptr` laundering at FFI boundaries give the rest of the
firewall.

The 13 sibling rlibs that landed in Phase 1: `cm-inlines` (8 KCB-locks),
`ke-inlines` (49 ke_x.h FORCEINLINEs), `ob-inlines` (22), `ps-inlines`
(18), `se-inlines` (10), `common-inlines` (34), `mm-{pte-class, pte-mut,
util, ws-lock, addr-space, pte-atomic, lock}-inlines` (45 total across 7
mm rlibs). Phase 2 added `ke-macros`, `io-inlines`, `mm-macros`. Phase 4
added `mm-avl` (8 fns: Find/Insert/Delete + 5 accessors, ~330 LOC of
algorithm). ~173 fns total of Rust impl across the rlib zoo; ~3340 →
~2700 LOC reduction in `ntoskrnl-rust::shim_exports.rs` from the
extractions.

## Where the file count stands

| File | LOC | Notes |
|---|---:|---|
| `ntoskrnl/ke/ke_rust_shims.c` | ~2,100 | Mostly Phase 1 trampolines; large i386 register/V86/FXSAVE block (~50 fns) is the bulk. |
| `ntoskrnl/mm/mm_rust_shims.c` | ~335 | Down from ~1,500. WS-lock family + 13 macro/util trampolines + AVL insert-guard collapsed; 3 large `#if 0` debug-preservation blocks deleted (git history has them). What's left: ExAllocatePoolZero (i386 stdcall), Arm64DumpPte, ARM64 UART helpers, MmVerifyImageIsOkForMpUse (CONFIG_SMP), MmWait/Subsection globals, MmDbgCheckIrql. |
| `ntoskrnl/io/iomgr/io_rust_shims.c` | ~750 | Down from ~1,465. captures(none) and `_c` PnP families fully eliminated 2026-05-06. |
| `ntoskrnl/kernel_c_helpers.c` | ~635 | Canonical "must-stay" home — varargs, LTO workarounds, AVL include trick, fastcall→cdecl bridges. |
| `ntoskrnl/kdbg/kdbg_rust_shims.c` | ~540 | Mostly KDBG-only debugger code (lookup tables, register dumps). |
| `ntoskrnl/config/cm_rust_shims.c` | ~340 | KCB lock wrappers + a few macros. Phase 3 dual-decoration cleanup landed. |
| `ntoskrnl/kd/kd_rust_shims.c` | 28 | HAL `KdComPortInUse` getter/setter. |
| `ntoskrnl/ntprintf_trampoline.c` | ~100 | `va_list` adapter for Rust ntprintf. |

The ke_rust_shims.c i386 register/V86 block is the single biggest item;
some of it is genuinely arch-specific glue, some could move to Rust
inline asm.

## What we've learned

Track this. The pattern keeps repeating.

### "Captures(none) is an LLVM bug" — was wrong, mostly

The `*_shims.c` files had ~10 entries with comments like "LLVM cross-CGU
captures(none) attribute propagation" + "calling it through black_box() in
caller miscompiles to ud2" + "even with volatile reads, the compiler
hard-codes the else branch". Every single one of these on the io ledger
turned out to be solvable by the workspace `lto = false` +
`codegen-units = 256` firewall (which has been the default for months — the
shim comments were stale).

When you see ud2 + folded success path on i386, **check ABI first** before
going down the captures(none) rabbit hole. Both produce the same disasm
pattern. ABI is a 1-second `nm` check.

### "Must stay in C because the Rust version causes amd64-specific hang"

Eight `_c`-suffixed PnP shims had this comment. All were captures(none)
ghosts. Five became real Rust translations; three were dead-code deletions
(the Rust version had quietly been promoted past the trampoline already,
nobody updated the comment).

### "Complex delegation, real translation work needed"

`IopInitializeDriverModule` (240 LOC) was the last "complex delegation" on
the io ledger. The translation took ~30 min of careful porting plus ~30
min of debugging — but the bug found wasn't a translation bug, it was an
`extern "C"` vs `extern "system"` ABI mismatch on `IopReadyDeviceObjects`
that had been latent in driver.rs for months. The shim was hiding it.

### "Shim is dead weight; safe to delete"

Sometimes true (e.g., `ObCreateObject_shim` had no callers by the time we
re-probed). Sometimes wrong because the agent grepped only the cdecl name
and missed the stdcall variant (`Ke386GetTr` was needed under
`_Ke386GetTr@0`). Manually grep all three i386 decoration variants
(`_X` cdecl, `_X@N` stdcall, `@X@N` fastcall) before deleting.

### "The shim hides a real bug"

Most repeated lesson. Many shim categories that were classified
"must-stay" turned out to be hiding latent bugs in the Rust caller — ABI
mismatch, captures(none) workaround left over from before the firewall,
flexible-array indexing bug, etc. The shim wraps the symptom; the
underlying bug only surfaces when the call goes direct.

## Recipe for re-probing a shim

1. Read the in-source comment for the original symptom.
2. Grep for Rust callers (all three i386 decorations).
3. If used: drop the C trampoline, declare the underlying Rust fn
   directly with the right ABI (`extern "system"` for NTAPI,
   `extern "C"` for cdecl, `extern "fastcall"` for fastcall — verify
   against actual symbol via `nm`).
4. Build all 3 arches, run `scan_ud2.py --check`. Unexpected ud2 in a
   named function is the smoking gun for either captures(none) (rare
   now) or ABI mismatch (more likely).
5. Run smoke. If ud2 is clean and smoke passes on i386 + amd64, the shim
   was a ghost. If something fails, the shim was hiding a real bug —
   diagnose with `verify_abi.py` first, then `nm`, then captures(none)
   defenses (write_volatile, opaque_ptr).

## Verification tools

| Tool | What it checks | Location |
|---|---|---|
| `verify_abi.py` | extern "C" vs extern "system" mismatch | `tools/rustconv/rustconv/verify_abi.py` |
| `verify_ffi.py` | ABI + params + C macros + flexible arrays | `tools/rustconv/rustconv/verify_ffi.py` |
| `verify_constants.py` | Constant values vs C headers | `tools/rustconv/rustconv/verify_constants.py` |
| `lint_kernel.py` | Unprotected output pointer writes | `tools/lint_kernel.py` |
| `scan_ud2.py` | LLVM miscompilation (ud2 in named functions) | `tools/scan_ud2.py` |

`nm` against the built binary is the fastest decoration check:

```
i686-w64-mingw32-nm build-i386/ntoskrnl/ntoskrnl.exe | grep <FunctionName>
```

`_X` = cdecl, `_X@N` = stdcall, `@X@N` = fastcall. If the Rust caller's
extern decl produces a different decoration than the symbol's actual
decoration, that's the bug.

## What's next

No fixed roadmap. Pick a target, re-probe, build, smoke. The pattern that
holds:

- The captures(none) class is empty (io); a similar re-probe round on
  mm and cm shims is likely to find more ghosts.
- The `~50 i386 register/V86/segment shims` in `ke_rust_shims.c` are the
  single biggest LOC item. Most are intentional-stay, but a focused audit
  with `core::arch::asm!` would probably move 10-20 of them.
- ~~The mm AVL refactor~~ — CLOSED 2026-05-06. Took the standalone-rlib
  path (mm-avl) rather than the generic-trait refactor of avltable, since
  it's lower-risk and matches the existing augment+isolated-rlib pattern.
- M8 (PFN ref-counting, 5 fns deferred from Phase 1) is structural — the
  PFN database design needs its own thinking.
- GLOBAL_ACCESSOR fns (~50) come for free as subsystems get translated;
  no separate effort needed.

Smoke should pass on i386 and amd64 after every shim removal. Shim
elimination work that breaks smoke is hiding a Rust bug that needs fixing
before the shim can come out.
