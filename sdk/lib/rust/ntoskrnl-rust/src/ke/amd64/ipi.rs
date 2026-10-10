use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    KAFFINITY,
    PKIPI_BROADCAST_WORKER,
};

const IPI_APC: ULONG = 1;
const IPI_DPC: ULONG = 2;
const IPI_FREEZE: ULONG = 0x4;

extern "system" {
    fn HalSendSoftwareInterrupt(TargetSet: KAFFINITY, Irql: KIRQL);
    fn HalSendNMI(TargetSet: KAFFINITY);
    fn DbgBreakPoint();
    fn KeGetCurrentIrql() -> KIRQL;
    fn KeLowerIrql(NewIrql: KIRQL);
    fn KeBugCheckEx(Code: ULONG, P1: u64, P2: u64, P3: u64, P4: u64) -> !;
}
#[cfg(feature = "smp")]
extern "system" {
    fn HalRequestIpi(TargetSet: KAFFINITY);
}
extern "C" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
}
#[cfg(feature = "smp")]
use ntkrnl_sys::inline::ke::KeGetCurrentPrcb;

#[no_mangle]
pub unsafe extern "C" fn KiIpiSend(
    TargetSet: KAFFINITY,
    IpiRequest: ULONG,
) {
    if IpiRequest == IPI_APC {
        HalSendSoftwareInterrupt(TargetSet, APC_LEVEL);
    } else if IpiRequest == IPI_DPC {
        HalSendSoftwareInterrupt(TargetSet, DISPATCH_LEVEL);
        // Divergence from the (never-SMP-validated) C: the DISPATCH soft vector is
        // priority class 2, and on amd64 IRQL *is* the APIC TPR -- so a target
        // sitting at DISPATCH_LEVEL, which is where the idle loop blocks in hlt,
        // holds 0x2F pending in its IRR and never wakes. The thread just handed to
        // it as NextThread (or the DPC just queued on it) then waits for the next
        // clock interrupt: sampled at -smp 2, a halted idle CPU had a NextThread
        // waiting in over half of all samples, and boot ran 4x slower than -smp 1.
        // 0xE1 is above every IRQL the idle loop runs at; its handler services
        // nothing here, but hlt returns and the idle loop re-checks NextThread and
        // the DPC queue. The 0x2F request stays: it is what preempts a BUSY target
        // once its IRQL drops. Not filtered to KiTbLazyIdleSet, because the idle
        // loop joins that set after its NextThread check -- a filtered wake races.
        #[cfg(feature = "smp")]
        HalRequestIpi(TargetSet);
    } else if IpiRequest == IPI_FREEZE {
        HalSendNMI(TargetSet);
    } else {
        ntkrnl_sys::ASSERT!(false);
    }
}

// ---------------------------------------------------------------------------
// M3 cross-CPU operation transport over vector 0xE1 (TLB shootdown +
// generic call).
//
// NT-shaped protocol: the interrupt is only a WAKE-UP; the actual request
// lives in per-op pending masks (bit per target CPU), so any CPU stuck
// spinning in this module — even with interrupts disabled — can service a
// pending op by polling (KiIpiServiceSelf). First cut of this used a pure
// interrupt/ack barrier and deadlocked exactly like the classic textbook
// case: a sender that entered with IF=0 spun for the sender lock while the
// lock holder waited forever for its ack (caught live with GDB on both
// vCPUs, 2026-07-11).
//
//   sender:   serialize on KiTbFlushLock — ONE descriptor op in flight at a
//             time, TLB or generic call (servicing self while spinning) ->
//             publish descriptor -> set pending bits + wait count ->
//             HalRequestIpi(targets) -> spin { service self; until acked }
//             -> unlock.
//   receiver: 0xE1 ISR (KiIpiHandlerAmd64, raise/EOI/lower like the DPC
//             handler) -> service self.
//
// TLB descriptor kinds: FULL (cr3 reload) or a single VA (invlpg) — the
// KiTbFlushVa slot is stable while any pending bit is set because the
// sender holds KiTbFlushLock until every ack lands. Lazy-idle CPUs always
// get a deferred FULL flush on wake (no VA history is kept for them).
//
// Generic call (KeIpiGenericCall): NT rendezvous semantics over the same
// transport — targets arrive (KiGcArrive countdown), spin for the sender's
// go signal, run the worker at IPI_LEVEL simultaneously with the sender,
// then ack (KiGcDone countdown). Lazy-idle CPUs are NOT skipped here (the
// worker must run on every processor); hlt wakes on the 0xE1 interrupt.
// ---------------------------------------------------------------------------

#[cfg(feature = "smp")]
const IPI_LEVEL: KIRQL = 14;

/// Serializes ALL cross-CPU 0xE1 descriptor operations (TLB + generic call).
#[cfg(feature = "smp")]
static KiTbFlushLock: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
#[cfg(feature = "smp")]
static KiTbFlushPending: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
#[cfg(feature = "smp")]
static KiTbFlushPendingLazy: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
#[cfg(feature = "smp")]
static KiTbFlushWaitCount: core::sync::atomic::AtomicI64 = core::sync::atomic::AtomicI64::new(0);
/// CPUs currently idle-in-hlt with TLB-coherence deferred ("lazy TLB").
/// A lazy CPU touches only stable kernel mappings (idle loop + PRCB), so
/// senders skip the IPI round-trip for it and just mark PendingLazy; the CPU
/// flushes on every exit from the lazy state (idle-exit, DPC ISR, 0xE1 ISR)
/// BEFORE running anything that could touch remapped memory.
#[cfg(feature = "smp")]
static KiTbLazyIdleSet: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// The in-flight TLB descriptor: KI_TB_FLUSH_FULL = flush everything, any
/// other value = invlpg exactly this VA. Written only under KiTbFlushLock;
/// stable for readers while their KiTbFlushPending bit is set (the sender
/// spins on KiTbFlushWaitCount before releasing the lock).
#[cfg(feature = "smp")]
const KI_TB_FLUSH_FULL: u64 = u64::MAX;
#[cfg(feature = "smp")]
static KiTbFlushVa: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(KI_TB_FLUSH_FULL);

// Generic-call (KeIpiGenericCall) descriptor — same single-writer-under-
// KiTbFlushLock discipline as the TLB descriptor above.
#[cfg(feature = "smp")]
static KiGcPending: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
#[cfg(feature = "smp")]
static KiGcFunction: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
#[cfg(feature = "smp")]
static KiGcArgument: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
#[cfg(feature = "smp")]
static KiGcArrive: core::sync::atomic::AtomicI64 = core::sync::atomic::AtomicI64::new(0);
#[cfg(feature = "smp")]
static KiGcGo: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
#[cfg(feature = "smp")]
static KiGcDone: core::sync::atomic::AtomicI64 = core::sync::atomic::AtomicI64::new(0);

#[cfg(feature = "smp")]
#[inline(always)]
unsafe fn KiTbFlushLocal() {
    let cr3: usize;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, nomem));
    core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
}

/// Claim + service this CPU's pending flush requests (counted and lazy).
#[cfg(feature = "smp")]
#[inline(always)]
unsafe fn KiTbFlushServiceSelf(me: u64) {
    use core::sync::atomic::Ordering;
    let mut flushed_full = false;
    if KiTbFlushPending.load(Ordering::Acquire) & me != 0 {
        if KiTbFlushPending.fetch_and(!me, Ordering::AcqRel) & me != 0 {
            // Read the descriptor AFTER claiming our bit and BEFORE the ack —
            // the sender can't recycle the slot until KiTbFlushWaitCount hits 0.
            // (Bisected 2026-07-11: the targeted invlpg was suspected in the
            // -smp 2 creep-wedge flake and exonerated — receiver-full-flush
            // showed the identical failure rate, and the M3b-HEAD control
            // reproduced the flake with none of this code present.)
            let va = KiTbFlushVa.load(Ordering::Acquire);
            if va == KI_TB_FLUSH_FULL {
                KiTbFlushLocal();
                flushed_full = true;
            } else {
                core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack, preserves_flags));
            }
            KiTbFlushWaitCount.fetch_sub(1, Ordering::SeqCst);
        }
    }
    if KiTbFlushPendingLazy.load(Ordering::Acquire) & me != 0 {
        if KiTbFlushPendingLazy.fetch_and(!me, Ordering::AcqRel) & me != 0 {
            if !flushed_full {
                KiTbFlushLocal();
            }
        }
    }
}

/// Claim + service a pending generic-call rendezvous for this CPU: arrive,
/// wait for the sender's go signal, run the worker at IPI_LEVEL, ack.
#[cfg(feature = "smp")]
#[inline(always)]
unsafe fn KiGcServiceSelf(me: u64) {
    use core::sync::atomic::Ordering;
    if KiGcPending.load(Ordering::Acquire) & me != 0 {
        if KiGcPending.fetch_and(!me, Ordering::AcqRel) & me != 0 {
            let fp = KiGcFunction.load(Ordering::Acquire);
            let arg = KiGcArgument.load(Ordering::Acquire);
            KiGcArrive.fetch_sub(1, Ordering::SeqCst);
            let mut guard: u64 = 0;
            while KiGcGo.load(Ordering::SeqCst) == 0 {
                core::hint::spin_loop();
                guard += 1;
                if guard > 4_000_000_000 {
                    // nolint: silent-failure (loud unresponsive-sender bugcheck)
                    KeBugCheckEx(ntkrnl_sys::bugcodes::CLOCK_WATCHDOG_TIMEOUT, 6, me, 0, 0);
                }
            }
            let f: PKIPI_BROADCAST_WORKER = core::mem::transmute(fp);
            // Run at IPI_LEVEL like NT; poll sites may sit below it, the
            // 0xE1 ISR is already there (raise-to-same is a no-op).
            let cur = KeGetCurrentIrql();
            let old = if cur < IPI_LEVEL { Some(KfRaiseIrql(IPI_LEVEL)) } else { None };
            if let Some(f) = f {
                f(arg as _);
            }
            if let Some(o) = old {
                KeLowerIrql(o);
            }
            KiGcDone.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

/// Service EVERY pending 0xE1 descriptor op for this CPU. This is the single
/// poll point: callable from the 0xE1 ISR, lazy-idle exit, and any spin loop
/// in this module (so a spinner never deadlocks a lock-holding sender).
#[cfg(feature = "smp")]
#[inline(always)]
unsafe fn KiIpiServiceSelf(me: u64) {
    KiTbFlushServiceSelf(me);
    KiGcServiceSelf(me);
}

/// Idle loop is about to block in hlt: enter lazy-TLB state.
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiTbLazyIdleEnter() {
    use core::sync::atomic::Ordering;
    let me = (*(KeGetCurrentPrcb())).SetMember;
    KiTbLazyIdleSet.fetch_or(me, Ordering::SeqCst);
}

/// Leaving idle (or about to run DPCs / dispatch a thread): drop lazy-TLB
/// state and service anything that was deferred while we slept.
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiTbLazyIdleExit() {
    use core::sync::atomic::Ordering;
    let me = (*(KeGetCurrentPrcb())).SetMember;
    KiTbLazyIdleSet.fetch_and(!me, Ordering::SeqCst);
    KiIpiServiceSelf(me);
    crate::ke::ipi::KiSmpApMarkerFlush((*(KeGetCurrentPrcb())).Number as u32);
}

/// Device-interrupt entry (trap.S KiInterruptDispatch, before the ISR runs).
///
/// The lazy protocol's premise was "a lazy CPU touches only stable kernel mappings",
/// but a CPU blocked in hlt is woken by DEVICE interrupts, and an ISR touches whatever
/// its driver mapped -- the IDE ISR copies PIO sector data through the system-VA
/// mapping of the request's MDL. Senders skip a lazy CPU, so that CPU still held a
/// translation for a system VA the pool had since recycled; the sector data went to
/// the old physical page. At amd64 -smp 2 that showed as successful 16 KB image reads
/// leaving their pages all zero (every one on CPU 1), csrss executing zeros, filesystem
/// metadata reading back corrupt (0x6B/STATUS_DISK_CORRUPT_ERROR), and wild kernel PCs
/// from the pages that took the misdirected writes. i386 has no lazy state and never
/// showed it. Only the idle-exit and DPC paths serviced the deferral before; this is
/// the third. Check first so a non-lazy CPU pays one load, not an atomic RMW.
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiTbLazyIdleExitForInterrupt() {
    use core::sync::atomic::Ordering;
    let me = (*(KeGetCurrentPrcb())).SetMember;
    if KiTbLazyIdleSet.load(Ordering::SeqCst) & me != 0 {
        KiTbLazyIdleExit();
    }
}

/// 0xE1 receive body. Called from trap.S's KiIpiInterrupt with cr8 still at
/// the interrupted level — raise/EOI/lower here, mirroring
/// KiDpcInterruptHandler (ExitTrap asserts cr8 is restored on return).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiIpiHandlerAmd64() {
    let old_irql = KfRaiseIrql(IPI_LEVEL);
    ntkrnl_sys::inline::ke_amd64::KiSendEOI();
    let me = (*(KeGetCurrentPrcb())).SetMember;
    KiIpiServiceSelf(me);
    KeLowerIrql(old_irql);
}

/// Broadcast "flush your whole TLB" to every other active processor and wait
/// for the acks of the AWAKE ones. Idle (lazy-TLB) CPUs are skipped — they
/// flush on wake, before touching anything remappable. Deadlock-free at any
/// IF state (senders poll-service while spinning; see module comment).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiTbShootdownBroadcast() {
    ki_tb_shootdown(KI_TB_FLUSH_FULL);
}

/// Targeted remote invalidation of a single VA (NT's KiFlushSingleTb shape):
/// same protocol as the full broadcast, but awake receivers invlpg exactly
/// this address instead of reloading cr3. Lazy-idle CPUs still get a
/// deferred FULL flush on wake (no per-VA history is kept for them).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiTbShootdownSingle(Address: u64) {
    // u64::MAX (the FULL sentinel) can't be a live invlpg target in practice;
    // if a caller ever passes it, we over-flush — harmless.
    ki_tb_shootdown(Address);
}

/// Which processors can hold a translation for `va`.
///
/// A USER address is mapped only through its process's page tables, so only the CPUs
/// whose CR3 is that process can have cached it: `ActiveProcessors` of the current
/// (attached) process, which KiSwapProcess maintains — set with a SeqCst RMW and an
/// mfence BEFORE the CR3 load. A CPU that joins after we read the mask therefore loads
/// a fresh CR3, and no user PTE is Global here (only the HAL's APIC page sets G), so
/// that load drops anything it cached for this address while it last ran us. The PTE
/// store the caller made is SeqCst too, which closes the Dekker race with a joiner.
/// NT scopes its process TB flushes the same way.
///
/// Measured why it matters (2026-10-09, `stress --sample-pcs`): two vCPUs on one host
/// core spent 91% of the boot in this protocol, ~46% of it user-mapping teardown
/// (MmDeleteVirtualMappingEx) broadcast to a CPU running some other process.
///
/// Everything else (system VAs, and the FULL sentinel, which callers use for kernel
/// ranges too) stays a broadcast.
#[cfg(feature = "smp")]
unsafe fn ki_tb_scope(va: u64) -> u64 {
    const USER_LIMIT: u64 = 0x0000_8000_0000_0000; // canonical lower half
    if va == KI_TB_FLUSH_FULL || va >= USER_LIMIT {
        return u64::MAX;
    }
    let thread = (*KeGetCurrentPrcb()).CurrentThread;
    let process = (*thread).__bindgen_anon_1.ApcState.Process;
    if process.is_null() {
        return u64::MAX;
    }
    core::sync::atomic::AtomicU64::from_ptr(
        core::ptr::addr_of_mut!((*process).ActiveProcessors) as *mut u64,
    )
    .load(core::sync::atomic::Ordering::SeqCst)
}

#[cfg(feature = "smp")]
unsafe fn ki_tb_shootdown(va: u64) {
    use core::sync::atomic::Ordering;
    use crate::ke::processor::KeActiveProcessors;

    let me = (*(KeGetCurrentPrcb())).SetMember;
    let scope = ki_tb_scope(va);
    let targets = core::ptr::read_volatile(core::ptr::addr_of!(KeActiveProcessors)) & !me & scope;
    if targets == 0 {
        return;
    }

    // Pin to this CPU for the duration (no migration mid-protocol).
    let old_irql = KeGetCurrentIrql();
    let raised = old_irql < DISPATCH_LEVEL;
    let mut prev: KIRQL = old_irql;
    if raised {
        prev = KfRaiseIrql(DISPATCH_LEVEL);
    }

    // Serialize senders, servicing our own pending requests while we wait.
    let mut guard: u64 = 0;
    while KiTbFlushLock.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_err() {
        KiIpiServiceSelf(me);
        core::hint::spin_loop();
        guard += 1;
        if guard > 4_000_000_000 {
            // nolint: silent-failure (loud unresponsive-processor bugcheck)
            KeBugCheckEx(ntkrnl_sys::bugcodes::CLOCK_WATCHDOG_TIMEOUT, 1, 0, 0, 0);
        }
    }

    // Publish the descriptor under the lock, before any pending bit is set.
    KiTbFlushVa.store(va, Ordering::SeqCst);

    // Recompute under the lock — the active set may have grown (and so may the
    // process's: ki_tb_scope re-reads ActiveProcessors).
    let targets =
        core::ptr::read_volatile(core::ptr::addr_of!(KeActiveProcessors)) & !me & ki_tb_scope(va);
    let lazy = KiTbLazyIdleSet.load(Ordering::SeqCst) & targets;
    let mut awake = targets & !lazy;

    // Lazy CPUs: mark PendingLazy, then re-check for escapees — a CPU that
    // left the lazy set between our read and the mark may have missed the
    // bit; reclaim it and treat it as awake (IPI + counted ack).
    if lazy != 0 {
        KiTbFlushPendingLazy.fetch_or(lazy, Ordering::SeqCst);
        let escaped = lazy & !KiTbLazyIdleSet.load(Ordering::SeqCst);
        if escaped != 0 {
            let reclaimed = KiTbFlushPendingLazy.fetch_and(!escaped, Ordering::SeqCst) & escaped;
            awake |= reclaimed; // it never serviced the lazy bit — count it
        }
    }

    let n = awake.count_ones() as i64;
    if n > 0 {
        KiTbFlushWaitCount.store(n, Ordering::SeqCst);
        KiTbFlushPending.fetch_or(awake, Ordering::SeqCst);
        HalRequestIpi(awake);
        let mut guard: u64 = 0;
        while KiTbFlushWaitCount.load(Ordering::SeqCst) > 0 {
            KiIpiServiceSelf(me);
            core::hint::spin_loop();
            guard += 1;
            if guard > 4_000_000_000 {
                // nolint: silent-failure (loud unresponsive-processor bugcheck)
                KeBugCheckEx(ntkrnl_sys::bugcodes::CLOCK_WATCHDOG_TIMEOUT, 2, awake, 0, 0);
            }
        }
    }

    KiTbFlushLock.store(0, Ordering::Release);
    if raised {
        KeLowerIrql(prev);
    }
}

/// UP build: trap.S still references the symbol; 0xE1 never fires there.
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn KiIpiHandlerAmd64() {}

// ---------------------------------------------------------------------------
// M3 visibility ("[SMP] CPUn scheduling live") — HOISTED to ke/ipi.rs
// (M5c, arch-neutral so the arm64 leg shares it). The count tap stays in
// this arch's KiSwapContextResume (thrdini.rs); the flush call below.
// ---------------------------------------------------------------------------

/// amd64 KeIpiGenericCall — NT rendezvous semantics ("run Function on every
/// processor simultaneously at IPI_LEVEL, return the caller's own result")
/// over the 0xE1 descriptor transport above. Replaces the upstream
/// DbgBreakPoint stub (amd64 ipi.c never implemented the packet mechanism).
#[no_mangle]
pub unsafe extern "system" fn KeIpiGenericCall(
    Function: PKIPI_BROADCAST_WORKER,
    Argument: ULONG_PTR,
) -> ULONG_PTR {
    #[cfg(feature = "smp")]
    {
        use core::sync::atomic::Ordering;
        use crate::ke::processor::KeActiveProcessors;

        let old_irql = KeGetCurrentIrql();
        let raised = old_irql < DISPATCH_LEVEL;
        let mut prev: KIRQL = old_irql;
        if raised {
            prev = KfRaiseIrql(DISPATCH_LEVEL);
        }

        let me = (*(KeGetCurrentPrcb())).SetMember;

        // Serialize with every other 0xE1 descriptor op (TLB + generic call),
        // servicing our own pending requests while we wait.
        let mut guard: u64 = 0;
        while KiTbFlushLock.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_err() {
            KiIpiServiceSelf(me);
            core::hint::spin_loop();
            guard += 1;
            if guard > 4_000_000_000 {
                // nolint: silent-failure (loud unresponsive-processor bugcheck)
                KeBugCheckEx(ntkrnl_sys::bugcodes::CLOCK_WATCHDOG_TIMEOUT, 3, 0, 0, 0);
            }
        }

        let targets = core::ptr::read_volatile(core::ptr::addr_of!(KeActiveProcessors)) & !me;
        let n = targets.count_ones() as i64;
        if n > 0 {
            // Publish the descriptor, then ring every target — including
            // lazy-idle CPUs (the worker must run on ALL processors; hlt
            // wakes on the 0xE1 interrupt and the ISR services us).
            KiGcFunction.store(core::mem::transmute::<PKIPI_BROADCAST_WORKER, usize>(Function), Ordering::SeqCst);
            KiGcArgument.store(Argument as u64, Ordering::SeqCst);
            KiGcGo.store(0, Ordering::SeqCst);
            KiGcArrive.store(n, Ordering::SeqCst);
            KiGcDone.store(n, Ordering::SeqCst);
            KiGcPending.fetch_or(targets, Ordering::SeqCst);
            HalRequestIpi(targets);

            // Wait for every target to arrive at the rendezvous.
            let mut guard: u64 = 0;
            while KiGcArrive.load(Ordering::SeqCst) > 0 {
                core::hint::spin_loop();
                guard += 1;
                if guard > 4_000_000_000 {
                    // nolint: silent-failure (loud unresponsive-processor bugcheck)
                    KeBugCheckEx(ntkrnl_sys::bugcodes::CLOCK_WATCHDOG_TIMEOUT, 4, targets, 0, 0);
                }
            }
        }

        // Everyone is staged: go — all CPUs (us included) run at IPI_LEVEL.
        let ipi_prev = KfRaiseIrql(IPI_LEVEL);
        KiGcGo.store(1, Ordering::SeqCst);
        let status = (Function.unwrap())(Argument as _); // nolint: silent-failure (non-null by PKIPI_BROADCAST_WORKER contract; a None here is a caller bug — panic is the fail-loud path)

        if n > 0 {
            let mut guard: u64 = 0;
            while KiGcDone.load(Ordering::SeqCst) > 0 {
                core::hint::spin_loop();
                guard += 1;
                if guard > 4_000_000_000 {
                    // nolint: silent-failure (loud unresponsive-processor bugcheck)
                    KeBugCheckEx(ntkrnl_sys::bugcodes::CLOCK_WATCHDOG_TIMEOUT, 5, targets, 0, 0);
                }
            }
        }

        KiTbFlushLock.store(0, Ordering::Release);
        KeLowerIrql(ipi_prev);
        if raised {
            KeLowerIrql(prev);
        }
        status as ULONG_PTR
    }
    #[cfg(not(feature = "smp"))]
    {
        // UP: no other processors — run the worker at IPI_LEVEL, like NT's
        // single-processor path. (Was a DbgBreakPoint stub.)
        const IPI_LEVEL_UP: KIRQL = 14;
        let old = KfRaiseIrql(IPI_LEVEL_UP);
        let status = (Function.unwrap())(Argument as _); // nolint: silent-failure (non-null by PKIPI_BROADCAST_WORKER contract; a None here is a caller bug — panic is the fail-loud path)
        KeLowerIrql(old);
        status as ULONG_PTR
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     IPI code for x64
//  * COPYRIGHT:   Copyright 2023 Timo Kreuzer <timo.kreuzer@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// FASTCALL
// KiIpiSend(
//     _In_ KAFFINITY TargetSet,
//     _In_ ULONG IpiRequest)
// {
//     /* Check if we can send the IPI directly */
//     if (IpiRequest == IPI_APC)
//     {
//         HalSendSoftwareInterrupt(TargetSet, APC_LEVEL);
//     }
//     else if (IpiRequest == IPI_DPC)
//     {
//         HalSendSoftwareInterrupt(TargetSet, DISPATCH_LEVEL);
//     }
//     else if (IpiRequest == IPI_FREEZE)
//     {
//         /* On x64 the freeze IPI is an NMI */
//         HalSendNMI(TargetSet);
//     }
//     else
//     {
//         ASSERT(FALSE);
//     }
// }
//
// ULONG_PTR
// NTAPI
// KeIpiGenericCall(
//     _In_ PKIPI_BROADCAST_WORKER Function,
//     _In_ ULONG_PTR Argument)
// {
//     __debugbreak();
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
