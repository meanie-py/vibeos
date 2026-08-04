"""Convention aliases that also carry the RETURN TYPE.

`STDAPI` is `EXTERN_C HRESULT STDAPICALLTYPE`, so normalizing it to a bare `__stdcall` —
which is what the scanner does to every alias tree-sitter cannot parse — leaves a
definition with no type at all. tree-sitter does not call that a function_definition, and
the function silently leaves the translation plan. netcfgx.c lost `DllGetClassObject`,
`DllRegisterServer` and `DllUnregisterServer`, all three .spec-exported, and the TU
refused to translate for a year on "3 SPEC-EXPORTED functions are in the unextractable
set". Same class as the `TYPE * CDECL fn()` drop that cost msvcrt 15 exports.

Every KW_EXPANSION entry is exercised here rather than trusted: an expansion nobody
parsed is a guess, and a guess drops functions without saying so.
"""

import pytest

from rustconv.conventions import KW_EXPANSION
from rustconv.scanner import scan_file

# What each alias supplies as the return type, once expanded. `None` means the macro takes
# the type from elsewhere in the same expansion chain and the test only demands extraction.
RETURNS = {
    "STDAPI": "HRESULT",
    "STDAPIV": "HRESULT",
    "STDMETHODIMP": "HRESULT",
    "STDMETHODIMPV": "HRESULT",
    "WINOLEAPI": "HRESULT",
    "PSSTDAPI": "HRESULT",
    "BOOLAPI": "BOOL",
    "NETIOAPI_API": "NETIO_STATUS",
    "NCB_POST": "void",
    "STRSAFEAPI": "HRESULT",
    "STRSAFE_INLINE_API": "HRESULT",
}


def test_the_table_and_the_test_cover_the_same_aliases():
    assert set(KW_EXPANSION) == set(RETURNS)


@pytest.mark.parametrize("alias", sorted(KW_EXPANSION))
def test_a_function_defined_with_the_alias_is_extracted(alias, tmp_path):
    source = tmp_path / "t.c"
    source.write_text(f"{alias}\nDoTheThing(int a)\n{{\n    return a;\n}}\n")
    scan = scan_file(source)
    assert [fn.name for fn in scan.functions] == ["DoTheThing"], f"{alias} dropped the function"
    assert scan.unextracted == []
    assert scan.functions[0].return_type == RETURNS[alias]


def test_the_netcfgx_shape_end_to_end(tmp_path):
    """The live case: `STDAPI` on its own line above the name, three of them in one TU."""
    source = tmp_path / "netcfgx.c"
    source.write_text(
        "HRESULT\nWINAPI\nDllCanUnloadNow(void)\n{\n    return S_FALSE;\n}\n\n"
        "STDAPI\nDllRegisterServer(void)\n{\n    return S_OK;\n}\n\n"
        "STDAPI\nDllUnregisterServer(void)\n{\n    return S_OK;\n}\n\n"
        "STDAPI\nDllGetClassObject(REFCLSID rclsid, REFIID riid, LPVOID* ppv)\n"
        "{\n    return E_NOINTERFACE;\n}\n"
    )
    scan = scan_file(source)
    assert sorted(fn.name for fn in scan.functions) == [
        "DllCanUnloadNow",
        "DllGetClassObject",
        "DllRegisterServer",
        "DllUnregisterServer",
    ]
    assert scan.unextracted == []


def test_a_chained_alias_resolves_through_its_stem(tmp_path):
    """`WINOLEAPI` is `STDAPI` is `EXTERN_C HRESULT STDAPICALLTYPE` — two hops, and the
    expansion is the header's own text at each one so no finished form is hand-encoded."""
    from rustconv.scanner import _expand_composite_conventions

    assert _expand_composite_conventions("WINOLEAPI f(void);") == (
        "EXTERN_C HRESULT STDAPICALLTYPE f(void);"
    )
