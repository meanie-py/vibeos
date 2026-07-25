"""A cast is not a pattern — `(_MB_CP_SBCS) as i32 => {` does not parse.

rustc reports a `match` arm whose constant type disagrees with the scrutinee exactly like any
other integer mismatch, so the cast rule fired on it and wrote ungrammatical Rust. Real case:
msvcrt's `_setmbcp_l`, where bindgen typed one member of a four-macro `#define` block `u32`
(its value is 0) and the other three `i32` (theirs are negative), splitting a family that is
all `int` in C.
"""

from __future__ import annotations

from rustconv.crate_repair import _in_match_pattern, _mask_lexical


def _at(src: str, needle: str) -> bool:
    """Is `needle`'s end position inside a match pattern?"""
    data = src.encode()
    return _in_match_pattern(_mask_lexical(data), src.index(needle) + len(needle))


def test_a_bare_constant_arm_is_a_pattern():
    assert _at("match cp {\n    _MB_CP_SBCS => {\n        x = 1;\n    }\n}", "_MB_CP_SBCS")


def test_an_or_pattern_arm_is_a_pattern():
    assert _at("match cp {\n    A | B_CONST => {\n        x = 1;\n    }\n}", "B_CONST")


def test_an_arm_body_is_not_a_pattern():
    assert not _at("match cp {\n    A => {\n        newcp = GetACP();\n    }\n}", "GetACP()")


def test_an_ordinary_assignment_is_not_a_pattern():
    assert not _at("let x = SOME_CONST;\nlet y = 2;\n", "SOME_CONST")


def test_a_call_argument_is_not_a_pattern():
    assert not _at("foo(SOME_CONST, 1);\n", "SOME_CONST")


def test_a_block_open_stops_the_scan():
    assert not _at("if x == SOME_CONST {\n    y => z\n}", "SOME_CONST")
