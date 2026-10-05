"""tools/ramdisk_tree.py lays the livecd graft list out for the arm64 FAT ramdisk.

Each test is a way the old hand-staged ramdisk went wrong, or a way the list itself
is inconsistent, that the tree must handle loudly rather than by quietly dropping files.
"""

from __future__ import annotations

from pathlib import Path

from tools.ramdisk_tree import build


def _src(tmp: Path, rel: str, data: bytes = b"MZ") -> Path:
    p = tmp / "src" / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_bytes(data)
    return p


def _lst(tmp: Path, name: str, lines: list[str]) -> Path:
    p = tmp / name
    p.write_text("\n".join(lines) + "\n")
    return p


def test_every_entry_lands_at_its_destination(tmp_path):
    a = _src(tmp_path, "a.dll")
    b = _src(tmp_path, "b.inf", b"[Version]")
    lst = _lst(tmp_path, "l.lst", [f"reactos/system32/a.dll={a}", f"reactos/inf/b.inf={b}"])
    out = tmp_path / "tree"
    assert build([lst], out, None) == a.stat().st_size + b.stat().st_size
    assert (out / "reactos/system32/a.dll").resolve() == a
    assert (out / "reactos/inf/b.inf").resolve() == b


def test_a_missing_source_fails_the_whole_tree(tmp_path):
    """A file silently absent from the media is the failure the tool exists to end."""
    a = _src(tmp_path, "a.dll")
    lst = _lst(
        tmp_path, "l.lst", [f"reactos/a.dll={a}", "reactos/setup.exe=/nonexistent/setup.exe"]
    )
    out = tmp_path / "tree"
    assert build([lst], out, None) == -1
    assert not out.exists()


def test_directory_case_folds_to_the_first_spelling(tmp_path):
    """FAT is case-insensitive; the list says both reactos/Fonts and reactos/fonts."""
    a = _src(tmp_path, "a.ttf")
    b = _src(tmp_path, "b.ttf")
    lst = _lst(tmp_path, "l.lst", [f"reactos/Fonts/a.ttf={a}", f"reactos/fonts/b.ttf={b}"])
    out = tmp_path / "tree"
    build([lst], out, None)
    assert sorted(p.name for p in (out / "reactos").iterdir()) == ["Fonts"]
    assert {p.name for p in (out / "reactos/Fonts").iterdir()} == {"a.ttf", "b.ttf"}


def test_a_later_list_overrides_an_earlier_one(tmp_path):
    """The arm64 overlay (the stack-patched csrss.exe) replaces the livecd's copy."""
    stock = _src(tmp_path, "stock/csrss.exe", b"MZstock")
    patched = _src(tmp_path, "patched/csrss.exe", b"MZpatched")
    base = _lst(tmp_path, "base.lst", [f"reactos/system32/csrss.exe={stock}"])
    overlay = _lst(tmp_path, "overlay.lst", [f"reactos/system32/csrss.exe={patched}"])
    out = tmp_path / "tree"
    build([base, overlay], out, None)
    assert (out / "reactos/system32/csrss.exe").resolve() == patched


def test_one_list_naming_a_destination_twice_with_different_bytes_fails(tmp_path):
    """testdata/old/test.ttf and testdata/test.ttf once collided like this; which copy
    a test saw depended on how the media builder resolved the duplicate."""
    a = _src(tmp_path, "new/test.ttf", b"new")
    b = _src(tmp_path, "old/test.ttf", b"old")
    lst = _lst(
        tmp_path,
        "l.lst",
        [f"reactos/bin/testdata/test.ttf={a}", f"reactos/bin/testdata/test.ttf={b}"],
    )
    assert build([lst], tmp_path / "tree", None) == -1


def test_identical_duplicates_within_a_list_are_fine(tmp_path):
    a = _src(tmp_path, "x/a.dll", b"same")
    b = _src(tmp_path, "y/a.dll", b"same")
    lst = _lst(tmp_path, "l.lst", [f"reactos/a.dll={a}", f"reactos/a.dll={b}"])
    assert build([lst], tmp_path / "tree", None) == 4


def test_directory_sources_make_empty_directories(tmp_path):
    """The list creates Profiles/... and reactos/TEMP by grafting an empty directory."""
    empty = tmp_path / "empty"
    empty.mkdir()
    lst = _lst(
        tmp_path,
        "l.lst",
        [str(empty), f"reactos/TEMP={empty}", f"Profiles/All Users/Desktop={empty}"],
    )
    out = tmp_path / "tree"
    assert build([lst], out, None) == 0
    assert (out / "reactos/TEMP").is_dir()
    assert (out / "Profiles/All Users/Desktop").is_dir()


def test_canonical_build_root_is_mapped_back(tmp_path):
    """Sources are written against /tmp/vibeos, which is only mounted during a build."""
    a = _src(tmp_path, "build-arm64/a.dll")
    lst = _lst(tmp_path, "l.lst", ["reactos/a.dll=/tmp/vibeos/build-arm64/a.dll"])
    out = tmp_path / "tree"
    build([lst], out, ("/tmp/vibeos/", f"{tmp_path}/src/"))
    assert (out / "reactos/a.dll").resolve() == a
