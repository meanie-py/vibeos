#!/usr/bin/env python3
"""
scan_dowhile_continue.py — a `continue` that skips a translated do-while condition.

THE CLASS. C `do { ... continue; ... } while (cond);` jumps to the CONDITION on
`continue`. The translator renders a do-while as

    loop { ...; if !(cond) { break; } }

where a `continue` jumps back to the TOP and never evaluates `cond`. The loop then
spins forever, re-processes the same element, or runs past where the C stopped.
Four shipped (2026-10): kernel32 GetTempFileNameW (a CSRSS ID of 0 looped
forever), syssetup RegisterTypeLibraries (one bad INF line: endless FatalError
dialogs), win32k co_IntGetPeekMessage (re-peeked a DDE message whose data was
already freed: use-after-free + double free), user32 ScrollTrackScrollBar (a
filtered button-up kept tracking). Thirteen more sites had been fixed by hand,
one at a time, by re-running the advance + condition before each `continue`.

WHAT IS FLAGGED: a Rust `loop` whose last statement is `if <cond> { break; }` and
which holds a `continue` aimed at that loop (bare, or labelled with its label;
`continue`s inside nested loops are theirs, not this one's). That is the shape of
a do-while with a skippable condition. It is ALSO the shape of a faithful
`for (;;) { ... if (x) break; }`, so a finding is a question, not a verdict: read
the C. Faithful sites are baselined (tools/lint-baseline/scan_dowhile_continue.txt)
or carry `// nolint: dowhile-continue (why)` on the `loop {` line.

Findings are keyed by file and enclosing function, not line, so they survive edits.

THE SIBLING IN A MACRO. A `for (init; cond; step)` lowered to `while cond { body; step }`
has the same defect: C runs `step` on `continue`, Rust jumps past it. Written by hand that
is ambiguous (a C `while` with a trailing update is faithful), but inside a `macro_rules!`
it is not: a loop macro that splices its caller's `$body` and THEN advances is a lowered
`for` by construction, and every caller that writes `continue` spins. kernel32's and
ole32's wine `LIST_FOR_EACH_ENTRY!` were both this shape (2026-10-08) -- kernel32's
UpdateResource hung on the second resource it added -- and ole32 had already hand-rolled
one walk around the macro rather than fix it. So a macro whose `while`/`loop` has anything
after a `$metavar` statement is flagged with no C reading needed; the fix is to advance at
the TOP of the loop, behind a first-pass flag, or before the body when `next` is cached.

Usage:
    python3 tools/scan_dowhile_continue.py [--lint] [paths...]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
STRING_RE = re.compile(r'b?"(?:\\.|[^"\\])*"')
CHAR_RE = re.compile(r"b?'(?:\\.|[^'\\])'")
LOOP_RE = re.compile(r"(?:'(\w+)\s*:\s*)?\bloop\s*\{")
NESTED_LOOP_RE = re.compile(r"\b(?:loop|while|for)\b")
TRAILING_BREAK_IF_RE = re.compile(r"\bif\b[^{};]*\{\s*break\s*;\s*\}\s*$")
FN_RE = re.compile(r"\bfn\s+(\w+)")
NOLINT = "nolint: dowhile-continue"


def _blank(m: re.Match) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


def _match_brace(text: str, open_idx: int) -> int:
    depth = 0
    for i in range(open_idx, len(text)):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
    return -1


def scan_text(raw: str) -> list[tuple[int, str, int]]:
    """(line, enclosing fn, number of skipping continues) per suspicious loop."""
    text = COMMENT_RE.sub(_blank, raw)
    text = STRING_RE.sub(_blank, text)
    text = CHAR_RE.sub(_blank, text)
    raw_lines = raw.splitlines()
    out = []
    for m in LOOP_RE.finditer(text):
        label = m.group(1)
        open_idx = text.index("{", m.start())
        close_idx = _match_brace(text, open_idx)
        if close_idx < 0:
            continue
        body = text[open_idx + 1 : close_idx]
        if not TRAILING_BREAK_IF_RE.search(body.rstrip()):
            continue
        # Mask nested loop bodies: an unlabelled `continue` there belongs to them.
        masked = list(body)
        for n in NESTED_LOOP_RE.finditer(body):
            k = body.find("{", n.end())
            if k < 0:
                continue
            e = _match_brace(body, k)
            if e < 0:
                continue
            for q in range(k, e + 1):
                masked[q] = " "
        own = re.findall(r"\bcontinue\b(?:\s*'(\w+))?\s*;", "".join(masked))
        count = sum(1 for lab in own if lab in ("", label))
        if label:
            # A labelled continue from inside a nested loop still targets this one.
            count += len(re.findall(r"\bcontinue\s*'" + re.escape(label) + r"\s*;", body)) - sum(
                1 for lab in own if lab == label
            )
        if not count:
            continue
        line = text.count("\n", 0, m.start()) + 1
        if NOLINT in raw_lines[line - 1]:
            continue
        fns = FN_RE.findall(text, 0, m.start())
        out.append((line, fns[-1] if fns else "?", count))
    return out


MACRO_RE = re.compile(r"\bmacro_rules!\s*(\w+)\s*\{")
MACRO_LOOP_RE = re.compile(r"\b(?:while\b[^{;]*|loop\s*)\{")
SPLICE_RE = re.compile(r"(?m)^\s*\$(\w+)\s*;?\s*$")


def scan_macros(raw: str) -> list[tuple[int, str]]:
    """(line, macro name) for each loop macro that advances AFTER splicing `$body`."""
    text = COMMENT_RE.sub(_blank, raw)
    text = STRING_RE.sub(_blank, text)
    text = CHAR_RE.sub(_blank, text)
    out = []
    for m in MACRO_RE.finditer(text):
        end = _match_brace(text, m.end() - 1)
        if end < 0:
            continue
        macro = text[m.end() : end]
        for lm in MACRO_LOOP_RE.finditer(macro):
            o = lm.end() - 1
            c = _match_brace(macro, o)
            if c < 0:
                continue
            body = macro[o + 1 : c]
            # Only the loop's OWN statements: blank nested blocks (an `if` holding the
            # advance at the top is the fix, and must not read as "after the body").
            flat = list(body)
            i = 0
            while i < len(body):
                if body[i] == "{":
                    e = _match_brace(body, i)
                    if e < 0:
                        break
                    for q in range(i + 1, e):
                        flat[q] = " "
                    i = e + 1
                else:
                    i += 1
            flat_s = "".join(flat)
            splice = None
            for sm in SPLICE_RE.finditer(flat_s):
                splice = sm
            if splice and flat_s[splice.end() :].strip():
                out.append((text.count("\n", 0, m.start()) + 1, m.group(1)))
                break
    return out


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", type=Path)
    ap.add_argument(
        "--lint", action="store_true", help="terse, line-free output; exit 1 on findings"
    )
    args = ap.parse_args()

    files = []
    for p in args.paths or [RUST_ROOT]:
        files += [p] if p.is_file() else sorted(source_files(p))
    findings = []
    for f in files:
        for line, fn, count in scan_text(f.read_text(errors="replace")):
            rel = f.resolve().relative_to(REPO) if f.resolve().is_relative_to(REPO) else f
            findings.append((str(rel), line, fn, count))

    macro_findings = []
    for f in files:
        for line, name in scan_macros(f.read_text(errors="replace")):
            rel = f.resolve().relative_to(REPO) if f.resolve().is_relative_to(REPO) else f
            macro_findings.append((str(rel), line, name))
    for rel, line, name in macro_findings:
        if args.lint:
            print(f"{rel}: macro {name}: loop advances after the spliced body; `continue` skips it")
        else:
            print(
                f"{rel}:{line}: macro {name}: a `continue` in the caller's body skips the advance"
            )

    for rel, line, fn, count in findings:
        if args.lint:
            print(f"{rel}: fn {fn}: loop ends in a do-while condition that `continue` skips")
        else:
            print(
                f"{rel}:{line}: fn {fn}: {count} continue(s) skip the trailing `if .. {{ break; }}`"
            )
    return 1 if findings or macro_findings else 0


if __name__ == "__main__":
    sys.exit(main())
