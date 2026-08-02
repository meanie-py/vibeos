"""Nothing inside a preprocessor directive is a value or a callee, and an array
parameter's bound is neither.

Both are UNSATISFIABLE envelopes rather than merely awkward ones: the sealed token has
nowhere to go in any correct Rust output, so every paid attempt is rejected and one
rejected worker assembles no file at all. shell32's `changenotify.cpp` hit both in one
translation and burned eight paid attempts.

* `DumpEvent` defines `DUMPEV(x)` in its own body, pastes `SHCNE_##x`, and `#undef`s it
  at the end. That sealed as `#define __RUSTCONV_CALL_0000__(x)`, `__RUSTCONV_FACT_0001__##x`
  and `#undef __RUSTCONV_FACT_0003__` — a macro DEFINITION name demanded back "as the called
  function identifier in executable Rust", and half of a token paste demanded back as a value.
* `DoGetAliasPIDLs(LPITEMIDLIST apidls[2], ...)` sealed the `2`. A C array parameter decays to
  a pointer, so the Rust signature has no place for it.

The `#undef DUMPEV` is also why `macro_authority` cannot rely on `-E -dM` alone: that dump
lists only macros still defined when preprocessing ends.
"""

from rustconv.call_seal import seal_calls
from rustconv.fact_seal import prepare_sealed_source
from rustconv.lex_mask import directive_spans
from rustconv.macro_authority import _DEFINE_FN_LIKE

DUMP_EVENT = """static LPCSTR DumpEvent(LONG event)
{
    if (event == SHCNE_ALLEVENTS)
        return "SHCNE_ALLEVENTS";
#define DUMPEV(x)  ,( event & SHCNE_##x )? #x " " : ""
    return wine_dbg_sprintf( "%s%s"
    DUMPEV(RENAMEITEM)
    DUMPEV(CREATE)
    );
#undef DUMPEV
}"""

ARRAY_PARAM = """static BOOL DoGetAliasPIDLs(LPITEMIDLIST apidls[2], PCIDLIST_ABSOLUTE pidl)
{
    apidls[0] = apidls[1] = NULL;
    INT k = 0;
    if (k >= 2)
        return FALSE;
    return k > 0;
}"""


def _originals(body: str) -> list[str]:
    return [fact.original for fact in prepare_sealed_source(body, dialect="cpp").facts]


def test_directive_spans_cover_a_continued_define():
    text = "int a;\n#define FOO(x) \\\n    bar(x)\nint b;\n"
    inside = directive_spans(text)
    assert not inside[text.index("int a")]
    assert inside[text.index("#define")]
    assert inside[text.index("bar(x)")], "a backslash-continued line is still the directive"
    assert not inside[text.index("int b")]


def test_a_hash_inside_a_string_does_not_open_a_directive():
    text = 'const char *s = "\\n#define NOPE(x)";\nint after;\n'
    inside = directive_spans(text)
    assert not any(inside), "the # is literal text, not a directive"


def test_define_and_undef_names_are_not_sealed_as_calls():
    sealed = seal_calls(DUMP_EVENT, {"wine_dbg_sprintf", "DUMPEV"}, frozenset({"DUMPEV"}))
    assert "#define DUMPEV(x)" in sealed.text
    assert "#undef DUMPEV" in sealed.text
    # The one real runtime callee still gets exactly one token.
    assert [call.original for call in sealed.calls] == ["wine_dbg_sprintf"]


def test_macro_invocations_are_not_sealed_as_calls():
    """Even with the macro excluded, its INVOCATIONS must survive untouched."""
    sealed = seal_calls(DUMP_EVENT, {"wine_dbg_sprintf", "DUMPEV"}, frozenset({"DUMPEV"}))
    assert sealed.text.count("DUMPEV(") == 3  # one definition + two invocations


def test_paste_operand_is_not_sealed_as_a_fact():
    assert "SHCNE_##x" in prepare_sealed_source(DUMP_EVENT, dialect="cpp").text
    assert "#undef DUMPEV" in prepare_sealed_source(DUMP_EVENT, dialect="cpp").text
    # The genuine constant outside the directive is still protected.
    assert "SHCNE_ALLEVENTS" in _originals(DUMP_EVENT)


def test_array_parameter_bound_is_not_sealed():
    sealed = prepare_sealed_source(ARRAY_PARAM, dialect="cpp")
    assert "apidls[2], PCIDLIST_ABSOLUTE pidl)" in sealed.text, "signature bound must survive"
    # Body indices and the body's own `2` remain sealed — they are real values.
    assert sealed.text.count("__RUSTCONV_FACT_") >= 4
    assert "apidls[0]" not in sealed.text
    assert "k >= 2" not in sealed.text


def test_a_local_array_bound_is_still_sealed():
    """The exclusion is the PARAMETER LIST, not "before the first brace".

    Scoping it to the first brace regressed `int a[2] = {0, 1};`, whose first brace is the
    INITIALIZER — the bound stopped being sealed even though it is a real value in a real
    local declaration. Caught by test_fact_seal_zero_init.
    """
    assert "2" in _originals("int a[2] = {0, 1};")
    body = "static void f(void)\n{\n    WCHAR buf[260];\n    g(buf);\n}"
    assert "260" in _originals(body), "a bound in the BODY is a value"


def test_define_regex_sees_an_undeffed_body_macro():
    """`-E -dM` cannot report DUMPEV, so the TU source scan is what catches it."""
    assert "DUMPEV" in set(_DEFINE_FN_LIKE.findall(DUMP_EVENT))
    assert set(_DEFINE_FN_LIKE.findall("#define OBJECT_LIKE 3\n")) == set()
    assert "Indented" in set(_DEFINE_FN_LIKE.findall("  #  define Indented(a) a\n"))
