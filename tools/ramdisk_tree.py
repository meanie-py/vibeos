#!/usr/bin/env python3
"""ramdisk_tree.py -- lay mkisofs graft lists out as a symlink tree for the arm64 ramdisk.

The x86 livecd is `mkisofs -graft-points -path-list livecd.stripped.lst`: every
add_cd_file(... FOR livecd) lands on the media by construction. arm64 boots a FAT ramdisk
instead of an ISO, and for most of its life that ramdisk was filled by ~250 hand-written
mcopy lines in qemu/mkramdisk.sh. Every file the livecd gained afterwards had to be noticed
and copied by hand, and mostly was not: setup.exe, syssetup.dll and every .inf were absent,
so the first-boot `setup -mini` the livecd hive asks for failed silently on every arm64
boot, along with everything it registers (COM classes, Tcpip/Dhcp/Dnscache, devices).

This consumes the SAME list the x86 ISO is built from, plus an arm64 overlay, and builds a
directory of symlinks that `mcopy -s` copies into the FAT image in one call.

  * FAT is case-insensitive and the lists are not consistent (reactos/Fonts and
    reactos/fonts both appear), so directory components are folded to their first spelling.
  * Two lists naming the same destination: a later list overrides an earlier one (that is
    what the overlay is for); within ONE list it is an error unless the bytes are identical.
  * A missing source is an error. mkisofs would fail on it too, and a file silently absent
    from the media is the failure this tool exists to end.
  * Sources are written against the canonical build root (/tmp/vibeos, tools/canon-run.sh);
    --canon maps that prefix back to this worktree.

Output: the tree under --out (recreated), and the total payload size in bytes on stdout.
"""

from __future__ import annotations

import argparse
import filecmp
import os
import shutil
import sys
from pathlib import Path


def parse_lst(path: Path, canon: tuple[str, str] | None) -> list[tuple[str, str]]:
    """(dest, src) pairs. Lines with no '=' are mkisofs root grafts and carry no dest."""
    out = []
    for raw in path.read_text().splitlines():
        line = raw.strip()
        if not line or "=" not in line:
            continue
        dest, src = line.split("=", 1)
        if canon and src.startswith(canon[0]):
            src = canon[1] + src[len(canon[0]) :]
        out.append((dest.strip("/"), src))
    return out


def build(lists: list[Path], out: Path, canon: tuple[str, str] | None) -> int:
    spelling: dict[str, str] = {}  # lowercased dir path -> first spelling seen

    def fold(dest: str) -> str:
        parts = dest.split("/")
        folded = []
        for i in range(len(parts) - 1):
            key = "/".join(p.lower() for p in parts[: i + 1])
            spelling.setdefault(key, parts[i])
            folded.append(spelling[key])
        return "/".join([*folded, parts[-1]])

    entries: dict[str, tuple[str, str, int]] = {}  # lower dest -> (dest, src, list index)
    dirs: dict[str, str] = {}
    errors = []
    for li, lst in enumerate(lists):
        for dest, src in parse_lst(lst, canon):
            dest = fold(dest)
            if os.path.isdir(src):
                dirs.setdefault(dest.lower(), dest)
                continue
            if not os.path.isfile(src):
                errors.append(f"{lst.name}: {dest}: source missing: {src}")
                continue
            prev = entries.get(dest.lower())
            if prev and prev[2] == li and not filecmp.cmp(prev[1], src, shallow=False):
                errors.append(
                    f"{lst.name}: {dest}: named twice with different bytes ({prev[1]} / {src})"
                )
                continue
            entries[dest.lower()] = (dest, src, li)
    if errors:
        for e in errors:
            print(f"ramdisk_tree: {e}", file=sys.stderr)
        print(
            f"ramdisk_tree: {len(errors)} error(s); refusing to build a partial tree",
            file=sys.stderr,
        )
        return -1

    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    for d in dirs.values():
        (out / d).mkdir(parents=True, exist_ok=True)
    total = 0
    for dest, src, _ in entries.values():
        target = out / dest
        target.parent.mkdir(parents=True, exist_ok=True)
        target.symlink_to(os.path.abspath(src))
        total += os.path.getsize(src)
    return total


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("lists", nargs="+", type=Path, help="graft lists, later ones override earlier")
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--canon", nargs=2, metavar=("FROM", "TO"), help="rewrite a source path prefix")
    args = ap.parse_args()
    total = build(args.lists, args.out, tuple(args.canon) if args.canon else None)
    if total < 0:
        return 1
    print(total)
    return 0


if __name__ == "__main__":
    sys.exit(main())
