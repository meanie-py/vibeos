"""wined3d: nine bodies rejected because wine's `priv` is a reserved Rust keyword."""

from rustconv.c_body_checks import parse_findings
from rustconv.reserved_idents import raw_reserved_keywords


def test_a_reserved_keyword_identifier_becomes_raw_and_parses():
    body = (
        'unsafe extern "C" fn shader_sm5_read_sync(ins: *mut wined3d_shader_instruction,\n'
        "    priv: *mut wined3d_sm4_data) {\n"
        "    let r#box = 1; let final = priv; (*final).x = r#box;\n}\n"
    )
    assert parse_findings(body)
    got = raw_reserved_keywords(body)
    assert "r#priv: *mut" in got and "let r#final = r#priv;" in got and "r#r#" not in got
    assert parse_findings(got) == []


def test_fields_strings_comments_and_macros_are_left_alone():
    body = "// priv here\nlet s = \"priv box\"; let c = 'b'; x.priv_ = y.priv; macro_rules! m {}\n"
    assert raw_reserved_keywords(body) == body
