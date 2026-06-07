# deepdrive — a Python controller for driving DeepCode (DeepSeek) as a sub-agent

**Status:** design (pre-implementation) · **Date:** 2026-06-07 · **Owner:** rust-port

## 1. Motivation

The Rust translation effort has a recurring tax: LLM translation + the grind of getting
mechanical output to compile. We want a **two-tier division of labor**:

- **Frontier driver (Claude / any frontier agent):** decomposes work, makes the hard
  design-level decisions, steers, reviews, integrates.
- **Cheap executor (DeepSeek via DeepCode):** does the high-volume grind — mechanical
  translation, the `cargo check → fix → recheck` loop, boilerplate.

This is validated economically. In testing, **DeepSeek cost ~$0.20** for a full `version.c`
mechanical translation, several smoke/check drives, *and* a 212k-token integration attempt —
because DeepSeek's server-side prompt caching makes the per-turn re-send nearly free (one turn
billed 18M prompt tokens, 18M of them cached). **Dollar cost is not the binding constraint on
the inner loop.** The scarce resources are (a) wall-clock and (b) the *frontier driver's
context/attention*. The controller must optimize for those, not for DeepSeek tokens.

DeepCode = DeepSeek's official terminal coding agent (`@vegamo/deepcode-cli`), a Claude-Code-like
TUI with its own tools (bash/read/edit/web/MCP). We want to invoke it as a **tool** that *any*
Claude can call — not just hand-puppeteer it in one operator's session.

### 1.1 Positioning — the middle tier (NOT a general solver)

deepdrive is deliberately **not** a do-anything agent. Its niche is the gap between deterministic
codemods and frontier reasoning:

| Tier | Tool | For |
|---|---|---|
| 1 — deterministic | python / sed / codemod | mechanical, **zero** judgment (rename, format, regex-replace) |
| **1.5 — cheap judgment-bulk** | **deepdrive / DeepCode** | **N repetitive units, each needing per-unit judgment, with a measurable convergence signal** — idiomize 100 callsites to `*_rs() -> Result<T,E>`, squash 50 compile errors, migrate trampolines. The frontier driver decomposes + steers the ambiguous units; the cheap executor grinds the rest. |
| 2 — frontier reasoning | Opus agent / subagent | novel, architectural, cross-cutting design |

The "100 callsites → Result idiom" task is canonical tier-1.5: too nuanced for sed (each site
needs judgment), too numerous/cheap to spend a frontier agent per site. The **convergence signal**
generalizes from "compile-error count" to "**remaining units**" (e.g. `grep -c 'old_trampoline('`):
the steering Claude watches the count trend (100 → 60 → 12 → 0) and intervenes when it plateaus.

## 2. Investigation findings (what shaped the design)

We disassembled the DeepCode bundle (`dist/cli.js`) and inspected its on-disk state.

| Finding | Consequence |
|---|---|
| **No headless mode.** `-p`/`--prompt` only pre-fills + auto-submits inside an **Ink (React-for-TTY) app**; it needs a real terminal. No `--print`, no exit-after-turn, no non-interactive branch. | **Input must go through a PTY** → we drive it via `tmux`. |
| **Structured state on disk** under `~/.deepcode/projects/<projectCode>/`, where `projectCode = <abs-project-path with '/' → '-'>`. | **Output + state come from files, not pane scraping.** |
| `sessions-index.json` active entry has `status` (`processing`/`completed`/`interrupted`/`error`), `usage` (tokens incl. `cached_tokens`/`reasoning_tokens`), `toolCalls`, `assistantThinking`, `summary`. | **Reliable turn-state + token accounting** without the `notify` hook. |
| `<sessionId>.jsonl` = full append-live message log (user/assistant/tool, with tool results). | **Structured transcript** — read the tool calls + their outputs (e.g. the cargo error counts DeepCode itself printed). |
| `file-history/.git` per project = DeepCode's own checkpoint git (powers `/undo`). | A built-in revert layer we can lean on. |
| Tools run autonomously by default (no approval prompt observed). | No auto-approve flag needed. |

Earlier we *did* try the `notify` settings hook for completion signaling; it proved unreliable
(didn't fire on a long turn that we later interrupted) and is **superseded** by the `status`
field above.

## 3. Architecture: **tmux-in, files-out**

```
  invoking Claude ──(Bash)──► deepdrive run --dir D --prompt "<step>" --verify "<cmd>"
                                   │
                                   ├─ ensure tmux session for D (start deepcode if absent)
                                   ├─ paste prompt into the TUI (load-buffer/paste-buffer) + Enter
                                   ├─ BLOCK: poll ~/.deepcode/projects/<code(D)>/sessions-index.json
                                   │         until active.status != "processing"
                                   ├─ parse <sessionId>.jsonl for THIS turn's tool calls + results
                                   ├─ run --verify cmd (optional) → error count
                                   ├─ git -C D diff --stat
                                   └─◄ returns ONE structured JSON result
```

The invoking Claude makes a **single blocking call per bounded turn** and never polls. Input is
the only thing that needs the PTY; everything observable comes from structured files.

## 4. Key decisions (with rationale)

- **D1 — Python controller, one blocking call per turn.** The controller absorbs all
  send/wait/parse. Caller-Claude gets request→response semantics. *Why:* makes it usable as a
  tool by arbitrary agents; keeps the driver's context clean.
- **D2 — CLI, not MCP (for now).** Any agent can call it via Bash with zero server setup.
  *Why:* portability for "arbitrary Claudes." MCP wrapper is a later option if we want it typed.
- **D3 — Invoke in the CALLER'S directory; do NOT auto-create a worktree.** The caller passes
  `--dir <abs>`. *Why:* the caller already has a known-good build dir / toolchain / firmware —
  we inherit it instead of reconstructing. This **parks the build-dir-sharing problem entirely**.
  The handoff is synchronous (caller is blocked during the turn), so there's **no concurrent
  mutation** of the shared tree. DeepCode's per-path session storage keeps sessions isolated by
  dir automatically. *Tradeoff:* DeepCode edits the caller's live tree — mitigated by D6/D7 + the
  safety model (§6).
- **D4 — Turn state from `sessions-index.json` `status`; drop the `notify` hook.** *Why:*
  reliable, structured, no race; notify was unreliable.
- **D5 — Input via `tmux load-buffer` + `paste-buffer`, not `send-keys -l`.** *Why:* robust
  against quotes/backticks/newlines/code in steering prompts; no escaping, no premature submit.
  Long prompts may instead be written to a file in `--dir` with the sent line being
  "read FILE and execute it" (avoids any TUI input-size limits).
- **D6 — `--verify "<cmd>"` runs after the turn; its error/exit summary is a result field.**
  *Why:* the **convergence signal** (e.g. cargo error count `62 → 41 → 23`) becomes data the
  steering Claude reads directly — no scraping, trivial plateau/regression detection.
- **D7 — Always return `git -C <dir> diff --stat`.** *Why:* the caller reviews every turn's
  blast radius and owns revert. Keeps the driver in control under D3.
- **D8 — Per-dir `flock` + session reuse; `--new` to reset.** *Why:* two callers can't interleave
  on one DeepCode session; reusing the session preserves DeepCode's conversation context across
  bounded turns; `--new` (`/new`) starts fresh when context should be dropped.
- **D9 — Bounded turns are the operating model.** The driver decomposes into short, verifiable
  steps ("apply rule X to error-class Y, then stop") and inspects ground truth between each.
  *Why:* the 20-min/212k-token runaway happened because of one giant open-ended turn; bounded
  turns keep the driver in the loop and make convergence-guarding trivial (it's just: is the
  `--verify` count strictly dropping turn-over-turn?). Convergence guarding is about wall-clock +
  driver attention, **not** dollar cost (see §1).

## 5. CLI contract

```
deepdrive run --dir <abs> --prompt "<text>" | --prompt-file <f>
              [--verify "<cmd>"]    # pass/fail gate: returns exit + error_count
              [--metric "<cmd>"]    # remaining-units count (trailing int) = convergence signal
              [--model <id>]        # e.g. deepseek-v4-flash; restarts session if changed
              [--effort max|high|medium|low|none]   # reasoning effort (none = no thinking)
              [--timeout <s=1800>] [--new]
   → JSON: {
       "status": "completed|interrupted|error|timeout",
       "session_id": "...", "model": "deepseek-v4-pro", "elapsed_s": N,
       "turn": {            // PER-TURN DELTA (DeepCode's index usage is cumulative; we subtract)
         "prompt_tokens": N, "completion_tokens": N, "cached_tokens": N,
         "n_tool_calls": N, "cost_estimate_usd": N   // ~estimate; dashboard is truth
       },
       "tool_calls": [ { "name":"bash", "arg":"...", "ok":true }, ... ],
       "reply": "<final assistant text>",
       "diff": { "diff_stat": "...", "status_short": "..." },
       "verify": { "cmd":"...", "exit":N, "error_count":N, "tail":"..." },   // if --verify
       "metric": { "cmd":"...", "value":N, "tail":"..." }                    // if --metric
     }

deepdrive status   --dir <abs>     → current session status + usage
deepdrive tail     --dir <abs>     → structured transcript of the last turn
deepdrive interrupt --dir <abs>    → send Esc to the active turn
deepdrive stop     --dir <abs>     → kill the tmux session
deepdrive sessions --dir <abs>     → list sessions-index entries
```

Model/effort override via `systemEnv.MODEL` / `REASONING_EFFORT` / `THINKING_ENABLED` (highest
precedence in DeepCode) set at launch — no settings-file littering. Launch config recorded in
`~/.deepdrive/state/<sess>.cfg`; the session is only restarted when an explicit `--model`/`--effort`
differs from what's running (env is read once, at launch).

## 5a. Cost model & caching (measured)

Validated on the version-rust integration (deepseek-v4-pro, effort high):

- **Caching works: 98.8–99.0% cache-hit** on input. Each tool call is a separate API request
  that re-sends the whole conversation; only the newest tool result is new, so ~99% of every
  re-send is a cache read. DeepSeek caches automatically (no `cache_control` to set).
- **Cost ≈ rounds × cached-context.** The "scary" 8–18M prompt tokens per turn are ~99% cache
  reads. Turn 1 (80 tool-call rounds) was ~2.5× Turn 2 (20 rounds). **The cost lever is round
  count, not caching** — so prompts should say "batch your edits, verify once per batch."
- **~100× cheaper than Opus** for the same agentic grind (that ~11M cache-read volume is ~$17 at
  Opus's $1.50/M cache-read vs ~$0.2 on DeepSeek). The whole 62→0 version-rust drive cost ~$0.2.
- **Reporting:** DeepCode's `sessions-index` usage is **cumulative per session**, so deepdrive
  snapshots pre-turn usage and reports the **per-turn delta** + a `cost_estimate_usd` (approx).
- **Tiers:** `--model deepseek-v4-flash` + `--effort medium/low` for the bulk grind; reserve
  v4-pro + high for genuinely hard judgment. reasoning_tokens (22–28k/turn at high effort) are
  pure output cost.

Session/tmux naming: `deepdrive-<sha1(abs dir)[:12]>` so it's deterministic per dir.
`projectCode` for the storage path is computed as `<abs dir>` with `/`→`-` (matches DeepCode).

## 5b. Trial results & economics (measured)

Two end-to-end trials, deepseek-v4-pro:

| Trial | C LOC | Rust LOC | Translate (DeepSeek actual) | vs Opus API | Grind to cargo-green |
|---|---|---|---|---|---|
| version.dll (1 file) | 1688 | ~2700 | (bundled in early tests) | — | 62→0 errors, 2 deepdrive turns, ~$0.2 |
| mscms.dll (6 files) | 2681 | 4764 | **~$0.31** | $1.34 floor / **$6.69 realistic (~21×)** | (pending) |

Headline: the **translate step** (vs Opus API) runs **~5–28× cheaper** for faithful multi-file
output; the **grind step** (vs an Opus Max subscription agent) converges in a few bounded turns at
cents. Both are parked pre-integration (banked .rs, like the version-rust trial).

**Quality (honest):** the mechanical output post-grind is **correct, compiling, and faithful, but
deliberately un-idiomatic** — raw pointers + `as` casts throughout, declare-then-assign locals,
PascalCase names (→ hundreds of naming warnings), occasional transliteration warts (e.g. an inline
`// comment` swallowing the next `extern` decl) that the `--verify` grind must catch. It reads like
c2rust output. Opus's idiomatic translation lands *closer to integration-ready* (snake_case, some
Result/Option, fewer warnings) for ~20× the cost. The gap is exactly a **pass-2 idiomation** job:
mechanical renames via `cargo fix`, Result/RAII via a judgment-bulk deepdrive pass — so we reach
Opus-level quality without paying Opus for the bulk transliteration. Caveat: the cheap model
produces more warts, so the verify gate matters more than with Opus. (No c2rust is used anywhere —
the c2rust-*style* is purely the mechanical prompt instructing the LLM; verified absent from all
rustconv prompts/code.)

## 6. Safety model

- DeepCode edits the caller's **live working tree** (D3). The contract: `deepdrive run` is like
  handing over the keyboard for one bounded turn.
- Mitigations: (a) `diff_stat` returned every turn (D7); (b) `--verify` gate (D6); (c) bounded
  turns (D9); (d) DeepCode's own `file-history/.git` checkpoints; (e) the caller can `git stash`
  before delegating and review/revert after.
- The caller **must not** point `deepdrive` at a dir with precious uncommitted work it hasn't
  checkpointed. Document this prominently in the tool help.
- Config/keys: `~/.deepcode/settings.json` (user-global, holds the DeepSeek key from `.env`,
  model `deepseek-v4-pro`/`v4-flash`). `deepdrive run` validates config exists and errors clearly.

## 7. Out of scope / parked

- **Auto-worktree creation + build-dir copy-warm** (the old `dc-worktree.sh` path) — superseded
  by D3. The provisioning lessons (toolchain symlink, prefix-safe repoint, firmware) are recorded
  in memory in case we ever want throwaway worktrees for *parallel* DeepCode fan-out.
- **Build-dir sharing (ccache/hardlink/overlay)** — parked; not needed when we invoke in the
  caller's existing dir.
- **Headless DeepCode** — doesn't exist; revisit only if upstream adds it (would let us drop tmux).

## 8. Future / open

- **Parallel fan-out — ALREADY POSSIBLE (no code change).** deepdrive isolates *everything* by
  `--dir`: the tmux session (`sha(dir)`), DeepCode's project storage (`projects/<dir>`), and the
  flock are all per-dir. So translating + grinding two DLLs (e.g. mlang + cabinet) concurrently is
  just: `git worktree add` one **lightweight** worktree each → `rustconv translate` (DeepSeek API,
  trivially parallel, separate output crates) → `deepdrive run --dir <wt>` per worktree, in
  parallel. **The translate+grind loop only uses `cargo check`, which needs NO `build-<arch>` dirs**
  — so these worktrees are cheap (`git worktree add`, ~0 extra disk), and the build-dir-sharing
  quest does NOT block parallel fan-out. (Build dirs are only needed later, for the per-DLL
  cohab + smoke, which can run serially in a build-equipped worktree.) Same-`--dir` parallelism is
  NOT safe (shared session + shared `Cargo.toml` workspace-members → collisions); use separate
  worktrees.
- **MCP wrapper** around the CLI for typed/discoverable tool use.
- **Escalation ladder:** v4-flash (bulk) → v4-pro (stuck) → frontier driver (systemic/design).
  `deepdrive run --model` already enables the model tiers; the ladder is a driver-side policy.
- **Idiomation pass (Stage 3):** the same loop with a "make it idiomatic, keep it compiling"
  prompt and `--verify cargo check` as the gate.

## 9. Implementation plan

1. `tools/deepdrive/deepdrive.py` (single-file CLI; stdlib + the existing `.venv`).
   - `projectdir(dir)`, `session_name(dir)`, config validation.
   - tmux session lifecycle (start deepcode, detect ready, reuse, `--new` via `/new`).
   - input: `load-buffer`/`paste-buffer` + Enter (or prompt-file + "read & execute").
   - wait: poll `sessions-index.json` active entry `status` (with `--timeout`); optional
     `inotifywait` fast-path.
   - parse: last-turn tool calls + reply from `<sessionId>.jsonl`.
   - `--verify` runner + error-count heuristic (count `^error` / `error[` lines).
   - `git diff --stat`, JSON result assembly.
   - per-dir `flock`.
2. Smoke-test the controller itself on a trivial prompt in a scratch dir.
3. **Dogfood:** drive the `version-rust` integration to green in bounded steps, using
   `--verify "cargo check -p version-rust --target i686-pc-windows-gnu"` as the convergence gate.
   This is both the first real use and the acceptance test for `deepdrive`.
```
