"""Roster tests — remove() (teardown cleanup) + last_seen-based liveness.

NB: liveness is deliberately last_seen-based, not pid-based — Claude Code hooks run as
short-lived subprocesses, so the recorded pid is dead moments later and a pid reap would
offline every live agent. A crashed session stops heartbeating → ages to 'stale' → prune.
"""

from __future__ import annotations

import socket
import time

from agent_swarm.roster import LocalSqliteRoster


def test_remove_deletes_row(env):
    """teardown's local-cleanup primitive: the row is gone, not just offlined."""
    r = LocalSqliteRoster()
    r.upsert("doomed", pid=1234, machine=socket.gethostname())
    assert r.get("doomed") is not None
    r.remove("doomed")
    assert r.get("doomed") is None
    assert "doomed" not in {a.identity for a in r.list(include_offline=True)}


def test_fresh_agent_is_online(env):
    r = LocalSqliteRoster()
    r.upsert("fresh", pid=4321, machine=socket.gethostname())
    live = {a.identity: a for a in r.list()}
    assert "fresh" in live and live["fresh"].status.value == "online"


def test_stale_by_last_seen(env):
    """An online row that hasn't heartbeated past stale_secs shows 'stale' (not online)."""
    r = LocalSqliteRoster()
    r.upsert("quiet", pid=4321, machine=socket.gethostname())
    # backdate last_seen well past the staleness window
    with r._conn() as c:
        c.execute("UPDATE agents SET last_seen=? WHERE identity=?", (time.time() - 10_000, "quiet"))
    statuses = {a.identity: a.status.value for a in r.list(stale_secs=300)}
    assert statuses.get("quiet") == "stale"


def test_offline_hidden_unless_requested(env):
    r = LocalSqliteRoster()
    r.upsert("gone", pid=4321, machine=socket.gethostname())
    r.offline("gone")
    assert "gone" not in {a.identity for a in r.list()}
    assert "gone" in {a.identity for a in r.list(include_offline=True)}
