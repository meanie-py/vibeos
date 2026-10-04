"""scan_byteptr_stride: byte arithmetic through a pointer to a byte pointer."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_byteptr_stride import scan_text

SRC = """
let a = (rec as *mut PBYTE)
    .offset(off);                                      // finding (spans lines)
let b = (rec as *mut BYTE).offset(off);                // right: BYTE*
let c = (rec as PBYTE).offset(off);                    // right: PBYTE itself
let d = (tbl as *const PULONG).add(i);                 // an array of pointers: other type
let e = (argv as *mut PUCHAR).add(1); // nolint: byteptr-stride (argv-style table)
"""


def test_only_the_pointer_to_byte_pointer_arithmetic():
    assert [(line, t, m) for line, t, m in scan_text(SRC)] == [(2, "PBYTE", "offset")]


def test_tree_is_clean():
    import subprocess

    r = subprocess.run(
        [sys.executable, str(Path(__file__).resolve().parent / "scan_byteptr_stride.py"), "--lint"],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, r.stdout
