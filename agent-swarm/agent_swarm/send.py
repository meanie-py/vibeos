"""The shared `send` primitive — post to a channel as an identity.

Deliberately small and dependency-light so *any* component reuses it (the channel
server's reply tool, the merge announcer, the Vikunja bridge, the CLI). One credential
model (Config), one code path.
"""

from __future__ import annotations

from . import identity as ident
from .config import Config
from .config import config as default_config
from .errors import HttpError
from .mattermost import MattermostClient


def send(
    text: str,
    *,
    channel: str,
    as_identity: str | None = None,
    icon_emoji: str | None = None,
    cfg: Config | None = None,
) -> bool:
    """Post `text` to `channel` (name) as `as_identity`.

    Preferred path: the agent's OWN personal-access token via REST → posts as a proper
    account (no "BOT" badge). Falls back to the incoming webhook (username override,
    BOT-tagged) when the agent has no token yet. Best-effort: never raises.
    """
    cfg = cfg or default_config()
    mm = MattermostClient.from_config(cfg)
    if as_identity:
        token = (ident.creds_for(as_identity) or {}).get("mm_token")
        if token:
            ch_id = mm.resolve_channel_id(channel, token=token)
            if ch_id:
                try:
                    mm.post(ch_id, text, token=token)
                    return True
                except HttpError:
                    pass  # fall through to webhook
    return mm.webhook_post(text, channel=channel, username=as_identity, icon_emoji=icon_emoji)


def send_threaded(channel_id: str, text: str, *, root_id: str = "", cfg: Config | None = None):
    """Bot REST post when you need the post id / threading (no per-message avatar)."""
    cfg = cfg or default_config()
    mm = MattermostClient.from_config(cfg)
    return mm.post(channel_id, text, root_id=root_id)
