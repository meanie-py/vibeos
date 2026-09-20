# QEMU execution measurements — 2026-09-19

The default remains Windows-host TCG. Read-only media reuse and Windows staging
cleanup are automatic for smoke, gate, farm, and debug launches. No environment
variable or special command is needed to benefit.

Follow-up: [launch concurrency and checkpoint preparation](qemu-launch-concurrency.md).

## Changes and measurements

Durations below use monotonic timers; filesystem wall-clock timestamps are not
a substitute on this host.

Read-only media is SHA-256 keyed, atomically populated, and hard-linked into each
run's private staging directory. Every launch hashes its actual input; matching
size/mtime alone cannot reuse stale media. Writable variables and scratch disks
are copied privately. Cache pruning retains 2 GiB plus pinned images. Active run
links survive cache-name removal. The existing source/media freshness checks
still run before a smoke.

An amd64 ISO took **8.596s** to populate the cache and **0.144s /
0.148s** to hash and pin on two subsequent calls (`/tmp/qemu-perf-cache.json`).
Real launch preparation, including other media and process setup:

| Architecture | Cold preparation | Next two preparations | Three desktop results |
|---|---:|---:|---|
| amd64 | 9.177s | 0.346s, 0.332s | pass / pass / pass |
| i386 | 9.486s | 0.336s, 0.334s | pass / pass / pass |
| arm64 | 9.298s | 0.482s, 0.456s | pass / pass / pass |

Artifact directories under `/tmp/vibeos-cli/`:

- `20260919-014923-mrfcuar9-smoke-amd64`
- `20260919-014924-9hs672e5-smoke-i386`
- `20260919-014952-51f5a_n6-smoke-arm64`

Other lanes were booting concurrently. The first amd64 launch additionally waited
12.684s for the host launch lock. These are preparation measurements, not a claim
of a proportional whole-boot speedup. All nine boots retained the explorer and
shell-ready checks and settle window; screenshots were inspected.

The warm-cache concurrent execution workload
`./vibeos-cli gate --no-build --no-lint --no-conformance --retries 0` completed in
**48.521s**, passing all three desktops and the default net suite. Individual
stage walls: i386 35.910s, amd64 38.535s, arm64 46.862s, net 48.485s.
Evidence: `20260919-015153-gdfj_7b9-gate-i386+amd64+arm64/summary.json`.
The same command with `VIBEOS_QEMU_WINDOWS_ACCEL=whpx` took **150 seconds**,
passed the desktops, and failed networking (13/15). Its net VM spent **64.071s**
waiting for the shared amd64 slot before launching. Evidence:
`20260919-015252-f31nrra2-gate-i386+amd64+arm64/summary.json` and
`smoke-net-a1/amd64-uefi-net_launch.json`. Both workloads used the same media,
cache and harness; this comparison includes the accelerator admission policy.

This is an execution benchmark, **not a full landing gate**: builds, source lints,
and kmtest conformance were excluded, and no push stamp was minted.

## Windows cleanup

`./vibeos-cli windows-cleanup` removed **179 abandoned paths** in six seconds.
Independent `du -sh /mnt/c/temp/vibeos-qemu` measured **53G before, 0 after**;
subsequent boots populated the new cache. Evidence:
`20260919-014330-4p7rk_rg-windows-cleanup-all/summary.json`.

Launches sweep at most once per five minutes. Only recognized tool-owned paths
older than one hour qualify; live Linux owners and Windows QEMU command lines
protect their media. The sweep shares the launch lock, refuses deletion when
Windows process inspection fails, leaves unknown files alone, and prunes
abandoned cache fills and unpinned cache entries. `windows-cleanup --dry-run`
provides a reviewable list. Completed runs still remove their own directory.

## WHPX disposition and remaining investigation

WHPX is **not promoted**. With the final cache/socket changes, amd64 desktop took
53.537s, then networking returned **13 passed / 2 failed** in 79.01s:
`20260919-014953-slb4fvi3-smoke-amd64`. Both inbound tests reached their listener
checks but timed out receiving data (`test_tcp_echo_roundtrip`,
`test_tdi_server_listen_accept`). The earlier spike's absent-listener diagnosis
therefore does not describe this run. QEMU stderr includes a WSAEventSelect
invalid-socket warning; that warning alone does not establish the fault's cause.

Before caching, three TCG amd64 desktops took 43.247/35.213/37.926s
(`20260919-012459-x103nk39-smoke-amd64`); WHPX desktops took 61.277/54.212/59.969s
(`20260919-012312-hq9vqnpt-smoke-amd64`, `20260919-012859-sp5beyqm-smoke-amd64`,
`20260919-013230-j7fc_ffs-smoke-amd64`). i386 TCG took
35.361/35.788/34.033s (`20260919-012615-l9q1jng4-smoke-i386`); WHPX's two
successful samples took 49.428/50.277s (`20260919-013107-30w8fqby-smoke-i386`).
These exploratory runs used an experimental copier subsequently rejected below;
load was not isolated. They supply no case for a WHPX speed advantage.

The all-15-setup-errors incident was a Windows QEMU crash, independently confirmed
by Application Event 1000: PID 77420 at 01:24:23, exception `0x40000015`, image
offset `0x61022b`. The minidump remains at
`C:/Users/Lambda/AppData/Local/CrashDumps/qemu-system-x86_64.exe.77420.dmp`.
The corresponding lane artifact is `20260919-012312-hq9vqnpt-smoke-amd64`.
Raw stack scanning found asynchronous QIO task/socket addresses; it is not a
symbolized unwind or a complete root-cause proof.

The harness unnecessarily requested `reconnect-ms=1000` for ready, single-session
listeners. QEMU's [socket client implementation](https://raw.githubusercontent.com/qemu/qemu/54e84cdc7a/chardev/char-socket.c)
selects asynchronous initial connection when reconnect is enabled. The harness
now connects synchronously and fails at launch if the connection cannot be made.
QEMU stderr is retained on disk, avoiding a full undrained pipe, and boot EOF
retains partial serial plus process status. Subsequent success does **not** prove
an intermittent host crash eradicated. Owed before promoting WHPX: identify the
inbound data failure, repeat network soaks without host crashes, then demonstrate
better gate/farm throughput.

Admission now resolves WHPX before acquiring slots; previously the default `tcg`
placeholder bypassed its cap. Net and UEFI variants now share their base-arch
lock. TCG remains parallel across processes. `smoke --repeat N` itself is
sequential; farm has one worker per architecture.

## Rejected experiment and checks

Increasing DrvFS copy writes from 16 KiB to 256 KiB reduced a copy from about
8.5s to 2.1s, but a later i386 launch failed with `ENOMEM`. Kernel diagnostics
independently showed an order-4 allocation failure in `p9_client_write` for a
256 KiB write while over 39 GiB was available. That diagnostic's PID differs
from the failed launcher, so it supports the mechanism without proving that
specific event. The final implementation retains 16 KiB writes and avoids
repeated transfers through caching.

All three architectures completed cold builds earlier in this session; warm
builds refreshed media after harness edits. Final build artifacts were checked
on disk. `./vibeos-cli check` passed
(`20260919-015035-4mmukf3c-check-all`), and all **56 check-tooling checks passed**.
Cache tests cover content changes with identical size/mtime, writable isolation,
warm reuse, pin-aware pruning, live-process protection, and failed inspection.
A subprocess regression writes 128 KiB to stderr and proves the launcher cannot
block on an undrained stderr pipe.

The final execution rerun also passed all four stages in 73.783s, including cold
cache fills after refreshed builds and concurrent fleet activity:
`20260919-015708-ovxdi2rn-gate-i386+amd64+arm64`. Final source check:
`20260919-020155-j6kzux3b-check-all`.

Repeated tooling validation exposed an independent fixture problem in
`tools/test_nested_cargo_deps.py`: the staticlib variant sometimes executed the
previous value (`33` instead of `60`, or `6` instead of `15`). The fixture now
orders edits after prior artifacts, following `test_build_invalidation.py`,
without sleeps or retries. It also has negative controls that omit nested
tracking and require the stale executable to remain stale. All four cases
passed. This fixes the test's timestamp ordering; it does not establish the
cause of any host clock anomaly or change shipping Cargo dependency logic.

After the fixture correction, the complete `check-tooling` rerun passed all
56 checks. The shipping QEMU modules also passed direct Pyright checking.

A separate 35-second probe subsequently observed `CLOCK_REALTIME -
CLOCK_MONOTONIC` drop **1,123.1 ms**, 9.27s into sampling. This confirms a host
wall-clock step independent of Cargo's reporting; its source is not established.
It also explains why filesystem timestamp differences must not replace the
monotonic benchmark durations. Host clock stability remains a separate follow-up.
