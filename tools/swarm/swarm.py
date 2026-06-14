#!/usr/bin/env python3
"""vibeos swarm registry — local, SQLite-backed agent presence roster.

Part of the swarm-coordination technique (see docs/engineering/swarm-comms-plan.md).

WHY THIS EXISTS
    Multiple Claude Code / Codex agents work vibeos concurrently, now each invoked
    *in its own worktree*. They need a shared, concurrency-safe answer to "who is
    live, on which worktree/branch, working on what?". Agent project-memory can't
    do it (it's stored per-cwd-hash, so per-worktree → not shared). A flat file
    races. So: one SQLite DB at a fixed shared path, written through SQLite's own
    locking (WAL mode) → concurrency-safe roster with no daemon to babysit.

DESIGN
    - Serverless. Every hook invocation opens the DB, does one op, closes. SQLite
      handles concurrent writers. "Registration server" == this DB + this CLI.
    - Dependency-free (stdlib only) and invoked by absolute path — deliberately NOT
      pip-installable, to avoid the worktree site-packages shadowing trap (see
      AGENTS.md "pip install -e shadowing").
    - Identity is free: `/rename` writes {"type":"agent-name","agentName":...} into
      the session transcript; we read it back (→ ai-title → short session-id).
    - Mattermost is an optional MIRROR (best-effort webhook post), never a
      dependency. The registry is authoritative and works while the server is down.

DB LOCATION
    $VIBEOS_SWARM_DB, else ~/.vibeos-swarm/registry.db  (shared across all worktrees).

CLI
    swarm.py register  --session ID --transcript PATH --cwd DIR [--pid N]
    swarm.py heartbeat --session ID [--transcript PATH] [--cwd DIR]
    swarm.py offline   --session ID [--reason R]
    swarm.py list      [--json] [--stale-secs N] [--all]
    swarm.py prune     [--secs N] [--delete]
    swarm.py whoami    --transcript PATH        # debug: resolve identity only
"""
from __future__ import annotations

import argparse
import json
import os
import sqlite3
import subprocess
import sys
import time
import urllib.error
import urllib.request

# ---------------------------------------------------------------------------
# DB
# ---------------------------------------------------------------------------

def db_path() -> str:
    p = os.environ.get("VIBEOS_SWARM_DB")
    if not p:
        p = os.path.join(os.path.expanduser("~"), ".vibeos-swarm", "registry.db")
    os.makedirs(os.path.dirname(p), exist_ok=True)
    return p


def connect() -> sqlite3.Connection:
    conn = sqlite3.connect(db_path(), timeout=10.0)
    conn.row_factory = sqlite3.Row
    # WAL = concurrent readers don't block the writer; safe for a multi-agent swarm.
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=10000")
    conn.executescript(
        """
        CREATE TABLE IF NOT EXISTS agents (
            session_id TEXT PRIMARY KEY,
            name       TEXT,
            worktree   TEXT,
            branch     TEXT,
            cwd        TEXT,
            pid        INTEGER,
            status     TEXT,      -- 'online' | 'offline'
            started_at REAL,
            last_seen  REAL
        );
        CREATE TABLE IF NOT EXISTS events (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            ts         REAL,
            session_id TEXT,
            kind       TEXT,      -- register | rename | heartbeat | offline
            detail     TEXT
        );
        """
    )
    return conn


def log_event(conn, session_id, kind, detail=""):
    conn.execute(
        "INSERT INTO events(ts, session_id, kind, detail) VALUES(?,?,?,?)",
        (time.time(), session_id, kind, detail),
    )


# ---------------------------------------------------------------------------
# Identity — read the name the human set via /rename (then ai-title, then short id)
# ---------------------------------------------------------------------------

def resolve_name(transcript_path: str | None, session_id: str | None = None,
                 tail_bytes: int = 1_048_576) -> str | None:
    """Return the agent's display name from its transcript.

    /rename appends {"type":"agent-name","agentName":...} repeatedly, so the last
    occurrence in the tail is current. Falls back to the auto "ai-title", then a
    short session id. Reads only the tail (transcripts get huge).
    """
    name = None
    title = None
    if transcript_path and os.path.exists(transcript_path):
        try:
            size = os.path.getsize(transcript_path)
            with open(transcript_path, "rb") as fh:
                if size > tail_bytes:
                    fh.seek(size - tail_bytes)
                chunk = fh.read()
            text = chunk.decode("utf-8", errors="ignore")
            for line in reversed(text.splitlines()):
                if '"agent-name"' not in line and '"ai-title"' not in line:
                    continue
                try:
                    obj = json.loads(line)
                except ValueError:
                    continue  # partial line at chunk boundary
                t = obj.get("type")
                if t == "agent-name" and name is None and obj.get("agentName"):
                    name = obj["agentName"]
                elif t == "ai-title" and title is None and obj.get("aiTitle"):
                    title = obj["aiTitle"]
                if name:
                    break
        except OSError:
            pass
    if name:
        return name
    if title:
        return title
    if session_id:
        return session_id[:8]
    return None


# ---------------------------------------------------------------------------
# Git — worktree + branch are ground truth now that agents run in-worktree
# ---------------------------------------------------------------------------

def git_info(cwd: str | None) -> tuple[str | None, str | None]:
    if not cwd or not os.path.isdir(cwd):
        return None, None

    def _q(args):
        try:
            out = subprocess.run(
                ["git", "-C", cwd, *args],
                capture_output=True, text=True, timeout=5,
            )
            return out.stdout.strip() if out.returncode == 0 else None
        except (OSError, subprocess.SubprocessError):
            return None

    return _q(["rev-parse", "--show-toplevel"]), _q(["rev-parse", "--abbrev-ref", "HEAD"])


# ---------------------------------------------------------------------------
# Mattermost mirror — best-effort, never fatal
# ---------------------------------------------------------------------------

def mattermost_post(text: str, username: str | None = None,
                    channel: str | None = None) -> bool:
    url = os.environ.get("MATTERMOST_WEBHOOK_URL")
    if not url:
        return False  # server not wired yet — registry still works
    payload = {"text": text}
    if username:
        payload["username"] = username
    ch = channel or os.environ.get("MATTERMOST_PRESENCE_CHANNEL")
    if ch:
        payload["channel"] = ch
    icon = os.environ.get("MATTERMOST_ICON_URL")
    if icon:
        payload["icon_url"] = icon
    try:
        req = urllib.request.Request(
            url, data=json.dumps(payload).encode(),
            headers={"Content-Type": "application/json"},
        )
        urllib.request.urlopen(req, timeout=5).read()
        return True
    except (urllib.error.URLError, OSError):
        return False  # swallow: a down server must never break an agent's session


def _mm_username(name, worktree):
    wt = os.path.basename(worktree) if worktree else "?"
    return f"{name} @ {wt}"


# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

def cmd_register(args):
    name = resolve_name(args.transcript, args.session)
    worktree, branch = git_info(args.cwd)
    now = time.time()
    conn = connect()
    with conn:
        row = conn.execute(
            "SELECT started_at FROM agents WHERE session_id=?", (args.session,)
        ).fetchone()
        started = row["started_at"] if row else now
        conn.execute(
            """INSERT INTO agents(session_id,name,worktree,branch,cwd,pid,status,started_at,last_seen)
               VALUES(?,?,?,?,?,?,'online',?,?)
               ON CONFLICT(session_id) DO UPDATE SET
                 name=excluded.name, worktree=excluded.worktree, branch=excluded.branch,
                 cwd=excluded.cwd, pid=excluded.pid, status='online', last_seen=excluded.last_seen""",
            (args.session, name, worktree, branch, args.cwd, args.pid, started, now),
        )
        log_event(conn, args.session, "register", f"{name} @ {worktree} [{branch}]")
    mattermost_post(
        f"🟢 **{name or args.session[:8]}** online — `{branch}` in `{worktree}`",
        username=_mm_username(name or args.session[:8], worktree),
    )
    print(f"registered {name or args.session[:8]} @ {worktree} [{branch}]")


def cmd_heartbeat(args):
    now = time.time()
    conn = connect()
    with conn:
        row = conn.execute(
            "SELECT name, worktree, status FROM agents WHERE session_id=?",
            (args.session,),
        ).fetchone()
        if row is None:
            # Heartbeat before register (hook ordering) — register lazily.
            conn.close()
            return cmd_register(args)
        # Refresh the name in case /rename happened since register/last beat.
        new_name = resolve_name(args.transcript, args.session) if args.transcript else row["name"]
        renamed = new_name and new_name != row["name"]
        conn.execute(
            "UPDATE agents SET last_seen=?, status='online', name=COALESCE(?,name) WHERE session_id=?",
            (now, new_name, args.session),
        )
        if renamed:
            log_event(conn, args.session, "rename", f"{row['name']} -> {new_name}")
    if renamed:
        mattermost_post(
            f"✏️ **{row['name'] or args.session[:8]}** is now **{new_name}**",
            username=_mm_username(new_name, row["worktree"]),
        )
    if args.verbose:
        print(f"heartbeat {new_name or args.session[:8]}")


def cmd_offline(args):
    now = time.time()
    conn = connect()
    with conn:
        row = conn.execute(
            "SELECT name, worktree, branch FROM agents WHERE session_id=?",
            (args.session,),
        ).fetchone()
        conn.execute(
            "UPDATE agents SET status='offline', last_seen=? WHERE session_id=?",
            (now, args.session),
        )
        log_event(conn, args.session, "offline", args.reason or "")
    if row:
        mattermost_post(
            f"⚫ **{row['name'] or args.session[:8]}** offline ({args.reason or 'ended'}) — `{row['branch']}`",
            username=_mm_username(row["name"] or args.session[:8], row["worktree"]),
        )
    print(f"offline {args.session[:8]}")


def cmd_list(args):
    now = time.time()
    conn = connect()
    rows = conn.execute(
        "SELECT * FROM agents ORDER BY last_seen DESC"
    ).fetchall()
    out = []
    for r in rows:
        age = now - (r["last_seen"] or 0)
        stale = age > args.stale_secs
        live = r["status"] == "online" and not stale
        if not args.all and r["status"] == "offline":
            continue
        out.append({
            "session": r["session_id"],
            "name": r["name"],
            "worktree": r["worktree"],
            "branch": r["branch"],
            "status": "offline" if r["status"] == "offline" else ("stale" if stale else "online"),
            "live": live,
            "age_secs": round(age, 1),
            "pid": r["pid"],
        })
    if args.json:
        print(json.dumps(out, indent=2))
        return
    if not out:
        print("(no agents)")
        return
    icon = {"online": "🟢", "stale": "🟡", "offline": "⚫"}
    print(f"{'':2} {'NAME':<22} {'BRANCH':<22} {'AGE':>6}  WORKTREE")
    for a in out:
        age = a["age_secs"]
        agestr = f"{age:.0f}s" if age < 120 else f"{age/60:.0f}m"
        print(f"{icon.get(a['status'],'?'):2} {(a['name'] or a['session'][:8]):<22} "
              f"{(a['branch'] or '?'):<22} {agestr:>6}  {a['worktree'] or '?'}")


def cmd_prune(args):
    now = time.time()
    conn = connect()
    with conn:
        stale = conn.execute(
            "SELECT session_id FROM agents WHERE last_seen < ? AND status='online'",
            (now - args.secs,),
        ).fetchall()
        for r in stale:
            conn.execute(
                "UPDATE agents SET status='offline' WHERE session_id=?", (r["session_id"],)
            )
            log_event(conn, r["session_id"], "offline", "pruned (stale)")
        if args.delete:
            conn.execute("DELETE FROM agents WHERE last_seen < ?", (now - args.secs,))
    print(f"pruned {len(stale)} stale agent(s)" + (" (+deleted)" if args.delete else ""))


def cmd_whoami(args):
    print(resolve_name(args.transcript, args.session) or "(unknown)")


def _hook_block():
    base = '$CLAUDE_PROJECT_DIR/tools/swarm/hooks'
    cmd = lambda s: {"type": "command", "command": f'bash "{base}/{s}"'}
    return {
        "SessionStart": [{"hooks": [cmd("session_start.sh")]}],
        "PostToolUse": [{"matcher": "*", "hooks": [cmd("post_tool_use.sh")]}],
        "SessionEnd": [{"hooks": [cmd("session_end.sh")]}],
    }


def cmd_install_hooks(args):
    block = _hook_block()
    if not args.write:
        print("Add this to your .claude/settings.json (or ~/.claude/settings.json):\n")
        print(json.dumps({"hooks": block}, indent=2))
        print("\n(Re-run with --write [PATH] to merge it in automatically.)")
        return
    path = args.write if isinstance(args.write, str) else os.path.expanduser("~/.claude/settings.json")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    settings = {}
    if os.path.exists(path):
        try:
            settings = json.load(open(path))
        except ValueError:
            print(f"refusing to overwrite unparseable {path}"); return
    hooks = settings.setdefault("hooks", {})
    for event, entries in block.items():
        existing = hooks.setdefault(event, [])
        # idempotent: skip if a swarm hook is already wired for this event
        flat = json.dumps(existing)
        if "tools/swarm/hooks" in flat:
            continue
        existing.extend(entries)
    json.dump(settings, open(path, "w"), indent=2)
    print(f"merged swarm hooks into {path}")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv=None):
    p = argparse.ArgumentParser(prog="swarm", description="vibeos swarm agent registry")
    sub = p.add_subparsers(dest="cmd", required=True)

    pr = sub.add_parser("register"); pr.set_defaults(fn=cmd_register)
    pr.add_argument("--session", required=True)
    pr.add_argument("--transcript")
    pr.add_argument("--cwd")
    pr.add_argument("--pid", type=int)

    ph = sub.add_parser("heartbeat"); ph.set_defaults(fn=cmd_heartbeat)
    ph.add_argument("--session", required=True)
    ph.add_argument("--transcript")
    ph.add_argument("--cwd")
    ph.add_argument("--pid", type=int)
    ph.add_argument("--verbose", action="store_true")

    po = sub.add_parser("offline"); po.set_defaults(fn=cmd_offline)
    po.add_argument("--session", required=True)
    po.add_argument("--reason")

    pl = sub.add_parser("list"); pl.set_defaults(fn=cmd_list)
    pl.add_argument("--json", action="store_true")
    pl.add_argument("--all", action="store_true", help="include offline agents")
    pl.add_argument("--stale-secs", type=int, default=300)

    pp = sub.add_parser("prune"); pp.set_defaults(fn=cmd_prune)
    pp.add_argument("--secs", type=int, default=900)
    pp.add_argument("--delete", action="store_true")

    pw = sub.add_parser("whoami"); pw.set_defaults(fn=cmd_whoami)
    pw.add_argument("--session")
    pw.add_argument("--transcript", required=True)

    pi = sub.add_parser("install-hooks"); pi.set_defaults(fn=cmd_install_hooks)
    pi.add_argument("--write", nargs="?", const=True, default=False,
                    help="merge into settings.json (default ~/.claude/settings.json) instead of printing")

    args = p.parse_args(argv)
    args.fn(args)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as e:  # a registry bug must NEVER break the agent's session
        sys.stderr.write(f"swarm: {e}\n")
        sys.exit(0)
