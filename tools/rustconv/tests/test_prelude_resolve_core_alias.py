"""A copied extern carries its source crate's core-only aliases inline (ucrt's qsort)."""

from __future__ import annotations

from rustconv.prelude_resolve import _inline_core_aliases


def test_a_core_only_alias_is_inlined(tmp_path):
    src = tmp_path / "ucrt-rust" / "src" / "stdlib"
    src.mkdir(parents=True)
    (src / "search.rs").write_text(
        'pub type Compare = Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>;\n'
        "pub type Island = *mut _SOME_STRUCT;\n"
    )
    cache: dict = {}
    decl = "pub fn qsort(base: *mut c_void, num: usize, width: usize, comp: Compare);"
    out = _inline_core_aliases(decl, tmp_path / "ucrt-rust", cache)
    assert out == (
        "pub fn qsort(base: *mut c_void, num: usize, width: usize, "
        'comp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>);'
    )
    # An alias over a non-core type stays a name (the island's job, as before).
    assert _inline_core_aliases("pub fn f(p: Island);", tmp_path / "ucrt-rust", cache) == (
        "pub fn f(p: Island);"
    )


def test_an_alias_two_files_disagree_on_is_not_inlined(tmp_path):
    src = tmp_path / "c-rust" / "src"
    src.mkdir(parents=True)
    (src / "a.rs").write_text("pub type T = u32;\n")
    (src / "b.rs").write_text("pub type T = u64;\n")
    assert _inline_core_aliases("pub fn f(x: T);", tmp_path / "c-rust", {}) == "pub fn f(x: T);"


def test_a_private_alias_counts_too(tmp_path):
    # gdi32's font.rs: `type QsortCmp = unsafe extern "C" fn(..) -> c_int;` (no `pub`).
    src = tmp_path / "gdi32-rust" / "src"
    src.mkdir(parents=True)
    (src / "font.rs").write_text(
        'type QsortCmp = unsafe extern "C" fn(*const c_void, *const c_void) -> c_int;\n'
    )
    out = _inline_core_aliases(
        "pub fn qsort(b: *mut c_void, compar: QsortCmp);", tmp_path / "gdi32-rust", {}
    )
    assert "compar: unsafe extern" in out


def test_local_types_take_bindgens_keyword_spelling(tmp_path):
    from rustconv.prelude_resolve import write_local_types

    prelude = tmp_path / "prelude.rs"
    prelude.write_text("// prelude\n")
    struct = "pub struct data_type {\n    pub tag: *const u16,\n    pub type_0: c_int,\n    pub kind_0: c_int,\n}"
    write_local_types(prelude, {"data_type": (struct, "import.c")})
    text = prelude.read_text()
    assert "pub type_: c_int" in text and "pub kind_0: c_int" in text
