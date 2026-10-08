from rustconv.loop_step import scan

BAD = """
fn f() {
    i = 0;
    while i < n {
        if p.is_null() {
            continue;
        }
        match s {
            A => { i = i.wrapping_add(1); continue; }
            _ => {}
        }
        g();
        i = i.wrapping_add(1);
    }
}
"""

GOOD_NESTED = """
fn f() {
    while i < n {
        let mut j = 0;
        while j < m {
            if x { j += 1; continue; }
            j += 1;
        }
        for k in 0..3 { if y { continue; } }
        loop { if z { continue 'outer; } break; }
        if q { i = i.wrapping_add(1); continue; }
        // continue; in a comment
        let s = "continue;";
        i = i.wrapping_add(1);
    }
}
"""


def test_flags_the_continue_that_skips_the_step():
    hits = scan(BAD)
    assert [(h.line, h.var) for h in hits] == [(6, "i")]


def test_nested_loops_strings_and_stepped_continues_are_clean():
    assert scan(GOOD_NESTED) == []


def test_while_without_trailing_step_is_ignored():
    assert scan("fn f() { while x { if y { continue; } g(); } }") == []


def test_a_different_write_to_the_loop_variable_advances_it():
    src = "fn f() { while va < end { if big { va = (va | M) + 1; continue; } g(); va += P; } }"
    assert scan(src) == []
    assert len(scan("fn f() { while va < end { if big { other = 1; continue; } va += P; } }")) == 1
