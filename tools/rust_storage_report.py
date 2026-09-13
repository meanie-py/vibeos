"""Measure allocated bytes and byte-identical Rust artifacts in one local build.

Read-only, never follows symlinks or searches sibling worktrees. Hashing is
explicit because it reads every .a/.rlib/.rmeta byte; do not put it on a hot path.
"""

import argparse
import collections
import hashlib
import json
import os
from pathlib import Path


def measure(root: Path) -> dict:
    totals = collections.Counter()
    groups = collections.defaultdict(dict)
    inodes = set()
    for parent, dirs, files in os.walk(root, followlinks=False):
        dirs[:] = [d for d in dirs if not (Path(parent) / d).is_symlink()]
        for name in files:
            path = Path(parent) / name
            if path.is_symlink():
                continue
            st = path.stat()
            inode = (st.st_dev, st.st_ino)
            if inode in inodes:
                continue  # Cargo's deps/final hardlinks occupy storage only once.
            inodes.add(inode)
            ext = path.suffix
            totals[ext] += st.st_blocks * 512
            if ext in (".a", ".rlib", ".rmeta"):
                digest = hashlib.sha256()
                with path.open("rb") as stream:
                    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                        digest.update(chunk)
                groups[ext].setdefault(digest.hexdigest(), []).append(st.st_size)
    return {
        "root": str(root.resolve()),
        "allocated_bytes": sum(totals.values()),
        "allocated_by_extension": dict(totals),
        "duplicates": {
            ext: {
                "total_bytes": sum(sum(g) for g in hashes.values()),
                "duplicate_bytes": sum(sum(g[1:]) for g in hashes.values()),
                "files": sum(map(len, hashes.values())),
                "distinct": len(hashes),
            }
            for ext, hashes in groups.items()
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--arch", required=True, choices=("i386", "amd64", "arm64"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1] / f"build-{args.arch}"
    if not root.is_dir() or root.is_symlink():
        parser.error(f"missing or symlinked local build directory: {root}")
    print(json.dumps(measure(root), indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
