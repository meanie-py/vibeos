"""ninja does not delete the object of a source you remove from a CMake list.

So a flipped TU keeps a stale `.obj` on disk indefinitely, and measuring its retained closure
against that object reports a TU that has already left the build as though it were still live,
with a full set of consumers. That is not a degraded answer, it is the opposite of the right
one. Liveness comes from CMake, not from the object tree.
"""

from __future__ import annotations

from rustconv.hybrid_closure import is_live_in_cmake


def _repo(tmp_path, body: str, rel: str = "dll/win32/shell32"):
    d = tmp_path / rel
    d.mkdir(parents=True)
    (d / "CMakeLists.txt").write_text(body)
    return tmp_path


TARGET = "dll/win32/shell32"


def test_an_uncommented_entry_is_live(tmp_path):
    repo = _repo(tmp_path, "list(APPEND SOURCE\n    folders/CRecycleBin.cpp\n)\n")
    assert is_live_in_cmake(repo, "dll/win32/shell32/folders/CRecycleBin.cpp", TARGET)


def test_a_commented_out_entry_is_not_live(tmp_path):
    repo = _repo(
        tmp_path,
        "list(APPEND SOURCE\n"
        "    # folders/CNetFolder.cpp — replaced by Rust; cohab via libshell32_rust.a\n"
        ")\n",
    )
    assert not is_live_in_cmake(repo, "dll/win32/shell32/folders/CNetFolder.cpp", TARGET)


def test_a_prose_mention_in_a_comment_does_not_resurrect_it(tmp_path):
    """The flip comments NAME the file they removed, so substring matching would always say yes."""
    repo = _repo(
        tmp_path,
        "list(APPEND SOURCE\n"
        "    # folders/CFSFolder.cpp — replaced by Rust. The file-local CFileSysEnum\n"
        "    # enumerator is shell32-rust/src/file_sys_enum.rs.\n"
        "    folders/CRecycleBin.cpp\n"
        ")\n",
    )
    assert not is_live_in_cmake(repo, "dll/win32/shell32/folders/CFSFolder.cpp", TARGET)
    assert is_live_in_cmake(repo, "dll/win32/shell32/folders/CRecycleBin.cpp", TARGET)


def test_a_trailing_comment_after_a_live_entry_still_counts_as_live(tmp_path):
    repo = _repo(tmp_path, "list(APPEND SOURCE\n    folders/CRecycleBin.cpp # keep\n)\n")
    assert is_live_in_cmake(repo, "dll/win32/shell32/folders/CRecycleBin.cpp", TARGET)


def test_a_nested_cmakelists_counts(tmp_path):
    """shell32 pulls in shellmenu/shelldesktop/shellrecyclebin, each with its own list."""
    repo = _repo(tmp_path, "add_subdirectory(shellmenu)\n")
    sub = tmp_path / "dll/win32/shell32/shellmenu"
    sub.mkdir(parents=True)
    (sub / "CMakeLists.txt").write_text("list(APPEND SOURCE\n    CMergedFolder.cpp\n)\n")
    assert is_live_in_cmake(repo, "dll/win32/shell32/shellmenu/CMergedFolder.cpp", TARGET)


def test_a_missing_target_dir_reports_not_live_rather_than_raising(tmp_path):
    assert not is_live_in_cmake(tmp_path, "dll/win32/shell32/x.cpp", TARGET)
