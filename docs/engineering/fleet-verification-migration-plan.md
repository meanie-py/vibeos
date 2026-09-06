# Fleet verification migration

Owner: `bughunt` lane (Codex), first gate-partition slice.
Status: first slice implemented and verified locally, 2026-09-06; not pushed. This plan does
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

## Then: focused preflight and evidence reuse

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

This first slice does not change admission policy, skip checks, alter retry
rules, implement preflight, install regenerated bindings, or push a lane. Other
agents should coordinate ownership before editing the broker, queue, or gate.
The temporary coordination note points here; this version travels with the repo.
