"""Roster — who is in the swarm, on what, and live-or-not.

Keyed by **identity** (not session). Local SQLite implementation is the offline cache;
the cross-machine roster of record will be the Go Mattermost plugin (same interface).
Liveness combines last_seen with the §state machine's PID-liveness.
"""
from __future__ import annotations

import socket
import sqlite3
import time

from .config import config


class LocalSqliteRoster:
    """SQLite-backed roster cache. Concurrency-safe via WAL."""

    def __init__(self, db_path=None):
        self.db_path = db_path or config().db_path
        self.db_path.parent.mkdir(parents=True, exist_ok=True)

    def _conn(self):
        c = sqlite3.connect(self.db_path, timeout=10.0)
        c.row_factory = sqlite3.Row
        c.execute("PRAGMA journal_mode=WAL")
        c.execute("PRAGMA busy_timeout=10000")
        c.executescript(
            """
            CREATE TABLE IF NOT EXISTS agents (
                identity   TEXT PRIMARY KEY,
                lead       TEXT,
                session_id TEXT,
                machine    TEXT,
                pid        INTEGER,
                project    TEXT,
                quest      TEXT,
                branch     TEXT,
                status     TEXT,
                started_at REAL,
                last_seen  REAL
            );
            """
        )
        return c

    def upsert(self, identity, **fields):
        now = time.time()
        fields.setdefault("machine", socket.gethostname())
        fields.setdefault("status", "online")
        fields["last_seen"] = now
        cols = ["identity", "started_at"] + list(fields.keys())
        with self._conn() as c:
            row = c.execute("SELECT started_at FROM agents WHERE identity=?", (identity,)).fetchone()
            started = row["started_at"] if row else now
            placeholders = ",".join("?" for _ in cols)
            updates = ",".join(f"{k}=excluded.{k}" for k in fields)
            c.execute(
                f"INSERT INTO agents({','.join(cols)}) VALUES({placeholders}) "
                f"ON CONFLICT(identity) DO UPDATE SET {updates}",
                [identity, started, *fields.values()],
            )

    def heartbeat(self, identity):
        with self._conn() as c:
            c.execute("UPDATE agents SET last_seen=?, status='online' WHERE identity=?",
                      (time.time(), identity))

    def set_status(self, identity, status):
        with self._conn() as c:
            c.execute("UPDATE agents SET status=?, last_seen=? WHERE identity=?",
                      (status, time.time(), identity))

    def offline(self, identity):
        self.set_status(identity, "offline")

    def get(self, identity):
        with self._conn() as c:
            r = c.execute("SELECT * FROM agents WHERE identity=?", (identity,)).fetchone()
            return dict(r) if r else None

    def list(self, *, include_offline=False, stale_secs=300):
        now = time.time()
        out = []
        with self._conn() as c:
            for r in c.execute("SELECT * FROM agents ORDER BY last_seen DESC"):
                age = now - (r["last_seen"] or 0)
                status = r["status"]
                if status == "online" and age > stale_secs:
                    status = "stale"
                if status == "offline" and not include_offline:
                    continue
                d = dict(r)
                d["status"] = status
                d["age_secs"] = round(age, 1)
                out.append(d)
        return out

    def prune(self, *, secs=900, delete=False):
        now = time.time()
        with self._conn() as c:
            stale = c.execute("SELECT identity FROM agents WHERE last_seen < ? AND status='online'",
                              (now - secs,)).fetchall()
            for r in stale:
                c.execute("UPDATE agents SET status='offline' WHERE identity=?", (r["identity"],))
            if delete:
                c.execute("DELETE FROM agents WHERE last_seen < ?", (now - secs,))
        return len(stale)


def roster() -> LocalSqliteRoster:
    return LocalSqliteRoster()
