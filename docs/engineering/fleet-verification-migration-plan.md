# Fleet verification migration

Owner: `bughunt` lane (Codex), gate partitioning and admission migration.
Status: gate partitioning and admission shadow/telemetry verified locally,
2026-09-06. Landing status is reported by `vibeos-cli land-status --mine`. This plan does
not relax current AGENTS.md requirements or authorize competing fleet schedulers.

## Evidence and objective

The reference gate took 35m06s. All builds were submitted at time zero; its last
build completed at 1196s. The shared source partition then occupied another
910.5s, about 43% of total wall time. amd64's first recorded build command was
at 755s, while its main Ninja invocation took 127s. Its desktop smoke command
took 47s. Queueing and a serial analysis tail dominate this example; it is not
proof of the bottleneck in every gate or of the fleet's safe memory ceiling.

Reference: `/tmp/vibeos-cli/20260906-105002-gate-i386+amd64+arm64/summary.json`
and child summaries (local temporary evidence).

The objective is fast lane feedback plus full certification of the exact merged
tree, without duplicating full CI in every agent lane. Preserve all required
checks, real three-architecture desktop markers, and worktree isolation.

## First slice: split and hoist audited source checks

`lint:source` is not entirely source-only. `verify_signatures` needs generated
headers, and `marshalgen audit` resolves configured build directories. Moving
the whole suite to time zero would risk stale reads or races with generation.

The initial early set reads Rust source without following generated includes:

| Check | Input contract | Reference duration |
|---|---|---:|
| `scan_extern_decl_drift` | Rust workspace types and `*-rust/src` declarations; excludes `target/` | 127s |
| `scan_octal_literals` | Rust files under `sdk/`; excludes `target/` | 109s |
| `scan_type_alias_contradictions` | Rust workspace aliases; excludes `target/` and `.responses/` | 83s |
| `scan_irql_balance` | Workspace Rust call-graph facts; excludes target output, bindgen files, build scripts | 84s |
| `scan_ob_refcount` | Workspace Rust function bodies; same exclusions as IRQL scanner | 48s |

These five commands run sequentially in one early worker, overlapping builds.
They previously occupied about 7½ minutes. This is potential overlap, not a
promised wall-time saving: contention can change their and the builds' duration.

Landing follow-up (2026-09-06): `scan_invented_constants` is now the sixth early
check after auditing its Rust-source and scoped C/SDK-header reads. Its fixture
requires a real source/header mismatch, then proves changing generated headers
and Cargo output cannot affect it. New inventory entries still default to
deferred; only duplicate names or removal of an explicitly audited entry fail
inventory construction. The five-check measurements below remain historical.
Verification after syncing master: 442 scanner/CLI tests and 56 build-invalidation
tests passed; type checking passed. A strict real-tree check of
`scan_invented_constants` passed its unchanged baseline in 32s (explicitly partial,
not a gate). Full certification remains the merge conductor's job.

The ARM64 `scan_ud2` corpus concern is tracked in the roadmap's live bug registry;
it is not fixed by rescheduling source checks.

### Concurrent traversal and incomplete checks

The fleet handoff exposed a stronger requirement than source-only inputs:
post-filtering `rglob` results still enters mutable Cargo/build directories and
can crash while a sibling process removes them. All six early scanners now use
`tools/source_walk.py`, pruning excluded directories before traversal. The
invented-constant C-oracle search uses the same mechanism. Missing or unreadable
included source fails loudly rather than silently shrinking coverage.

Regression fixtures audit both directory entry and file reads, require actual
source reads, and compare findings while generated files/directories churn.
A source-I/O failure is tested separately from harmless generated-output churn.
This traversal audit is a prerequisite for adding any scanner to the early set;
source independence alone is insufficient. Newly added scanners still default
to deferred, not an inventory-construction failure.

Scanner tracebacks, signals, and nonzero exits without findings produce
`SCANNER_ERROR`, even if partial output matches the baseline. Baseline updates
also refuse incomplete results. The gate preserves this classification, blocks
certification, and the merge queue treats it as inconclusive rather than blaming
or recursively bisecting lanes as though it were a new source finding. Required
check inventories and baseline rules for completed scans are unchanged.

Handoff reconciliation: Claude's `f9ca41a6a88` identified the C-oracle traversal
fix; this implementation generalizes pruning without swallowing source I/O
errors. The reported `scan_rawdylib_linkname` scanner is not in this checkout;
audit its traversal before promoting it when it arrives. Neither this change
nor early scheduling resolves the separately tracked ARM64 binary-corpus gap.

Local validation: all 31 tooling checks and 465 CLI/traversal tests passed.
`20260906-185617-check-source-early-all` completed all six scanners in 6m52s;
each finding set exactly matched the previous cold gate (11/49/0/9/8/0), not
merely its baseline threshold. Its WARN explicitly means partial coverage,
not a full gate or push stamp. No new OS build/smoke run is claimed for this
tooling-only slice; the conductor must certify the eventual merged tree.

Mechanism:

- `source_partition_specs` partitions the original command inventory without
  changing commands, findings, baselines, or checker semantics.
- An explicit audited early set runs at gate start. Everything else, including
  newly added checks, defaults to the deferred partition after all builds.
- Duplicate names or missing audited checks fail loudly. Tests require the two
  partitions to be disjoint and their union to equal the original inventory.
- Each child verifies its executed check inventory and reports checked/unchecked
  source checks plus omitted verification categories. The parent independently
  requires the expected inventory and successful results from both partitions.
- Standalone `check` and `check --scope source` retain the complete source suite.
  `source-early` and `source-deferred` scopes are explicitly partial diagnostics.
- Fresh-worktree fixtures and concurrent generated-header/target-output writes
  exercise each early scanner unchanged, with audit hooks rejecting generated
  input reads. Event-based gate tests prove overlap and the deferred barrier.

First evaluate this change without changing broker priorities or concurrency.
Rollback is to restore the single post-build source partition, never omit checks.
Do not enable result caching or parallelize all scanners as part of this slice.

Verification: the clean three-architecture gate completed in 23m44s with every
required stage passing: 5 early + 40 deferred source checks (disjoint union 45),
architecture guards, 28 tooling checks, host difftests, cohab checks, three
desktop smokes, and the 15-test network smoke. Screenshots were inspected.
The early partition took 452s and finished before any build began; deferred
checks took 409s after the build matrix. Fleet admission wait confounds a direct
speedup claim. CLI unit tests passed 394 cases; generated-input isolation
fixtures passed all five cases. The gate reports WARN only because the tree was
dirty at invocation, so no push stamp was minted.

Local evidence: `/tmp/vibeos-cli/20260906-130606-gate-i386+amd64+arm64/summary.json`
and its child summaries. These temporary artifacts are not portable certificates.

## Next: predictable admission for the merge train

Capacity trial: [build-capacity-trial-plan.md](build-capacity-trial-plan.md)
defines a coordinated three-build rollout with shared policy, per-build job
caps, and pause/drain/rollback commands. It is not yet enabled; it does not
replace the priority or lease-lifetime work below.

Second slice implements observation only. Queue and direct-land gate children
inherit `VIBEOS_JOB_CLASS=integration`; ordinary invocations default to
`interactive`, with `background` available as an advisory tag. Tags do not grant
priority or authority. Ticket filenames and actual FIFO admission are unchanged.

`host-status --json` includes an `admission_shadow` proposal, and CPU admissions
emit one JSON record per admission into their existing command logs. The candidate gives
integration a 120-second virtual head start: an ordinary waiter older by more
than 120 seconds wins, so a continuous train cannot indefinitely jump it. This
constant is a hypothesis to measure, not an enforced resource policy. Background
currently has the same candidate priority as interactive work.

New tickets record monotonic queue time. Legacy, malformed, or unknown-class
waiters suppress the proposal rather than being guessed into a class. A shared
`<HOSTLOCK_DIR>/admission-shadow.disabled` file disables proposals across updated
lanes; it never releases locks or changes FIFO. Default directory:
`/tmp/vibeos-qemu-locks/host`. No shared switch is created by this slice.

Permit metadata records `wait_seconds`; build summaries additionally report
`admission.cpu_wait_s` and `admission.builddir_wait_s`, plus whether each grant
actually succeeded (existing timeout behavior is not disguised as a grant).
These distinguish pool queueing from CPU capacity held while waiting for a
directory. They do not yet measure every execution/cleanup phase.

Still owed before enforcement: collect mixed-fleet shadow observations, select
host-wide policy/version negotiation, test crash and same-directory contention
across old/new acquisition protocols, and separate build CPU capacity from
analysis without releasing directory protection. This slice intentionally does
not reverse lock acquisition order or release permits early in a mixed fleet.

Second-slice verification: 404 CLI unit tests passed; after correcting two type
issues, 37 focused tests and all nine static tooling checks passed again. The
full tooling invocation passed every test suite and failed only the corrected
type check (27/28 before correction). A real ARM64 warm build exercised the new
summary fields: CPU wait 0.011s, directory wait 0.002s, both grants confirmed.
Its desktop smoke reached the marker in 45s; screenshot inspected. Both carry
the expected warm-build warning. No new full three-architecture gate or push
stamp is claimed for this observation-only slice.

Local artifacts: `/tmp/vibeos-cli/20260906-133914-build-arm64/summary.json` and
`/tmp/vibeos-cli/20260906-134035-smoke-arm64/summary.json`.

Classify jobs as interactive, integration, or background. Measure queue wait,
resource wait, execution, and cleanup separately. Introduce train priority at
admission boundaries with aging, without interrupting active builds. Observe
proposed decisions in shadow mode before enforcement.

Policy must be host-wide rather than varying with each lane's checkout. Use a
host-wide kill switch and finish/drain active jobs when switching accounting.
Coordinate Ninja/Cargo job limits with resource grants, account for memory-heavy
links, and release build grants before unrelated analysis. Measure landing
latency, throughput, memory pressure, starvation, and boot timeouts before
raising concurrency. Windows-host QEMU and WSL still share physical resources.

During first-slice verification, `host-status` showed both build-permit pairs
occupied while their owners' active children were source scanners, with our
build matrix queued. This is direct evidence that admission grants can outlive
compilation. Releasing CPU capacity must still preserve any build-directory
lock needed by header/binary readers, and new jobs must not hold CPU grants
while merely waiting for a busy build directory. Test that interaction before
changing acquisition order.

Implementation entry points: `tests/qemu/cli/hostlock.py` owns FIFO tickets and
the combined `build_lock`; `tests/qemu/cli/build.py` holds that lock around
`_build_locked`, including binary lints and optional advisory source checks.
Extend the existing broker rather than introducing a second scheduler. Include
mixed-version lanes, crashed waiters, aging, and same-directory contention in
admission tests.

## Disk admission and retention (promoted ahead of priority enforcement)

First disk slice: `vibeos-cli disk-status [--sizes]` reports free bytes, live
reservations, and optional sizes for this lane's build/target outputs and shared
compiler caches. CLI build and tooling entry points reserve estimated growth
before expensive work, under one host lock. Defaults are 32 GiB per build,
4 GiB per tooling run, and 20 GiB unreserved headroom. Gate/land parents perform
the headroom check but reserve zero additional bytes; child jobs reserve their
own budgets. These initial estimates require measurement and tuning.

One host policy file, `<HOSTLOCK_DIR>/disk/policy.json`, controls all updated
lanes; the first reservation pins the documented defaults there under the host
lock without overwriting existing policy. Read-only status never creates it.
It has exactly these keys:

```json
{"version":1,"min_free_gib":20,"requests_gib":{"build":32,"check-tooling":4,"gate":0,"land":0}}
```

Invalid policy or unreadable live reservations fail closed. Reservations are
per filesystem (repo, cache parent, temporary and artifact roots, deduplicated
by device); flock ownership makes dead CLI processes reclaimable. The complete
growth estimate stays reserved until exit, deliberately conservative as bytes
are written. No CPU lease is acquired first. Refusals emit `DISK_ADMISSION_FAIL`
on stdout even when an artifact file cannot be created, and the merge conductor
treats them as infrastructure failures rather than rejecting code.

This is cooperative admission, NOT a physical quota or a guarantee against
ENOSPC: old lanes, raw tool invocations, standalone smoke/difftest, Windows-side
QEMU storage, and unexpected growth remain outside this first slice. Do not
raise parallelism on the assumption that the whole fleet is already covered.
The initial host policy is created once; nothing is auto-deleted.

Still owed for retention: classify disk growth, choose byte budgets and retention
windows for successful/failed runs, protect the integration working set, and
make cleanup targeted and host-locked with active-job exclusion. Start with
rebuildable completed outputs, not paid translations or blanket cache eviction.
Admission prevents more cooperating jobs starting under pressure; it does not
solve accumulation. Extend the accounting to the remaining writers before
claiming fleet-wide disk safety.

The first within-lane storage pilot and retention interlock are tracked in
[`rust-output-retention-plan.md`](rust-output-retention-plan.md), including the
required fleet cutover before cleanup may be enabled. Debug information stays on.

Cache audit: compiler caches are shared under the worktree parent, and canonical
build paths make unchanged compiler inputs reusable across lanes. Per-worktree
servers preserve mount-namespace correctness; they do not imply separate disk
caches. Cargo/Ninja outputs and links remain private, as required by isolation.
On 2026-09-06 the shared caches occupied about 5.5 GiB, versus about 37 GiB for
this lane's three build directories plus Cargo target output. This is a point
measurement after manual cleanup, not a historical attribution of the full disk.
Per-build cache-counter resets have been removed: shared counters are now
explicitly cumulative snapshots, not trustworthy per-build hit-rate deltas.
All four observed live sccache processes used the same shared cache directory.

Disk-slice verification: 435 CLI tests passed, including concurrent reservations,
crashed-owner release, malformed-policy refusal, and stdout-only infrastructure
classification and actionable land-preflight errors. All 30 tooling checks
passed; final CLI tests (including 14 disk cases) and static checks were rerun
green after policy bootstrap and error propagation were finalized.
The ARM64 warm build passed with fresh images: 151s CPU wait, 209s Ninja, 407s
total. Its shared sccache snapshot recorded 87 hits, 23 misses, and zero cache
errors, with explicit shared-counter labelling. Reservations were empty after
both real jobs exited. This is not a new full gate or three-architecture smoke.
Local evidence: `/tmp/vibeos-cli/20260906-171615-build-arm64/summary.json`.

## Focused preflight and evidence reuse

A proposed `vibeos-cli preflight` should provide focused development feedback,
broaden conservatively for uncertain dependencies, and never mint a push stamp.
Its pasted output must say what was checked AND what was not checked; record
the same in structured results. Change CLI guidance and AGENTS.md together when
transferring routine full certification to the train. No policy change yet.

After scheduling is stable, coalesce identical requests and cache deterministic
results keyed by inputs, checker, configuration, and toolchain identities.
Timestamps or changed-file lists are not content identities. Existing fixture
clock/invalidation controls in `tools/test_build_invalidation.py` demonstrate
why timestamp ordering needs independent scrutiny; attach the fleet clock-fix
incident's verified commit/evidence before citing that separate incident as fact.

Share only immutable content-addressed artifacts, never live Cargo/build trees
or cross-worktree hardlinks (the SIGBUS and stale-fingerprint incidents in
AGENTS.md). Fine-grained reuse needs proven dependency coverage. Keep clean
integration builds and fresh desktop boots on all three architectures initially.
Periodic-only cold-build assurance would require a separate later decision.

Failed batches can eventually be diagnosed using the failing check/build stage
on subsets, but every resulting tree still needs full certification before push.

## Boundaries

These slices do not change enforced admission policy, skip checks, alter retry
rules, implement preflight, install regenerated bindings, or push a lane. Other
agents should coordinate ownership before editing the broker, queue, or gate.
The temporary coordination note points here; this version travels with the repo.
