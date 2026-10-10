use super::ke_defs::*;
use ntkrnl_sys::inline::ke::KiInsertDeferredReadyList; // §9.56: inline (not the extern-C shim) so #[track_caller] names the true site
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _KTHREAD,
    _KPRCB,
    _KPROCESS,
    _EPROCESS,
    PKTHREAD,
    PKPRCB,
    PKPROCESS,
    PEPROCESS,
    KAFFINITY,
    KPRIORITY,
    SCHAR,
    HANDLE,
    KPROCESSOR_MODE,
    OB_OPEN_REASON,
    KeBugCheck,
    KeGetCurrentProcessorNumber,
    KeRaiseIrqlToSynchLevel,
    KiAcquirePrcbLock,
    KiAcquireThreadLock,
    KiComputeNewPriority,
    KiDeliverApc,
    KiExitDispatcher,
    KiIpiSend,
    KiReleasePrcbLock,
    KiReleaseThreadLock,
    KiSelectReadyThread,
    KiSetThreadSwapBusy,
    KiSwapContext,
    _KPCR,
};

extern "C" {
    fn CONTAINING_RECORD(ptr: PVOID, type_size: usize, field_offset: usize) -> PVOID;
}

#[cfg(target_pointer_width = "64")]
#[inline]
unsafe fn InterlockedOr64(target: *mut i64, value: i64) -> i64 {
    use core::sync::atomic::{AtomicI64, Ordering};
    let atomic = &*(target as *const AtomicI64);
    atomic.fetch_or(value, Ordering::SeqCst)
}

#[cfg(not(target_pointer_width = "64"))]
#[inline]
unsafe fn InterlockedOr_local(target: *mut i32, value: i32) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(target as *const AtomicI32);
    atomic.fetch_or(value, Ordering::SeqCst)
}

#[cfg(target_pointer_width = "64")]
macro_rules! InterlockedOrSetMember {
    ($dest:expr, $member:expr) => {
        InterlockedOr64(($dest) as *mut _ as *mut i64, $member as i64)
    };
}

#[cfg(not(target_pointer_width = "64"))]
macro_rules! InterlockedOrSetMember {
    ($dest:expr, $member:expr) => {
        InterlockedOr_local(($dest) as *mut _ as *mut i32, $member as i32)
    };
}

#[no_mangle]
pub static mut KiIdleSummary: KAFFINITY = 0;
#[no_mangle]
pub static mut KiIdleSummary_SMT: KAFFINITY = 0;

/// Drop `SetMember` from `KiIdleSummary`: that processor has work now.
///
/// Divergence from the C, which on SMP only ever SETS idle bits (its sole clear
/// is the `#ifndef CONFIG_SMP` path in KiDeferredReadyThread). After boot every
/// CPU read as idle forever, so KiSelectNextProcessor's idle preference was
/// always the whole affinity: a readied thread went to its ideal processor even
/// while that one was busy and another sat in WFI/hlt, and two readiers could
/// pick the same idle CPU. NT clears the bit when it hands an idle processor a
/// thread; so do we, plus where the arm64 idle loop takes work on its own.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn ki_idle_summary_clear(SetMember: KAFFINITY) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    (*(core::ptr::addr_of_mut!(KiIdleSummary) as *const AtomicUsize))
        .fetch_and(!(SetMember as usize), Ordering::SeqCst);
}

extern "C" {
    static mut PsPrioritySeparation: ULONG;
}
use super::processor::KeActiveProcessors;
use super::krnlinit::KiProcessorBlock;
// Use actual definition from clock.rs, not extern (lto=false splits them)
use super::clock::KeTickCount;

const MEMORY_PRIORITY_FOREGROUND: ULONG = 0x2;
const LOW_REALTIME_PRIORITY: KPRIORITY = 16;
const HIGH_PRIORITY: KPRIORITY = 31;
const MAXULONG_PTR: usize = usize::MAX;

const AdjustNone: UCHAR = 0;
const AdjustBoost: UCHAR = 0x02;
const AdjustUnwait: UCHAR = 0x01;

const DeferredReady: UCHAR = 7;
const Ready: UCHAR = 1;
const Standby: UCHAR = 3;
const Running: UCHAR = 2;
const Transition: UCHAR = 6;

const ProcessInMemory: UCHAR = 0;

const WrYieldExecution: UCHAR = 0x00000021;

const MUTANT_INCREMENT: LONG = 1;
const IPI_DPC: ULONG = 2;

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiIdleSchedule(Prcb: PKPRCB) -> PKTHREAD {
    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiProcessDeferredReadyList(Prcb: PKPRCB) {
    KiProcessDeferredReadyListInner(Prcb)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiProcessDeferredReadyList(Prcb: PKPRCB) {
    KiProcessDeferredReadyListInner(Prcb)
}

#[inline(always)]
unsafe fn KiProcessDeferredReadyListInner(Prcb: PKPRCB) {
    let mut ListEntry: *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY;
    let mut Thread: PKTHREAD;

    // §9.56 tripwire 0x5C — validate the deferred chain BEFORE consuming it.
    // Every node must be a thread with State==DeferredReady parked for THIS
    // cpu; anything else means a foreign write rerouted the SLIST through the
    // WaitListEntry union (e.g. a ready-list insert of a parked node) — catch
    // it here with the offending node's identity, before the walk spreads it.
    #[cfg(feature = "smp")]
    {
        let mut probe = (*Prcb).DeferredReadyListHead.Next;
        let mut n: u32 = 0;
        while !probe.is_null() {
            let t = CONTAINING_RECORD(
                probe as PVOID,
                0,
                core::mem::offset_of!(_KTHREAD, __bindgen_anon_3),
            ) as PKTHREAD;
            n += 1;
            if *kt_State(t) != DeferredReady
                || *kt_DeferredProcessor(t) != (*Prcb).Number as u8
                || n > 64
            {
                crate::dbg_print!(
                    b"[RQ5C] deferred-chain BAD node thr=%p state=%u defproc=%u thiscpu=%u pos=%u reason=%u\n\0",
                    t,
                    *kt_State(t) as u32,
                    *kt_DeferredProcessor(t) as u32,
                    (*Prcb).Number as u32,
                    n,
                    (*t).WaitReason as u32,
                );
                if ntkrnl_sys::inline::ke::RQ_ORACLE_ARMED {
                ntkrnl_sys::ddk::KeBugCheckEx(
                    ntkrnl_sys::bugcodes::INVALID_WORK_QUEUE_ITEM,
                    t as ntkrnl_sys::ddk::ULONG_PTR,
                    ((0x5Cu64 << 40) | ((*kt_State(t)) as u64) << 8 | n as u64)
                        as ntkrnl_sys::ddk::ULONG_PTR,
                    Prcb as ntkrnl_sys::ddk::ULONG_PTR,
                    probe as ntkrnl_sys::ddk::ULONG_PTR,
                );
                }
            }
            probe = (*probe).Next;
        }
    }

    ListEntry = (*Prcb).DeferredReadyListHead.Next;
    (*Prcb).DeferredReadyListHead.Next = core::ptr::null_mut();

    // §9.56 tripwire 0x5B — deferred-chain CYCLE detector. SwapListEntry.Next
    // is the WaitListEntry.Flink union byte-for-byte: one foreign write to a
    // parked deferred node cycles the chain, and this walk would then insert
    // the same thread twice (the self-loop tear) with 0x59 silent (single
    // push). A 2-CPU boot's deferred chain is short; >64 nodes = a cycle.
    #[cfg(feature = "smp")]
    let mut walk_count: u32 = 0;

    loop {
        #[cfg(feature = "smp")]
        {
            walk_count += 1;
            if walk_count > 64 {
                if ntkrnl_sys::inline::ke::RQ_ORACLE_ARMED {
                ntkrnl_sys::ddk::KeBugCheckEx(
                    ntkrnl_sys::bugcodes::INVALID_WORK_QUEUE_ITEM,
                    Prcb as ntkrnl_sys::ddk::ULONG_PTR,
                    (0x5Bu64 << 40) as ntkrnl_sys::ddk::ULONG_PTR,
                    ListEntry as ntkrnl_sys::ddk::ULONG_PTR,
                    walk_count as ntkrnl_sys::ddk::ULONG_PTR,
                );
                }
            }
        }
        Thread = CONTAINING_RECORD(
            ListEntry as PVOID,
            0,
            core::mem::offset_of!(_KTHREAD, __bindgen_anon_3),
        ) as PKTHREAD;
        ListEntry = (*ListEntry).Next;

        KiDeferredReadyThread(Thread);

        if ListEntry.is_null() {
            break;
        }
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiQueueReadyThread(Thread: PKTHREAD, Prcb: PKPRCB) {
    let Thread = opaque_ptr(Thread);
    KxQueueReadyThread(Thread, Prcb);
}

#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "system" fn KiFindIdealProcessor(
    mut ProcessorSet: KAFFINITY,
    OriginalIdealProcessor: UCHAR,
) -> ULONG {
    let OriginalIdealPrcb: PKPRCB;
    let NodeMask: KAFFINITY;
    let mut Processor: ULONG = 0;

    if (ProcessorSet & ((1 as KAFFINITY) << OriginalIdealProcessor)) != 0 {
        return OriginalIdealProcessor as ULONG;
    }

    ProcessorSet &= KeActiveProcessors;

    OriginalIdealPrcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB)
        .add(OriginalIdealProcessor as usize);

    NodeMask = (*(*OriginalIdealPrcb).ParentNode).ProcessorMask & ProcessorSet;
    if NodeMask != 0 {
        ProcessorSet = NodeMask;
    }

    // BitScanReverse(64): highest set bit index. ProcessorSet is non-zero
    // here (caller affinity is non-zero; NodeMask only narrows when set).
    Processor = (KAFFINITY::BITS - 1).wrapping_sub(ProcessorSet.leading_zeros()) as ULONG;

    Processor
}

#[cfg(feature = "smp")]
unsafe fn KiSelectNextProcessor(Thread: PKTHREAD) -> ULONG {
    let mut PreferredSet: KAFFINITY;
    let IdleSet: KAFFINITY;
    let mut Processor: ULONG = 0;

    PreferredSet = (*Thread).Affinity;

    // §9.56 guard 0x63: a zero affinity makes trailing_zeros() return 64 below
    // and KiProcessorBlock[64] is walked OUT OF BOUNDS — fail loud instead.
    if PreferredSet == 0 {
        if ntkrnl_sys::inline::ke::RQ_ORACLE_ARMED {
        ntkrnl_sys::ddk::KeBugCheckEx(
            ntkrnl_sys::bugcodes::INVALID_WORK_QUEUE_ITEM,
            Thread as ntkrnl_sys::ddk::ULONG_PTR,
            (0x63u64 << 40 | 1) as ntkrnl_sys::ddk::ULONG_PTR,
            0,
            0,
        );
        }
    }

    IdleSet = PreferredSet & KiIdleSummary;
    if IdleSet != 0 {
        PreferredSet = IdleSet;
    }

    if (PreferredSet & ((1 as KAFFINITY) << (*Thread).IdealProcessor)) != 0 {
        return (*Thread).IdealProcessor as ULONG;
    }

    // BitScanForward(64): lowest set bit index (PreferredSet non-zero — it
    // always contains at least the thread's affinity).
    Processor = PreferredSet.trailing_zeros() as ULONG;

    Processor
}

#[cfg(not(feature = "smp"))]
unsafe fn KiSelectNextProcessor(_Thread: PKTHREAD) -> ULONG {
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiDeferredReadyThread(Thread: PKTHREAD) {
    let Thread = opaque_ptr(Thread);
    KiDeferredReadyThreadImpl(Thread)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiDeferredReadyThread(Thread: PKTHREAD) {
    let Thread = opaque_ptr(Thread);
    KiDeferredReadyThreadImpl(Thread)
}

unsafe fn KiDeferredReadyThreadImpl(Thread: PKTHREAD) {
    let Prcb: PKPRCB;
    let Preempted: BOOLEAN;
    let Processor: ULONG;
    let mut OldPriority: KPRIORITY;
    let NextThread: PKTHREAD;

    // §9.56 diagnostic (armed): catches a thread being deferred-readied while it is
    // still CurrentThread (running) on some CPU — the confirmed root of the ready-list
    // tear. NOTE: the "spin on SwapBusy here" fix was WRONG — deferred-ready threads
    // NORMALLY carry SwapBusy set (it's part of the readying transition), so spinning
    // deadlocks every boot (0/16, wedge at 1 cswitch). The correct fix is upstream in
    // the wait-commit / unwait serialization (dethrone-before-readiable), NOT here.
    #[cfg(feature = "smp")]
    ntkrnl_sys::inline::ke::ki_check_not_running(Thread, 8888); // 8888 = via KiDeferredReadyThreadImpl drain

    if *kt_AdjustReason(Thread) == AdjustBoost {
        KiAcquireThreadLock(Thread);

        if ((*Thread).Priority <= *kt_AdjustIncrement(Thread))
            && ((*Thread).Priority < (LOW_REALTIME_PRIORITY as SCHAR - 3))
            && (kt_DisableBoost(Thread) == 0)
        {
            OldPriority = core::cmp::min(
                *kt_AdjustIncrement(Thread) as i32 + 1,
                LOW_REALTIME_PRIORITY as i32 - 3,
            );

            (*Thread).PriorityDecrement += (OldPriority as i8 - (*Thread).Priority) as SCHAR;
            (*Thread).Priority = OldPriority as SCHAR;
        }

        if *kt_Quantum(Thread) < 4 {
            *kt_Quantum(Thread) = 4;
        }
        *kt_Quantum(Thread) -= 1;

        KiReleaseThreadLock(Thread);
        *kt_AdjustReason(Thread) = AdjustNone;
    } else if *kt_AdjustReason(Thread) == AdjustUnwait {
        KiAcquireThreadLock(Thread);
        if (*Thread).Priority < LOW_REALTIME_PRIORITY as SCHAR {
            if (*Thread).BasePriority >= (LOW_REALTIME_PRIORITY as SCHAR - 2) {
                *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;
            } else {
                if ((*Thread).PriorityDecrement == 0) && (*kt_AdjustIncrement(Thread) != 0) {
                    *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;
                }

                if (*Thread).WaitStatus != STATUS_KERNEL_APC as ntkrnl_sys::ddk::LONG_PTR {
                    *kt_Quantum(Thread) -= 1;
                    if *kt_Quantum(Thread) <= 0 {
                        *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;
                        (*Thread).Priority = KiComputeNewPriority(Thread, 1);
                    }
                }
            }

            if ((*Thread).PriorityDecrement == 0) && (kt_DisableBoost(Thread) == 0) {
                OldPriority = (*Thread).BasePriority as i32 + *kt_AdjustIncrement(Thread) as i32;

                let process_ptr = (*Thread).__bindgen_anon_1.ApcState.Process as *mut _EPROCESS;
                if (*process_ptr).Vm.Flags.MemoryPriority() == MEMORY_PRIORITY_FOREGROUND {
                    OldPriority += PsPrioritySeparation as i32;
                }

                if OldPriority > (*Thread).Priority as i32 {
                    if OldPriority >= LOW_REALTIME_PRIORITY as i32 {
                        OldPriority = LOW_REALTIME_PRIORITY as i32 - 1;
                    }

                    if OldPriority > ((*Thread).BasePriority as i32 + *kt_AdjustIncrement(Thread) as i32)
                    {
                        (*Thread).PriorityDecrement = (OldPriority
                            - (*Thread).BasePriority as i32
                            - *kt_AdjustIncrement(Thread) as i32)
                            as SCHAR;
                    }

                    (*Thread).Priority = OldPriority as SCHAR;
                }
            }
        } else {
            *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;
        }

        KiReleaseThreadLock(Thread);
        *kt_AdjustReason(Thread) = AdjustNone;
    }

    Preempted = (*Thread).Preempted;
    OldPriority = (*Thread).Priority as KPRIORITY;
    (*Thread).Preempted = 0u8;

    Processor = KiSelectNextProcessor(Thread);
    *kt_NextProcessor(Thread) = Processor as UCHAR;

    Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB)
        .add(Processor as usize);

    KiAcquirePrcbLock(Prcb);

    #[cfg(not(feature = "smp"))]
    {
        if KiIdleSummary != 0 {
            KiIdleSummary = 0;
            *kt_State(Thread) = Standby;
            (*Prcb).NextThread = Thread;
            KiReleasePrcbLock(Prcb);
            return;
        }
    }

    // DBG: dump Prcb pointer value as hex
    #[cfg(target_arch = "x86")]
    {
        let prcb_val = Prcb as u32;
        for shift in (0..8).rev() {
            let nibble = ((prcb_val >> (shift * 4)) & 0xF) as u8;
            let ch = if nibble < 10 { b'0' + nibble } else { b'a' + nibble - 10 };
        }
    }

    let mut NextThread = (*Prcb).NextThread;
    if !NextThread.is_null() {
        if OldPriority > (*NextThread).Priority as KPRIORITY {
            (*NextThread).Preempted = 1u8;
            *kt_State(Thread) = Standby;
            (*Prcb).NextThread = Thread;
            *kt_State(NextThread) = DeferredReady;
            *kt_DeferredProcessor(NextThread) = (*Prcb).Number as u8;
            // §9.56: direct re-defer (displaced NextThread) — mark+stamp so
            // deferred_by attributes it (9200) instead of reading 0.
            #[cfg(feature = "smp")]
            {
                let _ = ntkrnl_sys::inline::ke::rq_inflight_mark(NextThread);
                ntkrnl_sys::inline::ke::rq_waker_stamp_pub(NextThread, 9200);
            }
            KiReleasePrcbLock(Prcb);
            KiDeferredReadyThreadImpl(NextThread);
            return;
        }
    } else {
        NextThread = (*Prcb).CurrentThread;
        if OldPriority > (*NextThread).Priority as KPRIORITY {
            if *kt_State(NextThread) == Running {
                (*NextThread).Preempted = 1u8;
            }

            *kt_State(Thread) = Standby;
            (*Prcb).NextThread = Thread;

            // Claiming an idle processor: it is no longer idle for the next
            // readier's KiSelectNextProcessor.
            #[cfg(feature = "smp")]
            if NextThread == (*Prcb).IdleThread {
                ki_idle_summary_clear((*Prcb).SetMember);
            }

            KiReleasePrcbLock(Prcb);

            if KeGetCurrentProcessorNumber() != *kt_NextProcessor(Thread) as ULONG {
                KiIpiSend((1 as KAFFINITY) << *kt_NextProcessor(Thread), IPI_DPC);
            }
            return;
        }
    }

    *kt_State(Thread) = Ready;
    *kt_WaitTime(Thread) = KeTickCount.LowPart as ULONG;

    // §9.56 tripwire 0x5A — the ONE ready-list insert with no already-linked
    // guard (KxQueueReadyThread has 0x54). The recurring self-loop tear at
    // site 0x22 is mechanically "InsertTailList of an entry that is ALREADY
    // the tail/sole element of this list" — catch it pre-mutation, with the
    // thread state + wait attribution. P2 tag 0x5A | state<<8 | reason.
    #[cfg(feature = "smp")]
    {
        let entry = core::ptr::addr_of_mut!((*Thread).__bindgen_anon_3.WaitListEntry)
            as PLIST_ENTRY;
        let f = (*entry).Flink;
        let b = (*entry).Blink;
        let still_linked = !f.is_null()
            && !b.is_null()
            && f != entry
            && (*f).Blink == entry
            && (*b).Flink == entry;
        if still_linked {
            crate::dbg_print!(
                b"[RQ5A] drain-insert of ALREADY-LINKED thr=%p state=%u reason=%u wstat=%p pri=%u F=%p B=%p deferred_by=%u\n\0",
                Thread,
                *kt_State(Thread) as u32,
                (*Thread).WaitReason as u32,
                (*Thread).WaitStatus as usize as *const u8,
                OldPriority as u32,
                f,
                b,
                ntkrnl_sys::inline::ke::rq_inflight_waker(Thread),
            );
            if ntkrnl_sys::inline::ke::RQ_ORACLE_ARMED {
            ntkrnl_sys::ddk::KeBugCheckEx(
                ntkrnl_sys::bugcodes::INVALID_WORK_QUEUE_ITEM,
                Thread as ntkrnl_sys::ddk::ULONG_PTR,
                ((0x5Au64 << 40)
                    | ((*kt_State(Thread)) as u64) << 8
                    | (*Thread).WaitReason as u64) as ntkrnl_sys::ddk::ULONG_PTR,
                f as ntkrnl_sys::ddk::ULONG_PTR,
                b as ntkrnl_sys::ddk::ULONG_PTR,
            );
            }
        }
    }

    if Preempted != 0 {
        InsertHeadList(
            core::ptr::addr_of_mut!(
                (*Prcb).DispatcherReadyListHead[OldPriority as usize]
            ) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*Thread).__bindgen_anon_3.WaitListEntry) as PLIST_ENTRY,
        );
    } else {
        InsertTailList(
            core::ptr::addr_of_mut!(
                (*Prcb).DispatcherReadyListHead[OldPriority as usize]
            ) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*Thread).__bindgen_anon_3.WaitListEntry) as PLIST_ENTRY,
        );
    }

    (*Prcb).ReadySummary |= 1u32 << OldPriority;

    #[cfg(feature = "smp")]
    ntkrnl_sys::inline::ke::ki_rq_integrity(Prcb, OldPriority as u32, 0x22); // site 0x22 = KiDeferredReadyThread insert
    KiReleasePrcbLock(Prcb);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiSelectNextThread(Prcb: PKPRCB) -> PKTHREAD {
    KiSelectNextThreadImpl(Prcb)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiSelectNextThread(Prcb: PKPRCB) -> PKTHREAD {
    KiSelectNextThreadImpl(Prcb)
}

unsafe fn KiSelectNextThreadImpl(Prcb: PKPRCB) -> PKTHREAD {
    let Thread: PKTHREAD;

    Thread = KiSelectReadyThread(0, Prcb);
    if Thread.is_null() {
        let idle_thread = (*Prcb).IdleThread;
        InterlockedOrSetMember!(&mut KiIdleSummary, (*Prcb).SetMember);
        (*Prcb).IdleSchedule = 1u8;
        idle_thread
    } else {
        Thread
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiSwapThread(
    CurrentThread: PKTHREAD,
    Prcb: PKPRCB,
) -> isize {
    let CurrentThread = opaque_ptr(CurrentThread);
    KiSwapThreadImpl(CurrentThread, Prcb)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiSwapThread(
    CurrentThread: PKTHREAD,
    Prcb: PKPRCB,
) -> isize {
    let CurrentThread = opaque_ptr(CurrentThread);
    KiSwapThreadImpl(CurrentThread, Prcb)
}

unsafe fn KiSwapThreadImpl(
    CurrentThread: PKTHREAD,
    Prcb: PKPRCB,
) -> isize {
    let mut ApcState: BOOLEAN = 0u8;
    let WaitIrql: KIRQL;
    let WaitStatus: isize;
    let mut NextThread: PKTHREAD;

    KiAcquirePrcbLock(Prcb);

    NextThread = (*Prcb).NextThread;
    if !NextThread.is_null() {
        // ARM64-only, mirrors C #ifdef _M_ARM64 (ntoskrnl/ke/thrdschd.c KiSwapThread):
        // arm64 lacks working HalRequestSoftwareInterrupt, so a stale Prcb->NextThread
        // is never re-dispatched via a DISPATCH software interrupt and a higher-priority
        // ready thread (e.g. queued by NtYieldExecution) can be starved — re-select the
        // true highest here. NOT drift: x86's software interrupt keeps this invariant so
        // the branch never fires there. Removable once arm64 has real software-interrupt
        // preemption — see docs/engineering/arm64-interrupts-quest.md.
        #[cfg(target_arch = "aarch64")]
        {
            if (*Prcb).ReadySummary != 0 {
                let HighBit: ULONG = 31 - (*Prcb).ReadySummary.leading_zeros();
                if (HighBit as SCHAR) > (*NextThread).Priority {
                    (*NextThread).Preempted = 0u8;
                    KxQueueReadyThread(NextThread, Prcb);
                    NextThread = KiSelectReadyThread(0, Prcb);
                }
            }
        }

        #[cfg(feature = "smp")]
        ntkrnl_sys::inline::ke::ki_check_promote_not_linked(NextThread, 0x04); // KiSwapThread

        (*Prcb).NextThread = core::ptr::null_mut();
        (*Prcb).CurrentThread = NextThread;
        *kt_State(NextThread) = Running;
        #[cfg(feature = "smp")]
        ntkrnl_sys::inline::ke::rq_inflight_unmark(NextThread);
    } else {
        NextThread = KiSelectReadyThread(0, Prcb);
        if !NextThread.is_null() {
            (*Prcb).CurrentThread = NextThread;
            *kt_State(NextThread) = Running;
            #[cfg(feature = "smp")]
            ntkrnl_sys::inline::ke::rq_inflight_unmark(NextThread);
        } else {
            InterlockedOrSetMember!(&mut KiIdleSummary, (*Prcb).SetMember);
            NextThread = (*Prcb).IdleThread;
            (*Prcb).CurrentThread = NextThread;
            *kt_State(NextThread) = Running;
        }
    }

    KiReleasePrcbLock(Prcb);

    // §9.56 THE SELF-SWAP. During the legal post-lock-release wait-commit window a
    // cross-CPU readier (deferred drain, often after an AdjustUnwait boost) can
    // standby/queue the committing thread onto ITS OWN still-running CPU, so the
    // selection above resolves NextThread == CurrentThread. KiSwapContext(T,T)
    // then wedges forever: ctxswitch.S spins until the NEW thread's SwapBusy
    // clears, and T set its own SwapBusy on the way in (KiSetThreadSwapBusy in
    // every wait path), which only KiSwapContextResume clears — after the spin.
    // Caught live 2026-10-09 by `stress --profile starved-smp --cores 2`: CPU#1
    // parked at .SwapBusySet with RDX == R8 (old == new), CPU#0 idle, boot
    // HANG_TIMEOUT. Upstream's CONFIG_SMP never ran this path; the 0x57 tripwire
    // only proved the mechanism and left every unarmed boot to wedge silently.
    //
    // T was already unwaited (WaitStatus set by the readier) and is already this
    // CPU's CurrentThread in state Running, so "switch to T" means "keep running
    // T": do exactly what KiSwapContextResume does for a thread resuming here —
    // release SwapBusy and report a deliverable kernel APC — minus the stack,
    // address-space and TEB switch, which are already T's.
    #[cfg(feature = "smp")]
    if NextThread == CurrentThread {
        // Breadcrumb, bounded: proves the path fires AND that the boot survives it
        // (before this fix a firing was a silent HANG_TIMEOUT).
        static SELF_SWAPS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        let n = SELF_SWAPS.fetch_add(1, core::sync::atomic::Ordering::Relaxed) + 1;
        if n <= 4 {
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[SMP] self-swap #%u resolved in place on CPU%u (thread %p)\n\0".as_ptr() as *const i8,
                n,
                (*Prcb).Number as u32,
                CurrentThread,
            );
        }
        WaitIrql = *kt_WaitIrql(CurrentThread);
        core::sync::atomic::AtomicU8::from_ptr(core::ptr::addr_of_mut!((*CurrentThread).SwapBusy))
            .store(0, core::sync::atomic::Ordering::Release);
        if (*CurrentThread).__bindgen_anon_1.ApcState.KernelApcPending != 0 {
            if *kt_SpecialApcDisable(CurrentThread) == 0 && WaitIrql == PASSIVE_LEVEL as KIRQL {
                ApcState = 1u8;
            } else {
                ntkrnl_sys::ddk::HalRequestSoftwareInterrupt(APC_LEVEL as KIRQL);
            }
        }
        WaitStatus = (*CurrentThread).WaitStatus as isize;
        if ApcState != 0 {
            KeLowerIrql(APC_LEVEL);
            KiDeliverApc(0i8, core::ptr::null_mut(), core::ptr::null_mut());
        }
        KeLowerIrql(WaitIrql);
        return WaitStatus;
    }

    WaitIrql = *kt_WaitIrql(CurrentThread);

    // cswitchdep tap-1 entry (cswitchdep-design.md): a voluntary switch must
    // begin at exactly DISPATCH (leaked-HIGH raise = §9.24 class); snapshot
    // the old thread's WaitIrql; evict its shadow on the final switch away
    // from a Terminated thread (KTHREAD pool reuse must not inherit it).
    #[cfg(feature = "cswitchdep")]
    super::cswitchdep::cswitch_suspend(
        CurrentThread as usize,
        WaitIrql,
        *kt_State(CurrentThread) == super::cswitchdep::STATE_TERMINATED,
        crate::code_addr!(),
    );

    ApcState = KiSwapContext(WaitIrql, CurrentThread);

    WaitStatus = (*CurrentThread).WaitStatus as isize;

    if ApcState != 0 {
        KeLowerIrql(APC_LEVEL);
        KiDeliverApc(0i8, core::ptr::null_mut(), core::ptr::null_mut());
    }

    KeLowerIrql(WaitIrql);

    // cswitchdep tap-1 exit: after the lower, live IRQL must equal the saved
    // WaitIrql — anything else is a broken raise/lower pairing across the wait.
    #[cfg(feature = "cswitchdep")]
    super::cswitchdep::cswitch_after_switch(CurrentThread as usize, WaitIrql, crate::code_addr!());

    WaitStatus
}

#[no_mangle]
pub unsafe extern "system" fn KiReadyThread(Thread: PKTHREAD) {
    let Thread = opaque_ptr(Thread);

    let Process: PKPROCESS = (*Thread).__bindgen_anon_1.ApcState.Process;

    if (*Process).State != ProcessInMemory {
    } else if (*Thread).KernelStackResident == 0 {
        (*Process).StackCount += 1;
        *kt_State(Thread) = Transition;
    } else {
        KiInsertDeferredReadyList(Thread);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KiAdjustQuantumThread(Thread: PKTHREAD) {
    let Thread = opaque_ptr(Thread);
    let Prcb: PKPRCB = KeGetCurrentPrcb();
    let NextThread: PKTHREAD;

    KiAcquireThreadLock(Thread);
    KiAcquirePrcbLock(Prcb);

    if ((*Thread).Priority < LOW_REALTIME_PRIORITY as SCHAR)
        && ((*Thread).BasePriority < (LOW_REALTIME_PRIORITY as SCHAR - 2))
    {
        *kt_Quantum(Thread) -= 1;
        if *kt_Quantum(Thread) <= 0 {
            *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;
            (*Thread).Priority = KiComputeNewPriority(Thread, 1);

            if (*Prcb).NextThread.is_null() {
                let next = KiSelectReadyThread((*Thread).Priority as KPRIORITY, Prcb);
                if !next.is_null() {
                    *kt_State(next) = Standby;
                    (*Prcb).NextThread = next;
                }
            } else {
                (*Thread).Preempted = 0u8;
            }
        }
    }

    KiReleasePrcbLock(Prcb);
    KiReleaseThreadLock(Thread);
    KiExitDispatcher(*kt_WaitIrql(Thread));
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiSetPriorityThread(
    Thread: PKTHREAD,
    Priority: KPRIORITY,
) {
    let Thread = opaque_ptr(Thread);
    KiSetPriorityThreadImpl(Thread, Priority)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiSetPriorityThread(
    Thread: PKTHREAD,
    Priority: KPRIORITY,
) {
    let Thread = opaque_ptr(Thread);
    KiSetPriorityThreadImpl(Thread, Priority)
}

unsafe fn KiSetPriorityThreadImpl(
    Thread: PKTHREAD,
    Priority: KPRIORITY,
) {
    let mut Prcb: PKPRCB;
    let mut Processor: ULONG;
    let mut RequestInterrupt: BOOLEAN = 0u8;
    let OldPriority: KPRIORITY;
    let NewThread: PKTHREAD;

    // DBG: KiSetPriorityThreadImpl entry

    if (*Thread).Priority as KPRIORITY != Priority {
        // DBG: priority differs, entering loop

        // DBG: emit thread state
        #[cfg(target_arch = "x86")]
        {
            let state = *kt_State(Thread);
        }

        loop {
            if *kt_State(Thread) == Ready {
                if (*Thread).ProcessReadyQueue == 0 {
                    Processor = *kt_NextProcessor(Thread) as ULONG;
                    Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB)
                        .add(Processor as usize);
                    KiAcquirePrcbLock(Prcb);

                    if (*kt_State(Thread) == Ready) && (*kt_NextProcessor(Thread) as ULONG == (*Prcb).Number as ULONG) {
                        if RemoveEntryList(
                            core::ptr::addr_of_mut!((*Thread).__bindgen_anon_3.WaitListEntry) as PLIST_ENTRY
                        ) {
                            (*Prcb).ReadySummary ^= 1u32 << ((*Thread).Priority as u32);
                        }

                        (*Thread).Priority = Priority as SCHAR;
                        #[cfg(feature = "smp")]
                        ntkrnl_sys::inline::ke::KiInsertDeferredReadyListRequeue(Thread);
                        #[cfg(not(feature = "smp"))]
                        KiInsertDeferredReadyList(Thread);
                        KiReleasePrcbLock(Prcb);
                    } else {
                        KiReleasePrcbLock(Prcb);
                        continue;
                    }
                } else {
                    (*Thread).Priority = Priority as SCHAR;
                }
            } else if *kt_State(Thread) == Standby {
                Processor = *kt_NextProcessor(Thread) as ULONG;
                Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB)
                    .add(Processor as usize);
                KiAcquirePrcbLock(Prcb);

                if Thread == (*Prcb).NextThread {
                    let old_pri = (*Thread).Priority as KPRIORITY;
                    (*Thread).Priority = Priority as SCHAR;

                    if Priority < old_pri {
                        let new_th = KiSelectReadyThread(Priority + 1, Prcb);
                        if !new_th.is_null() {
                            *kt_State(new_th) = Standby;
                            (*Prcb).NextThread = new_th;
                            #[cfg(feature = "smp")]
                            ntkrnl_sys::inline::ke::KiInsertDeferredReadyListRequeue(Thread);
                            #[cfg(not(feature = "smp"))]
                            KiInsertDeferredReadyList(Thread);
                        }
                    }

                    KiReleasePrcbLock(Prcb);
                } else {
                    KiReleasePrcbLock(Prcb);
                    continue;
                }
            } else if *kt_State(Thread) == Running {
                // DBG: Running branch

                Processor = *kt_NextProcessor(Thread) as ULONG;
                Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB)
                    .add(Processor as usize);

                // DBG: got Prcb

                KiAcquirePrcbLock(Prcb);

                // DBG: after PrcbLock

                if Thread == (*Prcb).CurrentThread {
                    let old_pri = (*Thread).Priority as KPRIORITY;
                    (*Thread).Priority = Priority as SCHAR;

                    // DBG: before KiSelectReadyThread check

                    if (Priority < old_pri) && ((*Prcb).NextThread.is_null()) {
                        // DBG: calling KiSelectReadyThread

                        let new_th = KiSelectReadyThread(Priority + 1, Prcb);

                        // DBG: after KiSelectReadyThread

                        if !new_th.is_null() {
                            *kt_State(new_th) = Standby;
                            (*Prcb).NextThread = new_th;
                            RequestInterrupt = 1u8;
                        }
                    }

                    // DBG: before KiReleasePrcbLock

                    KiReleasePrcbLock(Prcb);
                    if RequestInterrupt != 0 {
                        if KeGetCurrentProcessorNumber() != Processor {
                            KiIpiSend((1 as KAFFINITY) << Processor, IPI_DPC);
                        }
                    }
                } else {
                    KiReleasePrcbLock(Prcb);
                    continue;
                }
            } else if *kt_State(Thread) == DeferredReady {
            } else {
                (*Thread).Priority = Priority as SCHAR;
            }

            break;
        }
    }
}

#[cfg(feature = "smp")]
unsafe fn KiUpdateEffectiveAffinityThread(Thread: PKTHREAD) {
    let Prcb: PKPRCB;

    KiAcquireThreadLock(Thread);

    Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB)
        .add(*kt_NextProcessor(Thread) as usize);
    KiAcquirePrcbLock(Prcb);

    (*Thread).Affinity = (*Thread).UserAffinity;
    (*Thread).IdealProcessor = *kt_UserIdealProcessor(Thread);

    if ((*Prcb).SetMember & (*Thread).Affinity) == 0 {
        if *kt_State(Thread) == Running {
            if (*Prcb).NextThread.is_null() {
                (*Prcb).NextThread = KiSelectNextThread(Prcb);
                *kt_State((*Prcb).NextThread) = Standby;
            }

            if Prcb != KeGetCurrentPrcb() {
                KiIpiSend((1 as KAFFINITY) << *kt_NextProcessor(Thread), IPI_DPC);
            }
        } else if *kt_State(Thread) == Standby {
            (*Prcb).NextThread = KiSelectNextThread(Prcb);
            *kt_State((*Prcb).NextThread) = Standby;
            ntkrnl_sys::inline::ke::KiInsertDeferredReadyListRequeue(Thread);
        } else if *kt_State(Thread) == Ready {
            if RemoveEntryList(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_3.WaitListEntry) as PLIST_ENTRY)
            {
                (*Prcb).ReadySummary &= !(1u32 << ((*Thread).Priority as u32));
            }
            ntkrnl_sys::inline::ke::KiInsertDeferredReadyListRequeue(Thread);
        }
    }

    KiReleasePrcbLock(Prcb);
    KiReleaseThreadLock(Thread);
}

unsafe fn KiSetAffinityThread_impl(
    Thread: PKTHREAD,
    Affinity: KAFFINITY,
) -> KAFFINITY {
    let OldAffinity: KAFFINITY;

    OldAffinity = (*Thread).UserAffinity;

    if ((Affinity & (*((*Thread).__bindgen_anon_1.ApcState.Process as *mut _KPROCESS)).Affinity) != Affinity)
        || (Affinity == 0)
    {
        KeBugCheck(ntkrnl_sys::bugcodes::INVALID_AFFINITY_SET);
    }

    (*Thread).UserAffinity = Affinity;

    #[cfg(feature = "smp")]
    {
        if *kt_SystemAffinityActive(Thread) == 0 {
            *kt_UserIdealProcessor(Thread) =
                KiFindIdealProcessor(Affinity, *kt_UserIdealProcessor(Thread)) as u8;
            KiUpdateEffectiveAffinityThread(Thread);
        }
    }

    OldAffinity
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiSetAffinityThread(
    Thread: PKTHREAD,
    Affinity: KAFFINITY,
) -> KAFFINITY {
    let Thread = opaque_ptr(Thread);
    KiSetAffinityThread_impl(Thread, Affinity)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiSetAffinityThread(
    Thread: PKTHREAD,
    Affinity: KAFFINITY,
) -> KAFFINITY {
    let Thread = opaque_ptr(Thread);
    KiSetAffinityThread_impl(Thread, Affinity)
}

fn KiGetCurrentReadySummary() -> ULONG {
    unsafe { (*KeGetCurrentPrcb()).ReadySummary }
}

#[no_mangle]
pub unsafe extern "system" fn NtYieldExecution() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let OldIrql: KIRQL;
    let Prcb: PKPRCB;
    let Thread: PKTHREAD;
    let mut NextThread: PKTHREAD;

    if KiGetCurrentReadySummary() == 0 {
        return STATUS_NO_YIELD_PERFORMED;
    }

    Status = STATUS_NO_YIELD_PERFORMED;
    Thread = super::ke_defs::KeGetCurrentThread();

    OldIrql = KeRaiseIrqlToSynchLevel();
    Prcb = KeGetCurrentPrcb();

    if (*Prcb).ReadySummary != 0 {
        KiAcquireThreadLock(Thread);
        KiAcquirePrcbLock(Prcb);

        // ARM64-only, mirrors C #ifdef _M_ARM64 (ntoskrnl/ke/thrdschd.c): without a
        // working HalRequestSoftwareInterrupt, the thread NtYieldExecution just queued
        // could lose to a stale Prcb->NextThread of lower priority — drop the stale
        // NextThread so the reselect below picks the true highest. NOT drift: x86's
        // software interrupt makes this unnecessary (the branch never fires). Removable
        // once arm64 has real software-interrupt preemption — see
        // docs/engineering/arm64-interrupts-quest.md.
        #[cfg(target_arch = "aarch64")]
        {
            if !(*Prcb).NextThread.is_null() && (*Prcb).ReadySummary != 0 {
                let HighBit: ULONG = 31 - (*Prcb).ReadySummary.leading_zeros();
                if (HighBit as SCHAR) > (*(*Prcb).NextThread).Priority {
                    let Scheduled = (*Prcb).NextThread;
                    (*Scheduled).Preempted = 0u8;
                    KxQueueReadyThread(Scheduled, Prcb);
                    (*Prcb).NextThread = core::ptr::null_mut();
                }
            }
        }

        if (*Prcb).NextThread.is_null() {
            (*Prcb).NextThread = KiSelectReadyThread(1, Prcb);
        }

        NextThread = (*Prcb).NextThread;
        if !NextThread.is_null() {
            *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;
            (*Thread).Priority = KiComputeNewPriority(Thread, 1);

            KiReleaseThreadLock(Thread);

            KiSetThreadSwapBusy(Thread);

            (*Prcb).NextThread = core::ptr::null_mut();
            (*Prcb).CurrentThread = NextThread;
            *kt_State(NextThread) = Running;
            #[cfg(feature = "smp")]
            ntkrnl_sys::inline::ke::rq_inflight_unmark(NextThread);

            (*Thread).WaitReason = _KWAIT_REASON_WrYieldExecution as UCHAR;
            KxQueueReadyThread(Thread, Prcb);

            *kt_WaitIrql(Thread) = APC_LEVEL;

            KiSwapContext(APC_LEVEL, Thread);
            Status = STATUS_SUCCESS;
        } else {
            KiReleasePrcbLock(Prcb);
            KiReleaseThreadLock(Thread);
        }
    }

    KeLowerIrql(OldIrql);
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/thrdschd.c
//  * PURPOSE:         Kernel Thread Scheduler (Affinity, Priority, Scheduling)
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #ifdef _WIN64
// # define InterlockedOrSetMember(Destination, SetMember) \
//     InterlockedOr64((PLONG64)Destination, SetMember);
// #else
// # define InterlockedOrSetMember(Destination, SetMember) \
//     InterlockedOr((PLONG)Destination, SetMember);
// #endif
//
// /* GLOBALS *******************************************************************/
//
// KAFFINITY KiIdleSummary;
// KAFFINITY KiIdleSMTSummary;
//
// /* FUNCTIONS *****************************************************************/
//
// PKTHREAD
// FASTCALL
// KiIdleSchedule(IN PKPRCB Prcb)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("SMP: Not yet implemented\n", FALSE);
//     return NULL;
// }
//
// VOID
// FASTCALL
// KiProcessDeferredReadyList(IN PKPRCB Prcb)
// {
//     PSINGLE_LIST_ENTRY ListEntry;
//     PKTHREAD Thread;
//
//     /* Make sure there is something on the ready list */
//     ASSERT(Prcb->DeferredReadyListHead.Next != NULL);
//
//     /* Get the first entry and clear the list */
//     ListEntry = Prcb->DeferredReadyListHead.Next;
//     Prcb->DeferredReadyListHead.Next = NULL;
//
//     /* Start processing loop */
//     do
//     {
//         /* Get the thread and advance to the next entry */
//         Thread = CONTAINING_RECORD(ListEntry, KTHREAD, SwapListEntry);
//         ListEntry = ListEntry->Next;
//
//         /* Make the thread ready */
//         KiDeferredReadyThread(Thread);
//     } while (ListEntry != NULL);
//
//     /* Make sure the ready list is still empty */
//     ASSERT(Prcb->DeferredReadyListHead.Next == NULL);
// }
//
// VOID
// FASTCALL
// KiQueueReadyThread(IN PKTHREAD Thread,
//                    IN PKPRCB Prcb)
// {
//     /* Call the macro. We keep the API for compatibility with ASM code */
//     KxQueueReadyThread(Thread, Prcb);
// }
//
// #ifdef CONFIG_SMP
// ULONG
// NTAPI
// KiFindIdealProcessor(
//     _In_ KAFFINITY ProcessorSet,
//     _In_ UCHAR OriginalIdealProcessor)
// {
//     PKPRCB OriginalIdealPrcb;
//     KAFFINITY NodeMask;
//     ULONG Processor;
//
//     /* Check if we can use the original ideal processor */
//     if (ProcessorSet & AFFINITY_MASK(OriginalIdealProcessor))
//     {
//         /* We can, so use it */
//         return OriginalIdealProcessor;
//     }
//
//     /* Only use active processors */
//     ProcessorSet &= KeActiveProcessors;
//
//     /* Get the original ideal PRCB */
//     OriginalIdealPrcb = KiProcessorBlock[OriginalIdealProcessor];
//
//     /* Check if we can use the original node */
//     NodeMask = OriginalIdealPrcb->ParentNode->ProcessorMask & ProcessorSet;
//     if (NodeMask)
//     {
//         /* Use the node set instead */
//         ProcessorSet = NodeMask;
//     }
//
//     /* Calculate the ideal CPU from the affinity set */
//     BitScanReverseAffinity(&Processor, ProcessorSet);
//     return Processor;
// }
//
// static
// ULONG
// KiSelectNextProcessor(
//     _In_ PKTHREAD Thread)
// {
//     KAFFINITY PreferredSet, IdleSet;
//     ULONG Processor;
//
//     /* Start with the affinity */
//     PreferredSet = Thread->Affinity;
//
//     /* If we have matching idle processors, use them */
//     IdleSet = PreferredSet & KiIdleSummary;
//     if (IdleSet != 0)
//     {
//         PreferredSet = IdleSet;
//     }
//
//     /* Check if we can use the ideal processor */
//     if (PreferredSet & AFFINITY_MASK(Thread->IdealProcessor))
//     {
//         return Thread->IdealProcessor;
//     }
//
//     /* Return the first set bit */
//     NT_VERIFY(BitScanForwardAffinity(&Processor, PreferredSet) != FALSE);
//     ASSERT(Processor < KeNumberProcessors);
//
//     return Processor;
// }
// #else
// #define KiSelectNextProcessor(Thread) 0
// #endif
//
// VOID
// FASTCALL
// KiDeferredReadyThread(IN PKTHREAD Thread)
// {
//     PKPRCB Prcb;
//     BOOLEAN Preempted;
//     ULONG Processor;
//     KPRIORITY OldPriority;
//     PKTHREAD NextThread;
//
//     /* Sanity checks */
//     ASSERT(Thread->State == DeferredReady);
//     ASSERT((Thread->Priority >= 0) && (Thread->Priority <= HIGH_PRIORITY));
//
//     /* Check if we have any adjusts to do */
//     if (Thread->AdjustReason == AdjustBoost)
//     {
//         /* Lock the thread */
//         KiAcquireThreadLock(Thread);
//
//         /* Check if the priority is low enough to qualify for boosting */
//         if ((Thread->Priority <= Thread->AdjustIncrement) &&
//             (Thread->Priority < (LOW_REALTIME_PRIORITY - 3)) &&
//             !(Thread->DisableBoost))
//         {
//             /* Calculate the new priority based on the adjust increment */
//             OldPriority = min(Thread->AdjustIncrement + 1,
//                               LOW_REALTIME_PRIORITY - 3);
//
//             /* Make sure we're not decreasing outside of the priority range */
//             ASSERT((Thread->PriorityDecrement >= 0) &&
//                    (Thread->PriorityDecrement <= Thread->Priority));
//
//             /* Calculate the new priority decrement based on the boost */
//             Thread->PriorityDecrement += ((SCHAR)OldPriority - Thread->Priority);
//
//             /* Again verify that this decrement is valid */
//             ASSERT((Thread->PriorityDecrement >= 0) &&
//                    (Thread->PriorityDecrement <= OldPriority));
//
//             /* Set the new priority */
//             Thread->Priority = (SCHAR)OldPriority;
//         }
//
//         /* We need 4 quanta, make sure we have them, then decrease by one */
//         if (Thread->Quantum < 4) Thread->Quantum = 4;
//         Thread->Quantum--;
//
//         /* Make sure the priority is still valid */
//         ASSERT((Thread->Priority >= 0) && (Thread->Priority <= HIGH_PRIORITY));
//
//         /* Release the lock and clear the adjust reason */
//         KiReleaseThreadLock(Thread);
//         Thread->AdjustReason = AdjustNone;
//     }
//     else if (Thread->AdjustReason == AdjustUnwait)
//     {
//         /* Acquire the thread lock and check if this is a real-time thread */
//         KiAcquireThreadLock(Thread);
//         if (Thread->Priority < LOW_REALTIME_PRIORITY)
//         {
//             /* It's not real time, but is it time critical? */
//             if (Thread->BasePriority >= (LOW_REALTIME_PRIORITY - 2))
//             {
//                 /* It is, so simply reset its quantum */
//                 Thread->Quantum = Thread->QuantumReset;
//             }
//             else
//             {
//                 /* Has the priority been adjusted previously? */
//                 if (!(Thread->PriorityDecrement) && (Thread->AdjustIncrement))
//                 {
//                     /* Yes, reset its quantum */
//                     Thread->Quantum = Thread->QuantumReset;
//                 }
//
//                 /* Wait code already handles quantum adjustment during APCs */
//                 if (Thread->WaitStatus != STATUS_KERNEL_APC)
//                 {
//                     /* Decrease the quantum by one and check if we're out */
//                     if (--Thread->Quantum <= 0)
//                     {
//                         /* We are, reset the quantum and get a new priority */
//                         Thread->Quantum = Thread->QuantumReset;
//                         Thread->Priority = KiComputeNewPriority(Thread, 1);
//                     }
//                 }
//             }
//
//             /* Now check if we have no decrement and boosts are enabled */
//             if (!(Thread->PriorityDecrement) && !(Thread->DisableBoost))
//             {
//                 /* Make sure we have an increment */
//                 ASSERT(Thread->AdjustIncrement >= 0);
//
//                 /* Calculate the new priority after the increment */
//                 OldPriority = Thread->BasePriority + Thread->AdjustIncrement;
//
//                 /* Check if this is a foreground process */
//                 if (CONTAINING_RECORD(Thread->ApcState.Process, EPROCESS, Pcb)->
//                     Vm.Flags.MemoryPriority == MEMORY_PRIORITY_FOREGROUND)
//                 {
//                     /* Apply the foreground boost */
//                     OldPriority += PsPrioritySeparation;
//                 }
//
//                 /* Check if this new priority is higher */
//                 if (OldPriority > Thread->Priority)
//                 {
//                     /* Make sure we don't go into the real time range */
//                     if (OldPriority >= LOW_REALTIME_PRIORITY)
//                     {
//                         /* Normalize it back down one notch */
//                         OldPriority = LOW_REALTIME_PRIORITY - 1;
//                     }
//
//                     /* Check if the priority is higher then the boosted base */
//                     if (OldPriority > (Thread->BasePriority +
//                                        Thread->AdjustIncrement))
//                     {
//                         /* Setup a priority decrement to nullify the boost  */
//                         Thread->PriorityDecrement = ((SCHAR)OldPriority -
//                                                     Thread->BasePriority -
//                                                     Thread->AdjustIncrement);
//                     }
//
//                     /* Make sure that the priority decrement is valid */
//                     ASSERT((Thread->PriorityDecrement >= 0) &&
//                            (Thread->PriorityDecrement <= OldPriority));
//
//                     /* Set this new priority */
//                     Thread->Priority = (SCHAR)OldPriority;
//                 }
//             }
//         }
//         else
//         {
//             /* It's a real-time thread, so just reset its quantum */
//             Thread->Quantum = Thread->QuantumReset;
//         }
//
//         /* Make sure the priority makes sense */
//         ASSERT((Thread->Priority >= 0) && (Thread->Priority <= HIGH_PRIORITY));
//
//         /* Release the thread lock and reset the adjust reason */
//         KiReleaseThreadLock(Thread);
//         Thread->AdjustReason = AdjustNone;
//     }
//
//     /* Clear thread preemption status and save current values */
//     Preempted = Thread->Preempted;
//     OldPriority = Thread->Priority;
//     Thread->Preempted = FALSE;
//
//     /* Select a processor to run on */
//     Processor = KiSelectNextProcessor(Thread);
//     Thread->NextProcessor = Processor;
//
//     /* Get the PRCB and lock it */
//     Prcb = KiProcessorBlock[Processor];
//     KiAcquirePrcbLock(Prcb);
//
// #ifndef CONFIG_SMP
//     /* Check if we have an idle summary */
//     if (KiIdleSummary)
//     {
//         /* Clear it and set this thread as the next one */
//         KiIdleSummary = 0;
//         Thread->State = Standby;
//         Prcb->NextThread = Thread;
//
//         /* Unlock the PRCB and return */
//         KiReleasePrcbLock(Prcb);
//         return;
//     }
// #endif // !CONFIG_SMP
//
//     /* Get the next scheduled thread */
//     NextThread = Prcb->NextThread;
//     if (NextThread)
//     {
//         /* Sanity check */
//         ASSERT(NextThread->State == Standby);
//
//         /* Check if priority changed */
//         if (OldPriority > NextThread->Priority)
//         {
//             /* Preempt the thread */
//             NextThread->Preempted = TRUE;
//
//             /* Put this one as the next one */
//             Thread->State = Standby;
//             Prcb->NextThread = Thread;
//
//             /* Set it in deferred ready mode */
//             NextThread->State = DeferredReady;
//             NextThread->DeferredProcessor = Prcb->Number;
//             KiReleasePrcbLock(Prcb);
//             KiDeferredReadyThread(NextThread);
//             return;
//         }
//     }
//     else
//     {
//         /* Set the next thread as the current thread */
//         NextThread = Prcb->CurrentThread;
//         if (OldPriority > NextThread->Priority)
//         {
//             /* Preempt it if it's already running */
//             if (NextThread->State == Running) NextThread->Preempted = TRUE;
//
//             /* Set the thread on standby and as the next thread */
//             Thread->State = Standby;
//             Prcb->NextThread = Thread;
//
//             /* Release the lock */
//             KiReleasePrcbLock(Prcb);
//
//             /* Check if we're running on another CPU */
//             if (KeGetCurrentProcessorNumber() != Thread->NextProcessor)
//             {
//                 /* We are, send an IPI */
//                 KiIpiSend(AFFINITY_MASK(Thread->NextProcessor), IPI_DPC);
//             }
//             return;
//         }
//     }
//
//     /* Sanity check */
//     ASSERT((OldPriority >= 0) && (OldPriority <= HIGH_PRIORITY));
//
//     /* Set this thread as ready */
//     Thread->State = Ready;
//     Thread->WaitTime = KeTickCount.LowPart;
//
//     /* Insert this thread in the appropriate order */
//     Preempted ? InsertHeadList(&Prcb->DispatcherReadyListHead[OldPriority],
//                                &Thread->WaitListEntry) :
//                 InsertTailList(&Prcb->DispatcherReadyListHead[OldPriority],
//                                &Thread->WaitListEntry);
//
//     /* Update the ready summary */
//     Prcb->ReadySummary |= PRIORITY_MASK(OldPriority);
//
//     /* Sanity check */
//     ASSERT(OldPriority == Thread->Priority);
//
//     /* Release the lock */
//     KiReleasePrcbLock(Prcb);
// }
//
// PKTHREAD
// FASTCALL
// KiSelectNextThread(IN PKPRCB Prcb)
// {
//     PKTHREAD Thread;
//
//     /* Select a ready thread */
//     Thread = KiSelectReadyThread(0, Prcb);
//     if (!Thread)
//     {
//         /* Didn't find any, get the current idle thread */
//         Thread = Prcb->IdleThread;
//
//         /* Enable idle scheduling */
//         InterlockedOrSetMember(&KiIdleSummary, Prcb->SetMember);
//         Prcb->IdleSchedule = TRUE;
//
//         /* FIXME: SMT support */
//         //ASSERTMSG("SMP: Not yet implemented\n", FALSE);
//     }
//
//     /* Sanity checks and return the thread */
//     ASSERT(Thread != NULL);
//     //ASSERT((Thread->BasePriority == 0) || (Thread->Priority != 0));
//     return Thread;
// }
//
// LONG_PTR
// FASTCALL
// KiSwapThread(IN PKTHREAD CurrentThread,
//              IN PKPRCB Prcb)
// {
//     BOOLEAN ApcState = FALSE;
//     KIRQL WaitIrql;
//     LONG_PTR WaitStatus;
//     PKTHREAD NextThread;
//     ASSERT(KeGetCurrentIrql() >= DISPATCH_LEVEL);
//
//     /* Acquire the PRCB lock */
//     KiAcquirePrcbLock(Prcb);
//
//     /* Get the next thread */
//     NextThread = Prcb->NextThread;
//     if (NextThread)
//     {
// #ifdef _M_ARM64
//         /* ARM64: Without HalRequestSoftwareInterrupt, Prcb->NextThread set by
//          * KiDeferredReadyThread / KiExitDispatcher is never dispatched via
//          * preemption.  A higher-priority thread in the ready list (e.g. one
//          * queued by NtYieldExecution) can be starved.  Re-queue NextThread if
//          * a higher-priority thread is waiting. */
//         if (Prcb->ReadySummary)
//         {
//             ULONG HighBit;
//             BitScanReverse((PULONG)&HighBit, Prcb->ReadySummary);
//             if ((SCHAR)HighBit > NextThread->Priority)
//             {
//                 NextThread->Preempted = FALSE;
//                 KxQueueReadyThread(NextThread, Prcb);
//                 NextThread = KiSelectReadyThread(0, Prcb);
//             }
//         }
// #endif
//         /* Already got a thread, set it up */
//         Prcb->NextThread = NULL;
//         Prcb->CurrentThread = NextThread;
//         NextThread->State = Running;
//     }
//     else
//     {
//         /* Try to find a ready thread */
//         NextThread = KiSelectReadyThread(0, Prcb);
//         if (NextThread)
//         {
//             /* Switch to it */
//             Prcb->CurrentThread = NextThread;
//             NextThread->State = Running;
//         }
//         else
//         {
//             /* Set the idle summary */
//             InterlockedOrSetMember(&KiIdleSummary, Prcb->SetMember);
//
//             /* Schedule the idle thread */
//             NextThread = Prcb->IdleThread;
//             Prcb->CurrentThread = NextThread;
//             NextThread->State = Running;
//         }
//     }
//
//     /* Sanity check and release the PRCB */
//     ASSERT(CurrentThread != Prcb->IdleThread);
//     KiReleasePrcbLock(Prcb);
//
//     /* Save the wait IRQL */
//     WaitIrql = CurrentThread->WaitIrql;
//
//     /* Swap contexts */
//     ApcState = KiSwapContext(WaitIrql, CurrentThread);
//
//     /* Get the wait status */
//     WaitStatus = CurrentThread->WaitStatus;
//
//     /* Check if we need to deliver APCs */
//     if (ApcState)
//     {
//         /* Lower to APC_LEVEL */
//         KeLowerIrql(APC_LEVEL);
//
//         /* Deliver APCs */
//         KiDeliverApc(KernelMode, NULL, NULL);
//         ASSERT(WaitIrql == 0);
//     }
//
//     /* Lower IRQL back to what it was and return the wait status */
//     KeLowerIrql(WaitIrql);
//     return WaitStatus;
// }
//
// VOID
// NTAPI
// KiReadyThread(IN PKTHREAD Thread)
// {
//     IN PKPROCESS Process = Thread->ApcState.Process;
//
//     /* Check if the process is paged out */
//     if (Process->State != ProcessInMemory)
//     {
//         /* We don't page out processes in ROS */
//         ASSERT(FALSE);
//     }
//     else if (!Thread->KernelStackResident)
//     {
//         /* Increase the stack count */
//         ASSERT(Process->StackCount != MAXULONG_PTR);
//         Process->StackCount++;
//
//         /* Set the thread to transition */
//         ASSERT(Thread->State != Transition);
//         Thread->State = Transition;
//
//         /* The stack is always resident in ROS */
//         ASSERT(FALSE);
//     }
//     else
//     {
//         /* Insert the thread on the deferred ready list */
//         KiInsertDeferredReadyList(Thread);
//     }
// }
//
// VOID
// NTAPI
// KiAdjustQuantumThread(IN PKTHREAD Thread)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PKTHREAD NextThread;
//
//     /* Acquire thread and PRCB lock */
//     KiAcquireThreadLock(Thread);
//     KiAcquirePrcbLock(Prcb);
//
//     /* Don't adjust for RT threads */
//     if ((Thread->Priority < LOW_REALTIME_PRIORITY) &&
//         (Thread->BasePriority < (LOW_REALTIME_PRIORITY - 2)))
//     {
//         /* Decrease Quantum by one and see if we've ran out */
//         if (--Thread->Quantum <= 0)
//         {
//             /* Return quantum */
//             Thread->Quantum = Thread->QuantumReset;
//
//             /* Calculate new Priority */
//             Thread->Priority = KiComputeNewPriority(Thread, 1);
//
//             /* Check if there's no next thread scheduled */
//             if (!Prcb->NextThread)
//             {
//                 /* Select a ready thread and check if we found one */
//                 NextThread = KiSelectReadyThread(Thread->Priority, Prcb);
//                 if (NextThread)
//                 {
//                     /* Set it on standby and switch to it */
//                     NextThread->State = Standby;
//                     Prcb->NextThread = NextThread;
//                 }
//             }
//             else
//             {
//                 /* This thread can be preempted again */
//                 Thread->Preempted = FALSE;
//             }
//         }
//     }
//
//     /* Release locks */
//     KiReleasePrcbLock(Prcb);
//     KiReleaseThreadLock(Thread);
//     KiExitDispatcher(Thread->WaitIrql);
// }
//
// VOID
// FASTCALL
// KiSetPriorityThread(IN PKTHREAD Thread,
//                     IN KPRIORITY Priority)
// {
//     PKPRCB Prcb;
//     ULONG Processor;
//     BOOLEAN RequestInterrupt = FALSE;
//     KPRIORITY OldPriority;
//     PKTHREAD NewThread;
//     ASSERT((Priority >= 0) && (Priority <= HIGH_PRIORITY));
//
//     /* Check if priority changed */
//     if (Thread->Priority != Priority)
//     {
//         /* Loop priority setting in case we need to start over */
//         for (;;)
//         {
//             /* Choose action based on thread's state */
//             if (Thread->State == Ready)
//             {
//                 /* Make sure we're not on the ready queue */
//                 if (!Thread->ProcessReadyQueue)
//                 {
//                     /* Get the PRCB for the thread and lock it */
//                     Processor = Thread->NextProcessor;
//                     Prcb = KiProcessorBlock[Processor];
//                     KiAcquirePrcbLock(Prcb);
//
//                     /* Make sure the thread is still ready and on this CPU */
//                     if ((Thread->State == Ready) &&
//                         (Thread->NextProcessor == Prcb->Number))
//                     {
//                         /* Sanity check */
//                         ASSERT((Prcb->ReadySummary &
//                                 PRIORITY_MASK(Thread->Priority)));
//
//                         /* Remove it from the current queue */
//                         if (RemoveEntryList(&Thread->WaitListEntry))
//                         {
//                             /* Update the ready summary */
//                             Prcb->ReadySummary ^= PRIORITY_MASK(Thread->
//                                                                 Priority);
//                         }
//
//                         /* Update priority */
//                         Thread->Priority = (SCHAR)Priority;
//
//                         /* Re-insert it at its current priority */
//                         KiInsertDeferredReadyList(Thread);
//
//                         /* Release the PRCB Lock */
//                         KiReleasePrcbLock(Prcb);
//                     }
//                     else
//                     {
//                         /* Release the lock and loop again */
//                         KiReleasePrcbLock(Prcb);
//                         continue;
//                     }
//                 }
//                 else
//                 {
//                     /* It's already on the ready queue, just update priority */
//                     Thread->Priority = (SCHAR)Priority;
//                 }
//             }
//             else if (Thread->State == Standby)
//             {
//                 /* Get the PRCB for the thread and lock it */
//                 Processor = Thread->NextProcessor;
//                 Prcb = KiProcessorBlock[Processor];
//                 KiAcquirePrcbLock(Prcb);
//
//                 /* Check if we're still the next thread to run */
//                 if (Thread == Prcb->NextThread)
//                 {
//                     /* Get the old priority and update ours */
//                     OldPriority = Thread->Priority;
//                     Thread->Priority = (SCHAR)Priority;
//
//                     /* Check if there was a change */
//                     if (Priority < OldPriority)
//                     {
//                         /* Find a new thread */
//                         NewThread = KiSelectReadyThread(Priority + 1, Prcb);
//                         if (NewThread)
//                         {
//                             /* Found a new one, set it on standby */
//                             NewThread->State = Standby;
//                             Prcb->NextThread = NewThread;
//
//                             /* Dispatch our thread */
//                             KiInsertDeferredReadyList(Thread);
//                         }
//                     }
//
//                     /* Release the PRCB lock */
//                     KiReleasePrcbLock(Prcb);
//                 }
//                 else
//                 {
//                     /* Release the lock and try again */
//                     KiReleasePrcbLock(Prcb);
//                     continue;
//                 }
//             }
//             else if (Thread->State == Running)
//             {
//                 /* Get the PRCB for the thread and lock it */
//                 Processor = Thread->NextProcessor;
//                 Prcb = KiProcessorBlock[Processor];
//                 KiAcquirePrcbLock(Prcb);
//
//                 /* Check if we're still the current thread running */
//                 if (Thread == Prcb->CurrentThread)
//                 {
//                     /* Get the old priority and update ours */
//                     OldPriority = Thread->Priority;
//                     Thread->Priority = (SCHAR)Priority;
//
//                     /* Check if there was a change and there's no new thread */
//                     if ((Priority < OldPriority) && !(Prcb->NextThread))
//                     {
//                         /* Find a new thread */
//                         NewThread = KiSelectReadyThread(Priority + 1, Prcb);
//                         if (NewThread)
//                         {
//                             /* Found a new one, set it on standby */
//                             NewThread->State = Standby;
//                             Prcb->NextThread = NewThread;
//
//                             /* Request an interrupt */
//                             RequestInterrupt = TRUE;
//                         }
//                     }
//
//                     /* Release the lock and check if we need an interrupt */
//                     KiReleasePrcbLock(Prcb);
//                     if (RequestInterrupt)
//                     {
//                         /* Check if we're running on another CPU */
//                         if (KeGetCurrentProcessorNumber() != Processor)
//                         {
//                             /* We are, send an IPI */
//                             KiIpiSend(AFFINITY_MASK(Processor), IPI_DPC);
//                         }
//                     }
//                 }
//                 else
//                 {
//                     /* Thread changed, release lock and restart */
//                     KiReleasePrcbLock(Prcb);
//                     continue;
//                 }
//             }
//             else if (Thread->State == DeferredReady)
//             {
//                 /* FIXME: TODO */
//                 DPRINT1("Deferred state not yet supported\n");
//                 ASSERT(FALSE);
//             }
//             else
//             {
//                 /* Any other state, just change priority */
//                 Thread->Priority = (SCHAR)Priority;
//             }
//
//             /* If we got here, then thread state was consistent, so bail out */
//             break;
//         }
//     }
// }
//
// #ifdef CONFIG_SMP
// static
// VOID
// KiUpdateEffectiveAffinityThread(
//     _In_ PKTHREAD Thread)
// {
//     PKPRCB Prcb;
//
//     /* Acquire the thread lock */
//     KiAcquireThreadLock(Thread);
//
//     /* Get the PRCB that the thread is to be run on and lock it */
//     Prcb = KiProcessorBlock[Thread->NextProcessor];
//     KiAcquirePrcbLock(Prcb);
//
//     /* Set the thread's affinity and ideal processor */
//     Thread->Affinity = Thread->UserAffinity;
//     Thread->IdealProcessor = Thread->UserIdealProcessor;
//
//     /* Check if the affinity doesn't match with the current processor */
//     if ((Prcb->SetMember & Thread->Affinity) == 0)
//     {
//         if (Thread->State == Running)
//         {
//             /* Check if there is the next thread is selected already */
//             if (Prcb->NextThread == NULL)
//             {
//                 /* It is not, select a new thread and set it on standby */
//                 Prcb->NextThread = KiSelectNextThread(Prcb);
//                 Prcb->NextThread->State = Standby;
//             }
//
//             /* Check if the thread is running on a different processor */
//             if (Prcb != KeGetCurrentPrcb())
//             {
//                 /* It is, send an IPI */
//                 KiIpiSend(AFFINITY_MASK(Thread->NextProcessor), IPI_DPC);
//             }
//         }
//         else if (Thread->State == Standby)
//         {
//             /* Select a new thread and set it on standby */
//             Prcb->NextThread = KiSelectNextThread(Prcb);
//             Prcb->NextThread->State = Standby;
//
//             /* Insert the thread back into the ready list */
//             KiInsertDeferredReadyList(Thread);
//         }
//         else if (Thread->State == Ready)
//         {
//             /* Remove it from the list */
//             if (RemoveEntryList(&Thread->WaitListEntry))
//             {
//                 /* The list is empty now, reset the ready summary */
//                 Prcb->ReadySummary &= ~PRIORITY_MASK(Thread->Priority);
//             }
//
//             /* Insert the thread back into the ready list */
//             KiInsertDeferredReadyList(Thread);
//         }
//     }
//
//     KiReleasePrcbLock(Prcb);
//     KiReleaseThreadLock(Thread);
// }
// #endif // CONFIG_SMP
//
// KAFFINITY
// FASTCALL
// KiSetAffinityThread(IN PKTHREAD Thread,
//                     IN KAFFINITY Affinity)
// {
//     KAFFINITY OldAffinity;
//
//     /* Get the current affinity */
//     OldAffinity = Thread->UserAffinity;
//
//     /* Make sure that the affinity is valid */
//     if (((Affinity & Thread->ApcState.Process->Affinity) != (Affinity)) ||
//         (!Affinity))
//     {
//         /* Bugcheck the system */
//         KeBugCheck(INVALID_AFFINITY_SET);
//     }
//
//     /* Update the new affinity */
//     Thread->UserAffinity = Affinity;
//
// #ifdef CONFIG_SMP
//     /* Check if system affinity is not active */
//     if (!Thread->SystemAffinityActive)
//     {
//         /* Calculate the new ideal processor from the affinity set */
//         Thread->UserIdealProcessor =
//             KiFindIdealProcessor(Affinity, Thread->UserIdealProcessor);
//
//         /* Update the effective affinity */
//         KiUpdateEffectiveAffinityThread(Thread);
//     }
// #endif
//
//     /* Return the old affinity */
//     return OldAffinity;
// }
//
// //
// // This macro exists because NtYieldExecution locklessly attempts to read from
// // the KPRCB's ready summary, and the usual way of going through KeGetCurrentPrcb
// // would require getting fs:1C first (or gs), and then doing another dereference.
// // In an attempt to minimize the amount of instructions and potential race/tear
// // that could happen, Windows seems to define this as a macro that directly acceses
// // the ready summary through a single fs: read by going through the KPCR's PrcbData.
// //
// // See http://research.microsoft.com/en-us/collaboration/global/asia-pacific/programs/trk_case4_process-thread_management.pdf (DEAD_LINK)
// //
// // We need this per-arch because sometimes it's Prcb and sometimes PrcbData, and
// // because on x86 it's FS, and on x64 it's GS (not sure what it is on ARM/PPC).
// //
// #ifdef _M_IX86
// #define KiGetCurrentReadySummary() __readfsdword(FIELD_OFFSET(KIPCR, PrcbData.ReadySummary))
// #elif _M_AMD64
// #define KiGetCurrentReadySummary() __readgsdword(FIELD_OFFSET(KIPCR, Prcb.ReadySummary))
// #else
// #define KiGetCurrentReadySummary() KeGetCurrentPrcb()->ReadySummary
// #endif
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtYieldExecution(VOID)
// {
//     NTSTATUS Status;
//     KIRQL OldIrql;
//     PKPRCB Prcb;
//     PKTHREAD Thread, NextThread;
//
//     /* NB: No instructions (other than entry code) should preceed this line */
//
//     /* Fail if there's no ready summary */
//     if (!KiGetCurrentReadySummary()) return STATUS_NO_YIELD_PERFORMED;
//
//     /* Now get the current thread, set the status... */
//     Status = STATUS_NO_YIELD_PERFORMED;
//     Thread = KeGetCurrentThread();
//
//     /* Raise IRQL to synch and get the KPRCB now */
//     OldIrql = KeRaiseIrqlToSynchLevel();
//     Prcb = KeGetCurrentPrcb();
//
//     /* Now check if there's still a ready summary */
//     if (Prcb->ReadySummary)
//     {
//         /* Acquire thread and PRCB lock */
//         KiAcquireThreadLock(Thread);
//         KiAcquirePrcbLock(Prcb);
//
//         /* Find a new thread to run if none was selected */
// #ifdef _M_ARM64
//         /* ARM64: Same preemption bypass fix as in KiSwapThread — ensure
//          * the ready list's highest-priority thread is not starved by a
//          * stale Prcb->NextThread. */
//         if (Prcb->NextThread && Prcb->ReadySummary)
//         {
//             ULONG HighBit;
//             BitScanReverse((PULONG)&HighBit, Prcb->ReadySummary);
//             if ((SCHAR)HighBit > Prcb->NextThread->Priority)
//             {
//                 PKTHREAD Scheduled = Prcb->NextThread;
//                 Scheduled->Preempted = FALSE;
//                 KxQueueReadyThread(Scheduled, Prcb);
//                 Prcb->NextThread = NULL;
//             }
//         }
// #endif
//         if (!Prcb->NextThread) Prcb->NextThread = KiSelectReadyThread(1, Prcb);
//
//         /* Make sure we still have a next thread to schedule */
//         NextThread = Prcb->NextThread;
//         if (NextThread)
//         {
//             /* Reset quantum and recalculate priority */
//             Thread->Quantum = Thread->QuantumReset;
//             Thread->Priority = KiComputeNewPriority(Thread, 1);
//
//             /* Release the thread lock */
//             KiReleaseThreadLock(Thread);
//
//             /* Set context swap busy */
//             KiSetThreadSwapBusy(Thread);
//
//             /* Set the new thread as running */
//             Prcb->NextThread = NULL;
//             Prcb->CurrentThread = NextThread;
//             NextThread->State = Running;
//
//             /* Setup a yield wait and queue the thread */
//             Thread->WaitReason = WrYieldExecution;
//             KxQueueReadyThread(Thread, Prcb);
//
//             /* Make it wait at APC_LEVEL */
//             Thread->WaitIrql = APC_LEVEL;
//
//             /* Sanity check */
//             ASSERT(OldIrql <= DISPATCH_LEVEL);
//
//             /* Swap to new thread */
//             KiSwapContext(APC_LEVEL, Thread);
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Release the PRCB and thread lock */
//             KiReleasePrcbLock(Prcb);
//             KiReleaseThreadLock(Thread);
//         }
//     }
//
//     /* Lower IRQL and return */
//     KeLowerIrql(OldIrql);
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
