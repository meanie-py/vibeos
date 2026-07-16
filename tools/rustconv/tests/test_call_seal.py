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
    with pytest.raises(CallSealError, match="not exactly once"):
        unseal_calls(
            "unsafe fn f() { __RUSTCONV_CALL_0000__(1); /* __RUSTCONV_CALL_0000__ */ }",
            sealed,
        )


def test_unseal_recovers_one_exact_prematurely_restored_direct_call():
    sealed = seal_calls("int f() { return Worker_body(1); }", {"Worker_body"})
    restored = unseal_calls("unsafe fn f() { Worker_body(1) }", sealed)
    assert "Worker_body(1)" in restored


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
