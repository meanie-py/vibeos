"""Mattermost client — messaging, reads, and (registrar) identity provisioning.

Native Mattermost /api/v4 with bearer tokens; the incoming webhook (Slack-shaped, Traefik
basic-auth-gated) is the per-agent-avatar post path. Three token roles:
  - bot token       → reads + bot posts
  - registrar token → create bots/channels/memberships (system-admin)
  - webhook + basic  → cheap posts with a per-message username/avatar
"""

from __future__ import annotations

from . import http
from .errors import HttpError
from .types import Post


class MattermostClient:
    def __init__(
        self,
        *,
        api,
        bot_token=None,
        registrar_token=None,
        webhook_url=None,
        webhook_basic=None,
        team="vibeos",
        icon_emoji=None,
    ):
        self.api = (api or "").rstrip("/")
        self.bot_token = bot_token
        self.registrar_token = registrar_token
        self.webhook_url = webhook_url
        self.webhook_basic = webhook_basic
        self.team = team
        self.icon_emoji = icon_emoji
        self._uname_cache = {}  # user_id -> username (resolved lazily for REST posts)

    @classmethod
    def from_config(cls, cfg):
        return cls(
            api=cfg.mm_api,
            bot_token=cfg.mm_bot_token,
            registrar_token=cfg.mm_registrar_token,
            webhook_url=cfg.mm_webhook_url,
            webhook_basic=cfg.mm_webhook_basic,
            team=cfg.mm_team,
            icon_emoji=cfg.mm_icon_emoji,
        )

    # --- posting ---------------------------------------------------------
    def webhook_post(self, text, *, channel, username=None, icon_emoji=None):
        """Post via the incoming webhook (per-message username/avatar). Best-effort."""
        if not self.webhook_url:
            return False
        payload = {"text": text, "channel": channel}
        if username:
            payload["username"] = username
        emoji = icon_emoji or self.icon_emoji
        if emoji:
            payload["icon_emoji"] = emoji
        try:
            http.post(self.webhook_url, basic=self.webhook_basic, json_body=payload)
            return True
        except HttpError:
            return False

    def post(self, channel_id, message, *, root_id="", token=None):
        """Bot REST post (returns the created post; supports threading via root_id)."""
        return http.post(
            f"{self.api}/posts",
            token=token or self.bot_token,
            json_body={"channel_id": channel_id, "message": message, "root_id": root_id},
        )

    # --- reading ---------------------------------------------------------
    def read_posts(self, channel_id, since_ms, *, token=None) -> list[Post]:
        """Posts created after since_ms, oldest-first; skips system messages.

        Webhook posts carry override_username; real REST users (humans, and agents
        posting via their own token) don't — so we resolve those user_ids to @handles.
        """
        data = http.get(
            f"{self.api}/channels/{channel_id}/posts",
            token=token or self.bot_token,
            params={"since": since_ms},
        )
        if not data:
            return []
        raw: list[tuple] = []
        for pid in reversed(data.get("order", [])):
            p = data["posts"].get(pid, {})
            if p.get("create_at", 0) <= since_ms or p.get("type"):
                continue
            raw.append(
                (
                    pid,
                    p.get("create_at", 0),
                    (p.get("props") or {}).get("override_username"),
                    p.get("user_id", "?"),
                    p.get("message", ""),
                )
            )
        need = [r[3] for r in raw if not r[2]]
        names = self.usernames(need, token=token) if need else {}
        return [
            Post(
                id=str(pid),
                create_at=int(ca or 0),
                user=str(override or names.get(uid, uid)),
                user_id=str(uid),
                message=str(msg),
            )
            for (pid, ca, override, uid, msg) in raw
        ]

    def usernames(self, ids, *, token=None) -> dict[str, str]:
        """Batch-resolve user_ids -> usernames via POST /users/ids, memoized."""
        miss = [i for i in dict.fromkeys(ids) if i and i not in self._uname_cache]
        if miss:
            try:
                users = (
                    http.post(
                        f"{self.api}/users/ids", token=token or self.bot_token, json_body=miss
                    )
                    or []
                )
            except HttpError:
                users = []
            for u in users:
                self._uname_cache[u["id"]] = u.get("username") or u["id"]
        return {i: self._uname_cache.get(i, i) for i in ids if i}

    def latest_create_at(self, channel_id, *, token=None):
        """Newest post's create_at (server clock) — seed cursors with this, not local time."""
        data = http.get(
            f"{self.api}/channels/{channel_id}/posts",
            token=token or self.bot_token,
            params={"per_page": 1},
        )
        order = (data or {}).get("order", [])
        return data["posts"][order[0]]["create_at"] if order else None

    def me(self, *, token=None):
        return http.get(f"{self.api}/users/me", token=token or self.bot_token)

    def search_posts(self, terms, *, token=None, is_or=True, per_page=20):
        """Full-text search, scoped to the channels the token's user can see."""
        tid = self.team_id()
        return http.post(
            f"{self.api}/teams/{tid}/posts/search",
            token=token or self.bot_token,
            json_body={"terms": terms, "is_or_search": is_or, "per_page": per_page},
        )

    # --- resolution ------------------------------------------------------
    def team_id(self):
        return http.get(f"{self.api}/teams/name/{self.team}", token=self._reg())["id"]

    def channel_id_by_name(self, name):
        ch = http.get(f"{self.api}/teams/name/{self.team}/channels/name/{name}", token=self._reg())
        return ch["id"]

    # --- registrar (provisioning) ---------------------------------------
    def _reg(self):
        return self.registrar_token or self.bot_token

    def create_bot(self, username, display_name):
        """Create a bot account; returns {user_id, ...}. Idempotent-ish: caller checks first."""
        return http.post(
            f"{self.api}/bots",
            token=self._reg(),
            json_body={"username": username, "display_name": display_name},
        )

    def create_user(self, username, email, password):
        """Create a REGULAR user (not a bot → no BOT badge). Posts via its own token."""
        return http.post(
            f"{self.api}/users",
            token=self._reg(),
            json_body={"username": username, "email": email, "password": password},
        )

    def find_user(self, username):
        try:
            return http.get(f"{self.api}/users/username/{username}", token=self._reg())
        except HttpError as e:
            if e.status == 404:
                return None
            raise

    def resolve_channel_id(self, name, *, token=None):
        try:
            ch = http.get(
                f"{self.api}/teams/name/{self.team}/channels/name/{name}",
                token=token or self._reg(),
            )
            return ch["id"]
        except HttpError as e:
            if e.status in (403, 404):
                return None
            raise

    def find_bot(self, username):
        try:
            u = http.get(f"{self.api}/users/username/{username}", token=self._reg())
            return u.get("id")
        except HttpError as e:
            if e.status == 404:
                return None
            raise

    def create_user_token(self, user_id, description="agent token"):
        return http.post(
            f"{self.api}/users/{user_id}/tokens",
            token=self._reg(),
            json_body={"description": description},
        )

    def add_to_team(self, team_id, user_id):
        return http.post(
            f"{self.api}/teams/{team_id}/members",
            token=self._reg(),
            json_body={"team_id": team_id, "user_id": user_id},
        )

    def add_to_channel(self, channel_id, user_id):
        return http.post(
            f"{self.api}/channels/{channel_id}/members",
            token=self._reg(),
            json_body={"user_id": user_id},
        )

    def create_channel(self, team_id, name, display_name, *, ch_type="O"):
        """ch_type: 'O' public, 'P' private."""
        return http.post(
            f"{self.api}/channels",
            token=self._reg(),
            json_body={
                "team_id": team_id,
                "name": name,
                "display_name": display_name,
                "type": ch_type,
            },
        )

    def find_channel(self, name):
        try:
            return self.channel_id_by_name(name)
        except HttpError as e:
            if e.status == 404:
                return None
            raise

    def pin_post(self, post_id):
        return http.post(f"{self.api}/posts/{post_id}/pin", token=self._reg())

    def pinned_posts(self, channel_id):
        return http.get(f"{self.api}/channels/{channel_id}/pinned", token=self._reg())

    # --- cleanup (test/teardown) ----------------------------------------
    def delete_bot_user(self, user_id):
        """Deactivate a user (Mattermost soft-deletes). Used by test teardown."""
        return http.delete(f"{self.api}/users/{user_id}", token=self._reg())

    def delete_channel(self, channel_id):
        return http.delete(f"{self.api}/channels/{channel_id}", token=self._reg())
