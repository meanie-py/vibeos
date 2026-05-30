# Plan: `vibeos-cli` — unified build / check / smoke / debug CLI

**Status:** In progress (design consolidated 2026-05-30). Supersedes the original
"port ps.sh to Python" scope. `qemu/ps` is already done (first piece). This plan
is the agreed design for the full tool.

`vibeos-cli` is the single Python front door for the whole dev loop: build the OS,
lint it, smoke-boot it, and debug it — replacing the pile of `qemu/*.sh` scripts
and the bare `pytest tests/qemu/...` invocation. Shell scripts retire to shims and
are deleted per-command only after each is proven on a real run.

---

## 1. Why

The smoke harness (`tests/qemu/*.py`) and the shell toolkit (`qemu/*.sh`)
independently reimplement the same logic and must be hand-kept in sync; shell is
also actively fragile for this domain. Evidence from building the resource layer:

- **Duplicated resource layer** — slot pool, reaper, owner-pid walk, deadline cap,
  lockfile schema all exist twice (bash + Python), hand-matched.
- **Shell bugs that Python doesn't have**, both hit in one session:
  - `ps.sh` `$(( now - deadline ))` crashed on the **float** deadline the harness
    writes — silently blanked the row; the same float broke `reap.sh`'s cap
    enforcement for smoke VMs (fixed in `57f3fcb`).
  - multibyte `printf %-Ns` misaligned columns.
- `jq`-per-field forking (`reap.sh`: 8/iteration), no structured data.

A Python CLI parses JSON natively, unit-tests cleanly, and gives one coherent
front door with a token-frugal output contract built for agents.

---

## 2. Principles

1. **One front door.** `vibeos-cli {build,check,smoke,ps,debug}`. `smoke` replaces
   direct `pytest` calls; `build` replaces ad-hoc `ninja`; `debug` replaces the
   `start/inspect/run-until/...` scripts.
2. **No default arch.** `--arch {i386,amd64,arm64}` is required for build/smoke/debug.
   An `all` (parallel) mode is future, not now.
3. **Natural extensions auto-chain; prereq boundaries hard-fail.** `build` auto-runs
   `check` + `build-index`; `smoke` auto-runs triage on failure. But crossing a
   boundary with unmet prereqs fails: `smoke` on stale/missing media fails and tells
   you to `build` (override `--allow-stale` for bisecting agents).
4. **Advisory vs blocking.** Translation lints (`lint_kernel`, `verify_abi/ffi/
   constants`) are **advisory** — they print but never fail the run (respects
   "a broken Rust translation integrated and failing is the correct state").
   Only hard errors (shell/Python syntax, shellcheck errors) and the staleness gate
   block.
5. **Token-frugal output.** Every command prints a compact summary to stdout and
   writes the full detail to a log file. Agents read the summary; they `grep`/`Read`
   the log only on failure.
6. **Whole-tree lint.** Not diff-scoped — linting isn't the bottleneck.
7. **Delegate, don't reinvent.** ninja is the manifest for its DAG; git is the
   manifest for source content; we only track what they can't see / can't be
   trusted for.

---

## 3. Output contract (summary + log file)

**stdout = compact summary (~12 lines)**, with the few salient failing lines inline
so an agent usually doesn't open the log:

```
build i386  —  cold · ninja OK · 5 lint ok · 1 warn   (2m41s)
  ✓ shellcheck   ✓ verify_abi   ✓ verify_ffi   ✓ verify_const
  ⚠ lint_kernel (3 unprotected writes)   ✓ scan_ud2   ✓ build-index
full log:  qemu/vibeos-build-i386-0530-1412.log
```
```
smoke i386  —  FAIL: livelock   (1m48s)
  staleness: ✓ current (cold build @ a2d5a6e)
  hang: spinning in KiReadyThread+0x3c after [CSRSS], repeating "I/J/i/#"
  bt: KiReadyThread ← KiUnwaitThread ← KeSetEvent ← …
full log:    qemu/vibeos-smoke-i386-0530-1420.log  (4,201 lines)
live serial: qemu/i386_smoke_live.log
```

**Full log** = every stage's raw output under headers (`=== build (ninja livecd) —
exit 0, 41.2s ===`), greppable. Logs at `qemu/vibeos-<sub>-<arch>-<ts>.log`, the
build stamp at `build-<arch>/.vibeos-build-stamp.json` — both gitignored.

Exit code: nonzero on hard FAIL, zero on WARN-only.

---

## 4. Subcommand surface

```
vibeos-cli build  --arch X [--clean] [--no-check]   # ninja → binary-lints → build-index → stamp
vibeos-cli check  --arch X                          # lints only, no build, no stamp
vibeos-cli smoke  --arch X [--allow-stale] [--no-gdb] [--attach-on-fail]
vibeos-cli ps [--json]                              # all runs (smoke+debug) across worktrees + slots  [DONE]
vibeos-cli debug start    --arch X [--gdb-wait] [--allow-stale]   # investigative boot; prints run-id
vibeos-cli debug attach   <run-id> --command triage-bugcheck|bt|dump-thread|… [--json]
vibeos-cli debug run-until <run-id> --break SYM --timeout S --then <cmd>…
vibeos-cli debug wait-for | tail | status | stop <run-id>
vibeos-cli sym addr2sym | sym2addr <addr> --arch X
```

`build` is the everyday path; `check` is for linting without rebuilding.
`smoke` is the verdict path; `debug` is the investigation surface (§8).

---

## 5. Staleness & build-provenance model

A smoke run can lie in four independent ways:

| Layer | Detects via | Blind to |
|---|---|---|
| **C / cmake / packaging** | reconfigure + `ninja -n <target>` (post-reconfigure dirty set) | cargo/rust |
| **Rust source → `.a`** | `rust_src_hash` (git tree hash of rust dirs) | incremental masking |
| **Cargo incremental masking** | warm/cold provenance in the stamp | — |
| **Source-tree state in general** | git fingerprint (HEAD + dirty hash) | nothing build-relevant |

**Don't reinvent a file inventory** — ninja already tracks its DAG, git already
tracks source content. The stamp records only the gaps.

### The stamp (rewritten atomically at the end of each successful `build`)

`build-<arch>/.vibeos-build-stamp.json`:
```json
{ "arch":"i386", "ts":…, "flags": { "clean": false },
  "git":  { "head":"…", "dirty_hash": "sha256(git status --porcelain + git diff)" },
  "rust_src_hash": "sha256(git tree of sdk/lib/rust + *-rust dirs)",
  "cold": false, "rust_targets_nuked": [ …discovered dynamically… ],
  "boot_media": "build-i386/livecd.iso", "boot_media_sha": "…",
  "staged_bins": { "ntoskrnl.exe": mtime, … }   // arm64 re-stage check
}
```
A failed build does **not** update the stamp (staleness keeps pointing at the last
good build).

### Staleness check — tiered, cheap-first

1. Recompute git fingerprint; **matches stamp → FRESH, done** (instant; the common
   case — built and not edited — needs no ninja/cargo I/O).
2. Differs → escalate to authoritative per-layer:
   - reconfigure + `ninja -n <boot-target>` has work → **stale** (C/packaging).
   - `rust_src_hash` changed → **stale** (rust); if last build warm, also flag masking.
   - arm64: any `staged_bins` mtime > `boot.img` mtime → **stale** (re-stage forgotten).
   - none of those → build-irrelevant change (docs/this plan) → **not stale**.
3. `cold == false` → **WARN warm** regardless (advisory, never blocks).
4. boot media missing or `boot_media_sha` mismatch → stale/absent.

### Why git fingerprint, not mtime

`git checkout` / branch switch / `git worktree` operations rewrite file mtimes
wholesale — an mtime-based source-vs-media check is unreliable *exactly* in the
multi-worktree workflow. A git content fingerprint is immune. mtime survives only
in the arm64 binary-vs-`boot.img` check, where both are build outputs in one
worktree that git never touches.

### Dynamic target discovery

`--clean` and the staleness/stamp logic discover rust target dirs at runtime
(`find build-<arch> -name 'rust-*-target' -o -name 'rust-target'`) — there are ~6
per arch (ntoskrnl, hal, hal-mini, freeldr, rtl, win32k), not 1. No hardcoded list,
so new crates auto-expand the nuke/staleness set.

---

## 6. Per-arch boot media (build is NOT one shape)

**i386 / amd64 — `ninja livecd`** → `build-<arch>/livecd.iso` (loaded `-cdrom`).
One target; the `livecd` DAG depends on the full module set, so ninja rebuilds the
binaries *and* re-rolls the iso. Packaging staleness fully ninja-tracked.

**arm64 — no iso.** Boots `qemu/boot.img` (⊃ `ramdisk.img`), which live in `qemu/`,
not `build-arm64/`. Two steps:
1. `ninja ntoskrnl hal uefildr arm64input arm64vid …` (arm64 drivers best-effort —
   absent on some branches; build the **full staged set** so a refresh never folds
   in a stale sibling — more thorough than today's conftest `ninja ntoskrnl`).
2. **`qemu/mkramdisk.sh --refresh`** — re-stage built binaries into the existing
   `ramdisk.img`, fold into `boot.img`. (mkramdisk's own header: "the arm64
   structural equivalent of `ninja livecd`.") If `ramdisk.img` is absent, fall back
   to a full `mkramdisk.sh`.

The re-stage is the #1 arm64 stale trap: ninja rebuilds `ntoskrnl.exe` but nothing
re-stages it → smoke boots the old kernel. Hence the arm64 staleness check compares
**built-binary mtimes** against `boot.img`, not just source.

---

## 7. build / check / smoke semantics

- **`build --arch X`** — warm/incremental by default; `--clean` nukes all discovered
  rust target dirs (cold). Runs ninja per §6, then the **binary-lints** (`scan_ud2`,
  `verify_sacred_primitives`), then **`build-index`** (so symbolication is always
  ready — kills "forgot to build-index"), then writes the stamp. Auto-runs `check`
  unless `--no-check`. A warm build prints a warning.
- **`check --arch X`** — source lints always (`shellcheck`, py-syntax, `verify_abi`,
  `verify_ffi`, `verify_constants`, `lint_kernel`); binary-lints if a binary exists,
  else skip with a note. No build, no stamp.
- **`smoke --arch X`** — staleness preflight (§5): **hard-fail on stale/missing**
  media (`--allow-stale` overrides for bisect), **WARN on warm**. Boots with a
  gdbstub by default (`--no-gdb` to opt out) so failures auto-triage (§8/§9).
  Delegates the boot+asserts to the pytest harness, wraps it into the summary+log.
  On failure: auto-triage; `--attach-on-fail` preserves the VM (run-id + gdbstub)
  for `debug attach`, otherwise tears down after symbolicating.

---

## 8. Debug integration — two creators, one engine

Smoke and debug are **siblings sharing one inspection engine**, not parent/child.

- **Creators:** `smoke` (verdict; auto-triage on fail; `--attach-on-fail` preserves)
  and `debug start` (investigate; `--gdb-wait` halt-at-first-instruction; stays
  alive; no verdict — the early-bringup / step-from-zero workflow smoke can't express).
- **Shared engine (`gdb.py`):** build a gdb command, run it against a gdbstub port,
  parse + symbolicate with `[Rust]/[C]/[ASM]` tags. Boot-model-agnostic — it just
  needs a port. smoke calls it on failure; `debug` calls it on demand.
- **Run-scoped verbs (grouped under `debug`):** `attach`, `run-until`, `inspect`
  (`bt`, `dump-thread`, `dump-trapframe`, `pool-tag`, `identify-object`,
  `triage-bugcheck`, `symbolicate-pc`, …), `wait-for`, `tail`, `status`, `stop`.
  They operate on **any** run-id — a preserved smoke failure or a `debug start`
  daemon. The bridge: `smoke --attach-on-fail` → run-id → `debug attach <run-id>`.
- `debug start` inherits the staleness gate + `preflight.sh` gdb checks
  (gdb-multiarch, DWARF richness) — fail early with the fix command.

If `debug start` ever proves unused (most debugging is post-smoke-failure), collapse
it and `debug` becomes purely the run-scoped verbs. Keep it for now (arm64
step-from-zero, early init).

---

## 9. Hang monitor (continuous, in Python)

Replaces the coarse post-hoc CPU-idleness check with a continuous monitor that
classifies *and locates* hangs, feeding §8's auto-triage. A hang is four things:

| Mode | CPU | Serial | Classify by |
|---|---|---|---|
| stage stall | any | stops advancing | no new milestone in N s |
| spin / livelock | ~90% | silent | CPU% + PC confined to a small range |
| deadlock | ~0% | silent | CPU% + thread-state walk |
| looping output | high | repeating | serial tail cycle detection |

**Tiered, to not perturb healthy boots.** Cheap always-on signals (zero guest
impact): per-milestone progress watchdog (uses `required_milestones`), serial
cycle detection, QMP `query-status`, periodic CPU%. **Escalation** only when a stall
is suspected: gdb PC-sampling (briefly pauses the guest) → confine the PC to a
symbol = *where* it's spinning; for the idle case, walk threads = *what* they wait
on. Output is a classified verdict **with a location**, which is the input
auto-triage wants. The old `test_idle_cpu_not_spinning` becomes one signal in this
monitor.

---

## 10. Architecture & the two-lifecycle constraint

`QemuVM` (smoke, in-process: pytest holds QMP/serial-thread/slot-fd) and the debug
daemon (`debug start`, detached: VM outlives the launcher; slot held via an
fd QEMU inherits; cap via detached watchdog) are **genuinely different lifecycle
models** — do **not** unify them into one object. Only the **data/resource layer**
is shared. `gdb.py` is shared because it's port-keyed, not lifecycle-keyed.

```
tests/qemu/ (importable package, used by pytest AND the CLI)
  vm.py            QemuVM — in-process model (smoke)            [exists]
  locks.py   (new) manifest+lockfile schema, slot pool, reaper, owner-pid, deadline,
                   staleness/stamp, dynamic target discovery   (de-dups bash+Py)
  daemon.py  (new) detached-launch model (debug start)
  build.py   (new) per-arch ninja + arm64 mkramdisk + binary-lints + stamp
  lints.py   (new) source-lint runners (shellcheck/py/verify_*/lint_kernel)
  gdb.py     (new) inspection/triage engine (bt, dump-thread, triage-bugcheck, …)
  hang.py    (new) continuous hang monitor + classifier
  symbols.py (new) build-index / addr2sym / sym2addr
  cli.py     (new) argparse dispatcher: vibeos-cli <sub>
  serial_client.py qmp_client.py debugcon.py   [exist; reused by both models]
vibeos-cli  (new) one-line venv wrapper → python -m tests.qemu.cli
qemu/ps     [done]  — folds into `vibeos-cli ps`
qemu/*.sh           — become shims → vibeos-cli; deleted per-command once proven
```

---

## 11. Conftest blast radius (smoke stops building)

Nuking/building leaves smoke entirely — `build --clean` is the **only** nuker;
smoke only *observes* the stamp. This kills the "conftest nukes 1 of ~6 targets, in
the wrong place for worktrees" problem.

- Delete `_ensure_clean_rust_build` (the nuke + ninja) and the **`--skip-clean-build`
  flag** (meaningless once smoke never builds).
- Move `scan_ud2` / `verify_sacred_primitives` (`_run_post_build_checks`) into
  `build`'s binary-lint stage.
- Session fixture becomes: **staleness preflight → fail unless current (or
  `--allow-stale`) → boot (gdbstub on) → assert → on-fail auto-triage.**

Net agent flow: `vibeos-cli build --arch i386` (warm, fast, auto-lints+index) →
`vibeos-cli smoke --arch i386`; `build --clean` before a "is it really green" /
pre-commit run.

---

## 12. Migration phases (risk-ordered; shims until proven)

Cardinal rule: **don't break the debugger to make it prettier.** Each shell script
stays a shim, deleted only after its `vibeos-cli` equivalent survives a real run.

- **Done:** `qemu/ps` (Python).
- **P1 — spine (daily driver):** `locks.py` (de-dup + staleness/stamp), `build.py`
  (per-arch build + stamp), `lints.py`, `cli.py`, `vibeos-cli` wrapper; `build` +
  `check` + `smoke` (front door, staleness gate, summary+log). Rewire conftest (§11).
- **P2 — debug engine:** `gdb.py` + `daemon.py` + the `debug` group; wire smoke's
  auto-triage. Prove `triage-bugcheck` on a real bugcheck before deleting
  `inspect.sh`/`run-until.sh`.
- **P3 — hang monitor:** `hang.py`; replace the post-hoc CPU check.
- **P4 — symbols + cleanup:** `symbols.py`; retire remaining shims.

Resource layer (slots/reaper/cap/live-log/`ps`) already shipped (`e095381`,
`905d412`, `57f3fcb`) — this builds on it.

---

## 13. Acceptance criteria

1. One implementation of the resource + staleness layer (no shell/Python dup).
2. `vibeos-cli` covers build/check/smoke/ps/debug; `qemu/*.sh` are shims or removed.
3. All 3 arches' smoke stays green throughout; `build`/`smoke` per-arch boot media
   correct (x86 iso, arm64 ramdisk re-stage).
4. Staleness gate catches stale media on every arch (incl. arm64 re-stage trap and
   warm-vs-cold) without false-stale on git/worktree mtime churn.
5. A smoke failure auto-produces a classified hang/crash verdict + triage in the
   summary+log with no manual step; GDB path proven on a real bugcheck before shell
   removal.
6. Docs updated: `qemu/README.md`, `kernel-debugging-infra.md`, `AGENTS.md`.
