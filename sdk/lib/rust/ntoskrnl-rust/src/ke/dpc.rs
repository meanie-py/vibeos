use super::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    _KDPC, _KTIMER, _KPRCB, _KPROCESS, _KTIMER_TABLE_ENTRY,
    PKDPC, PKTIMER, PKPRCB, PKEVENT, PKTHREAD,
    PKDPC_DATA, PKSPIN_LOCK_QUEUE, PLARGE_INTEGER,
    KOBJECTS, KDPC_IMPORTANCE, ULARGE_INTEGER, FAST_MUTEX, KDPC,
    KAFFINITY,
    HalRequestSoftwareInterrupt, KeDisableInterrupts, KeRestoreInterrupts,
    KeSetSystemAffinityThread, KeRevertToUserAffinityThread,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// Use actual definitions, not extern declarations (lto=false splits them)
use super::krnlinit::KiProcessorBlock;
use super::processor::KeNumberProcessors;
use super::thrdschd::KiIdleSummary;
use super::clock::KeTickCount;
extern "system" {
    fn _enable();
    fn _disable();
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiSwapContext(WaitIrql: KIRQL, Thread: PKTHREAD);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiSwapContext(WaitIrql: KIRQL, Thread: PKTHREAD);
}
extern "C" {
    fn KiAcquireDispatcherLock() -> KIRQL;
    fn KiReleaseDispatcherLock(OldIrql: KIRQL);
    fn KiAcquireTimerLock(Index: ULONG) -> PKSPIN_LOCK_QUEUE;
    fn KiReleaseTimerLock(LockQueue: PKSPIN_LOCK_QUEUE);
    fn KiRemoveEntryTimer(Timer: PKTIMER);
    fn KxUnwaitThread(Object: PVOID, Increment: LONG);
    fn KxUnwaitThreadForEvent(Event: PKEVENT, Increment: LONG);
    fn KiAcquirePrcbLock(Prcb: PKPRCB);
    fn KiReleasePrcbLock(Prcb: PKPRCB);
    fn KiAcquireThreadLock(Thread: PKTHREAD);
    fn KiReleaseThreadLock(Thread: PKTHREAD);
    fn KiComputeNewPriority(Thread: PKTHREAD, Adjustment: i8) -> i8;
    fn KiSelectReadyThread(Priority: LONG, Prcb: PKPRCB) -> PKTHREAD;
    fn KiSetThreadSwapBusy(Thread: PKTHREAD);
    #[cfg(not(feature = "smp"))]
    fn KxQueueReadyThread(Thread: PKTHREAD, Prcb: PKPRCB);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiInsertTreeTimer(Timer: PKTIMER, Interval: LARGE_INTEGER) -> BOOLEAN;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiInsertTreeTimer(Timer: PKTIMER, Interval: LARGE_INTEGER) -> BOOLEAN;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiProcessDeferredReadyList(Prcb: PKPRCB);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiProcessDeferredReadyList(Prcb: PKPRCB);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiAcquireSpinLock(SpinLock: PKSPIN_LOCK);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiAcquireSpinLock(SpinLock: PKSPIN_LOCK);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiReleaseSpinLock(SpinLock: PKSPIN_LOCK);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiReleaseSpinLock(SpinLock: PKSPIN_LOCK);
}
// ipi.rs defines KiIpiSend FASTCALL on x86 / cdecl elsewhere (§9.30 cc-match rule)
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiIpiSend(TargetSet: ULONG_PTR, IpiRequest: ULONG);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiIpiSend(TargetSet: ULONG_PTR, IpiRequest: ULONG);
}
extern "system" {
    fn KeQuerySystemTime(CurrentTime: PLARGE_INTEGER);
}

use super::timerobj::KiTimerTableListHead;

const TIMER_TABLE_SIZE: ULONG = 512;
const MAX_TIMER_DPCS: usize = 16;
const IO_NO_INCREMENT: LONG = 0;
const MAX_QUANTUM: LONG = 0x7f;
const LOW_REALTIME_PRIORITY: LONG = 16;
const DPC_NORMAL: usize = 0;
const DPC_THREADED: usize = 1;
const IPI_DPC: ULONG = 0x2;

const TimerNotificationObject: KOBJECTS = 8;
const ThreadedDpcObject: KOBJECTS = 0x18;
const DpcObject: KOBJECTS = 19;

const HighImportance: KDPC_IMPORTANCE = 2;
const MediumImportance: KDPC_IMPORTANCE = 1;
const LowImportance: KDPC_IMPORTANCE = 0;

const WrQuantumEnd: u8 = 0x0000001E;
const Standby: u8 = 3;
const Running: u8 = 2;

/// DPC routine type — matches bindgen's PKDEFERRED_ROUTINE (Option<fn>) per-arch.
type DPC_ROUTINE_OPTION = ntkrnl_sys::ddk::PKDEFERRED_ROUTINE;

#[repr(C)]
struct DPC_QUEUE_ENTRY {
    Dpc: PKDPC,
    Routine: DPC_ROUTINE_OPTION,
    Context: PVOID,
}

#[repr(C)]
struct DEFERRED_REVERSE_BARRIER {
    Barrier: ULONG,
    TotalProcessors: ULONG,
}

#[no_mangle]
pub static mut KiMaximumDpcQueueDepth: ULONG = 4;
#[no_mangle]
pub static mut KiMinimumDpcRate: ULONG = 3;
#[no_mangle]
pub static mut KiAdjustDpcThreshold: ULONG = 20;
#[no_mangle]
pub static mut KiIdealDpcRate: ULONG = 20;
#[no_mangle]
pub static mut KeThreadDpcEnable: BOOLEAN = 0;
#[no_mangle]
pub static mut KiGenericCallDpcMutex: FAST_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KiTimerExpireDpc: KDPC = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KiTimeLimitIsrMicroseconds: ULONG = 0;
#[no_mangle]
pub static mut KiDPCTimeout: ULONG = 110;

unsafe fn KiCheckTimerTable(CurrentTime: ULARGE_INTEGER) {
}

#[no_mangle]
pub unsafe extern "system" fn KiTimerExpiration(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    let mut SystemTime: ULARGE_INTEGER = core::mem::zeroed();
    let mut InterruptTime: ULARGE_INTEGER = core::mem::zeroed();
    let mut Interval: LARGE_INTEGER = core::mem::zeroed();
    let mut Limit: LONG;
    let mut Index: LONG;
    let mut i: LONG;
    let mut Timers: ULONG;
    let mut ActiveTimers: ULONG;
    let mut DpcCalls: ULONG;
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let OldIrql: KIRQL;
    let mut Timer: PKTIMER;
    let mut TimerDpc: PKDPC;
    let mut Period: ULONG;
    let mut DpcEntry: [DPC_QUEUE_ENTRY; MAX_TIMER_DPCS] = core::mem::zeroed();
    let mut LockQueue: PKSPIN_LOCK_QUEUE;
    let Prcb: PKPRCB = KeGetCurrentPrcb();

    _disable();

    KeQuerySystemTime(&mut SystemTime as *mut ULARGE_INTEGER as PLARGE_INTEGER);

    InterruptTime.QuadPart = KeQueryInterruptTime();

    Limit = (core::ptr::addr_of!(KeTickCount) as *const LARGE_INTEGER).cast::<LONG>().read();

    _enable();

    Index = SystemArgument1 as LONG;
    if (Limit - Index) >= TIMER_TABLE_SIZE as LONG {
        Limit = Index + TIMER_TABLE_SIZE as LONG - 1;
    }

    Index -= 1;
    Limit &= (TIMER_TABLE_SIZE - 1) as LONG;

    DpcCalls = 0;
    Timers = 24;
    ActiveTimers = 4;

    OldIrql = KiAcquireDispatcherLock();

    loop {
        Index = (Index + 1) & ((TIMER_TABLE_SIZE - 1) as LONG);

        ListHead = core::ptr::addr_of_mut!(KiTimerTableListHead[Index as usize].Entry) as PLIST_ENTRY;

        while ListHead != (*ListHead).Flink {

            LockQueue = KiAcquireTimerLock(Index as ULONG);
            NextEntry = (*ListHead).Flink;

            Timers -= 1;
            Timer = ((NextEntry as usize) - core::mem::offset_of!(_KTIMER, TimerListEntry)) as PKTIMER;

            let next_ne_head = NextEntry != ListHead;

            let timer_expired = if next_ne_head {
                (*Timer).DueTime.QuadPart <= InterruptTime.QuadPart
            } else {
                false
            };

            if timer_expired {
                ActiveTimers -= 1;
                KiRemoveEntryTimer(Timer);

                (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 0;
                KiReleaseTimerLock(LockQueue);
                (*Timer).Header.SignalState = 1;

                TimerDpc = (*Timer).Dpc;
                Period = (*Timer).Period;

                if !IsListEmpty(&(*Timer).Header.WaitListHead as *const LIST_ENTRY as PLIST_ENTRY) {
                    if (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.Type == TimerNotificationObject as u8 {
                        KxUnwaitThread(&(*Timer).Header as *const _ as PVOID, IO_NO_INCREMENT);
                    } else {
                        KxUnwaitThreadForEvent(Timer as PKEVENT, IO_NO_INCREMENT);
                    }
                }

                if Period != 0 {
                    Interval.QuadPart = (Period as i64) * -10000i64;
                    while KiInsertTreeTimer(Timer, Interval) == 0 {}
                }

                if !TimerDpc.is_null() {
                    DpcEntry[DpcCalls as usize].Dpc = TimerDpc;
                    DpcEntry[DpcCalls as usize].Routine = (*TimerDpc).DeferredRoutine;
                    DpcEntry[DpcCalls as usize].Context = (*TimerDpc).DeferredContext;
                    DpcCalls += 1;
                }

                if (ActiveTimers == 0) || (Timers == 0) {
                    KiReleaseDispatcherLock(DISPATCH_LEVEL);

                    i = 0;
                    while DpcCalls > 0 {
                        (DpcEntry[i as usize].Routine.unwrap())(
                            DpcEntry[i as usize].Dpc,
                            DpcEntry[i as usize].Context,
                            SystemTime.__bindgen_anon_1.LowPart as PVOID,
                            SystemTime.__bindgen_anon_1.HighPart as PVOID,
                        );
                        DpcCalls -= 1;
                        i += 1;
                    }

                    Timers = 24;
                    ActiveTimers = 4;

                    KiAcquireDispatcherLock();
                }
            } else {

                if NextEntry != ListHead {
                    _disable();
                    KiTimerTableListHead[Index as usize].Time.QuadPart = (*Timer).DueTime.QuadPart;
                    _enable();
                }

                KiReleaseTimerLock(LockQueue);

                if Timers == 0 {
                    KiReleaseDispatcherLock(DISPATCH_LEVEL);

                    i = 0;
                    while DpcCalls > 0 {
                        (DpcEntry[i as usize].Routine.unwrap())(
                            DpcEntry[i as usize].Dpc,
                            DpcEntry[i as usize].Context,
                            SystemTime.__bindgen_anon_1.LowPart as PVOID,
                            SystemTime.__bindgen_anon_1.HighPart as PVOID,
                        );
                        DpcCalls -= 1;
                        i += 1;
                    }

                    Timers = 24;
                    ActiveTimers = 4;

                    KiAcquireDispatcherLock();
                }

                break;
            }
        }

        if Index == Limit {
            break;
        }
    }

    if KeNumberProcessors == 1 {
        KiCheckTimerTable(InterruptTime);
    }

    if DpcCalls != 0 {

        KiReleaseDispatcherLock(DISPATCH_LEVEL);

        i = 0;
        while DpcCalls > 0 {
            (DpcEntry[i as usize].Routine.unwrap())(
                DpcEntry[i as usize].Dpc,
                DpcEntry[i as usize].Context,
                SystemTime.__bindgen_anon_1.LowPart as PVOID,
                SystemTime.__bindgen_anon_1.HighPart as PVOID,
            );
            DpcCalls -= 1;
            i += 1;
        }

        if OldIrql != DISPATCH_LEVEL {
            KeLowerIrql(OldIrql);
        }
    } else {
        KiReleaseDispatcherLock(OldIrql);
    }

}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiTimerListExpire(
    ExpiredListHead: PLIST_ENTRY,
    OldIrql: KIRQL,
) {
    let ExpiredListHead = opaque_ptr(ExpiredListHead);
    KiTimerListExpire_body(ExpiredListHead, OldIrql)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiTimerListExpire(
    ExpiredListHead: PLIST_ENTRY,
    OldIrql: KIRQL,
) {
    let ExpiredListHead = opaque_ptr(ExpiredListHead);
    KiTimerListExpire_body(ExpiredListHead, OldIrql)
}

unsafe fn KiTimerListExpire_body(ExpiredListHead: PLIST_ENTRY, OldIrql: KIRQL) {
    let mut SystemTime: ULARGE_INTEGER = core::mem::zeroed();
    let mut Interval: LARGE_INTEGER = core::mem::zeroed();
    let mut i: LONG;
    let mut DpcCalls: ULONG = 0;
    let mut Timer: PKTIMER;
    let mut TimerDpc: PKDPC;
    let mut Period: ULONG;
    let mut DpcEntry: [DPC_QUEUE_ENTRY; MAX_TIMER_DPCS] = core::mem::zeroed();
    let Prcb: PKPRCB = KeGetCurrentPrcb();

    KeQuerySystemTime(&mut SystemTime as *mut ULARGE_INTEGER as PLARGE_INTEGER);

    while (*ExpiredListHead).Flink != ExpiredListHead {
        Timer = (((*ExpiredListHead).Flink as usize) - core::mem::offset_of!(_KTIMER, TimerListEntry)) as PKTIMER;

        RemoveEntryList(&mut (*Timer).TimerListEntry as *mut LIST_ENTRY);

        (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 0;
        (*Timer).Header.SignalState = 1;

        TimerDpc = (*Timer).Dpc;
        Period = (*Timer).Period;

        if !IsListEmpty(&(*Timer).Header.WaitListHead as *const LIST_ENTRY as PLIST_ENTRY) {
            if (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.Type == TimerNotificationObject as u8 {
                KxUnwaitThread(&(*Timer).Header as *const _ as PVOID, IO_NO_INCREMENT);
            } else {
                KxUnwaitThreadForEvent(Timer as PKEVENT, IO_NO_INCREMENT);
            }
        }

        if Period != 0 {
            Interval.QuadPart = (Period as i64) * -10000i64;
            while KiInsertTreeTimer(Timer, Interval) == 0 {}
        }

        if !TimerDpc.is_null() {
            DpcEntry[DpcCalls as usize].Dpc = TimerDpc;
            DpcEntry[DpcCalls as usize].Routine = (*TimerDpc).DeferredRoutine;
            DpcEntry[DpcCalls as usize].Context = (*TimerDpc).DeferredContext;
            DpcCalls += 1;
        }
    }

    if DpcCalls != 0 {
        KiReleaseDispatcherLock(DISPATCH_LEVEL);

        i = 0;
        while DpcCalls > 0 {
            (DpcEntry[i as usize].Routine.unwrap())(
                DpcEntry[i as usize].Dpc,
                DpcEntry[i as usize].Context,
                SystemTime.__bindgen_anon_1.LowPart as PVOID,
                SystemTime.__bindgen_anon_1.HighPart as PVOID,
            );
            DpcCalls -= 1;
            i += 1;
        }

        KeLowerIrql(OldIrql);
    } else {
        KiReleaseDispatcherLock(OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KiQuantumEnd() {
    let Prcb: PKPRCB = KeGetCurrentPrcb();
    let mut NextThread: PKTHREAD;
    let Thread: PKTHREAD = (*Prcb).CurrentThread;

    if InterlockedExchange(&mut (*Prcb).DpcSetEventRequest, 0) != 0 {
        KeSetEvent(&mut (*Prcb).DpcEvent as *mut _ as PVOID, 0, 0);
    }

    KeRaiseIrqlToSynchLevel();
    KiAcquireThreadLock(Thread);
    KiAcquirePrcbLock(Prcb);

    if *kt_Quantum(Thread) <= 0 {
        if ((*Thread).Priority >= LOW_REALTIME_PRIORITY as i8) &&
           ((*(kt_ApcState(Thread).Process as *mut _KPROCESS)).__bindgen_anon_1.__bindgen_anon_1.DisableQuantum() != 0)
        {
            *kt_Quantum(Thread) = MAX_QUANTUM as i8;
        } else {
            {
                #[cfg(target_arch = "x86")]
                let qr = (*Thread).__bindgen_anon_9.__bindgen_anon_2.QuantumReset as i8;
                #[cfg(not(target_arch = "x86"))]
                let qr = (*Thread).__bindgen_anon_7.__bindgen_anon_2.QuantumReset as i8;
                *kt_Quantum(Thread) = qr;
            }
            (*Thread).Priority = KiComputeNewPriority(Thread, 1) as i8;

            if (*Prcb).NextThread.is_null() {
                NextThread = KiSelectReadyThread((*Thread).Priority as LONG, Prcb);
                if !NextThread.is_null() {
                    *kt_State(NextThread) = Standby;
                    (*Prcb).NextThread = NextThread;
                }
            } else {
                (*Thread).Preempted = 0;
            }
        }
    }

    KiReleaseThreadLock(Thread);

    if (*Prcb).NextThread.is_null() {
        KiReleasePrcbLock(Prcb);
        KeLowerIrql(DISPATCH_LEVEL);
        return;
    }

    NextThread = (*Prcb).NextThread;

    #[cfg(feature = "smp")]
    ntkrnl_sys::inline::ke::ki_check_promote_not_linked(NextThread, 0x03); // KiQuantumEnd

    KiSetThreadSwapBusy(Thread);

    (*Prcb).NextThread = core::ptr::null_mut();
    (*Prcb).CurrentThread = NextThread;

    *kt_State(NextThread) = Running;
    #[cfg(feature = "smp")]
    ntkrnl_sys::inline::ke::rq_inflight_unmark(NextThread);
    (*Thread).WaitReason = _KWAIT_REASON_WrQuantumEnd as u8;

    KxQueueReadyThread(Thread, Prcb);

    *kt_WaitIrql(Thread) = APC_LEVEL;

    KiSwapContext(APC_LEVEL, Thread);

    KeLowerIrql(DISPATCH_LEVEL);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiRetireDpcList(Prcb: PKPRCB) {
    KiRetireDpcList_body(Prcb)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiRetireDpcList(Prcb: PKPRCB) {
    KiRetireDpcList_body(Prcb)
}

unsafe fn KiRetireDpcList_body(Prcb: PKPRCB) {
    let DpcData: PKDPC_DATA;
    let ListHead: PLIST_ENTRY;
    let mut DpcEntry: PLIST_ENTRY;
    let mut Dpc: PKDPC;
    let mut DeferredRoutine: DPC_ROUTINE_OPTION;
    let mut DeferredContext: PVOID;
    let mut SystemArgument1: PVOID;
    let mut SystemArgument2: PVOID;
    let mut TimerHand: ULONG_PTR;

    DpcData = &mut (*Prcb).DpcData[DPC_NORMAL] as *mut _;
    ListHead = &mut (*DpcData).DpcListHead as *mut LIST_ENTRY;

    loop {
        (*Prcb).DpcRoutineActive = 1;

        if (*Prcb).TimerRequest != 0 {
            TimerHand = (*Prcb).TimerHand as ULONG_PTR;
            (*Prcb).TimerRequest = 0;

            _enable();
            KiTimerExpiration(core::ptr::null_mut(), core::ptr::null_mut(), TimerHand as PVOID, core::ptr::null_mut());
            _disable();
        }

        while (*DpcData).DpcQueueDepth != 0 {
            // [guard] KeAcquireSpinLockAtDpcLevel(&mut (*DpcData).DpcLock);
            let dpc_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockAtDpcGuard::acquire(
                &mut (*DpcData).DpcLock as *mut _ as ntkrnl_sys::ke::PKSPIN_LOCK
            ) };
            DpcEntry = (*ListHead).Flink;

            if DpcEntry != ListHead {
                RemoveEntryList(DpcEntry);
                Dpc = ((DpcEntry as usize) - core::mem::offset_of!(_KDPC, DpcListEntry)) as PKDPC;

                (*Dpc).DpcData = core::ptr::null_mut();
                DeferredRoutine = (*Dpc).DeferredRoutine;
                DeferredContext = (*Dpc).DeferredContext;
                SystemArgument1 = (*Dpc).SystemArgument1;
                SystemArgument2 = (*Dpc).SystemArgument2;

                (*DpcData).DpcQueueDepth -= 1;

                // [guard-drop] KeReleaseSpinLockFromDpcLevel — guard Drop handles this
                drop(dpc_lock_guard);

                _enable();

                (DeferredRoutine.unwrap())(Dpc, DeferredContext, SystemArgument1, SystemArgument2);

                _disable();
            } else {
                // [guard-drop] KeReleaseSpinLockFromDpcLevel — guard Drop handles this
                drop(dpc_lock_guard);
            }
        }

        (*Prcb).DpcRoutineActive = 0;
        (*Prcb).DpcInterruptRequested = 0;

        // C dpc.c #ifdef CONFIG_SMP block (dropped in the original translation,
        // when it was UP-dead code): drain deferred-ready threads accumulated by
        // timer-expiration/DPC routines on THIS cpu before leaving. The idle
        // loops' work-check tests DeferredReadyListHead but relies on THIS drain
        // to empty it — without it an idle CPU re-enters hlt with runnable
        // threads parked on its own deferred list until an unrelated cross-CPU
        // wake happens to drain them (the -smp 2 KeGuardedMutex ~60s wake
        // strands / kmtest TIMEOUT).
        #[cfg(feature = "smp")]
        if !(*Prcb).DeferredReadyListHead.Next.is_null() {
            _enable();
            let OldIrql = KeRaiseIrqlToSynchLevel();
            KiProcessDeferredReadyList(Prcb);
            KeLowerIrql(OldIrql);
            _disable();
        }

        if (*DpcData).DpcQueueDepth == 0 {
            break;
        }
    }
}

/// Drain the plain (device) DPC queue WITHOUT touching TimerRequest — the
/// IRQ-exception-exit retire path (arm64 KiIrqHandler). KiRetireDpcList runs
/// KiTimerExpiration when TimerRequest is set, and timer expiration from
/// exception context reliably wedges boot (field manual §9.19) — so the IRQ
/// exit must retire ONLY the device-DPC list and leave timer work to thread
/// context. Crucially, TimerRequest being set must NOT block device-DPC
/// delivery: on this port TimerRequest can sit non-zero indefinitely when a
/// busy thread starves the idle loop (its only consumer), and gating device
/// DPCs on it turned one starved flag into a dead input pipeline.
#[no_mangle]
pub unsafe extern "C" fn KiRetireDeviceDpcs(Prcb: PKPRCB) {
    let DpcData: PKDPC_DATA = &mut (*Prcb).DpcData[DPC_NORMAL] as *mut _;
    let ListHead: PLIST_ENTRY = &mut (*DpcData).DpcListHead as *mut LIST_ENTRY;

    (*Prcb).DpcRoutineActive = 1;

    while (*DpcData).DpcQueueDepth != 0 {
        let dpc_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockAtDpcGuard::acquire(
            &mut (*DpcData).DpcLock as *mut _ as ntkrnl_sys::ke::PKSPIN_LOCK
        ) };
        let DpcEntry = (*ListHead).Flink;

        if DpcEntry != ListHead {
            RemoveEntryList(DpcEntry);
            let Dpc = ((DpcEntry as usize) - core::mem::offset_of!(_KDPC, DpcListEntry)) as PKDPC;

            (*Dpc).DpcData = core::ptr::null_mut();
            let DeferredRoutine = (*Dpc).DeferredRoutine;
            let DeferredContext = (*Dpc).DeferredContext;
            let SystemArgument1 = (*Dpc).SystemArgument1;
            let SystemArgument2 = (*Dpc).SystemArgument2;

            (*DpcData).DpcQueueDepth -= 1;

            drop(dpc_lock_guard);

            if let Some(routine) = DeferredRoutine {
                // The arm64 IRQ-exit retire path is already running in an IRQ
                // exception. Keep PSTATE.I masked there so the controller line
                // cannot re-enter and starve the DPC that is draining it.
                #[cfg(target_arch = "aarch64")]
                let KeepIrqsMasked = crate::ke::arm64::trap::KI_IN_IRQ_EXCEPTION_RETIRE
                    .load(core::sync::atomic::Ordering::SeqCst);
                #[cfg(not(target_arch = "aarch64"))]
                let KeepIrqsMasked = false;

                if !KeepIrqsMasked {
                    _enable();
                }
                routine(Dpc, DeferredContext, SystemArgument1, SystemArgument2);
                if !KeepIrqsMasked {
                    _disable();
                }
            } else {
                crate::dbg_print!(b"KiRetireDeviceDpcs: NULL DeferredRoutine, KDPC %p\n\0", Dpc as usize);
            }
        } else {
            drop(dpc_lock_guard);
        }
    }

    (*Prcb).DpcRoutineActive = 0;
    (*Prcb).DpcInterruptRequested = 0;
}

/// arm64 DISPATCH-level software-interrupt body — the delivery half of
/// `HalRequestSoftwareInterrupt(DISPATCH_LEVEL)`. hal's KfLowerIrql calls this
/// (via the ntoskrnl export) when the soft IRQL crosses below DISPATCH_LEVEL
/// with the DISPATCH bit pended (clock tick saw due timers, DPC queued outside
/// an ISR, ...). Runs in NORMAL THREAD context at DISPATCH_LEVEL with IRQs
/// enabled — the §9.19 exception-context wake hazard does not apply here, so
/// unlike the IRQ-exit path (KiRetireDeviceDpcs) this MAY run KiTimerExpiration
/// and wake/swap threads.
///
/// This closes landmine #4's second half: timer expiration used to depend on
/// the idle loop alone, so ONE ready-spinning thread froze every kernel timer.
/// A spinner still crosses DISPATCH constantly in its own context (syscall
/// paths, spinlock releases), so its own IRQL crossings now expire timers.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn KiDispatchSoftwareIntArm64() {
    let Prcb = KeGetCurrentPrcb();
    if (*Prcb).DpcRoutineActive != 0 {
        return;
    }

    // cswitchdep tap 3: deferred dispatch runs at DISPATCH; DAIF must be
    // balanced across the body (the §9.19 wake-from-exception-context family
    // corrupts exactly this hand-off).
    #[cfg(feature = "cswitchdep")]
    let cswitch_entry_daif: u64 = {
        crate::ke::cswitchdep::cswitch_dpc_dispatch(crate::code_addr!());
        let daif: u64;
        core::arch::asm!("mrs {0}, daif", out(reg) daif, options(nomem, nostack, preserves_flags));
        daif
    };

    if (*Prcb).TimerRequest != 0 {
        let TimerHand = (*Prcb).TimerHand as ULONG_PTR;
        (*Prcb).TimerRequest = 0;
        // Mirror the idle-loop call in KiRetireDpcList_body: DpcRoutineActive
        // set (so KiExitDispatcher inside the expiration DEFERS a woken
        // higher-priority thread instead of swapping mid-delivery), entry at
        // DISPATCH_LEVEL, IRQs enabled (KfLowerIrql thread context).
        (*Prcb).DpcRoutineActive = 1;
        KiTimerExpiration(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            TimerHand as PVOID,
            core::ptr::null_mut(),
        );
        (*Prcb).DpcRoutineActive = 0;
    }

    if (*Prcb).DpcData[DPC_NORMAL].DpcQueueDepth != 0 {
        // KiRetireDeviceDpcs expects entry with IRQs masked (it _enable()s
        // around each DPC body and leaves them masked).
        _disable();
        KiRetireDeviceDpcs(Prcb);
        _enable();
    }

    // Same CONFIG_SMP drain as KiRetireDpcList_body: timer expiration above
    // unwaits threads onto THIS cpu's deferred-ready list; this softint body
    // is the arm64 stand-in for x86's KiRetireDpcList-in-KiDispatchInterrupt,
    // so it must drain too. Thread context at DISPATCH, IRQs enabled — the
    // §9.19 exception-context wake hazard does not apply here (it DOES apply
    // to KiRetireDeviceDpcs' IRQ-exit caller, which deliberately has no
    // drain — its inserts are picked up here or at the idle loop's retire).
    #[cfg(feature = "smp")]
    if !(*Prcb).DeferredReadyListHead.Next.is_null() {
        let OldIrql = KeRaiseIrqlToSynchLevel();
        KiProcessDeferredReadyList(Prcb);
        KeLowerIrql(OldIrql);
    }

    #[cfg(feature = "cswitchdep")]
    {
        let exit_daif: u64;
        core::arch::asm!("mrs {0}, daif", out(reg) exit_daif, options(nomem, nostack, preserves_flags));
        crate::ke::cswitchdep::cswitch_intr_state(cswitch_entry_daif, exit_daif, crate::code_addr!());
    }
}

/// arm64's half of x86's DPC-interrupt tail (amd64 `KiDpcInterruptHandler`,
/// i386 `KiDispatchInterrupt`): honour a quantum end, or hand the CPU to the
/// thread timer expiration readied, at the moment the IRQL crosses under
/// DISPATCH_LEVEL. The HAL's KfLowerIrql calls this at DISPATCH on that
/// crossing, in thread context, after its softint drain has finished.
///
/// Without it arm64 had NO preemption: `KiDispatchSoftwareIntArm64` expires
/// timers with DpcRoutineActive set, so KiExitDispatcher only parks the woken
/// thread in `Prcb->NextThread`, and nothing but the idle loop or a voluntary
/// wait ever switched to it; `KeUpdateRunTime`'s QuantumEnd was never read at
/// all. So a thread that never blocks kept the CPU: the priority-0 zero-page
/// thread, zeroing the free list after kmtest MmSection, held one arm64 CPU
/// for ~95 s of every conformance round -- clock ticking, not one thread
/// scheduled -- until the list ran dry.
///
/// Declines (leaving NextThread for the next crossing, a wait, or the idle
/// loop) when IRQs are masked -- PSTATE.DAIF is not per-thread, and a switch
/// from a masked SVC path would hand the next thread a masked CPU -- from the
/// idle thread, which consumes NextThread itself and must never be queued
/// ready, and inside a DPC.
///
/// For the same reason the preempted thread puts its OWN DAIF back when it is
/// resumed: whoever switches back to it may do so from a masked context (every
/// SVC wait does), and a system thread never ERETs to heal that. The first cut
/// without the restore had the zero-page thread resume masked and zero the
/// free list with every IRQ off -- 63 s of host time, 0.4 s of guest time.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn KiDispatchPreemptArm64() {
    let Prcb = KeGetCurrentPrcb();
    if (*Prcb).QuantumEnd == 0 && (*Prcb).NextThread.is_null() {
        return;
    }
    if (*Prcb).DpcRoutineActive != 0 || (*Prcb).CurrentThread == (*Prcb).IdleThread {
        return;
    }
    let daif: u64;
    core::arch::asm!("mrs {0}, daif", out(reg) daif, options(nomem, nostack, preserves_flags));
    if daif & (1 << 7) != 0 {
        return;
    }

    if (*Prcb).QuantumEnd != 0 {
        (*Prcb).QuantumEnd = 0u8;
        KiQuantumEnd();
        core::arch::asm!("msr daif, {0}", in(reg) daif, options(nomem, nostack));
        return;
    }

    KiAcquirePrcbLock(Prcb);
    let NewThread: PKTHREAD = (*Prcb).NextThread;
    if NewThread.is_null() {
        KiReleasePrcbLock(Prcb);
        return;
    }
    let OldThread: PKTHREAD = (*Prcb).CurrentThread;

    #[cfg(feature = "smp")]
    ntkrnl_sys::inline::ke::ki_check_promote_not_linked(NewThread, 0x05); // KiDispatchPreemptArm64

    (*Prcb).NextThread = core::ptr::null_mut();
    (*Prcb).CurrentThread = NewThread;
    *kt_State(NewThread) = Running;
    #[cfg(feature = "smp")]
    ntkrnl_sys::inline::ke::rq_inflight_unmark(NewThread);
    (*OldThread).WaitReason = ntkrnl_sys::ddk::_KWAIT_REASON_WrDispatchInt as u8;

    // Swap-busy BEFORE the ready insert, as in KiQuantumEnd: another CPU that
    // picks OldThread up must wait for its context to be saved.
    KiSetThreadSwapBusy(OldThread);
    KxQueueReadyThread(OldThread, Prcb);
    *kt_WaitIrql(OldThread) = APC_LEVEL;

    KiSwapContext(APC_LEVEL, OldThread);
    core::arch::asm!("msr daif, {0}", in(reg) daif, options(nomem, nostack));
}

unsafe fn KiInitializeDpc(
    Dpc: PKDPC,
    DeferredRoutine: PKDEFERRED_ROUTINE,
    DeferredContext: PVOID,
    Type: KOBJECTS,
) {
    (*Dpc).Type = Type as u8;
    (*Dpc).Number = 0;
    (*Dpc).Importance = MediumImportance as u8;
    (*Dpc).DeferredRoutine = core::mem::transmute(DeferredRoutine);
    (*Dpc).DeferredContext = DeferredContext;
    (*Dpc).DpcData = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeThreadedDpc(
    Dpc: PKDPC,
    DeferredRoutine: PKDEFERRED_ROUTINE,
    DeferredContext: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    KiInitializeDpc(Dpc, DeferredRoutine, DeferredContext, ThreadedDpcObject);
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeDpc(
    Dpc: PKDPC,
    DeferredRoutine: PKDEFERRED_ROUTINE,
    DeferredContext: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    KiInitializeDpc(Dpc, DeferredRoutine, DeferredContext, DpcObject);
}

#[no_mangle]
pub unsafe extern "system" fn KeInsertQueueDpc(
    Dpc: PKDPC,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) -> BOOLEAN {
    let Dpc = opaque_ptr(Dpc);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    let mut OldIrql: KIRQL = 0;
    let Prcb: PKPRCB;
    let CurrentPrcb: PKPRCB;
    let Cpu: ULONG;
    let DpcData: PKDPC_DATA;
    let mut DpcConfigured: BOOLEAN = 0;
    let mut DpcInserted: BOOLEAN = 0;

    // [guard] KeRaiseIrql(HIGH_LEVEL, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(HIGH_LEVEL) };
    CurrentPrcb = KeGetCurrentPrcb();

    if (*Dpc).Number as ULONG >= MAXIMUM_PROCESSORS {
        Cpu = (*Dpc).Number as ULONG - MAXIMUM_PROCESSORS;
        Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB).add(Cpu as usize);
    } else {
        Prcb = CurrentPrcb;
        Cpu = (*Prcb).Number as ULONG;
    }

    if ((*Dpc).Type == ThreadedDpcObject as u8) && ((*Prcb).ThreadDpcEnable != 0) {
        DpcData = &mut (*Prcb).DpcData[DPC_THREADED] as *mut _;
    } else {
        DpcData = &mut (*Prcb).DpcData[DPC_NORMAL] as *mut _;
    }

    KiAcquireSpinLock(&mut (*DpcData).DpcLock as *mut _ as PKSPIN_LOCK);

    if InterlockedCompareExchangePointer(
        &mut (*Dpc).DpcData as *mut _ as *mut PVOID,
        DpcData as PVOID,
        core::ptr::null_mut(),
    ) == core::ptr::null_mut()
    {
        (*Dpc).SystemArgument1 = SystemArgument1;
        (*Dpc).SystemArgument2 = SystemArgument2;
        (*DpcData).DpcQueueDepth += 1;
        (*DpcData).DpcCount += 1;
        DpcConfigured = 1;

        if (*Dpc).Importance == HighImportance as u8 {
            InsertHeadList(&mut (*DpcData).DpcListHead as *mut LIST_ENTRY, &mut (*Dpc).DpcListEntry as *mut LIST_ENTRY);
        } else {
            InsertTailList(&mut (*DpcData).DpcListHead as *mut LIST_ENTRY, &mut (*Dpc).DpcListEntry as *mut LIST_ENTRY);
        }

        if &(*Prcb).DpcData[DPC_THREADED] as *const _ as PKDPC_DATA == DpcData {
            #[cfg(target_arch = "aarch64")]
            let is_dpc_thread_idle = (*Prcb).__bindgen_anon_2.__bindgen_anon_2.DpcThreadActive() == 0
                && (*Prcb).__bindgen_anon_2.__bindgen_anon_2.DpcThreadRequested() == 0;
            #[cfg(not(target_arch = "aarch64"))]
            let is_dpc_thread_idle = (*Prcb).DpcThreadActive == 0
                && (*Prcb).DpcThreadRequested == 0;
            if is_dpc_thread_idle {
            }
        } else {
            if ((*Prcb).DpcRoutineActive == 0) && ((*Prcb).DpcInterruptRequested == 0) {
                if Prcb != CurrentPrcb {
                    if (((*Dpc).Importance == HighImportance as u8) ||
                        ((*DpcData).DpcQueueDepth >= (*Prcb).MaximumDpcQueueDepth as _)) &&
                       (((((1 as KAFFINITY) << Cpu) & KiIdleSummary) == 0) || ((*Prcb).Sleeping != 0))
                    {
                        (*Prcb).DpcInterruptRequested = 1;
                        DpcInserted = 1;
                    }
                } else {
                    if ((*Dpc).Importance != LowImportance as u8) ||
                       ((*DpcData).DpcQueueDepth >= (*Prcb).MaximumDpcQueueDepth as _) ||
                       ((*Prcb).DpcRequestRate < (*Prcb).MinimumDpcRate)
                    {
                        (*Prcb).DpcInterruptRequested = 1;
                        DpcInserted = 1;
                    }
                }
            }
        }
    }

    KiReleaseSpinLock(&mut (*DpcData).DpcLock as *mut _ as PKSPIN_LOCK);

    if DpcInserted != 0 {
        if Prcb != CurrentPrcb {
            KiIpiSend(1usize << Cpu, IPI_DPC);
        } else {
            HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
        }
    }

    drop(irql_guard);
    DpcConfigured
}

#[no_mangle]
pub unsafe extern "system" fn KeRemoveQueueDpc(Dpc: PKDPC) -> BOOLEAN {
    let Dpc = opaque_ptr(Dpc);
    let DpcData: PKDPC_DATA;
    let Enable: BOOLEAN;

    Enable = KeDisableInterrupts();

    DpcData = (*Dpc).DpcData as PKDPC_DATA;
    if !DpcData.is_null() {
        KiAcquireSpinLock(&mut (*DpcData).DpcLock as *mut _ as PKSPIN_LOCK);

        if DpcData == (*Dpc).DpcData as PKDPC_DATA {
            (*DpcData).DpcQueueDepth -= 1;
            RemoveEntryList(&mut (*Dpc).DpcListEntry as *mut LIST_ENTRY);
            (*Dpc).DpcData = core::ptr::null_mut();
        }

        KiReleaseSpinLock(&mut (*DpcData).DpcLock as *mut _ as PKSPIN_LOCK);
    }

    KeRestoreInterrupts(Enable);

    if !DpcData.is_null() { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn KeFlushQueuedDpcs() {
    let mut ProcessorIndex: ULONG;
    let mut TargetPrcb: PKPRCB;

    for ProcessorIndex in 0..KeNumberProcessors {
        TargetPrcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB).add(ProcessorIndex as usize);

        if ((*TargetPrcb).DpcData[DPC_NORMAL].DpcQueueDepth > 0) ||
           ((*TargetPrcb).DpcData[DPC_THREADED].DpcQueueDepth > 0)
        {
            if TargetPrcb == KeGetCurrentPrcb() {
                HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
            } else {
                KeSetSystemAffinityThread((*TargetPrcb).SetMember);
            }
        }
    }

    let thread = ntkrnl_sys::mm::KeGetCurrentThread() as PKTHREAD;
    #[cfg(target_arch = "x86")]
    let saa = (*thread).__bindgen_anon_7.__bindgen_anon_1.SystemAffinityActive;
    #[cfg(not(target_arch = "x86"))]
    let saa = (*thread).__bindgen_anon_5.__bindgen_anon_1.SystemAffinityActive;
    if saa != 0 {
        KeRevertToUserAffinityThread();
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeIsExecutingDpc() -> BOOLEAN {
    (*KeGetCurrentPrcb()).DpcRoutineActive
}

#[no_mangle]
pub unsafe extern "system" fn KeSetImportanceDpc(Dpc: PKDPC, Importance: KDPC_IMPORTANCE) {
    let Dpc = opaque_ptr(Dpc);
    (*Dpc).Importance = Importance as u8;
}

#[no_mangle]
pub unsafe extern "system" fn KeSetTargetProcessorDpc(Dpc: PKDPC, Number: i8) {
    let Dpc = opaque_ptr(Dpc);
    (*Dpc).Number = (Number as ULONG + MAXIMUM_PROCESSORS) as u16;
}

#[no_mangle]
pub unsafe extern "system" fn KeGenericCallDpc(Routine: PKDEFERRED_ROUTINE, Context: PVOID) {
    let Context = opaque_ptr(Context);
    let mut Barrier: ULONG = KeNumberProcessors as ULONG;
    let mut OldIrql: KIRQL = 0;
    let mut ReverseBarrier: DEFERRED_REVERSE_BARRIER = DEFERRED_REVERSE_BARRIER {
        Barrier,
        TotalProcessors: Barrier,
    };

    // [guard] KeRaiseIrql(DISPATCH_LEVEL, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(DISPATCH_LEVEL) };
    (Routine)(
        &mut (*KeGetCurrentPrcb()).CallDpc as *mut _ as ntkrnl_sys::ke::PKDPC,
        Context,
        &mut Barrier as *mut ULONG as PVOID,
        &mut ReverseBarrier as *mut DEFERRED_REVERSE_BARRIER as PVOID,
    );
    drop(irql_guard);
}

#[no_mangle]
pub unsafe extern "system" fn KeSignalCallDpcDone(SystemArgument1: PVOID) {
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    InterlockedDecrement(SystemArgument1 as *mut LONG);
}

#[no_mangle]
pub unsafe extern "system" fn KeSignalCallDpcSynchronize(SystemArgument2: PVOID) -> BOOLEAN {
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    1u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/dpc.c
//  * PURPOSE:         Deferred Procedure Call (DPC) Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Philip Susi (phreak@iag.net)
//  *                  Eric Kohl
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// ULONG KiMaximumDpcQueueDepth = 4;
// ULONG KiMinimumDpcRate = 3;
// ULONG KiAdjustDpcThreshold = 20;
// ULONG KiIdealDpcRate = 20;
// BOOLEAN KeThreadDpcEnable;
// FAST_MUTEX KiGenericCallDpcMutex;
// KDPC KiTimerExpireDpc;
// ULONG KiTimeLimitIsrMicroseconds;
// ULONG KiDPCTimeout = 110;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// KiCheckTimerTable(IN ULARGE_INTEGER CurrentTime)
// {
// #if DBG
//     ULONG i = 0;
//     PLIST_ENTRY ListHead, NextEntry;
//     KIRQL OldIrql;
//     PKTIMER Timer;
//
//     /* Raise IRQL to high and loop timers */
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//     do
//     {
//         /* Loop the current list */
//         ListHead = &KiTimerTableListHead[i].Entry;
//         NextEntry = ListHead->Flink;
//         while (NextEntry != ListHead)
//         {
//             /* Get the timer and move to the next one */
//             Timer = CONTAINING_RECORD(NextEntry, KTIMER, TimerListEntry);
//             NextEntry = NextEntry->Flink;
//
//             /* Check if it expired */
//             if (Timer->DueTime.QuadPart <= CurrentTime.QuadPart)
//             {
//                 /* Check if the DPC was queued, but didn't run */
//                 if (!(KeGetCurrentPrcb()->TimerRequest) &&
//                     !(*((volatile PULONG*)(&KiTimerExpireDpc.DpcData))))
//                 {
//                     /* This is bad, breakpoint! */
//                     DPRINT1("Invalid timer state!\n");
//                     DbgBreakPoint();
//                 }
//             }
//         }
//
//         /* Move to the next timer */
//         i++;
//     } while(i < TIMER_TABLE_SIZE);
//
//     /* Lower IRQL and return */
//     KeLowerIrql(OldIrql);
// #endif
// }
//
// VOID
// NTAPI
// KiTimerExpiration(IN PKDPC Dpc,
//                   IN PVOID DeferredContext,
//                   IN PVOID SystemArgument1,
//                   IN PVOID SystemArgument2)
// {
//     ULARGE_INTEGER SystemTime, InterruptTime;
//     LARGE_INTEGER Interval;
//     LONG Limit, Index, i;
//     ULONG Timers, ActiveTimers, DpcCalls;
//     PLIST_ENTRY ListHead, NextEntry;
//     KIRQL OldIrql;
//     PKTIMER Timer;
//     PKDPC TimerDpc;
//     ULONG Period;
//     DPC_QUEUE_ENTRY DpcEntry[MAX_TIMER_DPCS];
//     PKSPIN_LOCK_QUEUE LockQueue;
//     PKPRCB Prcb = KeGetCurrentPrcb();
//
//     /* Disable interrupts */
//     _disable();
//
//     /* Query system and interrupt time */
//     KeQuerySystemTime((PLARGE_INTEGER)&SystemTime);
//     InterruptTime.QuadPart = KeQueryInterruptTime();
//     Limit = KeTickCount.LowPart;
//
//     /* Bring interrupts back */
//     _enable();
//
//     /* Get the index of the timer and normalize it */
//     Index = PtrToLong(SystemArgument1);
//     if ((Limit - Index) >= TIMER_TABLE_SIZE)
//     {
//         /* Normalize it */
//         Limit = Index + TIMER_TABLE_SIZE - 1;
//     }
//
//     /* Setup index and actual limit */
//     Index--;
//     Limit &= (TIMER_TABLE_SIZE - 1);
//
//     /* Setup accounting data */
//     DpcCalls = 0;
//     Timers = 24;
//     ActiveTimers = 4;
//
//     /* Lock the Database and Raise IRQL */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Start expiration loop */
//     do
//     {
//         /* Get the current index */
//         Index = (Index + 1) & (TIMER_TABLE_SIZE - 1);
//
//         /* Get list pointers and loop the list */
//         ListHead = &KiTimerTableListHead[Index].Entry;
//         while (ListHead != ListHead->Flink)
//         {
//             /* Lock the timer and go to the next entry */
//             LockQueue = KiAcquireTimerLock(Index);
//             NextEntry = ListHead->Flink;
//
//             /* Get the current timer and check its due time */
//             Timers--;
//             Timer = CONTAINING_RECORD(NextEntry, KTIMER, TimerListEntry);
//             if ((NextEntry != ListHead) &&
//                 (Timer->DueTime.QuadPart <= InterruptTime.QuadPart))
//             {
//                 /* It's expired, remove it */
//                 ActiveTimers--;
//                 KiRemoveEntryTimer(Timer);
//
//                 /* Make it non-inserted, unlock it, and signal it */
//                 Timer->Header.Inserted = FALSE;
//                 KiReleaseTimerLock(LockQueue);
//                 Timer->Header.SignalState = 1;
//
//                 /* Get the DPC and period */
//                 TimerDpc = Timer->Dpc;
//                 Period = Timer->Period;
//
//                 /* Check if there's any waiters */
//                 if (!IsListEmpty(&Timer->Header.WaitListHead))
//                 {
//                     /* Check the type of event */
//                     if (Timer->Header.Type == TimerNotificationObject)
//                     {
//                         /* Unwait the thread */
//                         KxUnwaitThread(&Timer->Header, IO_NO_INCREMENT);
//                     }
//                     else
//                     {
//                         /* Otherwise unwait the thread and signal the timer */
//                         KxUnwaitThreadForEvent((PKEVENT)Timer, IO_NO_INCREMENT);
//                     }
//                 }
//
//                 /* Check if we have a period */
//                 if (Period)
//                 {
//                     /* Calculate the interval and insert the timer */
//                     Interval.QuadPart = Int32x32To64(Period, -10000);
//                     while (!KiInsertTreeTimer(Timer, Interval));
//                 }
//
//                 /* Check if we have a DPC */
//                 if (TimerDpc)
//                 {
// #ifdef CONFIG_SMP
//                     /*
//                      * If the DPC is targeted to another processor,
//                      * then insert it into that processor's DPC queue
//                      * instead of delivering it now.
//                      * If the DPC is a threaded DPC, and the current CPU
//                      * has threaded DPCs enabled (KiExecuteDpc is actively parsing DPCs),
//                      * then also insert it into the DPC queue for threaded delivery,
//                      * instead of doing it here.
//                      */
//                     if (((TimerDpc->Number >= MAXIMUM_PROCESSORS) &&
//                         ((TimerDpc->Number - MAXIMUM_PROCESSORS) != Prcb->Number)) ||
//                         ((TimerDpc->Type == ThreadedDpcObject) && (Prcb->ThreadDpcEnable)))
//                     {
//                         /* Queue it */
//                         KeInsertQueueDpc(TimerDpc,
//                                          UlongToPtr(SystemTime.LowPart),
//                                          UlongToPtr(SystemTime.HighPart));
//                     }
//                     else
// #endif
//                     {
//                         /* Setup the DPC Entry */
//                         DpcEntry[DpcCalls].Dpc = TimerDpc;
//                         DpcEntry[DpcCalls].Routine = TimerDpc->DeferredRoutine;
//                         DpcEntry[DpcCalls].Context = TimerDpc->DeferredContext;
//                         DpcCalls++;
//                         ASSERT(DpcCalls < MAX_TIMER_DPCS);
//                     }
//                 }
//
//                 /* Check if we're done processing */
//                 if (!(ActiveTimers) || !(Timers))
//                 {
//                     /* Release the dispatcher while doing DPCs */
//                     KiReleaseDispatcherLock(DISPATCH_LEVEL);
//
//                     /* Start looping all DPC Entries */
//                     for (i = 0; DpcCalls; DpcCalls--, i++)
//                     {
// #if DBG
//                         /* Clear DPC Time */
//                         Prcb->DebugDpcTime = 0;
// #endif
//
//                         /* Call the DPC */
//                         DpcEntry[i].Routine(DpcEntry[i].Dpc,
//                                             DpcEntry[i].Context,
//                                             UlongToPtr(SystemTime.LowPart),
//                                             UlongToPtr(SystemTime.HighPart));
//                     }
//
//                     /* Reset accounting */
//                     Timers = 24;
//                     ActiveTimers = 4;
//
//                     /* Lock the dispatcher database */
//                     KiAcquireDispatcherLock();
//                 }
//             }
//             else
//             {
//                 /* Check if the timer list is empty */
//                 if (NextEntry != ListHead)
//                 {
//                     /* Sanity check */
//                     ASSERT(KiTimerTableListHead[Index].Time.QuadPart <=
//                            Timer->DueTime.QuadPart);
//
//                     /* Update the time */
//                     _disable();
//                     KiTimerTableListHead[Index].Time.QuadPart =
//                         Timer->DueTime.QuadPart;
//                     _enable();
//                 }
//
//                 /* Release the lock */
//                 KiReleaseTimerLock(LockQueue);
//
//                 /* Check if we've scanned all the timers we could */
//                 if (!Timers)
//                 {
//                     /* Release the dispatcher while doing DPCs */
//                     KiReleaseDispatcherLock(DISPATCH_LEVEL);
//
//                     /* Start looping all DPC Entries */
//                     for (i = 0; DpcCalls; DpcCalls--, i++)
//                     {
// #if DBG
//                         /* Clear DPC Time */
//                         Prcb->DebugDpcTime = 0;
// #endif
//
//                         /* Call the DPC */
//                         DpcEntry[i].Routine(DpcEntry[i].Dpc,
//                                             DpcEntry[i].Context,
//                                             UlongToPtr(SystemTime.LowPart),
//                                             UlongToPtr(SystemTime.HighPart));
//                     }
//
//                     /* Reset accounting */
//                     Timers = 24;
//                     ActiveTimers = 4;
//
//                     /* Lock the dispatcher database */
//                     KiAcquireDispatcherLock();
//                 }
//
//                 /* Done looping */
//                 break;
//             }
//         }
//     } while (Index != Limit);
//
//     /* Verify the timer table, on debug builds */
//     if (KeNumberProcessors == 1) KiCheckTimerTable(InterruptTime);
//
//     /* Check if we still have DPC entries */
//     if (DpcCalls)
//     {
//         /* Release the dispatcher while doing DPCs */
//         KiReleaseDispatcherLock(DISPATCH_LEVEL);
//
//         /* Start looping all DPC Entries */
//         for (i = 0; DpcCalls; DpcCalls--, i++)
//         {
// #if DBG
//             /* Clear DPC Time */
//             Prcb->DebugDpcTime = 0;
// #endif
//
//             /* Call the DPC */
//             DpcEntry[i].Routine(DpcEntry[i].Dpc,
//                                 DpcEntry[i].Context,
//                                 UlongToPtr(SystemTime.LowPart),
//                                 UlongToPtr(SystemTime.HighPart));
//         }
//
//         /* Lower IRQL if we need to */
//         if (OldIrql != DISPATCH_LEVEL) KeLowerIrql(OldIrql);
//     }
//     else
//     {
//         /* Unlock the dispatcher */
//         KiReleaseDispatcherLock(OldIrql);
//     }
// }
//
// VOID
// FASTCALL
// KiTimerListExpire(IN PLIST_ENTRY ExpiredListHead,
//                   IN KIRQL OldIrql)
// {
//     ULARGE_INTEGER SystemTime;
//     LARGE_INTEGER Interval;
//     LONG i;
//     ULONG DpcCalls = 0;
//     PKTIMER Timer;
//     PKDPC TimerDpc;
//     ULONG Period;
//     DPC_QUEUE_ENTRY DpcEntry[MAX_TIMER_DPCS];
//     PKPRCB Prcb = KeGetCurrentPrcb();
//
//     /* Query system */
//     KeQuerySystemTime((PLARGE_INTEGER)&SystemTime);
//
//     /* Loop expired list */
//     while (ExpiredListHead->Flink != ExpiredListHead)
//     {
//         /* Get the current timer */
//         Timer = CONTAINING_RECORD(ExpiredListHead->Flink, KTIMER, TimerListEntry);
//
//         /* Remove it */
//         RemoveEntryList(&Timer->TimerListEntry);
//
//         /* Not inserted */
//         Timer->Header.Inserted = FALSE;
//
//         /* Signal it */
//         Timer->Header.SignalState = 1;
//
//         /* Get the DPC and period */
//         TimerDpc = Timer->Dpc;
//         Period = Timer->Period;
//
//         /* Check if there's any waiters */
//         if (!IsListEmpty(&Timer->Header.WaitListHead))
//         {
//             /* Check the type of event */
//             if (Timer->Header.Type == TimerNotificationObject)
//             {
//                 /* Unwait the thread */
//                 KxUnwaitThread(&Timer->Header, IO_NO_INCREMENT);
//             }
//             else
//             {
//                 /* Otherwise unwait the thread and signal the timer */
//                 KxUnwaitThreadForEvent((PKEVENT)Timer, IO_NO_INCREMENT);
//             }
//         }
//
//         /* Check if we have a period */
//         if (Period)
//         {
//             /* Calculate the interval and insert the timer */
//             Interval.QuadPart = Int32x32To64(Period, -10000);
//             while (!KiInsertTreeTimer(Timer, Interval));
//         }
//
//         /* Check if we have a DPC */
//         if (TimerDpc)
//         {
// #ifdef CONFIG_SMP
//             /*
//              * If the DPC is targeted to another processor,
//              * then insert it into that processor's DPC queue
//              * instead of delivering it now.
//              * If the DPC is a threaded DPC, and the current CPU
//              * has threaded DPCs enabled (KiExecuteDpc is actively parsing DPCs),
//              * then also insert it into the DPC queue for threaded delivery,
//              * instead of doing it here.
//              */
//             if (((TimerDpc->Number >= MAXIMUM_PROCESSORS) &&
//                 ((TimerDpc->Number - MAXIMUM_PROCESSORS) != Prcb->Number)) ||
//                 ((TimerDpc->Type == ThreadedDpcObject) && (Prcb->ThreadDpcEnable)))
//             {
//                 /* Queue it */
//                 KeInsertQueueDpc(TimerDpc,
//                                  UlongToPtr(SystemTime.LowPart),
//                                  UlongToPtr(SystemTime.HighPart));
//             }
//             else
// #endif
//             {
//                 /* Setup the DPC Entry */
//                 DpcEntry[DpcCalls].Dpc = TimerDpc;
//                 DpcEntry[DpcCalls].Routine = TimerDpc->DeferredRoutine;
//                 DpcEntry[DpcCalls].Context = TimerDpc->DeferredContext;
//                 DpcCalls++;
//                 ASSERT(DpcCalls < MAX_TIMER_DPCS);
//             }
//         }
//     }
//
//     /* Check if we still have DPC entries */
//     if (DpcCalls)
//     {
//         /* Release the dispatcher while doing DPCs */
//         KiReleaseDispatcherLock(DISPATCH_LEVEL);
//
//         /* Start looping all DPC Entries */
//         for (i = 0; DpcCalls; DpcCalls--, i++)
//         {
// #if DBG
//             /* Clear DPC Time */
//             Prcb->DebugDpcTime = 0;
// #endif
//
//             /* Call the DPC */
//             DpcEntry[i].Routine(DpcEntry[i].Dpc,
//                                 DpcEntry[i].Context,
//                                 UlongToPtr(SystemTime.LowPart),
//                                 UlongToPtr(SystemTime.HighPart));
//         }
//
//         /* Lower IRQL */
//         KeLowerIrql(OldIrql);
//     }
//     else
//     {
//         /* Unlock the dispatcher */
//         KiReleaseDispatcherLock(OldIrql);
//     }
// }
//
// _Requires_lock_not_held_(Prcb->PrcbLock)
// VOID
// NTAPI
// KiQuantumEnd(VOID)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PKTHREAD NextThread, Thread = Prcb->CurrentThread;
//
//     /* Check if a DPC Event was requested to be signaled */
//     if (InterlockedExchange(&Prcb->DpcSetEventRequest, 0))
//     {
//         /* Signal it */
//         KeSetEvent(&Prcb->DpcEvent, 0, 0);
//     }
//
//     /* Raise to synchronization level and lock the PRCB and thread */
//     KeRaiseIrqlToSynchLevel();
//     KiAcquireThreadLock(Thread);
//     KiAcquirePrcbLock(Prcb);
//
//     /* Check if Quantum expired */
//     if (Thread->Quantum <= 0)
//     {
//         /* Check if we're real-time and with quantums disabled */
//         if ((Thread->Priority >= LOW_REALTIME_PRIORITY) &&
//             (Thread->ApcState.Process->DisableQuantum))
//         {
//             /* Otherwise, set maximum quantum */
//             Thread->Quantum = MAX_QUANTUM;
//         }
//         else
//         {
//             /* Reset the new Quantum */
//             Thread->Quantum = Thread->QuantumReset;
//
//             /* Calculate new priority */
//             Thread->Priority = KiComputeNewPriority(Thread, 1);
//
//             /* Check if a new thread is scheduled */
//             if (!Prcb->NextThread)
//             {
//                 /* Get a new ready thread */
//                 NextThread = KiSelectReadyThread(Thread->Priority, Prcb);
//                 if (NextThread)
//                 {
//                     /* Found one, set it on standby */
//                     NextThread->State = Standby;
//                     Prcb->NextThread = NextThread;
//                 }
//             }
//             else
//             {
//                 /* Otherwise, make sure that this thread doesn't get preempted */
//                 Thread->Preempted = FALSE;
//             }
//         }
//     }
//
//     /* Release the thread lock */
//     KiReleaseThreadLock(Thread);
//
//     /* Check if there's no thread scheduled */
//     if (!Prcb->NextThread)
//     {
//         /* Just leave now */
//         KiReleasePrcbLock(Prcb);
//         KeLowerIrql(DISPATCH_LEVEL);
//         return;
//     }
//
//     /* Get the next thread now */
//     NextThread = Prcb->NextThread;
//
//     /* Set current thread's swap busy to true */
//     KiSetThreadSwapBusy(Thread);
//
//     /* Switch threads in PRCB */
//     Prcb->NextThread = NULL;
//     Prcb->CurrentThread = NextThread;
//
//     /* Set thread to running and the switch reason to Quantum End */
//     NextThread->State = Running;
//     Thread->WaitReason = WrQuantumEnd;
//
//     /* Queue it on the ready lists */
//     KxQueueReadyThread(Thread, Prcb);
//
//     /* Set wait IRQL to APC_LEVEL */
//     Thread->WaitIrql = APC_LEVEL;
//
//     /* Swap threads */
//     KiSwapContext(APC_LEVEL, Thread);
//
//     /* Lower IRQL back to DISPATCH_LEVEL */
//     KeLowerIrql(DISPATCH_LEVEL);
// }
//
// VOID
// FASTCALL
// KiRetireDpcList(IN PKPRCB Prcb)
// {
//     PKDPC_DATA DpcData;
//     PLIST_ENTRY ListHead, DpcEntry;
//     PKDPC Dpc;
//     PKDEFERRED_ROUTINE DeferredRoutine;
//     PVOID DeferredContext, SystemArgument1, SystemArgument2;
//     ULONG_PTR TimerHand;
// #ifdef CONFIG_SMP
//     KIRQL OldIrql;
// #endif
//
//     /* Get data and list variables before starting anything else */
//     DpcData = &Prcb->DpcData[DPC_NORMAL];
//     ListHead = &DpcData->DpcListHead;
//
//     /* Main outer loop */
//     do
//     {
//         /* Set us as active */
//         Prcb->DpcRoutineActive = TRUE;
//
//         /* Check if this is a timer expiration request */
//         if (Prcb->TimerRequest)
//         {
//             /* It is, get the timer hand and disable timer request */
//             TimerHand = Prcb->TimerHand;
//             Prcb->TimerRequest = 0;
//
//             /* Expire timers with interrupts enabled */
//             _enable();
//             KiTimerExpiration(NULL, NULL, (PVOID)TimerHand, NULL);
//             _disable();
//         }
//
//         /* Loop while we have entries in the queue */
//         while (DpcData->DpcQueueDepth != 0)
//         {
//             /* Lock the DPC data and get the DPC entry*/
//             KeAcquireSpinLockAtDpcLevel(&DpcData->DpcLock);
//             DpcEntry = ListHead->Flink;
//
//             /* Make sure we have an entry */
//             if (DpcEntry != ListHead)
//             {
//                 /* Remove the DPC from the list */
//                 RemoveEntryList(DpcEntry);
//                 Dpc = CONTAINING_RECORD(DpcEntry, KDPC, DpcListEntry);
//
//                 /* Clear its DPC data and save its parameters */
//                 Dpc->DpcData = NULL;
//                 DeferredRoutine = Dpc->DeferredRoutine;
//                 DeferredContext = Dpc->DeferredContext;
//                 SystemArgument1 = Dpc->SystemArgument1;
//                 SystemArgument2 = Dpc->SystemArgument2;
//
//                 /* Decrease the queue depth */
//                 DpcData->DpcQueueDepth--;
//
// #if DBG
//                 /* Clear DPC Time */
//                 Prcb->DebugDpcTime = 0;
// #endif
//
//                 /* Release the lock */
//                 KeReleaseSpinLockFromDpcLevel(&DpcData->DpcLock);
//
//                 /* Re-enable interrupts */
//                 _enable();
//
//                 /* Call the DPC */
//                 DeferredRoutine(Dpc,
//                                 DeferredContext,
//                                 SystemArgument1,
//                                 SystemArgument2);
//                 ASSERT(KeGetCurrentIrql() == DISPATCH_LEVEL);
//
//                 /* Disable interrupts and keep looping */
//                 _disable();
//             }
//             else
//             {
//                 /* The queue should be flushed now */
//                 ASSERT(DpcData->DpcQueueDepth == 0);
//
//                 /* Release DPC Lock */
//                 KeReleaseSpinLockFromDpcLevel(&DpcData->DpcLock);
//             }
//         }
//
//         /* Clear DPC Flags */
//         Prcb->DpcRoutineActive = FALSE;
//         Prcb->DpcInterruptRequested = FALSE;
//
// #ifdef CONFIG_SMP
//         /* Check if we have deferred threads */
//         if (Prcb->DeferredReadyListHead.Next)
//         {
//
//             /* Re-enable interrupts and raise to synch */
//             _enable();
//             OldIrql = KeRaiseIrqlToSynchLevel();
//
//             /* Process deferred threads */
//             KiProcessDeferredReadyList(Prcb);
//
//             /* Lower IRQL back and disable interrupts */
//             KeLowerIrql(OldIrql);
//             _disable();
//         }
// #endif
//     } while (DpcData->DpcQueueDepth != 0);
// }
//
// VOID
// NTAPI
// KiInitializeDpc(IN PKDPC Dpc,
//                 IN PKDEFERRED_ROUTINE DeferredRoutine,
//                 IN PVOID DeferredContext,
//                 IN KOBJECTS Type)
// {
//     /* Setup the DPC Object */
//     Dpc->Type = Type;
//     Dpc->Number = 0;
//     Dpc->Importance= MediumImportance;
//     Dpc->DeferredRoutine = DeferredRoutine;
//     Dpc->DeferredContext = DeferredContext;
//     Dpc->DpcData = NULL;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeThreadedDpc(IN PKDPC Dpc,
//                         IN PKDEFERRED_ROUTINE DeferredRoutine,
//                         IN PVOID DeferredContext)
// {
//     /* Call the internal routine */
//     KiInitializeDpc(Dpc, DeferredRoutine, DeferredContext, ThreadedDpcObject);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeDpc(IN PKDPC Dpc,
//                 IN PKDEFERRED_ROUTINE DeferredRoutine,
//                 IN PVOID DeferredContext)
// {
//     /* Call the internal routine */
//     KiInitializeDpc(Dpc, DeferredRoutine, DeferredContext, DpcObject);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeInsertQueueDpc(IN PKDPC Dpc,
//                  IN PVOID SystemArgument1,
//                  IN PVOID SystemArgument2)
// {
//     KIRQL OldIrql;
//     PKPRCB Prcb, CurrentPrcb;
//     ULONG Cpu;
//     PKDPC_DATA DpcData;
//     BOOLEAN DpcConfigured = FALSE, DpcInserted = FALSE;
//     ASSERT_DPC(Dpc);
//
//     /* Check IRQL and Raise it to HIGH_LEVEL */
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//     CurrentPrcb = KeGetCurrentPrcb();
//
//     /* Check if the DPC has more then the maximum number of CPUs */
//     if (Dpc->Number >= MAXIMUM_PROCESSORS)
//     {
//         /* Then substract the maximum and get that PRCB. */
//         Cpu = Dpc->Number - MAXIMUM_PROCESSORS;
//         Prcb = KiProcessorBlock[Cpu];
//     }
//     else
//     {
//         /* Use the current one */
//         Prcb = CurrentPrcb;
//         Cpu = Prcb->Number;
//     }
//
//     /* ROS Sanity Check */
//     ASSERT(Prcb == CurrentPrcb);
//
//     /* Check if this is a threaded DPC and threaded DPCs are enabled */
//     if ((Dpc->Type == ThreadedDpcObject) && (Prcb->ThreadDpcEnable))
//     {
//         /* Then use the threaded data */
//         DpcData = &Prcb->DpcData[DPC_THREADED];
//     }
//     else
//     {
//         /* Otherwise, use the regular data */
//         DpcData = &Prcb->DpcData[DPC_NORMAL];
//     }
//
//     /* Acquire the DPC lock */
//     KiAcquireSpinLock(&DpcData->DpcLock);
//
//     /* Get the DPC Data */
//     if (!InterlockedCompareExchangePointer(&Dpc->DpcData, DpcData, NULL))
//     {
//         /* Now we can play with the DPC safely */
//         Dpc->SystemArgument1 = SystemArgument1;
//         Dpc->SystemArgument2 = SystemArgument2;
//         DpcData->DpcQueueDepth++;
//         DpcData->DpcCount++;
//         DpcConfigured = TRUE;
//
//         /* Check if this is a high importance DPC */
//         if (Dpc->Importance == HighImportance)
//         {
//             /* Pre-empty other DPCs */
//             InsertHeadList(&DpcData->DpcListHead, &Dpc->DpcListEntry);
//         }
//         else
//         {
//             /* Add it at the end */
//             InsertTailList(&DpcData->DpcListHead, &Dpc->DpcListEntry);
//         }
//
//         /* Check if this is the DPC on the threaded list */
//         if (&Prcb->DpcData[DPC_THREADED] == DpcData)
//         {
//             /* Make sure a threaded DPC isn't already active */
//             if (!(Prcb->DpcThreadActive) && !(Prcb->DpcThreadRequested))
//             {
//                 /* FIXME: Setup Threaded DPC */
//                 UNIMPLEMENTED_FATAL("Threaded DPC not supported\n");
//             }
//         }
//         else
//         {
//             /* Make sure a DPC isn't executing already */
//             if (!(Prcb->DpcRoutineActive) && !(Prcb->DpcInterruptRequested))
//             {
//                 /* Check if this is the same CPU */
//                 if (Prcb != CurrentPrcb)
//                 {
//                     /*
//                      * Check if the DPC is of high importance or above the
//                      * maximum depth. If it is, then make sure that the CPU
//                      * isn't idle, or that it's sleeping.
//                      */
//                     if (((Dpc->Importance == HighImportance) ||
//                         (DpcData->DpcQueueDepth >=
//                          Prcb->MaximumDpcQueueDepth)) &&
//                         (!(AFFINITY_MASK(Cpu) & KiIdleSummary) ||
//                          (Prcb->Sleeping)))
//                     {
//                         /* Set interrupt requested */
//                         Prcb->DpcInterruptRequested = TRUE;
//
//                         /* Set DPC inserted */
//                         DpcInserted = TRUE;
//                     }
//                 }
//                 else
//                 {
//                     /* Check if the DPC is of anything but low importance */
//                     if ((Dpc->Importance != LowImportance) ||
//                         (DpcData->DpcQueueDepth >=
//                          Prcb->MaximumDpcQueueDepth) ||
//                         (Prcb->DpcRequestRate < Prcb->MinimumDpcRate))
//                     {
//                         /* Set interrupt requested */
//                         Prcb->DpcInterruptRequested = TRUE;
//
//                         /* Set DPC inserted */
//                         DpcInserted = TRUE;
//                     }
//                 }
//             }
//         }
//     }
//
//     /* Release the lock */
//     KiReleaseSpinLock(&DpcData->DpcLock);
//
//     /* Check if the DPC was inserted */
//     if (DpcInserted)
//     {
//         /* Check if this was SMP */
//         if (Prcb != CurrentPrcb)
//         {
//             /* It was, request and IPI */
//             KiIpiSend(AFFINITY_MASK(Cpu), IPI_DPC);
//         }
//         else
//         {
//             /* It wasn't, request an interrupt from HAL */
//             HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
//         }
//     }
//
//     /* Lower IRQL */
//     KeLowerIrql(OldIrql);
//     return DpcConfigured;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeRemoveQueueDpc(IN PKDPC Dpc)
// {
//     PKDPC_DATA DpcData;
//     BOOLEAN Enable;
//     ASSERT_DPC(Dpc);
//
//     /* Disable interrupts */
//     Enable = KeDisableInterrupts();
//
//     /* Get DPC data */
//     DpcData = Dpc->DpcData;
//     if (DpcData)
//     {
//         /* Acquire the DPC lock */
//         KiAcquireSpinLock(&DpcData->DpcLock);
//
//         /* Make sure that the data didn't change */
//         if (DpcData == Dpc->DpcData)
//         {
//             /* Remove the DPC */
//             DpcData->DpcQueueDepth--;
//             RemoveEntryList(&Dpc->DpcListEntry);
//             Dpc->DpcData = NULL;
//         }
//
//         /* Release the lock */
//         KiReleaseSpinLock(&DpcData->DpcLock);
//     }
//
//     /* Re-enable interrupts */
//     KeRestoreInterrupts(Enable);
//
//     /* Return if the DPC was in the queue or not */
//     return DpcData ? TRUE : FALSE;
// }
//
// /*
//  * @implemented
//  */
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// NTAPI
// KeFlushQueuedDpcs(VOID)
// {
//     ULONG ProcessorIndex;
//     PKPRCB TargetPrcb;
//
//     PAGED_CODE();
//     ASSERT(KeGetCurrentThread()->SystemAffinityActive == FALSE);
//
//     /* Loop all processors */
//     for (ProcessorIndex = 0; ProcessorIndex < KeNumberProcessors; ProcessorIndex++)
//     {
//         /* Get the target processor's PRCB */
//         TargetPrcb = KiProcessorBlock[ProcessorIndex];
//
//         /* Check if there are DPCs on either queues */
//         if ((TargetPrcb->DpcData[DPC_NORMAL].DpcQueueDepth > 0) ||
//             (TargetPrcb->DpcData[DPC_THREADED].DpcQueueDepth > 0))
//         {
//             /* Check if this is the current processor */
//             if (TargetPrcb == KeGetCurrentPrcb())
//             {
//                 /* Request a DPC interrupt */
//                 HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
//             }
//             else
//             {
//                 /* Attach to the target processor. This will cause a DPC
//                    interrupt on the target processor and flush all DPCs. */
//                 KeSetSystemAffinityThread(TargetPrcb->SetMember);
//             }
//         }
//     }
//
//     /* Revert back to user affinity */
//     if (KeGetCurrentThread()->SystemAffinityActive)
//     {
//         KeRevertToUserAffinityThread();
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeIsExecutingDpc(VOID)
// {
//     /* Return if the Dpc Routine is active */
//     return KeGetCurrentPrcb()->DpcRoutineActive;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeSetImportanceDpc (IN PKDPC Dpc,
//                     IN KDPC_IMPORTANCE Importance)
// {
//     /* Set the DPC Importance */
//     ASSERT_DPC(Dpc);
//     Dpc->Importance = Importance;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeSetTargetProcessorDpc(IN PKDPC Dpc,
//                         IN CCHAR Number)
// {
//     /* Set a target CPU */
//     ASSERT_DPC(Dpc);
//     Dpc->Number = Number + MAXIMUM_PROCESSORS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeGenericCallDpc(IN PKDEFERRED_ROUTINE Routine,
//                  IN PVOID Context)
// {
//     ULONG Barrier = KeNumberProcessors;
//     KIRQL OldIrql;
//     DEFERRED_REVERSE_BARRIER ReverseBarrier;
//     ASSERT(KeGetCurrentIrql () < DISPATCH_LEVEL);
//
//     //
//     // The barrier is the number of processors, each processor will decrement it
//     // by one, so when all processors have run the DPC, the barrier reaches zero
//     //
//     ReverseBarrier.Barrier = Barrier;
//     ReverseBarrier.TotalProcessors = Barrier;
//
//     //
//     // But we don't need the barrier on UP, since we can simply call the routine
//     // directly while at DISPATCH_LEVEL and not worry about anything else
//     //
//     KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//     Routine(&KeGetCurrentPrcb()->CallDpc, Context, &Barrier, &ReverseBarrier);
//     KeLowerIrql(OldIrql);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeSignalCallDpcDone(IN PVOID SystemArgument1)
// {
//     //
//     // Decrement the barrier, which is actually the processor count
//     //
//     InterlockedDecrement((PLONG)SystemArgument1);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeSignalCallDpcSynchronize(IN PVOID SystemArgument2)
// {
//     //
//     // There is nothing to do on UP systems -- the processor calling this wins
//     //
//     UNREFERENCED_PARAMETER(SystemArgument2);
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
