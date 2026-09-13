#!/usr/bin/env python3
"""Which binaries a build dir's post-link scanners are ALLOWED to judge.

The post-link scanners (scan_ud2, scan_self_recursive) sweep "every binary in
`build-<arch>/`". On x86 that is a real corpus: the `livecd` DAG re-rolls every
shipped binary on every build (each one is an `add_cd_file` dependency), so the
set of files in the build dir tracks the tree.

**arm64 has no livecd DAG.** `archcfg.py` builds a hand-picked list of ninja
targets and `qemu/mkramdisk.sh` stages a hand-picked list of files, and nothing
rebuilds anything else. So a `build-arm64/` accumulates binaries from whatever
broader `ninja` anyone ever ran in it, and the sweep grades those -- a verdict
that depends on which files happen to sit in a directory, not on the tree.

Measured 2026-09-13 on a real build-arm64: 131 binaries swept, of which **114
(87%) are rebuilt by nothing in the arm64 build**, and every one of the 131 was
7-8 days old. It bites both ways: a real finding in an ungoverned binary has
never blocked a gate (nobody rebuilds it, so nobody fixes it), and a lane can
eat a red arm64 sweep for a driver it never touched. Worse, the corpus changes
across a rebuild boundary, so a CORPUS change reads as a CODE change.

The fix is to make the corpus DECLARED rather than DISCOVERED. On arm64 the
declaration already exists and is already the thing that ships: the `RMC` lines
in `qemu/mkramdisk.sh`. Parsing them is the same mechanism
`tools/scan_boot_media_drift.py` already uses on the same file, and it keeps the
verdict a function of the tree -- mkramdisk.sh is source, the directory is not.
"""

from __future__ import annotations

import os
import re
from dataclasses import dataclass
from pathlib import Path

PE_SUFFIXES = (".sys", ".dll", ".exe", ".drv")

# Directories under a build dir that hold COPIES of binaries built elsewhere in
# the same tree (staging + strip caches), plus host tooling that is not part of
# the target OS. Judging a copy double-reports its original.
STAGING_DIRS = ("boot/stripped", "livecd", "bootcd", "CMakeFiles", "host-tools")

MKRAMDISK = "qemu/mkramdisk.sh"
# Any `"$BUILD_DIR/<rel>"` in the script, not just the `RMC` stage calls. Two
# shipped binaries are reachable no other way: csrss.exe is staged from a temp
# copy (its stack commit is patched first, so the RMC source is `$CSRSS_TMP`),
# and the VIBEOS_FEX block stages ucrtbase.dll with a raw `mcopy`. Matching the
# variable rather than one call site means a future stage that does not go
# through RMC still lands in the corpus instead of silently falling out of it.
_BUILD_DIR_REF = re.compile(r'"\$BUILD_DIR/([^"]+)"')
# The kmtest corpus is staged by a `find ... -name '*_drv.sys'` loop rather than
# by literal RMC lines, so it is declared here as a directory rule instead. It
# is governed: archcfg builds `kmtest_all`, so these DO re-roll with the tree.
KMTESTS_REL = "modules/rostests/kmtests"


@dataclass(frozen=True)
class Corpus:
    """The binaries a scanner may judge, and the provenance of that list."""

    paths: list[Path]
    basis: str
    #: Declared by mkramdisk.sh but absent from the build dir. Some staging is
    #: conditional (wow64, lsass, payloads), so this is reported, not fatal --
    #: mkramdisk's own REQUIRED_FILES check is what hard-fails a real gap.
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
            if os.path.splitext(name)[1].lower() in suffixes:
                out.append(Path(dirpath) / name)
    return sorted(out)


def declared_arm64_sources(repo: Path) -> list[str]:
    """Build-dir-relative paths of every PE `qemu/mkramdisk.sh` stages.

    Source of truth for "what ships on the arm64 boot media", read from the
    script that ships it. Non-PE references (hives, .nls) are dropped; PE
    references from conditional blocks (VIBEOS_FEX's ucrtbase.dll) are KEPT,
    because they ship in a supported configuration and ninja does rebuild them.
    """
    text = (repo / MKRAMDISK).read_text()
    return sorted(
        {
            rel
            for rel in _BUILD_DIR_REF.findall(text)
            if os.path.splitext(rel)[1].lower() in PE_SUFFIXES
        }
    )


def governed_binaries(
    build_dir: Path,
    repo: Path | None = None,
    *,
    suffixes: tuple[str, ...] = PE_SUFFIXES,
    staging_dirs: tuple[str, ...] = STAGING_DIRS,
) -> Corpus:
    """The binaries in `build_dir` whose contents track the current tree.

    `staging_dirs` is per-caller because the two post-link scanners historically
    prune different sets over the same build dir; each keeps its own so this
    change is arm64-only in effect.
    """
    build_dir = Path(build_dir)
    repo = Path(repo) if repo is not None else build_dir.resolve().parent
    if "arm64" not in build_dir.name:
        return Corpus(
            paths=_pruned_walk(build_dir, suffixes, staging_dirs),
            basis="build tree (the livecd DAG re-rolls every shipped binary each build)",
            missing=[],
        )

    paths: list[Path] = []
    missing: list[str] = []
    for rel in declared_arm64_sources(repo):
        p = build_dir / rel
        (paths if p.is_file() else missing).append(p if p.is_file() else rel)  # type: ignore[arg-type]
    kmtests = build_dir / KMTESTS_REL
    if kmtests.is_dir():
        paths += [p for p in _pruned_walk(kmtests, suffixes) if p.is_file()]
    return Corpus(
        paths=sorted(set(paths)),
        basis=f"{MKRAMDISK} staging list (arm64 has no livecd DAG; the build dir is not a corpus)",
        missing=missing,
    )
