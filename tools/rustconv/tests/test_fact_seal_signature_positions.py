"""A value that lives only in C++ declarator syntax must not be sealed.

Both constructs here are UNSATISFIABLE seal envelopes: the token would be demanded of a Rust
signature that has nowhere to put it, so the deterministic audit rejects every attempt no
matter how correct the body is, and each retry is a paid call. `DoGetAliasPIDLs` burned four
on the array bound; `ExtensionFromPidl` burned three on the default argument.

The regression the other direction matters just as much: an array bound or an `=` in the BODY
is a genuine value and must STAY sealed. Scoping the exclusion to "everything before the first
brace" silently unsealed `int a[2] = {0, 1};`, whose first brace is the initializer.
"""

from rustconv.fact_seal import seal_facts


def _sig(text: str) -> str:
    return text.split("{", 1)[0]


def test_default_argument_value_is_not_sealed():
    src = (
        "static LPCWSTR ExtensionFromPidl(PCUIDLIST_RELATIVE pidl, LPWSTR Buf, UINT cchMax,\n"
        "                                 BOOL AllowFolder = FALSE)\n"
        "{\n"
        "    return AllowFolder ? Buf : 0;\n"
        "}\n"
    )
    sealed = seal_facts(src)
    assert "AllowFolder = FALSE" in _sig(sealed.text)
    assert "__RUSTCONV_FACT_" not in _sig(sealed.text)


def test_every_default_argument_in_the_list_is_skipped():
    src = "int h(int a = 3, int b = 4)\n{\n    return a + b + 99;\n}\n"
    sealed = seal_facts(src)
    assert "int a = 3, int b = 4" in _sig(sealed.text)
    # The body's 99 is a real value and stays sealed.
    assert sealed.text.count("__RUSTCONV_FACT_") == 1


def test_array_parameter_bound_is_not_sealed():
    src = "BOOL g(LPITEMIDLIST apidls[2], int x)\n{\n    return x + 7;\n}\n"
    sealed = seal_facts(src)
    assert "apidls[2]" in _sig(sealed.text)
    assert sealed.text.count("__RUSTCONV_FACT_") == 1


def test_local_array_bound_and_initializer_stay_sealed():
    """The regression guard: the first brace is the INITIALIZER, not the body."""
    src = "void f(void)\n{\n    int a[2] = {0, 1};\n}\n"
    sealed = seal_facts(src)
    assert sealed.text.count("__RUSTCONV_FACT_") == 3


def test_comparison_operators_in_a_parameter_list_are_not_default_arguments():
    """`==`/`!=`/`<=`/`>=` must not open a default-argument span and swallow the rest."""
    src = "int k(int a, int b)\n{\n    if (a == 5 && b != 6) return 7;\n    return 8;\n}\n"
    sealed = seal_facts(src)
    assert sealed.text.count("__RUSTCONV_FACT_") == 4
