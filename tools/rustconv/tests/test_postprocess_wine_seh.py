from rustconv.postprocess import lower_wine_seh_markers


def test_markers_dropped_and_handler_guard_lowered():
    # crypt32 encode.c, bought while the fact seal hid __TRY behind a placeholder
    src = (
        "fn f() -> BOOL {\n"
        "    let mut ret: BOOL = 0;\n"
        "    let _ = __TRY;\n"
        "    let _seh2_try_marker: i32 = __TRY;\n"
        "    {\n        ret = g();\n    }\n"
        "    __EXCEPT_PAGE_FAULT;\n"
        "    if __EXCEPT_PAGE_FAULT {\n        SetLastError(STATUS_ACCESS_VIOLATION);\n    }\n"
        "    if _SEH2_Volatile0 != 0\n        || __EXCEPT_PAGE_FAULT as c_int != 0 && x\n    {}\n"
        "    __ENDTRY;\n"
        "    return ret;\n}\n"
    )
    out = lower_wine_seh_markers(src)
    assert "__TRY" not in out and "__ENDTRY" not in out and "__EXCEPT" not in out
    assert "if _SEH2_Volatile0 != 0 {\n        SetLastError" in out
    assert "|| _SEH2_Volatile0 as c_int != 0" in out
    assert "ret = g();" in out and "return ret;" in out


def test_untouched_without_markers():
    src = "fn f() { let x = __TRY_COUNT; }\n"
    assert lower_wine_seh_markers(src) == src
