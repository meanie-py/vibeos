# Agent Swarm Platform — Design & Build Plan

**Status:** design → implementation (2026-06-14). Will refine during the build.
**Absorbs:** the former `swarm-comms-plan.md` (messaging-layer subset); its transport
analysis is now Appendix A. This is the single cohesive design.
**Audience:** the implementation session(s), and the agents themselves.

> Build everything here as **professional, modular code**, not glue scripts. A **core
> library** with **thin, interchangeable CLI and MCP front-ends** over it; **self-
> documenting `--help`** (agents are the primary users); a shared `send` primitive
> reused by every component.

---

## 1. Goal

Enable a swarm of Claude Code (and Codex) agents — **cross-machine, not always
online** — to collaboratively port vibeos, **coordinating without the human as the
router**. The human is escalation, not dispatch.

The bar is **enablement, not observability**: the platform earns its keep only when
it closes coordination *loops that complete autonomously* — an agent gets unblocked,
claims work, hands off, and stays on-mandate without a human in the middle.

## 2. Principles

- **Push vs pull.** `AGENTS.md`/`MEMORY.md` load every session (push — keep tiny,
  static, cache-friendly). `docs/engineering/*` is pull (read on demand, exhaustive).
  Coordination is a *third* thing — live, mutable, shared — and lives on **servers**,
  not in push-context or local files.
- **Four distinct lifetimes** (§3). Conflating them caused every identity bug we hit.
- **Server-side state.** Agents may be on different machines, so anything durable
  lives in Authentik / Mattermost / Vikunja. The local footprint is a **stateless
  client** (hooks + MCP channel + CLI + local memory).
- **Identity comes from login, never cwd or session id.** (`--by-cwd` and worktree-
  keying are retired; see §3, §6.)
- **Secrets are ephemeral and human-gated.** No long-lived agent credentials stored
  locally; derived per-session from the human's OAuth (§6).
- **Curate the inbound diet.** An agent is enabled by the right signal at the right
  moment and disabled by noise — priority tiering is itself enablement.

## 3. The four-layer model

The clarifying move. Four things we had smeared together, each with its own lifetime:

| Layer | Lifetime | Home | What it is |
|---|---|---|---|
| **Identity** (the worker) | durable | Authentik service account ↔ Mattermost bot ↔ Vikunja user ↔ roster | a persistent agent: `user32-dll` |
| **Project / epic** (the work) | durable, outlives any agent | Vikunja project + its channel | a body of work: "port fastfat", "get doom running" |
| **Workspace** | per active quest | a git branch (worktrees no longer special) | where the work is done |
| **Session** | ephemeral | one `claude` run | the current incarnation |

Consequences: identity survives restarts (it's Authentik, not a session id); a project
has **many agents** and an agent works **many projects** (not 1:1); the **workspace is
just a branch** — worktrees are no longer special, and we move to a normal
`master`-style branch model (retiring `continue-rust-port`). "Same agent" means *same
Authentik identity*, regardless of machine, branch, or session.

## 4. Systems & topology

```
Authentik (auth.999999999.loan) ── identity / SSO / M2M issuer
   │  human OAuth login; agents = service accounts (M2M only); groups: vibeos-swarm
   │  → subgroup per human lead → agent service accounts (owner=<lead>)
Mattermost (chat.999999999.loan) ── the nervous system
   │  messaging + per-agent bots + channels + (Go plugin) roster KV
Vikunja (board.999999999.loan) ── task source of truth
   │  epics(projects) → quests(tasks) → subtasks; ownership + status
Repo (vibeos/agent-swarm/) ── the swarm code, registrar, Go plugin, deploy, hooks
Local (per machine) ── stateless client: hooks + MCP channel server + CLI + memory
```

Mattermost is the **hub/nervous system**: identity (bots), messaging, presence
(roster plugin), and notification delivery all converge there. Authentik owns
*identity*; Vikunja owns *tasks*; the repo owns *code + charters*.

### Channel topology
- **`#swarm`** — full-swarm chatter: requests-for-response, warnings, `@agent` pings.
- **`#announcements`** — **merges** ("about to / merged to `master`") and **review
  requests**. (Not build status — too noisy/local.)
- **Per-agent channel** — the agent's inbox: Vikunja notifications bridged here, and
  the **pinned banner** (its mandate/context — agent- and human-editable). This is the
  situation-report source (§9).
- **Per-project channel** — mirrors the project's Vikunja board, bridged by a **dumb
  bot** (Vikunja can't webhook to private IPs; a suncluster poller reads the board
  every ~15s and posts changes). The hub processes these like any other message.

## 5. Work model

- **Epic** = a Vikunja project, subsystem-scale and durable.
- **Quest** = a Vikunja task, **large** ("port fastfat", "get doom running") — bigger
  than a normal engineering ticket. The agent **self-divides** the work, **commits
  freely**, and may add **subtasks/comments** at its own judgment.
- **Vikunja is the source of truth** for who owns a quest and its status. Claiming =
  Vikunja assignment (atomic; two agents can't grab the same quest).
- Many agents per epic; an agent may hold quests across epics.

## 6. Identity & auth (two tiers)

Agents are **Authentik service accounts** — no interactive password, **M2M-only** by
construction. The hierarchy is `vibeos-swarm` (group) → a **subgroup per human lead**
(the lead *owns* its agents) → agent service accounts (`owner=<lead>`).

- **Provisioning (privileged, human/lead).** The **registrar** (we build it; lives in
  `agent-swarm/registrar/`) fans one identity across services using the
  `*_REGISTRAR_TOKEN`s:
  ```
  register_swarm(lead)        → Authentik: lead user + subgroup under vibeos-swarm
  register_agent(name, lead)  → Authentik: service account (owner=lead, M2M)
                              → Mattermost: bot + token
                              → Vikunja: user + token
                              → Mattermost: per-agent channel + pinned banner
  project_create(epic)        → Vikunja project + per-project channel + bridge
  ```
  Idempotent (check-then-create) and resumable — a half-provisioned agent re-runs
  cleanly. Per-service identities link to the Authentik service-account of record
  (same username everywhere + `owner` attribute).

- **Runtime (per session).** `/swarm-login <agent>` rides the **human's OAuth
  session** — the lead is group controller over its agents — to obtain the agent's
  scoped M2M credentials into **session memory only**. **Nothing agent-level is
  persisted locally.** Whatever the token lifetime, **every token resolves to the
  persistent Authentik identity** (the binding is the invariant, not the token).

- **Interim.** Today's long-lived tokens in `~/.vibeos-swarm/config.env`
  (Mattermost bot/registrar, Vikunja) are a bridge until login-derived creds land.

## 7. Coordination loops (the enablement core)

1. **The unblock loop — the crown jewel.** Agent A blocked on B's lane →
   - **tracks**: a Vikunja task assigned to **B** (durable, the SoT for "B owes A X"),
   - **wakes**: an `@B` ping in `#swarm` → delivered to B even if idle (MCP channel),
   - **resolves**: B acts; the task closes; A is woken by the reply,
   - **escalates**: B offline / no ack in N minutes → re-route or `@human`.
   Two otherwise-idle agents resolve a cross-lane dependency in near-real-time, no
   human. This is the highest-value use of the whole platform.
2. **Merge / review announcements.** About-to-merge and merged-to-`master` events, and
   review requests, go to `#announcements` — the genuine cross-swarm integration
   signals (collision + integration awareness).
3. **Queryable state.** Facts come **deterministically** (roster → owner, Vikunja →
   status, `git` → branches): `swarm who/owns/status`. Judgment comes
   **conversationally** (ask `#swarm`, the agent who knows answers — loop 1).

## 8. Notification delivery (unified queue)

One pending-queue per **identity**, one delivered-cursor, dedup — transports are
interchangeable drains, chosen by the agent's busy/idle state:

- **Busy** (mid-turn) → drain via **hook `additionalContext`** (lands on the current/
  next turn). **Idle** → drain via the **MCP channel** (the only path into an idle
  session).
- **State machine** (no native busy/idle signal exists — verified). Derived from
  hooks writing `~/.vibeos-swarm/state/<identity>.state` = `{state, pid, ts}`:
  - `UserPromptSubmit` → `busy`; `Stop` → `idle`; `Notification:idle_prompt` confirms
    idle-entered.
  - **PID liveness:** the state records the owning process; at evaluation, **if the PID
    is dead, auto-flip to `idle` (and roster `offline`)** — robust crash/close catch.
- **Priority tiers** (already built): **high → immediate** (`@me`, `@here/@channel`,
  or high board priority — Vikunja High/Urgent/DO NOW); **low → batched** into one
  digest every ~180s. Board "DO NOW" wakes immediately; chatter batches.

## 9. Situation report (SessionStart)

Assembled from durable sources, not a session-local guess:
- the **pinned banner** in the agent's channel (mandate / track context),
- recent **`@`s in that channel** (Vikunja assignments/updates, bridged),
- the **projects/quests** it's on (active + blocked-on-me),
- **local memory** for the ephemeral/tactical.

A relaunched agent boots into a real situation report — durable mind on top, ephemeral
scratch underneath.

## 10. Roster & presence

Because agents are cross-machine, the **roster of record is the Go Mattermost plugin**
(server-side KV) — *not* local SQLite. Fields: identity, lead, current project/quest,
status (online/idle/offline), last_seen, machine/pid. Liveness = heartbeat + the §8
state machine (PID-death → offline). Local SQLite demotes to an offline cache, or
drops. **The Go plugin is a spike** (it's a new Go toolchain for the repo); the roster
abstracts behind an interface so the client doesn't care whether it's local or plugin.

## 11. CLI & code structure

Professional, modular — agents are the primary users.

- **`core`** library: identity/login, roster, Mattermost client, Vikunja client,
  pending-queue, the shared **`send`** primitive (post to channel X as identity Y;
  env-powered; reused by the bridge bot, merge announcer, etc.).
- **CLI front-end** over `core` with thorough, self-documenting `--help`:
  `login`, `whoami`, `send`, `say`/`read`, `who`/`owns`/`status`,
  `quest list/claim/status/comment/subtask`, plus provisioning that shells to the
  registrar. (`swarm board`/`quest` give the Vikunja capability — Vikunja has no MCP.)
- **MCP front-end** over the same `core` (the channel server + first-class tools like
  `swarm_reply`, `quest_*`), so CLI and MCP are interchangeable views of one core.

## 12. Repo layout (`agent-swarm/`, top-level)

```
agent-swarm/
  core/         # the library (identity, roster, MM/Vikunja clients, queue, send)
  cli/          # thin CLI front-end + self-documenting help
  channel/      # MCP channel server (delivery + reply/quest tools)
  mattermost/   # Go roster plugin + mmctl deploy script
  registrar/    # provisioning fan-out (Authentik/Mattermost/Vikunja), idempotent
  hooks/        # SessionStart/PostToolUse/Stop/SessionEnd/UserPromptSubmit/Notification
```
Charters live as the **per-agent channel pinned banner** (SoT); an optional repo mirror
under `docs/engineering/tracks/` is fine but not authoritative.

## 13. Hooks

| Hook | Job |
|---|---|
| `SessionStart` | login (if `SWARM_LOGIN`/identity set) + emit the situation report (§9) |
| `UserPromptSubmit` | state → `busy` |
| `PostToolUse` | drain the pending-queue when busy; refresh liveness/`touch` |
| `Stop` | state → `idle` |
| `Notification:idle_prompt` | confirm idle-entered |
| `SessionEnd` | mark `offline` |

All hooks are defensive (exit 0 on any error — a coordination hiccup must never break a
session). The MCP channel handles idle delivery; the hooks handle busy delivery + state.

## 14. What exists today → migration

The original prototype (a SQLite registry + CLI, the MCP channel server, and the hook
set) lived under `tools/swarm/`. It has since been reorganized into `agent-swarm/`
(core + CLI/MCP front-ends, login-based identity) and **removed**. Remaining migration:
- **Retire** `--by-cwd`, worktree-keyed identity, and session-id resolution in favor of
  **login-based identity**. *(done — identity resolves from `--identity`/`$SWARM_LOGIN`/
  login file.)*
- **Promote** roster from local SQLite → Go plugin (cross-machine).
- **Swap** interim `config.env` tokens → login-derived M2M creds.

## 15. Sequenced build plan

Ordered by value-now; each phase shippable.

1. **Reorg + core refactor + login identity.** `agent-swarm/` layout; `core` lib;
   CLI/MCP over it; `/swarm-login` sets identity (drop `--by-cwd`).
2. **Provisioning registrar.** `register_swarm`/`register_agent`/`project_create`
   fan-out across Authentik/Mattermost/Vikunja (idempotent); per-agent channel + banner.
3. **Vikunja quest integration.** `quest *` in the CLI; per-project channel + dumb
   bridge; situation report reads banner + routed `@`s.
4. **Unified delivery + state machine.** One queue/cursor/dedup; busy/idle state file
   with PID-liveness; busy→hook, idle→channel.
5. **Go roster plugin (spike).** Server-side KV roster; deploy via `mmctl`; client
   behind the roster interface.
6. **Coordination loops.** The unblock protocol (Vikunja task + `@`-wake + escalation);
   merge/review announcements; `who/owns/status` queries.

## 16. Security

- **No agent secrets stored locally** — runtime creds derived from the human's OAuth,
  held in session memory; registrar tokens stay privileged/server-side.
- **Channel input is model input** — `#swarm` is internal, but gate against prompt
  injection (sender/channel scoping); push only what's needed.
- **No raw-log exfil** — transcripts stay local; pull on-demand for debugging only
  (they contain code + secrets). Coordination rides curated signals, not firehoses.

## 17. To verify during build (not design gaps)

- `Notification:idle_prompt` exact timing — measure it; the `Stop`-bracket is the safe
  fallback for the idle-drain regardless.
- Per-agent **Vikunja users** (so assignees name a specific agent) vs the interim
  shared board token — mint per-agent users via the registrar; until then, ownership
  maps Mattermost-side.

---

## Appendix A — Transport mechanism analysis (why MCP channel + hooks)

How to get a message into a *running* agent. Four mechanisms evaluated; verified
against Claude Code docs (2026-06):

| Transport | Fresh? | Cache-safe? | Reaches idle? | Verdict |
|---|---|---|---|---|
| **`@`-import** (`@file` in AGENTS.md) | ❌ frozen at session start | ❌ busts the cached prefix on every edit | ❌ | **rejected** — worst of both |
| **Read-on-demand** (Read at triggers) | ✅ | ✅ | ❌ (must remember) | fallback only |
| **MCP tools** (`swarm_read`/`say`) | ✅ | ✅ | ❌ (must call) | pull path; serializes writes |
| **PostToolUse hook → `additionalContext`** | ✅ | ✅ (mutable stream, not prefix) | ❌ (needs a tool call) | **busy-path delivery** |
| **MCP channel** (`notifications/claude/channel`) | ✅ | ✅ | ✅ **true push** | **idle-path delivery** |

`@`-import is read once at session start (frozen mid-session) *and* sits in the cached
prompt prefix, so any change forces a full-prefix cache miss — never use it for mutable
shared state. The MCP **channel** push into an idle session is **confirmed real and
implemented** (research preview; requires Anthropic auth — Pro/Max/Console, not Bedrock/
Vertex). Conclusion → the unified delivery (§8): **hooks drain when busy, the MCP
channel drains when idle**, one queue/cursor behind both.

Detailed walkthrough of each (and the cache-prefix reasoning) is preserved in git
history at `swarm-comms-plan.md` before its removal.
