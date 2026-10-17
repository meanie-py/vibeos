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


def test_a_struct_nested_anonymous_enumerator_is_aliased(tmp_path):
    # winhlp32: `struct tagHlpFileLink { enum {hlp_link_link, ..} cookie; .. }`.
    for arch in ("i386", "amd64", "arm64"):
        (tmp_path / f"types_{arch}.rs").write_text(
            "pub const tagHlpFileLink_hlp_link_popup: tagHlpFileLink__bindgen_ty_1 = 1;\n"
        )
    got = pr.island_enum_aliases({"hlp_link_popup"}, tmp_path)
    assert got["hlp_link_popup"].text == (
        "const hlp_link_popup: tagHlpFileLink__bindgen_ty_1 = tagHlpFileLink_hlp_link_popup;"
    )


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
    (src / "lib.rs").write_text(
        "#[macro_export]\nmacro_rules! LOWORD {}\n#[macro_export]\nmacro_rules! NEWONE {}\n"
        # A function-local helper (wine_dbgstr_variant's `at!`) is not the crate's to export.
        "fn f() { macro_rules! at { () => {} } }\n"
    )
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


def test_a_sibling_struct_is_found_for_a_hoisted_alias(tmp_path):
    # cmd: the prelude's LPDIRSWITCHFLAGS = *mut _DirSwitchesFlags, the struct in dir.rs.
    mods = tmp_path / "cmd"
    mods.mkdir()
    (mods / "dir.rs").write_text("#[repr(C)]\npub struct _DirSwitchesFlags {\n    pub b: i32,\n}\n")
    (mods / "vol.rs").write_text("#[repr(C)]\nstruct Private {\n    x: i32,\n}\n")
    found = pr.sibling_definitions({"_DirSwitchesFlags", "Private"}, mods)
    assert found == {"_DirSwitchesFlags": [("dir", True)], "Private": [("vol", False)]}
    assert pr.promote_pub(mods / "vol.rs", "Private")
    assert "pub struct Private" in (mods / "vol.rs").read_text()


def test_shared_constants_are_reexported_unless_the_prelude_defines_them(tmp_path):
    src = tmp_path / "src"
    src.mkdir()
    (src / "lib.rs").write_text(
        "#[macro_export]\nmacro_rules! LOWORD {}\npub const INVALID_HANDLE_VALUE: HANDLE = 0;\n"
        "pub const UNICODE_NULL: WCHAR = 0;\n"
    )
    p = tmp_path / "prelude.rs"
    p.write_text("pub use win32macros_rust::{LOWORD};\npub const UNICODE_NULL: u16 = 0;\n")
    assert pr.refresh_shared_macros(p, src)
    assert "pub use win32macros_rust::{INVALID_HANDLE_VALUE, LOWORD};" in p.read_text()


def test_shared_crates_are_reexported_by_path_and_depended_on(tmp_path):
    ws = tmp_path
    (ws / "win32macros-rust" / "src").mkdir(parents=True)
    (ws / "win32macros-rust" / "src" / "lib.rs").write_text(
        "#[macro_export]\nmacro_rules! LOWORD {}\n"
    )
    (ws / "strsafe-rust" / "src").mkdir(parents=True)
    (ws / "strsafe-rust" / "src" / "lib.rs").write_text(
        "#[macro_export]\nmacro_rules! StringCchPrintfW {}\n"
        'pub unsafe extern "system" fn StringCchCopyW() {}\n'
    )
    (ws / "strsafe-rust" / "src" / "ntstrsafe.rs").write_text(
        "pub unsafe fn RtlStringCchCopyW() {}\n"
        "#[macro_export]\nmacro_rules! RtlStringCchPrintfW {}\n"
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


def test_winelist_is_taken_only_by_a_crate_that_calls_it(tmp_path):
    # wine/list.h's helpers are opt-in: every crate getting `list_*` and a winelist-rust
    # dependency would be noise, but a crate that calls one must import it, never copy it
    # (rpcss got cabinet-rust's #[no_mangle] copies as externs, which do not link).
    ws = tmp_path
    (ws / "winelist-rust" / "src").mkdir(parents=True)
    (ws / "winelist-rust" / "src" / "lib.rs").write_text(
        "pub struct list {}\npub unsafe fn list_init() {}\npub unsafe fn list_add_tail() {}\n"
    )
    for name, body in (
        ("quiet-rust", "fn f() {}\n"),
        ("walker-rust", "fn f() { list_init(h); }\n"),
    ):
        crate = ws / name
        (crate / "src").mkdir(parents=True)
        (crate / "Cargo.toml").write_text('[package]\nname = "x"\n\n[dependencies]\n')
        (crate / "src" / "prelude.rs").write_text("// prelude\n")
        (crate / "src" / "m.rs").write_text(body)
        pr.refresh_shared_crates(crate, ws)
    assert "winelist" not in (ws / "quiet-rust" / "src" / "prelude.rs").read_text()
    assert "winelist" not in (ws / "quiet-rust" / "Cargo.toml").read_text()
    walker = (ws / "walker-rust" / "src" / "prelude.rs").read_text()
    assert "pub use winelist_rust::{list_add_tail, list_init};" in walker
    assert "winelist-rust = " in (ws / "walker-rust" / "Cargo.toml").read_text()


def test_a_shared_name_the_prelude_already_imports_is_not_imported_again(tmp_path):
    ws = tmp_path
    (ws / "win32macros-rust" / "src").mkdir(parents=True)
    (ws / "win32macros-rust" / "src" / "lib.rs").write_text(
        "#[macro_export]\nmacro_rules! LOWORD {}\npub unsafe fn heap_alloc() {}\n"
    )
    crate = ws / "x-rust"
    (crate / "src").mkdir(parents=True)
    (crate / "Cargo.toml").write_text('[package]\nname = "x"\n\n[dependencies]\n')
    (crate / "src" / "prelude.rs").write_text("pub use crate::x::m::heap_alloc;\n")
    pr.refresh_shared_crates(crate, ws)
    assert "pub use win32macros_rust::{LOWORD};" in (crate / "src" / "prelude.rs").read_text()


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


def test_char_pointee_split_is_not_consensus():
    # winhlp32 got ucrt-rust's `*const u8` strrchr by count; `i8` vs `u8` vs `c_char` differ
    # on some target, so the prototype route decides.
    u8s = [
        F("pub fn strrchr(s: *const u8, c: c_int) -> *mut u8;", '"C"', f"u{n}.rs") for n in range(3)
    ]
    i8s = [F("pub fn strrchr(s: *const i8, c: i32) -> *mut i8;", '"C"', "i.rs")]
    pick, why = consensus("strrchr", u8s + i8s)
    assert pick is None and why.startswith("signature disagreement")
    # The PSTR family is c_char, and agrees with an explicit c_char.
    a = F("pub fn strchr(s: PCSTR, c: i32) -> PSTR;", '"C"', "a.rs")
    b = F("pub fn strchr(s: *const c_char, c: c_int) -> *mut c_char;", '"C"', "b.rs")
    assert consensus("strchr", [a, b])[0] is not None
    # A BYTE* beside a char* in one agreed declaration is not a split.
    m = F("pub fn mix(b: *const u8, s: *const c_char);", '"C"', "m.rs")
    assert consensus("mix", [m, m])[0] is not None


def test_same_name_different_arity_is_not_consensus():
    # diskpart's command handler and setuplib's partlist function share a name and an ABI;
    # the shorter text used to win and usetup called setuplib's with diskpart's signature.
    a = F("pub fn SelectPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE;", '"C"', "d.rs")
    b = F(
        "pub fn SelectPartition(List: PPARTLIST, DiskNumber: ULONG, PartitionNumber: ULONG,) "
        "-> PPARTENTRY;",
        '"C"',
        "s.rs",
    )
    pick, why = consensus("SelectPartition", [a, b])
    assert pick is None and why.startswith("signature disagreement")
    assert pr._arity('pub fn g(cb: Option<unsafe extern "C" fn(a: i32, b: i32)>, ...);') == 2


def test_clang_param_count_reads_past_the_calling_convention():
    from rustconv.sdk_facts import param_count

    assert param_count("PPARTLIST (*)(void) __attribute__((stdcall))") == 0
    assert (
        param_count("BOOLEAN (*)(PPARTLIST, PPARTENTRY, PPARTENTRY *) __attribute__((stdcall))")
        == 3
    )
    assert param_count("void (*)(SHORT, SHORT, LPCSTR, ...)") == 4
    assert param_count("int (*)(int (*)(int, int), char)") == 2


def test_a_crate_local_path_is_not_copied_into_another_crate():
    """shell32's `IDC_WAIT: crate::island::LPCWSTR` named shell32's own module; copied into
    notepad it resolved to nothing."""
    assert pr._norm(
        "pub const IDC_WAIT: crate::island::LPCWSTR = 32514isize as crate::island::LPCWSTR;"
    ) == ("const IDC_WAIT: LPCWSTR = 32514isize as LPCWSTR;")


def test_bindgen_declarations_outvote_a_hand_written_extern():
    """One crate's `extern "C" { fn CloseHandle }` (a stdcall import) vetoed every module."""
    pick, why = consensus(
        "CloseHandle",
        [
            F("fn CloseHandle(hObject: HANDLE) -> BOOL;", '"C"', "x-rust/src/prelude.rs"),
            F("fn CloseHandle(arg1: HANDLE) -> BOOL;", '"system"', "w-sys/src/bindgen_i386.rs"),
            F("fn CloseHandle(arg1: HANDLE) -> BOOL;", '"C"', "y-rust/src/types_amd64.rs"),
        ],
    )
    assert pick is not None and pick.abi == '"system"', why
    # a 64-bit island's "C" is not evidence: it has no stdcall to write
    pick, why = consensus(
        "HeapFree",
        [
            F("fn HeapFree(a: HANDLE) -> BOOL;", '"system"', "x-rust/src/prelude.rs"),
            F("fn HeapFree(a: HANDLE) -> BOOL;", '"C"', "y-rust/src/types_amd64.rs"),
        ],
    )
    assert pick is not None and pick.abi == '"system"', why
    # without bindgen evidence the disagreement still refuses
    pick, why = consensus("F", [F("fn F();", '"C"', "a.rs"), F("fn F();", '"system"', "b.rs")])
    assert pick is None and "ABI disagreement" in why


def test_long_ptr_aliases_are_winapi_even_when_clang_saw_a_macro():
    block = pr.render_externs(
        {
            "GetWindowLongPtrW": (
                "C",
                "fn GetWindowLongPtrW(arg1: HWND, arg2: c_int) -> LONG_PTR;",
                "x",
            )
        }
    )
    assert 'extern "system" {' in block and 'extern "C"' not in block
    assert 'link_name = "GetWindowLongW"' in block


def test_sibling_definitions_finds_a_type_a_module_defines(tmp_path):
    """msctf: `LPFNCONSTRUCTOR` is a typedef in msctf.cpp's unit that cpp_classes.rs names."""
    from rustconv.prelude_resolve import sibling_definitions

    (tmp_path / "msctf_cpp_h.rs").write_text(
        'pub type LPFNCONSTRUCTOR = unsafe extern "C" fn() -> i32;\n'
        "pub struct ClassesTableEntry {\n    pub clsid: u32,\n}\n"
    )
    found = sibling_definitions({"LPFNCONSTRUCTOR", "ClassesTableEntry", "Nope"}, tmp_path)
    assert found == {
        "LPFNCONSTRUCTOR": [("msctf_cpp_h", True)],
        "ClassesTableEntry": [("msctf_cpp_h", True)],
    }


def test_an_extern_static_is_copied_from_the_island():
    """uuid's GUID_NULL: an island `static GUID_NULL: GUID;` beats a crate's const copy."""
    from rustconv.prelude_resolve import Item, consensus, render

    items = [
        Item("static", "static GUID_NULL: GUID;", '"C"', "audiosrv-rust/src/types_i386.rs"),
        Item("const", "const GUID_NULL: GUID = GUID { Data1: 0 };", None, "x-rust/src/lib.rs"),
    ]
    pick, why = consensus("GUID_NULL", items)
    assert pick is items[0], why
    assert 'extern "C" {\n    pub static GUID_NULL: GUID;' in render({"GUID_NULL": pick})


def test_a_bare_long_ptr_with_only_64bit_link_names_gets_the_i386_alias():
    """mspaint's prelude copied `GetWindowLongPtr` with x86_64/aarch64 link_names only, and
    i386 linked the literal macro name: `undefined symbol: _GetWindowLongPtr@8`."""
    from rustconv.prelude_resolve import _x86_alias

    sig = (
        '#[cfg_attr(target_arch = "x86_64", link_name = "GetWindowLongPtrW")]\n'
        "fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR;"
    )
    assert _x86_alias(sig) == '    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]\n'
    assert (
        _x86_alias("fn SetClassLongPtrA(h: HWND, i: INT, v: LONG_PTR) -> ULONG_PTR;").count(
            '"SetClassLongA"'
        )
        == 1
    )
    done = '#[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]\n' + sig
    assert _x86_alias(done) == ""


def test_a_kernel_sys_constant_is_wired_from_the_one_module_that_defines_it(tmp_path):
    """NTSTATUS values live in ntkrnl-sys's hand `base`/`constants`, which the island's `ddk`
    glob does not reach; globbing `base` instead made 215 names ambiguous."""
    from rustconv.prelude_resolve import substrate_constants

    src = tmp_path / "src"
    src.mkdir()
    (src / "lib.rs").write_text("pub mod base;\npub mod constants;\npub mod bindgen_amd64;\n")
    (src / "base.rs").write_text(
        "pub const STATUS_SUCCESS: NTSTATUS = 0;\npub const TWICE: u32 = 1;\n"
        "    pub const NESTED: u32 = 2;\n"
    )
    (src / "constants.rs").write_text(
        "pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as i32;\n"
        "pub const TWICE: u32 = 1;\n"
    )
    (src / "bindgen_amd64.rs").write_text("pub const FROM_GLOB: u32 = 3;\n")
    found, why = substrate_constants(
        {"STATUS_SUCCESS", "STATUS_INVALID_PARAMETER", "TWICE", "NESTED", "FROM_GLOB"}, src
    )
    assert set(found["STATUS_SUCCESS"].values()) == {"base"}
    assert set(found["STATUS_INVALID_PARAMETER"].values()) == {"constants"}
    assert "TWICE" not in found and "base" in why["TWICE"] and "constants" in why["TWICE"]
    assert "NESTED" not in found  # not top level
    assert "FROM_GLOB" not in found  # the glob is the island's own


def test_an_integer_width_split_is_the_prototypes_to_settle():
    """fc got the vote's `towupper(c: c_int) -> c_int` for the CRT's `wint_t towupper(wint_t)`."""
    int_ = [F("pub fn towupper(c: c_int) -> c_int;", path=f"{c}.rs") for c in "abc"]
    wint = F("pub fn towupper(c: wint_t) -> wint_t;", path="d.rs")
    pick, why = consensus("towupper", [*int_, wint])
    assert pick is None and why == "signature disagreement (integer width)"
    # Same widths spelled differently are one signature; pointers never vote.
    same = [
        F("pub fn f(a: DWORD, p: *mut u16) -> BOOL;"),
        F("pub fn f(a: u32, p: *mut WCHAR) -> i32;", path="b.rs"),
    ]
    assert consensus("f", same)[0] is not None


def test_a_count_form_swprintf_binds_the_function_its_inline_calls():
    # msvcrt's exported `swprintf` is the legacy (buf, fmt, ...): a copied count-form
    # declaration read `n` as the format pointer and cscript crashed on a script error.
    from rustconv.prelude_resolve import _crt_inline_alias

    count = "fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;"
    assert _crt_inline_alias(count) == '    #[link_name = "_snwprintf"]\n'
    vcount = "fn vswprintf(s: *mut u16, n: usize, fmt: *const u16, a: va_list) -> c_int;"
    assert _crt_inline_alias(vcount) == '    #[link_name = "_vsnwprintf"]\n'
    assert _crt_inline_alias("fn swprintf(buf: *mut u16, fmt: *const u16, ...) -> i32;") == ""
    assert _crt_inline_alias("fn vswprintf(b: *mut u16, f: *const u16, a: va_list) -> i32;") == ""
    assert _crt_inline_alias('#[link_name = "_snwprintf"]\n' + count) == ""
    assert _crt_inline_alias("fn sprintf(s: *mut i8, n: usize, f: *const i8, ...) -> c_int;") == ""


def test_a_kernel_crate_never_takes_the_console_crate(tmp_path):
    # storport-rust (on ntkrnl-sys) was handed all of conutils-h and the dependency.
    ws = tmp_path / "ws"
    (ws / "conutils-h" / "src").mkdir(parents=True)
    (ws / "conutils-h" / "src" / "lib.rs").write_text("pub unsafe fn ConPuts() {}\n")
    for name, dep in (("kern", "ntkrnl-sys"), ("user", "ntdll-sys")):
        crate = ws / name
        (crate / "src").mkdir(parents=True)
        (crate / "src" / "prelude.rs").write_text("// x\n")
        (crate / "Cargo.toml").write_text(f'[dependencies]\n{dep} = {{ path = "../{dep}" }}\n')
    pr.refresh_shared_crates(ws / "kern", ws)
    assert "conutils" not in (ws / "kern" / "src" / "prelude.rs").read_text()
    assert "conutils" not in (ws / "kern" / "Cargo.toml").read_text()
    pr.refresh_shared_crates(ws / "user", ws)
    assert "pub use conutils_h::{ConPuts};" in (ws / "user" / "src" / "prelude.rs").read_text()
