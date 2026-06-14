# `tools/swarm` — agent presence registry

Local, SQLite-backed roster of the Claude/Codex agents working vibeos
concurrently. Answers *"who is live, on which worktree/branch, working on what?"* —
the **board's presence layer** in the swarm-coordination design
(`docs/engineering/swarm-comms-plan.md`).

## Why it exists

Agents now run **one-per-worktree**. Two things follow:
- Worktree + branch are **ground truth** (`git rev-parse` in the agent's own cwd).
- Agent project-memory is stored **per-cwd-hash**, so per-worktree → **not shared**.
  It cannot be the roster.

So the roster lives in **one SQLite DB at a fixed shared path**, written through
SQLite's own locking (WAL). Concurrency-safe, no daemon. "Registration server" ==
this DB + a tiny CLI; every hook invocation opens it, does one op, closes.

## Identity is free

`/rename` writes `{"type":"agent-name","agentName":"fastfat",...}` into the session
transcript (repeatedly). We read it back. Resolution order:
`agentName` (your `/rename`) → `aiTitle` (auto) → short session-id.

## Design constraints (deliberate)

- **stdlib only**, invoked by **absolute path** — *not* pip-installable, to dodge
  the worktree site-packages shadowing trap (see `AGENTS.md` "pip install -e shadowing").
- **Defensive:** every hook script and the CLI's top-level exit 0 on any error. A
  registry hiccup must **never** break an agent's session.
- **Mattermost is an optional mirror**, never a dependency. Registry is authoritative
  and works while the chat server is down/being-configured.

## CLI

```bash
PY=python3; S="$CLAUDE_PROJECT_DIR/tools/swarm/swarm.py"
$PY $S register  --session ID --transcript PATH --cwd DIR [--pid N]
$PY $S heartbeat --session ID [--transcript PATH]      # last_seen + pick up /rename
$PY $S offline   --session ID [--reason R]
$PY $S list      [--json] [--all] [--stale-secs 300]   # the roster
$PY $S prune     [--secs 900] [--delete]               # stale online → offline
$PY $S whoami    --transcript PATH                      # debug: resolve name only
$PY $S install-hooks [--write [settings.json]]          # wire the hooks
```

`list` marks each agent 🟢 online / 🟡 stale (last_seen older than `--stale-secs`) /
⚫ offline.

## Wiring the hooks (opt-in, per-user)

`.claude/settings.json` is gitignored by repo policy (hook wiring is per-user, not
forced on the swarm), so install it yourself:

```bash
python3 tools/swarm/swarm.py install-hooks --write   # → ~/.claude/settings.json
```

This wires three hooks (all defensive, all exit 0):
- **SessionStart** → `register` (announce worktree/branch/name at boot).
- **PostToolUse** → `heartbeat`, throttled to once / `VIBEOS_SWARM_HB_INTERVAL`s
  (default 60) per session; also re-reads `/rename`.
- **SessionEnd** → `offline`.

Claude Code shows project/user hooks for approval before running them — nothing runs
silently.

## Config (env)

| Var | Default | Meaning |
|---|---|---|
| `VIBEOS_SWARM_DB` | `~/.vibeos-swarm/registry.db` | shared DB path (must be reachable from every worktree) |
| `VIBEOS_SWARM_HB_INTERVAL` | `60` | heartbeat throttle seconds |
| `MATTERMOST_WEBHOOK_URL` | — | if set, mirror register/rename/offline to chat |
| `MATTERMOST_PRESENCE_CHANNEL` | webhook default | channel for mirror posts |
| `MATTERMOST_ICON_URL` | — | optional avatar |

Mattermost mirror is **off** until `MATTERMOST_WEBHOOK_URL` is set; the registry is
fully functional without it.

## Schema

`agents(session_id PK, name, worktree, branch, cwd, pid, status, started_at, last_seen)`
plus an append-only `events(ts, session_id, kind, detail)` audit log
(`register|rename|heartbeat|offline`).
