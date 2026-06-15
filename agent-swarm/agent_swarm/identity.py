"""Identity resolution and login.

Identity is first-class and persistent — an Authentik service account, *not* a session
id or a cwd. Resolution precedence:
  1. $SWARM_LOGIN                       (set when launching an agent — automation path)
  2. session file for $CLAUDE_CODE_SESSION_ID   (written by `/swarm-login` interactively)
  3. ~/.vibeos-swarm/identity           (single-agent default)

`login()` binds this context to an identity and (interim) records a creds bundle. The
full runtime path derives short-lived M2M creds from the human lead's OAuth via Authentik;
until that Application is live, the shared service tokens from Config are used and the
binding is just the name (posts use the per-message username override).
"""

from __future__ import annotations

import json
import os
from pathlib import Path


def _sessions_dir() -> Path:
    return Path.home() / ".vibeos-swarm" / "sessions"


def _default_identity_file() -> Path:
    return Path.home() / ".vibeos-swarm" / "identity"


def _session_id() -> str | None:
    return os.environ.get("CLAUDE_CODE_SESSION_ID")


def current_identity() -> str | None:
    env = os.environ.get("SWARM_LOGIN")
    if env:
        return env.strip()
    sid = _session_id()
    if sid:
        f = _sessions_dir() / f"{sid}.json"
        if f.exists():
            try:
                return json.loads(f.read_text()).get("identity")
            except (ValueError, OSError):
                pass
    f = _default_identity_file()
    if f.exists():
        try:
            return f.read_text().strip() or None
        except OSError:
            pass
    return None


def login(name: str, *, creds: dict | None = None, set_default: bool = True) -> None:
    """Bind this context (session, and optionally the machine default) to `name`."""
    sid = _session_id()
    if sid:
        d = _sessions_dir()
        d.mkdir(parents=True, exist_ok=True)
        (d / f"{sid}.json").write_text(json.dumps({"identity": name, "creds": creds or {}}))
    if set_default:
        f = _default_identity_file()
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text(name)


def logout() -> None:
    sid = _session_id()
    if sid:
        try:
            (_sessions_dir() / f"{sid}.json").unlink()
        except OSError:
            pass


def _agents_dir() -> Path:
    return Path.home() / ".vibeos-swarm" / "agents"


def save_agent_creds(name: str, creds: dict) -> None:
    """Persist an agent's runtime creds (e.g. its MM personal-access token) locally.

    Interim single-machine store written by the registrar; the cross-machine path derives
    these from the human lead's OAuth at login. Mode 600 — never committed.
    """
    d = _agents_dir()
    d.mkdir(parents=True, exist_ok=True)
    f = d / f"{name}.json"
    f.write_text(json.dumps(creds))
    try:
        os.chmod(f, 0o600)
    except OSError:
        pass


def load_agent_creds(name: str) -> dict:
    f = _agents_dir() / f"{name}.json"
    if f.exists():
        try:
            return json.loads(f.read_text())
        except (ValueError, OSError):
            pass
    return {}


def creds_for(name: str) -> dict:
    # session bundle first, then the local agent creds file
    sid = _session_id()
    if sid:
        f = _sessions_dir() / f"{sid}.json"
        if f.exists():
            try:
                d = json.loads(f.read_text())
                if d.get("identity") == name and d.get("creds"):
                    return d["creds"]
            except (ValueError, OSError):
                pass
    return load_agent_creds(name)
