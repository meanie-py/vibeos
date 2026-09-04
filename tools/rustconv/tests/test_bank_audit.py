"""bank_audit resolves paid output against the tree — and must not resolve it wrongly.

Every case here is a false positive or false negative the audit actually produced while
it was being written, on the real 444-dir bank. The direction that matters is the
optimistic one: an audit that reports abandoned paid work as shipped is worse than no
audit, because it retires the question.

  - an `extern` declaration counted as a definition (a prelude promising `Foo` exists
    would have "proved" Foo shipped)
  - `debug` matched `debug_block_stamp`, `error` matched `WDML_Initialize_error` — short
    generic variadic names against a loose containment test
  - a multi-function body counted as integrated when only ONE of its functions shipped
"""

from pathlib import Path

import pytest

from rustconv.bank_audit import (
    BankedFn,
    audit,
    banked_fns,
    declared_names,
    format_report,
    resolve,
    tchar_variants,
    truncated_fns,
    varargs_twins,
)


def _write(tmp_path: Path, name: str, text: str) -> Path:
    path = tmp_path / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


# --------------------------------------------------------------------------- definitions


def test_extern_declaration_is_not_a_definition(tmp_path):
    """The bug that made the tree index lie: a prelude's promise is not a definition."""
    src = _write(
        tmp_path,
        "prelude.rs",
        'extern "C" {\n    pub fn SetLastError(code: u32);\n    pub fn GetLastError() -> u32;\n}\n',
    )
    assert declared_names(src) == set()


def test_definition_is_found_alongside_declarations(tmp_path):
    src = _write(
        tmp_path,
        "m.rs",
        'extern "C" {\n    pub fn Declared(x: u32);\n}\n'
        "pub unsafe fn Defined(x: u32) -> u32 { x + 1 }\n",
    )
    assert declared_names(src) == {"Defined"}


def test_semicolon_inside_array_return_type_is_not_a_declaration(tmp_path):
    """`-> [u8; 4]` must not read as the `;` that ends a declaration."""
    src = _write(tmp_path, "m.rs", "fn digits() -> [u8; 4] { [0; 4] }\n")
    assert declared_names(src) == {"digits"}


def test_commented_out_c_source_is_not_indexed(tmp_path):
    """Every translated file trails the ORIGINAL C SOURCE as comments."""
    src = _write(
        tmp_path,
        "m.rs",
        "pub fn Real() {}\n// fn Ghost() {}\n/* fn AlsoGhost() {} */\n",
    )
    assert declared_names(src) == {"Real"}


# --------------------------------------------------------------------------- layouts


def test_manifest_layout_skips_bookkeeping_keys(tmp_path):
    d = tmp_path / "mod__file"
    _write(
        tmp_path,
        "mod__file/manifest.json",
        '{"Foo": {"file": "Foo.rs"}, "__preamble__": {"file": "__preamble__.rs"}}',
    )
    assert [f.name for f in banked_fns(d)] == ["Foo"]


def test_legacy_layout_strips_sequence_prefix_and_preamble(tmp_path):
    d = tmp_path / "20260101-000000-x.c"
    _write(tmp_path, "20260101-000000-x.c/00_preamble.rs", "")
    _write(tmp_path, "20260101-000000-x.c/01_Alpha.rs", "")
    _write(tmp_path, "20260101-000000-x.c/02_Beta.rs", "")
    assert sorted(f.name for f in banked_fns(d)) == ["Alpha", "Beta"]


# --------------------------------------------------------------------------- varargs twins


@pytest.mark.parametrize(
    "name,tree,expected",
    [
        ("FatalError", {"FatalError_v"}, "FatalError_v"),
        ("FormatMessageBox", {"netid_FormatMessageBox_body"}, "netid_FormatMessageBox_body"),
        ("Foo", {"Foo_impl"}, "Foo_impl"),
    ],
)
def test_varargs_twin_conventions(name, tree, expected):
    assert varargs_twins(name, tree) == [expected]


@pytest.mark.parametrize(
    "name,tree",
    [
        # the two real cross-crate mismatches a containment test produced
        ("debug", {"debug_block_stamp"}),
        ("error", {"WDML_Initialize_error"}),
        # decoration must be a known twin suffix, not any trailing word
        ("Foo", {"Foo_helper"}),
        # and a bare prefix is not a twin either
        ("Foo", {"module_Foo"}),
    ],
)
def test_varargs_twin_rejects_merely_similar_names(name, tree):
    assert varargs_twins(name, tree) == []


def test_varargs_twin_prefers_undecorated_over_module_prefixed():
    tree = {"Foo_v", "other_Foo_body"}
    assert varargs_twins("Foo", tree)[0] == "Foo_v"


def test_varargs_rule_only_applies_to_variadic_bodies(tmp_path):
    """A non-variadic `Foo` must not be satisfied by an unrelated `Foo_v`."""
    path = _write(tmp_path, "Foo.rs", "unsafe fn Foo(x: u32) -> u32 { x }\n")
    res = resolve(BankedFn("Foo", path), {"Foo_v"}, {})
    assert not res.integrated


def test_variadic_body_resolves_to_twin(tmp_path):
    path = _write(
        tmp_path,
        "Log.rs",
        "unsafe fn Log(fmt: *const u8, mut args: ...) -> i32 { va_start(args, fmt); 0 }\n",
    )
    res = resolve(BankedFn("Log", path), {"Log_v"}, {})
    assert (res.found_as, res.via) == ("Log_v", "varargs_twin")


# --------------------------------------------------------------------------- body rule


def test_body_rule_requires_every_function_to_ship(tmp_path):
    """A 3-function body with 1 shipped is partially-lost paid work, not integrated."""
    path = _write(
        tmp_path,
        "MACRO.rs",
        "unsafe fn Alpha() {}\nunsafe fn Beta() {}\nunsafe fn Gamma() {}\n",
    )
    partial = resolve(BankedFn("MACRO", path), {"Alpha"}, {})
    assert not partial.integrated

    whole = resolve(BankedFn("MACRO", path), {"Alpha", "Beta", "Gamma"}, {})
    assert whole.via == "declared_in_body"


# --------------------------------------------------------------------------- misc rules


def test_tchar_variants():
    assert tchar_variants("_tmain") == ["wmain", "main"]
    assert tchar_variants("_tWinMain") == ["wWinMain", "WinMain"]
    assert tchar_variants("plain") == []


def test_allowlist_resolves_only_with_an_entry(tmp_path):
    path = _write(tmp_path, "Foo.rs", "unsafe fn Foo() {}\n")
    assert not resolve(BankedFn("Foo", path), set(), {}).integrated
    assert resolve(BankedFn("Foo", path), set(), {"Foo": "reviewed"}).via == "allowlist"


# --------------------------------------------------------------------------- truncated


def test_partial_files_are_scanned_and_marked_truncated(tmp_path):
    """A billed-but-cut-off call is paid work too, and needs a different remedy."""
    d = tmp_path / "mod__file"
    _write(tmp_path, "mod__file/06_Interrupted.partial", "unsafe fn Interrupted(")
    (fn,) = truncated_fns(d)
    assert (fn.name, fn.truncated) == ("Interrupted", True)


def test_truncated_body_absent_from_tree_is_reported(tmp_path):
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "responses/mod__file/06_Interrupted.partial", "unsafe fn Interrupted(")
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Unrelated() {}\n")

    (report,) = audit(responses, tree, allow={})
    (missing,) = report.absent
    assert missing.fn.truncated
    assert "RE-TRANSLATE" in format_report([report])


def test_completed_body_supersedes_its_leftover_partial(tmp_path):
    """A retried call leaves the `.partial` behind; the function is still one function."""
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "responses/mod__file/manifest.json", '{"Foo": {"file": "Foo.rs"}}')
    _write(tmp_path, "responses/mod__file/Foo.rs", "unsafe fn Foo() {}\n")
    _write(tmp_path, "responses/mod__file/06_Foo.partial", "unsafe fn Foo(")
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Foo() {}\n")

    (report,) = audit(responses, tree, allow={})
    assert report.total == 1


def test_truncated_body_that_later_landed_is_not_reported(tmp_path):
    """Every `.partial` in the real bank belongs to a call that was retried."""
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "responses/mod__file/06_Retried.partial", "unsafe fn Retried(")
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Retried() {}\n")

    (report,) = audit(responses, tree, allow={})
    assert report.absent == []


# --------------------------------------------------------------------------- end to end


def test_audit_reports_a_genuine_absence(tmp_path):
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "responses/mod__file/manifest.json", '{"Shipped": {}, "Dropped": {}}')
    _write(tmp_path, "responses/mod__file/Shipped.rs", "unsafe fn Shipped() {}\n")
    _write(tmp_path, "responses/mod__file/Dropped.rs", "unsafe fn Dropped() {}\n")
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Shipped() {}\n")

    (report,) = audit(responses, tree, allow={})
    assert report.status == "PARTIAL"
    assert [r.fn.name for r in report.absent] == ["Dropped"]
