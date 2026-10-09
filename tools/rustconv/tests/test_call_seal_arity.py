"""Call-seal arity: a site whose argument count the C text cannot settle is skipped."""

from rustconv.call_seal import seal_calls, unseal_calls

TOOLTIPS_SHOW_C = """void f(void)
{
    hRgn = CreateRoundRectRgn(0,
                              top,
                              width,
#ifdef __REACTOS__
                              height + 1,
#else
                              height,
#endif
                              16, 16);
}
"""


def test_an_ifdef_inside_the_argument_list_is_not_counted():
    """comctl32's TOOLTIPS_Show passes one argument per `__REACTOS__` arm: the C text reads
    as 7 arguments, and the model's correct 6 was refused on every sample."""
    sealed = seal_calls(TOOLTIPS_SHOW_C, {"CreateRoundRectRgn"}, frozenset())
    rust = "unsafe fn f() {\n    hRgn = __RUSTCONV_CALL_0000__(0, top, width, height + 1, 16, 16);\n}"
    assert "CreateRoundRectRgn(0, top, width, height + 1, 16, 16)" in unseal_calls(rust, sealed)
