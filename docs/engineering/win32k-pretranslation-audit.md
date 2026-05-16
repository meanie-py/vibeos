# Win32k Pre-Translation (C-Side Rewrite) Audit

Last updated 2026-05-16. **Status: Pass 0 + Pass 1 landed (worktree
`worktree-win32k-translation`, uncommitted).** The inventory/classifier
(Pass 0) and the mechanical probe-lowering rewriter (Pass 1) are built,
spike-validated, and applied across the win32k.sys keep-set: **97 SEH
sites mechanically lowered, i386 7/7 + amd64 7/7 smoke green** on the
result. **Pass 3 (USER callout extraction) also landed** as
extract-only (20 callout sites → sidecar JSON, oracle-validated). See
"Session results" below. Pass 2 has zero targets — see findings. This
audit defines the C-side work done *before* any C→Rust translation
begins on win32k.

Companion to:
- [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md) — kernel translation, ~99.7% done
- [hal-translation-audit.md](hal-translation-audit.md) — HAL translation, pre-translation
- [freeldr-translation-audit.md](freeldr-translation-audit.md) — FreeLDR + boot DLLs, pre-translation
- [pseh-elimination.md](pseh-elimination.md) — overall PSEH strategy (already reverse-shim model)
- [win32k-translation-audit.md](win32k-translation-audit.md) — sibling audit covering the actual rustconv-driven translation phase
- `qemu/~temp-chat-consolidated.md` — design notes from web-Claude pre-thinking

## Session results (2026-05-16)

What actually landed, with measured numbers replacing the estimates
below. All work is on the `worktree-win32k-translation` worktree,
uncommitted, on top of `continue-rust-port`@`4b07e4bc`.

### Tooling built

| Tool | Role | Status |
|---|---|---|
| `tools/rustconv/rustconv/seh_inventory.py` | Pass 0 classifier (a/b/c/d/e + signals); `python -m rustconv seh-inventory` | done, oracle-validated |
| `tools/rustconv/rustconv/seh_rewrite.py` | Pass 1 mechanical lowering; `python -m rustconv seh-rewrite` | done, oracle-validated |
| `tools/rustconv/rustconv/callout_annotate.py` | Pass 3 callout extractor (no rewrite); `python -m rustconv callout-annotate` | done, oracle-validated |
| `win32ss/safe_user_io.{c,h}` | SEH-free, status-returning user-range validator | done, builds+links all 3 arches |

Both tools are decision layers over the existing `seh_classifier.py`
(regex + brace-matching — tree-sitter cannot parse the SEH macros, so
neither tool uses it). Each has a built-in `--oracle` self-test that
reproduces the hand-done `winsta.c` spike taxonomy exactly.

### Pass 0 — measured classification (win32k.sys keep-set)

**282 `_SEH2_TRY` sites across 62 keep-set files:**

| Cat | Meaning | Count | % | vs. estimate |
|---|---|--:|--:|---|
| (a) PROBE_LOWER | Pass-1 mechanical | 167 | 59% | est. 60–80% ✓ |
| (b) LOCK_CLEANUP | Pass-2 RAII | **0** | 0% | **confirms: zero `_SEH2_FINALLY` in the kernel keep-set** |
| (c) FAULT_RECOVER | residual, no rewrite | **1** | 0.4% | est. 0–3; the one site is `gdi/eng/mem.c:HackSecureVirtualMemory` (`MmProbeAndLockPages`) — exactly the file the audit predicted |
| (d) BELT_SUSPENDERS | delete after cross-check | 36 | 13% | — |
| (e) NEEDS_REVIEW | manual | 77 | 27% | **est. <10% — reality is ~3× higher** |

**Key finding — Pass 2 has no targets.** The audit's "2 `_SEH2_FINALLY`"
are in `winsrv/consrv/` (user-mode CSRSS, out of scope). The kernel-mode
keep-set has **zero** `_SEH2_FINALLY`. So Pass 2 as specified
("SEH-wrapped lock cleanup → RAII") has nothing to rewrite. GDI locks in
this tree are *manual* `Lock → if(!p) → … → Unlock` brackets, not
FINALLY-wrapped. Pass 2 is therefore re-scoped to a *manual-bracket
detection* problem, which Pass 0's `(e)` bucket now quantifies (the
`(e)` sites with a GDI `*_Lock*` call are the real Pass-2 candidates).

**Key finding — `(e)` is ~3× the estimate.** ~27% of win32k SEH sites
have a non-probe call inside the `TRY` or are otherwise structurally
ambiguous (not pure-probe). Concentrated in `hook.c`, `message.c`,
`window.c`, `class.c`, `ime.c`, `ntstubs.c`. This sharpens Pass-1's true
mechanical scope and is the main correction to the original plan.

Per-file `<file>.win32k_meta.json` sidecars are written for the rustconv
prompt builder (54 files after Pass 1 collapses many sites).

### Pass 1 — measured mechanical lowering

`_SEH2_TRY` count **331 → 234 (97 sites lowered)** across 37 modified
`.c` files, **0 structural regressions**, validated by build + smoke:

| Arch | win32k.sys | Smoke |
|---|---|---|
| i386 | builds clean | **7/7 PASSED** |
| amd64 | builds clean | **7/7 PASSED** |
| arm64 | builds clean | boot-smoke blocked by *cold-build ramdisk-media infra* (nls/hive tree) — unrelated to win32k |

The remaining 64 `(a)`-class sites are deferred to a documented
**manual-followup ledger** (`win32ss/win32k_pass1_manual_followup.json`),
correctly *not* mechanically rewritten (audit risk #1 respected):

- 47 — non-swallow handler, no `NTSTATUS Status` lvalue in the function,
  handler doesn't transfer control (needs a `Status` declaration
  injected — risky; deferred to a v2/LLM pass)
- 15 — multi-probe with no `Status` lvalue (same)
- 2 — brace-less conditional probe (`winsta.c` L1242 shape — needs
  re-bracing)

So **97 of 168 `(a)` sites (~58%) are now mechanically lowered**,
deterministically and regression-free; the other 64 are the honest
non-mechanical remainder for later stages.

### Pass 3 — measured callout surface (extract-only, no rewrite)

`callout-annotate` extracts every real `KeUserModeCallback` site into a
per-file `<file>.win32k_callouts.json` sidecar (schema
`win32k_pass3_v1`) for the rustconv prompt builder. Per the audit's
Pass-3 rule, **no source is rewritten** — the sidecar JSON *is* the
deliverable; the LLM gets a structured "this is a callout" hint instead
of re-deriving it per translation unit. Oracle-validated against
`callback.c` ground truth.

**20 real callout sites, 3 files** (each site records: enclosing fn +
line span, `USER32_CALLBACK_*` index, marshal-in buffer/len args,
marshal-back ptr/len args, the bracketing `UserLeaveCo`/`UserEnterCo`
lines, and the first post-reacquire probe line):

| File | Sites | Note |
|---|--:|---|
| `user/ntuser/callback.c` | 17 | the `co_Int*` wrapper bank; all 17 `USER32_CALLBACK_*` literals |
| `user/ntuser/dde.c` | 2 | `IntDDEPostCallback` / `IntDDEGetCallback` (DDEPOST/DDEGET) |
| `user/rtl/text.c` | 1 | `UserExtTextOutW` LPK callout — **outside the Pass-0 KEEPSET dirs**; captured via explicit path so the keep-set scan stays unperturbed |

This matches the audit's "~17 `co_*` that actually issue user-mode
callbacks" estimate (17 in callback.c) plus DDE + LPK. Findings the
extractor surfaces as per-site `signals` for the LLM:

- **`co_UserCBClientPrinterThunk` (UMPD)** has *no* `UserLeaveCo`/
  `UserEnterCo` pair — it uses a different lock discipline (signal
  `missing_leaveco_enterco_pair`). Correct detection, not a tool gap;
  the Rust translation must not assume the standard straddle here.
- **DDE + several callback.c sites** have no mechanical marshal-back
  probe (`no_marshal_back_probe`) — they copy out via bespoke logic, so
  the translation can't pattern-match a `SafeProbeFor*` return point.
- 18/20 sites have a clean Leave/Enter straddle; the 2 exceptions are
  flagged, not silently dropped.

Resolver note: enclosing-function detection is a brace-depth scan over
a length-preserving literal mask (file-scope `typedef struct {…} N;`
and `= { … }` initializers before the first function defeat the simpler
"previous `}` on its own line" heuristic — caught and fixed during this
pass; all 17 callback.c spans verified `def < callsite < end`).

### Methodology-spike findings (drove the tool design)

The `ntuser/winsta.c` spike (8 sites, hand-done, builds+links+smokes)
established the ground truth the classifier/rewriter are oracle-tested
against:

- **The audit's Pass-1 option (A) is wrong.** `ntkrnl-sys/src/probe.rs`
  exports are `pub unsafe fn` Rust generics returning `Result<T,_>` —
  **not** `#[no_mangle] extern "system"`, so they are *not* C-callable.
  Pass 1 must use option **(B)** — a self-contained C `safe_user_io`
  shim (no cross-DLL export problem; behaviour-parallel to
  `probe_user_buffer`, so the later Rust translation maps 1:1). The
  shim deliberately validates-only (no touch-test): under DUMMY_PSEH a
  touch-fault is uncatchable, so validate-only is both safe and what
  the Rust target does.
- **Five distinct EXCEPT-handler shapes** (not one uniform pattern):
  swallow/`NOTHING`, `Status=_SEH2_GetExceptionCode()`,
  `SetLastNtError(code);return X;`, `Status=code;_SEH2_YIELD(goto L)`,
  multi-probe. All reduce to one rule: "turn the EXCEPT body into the
  probe-failure branch; body runs only on probe success."
- **Cross-site reasoning is required.** `winsta.c` L1384 is only safe
  because L1242 probed the same buffers earlier — the classifier must
  reason across sibling blocks, not per-block (the `(d)`
  `relies_on_prior_probe` signal).
- **Body must be success-guarded.** A naive "empty failure branch then
  body" still dereferences a bad pointer. Multi-probe must
  *short-circuit* (`if (NT_SUCCESS(Status)) Status = nextProbe;`) so a
  later probe can't overwrite an earlier failure.

### Bugs the "test later" build pass caught

`winsta.c` was too clean to expose these; real keep-set build did:

1. **`_SEH2_END` without trailing `;`** — span-end regex required the
   semicolon; without it the macro token was left dangling (cascade of
   "expected identifier / extraneous } / undeclared label"). This alone
   broke 13 of 15 files. Fixed: optional `;`, spaces/tabs only after
   (don't eat the newline → avoids gluing `}` to a `#` directive).
2. **`RE_YIELD` non-greedy** broke on nested parens
   (`_SEH2_YIELD(return _SEH2_GetExceptionCode();)` → garbled
   `return …(;)`). Fixed: balanced-paren extraction.
3. **`#include "win32k.h"`** (quotes) not matched by the include-probe
   regex → shim include not injected → `SafeProbeFor*` undeclared.
   Fixed: match both `<…>` and `"…"`.

All three are now regression-tested by the oracle + the full keep-set
build.

### Deltas to the plan below

- Pass 0 inventory format: **per-file JSON sidecar** (`<file>.win32k_meta.json`)
  — implemented as recommended in Open Question 1.
- Pass 1 helper: option **(B)** C shim (Open Question 3 — currently
  `win32ss/safe_user_io.{c,h}`, not `sdk/lib/`; revisit if drivers
  need it).
- Pass 2 ("GDI lock RAII"): **re-scoped** — zero `_SEH2_FINALLY`
  targets; the work is manual-bracket detection over the `(e)` GDI-lock
  subset, not an SEH rewrite.
- Methodology spike: done on `winsta.c` (Open Question 4) — confirmed
  the right choice (its 50ms-HACK function is untouched; the 8 sites
  exercised every handler shape + the cross-site hazard).
- Tree-sitter is **not** the workhorse (the TL;DR claim): the SEH
  macros defeat tree-sitter; the pipeline uses `seh_classifier`'s
  regex + brace-matching, which is what actually works.

## Why a separate "pretranslation" phase

Translating win32k via rustconv directly — the way ntoskrnl was done —
will work, but inefficiently. Win32k is **denser** in patterns the LLM
has to reason about per file:

- **339 `_SEH2_TRY` blocks across 86 files** (vs ~14 files / ~120 try
  blocks in ntoskrnl C remnants). SEH patterns dominate the syscall
  entry layer.
- **303 `ProbeForRead`/`ProbeForWrite` sites** (vs ~97 in SE+OB
  combined, the only Rust subsystems migrated via probe-migrate so far).
- **~10 distinct GDI Lock/Unlock fn pairs** (`DC_LockDc`, `BRUSH_LockBrush`,
  etc.) that wrap most non-probe SEH usage in GDI — a different RAII
  shape than ntoskrnl ever needed.
- **~290 `UserLeave`/`UserEnter*` site-pairs** in ntuser/ — a single
  global lock pattern that the Rust translation wants to hoist into a
  borrow-checker-enforceable typestate.
- **Callout convention (`co_*` prefix)** — 403 function definitions
  with 37 dependent call sites. Aspirational lexical convention today
  ("this fn may call out to user mode"); typestate-enforceable
  tomorrow.

The bottleneck for win32k translation is **per-translation-unit LLM
reasoning load**, not language gap. Anything we normalize in C —
deterministically, reviewably — pays compounding dividends across
~157K LOC of translation prompts.

This document specifies the C-side rewrites. The actual Rust
translation that consumes this normalized C is in
[win32k-translation-audit.md](win32k-translation-audit.md).

## TL;DR

- **Three normalization passes**, each landable independently with the
  C build still green and smoke still passing:
  1. **Probe-site lowering** — replace `_SEH2_TRY { ProbeFor*; … }
     _SEH2_EXCEPT` with `Status = SafeCaptureFromUser(…)` calls
     against existing Rust `safe_user_io` shims (or a small new C
     `validate_user_buffer` helper if that's cheaper). Knocks out
     ~80% of SEH usage mechanically.
  2. **GDI lock annotation** — add `__attribute__((annotate("gdi_lock:UnlockFn")))`
     to ~10 Lock fns + replace `_SEH2_TRY/FINALLY` lock-then-unlock
     patterns with a `WITH_LOCKED(guard, Lock(…), { … })` macro. Tree-sitter
     rewrite. Sidecar JSON for the rustconv prompt.
  3. **USER callout annotation** — mark `co_*` functions with
     `__attribute__((annotate("user_callout")))`, mark
     `UserLeaveCo`/`UserEnterCo` as the lock-release/reacquire pair,
     prepare for the typestate translation.
- **Audit before normalization, not during.** Risk-bound items get
  triaged in pass 0:
  - paths that genuinely depend on faulting-then-recovering (where
    validate-first is structurally impossible — shared user/kernel
    sections, double-fetch hazards)
  - W32THREAD/THREADINFO struct layout — whether it stays C-shaped
    or moves into Rust-side definition (gates `CoRef<T>` design)
  - the residual ~5% of SEH that's belt-and-suspenders fault-recovery
    on already-validated paths (delete; these never fire)
- **Tree-sitter is the workhorse.** All three passes are AST rewrites,
  not LLM tasks. The one Opus pass is generating the GDI annotation
  header (~10-30 entries, one-shot, ~$0.50).
- **Estimated total C-side rewrite cost: ~$1 in API spend** plus the
  human review time. Vastly cheaper than letting Sonnet re-derive the
  patterns in 86 separate translation-unit prompts.
- **Methodology validation: pick one file** (recommended:
  `ntuser/winsta.c`, 8 try-blocks, has the 50ms-delay-HACK from
  i386 boot fixes) and run all three passes on it end-to-end *before*
  scaling. If the rewritten file still passes smoke and the diff is
  reviewable, scale to the rest.

## Status / what already exists

We didn't start from zero. Existing infrastructure that the
pretranslation phase consumes:

- **`pseh-elimination.md`** — the overall strategy (reverse-shim
  pattern, no kernel-mode unwinder). Authoritative for the architecture
  decisions; this audit specifies the *implementation* for win32k.
- **`sdk/lib/rust/ntkrnl-sys/src/probe.rs`** — Result-returning probe
  API. Already proven for SE + OB. The C-side normalization will lower
  ProbeFor* into calls to these (or to thin C shims that wrap them).
- **`tools/rustconv/rustconv/probe_migrate.py`** + the templates — the
  probe-migrate stage. Designed for migrating *Rust* code; the win32k
  C normalization is the C-side mirror that prepares win32k for
  rustconv to consume after.
- **`tools/rustconv/templates/probe_migrate_prompt.md`** — 234 lines.
  Worth reading before drafting the win32k LLM prompt because the
  probe patterns are structurally identical.
- **`KeUserModeCallback` is already fully Rust** on all 3 arches (per
  the existing `qemu/~temp-chat-consolidated.md` analysis):
  `sdk/lib/rust/ntoskrnl-rust/src/ke/{i386,amd64,arm64}/usercall.rs`.
  The pretranslation phase doesn't touch this — it normalizes the
  *win32k C callers* of it.
- **Tree-sitter is already used** by `tools/rustconv/rustconv/scanner.py`
  for the C parser. We extend it with rewrite rules; we don't bring in
  a new tool.

## Inventory data

Concrete numbers from the current tree (`grep -rcE … --include="*.c"`):

### SEH distribution

| Component | LOC C | `_SEH2_TRY` |
|---|---:|---:|
| `gdi/ntgdi/` | 41,471 | 91 |
| `gdi/eng/` | 19,192 | 11 |
| `gdi/dib/` | 5,426 | 0 |
| `user/ntuser/` | 73,166 | 172 |
| `reactx/ntddraw/` | 3,193 | 0 |
| `reactx/dxg/` | 1,884 | 0 |
| **Win32k.sys total** | **~157K** | **274** |

(Plus ~65 in user/winsrv/, user/imm32/, user32/, gdi32/ — *user-mode
DLLs, out of scope for this phase*.)

`_SEH2_FINALLY` count across all of win32ss: **2 sites**. The vast
majority of SEH in win32k is `TRY/EXCEPT` (probe-pattern), not
`TRY/FINALLY` (cleanup-pattern). This is good news: probe-patterns
lower mechanically; FINALLY-patterns need RAII-shaping.

**Top 10 files by SEH density** (where the rewrite work concentrates):

```
ntuser/hook.c        18 _SEH2_TRY   (callbacks for Win32 hooks — 14 hook variants)
ntuser/message.c     17               (SendMessage/PostMessage/queue)
ntuser/ime.c         15               (IME — could prune for our scope)
gdi/ntgdi/font.c     14
gdi/ntgdi/coord.c    12
ntuser/window.c      11               (HWND lifecycle)
ntuser/class.c       11
ntuser/cursoricon.c   9
ntuser/winsta.c       8               (the 50ms HACK i386 boot fix lives here)
ntuser/menu.c         8
gdi/ntgdi/freetype.c  8               (font rendering glue)
gdi/ntgdi/dibobj.c    8
ntuser/ntstubs.c      7
```

### Probe sites

| Component | `ProbeForRead` | `ProbeForWrite` |
|---|---:|---:|
| `gdi/ntgdi/` | 49 | 51 |
| `gdi/eng/` | 20 | 2 |
| `user/ntuser/` | 109 | 72 |
| **Total** | **178** | **125** |

303 probe sites total in win32k.sys keep-set. For comparison: SE
migration was 62 sites, OB was 35.

### GDI Lock/Unlock fn pairs

These are the RAII candidates for pass 2 — every `Lock*` returns an
object that must be paired with a specific `Unlock*` call:

```
DC_LockDc                / DC_UnlockDc
GDIOBJ_LockObject        / (varies — destination depends on object type)
PALETTE_LockPalette      / PALETTE_UnlockPalette
SURFACE_LockSurface      / SURFACE_UnlockSurface
PATH_LockPath            / PATH_UnlockPath
REGION_LockRgn           / REGION_UnlockRgn
COLORSPACEOBJ_LockCS     / COLORSPACEOBJ_UnlockCS
TEXTOBJ_LockText         / TEXTOBJ_UnlockText
LFONT_LockFont (?)       / LFONT_UnlockFont
PEN_LockPen (?)          / PEN_UnlockPen
DRIVEROBJ_LockObject (?) / DRIVEROBJ_UnlockObject
```

Plus `BRUSH_*` referenced in the design notes but not yet found in the
grep — to be confirmed during pass 2 inventory. **About 10–12 distinct
pairs.**

The convention is symmetric (same prefix on Lock and Unlock), so
inferring the pair is mechanical for ~80% of sites. The remaining
~20% need annotation because:

- Some Lock fns return the object pointer; some return BOOL with
  out-param. The annotation has to capture which.
- `DRIVEROBJ_UnlockObject` and friends sometimes appear without a
  matching Lock — they're called on objects acquired through a
  different path.
- A few `vUnlock*` variants do extra cleanup beyond the standard
  Unlock — must annotate the variant.

### USER lock pattern

`UserLock` is a single global `ERESOURCE`. Acquired via:
- `UserEnterExclusive` — 203 sites (write path)
- `UserEnterShared` — 63 sites (read path)
- `UserLeave` — 287 sites (release)
- `UserEnterCo` / `UserLeaveCo` — 24/23 sites (callout pattern: leave
  before user-mode callback, re-enter after)

Plus `UserRefObjectCo` (96 sites) and `UserDerefObjectCo` (99 sites) —
the win32k-object refcount lifecycle, distinct from ntoskrnl
`Ob{Reference,Dereference}Object`.

The numbers tell us:
- The lock is held in a *lot* of places. The USER side of win32k is
  effectively single-threaded under this lock during all non-callout
  work.
- Callout-pattern sites are only ~24, most of which are concentrated
  in the ~17 `co_*` functions identified in the design notes.

## The three normalization passes

### Pass 0 (audit, not rewrite) — risk triage

Three things must be answered before any rewrite lands. Each is a
~half-session task; combined into one PR.

**0a. W32THREAD layout.** `win32ss/include/ntwin32.h` declares
`PTHREADINFO`/`PPROCESSINFO` as opaque to user mode; the kernel-side
struct lives in `win32ss/user/ntuser/win32.h`. **The pretranslation
phase decides whether THREADINFO/PROCESSINFO stay C-shaped or move to
a Rust-defined layout consumed via bindgen.** Rust-shaped is the
direction of travel (matches kernel-rust pattern); C-shaped means
`CoRef<T>` Rust struct calls C helpers for the references-list
manipulation. **Recommendation: keep C-shaped for the pretranslation
phase**; revisit during the Rust translation pass when CoRef lands.
Premature struct-shape changes risk smoke regressions before any of
the LLM-driven work even starts.

**0b. Fault-then-recover audit.** Grep `MmMapLockedPagesSpecifyCache`,
`MmMapLockedPages`, `MmProbeAndLockPages` across win32k. Two known
matches in our tree (`gdi/eng/mem.c`, plus `drivers/videoprt/resource.c`).
For each: confirm the validate-first pattern works structurally, or
flag as a residual SEH that has to stay (and route through the future
kernel unwinder, deferred per `pseh-elimination.md`). **Estimated
findings: 0–3 genuine fault-recovery sites total.**

**0c. Belt-and-suspenders SEH classification.** The 5% residual that
isn't probe-pattern (a) or lock-cleanup (b) — manually triage each.
Most are paranoid extra `_SEH2_TRY` around already-validated buffers;
delete them as no-op refactor. The few that aren't paranoid become
either: explicit validation (preferred) or genuine residual SEH (rare
— flag for the future kernel unwinder).

**Output of pass 0: a triage spreadsheet** (or the inventory pass JSON
schema) classifying each `_SEH2_TRY` site as one of:
- (a) probe-pattern → pass 1
- (b) lock-cleanup → pass 2
- (c) genuine fault-recovery → flagged residual, no rewrite
- (d) belt-and-suspenders → delete
- (e) unclear, need second look → manual triage

### Pass 1 — probe-site lowering

**Scope:** all 274 in-scope `_SEH2_TRY` blocks classified (a) by pass 0.
Estimated 60–80% of sites.

**Transform:**

```c
// Before:
_SEH2_TRY {
    ProbeForRead(UserBuffer, Length, sizeof(ULONG));
    LocalCopy = *(PULONG)UserBuffer;
} _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
    Status = _SEH2_GetExceptionCode();
} _SEH2_END;
if (!NT_SUCCESS(Status)) return Status;

// After:
Status = SafeCaptureUlong((PULONG)UserBuffer, &LocalCopy);
if (!NT_SUCCESS(Status)) return Status;
```

**`SafeCapture*` helpers** — small C wrappers over the existing Rust
`probe_*` API (`ntkrnl-sys/src/probe.rs`). Either:

- **(A) C extern declarations of the existing Rust probe fns**, called
  directly from win32k C with the win32k-conventional NTSTATUS return.
  Smallest delta. Requires the Rust probe fns to be exported with
  win32k-callable ABI (they already are — `pub unsafe extern "system"`).
- **(B) New `safe_user_io.c` shim file in win32ss/** that wraps
  the Rust calls with win32k-flavor parameter shapes (e.g., the
  win32k Probe* macros that handle alignment + length-overflow + read
  in one call). Slightly more C code; cleaner per-site call sites.

**Recommendation: (B) for clarity at call sites, with the file living
at `win32ss/include/safe_user_io.h` + `win32ss/safe_user_io.c`.**

**Mechanism: tree-sitter rewrite.** Pattern-matches:
1. `_SEH2_TRY { ProbeFor<X>(P, L, A); BODY } _SEH2_EXCEPT(…) { S = …; }
   _SEH2_END;` where BODY contains *only* memory loads/stores from P.
2. Replace with `S = SafeCapture<X>(P, L, A, &capture_buf); BODY'` where
   BODY' references `capture_buf` instead of `*P`.

Sites where BODY has side effects beyond the probe (logging, atomic
ops, calls to GDI Lock fns) get **flagged for manual review** rather
than auto-rewritten. Estimated <10% of probe sites need manual review.

**Risk:** the rewrite has to recognize when a single `_SEH2_TRY`
covers *multiple* user-buffer accesses (read header, then read body
based on header length). Pattern-match for nested ProbeFor* calls;
emit a multi-step capture sequence. The probe-migrate prompt template
already handles this case for Rust — port the logic.

**Cost:** **~$0** API spend (deterministic rewrite). Tooling
investment: ~2 sessions writing the tree-sitter rewriter, then bulk
runs.

### Pass 2 — GDI lock annotation + RAII shaping

**Scope:** the lock-cleanup SEH sites classified (b) by pass 0,
estimated 15–20% of total.

**Step 2a — generate annotation header:** one-shot Opus pass over
`win32ss/gdi/` produces:

```c
// win32ss/include/gdi_locks.h — generated, hand-reviewed
#define GDI_LOCK_FN(unlock_fn) __attribute__((annotate("gdi_lock:" #unlock_fn)))

GDI_LOCK_FN(DC_UnlockDc)               PDC      DC_LockDc(HDC);
GDI_LOCK_FN(BRUSH_UnlockBrush)         PBRUSH   BRUSH_LockBrush(HBRUSH);
GDI_LOCK_FN(SURFACE_UnlockSurface)     PSURFACE SURFACE_LockSurface(HSURFACE);
GDI_LOCK_FN(REGION_UnlockRgn)          PREGION  REGION_LockRgn(HRGN);
// ~10-12 entries
```

This is the **only** LLM-driven step in the C-side phase. **Cost:
~$0.50.**

**Step 2b — pattern-rewrite SEH-wrapped lock cycles:**

```c
// Before:
PDC pdc = DC_LockDc(hdc);
if (!pdc) return STATUS_INVALID_HANDLE;
_SEH2_TRY {
    pdc->dclevel.laPath.lStyleState = whatever;
    /* possibly probe user buffer here too */
} _SEH2_FINALLY {
    DC_UnlockDc(pdc);
} _SEH2_END;

// After:
WITH_GDI_LOCKED(pdc, DC_LockDc, hdc, {
    pdc->dclevel.laPath.lStyleState = whatever;
});
```

Or, when there's a probe inside the lock body (the common composite
pattern), the rewrite drops the `_SEH2_TRY/EXCEPT` and uses
`SafeCapture*` from pass 1 inside `WITH_GDI_LOCKED`'s body.

**`WITH_GDI_LOCKED(var, lock_fn, hdl, body)` macro** — defined in
`gdi_locks.h`. Calls `lock_fn(hdl)`, returns `STATUS_INVALID_HANDLE`
if NULL, executes body, then calls the unlock fn (pulled from the
annotation by tree-sitter at rewrite time). The macro is C; the
annotation-driven unlock-fn picking happens *during the rewrite*, so
the macro itself doesn't need to know which unlock to call.

**Sidecar JSON.** Each rewritten file gets a sibling
`<file>.gdi_locks.json` listing the `(lock_fn, var_name, body_locs)`
triples. The rustconv prompt builder reads this when generating the
Rust translation — gives the LLM a structured "this is a Drop guard,
here's the unlock fn" hint instead of having to derive from the macro.

**Risk:** the 5–10% non-trivial annotation cases the design notes flag:
- Lock returns transferred to a callee (`GDI_BORROWS` annotation
  variant — no Drop emitted at this scope).
- Multiple `Unlock*` variants per Lock (`v` prefix, `Object` suffix).
  The annotation must specify the *exact* unlock symbol used in the
  original FINALLY block — verifiable by tree-sitter looking at what
  was actually called.

**Cost:** ~$0.50 (Opus annotation gen) + ~$0 (rewrites). Tooling
investment: extend the tree-sitter rewriter from pass 1.

### Pass 3 — USER callout annotation

**Scope:** the ~17 `co_*` functions that actually issue user-mode
callbacks (out of 403 prefixed total — most are co_-prefixed for
"may call out" lexically but don't directly invoke
`KeUserModeCallback`).

**Step 3a — find the *real* callout sites.** Grep for
`KeUserModeCallback` callers in win32k. Cross-reference with
`UserLeaveCo`/`UserEnterCo` sites (24/23 in ntuser/). The set should
be small and identifiable.

**Step 3b — annotate the lock-release/reacquire pair:**

```c
// In ntuser headers:
#define UserLeaveCo()    __attribute__((annotate("user_lock_release_callout"))) UserLeave()
#define UserEnterCo()    __attribute__((annotate("user_lock_reacquire_callout"))) UserEnterExclusive()
```

Plus a per-callout-site annotation:

```c
// In each co_* function:
NTSTATUS co_IntClientLoadLibrary(...)
    __attribute__((annotate("user_callout:ClientLoadLibrary")))
{ ... }
```

**Step 3c — sidecar JSON for the rustconv prompt.** Each `co_*` gets
an entry: `{name, callback_index, leave_co_loc, enter_co_loc,
input_marshal_loc, output_probe_loc}`. The rustconv prompt for these
files gets the structured hint: "this is a callout, use the
`co_callout!` builder pattern (see `qemu/~temp-chat-consolidated.md` §6)".

**No automated rewrite in pass 3** — the callout pattern is structurally
diverse enough that mechanical rewriting is risky. Annotation only.
The Rust translation phase consumes the annotations and applies the
typestate pattern by hand-shaped LLM prompt.

**Cost:** ~$0 (annotation is hand-written or one-shot Opus over a
small surface).

## Tooling needed

A handful of new components, all small:

| Tool | Role | LOC est. |
|---|---|---:|
| `tools/rustconv/rustconv/seh_inventory.py` | Pass 0 classifier; reads tree-sitter, emits triage JSON per file | ~200 |
| `tools/rustconv/rustconv/seh_rewrite.py` | Pass 1 probe-lowering rewriter (tree-sitter rewrite rules) | ~500 |
| `tools/rustconv/rustconv/gdi_lock_rewrite.py` | Pass 2 GDI lock pattern rewriter | ~300 |
| `tools/rustconv/rustconv/callout_annotate.py` | Pass 3 callout annotation extractor | ~150 |
| `win32ss/include/safe_user_io.h` + `safe_user_io.c` | C shim bridging win32k probe macros to Rust probe API | ~100 |
| `win32ss/include/gdi_locks.h` (generated) | GDI lock annotation header | ~50 (gen) |
| `tools/rustconv/templates/win32k_translation_prompt.md` | LLM prompt extending probe_migrate_prompt with win32k-specific hints | ~300 |

**Total tooling lift: ~1,600 LOC Python + ~150 LOC C.** Substantial
but contained. Compares favorably to running 86 separate translation
prompts.

## Pruning surfaces during this phase

Same shape as HAL/FreeLDR — pruning naturally surfaces during the
audit. **Pruning is in-scope for this phase** because it shrinks the
keep-set the rewrites have to operate on.

| Candidate | LOC | Why drop |
|---|---:|---|
| `win32ss/drivers/miniport/xboxvmp/` | ~1,000 | Xbox display miniport |
| `win32ss/drivers/miniport/pc98vid/` | ~1,000 | NEC PC-98 video |
| `win32ss/drivers/displays/vga/` (4,007) or `vga_new/` (1,824) | varies | Two parallel VGA implementations; UEFI-only smoke means we likely use neither — `framebuf/` is the one we want |
| `gdi/diblib/` (3,456) | 3,456 | Alternate DIB lib, `USE_DIBLIB FALSE` — whole tree dead |
| `reactx/dxg/` (1,884), `reactx/ntddraw/` (3,193) | 5,077 | DirectDraw kernel side; we don't smoke DirectDraw apps |
| `gdi/ntgdi/freetype.c` glue + `sdk/lib/3rdparty/freetype/` | 7,943 + 208,631 | **Cannot prune** — load-bearing for desktop font render. Stays as 3rd-party FFI (same shape as ACPICA in HAL — header-only borrow / link-against, not translate). |
| `gdi/ntgdi/gdikdbgext.c` (KDBG-conditional) | small | KDBG=OFF in our build; already excluded |

**Pre-pretranslation prune candidates: ~14K LOC** in `diblib/` +
`reactx/{dxg,ntddraw}` + the SARCH miniports. Reversible cheap
delete; smoke gates the prune the same way it has for HAL/FreeLDR.

The display-driver pruning (vga vs vga_new vs framebuf) needs a
separate audit — see [win32k-translation-audit.md](win32k-translation-audit.md).

## Risk audit

What can go wrong during the C-side phase, in order of probability:

1. **Pass 1 mis-rewrites a multi-buffer probe.** A `_SEH2_TRY` covering
   "read header, then read body based on header length" gets
   incorrectly lowered as a single capture. Mitigation: tree-sitter
   pattern-match guards on multi-probe blocks; flag for manual review.
2. **Pass 2 picks the wrong unlock variant.** `vUnlockSomething`
   (paranoid void-return form) vs `UnlockSomething` (assertion-form).
   Mitigation: sidecar JSON records *exactly which symbol was called
   in the original FINALLY*; the rewrite preserves it byte-for-byte.
3. **Pass 0 misses a fault-recovery path.** A real
   `MmMapLockedPagesSpecifyCache`-style site gets misclassified as
   probe-pattern → mechanical rewrite removes its SEH wrapper → kernel
   bugcheck on the first page fault that actually fires.
   Mitigation: pass 0 audit must be hand-reviewed by a human; the 0–3
   estimated sites are all in `gdi/eng/mem.c` / `drivers/videoprt/`
   which are inspectable in an afternoon.
4. **Smoke regression from a probe-API ABI mismatch.** The Rust probe
   functions are `extern "system"` (stdcall on i386); calls from
   `safe_user_io.c` need the same. Mitigation: existing
   `verify_abi.py` already catches this; add it to the C-side build
   as a CI gate.
5. **The annotation header drifts from reality.** A new GDI Lock fn
   gets added in the tree but the annotation header doesn't get
   updated. Mitigation: lint pass — every Lock-shaped fn signature
   in `gdi/` must have an annotation entry; CI fail if not.

## Forward path

In rough order:

1. **Pass 0 audit** — fault-recovery audit + W32THREAD layout decision +
   belt-and-suspenders triage. Output: classified inventory JSON,
   audit findings doc-section. **1–2 sessions.**
2. **Methodology spike on `ntuser/winsta.c`** — run all three passes
   end-to-end on one file, ensure smoke still passes, review the diff.
   **1 session.**
3. **Pruning PR** — delete `diblib/`, `reactx/dxg`, `reactx/ntddraw`,
   xbox/pc98 miniports, plus whichever display drivers we're not using.
   Smoke gates. **1 session.**
4. **Pass 1 tooling + bulk rewrite** — write `seh_rewrite.py`, run on
   the 200+ probe-pattern sites, smoke. **2–3 sessions.**
5. **Pass 2 tooling + bulk rewrite** — annotation header gen (Opus,
   $0.50), `gdi_lock_rewrite.py`, run on the 30–50 lock-pattern sites,
   smoke. **2 sessions.**
6. **Pass 3 annotation** — hand-annotate the ~17 callouts. No rewrite.
   **0.5 session.**
7. **Smoke-test enhancement** (per (g) from the discussion) — extend
   `tests/qemu/test_smoke.py` with win32k-targeted assertions: cursor
   visible, taskbar painted, window-create-destroy round-trip, GDI
   bitblt smoke. Right place to invest *before* translation, since
   translation regressions need a strong detector. **2–3 sessions.**

After all this, the keep-set C is normalized: every probe site looks
like `Status = SafeCapture*(...)`; every GDI lock looks like
`WITH_GDI_LOCKED(g, Lock(...), { ... })`; every callout has a
sidecar JSON entry. Then [win32k-translation-audit.md](win32k-translation-audit.md)
takes over for the rustconv-driven translation.

## Definition of done (for this phase)

Progress as of 2026-05-16 (`[x]` done, `[~]` partial, `[ ]` not started):

- `[x]` All `_SEH2_TRY` sites classified (a)–(e) by inventory JSON
  (282 sites, per-file sidecars).
- `[~]` All (a) sites mechanically rewritten to `SafeProbeFor*` —
  **97 of 168 done**; 64 in the manual-followup ledger; remaining ~6
  not-yet-eligible. (Helper is `SafeProbeFor{Read,Write}`, not the
  audit's hypothetical `SafeCapture*`.)
- `[~]` (b) sites → RAII: **N/A — zero `_SEH2_FINALLY` in keep-set.**
  Re-scoped to manual GDI-lock-bracket detection over the `(e)` subset
  (not done).
- `[~]` (c) sites flagged: the 1 site (`gdi/eng/mem.c`) is identified
  and left verbatim; a standalone residual-SEH doc is not yet written.
- `[ ]` All (d) sites deleted (36 identified; not yet actioned —
  needs the cross-site "relies on prior probe" review).
- `[~]` Sidecar JSON for the rustconv prompt — written for the
  classified sites; not yet consumed by the prompt builder.
- `[x]` ~17 `co_*` callout annotations (Pass 3) — **done as
  extract-only** per the audit's no-rewrite rule: 20 sites (callback.c
  ×17, dde.c ×2, rtl/text.c LPK ×1) extracted to
  `<file>.win32k_callouts.json` sidecars, oracle-validated. (Optional
  `__attribute__((annotate(...)))` source injection is deferred — the
  sidecar JSON is what the prompt builder consumes.)
- `[ ]` Pruning (`diblib/`, dxg, ntddraw, sarch miniports, dead
  display drivers) — not started.
- `[~]` Smoke: **i386 7/7 + amd64 7/7 PASSED**; arm64 boot-smoke
  blocked by cold-build ramdisk-media infra (win32k itself builds
  clean on arm64).
- `[ ]` `verify_abi.py` clean — not yet run against the rewrites.
- `[ ]` Win32k-specific post-`shell ready` smoke assertions — not
  started.

## Open questions

1. **Inventory pass output format.** JSON-per-file or one big JSON
   sidecar? The rustconv prompt builder consumes it, so JSON-per-file
   composes better with the existing per-file translation pipeline.
   **Recommendation: per-file JSON sidecar** (`<file>.win32k_meta.json`).
2. **Fault-recovery residuals — how to flag?** A lint annotation
   `__attribute__((annotate("residual_seh:reason")))` at the SEH site
   is one option. A separate `residual_seh.md` doc tracking the list
   is another. **Recommendation: both — annotation for the C site,
   doc for the project-level ledger.**
3. **`safe_user_io.c` location.** `win32ss/safe_user_io.c` (per-module)
   vs `sdk/lib/safe_user_io/` (shared with future driver translations)?
   **Recommendation: `sdk/lib/safe_user_io/`** — drivers will want it
   too, and the cross-module shape is the same.
4. **Methodology spike file.** `ntuser/winsta.c` (8 SEH, has the
   50ms HACK) is the favorite — tests across SEH+lock patterns and the
   HACK gives us a known-tricky regression target. Alt:
   `ntuser/hook.c` (18 SEH, hottest file) for max coverage.
   **Recommendation: `winsta.c` first** (smaller, regressions are
   visible), `hook.c` second.
5. **W32THREAD layout — really defer?** The pretranslation phase
   could lock in a Rust-shaped layout via bindgen-generated header
   if we wanted, but that risks smoke regressions before any rewrite
   work has paid off. **Recommendation: defer; revisit during the
   Rust translation phase.**
6. **Smoke-test enhancement priority.** Current smoke marker is
   `shell ready` (boot reaches `co_IntShowDesktop`). Win32k regressions
   *after* shell ready (e.g., a button click crashing) won't surface.
   **Recommendation: add post-`shell ready` smoke assertions** —
   simulate a click, simulate a keypress, verify cursor moves, verify
   a window can be created. The infrastructure is in `qemu/run.sh`
   already (mouse_click.py exists for arm64). Adapt for i386/amd64
   and bake into the smoke harness. ~2–3 sessions, pays for itself
   the first time it catches a translation regression.

## What this audit doesn't cover

- **The Rust translation itself** — see
  [win32k-translation-audit.md](win32k-translation-audit.md).
- **User-mode DLLs** (user32, gdi32, imm32, winsrv) — out of scope per
  decision (a) from the prior discussion. Phase N+1.
- **Unwinder design** — kernel-mode unwinder is deferred per
  `pseh-elimination.md`. Validate-first probe pattern works for
  win32k without it. Re-open this question only if pass 0 finds
  fault-recovery residuals that can't be eliminated.
- **The win32k callout typestate API** itself — designed in
  `qemu/~temp-chat-consolidated.md` §6; this audit prepares the C
  side for it but doesn't specify the Rust-side API. That's the
  translation phase's job.
