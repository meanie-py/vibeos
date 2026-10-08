"""c_body_checks: a C goto/label that survived into Rust is caught; legal Rust is not."""

from rustconv.c_body_checks import control_flow_findings

C = """BOOLEAN f(void) {
    if (!x) goto Quit;
    y = 1;
Quit:
    return y;
}"""


def test_surviving_goto_and_label_are_findings():
    rust = "fn f() -> u8 {\n    if x == 0 { goto Quit; }\n    y = 1;\nQuit:\n    return y;\n}"
    found = control_flow_findings(C, rust)
    assert any("goto" in f for f in found) and any("Quit" in f for f in found)


def test_restructured_body_is_clean():
    rust = "fn f() -> u8 {\n    'quit: {\n        if x == 0 { break 'quit; }\n        y = 1;\n    }\n    y\n}"
    assert control_flow_findings(C, rust) == []


def test_a_struct_field_split_across_lines_is_not_a_label():
    c = "void g(void) { UNICODE_STRING s = { 0, 0, NULL }; }"
    rust = "fn g() {\n    let s = UNICODE_STRING {\n        Buffer:\n            core::ptr::null_mut(),\n    };\n}"
    assert control_flow_findings(c, rust) == []


def test_goto_in_a_comment_or_string_is_ignored():
    rust = 'fn f() {\n    // C: goto Quit;\n    let s = "goto Quit;";\n}'
    assert control_flow_findings(C, rust) == []


def test_a_body_that_does_not_parse_is_a_finding():
    from rustconv.c_body_checks import parse_findings

    assert parse_findings("unsafe fn f() -> u32 { let x: u32 = 1; x }") == []
    bad = 'unsafe fn f() { g(h, L"AddReg.NT" INF_ARCH, 0); }'
    assert parse_findings(bad) and "does not parse" in parse_findings(bad)[0]
    assert parse_findings("fn f() { (*p)->x = 1; }")


def test_an_invented_goto_shim_is_refused():
    c = "BOOL f(void) { if (x) goto NoSystemPartition; return TRUE; NoSystemPartition: return FALSE; }"
    for rust in ("fn f() { if x { goto_NoSystemPartition!(); } }", "fn f() { fn goto_Quit() {} }"):
        assert any("shim" in f for f in control_flow_findings(c, rust))


_DO_C = """{
    do {
        p = Next(p);
        if (Skip(p)) continue;
        Use(p);
        for (i = 0; i < 3; i++) { if (i) continue; }
    } while (p != NULL);
}"""


def test_a_continue_that_skips_the_do_while_condition_is_rejected():
    bad = """fn f() {
    loop {
        p = Next(p);
        if Skip(p) != 0 { continue; }
        Use(p);
        if !(p != core::ptr::null_mut()) { break; }
    }
}"""
    good = """fn f() {
    loop {
        'body: {
            p = Next(p);
            if Skip(p) != 0 { break 'body; }
            Use(p);
        }
        if !(p != core::ptr::null_mut()) { break; }
    }
}"""
    assert any("do-while condition" in f for f in control_flow_findings(_DO_C, bad))
    assert not any("do-while" in f for f in control_flow_findings(_DO_C, good))
    only_nested = _DO_C.replace("if (Skip(p)) continue;", "")
    assert not any("do-while" in f for f in control_flow_findings(only_nested, bad))


_STATUS_C = """{
    NTSTATUS Status;
    Status = GetFileSystemName_UStr(NULL, DeviceHandle, FileSystem, sizeof(FileSystem));
    if (!NT_SUCCESS(Status))
        goto Quit;
    if (!x)
        Status = STATUS_INVALID_DEVICE_REQUEST;
    for (i = 0; i < 3; i++) {}
    n = 0ULL;
Quit:
    return Status;
}"""


def test_an_invented_success_assignment_is_refused():
    # bootsup.c: the translation added `Status = 0; // keep as in C` on a failure path.
    bad = "fn f() {\n    if x { Status = 0; // keep as in C\n        return Status; }\n}"
    assert any("Status = 0" in f for f in control_flow_findings(_STATUS_C, bad))
    ok = "fn f() {\n    let mut Status: NTSTATUS = 0;\n    i = 0;\n    n = 0;\n    (*p).Status = 0;\n}"
    assert not any("invented" in f for f in control_flow_findings(_STATUS_C, ok))
    c_with_success = _STATUS_C.replace("STATUS_INVALID_DEVICE_REQUEST", "STATUS_SUCCESS")
    assert not any("invented" in f for f in control_flow_findings(c_with_success, bad))


def test_a_chained_c_assignment_gives_every_name_its_value():
    from rustconv.c_body_checks import invented_flag_assignments

    c = "Length = wcslen(d);\nPos = Length = 0;\n"
    rust = "Length = wcslen(d);\nLength = 0;\nPos = Length;\n"
    assert invented_flag_assignments(c, rust) == []
    assert invented_flag_assignments("Length = wcslen(d);\n", "Length = 0;\n")


def test_a_comma_operator_statement_is_several_assignments():
    """hotplug enum.c: `dwCapabilities = 0, dwSize = sizeof(dwCapabilities);`."""
    from rustconv.c_body_checks import invented_flag_assignments

    c = "dwCapabilities = 0,\n    dwSize = sizeof(dwCapabilities);\n"
    rust = "dwCapabilities = 0;\ndwSize = 4;\n"
    assert invented_flag_assignments(c, rust) == []
    assert invented_flag_assignments("x = f(a, b), y = 2;\n", "x = 0;\n")


def test_a_pointer_typedef_local_gets_no_extra_level():
    from rustconv.c_body_checks import pointer_typedef_indirection_findings as f

    c = "PFILEPATHS_W FilePathInfo;\nFilePathInfo = (PFILEPATHS_W)Param1;\n"
    assert f(c, "let FilePathInfo: *mut PFILEPATHS_W;")
    assert not f(c, "let FilePathInfo: PFILEPATHS_W;")
    two = "PCM_INT13_DRIVE_PARAMETER *Int13Drives = (PCM_INT13_DRIVE_PARAMETER*)Context;"
    assert not f(two, "let mut Int13Drives: *mut PCM_INT13_DRIVE_PARAMETER = Context.cast();")
