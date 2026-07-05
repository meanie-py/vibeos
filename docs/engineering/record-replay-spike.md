# QEMU record/replay spike — 2026-07-04 (task: deterministic wedge repro)

**Goal:** if QEMU TCG `rr` can record a boot, one bad boot becomes a permanently
replayable artifact — replay with GDB attached, at leisure, killing the
stochastic-repro tax (the ~50% amd64 wedge, the arm64 ghosts: catch once,
replay forever).

**Verdict: NO-GO on QEMU 9.0.2 (host qemu). Both canonical shapes stall
pre-guest-output.** Parked, not abandoned — re-spike criteria below.

## What was tried

Host: native Linux `qemu-system-x86_64` 9.0.2 (Debian backport), TCG,
`-icount shift=auto,rr=record,rrfile=...`, every block device wrapped in
`driver=blkreplay`, `-nic none` (kills the implicit e1000), serial to file.

| Attempt | Shape | Result |
|---|---|---|
| 1 | UEFI (OVMF 4M pflash ×2, snapshot vars) + IDE CD (blkreplay) + virtio-blk scratch (blkreplay) | Stalls **inside OVMF**: event file grows to ~78 MB in the first seconds, then freezes; vCPU at ~2% CPU (blocked, not grinding); serial never gets past the firmware ANSI init. 7 min observed. |
| 2 | BIOS El Torito (SeaBIOS) + IDE CD (blkreplay) only — the documented-working rr shape | Same signature: ~1.3 MB of events then frozen, ~1% CPU, serial empty. |

The ~1–2% CPU is the tell: rr isn't slow here, it's **deadlocked** waiting on
an event that never gets recorded/replayed — a known rr failure mode with
device models outside the narrow maintained set.

## Why parked rather than pushed harder

- rr in QEMU is semi-maintained; 9.0 is known-fragile. Upstream landed rr
  fixes after 9.0; a newer QEMU (10.x) is the first thing to try on a
  re-spike — not more config permutations on 9.0.2.
- The payoff shrank while the spike ran: census + lockdep (incl.
  FAULT-UNDER-WS) already make the wedge family SELF-REPORT on serial with
  the faulting address, and the boot coverage map (tools/coverage/) gives
  green-vs-wedge execution diffs. Deterministic replay is still the dream
  debugger, but it's no longer the only path to these bugs.

## Re-spike criteria (any of)

1. Host QEMU upgraded to ≥ 10.x → rerun attempt 2 verbatim (script shape in
   this doc; `qemu/start.sh` now honors `VIBEOS_NO_KVM=1` for the TCG side).
2. A bug class survives census/lockdep/coverage triage AND reproduces < 1/5
   boots — i.e. the stochastic tax is again the binding constraint.
3. Someone wants reverse-execution GDB specifically (rr replay +
   `reverse-continue` is the only way to get it).

Spike scripts preserved in this doc's history; the durable enabler
(`VIBEOS_NO_KVM=1` TCG path for amd64 start.sh) landed with the coverage
tooling.
