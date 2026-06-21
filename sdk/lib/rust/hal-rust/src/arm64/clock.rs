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

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// ---------------------------------------------------------------------------
// GICv2 register offsets (QEMU `virt`: GICD @ 0x08000000, GICC @ 0x08010000).
// ---------------------------------------------------------------------------

/// Distributor control register (bit0 = EnableGrp0).
const GICD_CTLR: usize = 0x000;
/// Set-enable, 1 bit per interrupt, 32 per 32-bit register.
const GICD_ISENABLER: usize = 0x100;
/// Priority, 1 byte per interrupt.
const GICD_IPRIORITYR: usize = 0x400;

/// CPU-interface control register (bit0 = Enable).
const GICC_CTLR: usize = 0x000;
/// Interrupt priority mask (interrupts with priority >= PMR are masked).
const GICC_PMR: usize = 0x004;
/// Interrupt acknowledge (read returns the pending INTID).
const GICC_IAR: usize = 0x00C;
/// End-of-interrupt (write the acknowledged IAR value back).
const GICC_EOIR: usize = 0x010;

/// EL1 virtual-timer PPI on the GIC (architectural INTID for CNTV).
const VTIMER_INTID: u32 = 27;

/// IAR values >= 1020 are special (1023 = spurious); no EOI is required.
const GIC_SPURIOUS_MIN: u32 = 1020;

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
    // Mid priority for PPI 27 (byte register; lower value = higher priority,
    // must be numerically below GICC_PMR to be delivered).
    core::ptr::write_volatile((gicd + GICD_IPRIORITYR + VTIMER_INTID as usize) as *mut u8, 0x80u8);
    // Set-enable PPI 27 (ISENABLER0, bit 27).
    mmio_write(
        gicd,
        GICD_ISENABLER + (VTIMER_INTID as usize / 32) * 4,
        1u32 << (VTIMER_INTID % 32),
    );

    // --- GICv2 CPU interface: unmask all priorities, enable. ---
    mmio_write(gicc, GICC_PMR, 0xF0);
    mmio_write(gicc, GICC_CTLR, 1);

    // --- ARM generic virtual timer: arm the first 10ms deadline. ---
    let freq = read_cntfrq();
    let interval = if freq == 0 { 0 } else { freq / TICK_HZ };
    TIMER_INTERVAL.store(interval, Ordering::SeqCst);
    if interval != 0 {
        vtimer_rearm(interval);
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

    let iar = mmio_read(gicc, GICC_IAR);
    let intid = iar & 0x3FF;

    // Spurious (1020-1023): no EOI required.
    if intid >= GIC_SPURIOUS_MIN {
        return;
    }

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
        KeUpdateSystemTime(trap_frame, CLOCK_INCREMENT_100NS, old_irql);
    }

    // EOI (write the acknowledged IAR value back).
    mmio_write(gicc, GICC_EOIR, iar);
}
