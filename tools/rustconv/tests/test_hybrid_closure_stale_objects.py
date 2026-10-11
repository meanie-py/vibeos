"""A flipped TU's leftover .obj must not be counted as a consumer.

ninja does not delete the object of a source removed from a CMake list, so every TU we flip
leaves a stale `.obj` under the link root indefinitely. The closure's undefined side globs
`*.obj`, so those leftovers keep voting: `SH_FormatByteSize` measured as still-needed-by
`drvdefext.cpp.obj` long after drvdefext.cpp became Rust, and `SHELL32_IsShellFolderNamespace-
ItemHidden` as needed-by `CDesktopFolder.cpp.obj`. Both consumers were dead, and the phantom
demand makes the next flip look harder than it is -- it argues for keeping C++ linkage on a
symbol nothing in the build still calls.

The filter is deliberately one-sided. Dropping an object we should have kept UNDERSTATES the
closure, which reads as "this TU is free to leave" right up until the link fails, so an object
is dropped only when its source is provably flipped: present under the target dir, absent from
every live list. Anything else is kept.
"""

from __future__ import annotations

from pathlib import Path

from rustconv.hybrid_closure import (
    _live_source_names,
    _stale_object_predicate,
)


def _repo(tmp_path: Path, cmake_body: str, sources: list[str]) -> Path:
    target = tmp_path / "dll" / "win32" / "shell32"
    target.mkdir(parents=True)
    (target / "CMakeLists.txt").write_text(cmake_body)
    for src in sources:
        p = target / src
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text("// source\n")
    return tmp_path


CMAKE = """
list(APPEND SOURCE
    folders/CFSFolder.cpp
    # folders/CDesktopFolder.cpp -- replaced by Rust; cohab via libshell32_rust.a
    dialogs/filedefext.cpp
)
"""


def test_a_flipped_tus_object_is_stale(tmp_path):
    repo = _repo(
        tmp_path,
        CMAKE,
        ["folders/CFSFolder.cpp", "folders/CDesktopFolder.cpp", "dialogs/filedefext.cpp"],
    )
    is_stale = _stale_object_predicate(repo, "dll/win32/shell32")
    assert is_stale(Path("CDesktopFolder.cpp.obj"))


def test_a_live_tus_object_is_kept(tmp_path):
    repo = _repo(
        tmp_path,
        CMAKE,
        ["folders/CFSFolder.cpp", "folders/CDesktopFolder.cpp", "dialogs/filedefext.cpp"],
    )
    is_stale = _stale_object_predicate(repo, "dll/win32/shell32")
    assert not is_stale(Path("CFSFolder.cpp.obj"))
    assert not is_stale(Path("filedefext.cpp.obj"))


def test_an_object_from_another_target_is_kept(tmp_path):
    """Its source is not under this target dir, so we cannot prove anything -- keep it."""
    repo = _repo(tmp_path, CMAKE, ["folders/CFSFolder.cpp"])
    is_stale = _stale_object_predicate(repo, "dll/win32/shell32")
    assert not is_stale(Path("recyclebin_v5.cpp.obj"))
    assert not is_stale(Path("shlfileop.cpp.obj"))


def test_no_target_dir_disables_the_filter(tmp_path):
    """Without a target dir there is no CMake to ask, so nothing may be dropped."""
    repo = _repo(tmp_path, CMAKE, ["folders/CDesktopFolder.cpp"])
    is_stale = _stale_object_predicate(repo, "")
    assert not is_stale(Path("CDesktopFolder.cpp.obj"))


def test_a_missing_target_dir_disables_the_filter(tmp_path):
    is_stale = _stale_object_predicate(tmp_path, "dll/win32/nonexistent")
    assert not is_stale(Path("anything.cpp.obj"))


def test_live_source_names_ignores_commented_entries(tmp_path):
    repo = _repo(tmp_path, CMAKE, [])
    names = _live_source_names(repo, "dll/win32/shell32")
    assert names == {"CFSFolder.cpp", "filedefext.cpp"}


def test_a_trailing_comment_does_not_hide_a_live_entry(tmp_path):
    body = "list(APPEND SOURCE\n    folders/CFSFolder.cpp  # still C++\n)\n"
    repo = _repo(tmp_path, body, [])
    assert _live_source_names(repo, "dll/win32/shell32") == {"CFSFolder.cpp"}


def test_a_non_object_name_is_not_mangled(tmp_path):
    """Guard the `.obj` strip: a name without the suffix must not lose its last four chars."""
    repo = _repo(tmp_path, CMAKE, ["folders/CDesktopFolder.cpp"])
    is_stale = _stale_object_predicate(repo, "dll/win32/shell32")
    assert not is_stale(Path("CDesktopFolder.cpp"))
