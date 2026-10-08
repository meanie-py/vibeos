#!/usr/bin/env python3
"""Which binaries a build dir's post-link scanners are ALLOWED to judge.

The post-link scanners (scan_ud2, scan_self_recursive) sweep "every binary in
`build-<arch>/`". That is a real corpus because every arch builds its `livecd`
DAG: each shipped binary is an `add_cd_file` dependency, so the set of files in
the build dir is re-rolled with the tree on every build.

arm64 was the exception until 2026-10-03. It built a hand-picked list of ninja
targets and `qemu/mkramdisk.sh` staged a hand-picked list of files, so
`build-arm64/` accumulated binaries nothing rebuilt (measured 2026-09-13: 114 of
131 swept binaries, every one 7-8 days old), and this module graded the RMC lines
of mkramdisk.sh instead of the directory. arm64 now builds `livecd` and its
ramdisk is laid out from the livecd list, so it takes the same corpus as x86.

What is pruned is COPIES: staging and strip caches, the ramdisk's symlink tree,
and host tooling. Judging a copy double-reports its original.
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path

PE_SUFFIXES = (".sys", ".dll", ".exe", ".drv")

# Directories under a build dir that hold COPIES of binaries built elsewhere in
# the same tree (staging + strip caches), plus host tooling that is not part of
# the target OS. Judging a copy double-reports its original.
STAGING_DIRS = (
    "boot/stripped",
    "boot/ramdisk",
    "boot/ramdisk-overlay",
    "livecd",
    "bootcd",
    "CMakeFiles",
    "host-tools",
)


# Files that carry a PE suffix but are NOT a PE image. The BIOS loader freeldr.sys
# is the 16-bit real-mode stub (frldr16.bin) with the PE body (freeldr_pe.exe)
# appended, so no PE tool can read it; the body is in the corpus under its own
# name. Paths are build-dir-relative.
CONTAINERS = ("boot/freeldr/freeldr/freeldr.sys",)


@dataclass(frozen=True)
class Corpus:
    """The binaries a scanner may judge, and the provenance of that list."""

    paths: list[Path]
    basis: str
    #: Declared but absent from the build dir. Always empty since the corpus is the
    #: tree; kept for the callers that report it.
    missing: list[str]


def _pruned_walk(
    root: Path, suffixes: tuple[str, ...], staging: tuple[str, ...] = STAGING_DIRS
) -> list[Path]:
    """Every PE under `root`, pruning staging dirs DURING the walk.

    Pruning during the descent rather than filtering paths afterwards is the
    point: `rglob("*")` finishes descending into `livecd/` before any filter
    sees a path, which is both wasted work and a live window for a
    FileNotFoundError out of the generator when a concurrent build removes a
    directory underneath it.
    """
    out: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(root):
        rel = os.path.relpath(dirpath, root)
        # Slash-wrap so a caller's token spells either way: "CMakeFiles" and
        # "/boot/stripped/" must both match, and neither may match a substring
        # that happens to straddle a component boundary.
        dirnames[:] = [
            d
            for d in dirnames
            if not any(
                e in "/" + os.path.normpath(os.path.join(rel, d)).replace(os.sep, "/") + "/"
                for e in staging
            )
        ]
        for name in filenames:
            if os.path.splitext(name)[1].lower() not in suffixes:
                continue
            if os.path.normpath(os.path.join(rel, name)).replace(os.sep, "/") in CONTAINERS:
                continue
            out.append(Path(dirpath) / name)
    return sorted(out)


def governed_binaries(
    build_dir: Path,
    repo: Path | None = None,
    *,
    suffixes: tuple[str, ...] = PE_SUFFIXES,
    staging_dirs: tuple[str, ...] = STAGING_DIRS,
) -> Corpus:
    """The binaries in `build_dir` whose contents track the current tree.

    `staging_dirs` is per-caller because the two post-link scanners historically
    prune different sets over the same build dir.
    """
    build_dir = Path(build_dir)
    return Corpus(
        paths=_pruned_walk(build_dir, suffixes, staging_dirs),
        basis="build tree (the livecd DAG re-rolls every shipped binary each build)",
        missing=[],
    )
