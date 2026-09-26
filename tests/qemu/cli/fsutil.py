"""Filesystem measurements done in-process, instead of shelling out to host tools.

A `du`/`find` shell-out is a dependency on whatever the host happens to have installed, and
its failure modes are the host tool's, not ours: a missing binary, a pipeline whose status
is read from the wrong stage, an orphan that outlives a killed parent. These are a few dozen
lines of Python with none of that.
"""

from __future__ import annotations

import os
from pathlib import Path


def disk_usage_bytes(path: Path) -> int:
    """Allocated bytes under `path`, as `du -s -B1` reports them: st_blocks (not apparent
    size), each hardlinked inode counted ONCE, symlinks not followed, directories included.

    A file that vanishes mid-walk is skipped (a live build rewrites its tree constantly); any
    other error RAISES, as `du` exiting nonzero did — a size that silently omits an unreadable
    subtree is a wrong answer, not a smaller one."""
    seen: set[tuple[int, int]] = set()
    total = 0
    stack = [str(path)]
    while stack:
        d = stack.pop()
        try:
            st = os.lstat(d)
        except FileNotFoundError:
            continue
        total += st.st_blocks * 512
        try:
            it = os.scandir(d)
        except FileNotFoundError:
            continue
        with it:
            for e in it:
                try:
                    if e.is_dir(follow_symlinks=False):
                        stack.append(e.path)
                        continue
                    st = e.stat(follow_symlinks=False)
                except FileNotFoundError:
                    continue
                if st.st_nlink > 1:
                    key = (st.st_dev, st.st_ino)
                    if key in seen:
                        continue
                    seen.add(key)
                total += st.st_blocks * 512
    return total
