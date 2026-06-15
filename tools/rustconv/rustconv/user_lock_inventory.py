"""Win32k Pre-Translation Pass 2b — USER critical-section bracket detection.

The largest unaddressed fragile family in win32k is the USER critical
section (618 call sites). Unlike GDI locks (Pass 2) it has **no return
value and no bound variable** — it is a global two-level nested lock
factored into a named primitive pair (`win32ss/user/ntuser/ntuser.c`):

    UserEnterExclusive: ASSERT_NOGDILOCKS(); KeEnterCriticalRegion();
                        ExAcquireResourceExclusiveLite(&UserLock,TRUE);
                        gptiCurrent = PsGetCurrentThreadWin32Thread();
    UserEnterShared:    KeEnterCriticalRegion();
                        ExAcquireResourceSharedLite(&UserLock,TRUE);
    UserLeave:          ExReleaseResourceLite(&UserLock);   <- inverse
                        KeLeaveCriticalRegion();                order

Because ReactOS already factored the KeEnterCriticalRegion+ERESOURCE
nest (and its inverse release order) *into* `UserEnter*`/`UserLeave`,
the **primitive** translates once as plain direct calls — exactly the
proven `ExAcquireTimeRefreshLock`/`ExReleaseTimeRefreshLock` pattern in
ntoskrnl-rust `ex/time.rs` (enter-CR-then-acquire / release-then-leave).
The ~618 **caller** sites are then a *single-level* RAII problem,
structurally identical to Pass 2: pair acquire (`UserEnter{Exclusive,
Shared}`) with release (`UserLeave`) and classify the bracket shape so
the translator emits a `UserExclusiveGuard`/`UserSharedGuard` (a
`PfnLockGuard`-shaped guard) with `drop()` at the *exact* original
`UserLeave()` site — never an implicit function-end drop (the #1
ntoskrnl RAII-translation failure mode; see system_prompt.md:823).

Like Pass 2/3 this is **extract-only** — no source rewrite. It emits a
per-file `<file>.c.win32k_userlocks.json` sidecar (schema
`win32k_pass2b_v1`) consumed by the prompt builder.

Shapes (what the translator is told to do):
- CLEAN_BRACKET — single unconditional enter at function top, single
  leave, single exit. → one whole-fn guard, RAII Drop is correct.
- MULTI_LEAVE   — one enter at top, ≥2 leaves (multi-exit). → still
  ONE guard; its Drop replaces *every* textual UserLeave. Do NOT emit
  manual leaves.
- REENTRANT     — leave-then-reenter inside the fn (the callout
  straddle: drop the lock across a user-mode callback, reacquire).
  → DO NOT use a whole-fn guard; scoped drop/reacquire at the marked
  lines. Cross-reference the Pass-3 callouts sidecar.
- BORROWED      — enter here, no leave in this fn (released by caller /
  transferred). → no Drop here; model the lock state as caller-owned.
- REVIEW        — conditional/late enter, or a return path that skips
  the leave. → keep explicit enter/leave + a manual-review TODO.

CLI: `python -m rustconv user-locks [path] [--json] [--write-sidecars]
      [--oracle]`  (mirrors `gdi-locks`).
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from .callout_annotate import (
    _enclosing_func_span,
    _line_of,
    _mask_literals,
)
from .seh_inventory import _in_keepset

RE_ENTER_EXCL = re.compile(r"\bUserEnterExclusive\s*\(")
RE_ENTER_SHARED = re.compile(r"\bUserEnterShared\s*\(")
RE_LEAVE = re.compile(r"\bUserLeave\s*\(")
RE_RETURN = re.compile(r"\breturn\b")

# Control-flow that, appearing between the function's opening brace and
# the first enter, means the acquire is *conditional* (not a clean
# prologue-level whole-function bracket).
RE_BRANCH = re.compile(r"\b(if|for|while|switch|case|do|goto)\b")

# The primitive definitions themselves — never a *use* site.
_PRIMITIVES = {"UserEnterExclusive", "UserEnterShared", "UserLeave"}


@dataclass
class UserLockSite:
    function: str
    func_line: int
    func_end_line: int
    mode: str  # exclusive | shared | mixed
    enter_line: int  # first acquire
    enter_at_top: bool  # unconditional, prologue-level acquire
    n_enters: int
    leave_lines: list[int]  # every UserLeave() in the fn body
    n_returns: int  # exit paths in the fn body
    shape: str  # CLEAN_BRACKET | MULTI_LEAVE |
    # REENTRANT | BORROWED | REVIEW
    signals: list[str] = field(default_factory=list)


def _line_offsets(text: str) -> list[int]:
    """Char offset of the start of each 1-based line (index 0 unused)."""
    offs = [0, 0]
    for ln in text.splitlines(keepends=True):
        offs.append(offs[-1] + len(ln))
    return offs


def extract_file(path: Path) -> list[UserLockSite]:
    text = path.read_text(errors="replace")
    masked = _mask_literals(text)
    loff = _line_offsets(text)
    sites: list[UserLockSite] = []

    # Group every enter by its enclosing function so a fn is reported once
    # even with multiple enters (re-entrant straddle).
    seen_fns: set[tuple[str, int]] = set()

    for m in list(RE_ENTER_EXCL.finditer(masked)) + list(RE_ENTER_SHARED.finditer(masked)):
        fn_name, fn_lo, fn_hi = _enclosing_func_span(text, m.start())
        if fn_name in _PRIMITIVES:
            continue  # the UserEnter*/UserLeave definition itself
        key = (fn_name, fn_lo)
        if key in seen_fns:
            continue
        seen_fns.add(key)

        lo_off = loff[fn_lo] if fn_lo < len(loff) else 0
        hi_off = loff[min(fn_hi + 1, len(loff) - 1)]
        body = masked[lo_off:hi_off]

        excl = [lo_off + x.start() for x in RE_ENTER_EXCL.finditer(body)]
        shrd = [lo_off + x.start() for x in RE_ENTER_SHARED.finditer(body)]
        enters = sorted(excl + shrd)
        leaves = [lo_off + x.start() for x in RE_LEAVE.finditer(body)]
        returns = [lo_off + x.start() for x in RE_RETURN.finditer(body)]

        if not enters:
            continue
        mode = "mixed" if excl and shrd else "exclusive" if excl else "shared"
        first_enter = enters[0]

        # enter_at_top: between the function body's `{` and the first
        # enter there is no branching keyword and no nested `{` — i.e.
        # the acquire is an unconditional prologue statement.
        body_open = body.find("{")
        prologue = (
            body[body_open + 1 : first_enter - lo_off]
            if body_open >= 0
            else body[: first_enter - lo_off]
        )
        enter_at_top = RE_BRANCH.search(prologue) is None and "{" not in prologue

        leave_lines = [_line_of(text, o) for o in leaves]
        signals: list[str] = []

        if not leaves:
            shape = "BORROWED"
            signals.append("no_leave_in_fn")
        elif not enter_at_top:
            # conditional/late acquire — including independent per-arm
            # enter/leave brackets in sibling switch cases. Must precede
            # the reentrant test (those aren't a single-lock straddle).
            shape = "REVIEW"
            signals.append("conditional_enter")
        elif any(lv < en for lv in leaves for en in enters if en > lv):
            # enter@top, then a leave precedes a later enter → the lock
            # is dropped and reacquired on one linear path (callout).
            shape = "REENTRANT"
            signals.append("leave_then_reenter:callout_straddle_see_pass3")
        else:
            # returns that sit *after* the acquire but *before* the first
            # release are lock-leak paths the translator must close.
            first_leave = leaves[0]
            leak = [r for r in returns if first_enter < r < first_leave]
            if leak:
                shape = "REVIEW"
                signals.append("return_before_first_leave")
            elif len(enters) == 1 and len(leaves) == 1:
                shape = "CLEAN_BRACKET"
            elif len(enters) == 1 and len(leaves) >= 2:
                shape = "MULTI_LEAVE"
                signals.append("multi_exit_release")
            else:
                shape = "REVIEW"
        if mode == "mixed":
            signals.append("mixed_excl_shared")

        sites.append(
            UserLockSite(
                function=fn_name,
                func_line=fn_lo,
                func_end_line=fn_hi,
                mode=mode,
                enter_line=_line_of(text, first_enter),
                enter_at_top=enter_at_top,
                n_enters=len(enters),
                leave_lines=leave_lines,
                n_returns=len(returns),
                shape=shape,
                signals=signals,
            )
        )
    return sites


def extract_keepset(win32ss_root: Path) -> dict[Path, list[UserLockSite]]:
    result: dict[Path, list[UserLockSite]] = {}
    for c in sorted(win32ss_root.rglob("*.c")):
        if not _in_keepset(c):
            continue
        sites = extract_file(c)
        if sites:
            result[c] = sites
    return result


def write_sidecar(path: Path, sites: list[UserLockSite]) -> Path:
    out = path.with_suffix(path.suffix + ".win32k_userlocks.json")
    payload = {
        "source": str(path),
        "schema": "win32k_pass2b_v1",
        "count": len(sites),
        "sites": [s.__dict__ for s in sites],
    }
    out.write_text(json.dumps(payload, indent=2))
    return out


def _shape_counts(inv: dict[Path, list[UserLockSite]]) -> dict[str, int]:
    c: dict[str, int] = {}
    for ss in inv.values():
        for s in ss:
            c[s.shape] = c.get(s.shape, 0) + 1
    return c


def print_summary(inv: dict[Path, list[UserLockSite]]) -> None:
    n = sum(len(v) for v in inv.values())
    print(f"\n{'=' * 64}")
    print(f"Win32k Pass 2b — {n} USER critsec brackets across {len(inv)} keep-set files")
    print(f"{'=' * 64}")
    for shape in ("CLEAN_BRACKET", "MULTI_LEAVE", "REENTRANT", "BORROWED", "REVIEW"):
        k = _shape_counts(inv).get(shape, 0)
        pct = (k * 100 / n) if n else 0
        print(f"  {shape:<14} {k:>4}  {pct:5.1f}%")
    modes: dict[str, int] = {}
    for ss in inv.values():
        for s in ss:
            modes[s.mode] = modes.get(s.mode, 0) + 1
    print("\n  Modes:", ", ".join(f"{k}={v}" for k, v in sorted(modes.items())))
    top = sorted(inv.items(), key=lambda x: -len(x[1]))[:10]
    print("\n  Hotspot files:")
    for p, ss in top:
        rel = p.as_posix().split("/win32ss/")[-1]
        print(f"    {len(ss):>3}  {rel}")


# --- simplecall.c ground-truth oracle ---------------------------------------
# Observed directly 2026-05-16:
#   NtUserCallNoParam — UserEnterExclusive() L65 (unconditional, after a
#     decl + TRACE), single UserLeave() L143, single return L145 → the
#     canonical CLEAN_BRACKET (exclusive, enter_at_top).
#   NtUserCallHwnd — UserEnterShared() L724 *inside a switch case*, with
#     sibling switch arms that `return` without ever entering the lock →
#     must classify REVIEW (conditional_enter), never a whole-fn guard.
def run_oracle(simplecall_path: Path) -> int:
    sites = extract_file(simplecall_path)
    print(f"simplecall.c: {len(sites)} USER critsec brackets")
    ok = True
    by = {s.function: s for s in sites}

    nop = by.get("NtUserCallNoParam")
    if nop and nop.mode == "exclusive" and nop.enter_at_top and nop.shape == "CLEAN_BRACKET":
        print("  NtUserCallNoParam = CLEAN_BRACKET(excl, top)  [OK]")
    else:
        got = f"{nop.mode}/{nop.enter_at_top}/{nop.shape}" if nop else "absent"
        print(f"  NtUserCallNoParam: expect exclusive/True/CLEAN_BRACKET, got {got}  [MISMATCH]")
        ok = False

    hwnd = by.get("NtUserCallHwnd")
    if hwnd and not hwnd.enter_at_top and hwnd.shape == "REVIEW":
        print("  NtUserCallHwnd = REVIEW (conditional enter)  [OK]")
    else:
        got = f"{hwnd.enter_at_top}/{hwnd.shape}" if hwnd else "absent"
        print(f"  NtUserCallHwnd: expect False/REVIEW, got {got}  [MISMATCH]")
        ok = False

    if len(sites) >= 4:
        print(f"  bracket count {len(sites)} (>=4)  [OK]")
    else:
        print(f"  bracket count: expect >=4, got {len(sites)}  [MISMATCH]")
        ok = False

    selfdef = [s for s in sites if s.function in _PRIMITIVES]
    if selfdef:
        print(f"  {len(selfdef)} primitive-definition false positives  [MISMATCH]")
        ok = False
    else:
        print("  no primitive-definition false positives  [OK]")

    print("ORACLE PASS" if ok else "ORACLE FAIL")
    return 0 if ok else 1
