"""Source-only traversal: exclude mutable output BEFORE scandir, never after.

Missing/unreadable source is an incomplete check, not an empty source corpus.
Build output churn is harmless because the walk never enters those directories.
"""

from __future__ import annotations

import fnmatch
import os
from collections.abc import Iterator
from pathlib import Path


def _raise(error: OSError) -> None:
    raise error


def source_walk(
    root: Path, *, exclude_prefixes: tuple[str, ...] = ()
) -> Iterator[tuple[Path, list[str], list[str]]]:
    root.stat()  # os.walk otherwise silently accepts an absent root
    if not root.is_dir():
        raise NotADirectoryError(root)
    if "target" in root.parts:
        return
    for parent, dirs, files in os.walk(root, topdown=True, onerror=_raise, followlinks=False):
        relative = Path(parent).relative_to(root)
        dirs[:] = sorted(
            name
            for name in dirs
            if name != "target" and not (relative / name).as_posix().startswith(exclude_prefixes)
        )
        yield Path(parent), dirs, sorted(files)


def source_files(root: Path, pattern: str = "*.rs") -> Iterator[Path]:
    for parent, _, files in source_walk(root):
        for name in files:
            if fnmatch.fnmatchcase(name, pattern):
                yield parent / name
