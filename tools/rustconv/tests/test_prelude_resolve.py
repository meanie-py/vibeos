"""prelude_resolve: copy only what the tree agrees on, never a guess."""

from rustconv import prelude_resolve as pr
from rustconv.prelude_resolve import Item, consensus, existing_block, render


def C(text, path="a.rs"):
    return Item("const", text, None, path)


def F(text, abi='"C"', path="a.rs"):
    return Item("fn", text, abi, path)


def test_constants_agree_by_value_not_spelling():
    pick, _ = consensus(
        "FILE_SHARE_READ",
        [
            C("const FILE_SHARE_READ: DWORD = 0x00000001;"),
            C("const FILE_SHARE_READ: DWORD = 0x00000001;", "b.rs"),
            C("const FILE_SHARE_READ: ULONG = 0x0000_0001;", "c.rs"),
            C("const FILE_SHARE_READ: u32 = 1u32 as u32;", "d.rs"),
        ],
    )
    assert pick is not None and pick.text == "const FILE_SHARE_READ: DWORD = 0x00000001;"


def test_constants_with_different_values_are_refused():
    pick, why = consensus("X", [C("const X: u32 = 1;"), C("const X: u32 = 2;")])
    assert pick is None and "different values" in why


def test_an_unevaluated_expression_neither_vetoes_nor_is_copied():
    pick, _ = consensus(
        "IOCTL",
        [
            C("const IOCTL: u32 = 0x74004;"),
            C(
                "const IOCTL: DWORD = CTL_CODE(FILE_DEVICE_DISK, 1, METHOD_BUFFERED, FILE_READ_ACCESS);"
            ),
        ],
    )
    assert pick is not None and "0x74004" in pick.text


def test_extern_abi_class_must_agree():
    pick, why = consensus(
        "RtlFreeHeap",
        [
            F("fn RtlFreeHeap(h: HANDLE, f: ULONG, p: PVOID) -> BOOLEAN;", '"system"'),
            F("fn RtlFreeHeap(h: HANDLE, f: ULONG, p: PVOID) -> BOOLEAN;", '"C"'),
        ],
    )
    assert pick is None and "ABI" in why
    pick, _ = consensus(
        "RtlFreeHeap",
        [
            F("fn RtlFreeHeap(h: HANDLE, f: ULONG, p: PVOID) -> BOOLEAN;", '"system"'),
            F("fn RtlFreeHeap(h: HANDLE, f: ULONG, p: PVOID) -> BOOLEAN;", '"stdcall"'),
            F("fn RtlFreeHeap(_: usize, _: usize, _: usize);", '"C"'),  # a link stub, not evidence
        ],
    )
    assert pick is not None


def test_types_are_never_copied():
    pick, why = consensus("HANDLE", [Item("type", "type HANDLE = *mut c_void;", None, "a.rs")])
    assert pick is None and "island" in why


def test_rendered_block_round_trips(tmp_path):
    block = render(
        {
            "X": C("const X: u32 = 1;"),
            "_wcsicmp": F("fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;"),
        }
    )
    assert 'extern "C" {' in block and '""C""' not in block
    prelude = tmp_path / "prelude.rs"
    prelude.write_text("pub use crate::types::*;\n\n" + block)
    back = existing_block(prelude)
    assert set(back) == {"X", "_wcsicmp"} and back["_wcsicmp"].abi == '"C"'


def test_bare_enumerator_aliases_the_island_constant(tmp_path):
    for arch in ("i386", "amd64", "arm64"):
        (tmp_path / f"types_{arch}.rs").write_text(
            "pub const _BOOT_STORE_TYPE_FreeLdr: _BOOT_STORE_TYPE = 0;\n"
            "pub const _A_Dup: _A = 0;\npub const _B_Dup: _B = 1;\n"
        )
    got = pr.island_enum_aliases({"FreeLdr", "Dup", "Nothing"}, tmp_path)
    assert set(got) == {"FreeLdr"}  # Dup is ambiguous, Nothing is absent
    assert got["FreeLdr"].text == "const FreeLdr: _BOOT_STORE_TYPE = _BOOT_STORE_TYPE_FreeLdr;"
    # and it round-trips through the generated block
    block = pr.render(got)
    p = tmp_path / "prelude.rs"
    p.write_text(block)
    assert pr.existing_block(p)["FreeLdr"].text == got["FreeLdr"].text


def test_shared_macro_list_is_refreshed_in_place(tmp_path):
    src = tmp_path / "src"
    src.mkdir()
    (src / "lib.rs").write_text("macro_rules! LOWORD {}\nmacro_rules! NEWONE {}\n")
    p = tmp_path / "prelude.rs"
    p.write_text("// x\npub use win32macros_rust::{LOWORD};\n// y\n")
    assert pr.refresh_shared_macros(p, src)
    assert "pub use win32macros_rust::{LOWORD, NEWONE};" in p.read_text()
    assert not pr.refresh_shared_macros(p, src)


def test_a_sibling_definition_is_imported_and_promoted(tmp_path):
    mods = tmp_path / "setuplib"
    mods.mkdir()
    (mods / "infsupp.rs").write_text(
        'pub unsafe extern "C" fn INF_GetData() {}\nunsafe extern "C" fn Hidden() {}\n'
    )
    (mods / "a.rs").write_text('unsafe extern "C" {\n    fn INF_GetData();\n}\nfn Twin() {}\n')
    (mods / "b.rs").write_text("fn Twin() {}\n")
    found = pr.sibling_definitions({"INF_GetData", "Hidden", "Twin", "Nope"}, mods)
    assert found["INF_GetData"] == [
        ("infsupp", True)
    ]  # the extern decl in a.rs is not a definition
    assert found["Hidden"] == [("infsupp", False)]
    assert sorted(m for m, _ in found["Twin"]) == ["a", "b"]
    assert pr.promote_pub(mods / "infsupp.rs", "Hidden")
    assert 'pub unsafe extern "C" fn Hidden' in (mods / "infsupp.rs").read_text()
    p = tmp_path / "prelude.rs"
    p.write_text("// x\n")
    block = pr.render_siblings({"INF_GetData": "infsupp"}, "setuplib")
    assert pr.write_siblings(p, block) and not pr.write_siblings(p, block)
    assert pr.existing_siblings(p) == {"INF_GetData": "infsupp"}


def test_shared_constants_are_reexported_unless_the_prelude_defines_them(tmp_path):
    src = tmp_path / "src"
    src.mkdir()
    (src / "lib.rs").write_text(
        "macro_rules! LOWORD {}\npub const INVALID_HANDLE_VALUE: HANDLE = 0;\n"
        "pub const UNICODE_NULL: WCHAR = 0;\n"
    )
    p = tmp_path / "prelude.rs"
    p.write_text("pub use win32macros_rust::{LOWORD};\npub const UNICODE_NULL: u16 = 0;\n")
    assert pr.refresh_shared_macros(p, src)
    assert "pub use win32macros_rust::{INVALID_HANDLE_VALUE, LOWORD};" in p.read_text()
