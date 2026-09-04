"""A macro-expansion error is reported against the macro's DEFINITION, not the call site.

So a repair keyed on that span edits the macro BODY — which is never the right fix, because
the offending value came from a caller the edit does not touch, and which therefore does not
converge: the next pass sees the same error and wraps the body again.

Real case: `msvcrt-crt-rust`'s `ARRAY_SIZE!` is `($a).len()`. One `locale.rs` caller wanting
an `i32` drove six passes of

    ((((((($a).len()) as i32) as usize) as i32) as usize) as i32) as usize

corrupting a macro that every translated module in the crate uses, to "fix" a call site that
was never edited. The repair must decline and leave it RED for a human.
"""

from __future__ import annotations

from rustconv.crate_repair import _in_macro_definition, _mask_lexical

ARRAY_SIZE = """#[macro_export]
macro_rules! ARRAY_SIZE {
    ($a:expr) => {
        ($a).len()
    };
}

pub unsafe fn caller(t: &[u16; 8]) -> i32 {
    ARRAY_SIZE!(t)
}
"""


def _at(src: str, needle: str) -> bool:
    """Is `needle`'s START position inside a `macro_rules!` body?"""
    return _in_macro_definition(_mask_lexical(src.encode()), src.index(needle))


def test_the_macro_body_is_declined():
    assert _at(ARRAY_SIZE, "($a).len()")


def test_the_call_site_is_not_declined():
    # The whole point: the caller stays repairable, so the fix lands where it belongs.
    assert not _at(ARRAY_SIZE, "ARRAY_SIZE!(t)")


def test_code_before_any_macro_is_not_declined():
    assert not _in_macro_definition(_mask_lexical(b"let x = y.len();\n"), 8)


def test_code_after_a_closed_macro_is_not_declined():
    # Negative control with teeth: same file, same `macro_rules!` above it, but the braces
    # have closed. If the walk-back forgot to balance, this would false-positive.
    assert not _at(ARRAY_SIZE, "pub unsafe fn caller")


def test_a_nested_brace_inside_the_body_still_counts_as_inside():
    src = """macro_rules! M {
    ($x:expr) => {{
        let v = { $x };
        v.len()
    }};
}
"""
    assert _at(src, "v.len()")


def test_macro_rules_inside_a_string_does_not_arm_the_guard():
    # `_mask_lexical` blanks string bodies, so the literal below must not be mistaken for a
    # real definition — otherwise every span after it in the file would be declined.
    src = 'let s = "macro_rules! fake {";\nlet n = t.len();\n'
    assert not _at(src, "t.len()")
