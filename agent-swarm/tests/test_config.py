"""Config: canonical keys with legacy aliases, and process-env precedence."""

from agent_swarm.config import Config


def _cfg(tmp_path, monkeypatch, **env):
    # isolate: no real config.env, no real repo .env
    monkeypatch.setenv("VIBEOS_SWARM_CONFIG", str(tmp_path / "none.env"))
    for k in ("MM_API_URL", "MATTERMOST_API", "VIKUNJA_API_URL", "AUTHENTIK_API_URL"):
        monkeypatch.delenv(k, raising=False)
    for k, v in env.items():
        monkeypatch.setenv(k, v)
    return Config(repo_root=tmp_path)


def test_canonical_key(tmp_path, monkeypatch):
    c = _cfg(tmp_path, monkeypatch, MM_API_URL="https://x/api/v4")
    assert c.mm_api == "https://x/api/v4"


def test_legacy_alias_fallback(tmp_path, monkeypatch):
    c = _cfg(tmp_path, monkeypatch, MATTERMOST_API="https://legacy/api/v4")
    assert c.mm_api == "https://legacy/api/v4"


def test_canonical_wins_over_legacy(tmp_path, monkeypatch):
    c = _cfg(
        tmp_path,
        monkeypatch,
        MM_API_URL="https://canon/api/v4",
        MATTERMOST_API="https://legacy/api/v4",
    )
    assert c.mm_api == "https://canon/api/v4"


def test_defaults(tmp_path, monkeypatch):
    c = _cfg(tmp_path, monkeypatch)
    assert c.mm_team == "vibeos"
    assert c.vikunja_project_id == "2"
    assert c.mm_api is None  # nothing set → None (caller decides)
