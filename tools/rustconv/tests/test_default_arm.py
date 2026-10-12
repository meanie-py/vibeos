"""C's `default:` is tried last wherever it sits; a Rust match arm is tried in order."""

from __future__ import annotations

from rustconv.default_arm import rewrite

RUST = """fn f(n: i32) {
    match n {
        // case 0 / default: month first
        0 | _ => {
            mdy("{");
        }
        1 => {
            dmy();
        }
        2 => ymd(),
    }
}
"""


def test_case_zero_default_moves_after_the_specific_cases():
    # net's ParseDate, cmd's date/dir: every locale was read as month-first.
    new, moved = rewrite(RUST)
    assert moved == 1
    assert new.index("1 =>") < new.index("2 => ymd(),") < new.index("0 | _ =>")
    assert "// case 0 / default" in new.split("2 => ymd(),")[1]
    assert new.count("{") == RUST.count("{") and new.count("}") == RUST.count("}")


def test_last_wildcard_guard_and_nested_are_left_alone():
    already = "match n { 1 => a(), _ => b() }"
    assert rewrite(already) == (already, 0)
    guarded = "match n { _ if x => a(), 1 => b(), _ => c() }"
    assert rewrite(guarded)[1] == 0


def test_single_line_match_without_trailing_comma():
    new, moved = rewrite("match n { _ => a(), 1 => b() }")
    assert moved == 1
    assert new == "match n { 1 => b(), _ => a(), }"
