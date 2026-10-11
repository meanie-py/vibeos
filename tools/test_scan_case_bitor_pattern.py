"""scan_case_bitor_pattern: a C `case A|B:` translated as a Rust or-pattern."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_case_bitor_pattern import c_bitor_labels, scan_text

C = """
switch (action) {
case TVGN_CARET|TVSI_NOSINGLEEXPAND:   /* one value: the OR */
case (A_FLAG | B_FLAG | C_FLAG):
    break;
case PLAIN:
case OTHER:
    break;
}
"""


def test_c_labels_are_or_expressions_only():
    labels = c_bitor_labels(C)
    assert frozenset({"TVGN_CARET", "TVSI_NOSINGLEEXPAND"}) in labels
    assert frozenset({"A_FLAG", "B_FLAG", "C_FLAG"}) in labels
    assert frozenset({"PLAIN", "OTHER"}) not in labels


def test_or_pattern_arm_and_matches_are_flagged():
    rust = """
match action {
    TVGN_CARET | TVSI_NOSINGLEEXPAND => {}
    crate::A_FLAG
        | B_FLAG
        | C_FLAG => {}
    _ => {}
}
let b = matches!(action, TVGN_CARET | TVSI_NOSINGLEEXPAND);
"""
    hits = scan_text(rust, c_bitor_labels(C))
    assert [line for line, _ in hits] == [3, 4, 9]


def test_guard_case_list_and_nolint_are_clean():
    rust = """
match action {
    x if x == TVGN_CARET | TVSI_NOSINGLEEXPAND => {}
    PLAIN | OTHER => {}
    TVGN_CARET | TVSI_NOSINGLEEXPAND => {} // nolint: case-bitor-pattern (C lists both)
    // TVGN_CARET | TVSI_NOSINGLEEXPAND => {}
    _ => {}
}
let m = TVGN_CARET | TVSI_NOSINGLEEXPAND;
"""
    assert scan_text(rust, c_bitor_labels(C)) == []
