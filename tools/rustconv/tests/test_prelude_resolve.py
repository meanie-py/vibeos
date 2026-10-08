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


def test_shared_crates_are_reexported_by_path_and_depended_on(tmp_path):
    ws = tmp_path
    (ws / "win32macros-rust" / "src").mkdir(parents=True)
    (ws / "win32macros-rust" / "src" / "lib.rs").write_text("macro_rules! LOWORD {}\n")
    (ws / "strsafe-rust" / "src").mkdir(parents=True)
    (ws / "strsafe-rust" / "src" / "lib.rs").write_text(
        'macro_rules! StringCchPrintfW {}\npub unsafe extern "system" fn StringCchCopyW() {}\n'
    )
    (ws / "strsafe-rust" / "src" / "ntstrsafe.rs").write_text(
        "pub unsafe fn RtlStringCchCopyW() {}\nmacro_rules! RtlStringCchPrintfW {}\n"
    )
    crate = ws / "x-rust"
    (crate / "src").mkdir(parents=True)
    (crate / "Cargo.toml").write_text('[package]\nname = "x"\n\n[dependencies]\nfoo = "1"\n')
    (crate / "src" / "prelude.rs").write_text("pub use win32macros_rust::{LOWORD};\n")
    changed = pr.refresh_shared_crates(crate, ws)
    text = (crate / "src" / "prelude.rs").read_text()
    assert "pub use strsafe_rust::{RtlStringCchPrintfW, StringCchCopyW, StringCchPrintfW};" in text
    assert "pub use strsafe_rust::ntstrsafe::{RtlStringCchCopyW};" in text
    assert set(changed) == {"strsafe_rust", "strsafe_rust::ntstrsafe"}
    assert 'strsafe-rust = { path = "../strsafe-rust" }' in (crate / "Cargo.toml").read_text()
    assert pr.refresh_shared_crates(crate, ws) == []


def test_facts_block_round_trips_and_types_by_typedef(tmp_path):
    facts = {
        "STATUS_PARTITION_FAILURE": ("NTSTATUS", -1073741454, "fsutil.c"),
        "MAX_PATH": ("i32", 260, "x.c"),
        "ONE": ("u32", 1, "x.c"),
    }
    prelude = tmp_path / "prelude.rs"
    prelude.write_text("use crate::types::*;\n")
    assert pr.write_facts(prelude, pr.render_facts(facts))
    text = prelude.read_text()
    assert "pub const STATUS_PARTITION_FAILURE: NTSTATUS = 0xC0000172_u32 as NTSTATUS;" in text
    assert pr.existing_facts(prelude) == facts
    assert not pr.write_facts(prelude, pr.render_facts(facts))
    known = {"NTSTATUS"}
    assert pr.fact_rust_type("NTSTATUS|long", -1, known) == "NTSTATUS"
    # Unknown typedef: its LLP64 canonical type -- C long is 32-bit on Windows.
    assert pr.fact_rust_type("ULONG|unsigned long", 1, set()) == "u32"
    assert pr.fact_rust_type("int", 1, set()) == "i32"
    # Kernel crates spell a bare non-negative `int` macro the way bindgen does.
    assert pr.fact_rust_type("int", 1, set(), bare_int_unsigned=True) == "u32"
    assert pr.fact_rust_type("int", -1, set(), bare_int_unsigned=True) == "i32"
    assert pr.looks_constant("STATUS_X") and not pr.looks_constant("Foo")


def test_local_types_block_round_trips(tmp_path):
    prelude = tmp_path / "prelude.rs"
    prelude.write_text("use crate::types::*;\n")
    item = "#[repr(C)]\npub struct _T {\n    pub a: UCHAR,\n}"
    assert pr.write_local_types(prelude, {"_T": (item, "x.c"), "T": ("pub type T = _T;", "x.c")})
    assert pr.existing_local_types(prelude) == {"_T", "T"}
    assert not pr.write_local_types(
        prelude, {"_T": (item, "x.c"), "T": ("pub type T = _T;", "x.c")}
    )


def test_externs_block_groups_by_abi_and_keeps_aliases(tmp_path):
    sigs = {
        "VfatFormat": (
            "system",
            "fn VfatFormat(\n        A: ULONG,\n    ) -> BOOLEAN;",
            "fsutil.c",
        ),
        "iswspace": ("C", "fn iswspace(c: wint_t) -> c_int;", "arcname.c"),
        "InstallFat12BootCode": (
            "use",
            "pub use crate::m::fsutil::InstallFatBootCode as InstallFat12BootCode;",
            "fsutil.h",
        ),
    }
    text = pr.render_externs(sigs)
    assert 'extern "system" {\n    pub fn VfatFormat(\n        A: ULONG,\n    ) -> BOOLEAN;' in text
    assert 'extern "C" {\n    pub fn iswspace(c: wint_t) -> c_int; // arcname.c\n}' in text
    assert "pub use crate::m::fsutil::InstallFatBootCode as InstallFat12BootCode;" in text
    prelude = tmp_path / "prelude.rs"
    prelude.write_text("")
    assert pr.write_externs(prelude, text)
    assert pr.existing_externs(prelude) == {"VfatFormat", "iswspace", "InstallFat12BootCode"}


def test_unresolved_keeps_the_namespace(monkeypatch, tmp_path):
    import json as _json
    import subprocess as _sp

    msgs = [
        "cannot find type `FILE_SYSTEM` in this scope",
        "expected type, found constant `PFOO`",
        "cannot find value `STATUS_X` in this scope",
        "cannot find struct, variant or union type `_ROOT_KEY` in this scope",
    ]
    out = "\n".join(
        _json.dumps({"reason": "compiler-message", "message": {"message": m}}) for m in msgs
    )
    monkeypatch.setattr(
        _sp, "run", lambda *a, **k: _sp.CompletedProcess(a, 101, stdout=out, stderr="")
    )
    got = pr.unresolved("c", "t", tmp_path, tmp_path)
    assert got == {
        "FILE_SYSTEM": {"type"},
        "PFOO": {"type"},
        "STATUS_X": {"value"},
        "_ROOT_KEY": {"type"},
    }


def test_an_exported_rust_definition_gives_its_extern_declaration():
    from rustconv.prelude_resolve import exported_definitions

    src = (
        '#[no_mangle]\npub unsafe extern "system" fn GetListEntryData(mut Entry: PGENERIC_LIST_ENTRY)'
        " -> PVOID {\n    (*Entry).Data\n}\n"
        '#[no_mangle]\npub unsafe extern "C" fn Other(a: *mut u8) {}\n'
        "pub unsafe fn NotExported() {}\n"
    )
    got = exported_definitions(src, {"GetListEntryData", "Other", "NotExported"})
    assert got == [
        (
            "GetListEntryData",
            "pub fn GetListEntryData(Entry: PGENERIC_LIST_ENTRY) -> PVOID;",
            '"system"',
        ),
        ("Other", "pub fn Other(a: *mut u8);", '"C"'),
    ]


def test_a_zero_argument_function_is_not_a_placeholder():
    from rustconv.prelude_resolve import Item, consensus

    item = Item("fn", "fn CreatePartitionList() -> *mut PARTLIST;", "system", "x.rs")
    assert consensus("CreatePartitionList", [item])[0] is item
    stub = Item("fn", "fn NtClose(_: usize);", "system", "y.rs")
    assert consensus("NtClose", [stub])[0] is None


def test_declared_outside_externs_sees_every_block_but_its_own(tmp_path):
    # appwiz: the tree-consensus block already declared RegQueryValueExW, and the externs pass
    # declared it again because `hand_written_names` skips every generated block (E0428).
    p = tmp_path / "prelude.rs"
    p.write_text(
        "// ==== port resolve: tree consensus (generated) ====\n"
        'extern "system" {\n    pub fn RegQueryValueExW(h: HKEY) -> LSTATUS;\n}\n'
        "// ==== end port resolve ====\n"
        f"{pr.EXTERNS_BEGIN}\n"
        'extern "system" {\n    pub fn HeapFree(h: HANDLE) -> BOOL;\n}\n'
        f"{pr.EXTERNS_END}\n"
        "pub unsafe fn strcmpW(a: *const u16, b: *const u16) -> c_int { 0 }\n"
    )
    names = pr.declared_outside_externs(p)
    assert {"RegQueryValueExW", "strcmpW"} <= names
    assert "HeapFree" not in names
