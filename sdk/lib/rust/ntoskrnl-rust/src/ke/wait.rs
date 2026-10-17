use super::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::STATUS_WAIT_0;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    _KGATE, _KPRCB, _KWAIT_BLOCK, _LARGE_INTEGER,
    KPROCESSOR_MODE, KPRIORITY, KWAIT_REASON, NTSTATUS, PLARGE_INTEGER,
    PKGUARDED_MUTEX, PKTHREAD, PKMUTANT, PKTIMER, PKWAIT_BLOCK,
    ExGetPreviousMode, HalRequestSoftwareInterrupt, KeBugCheck, KeWaitForGate,
    KiAcquireDispatcherLockAtSynchLevel, KiActivateWaiterQueue, KiAdjustQuantumThread,
    KiCheckAlertability, KiCheckDeferredReadyList, KiDeliverApc, KiReadyThread,
    KiRecalculateDueTime, KiReleaseDispatcherLock, KiReleaseDispatcherLockFromSynchLevel,
    KiSelectReadyThread, KiSetThreadSwapBusy, KiSwapContext, KiSwapThread,
    NtYieldExecution,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn KxRemoveTreeTimer(Timer: PKTIMER);
    fn KxSingleThreadWait(
        Object: PVOID,
        Timeout: PLARGE_INTEGER,
        WaitMode: KPROCESSOR_MODE,
        WaitReason: KWAIT_REASON,
        Alertable: BOOLEAN,
        Swappable: *mut BOOLEAN,
        DueTime: PLARGE_INTEGER,
        Hand: *mut ULONG,
    );
    fn KxDelayThreadWait(
        Interval: PLARGE_INTEGER,
        WaitMode: KPROCESSOR_MODE,
        Alertable: BOOLEAN,
        Swappable: *mut BOOLEAN,
        DueTime: PLARGE_INTEGER,
        Hand: *mut ULONG,
    );
    fn KxMultiThreadWait(
        WaitBlockArray: PKWAIT_BLOCK,
        Objects: *mut PVOID,
        Count: ULONG,
        Timeout: PLARGE_INTEGER,
        WaitType: u32,
        WaitMode: KPROCESSOR_MODE,
        WaitReason: KWAIT_REASON,
        Alertable: BOOLEAN,
        Swappable: *mut BOOLEAN,
        DueTime: PLARGE_INTEGER,
        Hand: *mut ULONG,
    );
    fn KxInsertTimer(Timer: PKTIMER, Hand: ULONG);
    #[cfg(not(feature = "smp"))]
    fn KxQueueReadyThread(Thread: PKTHREAD, Prcb: *mut _KPRCB);
    fn KiAddThreadToWaitList(Thread: PKTHREAD, Swappable: BOOLEAN);
    fn KiSatisfyObjectWait(Object: PKMUTANT, Thread: PKTHREAD);
    fn KiSatisfyMutantWait(Mutant: PKMUTANT, Thread: PKTHREAD);
    fn KiSatisfyNonMutantWait(Object: PKMUTANT);
    fn KiAcquirePrcbLock(Prcb: *mut _KPRCB);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange( Destination: *mut i32, Exchange: i32, Comparand: i32, ) -> i32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedCompareExchange( Destination: *mut i32, Exchange: i32, Comparand: i32, ) -> i32;
}

const TIMER_WAIT_BLOCK: usize = 3;
const THREAD_WAIT_OBJECTS: usize = 3;
const MAXIMUM_WAIT_OBJECTS: usize = 64;
const MUTANT_INCREMENT: i32 = 1;
const MINLONG: i32 = i32::MIN;
const SYNCH_LEVEL: KIRQL = 0x2;

const MutantObject: u8 = 2;
const QueueObject: u8 = 4;
const WaitAny: u32 = 1;
const WrMutex: u32 = 0x0000001D;
const WrGuardedMutex: u32 = 35;
const AdjustUnwait: u8 = 0x01;
const Waiting: u8 = 5;
const Running: u8 = 2;

const GM_LOCK_BIT: i32 = 0x1;
const GM_LOCK_WAITER_WOKEN: i32 = 0x2;
const GM_LOCK_WAITER_INC: i32 = 0x4;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiWaitTest(ObjectPointer: PVOID, Increment: KPRIORITY) {
    let ObjectPointer = opaque_ptr(ObjectPointer);
    KiWaitTestImpl(ObjectPointer, Increment)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiWaitTest(ObjectPointer: PVOID, Increment: KPRIORITY) {
    let ObjectPointer = opaque_ptr(ObjectPointer);
    KiWaitTestImpl(ObjectPointer, Increment)
}

unsafe fn KiWaitTestImpl(ObjectPointer: PVOID, Increment: KPRIORITY) {
    let mut WaitEntry: PLIST_ENTRY;
    let WaitList: PLIST_ENTRY;
    let mut WaitBlock: PKWAIT_BLOCK;
    let mut WaitThread: PKTHREAD;
    let FirstObject: PKMUTANT = ObjectPointer as PKMUTANT;
    let mut WaitStatus: NTSTATUS;

    WaitList = core::ptr::addr_of_mut!((*FirstObject).Header.WaitListHead) as PLIST_ENTRY;
    // waitdep tripwire: prove the waiter list is intact BEFORE consuming it.
    super::waitdep::check_waittest(FirstObject as usize, WaitList as *mut _, true);
    WaitEntry = (*WaitList).Flink;

    while ((*FirstObject).Header.SignalState > 0) && (WaitEntry != WaitList) {
        WaitBlock = ((WaitEntry as usize) - core::mem::offset_of!(_KWAIT_BLOCK, WaitListEntry)) as PKWAIT_BLOCK;
        WaitThread = (*WaitBlock).Thread;
        WaitStatus = STATUS_KERNEL_APC;

        if (*WaitBlock).WaitType as u32 == WaitAny {
            WaitStatus = (*WaitBlock).WaitKey as NTSTATUS;
            KiSatisfyObjectWait(FirstObject, WaitThread);
        }

        KiUnwaitThread(WaitThread, WaitStatus as isize, Increment);
        WaitEntry = (*WaitList).Flink;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiUnlinkThread(Thread: PKTHREAD, WaitStatus: isize) {
    let Thread = opaque_ptr(Thread);
    KiUnlinkThreadImpl(Thread, WaitStatus)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiUnlinkThread(Thread: PKTHREAD, WaitStatus: isize) {
    let Thread = opaque_ptr(Thread);
    KiUnlinkThreadImpl(Thread, WaitStatus)
}

unsafe fn KiUnlinkThreadImpl(Thread: PKTHREAD, WaitStatus: isize) {
    let mut WaitBlock: PKWAIT_BLOCK;
    let Timer: PKTIMER;

    // waitdep tripwire: prove the wait-block chain is intact BEFORE the
    // RemoveEntryList walk below mutates it (field-manual §9.24 0x1E flake).
    super::waitdep::check_unwait(Thread);

    (*Thread).WaitStatus |= WaitStatus as ntkrnl_sys::ddk::LONG_PTR;

    WaitBlock = (*Thread).__bindgen_anon_2.WaitBlockList;
    loop {
        RemoveEntryList(core::ptr::addr_of_mut!((*WaitBlock).WaitListEntry) as PLIST_ENTRY);
        WaitBlock = (*WaitBlock).NextWaitBlock;
        if WaitBlock == (*Thread).__bindgen_anon_2.WaitBlockList {
            break;
        }
    }

    if !(*Thread).__bindgen_anon_3.WaitListEntry.Flink.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_3.WaitListEntry) as PLIST_ENTRY);
    }

    Timer = kt_Timer(Thread) as *mut _;
    if (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted != 0 {
        KxRemoveTreeTimer(Timer);
    }

    if !(*Thread).Queue.is_null() {
        (*(*Thread).Queue).CurrentCount += 1;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiUnwaitThread(Thread: PKTHREAD, WaitStatus: isize, Increment: KPRIORITY) {
    let Thread = opaque_ptr(Thread);
    KiUnwaitThreadImpl(Thread, WaitStatus, Increment)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiUnwaitThread(Thread: PKTHREAD, WaitStatus: isize, Increment: KPRIORITY) {
    let Thread = opaque_ptr(Thread);
    KiUnwaitThreadImpl(Thread, WaitStatus, Increment)
}

unsafe fn KiUnwaitThreadImpl(Thread: PKTHREAD, WaitStatus: isize, Increment: KPRIORITY) {
    // §9.56 tripwire 0x5E — SPURIOUS UNWAIT. Every legal unwait targets a
    // thread whose commit already set State=Waiting BEFORE the dispatcher
    // lock dropped (wait.rs commit order; NT asserts the same). Unwaiting a
    // Running/Ready/Standby/DeferredReady thread is the §9.56 first-illegal
    // event: it re-readies a thread that never parked (or parked and already
    // woke), deferred-parking a RUNNING thread → the preempt requeue then
    // clobbers the deferred SLIST via the WaitListEntry union (0x5D's catch,
    // one level downstream). The WaitStatus param names the waker class:
    // 0x102=timer expiry, 0x100=alert, 0=object satisfy, 0x100(APC)=…
    #[cfg(feature = "smp")]
    {
        if *kt_State(Thread) != Waiting {
            crate::dbg_print!(
                b"[RQ5E] UNWAIT of non-Waiting thr=%p state=%u delivering wstat=%p reason=%u\n\0",
                Thread,
                *kt_State(Thread) as u32,
                WaitStatus as usize as *const u8,
                (*Thread).WaitReason as u32,
            );
            if ntkrnl_sys::inline::ke::RQ_ORACLE_ARMED {
            ntkrnl_sys::ddk::KeBugCheckEx(
                ntkrnl_sys::bugcodes::INVALID_WORK_QUEUE_ITEM,
                Thread as ntkrnl_sys::ddk::ULONG_PTR,
                ((0x5Eu64 << 40)
                    | ((*kt_State(Thread)) as u64) << 8
                    | ((WaitStatus as u64) & 0xFF)) as ntkrnl_sys::ddk::ULONG_PTR,
                WaitStatus as ntkrnl_sys::ddk::ULONG_PTR,
                (*Thread).WaitReason as u64 as ntkrnl_sys::ddk::ULONG_PTR,
            );
            }
        }
    }

    KiUnlinkThread(Thread, WaitStatus);

    (*Thread).__bindgen_anon_1.__bindgen_anon_1.AdjustIncrement = Increment as i8;
    (*Thread).__bindgen_anon_1.__bindgen_anon_1.AdjustReason = AdjustUnwait;

    KiReadyThread(Thread);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiAcquireFastMutex(FastMutex: *mut FAST_MUTEX) {
    let FastMutex = opaque_ptr(FastMutex);
    KiAcquireFastMutexImpl(FastMutex)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiAcquireFastMutex(FastMutex: *mut FAST_MUTEX) {
    let FastMutex = opaque_ptr(FastMutex);
    KiAcquireFastMutexImpl(FastMutex)
}

unsafe fn KiAcquireFastMutexImpl(FastMutex: *mut FAST_MUTEX) {
    (*FastMutex).Contention += 1;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FastMutex).Event) as PVOID,
        _KWAIT_REASON_WrMutex,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    KiAcquireGuardedMutexImpl(GuardedMutex)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    KiAcquireGuardedMutexImpl(GuardedMutex)
}

unsafe fn KiAcquireGuardedMutexImpl(GuardedMutex: PKGUARDED_MUTEX) {
    let mut BitsToRemove: i32;
    let mut BitsToAdd: i32;
    let mut OldValue: i32;
    let mut NewValue: i32;

    (*GuardedMutex).Contention += 1;

    BitsToRemove = GM_LOCK_BIT;
    BitsToAdd = GM_LOCK_WAITER_INC;

    loop {
        OldValue = (*GuardedMutex).Count;

        loop {
            if (OldValue & GM_LOCK_BIT) != 0 {
                NewValue = OldValue ^ BitsToRemove;
                NewValue = InterlockedCompareExchange(
                    core::ptr::addr_of_mut!((*GuardedMutex).Count),
                    NewValue,
                    OldValue,
                );
                if NewValue == OldValue {
                    return;
                }
            } else {
                NewValue = OldValue + BitsToAdd;
                NewValue = InterlockedCompareExchange(
                    core::ptr::addr_of_mut!((*GuardedMutex).Count),
                    NewValue,
                    OldValue,
                );
                if NewValue == OldValue {
                    break;
                }
            }

            OldValue = NewValue;
        }

        // lockdep §4: about to block on this gmutex — if we already hold it,
        // that's a self-deadlock (records, print-free). Keyed on the MUTEX
        // address (matching the acquire tap), not the per-wait gate.
        #[cfg(feature = "lockdep")]
        crate::ke::lockdep::lockdep_check_park(
            ntkrnl_sys::mm::KeGetCurrentThread() as usize,
            GuardedMutex as usize,
            crate::ke::lockdep::MODE_GM,
        );
        KeWaitForGate(
            core::ptr::addr_of_mut!((*GuardedMutex).Gate) as *mut _KGATE,
            _KWAIT_REASON_WrGuardedMutex,
            0i8,
        );

        BitsToRemove = GM_LOCK_BIT | GM_LOCK_WAITER_WOKEN;
        BitsToAdd = GM_LOCK_WAITER_WOKEN;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiExitDispatcher(OldIrql: KIRQL) {
    KiExitDispatcherImpl(OldIrql)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiExitDispatcher(OldIrql: KIRQL) {
    KiExitDispatcherImpl(OldIrql)
}

#[inline(never)]
unsafe fn KiExitDispatcherImpl(OldIrql: KIRQL) {
    let Prcb: *mut _KPRCB = KeGetCurrentPrcb();
    let mut Thread: PKTHREAD;
    let mut NextThread: PKTHREAD;
    let mut PendingApc: BOOLEAN;

    // DBG: KiExitDispatcher entry

    KiCheckDeferredReadyList(Prcb);

    // DBG: after KiCheckDeferredReadyList

    if OldIrql >= DISPATCH_LEVEL {
        // DBG: fast path (OldIrql >= DISPATCH_LEVEL)

        if !(*Prcb).NextThread.is_null() && (*Prcb).DpcRoutineActive == 0 {
            HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
        }

        KeLowerIrql(OldIrql);
        return;
    }

    // DBG: slow path (will context switch)

    if (*Prcb).NextThread.is_null() {
        // DBG: no NextThread, just lower IRQL
        KeLowerIrql(OldIrql);
        return;
    }

    // DBG: about to context switch!

    KiAcquirePrcbLock(Prcb);

    NextThread = (*Prcb).NextThread;
    Thread = (*Prcb).CurrentThread;

    // ARM64-only, mirrors C #ifdef _M_ARM64 (ntoskrnl/ke/thrdschd.c KiSwapThread /
    // this KiExitDispatcher). arm64 has no working HalRequestSoftwareInterrupt, so a
    // Prcb->NextThread set earlier is never dispatched via a DISPATCH-level software
    // interrupt; a higher-priority thread queued afterwards (e.g. by NtYieldExecution)
    // would be starved. Re-enforce NextThread->Priority >= max(ReadySummary) here.
    // NOT per-arch drift: on x86 the software interrupt keeps this invariant for free,
    // so the branch never fires there. Removable once arm64 gains real
    // software-interrupt preemption — see docs/engineering/arm64-interrupts-quest.md.
    #[cfg(target_arch = "aarch64")]
    {
        if (*Prcb).ReadySummary != 0 {
            let mut HighBit: ULONG = 0;
            core::arch::asm!(
                "clz {tmp:w}, {val:w}",
                "mov {out:w}, #31",
                "sub {out:w}, {out:w}, {tmp:w}",
                tmp = out(reg) _,
                val = in(reg) (*Prcb).ReadySummary,
                out = inout(reg) HighBit,
            );
            if (HighBit as i8) > (*NextThread).Priority {
                ntkrnl_sys::inline::ke::KiRequeueStandbyThreadLocked(NextThread, Prcb);
                NextThread = KiSelectReadyThread(0, Prcb);
            }
        }
    }

    KiSetThreadSwapBusy(Thread);

    #[cfg(feature = "smp")]
    ntkrnl_sys::inline::ke::ki_check_promote_not_linked(NextThread, 0x01); // KiExitDispatcher

    (*Prcb).NextThread = core::ptr::null_mut();
    (*Prcb).CurrentThread = NextThread;

    *kt_State(NextThread) = Running;
    #[cfg(feature = "smp")]
    ntkrnl_sys::inline::ke::rq_inflight_unmark(NextThread);

    KxQueueReadyThread(Thread, Prcb);

    *kt_WaitIrql(Thread) = OldIrql;

    // DBG: about to KiSwapContext

    PendingApc = KiSwapContext(OldIrql, Thread);

    // DBG: returned from KiSwapContext

    if PendingApc != 0 {
        KeLowerIrql(APC_LEVEL);
        KiDeliverApc(0i8, core::ptr::null_mut(), core::ptr::null_mut());
    }

    KeLowerIrql(OldIrql);

    // DBG: KiExitDispatcher done
}

#[no_mangle]
pub unsafe extern "system" fn KeIsWaitListEmpty(Object: PVOID) -> BOOLEAN {
    // Implemented upstream (1477b340bec); the stub answered "has waiters" forever.
    // KeMemoryBarrier for a consistent snapshot of the head, then IsListEmpty.
    let Header = opaque_ptr(Object) as *mut ntkrnl_sys::ddk::_DISPATCHER_HEADER;
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    let Head = core::ptr::addr_of_mut!((*Header).WaitListHead) as *mut ntkrnl_sys::ddk::LIST_ENTRY;
    (core::ptr::read_volatile(core::ptr::addr_of!((*Head).Flink)) == Head) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn KeDelayExecutionThread(
    WaitMode: KPROCESSOR_MODE,
    Alertable: BOOLEAN,
    mut Interval: PLARGE_INTEGER,
) -> NTSTATUS {
    let mut Interval = opaque_ptr(Interval);
    // arm64 note: this used to be stubbed to `NtYieldExecution(); STATUS_SUCCESS`
    // ("no HAL timer interrupt yet"). That lore died with the interrupts quest —
    // the vtimer clock tick fires and KiTimerExpiration runs from the idle loop.
    // Worse, the stub was landmine #4's ROOT CAUSE: every kernel/user sleep
    // (netstack's poll delay, user Sleep via NtDelayExecution) became a busy-
    // yield, ready threads yielded to each other forever, idle starved, and
    // idle-driven timer expiration froze the desktop (field manual §9.20).
    {
        let mut Timer: PKTIMER;
        let mut TimerBlock: PKWAIT_BLOCK;
        let Thread: PKTHREAD = PsGetCurrentThread();
        let mut WaitStatus: NTSTATUS;
        let mut Swappable: BOOLEAN = 0;
        let OriginalDueTime: PLARGE_INTEGER = Interval;
        let mut DueTime: _LARGE_INTEGER = core::mem::zeroed();
        let mut NewDueTime: _LARGE_INTEGER = core::mem::zeroed();
        let mut InterruptTime: _LARGE_INTEGER = core::mem::zeroed();
        let mut Hand: ULONG = 0;

        if !Interval.is_null() && (*Interval).QuadPart == 0 && WaitMode != 0i8 {
            if Alertable == 0 && (*Thread).__bindgen_anon_1.ApcState.UserApcPending == 0 {
                return NtYieldExecution();
            }
        }

        Timer = kt_Timer(Thread) as *mut _;
        TimerBlock = (kt_WaitBlock(Thread).as_mut_ptr()).add(TIMER_WAIT_BLOCK);

        if (*Thread).WaitNext == 0 {
            loop {
                *kt_WaitIrql(Thread) = KeRaiseIrqlToSynchLevel();
                KxDelayThreadWait(Interval, WaitMode, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);
                KiAcquireDispatcherLockAtSynchLevel();

                (*Thread).Preempted = 0u8;

                if (*Thread).__bindgen_anon_1.ApcState.KernelApcPending != 0
                    && *kt_SpecialApcDisable(Thread) == 0
                    && *kt_WaitIrql(Thread) < APC_LEVEL
                {
                    KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                    continue;
                }

                WaitStatus = KiCheckAlertability(Thread, Alertable, WaitMode);
                if WaitStatus != STATUS_WAIT_0 {
                    break;
                }

                InterruptTime.QuadPart = KeQueryInterruptTime() as i64;
                if (InterruptTime.QuadPart as u64) >= ((*Timer).DueTime.QuadPart as u64) {
                    if Interval.is_null() || (*Interval).QuadPart == 0 {
                        KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                        return NtYieldExecution();
                    }

                    KiReleaseDispatcherLockFromSynchLevel();
                    KiAdjustQuantumThread(Thread);
                    return STATUS_SUCCESS;
                }

                (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 1;

                if !(*Thread).Queue.is_null() {
                    KiActivateWaiterQueue((*Thread).Queue);
                }

                *kt_State(Thread) = Waiting;

                KiAddThreadToWaitList(Thread, Swappable);

                KiSetThreadSwapBusy(Thread);
                KxInsertTimer(Timer, Hand);
                WaitStatus = KiSwapThread(Thread, KeGetCurrentPrcb()) as NTSTATUS;

                if WaitStatus != STATUS_KERNEL_APC {
                    if WaitStatus == STATUS_TIMEOUT {
                        WaitStatus = STATUS_SUCCESS;
                    }
                    return WaitStatus;
                }

                Interval = KiRecalculateDueTime(
                    OriginalDueTime,
                    core::ptr::addr_of_mut!(DueTime),
                    core::ptr::addr_of_mut!(NewDueTime),
                );
            }

            KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
            return WaitStatus;
        }

        (*Thread).WaitNext = 0u8;
        KxDelayThreadWait(Interval, WaitMode, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);

        loop {
            (*Thread).Preempted = 0u8;

            if (*Thread).__bindgen_anon_1.ApcState.KernelApcPending != 0
                && *kt_SpecialApcDisable(Thread) == 0
                && *kt_WaitIrql(Thread) < APC_LEVEL
            {
                KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
            } else {
                WaitStatus = KiCheckAlertability(Thread, Alertable, WaitMode);
                if WaitStatus != STATUS_WAIT_0 {
                    break;
                }

                InterruptTime.QuadPart = KeQueryInterruptTime() as i64;
                if (InterruptTime.QuadPart as u64) >= ((*Timer).DueTime.QuadPart as u64) {
                    if Interval.is_null() || (*Interval).QuadPart == 0 {
                        KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                        return NtYieldExecution();
                    }

                    KiReleaseDispatcherLockFromSynchLevel();
                    KiAdjustQuantumThread(Thread);
                    return STATUS_SUCCESS;
                }

                (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 1;

                if !(*Thread).Queue.is_null() {
                    KiActivateWaiterQueue((*Thread).Queue);
                }

                *kt_State(Thread) = Waiting;

                KiAddThreadToWaitList(Thread, Swappable);

                KiSetThreadSwapBusy(Thread);
                KxInsertTimer(Timer, Hand);
                WaitStatus = KiSwapThread(Thread, KeGetCurrentPrcb()) as NTSTATUS;

                if WaitStatus != STATUS_KERNEL_APC {
                    if WaitStatus == STATUS_TIMEOUT {
                        WaitStatus = STATUS_SUCCESS;
                    }
                    return WaitStatus;
                }

                Interval = KiRecalculateDueTime(
                    OriginalDueTime,
                    core::ptr::addr_of_mut!(DueTime),
                    core::ptr::addr_of_mut!(NewDueTime),
                );
            }

            *kt_WaitIrql(Thread) = KeRaiseIrqlToSynchLevel();
            KxDelayThreadWait(Interval, WaitMode, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);
            KiAcquireDispatcherLockAtSynchLevel();
        }

        KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
        WaitStatus
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeWaitForSingleObject(
    Object: PVOID,
    WaitReason: KWAIT_REASON,
    WaitMode: KPROCESSOR_MODE,
    Alertable: BOOLEAN,
    mut Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let mut Timeout = opaque_ptr(Timeout);

    let Thread: PKTHREAD = PsGetCurrentThread();
    let mut CurrentObject: PKMUTANT = Object as PKMUTANT;
    let mut WaitBlock: PKWAIT_BLOCK = (kt_WaitBlock(Thread).as_mut_ptr()).add(0);
    let mut TimerBlock: PKWAIT_BLOCK = (kt_WaitBlock(Thread).as_mut_ptr()).add(TIMER_WAIT_BLOCK);
    let mut Timer: PKTIMER = kt_Timer(Thread) as *mut _;
    let mut WaitStatus: NTSTATUS;
    let mut Swappable: BOOLEAN = 0;
    let mut DueTime: _LARGE_INTEGER = core::mem::zeroed();
    let mut NewDueTime: _LARGE_INTEGER = core::mem::zeroed();
    let mut InterruptTime: _LARGE_INTEGER = core::mem::zeroed();
    let OriginalDueTime: PLARGE_INTEGER = Timeout;
    let mut Hand: ULONG = 0;

    if (*Thread).WaitNext == 0 {
        loop {
            let _raise_result = KeRaiseIrqlToSynchLevel();
            *kt_WaitIrql(Thread) = _raise_result;
            KxSingleThreadWait(Object, Timeout, WaitMode, WaitReason, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);
            KiAcquireDispatcherLockAtSynchLevel();

            (*Thread).Preempted = 0u8;

            if (*Thread).__bindgen_anon_1.ApcState.KernelApcPending != 0
                && *kt_SpecialApcDisable(Thread) == 0
                && *kt_WaitIrql(Thread) < APC_LEVEL
            {
                KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                continue;
            }

            if (*CurrentObject).Header.__bindgen_anon_1.__bindgen_anon_1.Type == MutantObject {
                if (*CurrentObject).Header.SignalState > 0
                    || Thread == (*CurrentObject).OwnerThread
                {
                    if (*CurrentObject).Header.SignalState != MINLONG {
                        KiSatisfyMutantWait(CurrentObject, Thread);
                        WaitStatus = (*Thread).WaitStatus as NTSTATUS;
                        KiReleaseDispatcherLockFromSynchLevel();
                        KiAdjustQuantumThread(Thread);
                        return WaitStatus;
                    } else {
                        KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                        ExRaiseStatus(STATUS_MUTANT_LIMIT_EXCEEDED);
                    }
                }
            } else if (*CurrentObject).Header.SignalState > 0 {
                KiSatisfyNonMutantWait(CurrentObject);
                WaitStatus = STATUS_WAIT_0;
                KiReleaseDispatcherLockFromSynchLevel();
                KiAdjustQuantumThread(Thread);
                return WaitStatus;
            }

            WaitStatus = KiCheckAlertability(Thread, Alertable, WaitMode);
            if WaitStatus != STATUS_WAIT_0 {
                break;
            }

            if !Timeout.is_null() {
                InterruptTime.QuadPart = KeQueryInterruptTime() as i64;
                if (InterruptTime.QuadPart as u64) >= ((*Timer).DueTime.QuadPart as u64) {
                    WaitStatus = STATUS_TIMEOUT;
                    KiReleaseDispatcherLockFromSynchLevel();
                    KiAdjustQuantumThread(Thread);
                    return WaitStatus;
                }

                (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 1;
            }

            InsertTailList(
                core::ptr::addr_of_mut!((*CurrentObject).Header.WaitListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*WaitBlock).WaitListEntry) as PLIST_ENTRY,
            );

            if !(*Thread).Queue.is_null() {
                KiActivateWaiterQueue((*Thread).Queue);
            }

            *kt_State(Thread) = Waiting;

            KiAddThreadToWaitList(Thread, Swappable);

            KiSetThreadSwapBusy(Thread);

            if !Timeout.is_null() {
                KxInsertTimer(Timer, Hand);
            } else {
                KiReleaseDispatcherLockFromSynchLevel();
            }

            WaitStatus = KiSwapThread(Thread, KeGetCurrentPrcb()) as NTSTATUS;

            if WaitStatus != STATUS_KERNEL_APC {
                return WaitStatus;
            }

            if !Timeout.is_null() {
                Timeout = KiRecalculateDueTime(
                    OriginalDueTime,
                    core::ptr::addr_of_mut!(DueTime),
                    core::ptr::addr_of_mut!(NewDueTime),
                );
            }
        }

        KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
        return WaitStatus;
    }

    (*Thread).WaitNext = 0u8;
    KxSingleThreadWait(Object, Timeout, WaitMode, WaitReason, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);

    loop {
        (*Thread).Preempted = 0u8;

        if (*Thread).__bindgen_anon_1.ApcState.KernelApcPending != 0
            && *kt_SpecialApcDisable(Thread) == 0
            && *kt_WaitIrql(Thread) < APC_LEVEL
        {
            KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
        } else {
            if (*CurrentObject).Header.__bindgen_anon_1.__bindgen_anon_1.Type == MutantObject {
                if (*CurrentObject).Header.SignalState > 0
                    || Thread == (*CurrentObject).OwnerThread
                {
                    if (*CurrentObject).Header.SignalState != MINLONG {
                        KiSatisfyMutantWait(CurrentObject, Thread);
                        WaitStatus = (*Thread).WaitStatus as NTSTATUS;
                        KiReleaseDispatcherLockFromSynchLevel();
                        KiAdjustQuantumThread(Thread);
                        return WaitStatus;
                    } else {
                        KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                        ExRaiseStatus(STATUS_MUTANT_LIMIT_EXCEEDED);
                    }
                }
            } else if (*CurrentObject).Header.SignalState > 0 {
                KiSatisfyNonMutantWait(CurrentObject);
                WaitStatus = STATUS_WAIT_0;
                KiReleaseDispatcherLockFromSynchLevel();
                KiAdjustQuantumThread(Thread);
                return WaitStatus;
            }

            WaitStatus = KiCheckAlertability(Thread, Alertable, WaitMode);
            if WaitStatus != STATUS_WAIT_0 {
                break;
            }

            if !Timeout.is_null() {
                InterruptTime.QuadPart = KeQueryInterruptTime() as i64;
                if (InterruptTime.QuadPart as u64) >= ((*Timer).DueTime.QuadPart as u64) {
                    WaitStatus = STATUS_TIMEOUT;
                    KiReleaseDispatcherLockFromSynchLevel();
                    KiAdjustQuantumThread(Thread);
                    return WaitStatus;
                }

                (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 1;
            }

            InsertTailList(
                core::ptr::addr_of_mut!((*CurrentObject).Header.WaitListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*WaitBlock).WaitListEntry) as PLIST_ENTRY,
            );

            if !(*Thread).Queue.is_null() {
                KiActivateWaiterQueue((*Thread).Queue);
            }

            *kt_State(Thread) = Waiting;

            KiAddThreadToWaitList(Thread, Swappable);

            KiSetThreadSwapBusy(Thread);

            if !Timeout.is_null() {
                KxInsertTimer(Timer, Hand);
            } else {
                KiReleaseDispatcherLockFromSynchLevel();
            }

            WaitStatus = KiSwapThread(Thread, KeGetCurrentPrcb()) as NTSTATUS;

            if WaitStatus != STATUS_KERNEL_APC {
                return WaitStatus;
            }

            if !Timeout.is_null() {
                Timeout = KiRecalculateDueTime(
                    OriginalDueTime,
                    core::ptr::addr_of_mut!(DueTime),
                    core::ptr::addr_of_mut!(NewDueTime),
                );
            }
        }

        *kt_WaitIrql(Thread) = KeRaiseIrqlToSynchLevel();
        KxSingleThreadWait(Object, Timeout, WaitMode, WaitReason, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);
        KiAcquireDispatcherLockAtSynchLevel();
    }

    KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
    WaitStatus
}

// The CPS RunToken wait (`wait_for`) lives in `ntkrnl_sys::token` — NOT here —
// so extracted kernel crates (po-rust, …) can use it without a dependency on
// ntoskrnl-rust; it calls the `KeWaitForSingleObject` export above by link
// name with a primitive-width ABI-identical signature.

#[no_mangle]
pub unsafe extern "system" fn KeWaitForMultipleObjects(
    Count: ULONG,
    Object: *mut PVOID,
    WaitType: u32,
    WaitReason: KWAIT_REASON,
    WaitMode: KPROCESSOR_MODE,
    Alertable: BOOLEAN,
    mut Timeout: PLARGE_INTEGER,
    WaitBlockArray: PKWAIT_BLOCK,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let mut Timeout = opaque_ptr(Timeout);
    let mut CurrentObject: PKMUTANT = core::ptr::null_mut();
    let mut WaitBlock: PKWAIT_BLOCK = core::ptr::null_mut();
    let Thread: PKTHREAD = PsGetCurrentThread();
    let mut TimerBlock: PKWAIT_BLOCK = (kt_WaitBlock(Thread).as_mut_ptr()).add(TIMER_WAIT_BLOCK);
    let mut Timer: PKTIMER = kt_Timer(Thread) as *mut _;
    let mut WaitStatus: NTSTATUS = STATUS_SUCCESS;
    let mut Swappable: BOOLEAN = 0;
    let OriginalDueTime: PLARGE_INTEGER = Timeout;
    let mut DueTime: _LARGE_INTEGER = core::mem::zeroed();
    let mut NewDueTime: _LARGE_INTEGER = core::mem::zeroed();
    let mut InterruptTime: _LARGE_INTEGER = core::mem::zeroed();
    let mut Index: ULONG;
    let mut Hand: ULONG = 0;

    let WaitBlockArray = if WaitBlockArray.is_null() {
        if Count as usize > THREAD_WAIT_OBJECTS {
            KeBugCheck(ntkrnl_sys::bugcodes::MAXIMUM_WAIT_OBJECTS_EXCEEDED);
        }
        kt_WaitBlock(Thread).as_mut_ptr()
    } else {
        if Count as usize > MAXIMUM_WAIT_OBJECTS {
            KeBugCheck(ntkrnl_sys::bugcodes::MAXIMUM_WAIT_OBJECTS_EXCEEDED);
        }
        WaitBlockArray
    };

    if (*Thread).WaitNext == 0 {
        loop {
            *kt_WaitIrql(Thread) = KeRaiseIrqlToSynchLevel();
            KxMultiThreadWait(WaitBlockArray, Object, Count, Timeout, WaitType, WaitMode, WaitReason, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);
            // KxMultiThreadWait is a C macro that sets WaitBlock to the last entry;
            // the shim function can't modify our local, so set it here.
            WaitBlock = WaitBlockArray.add(Count as usize - 1);
            KiAcquireDispatcherLockAtSynchLevel();

            (*Thread).Preempted = 0u8;

            if (*Thread).__bindgen_anon_1.ApcState.KernelApcPending != 0
                && *kt_SpecialApcDisable(Thread) == 0
                && *kt_WaitIrql(Thread) < APC_LEVEL
            {
                KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                continue;
            }

            Index = 0;
            if WaitType == WaitAny {
                while Index < Count {
                    CurrentObject = *Object.add(Index as usize) as PKMUTANT;

                    if (*CurrentObject).Header.__bindgen_anon_1.__bindgen_anon_1.Type == MutantObject {
                        if (*CurrentObject).Header.SignalState > 0
                            || Thread == (*CurrentObject).OwnerThread
                        {
                            if (*CurrentObject).Header.SignalState != MINLONG {
                                KiSatisfyMutantWait(CurrentObject, Thread);
                                WaitStatus = ((*Thread).WaitStatus as i32 | Index as i32) as NTSTATUS;
                                KiReleaseDispatcherLockFromSynchLevel();
                                KiAdjustQuantumThread(Thread);
                                return WaitStatus;
                            } else {
                                KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                                ExRaiseStatus(STATUS_MUTANT_LIMIT_EXCEEDED);
                            }
                        }
                    } else if (*CurrentObject).Header.SignalState > 0 {
                        KiSatisfyNonMutantWait(CurrentObject);
                        WaitStatus = Index as NTSTATUS;
                        KiReleaseDispatcherLockFromSynchLevel();
                        KiAdjustQuantumThread(Thread);
                        return WaitStatus;
                    }

                    Index += 1;
                }
            } else {
                while Index < Count {
                    CurrentObject = *Object.add(Index as usize) as PKMUTANT;

                    if (*CurrentObject).Header.__bindgen_anon_1.__bindgen_anon_1.Type == MutantObject {
                        if Thread == (*CurrentObject).OwnerThread
                            && (*CurrentObject).Header.SignalState == MINLONG
                        {
                            KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                            ExRaiseStatus(STATUS_MUTANT_LIMIT_EXCEEDED);
                        } else if (*CurrentObject).Header.SignalState <= 0
                            && Thread != (*CurrentObject).OwnerThread
                        {
                            break;
                        }
                    } else if (*CurrentObject).Header.SignalState <= 0 {
                        break;
                    }

                    Index += 1;
                }

                if Index == Count {
                    WaitBlock = WaitBlockArray;
                    loop {
                        CurrentObject = (*WaitBlock).Object as PKMUTANT;
                        KiSatisfyObjectWait(CurrentObject, Thread);

                        WaitBlock = (*WaitBlock).NextWaitBlock;
                        if WaitBlock == WaitBlockArray {
                            break;
                        }
                    }

                    WaitStatus = (*Thread).WaitStatus as NTSTATUS;
                    KiReleaseDispatcherLockFromSynchLevel();
                    KiAdjustQuantumThread(Thread);
                    return WaitStatus;
                }
            }

            WaitStatus = KiCheckAlertability(Thread, Alertable, WaitMode);
            if WaitStatus != STATUS_WAIT_0 {
                break;
            }

            if !Timeout.is_null() {
                InterruptTime.QuadPart = KeQueryInterruptTime() as i64;
                if (InterruptTime.QuadPart as u64) >= ((*Timer).DueTime.QuadPart as u64) {
                    WaitStatus = STATUS_TIMEOUT;
                    KiReleaseDispatcherLockFromSynchLevel();
                    KiAdjustQuantumThread(Thread);
                    return WaitStatus;
                }

                (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 1;

                (*WaitBlock).NextWaitBlock = TimerBlock;
            }

            WaitBlock = WaitBlockArray;
            loop {
                CurrentObject = (*WaitBlock).Object as PKMUTANT;

                InsertTailList(
                    core::ptr::addr_of_mut!((*CurrentObject).Header.WaitListHead) as PLIST_ENTRY,
                    core::ptr::addr_of_mut!((*WaitBlock).WaitListEntry) as PLIST_ENTRY,
                );

                WaitBlock = (*WaitBlock).NextWaitBlock;
                if WaitBlock == WaitBlockArray {
                    break;
                }
            }

            if !(*Thread).Queue.is_null() {
                KiActivateWaiterQueue((*Thread).Queue);
            }

            *kt_State(Thread) = Waiting;

            KiAddThreadToWaitList(Thread, Swappable);

            KiSetThreadSwapBusy(Thread);

            if !Timeout.is_null() {
                KxInsertTimer(Timer, Hand);
            } else {
                KiReleaseDispatcherLockFromSynchLevel();
            }

            WaitStatus = KiSwapThread(Thread, KeGetCurrentPrcb()) as NTSTATUS;

            if WaitStatus != STATUS_KERNEL_APC {
                return WaitStatus;
            }

            if !Timeout.is_null() {
                Timeout = KiRecalculateDueTime(
                    OriginalDueTime,
                    core::ptr::addr_of_mut!(DueTime),
                    core::ptr::addr_of_mut!(NewDueTime),
                );
            }
        }

        KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
        return WaitStatus;
    }

    (*Thread).WaitNext = 0u8;
    KxMultiThreadWait(WaitBlockArray, Object, Count, Timeout, WaitType, WaitMode, WaitReason, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);
    WaitBlock = WaitBlockArray.add(Count as usize - 1);

    loop {
        (*Thread).Preempted = 0u8;

        if (*Thread).__bindgen_anon_1.ApcState.KernelApcPending != 0
            && *kt_SpecialApcDisable(Thread) == 0
            && *kt_WaitIrql(Thread) < APC_LEVEL
        {
            KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
        } else {
            Index = 0;
            if WaitType == WaitAny {
                while Index < Count {
                    CurrentObject = *Object.add(Index as usize) as PKMUTANT;

                    if (*CurrentObject).Header.__bindgen_anon_1.__bindgen_anon_1.Type == MutantObject {
                        if (*CurrentObject).Header.SignalState > 0
                            || Thread == (*CurrentObject).OwnerThread
                        {
                            if (*CurrentObject).Header.SignalState != MINLONG {
                                KiSatisfyMutantWait(CurrentObject, Thread);
                                WaitStatus = ((*Thread).WaitStatus as i32 | Index as i32) as NTSTATUS;
                                KiReleaseDispatcherLockFromSynchLevel();
                                KiAdjustQuantumThread(Thread);
                                return WaitStatus;
                            } else {
                                KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                                ExRaiseStatus(STATUS_MUTANT_LIMIT_EXCEEDED);
                            }
                        }
                    } else if (*CurrentObject).Header.SignalState > 0 {
                        KiSatisfyNonMutantWait(CurrentObject);
                        WaitStatus = Index as NTSTATUS;
                        KiReleaseDispatcherLockFromSynchLevel();
                        KiAdjustQuantumThread(Thread);
                        return WaitStatus;
                    }

                    Index += 1;
                }
            } else {
                while Index < Count {
                    CurrentObject = *Object.add(Index as usize) as PKMUTANT;

                    if (*CurrentObject).Header.__bindgen_anon_1.__bindgen_anon_1.Type == MutantObject {
                        if Thread == (*CurrentObject).OwnerThread
                            && (*CurrentObject).Header.SignalState == MINLONG
                        {
                            KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
                            ExRaiseStatus(STATUS_MUTANT_LIMIT_EXCEEDED);
                        } else if (*CurrentObject).Header.SignalState <= 0
                            && Thread != (*CurrentObject).OwnerThread
                        {
                            break;
                        }
                    } else if (*CurrentObject).Header.SignalState <= 0 {
                        break;
                    }

                    Index += 1;
                }

                if Index == Count {
                    WaitBlock = WaitBlockArray;
                    loop {
                        CurrentObject = (*WaitBlock).Object as PKMUTANT;
                        KiSatisfyObjectWait(CurrentObject, Thread);

                        WaitBlock = (*WaitBlock).NextWaitBlock;
                        if WaitBlock == WaitBlockArray {
                            break;
                        }
                    }

                    WaitStatus = (*Thread).WaitStatus as NTSTATUS;
                    KiReleaseDispatcherLockFromSynchLevel();
                    KiAdjustQuantumThread(Thread);
                    return WaitStatus;
                }
            }

            WaitStatus = KiCheckAlertability(Thread, Alertable, WaitMode);
            if WaitStatus != STATUS_WAIT_0 {
                break;
            }

            if !Timeout.is_null() {
                InterruptTime.QuadPart = KeQueryInterruptTime() as i64;
                if (InterruptTime.QuadPart as u64) >= ((*Timer).DueTime.QuadPart as u64) {
                    WaitStatus = STATUS_TIMEOUT;
                    KiReleaseDispatcherLockFromSynchLevel();
                    KiAdjustQuantumThread(Thread);
                    return WaitStatus;
                }

                (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 1;

                (*WaitBlock).NextWaitBlock = TimerBlock;
            }

            WaitBlock = WaitBlockArray;
            loop {
                CurrentObject = (*WaitBlock).Object as PKMUTANT;

                InsertTailList(
                    core::ptr::addr_of_mut!((*CurrentObject).Header.WaitListHead) as PLIST_ENTRY,
                    core::ptr::addr_of_mut!((*WaitBlock).WaitListEntry) as PLIST_ENTRY,
                );

                WaitBlock = (*WaitBlock).NextWaitBlock;
                if WaitBlock == WaitBlockArray {
                    break;
                }
            }

            if !(*Thread).Queue.is_null() {
                KiActivateWaiterQueue((*Thread).Queue);
            }

            *kt_State(Thread) = Waiting;

            KiAddThreadToWaitList(Thread, Swappable);

            KiSetThreadSwapBusy(Thread);

            if !Timeout.is_null() {
                KxInsertTimer(Timer, Hand);
            } else {
                KiReleaseDispatcherLockFromSynchLevel();
            }

            WaitStatus = KiSwapThread(Thread, KeGetCurrentPrcb()) as NTSTATUS;

            if WaitStatus != STATUS_KERNEL_APC {
                return WaitStatus;
            }

            if !Timeout.is_null() {
                Timeout = KiRecalculateDueTime(
                    OriginalDueTime,
                    core::ptr::addr_of_mut!(DueTime),
                    core::ptr::addr_of_mut!(NewDueTime),
                );
            }
        }

        *kt_WaitIrql(Thread) = KeRaiseIrqlToSynchLevel();
        KxMultiThreadWait(WaitBlockArray, Object, Count, Timeout, WaitType, WaitMode, WaitReason, Alertable, &mut Swappable, &mut DueTime as *mut _ as PLARGE_INTEGER, &mut Hand);
        WaitBlock = WaitBlockArray.add(Count as usize - 1);
        KiAcquireDispatcherLockAtSynchLevel();
    }

    KiReleaseDispatcherLock(*kt_WaitIrql(Thread));
    WaitStatus
}

#[no_mangle]
pub unsafe extern "system" fn NtDelayExecution(
    Alertable: BOOLEAN,
    DelayInterval: PLARGE_INTEGER,
) -> NTSTATUS {
    let DelayInterval = opaque_ptr(DelayInterval);
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut SafeInterval: _LARGE_INTEGER = core::mem::zeroed();

    if PreviousMode != 0i8 {
        let result = (|| -> Result<(), NTSTATUS> {
            SafeInterval = probe_read_user::<_LARGE_INTEGER>(DelayInterval, PreviousMode)?;
            Ok(())
        })();
        if let Err(e) = result {
            return e;
        }
        let DelayInterval = core::ptr::addr_of_mut!(SafeInterval);
        let Status = KeDelayExecutionThread(PreviousMode, Alertable, DelayInterval);
        return Status;
    }

    let Status = KeDelayExecutionThread(PreviousMode, Alertable, DelayInterval);
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/wait.c
//  * PURPOSE:         Manages waiting for Dispatcher Objects
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Gunnar Dalsnes
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// FASTCALL
// KiWaitTest(IN PVOID ObjectPointer,
