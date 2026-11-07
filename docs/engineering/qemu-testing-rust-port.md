# QEMU/testing mechanisms in Rust

`tools/test-infra` provisions the pinned stable `vibeos-test` binary. The existing
`vibeos-cli` commands and Python test APIs are compatibility entry points;
`tools/qemu_infra_compat.py` owns the typed transport, and
`tools/qemu_infra_runner.py` binds CLI configuration and Run/Stage presentation.

The native modules own these mechanisms:

- `vibeos-build::provenance`: Git/content fingerprints, source/dependency closure,
  copied PE evidence, artifact references, stamps and freshness refusal. Build and
  test consumers use the same implementation.
- `verdict`, `debugcon`, `conformance`, `ladder`, `smoke_checks`: serial decoding,
  crash precedence, symbol resolution, assertion identity and baseline ratchets,
  guest verdicts and requested hardware checks.
- `publication`: verified local byte publication, immutable cache pins and pruning.
  Windows wrappers retain strict Windows-visible path validation.
- `config`, `media`, `backend`, `protocol`, `admission`, `vm`, `lifecycle`: launch
  plans, media preparation, QMP/serial streams, shared admission and VM ownership.
- `smoke`, `smoke_run`, `ladder_watch`, `farm`, `conformance_run`: bounded watches,
  launch retries, repeated boots, payload admission, sweeps and farm workers.

Use the native modules for new testing mechanisms. Python may adapt existing CLI
arguments, configuration and reports; it must not grow a second implementation of
the migrated algorithms. The resource JSONL connection owns its sessions. Metadata
connections are separate so a long VM watch cannot block an unrelated metadata
request. Source changes invalidate a connection rather than mixing implementations
inside a live session.

The VM guardian runs independently of the API process. It holds admission leases
until cleanup is confirmed and identifies Linux processes by their incarnation,
not a name match. Windows cleanup uses the recorded QEMU UID. An explicit debug
handoff transfers ownership. Existing host-jobs and hostlock capacity policies
remain authoritative; the native implementation speaks their existing protocols.
Admission timeout refuses launch. It does not run a VM without a lease.

`record_replay`, `pc_sample`, `replay_sample`, and `diagnostics` own trace
transforms, media/shape/finalization guards, replay decoding, per-vCPU sampling,
symbol attribution, and guest-state fingerprints. VM diagnostic workers execute
`vibeos-test --diagnostic-worker` directly with the existing parent-death and
sidecar-health checks. Each sampler publishes its own result JSON; concurrent
workers share only an append-only diagnostic log. Python modules retain
compatibility DTOs and QMP bindings;
there is no Python fallback. Replay events cross that compatibility API in bounded
pages; native divergence checks stream the trace directly. RAM fingerprints stream
through a FIFO in 1 MiB chunks and preserve Python's BLAKE2b-8/zlib artifact format.

`rr_cli` owns trace lookup, retention, transcript normalization/comparison,
quiescence, and progress monitoring. A quiescence timeout refuses finalization.
`rr_commands` now owns all nine RR command workflows and report presentation,
including record finalisation, replay refusal, serial verification, divergence
coverage and retention. `rrcmd.py` adapts arguments and execution/display I/O;
commands still use the same guarded smoke/conformance entry points.

Native tests run with:

```sh
cargo test --offline --locked --jobs 1 \
  --manifest-path tools/vibeos-test/Cargo.toml \
  --target-dir .cache/vibeos-build/target -- --test-threads=1
```

`vibeos-cli check-tooling` includes this suite alongside the build/codegen native
suites and the existing Python fixture tests. Full validation also requires the
source gate and real desktop smoke on i386, amd64 and arm64. Native unit controls
alone do not establish boot compatibility.

Standalone file transports evaluate caller liveness predicates through explicit
callbacks inside native waits. Required serial-log failures stay visible to every
consuming read before a marker can succeed; the drainer preserves raw evidence
until teardown. Command reports carry an explicit timeout bit, including when a
program itself returns exit code 124.

Payload specifications are parsed by `vibeos-build::payload_spec` for build and
smoke consumers. `vibeos-test::payload` owns launch/idle budgets, verdict-line
completion, silence samples, serial-triggered screenshots, render deltas, input
scripts, retries, VM survival checks and concurrent HTTP body/accounting checks.
`tests/qemu/test_payload.py` supplies VM I/O and pytest outcomes. Rendering uses
native PPM decoding and PNG publication, so missing Pillow cannot skip a gate.
The modrun host grader is native too; unknown providers or graders fail loudly.

Payload media staging, borrowed-output rebuilds, cross-architecture guest build
receipts and complete PE closures live in `vibeos-build`.

`vibeos-test::characterize` owns the apitest characterization loop, worst-result
merging, fault attribution and cumulative same-suite stop guard. Its Python entry
point retains CLI parsing and uses the existing command/report bindings. Each
round supplies its own `VIBEOS_ARTIFACT_DIR`; a missing serial is an error, never
a reason to select the newest unrelated pytest log. Evidence is archived before
quarantine writes, and an archive failure or existing destination fails loudly.
The payload drain, pytest and VM watchdog bounds remain 7200, 7800 and 8400 seconds.
Reproduction runs retain their partial-coverage warning and quarantine opt-out.
Resolved timeouts and failed launches cannot be accused of a later bugcheck merely
because they have no exit marker; only a still-in-flight row is eligible.

Wine `todo_wine`/`todo_wine_if` source attribution lives in
`vibeos-test::wine_failures`; the report entry point only formats native results.
Chrome's host installation provider joins hostapps, vendor drivers and Pi Wi-Fi
under `vibeos-build::payload_providers`, so native payload builds accept its
`binaries_from` entry without executing Python providers.

Physical Pi 4/5 bench controllers, doctor checks, staging and boot verdicts live in
`vibeos-test::bench*`. They consume the maintained arm64 build configuration and
use `vibeos-build::provenance` before staging. Shell scripts retain external device
commands. Python entry points adapt CLI options and reports.

`tools/bench-infra` provisions portable `vibeos-bench` services: raw Pico TCP UART
capture, serial arrival-time reports, read-only HTTP ranges and TFTP transfers.
HTTP and TFTP run as Windows executables because the Pi cannot reach WSL NAT.
`tools/bench-infra --windows-path` builds the Windows LLVM target; install its
pinned standard library with `rustup target add --toolchain 1.92.0
x86_64-pc-windows-gnullvm`. The Linux and Windows caches have separate publication
locks and source receipts. Native bench tests run through `check-tooling`.

Bench leases remain shared with existing device tools, including the conservative
Pi 4/OptiPlex alias. Unreadable or corrupt lease state refuses control; inspect it
and use an explicit force-drop to recover it. Native serial receipts include the
Linux process start ticks, configured host and capture directory so a reused PID
cannot validate another listener. Hardware validation still requires a real lease,
fresh payload, new serial slice and HDMI evidence; host fixtures do not substitute
for a physical boot.
Serial desktop verdicts require both `[EXPLORER] _tWinMain entered` and `shell ready`.
Win32k's earlier `shell ready` line alone is insufficient; panic/bugcheck output
in the same slice takes precedence over success markers.

Service mutations take the bench lease and a per-bench service lock. Windows
launchers publish full process ownership before the controller confirms listening
sockets; Linux interop launch stubs are then reaped independently of the server.
CIM start times are compared at their microsecond precision against opened Windows
process handles. Firewall migration changes only the application filter of the two
existing named Pi 4 rules and verifies that their network scope is unchanged.
`pi4 services recover --pid PID --started TICKS` explicitly recovers an interrupted
native launch after inspecting that exact process; ordinary commands refuse an
unowned matching process. `--allow-stale` remains an explicit bench debugging
override and is recorded in payload manifests; it cannot bypass a broken provenance
oracle or a lease refusal.

When physical gear moves between boards, set `VIBEOS_BENCH_LEASE` to its original
lease name alongside the destination board's plug/Pico configuration. This keeps
both board names contending for one physical resource; it does not change which
board owns the payload or logs. For the Pi 4 gear's existing conservative alias,
that shared lease name is `optiplex`.

Bulk RAM capture uses FIFO readiness polling and a bounded transfer deadline;
interactive QMP requests recover their normal timeout after each transfer.
`rr record` publishes `record-diagnostics.json` separately from trace finalisation.
Requested RAM/PC sampling must produce valid rows, and a worker error fails the
command even if earlier samples exist. A guest bugcheck can still yield a useful,
successful recording when the requested host diagnostics completed. Host sampler
failures propagate as diagnostic errors rather than guest-death classifications.

Smoke refreshes its watch configuration from the launched VM's handles, so
record/replay's extended watchdog budgets reach the outer watch as well as the VM.
A control reproduces the pre-launch/post-launch configuration mismatch.

On `15634cca5810`, a real ARM64 recording reached desktop with 12 RAM fingerprints
and 108 PC samples. Independent Python zlib/BLAKE2b8 decoding verified all 883
unique RAM chunks. Trace: `~/vibeos-rr/native-diag-bulk-20261106-1634-arm64`;
boot evidence: `/tmp/vibeos-cli/20261106-163427-aioj3w4p-smoke-arm64`.
Trace finalisation and recording diagnostics passed; this records evidence about
recording, not a claim that playback reached desktop.

Finite replay comparison workers finish before smoke tears down a desktop VM;
continuous PC workers remain tied to VM lifetime. The VM lifetime watchdog still
bounds comparison completion, and worker failures remain diagnostic errors.
A finished comparison loop is not proof of equivalence: uncertain, skipped,
empty or partially compared fingerprint sets produce an inconclusive verdict and
CLI exit 2. `all_matched` requires every requested fingerprint to match exactly.

The completion/coverage fixes were exercised on `96011f9acf7` with
`rr diverge native-diag-bulk-20261106-1634-arm64`. The replay reached both desktop
markers; raw serial has no bugcheck or Rust panic, and its saved screenshot shows
the desktop. Evidence: `/tmp/vibeos-cli/20261106-165507-rnywt0u4-smoke-arm64`.
Smoke waited 32.677 seconds after desktop settling for the finite checker.
The final verdict has `completed: true`, four exact matches, seven inconclusive
pause-time I/O checkpoints, and one skipped checkpoint (already past its icount).
There are no unprocessed checkpoints or definitive mismatches. CLI exit 2 and
`all_matched: false` correctly leave full equivalence unproven. The prior run's
false-success output is in `.cache/build-infra/diagnostics-rr-bulk-diverge.log`;
its verdict was preserved at
`~/vibeos-rust-staging/diagnostics-20261106/diverge-before-completion-fix.json`.

Follow-up work: arm the first replay checkpoint before it can be passed, and
make pause-time I/O checkpoints comparable without weakening the inconclusive
classification. The RR coordinator has since moved into `rr_commands`. Shared CLI
configuration, argument parsing, subprocess ownership, artifact publication and
summary rendering now live in `vibeos-build::{cli_arguments,cli_artifacts}`.
`vibeos-test::tooling` plans and schedules tooling checks; the build library owns
its locked content receipts. Python supplies interpreter/package discovery facts
and retains Namespace/Run/Stage DTOs for existing callers. Failure-signature policy
is shared through `vibeos-test::cli_report`.

Build/gate composition now lives in Rust: `vibeos-build::cli_build*` owns build
preflight, reconciliation, generated-header barriers, target passes, immutable-store
acceptance and provenance. `vibeos-test::{gate_run,gate_child,gate_stamp}` owns
pipelining, child verdicts, serial lane retries and push stamps. The existing
host-jobs broker remains the admission authority. Python compatibility objects
and adapters hold external lease/context/task handles and pass data; they do not
choose build stages or gate retries. Canonical toolchain provisioning is native.

A gate with `--no-lint` cannot mint a push stamp. Requested conformance/network
lanes fail visibly when their implementation, payload or baseline is unavailable;
they are recorded as ungraded rather than silently omitted. Child consoles,
command output and incomplete-run markers remain durable. Gate child launch failures
retain the OSError compatibility contract; a program returning 127 is a separate outcome.

Host-job process-contract fixtures now own a private process group through an
EOF guard. Unwinding or killing the test owner closes the ownership pipe; the
guard kills only that pinned group before fixture-directory removal. A regression
control panics with a live barrier child and requires its captured output to reach
EOF. A separate before/after probe reproduced the old failure masking: the test
process was already a zombie with exit 101, while inherited output descriptors
kept the outer subprocess waiting. Evidence is in
`~/vibeos-rust-staging/host-jobs-flake-20261106/results.json` and adjacent probe logs.
The process-contract timeout remains 270 seconds. Batch 128's original tooling
log confirms Cargo had already returned 101 when Python timed out on its pipes.
The original was supplied at
`/tmp/claude-1000/-home-curi-Projects-os-playground-vibeos-integration/7678f5f2-0201-4aed-b78e-717263573b59/scratchpad/tool128e.out`;
a durable copy and its SHA-256 receipt are at
`~/vibeos-rust-staging/host-jobs-flake-20261106/batch128-tooling.out` and
`batch128-evidence.json`. Batch 128’s initiating Rust test remains unidentified because
pytest truncated the captured output. Any recurrence still needs that underlying
failure investigated; the fixture guard now lets the actual test failure reach
the caller rather than being masked by a pipe timeout.

The 2026-11-07 cold gate at
`/tmp/vibeos-cli/20261107-035206-lPZuDY-gate-i386+amd64+arm64` exposed a later
process-contract failure without the pipe hang:
`queued_compilers_fill_the_pool_and_report_run_ownership` saw 22 of 24 ownership
records. Its eight-active-worker assertion passed, but it assumed every queued
launcher had registered after a 100 ms sleep. The fixture now waits for all 24
registrations, preserving the eight-active-worker and ownership assertions.
A control holds two launchers before broker entry until the other 22 are visible;
it reproduces the old 22-versus-24 failure and passes with the registration barrier.
All 19 unit and 17 process tests passed in the isolated control crate. The outer
270-second timeout is unchanged. Evidence, the captured failure and SHA-256 receipt
are under `~/vibeos-rust-staging/overnight-cli-20261107/host-jobs-registration/`.
This identifies the later failure; the truncated batch 128 log cannot establish
whether its initiating test was the same one.

That cold gate passed all three cold builds, desktop boots on all three arches
and amd64 BIOS, network, conformance, storage, source/artifact checks and difftests;
only the tooling fixture failed. ARM64 storage recovered on retry. Its first
attempt lost time to a 120-second port-handshake wait, then pytest’s global
300-second timer interrupted the boot RPC before host readback. The raw serial
already contained `[STORTEST] ALL OK` and both desktop markers. Storage now uses
the gate’s existing supervised 900-second process-tree deadline instead of that
competing pytest timer. The VM’s operation deadlines and storage assertions remain
in force. A failed diagnostic snapshot preserves the original boot exception.
The isolated timer control fails with the inner timer, passes with the owned
deadline, and still terminates a hung child at the outer deadline. Results are in
`~/vibeos-rust-staging/overnight-cli-20261107/stortest-deadline/`.
