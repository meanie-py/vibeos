"""Vikunja → Mattermost ingestion bridge.

Vikunja can't webhook to private IPs (SSRF guard), so we poll. This bridge turns
*agent-relevant* board activity — a quest assigned to an agent, a new comment on a quest
it owns, a status/priority change — into a Mattermost post **in that agent's channel**,
`@`-mentioning the agent. From there the normal agent layer takes over: the agent's
`drain`/channel surfaces it (the `@` makes it high-priority → it wakes), and the human
sees it too.

Routing — who owns a task:
  - **assignees** whose Vikunja username is a provisioned agent (the real path; needs
    per-agent Vikunja users, pending server-side `vikunja user create`), and
  - **labels** of the form `agent:<name>` or a bare `<name>` matching an agent (the
    testable-now interim until per-agent users land).

State (cursor + per-task last-comment marker) lives in ~/.vibeos-swarm/bridge.json.
"""

from __future__ import annotations

import json
from pathlib import Path

from .config import Config
from .config import config as default_config
from .mattermost import MattermostClient
from .send import send
from .types import VikunjaPriority
from .vikunja import VikunjaClient


class VikunjaBridge:
    def __init__(self, cfg: Config | None = None, *, post_as: str = "board"):
        self.cfg = cfg or default_config()
        self.vik = VikunjaClient.from_config(self.cfg)
        self.mm = MattermostClient.from_config(self.cfg)
        self.post_as = post_as
        self.state_file = Path.home() / ".vibeos-swarm" / "bridge.json"
        self._agent_cache: dict[str, bool] = {}

    # --- state ----------------------------------------------------------
    def _load(self) -> dict:
        if self.state_file.exists():
            try:
                return json.loads(self.state_file.read_text())
            except (ValueError, OSError):
                pass
        return {"tasks": {}}  # tid -> {updated, agents:[...], comment_max}

    def _save(self, st: dict) -> None:
        self.state_file.parent.mkdir(parents=True, exist_ok=True)
        self.state_file.write_text(json.dumps(st))

    # --- routing --------------------------------------------------------
    def _is_agent(self, name: str | None) -> bool:
        if not name:
            return False
        if name not in self._agent_cache:
            self._agent_cache[name] = bool(self.mm.find_channel(f"agent-{name}"))
        return self._agent_cache[name]

    def _agents_for(self, task: dict) -> set[str]:
        out: set[str] = set()
        for a in task.get("assignees") or []:
            if self._is_agent(a.get("username")):
                out.add(a["username"])
        for lab in task.get("labels") or []:
            title = (lab.get("title") or "").strip()
            name = title.split(":", 1)[1].strip() if title.lower().startswith("agent:") else title
            if self._is_agent(name):
                out.add(name)
        return out

    def _post(self, agent: str, text: str) -> None:
        send(text, channel=f"agent-{agent}", as_identity=self.post_as)

    # --- sync -----------------------------------------------------------
    def sync(self, *, verbose: bool = False) -> dict:
        st = self._load()
        prev = st["tasks"]
        posted = 0

        tasks = self.vik.list_tasks(done=None)
        for t in tasks:
            tid = str(t["id"])
            now_agents = self._agents_for(t)
            pr = VikunjaPriority.parse(t.get("priority")).label
            status = "✅ done" if t.get("done") else "open"
            comments = self.vik.list_comments(t["id"]) or []
            comment_max = max([c.get("id", 0) for c in comments], default=0)

            p = prev.get(tid)
            if p is None:
                # first sight of this task — baseline only, never replay history
                prev[tid] = {
                    "updated": t.get("updated"),
                    "agents": sorted(now_agents),
                    "comment_max": comment_max,
                }
                continue

            prev_agents = set(p.get("agents") or [])

            # (a) newly assigned → "assigned to you"
            for ag in now_agents - prev_agents:
                self._post(
                    ag,
                    f"@{ag} 🆕 you've been assigned quest #{t['id']} — "
                    f"*{t['title']}* · priority {pr} · {status}",
                )
                posted += 1
            # (b) status/priority/title change on an already-owned quest
            if t.get("updated") != p.get("updated"):
                for ag in now_agents & prev_agents:
                    self._post(
                        ag,
                        f"@{ag} 📋 quest #{t['id']} updated — "
                        f"*{t['title']}* · priority {pr} · {status}",
                    )
                    posted += 1
            # (c) new comments on an owned quest (skip the commenter's own)
            last_max = p.get("comment_max", 0)
            for c in comments:
                if c.get("id", 0) > last_max:
                    author = ((c.get("author") or {}).get("username")) or "?"
                    body = (c.get("comment") or "").strip()
                    for ag in now_agents:
                        if author != ag:
                            self._post(
                                ag, f"@{ag} 💬 #{t['id']} *{t['title']}* — {author}: {body[:200]}"
                            )
                            posted += 1

            prev[tid] = {
                "updated": t.get("updated"),
                "agents": sorted(now_agents),
                "comment_max": comment_max,
            }

        self._save(st)
        if verbose:
            print(f"bridge: {len(tasks)} task(s) scanned, {posted} notification(s) posted")
        return {"tasks": len(tasks), "posted": posted}
