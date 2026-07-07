# Lane: wtf do we do about DUMMY_PSEH?

Work in this worktree (`/home/curi/Projects/os-playground/vibeos-pseh`, branch
`dummy-pseh-question`). Read `AGENTS.md` first for the house rules.

**This is an open question, not a task with a known answer.** There is no
predetermined "do X." Your job is to figure out what we *should* do — or to prove
that the status quo is actually fine and say why. Reach a recommendation you'd stake
a design review on; the deliverable is a decision + its justification, not a
particular implementation. If the honest answer is "leave it, here's the evidence
it's safe," that's a legitimate and valuable outcome.

## The situation

Windows uses structured exception handling (`__try`/`__except`/`__finally`)
pervasively — kernel, drivers, win32k, user DLLs — to catch hardware faults
(access violations probing user memory, guard pages, divide-by-zero) and software
exceptions (`ExRaiseStatus`). Our llvm-mingw / Clang 19 toolchain can't drive any of
the existing PSEH implementations, so we compile with **DUMMY_PSEH**: the `_SEH2_TRY`
body runs, but every `_SEH2_EXCEPT` handler is **dead code**. A fault that should be
caught-and-handled instead propagates unhandled → bugcheck / killed process.

It is everywhere: **~99 C files** with `_SEH2_TRY` and **~248 Rust files** with
SEH-shaped constructs. Today we cope by:
- **Validate-first** — pre-check every user pointer instead of fault-and-catch
  (the Rust `probe.rs` API exists solely for this).
- **Defensive swallowing** — `KiUserExceptionDispatcher` patches, `RtlRaiseException`
  swallow fixes, etc. (see the bug history in `pseh-elimination.md`).
- **`panic = "abort"` everywhere** — no unwinder, no `catch_unwind`; the only
  RIP-rewrite recovery that exists is the one-off `KiCheckForSListFault`.

So the OS boots and runs, but a whole class of "catch the fault and recover" behavior
is silently absent, and every new user-buffer path has to remember to pre-validate or
it's a latent bugcheck.

## Why this is a real question and not obvious

There are several coherent directions, none clearly correct, and they trade off
differently:

- **Double down on validate-first** and *prove it sufficient* — i.e., establish that
  every place real Windows relied on fault-and-catch, we can (and do) pre-validate,
  and build tooling/lints so a missing validation can't ship. Cheapest; but is it
  actually complete, or does it leave holes we can't enumerate?
- **Build the trap-to-panic bridge** already designed (but deferred, not started) in
  `kernel-exception-handling.md`: hook the CPU trap chain into Rust unwind/`Result`
  semantics (`try_user_access`, per-thread probe-region stack, trap-frame rewriter,
  catch-unwind) across all three arches. Most powerful; also the most work and risk,
  and it fights `panic = "abort"`.
- **Generalize an existing partial** — `PSEH_CLANG` (setjmp/longjmp) works on i386
  only; could it be made real cross-arch? What does amd64/arm64 SEH actually require?
- **Re-test the toolchain assumption** — the "Clang can't do SEH" verdict is a few
  years old. What's the current state of LLVM's SEH (`__C_specific_handler`, i386
  `_except_handler3`, ARM64 `.pdata`/`.xdata`) on our llvm-mingw? Is DUMMY_PSEH still
  forced, or is it now a choice?
- **Something narrower** — maybe the right move is a small, targeted primitive
  (a single blessed `try_user_access`) that covers the ~handful of paths that truly
  need fault-and-catch, and everything else stays validate-first.

## Suggested first move (not a mandate): scope the real exposure

Before picking a direction, it's probably worth knowing *how much this actually
matters at runtime*. Of the ~99+248 SEH sites, how many have a `_SEH2_EXCEPT` that is
a genuine fault-catch the system depends on, vs. a defensive no-op that never fires on
a correct path? A boot/stress coverage pass (which SEH regions are entered, which
faults would actually be swallowed) would turn "everywhere, scary" into a bounded,
rankable list — and might itself decide the answer. But how you scope it is yours to
choose.

## The corpus (read before deciding — the analysis mostly exists)

- `docs/engineering/pseh-elimination.md` — the problem, the PSEH3/PSEH_CLANG/DUMMY
  landscape, the validate-first strategy, and the bug history DUMMY_PSEH caused.
- `docs/engineering/kernel-exception-handling.md` — the deferred, fully-designed
  trap-to-panic bridge (`try_user_access`, probe-region stack, trap-frame rewriter).
  Read *why it's not built yet* as carefully as what it is.
- `docs/engineering/rust-kernel-patterns.md` — the probe/validate patterns in use.
- `sdk/lib/rust/ntoskrnl-rust/src/**/probe.rs` and the `probe-migrate` pipeline stage
  — the current validate-first machinery.
- `sdk/lib/rust/pseh-rust/src/lib.rs` — the translated SEH runtime exports (dead
  bodies under DUMMY_PSEH, kept so the driver-facing `_except_handler*` ABI resolves).
- `sdk/lib/pseh/include/pseh/pseh2.h:179` — the actual gate
  (`_USE_DUMMY_PSEH || __arm__ || _M_AMD64`).
- `KiCheckForSListFault` (`ke/i386/traphdlr.rs`) — the one existing RIP-rewrite
  recovery, as a shape for what generic recovery could look like.
- The `USE_DUMMY_PSEH` / `DUMMY_PSEH` gate in the cmake + `*-sys/wrapper*.h`.

## Definition of done

A written recommendation (land it as a doc, e.g. `docs/engineering/dummy-pseh-verdict.md`)
that a maintainer could act on: what to do, why that over the alternatives, what it
costs, what it buys, and — if it's a build — a concrete first increment. Evidence over
assertion: if you claim validate-first is sufficient, show the exposure analysis that
backs it; if you claim we need the bridge, show the paths that can't be validated
away. No code change is required to succeed here, though a small proof-of-concept or a
scoping tool may be the most persuasive artifact.

Think holistically and categorically. The prize is clarity on a problem the codebase
has been routing around for a long time.
