"""A sealed token in code the build never executes is permitted, never demanded.

Every case here is drawn from the Tier 3 batch A run (2026-07-25), where this class
rejected 10 of 56 functions -- each one a CORRECT translation -- and would have billed a
retry for every one of them.
"""

import pytest
from rustconv.call_seal import CallSealError, seal_calls, unseal_calls
from rustconv.dead_regions import (
    dead_region_spans,
    dummy_pseh_except_spans,
    preprocessor_conditional_spans,
)
from rustconv.fact_seal import FactSealError, prepare_sealed_source, unseal_facts

# snmpapi/main.c DllMain, trimmed to the shape that matters.
SNMPAPI_DLLMAIN = """BOOL WINAPI DllMain(HINSTANCE hInstDLL, DWORD fdwReason, LPVOID lpvReserved)
{
    switch(fdwReason) {
#ifndef __REACTOS__
    case DLL_WINE_PREATTACH:
        return FALSE;  /* prefer native version */
#endif
    case DLL_PROCESS_ATTACH:
        DisableThreadLibraryCalls(hInstDLL);
        break;
    }

    return TRUE;
}"""

# wlanapi/main.c WlanCloseHandle, the shape all 9 rejected RPC clients share.
WLANAPI_RPC_CLIENT = """WlanCloseHandle(IN HANDLE hClientHandle, PVOID pReserved)
{
    DWORD dwResult = ERROR_SUCCESS;

    RpcTryExcept
    {
        dwResult = _RpcCloseHandle(&hClientHandle);
    }
    RpcExcept(EXCEPTION_EXECUTE_HANDLER)
    {
        dwResult = WlanRpcStatusToWinError(RpcExceptionCode());
    }
    RpcEndExcept;

    return dwResult;
}"""

RPC_CALLEES = {"_RpcCloseHandle", "WlanRpcStatusToWinError", "RpcExceptionCode"}


# --- span detection --------------------------------------------------------


def test_preprocessor_arm_is_dead():
    spans = preprocessor_conditional_spans(SNMPAPI_DLLMAIN)
    assert len(spans) == 1
    covered = SNMPAPI_DLLMAIN[spans[0][0] : spans[0][1]]
    assert "DLL_WINE_PREATTACH" in covered
    assert "DLL_PROCESS_ATTACH" not in covered


def test_nested_conditionals_collapse_to_the_outermost_span():
    text = "a\n#ifdef A\nb\n#ifdef B\nc\n#endif\nd\n#endif\ne\n"
    spans = preprocessor_conditional_spans(text)
    assert len(spans) == 1
    # Exactly the outer #ifdef line through the outer #endif line -- the code before
    # and after the conditional stays live.
    assert text[spans[0][0] : spans[0][1]] == "#ifdef A\nb\n#ifdef B\nc\n#endif\nd\n#endif"


def test_unterminated_conditional_runs_to_end_of_text():
    """A truncated body must leave its tokens OPTIONAL, not silently required."""
    text = "x = A;\n#ifdef NEVER\ny = B;\n"
    spans = preprocessor_conditional_spans(text)
    assert len(spans) == 1 and spans[0][1] == len(text)


def test_stray_endif_does_not_open_a_span():
    assert preprocessor_conditional_spans("a\n#endif\nb\n") == []


def test_rpc_except_block_is_dead():
    spans = dummy_pseh_except_spans(WLANAPI_RPC_CLIENT)
    assert len(spans) == 1
    covered = WLANAPI_RPC_CLIENT[spans[0][0] : spans[0][1]]
    assert "WlanRpcStatusToWinError" in covered
    assert "_RpcCloseHandle" not in covered, "the TRY body is live and must stay required"


def test_except_named_only_in_a_comment_opens_nothing():
    text = "{\n    /* __except (EXCEPTION_EXECUTE_HANDLER) { x(); } */\n    y();\n}"
    from rustconv.call_seal import _code_mask

    assert dummy_pseh_except_spans(text, _code_mask(text)) == []


def test_unbraced_except_is_not_treated_as_a_block():
    assert dummy_pseh_except_spans("__except(1) return 0;") == []


def test_clean_body_has_no_dead_spans():
    assert dead_region_spans("int f(void) { return g(); }") == []


# --- fact envelope ---------------------------------------------------------


def test_values_only_in_a_disabled_arm_are_optional():
    sealed = prepare_sealed_source(SNMPAPI_DLLMAIN, dialect="c")
    by_name = {fact.original: fact for fact in sealed.facts}
    assert by_name["DLL_WINE_PREATTACH"].dead
    assert by_name["FALSE"].dead
    assert not by_name["DLL_PROCESS_ATTACH"].dead
    assert not by_name["TRUE"].dead

    live = [fact.placeholder for fact in sealed.facts if not fact.dead]
    rust = "pub unsafe fn DllMain() -> BOOL {\n"
    rust += "".join(f"    let _ = {token};\n" for token in live)
    rust += "}\n"

    omissions: list[str] = []
    out = unseal_facts(rust, sealed, dead_omissions=omissions)
    assert "DLL_PROCESS_ATTACH" in out
    assert sorted(omissions) == [
        "__RUSTCONV_FACT_0000__=DLL_WINE_PREATTACH",
        "__RUSTCONV_FACT_0001__=FALSE",
    ]


def test_dropping_a_LIVE_value_still_fails_loudly():
    sealed = prepare_sealed_source(SNMPAPI_DLLMAIN, dialect="c")
    dead = [fact.placeholder for fact in sealed.facts if fact.dead]
    rust = "pub unsafe fn DllMain() {" + "".join(f" let _ = {t};" for t in dead) + " }"
    with pytest.raises(FactSealError, match="missing"):
        unseal_facts(rust, sealed)


def test_an_invented_value_is_still_rejected_inside_a_dead_arm():
    """Relaxing 'required' must not relax 'closed'."""
    sealed = prepare_sealed_source(SNMPAPI_DLLMAIN, dialect="c")
    live = [fact.placeholder for fact in sealed.facts if not fact.dead]
    rust = "pub unsafe fn DllMain() {"
    rust += "".join(f" let _ = {t};" for t in live)
    rust += " let _ = __RUSTCONV_FACT_0099__; }"
    with pytest.raises(FactSealError, match="invented"):
        unseal_facts(rust, sealed)


# --- call envelope ---------------------------------------------------------


def test_calls_only_in_an_rpc_except_handler_are_optional():
    sealed = seal_calls(WLANAPI_RPC_CLIENT, set(RPC_CALLEES))
    by_name = {call.original: call for call in sealed.calls}
    assert not by_name["_RpcCloseHandle"].dead
    assert by_name["WlanRpcStatusToWinError"].dead
    assert by_name["RpcExceptionCode"].dead

    live = by_name["_RpcCloseHandle"].placeholder
    rust = f"pub unsafe fn WlanCloseHandle() -> DWORD {{ {live}(core::ptr::null_mut()) }}"
    omissions: list[str] = []
    out = unseal_calls(rust, sealed, dead_omissions=omissions)
    assert "_RpcCloseHandle" in out
    assert len(omissions) == 2


def test_dropping_the_LIVE_rpc_stub_call_still_fails_loudly():
    """The whole point: omitting the actual RPC call is still a hard error."""
    sealed = seal_calls(WLANAPI_RPC_CLIENT, set(RPC_CALLEES))
    rust = "pub unsafe fn WlanCloseHandle() -> DWORD { 0 }"
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls(rust, sealed)


def test_no_dead_regions_means_every_call_stays_required():
    body = "int f(void)\n{\n    return g(h());\n}"
    sealed = seal_calls(body, {"g", "h"})
    assert [call.dead for call in sealed.calls] == [False, False]
    with pytest.raises(CallSealError):
        unseal_calls("pub unsafe fn f() -> i32 { 0 }", sealed)
