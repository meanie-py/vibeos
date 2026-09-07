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
