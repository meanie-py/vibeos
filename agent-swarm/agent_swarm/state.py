"""Busy/idle state machine for an agent session, with PID-liveness.

Claude Code exposes no native busy/idle signal, so hooks bracket it:
  UserPromptSubmit → busy ;  Stop → idle ;  Notification:idle_prompt confirms idle.
The delivery layer reads this to choose its drain: busy → hook additionalContext;
idle → MCP channel push.

PID-liveness: each state record carries the owning process id. If that process is dead
when we evaluate, we auto-flip to idle (and the roster treats it offline) — a crashed or
closed session never gets stuck 'busy'.
"""
from __future__ import annotations

import json
import os
import time
from pathlib import Path

from .config import config


def _state_path(identity: str) -> Path:
    return config().state_dir / f"{identity}.json"


def _pid_alive(pid) -> bool:
    try:
        os.kill(int(pid), 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True  # exists, owned by someone else
    except (OSError, ValueError, TypeError):
        return False
    return True


def set_state(identity: str, state: str, *, pid: int | None = None) -> None:
    d = config().state_dir
    d.mkdir(parents=True, exist_ok=True)
    (_state_path(identity)).write_text(json.dumps({
        "state": state, "pid": pid or os.getpid(), "ts": time.time(),
    }))


def get_state(identity: str) -> dict:
    p = _state_path(identity)
    if not p.exists():
        return {"state": "unknown", "pid": None, "ts": 0, "alive": False}
    try:
        d = json.loads(p.read_text())
    except (ValueError, OSError):
        return {"state": "unknown", "pid": None, "ts": 0, "alive": False}
    alive = _pid_alive(d.get("pid"))
    d["alive"] = alive
    if not alive:
        d["state"] = "idle"  # dead process → idle, regardless of last write
    return d


def is_busy(identity: str) -> bool:
    return get_state(identity)["state"] == "busy"


def clear(identity: str) -> None:
    try:
        _state_path(identity).unlink()
    except OSError:
        pass
