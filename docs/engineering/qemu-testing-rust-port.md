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
