"""Registrar — provision a swarm identity across Authentik + Mattermost + Vikunja.

Privileged (uses the *_REGISTRAR_TOKEN credentials); run by a human lead, not an agent.
Every step is idempotent (find-then-create) so a half-provisioned agent re-runs cleanly.
`teardown_agent` exists for tests and de-provisioning.
"""

from __future__ import annotations

from typing import Any

from .authentik import AuthentikClient
from .config import Config
from .config import config as default_config
from .errors import HttpError
from .mattermost import MattermostClient
from .vikunja import VikunjaClient


class Registrar:
    def __init__(self, cfg: Config | None = None):
        self.cfg = cfg or default_config()
        self.authentik = AuthentikClient.from_config(self.cfg)
        self.mm = MattermostClient.from_config(self.cfg)
        self.vikunja = VikunjaClient.from_config(self.cfg)

    # --- swarm (per human lead) -----------------------------------------
    def register_swarm(self, lead: str) -> dict:
        """Ensure a subgroup `<lead>` under the vibeos-swarm group exists."""
        parent = self.authentik.find_group(self.cfg.authentik_group)
        if not parent:
            raise HttpError(404, "authentik", f"group {self.cfg.authentik_group} not found")
        sub = self.authentik.ensure_group(
            f"swarm-{lead}", parent=parent["pk"], attributes={"swarm_lead": lead}
        )
        return {"lead": lead, "group_pk": sub["pk"], "parent_pk": parent["pk"]}

    # --- agent ----------------------------------------------------------
    def register_agent(self, name: str, lead: str, *, charter: str | None = None) -> dict:
        bundle: dict[str, Any] = {"identity": name, "lead": lead}

        # 1. Authentik service account (M2M)
        existing = self.authentik.find_user(name)
        if existing:
            bundle["authentik"] = {
                "user_pk": existing["pk"],
                "username": existing["username"],
                "created": False,
            }
        else:
            sa = self.authentik.create_service_account(name)
            bundle["authentik"] = {
                "user_pk": sa.get("user_pk"),
                "username": sa.get("username"),
                "token": sa.get("token"),
                "created": True,
            }
        # group + owner attribute
        grp = self.authentik.find_group(f"swarm-{lead}") or self.authentik.find_group(
            self.cfg.authentik_group
        )
        upk = bundle["authentik"]["user_pk"]
        if grp and upk:
            try:
                self.authentik.add_user_to_group(grp["pk"], upk)
            except HttpError:
                pass
            try:
                self.authentik.set_user_attributes(upk, {"owner": lead, "swarm_agent": True})
            except HttpError:
                pass

        # 2. Mattermost REGULAR user (not a bot → no "BOT" badge) + personal-access token,
        #    added to the team, #swarm, and its own channel; posts use its own token.
        import time as _time

        u = self.mm.find_user(name)
        if u:
            uid = u["id"]
            bundle["mattermost"] = {"user_id": uid, "created": False}
        else:
            created = self.mm.create_user(
                name, f"{name}@swarm.local", f"Sw@rm-{name}-{int(_time.time())}"
            )
            uid = created["id"]
            bundle["mattermost"] = {"user_id": uid, "created": True}
        agent_token = None
        try:
            tok = self.mm.create_user_token(uid)
            agent_token = tok.get("token")
            bundle["mattermost"]["token"] = agent_token
        except HttpError:
            pass
        team_id = self.mm.team_id()
        try:
            self.mm.add_to_team(team_id, uid)
        except HttpError:
            pass
        try:
            self.mm.add_to_channel(self.cfg.mm_swarm_channel_id, uid)  # so it can post to #swarm
        except HttpError:
            pass
        ch_name = f"agent-{name}"
        ch_id = self.mm.find_channel(ch_name)
        if not ch_id:
            ch = self.mm.create_channel(team_id, ch_name, f"{name} inbox")
            ch_id = ch["id"]
            bundle["mattermost"]["channel_created"] = True
        bundle["mattermost"]["channel_id"] = ch_id
        try:
            self.mm.add_to_channel(ch_id, uid)
        except HttpError:
            pass
        # persist runtime creds locally (interim; OAuth-derived later)
        if agent_token:
            from . import identity

            identity.save_agent_creds(
                name, {"identity": name, "mm_user_id": uid, "mm_token": agent_token}
            )
        # pinned banner (charter) — post as the agent itself (it's a member of its own
        # channel; the shared registrar/bot is not, so its post would 403).
        banner = charter or f"# {name}\n\n(Charter not set — edit this pinned banner.)"
        agent_token = bundle["mattermost"].get("token")
        try:
            post = self.mm.post(ch_id, f"📌 **Charter — {name}**\n\n{banner}", token=agent_token)
            if post and post.get("id"):
                self.mm.pin_post(post["id"])
                bundle["mattermost"]["banner_post_id"] = post["id"]
        except HttpError:
            pass

        # 3. Vikunja — interim: shared project; per-agent user is a follow-up
        bundle["vikunja"] = {"note": "shared board (per-agent user pending)"}
        return bundle

    # --- project (epic) -------------------------------------------------
    def create_project(self, title: str, *, description="") -> dict:
        proj = self.vikunja.create_project(title, description=description)
        out = {"project_id": proj.get("id"), "title": proj.get("title")}
        # per-project Mattermost channel (bridge bot fills it)
        team_id = self.mm.team_id()
        slug = "proj-" + "".join(ch if ch.isalnum() else "-" for ch in title.lower())[:40].strip(
            "-"
        )
        ch_id = self.mm.find_channel(slug)
        if not ch_id:
            ch = self.mm.create_channel(team_id, slug, f"{title} (project)")
            ch_id = ch["id"]
        out["channel_id"] = ch_id
        return out

    # --- teardown (tests / de-provision) --------------------------------
    def teardown_agent(self, name: str) -> dict:
        result = {"identity": name}
        u = self.authentik.find_user(name)
        if u:
            try:
                self.authentik.delete_user(u["pk"])
                result["authentik"] = "deleted"
            except HttpError as e:
                result["authentik"] = f"err {e.status}"
        mu = self.mm.find_user(name)
        if mu:
            try:
                self.mm.delete_bot_user(mu["id"])  # DELETE /users/{id} deactivates
                result["mm_user"] = "deactivated"
            except HttpError as e:
                result["mm_user"] = f"err {e.status}"
        from . import identity

        try:
            (identity._agents_dir() / f"{name}.json").unlink()
        except OSError:
            pass
        ch_id = self.mm.find_channel(f"agent-{name}")
        if ch_id:
            try:
                self.mm.delete_channel(ch_id)
                result["mm_channel"] = "deleted"
            except HttpError as e:
                result["mm_channel"] = f"err {e.status}"
        return result
