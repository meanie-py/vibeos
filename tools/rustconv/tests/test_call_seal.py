import pytest

from rustconv.call_seal import CallSealError, seal_calls, unseal_calls


def test_seals_only_required_runtime_calls_inside_body():
    source = """HRESULT C::f() {
        TRACE("Call(x)");
        x = Call(x);
        obj->Method(x);
        return sizeof(x) + _countof(a);
    }"""
    sealed = seal_calls(source, {"TRACE", "Call", "Method", "sizeof", "_countof"})
    assert [call.original for call in sealed.calls] == ["Call", "Method"]
    assert 'TRACE("Call(x)")' in sealed.text
    assert "__RUSTCONV_CALL_0000__(x)" in sealed.text
    assert "obj->__RUSTCONV_CALL_0001__(x)" in sealed.text


def test_unseal_requires_each_call_once_in_executable_code():
    sealed = seal_calls("int f() { return Call(1); }", {"Call"})
    restored = unseal_calls("unsafe fn f() { __RUSTCONV_CALL_0000__(1); }", sealed)
    assert "Call(1)" in restored
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls("unsafe fn f() { /* __RUSTCONV_CALL_0000__(1) */ }", sealed)
    with pytest.raises(CallSealError, match="not exactly once"):
        unseal_calls(
            "unsafe fn f() { __RUSTCONV_CALL_0000__(1); __RUSTCONV_CALL_0000__(2); }",
            sealed,
        )
    restored = unseal_calls(
        "unsafe fn f() { __RUSTCONV_CALL_0000__(1); /* __RUSTCONV_CALL_0000__ */ }",
        sealed,
    )
    assert "Call(1)" in restored
    assert "COMMENT_TOKEN_ECHO" in restored


def test_unseal_recovers_one_exact_prematurely_restored_direct_call():
    sealed = seal_calls("int f() { return Worker_body(1); }", {"Worker_body"})
    restored = unseal_calls("unsafe fn f() { Worker_body(1) }", sealed)
    assert "Worker_body(1)" in restored


def test_unseal_recovers_one_exact_sibling_body_spelling():
    sealed = seal_calls("int f() { return _ILGetDriveType(pidl); }", {"_ILGetDriveType"})
    restored = unseal_calls("unsafe fn f() { _ILGetDriveType_body(pidl) }", sealed)
    assert "_ILGetDriveType(pidl)" in restored
    assert "_body" not in restored


def test_unseal_does_not_recover_ambiguous_or_declared_callee_names():
    sealed = seal_calls("int f() { return Worker_body(1); }", {"Worker_body"})
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls(
            "unsafe fn f() { Worker_body(1); Worker_body(2); }",
            sealed,
        )
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls("unsafe fn Worker_body() {}", sealed)


def test_call_seal_skips_cpp_direct_initializer_declarator():
    source = "void f() { CComHeapPtr<ITEMIDLIST> pidl(SHCreate(path)); Use(pidl); }"
    sealed = seal_calls(source, {"pidl", "SHCreate", "Use"})
    assert [call.original for call in sealed.calls] == ["SHCreate", "Use"]
    assert "CComHeapPtr<ITEMIDLIST> pidl(" in sealed.text


def test_call_seal_skips_block_scope_extern_prototype():
    source = "BOOL f(PCWSTR path) { extern BOOL Probe(PCWSTR); return Probe(path); }"
    sealed = seal_calls(source, {"Probe"})
    assert [call.original for call in sealed.calls] == ["Probe"]
    assert "extern BOOL Probe(PCWSTR)" in sealed.text
    assert "return __RUSTCONV_CALL_0000__(path)" in sealed.text


def test_unseal_restores_one_csed_repeated_pure_length_call():
    sealed = seal_calls(
        "int f(WCHAR *a, WCHAR *b) { return wcslen(a) + wcslen(b) + wcslen(b); }",
        {"wcslen"},
    )
    restored = unseal_calls(
        "unsafe fn f(a: LPCWSTR, b: LPCWSTR) { "
        "let first = __RUSTCONV_CALL_0000__(a); "
        "let len = __RUSTCONV_CALL_0001__(b) as usize; Use(len); Advance(len); }",
        sealed,
    )
    assert restored.count("wcslen") == 3
    assert "Advance(wcslen(b))" in restored


def test_unseal_does_not_restore_csed_impure_calls():
    sealed = seal_calls("int f() { return Next() + Next(); }", {"Next"})
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls(
            "unsafe fn f() { let value = __RUSTCONV_CALL_0000__(); Use(value); Use(value); }",
            sealed,
        )


def test_unseal_relabels_repeated_token_when_same_callee_count_is_closed():
    sealed = seal_calls("int f() { return Wrap(a) + Wrap(b); }", {"Wrap"})
    restored = unseal_calls(
        "unsafe fn f() { __RUSTCONV_CALL_0000__(a) + __RUSTCONV_CALL_0000__(b) }",
        sealed,
    )
    assert restored.count("Wrap(") == 2


def test_unseal_allows_identical_switch_fallthrough_tail_in_else_if_arms():
    sealed = seal_calls(
        "int f(int x) { switch (x) { case 1: First(); case 2: Tail(7); break; } }",
        {"First", "Tail"},
    )
    restored = unseal_calls(
        "unsafe fn f(x: i32) { if x == 1 { __RUSTCONV_CALL_0000__(); "
        "__RUSTCONV_CALL_0001__(7); } else if x == 2 { __RUSTCONV_CALL_0001__(7); } }",
        sealed,
    )
    assert restored.count("Tail(7)") == 2
