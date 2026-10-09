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


# ------------------------------------------------------- the shifted envelope (netcfgx)


NETCFGX_SHIFT_C = """
void ApplyOrCancelChanges(Item *pHead, const CLSID *lpClassGUID) {
    if (IsEqualGUID(lpClassGUID, &GUID_DEVCLASS_NET)) {
        if (StringFromCLSID(&pHead->InstanceId, &pszGuid) == NOERROR) {
            swprintf(szName, L"Control\\\\Network\\\\%s", pszGuid);
            CoTaskMemFree(pszGuid);
        }
    }
}
"""


def _netcfgx_shift_sealed():
    return seal_calls(
        NETCFGX_SHIFT_C,
        {"IsEqualGUID", "StringFromCLSID", "swprintf", "CoTaskMemFree"},
        frozenset({"IsEqualGUID"}),
    )


def test_a_function_like_macro_is_named_in_the_envelope_not_just_disclaimed():
    """SHIPPED BUG: the envelope said "a call inside a function-like macro has NO token"
    and left the model to work out WHICH. With three windowsx.h macros interleaved among
    the sealed calls, `ReadParameterValue` tokenized them too and invented three numbers;
    `ApplyOrCancelChanges` renumbered from the macro and shifted the whole body by one."""
    from rustconv.call_seal import call_contract

    sealed = _netcfgx_shift_sealed()
    assert sealed.macros == ("IsEqualGUID",)
    contract = call_contract(sealed)
    assert "function-like macros, not sealed callees: IsEqualGUID" in contract
    # `sizeof` is a keyword and the debug macros are stripped, so neither is worth naming.
    quiet = seal_calls("int f(void) { Log(1); return sizeof(int) + Call(x); }", {"Call", "sizeof"})
    assert quiet.macros == ()


def test_a_shifted_envelope_is_caught_by_the_argument_count():
    """The shift passes every count-based check — right number of tokens, each exactly
    once, none invented — because it is a permutation. Only the ARGUMENTS disagree: the
    model consumed CALL_0000 for the macro, so `StringFromCLSID`'s two arguments landed
    under `swprintf`'s three-argument token and `swprintf`'s three under `CoTaskMemFree`."""
    sealed = _netcfgx_shift_sealed()
    shifted = """unsafe fn f() {
        if __RUSTCONV_CALL_0000__(lpClassGUID, &GUID_DEVCLASS_NET) != 0 {
            if __RUSTCONV_CALL_0001__(&(*pHead).InstanceId, &mut pszGuid) == 0 {
                __RUSTCONV_CALL_0002__(szName, w!("Control\\\\Network\\\\%s"), pszGuid);
                __RUSTCONV_CALL_0003__(pszGuid);
            }
        }
    }"""
    with pytest.raises(CallSealError) as excinfo:
        unseal_calls(shifted, sealed)
    assert "argument count" in str(excinfo.value)
    assert "__RUSTCONV_CALL_0000__=StringFromCLSID wants 2, got 2" not in str(excinfo.value)
    assert "__RUSTCONV_CALL_0001__=swprintf wants 3, got 2" in str(excinfo.value)


def test_the_faithful_translation_of_the_same_body_still_passes():
    """The other half of the guard: the macro written as an ordinary named call, every
    token on its own site. Nothing here may be a finding."""
    sealed = _netcfgx_shift_sealed()
    faithful = """unsafe fn f() {
        if IsEqualGUID(lpClassGUID, &GUID_DEVCLASS_NET) != 0 {
            if __RUSTCONV_CALL_0000__(&(*pHead).InstanceId, &mut pszGuid) == 0 {
                __RUSTCONV_CALL_0001__(szName, w!("Control\\\\Network\\\\%s"), pszGuid);
                __RUSTCONV_CALL_0002__(pszGuid);
            }
        }
    }"""
    restored = unseal_calls(faithful, sealed)
    assert "StringFromCLSID(&(*pHead).InstanceId" in restored
    assert "CoTaskMemFree(pszGuid)" in restored


def test_a_macro_in_argument_position_is_never_counted():
    """shell32's `IID_PPV_ARG(IProgressDialog, &ppv)` is ONE C argument and TWO Rust
    ones, so counting that site would reject a correct translation."""
    sealed = seal_calls(
        "void f(void) { CoCreateInstance(clsid, NULL, ctx, IID_PPV_ARG(IFoo, &p)); }",
        {"CoCreateInstance", "IID_PPV_ARG"},
        frozenset({"IID_PPV_ARG"}),
    )
    assert [call.argc for call in sealed.calls] == [None]
    restored = unseal_calls(
        "unsafe fn f() { __RUSTCONV_CALL_0000__(clsid, null_mut(), ctx, &IID_IFoo, &mut p); }",
        sealed,
    )
    assert "CoCreateInstance(clsid" in restored


def test_a_turbofish_argument_is_not_a_second_argument():
    sealed = seal_calls("void f(void) { Take(x); }", {"Take"})
    restored = unseal_calls(
        "unsafe fn f() { __RUSTCONV_CALL_0000__(core::mem::transmute::<A, B>(x)); }", sealed
    )
    assert "Take(core::mem::transmute::<A, B>(x))" in restored


def test_one_agreeing_occurrence_is_enough_inside_the_goto_region():
    """A `goto` lowering replicates the cleanup tail, and the copies are the same call."""
    sealed = seal_calls(
        "int f(void) { if (a) goto done; Work(1, 2); done: Close(h); return 0; }",
        {"Work", "Close"},
    )
    restored = unseal_calls(
        "unsafe fn f() -> i32 { if a { __RUSTCONV_CALL_0001__(h); return 0; } "
        "__RUSTCONV_CALL_0000__(1, 2); __RUSTCONV_CALL_0001__(h); 0 }",
        sealed,
    )
    assert restored.count("Close(h)") == 2


def test_a_rustfmt_trailing_comma_is_not_an_argument():
    """rustfmt puts a trailing comma on every multi-line call, and the first cut of the
    arity check read a six-argument `RegQueryValueExW` spread over seven lines as seven
    arguments — which would have flagged 68 already-accepted bodies."""
    sealed = seal_calls("void f(void) { Reg(a, b, c); }", {"Reg"})
    restored = unseal_calls(
        "unsafe fn f() {\n    __RUSTCONV_CALL_0000__(\n        a,\n        b,\n        c,\n    );\n}",
        sealed,
    )
    assert "Reg(" in restored


def test_an_empty_argument_list_counts_as_zero_not_one():
    sealed = seal_calls("void f(void) { Now(); }", {"Now"})
    with pytest.raises(CallSealError, match="argument count"):
        unseal_calls("unsafe fn f() { __RUSTCONV_CALL_0000__(x); }", sealed)
    assert "Now()" in unseal_calls("unsafe fn f() { __RUSTCONV_CALL_0000__(); }", sealed)


def test_a_switch_arm_that_falls_through_may_repeat_its_tokens():
    """Rust `match` has no fall-through, so the only faithful lowering of a C arm that
    runs on into the next one is to DUPLICATE the next arm's body — which writes every
    token in it twice. ntmarta's `AccpGetTrusteeSid` did exactly that and had all five
    of its tokens rejected, throwing away a whole paid TU."""
    c = """
    int f(int k) {
        switch (k) {
            case A:
                if (bad) { break; }
                /* fall through */
            case B:
                return Lookup(k);
        }
        return 0;
    }
    """
    sealed = seal_calls(c, {"Lookup"})
    # The duplication the C forces: `Lookup` written once per arm.
    rust = (
        "unsafe fn f(k: i32) -> i32 {\n"
        "    match k {\n"
        "        A => { if bad { } else { return __RUSTCONV_CALL_0000__(k); } }\n"
        "        B => { return __RUSTCONV_CALL_0000__(k); }\n"
        "        _ => {}\n"
        "    }\n"
        "    0\n}"
    )
    assert unseal_calls(rust, sealed).count("Lookup(k)") == 2


def test_a_shared_case_label_is_not_a_fallthrough():
    """`case A: case B:` duplicates nothing — the arms are already one block — so it must
    not buy the body a licence to repeat tokens."""
    c = """
    int f(int k) {
        switch (k) {
            case A:
            case B:
                return Lookup(k);
            default:
                return 0;
        }
    }
    """
    sealed = seal_calls(c, {"Lookup"})
    with pytest.raises(CallSealError):
        unseal_calls(
            "unsafe fn f(k: i32) -> i32 {\n"
            "    match k {\n"
            "        A => __RUSTCONV_CALL_0000__(k),\n"
            "        B => __RUSTCONV_CALL_0000__(k),\n"
            "        _ => 0,\n"
            "    }\n}",
            sealed,
        )


def test_a_turbofish_in_an_argument_does_not_disable_the_arity_check():
    """thread.c's `_beginthread_trampoline` shipped through exactly this hole.

    `msvcrt_get_thread_data` takes ZERO arguments. The model slid its token onto the
    `memcpy` site, where it wore three -- one of them `core::mem::size_of::<T>()`. The
    old rule skipped any site whose arguments mentioned `::<`, so the disagreement was
    never compared, and a thread that leaks its argument block and never calls the user's
    entry point passed the envelope.
    """
    sealed = seal_calls("void f(void) { get(); }", {"get"})
    with pytest.raises(CallSealError, match="argument count"):
        unseal_calls(
            "unsafe fn f() { __RUSTCONV_CALL_0000__(&mut local as *mut _, arg, "
            "core::mem::size_of::<T>()); }",
            sealed,
        )


def test_a_turbofish_with_its_own_comma_still_counts_as_one_argument():
    """The commas that belong to `::<A, B>` are blanked, not the site."""
    sealed = seal_calls("void f(void) { Take(x); }", {"Take"})
    restored = unseal_calls(
        "unsafe fn f() { __RUSTCONV_CALL_0000__(core::mem::transmute::<A, B>(x)); }", sealed
    )
    assert "Take(core::mem::transmute::<A, B>(x))" in restored


def test_a_nested_turbofish_does_not_close_early():
    sealed = seal_calls("void f(void) { Take(x); }", {"Take"})
    restored = unseal_calls(
        "unsafe fn f() { __RUSTCONV_CALL_0000__(transmute::<Option<A>, fn(B) -> C>(x)); }",
        sealed,
    )
    assert "Take(transmute::<Option<A>, fn(B) -> C>(x))" in restored


def test_a_function_pointer_local_may_be_spelled_twice():
    """input layout_list.c: `FN_X fn; ... if (fn) hr = fn(a);` -- `fn` is storage."""
    from rustconv.call_seal import seal_calls, unseal_calls

    body = "HRESULT f(void)\n{\n    FN_X fn;\n    if (fn) hr = fn(a);\n    return hr;\n}\n"
    sealed = seal_calls(body, {"fn"})
    (call,) = sealed.calls
    tok = call.placeholder
    out = unseal_calls(f"if let Some({tok}) = r#fn {{ hr = {tok}(a); }}", sealed)
    assert out == "if let Some(r#fn) = r#fn { hr = r#fn(a); }"
