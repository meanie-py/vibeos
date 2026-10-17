#!/usr/bin/env python3
"""
scan_atl_slot_cast.py — atl_rust's `CAtlList<T>::GetNext(&mut pos)` returns `*mut T`, a
pointer to the element's SLOT in the list node, not the element.

    C++:          CConsoleWnd *console = (CConsoleWnd*)m_ViewList.GetNext(pos);
    translated:   let console = m_ViewList.GetNext(&mut pos) as *mut CConsoleWnd;   <-- slot
    faithful:     let console = *m_ViewList.GetNext(&mut pos);

For a list of pointers, T is itself `*mut X`, so the cast turns a `*mut *mut X` into a
`*mut X`: rustc allows any pointer-to-pointer `as`, nothing warns, and the result points
into the list node. mmc_new shipped it at five sites on 2026-10-17 (a re-bought body):
"close all views" sent WM_CLOSE to the object's address read as an HWND, then freed the
root node under the still-open views; every Save ran SaveView on list-node memory.

The C++ cast is how ATL's `E&` return reads in C; in Rust the deref IS the conversion, so a
cast of GetNext/GetPrev's result is never the faithful spelling. Baseline is EMPTY.

Usage:
    python3 tools/scan_atl_slot_cast.py [ROOT..]   # report; exit 1 if any finding
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
SLOT_CAST_RE = re.compile(r"\.Get(?:Next|Prev)\(\s*&mut\s+[^()]*\)\s+as\s+\*(?:mut|const)\b")
SKIP_PATH_RE = re.compile(r"(/target/|/\.responses/|/atlclasses-rust/)")


def main(argv: list[str] | None = None) -> int:
    roots = [Path(a).resolve() for a in (sys.argv[1:] if argv is None else argv)] or [RUST_ROOT]
    found = 0
    for root in roots:
        for f in sorted(source_files(root)):
            rel = f.relative_to(REPO).as_posix() if f.is_relative_to(REPO) else str(f)
            if SKIP_PATH_RE.search("/" + rel):
                continue
            for n, line in enumerate(f.read_text(errors="replace").splitlines(), 1):
                if SLOT_CAST_RE.search(line) and "nolint: atl-slot-cast" not in line:
                    print(f"{rel}:{n}: GetNext/GetPrev result cast -- deref the slot instead")
                    found += 1
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
