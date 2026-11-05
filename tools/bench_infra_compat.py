"""Typed JSON transport and resource lifetimes for native physical bench mechanisms."""

from __future__ import annotations

import atexit
import json
import os
import threading
from pathlib import Path
from typing import Any

from tools import build_infra_compat as transport

REPO = Path(__file__).resolve().parent.parent
NativeError = transport.NativeError


def _source_signature():
    paths = [REPO / "tools/bench-infra", Path(__file__), Path(transport.__file__)]
    directories = []
    for name in ("vibeos-bench",):
        crate = REPO / "tools" / name
        paths.extend((crate / "Cargo.toml", crate / "Cargo.lock"))
        directories.append(crate / "src")
    paths.extend((REPO / "rust-toolchain.toml", REPO / "tools/host-jobs/src/build_policy.rs"))
    return transport._signature_records(paths, directories)


class _Session(transport._ArtifactSession):
    def tool_path(self):
        return REPO / "tools/bench-infra"

    def source_signature(self):
        return _source_signature()


class DedicatedSession(transport.DedicatedSession):
    def __init__(self, root=REPO):
        super().__init__("--resource-jsonl", root, session_factory=_Session)


_sessions: dict[tuple[int, str], _Session] = {}
_lock = threading.RLock()


def _close():
    for session in list(_sessions.values()):
        session.close()
    _sessions.clear()


atexit.register(_close)


def _after_fork():
    global _lock
    _close()
    _lock = threading.RLock()


os.register_at_fork(after_in_child=_after_fork)


def call(op: str, kwargs: dict, root: Path = REPO) -> Any:
    with _lock:
        key = (os.getpid(), str(root))
        session = _sessions.get(key)
        if session and (
            session.signature != _source_signature() or session.environment != dict(os.environ)
        ):
            session.close()
            del _sessions[key]
            session = None
        if session is None:
            # Fixture repositories and board roots must not retain one process each.
            while len(_sessions) >= 8:
                oldest = next(iter(_sessions))
                _sessions.pop(oldest).close()
            session = _Session(root)
            _sessions[key] = session
        try:
            response = json.loads(
                session.request(
                    json.dumps({"op": op, "kwargs": transport._json(kwargs)}, ensure_ascii=False)
                )
            )
            if not isinstance(response, dict) or set(response) not in ({"ok"}, {"error"}):
                raise NativeError("invalid native bench response envelope")
            if "error" in response:
                error = response["error"]
                if not isinstance(error, dict) or not isinstance(error.get("message"), str):
                    raise NativeError("invalid native bench error envelope")
                raise NativeError(error["message"], error.get("kind"))
            return response["ok"]
        except BaseException:
            _sessions.pop(key, None)
            session.close()
            raise
