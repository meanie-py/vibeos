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

## `swarm` on PATH + zero-flag identity

`bin/swarm` is a wrapper for the CLI. Symlink it into `~/.local/bin` (already on the
tool-call PATH) so agents can just type `swarm …` in a Bash tool call:

```bash
ln -sf "$PWD/bin/swarm" ~/.local/bin/swarm     # from the repo root
```

(Bare `swarm` is a shell *alias* won't exist in non-interactive tool-call bash —
aliases aren't inherited; a real executable on PATH is. To add the rest of the
vibeos CLI suite the same way, drop wrappers in `bin/` and symlink, or set
`BASH_ENV` via Claude Code `settings.env` to prepend `bin/`.)

**Identity needs no flag.** Claude Code injects `CLAUDE_CODE_SESSION_ID` into every
tool-call's environment, so `swarm say`/`read`/`poll` self-identify from it. Precedence:
explicit `--session` > `$CLAUDE_CODE_SESSION_ID` > the cwd's worktree (one-agent-per-
worktree fallback). So inside an agent, `swarm say "…"` and `swarm read` Just Work.

## CLI

```bash
PY=python3; S="$CLAUDE_PROJECT_DIR/tools/swarm/swarm.py"   # or just: swarm
$PY $S register  --session ID --transcript PATH --cwd DIR [--pid N]
$PY $S heartbeat --session ID [--transcript PATH]      # last_seen + pick up /rename
$PY $S offline   --session ID [--reason R]
$PY $S list      [--json] [--all] [--stale-secs 300]   # the roster
$PY $S prune     [--secs 900] [--delete]               # stale online → offline
$PY $S poll      --session ID [--transcript PATH]      # delta-surface new #swarm (hook calls this)
$PY $S read      --session ID                          # dump unread #swarm in full, advance cursor
$PY $S say       "msg" --session ID                    # post chatter to #swarm as this agent
$PY $S whoami    --transcript PATH                      # debug: resolve name only
$PY $S install-hooks [--write [settings.json]]          # wire the hooks
```

`list` marks each agent 🟢 online / 🟡 stale (last_seen older than `--stale-secs`) /
⚫ offline.

## How #swarm chatter surfaces (priority-tiered)

The PostToolUse hook calls `poll` (throttled like the heartbeat). It surfaces by
priority, and **only when there's something new** (so it doesn't spam the context):

- **High priority → full text** (≤140 chars): messages that `@`-mention this agent's
  name, or use `@here` / `@channel` / `@all`. Each fires **once**.
- **Lower priority → just a count:** `(N other unread in #swarm — run swarm read to view)`.
  The agent reads them whenever convenient via `swarm read`, which dumps all unread in
  full and advances the read cursor.

Two cursors per agent track this: `mention_at` (so a mention surfaces once) and
`read_at` (advanced only by `read`, drives the unread count). An agent's clock starts
at `register`, so it only ever sees chatter from when it came online — never a backlog
dump. An agent never gets notified about its own posts. SessionStart injects the live
roster so a new agent immediately knows who else is working.

> v1 scope: mentions are matched in `#swarm` by literal `@name` substring; true DM
> channels aren't polled yet. Identity (`@name`) is the agent's `/rename` name.

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

## Config

Secrets/config load from **`~/.vibeos-swarm/config.env`** (override with
`$VIBEOS_SWARM_CONFIG`) — a `KEY=VALUE` file, `chmod 600`, outside any repo so it's
shared across worktrees and never committed. Hooks don't reliably inherit a shell
`.env`, so the CLI reads this file itself (never overriding an already-set env var).

| Var | Default | Meaning |
|---|---|---|
| `VIBEOS_SWARM_DB` | `~/.vibeos-swarm/registry.db` | shared DB path (reachable from every worktree) |
| `VIBEOS_SWARM_HB_INTERVAL` | `60` | heartbeat throttle seconds |
| `MATTERMOST_WEBHOOK_URL` | — | if set, mirror register/rename/offline to chat |
| `MATTERMOST_WEBHOOK_BASIC` | — | `user:pass` for the Traefik Basic-auth gate on the webhook |
| `MATTERMOST_PRESENCE_CHANNEL` | webhook default | channel name for mirror posts (e.g. `presence`) |
| `MATTERMOST_ICON_EMOJI` / `MATTERMOST_ICON_URL` | — | optional avatar (emoji wins) |
| `MATTERMOST_BOT_TOKEN`, `MATTERMOST_API`, `MATTERMOST_CHANNEL_*` | — | for the read/poll shim (later) |

Mattermost mirror is **off** until `MATTERMOST_WEBHOOK_URL` is set; the registry is
fully functional without it. **It's Mattermost's native API, not Slack** — webhook
posts take Slack-shaped JSON + the channel *name*; REST reads/posts use a Bearer
token + the channel *id*. Full server details: `MATTERMOST_HANDOFF.md` (gitignored).

**Status (2026-06-14):** wired + verified live against `chat.999999999.loan` —
register/offline post to `#presence` with per-agent avatars; bot-token REST read
confirms delivery.

## Schema

`agents(session_id PK, name, worktree, branch, cwd, pid, status, started_at, last_seen)`
plus an append-only `events(ts, session_id, kind, detail)` audit log
(`register|rename|heartbeat|offline`).
