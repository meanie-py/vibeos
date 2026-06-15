# agent-swarm

The vibeos agent-swarm platform — lets a swarm of Claude agents (cross-machine,
not-always-online) coordinate the vibeos port without the human as the router. Design &
rationale: [`docs/engineering/agent-swarm-platform-design.md`](../docs/engineering/agent-swarm-platform-design.md).

## Structure (core library + interchangeable front-ends)

```
agent-swarm/
  agent_swarm/        # the core library (Python, stdlib-only)
    config.py         #   merged config (~/.vibeos-swarm/config.env + repo .env + env)
    http.py           #   tiny stdlib HTTP client (no third-party deps)
    mattermost.py     #   messaging, reads, registrar provisioning
    vikunja.py        #   task source of truth (epics/quests/subtasks)
    authentik.py      #   identity (agents = M2M service accounts)
    identity.py       #   login + identity resolution (not session/cwd)
    roster.py         #   who's in the swarm (SQLite cache; Go plugin later)
    state.py          #   busy/idle state machine + PID-liveness
    queue.py          #   pending queue + priority classification (one delivery path)
    send.py           #   the shared `send` primitive
    registrar.py      #   provisioning fan-out across the three services
    cli.py            #   the CLI front-end (self-documenting --help)
  channel/            # MCP channel server — thin transport over the CLI core
  hooks/              # SessionStart / UserPromptSubmit / PostToolUse / Stop / SessionEnd
```

The **CLI** and the **MCP channel** are interchangeable front-ends over the same core:
the channel server delegates all delivery logic to `vibeos-cli swarm drain`, so there is
one queue, one cursor, one classifier.

## CLI

Folded into `vibeos-cli` (not a separate top-level tool): **`vibeos-cli swarm <cmd>`**.
It runs the package via `PYTHONPATH` — **not** pip-installed (avoids the worktree
site-packages shadowing trap).

```
vibeos-cli swarm login <name>     # bind this session to an identity ($SWARM_LOGIN also works)
vibeos-cli swarm whoami / who     # identity / roster
vibeos-cli swarm say "<msg>"      # post chatter to #swarm
vibeos-cli swarm read             # unread messages in full
vibeos-cli swarm search "<q>"     # full-text search Mattermost (what this agent can see)
vibeos-cli swarm drain [--json]   # new high-priority messages (+low count) — hooks/channel use this
vibeos-cli swarm quest list|show|search|claim|done|comment   # Vikunja tasks (task source of truth)
vibeos-cli swarm bridge sync|run  # Vikunja → MM ingestion (per-agent: assigned/comments)
vibeos-cli swarm agent create <name> --lead <lead> [--charter ...]   # provision (registrar)
vibeos-cli swarm agent teardown <name>     # de-provision (also used by tests)
vibeos-cli swarm lead <lead> ; vibeos-cli swarm project create <title>
vibeos-cli swarm register|heartbeat|offline|state busy|idle   # roster/state (hooks)
```

Every command has real `--help`; agents are the primary users. (Agents usually use the
**MCP tools** — `swarm_post`/`swarm_read`/`mm_search`/`quest_*` — rather than the CLI.)

## Development

```
cd agent-swarm
ruff check . && ruff format --check .    # lint + format
pyright                                  # types (basic mode)
python -m pytest                         # unit tests (no network — stubs + temp dirs)
```

## Config & identity

Service credentials load from `~/.vibeos-swarm/config.env` (interim) and the repo `.env`;
canonical keys are `MM_*`, `VIKUNJA_*`, `AUTHENTIK_*` (legacy `MATTERMOST_*` still works).
Identity resolves from `--identity` → `$SWARM_LOGIN` → the session/default written by
`swarm login` — **never from session id or cwd** (those proved unstable). Runtime M2M
creds will be derived from the human lead's OAuth; until that Authentik Application is
live, the shared service tokens are used.

## Channel server (idle-push)

```
cd agent-swarm/channel && npm install          # one-time (node_modules gitignored)
SWARM_LOGIN=<agent> claude --dangerously-load-development-channels server:mattermost-swarm
```
Pushes high-priority `#swarm` chat into a running (even idle) session; lower-priority
traffic arrives as a periodic digest. Requires Anthropic auth (Pro/Max/Console).

## Status

Built + live-tested (2026-06-14): the three service clients, the registrar (provision +
banner + teardown, verified against the live suncluster stack), the CLI, the
priority-tiered drain, and the channel handshake. Pending per the design's build plan:
Go roster plugin, per-agent Vikunja users, full OAuth-derived runtime creds, the unblock
protocol. The legacy `tools/swarm/` is superseded by this — see its README.
