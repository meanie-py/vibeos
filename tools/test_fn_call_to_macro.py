"""fn_call_to_macro: the rewrite that turned ~1,200 out-of-line debug-macro calls back
into macro calls. The cases that matter are the refusals -- a call it cannot convert
faithfully must stay a call, and its extern declaration must stay with it."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools import fn_call_to_macro as m


def test_self_test_passes():
    assert m.self_test() == 0


def test_a_refused_call_keeps_its_extern_declaration(tmp_path):
    f = tmp_path / "a.rs"
    f.write_text(
        'extern "C" {\n    fn TRACE(fmt: *const i8, ...);\n}\n'
        "fn f(fmt: *const i8) {\n    TRACE(fmt);\n"
        '    TRACE(b"ok %d\\n\\0".as_ptr() as *const i8, 1);\n}\n'
    )
    converted, refused = m.process(f, {"TRACE"}, True, False, import_=False)
    out = f.read_text()
    assert converted == 1 and [n for _, n in refused] == ["TRACE"]
    assert "fn TRACE(fmt: *const i8, ...);" in out, "a refused call would lose its callee"
    assert 'TRACE!("ok %d\\n", 1);' in out


def test_converted_file_drops_the_declaration_and_imports_the_macro(tmp_path):
    f = tmp_path / "b.rs"
    f.write_text(
        '//! doc\nextern "C" {\n    fn ERR(fmt: *const i8, ...);\n    fn Keep(x: i32);\n}\n'
        'fn f() { ERR(b"x\\0".as_ptr() as *const i8); }\n'
    )
    m.process(f, {"ERR"}, True, False)
    out = f.read_text()
    assert "fn ERR(" not in out and "fn Keep(x: i32);" in out
    assert out.splitlines()[1] == "#[allow(unused_imports)] use crate::{ERR};"


def test_a_dependency_macro_needs_no_import(tmp_path):
    f = tmp_path / "c.rs"
    f.write_text("fn f(p: usize) { WPP_CLEANUP(p); }\n")
    m.process(f, {"WPP_CLEANUP"}, False, False, prefix="ntkrnl_sys::")
    assert f.read_text() == "fn f(p: usize) { ntkrnl_sys::WPP_CLEANUP!(p); }\n"
