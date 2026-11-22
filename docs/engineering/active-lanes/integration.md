# Integration lane — the one place lanes land through (worktree `vibeos-integration`)

> **Opened 2026-10-16** (user's call). You are the integrator: the only session that merges
> lanes into `master`. Lanes message you to merge their branch; you merge it like a human
> would, resolve what conflicts, gate, push, and tell them how it went. They keep working
> the whole time.

## Why this exists

The merge train (`vibeos-cli land` → queue → whichever lane found the lock conducts) piled
optimisation on optimisation until it was slower than what it replaced. Measured on
2026-10-16 from `.git/vibeos-queue/`:

- Since 11:00, 20 submissions spent **802 lane-minutes** queued (median 37 min, queue up
  to 6 deep), and 8 of 11 batches were a **single lane**: the train gated one lane at a time.
- Before submitting, a lane paid warm builds ×3 (a receipt rule) plus ~6 min of preflight
  (p90 11), then stopped again for the verdict.
- 10 of 68 submissions in four days bounced on a merge conflict, mostly in files every lane
  appends to (`tools/rustconv/gen_driver_types.sh`, `sdk/lib/rust/Cargo.lock`,
  `sdk/lib/rust/Cargo.toml` members, `tools/cohab-census.txt`). Mechanical, and each bounce
  sent a lane back to rebuild and re-queue.

You replace the conductor, the bounces and the lane-side waiting. The gates themselves
(`vibeos-cli gate`: builds ×3, lints, tooling, difftest, smoke, conformance) stay exactly
as strict.

## Starting up

The merge train this replaced was retired on 2026-11-08: `land` no longer queues anything,
and plain `land` prints a lane's merge request instead of submitting it. One relic stays on
purpose: the train's hold record (`$GIT_COMMON_DIR/vibeos-queue/HOLD.json`, taken by the
first integrator session). A lane still on a commit from before the retirement runs the old
CLI, and that hold is what stops it conducting a train. Do not release it; current code
never reads it. If a lane on an old commit says its `land` "queued", treat that as a merge
request and tell it to merge master.

## RESUME HERE (fleet park, 2026-11-22)

Parked clean after batches 175-206 (2026-11-21/22). **Master was `d1233231e29`** with its last full
gate green on all 3 arches: `/tmp/vibeos-cli/20261122-173032-SdYcTz-gate-i386+amd64+arm64`;
/tmp does not survive a reboot, so re-gate master if you need a fresh artifact. The queue was
empty and every lane's RESUME HERE is on master.

**First item next weekend: the `ladder:arm64-fex` gate stage (owed by this lane).** The gate's
arm64 build has no FEX, so `ladder:arm64` prints "skip" and the arm64 FEX x64 regression from
`536a9761765` landed green in batch 179 (fixed by fex in batch 202, `7f6450c3d17`). Agreed shape
(control-plane, fex, integrator, 2026-11-22):
- `smoke:arm64` stays on plain shipping media.
- A separate stage `ladder:arm64-fex` on its own `build-arm64-fex` (`VIBEOS_FEX=1`, ~5.5G),
  wired into the gate tool, not an env var set by hand.
- It never skips: it provisions `build-fex-arm64` (`tools/fex/build-fex.sh`; fex measured a
  cold run at 57 s and 25M on a loaded host, ccache does not apply) and `build-arm64-fex` when
  absent, then calls `smoke --arch arm64 --ladder` unconditionally. That path already fails red
  with `LADDER_MEDIA_INCOMPLETE` / `LADDER_GUESTS_STALE` (smoke_run.rs). The only skip today is
  `gate_run.rs` mapping `staged=false` to "skip"; leave that mapping alone for the existing
  `ladder:{i386,amd64,arm64}` stages. No ladder-side knob is needed (fex checked).
- `build-fex-hello` already exists in vibeos-integration; `build-fex-arm64` does not.

**Disk.** `/data` ran down from 224G (after the AI caches moved to D:) to ~80G in a day of
gates. Batch 175 failed DISK_ADMISSION_FAIL before that move. `vibeos-cli prune` cannot get its
exclusive lock while any CLI command runs, so it only applies with the fleet idle; a
per-worktree lock is owed in control-plane's lane. Check `vibeos-cli disk-status` before a
batch: a cold 3-arch gate requests ~36G above the 20G floor.

**Flakes.** The arm64 ZwAllocateVirtualMemory "race" was a kmtest bug (conformance,
`5d426a3dcaa`, batch 197) and is fixed. The `test_ladder` drain-test flake is fixed
(`afd0d2b91f6`). The /proc ESRCH race class is now a lint (`scan_proc_esrch`). Still open:
`test_artifact_runtime_real` timing out under load (not seen here), WSL vsock in
`test_bench_windows`, and the gui32 GetPixel readback fex saw 1 in 15 on arm64 FEX.

**Load.** Load average 25-30 stretched builds; batches from 198 used
`VIBEOS_BUILD_TIMEOUT=7200`. Gates ran 7-65 min.

**Not for merge unless the lane asks:** `smp-bringup-msxml3-wip` (the msxml3 Rust crate,
parked). `graviton-diag-ctxwait` and `smp-bringup-crypt32-wip` still exist and were never
submitted; crypt32 itself was flipped from `smp-bringup` in batch 187.

## A merge request

A lane sends you a peer message along the lines of
`merge native-batch @ 3c125b75a210 — <one line: what it is, what it touches>`.
Acknowledge it in one line. The sha is what you merge: it is in the shared object store,
so `git merge <sha>` works with no fetch from the lane's worktree. If a lane names only a
branch, resolve it with `git rev-parse <branch>` and quote the sha back.

## The cycle

Collect every request that has arrived; that is the batch. Batch everything pending:
one gate for N lanes is the point.

```bash
cd /home/curi/Projects/os-playground/vibeos-integration
git fetch origin
git status --short                          # must be clean before a new batch
git switch -C integration origin/master     # same branch name every batch (see below)
git merge --no-edit <sha-1>                 # one ordinary merge per lane, in arrival order
git merge --no-edit <sha-2>
...
./vibeos-cli land --direct --cold --skip-lint
```

- **Ordinary merges only.** No cherry-picks, no rebases, no squashing, no "lineage audits".
  Identical content from another lineage collapses on its own; if git did not conflict,
  there is nothing to look at.
- `land --direct` merges master once more, runs the full gate on this exact tree and
  pushes `HEAD:master` under the land lock. `--cold` and `--skip-lint` are sanctioned
  here (and only here): the gate builds from scratch and runs every lint itself, so the
  lane-side receipt and preflight would only repeat it. Say "integrator --cold
  --skip-lint" in the landing report rather than reporting each use.
- **Every batch is the branch `integration`, re-created from fresh `origin/master`.** Lanes
  read it: `land-status --tip <sha>` answers IN BATCH when their sha is merged into
  `integration`, so a batch on another branch name is invisible to them. `switch -C` only on a
  clean tree, after the previous batch landed or was abandoned on purpose: it moves this
  branch, nothing else. Never `reset --hard`, never `checkout`/`restore` to discard, never
  move `master` locally. Dropping a lane from a batch means re-creating `integration` and
  merging the others again.

## Conflicts are yours to resolve

Resolve them; do not bounce them. You can read both sides and both lanes' intent.

- **Append-only files** (`gen_driver_types.sh` case arms, workspace `members`): take the
  union of both sides' entries.
- **Generated files:** take master's side, then regenerate: `tools/cohab-census.txt` with
  `python3 tools/cohab_census.py --write`; `sdk/lib/rust/Cargo.lock` with
  `cargo metadata --offline --format-version 1 >/dev/null` from `sdk/lib/rust`; the apiset
  table with `python3 tools/gen_apisets_table.py`.
- **Line endings / whitespace-only:** take either side, normalized to the file's existing
  ending. Note the file so a `.gitattributes` rule can end the class.
- **Real code conflicts:** resolve them the way the two authors would agree on, keeping
  both intents. If you genuinely cannot tell, merge the rest of the batch without that
  lane and message it the conflicting hunks and your best reading.
- Before committing a resolution: `git diff --cached --check` and
  `git grep -nE '^(<<<<<<<|=======|>>>>>>>)' -- <files>`.

## When the gate is red

1. Read `summary.json` at the top of the gate's artifact dir: verdict, classification,
   salient lines.
2. **Infrastructure** (DISK_ADMISSION_FAIL, RUN_INTERRUPTED, a known flake in the roadmap's
   registry): fix or wait and re-gate; nobody is at fault.
3. **A defect:** find the lane. The classification usually names the module; otherwise
   `git log --merges` on the batch branch says which lane touched it. If it is small and
   unambiguous (a lint finding, a stale generated file, a missed rename, a one-line build
   fix), **fix it on the batch branch**, commit it with a message naming the lane, re-gate,
   and tell the lane what you changed. Otherwise start a new batch without that lane, land
   the rest, and message the lane the classification, artifact dir, salient lines and what
   you think is wrong.
4. If you cannot tell which lane broke it, bisect by lanes (half the batch per new branch),
   not by commits.

A lane should rarely bounce. When one does, say why in the message so the lane can fix the
cause, not just the instance.

## Talking to lanes

- `ListAgents` shows the sessions; reply to the address a request came from.
- On landing: `landed <lane> @ <sha> as <master sha>; merge master when convenient`.
  Lanes do not wait for this, so keep it short.
- On a fix you made: what you changed and where, so the lane can merge it back.
- On a bounce: classification, artifact dir, the salient lines and your reading of the
  cause. Never relay a claim you did not check against the artifact (AGENTS.md: a relayed
  status claim needs its evidence).

## Rules that still apply

Everything in `AGENTS.md`. In particular: never revert a Rust translation to C or hide one
from the build to get a gate green; never delete a translated `.rs`; never run a paid
`rustconv translate`; never broad-kill QEMUs (`./vibeos-cli kill`); report every landing to
the user.

## Owed (not yet built)

- **Gate scoped to the change.** Docs-only already skips. A leaf app or DLL off the boot
  path could gate on builds ×3 plus PE checks without kernel smoke and conformance, with a
  full smoke of master every few landings to catch a mis-scoped change. Until it exists,
  every batch takes the full gate.
- **Merge drivers / `.gitattributes`** for the append-only and generated files above, so
  even you never see those conflicts.
- **`land` as submit-to-integrator:** plain `land` prints the filled-in merge request
  (2026-11-08); sending it is still the lane's SendMessage, since the CLI has no channel
  to a session.
- **`vibeos-lint`:** the lint suite is being rewritten in Rust as one binary (worktree
  `vibeos-lint`, `docs/engineering/vibeos-lint-design.md`); the gate's lint stage switches
  to it when it reaches parity.
