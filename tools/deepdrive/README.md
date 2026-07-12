# deepdrive — drive DeepCode (DeepSeek) as a tool

`deepdrive` lets **any Claude** hand a bounded, repetitive, judgment-requiring task to the cheap
DeepSeek agent (DeepCode) and get back a structured result from **one blocking call** — no polling,
no pane-scraping. It's the **middle tier** between deterministic codemods (sed) and frontier
subagents (Opus): use it for *N repetitive units each needing per-unit judgment with a measurable
convergence signal* — squashing compile errors, idiomizing 100 callsites, migrating trampolines.

Full design + rationale: `docs/engineering/deepdrive-controller-design.md`. **Read that once.**

## The headline use case: compile-grind banked translations

You have a banked translation (Opus or DeepSeek) that doesn't compile yet. deepdrive drives DeepCode
to squash the errors to `cargo check` green, in bounded turns, while **you steer** the systemic ones.
This is the cheap "agentic" half — it is **not** translation. (~100× cheaper than doing the grind on
an Opus subscription; the whole version-rust grind, 62→0 errors, cost cents.)

## Prereqs (one-time, already set up on this box)

- `deepcode` CLI on PATH (`npm i -g @vegamo/deepcode-cli`).
- `~/.deepcode/settings.json` with the DeepSeek key (`env.MODEL=deepseek-v4-pro`, `BASE_URL`,
  `API_KEY` from repo `.env`), `thinkingEnabled`, `reasoningEffort`.
- `tmux`. Run via the repo `.venv` python (deepdrive is stdlib-only, but be consistent).

## The contract

```bash
.venv/bin/python tools/deepdrive/deepdrive.py run \
  --dir <ABSOLUTE working dir = YOUR worktree> \
  --prompt-file <bounded instruction file>  \   # or --prompt "<text>"
  --verify "cargo check -p <crate> --target i686-pc-windows-gnu 2>&1" \  # pass/fail + error_count
  --metric "<cmd whose trailing int = remaining units>" \                # optional convergence signal
  --model deepseek-v4-flash --effort medium \   # optional tier knobs (default: settings = v4-pro/high)
  --timeout 1800
```

Returns JSON (this turn only — usage is per-turn delta, not cumulative):

```json
{ "status":"completed|interrupted|error|timeout", "model":"...", "elapsed_s":N,
  "turn":{ "prompt_tokens":N, "completion_tokens":N, "cached_tokens":N,
           "n_tool_calls":N, "cost_estimate_usd":N },
  "tool_calls":[{"name":"bash","arg":"...","ok":true}],
  "reply":"<final summary>",
  "diff":{ "diff_stat":"...", "status_short":"..." },
  "verify":{ "cmd":"...","exit":N,"error_count":N,"tail":"..." },
  "metric":{ "cmd":"...","value":N,"tail":"..." } }
```

Other subcommands: `status` / `tail` / `interrupt` / `stop` / `sessions` (all `--dir <abs>`).

## The operating model — bounded turns, you steer

1. **Decompose.** Don't say "make it compile." Say "fix only error-class X with rule Y, then stop and
   report." Each turn is short, checkpointed, and you read the **ground truth** (`verify.error_count`,
   `diff`) before the next instruction.
2. **Steer on plateau.** If `error_count` isn't strictly dropping turn-over-turn, the cheap agent is
   stuck on something systemic — *that's your cue to intervene* with a precise global rule (the thing
   DeepCode can't reason out itself). This is the whole point of the tool.
3. **Review the diff every turn.** DeepCode edits your live tree (see Safety).

## Parallel fan-out (safe by construction)

deepdrive isolates **everything by `--dir`** (tmux session, DeepCode project storage, flock). So N
Claudes — or one Claude over N tasks — run concurrently with **one lightweight worktree per task**:

```bash
git worktree add ../wt-foo HEAD     # lightweight: cargo-check grind needs NO build-<arch> dirs
cp .env ../wt-foo/.env              # gitignored key; needed for the DeepSeek backend
# ...translate or use a banked crate, then:
.venv/bin/python tools/deepdrive/deepdrive.py run --dir $(realpath ../wt-foo) ...
```

`--model`/`--effort` are per-session (launch env), so different tasks can run different tiers.
**Same-`--dir` parallelism is NOT safe** (shared session + shared workspace `Cargo.toml` members) —
use a separate worktree per concurrent task. Build dirs are only needed later, for cohab + smoke.

## Safety (read before pointing it at precious work)

- DeepCode edits your **live working tree**. Treat `run` as handing over the keyboard for one turn.
- **Checkpoint first**: `git commit` (or stash) the banked translation before the first grind turn,
  so any over-edit is recoverable. DeepCode also keeps its own `~/.deepcode/projects/<dir>/file-history/.git`.
- Don't point it at a `--dir` with precious *uncommitted* work you haven't checkpointed.
- The returned `diff` + `--verify` are your per-turn guardrails.

## Cost reality

Input caching is automatic and ~99% effective; **cost ≈ tool-call-rounds × cached-context**, so the
lever is *fewer rounds* (prompt "batch your edits, verify once per batch"), not caching. `v4-flash` +
`--effort medium` for the bulk grind; reserve `v4-pro` + `high` for genuinely hard judgment. The
per-turn `cost_estimate_usd` is approximate — the DeepSeek dashboard is truth.

## Grind preset

`tools/deepdrive/grind_prompt.md` is a fill-in-the-blanks bounded-grind instruction (incl. the
**don't-rewrite** discipline for banked translations). Copy it, fill the `<<...>>` slots with your
crate/target and any subsystem-specific rules, and pass via `--prompt-file`.

## Design rationale & economics

The two-tier division of labor — **frontier driver** (Claude/any frontier agent: decompose,
design decisions, steer, review, integrate) + **cheap executor** (DeepSeek via DeepCode: the
high-volume mechanical translation + `cargo check → fix → recheck` grind + boilerplate) — is
validated economically: DeepSeek cost ~$0.20 for a full `version.c` mechanical translation,
several smoke/check drives, *and* a 212k-token integration attempt, because DeepSeek's
server-side prompt caching makes per-turn re-send nearly free (one turn billed 18M prompt
tokens, 18M cached). **Dollar cost is not the binding constraint on the inner loop** — the
scarce resources are wall-clock and the *frontier driver's context/attention*, so the
controller optimizes for those, not for executor tokens. DeepCode = DeepSeek's official
terminal coding agent (`@vegamo/deepcode-cli`); deepdrive invokes it as a **tool any Claude
can call**, not a hand-puppeteered operator session. (Full design narrative:
`docs/engineering/archive/deepdrive-controller-design.md`.)
