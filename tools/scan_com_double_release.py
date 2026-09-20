#!/usr/bin/env python3
"""
scan_com_double_release.py — a C++ `CComPtr` local lowered to a raw pointer plus
an RAII guard, whose explicit `Release()` calls were ALSO kept.

The translation of a C++ COM method renders `CComPtr<IFoo> p;` as a raw
`*mut IFoo` plus `let _p_cpp_com_guard = CppComPtrGuard::new(addr_of_mut!(p));`.
That guard IS the destructor: on every return path it releases and NULLS the
slot. So an explicit `p->Release()` left in the body is a SECOND release unless
the body nulls `p` right after — and the guard then reads `lpVtbl` out of a
freed block and calls through it.

Why a scanner and not a rule. CDefView's `FillList` shipped exactly this on three
exit paths. arm64 died at once (`PC=0x1600000004000000`, explorer killed, no
desktop); amd64 and i386 ran the identical code and passed the 91-test
conformance sweep three times in a row, because the freed block still held a
plausible pointer. A defect that two of three arches certify as green is not
something a reviewer will reliably catch, and no boot gate will either.

The model is deliberately narrow, which is what makes it precise:

  * GUARD:   `CppComPtrGuard::new(core::ptr::addr_of_mut!(NAME))`
  * RELEASE: `((*(*NAME).lpVtbl).Release)(NAME)`
  * EXEMPT:  a release followed within two lines by an ASSIGNMENT to NAME — either
             `NAME = core::ptr::null_mut()` (the guard then sees NULL and does
             nothing, which is how five other locals in CDefView are written) or a
             hand-over-hand walk like `popup = next`, where the guard goes on to own
             the new node. Testing for NULL alone reported that correct loop.

Guard and release are matched by ENCLOSING FUNCTION, never by name alone: the
same local name recurs across methods, and in CDefView `pPB`/`pStream` in
`SaveViewState` are raw locals the C++ really does release explicitly, with no
guard at all. Matching on the name alone reports those and tells you to delete
correct code.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent

_FN = re.compile(r"^\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"[A-Za-z-]+\"\s+)?fn\s+(\w+)")
_GUARD = re.compile(r"CppComPtrGuard::new\(\s*core::ptr::addr_of_mut!\(\s*(\w+)\s*\)\s*\)")
_RELEASE = re.compile(r"\(\*\(\*(\w+)\)\.lpVtbl\)\.Release\)\(\s*\1\s*[,)]")


def _reassigned_near(lines: list[str], idx: int, name: str) -> bool:
    """The slot is ASSIGNED again within two lines of the release.

    Nulling (`p = core::ptr::null_mut()`) is the common case, but a hand-over-hand
    walk is the same idiom and equally correct: `CMenuDeskBar::_IsSubMenuParent`
    releases the current node and immediately does `popup = next`, so the guard
    owns the NEW node rather than a freed one. Requiring a NULL specifically
    reported that correct loop, so the test is assignment, not null.
    """
    pat = re.compile(r"(?<![=!<>])\b" + re.escape(name) + r"\s*=(?!=)")
    return any(pat.search(lines[j]) for j in range(idx + 1, min(idx + 3, len(lines))))


def scan_source(text: str) -> list[tuple[int, str, str]]:
    lines = text.split("\n")
    fn = "<file scope>"
    guards: set[tuple[str, str]] = set()
    releases: list[tuple[int, str, str]] = []
    for i, line in enumerate(lines):
        m = _FN.match(line)
        if m:
            fn = m.group(1)
        g = _GUARD.search(line)
        if g:
            guards.add((g.group(1), fn))
        r = _RELEASE.search(line)
        if r and not _reassigned_near(lines, i, r.group(1)):
            releases.append((i + 1, r.group(1), fn))
    return [
        (
            ln,
            "com-double-release",
            f"`{name}` in `{f}` is CppComPtrGuard-owned; this explicit Release "
            f"is a second one (null the slot, or delete it)",
        )
        for ln, name, f in releases
        if (name, f) in guards
    ]


def _iter_files(roots: list[Path]):
    """`source_walk`, never `rglob` + a filter.

    `rglob` finishes descending into `sdk/lib/rust/target` BEFORE any filter sees a
    path, so cargo deleting a directory mid-walk kills the scanner with a traceback
    that the gate reports as findings-less lint failure. `scan_unpruned_walk` gates
    this, and it caught this very file on its first run.
    """
    for root in roots:
        if root.is_file():
            yield root
        elif root.is_dir():
            yield from source_files(root)


def main(argv: list[str]) -> int:
    lint = "--lint" in argv
    args = [a for a in argv if not a.startswith("--")]
    roots = [Path(a) for a in args] or [REPO / "sdk" / "lib" / "rust"]

    total = 0
    for f in sorted(set(_iter_files(roots))):
        try:
            findings = scan_source(f.read_text(encoding="utf-8", errors="replace"))
        except OSError:
            continue
        try:
            rel = f.resolve().relative_to(REPO)
        except ValueError:
            rel = f
        for ln, kind, msg in findings:
            total += 1
            print(f"{rel}:{ln}: {kind}" if lint else f"{rel}:{ln}: [{kind}] {msg}")

    if not lint:
        print(f"\n{total} COM double-release finding(s)")
    return 1 if total else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
