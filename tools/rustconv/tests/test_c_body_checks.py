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
