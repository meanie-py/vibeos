"""scan_dowhile_continue: a `continue` that skips a translated do-while condition."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_dowhile_continue import scan_macros, scan_text

BUGGY = """
fn register(ctx: *mut Ctx) {
    loop {
        if read(ctx) == 0 {
            continue; // skips SetupFindNextLine below
        }
        for i in 0..4 {
            if i == 2 { continue; } // the for's own continue: not a finding
        }
        if next_line(ctx) == 0 {
            break;
        }
    }
}
"""

GUARDED_AND_UNRELATED = """
fn a() {
    loop {
        if x() { break; }
        work();
    }
}
fn b() {
    loop { // nolint: dowhile-continue (C is for(;;))
        if y() { continue; }
        if done() { break; }
    }
}
fn c() {
    'outer: loop {
        for i in 0..3 {
            if i == 1 { continue 'outer; }
        }
        if done() { break; }
    }
}
"""


def test_flags_the_skipped_condition():
    assert [(fn, n) for _, fn, n in scan_text(BUGGY)] == [("register", 1)]


def test_trailing_break_without_continue_nolint_and_labelled_continue():
    # a(): no continue -> clean. b(): nolint. c(): a LABELLED continue from a nested
    # loop still targets the outer loop and skips its condition -> flagged.
    assert [(fn, n) for _, fn, n in scan_text(GUARDED_AND_UNRELATED)] == [("c", 1)]


MACRO_BUGGY = """
macro_rules! LIST_FOR_EACH_ENTRY {
    ($elem:ident, $head:expr, $ty:ty, $field:ident, $body:block) => {{
        $elem = first($head);
        while not_head($elem) {
            $body
            $elem = next($elem);
        }
    }};
}
"""

MACRO_FIXED = """
macro_rules! LIST_FOR_EACH_ENTRY {
    ($elem:ident, $head:expr, $body:block) => {{
        let mut __advance = false;
        loop {
            if __advance {
                $elem = next($elem);
            }
            __advance = true;
            if is_head($elem) {
                break;
            }
            $body
        }
    }};
}
macro_rules! LIST_FOR_EACH_ENTRY_SAFE {
    ($elem:ident, $head:expr, $body:block) => {{
        while cur != head {
            let next = (*cur).next;
            cur = next;
            $body
        }
    }};
}
macro_rules! NOT_A_LOOP {
    ($body:block) => {{
        $body
        done();
    }};
}
"""


def test_macro_advancing_after_its_body_is_flagged():
    assert [name for _, name in scan_macros(MACRO_BUGGY)] == ["LIST_FOR_EACH_ENTRY"]


def test_macro_advancing_first_or_outside_a_loop_is_clean():
    assert scan_macros(MACRO_FIXED) == []
