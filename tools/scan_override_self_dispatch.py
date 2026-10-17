#!/usr/bin/env python3
"""
scan_override_self_dispatch.py — an override that calls its own virtual through a base.

    C++:          void SmoothDrawTool::OnEndDraw(BOOL bCancel)
                  { m_bDrawing = FALSE; ToolBase::OnEndDraw(bCancel); }
    translated:   pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {
                      (*this_).m_bDrawing = FALSE; (*this_).__base0.OnEndDraw(bCancel); }

A QUALIFIED base call is non-virtual in C++. In a `port cppapp` crate, `__base0.OnEndDraw` is
the VIRTUAL dispatcher: it reads the object's vtable, which is the derived class's, and lands
on the very override making the call. Infinite recursion, a stack overflow the first time
the path runs, and it compiles clean. The base's own body is `__base0.OnEndDraw__body(..)`.

Found 2026-10-16, 22 sites in shipped crates: cicero's CUIFWindow/CUIFShadow/CUIFMenu
Initialize (3), mspaint's drawing tools (9 -- every stroke's OnEndDraw), msutb (9) and
msctfime (1). Smoke and appstorm reached none of them. The generator no longer writes the
shape (`cpp_app._qualified_base_calls`); this keeps it out of hand edits and old crates.

Usage:
    python3 tools/scan_override_self_dispatch.py            # report
    python3 tools/scan_override_self_dispatch.py --lint     # terse (CLI baseline gate)

Exit 1 if any finding.
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

SKIP_PATH_RE = re.compile(r"(/target/|/\.responses/)")
NOLINT_RE = re.compile(r"//\s*nolint(?::\s*override-self-dispatch\b|\s*$)")
BODY_FN_RE = re.compile(r"\bfn (\w+)__body\s*(?:<[^>]*>)?\s*\(")


def _body_span(text: str, after: int) -> tuple[int, int] | None:
    """The `{...}` of the fn whose signature starts before `after`."""
    depth, i = 1, after
    while i < len(text) and depth:  # the parameter list
        depth += (text[i] == "(") - (text[i] == ")")
        i += 1
    brace = text.find("{", i)
    if brace < 0:
        return None
    depth, j = 0, brace
    while j < len(text):
        depth += (text[j] == "{") - (text[j] == "}")
        if depth == 0:
            return brace, j
        j += 1
    return None


def scan_text(text: str) -> list[tuple[int, str]]:
    findings = []
    for m in BODY_FN_RE.finditer(text):
        name = m.group(1)
        span = _body_span(text, m.end())
        if span is None:
            continue
        lo, hi = span
        # The override's OWN object: `(*this_).__base0.M(` or `self.__base0.M(`. The same
        # call on another object (`(*(*this_).m_pToolTip).__base0.Initialize()`) is an
        # ordinary virtual call on a member.
        call = re.compile(
            rf"(?:\(\s*\*\s*this_\s*\)|(?<![\w.)])self|(?<![\w.)])this_)\s*"
            rf"\.(?:__base\d+\.)+{re.escape(name)}\s*\("
        )
        for c in call.finditer(text, lo, hi):
            line_start = text.rfind("\n", 0, c.start()) + 1
            line_end = text.find("\n", c.start())
            line = text[line_start : line_end if line_end >= 0 else len(text)]
            if NOLINT_RE.search(line):
                continue
            findings.append((text.count("\n", 0, c.start()) + 1, line.strip()))
    return findings


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    lint = "--lint" in sys.argv
    roots = [Path(a) for a in args] if args else [RUST_ROOT]

    files: list[Path] = []
    for root in roots:
        if root.is_file():
            files.append(root)
        elif root.is_dir():
            files.extend(source_files(root))

    findings = []
    for f in sorted(set(files)):
        if SKIP_PATH_RE.search(str(f)):
            continue
        text = f.read_text(errors="replace")
        if "__body" not in text:
            continue
        for ln, src in scan_text(text):
            try:
                rel = f.relative_to(REPO)
            except ValueError:
                rel = f
            findings.append((str(rel), ln, src))

    for rel, ln, src in findings:
        if lint:
            print(f"{rel}:{ln}: override-self-dispatch")
        else:
            print(f"{rel}:{ln}: an override calls its own virtual via a base: infinite recursion")
            print(f"    {src}")

    if not lint:
        print(f"\n{len(findings)} site(s)")
        if findings:
            print("fix: C++ `Base::M(..)` is non-virtual -- call `.__baseN.M__body(..)`.")
            print("     if the dispatch really is wanted: // nolint: override-self-dispatch")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
