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
}
#[cfg(feature = "smp")]
extern "system" {
    fn HalRequestIpi(TargetSet: KAFFINITY);
    fn KeGetCurrentIrql() -> KIRQL;
    fn KeLowerIrql(NewIrql: KIRQL);
    fn KeBugCheckEx(Code: ULONG, P1: u64, P2: u64, P3: u64, P4: u64) -> !;
}
#[cfg(feature = "smp")]
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
    } else if IpiRequest == IPI_FREEZE {
        HalSendNMI(TargetSet);
    } else {
        core::debug_assert!(false);
    }
}

// ---------------------------------------------------------------------------
// M3 TLB shootdown — coarse broadcast transport over vector 0xE1.
//
// NT-shaped protocol: the interrupt is only a WAKE-UP; the actual request
// lives in KiTbFlushPending (bit per target CPU), so any CPU stuck spinning
// in this module — even with interrupts disabled — can service a pending
// flush by polling (KiTbFlushServiceSelf). First cut of this used a pure
// interrupt/ack barrier and deadlocked exactly like the classic textbook
// case: a sender that entered with IF=0 spun for the sender lock while the
// lock holder waited forever for its ack (caught live with GDB on both
// vCPUs, 2026-07-11).
//
//   sender:   serialize on KiTbFlushLock (servicing self while spinning) ->
//             set pending bits + wait count -> HalRequestIpi(targets) ->
//             spin { service self; until count == 0 } -> unlock.
//   receiver: 0xE1 ISR (KiIpiHandlerAmd64, raise/EOI/lower like the DPC
//             handler) -> service self.
//
// Deliberately coarse for bring-up: every remote flush is a full flush.
// NT's targeted KiFlushSingleTb / packet mailbox comes later (staged
// wave-2 A groundwork in ~/vibeos-rust-staging/smp/ipi-recv/).
// ---------------------------------------------------------------------------

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

#[cfg(feature = "smp")]
#[inline(always)]
unsafe fn KiTbFlushLocal() {
    let cr3: usize;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, nomem));
    core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
}

/// Claim + service this CPU's pending flush requests (counted and lazy).
/// Callable from ISRs and from any spin loop in this module.
#[cfg(feature = "smp")]
#[inline(always)]
unsafe fn KiTbFlushServiceSelf(me: u64) {
    use core::sync::atomic::Ordering;
    let mut flushed = false;
    if KiTbFlushPending.load(Ordering::Acquire) & me != 0 {
        if KiTbFlushPending.fetch_and(!me, Ordering::AcqRel) & me != 0 {
            KiTbFlushLocal();
            flushed = true;
            KiTbFlushWaitCount.fetch_sub(1, Ordering::SeqCst);
        }
    }
    if KiTbFlushPendingLazy.load(Ordering::Acquire) & me != 0 {
        if KiTbFlushPendingLazy.fetch_and(!me, Ordering::AcqRel) & me != 0 {
            if !flushed {
                KiTbFlushLocal();
            }
        }
    }
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
    KiTbFlushServiceSelf(me);
}

/// 0xE1 receive body. Called from trap.S's KiIpiInterrupt with cr8 still at
/// the interrupted level — raise/EOI/lower here, mirroring
/// KiDpcInterruptHandler (ExitTrap asserts cr8 is restored on return).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiIpiHandlerAmd64() {
    const IPI_LEVEL: KIRQL = 14;
    let old_irql = KfRaiseIrql(IPI_LEVEL);
    ntkrnl_sys::inline::ke_amd64::KiSendEOI();
    let me = (*(KeGetCurrentPrcb())).SetMember;
    KiTbFlushServiceSelf(me);
    KeLowerIrql(old_irql);
}

/// Broadcast "flush your whole TLB" to every other active processor and wait
/// for the acks of the AWAKE ones. Idle (lazy-TLB) CPUs are skipped — they
/// flush on wake, before touching anything remappable. Deadlock-free at any
/// IF state (senders poll-service while spinning; see module comment).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn KiTbShootdownBroadcast() {
    use core::sync::atomic::Ordering;
    use crate::ke::processor::KeActiveProcessors;

    let me = (*(KeGetCurrentPrcb())).SetMember;
    let targets = core::ptr::read_volatile(core::ptr::addr_of!(KeActiveProcessors)) & !me;
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
        KiTbFlushServiceSelf(me);
        core::hint::spin_loop();
        guard += 1;
        if guard > 4_000_000_000 {
            // nolint: silent-failure (loud unresponsive-processor bugcheck)
            KeBugCheckEx(0x101, 1, 0, 0, 0);
        }
    }

    // Recompute under the lock — the active set may have grown.
    let targets = core::ptr::read_volatile(core::ptr::addr_of!(KeActiveProcessors)) & !me;
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
            KiTbFlushServiceSelf(me);
            core::hint::spin_loop();
            guard += 1;
            if guard > 4_000_000_000 {
                // nolint: silent-failure (loud unresponsive-processor bugcheck)
                KeBugCheckEx(0x101, 2, awake, 0, 0);
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

#[no_mangle]
pub unsafe extern "system" fn KeIpiGenericCall(
    Function: PKIPI_BROADCAST_WORKER,
    Argument: ULONG_PTR,
) -> ULONG_PTR {
    DbgBreakPoint();
    0
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
