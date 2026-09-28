#!/usr/bin/env python3
"""scan_bindgen_padding_index.py -- an index into a bindgen `__bindgen_padding_N` array.

bindgen emits `__bindgen_padding_N` for a member it cannot name (the anonymous HEAD at the
front of THRDESKHEAD/PROCDESKHEAD), and the ELEMENT type of that array is its own choice:
`[u64; 3]` from one bindgen, `[u8; 24]` from the next. Code that indexes it to reach the
hidden member -- `(*pWnd).head.__bindgen_padding_0[2] as PTHREADINFO` -- is reading a
position whose meaning the generator can change, and a regenerated island keeps compiling
because `u8 as *mut T` is a legal cast.

It happened: the 2026-09-27 win32k-sys regeneration turned [u64; 3] into [u8; 24], `[2]`
became one byte of the window handle, user32's IntCallMessageProc saw a thread mismatch on
every window, and every posted message it dispatched -- WM_CLOSE included -- was dropped
as ERROR_MESSAGE_SYNC_ONLY. No app closed on any arch, and every gate stayed green.

Reach the member through the typed struct instead (`(*(p as *const _THROBJHEAD)).pti`,
`(*(p as *const _HEAD)).h`). Taking the padding's ADDRESS (`addr_of!(x.__bindgen_padding_0)`,
`offset_of!(T, __bindgen_padding_0)`) is layout-stable and is not flagged.

Output: one line per site. Exit 1 on findings.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"

INDEX_RE = re.compile(r"__bindgen_padding_\d+\s*\[")
# Generated islands declare the arrays; only hand-written code indexes them.
GENERATED_RE = re.compile(r"(^|/)(hal_)?bindgen_[a-z0-9_]+\.rs$")


def scan_text(text: str) -> list[int]:
    """1-based line numbers that index a bindgen padding array (comments skipped)."""
    hits = []
    for n, line in enumerate(text.splitlines(), 1):
        code = line.split("//", 1)[0]
        if INDEX_RE.search(code):
            hits.append(n)
    return hits


def main() -> int:
    findings = []
    for path in sorted(source_files(RUST_ROOT)):
        rel = path.relative_to(REPO).as_posix()
        if "/target/" in rel or GENERATED_RE.search(rel):
            continue
        for n in scan_text(path.read_text(errors="replace")):
            findings.append(
                f"{rel}:{n}: [PADDING_INDEX] indexes a bindgen padding array -- read the "
                "member through its typed struct (_HEAD / _THROBJHEAD)"
            )
    for line in findings:
        print(line)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
