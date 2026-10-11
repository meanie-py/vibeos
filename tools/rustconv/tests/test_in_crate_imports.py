"""Deterministic in-crate import resolution.

Splitting a C++ TU into one module per class turns intra-TU references into cross-module
ones. Nothing deterministic used to emit the `use` for those — only a prompt hint, i.e.
the model — which both violates the substrate-closure rule and cannot repair a body that
is already banked.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.crate_integrate import (
    index_crate_exports,
    island_names,
    resolve_in_crate_imports,
)


def _crate(tmp_path: Path, files: dict[str, str]) -> Path:
    crate = tmp_path / "demo-rust"
    for name, text in files.items():
        path = crate / "src" / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    return crate


class TestTheIndex:
    def test_a_sibling_modules_pub_item_is_indexed_under_its_module_path(self, tmp_path):
        crate = _crate(tmp_path, {"a/b.rs": 'pub unsafe extern "C" fn Foo() {}\n'})
        assert index_crate_exports(crate / "src")["Foo"] == {"crate::a::b"}

    def test_an_inherent_method_is_not_importable_so_it_is_not_indexed(self, tmp_path):
        # `defined_names` anchors at ^\s*, which matches an impl method too. Indexing
        # recyclebin_support.rs that way proposed `use ...::recyclebin_support::len;`
        # for CStringW::len -- a name that cannot be imported at all.
        crate = _crate(
            tmp_path,
            {"sup.rs": "pub struct S;\nimpl S {\n    pub fn len(&self) -> i32 { 0 }\n}\n"},
        )
        owners = index_crate_exports(crate / "src")
        assert owners["S"] == {"crate::sup"}
        assert "len" not in owners

    def test_an_orphan_by_design_file_is_not_a_module_so_it_is_not_indexed(self, tmp_path):
        # recyclebin's superseded whole-TU files still define RecycleBin5_Constructor, which
        # made the live definition look ambiguous; another name resolved to the orphan ALONE
        # and would have produced a `use` of a module that is not declared anywhere.
        crate = _crate(
            tmp_path,
            {
                "live.rs": 'pub unsafe extern "C" fn Ctor() {}\n',
                "dead.rs": "//! ORPHAN-BY-DESIGN: superseded by the per-class modules.\n"
                'pub unsafe extern "C" fn Ctor() {}\n',
            },
        )
        assert index_crate_exports(crate / "src")["Ctor"] == {"crate::live"}

    def test_the_island_is_a_glob_not_an_import_source(self, tmp_path):
        crate = _crate(tmp_path, {"types_i386.rs": "pub struct HWND__;\n"})
        assert "HWND__" not in index_crate_exports(crate / "src")
        assert "HWND__" in island_names(crate / "src")

    def test_island_functions_declared_in_an_extern_block_still_count_as_provided(self, tmp_path):
        # An island states most of its FUNCTIONS inside `unsafe extern "C" { pub fn X(); }`,
        # which `defined_names` deliberately skips. Missing them would let a sibling's
        # same-named `pub fn` look like the only owner.
        crate = _crate(
            tmp_path, {"island_amd64.rs": 'unsafe extern "C" {\n    pub fn SHStrDupW();\n}\n'}
        )
        assert "SHStrDupW" in island_names(crate / "src")


class TestResolution:
    def test_a_cross_module_reference_gets_a_use(self, tmp_path):
        crate = _crate(
            tmp_path,
            {
                "prelude.rs": "pub use crate::types::*;\n",
                "owner.rs": 'pub unsafe extern "C" fn GetDefaultRecycleBin() {}\n',
            },
        )
        code = "use crate::prelude::*;\npub unsafe fn f() { GetDefaultRecycleBin(); }\n"
        new, notes = resolve_in_crate_imports(code, crate)
        assert "use crate::owner::GetDefaultRecycleBin;" in new
        assert notes == []

    def test_an_import_whose_definition_moved_to_the_prelude_is_dropped(self, tmp_path):
        # regedit: error.rs's ErrorMessageBox became `ErrorMessageBox_v` + a prelude extern.
        crate = _crate(
            tmp_path,
            {
                "prelude.rs": 'extern "C" {\n    pub fn ErrorMessageBox(h: HWND, ...) -> i32;\n}\n',
                "error.rs": 'pub unsafe extern "C" fn ErrorMessageBox_v(h: HWND) -> i32 { 0 }\n',
                "keep.rs": 'pub unsafe extern "C" fn Kept() {}\n',
            },
        )
        code = (
            "use crate::error::ErrorMessageBox;\nuse crate::keep::Kept;\nuse crate::prelude::*;\n"
            "pub unsafe fn f() { ErrorMessageBox(h); Kept(); }\n"
        )
        new, notes = resolve_in_crate_imports(code, crate)
        assert "use crate::error::ErrorMessageBox;" not in new
        assert "use crate::keep::Kept;" in new
        assert any("ErrorMessageBox" in n for n in notes)

    def test_a_c2rust_anonymous_type_is_never_imported_from_a_sibling(self, tmp_path):
        # c2rust numbers C2RustUnnamed_N per transpile: svw_pata's _163 is a timing struct,
        # via's is a bridge-table struct. Importing one into the other gave 48 field errors.
        crate = _crate(tmp_path, {"svw.rs": "pub struct C2RustUnnamed_163 { pub Value: u8 }\n"})
        code = "use crate::prelude::*;\nstatic T: [C2RustUnnamed_163; 1] = [];\n"
        assert "use crate::svw" not in resolve_in_crate_imports(code, crate)[0]

    def test_the_use_lands_after_the_prelude_glob(self, tmp_path):
        crate = _crate(tmp_path, {"owner.rs": 'pub unsafe extern "C" fn Foo() {}\n'})
        code = "use crate::prelude::*;\npub unsafe fn f() { Foo(); }\n"
        lines = resolve_in_crate_imports(code, crate)[0].split("\n")
        assert lines[0] == "use crate::prelude::*;"
        assert lines[1] == "use crate::owner::Foo;"

    def test_a_name_this_file_defines_is_never_imported(self, tmp_path):
        # Otherwise a re-translate would import a file's own previous definition of itself.
        crate = _crate(tmp_path, {"owner.rs": 'pub unsafe extern "C" fn Foo() {}\n'})
        code = 'pub unsafe extern "C" fn Foo() {}\npub unsafe fn g() { Foo(); }\n'
        assert "use crate::owner::Foo;" not in resolve_in_crate_imports(code, crate)[0]

    def test_a_name_already_explicitly_imported_is_not_imported_twice(self, tmp_path):
        crate = _crate(tmp_path, {"owner.rs": 'pub unsafe extern "C" fn Foo() {}\n'})
        code = "use crate::owner::Foo;\npub unsafe fn f() { Foo(); }\n"
        assert resolve_in_crate_imports(code, crate)[0].count("use crate::owner::Foo;") == 1

    def test_a_name_the_island_provides_is_left_to_the_glob(self, tmp_path):
        # The safety property: an explicit `use` SHADOWS a glob, so importing a sibling's
        # `pub fn Foo` when the island also exports `Foo` would silently rebind every call.
        crate = _crate(
            tmp_path,
            {
                "types_i386.rs": 'unsafe extern "C" {\n    pub fn Foo();\n}\n',
                "sibling.rs": 'pub unsafe extern "C" fn Foo() {}\n',
            },
        )
        code = "use crate::prelude::*;\npub unsafe fn f() { Foo(); }\n"
        assert "use crate::sibling::Foo;" not in resolve_in_crate_imports(code, crate)[0]

    def test_two_owners_are_reported_and_never_guessed_between(self, tmp_path):
        crate = _crate(
            tmp_path,
            {
                "a.rs": 'pub unsafe extern "C" fn Foo() {}\n',
                "b.rs": 'pub unsafe extern "C" fn Foo() {}\n',
            },
        )
        code = "pub unsafe fn f() { Foo(); }\n"
        new, notes = resolve_in_crate_imports(code, crate)
        assert "use crate::a::Foo;" not in new
        assert "use crate::b::Foo;" not in new
        assert len(notes) == 1
        assert "ambiguous" in notes[0]

    def test_a_name_nothing_in_the_crate_defines_is_left_alone(self, tmp_path):
        crate = _crate(tmp_path, {"owner.rs": 'pub unsafe extern "C" fn Foo() {}\n'})
        code = "pub unsafe fn f() { SomethingExternal(); }\n"
        assert resolve_in_crate_imports(code, crate) == (code, [])

    @pytest.mark.parametrize("missing", ["src", "nothing"])
    def test_a_crate_with_no_src_is_returned_untouched(self, tmp_path, missing):
        code = "pub unsafe fn f() {}\n"
        assert resolve_in_crate_imports(code, tmp_path / missing) == (code, [])


class TestPreambleDeduplication:
    def test_a_local_raii_type_redeclared_by_the_preamble_is_dropped(self):
        from rustconv.crate_integrate import strip_top_level_items

        preamble = (
            "#[repr(C)]\npub struct CStringW {\n    pub m_pszData: LPWSTR,\n}\n\n"
            "#[repr(C)]\npub struct Keep {\n    pub x: i32,\n}\n"
        )
        out, dropped = strip_top_level_items(preamble, {"CStringW"})
        assert dropped == ["CStringW"]
        assert "pub struct CStringW" not in out
        assert "pub struct Keep" in out
        assert "#[repr(C)]" in out  # Keep's attribute survives

    def test_the_impl_block_goes_with_the_type(self):
        from rustconv.crate_integrate import strip_top_level_items

        src = "pub struct S;\nimpl S {\n    pub fn f(&self) {}\n}\npub type T = i32;\n"
        out, dropped = strip_top_level_items(src, {"S"})
        assert dropped == ["S", "S"]
        assert "impl S" not in out
        assert "pub type T = i32;" in out

    def test_a_nested_item_is_never_touched(self):
        from rustconv.crate_integrate import strip_top_level_items

        src = "pub fn f() {\n    struct S;\n}\n"
        assert strip_top_level_items(src, {"S"}) == (src.rstrip("\n"), [])

    def test_no_names_means_no_work(self):
        from rustconv.crate_integrate import strip_top_level_items

        src = "pub struct S;\n"
        assert strip_top_level_items(src, set()) == (src, [])


def test_path_attr_module_takes_its_declared_name_not_its_location(tmp_path):
    """`#[path = "..."] pub mod NAME;` binds NAME; the file's location is irrelevant.

    shell32-rust maps `dialogs/drvdefext.rs` to `crate::drv_def_ext` and
    `shell32/CDefView.rs` to `crate::def_view`, so that a rustconv re-run can keep
    assembling at the C tree's path while the crate keeps its own module naming. Deriving
    the module from the FILE PATH emitted `use crate::dialogs::drvdefext::ShellObjectCreator;`
    into a freshly translated TU -- a module that does not exist. 36 modules in that one
    crate are `#[path]`-mapped, so this was not a corner case.
    """
    from rustconv.crate_integrate import _module_path

    src = tmp_path / "src"
    (src / "dialogs").mkdir(parents=True)
    (src / "shell32").mkdir()
    (src / "lib.rs").write_text(
        '#[path = "dialogs/drvdefext.rs"]\n'
        "pub mod drv_def_ext;\n"
        '#[path = "shell32/CDefView.rs"]\n'
        "pub mod def_view;\n"
        "pub mod shell_link;\n"
    )
    (src / "dialogs" / "drvdefext.rs").write_text("pub struct ShellObjectCreator;\n")
    (src / "shell32" / "CDefView.rs").write_text("pub struct CDefView;\n")
    (src / "shell_link.rs").write_text("pub struct CShellLink;\n")

    assert _module_path(src / "dialogs" / "drvdefext.rs", src) == "crate::drv_def_ext"
    assert _module_path(src / "shell32" / "CDefView.rs", src) == "crate::def_view"
    # An ordinary module is unaffected, and so is a file no `#[path]` names.
    assert _module_path(src / "shell_link.rs", src) == "crate::shell_link"
    (src / "shell32" / "other.rs").write_text("pub struct Other;\n")
    assert _module_path(src / "shell32" / "other.rs", src) == "crate::shell32::other"


def test_an_unknown_single_substrate_import_is_dropped(tmp_path, monkeypatch):
    # sil680.rs's header named one item, so it had no braces: `use ntkrnl_sys::ddk::Channel;`
    # (a FIELD name), which the pruner never matched. A known one is left exactly as written.
    from rustconv import crate_integrate as ci

    monkeypatch.setattr(ci, "_module_names", lambda ws, path: frozenset({"PCSTR"}))
    src = "use crate::prelude::*;\nuse ntkrnl_sys::ddk::Channel;\nuse ntkrnl_sys::ddk::PCSTR;\n"
    new, dropped = ci.drop_unknown_substrate_imports(src, tmp_path)
    assert dropped == ["Channel"]
    assert new == "use crate::prelude::*;\nuse ntkrnl_sys::ddk::PCSTR;\n"
