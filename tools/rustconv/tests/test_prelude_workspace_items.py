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
