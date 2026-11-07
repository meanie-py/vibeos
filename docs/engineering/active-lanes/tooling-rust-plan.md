# Tooling rewrites — `lint-rs`

Status checked against this checkout on 2026-11-07. This is the lane’s current
work queue; the linked component documents retain implementation details.
The execution order below is proposed next work, not an instruction to launch it.

## Completed checkpoint

`lint-rs` at `51b917c7f84` contains the RR command coordinator, shared CLI core,
and build/gate orchestration in Rust. Python entry points retain compatibility
DTOs and external resource handles. Native source linting, code generators,
build mechanisms, VM lifecycle, smoke/conformance, farm, bench and diagnostic
mechanisms already have their own Rust implementations; do not reopen them just
because a Python facade exists.

The checkpoint also fixes broker fixture registration readiness, gives storage
its existing owned 900-second deadline instead of a competing pytest timer, and
preserves primary boot exceptions when diagnostic snapshot retrieval fails.

Validation evidence:

- Source check: `/tmp/vibeos-cli/20261107-045813-c1ibtP-check-all/summary.json` — OK.
- Full gate: `/tmp/vibeos-cli/20261107-050135-CwL0aj-gate-i386+amd64+arm64/summary.json`
  — OK, all runtime lanes passed on their first attempts, including amd64 BIOS;
  its tooling child passed all 127 checks. This gate used warm builds.
- Earlier cold builds passed on all three architectures in
  `/tmp/vibeos-cli/20261107-035206-lPZuDY-gate-i386+amd64+arm64/`.
  That gate’s sole final failure was the broker fixture, fixed in this checkpoint.
- Raw desktop markers and screenshots were independently checked for all four
  final smoke variants. The gate stamp records tree
  `b9fc5dcb2baad142099235f1206cb2322471e548`.

Details: [QEMU/testing ownership and evidence](../qemu-testing-rust-port.md),
[build migration](../build-infra-rust-plan.md),
[native build library](../../../tools/vibeos-build/README.md).

## Outstanding work, in proposed order

### 1. RR correctness follow-ups

The coordinator port is complete. Two of the three comparability gaps are closed
(`lint-rs` `d8dbd0ebad2`, `5def0122cf1`):

- **Replay tail — fixed.** Smoke ended a replay on the live boot's marker +
  wall-clock settle, so a replay (slower per wall second) stopped at 84% of its
  trace. `smoke::drain_replay` now runs a replay until QEMU has read the whole
  trace, exits, or stalls. The overnight trace that verified PARTIAL
  (2,083/2,230) now verifies 2,230/2,230.
- **Pause-time I/O — fixed at the sampler.** `rr record --sample` compares
  `query-blockstats` completion counters across each `stop` and retries, with a
  growing backoff, until the pause drained nothing; samples carry
  `pause_retries` / `io_at_pause`. Comparison policy is unchanged.
  Fresh trace `~/vibeos-rr/drain-sample-20261107c-arm64`: verify VERIFIED
  (2,552 lines), diverge **12/12 exact** (was 3/10; the intermediate run without
  growing backoff was 7/11, with `io_at_pause` predicting every inconclusive).
- **Open: interrupt entry at the sampled icount.** Not seen in the 12/12 run,
  which proves nothing. Still `inconclusive` by policy. Also still tracked: the
  first-checkpoint arming race.

Acceptance unchanged: complete exact coverage for an equivalence claim; partial,
inconclusive and diagnostic-error results keep their nonzero outcomes.

### 2. Remaining CLI policy and maintenance

Audit functions and callers before moving ownership. File length is not a tool
count, and several of these modules already call native helpers.

- `tests/qemu/cli/{lints,rustlints}.py`: baseline comparison, source/artifact
  selection, provisioning and result demultiplexing. Scanner implementations
  stay Rust plugins in `vibeos-lint`; this is the remaining runner layer.
- `tests/qemu/cli/{diskspace,hostdisk,prune,trim,retention,hostlock,buildpolicy,warmth}.py`:
  remaining admission/maintenance decisions and facade cleanup. Reuse existing
  native policy/provenance mechanisms. Host-jobs remains the CPU/memory admission
  owner; preserve cross-worktree locks, maintenance interlocks and dry-run plans.
- Audit `land.py` and `queue.py` with the integrator before choosing migration or
  retirement. The legacy train is held; do not reactivate it as part of a port.

Acceptance: public contracts and refusal behavior remain covered, native code owns
newly migrated decisions, and deletion plans cannot touch live worktrees or paid
translation artifacts. Retain external handle adapters until callers move.

Progress (`lint-rs`):

- Done: the lint verdict layer (`vibeos-lint --verdicts`: classify, baseline
  capture, finding keys); `lints.py` is now an adapter. Every source lint was
  already native.
- Done: `trim` (`vibeos-build/src/trim.rs`). The post-build trim calls it
  directly; `trim.py` is an adapter. Parity: 75 plans, 9,907 paths, 0 mismatches.
- Done: `prune` decisions (`vibeos-build/src/prune.rs`): worktree discovery, live
  QEMU and in-flight build attribution (including canonical bind sources),
  protection gates, du-style sizing, idle window, retention and headroom budget.
  `prune.py` keeps the maintenance lock, the printout and the delete loop, which
  rechecks liveness natively before every directory. Its Python logic tests moved
  to Rust. Parity against the Python planner on frozen live/active facts, whole
  fleet: every keep/PRUNE decision and skip reason matched across plain,
  retention+headroom, idle-window and refused-budget runs. Idle trees were
  byte-identical; only trees with live writers drifted by <0.5 G between passes.
  Validating it found that a non-finite float flag (`--retain-gib inf`) parsed as
  *no* budget. The argument grammar now refuses it.
- Done: disk admission (`vibeos-build/src/diskspace.rs`): host policy, volume
  accounting (backing mounts included), live-lease scan and orphan reaping, the
  hopeless/contention verdicts, `disk-status`, and the `hostdisk` probe, ENOSPC and
  admission lines. `diskspace.py` keeps only the admission flock and the lease fd,
  which must outlive the command. Parity against HEAD on the live host covered
  every status key, probe verdict and admission line.
- Next: `warmth`/`retention`, then `hostlock` (with
  the maintenance interlock it owns), last. The `land`/`queue` audit is done with
  the integrator, who has offered.

### 3. Remaining diagnostic and measurement workflows

Confirmed Python implementations to split into reviewable batches:
`gdbengine.py` / `debugcmd.py`, `repro.py`, `difftest.py`, `stress.py`, and
`winref.py` under `tests/qemu/cli/`.
Reuse existing Rust PE, symbol, VM, farm and reporting mechanisms. Preserve
one-Cargo-invocation-per-difftest-crate isolation, genuine C oracles, reference-VM
identity and diagnostic ownership. Embedded GDB Python may remain an external
API adapter where GDB requires it.

Acceptance: compare actual outputs with the old implementation and exercise the
external debugger/VM boundary when relevant; native fixture tests alone do not
prove live transport behavior.

### 4. Native CLI entry point and compatibility retirement

Once the remaining workflows have native owners, move process entry and dispatch
onto the shared native libraries. Argument grammar, execution, reports and
build/gate decisions already have native owners. Audit imports and users before
removing facades; update documentation and tests with each cutover.

Acceptance: maintained commands run through the native front door, source/media
freshness checks and broker ownership remain enforced, and retained Python leaves
have an explicit external-API reason.

### 5. Translation tooling, last

`tools/rustconv/` remains a separate final migration program, following the user’s
chosen ordering. Inventory its boundaries before defining batches. Preserve paid
responses, replay/recovery, seals, audits and spend controls. Offline fixtures and
existing artifacts are the initial oracles; paid retranslations are not tests.

## Delivery discipline

Keep source rules in `vibeos-lint`, generators in `vibeos-codegen`, build mechanisms
in `vibeos-build`, and test/diagnostic mechanisms in `vibeos-test` or their existing
native owner. Do not create a second broker or evolving Python implementation.
Before each chunk, merge current mainline and reconcile behavior changes in Rust.
Run source checks and relevant native/adapter/tooling tests; use real boots and
raw evidence for changes to build or VM behavior. Keep source fixed during gates.
Checkpoint coherent chunks and give the user the branch and commit for integration.
