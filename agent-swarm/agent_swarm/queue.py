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

_BOARD_HIGH = re.compile(r"\bpriority\s*[:\-]?\s*(do\s*now|urgent|high)\b", re.I)
_DO_NOW = re.compile(r"\bDO\s*NOW\b", re.I)


def is_board_high(msg: str) -> bool:
    return bool(_BOARD_HIGH.search(msg or "")) or bool(_DO_NOW.search(msg or ""))


def is_mention(msg: str, name: str | None) -> bool:
    m = (msg or "").lower()
    if any(t in m for t in ("@here", "@channel", "@all")):
        return True
    return bool(name) and ("@" + name.lower()) in m


def classify(msg: str, name: str | None) -> str:
    return "high" if (is_mention(msg, name) or is_board_high(msg)) else "low"


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
        """
    )
    return c


def _init_cursor(c, identity, channel_id, mm):
    row = c.execute("SELECT 1 FROM cursors WHERE identity=? AND channel_id=?",
                    (identity, channel_id)).fetchone()
    if row:
        return
    at = None
    if mm:
        at = mm.latest_create_at(channel_id)  # server clock — skew-immune
    at = at or now_ms()
    c.execute("INSERT OR IGNORE INTO cursors(identity,channel_id,read_at,mention_at,last_count)"
              " VALUES(?,?,?,?,0)", (identity, channel_id, at, at))


def seed(identity, channel_ids, mm):
    """Seed cursors to 'now' so the agent only sees chatter from when it came online."""
    with _conn() as c:
        for ch in channel_ids:
            if ch:
                _init_cursor(c, identity, ch, mm)


def drain(identity, name, channel_ids, mm, *, high_only=False):
    """Read new posts across channels, classify, advance the mention cursor.

    Returns {"high": [{from, message, post_id, channel}], "low_count": N}.
    Does NOT advance the read cursor (that's mark_read / an explicit `read`).
    """
    high, low_count = [], 0
    with _conn() as c:
        for ch in channel_ids:
            if not ch:
                continue
            _init_cursor(c, identity, ch, mm)
            cur = c.execute("SELECT read_at, mention_at, last_count FROM cursors "
                            "WHERE identity=? AND channel_id=?", (identity, ch)).fetchone()
            read_at, mention_at = cur["read_at"], cur["mention_at"]
            posts = mm.read_posts(ch, read_at)
            posts = [p for p in posts if str(p["user"]).split(" @ ")[0] != (name or "")]
            new_high = [p for p in posts
                        if classify(p["message"], name) == "high" and p["create_at"] > mention_at]
            low_count += sum(1 for p in posts if classify(p["message"], name) == "low")
            if new_high:
                mention_at = max(mention_at, max(p["create_at"] for p in new_high))
            c.execute("UPDATE cursors SET mention_at=?, last_count=? WHERE identity=? AND channel_id=?",
                      (mention_at, low_count, identity, ch))
            for p in new_high:
                high.append({"from": str(p["user"]).split(" @ ")[0], "message": p["message"],
                             "post_id": p["id"], "channel": ch})
    return {"high": high, "low_count": 0 if high_only else low_count}


def unread(identity, name, channel_ids, mm):
    """All unread posts across channels (for `read`), then advance the read cursor."""
    out = []
    with _conn() as c:
        for ch in channel_ids:
            if not ch:
                continue
            _init_cursor(c, identity, ch, mm)
            cur = c.execute("SELECT read_at FROM cursors WHERE identity=? AND channel_id=?",
                            (identity, ch)).fetchone()
            posts = mm.read_posts(ch, cur["read_at"])
            posts = [p for p in posts if str(p["user"]).split(" @ ")[0] != (name or "")]
            hi = max([p["create_at"] for p in posts], default=cur["read_at"])
            c.execute("UPDATE cursors SET read_at=?, mention_at=?, last_count=0 "
                      "WHERE identity=? AND channel_id=?", (hi, hi, identity, ch))
            for p in posts:
                out.append({**p, "channel": ch,
                            "priority": classify(p["message"], name)})
    out.sort(key=lambda p: p["create_at"])
    return out
