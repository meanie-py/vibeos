"""scan_let_shadow: a C assignment translated as a shadowing `let _x`."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_let_shadow import scan_text

SRC = """
unsafe fn enumerate(left: *mut V) -> u8 {
    let DiskCount: u8 = 0;
    let mut c: u32;
    let mut _c: u32;                       // right: the C's own second variable
    while more() {
        let _DiskCount = DiskCount + 1;    // finding: C `DiskCount++`
        let _left = &mut tmp;              // finding: a parameter re-pointed
        let _hr = f(); // nolint: let-shadow (test)
        let s = "let _DiskCount = 1;";     // right: inside a string
        // let _DiskCount = 2;             right: a comment
    }
    DiskCount
}
unsafe fn other() {
    if x {
        let _DiskCount = 3;                // right: no DiskCount in THIS function
    }
}
"""


def test_only_nested_shadows_of_live_bindings():
    assert [line for line, _ in scan_text(SRC)] == [7, 8]


def test_tree_is_clean():
    import subprocess

    r = subprocess.run(
        [sys.executable, str(Path(__file__).resolve().parent / "scan_let_shadow.py"), "--lint"],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, r.stdout
