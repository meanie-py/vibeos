# arm64 interrupts — make the interrupt subsystem real (routing + dispatch + hardware IRQL)

**Status:** quest brief, opened 2026-07-05. Foundational — this is the root that forces
polling across arm64 and blocks every interrupt-driven driver (real NIC, interrupt-driven
USB, disk).

**Mandate:** get arm64 device interrupts genuinely working and **production-ready** — not
a beachhead demo. Real `HalGetInterruptVector` + GIC routing + a general interrupt dispatch
+ **hardware-backed IRQL**, then convert the polling drivers to interrupts. Build whatever
**IRQL/interrupt debugging tooling** you need to get to the bottom of crashes along the way
(see §Tooling). You have full autonomy to fix any bug you find in the HAL / kernel trap /
driver path en route.

---

## The current state (traced 2026-07-05 — verify, then extend)

Three facts define the gap. The good news is that **one real interrupt already works
end-to-end**, so this is a *generalization* with a working template, not from-scratch.

1. **IRQL is a pure-software counter — it never touches hardware.**
   `KfRaiseIrql` (`sdk/lib/rust/hal-rust/src/arm64/halstubs.rs:738`) just sets a global
   `HalpCurrentIrql` and returns the old value. No `GICC_PMR` / `ICC_PMR_EL1` programming.
   So raising IRQL tracks a *number* but does not actually mask any device interrupt. There
   is a `HalpSoftwareInterruptPending` / `HalpDispatchSoftwareInterrupts` path for DPC/APC
   delivery on lower, but nothing gates hardware IRQs.

2. **The timer interrupt works fully — this is your template.**
   `hal-rust/src/arm64/clock.rs` programs GICv2 for real: distributor (`GICD_CTLR`,
   `GICD_ISENABLER`, `GICD_IPRIORITYR`, `GICD_ITARGETSR`), CPU interface (`GICC_CTLR`,
   `GICC_PMR`, `GICC_IAR`, `GICC_EOIR`), and dispatches the EL1 virtual timer (INTID/PPI 27)
   through the read-IAR → handle → write-EOIR cycle. The hard mechanics (GIC MMIO mapping via
   `MmMapIoSpace`, IAR/EOI handshake, priority) are all proven here — but hardwired to the
   timer.

3. **Device interrupt routing is a stub.**
   `HalGetInterruptVector` (`halstubs.rs:404`) returns `0`. `HalBeginSystemInterrupt`
   (`halstubs.rs:769`) just raises the soft IRQL and returns `TRUE`. So a driver's
   `IoConnectInterrupt` can never obtain a usable vector, and there is no general path from
   "some device SPI fired" to "call the ISR that registered for it."

**Consequence:** every arm64 device polls — `arm64input` runs a `VirtioPollThread`
(`sdk/lib/rust/arm64input-rust/src/arm64input.rs:1288`), the netstack polls on a `delay_ms`
cadence (`netstack-rust/src/engine.rs`). Polling is a VM/UP-only crutch: it burns a CPU and
cannot meet real device rates.

**x86 is the reference for "done."** i386/amd64 have real PIC/APIC vectoring
(`hal-rust/src/x86/{busemul.rs:245 HalGetInterruptVector, pic_pic.rs, apic.rs}`), and there
is already an **interrupt-driven driver on amd64** to model the driver side after: the
netstack's virtio RX ISR (`netstack-rust/src/isr.rs`, M10 — real `IoConnectInterrupt` +
`HalGetInterruptVector`). Read both before starting arm64.

---

## The two coupled halves (doing one without the other is a latent bug)

This is the key insight: **you cannot ship device interrupts without also making IRQL
hardware-real.** They are two halves of one job.

### Half A — routing + dispatch
- **`HalGetInterruptVector`**: map a device's GIC INTID (an SPI, INTID ≥ 32) → a vector +
  the `KIRQL` it should run at + affinity. (On ARM the "vector" abstraction is largely the
  INTID itself; define the mapping this port will use.)
- **GIC distributor programming for SPIs**: enable + set priority + route (target CPU) for
  an arbitrary device INTID — generalize what `clock.rs` does for INTID 27 (a PPI) to SPIs.
  This is the body `HalEnableSystemInterrupt` should have.
- **General dispatch**: on an IRQ exception, read `GICC_IAR`, **demux the INTID to the ISR
  that registered for it**, call it at its DIRQL, then `GICC_EOIR`. Today the trap path
  assumes the timer INTID — it needs a registration table (INTID → ISR/context) that
  `IoConnectInterrupt` populates. Find the arm64 IRQ trap entry (in `ntoskrnl-rust` ke/arm64
  trap code) and route it through this.
- **`IoConnectInterrupt` plumbing**: the IO-manager side already exists (x86 uses it);
  confirm it flows to `HalEnableSystemInterrupt` + stores the `KINTERRUPT` on arm64.

### Half B — hardware-backed IRQL
- Make `KfRaiseIrql`/`KfLowerIrql` program the GIC priority mask (`GICC_PMR` on GICv2, or
  `ICC_PMR_EL1` if you move to the system-register interface) so that **raising IRQL
  actually masks device interrupts at or below that level**, and lowering re-enables +
  drains any now-deliverable pending interrupt.
- **The bug this prevents (write a tripwire for it):** a driver holding a spinlock at
  `DISPATCH_LEVEL` gets preempted by *its own* device ISR because the soft IRQL didn't mask
  the GIC → re-entrancy / deadlock / corruption. On UP with only the timer this never
  happens; add a second interrupt source and it becomes real. Preserve the soft
  DPC/APC-dispatch behavior on lower — hardware masking is for *device* (DIRQL) levels.

Scope: **UP-first** (single vCPU, as the smoke runs today). SMP/IPI affinity is a later
concern — don't block prod-readiness on it, but don't design it out.

---

## Beachhead (prove the pattern on ONE real device)

Pick the simplest real device interrupt and wire it fully through Halves A+B, using
`clock.rs` as the template:
- **PL011 UART RX** (`sdk/lib/rust/pl011-rust`) — SPI 33 on QEMU `virt`; a keypress on the
  serial console fires it. Simple, deterministic, no virtqueue.
- or **virtio-input's IRQ** — lets you *delete `arm64input`'s poll thread* the moment it
  works, which is the first real payoff.

Success = the device's ISR fires from a real GIC interrupt (not a poll), EOIs cleanly, runs
at the right IRQL, and IRQL actually masks it when raised. Once one device works, the
pattern generalizes to all of them.

## Production-ready (the actual finish line)
1. Beachhead device on real interrupts.
2. **Convert `arm64input` from `VirtioPollThread` to ISR+DPC** (mirror `i8042prt`'s
   ISR→DPC→class-callback shape) — kills the biggest poll and de-risks the input-unification
   quest.
3. IRQL hardware-backed and validated (the spinlock-vs-ISR tripwire stays green).
4. The path is ready for the real-NIC and interrupt-driven-USB quests to build on.
5. Gate ×3 green (all arches — you're touching shared HAL/trap code; x86 must not regress).

**Downstream cleanup this unblocks:** the scheduler carries an arm64-only ReadySummary
preemption recheck in `ke/wait.rs` (KiExitDispatcher) and `ke/thrdschd.rs` (KiSwapThread,
NtYieldExecution), mirroring C `#ifdef _M_ARM64` (ntoskrnl/ke/thrdschd.c:444). It exists
*specifically* because arm64 has no working `HalRequestSoftwareInterrupt`, so a stale
`Prcb->NextThread` is never re-dispatched via a DISPATCH software interrupt and a
higher-priority ready thread can starve. **Once Half B delivers real software-interrupt
preemption, verify this recheck is dead (instrument that the branch never fires on a boot)
and remove all three sites** — the invariant will then be kept the x86 way. Cross-ref:
`per-arch-cfg-divergence-audit.md` Tier 4.

---

## Tooling you should build (explicitly in scope)

Interrupt/IRQL bugs are miserable to debug blind. Build whatever you need — bias toward
*self-reporting on serial* over interactive GDB (the existing tripwire / sync-instrumentation
work is the model). Likely useful:
- **IRQL assertion/trace** — catch IRQL-rule violations: raise/lower imbalance, returning at
  the wrong IRQL, touching a DISPATCH-level lock at PASSIVE, an ISR running below DIRQL. A
  BUGCHECK-at-the-violation with the site symbolicated (like the special-pool tripwire) beats
  post-mortem.
- **GIC state dumper** — distributor + CPU-interface registers, and the enabled/pending/active
  INTID sets + current `PMR`, dumpable on demand (a GDB helper and/or a serial command).
- **Interrupt-delivery trace** — which INTID fired, which ISR was dispatched, the IAR/EOI
  pair, IRQL on entry/exit — so lost / spurious / storming interrupts are visible.
- Reuse what exists: `qemu/start.sh`+`run-until` GDB toolkit, `vibeos-cli gdb`, the fw_cfg
  **tripwire** channel (armable per-boot, no rebuild — `ntoskrnl-rust/src/tripwire.rs`), and
  the sync-instrumentation/lockdep serial self-reporting pattern
  (`docs/engineering/sync-instrumentation-design.md`).

Leave this tooling in the tree — it's the durable payoff and future interrupt work will need it.

---

## Guardrails & autonomy
- **Autonomy:** fix any real bug you find in the HAL / arm64 trap path / affected drivers —
  don't hand it off. This is deep low-level work; expect to touch `ke/arm64` trap code, the
  HAL, and `ntkrnl-sys` layout.
- Smoke tests are the north star — never weaken them. **Gate ×3** (`./vibeos-cli gate`)
  before landing; x86 must stay green (shared HAL/IRQL code).
- Never revert a Rust translation to C.
- **Do NOT kick off the flake farm** (`vibeos-cli farm`) while others may be working — it
  eats the whole host CPU. Save soaks for a confirmed-idle host.

## References
- IRQL + interrupt stubs: `sdk/lib/rust/hal-rust/src/arm64/halstubs.rs` (`KfRaiseIrql`:738,
  `HalGetInterruptVector`:404, `HalBeginSystemInterrupt`:769).
- Working GIC template: `sdk/lib/rust/hal-rust/src/arm64/clock.rs`.
- x86 reference (real vectoring): `hal-rust/src/x86/{busemul.rs:245,pic_pic.rs,apic.rs}`;
  interrupt-driven driver: `netstack-rust/src/isr.rs`.
- Poll drivers to convert: `arm64input-rust/src/arm64input.rs:1288`, `netstack-rust/src/engine.rs`.
- Downstream quests this unblocks: `input-architecture-and-unification.md`,
  `real-nic-networking-quest.md`, `usb-mass-storage-real-storage-quest.md`; roadmap §10c.
