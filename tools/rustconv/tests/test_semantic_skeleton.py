from rustconv.scanner import CPP_LANG
from rustconv.semantic_skeleton import RUST_LANG, compare_skeletons, extract_skeleton


def test_equivalent_cross_language_control_shape_scores_highly():
    c = "int f(int x) { int y = 0; if (x) { y = Call(x); return y; } return y; }"
    rust = "unsafe fn f(x: i32) -> i32 { let mut y = 0; if x != 0 { y = Call(x); return y; } y }"
    result = compare_skeletons(extract_skeleton(c, CPP_LANG), extract_skeleton(rust, RUST_LANG))
    assert result["control_exact"]
    assert result["score"] >= 0.9


def test_dropped_branch_lowers_control_similarity():
    c = "int f(int x) { if (x) return Call(x); return 0; }"
    rust = "unsafe fn f(x: i32) -> i32 { Call(x) }"
    result = compare_skeletons(extract_skeleton(c, CPP_LANG), extract_skeleton(rust, RUST_LANG))
    assert not result["control_exact"]
    assert result["control_score"] == 0.0


def test_cross_language_loop_jumps_are_normalized():
    c = "void f(int x) { while (x) { if (x == 1) break; x--; continue; } }"
    rust = "unsafe fn f(mut x: i32) { while x != 0 { if x == 1 { break; } x -= 1; continue; } }"
    result = compare_skeletons(extract_skeleton(c, CPP_LANG), extract_skeleton(rust, RUST_LANG))
    assert result["control_exact"]
