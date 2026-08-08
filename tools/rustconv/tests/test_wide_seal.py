"""The UTF-16 literal envelope: what it protects, what it refuses, and how it fails.

Every test here is a shape measured in the tree (2,645 plain-C files, 10,871 wide-literal
tokens), not an invented one. The counts in the comments are from that sweep.
"""

from pathlib import Path

import pytest
from rustconv.wide_seal import (
    KIND_ARRAY,
    KIND_CHAR,
    KIND_POINTER,
    WideSealError,
    render_wide_statics,
    seal_wide_literals,
    unseal_wide_literals,
    wide_contract,
)


# --- what the seal removes from the model's reach ---------------------------------


def test_a_string_literal_becomes_an_opaque_token():
    sealed = seal_wide_literals('SetWindowTextW(hwnd, L"Ready");')
    assert 'L"' not in sealed.text
    assert "__RUSTCONV_WIDE_0000__" in sealed.text
    assert sealed.wides[0].kind == KIND_POINTER
    # The bytes are rustconv's, and they carry the terminator C gives for free.
    assert sealed.wides[0].units == (0x52, 0x65, 0x61, 0x64, 0x79, 0)


def test_the_contract_reveals_the_type_and_never_the_value():
    sealed = seal_wide_literals('f(L"secret");')
    contract = wide_contract(sealed)
    assert "__RUSTCONV_WIDE_0000__ : *const u16" in contract
    assert "secret" not in contract


def test_a_declared_array_restores_inline_with_its_own_length():
    """`WCHAR buf[] = L"x"` — 219 sites. The length is what the model kept getting wrong."""
    sealed = seal_wide_literals('static const WCHAR cls[] = L"Edit";')
    assert sealed.wides[0].kind == KIND_ARRAY
    assert sealed.wides[0].rust_type == "[u16; 5]"
    restored = unseal_wide_literals("static CLS: [u16; 5] = __RUSTCONV_WIDE_0000__;", sealed)
    assert restored == ("static CLS: [u16; 5] = [0x0045u16, 0x0064, 0x0069, 0x0074, 0x0000];")


def test_sizeof_wants_the_array_not_the_pointer():
    """60 sites. `.as_ptr()` under a `sizeof` measures a pointer and is silently wrong."""
    sealed = seal_wide_literals('n = sizeof(L"ab");')
    assert sealed.wides[0].kind == KIND_ARRAY
    assert unseal_wide_literals("let n = size_of_val(&__RUSTCONV_WIDE_0000__);", sealed) == (
        "let n = size_of_val(&[0x0061u16, 0x0062, 0x0000]);"
    )


def test_a_wide_character_restores_as_one_code_unit():
    sealed = seal_wide_literals("if (c == L'\\\\') return;")
    assert sealed.wides[0].kind == KIND_CHAR
    assert unseal_wide_literals("if c == __RUSTCONV_WIDE_0000__ { return; }", sealed) == (
        "if c == 0x005Cu16 { return; }"
    )


def test_a_pointer_restores_through_a_generated_static():
    sealed = seal_wide_literals('f(L"hi");')
    rust = unseal_wide_literals("f(__RUSTCONV_WIDE_0000__);", sealed)
    name = sealed.wides[0].static_name
    assert rust == f"f({name}.as_ptr());"
    statics = render_wide_statics([sealed], rust)
    assert f"pub static {name}: [u16; 3] = [0x0068u16, 0x0069, 0x0000];" in statics


def test_a_long_array_wraps_so_a_human_can_review_it():
    """casemap.c's Unicode table is 4,570 elements; on one line that is 40KB of hex nobody
    can diff — and these tables are precisely the ones a human is asked to check."""
    sealed = seal_wide_literals("static const WCHAR t[] = {" + ",".join(["'a'"] * 40) + "};")
    rendered = sealed.wides[0].restore
    assert rendered.count("\n") == 5  # open + four rows of 10/10/10/10
    assert max(len(line) for line in rendered.splitlines()) <= 100


def test_equal_literals_share_one_static():
    """The name is a content hash, so two occurrences of one string emit one definition."""
    sealed = seal_wide_literals('f(L"same"); g(L"same");')
    assert sealed.wides[0].static_name == sealed.wides[1].static_name
    rust = unseal_wide_literals(
        "f(A); g(B);".replace("A", "__RUSTCONV_WIDE_0000__").replace("B", "__RUSTCONV_WIDE_0001__"),
        sealed,
    )
    assert render_wide_statics([sealed], rust).count("pub static") == 1


def test_only_referenced_statics_are_rendered():
    sealed = seal_wide_literals('f(L"used"); g(L"dropped");')
    used = sealed.wides[0].static_name
    assert render_wide_statics([sealed], f"{used}.as_ptr()").count("pub static") == 1


def test_a_static_name_with_no_source_raises_rather_than_dangling():
    sealed = seal_wide_literals('f(L"x");')
    with pytest.raises(WideSealError, match="lost deterministic UTF-16"):
        render_wide_statics([sealed], "RUSTCONV_WSTR_AAAAAAAAAAAA.as_ptr()")


# --- the envelope ------------------------------------------------------------------


def test_a_dropped_token_fails_loudly():
    """The whole point: hand-encoding is no longer a thing that can happen quietly."""
    sealed = seal_wide_literals('f(L"Ready");')
    with pytest.raises(WideSealError, match="missing: __RUSTCONV_WIDE_0000__"):
        unseal_wide_literals('f(b"R\\0e\\0a\\0d\\0y\\0\\0\\0".as_ptr() as *const u16);', sealed)


def test_an_invented_token_fails_loudly():
    sealed = seal_wide_literals('f(L"x");')
    with pytest.raises(WideSealError, match="invented: __RUSTCONV_WIDE_0007__"):
        unseal_wide_literals("f(__RUSTCONV_WIDE_0000__, __RUSTCONV_WIDE_0007__);", sealed)


def test_a_hand_written_static_name_fails_loudly():
    """A content hash is not something to guess at: a wrong one names the wrong bytes."""
    sealed = seal_wide_literals('f(L"x");')
    with pytest.raises(WideSealError, match="invented UTF-16 static"):
        unseal_wide_literals(
            "f(__RUSTCONV_WIDE_0000__); g(RUSTCONV_WSTR_0123456789AB.as_ptr());", sealed
        )


def test_a_reordered_envelope_is_reported_not_rejected():
    """The permutation `_tzset` slipped past the constant seal — closed sets cannot see it."""
    sealed = seal_wide_literals('wcscpy(a, L"first"); wcscpy(b, L"second");')
    findings: list[str] = []
    out = unseal_wide_literals(
        "wcscpy(a, __RUSTCONV_WIDE_0001__); wcscpy(b, __RUSTCONV_WIDE_0000__);",
        sealed,
        order_findings=findings,
    )
    assert len(findings) == 1 and "different order" in findings[0]
    assert "second" not in out  # it still restores; the human is told to check the slots
    assert sealed.wides[1].static_name in out.split(";")[0]


def test_a_faithful_order_reports_nothing():
    sealed = seal_wide_literals('f(L"a"); g(L"b");')
    findings: list[str] = []
    unseal_wide_literals(
        "f(__RUSTCONV_WIDE_0000__); g(__RUSTCONV_WIDE_0001__);", sealed, order_findings=findings
    )
    assert findings == []


def test_a_token_only_in_dead_code_is_permitted_and_reported():
    """Same bargain as the fact seal: a `#if`-disabled arm is compiled, never executed."""
    body = 'f(L"live");\n#ifndef __REACTOS__\ng(L"dead");\n#endif\n'
    sealed = seal_wide_literals(body)
    assert [w.dead for w in sealed.wides] == [False, True]
    omissions: list[str] = []
    unseal_wide_literals("f(__RUSTCONV_WIDE_0000__);", sealed, dead_omissions=omissions)
    assert omissions == ['__RUSTCONV_WIDE_0001__=L"dead"']


# --- decoding: where C and Python disagree -----------------------------------------


def test_a_multi_digit_hex_escape_is_one_code_unit():
    """C consumes every hex digit; Python's `\\x` takes exactly two. riched20 and kbd*."""
    sealed = seal_wide_literals('f(L"\\x00b7");')
    assert sealed.wides[0].units == (0x00B7, 0)


def test_a_question_mark_escape_loses_its_backslash():
    sealed = seal_wide_literals('f(L"\\?");')
    assert sealed.wides[0].units == (0x3F, 0)


def test_octal_and_simple_escapes_decode():
    sealed = seal_wide_literals('f(L"\\101\\n\\\\");')
    assert sealed.wides[0].units == (0x41, 0x0A, 0x5C, 0)


def test_a_non_bmp_character_becomes_a_surrogate_pair():
    sealed = seal_wide_literals('f(L"\\U0001F600");')
    assert sealed.wides[0].units == (0xD83D, 0xDE00, 0)


def test_an_unknown_escape_is_refused_rather_than_guessed():
    sealed = seal_wide_literals('f(L"\\q");')
    assert sealed.wides == ()
    assert "unsupported escape" in sealed.skipped[0].reason


# --- adjacency ---------------------------------------------------------------------


def test_adjacent_literals_fold_into_one_token():
    """C concatenation. Lowering only one half emits two Rust expressions side by side —
    the gap `cpp_com._wide_literal_tokens` documents and leaves open. 41 sites."""
    sealed = seal_wide_literals('f(L"\\\\Registry\\\\" L"Machine");')
    assert len(sealed.wides) == 1
    assert sealed.wides[0].units == tuple(ord(c) for c in "\\Registry\\Machine") + (0,)


def test_a_narrow_literal_adjacent_to_a_wide_one_is_wide():
    sealed = seal_wide_literals('f(L"a" "b");')
    assert len(sealed.wides) == 1
    assert sealed.wides[0].units == (0x61, 0x62, 0)


def test_a_literal_pasted_against_a_macro_is_skipped_and_reported():
    """`REGSTR_PATH_EXPLORER L"\\IconUnderline"` — the macro's text is not in this body."""
    sealed = seal_wide_literals('f(REGSTR_PATH L"\\\\IconUnderline");')
    assert sealed.wides == ()
    assert "pasted against a macro" in sealed.skipped[0].reason


def test_return_and_sizeof_are_keywords_not_macros():
    assert len(seal_wide_literals('return L"x";').wides) == 1
    assert len(seal_wide_literals('n = sizeof(L"x");').wides) == 1


def test_a_directive_keyword_before_a_literal_is_not_a_paste():
    """shell32's `#endif` / `L"My Music"` rows: three false positives before this."""
    body = '{\n#ifdef X\n    CSIDL_Type_User,\n#endif\n    L"My Music",\n}'
    assert len(seal_wide_literals(body).wides) == 1


# --- what is deliberately left alone -----------------------------------------------


def test_a_sized_array_declaration_is_skipped_and_reported():
    """`WCHAR k[200] = L"..."` lowers to a zero-fill plus a copy — a statement, not a
    value one token can stand for. 15 sites carrying real bytes."""
    sealed = seal_wide_literals('WCHAR keyname[200] = L"SYSTEM\\\\Services";')
    assert sealed.wides == ()
    assert "sized array initializer" in sealed.skipped[0].reason


def test_a_zero_fill_declaration_is_neither_sealed_nor_reported():
    """`= L""` / `= {0}` carry no bytes, and demanding a token there would reject the
    lowerings that ARE correct for them (`[0u16; N]`, `zeroed()`). Not a finding either:
    a report padded with non-findings is a report nobody reads."""
    for body in ('WCHAR CurLocale[256] = L"";', "WCHAR buf[MAX_PATH] = {0};"):
        sealed = seal_wide_literals(body)
        assert sealed.wides == () and sealed.skipped == (), body


def test_an_array_ELEMENT_store_is_a_pointer_not_a_declaration():
    """`names[0] = L"x"` has no type before the name, so it is an ordinary pointer store."""
    sealed = seal_wide_literals('names[0] = L"x";')
    assert sealed.wides[0].kind == KIND_POINTER


def test_nothing_inside_a_preprocessor_directive_is_sealed():
    """A `#define`'s token has nowhere to go in the Rust output (lex_mask.directive_spans)."""
    sealed = seal_wide_literals('#define WIN_PREVIEW_CLASS L"WinPreview"\nf(x);')
    assert sealed.wides == ()
    assert sealed.skipped == ()


def test_a_narrow_literal_is_left_alone():
    sealed = seal_wide_literals('printf("plain %s\\n", s);')
    assert sealed.wides == () and sealed.text.count('"plain') == 1


def test_a_narrow_string_ending_in_an_apostrophe_is_not_a_char_literal():
    sealed = seal_wide_literals('f("it\'s", L"w");')
    assert len(sealed.wides) == 1 and sealed.wides[0].units == (0x77, 0)


# --- the brace-list shape: Wine's house style, and where the bugs actually came from --


def test_a_brace_initialised_wide_string_is_sealed():
    """`{'C','o','m','b','o','L','B','o','x',0}` — 4,272 declarations in the tree, versus
    10,871 `L"..."` tokens. Sealing only the latter would have missed the majority."""
    sealed = seal_wide_literals(
        "static const WCHAR clbName[] = {'C','o','m','b','o','L','B','o','x',0};"
    )
    assert len(sealed.wides) == 1
    assert sealed.wides[0].kind == KIND_ARRAY
    assert sealed.wides[0].units == (0x43, 0x6F, 0x6D, 0x62, 0x6F, 0x4C, 0x42, 0x6F, 0x78, 0)
    assert sealed.wides[0].rust_type == "[u16; 10]"


def test_the_exact_array_that_shipped_one_short_can_no_longer_ship_one_short():
    """secur32's `negotiate_comment_W` came back as [u16; 28] for a 29-element C list; its
    sibling four lines up came back correct. That is a sampler, and this is why the seal
    exists — the token carries the length, so the model has nothing to miscount."""
    c = (
        "static WCHAR negotiate_comment_W[] =\n"
        "    {'M','i','c','r','o','s','o','f','t',' ','P','a','c','k','a','g','e',' ',\n"
        "     'N','e','g','o','t','i','a','t','o','r',0};"
    )
    sealed = seal_wide_literals(c)
    assert sealed.wides[0].units == tuple(ord(x) for x in "Microsoft Package Negotiator") + (0,)
    assert sealed.wides[0].rust_type == "[u16; 29]"
    with pytest.raises(WideSealError, match="missing"):
        # What actually shipped: retyped by hand, one element short.
        unseal_wide_literals("static negotiate_comment_W: [u16; 28] = [0x4D, 0x69];", sealed)


def test_the_seal_never_invents_a_terminator_for_a_brace_list():
    """195 of the tree's brace lists deliberately end without one (fixed-width fields).
    `L"..."` implies a terminator and this does not — copying what the C wrote is the
    guarantee, and adding a zero would be the same error pointing the other way."""
    sealed = seal_wide_literals("static const WCHAR tag[] = {'a','b'};")
    assert sealed.wides[0].units == (0x61, 0x62)


def test_a_comma_that_is_an_ELEMENT_does_not_split_the_list():
    sealed = seal_wide_literals("static const WCHAR sep[] = {',',' ',0};")
    assert sealed.wides[0].units == (0x2C, 0x20, 0)


def test_a_comment_inside_the_element_list_is_not_an_element():
    """The two kbd layout tables write `{ /* index */ 0x01bf, ... }`."""
    sealed = seal_wide_literals("static const WCHAR t[] = { /* index */ 0x01bf, 0x02bf };")
    assert sealed.wides[0].units == (0x01BF, 0x02BF)


def test_hex_octal_and_escape_elements_decode():
    sealed = seal_wide_literals("static const WCHAR m[] = {0x41,010,'\\\\','\\0'};")
    assert sealed.wides[0].units == (0x41, 8, 0x5C, 0)


def test_a_named_constant_element_is_refused_rather_than_guessed():
    """atl's `{'A','t','l','A','x','W','i','n',ATL_NAME_SUFFIX}` — the value is not here."""
    sealed = seal_wide_literals("static const WCHAR n[] = {'A','t','l',ATL_NAME_SUFFIX};")
    assert sealed.wides == ()
    assert "not a literal" in sealed.skipped[0].reason


def test_a_sized_brace_list_is_zero_filled_to_its_declared_bound():
    """C zero-fills the tail, so the Rust value is N units — not the written count."""
    sealed = seal_wide_literals("WCHAR drive[8] = {'C',':',0};")
    assert sealed.wides[0].units == (0x43, 0x3A, 0, 0, 0, 0, 0, 0)


def test_a_sized_brace_list_with_a_macro_bound_is_skipped_and_reported():
    sealed = seal_wide_literals("WCHAR path[MAX_PATH] = {'C',':',0};")
    assert sealed.wides == ()
    assert "MAX_PATH" in sealed.skipped[0].reason


def test_an_array_of_arrays_is_not_a_string():
    sealed = seal_wide_literals("static const WCHAR t[][2] = {{'a',0},{'b',0}};")
    assert sealed.wides == () and sealed.skipped == ()


def test_a_wide_char_element_inside_a_brace_list_belongs_to_the_array_token():
    """Otherwise the same bytes get two owners and the restore emits both."""
    sealed = seal_wide_literals("static const WCHAR m[] = {L'a',L'b',0};")
    assert len(sealed.wides) == 1
    assert sealed.wides[0].kind == KIND_ARRAY and sealed.wides[0].units == (0x61, 0x62, 0)


def test_both_shapes_number_in_source_order():
    sealed = seal_wide_literals("f(L\"first\");\nstatic const WCHAR s[] = {'b',0};")
    assert [w.kind for w in sealed.wides] == [KIND_POINTER, KIND_ARRAY]


# --- the file-scope preamble, which is a separate paid call ------------------------


def test_the_preamble_prompt_carries_the_sealed_text_and_its_contract():
    """The preamble is the ONLY pass that sees file-scope C, and it is where all 18 of the
    unterminated arrays lived — so it needs the envelope as much as any body does. The
    function-body seals never reach it: it is built by a different prompt builder."""
    from types import SimpleNamespace

    from rustconv.function_prompt import build_preamble_prompt

    c = "static const WCHAR clbName[] = {'C','o','m','b','o',0};"
    sealed = seal_wide_literals(c)
    scan = SimpleNamespace(path=Path("combo.c"), preamble_text=c, globals_text="")
    resolve = SimpleNamespace(imports={})
    prompt = build_preamble_prompt(
        scan, resolve, sealed_c=sealed.text, extra_contract=wide_contract(sealed)
    )
    assert "__RUSTCONV_WIDE_0000__ : [u16; 6]" in prompt
    assert "'C','o','m','b','o'" not in prompt


# --- interaction with the other two seals ------------------------------------------


def test_the_wide_token_survives_the_fact_seal():
    """Wide sealing runs FIRST, so the fact seal must not mistake the placeholder for a
    constant — and must still see the array BOUND beside it as the integer it is."""
    from rustconv.fact_seal import seal_facts

    wide = seal_wide_literals('WCHAR buf[MAX_PATH]; f(buf, L"x", 200);')
    facts = seal_facts(wide.text, dialect="c")
    assert "__RUSTCONV_WIDE_0000__" in facts.text
    assert {f.original for f in facts.facts} == {"MAX_PATH", "200"}


def test_an_arithmetic_element_that_begins_with_a_quote_is_not_a_char_literal():
    """msi builds stream numbers as `'0' + (n / 100) % 10`. Matched by `startswith("'")` that
    slices as a char literal and decodes 29 units of arithmetic before anything notices."""
    sealed = seal_wide_literals("static const WCHAR n[] = {'0' + (num / 100) % 10, 0};")
    assert sealed.wides == ()
    assert "not a literal" in sealed.skipped[0].reason
