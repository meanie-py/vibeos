"""Immutable raw purchases and append-only validation, independent of resume policy."""

from __future__ import annotations

import hashlib
import json
import os
import time
import uuid
from dataclasses import asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .config import RUSTCONV_RESPONSES_DIR, cost_usd
from .sink import ResponseSink


def digest(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


# Snapshot before any long API request; later disk edits cannot relabel this process.
CHECKER_FINGERPRINT = digest(
    "\n".join(
        f"{p.name}:{hashlib.sha256(p.read_bytes()).hexdigest()}"
        for p in sorted(Path(__file__).parent.glob("*.py"))
    )
)


def durable_json(path: Path, value: Any) -> None:
    """Exclusive creation: no attempt or validation can replace purchased evidence."""
    with path.open("x", encoding="utf-8") as stream:
        stream.write(json.dumps(value, indent=2, sort_keys=True) + "\n")
        stream.flush()
        os.fsync(stream.fileno())


class Attempt:
    def __init__(self, request: dict, root: Path | None = None, downstream=None):
        self.id = uuid.uuid4().hex
        self.path = (root or RUSTCONV_RESPONSES_DIR / "attempts") / self.id
        self.path.mkdir(parents=True, exist_ok=False)
        self.started = time.monotonic()
        self.downstream = downstream
        self.closed = False
        self.reasoning: ResponseSink | None = None
        durable_json(
            self.path / "request.json",
            {
                "schema": 1,
                "attempt_id": self.id,
                "created_utc": datetime.now(timezone.utc).isoformat(),
                "request_sha256": digest(json.dumps(request, sort_keys=True)),
                **request,
            },
        )
        self.sink = ResponseSink(self.path / "response.partial")

    def write(self, text: str) -> None:
        # Write the authoritative copy before any legacy sink can fail.
        self.sink.write(text)
        if self.downstream is not None:
            self.downstream.write(text)

    def write_reasoning(self, text: str) -> None:
        if self.reasoning is None:
            self.reasoning = ResponseSink(self.path / "reasoning.partial")
        self.reasoning.write(text)

    def finish(self, result) -> None:
        if self.reasoning is not None:
            self.reasoning.close()
        self.sink.close()
        self.closed = True
        with (self.path / "candidate.rs").open("x", encoding="utf-8") as stream:
            stream.write(result.rust_code)
            stream.flush()
            os.fsync(stream.fileno())
        durable_json(
            self.path / "result.json",
            {
                "schema": 1,
                "attempt_id": self.id,
                "elapsed_ms": round((time.monotonic() - self.started) * 1000),
                "candidate_sha256": digest(result.rust_code),
                "cost_usd": (
                    cost_usd(
                        result.model,
                        result.input_tokens,
                        result.output_tokens,
                        result.cache_read_tokens,
                        cache_write_tokens=getattr(result, "cache_write_tokens", 0),
                    )
                    if result.usage_known
                    else None
                ),
                **asdict(result),
            },
        )

    def close(self) -> None:
        if not self.closed:
            if self.reasoning is not None:
                self.reasoning.close()
            self.sink.close()
            self.closed = True


def record_validation(attempt_id: str | None, *, checker: str, verdict: str, **fields) -> None:
    if not attempt_id:
        return
    path = RUSTCONV_RESPONSES_DIR / "attempts" / attempt_id
    if not path.is_dir():
        raise ValueError(f"Missing immutable attempt {attempt_id}; cannot attach validation")
    durable_json(
        path / f"validation-{uuid.uuid4().hex}.json",
        {
            "schema": 1,
            "attempt_id": attempt_id,
            "checker": checker,
            "verdict": verdict,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            **fields,
        },
    )
