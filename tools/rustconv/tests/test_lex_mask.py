"""A Rust label is not a char literal, in any masker in this tree.

Three private C lexers used to blank everything between `'cleanup:` and the next
apostrophe, which is how a faithful lowering of `SHFileOperationA` — three
`goto cleanup` become three `'cleanup: loop` blocks — was rejected for two call
tokens the model had actually written.
"""

from rustconv.lex_mask import comment_spans, mask_code, opens_a_literal


def test_rust_label_does_not_open_a_literal():
    rust = "'cleanup: loop { CALL_A(); }\n'cleanup: loop { CALL_B(); }\nCALL_C();\n"
    masked = mask_code(rust)
    for name in ("CALL_A", "CALL_B", "CALL_C"):
        assert name in masked, f"{name} was blanked by the masker"
    assert len(masked) == len(rust)


def test_lifetimes_do_not_open_a_literal():
    rust = "fn f<'a>(x: &'a str) -> Option<&'static [u8]> { CALL_A(x) }"
    assert "CALL_A" in mask_code(rust)


def test_char_literals_are_still_masked():
    assert "SECRET" not in mask_code("let c = 'S'; let s = \"SECRET\"; KEEP();")
    assert "KEEP" in mask_code("let c = 'S'; let s = \"SECRET\"; KEEP();")


def test_c_multi_character_constant_is_still_a_literal():
    """`'MPEG'` is a legal C character constant, not a label."""
    assert opens_a_literal("x = 'MPEG';", 4)
    assert not opens_a_literal("'cleanup: loop {", 0)
    assert opens_a_literal("c = 'a';", 4)
    assert opens_a_literal(r"c = '\n';", 4)


def test_escaped_quote_inside_a_char_literal():
    assert "KEEP" in mask_code(r"let q = '\''; KEEP();")


def test_comment_spans_agree_with_the_label_rule():
    text = "'label: loop { /* HIDDEN */ VISIBLE(); }"
    inside = comment_spans(text)
    assert all(inside[text.index("HIDDEN") : text.index("HIDDEN") + 6])
    assert not any(inside[text.index("VISIBLE") : text.index("VISIBLE") + 7])
