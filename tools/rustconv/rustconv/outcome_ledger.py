"""Append-only evidence ledger for rustconv routing and cost decisions.

The response cache answers "can this exact unit resume for free?".  This ledger
answers a different question: "what happened to every attempted unit, including
the failures?"  Paid audit rejects are especially valuable training data and
must not disappear merely because they never reached ResponseCache.put().
"""

from __future__ import annotations

import fcntl
import hashlib
import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .config import RUSTCONV_RESPONSES_DIR

SCHEMA_VERSION = 1
LEDGER_PATH = Path(
    os.environ.get(
        "RUSTCONV_OUTCOME_LEDGER",
        str(RUSTCONV_RESPONSES_DIR / "outcomes.jsonl"),
    )
)

_COMMENT_OR_LITERAL = re.compile(
    r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'',
    re.S,
)
_IDENTIFIER = re.compile(r"\b[A-Za-z_]\w*\b")
_CALL = re.compile(r"\b[A-Za-z_]\w*\s*\(")


def _sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", "replace")).hexdigest()


def lexical_fingerprint(source: str) -> str:
    """Hash code with comments and insignificant whitespace removed."""
    parts: list[str] = []
    cursor = 0
    for match in _COMMENT_OR_LITERAL.finditer(source):
        parts.append(re.sub(r"\s+", "", source[cursor : match.start()]))
        token = match.group(0)
        if not token.startswith("/"):
            parts.append(token)  # literal bytes, including internal whitespace
        cursor = match.end()
    parts.append(re.sub(r"\s+", "", source[cursor:]))
    return _sha256("".join(parts))


def source_features(source: str) -> dict[str, int]:
    """Cheap, stable routing features; deliberately no semantic claims."""
    code = _COMMENT_OR_LITERAL.sub(" ", source)
    lines = [line for line in source.splitlines() if line.strip()]
    identifiers = _IDENTIFIER.findall(code)
    uppercase = [name for name in identifiers if re.fullmatch(r"[A-Z][A-Z0-9_]+", name)]
    return {
        "bytes": len(source.encode("utf-8", "replace")),
        "nonblank_lines": len(lines),
        "calls": sum(
            match.group(0).split("(", 1)[0].strip()
            not in {"if", "for", "while", "switch", "sizeof"}
            for match in _CALL.finditer(code)
        ),
        "branches": len(re.findall(r"\b(?:if|switch|case)\b", code)),
        "loops": len(re.findall(r"\b(?:for|while|do)\b", code)),
        "gotos": len(re.findall(r"\bgoto\b", code)),
        "returns": len(re.findall(r"\breturn\b", code)),
        "pointer_arrows": code.count("->"),
        "uppercase_tokens": len(uppercase),
    }


def unit_identity(source: str) -> dict[str, Any]:
    """Content identity and router features shared by every outcome event."""
    return {
        "source_sha256": _sha256(source),
        "lexical_sha256": lexical_fingerprint(source),
        "features": source_features(source),
    }


class OutcomeLedger:
    """Concurrency-safe JSONL writer.

    One OS-level append happens while holding an advisory lock, so concurrent
    DeepSeek workers/processes cannot interleave JSON fragments.  Every record is
    fsync'd: losing the ledger on a crash would selectively erase failures while
    the streamed paid output survived, defeating its purpose.
    """

    def __init__(self, run_id: str, base: dict[str, Any], path: Path | None = None):
        self.run_id = run_id
        self.base = dict(base)
        self.path = path or LEDGER_PATH

    def record(self, *, unit: str, source: str, outcome: str, **fields: Any) -> None:
        event = {
            "schema": SCHEMA_VERSION,
            "timestamp": datetime.now(timezone.utc).isoformat(timespec="milliseconds"),
            "run_id": self.run_id,
            **self.base,
            "unit": unit,
            **unit_identity(source),
            "outcome": outcome,
            **fields,
        }
        self.path.parent.mkdir(parents=True, exist_ok=True)
        payload = (json.dumps(event, sort_keys=True, separators=(",", ":")) + "\n").encode()
        fd = os.open(self.path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
        try:
            fcntl.flock(fd, fcntl.LOCK_EX)
            os.write(fd, payload)
            os.fsync(fd)
        finally:
            fcntl.flock(fd, fcntl.LOCK_UN)
            os.close(fd)
