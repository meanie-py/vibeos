"""A C aggregate zero-initializer is syntax, not a value.

`PROPSHEETPAGE psp = {0};` says "all bytes zero". Every faithful Rust lowering —
`core::mem::zeroed()`, `Default::default()`, a struct literal naming each field — has
nowhere to put the `0`, so demanding it back rejects a CORRECT translation. netid's
`CreateNetIDPropertyPage` died exactly there, on output that was otherwise perfect:

    let mut psp: PROPSHEETPAGE = core::mem::zeroed();

Same class the shell32 wave generalized (§9.127): the envelope assumes the lowering keeps a
1:1 token correspondence with the C, and any construct that must be RESTRUCTURED breaks it.

The rule is deliberately narrow — only the single-`0` form. Where an initializer's values
carry meaning they must still be sealed, and the invented/dropped checks must still fire.
"""

from __future__ import annotations

from rustconv.fact_seal import seal_facts

# The real function, verbatim.
NETID_CREATE_PAGE = """static HPROPSHEETPAGE
CreateNetIDPropertyPage(VOID)
{
    PROPSHEETPAGE psp = {0};

    psp.dwSize = sizeof(psp);
    psp.dwFlags = PSP_DEFAULT;
    psp.hInstance = hDllInstance;
    psp.pszTemplate = MAKEINTRESOURCE(IDD_PROPPAGECOMPUTER);
    psp.pfnDlgProc = NetIDPageProc;

    return CreatePropertySheetPage(&psp);
}"""


def _sealed(src: str) -> list[str]:
    return [fact.original for fact in seal_facts(src).facts]


def test_the_zero_initializer_is_not_sealed():
    assert "0" not in _sealed(NETID_CREATE_PAGE)


def test_the_real_constants_around_it_are_still_sealed():
    got = _sealed(NETID_CREATE_PAGE)
    assert "PSP_DEFAULT" in got
    assert "IDD_PROPPAGECOMPUTER" in got


def test_whitespace_inside_the_braces_is_still_a_zero_init():
    assert _sealed("FOO f = { 0 };") == []


def test_a_multi_value_initializer_still_seals_every_value():
    # {0, 1} is not "all bytes zero" — the values are meaningful and a drop must fail loudly.
    assert _sealed("int a[2] = {0, 1};") == ["2", "0", "1"]


def test_a_nonzero_single_initializer_still_seals():
    assert _sealed("FOO f = {5};") == ["5"]


def test_an_ordinary_zero_is_untouched():
    # The rule must not become "0 is never a fact" — a plain assignment or return still seals.
    assert _sealed("x = 0; return 0;") == ["0", "0"]


def test_a_zero_init_does_not_hide_a_later_zero_in_the_same_body():
    src = "FOO f = {0};\nint n = 0;"
    assert _sealed(src) == ["0"]


def test_pseh_statement_macros_are_syntax_not_facts():
    # tdihelpers: a sealed _SEH2_TRY/_SEH2_END hid the construct and the reply did not parse
    from rustconv.fact_seal import seal_facts

    c = (
        "{ NTSTATUS Status = STATUS_SUCCESS;\n"
        "  _SEH2_TRY { RtlCopyMemory(a, b, n); }\n"
        "  _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Status = _SEH2_GetExceptionCode(); }\n"
        "  _SEH2_END;\n  return Status; }\n"
    )
    sealed = seal_facts(c, dialect="c")
    originals = {f.original for f in sealed.facts}
    assert {"_SEH2_TRY", "_SEH2_END"}.isdisjoint(originals)
    assert "_SEH2_TRY {" in sealed.text and "_SEH2_END;" in sealed.text
    assert "STATUS_SUCCESS" in originals
