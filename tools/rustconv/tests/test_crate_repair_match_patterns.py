"""crate-repair must never wrap a match-arm PATTERN in a cast.

Rust forbids expressions in patterns, so `(RPC_S_INVALID_BINDING) as i32 =>` does not
parse -- it turns a compiling match into a hard parse error. It fired on wlanapi's
`WlanRpcStatusToWinError` (2026-07-25), destroying all 5 arms at once, and a `match` over
a C status code is precisely where integer-width mismatches surface, so the rule aims
straight at this shape.
"""

from rustconv.crate_repair import _in_match_pattern


def probe(source: str, needle: str) -> bool:
    """Is `needle` (its LAST byte) in pattern position within `source`?"""
    data = source.encode()
    return _in_match_pattern(data, data.index(needle.encode()) + len(needle))


MATCH = """
match Status as u32 {
    RPC_S_INVALID_BINDING | RPC_X_SS_IN_NULL_CONTEXT => ERROR_INVALID_HANDLE,
    RPC_X_NULL_REF_POINTER => ERROR_INVALID_ADDRESS,
    _ => Status as DWORD,
}
"""


def test_first_pattern_of_an_or_arm_is_detected():
    assert probe(MATCH, "RPC_S_INVALID_BINDING")


def test_second_pattern_of_an_or_arm_is_detected():
    assert probe(MATCH, "RPC_X_SS_IN_NULL_CONTEXT")


def test_single_pattern_arm_is_detected():
    assert probe(MATCH, "RPC_X_NULL_REF_POINTER")


def test_the_scrutinee_is_NOT_a_pattern():
    """Widening the scrutinee is legal and is usually the correct repair."""
    assert not probe(MATCH, "Status as u32")


def test_the_arm_VALUE_is_not_a_pattern():
    assert not probe(MATCH, "ERROR_INVALID_HANDLE")


def test_ordinary_statement_is_not_a_pattern():
    assert not probe("let x = SOME_CONST;\n", "SOME_CONST")


def test_call_argument_is_not_a_pattern():
    assert not probe("foo(SOME_CONST, other);\n", "SOME_CONST")


def test_struct_literal_field_is_not_a_pattern():
    assert not probe("Foo { a: SOME_CONST, b: 2 };\n", "SOME_CONST")


def test_comparison_operator_is_not_mistaken_for_an_arrow():
    """`>=` and `=>` differ only in order; a `>=` must not read as an arm arrow."""
    assert not probe("if SOME_CONST >= limit { work(); }\n", "SOME_CONST")


def test_arm_body_block_contents_are_not_patterns():
    source = "match x {\n    A => { let y = SOME_CONST; }\n}\n"
    assert not probe(source, "SOME_CONST")


def test_tuple_pattern_with_an_inner_comma_is_still_a_pattern():
    """The `,` stop must respect bracket depth."""
    assert probe("match p {\n    (SOME_CONST, 0) => v,\n}\n", "SOME_CONST")


def test_nested_call_argument_is_not_a_pattern():
    """`foo(bar(A), b)` and `(A, 0) => v` look identical until past the closing paren."""
    assert not probe("foo(bar(SOME_CONST), b);\n", "SOME_CONST")


def test_nested_tuple_pattern_is_still_a_pattern():
    assert probe("match p {\n    (a, (SOME_CONST, 0)) => v,\n}\n", "SOME_CONST")


def test_argument_in_an_arm_body_call_is_not_a_pattern():
    """The next arm's `=>` must not leak backwards into this arm's body."""
    source = "match x {\n    A => foo(SOME_CONST, y),\n    B => z,\n}\n"
    assert not probe(source, "SOME_CONST")
