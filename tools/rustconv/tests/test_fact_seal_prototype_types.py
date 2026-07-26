"""A parameter type in a body-local `extern` prototype is ABI context, not a value.

`seal_calls` already refuses to treat the CALLEE name in that construct as a runtime
call. The same construct's parameter TYPES were still being sealed as facts, and a
faithful Rust lowering is entitled to spell `LPCWSTR` as `*const u16` — at which
point the token is simply gone. `InvokeOpenWith` failed this way on four consecutive
paid attempts, and one rejected worker assembles no file at all.
"""

from rustconv.fact_seal import prepare_sealed_source

BODY = """static UINT_PTR InvokeOpenWith(HWND hWndOwner, SHELLEXECUTEINFOW &sei)
{
    extern HRESULT SH32_InvokeOpenWith(LPCWSTR, HWND, HANDLE *);
    HANDLE *phProc = (sei.fMask & SEE_MASK_NOCLOSEPROCESS) ? &sei.hProcess : NULL;
    return SUCCEEDED(hr) ? 42 : 31;
}"""


def _originals(body: str) -> list[str]:
    return [fact.original for fact in prepare_sealed_source(body, dialect="cpp").facts]


def test_prototype_parameter_types_are_not_sealed():
    originals = _originals(BODY)
    assert "LPCWSTR" not in originals
    assert "HANDLE" not in originals


def test_the_constants_sealing_exists_for_are_still_protected():
    """§9.113: an invented constant is the whole point. Nothing may weaken that."""
    originals = _originals(BODY)
    assert "SEE_MASK_NOCLOSEPROCESS" in originals
    assert "42" in originals and "31" in originals


def test_a_constant_that_merely_follows_a_prototype_is_still_sealed():
    """The exemption is the prototype's own span, not everything after it."""
    body = """static int f(void)
{
    extern HRESULT Helper(LPCWSTR, HWND);
    return (mask & SEE_MASK_NOCLOSEPROCESS) ? STATUS_SUCCESS : 7;
}"""
    originals = _originals(body)
    assert "SEE_MASK_NOCLOSEPROCESS" in originals
    assert "STATUS_SUCCESS" in originals
    assert "LPCWSTR" not in originals


def test_cpp_named_cast_template_argument_is_not_sealed():
    """`const_cast<LPWSTR>(x)` becomes `x as LPWSTR` — or elides the alias entirely."""
    body = """static void f(void)
{
    space = const_cast<LPWSTR>(strchrW(p, ' '));
    q = reinterpret_cast<LPBYTE>(buf);
    if (flags & SEE_MASK_NOCLOSEPROCESS) return;
}"""
    originals = _originals(body)
    assert "LPWSTR" not in originals
    assert "LPBYTE" not in originals
    assert "SEE_MASK_NOCLOSEPROCESS" in originals


def test_a_body_with_no_prototype_is_unchanged():
    body = "static int g(void) { return (m & FOO_FLAG) ? BAR_OK : 3; }"
    originals = _originals(body)
    assert "FOO_FLAG" in originals
    assert "BAR_OK" in originals
