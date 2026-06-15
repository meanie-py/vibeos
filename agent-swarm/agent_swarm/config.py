"""Configuration loading and accessors.

Sources, lowest → highest precedence:
  1. ~/.vibeos-swarm/config.env   (interim/legacy MATTERMOST_* keys)
  2. <repo>/.env                  (canonical MM_* / VIKUNJA_* / AUTHENTIK_* keys)
  3. process environment          (launch-time + login-injected creds win)

Hooks don't reliably inherit a shell .env, so we read the files ourselves. Accessors
expose canonical names and fall back to legacy aliases, so callers never branch on which
key a value came from.
"""

from __future__ import annotations

import os
from pathlib import Path

from .errors import ConfigError


def _parse_env_file(path: Path) -> dict:
    out = {}
    if not path.exists():
        return out
    try:
        for raw in path.read_text().splitlines():
            line = raw.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            k, v = line.split("=", 1)
            out[k.strip()] = v.strip().strip('"').strip("'")
    except OSError:
        pass
    return out


def _find_repo_root(start: Path) -> Path | None:
    for p in [start, *start.parents]:
        if (p / ".git").exists() or (p / "AGENTS.md").exists():
            return p
    return None


class Config:
    """Merged configuration with typed, alias-aware accessors."""

    def __init__(self, repo_root: Path | None = None):
        self._d: dict = {}
        # 1. legacy interim file
        legacy = os.environ.get("VIBEOS_SWARM_CONFIG")
        legacy_path = Path(legacy) if legacy else Path.home() / ".vibeos-swarm" / "config.env"
        self._d.update(_parse_env_file(legacy_path))
        # 2. repo .env (canonical)
        root = repo_root or _find_repo_root(Path(__file__).resolve()) or _find_repo_root(Path.cwd())
        self.repo_root = root
        if root:
            self._d.update(_parse_env_file(root / ".env"))
        # 3. process env wins
        self._d.update({k: v for k, v in os.environ.items()})

    def get(self, *keys: str, default=None):
        """First present value among aliases."""
        for k in keys:
            if self._d.get(k):
                return self._d[k]
        return default

    def require(self, *keys: str):
        v = self.get(*keys)
        if not v:
            raise ConfigError(f"missing config: one of {keys}")
        return v

    # --- Mattermost ---
    @property
    def mm_api(self):
        return self.get("MM_API_URL", "MATTERMOST_API")

    @property
    def mm_base(self):
        return self.get("MM_BASE_URL", "MATTERMOST_BASE_URL")

    @property
    def mm_bot_token(self):
        return self.get("MM_BOT_TOKEN", "MATTERMOST_BOT_TOKEN")

    @property
    def mm_registrar_token(self):
        return self.get("MM_REGISTRAR_TOKEN")

    @property
    def mm_webhook_url(self):
        return self.get("MM_WEBHOOK_URL", "MATTERMOST_WEBHOOK_URL")

    @property
    def mm_webhook_basic(self):
        u = self.get("MM_WEBHOOK_USER")
        p = self.get("MM_WEBHOOK_PASSWORD")
        if u and p:
            return f"{u}:{p}"
        return self.get("MATTERMOST_WEBHOOK_BASIC")

    @property
    def mm_team(self):
        return self.get("MM_TEAM", default="vibeos")

    @property
    def mm_swarm_channel_id(self):
        return self.get("MM_SWARM_CHANNEL_ID", "MATTERMOST_CHANNEL_SWARM")

    @property
    def mm_presence_channel_id(self):
        return self.get("MM_PRESENCE_CHANNEL_ID", "MATTERMOST_CHANNEL_PRESENCE")

    @property
    def mm_icon_emoji(self):
        return self.get("MATTERMOST_ICON_EMOJI", default=":robot_face:")

    # --- Vikunja ---
    @property
    def vikunja_api(self):
        return self.get("VIKUNJA_API_URL")

    @property
    def vikunja_token(self):
        return self.get("VIKUNJA_TOKEN")

    @property
    def vikunja_project_id(self):
        return self.get("VIKUNJA_SWARM_PROJECT_ID", default="2")

    # --- Authentik ---
    @property
    def authentik_api(self):
        return self.get("AUTHENTIK_API_URL")

    @property
    def authentik_registrar_token(self):
        return self.get("AUTHENTIK_REGISTRAR_TOKEN")

    @property
    def authentik_group(self):
        return self.get("AUTHENTIK_SWARM_GROUP", default="vibeos-swarm")

    # --- local paths ---
    @property
    def db_path(self):
        # New platform DB (the legacy tools/swarm registry.db has an incompatible schema).
        p = self.get("VIBEOS_SWARM_DB")
        return Path(p) if p else Path.home() / ".vibeos-swarm" / "swarm.db"

    @property
    def state_dir(self):
        return Path.home() / ".vibeos-swarm" / "state"


_default: Config | None = None


def config() -> Config:
    """Process-wide default Config (lazy)."""
    global _default
    if _default is None:
        _default = Config()
    return _default
