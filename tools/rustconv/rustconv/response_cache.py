"""Durable per-function translation cache → resume an interrupted run for free.

`translate` makes one PAID API call per function. Before this, a run killed at
function 35/40 (credit exhaustion, Ctrl-C, crash, reboot) lost all 35 completed
functions — the next invocation re-translated and re-PAID for every one. This
module persists each function's Rust the instant it returns, and lets the next
run skip every function whose C body is unchanged (a `$0` cache hit).

Layout (gitignored, in-repo so it survives kills/reboots):

    tools/rustconv/.responses/<subsystem>__<stem>/
        manifest.json          # {func_name: {file, hash}}  — hash = of the C body
        <func>.rs              # the translated Rust for that function
        __preamble__.rs        # the file-scope preamble unit (if any)

Correctness guard: a cached entry is reused ONLY if the sha256 of the current C
function body matches the hash stored when it was translated. Edit the C (or the
split chunks it inlines) and that function re-translates automatically; the rest
still resume. `--no-resume` ignores the cache entirely (force fresh).
"""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Optional

# tools/rustconv/.responses/  (parent.parent = the rustconv package's parent dir)
RESPONSES_DIR = Path(__file__).resolve().parent.parent / ".responses"

PREAMBLE_KEY = "__preamble__"


def _hash(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", "replace")).hexdigest()[:16]


def _safe_filename(name: str) -> str:
    """C identifiers are filename-safe, but be defensive about odd symbols."""
    return re.sub(r"[^A-Za-z0-9_.@-]", "_", name)


class ResponseCache:
    """One directory per translated file; one .rs per function + a manifest."""

    def __init__(self, key: str):
        self.dir = RESPONSES_DIR / _safe_filename(key)
        self.manifest_path = self.dir / "manifest.json"
        self._manifest: dict[str, dict] = {}
        if self.manifest_path.exists():
            try:
                self._manifest = json.loads(self.manifest_path.read_text())
            except (OSError, ValueError):
                self._manifest = {}

    def get(self, name: str, body_text: str) -> Optional[str]:
        """Cached Rust for ``name`` IFF the C body hash still matches, else None.

        An EMPTY (or whitespace-only) cached file is treated as a MISS so that a
        previously-poisoned entry (an empty/truncated translation that an older
        pipeline wrongly cached as success) self-heals: the next run re-translates
        it instead of silently reusing nothing. See `put`.
        """
        entry = self._manifest.get(name)
        if not entry or entry.get("hash") != _hash(body_text):
            return None
        f = self.dir / entry.get("file", "")
        if not f.exists():
            return None
        try:
            text = f.read_text()
        except OSError:
            return None
        if not text.strip():
            return None
        return text

    def put(self, name: str, body_text: str, rust: str) -> None:
        """Persist one function's Rust + record its body hash. Atomic-ish.

        Refuses to cache EMPTY/whitespace-only output: an empty translation is a
        failure, and caching it as a valid entry would make resume skip the unit
        forever (silently dropping it from the assembled file). Callers should
        only reach here on success, but this is the durable last line of defence.
        """
        if not rust.strip():
            raise ValueError(
                f"refusing to cache empty translation for {name!r} "
                f"(empty output is a failure, not a result)"
            )
        self.dir.mkdir(parents=True, exist_ok=True)
        fname = _safe_filename(name) + ".rs"
        (self.dir / fname).write_text(rust)
        self._manifest[name] = {"file": fname, "hash": _hash(body_text)}
        tmp = self.manifest_path.with_suffix(".json.tmp")
        tmp.write_text(json.dumps(self._manifest, indent=1, sort_keys=True))
        tmp.replace(self.manifest_path)

    def count(self) -> int:
        return len(self._manifest)
