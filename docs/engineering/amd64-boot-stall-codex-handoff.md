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

---

## UPDATE 2026-06-15 — root chain pinned; the handoff hypothesis was RIGHT (storage interrupt)

The original "amd64 interrupt delivery" hypothesis was correct in spirit. Full chain now nailed:

### Layer 1 — the clock (SOLVED, fix parked)
- **Pinned via QMP `info irq`:** QEMU's emulated mc146818 RTC intermittently **stops raising IRQ8** for ~2 s (IRQ8 cumulative count freezes; guest tick gap lands exactly on the freeze). amd64 clocks the system off this RTC (`rtctimer.rs`); i386 off the free-running 8254 PIT → the arch split. Not KVM delivery, not spurious, not `lost_tick_policy=slew`.
- **Fix (WORKS, proven clean 64 Hz, zero gaps):** rewire the amd64 system clock from the RTC to the **local APIC timer** (calibrated against the TSC — APIC bus ≈992 MHz ≠ TSC 3.79 GHz, so calibration matters). Also had to **neutralize the profile-interrupt controls** (`apictimer.rs` `HalStartProfileInterrupt`/`HalStopProfileInterrupt`/`HalSetProfileInterval`) — they share the single per-CPU LAPIC timer and were masking the clock LVT (`HalStopProfileInterrupt` runs right after `HalpInitializeClock` in `halinit.rs`).
- **Parked** in `git stash` on cdrom-port: message `amd64-lapic-clock-WIP-...` (rtctimer.rs + apictimer.rs, includes CLKINIT/CLKRATE probes). NOT committed because of Layer 2 below — `cdrom-port` stays green on codex's band-aid (`97d678e`, mm/balance NtYieldExecution).

### Layer 2 — the REAL nemesis: amd64 IDE completion-interrupt delivery (OPEN)
With a clean clock, boot advances through the whole kernel + most of userland (SMSS→CSRSS→WINLOGON→services) then **livelocks** at `rip=fffff80004603e91` (in **uniata.sys**), `TPR=0xa0` (IRQL 10), `RDX=0x170` (secondary IDE command-block port — the boot CD is `Ata(Secondary,Master)`), with **`IRR` vec `0x47` (the IDE completion IRQ) pending-but-blocked** by the raised IRQL. I.e. **uniata does an ATA PIO status-poll at IRQL 10 waiting for an IDE completion interrupt that its own IRQL masks → UP poll-at-DIRQL deadlock.**
- This is the SAME storage-interrupt problem from the start: gappy RTC clock → reads complete ~6.3 s late (the original stall); clean LAPIC clock → hard poll deadlock. The clock was a real bug; **storage IDE IRQ delivery on amd64 APIC is the true blocker.**
- **Note the IRQL mismatch:** uniata spins at IRQL 10 but the IDE IRQ vector 0x47 maps to IRQL 4 — worth checking uniata's SynchronizeIrql / `IoConnectInterrupt` params and the HAL vector→IRQL mapping on amd64.
- **Next:** disassemble `fffff80004603e91` (QMP `x/i` unsupported on x86_64 in this build — use gdb or QMP `memsave`+`llvm-objdump`); chase why the IDE completion interrupt (vec 0x47) isn't delivered/serviced, and why uniata enters a poll-at-DIRQL instead of waiting interrupt-driven. Tie back to IOAPIC IRQ14/15→vector routing + EOI on amd64.

---

## UPDATE 2 2026-06-15 (CORRECTION) — Layer 2 is NOT a deadlock; it's PIO-not-DMA

Disassembled `fffff80004603e91` (gdb): it's the `AtapiReadBuffer2` PIO `insw` loop
(`in (%dx),%ax; mov %ax,(buf); inc; cmp; jne`). Then instrumented `interrupt_read_drq`
(uniata `id_ata_part2.rs`, COM1 port-0x3F8 emit — DbgPrint deadlocks at DIRQL, don't use it
in the ISR). Every read-DRQ shows **`d=0` (DmaTransfer=0), `ctl=0` (no CTRFLAGS_DMA_OPERATION),
`a=1` (ATAPI CD), `wc==left`** (e.g. `wc=0x6000`=48 KB) while `wti=0x100` (one sector). So:

- **The ATAPI boot CD runs in PIO, never DMA.** Each transfer is `insw` of the *whole*
  remaining request (device reports a big ATAPI byte count), one `in` per word = one KVM
  VM-exit. 48 KB ≈ 24 K exits ⇒ pathologically slow. It is **NOT a deadlock** — `wc` varies
  read-to-read, boot *progresses* (reached `shell ready` → `[WINLOGON] GinaInit`), just far
  too slowly to finish inside the smoke window. (The old "IDE-IRQ poll deadlock" framing in
  UPDATE 1 was wrong — the high TPR/IRR snapshot was a transient inside the hot `insw`.)
- This **unifies the whole saga**: the CD was always slow PIO. With the gappy RTC clock it
  looked like "6.3 s read stalls" and *barely* completed (the codex band-aid is green). With
  the clean LAPIC clock the timing shifts and it overruns the timeout. **The clock was a real
  bug; slow PIO is the real perf blocker.**

### Why PIO and not DMA (pinpointed to the gate, not yet the culprit)
`uniata id_dma.rs:AtapiDmaInit` forces `TransferMode=ATA_PIO` when
`deviceExtension.BusMaster==0`; `id_init.rs:508` sets `MaxTransferMode=PIO4` when
`BaseIoAddressBM==0`. `BusMaster` is set to `DMA_MODE_BM` in `id_probe.rs:1272-1289` **only if**
`IsBusMaster(&pciData)` is true — and `IsBusMaster` = `(pci.Command & PCI_ENABLE_BUS_MASTER(0x4)) != 0`.
So **the PIIX3 IDE's PCI command-register bus-master-enable bit (0x4) is not set on amd64.**
That bit is enabled during PCI init — **strong suspicion: the (recently Rust-translated) `pci-rust`
bus driver isn't enabling bus-master on amd64** (and since i386 boots fast, i386 likely *does* get
DMA → this is the amd64-specific storage bug, mirroring the amd64-specific clock bug).

### To resume (probes parked in `git stash`)
- `stash@{1}` = the proven LAPIC-clock fix (hal-rust). `stash@{0}` = uniata DMA probes
  (`[RDRQ]` in `id_ata_part2.rs` + `[BMDBG]` in `id_probe.rs`; both COM1/DbgPrint, PASSIVE-safe
  in id_probe). The `[BMDBG]` line logs `pciCmd / isbm / bm / bar4 / maxmode` at the gate.
- **Next step:** boot with `[BMDBG]` (toolkit `qemu/start.sh amd64 --skip-preflight`, read
  `qemu/runs/<id>/serial.log`) → confirm `pciCmd` bit 0x4 is clear. Then find where bus-master
  *should* be enabled (pci-rust FDO start / `IoConnectInterrupt` resource translation /
  `UniataEnableIoPCI`) and why amd64 skips it. Compare to i386 (boot i386 with `[RDRQ]` → expect
  `d=1`). Fixing bus-master → CD DMA → fast reads → the LAPIC clock fix can land and amd64 boots
  comfortably on both clocks.
- **Tooling notes:** this session's sandbox became unreliable for VM launches (background
  `/tmp` invisible to foreground calls; `pkill -f vibeos-cli` kills the agent's own shell;
  `| tail`/`| head` SIGPIPE → exit 1; some `qemu/` + `/tmp` writes didn't persist between Bash
  calls). Prefer the toolkit (`qemu/runs/` persists), capture start output to a file read in a
  *separate* call, and avoid `pkill -f vibeos-cli`.
