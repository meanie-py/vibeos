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

The coordinator port is complete; replay equivalence is not established.
Trace: `~/vibeos-rr/overnight-cli-20261107-arm64` at `51b917c7f84`.
Durable results and independent serial normalization:
`~/vibeos-rust-staging/overnight-cli-20261107/rr-live-evidence/results.json`
and `serial-verification.json`.

- Make replay completion comparable with recording completion. The real serial
  verification exited 1: 2,083 replay lines matched a prefix of 2,230 recorded
  lines, with no differing shared lines. Both runs reached desktop; the tail
  remains unproven.
- Resolve pause-time I/O and interrupt-entry fingerprint comparability. The
  strict checker completed all 10 requests: 3 exact, 7 inconclusive, none skipped
  or unprocessed; it exited 2 with `all_matched: false`.
- Keep the previously documented first-checkpoint arming problem tracked. This
  particular run skipped none, which does not prove the startup race is solved.

Acceptance: actual recordings replay through the required comparison boundary;
complete exact coverage is required for an equivalence claim. Partial,
inconclusive and diagnostic-error results retain their nonzero outcomes.
Do not change markers, comparison policy or uncertainty handling to get green.

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
