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

The diagnostics bridge deliberately still invokes the existing record/replay and
sampling command authorities. It calls their trace/shape guards, and Rust owns the
resulting sidecars and QMP lifetime. Full migration of those CLI diagnostics belongs
with the later control-plane work. There is no Python fallback when a native
operation fails.

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
