"""Persistent memoization for pure source facts, never whole-workspace verdicts.

Keys include the actual input bytes, scanner source, explicit helper dependencies,
Python and parser versions. No mtime shortcuts and no pickle execution. Each scanner
owns a worktree-local SQLite database; WAL makes concurrent checks independent.
Changing/removing a file changes the facts fed to the *current* global comparison.
Set VIBEOS_LINT_CACHE=0 for the uncached oracle. Cache failures are loud.
"""

from __future__ import annotations

import hashlib
import marshal
import os
import sqlite3
import sys
from functools import wraps
from importlib.metadata import version
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def source_facts(*dependencies: str):
    """Decorate a pure function of marshal-compatible values (no paths or ASTs)."""

    def decorate(fn):
        connection = None
        namespace = None

        @wraps(fn)
        def cached(*args, **kwargs):
            nonlocal connection, namespace
            if os.environ.get("VIBEOS_LINT_CACHE") == "0":
                return fn(*args, **kwargs)
            if connection is None:
                digest = hashlib.sha256()
                for path in [
                    Path(__file__),
                    Path(fn.__code__.co_filename),
                    *(ROOT / p for p in dependencies),
                ]:
                    digest.update(path.read_bytes())
                digest.update(sys.version.encode())
                for package in ("tree-sitter", "tree-sitter-rust"):
                    digest.update(version(package).encode())
                namespace = digest.digest() + fn.__qualname__.encode()
                directory = ROOT / ".cache" / "vibeos-lint"
                directory.mkdir(parents=True, exist_ok=True)
                connection = sqlite3.connect(
                    directory / (Path(fn.__code__.co_filename).stem + ".sqlite3"),
                    timeout=60,
                    isolation_level=None,
                )
                connection.execute("PRAGMA journal_mode=WAL")
                connection.execute("PRAGMA synchronous=NORMAL")
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS facts (key BLOB PRIMARY KEY, value BLOB NOT NULL)"
                )
            key = hashlib.sha256(namespace + marshal.dumps((args, sorted(kwargs.items())))).digest()
            row = connection.execute("SELECT value FROM facts WHERE key=?", (key,)).fetchone()
            if row is not None:
                return marshal.loads(row[0])
            result = fn(*args, **kwargs)
            connection.execute(
                "INSERT OR REPLACE INTO facts VALUES (?, ?)", (key, marshal.dumps(result))
            )
            return result

        return cached

    return decorate
