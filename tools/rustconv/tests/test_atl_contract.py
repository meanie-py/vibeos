import pytest

from rustconv import atl_contract


def test_names_only_the_types_a_body_uses():
    assert atl_contract.contract("int x = 0; return x;") == ""
    block = atl_contract.contract("CAtlStringW Message; Message.LoadString(IDS_X);")
    assert "`CStringW`:" in block and "`CComPtr`:" not in block
    assert "pub unsafe fn LoadStringW(&mut self, nID: UINT) -> BOOL" in block


def test_signatures_come_from_the_crate_not_a_copy():
    api = atl_contract._api()
    assert "pub fn GetString(&self) -> *const WCHAR" in api["CStringW"]
    assert "pub fn GetAddressOf(&mut self) -> *mut *mut T" in api["CComPtr"]
    # the atl_format! macro's internals are doc(hidden), not API to hand a model
    assert not any("format_begin_pub" in s for s in api["CStringW"])


def test_tchar_spellings_are_the_wide_string_only_in_a_unicode_module():
    assert "`CStringW`:" in atl_contract.contract("CString s;")
    with pytest.raises(ValueError, match="CStringA"):
        atl_contract.contract("CString s;", unicode=False)


def test_comheapptr_brings_the_chheapptr_methods():
    used = atl_contract.types_used("CComHeapPtr<ITEMIDLIST> pidl;")
    assert used == ["CComHeapPtr", "CHeapPtr"]
