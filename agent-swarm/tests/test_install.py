"""`swarm install` config rendering — the pure merge logic that wires a worktree
onto the swarm (hooks + per-worktree identity + the MCP channel), non-destructively."""

from agent_swarm.cli import _default_identity, render_mcp, render_settings


class TestRenderSettings:
    def test_adds_all_five_hooks(self):
        s = render_settings({}, "fastfat")
        assert set(s["hooks"]) == {
            "SessionStart",
            "UserPromptSubmit",
            "PostToolUse",
            "Stop",
            "SessionEnd",
        }

    def test_posttooluse_has_matcher(self):
        s = render_settings({}, "fastfat")
        assert s["hooks"]["PostToolUse"][0]["matcher"] == "*"
        # the non-matcher events must NOT carry a matcher key
        assert "matcher" not in s["hooks"]["SessionStart"][0]

    def test_hooks_use_project_dir_relative_path(self):
        cmd = render_settings({}, "x")["hooks"]["SessionStart"][0]["hooks"][0]["command"]
        assert "$CLAUDE_PROJECT_DIR/agent-swarm/hooks/session_start.sh" in cmd

    def test_binds_identity_via_env(self):
        assert render_settings({}, "netstack")["env"]["SWARM_LOGIN"] == "netstack"

    def test_non_destructive_merge(self):
        existing = {"permissions": {"allow": ["Bash"]}, "env": {"FOO": "bar"}}
        s = render_settings(existing, "win32k")
        assert s["permissions"] == {"allow": ["Bash"]}  # untouched
        assert s["env"]["FOO"] == "bar"  # preserved
        assert s["env"]["SWARM_LOGIN"] == "win32k"  # added

    def test_idempotent(self):
        once = render_settings({}, "x")
        twice = render_settings(once, "x")
        assert once == twice


class TestRenderMcp:
    def test_adds_channel_server_with_abs_path(self):
        m = render_mcp({}, "/repo")
        srv = m["mcpServers"]["mattermost-swarm"]
        assert srv["command"] == "node"
        assert srv["args"] == ["/repo/agent-swarm/channel/mattermost.mjs"]

    def test_preserves_other_servers(self):
        m = render_mcp({"mcpServers": {"other": {"command": "x"}}}, "/repo")
        assert "other" in m["mcpServers"]
        assert "mattermost-swarm" in m["mcpServers"]


class TestDefaultIdentity:
    def test_strips_lane_affixes(self, tmp_path):
        # no git repo at tmp_path → falls back to dir name, then affix-stripping.
        (tmp_path / "wt-fastfat-port").mkdir()
        assert _default_identity(tmp_path / "wt-fastfat-port") == "fastfat"
