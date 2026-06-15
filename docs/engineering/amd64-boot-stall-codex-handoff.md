# amd64 LiveCD boot-stall — codex handoff

**Worktree:** `/home/curi/Projects/os-playground/vibeos-wt-cdrom` (branch `cdrom-port`).
Run everything here. **Date:** 2026-06-14. **Status:** OPEN, well-localized, two prior
root-causes disproven.

## The task (one sentence)

amd64 LiveCD boots but **periodically stalls ~6.3s with the CPU idle**, ~1 read in
every 30–60 throughout boot, never reaching the `[EXPLORER] _tWinMain entered` boot
marker → smoke `HANG_TIMEOUT`. **i386 and arm64 boot green on the identical Rust.**
Find why an occasional storage completion arrives ~6.3s late on amd64 and fix it.

## Reproduce + read the verdict

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-cdrom
./vibeos-cli build --arch amd64            # NOTE the kernel-relink gotcha below before trusting this
./vibeos-cli smoke --arch amd64            # FAIL: HANG_TIMEOUT
# stall signature in the serial log (committed cdrom trace CdromGapLog):
LOG=$(ls -t /tmp/vibeos-cli/*-smoke-amd64/full.log | head -1)
grep -ac "\[GAP\]" "$LOG"                   # ~37 gaps, each ~6.3s
grep -a "\[GAP\]" "$LOG" | head            # [GAP] read#40 gap=6231319 us mcnPolls=0
```

`[GAP]` is emitted by `CdromGapLog()` (`drivers/storage/class/cdrom/rust_dbgprint_shim.c:1292`),
called from cdrom's `SequentialQueueEvtIoReadWrite` — it measures the wall time **between
read-IRP arrivals** at cdrom's sequential queue. A 6.3s gap = cdrom sat idle 6.3s waiting
for the upper layer to hand it the next read.

## What is RULED OUT (do not re-investigate — each was disproven with live instrumentation)

1. **`MmMakeSegmentResident` collided-fault / `IoPageRead`** (`ntoskrnl-rust/src/mm/section.rs:911`/`:1003`).
   A prior session "root-caused" this — it was **wrong**, diagnosed against a stale kernel.
   Re-tested with in-place timing probes (`[MMSR-WAIT]`/`[MMSR-READ]`, >200ms threshold):
   **fired 0× while 37 gaps occurred.**
2. **scsiport SRB-timeout / bus-reset** (`scsiport-rust/src/scsiport.rs:2116`, the
   `RequestTimeout==0` path). Instrumented with `[SCSI-TIMEOUT]`: **fired 0× vs 37 gaps.**
   So the requests are NOT timing out; they complete, just late.
3. **scsiport LUNEX flag bug** (real, fixed in `78d6ec6` — wrong tick-handler flag bits) —
   smoke unchanged after the fix. Not this.
4. **cdrom / the storage drivers themselves.** cdrom only *observes* the gap; the underlying
   reads round-trip fast. cdrom boots i386+arm64 green.
5. **Wrong inline constants.** `tools/scan_inline_flag_literals.py` swept scsiport / uniata /
   atapi / HAL / ntoskrnl-rust: nothing in the interrupt path. (It found unrelated real bugs
   elsewhere, since fixed.)

## What is ESTABLISHED (the diagnosis, via DWARF GDB thread-walk during a gap)

Built a `debug=true` DWARF kernel, `qemu/start.sh amd64 --gdb`, halted during the gappy
phase, walked every thread:

- **All 15 System (Pid 4) threads are idle infrastructure** — 9× `ExpWorkerThreadEntryPoint`
  (WrQueue, idle pool), `ExpWorkerThreadBalanceManager`, `MiBalancerThread`, 2×
  `FsRtlWorkerThread`, + scheduler/`NtDelayExecution` waiters. The on-CPU "Running" thread is
  `KiIdleLoop`. **No thread is in any Io / Cc / Cdfs / Mm / read path.**
- **PC sampling during a stall:** ~84% `sti; hlt` (CPU genuinely halted/idle), ~10% a storage
  miniport PIO `in (%dx),%eax` loop.
- ⇒ During the 6.3s the box is **genuinely idle, waiting for a coarse timer to advance** — it
  is NOT spinning and NOT blocked in an in-flight read. Yet reads complete (count climbs) and
  there is **no SRB timeout/reset**.

## Leading hypothesis (where codex should start)

**The storage completion interrupt is delayed/lost on amd64.** A read SRB is issued, the
thread blocks on completion, the CPU idles — and the completion (DPC that signals the
waiter) only arrives ~6.3s later. i386 (8259 PIC) delivers the IDE IRQ promptly → **0 gaps on
the same Rust**; amd64 uses the APIC/IOAPIC. Same QEMU (`-M pc`) + same emulated IDE CD for
both arches, so the delta is **guest-side interrupt handling**, not the device. The IRQ isn't
totally absent (reads do finish, below the reset timeout), so think *lost-and-recovered* or
*delayed* delivery — e.g. edge-vs-level mismatch, a missed EOI leaving the line masked, or a
wrong vector/affinity from `IoConnectInterrupt` on amd64.

### Concrete next steps

1. **Confirm it's interrupt-driven-or-not.** Instrument the scsiport ISR
   (`ScsiPortInterrupt` — the port ISR — and/or the miniport `HwInterrupt`) with a global call
   counter, and at each read completion (`scsiport-rust/src/scsi.rs:566`,
   `ScsiPortNotification(RequestComplete)`) log the counter delta since the previous
   completion. **If the ISR count is flat across a 6.3s completion, the completion did NOT come
   from an interrupt** → the IRQ was lost and something else (a timer/poll) eventually advanced
   it. That nails the hypothesis.
2. If confirmed, chase the interrupt *connection*: compare i386 vs amd64
   `IoConnectInterrupt` parameters in scsiport (`scsiport-rust/src/scsiport.rs` — search
   `Interrupt`, `KINTERRUPT`, `InterruptMode`, `ShareVector`, `Affinity`), the
   `InterruptData.Flags & SCSI_PORT_DISABLE_INTERRUPTS` gating, and HAL EOI / IOAPIC routing.
3. Cross-check the IRQ actually fires: read the per-vector interrupt count (KPRCB) or watch the
   IOAPIC redirection entry for the IDE vector while a read is pending.
4. Cross-lane note: the arm64 fastfat lane reports a *related* Mm/Cc **section page-in** issue
   (`CcMapData → MmGetPageEntrySectionSegment`, a *busy* livelock — different mode, same Mm/Cc
   family). Probably independent of this interrupt issue, but worth keeping in view.

## ⚠️ Build gotcha that WILL bite you (field-manual §9.6b)

The cargo-driven amd64 kernel **does NOT relink on a `.rs` edit.** `ntoskrnl.exe` is a ninja
`CUSTOM_COMMAND` whose dep graph doesn't list the `.rs` sources, so `ninja livecd` /
`vibeos-cli build --arch amd64` (**even `--clean`**) silently ship a **stale kernel**. A prior
session wasted cycles (and reached a wrong root cause) on a 5-hour-stale kernel.

```bash
# Force a real kernel relink after editing ntoskrnl-rust/*.rs:
rm -f build-amd64/ntoskrnl/{ntoskrnl.exe,libkernel_rust.a,ntkrnlmp/ntkrnlmp.exe}
ninja -C build-amd64 livecd
strings build-amd64/ntoskrnl/ntoskrnl.exe | grep '<your-marker>'   # VERIFY before smoking
```

**Drivers are fine** — `scsiport-rust`, `cdrom-rust` etc. link via the main-workspace cargo
step which *does* rerun on `.rs` edits (verify by `stat -c %y .../scsiport.sys` after build).
Smoke built out-of-band: `./vibeos-cli smoke --arch amd64 --allow-stale`.

## GDB toolkit (now works on amd64 — `find-kbase` PE32+ fix is committed, `e429024`)

```bash
# For thread-walking you need DWARF: set [profile.release] debug=true in
# sdk/lib/rust/Cargo.toml (per-worktree; revert before commit), then force-relink the kernel.
RUN=$(qemu/start.sh amd64 --gdb)                      # preflight passes with a debug kernel; else add --skip-preflight
qemu/wait-for.sh $RUN --pattern "GAP" --timeout 150   # get into the gappy phase
qemu/inspect.sh $RUN --command list-threads           # walk PsActiveProcessHead (needs DWARF)
qemu/inspect.sh $RUN --command sample-pc --samples 50 --delay-ms 150
qemu/inspect.sh $RUN --command dump-thread <ETHREAD>  # KTHREAD fields incl KernelStack(+0x38)
qemu/stop.sh $RUN
```

To backtrace a *blocked* OS thread (gdbstub only exposes vCPUs): read its `KernelStack`
(ETHREAD+0x38), `set $sp` there, and `info symbol` each qword in the kernel range — the
return addresses give the call chain. (KVM does NOT fire gdb breakpoints; use halt+inspect
sampling, or `VIBEOS_NO_KVM=1` for TCG if you need `hbreak`/`watch`.)

## Parked diagnostics (reapply to re-instrument)

```bash
git stash list | grep "amd64-stall diag"   # MMSR timing (section.rs) + SCSI-TIMEOUT (scsiport.rs)
```
These are the >200ms timing probes used to rule out Mm and scsiport-timeout. Pop/cherry-pick
the scsiport portion as a template for the ISR-counter instrumentation in step 1.

## Pointers
- Full evidence trail: `docs/engineering/cdrom-scsi-send-handoff.md` (see the **RETRACTION
  2026-06-14(b)** and **FOLLOW-UP 2026-06-14(c)** sections).
- Relevant commits on `cdrom-port`: `e429024` (find-kbase PE32+ / start.sh scratch disk),
  `0f1d7ac` (field-manual §9.6b + retraction), `539d65d` (thread-walk follow-up),
  `78d6ec6` (scsiport LUNEX), `37e308e` (cdrom opcode fixes), `ddfc06d` (crp merge).
- Boot marker the smoke waits for: `[EXPLORER] _tWinMain entered` (`tests/qemu/vm.py`).
