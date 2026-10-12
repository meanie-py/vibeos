"""prelude_resolve.workspace_items: every definition of each wanted name, in a stable order."""

import subprocess

from rustconv.prelude_resolve import workspace_items


def _repo(tmp_path, files):
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    for rel, text in files.items():
        path = tmp_path / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    subprocess.run(["git", "-C", str(tmp_path), "add", "."], check=True)
    return tmp_path


def test_each_name_gets_its_consts_then_its_types_per_file(tmp_path):
    """One regex pass per kind (not per name) must keep the per-name order consensus reads."""
    ws = _repo(
        tmp_path,
        {
            "a-rust/src/types.rs": (
                "pub type HWND = *mut c_void;\n"
                "pub const WM_USER: u32 = 1024;\n"
                "pub const BS_LEFT: i32 = 256;\n"
                "pub type BS_LEFT = i32;\n"
                "pub const NOT_WANTED: u32 = 1;\n"
            ),
            "b-rust/src/types.rs": "pub const BS_LEFT: u32 = 256;\npub const WM_USERX: u32 = 2;\n",
            "mine-rust/src/lib.rs": "pub const BS_LEFT: i32 = 999;\n",
        },
    )
    found = workspace_items({"BS_LEFT", "WM_USER", "HWND"}, ws, ws / "mine-rust")
    assert [(i.kind, i.text, i.path) for i in found["BS_LEFT"]] == [
        ("const", "const BS_LEFT: i32 = 256;", "a-rust/src/types.rs"),
        ("type", "type BS_LEFT = i32;", "a-rust/src/types.rs"),
        ("const", "const BS_LEFT: u32 = 256;", "b-rust/src/types.rs"),
    ]
    assert [i.text for i in found["WM_USER"]] == ["const WM_USER: u32 = 1024;"]
    assert [i.kind for i in found["HWND"]] == ["type"]


def test_a_pointer_typed_fact_is_cast_from_an_integer_and_round_trips(tmp_path):
    """taskdialog.c's `TD_ERROR_ICON` is MAKEINTRESOURCEW(-2): an LPWSTR, not an integer."""
    from rustconv import prelude_resolve as pr

    ty = pr.fact_rust_type("LPWSTR|WCHAR *", 0xFFFE, {"LPWSTR"})
    block = pr.render_facts({"TD_ERROR_ICON": (ty, 0xFFFE, "taskdialog.c")})
    assert "pub const TD_ERROR_ICON: LPWSTR = 0xFFFEusize as LPWSTR;" in block
    p = tmp_path / "prelude.rs"
    p.write_text(block)
    assert pr.existing_facts(p) == {"TD_ERROR_ICON": ("*LPWSTR", 0xFFFE, "taskdialog.c")}


def test_a_copied_long_ptr_extern_gets_its_i386_alias():
    """i386 user32 has no `_GetWindowLongPtrW@8`: winuser.h maps it to GetWindowLongW."""
    from rustconv import prelude_resolve as pr

    block = pr.render_externs(
        {
            "GetWindowLongPtrW": (
                "system",
                "fn GetWindowLongPtrW(h: HWND, i: INT) -> LONG_PTR;",
                "x.c",
            ),
            "SetFocus": ("system", "fn SetFocus(h: HWND) -> HWND;", "x.c"),
        }
    )
    assert (
        '#[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]\n    pub fn GetWindowLongPtrW'
        in block
    )
    assert block.count("cfg_attr") == 1


def test_a_user_mode_crate_never_borrows_a_kernel_declaration(tmp_path):
    # gdi32_vista was handed disk-rust's DDK-shim `_InterlockedExchange`: i386 LINK_FAIL.
    decl = 'extern "C" {\n    pub fn _InterlockedExchange(t: *mut i32, v: i32) -> i32;\n}\n'
    ws = _repo(
        tmp_path,
        {
            "disk-rust/Cargo.toml": 'ntkrnl-sys = { path = "../ntkrnl-sys" }\n',
            "disk-rust/src/ddk_shims.rs": decl,
            "shared-rust/src/lib.rs": decl,
            "mine-rust/Cargo.toml": 'ntdll-sys = { path = "../ntdll-sys" }\n',
            "mine-rust/src/lib.rs": "",
        },
    )
    found = workspace_items({"_InterlockedExchange"}, ws, ws / "mine-rust")
    assert [i.path for i in found["_InterlockedExchange"]] == ["shared-rust/src/lib.rs"]


def test_an_executables_definition_is_not_an_export(tmp_path):
    """arp's private PrintMessage(dwMessage) was copied into ifmon as netsh's PrintMessage."""
    exe_fn = '#[no_mangle]\npub unsafe extern "C" fn PrintMessage(dwMessage: u32) {}\n'
    dll_fn = '#[no_mangle]\npub unsafe extern "system" fn SetupThing(x: u32) -> u32 { x }\n'
    ws = _repo(
        tmp_path,
        {
            "arp-rust/src/lib.rs": "//! (usermode module type `win32cui`).\n",
            "arp-rust/src/arp/arp.rs": exe_fn,
            "setuplib-rust/src/lib.rs": "//! (usermode module type `win32dll`).\n",
            "setuplib-rust/src/setuplib/lib.rs": dll_fn,
            "mine-rust/src/lib.rs": "",
        },
    )
    found = workspace_items({"PrintMessage", "SetupThing"}, ws, ws / "mine-rust")
    assert found["PrintMessage"] == []
    assert [i.path for i in found["SetupThing"]] == ["setuplib-rust/src/setuplib/lib.rs"]


def test_a_pseh_macro_is_never_donated_as_a_function(tmp_path):
    """cdfs's DUMMY_PSEH `_SEH2_TRY` export made sptilib's probe compile with no handler."""
    ws = _repo(
        tmp_path,
        {
            "cdfs-rust/src/cdmacros.rs": (
                '#[no_mangle]\npub unsafe extern "C" fn _SEH2_TRY() {}\n'
                'extern "C" {\n    pub fn _SEH2_END();\n}\n'
            ),
            "mine-rust/src/lib.rs": "",
        },
    )
    found = workspace_items({"_SEH2_TRY", "_SEH2_END"}, ws, ws / "mine-rust")
    assert "_SEH2_TRY" not in found and "_SEH2_END" not in found
