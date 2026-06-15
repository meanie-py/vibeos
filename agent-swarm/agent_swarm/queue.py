"""Pending-message queue + priority classification — one delivery path for all drains.

Both the CLI poll (busy/hook drain) and the MCP channel server (idle drain) call `drain()`,
so there is ONE cursor and messages are never double-delivered across transports. Cursors
are per (identity, channel) in the roster DB.

Priority: high → deliver immediately (wakes an idle agent); low → counted for a digest.
High = @-mention of this agent / @here|@channel|@all, OR a high board priority
(Vikunja High/Urgent/DO NOW).
"""

from __future__ import annotations

import re
import sqlite3
import time

from .config import config
from .types import DrainResult, HighMessage, Post, Priority

_BOARD_HIGH = re.compile(r"\bpriority\s*[:\-]?\s*(do\s*now|urgent|high)\b", re.I)
_DO_NOW = re.compile(r"\bDO\s*NOW\b", re.I)


def is_board_high(msg: str) -> bool:
    return bool(_BOARD_HIGH.search(msg or "")) or bool(_DO_NOW.search(msg or ""))


def is_mention(msg: str, name: str | None) -> bool:
    m = (msg or "").lower()
    if any(t in m for t in ("@here", "@channel", "@all")):
        return True
    return bool(name) and ("@" + name.lower()) in m


def classify(msg: str, name: str | None) -> Priority:
    return Priority.HIGH if (is_mention(msg, name) or is_board_high(msg)) else Priority.LOW


def now_ms() -> int:
    return int(time.time() * 1000)


def _conn():
    p = config().db_path
    p.parent.mkdir(parents=True, exist_ok=True)
    c = sqlite3.connect(p, timeout=10.0)
    c.row_factory = sqlite3.Row
    c.execute("PRAGMA journal_mode=WAL")
    c.execute("PRAGMA busy_timeout=10000")
    c.executescript(
        """
        CREATE TABLE IF NOT EXISTS cursors (
            identity   TEXT, channel_id TEXT,
            read_at    INTEGER, mention_at INTEGER, last_count INTEGER,
            PRIMARY KEY(identity, channel_id)
        );
        CREATE TABLE IF NOT EXISTS delivered (
            identity TEXT, post_id TEXT, ts REAL,
            PRIMARY KEY(identity, post_id)
        );
        """
    )
    return c


def _init_cursor(c, identity, channel_id, mm):
    row = c.execute(
        "SELECT 1 FROM cursors WHERE identity=? AND channel_id=?", (identity, channel_id)
    ).fetchone()
    if row:
        return
    at = None
    if mm:
        at = mm.latest_create_at(channel_id)  # server clock — skew-immune
    at = at or now_ms()
    c.execute(
        "INSERT OR IGNORE INTO cursors(identity,channel_id,read_at,mention_at,last_count)"
        " VALUES(?,?,?,?,0)",
        (identity, channel_id, at, at),
    )


def seed(identity, channel_ids, mm):
    """Seed cursors to 'now' so the agent only sees chatter from when it came online."""
    with _conn() as c:
        for ch in channel_ids:
            if ch:
                _init_cursor(c, identity, ch, mm)


def _self_set(name: str | None, self_ids) -> set[str]:
    selves = {name} | set(self_ids or ())
    selves.discard(None)
    return selves  # type: ignore[return-value]


def _not_self(posts: list[Post], selves: set[str]) -> list[Post]:
    # An agent's own posts show as its username (webhook) OR its user_id (REST) — drop both.
    return [p for p in posts if str(p.user).split(" @ ")[0] not in selves]


def drain(identity, name, channel_ids, mm, *, high_only=False, self_ids=None) -> DrainResult:
    """Read new posts across channels, classify, advance the mention cursor.

    Does NOT advance the read cursor (that's `unread`). `self_ids` (e.g. the agent's
    mm_user_id) are dropped alongside `name` so the agent is never notified of its own posts.
    """
    selves = _self_set(name, self_ids)
    result = DrainResult()
    with _conn() as c:
        for ch in channel_ids:
            if not ch:
                continue
            _init_cursor(c, identity, ch, mm)
            cur = c.execute(
                "SELECT read_at, mention_at FROM cursors WHERE identity=? AND channel_id=?",
                (identity, ch),
            ).fetchone()
            mention_at = cur["mention_at"]
            posts = _not_self(mm.read_posts(ch, cur["read_at"]), selves)
            candidates = [
                p
                for p in posts
                if classify(p.message, name) is Priority.HIGH and p.create_at > mention_at
            ]
            # post-id dedup: deliver each post at most ONCE per identity, even if the cursor
            # is reset or two drains race (the mention cursor alone can't guarantee it).
            new_high: list[Post] = []
            for p in candidates:
                if c.execute(
                    "SELECT 1 FROM delivered WHERE identity=? AND post_id=?", (identity, p.id)
                ).fetchone():
                    continue
                c.execute(
                    "INSERT OR IGNORE INTO delivered(identity,post_id,ts) VALUES(?,?,?)",
                    (identity, p.id, time.time()),
                )
                new_high.append(p)
            result.low_count += sum(1 for p in posts if classify(p.message, name) is Priority.LOW)
            if new_high:
                mention_at = max(mention_at, max(p.create_at for p in new_high))
            c.execute(
                "UPDATE cursors SET mention_at=?, last_count=? WHERE identity=? AND channel_id=?",
                (mention_at, result.low_count, identity, ch),
            )
            for p in new_high:
                result.high.append(
                    HighMessage(
                        sender=str(p.user).split(" @ ")[0],
                        message=p.message,
                        post_id=p.id,
                        channel=ch,
                    )
                )
    if high_only:
        result.low_count = 0
    return result


def unread(identity, name, channel_ids, mm, *, self_ids=None) -> list[Post]:
    """All unread posts across channels (for `read`), then advance the read cursor."""
    selves = _self_set(name, self_ids)
    out: list[Post] = []
    with _conn() as c:
        for ch in channel_ids:
            if not ch:
                continue
            _init_cursor(c, identity, ch, mm)
            cur = c.execute(
                "SELECT read_at FROM cursors WHERE identity=? AND channel_id=?", (identity, ch)
            ).fetchone()
            posts = _not_self(mm.read_posts(ch, cur["read_at"]), selves)
            hi = max((p.create_at for p in posts), default=cur["read_at"])
            c.execute(
                "UPDATE cursors SET read_at=?, mention_at=?, last_count=0 "
                "WHERE identity=? AND channel_id=?",
                (hi, hi, identity, ch),
            )
            out.extend(posts)
    out.sort(key=lambda p: p.create_at)
    return out
