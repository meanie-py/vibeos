"""The free-functions path after a TU is flipped, and the support it needs.

Three regressions, all found reassembling dialogs/filetypes.cpp AFTER its cohab flip:

* A flipped TU has no compile command of its own, and `_sibling_target_entry` only looked for
  a compiled sibling in the SAME directory. Once the last dialogs/*.cpp flipped there was none,
  so a bank could no longer be reassembled post-flip at all. The flags belong to the CMake
  target that LISTS the file (shell32's CMakeLists lists `dialogs/filetypes.cpp`).
* `LOWORD(x)` -> `LOWORD!(x)` and `StringCbPrintfW(` -> `StringCbPrintfW!(` fire on both paths,
  but only --cpp-com emitted the definitions; free-functions TUs hand-wrote them.
* The generated printf helper knew only `%s`: `%%` came out as `%%` (COpenWithList registered
  `"app.exe" "%%1"`), and any other conversion was copied through literally.
"""

from __future__ import annotations

import json
from pathlib import Path

from rustconv.cpp_com import (
    LOWORD_MACRO,
    STRING_CB_PRINTF_W_SUPPORT,
    defined_names,
    drop_defined_items,
    render_rewrite_support,
)
from rustconv.macro_authority import _sibling_target_entry


def _db(repo: Path, arch: str, entries: list[tuple[Path, str]]) -> None:
    build = repo / f"build-{arch}"
    build.mkdir(parents=True, exist_ok=True)
    build.joinpath("compile_commands.json").write_text(
        json.dumps([{"file": str(f), "command": f"clang {flags} -c {f}"} for f, flags in entries])
    )


def test_fully_flipped_subdir_borrows_from_the_owning_target(tmp_path):
    repo = tmp_path / "repo"
    target = repo / "dll" / "win32" / "shell32"
    (target / "dialogs").mkdir(parents=True)
    (target / "dialogs" / "filetypes.cpp").write_text("")
    # the target's CMakeLists names the file by its path RELATIVE to the target, commented out
    (target / "CMakeLists.txt").write_text("list(APPEND SOURCE\n    # dialogs/filetypes.cpp\n)\n")
    # nothing compiled is left in dialogs/ — only the target's own TU
    _db(repo, "amd64", [(target / "shell32.cpp", "-std=gnu++17 -I/shell32-inc -D_SHELL32_")])

    argv = _sibling_target_entry(repo, "amd64", target / "dialogs" / "filetypes.cpp")
    assert argv is not None, "a flipped TU must still resolve its target's flags"
    assert "-I/shell32-inc" in argv and "-D_SHELL32_" in argv
    assert str(target / "dialogs" / "filetypes.cpp") in argv


def test_an_unrelated_ancestor_is_not_borrowed_from(tmp_path):
    """Only the directory whose CMakeLists NAMES the file counts — not any ancestor."""
    repo = tmp_path / "repo"
    other = repo / "dll" / "win32" / "other"
    (other / "sub").mkdir(parents=True)
    (other / "sub" / "orphan.cpp").write_text("")
    (other / "CMakeLists.txt").write_text("list(APPEND SOURCE something_else.cpp)\n")
    _db(repo, "amd64", [(other / "something_else.cpp", "-I/wrong")])
    assert _sibling_target_entry(repo, "amd64", other / "sub" / "orphan.cpp") is None


def test_item_filter_drops_whole_items_and_single_extern_declarations():
    text = (
        "pub const A: u32 = 1;\n"
        "macro_rules! LOWORD { ($v:expr) => { $v }; }\n"
        'extern "system" {\n'
        "    fn Keep(a: u32) -> u32;\n"
        "    fn Gone(\n"
        "        a: u32,\n"
        "    ) -> u32;\n"
        "}\n"
        'extern "C" {\n'
        "    fn AlsoGone();\n"
        "}\n"
        "unsafe fn helper() -> u32 {\n"
        "    { 1 }\n"
        "}\n"
    )
    assert defined_names(text) >= {"A", "LOWORD", "Keep", "Gone", "AlsoGone", "helper"}
    out = drop_defined_items(text, {"A", "LOWORD", "Gone", "AlsoGone"})
    assert "fn Keep" in out and "fn helper" in out
    assert "Gone" not in out and "LOWORD" not in out and "const A" not in out
    assert 'extern "C"' not in out, "an extern block left empty must go too"


def test_rewrite_support_emits_only_what_the_rewrites_introduced():
    assert render_rewrite_support("let x = 1;", "") == ""
    assert LOWORD_MACRO in render_rewrite_support("let id = LOWORD!(w);", "")
    both = render_rewrite_support("LOWORD!(w); StringCbPrintfW!(d, n, f, a);", "")
    assert "macro_rules! StringCbPrintfW" in both and "fn cpp_com_string_cb_printf_w" in both
    # and NOT the rest of the --cpp-com closure (its DWORD-typed Win32 constants would shadow
    # the island's and change signedness under existing comparisons)
    assert "pub const" not in both


def test_the_preamble_is_the_authority():
    preamble = (
        "unsafe fn cpp_com_string_cb_printf_w() {}\nmacro_rules! StringCbPrintfW { () => {}; }\n"
    )
    out = render_rewrite_support("LOWORD!(w); StringCbPrintfW!(d, n, f, a);", preamble)
    assert "macro_rules! LOWORD" in out, "not defined by the preamble, so supplied"
    assert "StringCbPrintfW" not in out and "cpp_com_string_cb_printf_w" not in out


def test_printf_helper_handles_percent_percent_and_refuses_unknown_conversions():
    # Rust cannot run here; assert the contract is in the emitted text.
    helper = STRING_CB_PRINTF_W_SUPPORT
    assert "spec == b'%' as WCHAR" in helper, "`%%` must produce a single `%`"
    assert "spec != b's' as WCHAR" in helper and "0x80070057u32" in helper, (
        "an unimplemented conversion must be REFUSED (STRSAFE_E_INVALID_PARAMETER), "
        "never copied through literally"
    )
