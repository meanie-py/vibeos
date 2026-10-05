"""scan_gmem_flags: HEAP_* flags handed to Global/LocalAlloc."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_gmem_flags import scan_text

SRC = """
h = GlobalAlloc(HEAP_ZERO_MEMORY, n as SIZE_T);           // finding
h = GlobalAlloc(0 /* GMEM_FIXED */, n);                   // right
h = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, f(a, b));  // right
h = LocalAlloc(0x8, n);                                   // finding: not an LMEM bit
h = GlobalReAlloc(h, n, 0x42);                            // right: MOVEABLE|ZEROINIT
h = GlobalReAlloc(h, n,
    HEAP_REALLOC_IN_PLACE_ONLY);                          // finding (spans lines)
h = LocalAlloc(HEAP_ZERO_MEMORY, n); // nolint: gmem-flags (test)
pub unsafe extern "system" fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL {
"""


def test_only_the_foreign_flags():
    assert [line for line, _ in scan_text(SRC)] == [2, 5, 7]


def test_tree_is_clean():
    import subprocess

    r = subprocess.run(
        [sys.executable, str(Path(__file__).resolve().parent / "scan_gmem_flags.py"), "--lint"],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, r.stdout
