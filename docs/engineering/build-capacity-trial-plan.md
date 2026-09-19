# Three-build capacity trial

Owner: bughunt. Implemented locally; **not activated**. The old default remains
four CPU permits / two concurrent builds until the shared policy is installed.
The prior scanner/storage lane landed as `6527e86c881d`; this is subsequent work.

## Policy and bounds

`vibeos-cli host-policy` reports the active policy, proposed trial, and rollout
blockers without changing capacity. The trial is six CPU permits, two per build,
six Ninja jobs per build, and one Cargo job per invocation. Three builds can
therefore admit at most 18 simultaneous Ninja jobs on this 24-logical-CPU host.
This bounds compiler-process fanout, not rustc/LLVM internal threads or memory.
Windows-host boot capacity is unchanged. Debug info and cache settings are unchanged.

The policy lives at `<HOSTLOCK_DIR>/build-policy.json`, outside worktrees.
CPU clients pin it under a shared lock from admission through permit release;
activation/rollback requires the exclusive lock. Invalid policy, conflicting
per-client lease counts, and active-policy admission timeout fail closed.
`BUILD_POLICY_REFUSED` is inconclusive to the merge train, never a code regression.

## Coordinated rollout

1. Land this tooling and update every registered worktree before enabling it.
   `host-policy` lists older trees; it does not modify them. Drain old loaded
   CLI parents and raw compiler jobs too: updating files cannot update a running
   Python process. Independent clones/raw launchers must join the same cutover.
2. On upgraded clients, `host-policy --pause` refuses new and queued CPU work;
   current holders finish normally. It does not kill jobs or stop old clients.
   Stop external launch loops for the drain window. No pause was installed by
   implementation/testing of this slice.
3. Once clients and raw artifact jobs are drained, `host-policy --apply` installs
   the trial atomically. It refuses active CPU locks, policy-unaware worktrees,
   live artifact CLI/compiler processes, and a job budget above visible CPUs.
4. `host-policy --resume` allows new CPU work. Confirm `host-status --json` reports
   six permits and inspect each build summary's `build_policy` field.

This is cooperative admission, not an OS quota. The source-capability and
process checks are rollout guards, not protection against someone later checking
out old tooling or starting raw Ninja. Do not claim a safe mixed-version trial.

## Measure, then decide

Compare gate wall time, CPU/directory wait, and build-stage durations against
the previous two-build gate. Sample MemAvailable, swap activity, CPU/memory/I/O
pressure, and desktop boot verdicts throughout a cold three-build interval;
idle-memory snapshots alone cannot establish a safe peak.

Initial stop criteria: sustained MemAvailable below 8 GiB, sustained swap-out
or memory pressure, new OOMs, or materially worse boot/gate latency. Pause new
admissions, let holders drain, then `host-policy --rollback` and `--resume`.
Rollback restores four permits/two builds while retaining conservative job caps.
It does not kill active compilers or promise immediate memory reclamation.

Local verification: all 480 CLI tests (including 11 policy tests), lint/format,
and CLI type checks passed. Real `host-policy --apply`
refused the busy fleet without writing an active policy. No host-wide pause or
capacity change was applied, and no three-build performance result is claimed.

Do not increase to four builds before measuring this trial. Removing permits
held during source analysis and fixing nested cache-wrapper propagation remain
separate latency work; increasing capacity does not resolve those bottlenecks.

## Measured: `cargo_jobs` (2026-09-14, drained fleet)

Prompted by a suggestion to adopt the **GNU make jobserver** so a solo lane could
use the whole box. It is a real protocol and cargo is a client (`jobserver-0.1.34`),
but we should not adopt it, and the thing actually worth fixing is smaller.

**`host-jobs` already solves what a jobserver solves, and fails safer.** It calls
`admit()` *before* `spawn` (`tools/host-jobs/src/main.rs:583`), so a queued job is a
few-MB wrapper, never a 1-2 GB rustc. Its `flock` tokens are released by the kernel on
death; a FIFO jobserver **leaks a token permanently when a client dies**, so with a
reaper that kills builds the pool would silently shrink. Adopting it would mean running
two schedulers against one budget.

**The `cargo_jobs` policy key is dead code.** `sdk/cmake/gcc.cmake:731` bakes a literal
`CARGO_BUILD_JOBS=1` into each cell's `cmake -E env` line — 39 of them in a configured
`build.ninja` — which overrides the value `build.py:_policy_build_env` takes from the
policy. The inner cargos that `reactos-link` spawns from `build.rs` inherit it too.
So **every cargo cell walks its own crate graph one crate at a time**, and raising
`cargo_jobs` in `build-policy.json` changes nothing. Anyone measuring this by editing
the policy gets a null result that looks like "cargo fanout doesn't help" — a false
negative, not a finding. (The validator also hard-rejects `cargo_jobs != 1`.)

This matters because the cargo cells are the **eleven longest targets** in a
10,483-target amd64 graph (ntoskrnl 23.5s, ntkrnlmp 23.6s, win32k 15.6s, ntdll 10.7s …
124.5s across eleven, on a *warm* rebuild). Ninja runs 6 at once, so the Rust half of
the build is at most 6-wide and cannot fill the 18-slot host-jobs budget.

The width is real: `kernel-rust` pulls **33 crates**. Cold builds of it, wiped target
dir, no `RUSTC_WRAPPER`, idle host (24 CPU / 47 GB):

| `CARGO_BUILD_JOBS` | wall | speedup | peak concurrent rustc | peak Σ rustc RSS | min MemAvailable |
|---|---|---|---|---|---|
| 1  | 20.31s | 1.00× | 1  | 0.91 GB | 43.7 GB |
| 4  |  9.41s | 2.16× | 4  | 1.19 GB | 43.5 GB |
| 6  |  8.32s | 2.44× | 6  | 1.19 GB | 43.4 GB |
| 12 |  6.67s | 3.04× | 12 | 1.93 GB | 43.3 GB |

Reproduced across two independent sweeps (±0.4s). The knee is at 4-6.

**The memory premise behind `cargo_jobs = 1` does not hold for this cell.** Twelve
concurrent rustc cost 1.93 GB in aggregate and moved MemAvailable by 0.4 GB. The
`1` buys a *computable* ceiling (`max_builds x ninja_jobs`), not memory safety.

### Recommendation

Make the knob live and raise it to **4**, keeping `host-jobs` as the single scheduler —
the 18-slot broker still bounds total rustc, so this fills the existing budget rather
than raising it. Do **not** add a jobserver.

Two cautions before anyone acts on the table above:
- It measures **one cell on an idle host**, which is the solo-lane case and an *upper*
  bound. With 6 cells live under `ninja -j6` the host is already ~6-wide and the broker
  will throttle; the fleet-wide win is smaller and concentrated in the **tail**, when
  fewer than 6 cells remain runnable.
- Removing the `gcc.cmake` literal outright would make a raw `ninja` inherit an unset
  `CARGO_BUILD_JOBS` and default to `nproc` (24) — uncapped. The literal must become a
  CMake variable that defaults to 1, not disappear.

This is a capacity change, so it belongs at a drained fleet boundary under the
"Coordinated rollout" steps above. **Not implemented; no policy was changed.**


## Where the policy lives (2026-09-19)

It used to live under `HOSTLOCK_DIR` (`/tmp/vibeos-qemu-locks/host`). A reboot deletes
`/tmp`, so the cap silently stopped existing: `host-policy` reported `"active": null`,
`_ninja_jobs()` fell through to its uncapped branch, and `_max_builds()` returned its
own default of 4 — **4 builds x ~26 ninja jobs on 24 cores**, which is *more* permissive
than the pre-trial default, not less. Nothing announced this. It was found 13 minutes
into an uptime, while deciding whether it was safe to restart the fleet.

The lock tree is the wrong home for it. Everything else there is `flock` state whose
meaning dies with the process holding it, so losing it on reboot is *correct*; capacity
is durable config and a different lifetime entirely. They are now separate:

| | |
|---|---|
| `hostlock.HOSTLOCK_DIR` | ephemeral — flock files, queues, the land lock |
| `hostlock.POLICY_DIR` | durable — `build-policy.json`, the pause flag |

`POLICY_DIR` resolves to the **main worktree** (`buildpolicy.main_worktree()`, which reads
a linked worktree's `.git` file — no subprocess). It must be one file: eight lanes with
eight capacities is not a cap. Override with `VIBEOS_BUILD_POLICY_DIR`.

`build-policy.json` is gitignored; `build-policy.example.json` is tracked. Any
`vibeos-cli` run seeds the former from the latter if it is missing (`buildpolicy.heal()`),
so a fresh clone builds capped with no command to remember. **The example is deliberately
conservative — one build, four Ninja jobs.** Self-heal is a floor, not a restore: a box
that lost its policy unattended should come back slow. `host-policy --apply` is how you
opt into the trial, and that now persists across reboots.

### Transition shim — delete when no worktree is stale

A lane on older tooling reads `HOSTLOCK_DIR` and nothing else, so no longer writing there
would leave every un-merged lane uncapped — this change causing the exact crash it
prevents. `buildpolicy.mirror()` therefore keeps copying the active policy to the old path.
Old clients only ever *read* it, so a second writer cannot confuse them.

Because of the mirror, an un-merged worktree is no longer a mixed-fleet hazard, so it is
reported as `stale_worktrees` (advisory) rather than refusing the cutover. `blockers`
now means only live work that a capacity change would race. Refusing to install capacity
because a *paused* lane had not merged would pin the host to the conservative seed and
buy nothing. Remove `mirror()` and the shim once `host-policy` reports no stale
worktrees.
