use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _ETHREAD,
    _KTHREAD,
    _KPROCESS,
    _KMUTANT,
    _KAPC,
    _KSEMAPHORE,
    _KTIMER,
    _KWAIT_BLOCK,
    _KLOCK_QUEUE_HANDLE,
    _KPRCB,
    _KNODE,
    _KQUEUE,
    PEPROCESS,
    PETHREAD,
    PKTHREAD,
    PKPROCESS,
    PKMUTANT,
    PKAPC,
    PKSEMAPHORE,
    PKTIMER,
    PKWAIT_BLOCK,
    PKLOCK_QUEUE_HANDLE,
    PKPRCB,
    PKNODE,
    PKQUEUE,
    KAFFINITY,
    KPRIORITY,
    KPROCESSOR_MODE,
    _EX_WORK_QUEUE,
    _KWAIT_REASON_Suspended,
    KeInitializeApc,
    KeInitializeSemaphore,
    KeInitializeSpinLock,
    KeInitializeTimer,
    KeIsExecutingDpc,
    KeWaitForSingleObject,
    KiAcquireApcLockAtSynchLevel,
    KiAcquireApcLockRaiseToSynch,
    KiAcquirePrcbLock,
    KiAcquireProcessLockRaiseToSynch,
    KiActivateWaiterQueue,
    KiComputeNewPriority,
    KiExitDispatcher,
    KiInitializeContextThread,
    KiInsertQueue,
    KiInsertQueueApc,
    KiReadyThread,
    KiReleaseApcLock,
    KiReleaseApcLockFromSynchLevel,
    KiReleasePrcbLock,
    KiReleaseProcessLock,
    KiReleaseProcessLockFromSynchLevel,
    KiRundownThread,
    KiSelectNextThread,
    KiSetAffinityThread,
    KiSetPriorityThread,
    KiSetThreadSwapBusy,
    KiSwapThread,
    MmCreateKernelStack,
    MmDeleteKernelStack,
};

// Use actual definitions, not extern declarations (lto=false splits them)
use crate::ke::krnlinit::{KiProcessorBlock, KeNodeBlock};
use crate::ke::processor::KeNumberProcessors;
extern "system" {
    static mut ExWorkerQueue: [_EX_WORK_QUEUE; 7];
    static mut PspReaperListHead: LIST_ENTRY;
    static mut PspReaperWorkItem: ntkrnl_sys::ddk::_WORK_QUEUE_ITEM;
}

#[cfg(target_arch = "aarch64")]
extern "system" {
    static mut MmLargeStackSize: ULONG;
}
// Per-arch kernel stack size — sourced from sdk/include/xdk/<arch>/ke.h
// via the canonical constant in ARM3/helpers.rs (amd64=0xC000, x86/arm64=0x3000).
// Hardcoding 12288 here understated StackLimit on amd64 by 36KB and is one of
// the conditions that allowed the GUI-stack BUGCHECK 0x7F (8) regression.
use crate::ARM3::helpers::KERNEL_STACK_SIZE;
const MAXIMUM_SUSPEND_COUNT: i8 = 127;
const THREAD_WAIT_OBJECTS: usize = 3;
const TIMER_WAIT_BLOCK: usize = 3;
const LOW_PRIORITY: KPRIORITY = 0;
const LOW_REALTIME_PRIORITY: KPRIORITY = 16;
const HIGH_PRIORITY: KPRIORITY = 31;
const THREAD_ALERT_INCREMENT: KPRIORITY = 0x2;
const IO_NO_INCREMENT: KPRIORITY = 0;
const MUTANT_INCREMENT: KPRIORITY = 1;

const MaximumWorkQueue: usize = 7;
const HyperCriticalWorkQueue: usize = 2;

const OriginalApcEnvironment: usize = 0;
const AttachedApcEnvironment: usize = 1;

const ThreadObject: u8 = 6;
const Initialized: u8 = 0;
const Terminated: u8 = 4;
const Waiting: u8 = 5;
const Standby: u8 = 3;
const ProcessInMemory_: u8 = 0;

const Suspended_val: u32 = 5;
const AdjustNone: u8 = 0;
const WaitAny: u8 = 1;

const STATUS_ALERTED: NTSTATUS = 0x00000101u32 as NTSTATUS;
const STATUS_TIMEOUT: NTSTATUS = 0x00000102u32 as NTSTATUS;
const STATUS_SUSPEND_COUNT_EXCEEDED: NTSTATUS = 0xC000004Au32 as NTSTATUS;
const STATUS_UNSUCCESSFUL_: NTSTATUS = 0xC0000001u32 as NTSTATUS;

const THREAD_TERMINATE_HELD_MUTEX: ULONG = 0x4000008A;

macro_rules! AFFINITY_MASK {
    ($n:expr) => {
        (1 as KAFFINITY) << (($n) as KAFFINITY)
    };
}

macro_rules! ASSERT_THREAD {
    ($t:expr) => {};
}

macro_rules! ASSERT_MUTANT {
    ($m:expr) => {};
}

macro_rules! ASSERT_IRQL_LESS_OR_EQUAL {
    ($level:expr) => {};
}

macro_rules! ASSERT {
    ($cond:expr) => {};
}

macro_rules! UNIMPLEMENTED {
    () => {};
}

macro_rules! UNREFERENCED_PARAMETER {
    ($p:expr) => {
        let _ = $p;
    };
}

/// Extern declarations for functions that are in ke_defs but have signatures
/// we need to shadow/redeclare to avoid name clashes with the ddk imports.
extern "system" {
    fn KiFindIdealProcessor_impl(Affinity: KAFFINITY, Processor: u8) -> u8;
    fn BitScanReverseAffinity_impl(Index: *mut u32, Mask: KAFFINITY);
}
extern "C" {
    fn KxUnwaitThread_impl(Header: *mut ntkrnl_sys::ddk::_DISPATCHER_HEADER, Increment: KPRIORITY);
}

/// Local BitScanReverse implementation (not available in ntkrnl-sys/ddk)
#[inline(always)]
unsafe fn BitScanReverse_local(index: *mut u32, mask: u32) -> u8 {
    if mask == 0 { return 0; }
    *index = 31 - mask.leading_zeros();
    1
}

/// Wrapper for BitScanReverseAffinity
unsafe fn BitScanReverseAffinity(Index: *mut u32, Mask: KAFFINITY) {
    BitScanReverse_local(Index, Mask as u32);
    #[cfg(target_pointer_width = "64")]
    {
        // On 64-bit, check the upper 32 bits
        let mut hi: u32 = 0;
        if (Mask >> 32) != 0 {
            BitScanReverse_local(&mut hi, (Mask >> 32) as u32);
            *Index = hi + 32;
        }
    }
}

unsafe fn _KeGetCurrentThread() -> PKTHREAD {
    PsGetCurrentThread()
}

unsafe fn _KeGetPreviousMode() -> u8 {
    let thread = PsGetCurrentThread();
    *kt_PreviousMode(thread) as u8
}

pub(crate) unsafe fn KeFindNextRightSetAffinity(Number: u8, Set: KAFFINITY) -> u8 {
    let mut Bit: KAFFINITY;
    let mut Result: u32 = 0;
    ASSERT!(Set != 0);

    Bit = (AFFINITY_MASK!(Number).wrapping_sub(1)) & Set;

    if Bit == 0 {
        Bit = Set;
    }

    BitScanReverseAffinity(core::ptr::addr_of_mut!(Result), Bit);
    Result as u8
}

unsafe fn KeReadStateThread_impl(Thread: PKTHREAD) -> BOOLEAN {
    ASSERT_THREAD!(Thread);
    (*Thread).Header.SignalState as BOOLEAN
}

unsafe fn KeQueryBasePriorityThread_impl(Thread: PKTHREAD) -> KPRIORITY {
    let mut BaseIncrement: LONG;
    let OldIrql: KIRQL;
    let Process: PKPROCESS;
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    OldIrql = KeRaiseIrqlToSynchLevel();

    KiAcquireThreadLock(Thread);

    Process = (*Thread).ApcStatePointer[0] as *mut _KPROCESS;

    BaseIncrement = (*Thread).BasePriority as LONG - (*Process).BasePriority as LONG;

    if (*Thread).Saturation != 0 {
        BaseIncrement = ((HIGH_PRIORITY + 1) / 2) * (*Thread).Saturation as LONG;
    }

    KiReleaseThreadLock(Thread);

    KeLowerIrql(OldIrql);
    BaseIncrement
}

unsafe fn KeSetDisableBoostThread_impl(Thread: PKTHREAD, Disable: BOOLEAN) -> BOOLEAN {
    ASSERT_THREAD!(Thread);

    let flags_ptr = kt_ThreadFlags(Thread) as *mut LONG;
    if Disable != 0 {
        (InterlockedBitTestAndSet(flags_ptr, 1) != 0) as BOOLEAN
    } else {
        (InterlockedBitTestAndReset(flags_ptr, 1) != 0) as BOOLEAN
    }
}

unsafe fn KeReadyThread_impl(Thread: PKTHREAD) {
    let OldIrql: KIRQL;
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    OldIrql = KiAcquireDispatcherLock();

    // §9.56 tripwire 0x62 — the LAST unguarded KiReadyThread caller (stack
    // inswap / start paths). Legal inputs: Transition (stack back in) or
    // Initialized/DeferredReady on start. Readying a Running/Ready/Waiting
    // thread from here is the unnamed spurious wake.
    #[cfg(feature = "smp")]
    {
        let st = *kt_State(Thread);
        if st == 2u8 /*Running*/ || st == 1u8 /*Ready*/ || st == 3u8 /*Standby*/ {
            crate::dbg_print!(
                b"[RQ62] KeReadyThread on state=%u thr=%p reason=%u\n\0",
                st as u32,
                Thread,
                (*Thread).WaitReason as u32,
            );
            if ntkrnl_sys::inline::ke::RQ_ORACLE_ARMED {
            ntkrnl_sys::ddk::KeBugCheckEx(
                ntkrnl_sys::bugcodes::INVALID_WORK_QUEUE_ITEM,
                Thread as ntkrnl_sys::ddk::ULONG_PTR,
                ((0x62u64 << 40) | (st as u64) << 8) as ntkrnl_sys::ddk::ULONG_PTR,
                0,
                0,
            );
            }
        }
    }

    KiReadyThread(Thread);

    KiReleaseDispatcherLock(OldIrql);
}

unsafe fn KeAlertResumeThread_impl(Thread: PKTHREAD) -> ULONG {
    let PreviousCount: ULONG;
    let mut ApcLock: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));
    KiAcquireDispatcherLockAtSynchLevel();

    if (*Thread).Alerted[0usize] == 0 {
        if *kt_State(Thread) == Waiting && (*Thread).Alertable != 0 {
            KiUnwaitThread(Thread, STATUS_ALERTED as LONG_PTR, THREAD_ALERT_INCREMENT);
        } else {
            (*Thread).Alerted[0usize] = 1;
        }
    }

    PreviousCount = *kt_SuspendCount(Thread) as ULONG;

    if PreviousCount != 0 {
        *kt_SuspendCount(Thread) -= 1;
        if *kt_SuspendCount(Thread) == 0 && *kt_FreezeCount(Thread) == 0 {
            kt_SuspendSemaphore(Thread).Header.SignalState += 1;
            KiWaitTest(core::ptr::addr_of_mut!(kt_SuspendSemaphore(Thread).Header) as *mut core::ffi::c_void, IO_NO_INCREMENT);
        }
    }

    KiReleaseDispatcherLockFromSynchLevel();
    KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));
    KiExitDispatcher(core::ptr::read_volatile(&ApcLock.OldIrql));
    PreviousCount
}

unsafe fn KeAlertThread_impl(Thread: PKTHREAD, AlertMode: KPROCESSOR_MODE) -> BOOLEAN {
    let PreviousState: BOOLEAN;
    let mut ApcLock: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));
    KiAcquireDispatcherLockAtSynchLevel();

    PreviousState = (*Thread).Alerted[AlertMode as usize];

    if PreviousState == 0 {
        if *kt_State(Thread) == Waiting && (*Thread).Alertable != 0 && AlertMode <= *kt_WaitMode(Thread) {
            KiUnwaitThread(Thread, STATUS_ALERTED as LONG_PTR, THREAD_ALERT_INCREMENT);
        } else {
            (*Thread).Alerted[AlertMode as usize] = 1;
        }
    }

    KiReleaseDispatcherLockFromSynchLevel();
    KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));
    KiExitDispatcher(core::ptr::read_volatile(&ApcLock.OldIrql));

    PreviousState
}

unsafe fn KeBoostPriorityThread_impl(Thread: PKTHREAD, Increment: KPRIORITY) {
    let OldIrql: KIRQL;
    let mut Priority: KPRIORITY;
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    OldIrql = KiAcquireDispatcherLock();

    if ((*Thread).Priority as KPRIORITY) < LOW_REALTIME_PRIORITY {
        KiAcquireThreadLock(Thread);

        if ((*Thread).Priority as KPRIORITY) < LOW_REALTIME_PRIORITY && (*Thread).PriorityDecrement == 0 {
            Priority = (*Thread).BasePriority as KPRIORITY + Increment;
            if Priority > (*Thread).Priority as KPRIORITY {
                if Priority >= LOW_REALTIME_PRIORITY {
                    Priority = LOW_REALTIME_PRIORITY - 1;
                }

                *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;

                KiSetPriorityThread(Thread, Priority);
            }
        }

        KiReleaseThreadLock(Thread);
    }

    KiReleaseDispatcherLock(OldIrql);
}

unsafe fn KeForceResumeThread_impl(Thread: PKTHREAD) -> ULONG {
    let mut ApcLock: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let PreviousCount: ULONG;
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));

    PreviousCount = (*kt_SuspendCount(Thread) as ULONG).wrapping_add(*kt_FreezeCount(Thread) as u8 as ULONG);

    if PreviousCount != 0 {
        *kt_SuspendCount(Thread) = 0;
        *kt_FreezeCount(Thread) = 0;

        KiAcquireDispatcherLockAtSynchLevel();

        kt_SuspendSemaphore(Thread).Header.SignalState += 1;
        KiWaitTest(core::ptr::addr_of_mut!(kt_SuspendSemaphore(Thread).Header) as *mut core::ffi::c_void, IO_NO_INCREMENT);

        KiReleaseDispatcherLockFromSynchLevel();
    }

    KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));
    KiExitDispatcher(core::ptr::read_volatile(&ApcLock.OldIrql));
    PreviousCount
}

unsafe fn KeFreezeAllThreads_impl() {
    let mut LockHandle: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut ApcLock: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut Current: PKTHREAD;
    let CurrentThread = PsGetCurrentThread();
    let Process = kt_ApcState(CurrentThread).Process;
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(LockHandle));

    while *kt_FreezeCount(CurrentThread) != 0 {
        KiReleaseProcessLock(core::ptr::addr_of_mut!(LockHandle));
        KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(LockHandle));
    }

    KeEnterCriticalRegion();

    ListHead = core::ptr::addr_of_mut!((*(Process as *mut _KPROCESS)).ThreadListHead);
    NextEntry = (*ListHead).Flink;
    loop {
        Current = ((NextEntry as usize) - core::mem::offset_of!(_KTHREAD, ThreadListEntry)) as PKTHREAD;

        KiAcquireApcLockAtSynchLevel(Current, core::ptr::addr_of_mut!(ApcLock));

        if Current != CurrentThread && *kt_ApcQueueable(Current) != 0 {
            ASSERT!(*kt_SuspendCount(Current) != MAXIMUM_SUSPEND_COUNT);

            *kt_FreezeCount(Current) += 1;

            if *kt_FreezeCount(Current) == 1 && *kt_SuspendCount(Current) == 0 {
                if kt_SuspendApc(Current).Inserted == 0 {
                    kt_SuspendApc(Current).Inserted = 1;
                    KiInsertQueueApc(kt_SuspendApc(Current) as *mut _KAPC, IO_NO_INCREMENT);
                } else {
                    KiAcquireDispatcherLockAtSynchLevel();

                    kt_SuspendSemaphore(Current).Header.SignalState -= 1;

                    KiReleaseDispatcherLockFromSynchLevel();
                }
            }
        }

        KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));

        NextEntry = (*NextEntry).Flink;
        if NextEntry == ListHead {
            break;
        }
    }

    KiReleaseProcessLockFromSynchLevel(core::ptr::addr_of_mut!(LockHandle));
    KiExitDispatcher(core::ptr::read_volatile(&LockHandle.OldIrql));
}

unsafe fn KeResumeThread_impl(Thread: PKTHREAD) -> ULONG {
    let mut ApcLock: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let PreviousCount: ULONG;
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));

    PreviousCount = *kt_SuspendCount(Thread) as u8 as ULONG;

    if PreviousCount != 0 {
        *kt_SuspendCount(Thread) -= 1;

        if *kt_SuspendCount(Thread) == 0 && *kt_FreezeCount(Thread) == 0 {
            KiAcquireDispatcherLockAtSynchLevel();

            kt_SuspendSemaphore(Thread).Header.SignalState += 1;
            KiWaitTest(core::ptr::addr_of_mut!(kt_SuspendSemaphore(Thread).Header) as *mut core::ffi::c_void, IO_NO_INCREMENT);

            KiReleaseDispatcherLockFromSynchLevel();
        }
    }

    KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));
    KiExitDispatcher(core::ptr::read_volatile(&ApcLock.OldIrql));
    PreviousCount
}

unsafe fn KeRundownThread_impl() {
    let OldIrql: KIRQL;
    let Thread = PsGetCurrentThread();
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut Mutant: PKMUTANT;
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    if IsListEmpty(core::ptr::addr_of_mut!((*Thread).MutantListHead)) {
        return;
    }

    OldIrql = KiAcquireDispatcherLock();

    ListHead = core::ptr::addr_of_mut!((*Thread).MutantListHead);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        Mutant = ((NextEntry as usize) - core::mem::offset_of!(_KMUTANT, MutantListEntry)) as PKMUTANT;
        ASSERT_MUTANT!(Mutant);

        if (*Mutant).ApcDisable != 0 {
            KeBugCheckEx(
                THREAD_TERMINATE_HELD_MUTEX,
                Thread as ULONG_PTR,
                Mutant as ULONG_PTR,
                0,
                0,
            );
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*Mutant).MutantListEntry));

        (*Mutant).Header.SignalState = 1;
        (*Mutant).Abandoned = 1;
        (*Mutant).OwnerThread = core::ptr::null_mut();

        if !IsListEmpty(core::ptr::addr_of_mut!((*Mutant).Header.WaitListHead)) {
            KiWaitTest(core::ptr::addr_of_mut!((*Mutant).Header) as *mut core::ffi::c_void, MUTANT_INCREMENT);
        }

        NextEntry = (*Thread).MutantListHead.Flink;
    }

    KiReleaseDispatcherLock(OldIrql);
}

unsafe fn KeStartThread_impl(Thread: PKTHREAD) {
    let mut LockHandle: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    #[cfg(target_arch = "x86")]
    let mut IdealProcessor: u8 = 0;
    #[cfg(not(target_arch = "x86"))]
    let IdealProcessor: u8;
    let Process = kt_ApcState(Thread).Process;

    // DisableBoost is a bitfield — copy ProcessFlags bitfield value
    let proc_disable_boost = (*(Process as *mut _KPROCESS)).__bindgen_anon_1.__bindgen_anon_1.DisableBoost();
    // On i386, ThreadFlags union is at __bindgen_anon_6; on amd64/arm64 it's at __bindgen_anon_4.__bindgen_anon_1.__bindgen_anon_1
    #[cfg(target_arch = "x86")]
    {
        (*Thread).__bindgen_anon_6.__bindgen_anon_1.set_DisableBoost(proc_disable_boost as u32);
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*Thread).__bindgen_anon_4.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.set_DisableBoost(proc_disable_boost as u32);
    }
    #[cfg(target_arch = "x86")]
    {
        // On i386 _KTHREAD has Iopl field, _KPROCESS has Iopl field
        (*Thread).Iopl = (*(Process as *mut _KPROCESS)).Iopl;
    }
    *kt_Quantum(Thread) = (*(Process as *mut _KPROCESS)).QuantumReset;
    *kt_QuantumReset(Thread) = (*(Process as *mut _KPROCESS)).QuantumReset as u8;
    *kt_SystemAffinityActive(Thread) = 0;

    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(LockHandle));

    (*Thread).Priority = (*(Process as *mut _KPROCESS)).BasePriority;
    (*Thread).BasePriority = (*(Process as *mut _KPROCESS)).BasePriority;
    (*Thread).Affinity = (*(Process as *mut _KPROCESS)).Affinity;
    (*Thread).UserAffinity = (*(Process as *mut _KPROCESS)).Affinity;

    #[cfg(not(target_arch = "x86"))]
    {
        let Node: PKNODE;
        let NodePrcb: PKPRCB;
        let mut Set: KAFFINITY;
        let mut Mask: KAFFINITY;

        Node = *(core::ptr::addr_of!(KeNodeBlock) as *const PKNODE).add((*(Process as *mut _KPROCESS)).IdealNode as usize);
        NodePrcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add((*(Process as *mut _KPROCESS)).ThreadSeed as usize);

        Set = !(*NodePrcb).MultiThreadProcessorSet;
        Mask = (*Node).ProcessorMask & (*(Process as *mut _KPROCESS)).Affinity;
        Set &= Mask;
        if Set != 0 {
            Mask = Set;
        }

        IdealProcessor = KeFindNextRightSetAffinity((*(Process as *mut _KPROCESS)).ThreadSeed, Mask);
        (*(Process as *mut _KPROCESS)).ThreadSeed = IdealProcessor;

        ASSERT!(((*Thread).UserAffinity & AFFINITY_MASK!(IdealProcessor)) != 0);
    }

    (*Thread).IdealProcessor = IdealProcessor;
    *kt_UserIdealProcessor(Thread) = IdealProcessor;

    KiAcquireDispatcherLockAtSynchLevel();

    InsertTailList(
        core::ptr::addr_of_mut!((*(Process as *mut _KPROCESS)).ThreadListHead),
        core::ptr::addr_of_mut!((*Thread).ThreadListEntry),
    );

    ASSERT!((*(Process as *mut _KPROCESS)).StackCount != u32::MAX);
    (*(Process as *mut _KPROCESS)).StackCount += 1;

    KiReleaseDispatcherLockFromSynchLevel();
    KiReleaseProcessLock(core::ptr::addr_of_mut!(LockHandle));
}

unsafe extern "system" fn KiSuspendRundown(Apc: PKAPC) {
    UNREFERENCED_PARAMETER!(Apc);
}

unsafe extern "system" fn KiSuspendNop(
    Apc: PKAPC,
    NormalRoutine: *mut *mut core::ffi::c_void,
    NormalContext: *mut PVOID,
    SystemArgument1: *mut PVOID,
    SystemArgument2: *mut PVOID,
) {
    UNREFERENCED_PARAMETER!(Apc);
    UNREFERENCED_PARAMETER!(NormalRoutine);
    UNREFERENCED_PARAMETER!(NormalContext);
    UNREFERENCED_PARAMETER!(SystemArgument1);
    UNREFERENCED_PARAMETER!(SystemArgument2);
}

unsafe extern "system" fn KiSuspendThread(
    NormalContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    UNREFERENCED_PARAMETER!(NormalContext);
    UNREFERENCED_PARAMETER!(SystemArgument1);
    UNREFERENCED_PARAMETER!(SystemArgument2);

    let thread = PsGetCurrentThread();
    KeWaitForSingleObject(
        kt_SuspendSemaphore(thread) as *mut _KSEMAPHORE as PVOID,
        _KWAIT_REASON_Suspended,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
}

unsafe fn KeSuspendThread_impl(Thread: PKTHREAD) -> ULONG {
    let mut ApcLock: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let PreviousCount: ULONG;
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));

    PreviousCount = *kt_SuspendCount(Thread) as u8 as ULONG;

    if *kt_SuspendCount(Thread) == MAXIMUM_SUSPEND_COUNT {
        KiReleaseApcLock(core::ptr::addr_of_mut!(ApcLock));
        RtlRaiseStatus(STATUS_SUSPEND_COUNT_EXCEEDED);
    }

    if *kt_ApcQueueable(Thread) != 0 {
        *kt_SuspendCount(Thread) += 1;

        if PreviousCount == 0 && *kt_FreezeCount(Thread) == 0 {
            if kt_SuspendApc(Thread).Inserted == 0 {
                kt_SuspendApc(Thread).Inserted = 1;
                KiInsertQueueApc(kt_SuspendApc(Thread) as *mut _KAPC, IO_NO_INCREMENT);
            } else {
                KiAcquireDispatcherLockAtSynchLevel();

                kt_SuspendSemaphore(Thread).Header.SignalState -= 1;

                KiReleaseDispatcherLockFromSynchLevel();
            }
        }
    }

    KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));
    KiExitDispatcher(core::ptr::read_volatile(&ApcLock.OldIrql));
    PreviousCount
}

unsafe fn KeThawAllThreads_impl() {
    let mut LockHandle: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut ApcLock: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut Current: PKTHREAD;
    let CurrentThread = PsGetCurrentThread();
    let Process = kt_ApcState(CurrentThread).Process;
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut OldCount: LONG;
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(LockHandle));

    ListHead = core::ptr::addr_of_mut!((*(Process as *mut _KPROCESS)).ThreadListHead);
    NextEntry = (*ListHead).Flink;
    loop {
        Current = ((NextEntry as usize) - core::mem::offset_of!(_KTHREAD, ThreadListEntry)) as PKTHREAD;

        KiAcquireApcLockAtSynchLevel(Current, core::ptr::addr_of_mut!(ApcLock));

        OldCount = *kt_FreezeCount(Current) as LONG;
        if OldCount != 0 {
            *kt_FreezeCount(Current) -= 1;

            if *kt_SuspendCount(Current) == 0 && *kt_FreezeCount(Current) == 0 {
                KiAcquireDispatcherLockAtSynchLevel();

                kt_SuspendSemaphore(Current).Header.SignalState += 1;
                KiWaitTest(core::ptr::addr_of_mut!(kt_SuspendSemaphore(Current).Header) as *mut core::ffi::c_void, 0);

                KiReleaseDispatcherLockFromSynchLevel();
            }
        }

        KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));

        NextEntry = (*NextEntry).Flink;
        if NextEntry == ListHead {
            break;
        }
    }

    KiReleaseProcessLockFromSynchLevel(core::ptr::addr_of_mut!(LockHandle));
    KiExitDispatcher(core::ptr::read_volatile(&LockHandle.OldIrql));

    KeLeaveCriticalRegion();
}

unsafe fn KeTestAlertThread_impl(AlertMode: KPROCESSOR_MODE) -> BOOLEAN {
    let Thread = PsGetCurrentThread();
    let OldState: BOOLEAN;
    let mut ApcLock: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));

    OldState = (*Thread).Alerted[AlertMode as usize];

    if OldState != 0 {
        (*Thread).Alerted[AlertMode as usize] = 0;
    } else if AlertMode != 0 && !IsListEmpty(core::ptr::addr_of_mut!(kt_ApcState(Thread).ApcListHead[1usize])) {
        kt_ApcState(Thread).UserApcPending = 1;
    }

    KiReleaseApcLock(core::ptr::addr_of_mut!(ApcLock));
    OldState
}

unsafe fn KeInitThread_impl(
    Thread: PKTHREAD,
    KernelStack: PVOID,
    SystemRoutine: *mut core::ffi::c_void,
    StartRoutine: *mut core::ffi::c_void,
    StartContext: PVOID,
    Context: *mut core::ffi::c_void,
    Teb: PVOID,
    Process: PKPROCESS,
) -> NTSTATUS {
    let mut AllocatedStack = 0u8;
    let TimerWaitBlock: PKWAIT_BLOCK;
    let Timer: PKTIMER;
    let mut Status: NTSTATUS;
    let mut ActualKernelStack = KernelStack;

    (*Thread).Header.__bindgen_anon_1.__bindgen_anon_1.Type = ThreadObject;
    (*Thread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.__bindgen_anon_1.ThreadControlFlags = 0;
    (*Thread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.__bindgen_anon_1.DebugActive = 0;
    (*Thread).Header.SignalState = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Thread).Header.WaitListHead));

    InitializeListHead(core::ptr::addr_of_mut!((*Thread).MutantListHead));

    for i in 0..(THREAD_WAIT_OBJECTS + 1) {
        kt_WaitBlock(Thread)[i].Thread = Thread;
    }

    (*Thread).EnableStackSwap = 1;
    (*Thread).IdealProcessor = 1;
    (*Thread).SwapBusy = 0;
    (*Thread).KernelStackResident = 1;
    *kt_AdjustReason(Thread) = AdjustNone;

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Thread).ThreadLock));

    (*Thread).ServiceTable = core::ptr::addr_of_mut!(KeServiceDescriptorTable) as *mut core::ffi::c_void;

    InitializeListHead(core::ptr::addr_of_mut!(kt_ApcState(Thread).ApcListHead[0usize]));
    InitializeListHead(core::ptr::addr_of_mut!(kt_ApcState(Thread).ApcListHead[1usize]));
    kt_ApcState(Thread).Process = Process;
    (*Thread).ApcStatePointer[OriginalApcEnvironment] = kt_ApcState(Thread) as *mut ntkrnl_sys::ddk::_KAPC_STATE;
    (*Thread).ApcStatePointer[AttachedApcEnvironment] = kt_SavedApcState(Thread) as *mut ntkrnl_sys::ddk::_KAPC_STATE;
    (*Thread).ApcStateIndex = OriginalApcEnvironment as u8;
    *kt_ApcQueueable(Thread) = 1;
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Thread).ApcQueueLock));

    KeInitializeApc(
        kt_SuspendApc(Thread) as *mut _KAPC,
        Thread,
        OriginalApcEnvironment as u32,
        core::mem::transmute(KiSuspendNop as usize),
        core::mem::transmute(KiSuspendRundown as usize),
        core::mem::transmute(KiSuspendThread as usize),
        0i8,
        core::ptr::null_mut(),
    );

    KeInitializeSemaphore(kt_SuspendSemaphore(Thread) as *mut _KSEMAPHORE, 0, 2);

    Timer = kt_Timer(Thread) as *mut _KTIMER;
    KeInitializeTimer(Timer);
    TimerWaitBlock = &mut kt_WaitBlock(Thread)[TIMER_WAIT_BLOCK] as *mut _KWAIT_BLOCK;
    (*TimerWaitBlock).Object = Timer as PVOID;
    (*TimerWaitBlock).WaitKey = STATUS_TIMEOUT as USHORT;
    (*TimerWaitBlock).WaitType = WaitAny as u8;
    (*TimerWaitBlock).NextWaitBlock = core::ptr::null_mut();

    (*TimerWaitBlock).WaitListEntry.Flink = core::ptr::addr_of_mut!((*Timer).Header.WaitListHead);
    (*TimerWaitBlock).WaitListEntry.Blink = core::ptr::addr_of_mut!((*Timer).Header.WaitListHead);

    (*Thread).Teb = Teb as *mut ntkrnl_sys::ddk::_TEB;
    (*Thread).Process = Process;

    if ActualKernelStack.is_null() {
        ActualKernelStack = MmCreateKernelStack(0u8, 0);
        if ActualKernelStack.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        AllocatedStack = 1;
    }

    (*Thread).InitialStack = ActualKernelStack;
    (*Thread).StackBase = ActualKernelStack;
    #[cfg(target_arch = "aarch64")]
    {
        (*Thread).StackLimit = (ActualKernelStack as usize - MmLargeStackSize as usize) as _;
        *kt_LargeStack(Thread) = 1;
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        (*Thread).StackLimit = (ActualKernelStack as usize - KERNEL_STACK_SIZE) as _;
    }
    (*Thread).KernelStackResident = 1;

    Status = STATUS_SUCCESS;
    let result = 'seh: {
        KiInitializeContextThread(
            Thread,
            core::mem::transmute::<*mut core::ffi::c_void, ntkrnl_sys::ddk::PKSYSTEM_ROUTINE>(SystemRoutine),
            core::mem::transmute::<*mut core::ffi::c_void, ntkrnl_sys::ddk::PKSTART_ROUTINE>(StartRoutine),
            StartContext,
            Context as ntkrnl_sys::ddk::PCONTEXT,
        );
        STATUS_SUCCESS
    };

    if result < 0 {
        Status = STATUS_UNSUCCESSFUL_;
        if AllocatedStack != 0 {
            MmDeleteKernelStack((*Thread).StackBase, *kt_LargeStack(Thread));
            (*Thread).InitialStack = core::ptr::null_mut();
        }
    }

    *kt_State(Thread) = Initialized;
    Status
}

unsafe fn KeInitializeThread_impl(
    Process: PKPROCESS,
    Thread: PKTHREAD,
    SystemRoutine: *mut core::ffi::c_void,
    StartRoutine: *mut core::ffi::c_void,
    StartContext: PVOID,
    Context: *mut core::ffi::c_void,
    Teb: PVOID,
    KernelStack: PVOID,
) {
    let status = KeInitThread_impl(Thread, KernelStack, SystemRoutine, StartRoutine, StartContext, Context, Teb, Process);
    if status >= 0 {
        KeStartThread_impl(Thread);
    }
}

unsafe fn KeUninitThread_impl(Thread: PKTHREAD) {
    MmDeleteKernelStack((*Thread).StackBase, *kt_LargeStack(Thread));
    (*Thread).InitialStack = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn KeCapturePersistentThreadState(
    CurrentThread: PVOID,
    Setting1: ULONG,
    Setting2: ULONG,
    Setting3: ULONG,
    Setting4: ULONG,
    Setting5: ULONG,
    ThreadState: PVOID,
) {
    let CurrentThread = opaque_ptr(CurrentThread);
    let ThreadState = opaque_ptr(ThreadState);
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentThread() -> PKTHREAD {
    _KeGetCurrentThread()
}

#[no_mangle]
pub unsafe extern "system" fn KeGetPreviousMode() -> u8 {
    _KeGetPreviousMode()
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryRuntimeThread(
    Thread: PKTHREAD,
    UserTime: *mut ULONG,
) -> ULONG {
    let Thread = opaque_ptr(Thread);
    let UserTime = opaque_ptr(UserTime);
    ASSERT_THREAD!(Thread);

    *UserTime = *kt_UserTime(Thread);

    *kt_KernelTime(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeSetKernelStackSwapEnable(Enable: BOOLEAN) -> BOOLEAN {
    let PreviousState: BOOLEAN;
    let Thread = PsGetCurrentThread();

    PreviousState = (*Thread).EnableStackSwap;

    (*Thread).EnableStackSwap = Enable;

    PreviousState
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryPriorityThread(Thread: PKTHREAD) -> KPRIORITY {
    let Thread = opaque_ptr(Thread);
    ASSERT_THREAD!(Thread);

    (*Thread).Priority as KPRIORITY
}

#[no_mangle]
pub unsafe extern "system" fn KeRevertToUserAffinityThread() {
    let OldIrql: KIRQL;
    let Prcb: PKPRCB;
    let mut NextThread: PKTHREAD;
    let CurrentThread = PsGetCurrentThread();
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);
    ASSERT!(*kt_SystemAffinityActive(CurrentThread) != 0);

    OldIrql = KiAcquireDispatcherLock();

    (*CurrentThread).Affinity = (*CurrentThread).UserAffinity;
    (*CurrentThread).IdealProcessor = *kt_UserIdealProcessor(CurrentThread);
    *kt_SystemAffinityActive(CurrentThread) = 0;

    Prcb = KeGetCurrentPrcb();
    if ((*Prcb).SetMember & (*CurrentThread).Affinity) == 0 {
        KiAcquirePrcbLock(Prcb);

        if (*Prcb).NextThread.is_null() {
            NextThread = KiSelectNextThread(Prcb);
            *kt_State(NextThread) = Standby;
            (*Prcb).NextThread = NextThread;
        }

        KiReleasePrcbLock(Prcb);
    }

    KiReleaseDispatcherLock(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn KeSetIdealProcessorThread(
    Thread: PKTHREAD,
    Processor: u8,
) -> u8 {
    let Thread = opaque_ptr(Thread);
    let OldIdealProcessor: u8;
    let OldIrql: KIRQL;
    ASSERT!(Processor as ULONG <= 64);

    OldIrql = KiAcquireDispatcherLock();

    OldIdealProcessor = *kt_UserIdealProcessor(Thread);

    if (Processor as ULONG) < KeNumberProcessors as ULONG {
        if (*Thread).Affinity & AFFINITY_MASK!(Processor) != 0 {
            (*Thread).IdealProcessor = Processor;

            if *kt_SystemAffinityActive(Thread) == 0 {
                *kt_UserIdealProcessor(Thread) = Processor;
            }
        }
    }

    KiReleaseDispatcherLock(OldIrql);
    OldIdealProcessor
}

#[no_mangle]
pub unsafe extern "system" fn KeSetSystemAffinityThread(Affinity: KAFFINITY) {
    // [AFFPROBE task#10] capture caller x30 FIRST (before PsGetCurrentThread's bl)
    // so we can pin who passes an affinity excluding all active CPUs.
    //
    // 2026-07-25 — READ THIS BEFORE TRUSTING THIS PROBE'S SILENCE. It was added to
    // hunt the "arm64 BUGCHECK 0x3 INVALID_AFFINITY_SET" flake, and it never fired.
    // That was recorded as "the bad mask reaches the scheduler by another route."
    // It is not: there was never a bad mask. Until commit d3ca121cb09 the ONLY two
    // sites in the whole Rust kernel that could raise stop code 3 were in
    // ex-rust/src/resource.rs (RESOURCE_NOT_OWNED, whose number the LLM had invented
    // as 0x03 — field manual §9.113); KiSetAffinityThread was raising 0x48. The one
    // C raiser, KeBugCheck(INVALID_AFFINITY_SET), passes all-zero parameters, and
    // every historical hit carried FOUR non-zero ones matching
    // ExReleaseResourceForThreadLite's (Resource, Thread, OwnerTable, 3) exactly.
    // So the flake is an ERESOURCE ownership bug and this probe is aimed at the
    // wrong subsystem. Kept (it is nearly free, and post-d3ca121cb09
    // KiSetAffinityThread IS a live 0x3 candidate again) but its silence proves
    // nothing about the historical hits. Full reasoning: field manual §9.124.
    #[cfg(target_arch = "aarch64")]
    let caller_lr: ULONG_PTR = {
        let lr: ULONG_PTR;
        core::arch::asm!("mov {}, x30", out(reg) lr, options(nomem, nostack));
        lr
    };
    let OldIrql: KIRQL;
    let Prcb: PKPRCB;
    let mut NextThread: PKTHREAD;
    let CurrentThread = PsGetCurrentThread();
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);
    ASSERT!((Affinity & KeActiveProcessors) != 0);
    #[cfg(target_arch = "aarch64")]
    if (Affinity & KeActiveProcessors) == 0 {
        extern "C" {
            fn ARM64_UART_TRACE_HEX(prefix: *const u8, val: ULONG_PTR);
        }
        ARM64_UART_TRACE_HEX(b"[AFFPROBE] KeSetSystemAffinityThread BAD lr=\0".as_ptr(), caller_lr);
        ARM64_UART_TRACE_HEX(b"[AFFPROBE]   Affinity=\0".as_ptr(), Affinity as ULONG_PTR);
        ARM64_UART_TRACE_HEX(b"[AFFPROBE]   KeActiveProcessors=\0".as_ptr(), KeActiveProcessors as ULONG_PTR);
    }

    OldIrql = KiAcquireDispatcherLock();

    (*CurrentThread).Affinity = Affinity;
    *kt_SystemAffinityActive(CurrentThread) = 1;

    // thrdobj.c: `#ifdef CONFIG_SMP` — a BUILD option, not an arch. Gated `not(x86)` it
    // ran on UP amd64/arm64 and would have been skipped by the i386 SMP kernel.
    #[cfg(feature = "smp")]
    {
        (*CurrentThread).IdealProcessor = KeFindNextRightSetAffinity((*CurrentThread).IdealProcessor, Affinity);
    }

    Prcb = KeGetCurrentPrcb();
    if ((*Prcb).SetMember & (*CurrentThread).Affinity) == 0 {
        KiAcquirePrcbLock(Prcb);

        if (*Prcb).NextThread.is_null() {
            NextThread = KiSelectNextThread(Prcb);
            *kt_State(NextThread) = Standby;
            (*Prcb).NextThread = NextThread;
        }

        KiReleasePrcbLock(Prcb);
    }

    KiReleaseDispatcherLock(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn KeSetBasePriorityThread(
    Thread: PKTHREAD,
    Increment: LONG,
) -> LONG {
    let Thread = opaque_ptr(Thread);
    let OldIrql: KIRQL;
    let OldBasePriority: KPRIORITY;
    let mut Priority: KPRIORITY;
    let mut BasePriority: KPRIORITY;
    let mut OldIncrement: LONG;
    let Process: PKPROCESS;
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    Process = kt_ApcState(Thread).Process;

    OldIrql = KiAcquireDispatcherLock();

    KiAcquireThreadLock(Thread);

    OldBasePriority = (*Thread).BasePriority as KPRIORITY;
    OldIncrement = OldBasePriority as LONG - (*(Process as *mut _KPROCESS)).BasePriority as LONG;

    if (*Thread).Saturation != 0 {
        OldIncrement = ((HIGH_PRIORITY + 1) / 2) * (*Thread).Saturation as LONG;
    }

    (*Thread).Saturation = 0;

    if Increment.abs() >= ((HIGH_PRIORITY + 1) / 2) as LONG {
        (*Thread).Saturation = if Increment > 0 { 1 } else { -1 };
    }

    BasePriority = ((*(Process as *mut _KPROCESS)).BasePriority as LONG + Increment) as KPRIORITY;
    if (*(Process as *mut _KPROCESS)).BasePriority as KPRIORITY >= LOW_REALTIME_PRIORITY {
        if BasePriority < LOW_REALTIME_PRIORITY {
            BasePriority = LOW_REALTIME_PRIORITY;
        }

        if BasePriority > HIGH_PRIORITY {
            BasePriority = HIGH_PRIORITY;
        }

        Priority = BasePriority;
    } else {
        if BasePriority >= LOW_REALTIME_PRIORITY {
            BasePriority = LOW_REALTIME_PRIORITY - 1;
        }

        if BasePriority <= LOW_PRIORITY {
            BasePriority = 1;
        }

        if (*Thread).Saturation != 0 {
            Priority = BasePriority;
        } else {
            Priority = KiComputeNewPriority(Thread, 0) as KPRIORITY;
            Priority += BasePriority - OldBasePriority;

            if Priority >= LOW_REALTIME_PRIORITY {
                Priority = LOW_REALTIME_PRIORITY - 1;
            } else if Priority <= LOW_PRIORITY {
                Priority = 1;
            }
        }
    }

    (*Thread).BasePriority = BasePriority as i8;

    (*Thread).PriorityDecrement = 0;

    if Priority != (*Thread).Priority as KPRIORITY {
        *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;
        KiSetPriorityThread(Thread, Priority);
    }

    KiReleaseThreadLock(Thread);

    KiReleaseDispatcherLock(OldIrql);
    OldIncrement
}

#[no_mangle]
pub unsafe extern "system" fn KeSetAffinityThread(
    Thread: PKTHREAD,
    Affinity: KAFFINITY,
) -> KAFFINITY {
    let Thread = opaque_ptr(Thread);
    let OldIrql: KIRQL;
    let OldAffinity: KAFFINITY;
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    OldIrql = KiAcquireDispatcherLock();

    OldAffinity = KiSetAffinityThread(Thread, Affinity);

    KiReleaseDispatcherLock(OldIrql);
    OldAffinity
}

#[no_mangle]
pub unsafe extern "system" fn KeSetPriorityThread(
    Thread: PKTHREAD,
    Priority: KPRIORITY,
) -> KPRIORITY {
    let Thread = opaque_ptr(Thread);
    let OldIrql: KIRQL;
    let OldPriority: KPRIORITY;
    ASSERT_THREAD!(Thread);
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);
    ASSERT!(Priority <= HIGH_PRIORITY && Priority >= LOW_PRIORITY);
    ASSERT!(KeIsExecutingDpc() == 0);

    // DBG: KeSetPriorityThread entry

    OldIrql = KiAcquireDispatcherLock();

    // DBG: after KiAcquireDispatcherLock

    KiAcquireThreadLock(Thread);

    OldPriority = (*Thread).Priority as KPRIORITY;
    (*Thread).PriorityDecrement = 0;

    if Priority != (*Thread).Priority as KPRIORITY {
        *kt_Quantum(Thread) = *kt_QuantumReset(Thread) as i8;

        let mut AdjustedPriority = Priority;
        if (*Thread).BasePriority != 0 && AdjustedPriority == 0 {
            AdjustedPriority = 1;
        }

        // DBG: before KiSetPriorityThread

        KiSetPriorityThread(Thread, AdjustedPriority);

        // DBG: after KiSetPriorityThread
    }

    KiReleaseThreadLock(Thread);

    // DBG: before KiReleaseDispatcherLock

    KiReleaseDispatcherLock(OldIrql);

    // DBG: after KiReleaseDispatcherLock

    OldPriority
}

#[no_mangle]
pub unsafe extern "system" fn KeTerminateThread(Increment: KPRIORITY) {
    let mut ListHead: *mut PLIST_ENTRY;
    let mut Entry: PETHREAD;
    let mut SavedEntry: PETHREAD;
    let mut ThreadAddr: *mut PETHREAD;
    let mut LockHandle: _KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let Thread = PsGetCurrentThread();
    let Process = kt_ApcState(Thread).Process;
    ASSERT_IRQL_LESS_OR_EQUAL!(DISPATCH_LEVEL);

    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(LockHandle));

    KiSetThreadSwapBusy(Thread);

    (*(Process as *mut _KPROCESS)).KernelTime += *kt_KernelTime(Thread);
    (*(Process as *mut _KPROCESS)).UserTime += *kt_UserTime(Thread);

    Entry = PspReaperListHead.Flink as PETHREAD;
    ThreadAddr = core::ptr::addr_of_mut!((*((Thread as PETHREAD) as *mut _ETHREAD)).__bindgen_anon_3.ReaperLink);

    loop {
        ListHead = core::ptr::addr_of_mut!(PspReaperListHead.Flink);

        *ThreadAddr = Entry;
        SavedEntry = Entry;

        Entry = InterlockedCompareExchangePointer(
            ListHead as *mut PVOID,
            ThreadAddr as PVOID,
            Entry as PVOID,
        ) as PETHREAD;

        if Entry == SavedEntry {
            break;
        }
    }

    KiAcquireDispatcherLockAtSynchLevel();

    if SavedEntry.is_null() {
        KiInsertQueue(
            core::ptr::addr_of_mut!(ExWorkerQueue[HyperCriticalWorkQueue].WorkerQueue),
            core::ptr::addr_of_mut!(PspReaperWorkItem.List) as PLIST_ENTRY,
            0u8,
        );
    }

    if !(*Thread).Queue.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*Thread).QueueListEntry));
        KiActivateWaiterQueue((*Thread).Queue);
    }

    (*Thread).Header.SignalState = 1;
    if !IsListEmpty(core::ptr::addr_of_mut!((*Thread).Header.WaitListHead)) {
        // KxUnwaitThread — inline helper, call the ke_defs version
        KxUnwaitThread_impl(core::ptr::addr_of_mut!((*Thread).Header), Increment);
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*Thread).ThreadListEntry));

    KiReleaseProcessLockFromSynchLevel(core::ptr::addr_of_mut!(LockHandle));

    *kt_State(Thread) = Terminated;

    ASSERT!((*(Process as *mut _KPROCESS)).StackCount != 0);
    ASSERT!((*(Process as *mut _KPROCESS)).State == ProcessInMemory_);
    (*(Process as *mut _KPROCESS)).StackCount -= 1;
    if (*(Process as *mut _KPROCESS)).StackCount == 0 && !IsListEmpty(core::ptr::addr_of_mut!((*(Process as *mut _KPROCESS)).ThreadListHead)) {
    }

    KiRundownThread(Thread);

    KiReleaseDispatcherLockFromSynchLevel();
    KiSwapThread(Thread, KeGetCurrentPrcb());
}

// ============================================================================
// Wrapper #[no_mangle] exports that delegate to _impl functions
// (these names collide with the ddk extern declarations, so we use _impl
//  for the actual logic and export the names here)
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn KeReadStateThread(Thread: PKTHREAD) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    KeReadStateThread_impl(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryBasePriorityThread(Thread: PKTHREAD) -> KPRIORITY {
    let Thread = opaque_ptr(Thread);
    KeQueryBasePriorityThread_impl(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeSetDisableBoostThread(Thread: PKTHREAD, Disable: BOOLEAN) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    KeSetDisableBoostThread_impl(Thread, Disable)
}

#[no_mangle]
pub unsafe extern "system" fn KeReadyThread(Thread: PKTHREAD) {
    let Thread = opaque_ptr(Thread);
    KeReadyThread_impl(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeAlertResumeThread(Thread: PKTHREAD) -> ULONG {
    let Thread = opaque_ptr(Thread);
    KeAlertResumeThread_impl(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeAlertThread(Thread: PKTHREAD, AlertMode: KPROCESSOR_MODE) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    KeAlertThread_impl(Thread, AlertMode)
}

#[no_mangle]
pub unsafe extern "system" fn KeBoostPriorityThread(Thread: PKTHREAD, Increment: KPRIORITY) {
    let Thread = opaque_ptr(Thread);
    KeBoostPriorityThread_impl(Thread, Increment)
}

#[no_mangle]
pub unsafe extern "system" fn KeForceResumeThread(Thread: PKTHREAD) -> ULONG {
    let Thread = opaque_ptr(Thread);
    KeForceResumeThread_impl(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeFreezeAllThreads() {
    KeFreezeAllThreads_impl()
}

#[no_mangle]
pub unsafe extern "system" fn KeResumeThread(Thread: PKTHREAD) -> ULONG {
    let Thread = opaque_ptr(Thread);
    KeResumeThread_impl(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeRundownThread() {
    KeRundownThread_impl()
}

#[no_mangle]
pub unsafe extern "system" fn KeStartThread(Thread: PKTHREAD) {
    let Thread = opaque_ptr(Thread);
    KeStartThread_impl(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeSuspendThread(Thread: PKTHREAD) -> ULONG {
    let Thread = opaque_ptr(Thread);
    KeSuspendThread_impl(Thread)
}

#[no_mangle]
pub unsafe extern "system" fn KeThawAllThreads() {
    KeThawAllThreads_impl()
}

#[no_mangle]
pub unsafe extern "system" fn KeTestAlertThread(AlertMode: KPROCESSOR_MODE) -> BOOLEAN {
    KeTestAlertThread_impl(AlertMode)
}

#[no_mangle]
pub unsafe extern "system" fn KeInitThread(
    Thread: PKTHREAD,
    KernelStack: PVOID,
    SystemRoutine: *mut core::ffi::c_void,
    StartRoutine: *mut core::ffi::c_void,
    StartContext: PVOID,
    Context: *mut core::ffi::c_void,
    Teb: PVOID,
    Process: PKPROCESS,
) -> NTSTATUS {
    let Thread = opaque_ptr(Thread);
    let KernelStack = opaque_ptr(KernelStack);
    let SystemRoutine = opaque_ptr(SystemRoutine);
    let StartRoutine = opaque_ptr(StartRoutine);
    let StartContext = opaque_ptr(StartContext);
    let Context = opaque_ptr(Context);
    let Teb = opaque_ptr(Teb);
    let Process = opaque_ptr(Process);
    KeInitThread_impl(Thread, KernelStack, SystemRoutine, StartRoutine, StartContext, Context, Teb, Process)
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeThread(
    Process: PKPROCESS,
    Thread: PKTHREAD,
    SystemRoutine: *mut core::ffi::c_void,
    StartRoutine: *mut core::ffi::c_void,
    StartContext: PVOID,
    Context: *mut core::ffi::c_void,
    Teb: PVOID,
    KernelStack: PVOID,
) {
    let Process = opaque_ptr(Process);
    let Thread = opaque_ptr(Thread);
    let SystemRoutine = opaque_ptr(SystemRoutine);
    let StartRoutine = opaque_ptr(StartRoutine);
    let StartContext = opaque_ptr(StartContext);
    let Context = opaque_ptr(Context);
    let Teb = opaque_ptr(Teb);
    let KernelStack = opaque_ptr(KernelStack);
    KeInitializeThread_impl(Process, Thread, SystemRoutine, StartRoutine, StartContext, Context, Teb, KernelStack)
}

#[no_mangle]
pub unsafe extern "system" fn KeUninitThread(Thread: PKTHREAD) {
    let Thread = opaque_ptr(Thread);
    KeUninitThread_impl(Thread)
}
