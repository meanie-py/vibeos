"""scan_list_walk_from_head: a list walk whose cursor starts at the head it stops at."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_list_walk_from_head import scan_text

# rpcss IrotIsRunning as translated (2026-10-11): zero iterations, every lookup missed.
BAD = """
unsafe extern "C" fn IrotIsRunning(data: *const MonikerComparisonData) -> HRESULT {
    rot_entry = (core::ptr::addr_of_mut!(RunningObjectTable) as *mut list as *mut u8)
        .sub(core::mem::offset_of!(rot_entry, entry)) as *const rot_entry;
    while core::ptr::addr_of!((*rot_entry).entry)
        != core::ptr::addr_of_mut!(RunningObjectTable) as *const list
    {
        rot_entry = ((*rot_entry).entry.next as *mut u8).sub(8) as *const rot_entry;
    }
}
"""

GOOD = BAD.replace(
    "(core::ptr::addr_of_mut!(RunningObjectTable) as *mut list as *mut u8)",
    "(RunningObjectTable.next as *mut u8)",
)


def test_walk_from_head_is_flagged():
    assert scan_text(BAD) == [(5, "IrotIsRunning", "rot_entry")]


def test_walk_from_head_next_is_clean():
    assert scan_text(GOOD) == []


def test_nolint_and_comments():
    nolint = "    while core // nolint: list-walk-from-head (x)"
    assert scan_text(BAD.replace("    while core", nolint, 1)) == []
    commented = "// e = CONTAINING_RECORD!(addr_of_mut!(H), T, f);\nwhile e != addr_of_mut!(H) {}"
    assert scan_text(commented) == []
