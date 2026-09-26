//! ARM64 periodic clock interrupt — GICv2 + ARM generic (virtual) timer.
//!
//! Wires the EL1 **virtual** timer (INTID/PPI 27 on the QEMU `virt` GICv2) as
//! the NT periodic system clock so `KeUpdateSystemTime` is invoked ~every 10ms
//! and `KUSER_SHARED_DATA.InterruptTime` / `KeTickCount` advance.  Without this
//! the arm64 system clock is frozen at 0 and every relative timer,
//! `KeDelayExecutionThread`, DPC-timer and quantum-end is broken (they all key
//! off InterruptTime / KeTickCount, which never move).
//!
//! The kernel tick handler (`ntoskrnl-rust ke/time.rs::KeUpdateSystemTime`) is
//! already correct — this module only makes it *get called*.
//!
//! Layering:
//!   * ntoskrnl maps the GIC MMIO (`MmMapIoSpace`, in `ntoskrnl/ex/init.c`,
//!     after `MmInitSystem(1)` and before the first process switch) and passes
//!     the kernel VAs into [`HalInitializeClockArm64`].  The early-boot identity
//!     map (PXE[0]) is gone after SMSS, and the ISR runs in arbitrary process
//!     context, so the GIC must live at a real kernel VA — same reasoning as the
//!     `Arm64UartKernelBase` remap.
//!   * The single arm64 IRQ vector (`KiIrqHandler`, `ntoskrnl/ke/arm64/trap.c`)
//!     calls [`HalClockInterruptArm64`] on every IRQ.
//!
//! Timer-expiration follow-through (waking a delayed thread) is handled by the
//! existing arm64 `KiIdleLoop`, which already polls `Prcb->TimerRequest` and
//! calls `KiRetireDpcList` — so once InterruptTime advances and
//! `KiCheckForTimerExpiration` sets `TimerRequest`, the idle loop retires it.
//! That is why this module does **not** lower IRQL / dispatch DPCs itself.

#![cfg(target_arch = "aarch64")]

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use ntkrnl_sys::ddk;

// ---------------------------------------------------------------------------
// GICv2 register offsets (QEMU `virt`: GICD @ 0x08000000, GICC @ 0x08010000).
// ---------------------------------------------------------------------------

/// Distributor control register (bit0 = EnableGrp0).
const GICD_CTLR: usize = 0x000;
/// Set-enable, 1 bit per interrupt, 32 per 32-bit register.
const GICD_ISENABLER: usize = 0x100;
/// Clear-enable, 1 bit per interrupt, 32 per 32-bit register.
const GICD_ICENABLER: usize = 0x180;
/// Set-pending, 1 bit per interrupt (read = pending set).
const GICD_ISPENDR: usize = 0x200;
/// Set-active, 1 bit per interrupt (read = active set).
const GICD_ISACTIVER: usize = 0x300;
/// Priority, 1 byte per interrupt.
const GICD_IPRIORITYR: usize = 0x400;
/// Interrupt processor targets, 1 byte per interrupt (SPIs only; SGIs/PPIs RO).
const GICD_ITARGETSR: usize = 0x800;
/// Software-generated interrupt register (write-only; the SGI/IPI doorbell).
const GICD_SGIR: usize = 0xF00;

/// CPU-interface control register (bit0 = Enable).
const GICC_CTLR: usize = 0x000;
/// Interrupt priority mask (interrupts with priority >= PMR are masked).
const GICC_PMR: usize = 0x004;
/// Interrupt acknowledge (read returns the pending INTID).
const GICC_IAR: usize = 0x00C;
/// End-of-interrupt (write the acknowledged IAR value back).
const GICC_EOIR: usize = 0x010;
/// Running priority (priority of the highest active interrupt, 0xFF = idle).
const GICC_RPR: usize = 0x014;

// ---------------------------------------------------------------------------
// IRQL ↔ GIC priority map — the contract that makes IRQL hardware-real.
//
// GICv2 priorities: lower value = higher priority; an interrupt is delivered
// iff its priority is numerically BELOW GICC_PMR. We carve the 8-bit priority
// space in 0x10 steps (QEMU's GICv2 implements ≥4 priority bits, so 16 levels
// are architecturally safe):
//
//     priority(irql) = (15 - irql) << 4
//
// IRQL layout on arm64 (NT-style, 4-bit IRQL space):
//     0  PASSIVE / 1 APC / 2 DISPATCH   — software levels, no GIC line lives
//                                         here; PMR stays open (0xF0..0xD0)
//     3..=11  DIRQL                      — device SPIs (HalGetInterruptVector
//                                         hands out DEVICE_IRQL=8, prio 0x70)
//     13 CLOCK_LEVEL                     — vtimer PPI 27, prio 0x20 (a DIRQL
//                                         raise must NOT mask the clock)
//     15 HIGH_LEVEL                      — PMR 0x00, everything masked
//
// Raising IRQL to X programs PMR = priority(X), so every line whose IRQL ≤ X
// is masked by the GIC itself — a DISPATCH/DIRQL spinlock can no longer be
// broken by its own device's ISR. Transitions entirely inside the software
// band (old,new ≤ DISPATCH) skip the MMIO write: no line has a priority in
// [0xD0,0xF0), so the mask set is unchanged and the hot spinlock raise/lower
// path stays register-only.
// ---------------------------------------------------------------------------

pub(crate) const CLOCK_LEVEL: u8 = 13;
/// NT IPI_LEVEL on arm64 (ntkrnl-sys constants.rs): SGI priority 0x10 — above
/// CLOCK_LEVEL(13)/0x20, below only HIGH_LEVEL(15)/0x00.
const IPI_LEVEL: u8 = 14;
/// The DIRQL this port assigns to every device SPI (see HalGetInterruptVector).
pub(crate) const DEVICE_IRQL: u8 = 8;
const SOFT_IRQL_MAX: u8 = 2; // DISPATCH_LEVEL — top of the no-MMIO band

/// GIC priority byte for a line owned by `irql` (also the PMR value that masks
/// it: delivered iff prio < PMR, so PMR = priority(irql) masks IRQL ≤ irql).
#[inline(always)]
pub(crate) fn gic_priority(irql: u8) -> u8 {
    (15u8.saturating_sub(irql.min(15))) << 4
}

/// Program GICC_PMR for a software IRQL transition. Skips the MMIO write when
/// both levels are in the software band (no maskable line lives there) or the
/// GIC isn't mapped yet (early boot: IRQs can't fire before clock init anyway).
#[inline(always)]
pub(crate) unsafe fn update_pmr(old_irql: u8, new_irql: u8) {
    if old_irql <= SOFT_IRQL_MAX && new_irql <= SOFT_IRQL_MAX {
        return;
    }
    let gicc = GICC_BASE.load(Ordering::Relaxed);
    if gicc != 0 {
        mmio_write(gicc, GICC_PMR, gic_priority(new_irql) as u32);
    }
}

/// EL1 virtual-timer PPI on the GIC (architectural INTID for CNTV).
const VTIMER_INTID: u32 = 27;

/// The SGI INTID carrying the NT IPI (HalRequestIpi → KiIpiServiceRoutine).
const SGI_IPI_INTID: u32 = 0;
/// SGIs occupy INTIDs 0..16.
const SGI_INTID_MAX: u32 = 16;

/// IAR values >= 1020 are special (1023 = spurious); no EOI is required.
const GIC_SPURIOUS_MIN: u32 = 1020;

/// GICv2 physical bases, published by ntoskrnl from the loader block (which got
/// them from the ACPI MADT). Used AS virtual addresses through the boot identity
/// map during the early-SMP window — KeStartAllProcessors (and its
/// KeIpiGenericCall self-test) runs BEFORE MmInitSystem(1), so MmMapIoSpace is
/// not available when the APs come up. The loader guarantees these pages are
/// identity-mapped with Device attributes (winldr_arm64.rs
/// WinLdrMapSpecialPages); HalInitializeClockArm64 later swaps both bases to
/// the proper MmMapIoSpace kernel VAs (the identity map dies at SMSS).
///
/// These were fixed QEMU virt constants until 2026-09-20. They must not be:
/// this board's GIC is at 0xFF841000/0xFF842000, and 0x08000000 there is plain
/// DRAM, so programming it would corrupt memory rather than fail. They arrive
/// through the loader block rather than an ntoskrnl export, because the caller
/// (HalInitializeProcessor) is already handed it.

// ---------------------------------------------------------------------------
// Clock rate.
// ---------------------------------------------------------------------------

/// 100 Hz tick = 10ms period.
const TICK_HZ: u64 = 100;
/// 10ms expressed in 100ns units — matches `KeSetTimeIncrement(100000, 10000)`
/// in `HalInitSystem` and the kernel's `KeMaximumIncrement`, so each timer
/// interrupt advances InterruptTime by one full NT tick.
const CLOCK_INCREMENT_100NS: u32 = 100_000;

// ---------------------------------------------------------------------------
// State (set once at init, read in the ISR).
// ---------------------------------------------------------------------------

static GICD_BASE: AtomicUsize = AtomicUsize::new(0);
static GICC_BASE: AtomicUsize = AtomicUsize::new(0);
/// Virtual-timer reload value in counter ticks (CNTFRQ_EL0 / TICK_HZ).
static TIMER_INTERVAL: AtomicU64 = AtomicU64::new(0);

extern "C" {
    /// ntoskrnl export.  On aarch64 the `.spec` `fastcall` decays to the
    /// standard AAPCS64 ABI (== `extern "C"`) with the undecorated symbol
    /// `KeUpdateSystemTime`; matches the `extern "C"` definition in
    /// `ntoskrnl-rust ke/time.rs`.
    fn KeUpdateSystemTime(TrapFrame: *mut core::ffi::c_void, Increment: u32, Irql: u8);
    /// ntoskrnl export (ke/time.rs). AP clock ticks call this INSTEAD of
    /// KeUpdateSystemTime: per-thread user/kernel time, per-PRCB DPC-rate
    /// bookkeeping, and quantum decrement/quantum-end — everything per-CPU —
    /// without advancing the global InterruptTime/SystemTime/KeTickCount
    /// (exactly one CPU may own wall-clock advance: CPU0, NT rule).
    #[cfg(feature = "smp")]
    fn KeUpdateRunTime(TrapFrame: *mut core::ffi::c_void, Irql: u8);
}

extern "system" {
    /// ntoskrnl export (ke/arm64/stubs.rs): dispatch a non-timer GIC INTID to
    /// the ISR connected via IoConnectInterrupt/KeConnectInterrupt. The HAL owns
    /// the GIC (ack/EOI here); ntoskrnl owns the KINTERRUPT registry + ISR call.
    fn KiDispatchInterruptArm64(Intid: u32) -> u8;
}

// ---------------------------------------------------------------------------
// Device-interrupt GIC line control (SPIs), used by HalEnableSystemInterrupt /
// HalDisableSystemInterrupt. The clock module owns GICD_BASE, so the SPI
// enable/disable lives here next to the distributor programming it mirrors.
// ---------------------------------------------------------------------------

/// Enable a device interrupt line (SPI) in the GIC distributor at the GIC
/// priority derived from its IRQL, routed to CPU0, then set-enable. No-op
/// until the GIC is mapped (clock init).
pub(crate) unsafe fn enable_gic_interrupt(intid: u32, irql: u8) {
    let gicd = GICD_BASE.load(Ordering::SeqCst);
    if gicd == 0 || intid as usize >= 1020 {
        return;
    }
    let id = intid as usize;
    // Priority from the line's IRQL, so raising IRQL to (or above) it masks it
    // at the GIC (see the IRQL ↔ priority map above).
    core::ptr::write_volatile(
        (gicd + GICD_IPRIORITYR + id) as *mut u8,
        gic_priority(irql),
    );
    // SPIs (>=32) need an explicit CPU target; SGI/PPI target registers are RO.
    if id >= 32 {
        core::ptr::write_volatile((gicd + GICD_ITARGETSR + id) as *mut u8, 0x01u8);
    }
    mmio_write(gicd, GICD_ISENABLER + (id / 32) * 4, 1u32 << (id % 32));
}

/// Disable (clear-enable) a device interrupt line in the GIC distributor.
pub(crate) unsafe fn disable_gic_interrupt(intid: u32) {
    let gicd = GICD_BASE.load(Ordering::SeqCst);
    if gicd == 0 || intid as usize >= 1020 {
        return;
    }
    let id = intid as usize;
    mmio_write(gicd, GICD_ICENABLER + (id / 32) * 4, 1u32 << (id % 32));
}

// ---------------------------------------------------------------------------
// SMP IPI transport (smp-bringup-quest.md M5b): SGI 0 == the NT IPI.
// Send side: HalRequestIpi → gic_send_sgi → GICD_SGIR. Receive side: the
// SGI branch in HalClockInterruptArm64 below → KiIpiServiceRoutine.
// ---------------------------------------------------------------------------

extern "system" {
    /// ntoskrnl export (ke/ipi.rs): the NT IPI service routine — consumes the
    /// IPI_APC/IPI_DPC/IPI_SYNCH_REQUEST bits on the current CPU's PRCB and
    /// runs the sender's packet worker. Null frames are the sanctioned
    /// "no trap context" form (ke/ipi.rs uses it for the self-IPI shortcut).
    fn KiIpiServiceRoutine(
        TrapFrame: *mut core::ffi::c_void,
        ExceptionFrame: *mut core::ffi::c_void,
    ) -> u8;
}

/// Seed the GIC bases with the identity-mapped physical addresses if clock
/// init hasn't published real kernel VAs yet. Early-SMP only (see GICD_PA);
/// first writer wins, HalInitializeClockArm64 overwrites unconditionally.
pub(crate) unsafe fn gic_ensure_early_bases(loader_block: ddk::PLOADER_PARAMETER_BLOCK) {
    if loader_block.is_null() {
        return;
    }
    let gicd = (*loader_block).u.Arm.Arm64GicDistributorBase as usize;
    let gicc = (*loader_block).u.Arm.Arm64GicCpuInterfaceBase as usize;
    // Zero means firmware described no GICv2; seeding zero would be a no-op
    // anyway, and every consumer already treats a zero base as "not mapped".
    if gicd == 0 || gicc == 0 {
        return;
    }
    let _ = GICD_BASE.compare_exchange(0, gicd, Ordering::SeqCst, Ordering::SeqCst);
    let _ = GICC_BASE.compare_exchange(0, gicc, Ordering::SeqCst, Ordering::SeqCst);
}

/// Program THIS core's banked GICv2 state so it can receive the NT IPI:
/// SGI-IPI priority + set-enable (the GICD_IPRIORITYR/ISENABLER0 copies for
/// INTIDs 0-31 are banked per accessing core — the BSP cannot write an AP's),
/// distributor forwarding (global, idempotent), and this core's CPU interface
/// (PMR from the current soft IRQL, then enable). Called on the BSP by
/// HalInitializeClockArm64 and on each AP by HalInitializeProcessor.
pub(crate) unsafe fn gic_percpu_init() -> bool {
    let gicd = GICD_BASE.load(Ordering::SeqCst);
    let gicc = GICC_BASE.load(Ordering::SeqCst);
    if gicd == 0 || gicc == 0 {
        return false;
    }
    core::ptr::write_volatile(
        (gicd + GICD_IPRIORITYR + SGI_IPI_INTID as usize) as *mut u8,
        gic_priority(IPI_LEVEL),
    );
    mmio_write(gicd, GICD_ISENABLER, 1u32 << SGI_IPI_INTID);
    mmio_write(gicd, GICD_CTLR, 1);
    mmio_write(
        gicc,
        GICC_PMR,
        gic_priority(super::halstubs::KeGetCurrentIrql()) as u32,
    );
    mmio_write(gicc, GICC_CTLR, 1);
    // Read back what took. On a GICv2 with the Security Extensions our writes
    // are Non-secure, and every register bit of a Group 0 (Secure) interrupt
    // is RAZ/WI to them: if firmware left this core's SGIs in Group 0, the
    // enable above silently did nothing and this core can never take an IPI.
    let enabled = mmio_read(gicd, GICD_ISENABLER) & (1u32 << SGI_IPI_INTID) != 0;
    DbgPrint(
        b"[GIC] CPU%u percpu: SGI%u enabled=%u prio=%02x GICC_CTLR=%x PMR=%02x%s\n\0".as_ptr(),
        super::halstubs::halp_cpu_number(),
        SGI_IPI_INTID,
        enabled as u32,
        core::ptr::read_volatile((gicd + GICD_IPRIORITYR + SGI_IPI_INTID as usize) as *const u8) as u32,
        mmio_read(gicc, GICC_CTLR),
        mmio_read(gicc, GICC_PMR),
        if enabled { b"\0".as_ptr() } else { b" - NOT ENABLED: SGI is Group 0 (Secure)? IPIs to this CPU are dead\0".as_ptr() },
    );
    true
}

/// Send the NT IPI (SGI 0) to `targets` (bit N == GIC CPU interface N; GICv2
/// target-list filter 0, 8 CPUs max). The `dsb ish` orders the sender's
/// packet/request stores before the doorbell so the receiver cannot observe
/// a stale request block.
pub(crate) unsafe fn gic_send_sgi(targets: u64) {
    let gicd = GICD_BASE.load(Ordering::SeqCst);
    if gicd == 0 {
        // Fail loud: a dropped IPI is a silent sender hang (TargetSet never
        // clears). Cannot happen once gic_ensure_early_bases/clock init ran.
        DbgPrint(
            b"[GIC] HalRequestIpi with GIC unmapped - IPI DROPPED (targets=%llx)\n\0".as_ptr(),
            targets,
        );
        return;
    }
    let list = (targets & 0xFF) as u32;
    if list == 0 {
        return;
    }
    core::arch::asm!("dsb ish", options(nostack, preserves_flags));
    mmio_write(gicd, GICD_SGIR, (list << 16) | SGI_IPI_INTID);
}

/// M5c tail: arm THIS CPU's virtual timer — banked PPI-27 priority/enable
/// (the GICv2 IPRIORITYR/ISENABLER0 copies for INTIDs < 32 are per accessing
/// core, so this MUST run on the CPU being armed) + the first CNTV deadline.
/// APs call this lazily from their first IRQ after clock init published the
/// interval (they cannot arm in HalInitializeProcessor — that runs before
/// clock init, when the interval is still unknown); the BSP arms inline in
/// HalInitializeClockArm64 and never takes this path.
#[cfg(feature = "smp")]
unsafe fn vtimer_percpu_arm(interval: u64) {
    let gicd = GICD_BASE.load(Ordering::SeqCst);
    if gicd == 0 {
        return;
    }
    core::ptr::write_volatile(
        (gicd + GICD_IPRIORITYR + VTIMER_INTID as usize) as *mut u8,
        gic_priority(CLOCK_LEVEL),
    );
    mmio_write(
        gicd,
        GICD_ISENABLER + (VTIMER_INTID as usize / 32) * 4,
        1u32 << (VTIMER_INTID % 32),
    );
    vtimer_rearm(interval);
    // One-shot by construction (armed flag gates the caller). IRQ context;
    // DbgPrint on arm64 is the polled UART — safe here.
    DbgPrint(
        b"[SMP] CPU%u vtimer armed (banked PPI-27 + CNTV)\n\0".as_ptr(),
        super::halstubs::halp_cpu_number(),
    );
}

// ---------------------------------------------------------------------------
// Interrupt-delivery trace + GIC state dump (debugging tooling — serial
// self-reporting, no GDB needed). Lost / spurious / storming interrupts and
// mask bugs show up here instead of as silent hangs.
// ---------------------------------------------------------------------------

/// Per-INTID delivery counters for INTIDs 0..63 (covers every QEMU `virt`
/// source we route: SGIs, PPIs incl. the vtimer 27, UART SPI 33, virtio-mmio
/// SPIs 48..79 alias into the overflow bucket only above 63).
const TRACE_SLOTS: usize = 64;
static IRQ_COUNTS: [AtomicU32; TRACE_SLOTS] =
    [const { AtomicU32::new(0) }; TRACE_SLOTS];
static IRQ_COUNT_OTHER: AtomicU32 = AtomicU32::new(0);
/// Spurious IAR reads (1020..1023).
static IRQ_SPURIOUS: AtomicU32 = AtomicU32::new(0);
/// One-shot "fired but no ISR claimed it" bitmap for INTIDs 0..63 (+1 global
/// latch for the rest) so a mis-routed line reports once instead of flooding.
static UNHANDLED_SEEN: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];
static UNHANDLED_SEEN_OTHER: AtomicU32 = AtomicU32::new(0);
/// INTID armed by HalSelfTestGicSpiArm64 (0 = none): its delivery is reported
/// as a selftest success, not an unhandled-line anomaly.
static SELFTEST_INTID: AtomicU32 = AtomicU32::new(0);

extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

#[inline]
fn trace_delivery(intid: u32) {
    if (intid as usize) < TRACE_SLOTS {
        IRQ_COUNTS[intid as usize].fetch_add(1, Ordering::Relaxed);
    } else {
        IRQ_COUNT_OTHER.fetch_add(1, Ordering::Relaxed);
    }
}

/// Report (once per line per boot) a GIC INTID that fired with no connected
/// ISR. Runs in IRQ context — DbgPrint on arm64 is the polled UART, safe here.
unsafe fn report_unhandled(intid: u32) {
    let fresh = if (intid as usize) < TRACE_SLOTS {
        let bit = 1u32 << (intid % 32);
        UNHANDLED_SEEN[(intid / 32) as usize].fetch_or(bit, Ordering::Relaxed) & bit == 0
    } else {
        UNHANDLED_SEEN_OTHER.swap(1, Ordering::Relaxed) == 0
    };
    if fresh {
        DbgPrint(
            b"[GIC] unhandled INTID %u fired, no ISR claimed it (none connected, or the ISR returned FALSE) - EOIed\n\0".as_ptr(),
            intid,
        );
        // Full GIC state at first anomaly — a misrouted/storming line is
        // diagnosable from the serial log alone.
        HalDumpGicArm64();
    }
}

/// GIC delivery self-test: software-pend an SPI via GICD_ISPENDR and report
/// the line's enable/priority state. If GIC→CPU→dispatch wiring works, the
/// pended interrupt fires as soon as PSTATE.I opens (its ISR sees a quiet
/// device and returns FALSE → one `[GIC] unhandled`-style trace, harmless).
/// Distinguishes "the GIC never delivered" from "the device never asserted" —
/// the two failure modes a dead device interrupt collapses into.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSelfTestGicSpiArm64(intid: u32) {
    let gicd = GICD_BASE.load(Ordering::SeqCst);
    if gicd == 0 || !(32..1020).contains(&(intid as usize)) {
        DbgPrint(b"[GIC] selftest: GIC unmapped or INTID %u not an SPI\n\0".as_ptr(), intid);
        return;
    }
    let id = intid as usize;
    DbgPrint(
        b"[GIC] selftest INTID %u: enabled=%u prio=%02x target=%02x -> set-pending\n\0".as_ptr(),
        intid,
        (mmio_read(gicd, GICD_ISENABLER + (id / 32) * 4) >> (id % 32)) & 1,
        core::ptr::read_volatile((gicd + GICD_IPRIORITYR + id) as *const u8) as u32,
        core::ptr::read_volatile((gicd + GICD_ITARGETSR + id) as *const u8) as u32,
    );
    SELFTEST_INTID.store(intid, Ordering::SeqCst);
    mmio_write(gicd, GICD_ISPENDR + (id / 32) * 4, 1u32 << (id % 32));
}

/// Dump the GIC distributor + CPU-interface state and the delivery counters to
/// serial: enabled/pending/active sets (first 96 INTIDs — all of QEMU virt),
/// PMR/RPR, the software IRQL, and every nonzero per-INTID delivery count.
/// Callable on demand (debugger `call`, test hooks, crash paths).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalDumpGicArm64() {
    let gicd = GICD_BASE.load(Ordering::SeqCst);
    let gicc = GICC_BASE.load(Ordering::SeqCst);
    if gicd == 0 || gicc == 0 {
        DbgPrint(b"[GIC] not initialized\n\0".as_ptr());
        return;
    }
    let mpidr: u64;
    core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr, options(nomem, nostack));
    DbgPrint(
        b"[GIC] CTLR=%x PMR=%02x RPR=%02x soft-IRQL=%u cpu=%u mpidr=%x own-interface-mask=%02x\n\0".as_ptr(),
        mmio_read(gicc, GICC_CTLR),
        mmio_read(gicc, GICC_PMR),
        mmio_read(gicc, GICC_RPR),
        super::halstubs::KeGetCurrentIrql() as u32,
        super::halstubs::halp_cpu_number(),
        mpidr as u32,
        // GICD_ITARGETSR0 byte 0 is banked: it reads as the accessing core's
        // own CPU-interface bit, which is what an SGIR target list must name.
        core::ptr::read_volatile((gicd + GICD_ITARGETSR) as *const u8) as u32,
    );
    for bank in 0..3usize {
        DbgPrint(
            b"[GIC] bank%u (INTID %u..%u): enabled=%08x pending=%08x active=%08x\n\0".as_ptr(),
            bank as u32,
            (bank * 32) as u32,
            (bank * 32 + 31) as u32,
            mmio_read(gicd, GICD_ISENABLER + bank * 4),
            mmio_read(gicd, GICD_ISPENDR + bank * 4),
            mmio_read(gicd, GICD_ISACTIVER + bank * 4),
        );
    }
    for id in 0..TRACE_SLOTS {
        let n = IRQ_COUNTS[id].load(Ordering::Relaxed);
        if n != 0 {
            DbgPrint(
                b"[GIC] INTID %u: %u delivered, prio=%02x\n\0".as_ptr(),
                id as u32,
                n,
                core::ptr::read_volatile((gicd + GICD_IPRIORITYR + id) as *const u8) as u32,
            );
        }
    }
    let other = IRQ_COUNT_OTHER.load(Ordering::Relaxed);
    let spurious = IRQ_SPURIOUS.load(Ordering::Relaxed);
    if other != 0 || spurious != 0 {
        DbgPrint(b"[GIC] INTID>=64: %u delivered; spurious: %u\n\0".as_ptr(), other, spurious);
    }
}

// ---------------------------------------------------------------------------
// Low-level helpers.
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn mmio_write(base: usize, off: usize, val: u32) {
    core::ptr::write_volatile((base + off) as *mut u32, val);
}

#[inline(always)]
unsafe fn mmio_read(base: usize, off: usize) -> u32 {
    core::ptr::read_volatile((base + off) as *const u32)
}

#[inline(always)]
unsafe fn read_cntfrq() -> u64 {
    let f: u64;
    core::arch::asm!("mrs {0}, cntfrq_el0", out(reg) f, options(nomem, nostack, preserves_flags));
    f
}

/// (Re)arm the EL1 virtual timer: program a fresh deadline `interval` ticks from
/// *now* (writing TVAL sets CVAL = CNTVCT + interval, which deasserts the PPI),
/// then enable with IMASK clear (`CNTV_CTL_EL0` = 1).
#[inline(always)]
unsafe fn vtimer_rearm(interval: u64) {
    core::arch::asm!("msr cntv_tval_el0, {0}", in(reg) interval, options(nomem, nostack, preserves_flags));
    core::arch::asm!("msr cntv_ctl_el0, {0}", in(reg) 1u64, options(nomem, nostack, preserves_flags));
}

// ---------------------------------------------------------------------------
// Exports.
// ---------------------------------------------------------------------------

/// Initialize the GICv2 + ARM virtual timer for periodic 10ms clock interrupts.
///
/// `gic_dist` / `gic_cpu` are the kernel VAs of the GIC distributor / CPU
/// interface (already mapped by ntoskrnl via `MmMapIoSpace`).  Called once,
/// during Phase 1, before the first process switch.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalInitializeClockArm64(
    gic_dist: *mut core::ffi::c_void,
    gic_cpu: *mut core::ffi::c_void,
) {
    let gicd = gic_dist as usize;
    let gicc = gic_cpu as usize;
    if gicd == 0 || gicc == 0 {
        return;
    }
    GICD_BASE.store(gicd, Ordering::SeqCst);
    GICC_BASE.store(gicc, Ordering::SeqCst);

    // --- GICv2 distributor: enable + route the virtual-timer PPI (27). ---
    // Enable group-0 forwarding (single security state ⇒ group 0 + FIQEn=0 is
    // delivered as IRQ to EL1).
    mmio_write(gicd, GICD_CTLR, 1);
    // The clock runs at CLOCK_LEVEL (prio 0x20): above every device DIRQL, so
    // a driver raising to DIRQL never masks the system tick.
    core::ptr::write_volatile(
        (gicd + GICD_IPRIORITYR + VTIMER_INTID as usize) as *mut u8,
        gic_priority(CLOCK_LEVEL),
    );
    // Set-enable PPI 27 (ISENABLER0, bit 27).
    mmio_write(
        gicd,
        GICD_ISENABLER + (VTIMER_INTID as usize / 32) * 4,
        1u32 << (VTIMER_INTID % 32),
    );

    // --- GICv2 CPU interface + the BSP's banked SGI-IPI enable. (Also
    // overwrites the early-SMP identity-PA bases stored above with the real
    // kernel VAs — both are valid in this pre-SMSS window.) ---
    gic_percpu_init();

    // --- ARM generic virtual timer: arm the first 10ms deadline. ---
    let freq = read_cntfrq();
    let interval = if freq == 0 { 0 } else { freq / TICK_HZ };
    TIMER_INTERVAL.store(interval, Ordering::SeqCst);
    if interval != 0 {
        vtimer_rearm(interval);
        super::halstubs::halp_cpu_state()
            .vtimer_armed
            .store(1, Ordering::SeqCst);
    }

    // M5c tail: kick every AP with an SGI so each arms its own vtimer NOW
    // (HalClockInterruptArm64 lazy-arms on the first IRQ once TIMER_INTERVAL
    // is published — the banked PPI-27 registers and CNTV are per-CPU, so the
    // BSP cannot arm them remotely). GICv2 SGIR target-list-filter 01 =
    // "all CPUs but self" — no CPU enumeration needed, and a no-op on -smp 1.
    // Self-healing: if this SGI is ever lost, the next dispatch IPI arms it.
    #[cfg(feature = "smp")]
    {
        core::arch::asm!("dsb ish", options(nostack, preserves_flags));
        mmio_write(gicd, GICD_SGIR, (1u32 << 24) | SGI_IPI_INTID);
    }
}

/// Drop-guard that decrements this CPU's interrupt-context depth on every
/// exit path out of `HalClockInterruptArm64` (early returns included).
struct DepthGuard;
impl Drop for DepthGuard {
    fn drop(&mut self) {
        unsafe {
            super::halstubs::halp_cpu_state()
                .interrupt_context_depth
                .fetch_sub(1, Ordering::SeqCst);
        }
    }
}

/// IRQ-vector entry for arm64 — invoked by `KiIrqHandler` on every IRQ.
///
/// Acknowledges the GIC, and if it is the virtual-timer tick, re-arms the timer
/// and advances the NT system clock via `KeUpdateSystemTime`, then EOIs.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalClockInterruptArm64(trap_frame: *mut core::ffi::c_void) {
    let gicc = GICC_BASE.load(Ordering::SeqCst);
    if gicc == 0 {
        // Clock not initialized yet — nothing to acknowledge.
        return;
    }

    // Mark the whole IRQ exception window: KfLowerIrql must not deliver
    // DISPATCH software-interrupt work (timer expiration wakes/swaps threads)
    // from exception context — §9.19. The clock branch below runs
    // KeUpdateSystemTime with no IRQL bracket, so this vector-wide depth bump
    // is what keeps raise/lower pairs inside the tick path from dispatching.
    super::halstubs::halp_cpu_state()
        .interrupt_context_depth
        .fetch_add(1, Ordering::SeqCst);
    let _depth_guard = DepthGuard;

    // M5c tail: AP lazy vtimer arming. Any IRQ on an AP after clock init has
    // published the interval (the init-time SGI kick, or a dispatch IPI)
    // arms this CPU's banked PPI-27 + CNTV once.
    #[cfg(feature = "smp")]
    {
        let cpu = super::halstubs::halp_cpu_state();
        if cpu.vtimer_armed.load(Ordering::Relaxed) == 0
            && super::halstubs::halp_cpu_number() != 0
        {
            let interval = TIMER_INTERVAL.load(Ordering::SeqCst);
            if interval != 0 {
                vtimer_percpu_arm(interval);
                cpu.vtimer_armed.store(1, Ordering::SeqCst);
            }
        }
    }

    let iar = mmio_read(gicc, GICC_IAR);
    let intid = iar & 0x3FF;

    // Spurious (1020-1023): no EOI required.
    if intid >= GIC_SPURIOUS_MIN {
        IRQ_SPURIOUS.fetch_add(1, Ordering::Relaxed);
        return;
    }

    trace_delivery(intid);

    if intid == VTIMER_INTID {
        // Re-arm for the next deadline first so the timer PPI line deasserts
        // before we EOI (otherwise it immediately re-fires).
        let interval = TIMER_INTERVAL.load(Ordering::SeqCst);
        if interval != 0 {
            vtimer_rearm(interval);
        }

        // Advance the NT system clock.  `Irql` is the interrupted software IRQL
        // (used only for run-time accounting in `KeUpdateRunTime`).  We do NOT
        // raise/lower the software IRQL here: the hardware already masks IRQs
        // for the duration of the exception, and lowering would deliver APCs in
        // interrupt context.  The DISPATCH_LEVEL software interrupt that
        // timer-expiration raises is retired by `KiIdleLoop`.
        let old_irql = super::halstubs::KeGetCurrentIrql();
        #[cfg(not(feature = "smp"))]
        KeUpdateSystemTime(trap_frame, CLOCK_INCREMENT_100NS, old_irql);
        // SMP: exactly ONE CPU advances InterruptTime/SystemTime/KeTickCount
        // (double-advance = wall clock runs Nx fast). APs do the per-CPU
        // share only: thread time, DPC-rate bookkeeping, quantum end.
        #[cfg(feature = "smp")]
        if super::halstubs::halp_cpu_number() == 0 {
            KeUpdateSystemTime(trap_frame, CLOCK_INCREMENT_100NS, old_irql);
        } else {
            // One-shot M5c evidence: this AP's CNTV is delivering (armed=1 →
            // ticked=2). The -smp quest greps "CPUn clock tick live".
            let cpu = super::halstubs::halp_cpu_state();
            if cpu
                .vtimer_armed
                .compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                DbgPrint(
                    b"[SMP] CPU%u clock tick live (per-CPU CNTV delivering)\n\0".as_ptr(),
                    super::halstubs::halp_cpu_number(),
                );
            }
            KeUpdateRunTime(trap_frame, old_irql);
        }
    } else if intid < SGI_INTID_MAX {
        // SGI == the NT IPI (HalRequestIpi doorbell). Bracket at IPI_LEVEL —
        // raises this CPU's soft IRQL + PMR and marks exception context (no
        // softint drain on exit) — then run the kernel's IPI service routine
        // (APC/DPC/packet request bits on this CPU's PRCB). EOI below writes
        // back the full IAR, which for SGIs includes the source CPU field.
        let old_irql = super::halstubs::HalpEnterInterruptIrql(IPI_LEVEL);
        KiIpiServiceRoutine(core::ptr::null_mut(), core::ptr::null_mut());
        super::halstubs::HalpExitInterruptIrql(old_irql);
    } else {
        // Device interrupt (SPI etc.): dispatch to the ISR connected via
        // IoConnectInterrupt. ntoskrnl owns the KINTERRUPT registry; we EOI
        // below regardless of whether an ISR claimed it (an unconnected line
        // that somehow fired is acked-and-dropped, reported once per line).
        let handled = KiDispatchInterruptArm64(intid);
        if SELFTEST_INTID
            .compare_exchange(intid, 0, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // Armed by HalSelfTestGicSpiArm64: reaching here proves the
            // GIC→IRQ→dispatch wiring for this line end-to-end.
            DbgPrint(
                b"[GIC] selftest INTID %u DELIVERED (dispatch=%u)\n\0".as_ptr(),
                intid,
                handled as u32,
            );
        } else if handled == 0 {
            report_unhandled(intid);
        }
    }

    // EOI (write the acknowledged IAR value back).
    mmio_write(gicc, GICC_EOIR, iar);
}
