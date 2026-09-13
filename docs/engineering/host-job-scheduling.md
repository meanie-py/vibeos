# Host job scheduling

Build commands no longer reserve CPU permits. They hold only their own build
folder's exclusive lock. That lock fails closed and remains active even if legacy
CPU throttling is disabled. Waiting for a duplicate build cannot consume CPU
capacity. Host policy stays pinned for the build's lifetime.

`tools/host-jobs` is a dependency-free Rust runner. CMake native compile/link
rules, Cargo's Rust compiler wrapper, and reactos-link's native compiles pass
through it. Ninja and Cargo coordinators never hold execution slots. A lone
build can use the host's available parallelism; all worktrees share the same job
slots. The outer Cargo pool exposes the visible CPU count rather than reserving
six links per build. Individual Cargo invocations still use one job, avoiding
nested Cargo job multiplication.

The default is one slot per available logical CPU, with a separate link cap of
at most one quarter of that count and one link per 2 GiB of currently available
memory. Rust invocations producing executables or dynamic libraries count as
links. These are process admission limits, not CPU affinity or memory quotas.
`VIBEOS_HOST_JOB_SLOTS` and `VIBEOS_HOST_LINK_SLOTS` override them; an existing
explicit host build policy supplies its aggregate job budget.

Admission prefers interactive work, then integration, then background. After
120 seconds waiting, a job joins the oldest-first top priority. Running jobs
finish; priority applies at the next admission. Link waiters skip CPU admission
when the link limit is full, allowing ordinary compilation to proceed. A shared
futex event wakes waiters without consuming a notification descriptor per job;
a bounded timer covers aging and legacy lock changes.

One elected waiter dispatches a batch of eligible jobs when the queue changes.
It assigns concrete slots; recipients claim their own execution locks, while
other waiters sleep. Granted slots count against capacity before the compiler
starts. This avoids a convoy where every waiter rescans the entire queue under
one lock and the next eligible compiler waits behind those scans. No persistent
broker daemon is required.

Flocks represent ownership. Compiler children inherit direct execution leases.
Cache clients instead use a short-lived guardian: sccache daemons demonstrably
retain inherited descriptors, which would otherwise leak slots indefinitely.
The guardian holds close-on-exec descriptors until the cache client finishes,
even if the original launcher dies. This does not attempt to cancel a detached
merge conductor when an interactive agent pauses.

`vibeos-cli host-status` reports both legacy reservations and runnable jobs.
`--json` includes each job's command, state, class, checkout, owner build PID,
artifact directory, submitting launcher PID, and guardian PID when applicable.
This makes a detached integration build visible independently of whether its submitting agent is currently active.

## Migration

The old four-permit pool remains for older worktrees and native TCG. New jobs
discount capacity occupied by those legacy holders. This is one-way cooperation:
an old client cannot see new execution slots, so it can cause temporary
oversubscription if it starts after new jobs have been admitted. Updating lanes
removes that mismatch. No sibling worktree, shared policy, or running cron is
modified by installing this implementation.

Correctness locks, host disk admission, source checks, binary checks and smoke
criteria are retained. The job scheduler is not a substitute for those gates.

## Regression evidence

`tools/test_host_jobs.py`, included in check-tooling, runs the Rust process tests
and real CMake/Cargo fixtures. Controls cover global concurrency, priority,
aging, link waiters, changing capacity, legacy occupancy, launcher death,
cache-daemon descriptor isolation, 144 waiters under a 32-descriptor limit,
command failures, paths containing spaces,
and transitive native header invalidation. CLI unit tests verify that build
coordinators take no CPU reservation and still exclude duplicate writers.

The first admitted amd64 build passed binary and freshness checks:
`/tmp/vibeos-cli/20260912-191801-yf9z_c2o-build-amd64/summary.json`.
The overlapping gate's amd64 build waited 34.5 seconds for that same build
folder, with **zero CPU permits held while waiting**:
`/tmp/vibeos-cli/20260912-192001-5safcqv7-build-amd64/summary.json`.

The former strict-check Cargo link PoCs were retired. Their assumption that
ntoskrnl/ntdll/ntdll_vista live off the shipping path is obsolete: all three
are normal `add_cargo_module` targets and are exercised by the real build and
smoke gate. The duplicate probes reused mutable Cargo target directories,
bypassed admission, and could race a concurrent clean build. Source/binary
checks and committed-artifact drift checks remain in strict lint.
