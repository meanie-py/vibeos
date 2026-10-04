"""The post-link scanners judge the build tree minus staging copies, on every arch.

arm64 used to be graded from mkramdisk.sh's hand-written copy lines, because it
had no livecd DAG and build-arm64/ accumulated binaries nothing rebuilt. It builds
`livecd` now, and its ramdisk is a symlink tree of the livecd list, so the risk
that remains is the opposite one: grading the symlinked COPIES next to their
originals.
"""

from __future__ import annotations

import os
from pathlib import Path

import pytest

from tools.binary_corpus import PE_SUFFIXES, STAGING_DIRS, governed_binaries

DECLARED = [
    "drivers/serial/pl011/pl011.sys",
    "win32ss/printing/base/winspool/winspool.drv",
]


def _repo(tmp_path: Path, arch: str, files: list[str]) -> tuple[Path, Path]:
    repo = tmp_path / "repo"
    build = repo / f"build-{arch}"
    for rel in files:
        p = build / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(b"MZ")
    return repo, build


@pytest.mark.parametrize("arch", ["i386", "amd64", "arm64"])
def test_every_arch_corpus_is_the_build_tree(tmp_path, arch):
    """Same rule on all three: the livecd DAG re-rolls every shipped binary each build."""
    repo, build = _repo(tmp_path, arch, [*DECLARED, "base/shell/rshell/rshell.dll"])
    rels = {str(p.relative_to(build)) for p in governed_binaries(build, repo).paths}
    assert rels == {*DECLARED, "base/shell/rshell/rshell.dll"}


def test_the_ramdisk_symlink_tree_is_not_graded_twice(tmp_path):
    """mkramdisk.sh lays the livecd list out as symlinks under boot/ramdisk/.

    os.walk lists a symlink to a file as a file, so without the prune every shipped
    binary would be reported once for itself and once for its ramdisk link.
    """
    repo, build = _repo(tmp_path, "arm64", DECLARED)
    tree = build / "boot" / "ramdisk" / "reactos" / "system32" / "drivers"
    tree.mkdir(parents=True)
    (tree / "pl011.sys").symlink_to(build / DECLARED[0])
    (build / "boot" / "ramdisk-overlay").mkdir()
    (build / "boot" / "ramdisk-overlay" / "csrss.exe").write_bytes(b"MZ")
    rels = {
        str(p.relative_to(build))
        for p in governed_binaries(build, repo, staging_dirs=STAGING_DIRS).paths
    }
    assert rels == set(DECLARED)


def test_drv_is_a_pe_suffix(tmp_path):
    """winspool.drv ships on every arch and the old `.sys/.dll/.exe` sweep never saw it."""
    assert ".drv" in PE_SUFFIXES
    repo, build = _repo(tmp_path, "arm64", DECLARED)
    names = {p.name for p in governed_binaries(build, repo).paths}
    assert "winspool.drv" in names


@pytest.mark.parametrize(
    ("token", "pruned"),
    [("CMakeFiles", "CMakeFiles/x.dll"), ("/boot/stripped/", "boot/stripped/y.dll")],
)
def test_staging_dirs_are_pruned_in_both_spellings(tmp_path, token, pruned):
    """The two scanners spell their skip tokens differently over the same tree.

    scan_ud2 says "boot/stripped", scan_self_recursive says "/boot/stripped/".
    A matcher that honours only one spelling silently stops pruning for the other
    caller, which double-reports every finding through its staged copy.
    """
    strays = ["CMakeFiles/x.dll", "boot/stripped/y.dll"]
    repo, build = _repo(tmp_path, "i386", [*DECLARED, *strays])
    corpus = governed_binaries(build, repo, staging_dirs=(token,))
    rels = {str(p.relative_to(build)) for p in corpus.paths}
    kept = {*DECLARED, *strays} - {pruned}
    assert rels == kept


def test_staging_dirs_are_pruned_during_the_walk_not_after(tmp_path, monkeypatch):
    """Pruning must happen in os.walk's dirnames, so the pruned tree is never descended.

    Filtering paths afterwards (`rglob("*")` then `if token in str(p)`) finishes
    descending into the staging tree first — wasted work, and a live window for a
    FileNotFoundError when a concurrent build removes a directory underneath it.
    """
    repo, build = _repo(tmp_path, "i386", DECLARED)
    deep = build / "livecd" / "a" / "b" / "c"
    deep.mkdir(parents=True)
    (deep / "z.dll").write_bytes(b"MZ")
    seen: list[str] = []
    real_walk = os.walk

    def spy(top, *a, **kw):
        for dirpath, dirnames, filenames in real_walk(top, *a, **kw):
            seen.append(dirpath)
            yield dirpath, dirnames, filenames

    monkeypatch.setattr("tools.binary_corpus.os.walk", spy)
    governed_binaries(build, repo, staging_dirs=("livecd",))
    # Assert the spy actually ran FIRST: an implementation that went back to
    # rglob-then-filter would call os.walk zero times, and "no livecd in an
    # empty list" is true for the wrong reason.
    assert seen, "os.walk was never called — this assertion would pass vacuously"
    assert not any("livecd" in s for s in seen), "descended into a pruned directory"
