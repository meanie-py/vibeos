# AGENTS.md

This file provides guidance to AI coding agents (Claude Code, Codex CLI, etc.) when working with code in this repository.

## Project Overview

This is a **multi-architecture port of ReactOS** — an open-source Windows-compatible operating system. Originally forked from `github.com/reactos/reactos` to add ARM64. The OS cross-compiles to **ARM64, amd64, and i386** PE-COFF with zero build failures on all three architectures. All three boot to a graphical desktop in QEMU. The mainline branch is **`master`**. Work happens on short-lived feature branches — often in git worktrees so multiple agents run in parallel — and lands on `master` via a normal `git merge` (see Agent Working Rules below). *(History: work was previously staged on `continue-rust-port` with `master` as a periodically fast-forwarded mirror; the two were unified onto `master` on 2026-07-03 and `continue-rust-port` was retired.)*

## Translation Philosophy — Rust Is The Goal

**The project's goal is to translate ntoskrnl to Rust.** Getting the binary to build and boot is *important and necessary* but it is not the goal. Translation progress (lines of C → Rust) and boot status are independent axes.

**Rules:**
- **Never revert a Rust translation to C** to make a build link or a smoke test pass. Keep the Rust file. Fix it.
- **Never delete a translated Rust file** without explicit user approval. API translations cost real money; even broken output has value as a starting point for further work. Deletion is destructive and the work cannot be recovered without paying for the API call again.
- **Don't regenerate API translations without user approval.** Always ask before re-submitting a file that's already been translated. Cost matters.
- **Don't exclude a Rust translation from the build to dodge a regression.** Commenting out `pub mod X;` in `mod.rs` (or `lib.rs`) so the smoke tests pass while the C version is enabled is the same anti-pattern as reverting — it hides the regression from the smoke harness, which is *the diagnostic tool* for finding the bug.
- **A broken Rust translation in the tree, integrated and failing, is the correct state** when a regression is unsolved. The smoke harness reports it; you debug from there. The right response to a broken translation is (a) fix it in place, or (b) leave it broken with the failure visible in smoke and ask the user how to proceed — never (c) hide it from the build, and never (d) delete it.

These rules override the smoke-test imperative below when they conflict. If you have to choose between "smoke tests pass today" and "preserve in-progress Rust translation work as a debug signal," preserve the translation. A red smoke result that points at a real translation bug is more useful than a green smoke result that hides it.

## Agent Working Rules

Durable rules for how to work here (these live in the repo, not in per-agent memory — see Knowledge Routing below):

- **Don't commit or push without being asked.** A "keep going / do X" directive is commit approval for that work; pushing is separate and always needs an explicit ask.
- **Never `git checkout`/`restore` to discard** — `git stash push -m "<scope>: <msg>"` instead (and on the shared tree the stash stack is global — drop by message/hash, never index; see Worktree Discipline).
- **Never rewind or force-update `master`.** Work on feature branches and land by merging into `master` (next rule) + pushing; gate smoke ×3 first (North Star). There's **no PR-review gate** — at this velocity we merge directly, so the discipline is on *you* to gate before landing. What's forbidden is *rewinding* mainline: don't `reset --hard`, `branch -f` backwards, or force-push `master`. **NEVER `git reset --hard origin/master`** — that resets to your *local cached* tracking ref, which without a fresh `fetch` is stale, and rewinds shared `master` by however many commits you were behind (this rewound it ~81 commits to the 2024 fork once). To pick up the latest mainline, `git fetch && git merge master` (or `git pull`). A `reference-transaction` git hook hard-blocks non-fast-forward updates to `master` (install once per clone via `tools/install-git-hooks.sh`). If you see `git-guard: BLOCKED`, you were about to rewind mainline — `fetch` first.
- **Landing/integrating a lane: just merge it like a human would.** `git merge <lane-tip>` (or fast-forward when it's an ancestor) → gate ×3 → land. **Do NOT be clever about history:** no per-commit cherry-picking to keep a tidy line, no pre-merge "lineage audits" hunting for dedup/already-landed/resurrected-file surprises, no trying to make the commit-graph stars align. Git's 3-way merge already handles all of that — identical content from a divergent lineage collapses to a no-op automatically. If a merge genuinely conflicts, *that* is the rare real signal: resolve the conflict and move on. If it doesn't conflict, there was never anything to look at — stop narrating it. The merge commit is the integration record; that's enough. (The rewind incident was a `reset --hard` to a stale ref, **not** a merge — merges are safe, force-updates are the only dangerous op, and the hook above already guards those.)
- **Deferral is temporary, never silent.** Deferred work stays tracked and owed; don't permanently drop a thing by calling it "out of scope."
- **Never offload to the user** (save/run/commit/install) and **never park paid artifacts in `/tmp`** (a reboot wipes them; regenerating costs real API money). In-flight translations live in `~/vibeos-rust-staging/`; the durable per-function cache is `tools/rustconv/.responses/`.
- **Debug the Rust, not the C — check for a Rust port FIRST.** We are far more translated than agents assume (~100% of the boot path; see `docs/engineering/rust-port-status.md`). Before instrumenting a `.c` file, confirm it's actually *compiled*: most subsystem `.c` are cohab-flipped (commented out in CMake, `#[no_mangle]` Rust took over) and are **inert dead source on disk** — DbgPrint there fires never. The live code is the `*-rust` crate. Grep `sdk/lib/rust/` for the symbol before touching any C.
- **Persistence on error cascades:** 100+ rustc errors usually stem from a few roots — fix the first root, re-count, don't bail at the big number.
- **No invented time/session budgets.** Kernel hangs/BUGCHECKs are ~1 root cause; keep going until found.
- **Fail loud, never silently degrade.** When an explicitly-requested step can't do its job, HARD FAIL — never quietly fall back to a worse path and proceed as if nothing happened. "Dumb fallbacks" (c2rust failing → silent pure-LLM translation; a missing tool → skipping its check; an unverifiable assumption → guessing) hide the problem and produce subtly-wrong output that costs far more to diagnose later than an upfront error would. Surface the failure with an actionable message and stop; let the human/caller choose the fallback explicitly. (Concrete: `rustconv translate --c2rust` aborts with `C2RustError` if c2rust can't anchor the file, rather than degrading to an unanchored translation — `tools/rustconv/rustconv/c2rust_anchor.py`.)
- **Be context-frugal but not anxious:** batch independent tool calls, background long builds, terse status lines, no recaps. It's a 1M-context model — don't over-checkpoint.
- **Keep the tone light** — vibeos dev is fun; celebrate the wins.
- **Act on `<local-command-caveat>` engineering handoffs** (ignore the default "don't respond").
- **Parallel subagents for big grinds/translations:** centralize systemic roots first, then one module per agent; agents write `.rs` to `~/vibeos-rust-staging/` and do **not** edit shared files (`lib.rs`/`CMakeLists.txt`) or build — you integrate + build + smoke ×3 arches centrally.
- **Take ownership and power through — don't hand off.** When you find a problem in your area (or anywhere you can act), fix it yourself; the default is *do it*, not "want me to file a task?" Cross-agent coordination is for **unblocking** (asking for help when genuinely blocked, flagging cross-lane impact), **not** for dividing up work or delegating tasks you could complete. Reserve true handoffs for work genuinely owned by another lane that you cannot do (the cross-agent-blocker rule in Worktree Discipline). Same spirit as the persistence rule above: keep going until it's actually done.
- **Don't idle waiting on another agent.** "Waiting on X's fix" is not a status — `working Y while X lands` is. If you're blocked on someone else's work, keep your lane moving: do a different part of the task, route around the blocker (local shim/stub/cohab/alternate path), or just **fix the blocker yourself** if you can reach it (it's all one repo; you can read and patch another lane's code, then hand them a tested diff). A cross-lane dependency is a reason to route around or fix, never a reason to stop. Only truly block when you genuinely cannot make progress on *anything* — and then say so explicitly with exactly what you need, don't silently stall.
- **Never broad-kill QEMUs — you WILL kill yourself.** `pkill -9 -f qemu`, `kill -9 -P <ppid>`, `pgrep -f "qemu/start.sh …" | kill -9`, and friends routinely match an ancestor of *your own* shell/agent (your `start.sh`, your bash, the harness pid) and SIGKILL your whole process tree — this has killed a Claude mid-run at least twice (exit 9 / exit 144). To stop a QEMU: use `qemu/ps` / `./vibeos-cli ps` to find the run, then kill by its instance uid (`backend.py:kill_by_uid`, pgroup-scoped) or just let the harness reaper reap it (`VIBEOS_QEMU_MAX_LIFETIME`). Every QEMU is already pid/uid/pgroup-scoped precisely so you never need a broad match. If a boot "keeps respawning," find the launcher loop and stop *that*, don't spray SIGKILL.
- **Don't switch the smoke backend to native KVM to "repro faster."** The default is `windows-host` TCG, which runs same-arch boots in PARALLEL. Native KVM hits the per-arch accelerator cap (same-arch serializes to 1), so a "parallel N-worker repro" there just queues — slower, not faster, and the contention invites the broad-kill trap above. For a parallel repro use the default backend: `./vibeos-cli smoke --arch X --repeat N` (independent boots) or a parallel `./vibeos-cli gate`.

## Knowledge Routing — Where Things Go

**Two context channels.** `AGENTS.md` + the agent's `MEMORY.md` are **push** (loaded into *every* session, for every agent — keep both minimal and non-overlapping). Everything under `docs/engineering/` is **pull** (read on demand — can be exhaustive at zero standing cost). So the optimization is never "shrink the knowledge" — it's **put reusable knowledge in the repo (pull) and keep the push channel tiny**. Agent memory is also *local + uncommittable + per-worktree* (other agents/humans/CI/clones — and even the *same* agent in another worktree — never see it), so anything reusable belongs in the repo by definition. **Default to AGENTS.md (or `docs/engineering/`) for anything persistent**; reach for memory only for a genuinely user-personal preference. If you catch yourself writing a working rule, recipe, or status into memory, stop and put it in the committed home instead.

When you learn something, route it to the right committed home — do **not** default to memory:

| What you learned | Where it goes |
|---|---|
| A reusable bug / recipe / gotcha that will recur | `docs/engineering/translation-debugging-field-manual.md` (§1–8 translation bugs, §9 crash-hunting) |
| Per-subsystem / driver / lib status | `docs/engineering/rust-port-status.md` (the single source of truth for Rust-vs-C coverage; per-file history in `docs/engineering/archive/*-translation-audit.md`) |
| A roadmap / strategy / design | THE product roadmap is `docs/product/vibeos-roadmap.md` (phased, t-shirt-sized; `driver-roadmap.md` for the driver-model/WDF future). Engineering-scoped plans: `docs/engineering/*-roadmap.md` / `*-design.md` / `*-plan.md` / `*-quest.md` |
| A cohab / cargo-cutover / link recipe | `docs/engineering/rust-in-kernel.md` |
| An agent working / process rule | **this file** (`AGENTS.md`) |
| An open bug / who-owns-which-lane | *(interim: the open-bugs note in `MEMORY.md`; active lanes are visible via `git branch -a`)* |
| A genuinely user-personal preference | agent `MEMORY.md` (the **only** thing that belongs there) |

`MEMORY.md` should therefore stay a tiny stub: start-here pointers and user-personal prefs — nothing reusable, nothing already in this file, no rotting status. If you catch yourself writing a paragraph of durable technique into memory, stop and put it in the field manual instead.

## Smoke Tests — The North Star

The QEMU smoke tests (`tests/qemu/test_smoke.py`) are the **single source of truth** for whether the OS works. They boot each architecture in QEMU and verify the system reaches a graphical desktop — the boot marker is `[EXPLORER] _tWinMain entered` + `shell ready`, which proves the full chain (kernel → SMSS → csrss → win32k → winlogon → explorer.exe → desktop render) completed. **All three arches (i386/amd64/arm64) use this same real marker** — arm64 is *not* idle-detection (that's outdated lore); it reaches `_tWinMain`/`shell ready` like x86. The default smoke backend is **`windows-host` QEMU (the Windows `qemu-system-*.exe`) under TCG on all three arches** (see "Target Hardware"). **"arm64 is slow because TCG" is ALSO dead lore** (2026-07-05): the old ~300-500s arm64 boots were mostly a Mm bug — `MiPteToPde`/`Ppe`/`Pxe` were one self-map level too high, so every VA-teardown walked per-PTE (field-manual §9.16). With that fixed, arm64 boots ~40s, so its `boot_timeout` was right-sized 540→180s; other arm64 harness waits (net-suite/snapshot/per-milestone) may still be oversized (owed sweep). See `Arm64Config.boot_marker`/`required_milestones` in `tests/qemu/vm.py`. A screenshot is always saved for visual inspection.

**Rules:**
- **Never weaken a smoke test to make it pass.** If a test fails, fix the OS, not the test.
- **Never lower boot markers** (e.g., from "explorer.exe created" to "ksecdd.sys") to work around regressions.
- **Run smoke tests after any kernel/subsystem change** that touches boot paths (MM, KE, EX, IO, SE, OB, PS, CC, FSRTL, win32k, HAL).
- **All 3 architectures must pass independently.**
- **Always check the screenshot** — the test saves `qemu/<arch>_smoke.png` for visual inspection. **Caveat: the screenshot is captured at the boot marker (`shell ready`), *before* the taskbar finishes rendering — so a green run routinely shows bare wallpaper with no taskbar. An empty tray in the shot is NOT a failure signal; the `verdict` + boot duration are. Don't infer "explorer died" from a missing taskbar.**
- **Same-arch QEMU smokes run in PARALLEL on the default `windows-host` TCG backend — the per-arch serialize cap only bites the KVM/WHPX backends.** `per_arch_cap()` (tests/qemu/vm.py) caps i386/amd64 to one concurrent boot each, but `_acquire_arch_slot()` (vm.py:1295-1299) **early-returns before consulting it** when `backend==windows-host` and `accel!=whpx` — i.e. our default. So same-arch AND cross-arch boots are concurrent today; every per-run resource is already pid/port-scoped for it (staging subdir/ports/`-name` uid/reaper key — vm.py:892-902). The cap's *original* rationale was `/dev/kvm` + shared-IDE starvation between two KVM guests on the **native** backend (a 2026-06-19 incident burned hours: two agents gated i386 at once, both false-`HANG_TIMEOUT` + no-taskbar, misattributed to a code bug until a host-quiet re-gate came back 5/5 green ~103s vs 3/3 fail ~361s while contended) — that contention is real only where an accelerator is *shared*, so the cap still applies to `native`+KVM and `windows-host`+WHPX (WHPX is real host hardware-virt, so same-arch whpx guests contend like KVM). The real throttle for parallel TCG smokes is the host CPU-lease pool (`hostlock`, 4 leases). Drive all this with **`vibeos-cli gate`** (builds serial → smokes all arches in parallel → auto-retries documented flakes in parallel). `qemu/ps` / `./vibeos-cli ps` shows live boots + slot usage.

## `vibeos-cli` — The Build/Check/Smoke/Debug Front Door

**Before building, booting, or debugging by hand, reach for `./vibeos-cli` (repo root).** It is the unified control plane over the build, the smoke harness, the lint gate, and the GDB toolkit — built specifically so agents stop reinventing per-arch build incantations, booting stale media, and flooding the transcript with serial logs. Full guide + per-tool motivation: **`tests/qemu/cli/README.md`** (read it once).

```bash
./vibeos-cli build  --arch i386 [--clean]   # warm by default; --clean = cold (nukes rust targets, proves cold-link)
                                            # canonical-path + mandatory shared cache (ccache/sccache); auto-provisions
                                            # a missing build dir, auto-migrates a legacy one (loud one-time rebuild)
./vibeos-cli smoke  --arch i386             # THE verdict — refuses to boot stale media (build first, or --allow-stale)
./vibeos-cli check                          # source lints, baseline-diff gate (0 NEW vs tools/lint-baseline/)
./vibeos-cli check-tooling                  # ruff+pyright+pytest gate for OUR python (rustconv/cli/tools)
./vibeos-cli debug  start amd64 --gdb       # wraps qemu/*.sh (POSITIONAL arch); GDB over the real (UEFI) boot
./vibeos-cli ps                             # every QEMU across all worktrees + slot usage
./vibeos-cli smoke  --arch amd64 --tripwire "SPECIALPOOL=Gla5"   # arm per-boot kernel tripwires via fw_cfg
./vibeos-cli farm   --arches amd64 --minutes 480 --tripwire "SPECIALPOOL=*"   # overnight flake soak
./vibeos-cli farm-report                    # cluster the farm corpus into ranked failure families
```

Why it matters (the bugs it prevents):
- **No more false-green from stale artifacts.** `smoke` hard-fails on stale/missing media; warm builds are flagged because cargo incremental can mask regressions. Do a `--clean` build before trusting green or handing off.
- **No per-arch divergence.** `archcfg.py` is the single source of truth for build targets + boot media (x86 = `ninja livecd`; arm64 = staged set + `mkramdisk --refresh`). `boot` reuses smoke's exact arch-config so the fast path can't drift from the real test.
- **No token-flooding.** Stdout is a bounded summary; the durable verdict is `summary.json` + `full.log` under `/tmp/vibeos-cli/<ts>-<cmd>-<arch>/`. **Read `summary.json`, don't scrape stdout.**
- **Auto-triage on failure.** Classifies `RUST_PANIC` (file:line) / `BUGCHECK 0xNN` / `HANG_TIMEOUT` / `KNOWN_FLAKE`, symbolicates the faulting PC, captures a screenshot — no manual objdump round-trip.
- **Corruption bugs: arm special pool BEFORE forensics.** `--tripwire "SPECIALPOOL=<tag|*>"` (fw_cfg channel, no media rebuild, x86) makes a wild write/overrun fault **at the corrupting store** with the culprit PC symbolicated on serial, instead of surfacing minutes later as an innocent-victim BUGCHECK. For a *rare* flake, don't hand-loop boots — `farm` soaks armed boots into `~/vibeos-flake-farm/corpus.jsonl` and `farm-report` clusters the failures by (classification, stop code, symbol). Recipe + grammar: field manual **§9.18**.

**No default arch** (always pass `--arch`; `i386`/`amd64` alias to the UEFI boot configs). This is the front door; the raw `pytest tests/qemu/test_smoke.py` path and the `qemu/*.sh` toolkit still work underneath (see `tests/qemu/cli/README.md` → "Relationship to `qemu/`" for which layer to use when).

## Worktree Discipline — Work in Isolation When Other Agents Are Active

**Multiple agents routinely run against this repo at once.** The default working copy
(`/home/curi/Projects/os-playground/vibeos`) is *shared*: every agent sees the same
working tree and the same single `.git`. That makes uncommitted work collide in ways
that silently corrupt commits. **If there is any chance another agent is working
concurrently, do your work in a dedicated git worktree, not the shared main checkout.**

Why it matters (all are real incidents, not hypotheticals):
- **`git add` / `git commit -a` sweeps a concurrent agent's uncommitted edits into
  *your* commit.** A 2026-06-05 case landed unresolved conflict markers from another
  session into a commit and broke `cargo build` of `reactos-link` for every
  cargo-driven module on mainline. Stray edits in the shared tree become
  random hunks on someone else's branch.
- **The git stash is one global stack shared by all worktrees.** `stash@{0}` is *not*
  stable — another agent can push on top between your `stash push` and `stash drop`,
  so dropping `{0}` destroys their work. Always `git stash push -m "<scope>: <msg>"`
  and drop/pop by **message or captured hash**, never by index.
- **Editing the wrong tree is the failure mode** — worktree git isolation itself works.

### Rules

1. **Prefer a worktree** (`EnterWorktree`, or `git worktree add`) on its own branch
   when collaborating. Verify isolation early: after the first edit, `git -C <worktree>
   status` shows it and `git -C <mainrepo> status` does not.
2. **Path discipline (the #1 hazard).** In a worktree session, an Edit/Write/Read with
   an absolute path to the *main* repo root writes to the **main** checkout, not the
   worktree. Resolve `$W=<worktree toplevel>` once and prefix every file path with it.
   Never bare-path the main repo from a worktree session.
3. **Provision a worktree's `build-<arch>` with `./vibeos-cli build --arch <a>` — it's
   just a fresh configure now.** Builds run at the CANONICAL PATH (`/tmp/vibeos`, a
   per-build bind mount — see `tools/canon-run.sh`), so a build dir carries no
   worktree-specific paths and a missing one is auto-provisioned by a fresh cmake
   configure that's fast on shared-cache hits (ccache C/C++ + sccache Rust are
   mandatory and shared across worktrees). The old `cp -a` + sed-repoint ceremony is
   RETIRED — never symlink a build dir (a reconfigure through the symlink corrupts
   main's tree), and never hand-copy one. Do still copy the gitignored arm64 boot
   assets (`qemu/QEMU_EFI.fd efivars.fd`) from main for smoke/debug; `boot.img` +
   `ramdisk.img` are rebuilt by the build's mkramdisk step. Raw ninja against a
   canonical build dir needs the mount: `../tools/canon-run.sh ninja <target>` from
   inside `build-<arch>/` (plain `ninja` fails loudly — its baked paths only exist
   inside the mount).
4. **`pip install -e` shadowing.** `python -m rustconv` resolves to the **main-repo**
   path baked into site-packages, not the worktree. To run worktree tool edits, prefix
   `PYTHONPATH=$W/tools/rustconv:$PYTHONPATH`; verify with
   `python -c "import rustconv.__main__ as m; print(m.__file__)"`.
5. **Subagents with `isolation: "worktree"` leak their build step.** The fresh checkout
   lacks the gitignored `build-*` dirs, so `cd build-i386 && ninja` resolves to *another*
   real tree and the agent's edits to shared files (`lib.rs`, `CMakeLists.txt`) land
   there. Have parallel translation subagents write their `.rs` to a unique staging
   path and do **all** wiring + build + smoke centrally yourself, in one pass, across
   all 3 arches.
6. **Before committing on a shared tree:** `git diff --cached --check` (and
   `git grep -nE '^(<<<<<<<|=======|>>>>>>>)' -- <staged files>`) — a 1-second guard.
   A green smoke does **not** prove the about-to-be-committed tree is clean when the
   checkout is shared; re-check staged content at commit time. Only stage paths you
   authored; leave other agents' dirty/untracked files alone.
7. **Cross-agent blocker** (root-caused but owned by another agent's lane): commit your
   own work green + parked, then write a **paste-ready handoff brief** (exact symptom,
   symbols, oracle, what's needed) the user can route — not an `AskUserQuestion`.
8. **Paid translation outputs go to `~/vibeos-rust-staging/`, never `/tmp`** (a reboot
   wipes `/tmp`; regenerating costs real API money). The durable per-function cache is
   also in-repo at `tools/rustconv/.responses/` (gitignored).
9. **Fleet isolation law (2026-07-04 hardlink-dedup incident).** A tool may NEVER
   mutate another worktree's files — not even "safe" dedup/hardlinking of
   byte-identical content. Hardlinking live build/cargo trees failed BOTH ways:
   in-place writers (spec2def, `cmake -E copy`, ccache-miss clang) truncated shared
   inodes under a sibling's running `ld.lld` mmap (SIGBUS), and shared fingerprint/
   timestamp mtimes silently defeated cargo invalidation (an edited build.rs whose
   stale compiled script kept executing — false-green, no error anywhere). Sharing
   belongs ONLY in content-addressed caches designed for it (ccache/sccache).
   Fleet-scoped maintenance must be delete-only (`vibeos-cli prune`), run from ONE
   pinned copy, under the host lock, and kill-switchable via a stamp OUTSIDE all
   worktrees (a future-dated `$VIBEOS_ROOT/.dedup-last` is what actually stopped the
   incident — root-level stamps reach every lane instantly, regardless of each
   lane's code version; in-tree fixes propagate only at merge speed). `vibeos-cli
   build` enforces the invariant mechanically: an `ISOLATION_FAIL` preflight refuses
   to build a tree that shares inodes (build-* at links>1; cargo target at links>2 —
   cargo hardlinks deps↔final internally).

Build operational reality that interacts with the above: judge build success by
**artifact existence** (`ls livecd.iso`), not reported exit code — `run_in_background`
and some sandboxed Bash run in a copy-on-write overlay that reports `rc=0` with no real
artifacts; only **foreground Bash with `dangerouslyDisableSandbox: true`** persists.
Two more rc-masking traps from the same family: piping `vibeos-cli` through `| tail`
in a background shell swallows the exit code (its stdout is already bounded — don't
pipe it; read `summary.json`), and shell **cwd persists between calls and drifts** —
launch repo tools with an explicit `cd <repo-root> &&` or absolute paths (a build once
"succeeded" (rc=0 via the pipe) having never run: `./vibeos-cli` didn't exist in the
drifted cwd).

**Landing is mechanized (2026-07-04):** a green FULL `vibeos-cli gate` on a clean tree
mints a push stamp (`$GIT_COMMON_DIR/vibeos-gate-stamp.json`); the `pre-push` hook
refuses updates to `master` whose tree doesn't match a fresh (<24h) stamp. Docs-only
deltas pass automatically. Emergency override: `VIBEOS_SKIP_GATE_CHECK=1 git push`
(loud — report every use). Partial-arch or dirty-tree gates never stamp.

## Directory Layout

```
vibeos/                          # Git repo root (renamed from reactos/)
├── build-arm64/                 # CMake/Ninja build output (ARM64)
├── build-amd64/                 # CMake/Ninja build output (amd64)
├── build-i386/                  # CMake/Ninja build output (i386)
├── .build-tools/
│   └── llvm-mingw/              # Cross-compiler toolchain
├── qemu/                        # QEMU development environment
│   ├── run.sh                   # Launch QEMU (--arch arm64|amd64|i386, --gui, --debug)
│   ├── sym.sh                   # Symbolicate crash addresses
│   ├── mkramdisk.sh             # Create ARM64 ramdisk + boot images
│   ├── boot.img                 # FAT32 EFI boot disk (ARM64, gitignored)
│   ├── ramdisk.img              # FAT32 ramdisk with ReactOS files (ARM64, gitignored)
│   ├── QEMU_EFI.fd              # ARM64 UEFI firmware (EDK2, gitignored)
│   └── mouse_click.py           # Host-side mouse injection script
├── docs/
│   ├── product/                 # Roadmaps, status updates
│   └── engineering/             # Debug briefs, technical plans
├── ntoskrnl/                    # Kernel source
├── sdk/                         # SDK + Rust crates
├── tools/rustconv/              # Translation pipeline
└── tests/qemu/                  # Smoke test framework
```

## Build Commands

**Builds run at the CANONICAL PATH** (`/tmp/vibeos` — a per-build bind mount, see
`tools/canon-run.sh` and `docs/engineering/build-system.md`): build
dirs bake `/tmp/vibeos/...` paths so caches share across worktrees. `./vibeos-cli build`
handles this automatically (+ mandatory ccache/sccache, provisioning, lints). For raw
ninja iteration, prefix with the wrapper — **plain `ninja` in a build dir fails loudly**
(its baked paths only exist inside the mount):

```bash
# ARM64
cd build-arm64
../tools/canon-run.sh ninja            # Build everything
../tools/canon-run.sh ninja ntoskrnl   # Just the kernel

# amd64 / i386 (same pattern)
cd build-amd64
../tools/canon-run.sh ninja livecd     # Live CD ISO (livecd.iso)
../tools/canon-run.sh ninja bootcd     # Boot CD ISO (bootcd.iso)

# Common patterns
../tools/canon-run.sh ninja -k0        # Continue past errors
../tools/canon-run.sh ninja <target>   # Any target — list: ... ninja -t targets all | grep <name>
```

After modifying a `CMakeLists.txt` or `.cmake` file, re-run cmake before building:
```bash
../tools/canon-run.sh cmake .          # Re-configure (from any build-*/ dir)
```

## Toolchain

- **Compiler:** llvm-mingw (clang 19, lld 19.1.7)
- **Unified toolchain file:** `toolchain-llvm-mingw.cmake` — builds all 3 architectures
- **Targets:** `aarch64-w64-mingw32`, `x86_64-w64-mingw32`, `i686-w64-mingw32`
- **External dependency:** `windmc` from Homebrew mingw-w64 (message compiler, not in llvm-mingw)
- **16-bit assembly:** Homebrew `i686-w64-mingw32-gcc` for boot sectors/frldr16/MVDM (Clang can't handle `.code16`)
- **Output format:** PE-COFF (Windows executables/DLLs), not ELF

### Rust is on PINNED STABLE — write stable Rust (2026-07-12)

**The Rust tree builds on stable rustc** (pinned in `rust-toolchain.toml`). Your new code
must compile on stable. **Do NOT add `#![feature(...)]`, and do NOT add `RUSTC_BOOTSTRAP=1`
to a CMakeLists** — that env var is the stable-rustc escape hatch that silently unlocks
nightly, and it is how nightly crept back in the same day the lane removed it. Both are
now hard-gated by `scan_nightly_surface` (`vibeos-cli check`), so you'll get a loud failure
rather than a surprise.

The **one** sanctioned nightly user is the **i386 kernel** `*-cargo` cell: it needs
`-Z build-std` + `i686-kernel.json` for `rustc-abi: x86-softfloat` (no builtin triple
provides a no-SSE `core`), pinned in `.rust-nightly`. It *retires* with the wow64 lane.
amd64/arm64 build the **builtin triples** with rustup's precompiled std.

If you think you need a nightly feature, you almost certainly don't:
- **`c_variadic`** — needed only to *DEFINE* a C-variadic fn. *Declaring* and *calling* one
  is fine on stable. No-op debug shims (`TRACE(fmt, ...)`) should be a `macro_rules!` (see
  `ole32-rust/src/debug_shims.rs`); to genuinely define one, use the C `va_start` trampoline
  in `sdk/lib/rust/varargs-rust/`.
- **`linkage`** — perma-unstable upstream; it will never stabilize. Restructure.

Two related traps, both real bugs we ate (field manual §9.65):
- **Never translate a C debug MACRO into a no-op `fn`.** An out-of-line no-op is an
  optimization barrier that uncloaks latent ABI mismatches → `ud2` → BUGCHECK. Use a macro.
- **Never hand-write `memcpy`/`memset`/`memmove`/`memcmp` in Rust.** LLVM folds any byte-copy
  loop back into a call to the function itself → infinite self-recursion → silent hang.
  ntoskrnl exports them; just import them.

Full reference: `docs/engineering/build-system.md` §"Cargo target model".

## Key Technical Patterns

### SEH / DUMMY_PSEH (status 2026-07-10 — the ruling doc is `docs/engineering/dummy-pseh-verdict.md`)
`USE_DUMMY_PSEH` is still the default compile mode for **C** on all arches (PSEH3 is incompatible with Clang — ARM64 lacks x86 SEH; Clang's ESP-relative locals break i386 PSEH filters), so `_SEH2_TRY`-protected C must still validate pointers before dereferencing. **But the old "SEH is dead code everywhere, swallow software exceptions" era is OVER** (2026-07):
- **Rust frames have real unwind tables** (`requires-uwtable`): the kernel ships `.pdata`/`.xdata` for every Rust function and the unwinder works (`rtl-except`; field manual §9.35).
- **The exception dispatcher is live infrastructure**: hardware faults and software raises dispatch to user-mode SEH on all three arches (arm64 unwinder fixed 2026-07-10; user-SEH self-test in the gate). Do NOT re-add "swallow it" hacks.
- **Kernel-mode catch is possible** (validated PoC `ntoskrnl/seh_poc.c`); native SEH is a sanctioned per-module tool for the ~22 remaining C files with `_SEH2_TRY`.
- **Rust replaces PSEH semantics idiomatically**, not mechanically: validate-first probes returning `Result` (rustconv Stage 2), `try_*` contract conversions at raise sites, RAII `Drop` guards for `__finally`. See the verdict doc before writing any new SEH-shaped code.

### ARM64 PE-COFF Relocations
Use `adrp + add :lo12: + ldr` (not `adrp + ldr :lo12:`) in assembly. The latter generates `LDST64_ABS_LO12_NC` relocations requiring 8-byte alignment which PE-COFF sections may not provide.

### LLD i386 Forwarder Decoration Bug
LLD prepends `_` to PE export forwarder targets on i386. Workaround in FreeLDR (peloader.c) and ntoskrnl (sysldr.c): strip leading `_` from forwarder DLL name.

### GAS Intel Syntax on i386
In GAS `.intel_syntax noprefix`, `mov eax, label` does a **memory load** (reads bytes at label), NOT an address load. Must use `lea eax, [label]` or `mov eax, offset label`.

### Build System Architecture Conditionals
Architecture checks use: `if(ARCH STREQUAL "arm64")` / `"amd64"` / `"i386"`. ARM64 uses UEFI-only boot. The `USE_DUMMY_PSEH` flag is set for all architectures when using llvm-mingw.

### llvm-rc Limitations
llvm-rc is stricter than MSVC's rc.exe:
- ACCELERATORS: Cannot combine `ASCII` type with `ALT`/`SHIFT`/`CONTROL` modifiers — use `VIRTKEY` instead
- DIALOGEX coordinates must fit signed 16-bit (-32768 to 32767)
- FILEVERSION/PRODUCTVERSION fields must fit 16 bits (max 65535)

## QEMU Development Workflow

```bash
# ARM64 — UEFI ramdisk boot
bash qemu/run.sh --arch arm64                  # Serial console
bash qemu/run.sh --arch arm64 --gui            # GUI with virtio input

# amd64 — LiveCD ISO boot
bash qemu/run.sh --arch amd64                  # Serial console

# i386 — LiveCD ISO boot
bash qemu/run.sh --arch i386                   # Serial console

# Common options
bash qemu/run.sh --arch arm64 --debug          # Pause for GDB on :1234
bash qemu/run.sh --arch arm64 --log            # Enable QEMU tracing
```

### ARM64 Ramdisk Update
```bash
mcopy -o -i qemu/ramdisk.img build-arm64/path/to/file.exe ::/reactos/system32/file.exe
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img
# WARNING: Do NOT mcopy uefildr.efi into boot.img AFTER ramdisk — corrupts FAT32
```

### Symbolication
```bash
bash qemu/sym.sh 0xC804 0x50FC    # Look up offsets from UEFI bootloader (uefildr) crash dumps
# Formula: runtime_addr - module_base + PE_ImageBase = PE VA
```

(For ntoskrnl symbol lookups use the kernel debug toolkit's `addr2sym.sh`/`sym2addr.sh` — see next section.)

## Kernel Debugging Infrastructure (qemu/start.sh, inspect.sh, run-until.sh)

**For triaging kernel bugs (BUGCHECKs, hangs, regressions), prefer this toolkit over `qemu/run.sh` + ad-hoc DbgPrint instrumentation.** It is agent-driven: every operation is a single batch command in, structured result out, no interactive GDB sessions to babysit. The scripts are `start.sh` / `stop.sh` / `wait-for.sh` / `tail.sh` / `status.sh` / `inspect.sh` (bt, symbolicate-pc, dump-thread, dump-trapframe, pool-tag, identify-object, triage-bugcheck) / `run-until.sh` / `ps` / `build-index.sh` / `sym2addr.sh` / `addr2sym.sh` / `preflight.sh`.

**Standard recipe (validated end-to-end):**
```
RUN=$(qemu/start.sh amd64 --gdb)
qemu/wait-for.sh $RUN --pattern "ACPI Timer" --timeout 60
qemu/run-until.sh $RUN --break KeBugCheckEx --timeout 180 --then triage-bugcheck
```
On the amd64 KiUnwaitThread BUGCHECK flake, this returns the stop code, exception code, faulting PC + symbol, full KTRAP_FRAME, and `identify-object` on the bogus pointer — all in one tool call.

Full toolkit reference + design: `docs/engineering/kernel-debugging-infra.md` and `qemu/README.md`. (`qemu/ps` shows every live QEMU across all worktrees + slot usage; QEMUs are capped via `VIBEOS_QEMU_MAX_CONCURRENT`/`VIBEOS_QEMU_MAX_LIFETIME`; **same-arch i386/amd64 boots run in parallel on the default `windows-host` TCG backend** — the per-arch sub-cap is bypassed there (`vm.py:1295-1299`) and only serializes the accelerator-sharing backends (native+KVM, windows-host+WHPX); see the North Star note + `qemu/README.md` → Resource management); run `qemu/build-index.sh <arch>` after each kernel build for offline sym lookup.)

## Test Suite

### Rust Host Tests (no QEMU needed)

```bash
cd sdk/lib/rust

# Unit tests — algorithm correctness, pack/unpack roundtrips
cargo test -p slist

# Kani model checking — exhaustive verification of state transitions
cargo kani -p slist

# Loom concurrency tests — explores all thread interleavings
cargo test -p slist-loom
```

### QEMU Kernel Tests (28 tests via serial dispatch loop)

```bash
# 1. Build test kernel (canonical wrapper — see Build Commands)
cd build-arm64 && ../tools/canon-run.sh sh -c 'cmake . -DREACTOS_TEST_KERNEL=ON && ninja ntoskrnl'
cd ..

# 2. Update ramdisk
mcopy -o -i qemu/ramdisk.img build-arm64/ntoskrnl/ntoskrnl.exe ::/reactos/system32/ntoskrnl.exe
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img

# 3. Run tests (boots QEMU, sends commands over serial, checks [TEST PASS])
.venv/bin/python -m pytest tests/qemu/test_kernel.py --test-kernel -v       # All 28 tests
.venv/bin/python -m pytest tests/qemu/test_kernel.py --test-kernel -k SList # Just SList
```

### QEMU Smoke Tests (boot verification — see "Smoke Tests — The North Star" above)

```bash
source .venv/bin/activate

# Full test (clean Rust rebuild + boot + static analysis) — ~5 min per arch
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 600
pytest tests/qemu/test_smoke.py -v --arch amd64 --timeout 600

# Fast iteration (skip nuclear rebuild) — ~30s per arch
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 300 --skip-clean-build

# Static analysis only (no QEMU boot)
pytest tests/qemu/test_smoke.py -v --arch i386 -k "sacred_primitives or no_unexpected_ud2" --skip-clean-build
```

**IMPORTANT:** Cargo's incremental cache can mask regressions. Always run WITHOUT `--skip-clean-build` for final verification before committing. The clean build nukes `rust-kernel-target/` to ensure deterministic results.

Smoke/CLI harness internals (`vm.py`/`conftest.py`/`serial_client.py`/`qmp_client.py`/`debugcon.py`, boot media, QEMU backends + acceleration (windows-host TCG default / native KVM / WHPX opt-in), resource caps, debugcon boot tracing, test lifecycle): `tests/qemu/cli/README.md`.

## Rust in the Kernel

The Cargo workspace at `sdk/lib/rust/` holds Rust components that compile to PE-COFF static libraries (rlibs/staticlibs) and link into the cmake build. **All crates build for all 3 architectures unless explicitly noted.** The workspace long ago outgrew "kernel algorithms": it spans the bootloader (`freeldr-rust`), the HAL (`hal-rust`, all three arches), win32k (`win32k-rust`), user-mode components (`winlogon-rust`, `explorer`-side crates, server DLLs), and a growing driver fleet (`uniata`, the `usbport`/`usbhub`/HID stack, netstack, `disk`/`classpnp`, …).

```
sdk/lib/rust/
├── Cargo.toml              # Workspace (resolver v2). lto=false + codegen-units=256 firewall.
├── ntoskrnl-rust/          # The (shrinking) kernel-monolith crate — ~130K LOC (was ~456K).
│                           #   Being split per-subsystem (crate-split, ~70% done): what REMAINS
│                           #   here is the dense center — ke, mm/ARM3. The EXTRACTED crates
│                           #   (iomgr, ex, se, config, pnpmgr, ps, ob, fsrtl, kdbg, fstub,
│                           #   kd/kd64, rtlk, cc, po, dbgk, lpc, wmi, inbv, vdm, vf, …) talk to
│                           #   it over the FFI symbol boundary, NOT crate:: paths.
│                           #   See docs/engineering/ntoskrnl-crate-split-plan.md.
├── {iomgr,ex,se,config,pnpmgr,ps,ob,fsrtl,kdbg,fstub,kd,kd64,rtlk,cc,po,dbgk,lpc,wmi,inbv,vdm,vf}-rust/
│                           # Subsystems peeled out of the ntoskrnl-rust monolith (crate-split).
│                           #   Each deps ONLY ntkrnl-sys + dbgprint-rust; forced into the link by
│                           #   kernel-rust's `extern crate` (37 rlibs today). Extract another via
│                           #   the plan's recipe.
├── ntoskrnl-cargo/         # PRODUCTION cargo-driven ntoskrnl.exe link (graduated from cmake
│                           #   add_executable — reactos-link engine; ntoskrnl/CMakeLists.txt:118)
├── ntkrnlmp-cargo/         # SMP-kernel sibling link (ntkrnlmp.exe): kernel asm assembled with
│                           #   CONFIG_SMP. ON MASTER since 2026-07-11 (M0–M3 complete: -smp 2
│                           #   boots to desktop, both CPUs schedule, TLB shootdown + generic-call
│                           #   IPI live; all opt-in via `build --smp-kernel` + `smoke --smp N`,
│                           #   defaults stay UP). Quest log + remaining M4/M5 work:
│                           #   docs/engineering/active-lanes/smp-bringup-quest.md
├── kernel-rust/            # staticlib umbrella that pulls in all sibling rlibs via extern crate
├── ntkrnl-sys/             # Two-layer FFI crate: hand-written + bindgen (see below)
├── hal-rust/               # HAL Rust translation — x86 flavors (i386 + amd64, minihal feature
│                           #   for FreeLDR) AND arm64 (GIC/timer/DMA-adapter/PL031 modules)
├── rtl-rust/               # Umbrella over the algorithm rlibs from sdk/lib/rtl/* (~39 crates)
├── slist/                   # Lock-free SList (LDXP/CMPXCHG16B/CMPXCHG8B inline asm)
├── slist-loom/              # Loom concurrency tests for SList (std, not linked into kernel)
├── bitmap/                  # RtlBitmap operations
├── avltable/, splaytree/, generictable/  # Generic tree algorithms (used by mm, cm, etc.)
├── mm-avl/                  # Mm-flavor AVL on MMADDRESS_NODE (VadRoot etc.)
├── largeint/, time/, random/, crc32/, encode/, compress/, mem/, byteswap/,
│   utf8/, prefix/, luid/, access/, handle/, interlck/  # smaller algorithm crates
├── ke-inlines/, ke-macros/  # extracted ntoskrnl-rust ke/ helpers as separate rlibs
└── ir-output/              # rustc IR dumping helper
```

The structure isn't an exhaustive list — the workspace has ~250 members today. The principle is: anything that's pure-algorithm with a clean FFI surface gets factored into its own rlib so the workspace `lto=false` + `cgu=256` firewall keeps callers and callees in separate codegen units. (That firewall's *original* job — blocking LLVM's captures(none) volatile-blind miscompile across CGU boundaries — is **toolchain-fixed on LLVM ≥ 21**: we ship rustc 1.92 / LLVM 21.1.3, which respects volatile stores in attribute deduction (see rust-kernel-patterns.md "RESOLVED AT THE TOOLCHAIN"). The firewall *stays* as belt-and-suspenders and because it still firewalls the **toolchain-independent** ABI-mismatch `noreturn`/ud2 cascade; `tools/scan_captures.py` is the tripwire to rerun after each toolchain bump in case a future LLVM regresses it.) The **crate-split** effort (docs/engineering/ntoskrnl-crate-split-plan.md) applies that same principle to the kernel itself for a second reason — cutting the edit→rebuild loop, since rustc's frontend is whole-crate.

### Rust Targets (GNU/LLVM, not MSVC)
- **ARM64:** `aarch64-pc-windows-gnullvm`
- **amd64:** `x86_64-pc-windows-gnu`
- **i386:** `i686-pc-windows-gnu`

### Reusable Cross-Platform Pattern
```
component/src/
  lib.rs        — FFI (#[no_mangle] extern "C", cfg for arch-specific conventions)
  algorithm.rs  — Pure logic (no unsafe, no asm, no cfg — Kani-verifiable)
  types.rs      — Shared types (cfg for platform layout variants)
  atomics.rs    — Platform asm (cfg(target_arch), common interface)
```

Principles:
1. **Algorithm is pure** — testable, Kani-verifiable, identical on all platforms
2. **Types use cfg at the boundary** — pack/unpack hides layout differences
3. **Atomics provide a uniform interface** — loop structure is identical per arch
4. **FFI is thin glue** — combines atomics + algorithm, minimal logic
5. **One crate, three targets** — same Cargo.toml for all

### Key Rust/FFI Gotchas
- **LLVM reserves rbx (amd64) and ebx (i386)** — use `xchg` workaround for CMPXCHG16B/CMPXCHG8B inline asm. **On amd64, the xchg MUST be 64-bit** (`xchg rbx, rbx_save`), not 32-bit (`xchg ebx, ebx_save:e`) — a 32-bit register write zero-extends into rbx, silently destroying the upper 32 bits of any caller-held pointer there. Cost a multi-hour BUGCHECK 0x50 debug session in 2026-05.
- **`#[no_mangle] extern "stdcall"`** on i686-gnu correctly produces `_Name@N` decoration
- **`#[export_name]` gets double-decorated** on i686-gnu with stdcall — use `#[no_mangle]` instead
- **i386 SList Pop must be `global_asm!`** — `KiCheckForSListFault` checks exact instruction bytes (0x8B 0x18)
- **NTAPI vs cdecl ABI** matters per arch: i386 NTAPI = stdcall (`_Name@N`), amd64/arm64 NTAPI = cdecl (just `Name`). `extern "system"` resolves to the right thing per arch. Mismatched `extern "C"` vs `extern "system"` declarations across rlibs link silently on amd64/arm64 but break i386. Detect with `tools/rustconv/rustconv/verify_ffi.py` (the gate's workspace-wide ABI checker; `verify_abi.py` is the narrower calling-convention-only variant it grew out of).
- **Cross-DLL data imports** (HAL referencing kernel-data exports like `KdDebuggerNotPresent`, `KeServiceDescriptorTable`): need `#[link(name = "ntoskrnl", kind = "raw-dylib")]` on the extern static block. `postprocess_bindgen.py --hal` adds this automatically for the 41 affected externs.

### ntkrnl-sys — Kernel FFI Declarations Crate (overview)

Dual-layer FFI crate at `sdk/lib/rust/ntkrnl-sys/` providing kernel type and function declarations:
- **Layer 1 — hand-written modules** (`ntkrnl_sys::base`, `::ke`, `::mm`, `::ex`, `::ps`, `::ob`, `::rtl`, `::interlocked`): curated declarations, inline functions, macros, and convenience wrappers bindgen can't generate.
- **Layer 2 — bindgen kernel module** (`ntkrnl_sys::ddk::*`): auto-generated from `ntoskrnl.h` + `mm/ARM3/miarm.h`, covering all internal subsystems (~3,300 extern fns, ~1,850 structs, ~4,400 type aliases per arch, with correct i386 fastcall/stdcall conventions). ~1,500 compile-time `size_of` assertions per arch verify C layout.

Full reference (directory tree + line counts, size-verification `verify_sizes.py`, bindgen regeneration via `generate_bindings.sh`, limitations), plus the CMake build-integration mechanics and the "Adding New Crates" steps: `docs/engineering/rust-in-kernel.md`.

### Cohabitation pattern (HAL)

For HAL Rust translation, each .c → .rs swap is independently revertible: the C file gets commented out in the per-flavor cmake (`hal/halx86/{generic,legacy,pic,acpi,apic}.cmake`) with a `# replaced by Rust (path)` line, and the Rust file's `#[no_mangle]` symbol takes over. `--allow-multiple-definition` allows side-by-side C+Rust during the swap-in phase; lld first-match wins. (Killing that flag outright is a parked lane — branch `kill-allow-multiple-def`, resume state in-branch.) See `docs/engineering/rust-in-kernel.md` → "HAL ABI contract" for the durable ABI reference, and `docs/engineering/archive/hal-translation-audit.md` for the (closed) per-file mapping.

## rustconv — C-to-Rust Translation Pipeline

API-driven pipeline that translates ntoskrnl and HAL C files to Rust using LLM (Sonnet batch API by default; also supports Haiku, Opus, Gemini Pro/Flash, DeepSeek). Located at `tools/rustconv/`.

Full pipeline internals, post-processor rules, probe/Stage-3 design, and the Codex cross-model path: `docs/engineering/rustconv-pipeline.md`.

### CLI

```bash
source .venv/bin/activate

python -m rustconv scan ntoskrnl/ex/uuid.c            # Parse, print analysis
python -m rustconv resolve ntoskrnl/ex/uuid.c          # Resolve deps → import manifest
python -m rustconv prompt ntoskrnl/ex/uuid.c           # Generate prompt to stdout
python -m rustconv estimate ntoskrnl/ob/               # Token count + cost estimate
python -m rustconv submit ntoskrnl/ex/uuid.c -o out.rs # Single-file API call
python -m rustconv submit ntoskrnl/ex/uuid.c --assemble  # Translate + write to crate
python -m rustconv translate ntoskrnl/ob/file.c        # Single-file synchronous (better for >1000 LOC; batch silently stubs lazy fns)
python -m rustconv batch ntoskrnl/ob/ -m sonnet        # Batch entire subsystem (smaller files only)
python -m rustconv probe-migrate se/sqos.rs            # Stage 2: ProbeForRead/Write → Result<T>
python -m rustconv trace-catalog                        # Generate debugcon trace catalog from inline asm
python -m rustconv check                                # cargo check ×3 targets
python -m rustconv candidate-check candidates.json --repair  # Batched x3 candidate oracle
python -m rustconv substrate-plan path/to/report.json   # Cluster compiler roots by authority
```

> **⚠️ `translate`/`submit` SPEND REAL MONEY — read this before running them.** Multiple agents have wasted >$100 by killing translations mid-run and losing the output. The rules:
> - **NEVER Ctrl-C / kill a `translate` or `submit` mid-call to "adjust something."** Let it finish. `translate` makes one paid call **per function** — kill it at function 35/40 and you re-pay for all 35 on the retry. If you must change something, wait for it to finish, *then* re-run.
> - **Output is now saved as it streams** to `tools/rustconv/.responses/<ts>-<file>/` (gitignored, in-repo, survives kills/reboots). Each function's Rust lands there the instant it returns; the in-flight one is preserved as a `.partial`. That dir is the **canonical copy** — never write translations to `/tmp` (`-o /tmp/...` is flagged; the durable copy in `.responses/` exists regardless). Do not delete a `.responses/` run dir that holds unintegrated work.
> - **Cost estimates are a FLOOR, not a quote.** `estimate` shows the single-prompt cost; `translate` re-sends context per function, so the real bill is routinely **2–5×** higher, and `-m opus` is **full sync price ($5/$25 per 1M), not the batch Sonnet price** in the table below. Watch the live `[running $X.XX]` counter the command prints — that's the truth. Budget with the "sync @5x" column from `estimate`.
> - **Don't re-translate a file that already has output** (in `.responses/` or already integrated) without explicit user approval — you're re-paying for work that exists.

Model shortcuts: `sonnet`, `haiku`, `opus`, `gemini-pro`, `gemini-flash`, `deepseek`, or full model ID.

`rustconv translate` accepts HAL paths and is HAL-aware (separate `system_prompt_hal.md`, hal_ddk imports, auto-CMake edit). Output quality is ~70% useful for HAL — review before integrating.

The pipeline has multiple stages: Stage 1 (C → Rust, `submit`/`batch`), Stage 2 (probe migration, `probe-migrate` — `ProbeForRead`/`Write` → `Result<T>`; SE+OB done), Stage 3 (Result+Drop-RAII idiomation, planned). See `docs/engineering/rustconv-pipeline.md` for the per-stage design.

### Deterministic substrate closure

After assembling a candidate, run `candidate-check --repair`, then use the embedded report or
`substrate-plan` to fix distinct compiler roots by authority—not hundreds of individual rustc
sites. Full-module manifest entries should include `"source"` so source-local definitions are
routed correctly. Reuse/promote canonical Rust first, then project bindings, SDK/header
generation, explicit local/bridge generation, and scope repair. Constants, SDK layouts, WIDL
interfaces, vtables, ABI, and imports never go through the LLM. Ambiguous layouts stay red;
same-arity local overloads and trailing C++ default arguments are likewise resolved and
expanded deterministically before payment. If preflight cannot prove one binding, extend the
harness or leave the class red; never ask the model to guess C++ compiler semantics.
Sonnet is considered only after substrate closure leaves a genuine semantic/structural
residual. The complete protocol and stop rules are in
`docs/engineering/rustconv-substrate-closure-protocol.md`.

### Cost (Sonnet **Batch** API — the cheapest path, NOT what you pay synchronously)

These are **batch Sonnet** figures (50% off, async). The synchronous `translate`/`submit`
path you actually use pays **full list price**, and `-m opus` is **~3.3× the Sonnet rate**
($5/$25 vs $3/$15 per 1M) on top of losing the batch discount — so a "Full ntoskrnl"
Opus-synchronous run with per-function fan-out is an order of magnitude past the bottom row.
Always sanity-check against `rustconv estimate`'s "sync @5x" column and the live cost counter.

| Scope | Est. Cost (batch Sonnet, floor) |
|-------|-----------|
| Single file | ~$0.03 |
| Subsystem (11 files) | ~$0.30 |
| Full ntoskrnl (393 files) | ~$8.70 |
| 5 iterations | ~$43.50 |

### Static Analysis Tools

```bash
# Check for LLVM miscompilation (ud2 in named functions)
python3 tools/scan_ud2.py build-i386              # Report all ud2
python3 tools/scan_ud2.py --check build-i386      # Exit 1 if unexpected ud2

# Check for unprotected output pointer writes (captures(none) vulnerable)
python3 tools/lint_kernel.py                       # Report all violations
python3 tools/lint_kernel.py --fix                 # Show suggested fixes
python3 tools/lint_kernel.py path/to/file.rs       # Single file

# Check sacred primitives survive optimization (KfRaiseIrql, KeAcquireSpinLock)
bash tools/verify_sacred_primitives.sh build-i386

# Check constant values against C headers
.venv/bin/python tools/rustconv/rustconv/verify_constants.py                    # Check all
.venv/bin/python tools/rustconv/rustconv/verify_constants.py --fix              # Auto-fix

# Check ABI mismatches
.venv/bin/python tools/rustconv/rustconv/verify_ffi.py

# Silent-failure shapes: bare loop{} spins, unwrap/expect in kernel code,
# swallowed Err arms (AST/tree-sitter; wired into `vibeos-cli check`, 0-NEW gate)
.venv/bin/python tools/scan_silent_failures.py           # sweep no-std workspace
# Suppress a sanctioned loud halt: // nolint: silent-failure (reason)

# Hand-written NUMERIC struct offsets (the MMPFN offset-shift / KTHREAD.Teb-0x1e8
# drift class — verify_offsets.py proves bindgen DECLS, this catches USAGE sites;
# fix = core::mem::offset_of!(bindgen type, field), never a bare number)
python3 tools/scan_magic_offsets.py                      # sweep sdk/lib/rust
# Suppress a verified FP: // nolint: magic-offset (reason)

# Boot coverage map: which functions a real boot EXERCISES (TCG plugin boot)
tools/coverage/run.sh amd64        # -> per-function exercised set + the
                                   #    never-executed-rust.txt landmine registry

# captures(none)-class post-opt IR audit (POISON_ARG committed folds /
# READNONE_OUTPARAM latents / UNREACHABLE_BODY). Class is toolchain-FIXED on
# LLVM >= 21 — rerun this after every rustc/LLVM bump to prove it stayed fixed.
VIBEOS_EMIT_LLVM_IR=1 ./vibeos-cli build --arch amd64   # emit per-CGU .ll
.venv/bin/python tools/scan_captures.py sdk/lib/rust/target/ntoskrnl-cargo-*-rustlib
# Related: VIBEOS_LINK_MAP=1 (lld -Map per cargo module — layout ground truth);
# VIBEOS_LAYOUT_CANARY=1 (deliberately perturbed kernel link — boot tripwire)
```

`scan_ud2.py` finds `ud2` (0f 0b) in named functions — LLVM proved the body has UB (on LLVM ≥ 21 almost always an ABI mismatch; the captures(none) volatile-blind root is toolchain-fixed, see rust-kernel-patterns.md "RESOLVED AT THE TOOLCHAIN"); it's wired into smoke as `test_no_unexpected_ud2`. `lint_kernel.py` finds `*OutputParam = value` writes in `#[no_mangle]` functions that skip `write_volatile` (a captures(none) target) before compilation. (The smoke harness also nukes `rust-kernel-target/` per session unless `--skip-clean-build`, runs the ud2 scan pre-boot, and auto-symbolicates BSODs — see `tests/qemu/cli/README.md`.)

## ARM64 Specifics

### Memory Layout
```
User space:     0x0000000000000000 - 0x0000FFFFFFFFFFFF
Kernel (KSEG0): 0xFFFFF80000000000
HAL VA:         0xFFFFFFFFFFC00000 - 0xFFFFFFFFFFFFFFFF
```

### QEMU virt
RAM at physical 0x40000000 (1GB). PL011 UART at 0x09000000. Serial via Unix socket at `qemu/serial.sock`.

### UART Trace Pattern (kernel debugging)
```c
extern volatile PVOID Arm64UartKernelBase;
volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
if (u) { while (*msg) *u = (ULONG)*msg++; }
```

### ESR Exception Classes
0x25=data abort, 0x21=instr abort, 0x3C=brk, 0x15=SVC

## Target Hardware

All development and CI happens in **QEMU**: `virt` machine (ARM64), `q35` (amd64), or standard PC (i386).

**Acceleration / backend (this misled people — read it):** the default smoke backend is **`windows-host`** — the harness launches the **Windows** `qemu-system-*.exe` on the host via WSL interop (media staged to `/mnt/c/temp/vibeos-qemu`), running **TCG (pure emulation) on all three arches**. This deliberately escapes WSL2 nested-virt: host TCG beats nested KVM, and there's no single-virt-slot contention, so the beast host can run all arches in parallel. **We are *not* on `/dev/kvm`** despite it being present in WSL2. Two alternates exist: `VIBEOS_QEMU_BACKEND=native` (local Linux qemu + Unix sockets + KVM for x86 — the old path, still used by the kernel-debug toolkit's GDB flow), and **WHPX** (Windows Hypervisor Platform, real x86 hardware virt in `qemu-system-x86_64.exe`, `accel: "whpx"`) — **works since 2026-07-10: both x86 arches boot to desktop in ~52s** (`VIBEOS_QEMU_WINDOWS_ACCEL=whpx`; the boot CD rides an ich9-ahci controller under whpx so OVMF DMAs it instead of ~5min of PIO VM-exits — see `docs/engineering/windows-host-qemu-whpx-spike.md` + field manual §9.46 for the six guest-side bugs that bring-up flushed out). Still opt-in (default stays TCG) until whpx accumulates reliability history; same-arch whpx boots serialize (hardware-virt contention, like KVM). arm64 is always TCG (WHPX is x86-only). Config + per-arch resolution: `tests/qemu/cli/config.py` (`qemu.backend`/`qemu.windows.accel`) + `tests/qemu/backend.py` (`resolve_accel`). Real-hardware boot (e.g., Raspberry Pi for ARM64) is an aspirational target, not a current one.

## Notes
Don't ask the user to run commands or install things after they have told you to debug something, just debug it.

Use the API keys in .env when necessary. Use AI and the process we've developed in rustconv to automate the process of doing translations in bulk. If you have to work on multiple files and there are signficant (50+) number of edits, and it would be good learnings to incorporate into future refactors, then we should automate it. When we already have the AI output of the c->rust or rust->rust translation and you are just integrating it, then go ahead and do it manually.
