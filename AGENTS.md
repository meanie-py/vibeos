# AGENTS.md

Guidance for AI coding agents working in this repo (Claude Code, Codex CLI, Gemini — they all
load this file).

**This file is the PUSH channel: it is loaded into every session, for every agent, before
anyone knows what the task is.** So it holds only what changes *behaviour* — the goal, how to
work, and the rules whose violation has actually cost us money or a boot. Commands, paths,
and platform details are PULL: `docs/engineering/agent-reference.md` and the docs it links.
Keep this file small; when you learn something durable, route it (see Knowledge Routing).

Rules here that read as absolutes are absolutes. Nearly every one is the scar of a specific
incident, and the incident is named so you can judge how it applies to a new situation.

---

## What this project is

A multi-architecture port of **ReactOS** (open-source Windows-compatible OS), forked from
`github.com/reactos/reactos` to add ARM64. It cross-compiles to **ARM64, amd64, and i386**
PE-COFF with zero build failures, and all three boot to a graphical desktop in QEMU.

Mainline is **`master`**. Work happens on short-lived feature branches — often in git
worktrees so several agents run in parallel — and lands on `master` by ordinary `git merge`.

## The goal is Rust, not a green build

**The project's goal is to translate ntoskrnl (and the rest of the OS) to Rust.** Getting the
binary to build and boot is necessary but it is not the goal. Translation progress (lines of
C → Rust) and boot status are independent axes.

Translated Rust is *paid artifact*. Protect it:

- **Never revert a Rust translation to C** to make a build link or a smoke test pass. Keep the
  Rust file and fix it.
- **Never delete a translated `.rs`** without explicit user approval. Even broken output has
  value as a starting point, and regenerating costs a real API call.
- **Never re-translate a file that already has output** (in `.responses/` or integrated)
  without explicit user approval — you would be re-paying for work that exists.
- **Never hide a translation from the build to dodge a regression.** Commenting out `pub mod X;`
  so smoke passes while the C version is enabled is the same anti-pattern as reverting: it
  hides the bug from the smoke harness, which is *the diagnostic tool*.

**A broken Rust translation, integrated and failing, is the correct state** when a regression
is unsolved. Fix it in place, or leave it visibly failing and ask how to proceed. These rules
outrank the smoke-test imperative below when they conflict: a red smoke that points at a real
translation bug beats a green smoke that hides it.

**Debug the Rust, not the C — check for a Rust port FIRST.** We are far more translated than
agents assume (~100% of the boot path; see `docs/engineering/rust-port-status.md`). Most
subsystem `.c` files are cohab-flipped: commented out in CMake, with `#[no_mangle]` Rust taking
over, leaving **inert dead source on disk** where a `DbgPrint` fires never. Grep
`sdk/lib/rust/` for the symbol before touching any `.c`.

---

## How to work here

**Scope.** Deliver what was asked, at the scope intended. Make routine judgment calls
yourself, and check in only when different readings of the request would lead to materially
different work. If the request seems mistaken or a better approach exists, say so in a
sentence and continue with the task as asked rather than quietly narrowing, widening, or
transforming it. Finish the whole task.

**Take ownership; power through.** When you find a problem in your area — or anywhere you can
reach — fix it. The default is *do it*, not "want me to file a task?" Reserve true handoffs
for work owned by another lane that you genuinely cannot do.

**A wrapper script is a bug report against the tool.** If you catch yourself hand-rolling a
shell wrapper, or *remembering a recipe*, to work around how a tool behaves — that is the
detection event for a missing mechanism, and the fix is to change the tool, not to remember
harder. Two from one session: every merge-queue drain went through a hand-written
`nohup setsid drain.sh` because `land` blocked (now it conducts detached), and "never broad-kill
QEMUs" had no safe command for months so agents kept reaching for `pkill` (now `vibeos-cli
kill`). This is the rule whose job is to **delete other rules**: a rule in this file is a
*deferred mechanization*, it costs context in every session for every agent forever, and each
one is the scar of an incident that a tool could have made impossible. See
`e5f532ad1aa` — "the footgun program" — for the precedent.

**Don't idle waiting on another agent.** "Waiting on X's fix" is not a status; "working Y while
X lands" is. If blocked, do another part of the task, route around the blocker (local
shim/stub/cohab/alternate path), or just fix the blocker yourself — it is all one repo, you can
read and patch another lane's code and hand them a tested diff. Only block when you cannot make
progress on *anything*, and then say exactly what you need.

**Persistence.** 100+ rustc errors usually stem from a few roots: fix the first root, re-count,
don't bail at the big number. Kernel hangs and BUGCHECKs are ~1 root cause; keep going until
found. **Don't invent time or session budgets.**

**Deferral is temporary, never silent.** Deferred work stays tracked and owed. Don't drop a
thing permanently by calling it "out of scope."

**Fail loud, never silently degrade.** When an explicitly-requested step can't do its job, HARD
FAIL. Never quietly fall back to a worse path and proceed as if nothing happened — a "dumb
fallback" (c2rust fails → silent pure-LLM translation; a missing tool → skip its check; an
unverifiable assumption → guess) produces subtly-wrong output that costs far more to diagnose
than an upfront error. Surface it with an actionable message and stop. (Concrete:
`rustconv translate --c2rust` aborts with `C2RustError` rather than degrading to an unanchored
translation.)

**Verification.** Verify your own work as you go; you don't need to be told to, and you don't
need a separate "final verification pass" or a subagent to double-check you. *But* the gates
below (`vibeos-cli check`, `check-tooling`, smoke ×3) are **real external CI, not model
self-verification** — they catch what self-review structurally cannot, so run them before
landing and don't strip them as redundant.

**Subagents.** Delegate only for large, genuinely independent, parallelizable tracks — a wide
multi-file investigation, or one module per agent on a big translation grind. Don't delegate
work you can finish in a handful of tool calls, don't spawn subagents to verify your own work,
and prefer one agent over several. Two repo-specific constraints, both from real breakage:
centralize the systemic roots *first*, and have agents write only to their own new files
(`.rs` into `~/vibeos-rust-staging/`) — **you** do all shared-file edits (`lib.rs`,
`CMakeLists.txt`), the build, and the smoke runs, centrally.

**Communication.** Be context-frugal but not anxious: batch independent tool calls, background
long builds, keep status lines terse, skip recaps. It's a 1M-context model — don't
over-checkpoint. When you finish, lead with the outcome: the first sentence should answer "what
happened" or "what did you find." Report faithfully — if tests fail, say so with the output; if
a step was skipped, say that.

**Written deliverables.** Match the length of docs you write to what the task needs: cover the
substance, don't pad with filler sections, redundant summaries, or boilerplate. (This repo has
already had one "brutal consolidation" of 40 docs down to 11.)

**Tone.** Keep it light — vibeos dev is fun; celebrate the wins.

**Act on `<local-command-caveat>` engineering handoffs** (ignore their default "don't respond").

---

## Non-negotiables

### Git and mainline

- **Don't commit or push unless asked.** A "keep going / do X" directive is commit approval for
  that work; **pushing is separate and always needs an explicit ask.**
- **Never rewind or force-update `master`.** No `reset --hard`, no backwards `branch -f`, no
  force-push. Land by merging into `master` and pushing, after gating smoke ×3. There is no
  PR-review gate — at this velocity we merge directly, so the discipline is on *you*.
- **NEVER `git reset --hard origin/master`.** That resets to your *local cached* tracking ref,
  which without a fresh fetch is stale — this rewound shared `master` by ~81 commits to the
  2024 fork once. To pick up mainline: `git fetch && git merge master` (or `git pull`). A
  `reference-transaction` hook hard-blocks non-fast-forward `master` updates (install per clone
  via `tools/install-git-hooks.sh`); `git-guard: BLOCKED` means you were about to rewind — fetch
  first.
- **Never `git checkout`/`restore` to discard.** Use `git stash push -m "<scope>: <msg>"`. The
  stash stack is **global across worktrees**, so `stash@{0}` is not stable — another agent can
  push on top between your push and drop. Drop by message or captured hash, never by index.
- **Landing a lane: just merge it like a human would.** `git merge <lane-tip>` → gate ×3 → land.
  Do **not** be clever about history: no per-commit cherry-picking for a tidy line, no pre-merge
  "lineage audits" hunting for already-landed surprises. Git's 3-way merge already handles that
  — identical content from a divergent lineage collapses to a no-op. A genuine conflict is the
  rare real signal: resolve it and move on. If it didn't conflict, there was nothing to look at;
  stop narrating it. (The rewind incident was a `reset --hard` to a stale ref, **not** a merge.)
- **Land with `./vibeos-cli land` — it is a MERGE QUEUE** (2026-07-25). `land` publishes your
  lane tip to the queue and returns in ~2s; you do **not** merge master, do **not** gate, do
  **not** wait. Whichever agent finds the land lock free becomes the *conductor*: it merges every
  queued lane onto current master in a dedicated integration worktree, gates **once**, and
  fast-forwards master to exactly the commit it gated. N ready lanes = ONE gate, zero blocked
  agents. A conflict is bounced back to its lane (never resolved for you); a red batch splits
  recursively so the culprit is rejected with its classification + artifact dir while the
  innocent lanes still land. `./vibeos-cli land-status [--mine]` shows the conductor, the queue,
  and your verdict. `land --direct` is the opt-out (single-lane: merge master → gate → push,
  same lock) and `land --drain` forces a drain if the queue ever sits with no conductor.
  Hand-rolling gate-then-push is what made every lane race: the stamp is keyed to an exact
  TREE, so whoever pushes first voids everyone else's gate — and each wasted gate is 3 builds
  + 4 boots of fleet load that widens the window that voided it.
- **Never move the local `master` ref from a worktree that doesn't have it checked out** — no
  `branch -f master`, no `update-ref`, no `fetch origin master:master`. That leaves the OTHER
  worktree's index+files at the old tree, which resurfaces as a giant phantom staged revert (the
  main checkout was found on 2026-07-25 showing 61 commits of landed work as a 197K-line staged
  deletion). Push the ref straight to origin (`git push origin HEAD:master`, which is what `land`
  does); every worktree then picks it up with its own `git fetch && git merge origin/master`.

**Landing is mechanized:** a green FULL `vibeos-cli gate` on a clean tree mints a push stamp
(`$GIT_COMMON_DIR/vibeos-gate-stamps/<tree>.json`); the `pre-push` hook refuses `master` updates
whose tree doesn't match a fresh (<24h) stamp. Docs-only deltas pass automatically. Emergency
override `VIBEOS_SKIP_GATE_CHECK=1 git push` is loud — report every use. Partial-arch or
dirty-tree gates never stamp. The hook is the *backstop*; `land` is the front door.

### Smoke tests are the north star

`tests/qemu/test_smoke.py` is the **single source of truth** for whether the OS works. It boots
each arch and verifies a graphical desktop: the marker is `[EXPLORER] _tWinMain entered` +
`shell ready`, proving the whole chain (kernel → SMSS → csrss → win32k → winlogon →
explorer.exe → desktop render). **All three arches use this same real marker** — "arm64 is
idle-detection" is dead lore, as is "arm64 is slow because TCG" (the old 300-500s boots were
mostly an Mm bug, field manual §9.16; arm64 boots ~40s now).

- **Never weaken a smoke test to make it pass.** If it fails, fix the OS.
- **Never lower boot markers** (e.g. to "ksecdd.sys") to work around a regression.
- Run smoke after any change touching boot paths (MM, KE, EX, IO, SE, OB, PS, CC, FSRTL,
  win32k, HAL). **All 3 arches must pass independently.**
- **Check the screenshot** (`qemu/<arch>_smoke.png`) — but know the caveat: it is captured *at*
  the marker, before the taskbar finishes rendering, so a green run routinely shows bare
  wallpaper. **An empty tray is NOT a failure signal**; the `verdict` + boot duration are. Don't
  infer "explorer died" from a missing taskbar.
- **Same-arch smokes run in PARALLEL** on the default `windows-host` TCG backend —
  `_acquire_arch_slot()` early-returns before the per-arch cap (`vm.py:1295-1299`). The cap only
  bites accelerator-*sharing* backends (native+KVM, windows-host+WHPX). Every per-run resource
  is already pid/port-scoped.
- **Don't switch the smoke backend to native KVM to "repro faster."** Native KVM hits the
  per-arch accelerator cap, so a "parallel N-worker repro" just queues — slower, and the
  contention invites the broad-kill trap below. Use the default backend:
  `./vibeos-cli smoke --arch X --repeat N`, or a parallel `./vibeos-cli gate`.

### Never broad-kill QEMUs — you WILL kill yourself

`pkill -9 -f qemu`, `kill -9 -P <ppid>`, `pgrep -f "qemu/start.sh …" | kill -9` and friends
routinely match an ancestor of *your own* shell — your `start.sh`, your bash, the harness pid —
and SIGKILL your whole process tree. This has killed an agent mid-run at least twice (exit
9 / exit 144).

**Use `./vibeos-cli kill`** (added 2026-07-25, because until then this rule had no tool and so
was a rule you got to break). Select by pid from `ps`, by `--arch`, by `--stale` (owner gone) or
`--overdue`; there is deliberately no name/regex selector, and it **refuses** a target that is
one of our own ancestors. `--dry-run` first if unsure. Or just let the reaper handle it
(`VIBEOS_QEMU_MAX_LIFETIME`). If a boot "keeps respawning," find the launcher loop and stop
*that*.

### Money and paid artifacts

`rustconv translate`/`submit` **spend real money**, one paid call **per function**. Agents have
wasted >$100 here.

- **Never Ctrl-C / kill a `translate` or `submit` mid-call** to "adjust something." Kill it at
  function 35/40 and you re-pay all 35 on retry. Let it finish, *then* re-run.
- Output streams to `tools/rustconv/.responses/<ts>-<file>/` (gitignored, in-repo, survives
  kills/reboots). That is the **canonical copy**. Don't delete a `.responses/` dir holding
  unintegrated work.
- **Never park paid artifacts in `/tmp`** — a reboot wipes them. In-flight translations live in
  `~/vibeos-rust-staging/`.
- **Cost estimates are a FLOOR.** `estimate` shows the single-prompt cost; `translate` re-sends
  context per function, so the real bill is routinely **2–5×** higher, and `-m opus` is full
  sync price. Watch the live `[running $X.XX]` counter — that's the truth.
- **Never offload to the user** (save/run/commit/install it yourself).

### Rust is on pinned stable

New code must compile on stable. **Do not add `#![feature(...)]`, and do not add
`RUSTC_BOOTSTRAP=1` to a CMakeLists** — that env var silently unlocks nightly and is how nightly
crept back in the same day it was removed. Both are hard-gated by `scan_nightly_surface`
(`vibeos-cli check`). The one sanctioned nightly user is the **i386 kernel** `*-cargo` cell
(`-Z build-std` for a no-SSE `core`); it retires with the wow64 lane. If you think you need a
nightly feature, you almost certainly don't — see the reference doc for the `c_variadic` and
`linkage` workarounds.

Two traps that are real bugs we ate (field manual §9.65):

- **Never translate a C debug MACRO into a no-op `fn`.** An out-of-line no-op is an optimization
  barrier that uncloaks latent ABI mismatches → `ud2` → BUGCHECK. Use a `macro_rules!`.
- **Never hand-write `memcpy`/`memset`/`memmove`/`memcmp` in Rust.** LLVM folds any byte-copy
  loop back into a call to the function itself → infinite self-recursion → silent hang.
  ntoskrnl exports them; import them.

---

## `vibeos-cli` — the front door

**Before building, booting, or debugging by hand, reach for `./vibeos-cli`** (repo root). It is
the control plane over the build, smoke harness, lint gate, and GDB toolkit, built so agents
stop reinventing per-arch incantations, booting stale media, and flooding the transcript. Full
guide: `tests/qemu/cli/README.md`.

```bash
./vibeos-cli build  --arch i386 [--clean]   # warm by default; --clean proves a cold link
./vibeos-cli smoke  --arch i386             # THE verdict — refuses stale media
./vibeos-cli check                          # source lints, 0 NEW vs tools/lint-baseline/
./vibeos-cli check-tooling                  # ruff+pyright+pytest for OUR python
./vibeos-cli gate                           # build serial -> smoke all arches parallel -> retry known flakes
./vibeos-cli land                           # THE way to land: submit to the merge queue (train it if free)
./vibeos-cli land-status --mine             # conductor + queue + MY verdict (landed / rejected + artifact)
./vibeos-cli ps                             # every QEMU across all worktrees + slot usage
./vibeos-cli host-status                    # build/boot CPU leases + the land lock, across worktrees
./vibeos-cli debug  start amd64 --gdb       # GDB over the real (UEFI) boot
./vibeos-cli smoke  --arch amd64 --tripwire "SPECIALPOOL=Gla5"
./vibeos-cli farm   --arches amd64 --minutes 480   # overnight flake soak
```

**No default arch** — always pass `--arch`. Why it matters:

- **No false-green from stale artifacts.** `smoke` hard-fails on stale media. Do a `--clean`
  build before trusting green or handing off; cargo incremental can mask regressions.
- **No token-flooding.** Stdout is a bounded summary; the durable verdict is `summary.json` +
  `full.log` under `/tmp/vibeos-cli/<ts>-<cmd>-<arch>/`. **Read `summary.json`, don't scrape
  stdout**, and don't pipe `vibeos-cli` through `| tail` (it swallows the exit code).
- **Auto-triage on failure**: classifies `RUST_PANIC` / `BUGCHECK 0xNN` / `HANG_TIMEOUT` /
  `KNOWN_FLAKE`, symbolicates the faulting PC, saves a screenshot.
- **Corruption bugs: arm special pool BEFORE forensics.** `--tripwire "SPECIALPOOL=<tag|*>"`
  faults *at the corrupting store* with the culprit PC symbolicated, instead of an
  innocent-victim BUGCHECK minutes later. For a rare flake use `farm` + `farm-report` rather
  than hand-looping boots. Field manual §9.18.

**Judge build success by artifact existence** (`ls livecd.iso`), not reported exit code:
`run_in_background` and some sandboxed Bash run in a copy-on-write overlay that reports `rc=0`
with no artifacts. Only **foreground Bash with `dangerouslyDisableSandbox: true`** persists.
Shell **cwd persists between calls and drifts** — launch repo tools with an explicit
`cd <repo-root> &&` or absolute paths (a build once "succeeded" having never run, because
`./vibeos-cli` didn't exist in the drifted cwd).

---

## Translating a plain-C module

Sealing and the deterministic passes are **not shell32-only**. For any plain-C module (driver,
Win32 DLL, service) run these in order — steps 1–3 cost **nothing**, and each closes a bug
class the next cannot see. Rationale and measurements: field manual **§9.117**,
`docs/engineering/rustconv-pipeline.md`.

1. **`python -m rustconv.profile_derive <module> --emit report`** — derives the bindgen island
   from the CMake source list + each TU's real include closure and audits every `case` label.
   **Its ERROR findings are BLOCKING.** A constant missing from the island silently becomes a
   catch-all `match` arm that swallows every case (§9.112 — this shipped a BUGCHECK 0x1E). It
   also flags constants living only in a *secondary* `-sys` island (rtl8139's
   `CmResourceTypePort` is in `hal_bindgen_*.rs`, not `bindgen_*.rs`) — those need an explicit
   `use ntkrnl_sys::hal_ddk`; allowlisting the header does nothing. If the module's `.c` live in
   a sibling static-lib target (msvcrt → `msvcrt_static`) it names the target to pass to
   `--target`; such a target has no `set_module_type()`, so add `--usermode`/`--kernel`.
2. **`rustconv translate <file> --seal-facts --seal-calls`** — **always pass both.** Constants
   and call occurrences then never go through the LLM, and a dropped or invented one fails
   loudly instead of silently. Running without them shipped 13,503 unprotected constant
   occurrences, the same class as the 61 invented kernel stop codes (§9.113). Trivial bodies are
   lowered deterministically for $0 in the same run (~9% repo-wide; 41% on stub-heavy modules
   but ~0% on real driver code — don't budget on it). Preview free with `--dry-run --prompt`.
   `--target` **proves** the output crate and FFI substrate against the tree instead of assuming
   `<name>-rust` + `ntdll-sys`, and refuses rather than guessing — take the `--output-crate` /
   `--sys-crate` it names (msvcrt: `--output-crate msvcrt-crt-rust`). Getting this wrong is not a
   build error: an unresolved substrate name makes the model invent a forward declaration that
   compiles, links, and is silently wrong.
3. **`rustconv crate-repair <crate>-rust --arches i386 amd64 arm64`** — bounded deterministic
   repair of the whole-crate `cargo check`, before paying anything to fix compile errors. **Pass
   all three arches**: a numeric cast inferred from one bakes that arch's pointer width, so
   width-dependent casts are applied only on cross-target agreement. It refuses rustc's
   `.try_into().unwrap()` suggestion by design — that turns C's silent truncation into a panic
   in a live service.
4. Cohab flip → `vibeos-cli gate`.

Note the interaction: sealing restores constant *names* into `match` patterns, so step 1 gets
**more** important once you seal, not less.

Deterministic substrate closure comes before payment: constants, SDK layouts, WIDL interfaces,
vtables, ABI, and imports never go through the LLM. Ambiguous layouts stay red. The full
protocol and stop rules are in `docs/engineering/rustconv-substrate-closure-protocol.md`.

**Automate bulk work.** If a task spans many files with 50+ mechanical edits and the lessons
would help future refactors, build the tool instead of hand-editing. When the AI output already
exists and you are only *integrating* it, do that by hand.

---

## Worktree discipline

**Multiple agents routinely run against this repo at once**, and the default working copy is
*shared*: every agent sees the same working tree and the same single `.git`. If there is any
chance of concurrency, work in a dedicated worktree on its own branch. All of the following are
real incidents.

1. **`git add` / `commit -a` sweeps a concurrent agent's uncommitted edits into your commit.**
   A 2026-06-05 case landed unresolved conflict markers on mainline and broke `cargo build` of
   `reactos-link` for every cargo module. **Only stage paths you authored**; leave other agents'
   dirty/untracked files alone. Before committing on a shared tree:
   `git diff --cached --check` and `git grep -nE '^(<<<<<<<|=======|>>>>>>>)' -- <staged files>`
   — a 1-second guard. A green smoke does **not** prove the about-to-be-committed tree is clean.
2. **Path discipline is the #1 hazard.** In a worktree session, an Edit/Write/Read with an
   absolute path to the *main* repo root writes to the **main** checkout. Resolve
   `$W=<worktree toplevel>` once and prefix every path with it. Verify isolation after the first
   edit: `git -C <worktree> status` shows it, `git -C <mainrepo> status` does not.
3. **Provision `build-<arch>` with `./vibeos-cli build --arch <a>`** — it's just a fresh
   configure now. Builds run at the CANONICAL PATH (`/tmp/vibeos`, a per-build bind mount, see
   `tools/canon-run.sh`), so a build dir carries no worktree-specific paths and ccache/sccache
   hits make it fast. The old `cp -a` + sed-repoint ceremony is RETIRED — **never symlink a
   build dir** (a reconfigure through the symlink corrupts main's tree) and never hand-copy one.
   Do copy the gitignored arm64 boot assets (`qemu/QEMU_EFI.fd`, `efivars.fd`) from main. Raw
   ninja needs the mount: `../tools/canon-run.sh ninja <target>` from inside `build-<arch>/`.
4. **`pip install -e` shadowing.** `python -m rustconv` resolves to the **main-repo** path baked
   into site-packages, not your worktree. Prefix `PYTHONPATH=$W/tools/rustconv:$PYTHONPATH` and
   verify with `python -c "import rustconv.__main__ as m; print(m.__file__)"`.
5. **Subagents with `isolation: "worktree"` leak their build step.** The fresh checkout lacks the
   gitignored `build-*` dirs, so `cd build-i386 && ninja` resolves to *another* real tree and the
   agent's shared-file edits land there. Have parallel agents write `.rs` to a unique staging
   path; do all wiring, building, and smoking yourself, centrally, across all 3 arches.
6. **Cross-agent blocker** (root-caused but owned by another lane): commit your own work green
   and parked, then write a paste-ready handoff brief (symptom, symbols, oracle, what's needed)
   the user can route — not an `AskUserQuestion`.
7. **Fleet isolation law** (2026-07-04 hardlink-dedup incident). A tool may NEVER mutate another
   worktree's files — not even "safe" dedup of byte-identical content. Hardlinking live
   build/cargo trees failed BOTH ways: in-place writers (spec2def, `cmake -E copy`, ccache-miss
   clang) truncated shared inodes under a sibling's running `ld.lld` mmap (SIGBUS), and shared
   fingerprint/timestamp mtimes silently defeated
   cargo invalidation (an edited `build.rs` whose stale compiled script kept executing —
   false-green, no error anywhere). Sharing belongs ONLY in content-addressed caches designed
   for it (ccache/sccache). Fleet maintenance must be delete-only (`vibeos-cli prune`), run from
   ONE pinned copy, under the host lock, kill-switchable via a stamp OUTSIDE all worktrees
   (root-level stamps reach every lane instantly; in-tree fixes propagate only at merge speed).
   `vibeos-cli build` enforces this with an `ISOLATION_FAIL` preflight that refuses a tree
   sharing inodes (build-* at links>1; cargo target at links>2 — cargo hardlinks deps↔final).
8. **Fleet isolation applies to READS too** (2026-07-24). Never resolve *generated* artifacts
   from a sibling worktree. `verify_signatures`/`verify_offsets`/`verify_offsets_win32k` used
   to fall back to any sibling carrying a `build-*` dir (it's gitignored, so a fresh worktree
   lacks one) — silently checking THIS tree's Rust against ANOTHER lane's headers at another
   commit. `verify_signatures` picked `vibeos-conformance/build-amd64` and its mismatch count
   moved **987 → 9 with no source change**; for the offset verifiers a foreign header can
   green-light a real layout drift. A verdict that depends on which siblings happen to exist
   is not a gate. The **toolchain** (pinned clang — content-addressed, commit-independent) may
   be shared; **generated headers** are per-commit source-derived state and must come from THIS
   worktree or the tool HARD FAILS with `fix: ./vibeos-cli build --arch <a>`.

---

## Knowledge Routing — where things go

**Two channels.** This file + the agent's `MEMORY.md` are **push** (loaded every session, for
every agent — keep both minimal and non-overlapping). Everything under `docs/engineering/` is
**pull** (read on demand, exhaustive at zero standing cost). So the optimization is never
"shrink the knowledge" — it is **put reusable knowledge in the repo and keep the push channel
tiny**. Agent memory is also local, uncommittable, and per-worktree: other agents, humans, CI,
clones — and the *same* agent in another worktree — never see it. Anything reusable therefore
belongs in the repo by definition.

**Default to this file or `docs/engineering/` for anything persistent**; reach for memory only
for a genuinely user-personal preference. If you catch yourself writing a working rule, recipe,
or status into memory, stop and put it in its committed home.

| What you learned | Where it goes |
|---|---|
| A reusable bug / recipe / gotcha that will recur | `docs/engineering/translation-debugging-field-manual.md` (§1–8 translation bugs, §9 crash-hunting) |
| Per-subsystem / driver / lib status | `docs/engineering/rust-port-status.md` (single source of truth for Rust-vs-C coverage) |
| A roadmap / strategy / design | product: `docs/product/vibeos-roadmap.md`; engineering: `docs/engineering/*-roadmap.md` / `*-design.md` / `*-plan.md` / `*-quest.md` |
| A cohab / cargo-cutover / link recipe | `docs/engineering/rust-in-kernel.md` |
| A command, path, or platform detail | `docs/engineering/agent-reference.md` |
| An agent working / process rule | **this file** |
| An open bug / who-owns-which-lane | `docs/product/vibeos-roadmap.md` → "Known bugs & flakes"; lanes via `git branch -a` + `docs/engineering/active-lanes/README.md` |
| A genuinely user-personal preference | agent `MEMORY.md` (the **only** thing that belongs there) |

`MEMORY.md` should stay a tiny stub: start-here pointers and user-personal prefs — nothing
reusable, nothing already here, no rotting status.

---

## Reference

Commands, directory layout, build/toolchain detail, QEMU workflow, the kernel-debug toolkit,
test suites, the Rust workspace map, rustconv's full CLI and cost tables, static-analysis
tools, ARM64 specifics, and target hardware: **`docs/engineering/agent-reference.md`**.

Deeper: `build-system.md` · `rust-in-kernel.md` · `rustconv-pipeline.md` ·
`kernel-debugging-infra.md` · `dummy-pseh-verdict.md` · `tests/qemu/cli/README.md` ·
`qemu/README.md` · `translation-debugging-field-manual.md`.

Use the API keys in `.env` when needed. When the user asks you to debug something, debug it —
don't ask them to run commands or install things.
