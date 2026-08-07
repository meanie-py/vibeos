#!/usr/bin/env python3
"""C sites where a `++`/`--` inside a CALL ARGUMENT is observed later in the same expression.

WHY THIS EXISTS
---------------
Rust has no `++`, so `f(*p++, …)` has to become two statements — and where the second one
lands is the whole meaning of the code. msvcrt's demangler reads

    if (!get_calling_convention(*sym->current++, …) || !demangle_datatype(sym, …))

The `++` is a side effect inside the FIRST ARGUMENT, so the cursor moves past the
calling-convention letter before `demangle_datatype` reads the same `sym`. The translation
hoisted that increment out of the condition and to after the whole `if`, and every function
signature in msvcrt.dll then parsed its RETURN TYPE from the calling-convention letter:
`void * (__cdecl*)(long)` demangled as ` & (__cdecl*)(void,long)`.

**Both forms advance the cursor exactly once.** Nothing leaks, nothing faults, no bounds or
lifetime check can fire, and the arithmetic is identical — only the ORDER differs. It shipped,
booted, and passed every other scanner in this directory; a differential row found it. That is
the argument for a scanner rather than a habit: this shape is invisible to everything that
looks at values instead of sequence.

The same file translates the SAME shape correctly forty lines later, as
`{ let c = *(*sym).current; (*sym).current = (*sym).current.offset(1); c }` — a block
expression that keeps the side effect where C put it. So this is an inconsistency to catch,
not a limitation to work around, and that block expression is the fix to reach for.

WHAT IT FLAGS, AND WHAT IT DELIBERATELY DOES NOT
------------------------------------------------
Flagged: the increment sits inside a call's argument list, and the same object is READ again
later in the same full expression (later argument, later operand of `||`/`&&`, or the other
side of a comparison).

Not flagged, because C sequences them and no translation can get them wrong by hoisting:

  * a `for` loop's third clause — `for (p = s; *p; p++)`. It is already a separate statement.
  * `if (++x >= limit)` where the increment is directly in the condition rather than inside a
    call. There is no second operand evaluated after a function call boundary.

Expressions are joined across source lines before matching. The msvcrt bug spanned four of
them — the `*sym->current++` on the first and the dependent `sym` on the fourth — so a
per-line scan misses precisely the shape worth finding.

USAGE
    scan_arg_side_effects.py            # report
    scan_arg_side_effects.py --lint     # baseline-compared, for vibeos-cli check
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# C trees that have, or are getting, a Rust twin. A directory absent here is not scanned,
# which is the honest scope: this compares nothing against the Rust, it points at C worth
# re-reading beside its translation.
SRC_DIRS = ["dll/win32/msvcrt", "sdk/lib/rtl", "sdk/lib/crt", "ntoskrnl", "win32ss"]

IDENT = r"[A-Za-z_][A-Za-z0-9_]*"
MEMBER = rf"{IDENT}(?:\s*(?:->|\.)\s*{IDENT})*"
INCDEC = re.compile(rf"(?:(\+\+|--)\s*({MEMBER}))|(?:({MEMBER})\s*(\+\+|--))")
CALL_OPEN = re.compile(rf"({IDENT})\s*\($")
KEYWORDS = {"if", "while", "for", "switch", "return", "sizeof", "defined"}


def strip_comments(text: str) -> str:
    """Blank out comments, keeping line count. Prose is full of the words this scanner
    matches — `pool.c` explains an algorithm with "size++ … space", and without this the
    report is mostly sentences."""
    out, i, n = [], 0, len(text)
    while i < n:
        if text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = n if end < 0 else end + 2
            out.append("".join(ch if ch == "\n" else " " for ch in text[i:end]))
            i = end
        elif text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end < 0 else end
            out.append(" " * (end - i))
            i = end
        elif text[i] in "\"'":
            q, j = text[i], i + 1
            while j < n and text[j] != q:
                j += 2 if text[j] == "\\" else 1
            out.append(" " * (min(j + 1, n) - i))
            i = min(j + 1, n)
        else:
            out.append(text[i])
            i += 1
    return "".join(out)


def logical_lines(text: str):
    """Yield (line number, joined text) for each parenthesis-balanced run."""
    text = strip_comments(text)
    buf, start, depth = "", 0, 0
    for i, raw in enumerate(text.splitlines(), 1):
        s = raw.strip()
        if not buf and (not s or s.startswith(("*", "/", "#"))):
            continue
        if not buf:
            start = i
        buf += " " + s
        depth += s.count("(") - s.count(")")
        # A macro definition or a preprocessor artifact can leave parentheses permanently
        # unbalanced, and without a cap one of those glues the rest of the file into a
        # single "expression" — which is where 80 of this scanner's first 100 findings came
        # from. Twenty lines is far past any real call expression; past that, resync.
        if depth <= 0 or i - start > 20:
            yield start, buf.strip()
            buf, depth = "", 0
    if buf:
        yield start, buf.strip()


def enclosing_call(expr: str, at: int) -> str | None:
    """The function whose argument list position `at` sits in, or None.

    Walks back to the innermost unclosed `(` and asks what precedes it. `if (`/`while (` and
    friends are not calls — that distinction is the difference between this scanner and 45
    false positives.
    """
    depth = 0
    for i in range(at - 1, -1, -1):
        c = expr[i]
        if c == ")":
            depth += 1
        elif c == "(":
            if depth == 0:
                m = CALL_OPEN.search(expr[: i + 1])
                if not m or m.group(1) in KEYWORDS:
                    return None
                return m.group(1)
            depth -= 1
    return None


def in_for_clause(expr: str, at: int) -> bool:
    """`for (init; cond; step)` — the step clause is already its own statement."""
    head = expr[:at]
    opens = list(re.finditer(r"\bfor\s*\(", head))
    if not opens:
        return False
    at_paren = opens[-1].end() - 1
    return head.count("(", at_paren) - head.count(")", at_paren) >= 1


def scan_file(path: Path) -> list[tuple[int, str, str, str]]:
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return []
    out = []
    for lineno, expr in logical_lines(text):
        for m in INCDEC.finditer(expr):
            obj = m.group(2) or m.group(3)
            if not obj:
                continue
            obj = obj.replace(" ", "")
            if in_for_clause(expr, m.start()):
                continue
            callee = enclosing_call(expr, m.start())
            if callee is None:
                continue
            base = re.split(r"->|\.", obj)[0]
            if not re.search(rf"\b{re.escape(base)}\b", expr[m.end() :]):
                continue
            out.append((lineno, obj, callee, expr[:200]))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lint", action="store_true", help="compare against the baseline")
    args = ap.parse_args()

    findings = []
    for d in SRC_DIRS:
        base = ROOT / d
        if not base.is_dir():
            continue
        for c in sorted(base.rglob("*.c")):
            for lineno, obj, callee, _expr in scan_file(c):
                findings.append(
                    f"{c.relative_to(ROOT)}:{lineno}: `{obj}` is incremented inside the "
                    f"argument list of `{callee}` and read again in the same expression — "
                    f"check the Rust performs the side effect BEFORE the dependent operand"
                )

    baseline_path = ROOT / "tools" / "lint-baseline" / "scan_arg_side_effects.txt"
    if args.lint:
        baseline = set()
        if baseline_path.is_file():
            baseline = {ln for ln in baseline_path.read_text().splitlines() if ln.strip()}
        new = [f for f in findings if f not in baseline]
        for f in new:
            print(f)
        print(f"\nscan_arg_side_effects: {len(new)} new, {len(findings)} total")
        return 1 if new else 0

    for f in findings:
        print(f)
    print(f"\nscan_arg_side_effects: {len(findings)} site(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
