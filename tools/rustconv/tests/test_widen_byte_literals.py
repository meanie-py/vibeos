"""widen_byte_literals: both spellings of a mis-translated C `L"..."`, and the oracle."""

import importlib.util
import pathlib
import sys

_MOD = pathlib.Path(__file__).resolve().parents[1] / "widen_byte_literals.py"
_spec = importlib.util.spec_from_file_location("widen_byte_literals", _MOD)
assert _spec and _spec.loader
wbl = importlib.util.module_from_spec(_spec)
sys.modules["widen_byte_literals"] = wbl
_spec.loader.exec_module(wbl)


def test_decode_units_plain():
    assert wbl.decode_units("'A' as u16, 'b' as u16, 0") == "Ab"


def test_decode_units_strips_exactly_one_terminator():
    # `L"Terminal Server\0"` is double-NUL-terminated: the FIRST trailing zero is the string's
    # own, the second is the array terminator. Eating both changes what the callee reads.
    assert wbl.decode_units("'a' as u16, 0, 0") == "a\0"


def test_decode_units_empty_wide_string():
    assert wbl.decode_units("0") == ""


def test_decode_units_escapes():
    assert wbl.decode_units(r"'\\' as u16, 'n' as u16, 0") == "\\n"
    assert wbl.decode_units(r"'\n' as u16, 0") == "\n"


def test_decode_units_rejects_non_string():
    assert wbl.decode_units("100, 200") is None or wbl.decode_units("100, 200") == "\x64\xc8"


def test_numeric_array_is_not_touched(tmp_path):
    rs = tmp_path / "m.rs"
    rs.write_text("let p = [0u8; 4];\nlet q = [1, 2, 3].as_ptr();\n")
    n, _notes = wbl.convert(rs, "", apply=True)
    assert n == 0
    assert "[1, 2, 3].as_ptr()" in rs.read_text()


def test_elementwise_array_becomes_w_macro(tmp_path):
    rs = tmp_path / "m.rs"
    rs.write_text("static X: LPCWSTR = ['A' as u16, 'b' as u16, 0].as_ptr();\n")
    n, _notes = wbl.convert(rs, 'static const WCHAR x[] = L"Ab";', apply=True)
    assert n == 1
    assert rs.read_text() == 'static X: LPCWSTR = w!("Ab");\n'


def test_multiline_elementwise_array(tmp_path):
    rs = tmp_path / "m.rs"
    rs.write_text("let p = [\n    'H' as u16, 'i' as u16,\n    0,\n].as_ptr();\n")
    n, _ = wbl.convert(rs, 'L"Hi"', apply=True)
    assert n == 1
    assert rs.read_text() == 'let p = w!("Hi");\n'


def test_oracle_refuses_a_string_the_c_never_spells(tmp_path):
    rs = tmp_path / "m.rs"
    src = "let p = ['A' as u16, 'b' as u16, 0].as_ptr();\n"
    rs.write_text(src)
    n, notes = wbl.convert(rs, 'L"Something else"', apply=True)
    assert n == 0
    assert rs.read_text() == src
    assert any("never spells" in s for s in notes)


def test_empty_string_is_oracle_checked_too(tmp_path):
    rs = tmp_path / "m.rs"
    rs.write_text("let p = [0].as_ptr();\n")
    n, _ = wbl.convert(rs, 'const WCHAR *e = L"";', apply=True)
    assert n == 1
    assert rs.read_text() == 'let p = w!("");\n'

    rs2 = tmp_path / "n.rs"
    rs2.write_text("let p = [0].as_ptr();\n")
    n2, _ = wbl.convert(rs2, "int zero = 0;", apply=True)
    assert n2 == 0


def test_byte_literal_shape_still_works(tmp_path):
    rs = tmp_path / "m.rs"
    rs.write_text('let p = b"Foo\\0".as_ptr() as LPCWSTR;\n')
    n, _ = wbl.convert(rs, 'L"Foo"', apply=True)
    assert n == 1
    assert rs.read_text() == 'let p = w!("Foo") as LPCWSTR;\n'
