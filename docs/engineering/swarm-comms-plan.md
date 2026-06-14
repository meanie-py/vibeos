# Swarm Communication & Coordination — Design Plan

> **⚠️ Superseded (2026-06-14) by [`agent-swarm-platform-design.md`](agent-swarm-platform-design.md).**
> This doc is the messaging-layer subset (board vs channel); the platform design is the
> whole picture (identity/Authentik, quests/Vikunja, unified delivery, provisioning).
> Kept for the transport-mechanism analysis (§4, §7) which the platform design references.

**Status:** design approved, implementation deferred to a fresh (compacted) session.
**Branch/worktree:** `swarm-comms` at `/home/curi/Projects/os-playground/vibeos-swarm-comms`.
**Date:** 2026-06-14.

This started as an elaborate three-layer / four-transport / three-phase design.
We cut it down: it was solving a *board* problem inside the *chat* channel. The
core realization (§3) is the important part; the rest is small.

---

## 1. Problem

Multiple Claude Code (and Codex) agents work vibeos concurrently, **from the main
checkout** (`/home/curi/Projects/os-playground/vibeos`), editing worktrees via
absolute paths — so they share one working tree and one `.git`. Two distinct
coordination needs, with different lifecycles:

- **Ephemeral chatter** — "who left `continue-rust-port` dirty?", "I'm taking the
  fastfat lane", "heads-up, I'm rebasing crp, hands off for 10 min", "`build-amd64/`
  is mid-rebuild". Slack, basically. (We hit this live: crp HEAD advanced under us
  `ba9800e86b0`→`acd6ca9ad34` while another agent committed the cmlib fix; and
  `config.py`/`scanner.py`/`hal-cargo/build.rs` were left uncommitted in the shared
  tree by others.)
- **Durable records** — open bugs (amd64 taskbar `0x50`, cdrom i386 `ud2`), and who
  owns which lane. A bug exists until fixed; a lane is owned until released. These
  are *state*, not messages.

These used to live in agent **memory** (`MEMORY.md`), which is wrong for both:
memory is **local + uncommittable** (no other agent/human/CI/clone sees it) and
**push-context** (loaded every session, taxing every turn).

## 2. Push vs pull (the standing principle)

- **Push** — `AGENTS.md` (+`@`-imports) and `MEMORY.md` load into *every* session.
  Keep tiny and static (stays in the prompt-cache prefix).
- **Pull** — `docs/engineering/*` read on demand. Zero standing cost; can be exhaustive.

Reusable/durable knowledge → repo (pull). Push holds only routing + tiny volatile
state. Already encoded in `AGENTS.md` § "Knowledge Routing" (commit `ba9800e86b0`);
`MEMORY.md` shrunk 24KB→2.6KB the same day.

## 3. The key realization: a board and a channel — not three layers

The earlier draft had three "layers" and a four-transport analysis whose entire
complexity (MCP servers, serialized writes, phased rollout) existed to make the
chat channel **concurrency-safe** — so two agents couldn't both claim the fastfat
lane via a racing append.

But **lane ownership is not chatter — it's an assignment.** Assignment is a *board*
concern, and we already have an atomic board substrate: **git**. Claiming a lane =
pushing a branch / committing on it; that's serialized by git, no clobber possible.
Known defects are the same shape: durable state, not messages.

Once "who owns what" and "what's broken" move to the board, **the chat channel has
no correctness requirement left.** A clobbered append is just a lost Slack message —
repost. So the MCP-for-serialization machinery evaporates, and the channel becomes
a flat file you Read and Edit. Two things, not three; one of them needs no new
mechanism at all:

| Concern | Examples | Lifecycle | Home / mechanism |
|---|---|---|---|
| **Board** (state; needs atomicity) | who owns which lane; open bugs | durable | **git already provides it** — `git branch -a` + commit recency = lanes; `docs/engineering/open-bugs.md` = defects (committed). Future: GitHub issues. **No new mechanism.** |
| **Channel** (messages; atomicity irrelevant) | "crp dirty", "rebasing, hands off", "heads-up" | ephemeral | **`os-playground/SWARM.md`** — Read/Edit on demand at triggers (§5). Flat file, append-mostly, clobbers tolerable. **No MCP, no phases.** |

Why `SWARM.md` lives in `os-playground/` (one dir **above** the repo):
- One shared file all worktrees reach (they're siblings under `os-playground/`;
  agents cwd in main vibeos) — absolute path `/home/curi/Projects/os-playground/SWARM.md`.
- **Never committed** → no history pollution, no merge/sweep hazard, no per-branch
  divergence.

## 4. `SWARM.md` contract (the file's own header)

Hygiene matters because it's read often (and might one day be hook-injected, §7).

> **SWARM.md — live swarm channel (NOT committed; not `@`-imported).** Rolling
> ephemeral notes between concurrent agents. Rules: (1) **Append** a dated +
> agent-id-signed stanza; don't rewrite others' lines (no git merge here —
> last-writer-wins clobbers, so keep appends short and re-post if you get clobbered).
> (2) **Prune** resolved/old entries; keep the file small. (3) **Durable** knowledge
> does NOT live here — route it per AGENTS.md (field manual / audit docs / open-bugs.md);
> lane *ownership* lives in git branches, open *bugs* in open-bugs.md. (4) This is
> *coordination chatter* ("I'm on X", "crp dirty", "heads-up Y"), not a work log,
> not a bug tracker, not a status board.

## 5. `AGENTS.md` protocol text (Phase 0 — the static, cached block)

Short and static (never churns → stays cached). Draft:

> **Swarm coordination.** Multiple agents work this repo concurrently from the main
> checkout. (a) **Live channel (chatter):** `Read`
> `/home/curi/Projects/os-playground/SWARM.md` on demand and append a dated+signed
> stanza to post. **Read it at these triggers:** before committing/pushing to a
> shared branch; before claiming or starting a lane; when you see unexpected dirty
> files or suspect a collision; at session start for a long task. (b) **Lane
> ownership:** `git branch -a` + recent commits — that *is* the board; check before
> claiming. (c) **Known open bugs:** `docs/engineering/open-bugs.md`. Do NOT
> `@`-import SWARM.md (it freezes at session start and thrashes the prompt cache —
> see swarm-comms-plan.md §7).

Also update the routing-table row "An open bug / who-owns-which-lane" from
*(mechanism being designed)* → these three concrete homes.

## 6. `docs/engineering/open-bugs.md` (committed defect registry) — Phase 0

Concise, slow-changing known-defects registry. One stanza per open bug: symptom ·
status · what's-ruled-out · owner/lane · pointer to the detailed hunt-log (the
`project_*` memory topic file or an engineering brief). On fix, the generalizable
root-cause moves to the **field manual** (per routing) and the entry drops off.
Seed:

- **amd64 taskbar BUGCHECK 0x50** (OPEN since 2026-06-11) — amd64-only, non-det
  (~1/3), read-fault POST-`shell ready` via win32k taskbar child-control creation;
  static field-offset RULED OUT (verify_offsets ×3); runtime/logic bug. Possibly
  worked by a Codex agent. Hunt-log: memory `project_amd64_taskbar_bugcheck_0x50.md`.
- **cdrom i386 read-path ud2** (OPEN 2026-06-14) — i386 LLVM InstCombine UB-collapses
  the SRB read chain to `ud2` → BUGCHECK 0x1E at ARC boot read; amd64 fine. Hunt-log:
  memory `project_cdrom_i386_llvm_ud2_readpath.md`.

This is the smallest step toward the real endgame (§8): when crp gets pushed,
GitHub issues supersede this file (assignable, concurrency-safe, queryable).

## 7. Appendix — transports we evaluated (kept so we don't re-derive)

The chosen channel transport is **Read/Edit on demand** (§5). For the record, the
alternatives and why they're not the spine:

- **`@`-import into AGENTS.md — REJECTED.** `@../SWARM.md` resolves, but it's
  **read once at session start, never re-read mid-session** (only `/compact` re-reads
  root CLAUDE.md) → a long agent sees a *frozen* channel; and it sits in the **cached
  prefix**, so every SWARM.md edit **busts the prompt cache** for every new session.
  Worst of both: stale within a session *and* cache-thrashing across them. Never use
  `@`-import for mutable shared state.
- **MCP `swarm_post`/`swarm_read` tools — not needed.** Its one real advantage over
  Read/Edit was *serializing writes* to prevent clobber — but per §3 that's a board
  concern git already owns, so this is pure ergonomics now. Build only if the raw
  Read/Edit ergonomics genuinely annoy.
- **Auto-surface deltas (the one maybe-later worth keeping).** The trigger-list in §5
  already gets an agent to read the channel right before the risky op, so push isn't
  required. *If* "forgot to read" actually bites, a **`PostToolUse` hook** is the
  clean add-on: it injects new lines into the **mutable message stream (NOT the cached
  prefix) → does not bust the cache** (the property `@`-import lacked), keyed off a
  per-agent cursor (offset file by `$CLAUDE_SESSION_ID`), firing after each tool call.
  Gap: no hook fires without a tool call, so a pure-reasoning/idle stretch sees no
  delta. A claimed MCP **"channels"** true-push feature (~100ms, mid-loop) would close
  that gap **but is UNVERIFIED** — the citations smelled possibly-hallucinated and it
  was flagged research-preview, Console-API-key only (not Bedrock/Vertex). Verify it
  exists on our `.env` auth path before depending on it; PostToolUse-hook is the
  no-preview fallback.

If we ever do build the hook + a `/swarm` slash command, bundle them as a **plugin**
(`.claude-plugin/plugin.json` + `hooks/hooks.json` + skills). That's the only thing
that would warrant tooling — and it's optional.

## 8. Endgame / open decisions

1. **Push `continue-rust-port` + use GitHub issues** for durable, concurrency-safe,
   assignable bug/lane tracking — supersedes `open-bugs.md` and removes the
   local-only limitation. Ties to the standing "should we push crp?" question (crp is
   currently local-only, never pushed to `origin` = `meanie-py/vibeos.git`). This is
   the real board; `open-bugs.md` is the interim.
2. **open-bugs.md location** — `docs/engineering/` (recommended) vs a `docs/product/`
   roadmap that *links* to it.
3. **Auto-surface** — only if "forgot to read" bites (§7); verify MCP channels before
   choosing it over the PostToolUse hook.

## 9. First-session checklist (Phase 0 — docs only, no build)

1. Create `/home/curi/Projects/os-playground/SWARM.md` with the §4 contract header +
   one seed stanza (the crp-moved-under-us / dirty-files observation, 2026-06-14).
2. `AGENTS.md`: add the §5 protocol block + update the routing-table row. Commit.
3. `docs/engineering/open-bugs.md`: seed with the two bugs (§6). Commit.
4. `MEMORY.md` stub: drop the interim "Open bugs" section → one-line pointers to
   SWARM.md + open-bugs.md. (memory edit, not a commit)

No build, no smoke (docs only). Push decision (crp) is separate (§8.1). Everything
beyond this — MCP tools, hooks, plugin — is *don't-build-unless-it-bites*.
