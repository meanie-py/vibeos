# Swarm Communication & Coordination — Design Plan

**Status:** design approved, implementation deferred to a fresh (compacted) session.
**Branch/worktree:** `swarm-comms` at `/home/curi/Projects/os-playground/vibeos-swarm-comms`.
**Date:** 2026-06-14.

This document captures the full design + the tradeoffs we evaluated, so the
implementation session has the context without re-deriving it. It is verbose on
purpose — read the "Rejected / considered" sections before changing the design.

---

## 1. Problem

Multiple Claude Code agents (and Codex agents) work on vibeos concurrently. They
operate **from the main checkout** (`/home/curi/Projects/os-playground/vibeos`)
and edit worktrees via absolute paths, so they share one working tree and one
`.git`. They need to coordinate, and today there's no good mechanism:

- **Live operational status** — "who left `continue-rust-port` dirty?", "I'm
  taking the fastfat lane", "heads-up, `build-amd64/` is mid-rebuild, don't
  touch", "I just force-moved crp HEAD". This is *ephemeral chatter* — a Slack
  channel, basically. (We literally hit this mid-session: crp HEAD advanced
  under us from `ba9800e86b0`→`acd6ca9ad34` while another agent committed the
  cmlib hive-write fix; and earlier `config.py`/`scanner.py`/`hal-cargo/build.rs`
  were left uncommitted in the shared tree by other agents.)
- **Durable known defects** — the open bugs (amd64 taskbar `0x50`, cdrom i386
  read-path `ud2`). These are *records*, not chatter: a bug exists until fixed,
  and "we've known about 0x50 since 2026-06-11, field-offset ruled out" is part
  of the project history. Different lifecycle, different audience.

Previously these lived in agent **memory** (`~/.claude/.../memory/MEMORY.md`).
That is wrong for both: memory is **local + uncommittable** (no other machine /
human / CI / fresh clone sees it) and it's **push-context** (loaded into every
session, taxing every agent's every turn). See §2.

## 2. The governing principle: push vs pull context

Two context channels, and conflating them is the root mistake:

- **Push** — `AGENTS.md` (+ its `@`-imports) and the agent's `MEMORY.md` are
  loaded into **every** session for **every** agent. Cost is paid on every turn.
  Must stay tiny and, ideally, **static** (so it stays in the Anthropic
  prompt-cache prefix).
- **Pull** — everything under `docs/engineering/` is read **on demand** via the
  Read tool. Zero standing cost; can be exhaustive.

Therefore: **reusable / durable knowledge goes in the repo (pull); the push
channel holds only routing + truly-volatile-tiny state.** This is already
encoded in `AGENTS.md` § "Knowledge Routing — Where Things Go" (added
2026-06-14, commit `ba9800e86b0`), and `MEMORY.md` was shrunk 24KB→2.6KB to a
stub the same day.

Swarm coordination is a *third* thing that doesn't fit cleanly in either: it's
**live + mutable + shared**. The whole design below is about giving it the right
home without polluting the static push prefix or the durable repo record.

## 3. The three layers (keep them separate)

| Layer | Lifecycle | Home | Mechanism |
|---|---|---|---|
| **Active lanes** ("who's working on what") | ephemeral | **git** | `git branch -a` + commit recency *is* the board (wt-netstack, fastfat-port, uniata-port, kernel32-port…). Convention: check it before claiming a lane. **No new artifact.** |
| **Live channel** ("notes to each other right now") | ephemeral | **`os-playground/SWARM.md`** (outside the repo, above all worktrees) | pull (read-on-demand) now; MCP tool + push-hook later. See §4–5. |
| **Known defects** (open bugs) | durable | **`docs/engineering/open-bugs.md`** (committed) | pull (read-on-demand); concise registry, feeds the field manual on fix. See §6. |

Why `SWARM.md` lives in `os-playground/` (one dir **above** the repo):
- It's **one shared file** all worktrees/agents reach via `../SWARM.md` (they're
  siblings under `os-playground/`, and agents cwd in main vibeos).
- **Never committed** → no git-history pollution with chatter, no merge/sweep
  hazard, no per-branch divergence.

## 4. Mechanism analysis — how agents see/update SWARM.md (THE crux)

We evaluated four transports. **Verified facts** (via Claude Code docs, 2026-06-14):

### 4a. `@`-import into AGENTS.md — REJECTED
`@../SWARM.md` *does* resolve (relative imports may escape the repo root). But:
- **Read once at session start, baked in — never re-read mid-session** (only
  `/compact` re-reads project-root CLAUDE.md). So a long-running agent would see
  a **frozen** channel — useless for live coordination.
- It sits in the **cached prompt prefix**, so every SWARM.md change **invalidates
  the cache** → every new agent session eats a full-prefix cache miss
  (re-hash + re-transmit of the whole AGENTS.md+import). Constant tax across a
  busy swarm.
- No live-reload exists (`/memory` only views/edits; doesn't re-inject).
→ `@`-import gives the **worst of both**: stale within a session *and*
cache-thrashing across sessions. Do not use it for mutable shared state.

### 4b. Read-on-demand (Read tool) — PHASE 0 (zero build)
AGENTS.md holds a static *protocol pointer* (cached, never churns); agents
`Read` `SWARM.md` at trigger points and append via `Edit` to post.
- ✅ Always fresh (Read hits current disk), ✅ no cache impact (not in prefix),
  ✅ zero build.
- ⚠️ Relies on the agent remembering to read at the right moments (mitigated by
  an explicit trigger list, §7).
- ⚠️ TOCTOU on raw `Edit`: two agents append "fastfat unclaimed"→both claim, or
  concurrent edits clobber (last-writer-wins, no git merge). Rare + recoverable
  for a human-paced swarm; the MCP tool (4c) fixes it.

### 4c. MCP custom tools `swarm_post()` / `swarm_read()` — PHASE 1
An MCP server exposes first-class tools (like the built-in memory tools).
- ✅ Cleaner than raw Read/Edit; the agent calls `swarm_post("…")` /
  `swarm_read()`.
- ✅ **Single writer → serializes appends**, which *fixes the TOCTOU clobber*
  from 4b. This is a correctness win, not just ergonomics.
- Still **pull** (agent must call `swarm_read` to see new messages) → doesn't
  auto-surface. Combine with 4d for push.

### 4d. Auto-surface deltas — PHASE 2
Two options to push *new* messages at the agent without it polling:
- **`PostToolUse` hook** (confirmed). Three hook events can inject context:
  `SessionStart`, `UserPromptSubmit`, `PostToolUse`. Key properties:
  - Injected text lands in the **mutable message stream, NOT the cached prefix**
    → **does not bust the prompt cache.** (This is the property `@`-import lacked.)
  - A hook script keeps a **per-agent cursor** (offset file keyed by session id,
    e.g. `$CLAUDE_SESSION_ID`) and emits only lines newer than this agent saw.
  - `PostToolUse` fires after **every tool completion** → in a working coding
    loop that's ~every step, so deltas surface promptly.
  - **Gap (honest):** no hook fires *without* a tool call, so a long pure-reasoning
    stretch or the final text-only turn gets no delta until the next tool call.
    Acceptable for busy agents; a blind spot for stuck/idle ones.
- **MCP "channels" push** (UNVERIFIED — verify before depending). The
  claude-code-guide agent claimed an MCP "channels" research-preview feature
  where the server *pushes* events into a running session (~100ms) even mid-loop,
  closing the gap above. **Treat as plausible-but-unconfirmed**: the specific
  citations (a `--channels` flag, `channels-reference.md`, a "fakechat/Telegram"
  example plugin) smelled possibly-hallucinated, and it was flagged
  research-preview limited to Console-API-key / Team-Enterprise (NOT Bedrock /
  Vertex). **Action for implementer: verify it exists and works on our auth path
  (the `.env` API keys) before building on it.** PostToolUse-hook is the
  no-preview fallback.

### Summary table
| Transport | Fresh? | Cache-safe? | Auto-surface? | Concurrency-safe? | Build cost |
|---|---|---|---|---|---|
| `@`-import (4a) | ❌ frozen | ❌ busts prefix | ❌ | n/a | none — **rejected** |
| Read-on-demand (4b) | ✅ | ✅ | ❌ (must read) | ⚠️ raw Edit clobber | none — **Phase 0** |
| MCP tools (4c) | ✅ | ✅ | ❌ (must call) | ✅ serialized writes | small — **Phase 1** |
| PostToolUse hook (4d) | ✅ | ✅ | ✅ (per tool step) | n/a (read path) | small — **Phase 2** |
| MCP channels (4d) | ✅ | ✅ | ✅ (true push) | n/a | medium + **unverified** |

## 5. Phased rollout (earn each phase; protocol stable across transports)

The *protocol* the agent follows — "post a dated stanza; read the channel before
risky ops" — is identical regardless of transport, so nothing is wasted as we
swap the transport underneath.

- **Phase 0 (zero build, do first):** create `os-playground/SWARM.md` (contract
  header, §8) + add the read-on-demand **protocol** to `AGENTS.md` (§7) + create
  committed `docs/engineering/open-bugs.md` (§6). Update the AGENTS.md routing
  table's "swarm coordination" row to point at these. **This tells us whether the
  swarm even adopts a channel** before we build tooling.
- **Phase 1 (if adopted):** MCP server with `swarm_post`/`swarm_read` (serializes
  writes). Same protocol, nicer + correct transport.
- **Phase 2 (if "remember to read" bites):** `PostToolUse` delta hook
  (auto-surface, cache-safe, per-agent cursor) — or MCP channels *if verified*.
  Bundle hooks + MCP + a `/swarm` slash command as a **plugin** at this point
  (`.claude-plugin/plugin.json` + `.mcp.json` + `hooks/hooks.json` + skills).

## 6. `docs/engineering/open-bugs.md` (committed registry) — Phase 0

Concise, slow-changing **known-defects** registry. One stanza per open bug:
symptom · status · what's-ruled-out · owner/lane · pointer to the detailed
hunt-log (the `project_*` memory topic file or an engineering brief). On fix, the
generalizable root-cause moves to the **field manual** (per routing) and the
entry drops off. Seed with:
- **amd64 taskbar BUGCHECK 0x50** (OPEN since 2026-06-11) — amd64-only, non-det
  (~1/3), read-fault POST-`shell ready` via win32k taskbar child-control
  creation; static field-offset RULED OUT (verify_offsets ×3); runtime/logic bug.
  Possibly worked by a Codex agent. Hunt-log: memory
  `project_amd64_taskbar_bugcheck_0x50.md`.
- **cdrom i386 read-path ud2** (OPEN 2026-06-14) — i386 LLVM InstCombine
  UB-collapses the SRB read chain to `ud2` → BUGCHECK 0x1E at ARC boot read;
  amd64 fine. Hunt-log: memory `project_cdrom_i386_llvm_ud2_readpath.md`.

(Registry location decision: `docs/engineering/open-bugs.md`, since these are
engineering defects; a roadmap doc under `docs/product/` may *link* to it. Not
finalized — implementer may relocate.)

## 7. `AGENTS.md` protocol text to add (Phase 0)

A short, **static** block (it stays cached — no churn). Draft:

> **Swarm coordination.** Multiple agents work this repo concurrently from the
> main checkout. (a) **Live channel:** `Read` `/home/curi/Projects/os-playground/SWARM.md`
> on demand and append a dated+signed stanza when you post. **Read it at these
> triggers:** before committing/pushing to a shared branch; before claiming or
> starting a lane; when you see unexpected dirty files or suspect a collision;
> at session start for a long task. (b) **Known open bugs:** `docs/engineering/open-bugs.md`.
> (c) **Active lanes:** `git branch -a` + recent commits — check before claiming.
> Do NOT `@`-import SWARM.md (it would freeze at session start and thrash the
> prompt cache — see docs/engineering/swarm-comms-plan.md §4a).

Also update the routing-table row "An open bug / who-owns-which-lane" from
*(mechanism being designed)* → the three concrete homes above.

## 8. `SWARM.md` contract (header of the file itself) — Phase 0

The file's own header must enforce hygiene, because once Phase 2 hooks inject it
(or agents read it often), bloat = cost:

> **SWARM.md — live swarm channel (NOT committed; not `@`-imported).** Rolling
> ephemeral notes between concurrent agents. Rules: (1) **Append** a dated +
> agent-id-signed stanza; don't rewrite others' (no git merge here —
> last-writer-wins clobbers). (2) **Prune** resolved/old entries; keep the file
> small (it's read often / may be hook-injected). (3) **Durable** knowledge does
> NOT live here — route it per AGENTS.md (field manual / audit docs / open-bugs.md).
> (4) This is for *coordination* ("I'm on X", "crp dirty", "heads-up Y"), not a
> work log.

## 9. Open decisions for the implementation session

1. **open-bugs.md location** — `docs/engineering/` (recommended) vs `docs/product/`
   roadmap. (§6)
2. **MCP "channels" availability** — verify the push feature exists on our auth
   path before Phase 2 depends on it; otherwise PostToolUse hook. (§4d)
3. **How far to build now** — Phase 0 is the agreed immediate step; Phases 1–2
   are conditional on adoption / the read-reliability gap actually biting.
4. **Eventual upgrade: push `continue-rust-port` + GitHub issues** for durable,
   concurrency-safe, assignable bug/lane tracking — supersedes open-bugs.md and
   removes the local-only limitation. Ties to the standing "should we push crp?"
   question (crp is currently local-only, never pushed to `origin`
   = `meanie-py/vibeos.git`).

## 10. Concrete first-session checklist (Phase 0)

1. `os-playground/SWARM.md` with the §8 contract header + one seed stanza (e.g.
   the crp-moved-under-us / dirty-files observation from 2026-06-14).
2. `AGENTS.md`: add the §7 protocol block + update the routing-table row. Commit.
3. `docs/engineering/open-bugs.md`: seed with the two bugs (§6). Commit.
4. `MEMORY.md` stub: drop the interim "Open bugs" section, replace with one-line
   pointers to SWARM.md + open-bugs.md. (memory edit, not a commit)
5. Smoke-free (docs only) — no build needed. Push decision (crp) is separate.
