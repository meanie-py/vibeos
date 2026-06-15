"""Test fixtures — isolate state (DB, HOME, config) and provide a stub Mattermost."""

from __future__ import annotations

import pytest

from agent_swarm.types import Post


@pytest.fixture
def env(tmp_path, monkeypatch):
    """Isolate all on-disk state under a temp dir, and bypass the real config.env."""
    monkeypatch.setenv("VIBEOS_SWARM_DB", str(tmp_path / "swarm.db"))
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("VIBEOS_SWARM_CONFIG", str(tmp_path / "none.env"))
    import agent_swarm.config as cfgmod

    cfgmod._default = None  # reset the lazy singleton
    yield tmp_path
    cfgmod._default = None


def post(pid, create_at, user, message, user_id=None) -> Post:
    return Post(id=pid, create_at=create_at, user=user, user_id=user_id or user, message=message)


class FakeMM:
    """A Mattermost stand-in returning a fixed post list (no network)."""

    def __init__(self, posts: list[Post] | None = None):
        self.posts = posts or []

    def latest_create_at(self, channel_id, **_):
        return 1  # non-zero baseline below the test posts (cursor seeds here, not "now")

    def read_posts(self, channel_id, since, **_):
        return [p for p in self.posts if p.create_at > since]
