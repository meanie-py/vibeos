# Durable QEMU checkpoints

`vibeos-cli checkpoint` captures a stopped VM and resumes its saved execution in
fresh QEMU processes. It preserves the kernel image and CPU topology separately:
an MP kernel with one vCPU remains an MP-kernel run. This is general execution
layer tooling, not a special mode for the SMP lane.

## Use

```sh
./vibeos-cli build --arch arm64
./vibeos-cli checkpoint capture --arch arm64 --smp 1 --name before-window \
  --until '[SMSS] _main entered'
./vibeos-cli checkpoint inspect before-window
./vibeos-cli checkpoint verify before-window --pause-seconds 5
./vibeos-cli checkpoint run before-window --repeat 100 \
  --until 'shell ready' --timeout 30
./vibeos-cli checkpoint run before-window --gdb --until 'shell ready' --timeout 600
```

`--until` is a literal serial marker. Capture recognition is asynchronous, not an
instruction breakpoint: choose a marker sufficiently before the window. A run
passes only on **new** output after restoring; buffered serial is drained while
stopped. `--fail-on REGEX` overrides the default BUGCHECK/Rust-panic patterns and
wins if failure and success arrive in the same read. Timeout is a failed attempt,
not a pass. Deadlines and elapsed durations use monotonic time.

The store defaults to `~/vibeos-checkpoints`; `--root` selects another store.
Names cannot overwrite existing checkpoints. A bundle owns its qcow2 snapshots,
boot media, firmware, launch settings, available kernel/HAL binaries and symbol
indexes, capture transcript and provenance. Media and auxiliary files carry
SHA-256 hashes. QEMU executable/version identity is checked; Windows DLLs and
implicit ROM files are checked too. Use the same QEMU installation for restore.

The VM is stopped and saved, then QEMU exits to flush writable images **before**
export. The bundle is published by directory rename after all files are copied.
Restore verifies hashes, stages private writable copies, starts QEMU paused,
loads the snapshot and compares all-vCPU registers and CPU count before resuming.
It does not consult the current worktree's boot images or fall back to a new boot.
Read-only Windows media can use the ordinary content cache; writable images never
share a cache inode. Snapshot-bearing images are copied, never converted (image
conversion would discard the internal snapshots).

Normal admission, launch locking, process ownership, reaper and cleanup remain
those of `QemuVM`. Checkpoint stores inside Windows temporary staging are refused.
Named bundles are not automatically pruned. Windows cleanup only reclaims attempt
staging; the durable bundle remains outside it.

## Attempts, failures and debugger access

`attempts/run-*/` under the bundle holds the actual launch, QEMU stderr and serial.
Numbered attempt directories contain `serial.log` and `result.json`, with restore
and execution durations separately. A failed window additionally preserves CPU
state and a `failure/` checkpoint, naming its parent seed. If QEMU itself dies,
the command fails and reports that a failure snapshot could not be saved; logs
remain. Run `checkpoint inspect` or `checkpoint run --gdb` on a failure bundle's
full path to inspect it in another process.

`--gdb` prints the address and leaves the restored machine stopped. Attach with
a compatible GDB and continue when ready. Its timeout includes debugger time.
Use one attempt per debugger invocation: loading a snapshot with an attached
GDB client crashed Windows ARM64 QEMU during verification (11.0.50, Windows
Application Event 1000 at 2026-09-19 02:50:50, access violation). The CLI refuses
`--gdb --repeat N` for N > 1. Ordinary repeats do not attach GDB. The verifier
uses separate debugger connections around restores, and mutates known data
rather than firmware code or page tables. Debugger/JIT behavior after code
modification remains a separate unproven capability.

`checkpoint verify` deliberately changes guest RAM and scratch-disk bytes in a
private attempt, proves the changes took effect, then checks rollback. It also
checks all-vCPU registers and the guest's `SharedInterruptTime`: the value must
advance while executing, exclude a host pause, and return to its saved value after
restore. It needs a capture late enough for SharedUserData and the timer to exist;
an early firmware checkpoint remains usable but cannot pass that timer proof.
Disk reads use completed QMP backup jobs, not an external reader racing an active
qcow2 cache. HMP `qemu-io` prints its read result outside the QMP reply, so an empty
reply is not a disk-read witness.

## Supported configurations and limits

- Capture: Windows-host TCG on ARM64, amd64 and i386; native ARM64 TCG is wired
  through the same interface but has not yet been exercised in this lane.
- Multiple vCPUs stay multiple vCPUs. No switch to one vCPU or single-thread TCG.
- Networking is explicitly isolated with `-nic user,restrict=on`: local DHCP
  remains, outside connections are blocked. Removing ARM64's NIC changed the
  firmware boot path; leaving a disconnected NIC stalled the guest's network
  initialization. Neither is an acceptable default for checkpoint capture.
- Explicit network backends, USB passthrough, unknown external-state options,
  writable raw disks, backing chains, RR mode and hardware accelerators are
  refused. Restore cannot reconstruct a connection to an external peer.
- Freshness checks apply to capture. Pinned artifact checks apply to restore.
- Exact-PC capture, interactive checkpoint management and explicit retention
  commands remain follow-ups. Do not use Windows scratch cleanup to delete seeds.

## Retry is not deterministic replay

A checkpoint supplies the same starting state, not the same future CPU schedule
or external inputs. Repeated restores from one seed are correlated: preserve
multiple independently booted seeds when investigating a rare failure. Do not
count 500 restores as 500 independent boots. A failure checkpoint preserves the
faulted state, not the instructions that led to it.

The existing `vibeos-cli rr` path remains separate and unchanged. It requires
native TCG and one vCPU; the motivating ARM64 MP-kernel/one-vCPU case meets that
restriction. The playback stalls and debugger limitations documented in
`qemu-record-replay.md` still need resolution before promising deterministic
reproduction. Host NTP adjustments and guest virtual-clock replay are distinct.

## Evidence (2026-09-19)

Each following saved window reached `shell ready` on three restores in a fresh
QEMU process, with all-vCPU register equality checked before every continuation:

| Seed | Restore time | Execution from snapshot |
|---|---:|---:|
| ARM64 MP, 1 vCPU | 0.30–0.66s | 1.08–2.64s |
| ARM64 MP, 2 vCPUs | 0.57–0.72s | 1.20–2.36s |
| amd64 MP, 1 vCPU | 0.21–0.26s | 2.84–3.70s |
| i386 MP, 1 vCPU | 0.30–0.41s | 4.28–5.07s |

These are window timings, not whole-command timings: installation hashing and
initial private staging add startup cost, amortized over repeats. They are not a
measurement of any specific rare bug's reproduction rate.

Durable evidence lives under `~/vibeos-checkpoints/roundtrip-{arm64,amd64,i386}`
and `roundtrip-arm64-mp2-isolated/attempts/`. The one-vCPU ARM64 timer proof in
`roundtrip-arm64/attempts/run-8k681at8/proof.json` measured 0.47 guest seconds
across 0.636 execution seconds after a five-second host pause. i386's
`roundtrip-i386/attempts/run-k228rtju/proof.json` measured 0.50 guest seconds across
0.510 execution seconds after a three-second pause. Both also proved RAM and disk
rollback. A deliberately triggered failure oracle preserved an amd64 failure
bundle in `roundtrip-amd64/attempts/run-u6nvxfo1/0001/failure/`.
The two-vCPU destructive proof is
`roundtrip-arm64-mp2-isolated/attempts/run-hf7ocfle/proof.json`: RAM, disk,
both vCPUs and timer rollback passed; 0.45 guest seconds advanced across 0.608
execution seconds after a five-second host pause.

QEMU references: [VM snapshots](https://www.qemu.org/docs/master/system/images.html#vm-snapshots),
[record/replay](https://www.qemu.org/docs/master/system/replay.html),
[QMP block jobs](https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html).
