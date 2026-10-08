"""A sealed token in code the build never executes is permitted, never demanded.

Every case here is drawn from the Tier 3 batch A run (2026-07-25), where this class
rejected 10 of 56 functions -- each one a CORRECT translation -- and would have billed a
retry for every one of them.
"""

from pathlib import Path

import pytest

from rustconv.call_seal import CallSealError, seal_calls, unseal_calls
from rustconv.dead_regions import (
    dead_region_spans,
    dummy_pseh_except_spans,
    never_compiled_lines,
    pinned_int_macros,
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


# ---------------------------------------------------------------------------
# never_compiled_lines: keeping a body the build never sees OUT OF THE BILL.
#
# `translate` charges one paid call per function, so a function inside
# `#ifndef __REACTOS__` is a bill for code that exists in no object we produce.
# except_i386.c is 7 of 23 that way.
# ---------------------------------------------------------------------------

WINE_REACTOS_FORK = """\
#ifdef __i386__

static void live_one(void) { }

#ifndef __REACTOS__
void CDECL _global_unwind2(EXCEPTION_REGISTRATION_RECORD* frame)
{
    RtlUnwind( frame, 0, 0, 0 );
}
#endif

#ifdef __REACTOS__
void CDECL live_two(void);
#else
int CDECL _except_handler2(void)
{
    return 0;
}
#endif

#if SOMETHING_WE_CANNOT_SETTLE
void maybe_live(void) { }
#endif

#endif  /* __i386__ */
"""


def _line_of(text: str, needle: str) -> int:
    for n, line in enumerate(text.splitlines(), 1):
        if needle in line:
            return n
    raise AssertionError(f"{needle!r} not in the fixture")


def test_a_body_under_ifndef_REACTOS_is_never_compiled():
    dead = never_compiled_lines(WINE_REACTOS_FORK)
    assert _line_of(WINE_REACTOS_FORK, "void CDECL _global_unwind2") in dead


def test_the_else_arm_of_ifdef_REACTOS_is_never_compiled():
    """The other half of the same idiom, and the one a naive scan gets backwards."""
    dead = never_compiled_lines(WINE_REACTOS_FORK)
    assert _line_of(WINE_REACTOS_FORK, "int CDECL _except_handler2") in dead
    # ...while its live sibling arm is untouched.
    assert _line_of(WINE_REACTOS_FORK, "void CDECL live_two") not in dead


def test_an_undecidable_condition_leaves_its_body_LIVE():
    """Over-reporting live code costs money; under-reporting it drops a real body."""
    dead = never_compiled_lines(WINE_REACTOS_FORK)
    assert _line_of(WINE_REACTOS_FORK, "void maybe_live") not in dead


def test_unconditional_code_is_live():
    dead = never_compiled_lines(WINE_REACTOS_FORK)
    assert _line_of(WINE_REACTOS_FORK, "static void live_one") not in dead


def test_a_live_looking_inner_arm_nested_in_a_dead_one_is_still_dead():
    src = "#ifndef __REACTOS__\n#ifdef __REACTOS__\nvoid buried(void) { }\n#endif\n#endif\n"
    assert _line_of(src, "void buried") in never_compiled_lines(src)


# `-DNAME=<int>` the module puts on every compile. msvcrt pins `_MSVCR_VER=0`, which decides
# ~200 `#if _MSVCR_VER >= N` directives -- and two whole TUs, handler4.c and concurrency.c,
# whose objects carry zero defined symbols.
MSVCR_VER_GATED = """\
void live_at_any_version(void) { }

#if _MSVCR_VER >= 110
void __cdecl __crtCapturePreviousContext( CONTEXT *ctx ) { }
#endif

#if _MSVCR_VER < 140
void live_below_140(void) { }
#endif

#if defined(__x86_64__) && _MSVCR_VER>=140
void dead_by_one_false_conjunct(void) { }
#endif

#if _MSVCR_VER >= 100 || SOMETHING_UNKNOWN
void undecidable_disjunction(void) { }
#endif

#if _WIN32_WINNT >= 0x0600
void live_on_this_winnt(void) { }
#endif
"""

MSVCRT_PINS = {"_MSVCR_VER": 0, "_WIN32_WINNT": 0x0601}


def test_a_version_gated_body_is_dead_at_the_pinned_value():
    dead = never_compiled_lines(MSVCR_VER_GATED, MSVCRT_PINS)
    assert _line_of(MSVCR_VER_GATED, "__crtCapturePreviousContext") in dead


def test_the_same_body_is_LIVE_without_the_pins():
    """The pins are what decide it: no `-D` knowledge, no dead code, no silent drop."""
    dead = never_compiled_lines(MSVCR_VER_GATED)
    assert _line_of(MSVCR_VER_GATED, "__crtCapturePreviousContext") not in dead


def test_a_gate_the_pinned_value_SATISFIES_stays_live():
    dead = never_compiled_lines(MSVCR_VER_GATED, MSVCRT_PINS)
    assert _line_of(MSVCR_VER_GATED, "void live_below_140") not in dead
    assert _line_of(MSVCR_VER_GATED, "void live_at_any_version") not in dead
    assert _line_of(MSVCR_VER_GATED, "void live_on_this_winnt") not in dead


def test_one_false_conjunct_kills_the_block_however_unknown_its_neighbour():
    """handler4.c's `defined(__x86_64__) && _MSVCR_VER>=140` — dead on all three arches."""
    dead = never_compiled_lines(MSVCR_VER_GATED, MSVCRT_PINS)
    assert _line_of(MSVCR_VER_GATED, "dead_by_one_false_conjunct") in dead


def test_a_disjunction_with_an_unknown_term_stays_live():
    dead = never_compiled_lines(MSVCR_VER_GATED, MSVCRT_PINS)
    assert _line_of(MSVCR_VER_GATED, "undecidable_disjunction") not in dead


def test_pinned_int_macros_reads_values_not_just_names():
    got = pinned_int_macros(["-D_MSVCR_VER=0", "-DWINVER=0x601", "-DN=100L", "-DBARE", "-DX=str"])
    assert got == {"_MSVCR_VER": 0, "WINVER": 0x601, "N": 100}


def test_sibling_header_pins_a_bare_if_macro():
    """The MERGE_FOLDERS case: a decidable conditional whose macro lives in a header.

    `pinned_int_macros` reads the module's `-D` list and so called this undecidable, which
    made the whole `#if/#else` one dead span, made the live arm's tokens optional, and let a
    body that translated the `#else` pass the seal. The shipped result returned the USER
    start menu unmerged.
    """
    from rustconv.dead_regions import conditional_arm_notes, sibling_header_int_macros

    root = Path(__file__).resolve().parents[3]
    start_menu = root / "dll/win32/shell32/shellmenu/CStartMenu.cpp"
    if not start_menu.is_file():  # pragma: no cover - tree layout guard
        pytest.skip("shellmenu sources not present")

    pinned = sibling_header_int_macros(start_menu)
    assert pinned["MERGE_FOLDERS"] == 1

    body = start_menu.read_text(encoding="utf-8", errors="replace")
    start = body.index("static HRESULT GetMergedFolder")
    notes = conditional_arm_notes(body[start : body.index("\n}", start)], pinned)
    assert any("MERGE_FOLDERS" in n and "#if arm is LIVE" in n for n in notes)

    # Undecidable stays undecidable rather than guessing an arm.
    unknown = conditional_arm_notes("#if NOT_A_PINNED_MACRO\nx\n#endif\n", pinned)
    assert unknown and "undecidable" in unknown[0]


def test_sibling_header_macros_drop_conflicting_definitions():
    """Two headers, two values, no answer -- a guess here would mark live code dead."""
    import tempfile

    from rustconv.dead_regions import sibling_header_int_macros

    with tempfile.TemporaryDirectory() as tmp:
        folder = Path(tmp)
        (folder / "a.h").write_text("#define WOBBLE 1\n#define STEADY 7\n")
        (folder / "b.h").write_text("#define WOBBLE 2\n")
        pinned = sibling_header_int_macros(folder / "x.cpp")
    assert "WOBBLE" not in pinned
    assert pinned["STEADY"] == 7


def test_a_bare_pinned_macro_is_a_truth_test():
    """timedate.c: `#if DBG` opens one definition header and `#else` the other."""
    src = "#if DBG\nVOID FooDbg(int x)\n#else\nVOID Foo(int x)\n#endif\n{\n}\n"
    assert never_compiled_lines(src, {"DBG": 1}) == {4}
    assert never_compiled_lines(src, {"DBG": 0}) == {2}
    assert never_compiled_lines(src) == set()
    assert never_compiled_lines("#if !DBG\nint a;\n#endif\n", {"DBG": 1}) == {2}
    assert never_compiled_lines("#ifdef DBG\nint a;\n#endif\n", {"DBG": 0}) == set()
