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
- Done: `warmth` build receipts (`vibeos-build/src/warmth.rs`). The native build
  records its receipt and snapshots its tree directly, with no Python callback.
  `land`'s close-enough check keeps its inert predicate in `tools/docs_only.py`,
  which it shares with the pre-push hook, and hands its verdicts to native.
  Parity: identical reachable-Python set (198 files, 4.5x faster) and `missing`
  verdicts, with and without a master merge.
- Done, as ONE cluster: `retention`, `maintenance` and the live `hostlock` surface.
  `vibeos-build/src/retention.rs` owns the owner/kill-switch decision, policy
  validation, the systemd units and the fleet compatibility verdict; the flock, the
  control-file transaction and systemctl stay in the facades.
  `vibeos-test/src/hostlock.rs` owns the `host-status` snapshot (lease probes, holder
  liveness, protocol skew, the read-only queue view, the admission shadow), its
  renderer, `looks_starved`, the landing boot reserve and `foreign_boot_holders`.
  Parity: 250 randomized lock directories with real held flocks and pinned clocks, plus
  the live host, were identical; policy, unit and owner checks matched HEAD (the only
  wording difference is a JSON `Infinity` token, refused either way). Two deliberate
  changes. First, a lease file holding valid non-object JSON no longer crashes
  `host-status`. Second, the fleet check no longer compares `maintenance.py` byte for
  byte: a lane is compatible when its copy declares the same `INTERLOCK_PROTOCOL` or is
  the exact pre-declaration file, which implements protocol 1. Under the byte rule, any
  edit to that file (this one included) blocked enabling retention until all 24 lanes
  merged it.
  Untouched: `land_lock` and everything it calls (`_acquire_one`, the FIFO tickets,
  `_self_meta`/`job_class`, `_is_held`, and Python `queue_status`, which `land_queue`
  reads). The native snapshot carries its own read-only queue view.
  Settled with the integrator (2026-11-07): port only the live hostlock surface.
  `land.py`, `queue.py` and the `land_lock` family stay in Python. The integrator's
  `land --direct --cold --skip-lint` is the only path that pushes master, and it uses
  just `land_lock`, `land_holder`/`land_queue` and the queue hold-file reader. The
  rest of the train (submit/conductor/INCONCLUSIVE) is dead while it is held.
  Retiring the train is the user's / control-plane's decision. If it is retired,
  `--direct` collapses to merge+gate+push, so porting it first would be wasted
  work. `land --direct` must keep working at every intermediate commit, and any
  change touching it lands in a batch of its own. Keep the lease file protocol
  unchanged.
- Done: the dead Python lease API is retired (`acquire_cpu_lease/share/exclusive`,
  `cpu_*`, `release_all`, `acquire_boot_lease`, `build_permits`, `_max_builds`,
  `buildpolicy.guard`; hostlock.py 897 -> 636 lines). Its tests were the only coverage
  of boot admission semantics, so they moved onto the code that admits boots now:
  11 Rust tests in `vibeos-test/src/admission.rs` cover the landing reserve and its
  overflow, boot-vs-boot throttling, the build/boot pool split, native-TCG boots in
  the CPU pool, FIFO tickets, orphan reaping, the fairness opt-out, broker-off rules,
  and a pause releasing a waiter without touching a holder. `_acquire_one`'s FIFO,
  orphan and announce-pair tests now run through `land_lock`, its live caller. The
  policy-cutover tests use a raw flock holder, which is exactly what native admission
  holds. `release`/`take_guard` stay as they are: `release` is the land lock's.
  Native `build_policy_admission` lost its only Python caller (the guard); it can go
  with item 4, as can the build-dir lock, when the build entry itself goes native.

### 3. Remaining diagnostic and measurement workflows

Confirmed Python implementations to split into reviewable batches:
`gdbengine.py` / `debugcmd.py`, `repro.py`, `difftest.py`, `stress.py`, and
`winref.py` under `tests/qemu/cli/`.
Reuse existing Rust PE, symbol, VM, farm and reporting mechanisms. Preserve
one-Cargo-invocation-per-difftest-crate isolation, genuine C oracles, reference-VM
identity and diagnostic ownership. Embedded GDB Python may remain an external
API adapter where GDB requires it.

Progress: `repro` done (`vibeos-build/src/repro.rs`: PE parse, compare, report,
diff-dirs). Its output matched the Python byte for byte on 15 real kernel/DLL
images across 3 arches, plus non-PE and missing-file cases. `verify`'s two streamed
cold builds and the optional diffoscope stay in the facade.
`difftest` done (`vibeos-test/src/difftest.rs`): discovery, manifest-declared
targets and feature sets, the one-invocation-per-crate plan, graded counts and
verdicts. The existing Python tests now act as the native contract. Identical argv
and stages for all 21 invocations, and a real run passed 26/26.
`stress` decisions done (`vibeos-test/src/stress.rs`: the profile matrix, the
per-profile summary and the diagnosis). No mismatches across 400 random matrices.
The boots and host-core pinning stay with the smoke facade until item 4.
`winref` decisions done (`vibeos-test/src/winref.rs`): the reference VM's QEMU line,
the reading of its COM1 protocol (payload, install and maintenance verdicts), the
reference column graded on the native conformance parsers (including the banked
baseline's text), and the 7-Zip/qemu-img disk arithmetic. Parity against HEAD's
Python: 102 `--regrade` pairs from 17 real kmtest/apitest sweep logs, with and without
a guest floor, `--write-baseline` and `--only`; the three real `winref run` consoles
plus edge cases through `run`/`maint`/`install`; 200 launch lines; 5000 helper inputs.
Zero mismatches. The VM itself (console socket, QMP, the power-off watch, qemu-img,
7-Zip, snapshots) stays in the facade until item 4. The live check found a bug in the
OLD code as well: Windows turned the runner's shutdown into a restart, and the runner ran
the payload again (every output line twice, 15 min). One-shot boots (`run`, `apitest`,
`maint`) now pass `-no-reboot`; install and kmtest keep their reboots.
`gdb start` done: it boots the smoke VM through native `vm.rs` (no second launcher)
and hands it to the calling session with `detach_for_debug`. The lock records the
session as owner and `gdb start` as source, so the guardian, `vibeos-cli kill` and the
reaper all apply. Serial goes to a staged file (`VmConfig.serial_file`), because the
TCP dial-back cannot outlive the launching CLI call. `gdb inspect` relocates from the
`[KRNLBASE]` the kernel printed, instead of an unhinted scan that took 60 s on a
booted guest. Verified live on all three arches (start, wait-for `shell ready`,
inspect regs/lm/bt, stop). `debugcmd` decisions done (`vibeos-test/src/debugcmd.rs`):
the verb table, the session-owner walk, `debug diff`'s normalization and `check`'s
option plan. Parity: the owner walk on 406 live processes, 29k serial lines and 3000
random `check` invocations (output, exit code, run identity and lint calls) were all
identical. One intended difference: the pid/tid rule now normalizes `pid=N` and
`pid: N`, which it used to leave numbered (2436 of 2548 pid lines in the corpus).
That closes item 3 apart from the qemu/*.sh debug toolkit the verbs wrap, which goes
with item 4.

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
