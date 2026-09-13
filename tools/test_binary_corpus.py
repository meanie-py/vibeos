"""The post-link scanners must judge a DECLARED corpus, not a directory listing.

arm64 has no livecd DAG, so `build-arm64/` accumulates binaries that nothing in
the arm64 build rebuilds. Grading those makes the verdict a function of which
files happen to sit in a directory: a corpus change reads as a code change, and
a real finding in an unrebuilt binary can never be fixed by anyone.
"""

from __future__ import annotations

import os
from pathlib import Path

import pytest

from tools.binary_corpus import PE_SUFFIXES, declared_arm64_sources, governed_binaries

MKRAMDISK = """#!/bin/bash
BUILD_DIR="$PROJECT_DIR/build-arm64"
RMC "$BUILD_DIR/drivers/serial/pl011/pl011.sys"  ::/reactos/system32/drivers/pl011.sys
RMC "$BUILD_DIR/win32ss/printing/base/winspool/winspool.drv" ::/reactos/system32/winspool.drv
RMC "$BUILD_DIR/boot/bootdata/system"            ::/reactos/system32/config/system
"""


def _repo(tmp_path: Path, arch: str, files: list[str]) -> tuple[Path, Path]:
    repo = tmp_path / "repo"
    (repo / "qemu").mkdir(parents=True)
    (repo / "qemu" / "mkramdisk.sh").write_text(MKRAMDISK)
    build = repo / f"build-{arch}"
    for rel in files:
        p = build / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(b"MZ")
    return repo, build


DECLARED = [
    "drivers/serial/pl011/pl011.sys",
    "win32ss/printing/base/winspool/winspool.drv",
]


def test_arm64_corpus_excludes_a_binary_nothing_stages(tmp_path):
    """The defect: a stray binary in build-arm64/ was swept as if it were shipped."""
    repo, build = _repo(tmp_path, "arm64", [*DECLARED, "base/shell/filebrowser/rshell.dll"])
    rels = {str(p.relative_to(build)) for p in governed_binaries(build, repo).paths}
    assert rels == set(DECLARED)


def test_x86_corpus_is_the_build_tree(tmp_path):
    """x86 keeps the tree: the livecd DAG re-rolls every shipped binary each build."""
    repo, build = _repo(tmp_path, "i386", [*DECLARED, "base/shell/filebrowser/rshell.dll"])
    rels = {str(p.relative_to(build)) for p in governed_binaries(build, repo).paths}
    assert rels == {*DECLARED, "base/shell/filebrowser/rshell.dll"}


def test_declared_but_absent_is_reported_not_dropped(tmp_path):
    """A shipped binary missing from the build dir is a gap, not a smaller corpus."""
    repo, build = _repo(tmp_path, "arm64", DECLARED[:1])
    corpus = governed_binaries(build, repo)
    assert corpus.missing == ["win32ss/printing/base/winspool/winspool.drv"]


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


def test_the_real_mkramdisk_declares_a_nonempty_arm64_corpus():
    """Guard the failure mode this change INTRODUCES: a silently empty corpus.

    A gate that grades zero binaries passes. If a refactor of mkramdisk.sh ever
    stops spelling `"$BUILD_DIR/<path>"`, this fails here rather than turning the
    arm64 binary gate into a no-op that reports PASSED.
    """
    repo = Path(__file__).resolve().parent.parent
    declared = declared_arm64_sources(repo)
    assert len(declared) > 50, f"arm64 staging list collapsed to {len(declared)} binaries"
    assert "ntoskrnl/ntoskrnl.exe" in declared
    assert "win32ss/win32k.sys" in declared
