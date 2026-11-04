"""Compatibility resource scope for the native QEMU launch-handshake lock."""

from __future__ import annotations

import math
import os
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

from tools.test_infra_compat import DedicatedSession, NativeError


@contextmanager
def launch_lock(*, timeout: float = 120.0) -> Iterator[None]:
    """Bind one native lock resource; native owner death releases the flock."""
    # JSON has no NaN/Infinity representation: reject unencodable DTOs here as
    # well as in the native resource API. No filesystem/locking policy is Python.
    if not math.isfinite(timeout) or timeout < 0:
        raise ValueError("QEMU port handshake timeout must be finite and nonnegative")
    with DedicatedSession() as session:
        try:
            lease = session.call(
                "portlease_acquire",
                {
                    "timeout": timeout,
                    "owner_pid": os.getpid(),
                    "cwd": str(Path.cwd()),
                    "env": dict(os.environ),
                },
            )
        except NativeError as exc:
            if exc.kind == "TimeoutError":
                raise TimeoutError(str(exc)) from exc
            if exc.kind == "ValueError":
                raise ValueError(str(exc)) from exc
            raise
        try:
            yield
        finally:
            session.call("portlease_release", {"id": lease["id"]})
