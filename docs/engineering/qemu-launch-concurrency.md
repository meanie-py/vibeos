# QEMU launch concurrency

The harness prepares media before acquiring the host-wide port handshake lock.
This is the default for `QemuVM` consumers: smoke, gate, farm, pytest, and durable
checkpoint capture/restore. The accelerator and boot CPU admission rules remain
in force.

## Ownership and lock boundaries

1. Resolve the accelerator and acquire normal VM/CPU admission.
2. For Windows, briefly acquire the existing launch lock to run due cleanup and
   register the run's PID-owned staging directory. Cleanup protects live Linux
   owners even before their Windows QEMU process exists.
3. Prepare firmware variables, scratch/USB disks, and architecture media. Pin
   read-only media through the existing content cache. Each VM owns writable
   media. This work runs outside the port handshake lock.
4. Acquire the launch lock, choose hostfwd/GDB ports, bind transport listeners,
   spawn QEMU, and complete the serial/QMP handshake. Release the lock before
   waiting for guest boot. Bind-race retries replace ports and transport clients;
   prepared media is reused because these failures precede guest execution.

`ArchConfig.prepare_args()` performs media work; `network_args()` selects the
network ports. The existing `qemu_args()` interface still composes both for
callers outside `QemuVM`. Checkpoint restore prepares and verifies its pinned
media through the same lifecycle, without changing snapshot contents or rollback
checks. Preparation errors call normal teardown to release admission and remove
private staging.

Each `*_launch.json` now records `media_prepare_seconds`, `launch_seconds`,
`handshake_seconds`, and `launch_lock_held_seconds`. Existing
`launch_lock_seconds` means time waiting for the final handshake lock;
`cleanup_lock_seconds` records the earlier registration wait.
`media_prepare_seconds` includes that cleanup wait and `cleanup_seconds`.
`prepare_seconds` retains an aggregate of media preparation and process launch;
these component and aggregate timings must not be added together.

## Validation

Regression tests exercise concurrent preparation with real lock exclusion,
preparation-failure teardown, deferred network-port allocation, checkpoint
preparation before GDB allocation, and cleanup while a live harness is still
preparing media without a QEMU process. Existing hostfwd/GDB retry tests remain
in the tooling suite.

ARM64 checkpoint verification with the new lifecycle proved RAM, disk, all-vCPU
register, and guest InterruptTime rollback. A three-second host pause did not
advance the restored guest clock: 0.47 guest seconds elapsed during 0.611 seconds
of execution. Artifact:
`/tmp/vibeos-cli/20260919-232401-95ptqs7i-checkpoint-verify-saved/summary.json`.
Its 24.615 seconds of media preparation occurred outside the handshake lock;
launch plus handshake held that lock for 2.222 seconds.

An earlier launch of the **same checkpoint bundle** spent 21.198 seconds in
preparation while holding the global lock:
`~/vibeos-checkpoints/arm64-window-final/attempts/run-iejffl32/vm/arm64_launch.json`.
The new launch's 2.222-second lock hold above is the concrete improvement; it
is not a claim that an individual checkpoint restores ten times faster.

All three architectures and networking passed three post-change runs; screenshots
were inspected. Source checks passed at
`20260919-233405-5nuxximx-check-all`. All 57 `check-tooling` checks passed, including
the new regression tests; direct VM-module Ruff and Pyright checks also passed.
The first successful post-change run, including warm build refreshes, is
`20260919-233350-ptp9fsxi-gate-i386+amd64+arm64`.

## Execution trials

Each row ran the three desktop smokes plus networking with
`gate --no-build --no-lint --no-conformance --retries 0`. These are execution
trials, **not full landing gates**. Cold runs used a fresh
`VIBEOS_QEMU_WINDOWS_STAGING` directory; warm runs reused it. The benchmark's
Windows staging directories were removed afterward.

| Launcher | Cache | Wall seconds | Artifact directory under `/tmp/vibeos-cli/` |
|---|---|---:|---|
| Before | Cold | 68.824 | `20260919-232834-97pqmhj1-gate-i386+amd64+arm64` |
| Before | Warm | 76.220 | `20260919-233209-_gbsjr23-gate-i386+amd64+arm64` |
| After | Cold | 85.707 | `20260919-233648-dxmk4n7m-gate-i386+amd64+arm64` |
| After | Warm | 55.105 | `20260919-233913-gxzaz9mk-gate-i386+amd64+arm64` |

**No overall gate speedup is established by these samples.** Other workloads
shared the host: the old warm net launch waited 28.057 seconds for admission;
the new cold ARM64 launch waited 12.547 seconds. The freshness guard required
warm builds when switching harness versions. The UP/MP kernel hashes matched
across versions, but the rebuilt boot images did not; `media-comparison.json`
in the after/cold artifact directory preserves both hashes. These are not a
controlled comparison with identical media and host load.

The new cold desktop launches held the handshake lock for 0.620–1.129 seconds;
networking held it for 2.961 seconds. The cold run still spent 11.935–23.322
seconds preparing media outside the lock. The warm run's desktop lock holds
were 0.639–0.651 seconds, with networking at 3.052 seconds. Independent regression
tests verify that another preparation can actually enter during this phase,
while a second port handshake cannot enter the protected section.

The initial clean-build run
`20260919-231300-6ga54o15-gate-i386+amd64+arm64` verified all three builds but had
an ARM64 launch failure before the refactor was applied: QEMU stderr contained
`WSL ... UtilAcceptVsock:273: accept4 failed 110`, and serial never connected.
It is retained as a failed validation attempt, not a boot-time comparison.

## Remaining work

- Content-cache fill serialization is addressed by the subsequent
  [concurrent media-cache change](qemu-media-cache-concurrency.md). Publication,
  pinning and pruning still share the original lock for compatibility.
- The separate detached `debug start` implementation in `gdbengine.py` still
  prepares media under the launch lock. Migrating it needs ownership protection
  for its detached staging and a verified bind handshake; it does not use the
  `QemuVM` lifecycle changed here.
- Native record/replay's additional trace transformation still happens during
  command assembly under the lock. It requires its own compatibility checks.
- Fatal-serial handling is another measurable performance task: the old-launcher
  warm trial `20260919-233008-ostze2gd-gate-i386+amd64+arm64` emitted an ARM64
  `BUGCHECK 0x1E` at PC zero, then waited 120 seconds of silence before reporting
  `KNOWN_FLAKE`. Detect terminal failures promptly while retaining the complete
  crash dump; keep these failed runs out of successful-boot timing comparisons.
