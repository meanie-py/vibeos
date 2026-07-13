"""Tests for table_audit — the file-scope DATA TABLE oracle.

Every case below is a false positive or false negative this module actually
produced while being calibrated against shipped ole32/oleaut32 code. A data-table
auditor that cries wolf gets ignored, which is worse than not having one, so the
false-positive cases matter as much as the detection cases.
"""

from __future__ import annotations

import pytest

from rustconv.table_audit import (
    TableParseError,
    audit_tables,
    normalize_element,
    parse_c_tables,
    parse_rust_tables,
)


def _c(tmp_path, body: str):
    p = tmp_path / "t.c"
    p.write_text(body)
    return p


# --------------------------------------------------------------------------
# detection — the bugs it exists to catch
# --------------------------------------------------------------------------


def test_detects_a_wrong_element(tmp_path):
    c = _c(tmp_path, "static const BYTE fmt[3] = { 0x1, 0x2, 0x3 };\n")
    # 0x2 -> 0x9: the single mistranscribed byte that silently changes behavior
    diffs = audit_tables(c, "static fmt: [u8; 3] = [0x1, 0x9, 0x3];")
    (d,) = [d for d in diffs if d.kind != "ok"]
    assert d.kind == "value"
    assert "[1]" in d.detail


def test_detects_a_dropped_table(tmp_path):
    c = _c(tmp_path, "static const BYTE fmt[3] = { 1, 2, 3 };\n")
    (d,) = [d for d in audit_tables(c, "// nothing here") if d.kind != "ok"]
    assert d.kind == "missing"


def test_detects_a_truncated_table(tmp_path):
    c = _c(tmp_path, "static const BYTE fmt[4] = { 1, 2, 3, 4 };\n")
    (d,) = [d for d in audit_tables(c, "static fmt: [u8; 3] = [1, 2, 3];") if d.kind != "ok"]
    assert d.kind == "count"


def test_c_zero_fill_is_modelled(tmp_path):
    """C fills the tail with zeros; Rust must spell them. Both mistakes surface."""
    c = _c(tmp_path, "static const BYTE t[4] = { 7, 8 };\n")  # C: {7,8,0,0}
    assert [d.kind for d in audit_tables(c, "static t: [u8; 4] = [7, 8, 0, 0];")] == ["ok"]
    # a translation that only copied the written elements is a real bug
    (d,) = [d for d in audit_tables(c, "static t: [u8; 2] = [7, 8];") if d.kind != "ok"]
    assert d.kind == "count"


# --------------------------------------------------------------------------
# false positives — every one of these was observed against real shipped code
# --------------------------------------------------------------------------


def test_function_local_statics_are_ignored(tmp_path):
    """wine's variant.c has a DIFFERENT `prio2vt` inside VarAdd and VarMul.

    Scooping up function-local statics made the last one win, so VarMul's C table
    got compared against VarAdd's Rust one and reported a bug that did not exist.
    """
    c = _c(
        tmp_path,
        "void f(void) {\n  static const BYTE t[2] = { 1, 2 };\n}\n"
        "void g(void) {\n  static const BYTE t[3] = { 9, 9, 9 };\n}\n",
    )
    assert parse_c_tables(c) == {}


def test_duplicate_file_scope_names_raise_rather_than_guess(tmp_path):
    c = _c(tmp_path, "static const BYTE t[1] = { 1 };\nstatic const BYTE t[1] = { 2 };\n")
    with pytest.raises(TableParseError, match="refusing to guess"):
        parse_c_tables(c)


def test_dead_preprocessor_branch_is_not_compared(tmp_path):
    """ole32's compobj.c defines classes_rootW twice; #ifdef __REACTOS__ picks the
    first. A preprocessor-blind audit read the dead Wine branch and reported 7
    bogus 'wrong character' diffs against correct code."""
    c = _c(
        tmp_path,
        "#ifdef __REACTOS__\n"
        "static const BYTE t[2] = { 1, 2 };\n"
        "#else\n"
        "static const BYTE t[2] = { 9, 9 };\n"
        "#endif\n",
    )
    assert parse_c_tables(c)["t"] == ["1", "2"]


def test_undecidable_condition_keeps_the_branch(tmp_path):
    """Never blank code we cannot prove dead."""
    c = _c(tmp_path, "#if SOME_UNKNOWN_THING\nstatic const BYTE t[1] = { 5 };\n#endif\n")
    assert parse_c_tables(c)["t"] == ["5"]


def test_multiline_comments_do_not_shift_line_numbers(tmp_path):
    """Collapsing /* */ to one space desynced offsets from the scanner's function
    line-spans, so the function-local test read the wrong lines."""
    c = _c(
        tmp_path,
        "/* a\n * multi-line\n * banner\n */\nstatic const BYTE t[2] = { 3, 4 };\n"
        "void f(void) {\n  static const BYTE inner[1] = { 9 };\n}\n",
    )
    tables = parse_c_tables(c)
    assert tables == {"t": ["3", "4"]}  # `inner` is function-local → ignored


def test_wide_macro_expands(tmp_path):
    """`wide!("ab")` is [u16; 3] incl. the NUL — the C `{'a','b','\\0'}` shape."""
    c = _c(tmp_path, "static const WCHAR s[] = { 'a','b','\\0' };\n")
    assert [d.kind for d in audit_tables(c, 'static s: &[u16] = wide!("ab");')] == ["ok"]


def test_let_binding_without_type_annotation_is_found(tmp_path):
    c = _c(tmp_path, "static const WCHAR s[] = { 'h','i','\\0' };\n")
    assert [d.kind for d in audit_tables(c, '    let s = wide!("hi");')] == ["ok"]


def test_array_type_containing_a_semicolon_is_matched(tmp_path):
    """A Rust array TYPE contains `;` (`[u8; 8]`). Excluding `;` from the type
    pattern silently unmatched every array static — i.e. every table."""
    tables, unparsed = parse_rust_tables("static M: [u8; 3] = [1, 2, 3];")
    assert tables == {"M": [["1", "2", "3"]]} and not unparsed


def test_a_name_defined_at_two_rust_scopes_matches_either(tmp_path):
    """varformat has a file-scope `szPercent` ("Percent") AND a function-local one
    ("%"), and both translations are correct. Keeping only the last Rust
    definition compared the C's file-scope table against the local one and
    reported a bug in correct code."""
    c = _c(tmp_path, "static const WCHAR szPercent[] = { 'P','c','\\0' };\n")
    rust = (
        "pub static szPercent: [u16; 3] = [b'P' as u16, b'c' as u16, 0];\n"
        "fn f() {\n    static szPercent: [u16; 2] = ['%' as u16, 0];\n}\n"
    )
    assert [d.kind for d in audit_tables(c, rust)] == ["ok"]


def test_unparsable_rhs_is_reported_as_unparsed_not_missing(tmp_path):
    """ole32's guid_conv_table is built by a const block. It IS translated; we
    just cannot evaluate it. Calling that 'missing' is the false alarm that gets
    the whole report ignored."""
    c = _c(tmp_path, "static const BYTE guid_conv_table[2] = { 1, 2 };\n")
    (d,) = audit_tables(c, "static guid_conv_table: [u8; 2] = { build() };")
    assert d.kind == "unparsed"


# --------------------------------------------------------------------------
# element normalization
# --------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("elem", "want"),
    [
        ("0x0a", 10),
        ("10", 10),
        ("'A'", 65),
        ("b'A'", 65),
        (r"'\0'", 0),
        (r"'\n'", 10),
        (r"'\005'", 5),  # C octal — stg_prop's property-set names start with \005
        (r"'\x41'", 65),  # Rust hex escape
        ("VT_EMPTY", None),  # symbolic → unchecked, never a silent pass
    ],
)
def test_normalize_element(elem, want):
    assert normalize_element(elem) == want


def test_symbolic_elements_are_unchecked_not_passed(tmp_path):
    c = _c(tmp_path, "static const BYTE t[2] = { FOO, BAR };\n")
    (d,) = audit_tables(c, "static t: [u8; 2] = [FOO, BAZ];")
    assert d.kind == "ok" and "2 symbolic (unchecked)" in d.detail
