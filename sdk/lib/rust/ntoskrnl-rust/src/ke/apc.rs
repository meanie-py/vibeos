use super::ke_defs::*;
use ntkrnl_sys::inline::ke::KiInsertDeferredReadyList; // §9.56: inline (not the extern-C shim) so #[track_caller] names the true site
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _KAPC,
    _KAPC_STATE,
    _KTHREAD,
    KAPC_ENVIRONMENT,
    PKAPC,
    PKAPC_STATE,
    PKEXCEPTION_FRAME,
    PKKERNEL_ROUTINE,
    PKNORMAL_ROUTINE,
    PKRUNDOWN_ROUTINE,
    PKTHREAD,
    PKTRAP_FRAME,
    HalRequestSoftwareInterrupt,
    KeEnterCriticalRegion,
    KeLeaveCriticalRegion,
    KeTestAlertThread,
    KiAcquireApcLockRaiseToDpc,
    KiAcquireApcLockRaiseToSynch,
    KiAcquireDispatcherLockAtSynchLevel,
    KiAcquireDispatcherObject,
    KiAcquireThreadLock,
    KiExitDispatcher,
    KiInitializeUserApc,
    KiReleaseApcLock,
    KiReleaseApcLockFromSynchLevel,
    KiReleaseDispatcherLockFromSynchLevel,
    KiReleaseDispatcherObject,
    KiReleaseThreadLock,
    KiRequestApcInterrupt,
    PsExitSpecialApc,
};

// KiDeliverApc is provided by Rust on all architectures (apc.c is commented out).
// On amd64, trap.S calls KiDeliverApc as an extern — we must export it.

// KAPC_ENVIRONMENT values (u32 enum in bindgen)
const KAPC_ENV_CURRENT: u32 = 2;   // CurrentApcEnvironment
const KAPC_ENV_INSERT: u32 = 3;    // InsertApcEnvironment
// As CCHAR (i8) for comparison with ApcStateIndex
const INSERT_APC_ENV_I8: i8 = 3;

const THREAD_STATE_RUNNING: u8 = 2;
const THREAD_STATE_WAITING: u8 = 5;
const THREAD_STATE_GATE_WAIT: u8 = 8;
const APC_OBJECT_TYPE: u8 = 0x12;
// xdk/ketypes.h: FIELD_OFFSET(KAPC_STATE, UserApcPending) + sizeof(BOOLEAN) — the length
// WITHOUT tail padding (64-bit 43, i386 23). It must not be size_of (48 / 24): KTHREAD
// overlays FreezeCount/SuspendCount on SavedApcState's padding and Priority & co. on
// ApcState's, so a padded KiMoveApcState copy on every attach/detach wrote ApcState's tail
// into the suspend counts. The thread then read as suspended, and win32k refused every
// SendMessage to it ("Sending to Suspended Thread"): the console-window NCCREATE hang.
const KAPC_STATE_ACTUAL_LENGTH: usize =
    core::mem::offset_of!(_KAPC_STATE, UserApcPending) + core::mem::size_of::<BOOLEAN>();
const IRQL_UNEXPECTED_VALUE: u32 = 0x0000_00C8;

unsafe fn containing_record_apc(entry: PLIST_ENTRY) -> PKAPC {
    let field_offset = core::mem::offset_of!(_KAPC, ApcListEntry);
    (entry as *const u8).sub(field_offset) as PKAPC
}

/// Helper to cast Option<fn> to usize for KeBugCheckEx parameters.
#[inline(always)]
unsafe fn opt_fn_as_uptr<T>(opt: Option<T>) -> ULONG_PTR {
    // Option<fn> has the same size as a pointer
    core::mem::transmute_copy::<Option<T>, usize>(&opt)
}

#[no_mangle]
pub unsafe extern "system" fn KiCheckForKernelApcDelivery() {
    let mut OldIrql: KIRQL = 0;

    if KeGetCurrentIrql() == PASSIVE_LEVEL {
        // [guard] KeRaiseIrql(APC_LEVEL, &mut OldIrql);
        let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL) };
        KiDeliverApc(0i8, core::ptr::null_mut(), core::ptr::null_mut());
        drop(irql_guard);
    } else {
        let thread = PsGetCurrentThread();
        kt_ApcState(thread).KernelApcPending = 1u8;
        HalRequestSoftwareInterrupt(APC_LEVEL);
    }
}

// KiInsertQueueApc — FASTCALL on x86, extern "C" on other arches
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiInsertQueueApc(Apc: PKAPC, PriorityBoost: i32) {
    KiInsertQueueApcImpl(Apc, PriorityBoost);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiInsertQueueApc(Apc: PKAPC, PriorityBoost: i32) {
    KiInsertQueueApcImpl(Apc, PriorityBoost);
}

unsafe fn KiInsertQueueApcImpl(Apc: PKAPC, PriorityBoost: i32) {
    let Thread = (*Apc).Thread;
    // waitdep forensic (arm64 TDI-window flake): a stale/freed KAPC with
    // Thread=-1 reaching the unwait below is the remaining uncovered funnel
    // (the wait-list consume paths are all check_waittest'd and stayed silent
    // across 3 live hits). Dump the APC identity BEFORE the deref cascade —
    // KernelRoutine/RundownRoutine symbolicate straight to whoever built the
    // APC, and the APC address feeds pool-tag/identify-object.
    if (Thread as usize as isize) >= 0 || (Thread as usize) & (core::mem::align_of::<usize>() - 1) != 0 {
        crate::dbg_print!(
            b"[WAITDEP] bad APC.Thread: apc=%p thread=%p kroutine=%p nroutine=%p rroutine=%p mode=%u inserted=%u\n\0",
            Apc as *const (),
            Thread as *const (),
            core::mem::transmute::<_, *const ()>((*Apc).KernelRoutine),
            core::mem::transmute::<_, *const ()>((*Apc).NormalRoutine),
            core::mem::transmute::<_, *const ()>((*Apc).RundownRoutine),
            (*Apc).ApcMode as u32,
            (*Apc).Inserted as u32,
        );
    }
    let ApcState: PKAPC_STATE;
    let ApcMode: i8;
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut QueuedApc: PKAPC;
    let Gate: *mut ntkrnl_sys::ddk::_KGATE;
    let Status: isize;
    let mut RequestInterrupt = 0u8;

    if (*Apc).ApcStateIndex == INSERT_APC_ENV_I8 {
        (*Apc).ApcStateIndex = (*Thread).ApcStateIndex as i8;
    }

    ApcState = (*Thread).ApcStatePointer[(*Apc).ApcStateIndex as usize];
    ApcMode = (*Apc).ApcMode;

    if (*Apc).NormalRoutine.is_some() {
        if ApcMode != 0i8 && (*Apc).KernelRoutine == Some(PsExitSpecialApc) {
            kt_ApcState(Thread).UserApcPending = 1u8;
            InsertHeadList(
                &mut (*ApcState).ApcListHead[ApcMode as usize] as *mut LIST_ENTRY,
                &mut (*Apc).ApcListEntry as *mut LIST_ENTRY,
            );
        } else {
            InsertTailList(
                &mut (*ApcState).ApcListHead[ApcMode as usize] as *mut LIST_ENTRY,
                &mut (*Apc).ApcListEntry as *mut LIST_ENTRY,
            );
        }
    } else {
        ListHead = &mut (*ApcState).ApcListHead[ApcMode as usize] as *mut LIST_ENTRY;
        NextEntry = (*ListHead).Blink;
        while NextEntry != ListHead {
            QueuedApc = containing_record_apc(NextEntry);
            if (*QueuedApc).NormalRoutine.is_none() {
                break;
            }
            NextEntry = (*NextEntry).Blink;
        }
        InsertHeadList(NextEntry, &mut (*Apc).ApcListEntry as *mut LIST_ENTRY);
    }

    if (*Thread).ApcStateIndex as i8 == (*Apc).ApcStateIndex {
        if Thread == PsGetCurrentThread() {
            if ApcMode == 0i8 {
                kt_ApcState(Thread).KernelApcPending = 1u8;
                if *kt_SpecialApcDisable(Thread) == 0 {
                    HalRequestSoftwareInterrupt(APC_LEVEL);
                }
            }
        } else {
            KiAcquireDispatcherLock();

            if ApcMode == 0i8 {
                kt_ApcState(Thread).KernelApcPending = 1u8;

                if *kt_State(Thread) == THREAD_STATE_RUNNING {
                    RequestInterrupt = 1u8;
                } else if *kt_State(Thread) == THREAD_STATE_WAITING
                    && *kt_WaitIrql(Thread) == PASSIVE_LEVEL
                    && *kt_SpecialApcDisable(Thread) == 0
                    && ((*Apc).NormalRoutine.is_none()
                        || (*kt_KernelApcDisable(Thread) == 0
                            && kt_ApcState(Thread).KernelApcInProgress == 0u8))
                {
                    Status = STATUS_KERNEL_APC as isize;
                    KiUnwaitThread(Thread, Status, PriorityBoost);
                } else if *kt_State(Thread) == THREAD_STATE_GATE_WAIT {
                    KiAcquireThreadLock(Thread);

                    if *kt_State(Thread) == THREAD_STATE_GATE_WAIT
                        && *kt_WaitIrql(Thread) == PASSIVE_LEVEL
                        && *kt_SpecialApcDisable(Thread) == 0
                        && ((*Apc).NormalRoutine.is_none()
                            || (*kt_KernelApcDisable(Thread) == 0
                                && kt_ApcState(Thread).KernelApcInProgress == 0u8))
                    {
                        Gate = kt_GateObject(Thread);
                        KiAcquireDispatcherObject(&mut (*Gate).Header);
                        RemoveEntryList(&mut kt_WaitBlock(Thread)[0].WaitListEntry as *mut LIST_ENTRY);
                        KiReleaseDispatcherObject(&mut (*Gate).Header);

                        if !(*Thread).Queue.is_null() {
                            (*((*Thread).Queue)).CurrentCount += 1;
                        }

                        (*Thread).WaitStatus = STATUS_KERNEL_APC as _;
                        KiInsertDeferredReadyList(Thread);
                    }

                    KiReleaseThreadLock(Thread);
                }
            } else if *kt_State(Thread) == THREAD_STATE_WAITING
                && *kt_WaitMode(Thread) == 1i8
                && ((*Thread).Alertable != 0u8
                    || kt_ApcState(Thread).UserApcPending != 0u8)
            {
                kt_ApcState(Thread).UserApcPending = 1u8;
                Status = STATUS_USER_APC as isize;
                KiUnwaitThread(Thread, Status, PriorityBoost);
            }

            KiReleaseDispatcherLockFromSynchLevel();
            KiRequestApcInterrupt(RequestInterrupt, *kt_NextProcessor(Thread));
        }
    }
}

// KiDeliverApc — provided by arch-specific assembly on x86_64; Rust on x86/aarch64
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KiDeliverApc(
    DeliveryMode: i8,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
) {
    KiDeliverApcImpl(DeliveryMode, ExceptionFrame, TrapFrame);
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KiDeliverApc(
    DeliveryMode: i8,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
) {
    KiDeliverApcImpl(DeliveryMode, ExceptionFrame, TrapFrame);
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KiDeliverApc(
    DeliveryMode: i8,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
) {
    KiDeliverApcImpl(DeliveryMode, ExceptionFrame, TrapFrame);
}

unsafe fn KiDeliverApcImpl(
    DeliveryMode: i8,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
) {
    let Thread = PsGetCurrentThread();
    let Process = kt_ApcState(Thread).Process;
    let OldTrapFrame: PKTRAP_FRAME;
    let mut ApcListEntry: PLIST_ENTRY;
    let mut Apc: PKAPC;
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut KernelRoutine: PKKERNEL_ROUTINE;
    let mut NormalContext: PVOID;
    let mut NormalRoutine: PKNORMAL_ROUTINE;
    let mut SystemArgument1: PVOID;
    let mut SystemArgument2: PVOID;

    OldTrapFrame = (*Thread).TrapFrame;
    (*Thread).TrapFrame = TrapFrame;
    kt_ApcState(Thread).KernelApcPending = 0u8;

    if *kt_SpecialApcDisable(Thread) != 0 {
        (*Thread).TrapFrame = OldTrapFrame;
        return;
    }

    'kernel_apc_loop: loop {
        if IsListEmpty(&mut kt_ApcState(Thread).ApcListHead[0] as *mut LIST_ENTRY) {
            break;
        }

        KiAcquireApcLockRaiseToDpc(Thread as PKTHREAD, &mut ApcLock);

        if IsListEmpty(&mut kt_ApcState(Thread).ApcListHead[0] as *mut LIST_ENTRY) {
            KiReleaseApcLock(&mut ApcLock);
            break;
        }

        kt_ApcState(Thread).KernelApcPending = 0u8;
        ApcListEntry = kt_ApcState(Thread).ApcListHead[0].Flink;
        Apc = containing_record_apc(ApcListEntry);

        NormalRoutine = (*Apc).NormalRoutine;
        KernelRoutine = (*Apc).KernelRoutine;
        NormalContext = (*Apc).NormalContext;
        SystemArgument1 = (*Apc).SystemArgument1;
        SystemArgument2 = (*Apc).SystemArgument2;

        if NormalRoutine.is_none() {
            RemoveEntryList(ApcListEntry);
            (*Apc).Inserted = 0u8;
            KiReleaseApcLock(&mut ApcLock);

            (KernelRoutine.unwrap())(
                Apc,
                &mut NormalRoutine,
                &mut NormalContext,
                &mut SystemArgument1,
                &mut SystemArgument2,
            );

            if KeGetCurrentIrql() != ApcLock.OldIrql {
                KeBugCheckEx(
                    IRQL_UNEXPECTED_VALUE,
                    ((KeGetCurrentIrql() as ULONG_PTR) << 16) | ((ApcLock.OldIrql as ULONG_PTR) << 8),
                    opt_fn_as_uptr(KernelRoutine),
                    Apc as ULONG_PTR,
                    opt_fn_as_uptr(NormalRoutine),
                );
            }
        } else {
            if kt_ApcState(Thread).KernelApcInProgress != 0u8 || *kt_KernelApcDisable(Thread) != 0 {
                KiReleaseApcLock(&mut ApcLock);
                break 'kernel_apc_loop;
            }

            RemoveEntryList(ApcListEntry);
            (*Apc).Inserted = 0u8;
            KiReleaseApcLock(&mut ApcLock);

            (KernelRoutine.unwrap())(
                Apc,
                &mut NormalRoutine,
                &mut NormalContext,
                &mut SystemArgument1,
                &mut SystemArgument2,
            );

            if KeGetCurrentIrql() != ApcLock.OldIrql {
                KeBugCheckEx(
                    IRQL_UNEXPECTED_VALUE,
                    ((KeGetCurrentIrql() as ULONG_PTR) << 16) | ((ApcLock.OldIrql as ULONG_PTR) << 8),
                    opt_fn_as_uptr(KernelRoutine),
                    Apc as ULONG_PTR,
                    opt_fn_as_uptr(NormalRoutine),
                );
            }

            if NormalRoutine.is_some() {
                kt_ApcState(Thread).KernelApcInProgress = 1u8;
                KeLowerIrql(PASSIVE_LEVEL);

                (NormalRoutine.unwrap())(NormalContext, SystemArgument1, SystemArgument2);
                KeRaiseIrql(APC_LEVEL, &mut ApcLock.OldIrql);
            }

            kt_ApcState(Thread).KernelApcInProgress = 0u8;
        }
    }

    if DeliveryMode == 1i8
        && !IsListEmpty(&mut kt_ApcState(Thread).ApcListHead[1] as *mut LIST_ENTRY)
        && kt_ApcState(Thread).UserApcPending != 0u8
    {
        KiAcquireApcLockRaiseToDpc(Thread as PKTHREAD, &mut ApcLock);
        kt_ApcState(Thread).UserApcPending = 0u8;

        if IsListEmpty(&mut kt_ApcState(Thread).ApcListHead[1] as *mut LIST_ENTRY) {
            KiReleaseApcLock(&mut ApcLock);
        } else {
            ApcListEntry = kt_ApcState(Thread).ApcListHead[1].Flink;
            Apc = containing_record_apc(ApcListEntry);

            NormalRoutine = (*Apc).NormalRoutine;
            KernelRoutine = (*Apc).KernelRoutine;
            NormalContext = (*Apc).NormalContext;
            SystemArgument1 = (*Apc).SystemArgument1;
            SystemArgument2 = (*Apc).SystemArgument2;

            RemoveEntryList(ApcListEntry);
            (*Apc).Inserted = 0u8;
            KiReleaseApcLock(&mut ApcLock);

            (KernelRoutine.unwrap())(
                Apc,
                &mut NormalRoutine,
                &mut NormalContext,
                &mut SystemArgument1,
                &mut SystemArgument2,
            );

            if NormalRoutine.is_none() {
                KeTestAlertThread(1i8);
            } else {
                KiInitializeUserApc(
                    ExceptionFrame,
                    TrapFrame,
                    NormalRoutine,
                    NormalContext,
                    SystemArgument1,
                    SystemArgument2,
                );
            }
        }
    }

    if Process != kt_ApcState(Thread).Process {
        KeBugCheckEx(
            INVALID_PROCESS_ATTACH_ATTEMPT,
            Process as ULONG_PTR,
            kt_ApcState(Thread).Process as ULONG_PTR,
            (*Thread).ApcStateIndex as ULONG_PTR,
            (*KeGetCurrentPrcb()).DpcRoutineActive as ULONG_PTR,
        );
    }

    (*Thread).TrapFrame = OldTrapFrame;
}

unsafe fn RepairList(Original: *mut [LIST_ENTRY; 2], Copy: *mut [LIST_ENTRY; 2], Mode: i8) {
    let mode_idx = Mode as usize;
    if IsListEmpty(&mut (*Original)[mode_idx] as *mut LIST_ENTRY) {
        InitializeListHead(&mut (*Copy)[mode_idx] as *mut LIST_ENTRY);
    } else {
        (*Copy)[mode_idx].Flink = (*Original)[mode_idx].Flink;
        (*Copy)[mode_idx].Blink = (*Original)[mode_idx].Blink;
        (*(*Original)[mode_idx].Flink).Blink = &mut (*Copy)[mode_idx] as *mut LIST_ENTRY;
        (*(*Original)[mode_idx].Blink).Flink = &mut (*Copy)[mode_idx] as *mut LIST_ENTRY;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KiMoveApcState(OldState: PKAPC_STATE, NewState: PKAPC_STATE) {
    RtlCopyMemory(
        NewState as PVOID,
        OldState as PVOID,
        KAPC_STATE_ACTUAL_LENGTH,
    );

    RepairList(
        &mut (*OldState).ApcListHead as *mut [LIST_ENTRY; 2],
        &mut (*NewState).ApcListHead as *mut [LIST_ENTRY; 2],
        0i8,
    );
    RepairList(
        &mut (*OldState).ApcListHead as *mut [LIST_ENTRY; 2],
        &mut (*NewState).ApcListHead as *mut [LIST_ENTRY; 2],
        1i8,
    );
}

#[no_mangle]
pub unsafe extern "system" fn _KeEnterCriticalRegion() {
    KeEnterCriticalRegion();
}

#[no_mangle]
pub unsafe extern "system" fn _KeLeaveCriticalRegion() {
    KeLeaveCriticalRegion();
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeApc(
    Apc: PKAPC,
    Thread: PKTHREAD,
    TargetEnvironment: KAPC_ENVIRONMENT,
    KernelRoutine: PKKERNEL_ROUTINE,
    RundownRoutine: PKRUNDOWN_ROUTINE,
    NormalRoutine: PKNORMAL_ROUTINE,
    Mode: i8,
    Context: PVOID,
) {
    let Thread = opaque_ptr(Thread);
    let Context = opaque_ptr(Context);
    (*Apc).Type = APC_OBJECT_TYPE;
    (*Apc).Size = core::mem::size_of::<_KAPC>() as u8;

    if TargetEnvironment == KAPC_ENV_CURRENT {
        (*Apc).ApcStateIndex = (*(Thread as *mut _KTHREAD)).ApcStateIndex as i8;
    } else {
        (*Apc).ApcStateIndex = TargetEnvironment as i8;
    }

    (*Apc).Thread = Thread;
    (*Apc).KernelRoutine = KernelRoutine;
    (*Apc).RundownRoutine = RundownRoutine;
    (*Apc).NormalRoutine = NormalRoutine;

    if NormalRoutine.is_some() {
        (*Apc).ApcMode = Mode;
        (*Apc).NormalContext = Context;
    } else {
        (*Apc).ApcMode = 0i8;
        (*Apc).NormalContext = core::ptr::null_mut();
    }

    (*Apc).Inserted = 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn KeInsertQueueApc(
    Apc: PKAPC,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
    PriorityBoost: i32,
) -> BOOLEAN {
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    let Thread = (*Apc).Thread;
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut State = 1u8;

    KiAcquireApcLockRaiseToSynch(Thread, &mut ApcLock);

    if *kt_ApcQueueable(Thread) == 0u8 || (*Apc).Inserted != 0u8 {
        State = 0u8;
    } else {
        (*Apc).SystemArgument1 = SystemArgument1;
        (*Apc).SystemArgument2 = SystemArgument2;
        (*Apc).Inserted = 1u8;
        KiInsertQueueApc(Apc, PriorityBoost);
    }

    KiReleaseApcLockFromSynchLevel(&mut ApcLock);
    KiExitDispatcher(core::ptr::read_volatile(&ApcLock.OldIrql));
    State
}

#[no_mangle]
pub unsafe extern "system" fn KeFlushQueueApc(
    Thread: PKTHREAD,
    PreviousMode: i8,
) -> PLIST_ENTRY {
    let Thread = opaque_ptr(Thread);
    let mut Apc: PKAPC;
    let mut FirstEntry: PLIST_ENTRY;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    if PreviousMode == 1i8 {
        KiAcquireApcLockRaiseToSynch(Thread, &mut ApcLock);

        if IsListEmpty(&mut kt_ApcState(Thread as *mut _KTHREAD).ApcListHead[1] as *mut LIST_ENTRY) {
            FirstEntry = core::ptr::null_mut();
            KiReleaseApcLock(&mut ApcLock);
            return FirstEntry;
        }
    } else {
        if IsListEmpty(&mut kt_ApcState(Thread as *mut _KTHREAD).ApcListHead[0] as *mut LIST_ENTRY) {
            return core::ptr::null_mut();
        }
        KiAcquireApcLockRaiseToSynch(Thread, &mut ApcLock);
    }

    FirstEntry = kt_ApcState(Thread as *mut _KTHREAD).ApcListHead[PreviousMode as usize].Flink;
    if FirstEntry == &mut kt_ApcState(Thread as *mut _KTHREAD).ApcListHead[PreviousMode as usize] as *mut LIST_ENTRY {
        FirstEntry = core::ptr::null_mut();
    } else {
        RemoveEntryList(&mut kt_ApcState(Thread as *mut _KTHREAD).ApcListHead[PreviousMode as usize] as *mut LIST_ENTRY);

        CurrentEntry = FirstEntry;
        loop {
            Apc = containing_record_apc(CurrentEntry);
            (*Apc).Inserted = 0u8;
            CurrentEntry = (*CurrentEntry).Flink;
            if CurrentEntry == FirstEntry {
                break;
            }
        }

        InitializeListHead(&mut kt_ApcState(Thread as *mut _KTHREAD).ApcListHead[PreviousMode as usize] as *mut LIST_ENTRY);
    }

    KiReleaseApcLock(&mut ApcLock);
    FirstEntry
}

#[no_mangle]
pub unsafe extern "system" fn KeRemoveQueueApc(Apc: PKAPC) -> BOOLEAN {
    let Thread = (*Apc).Thread;
    let ApcState: PKAPC_STATE;
    let Inserted: BOOLEAN;
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    KiAcquireApcLockRaiseToSynch(Thread, &mut ApcLock);

    Inserted = (*Apc).Inserted;
    if Inserted != 0u8 {
        (*Apc).Inserted = 0u8;
        ApcState = (*(Thread as *mut _KTHREAD)).ApcStatePointer[(*Apc).ApcStateIndex as usize];

        KiAcquireDispatcherLockAtSynchLevel();
        if RemoveEntryList(&mut (*Apc).ApcListEntry as *mut LIST_ENTRY) {
            if (*Apc).ApcMode == 0i8 {
                (*ApcState).KernelApcPending = 0u8;
            } else {
                (*ApcState).UserApcPending = 0u8;
            }
        }
        KiReleaseDispatcherLockFromSynchLevel();
    }

    KiReleaseApcLock(&mut ApcLock);
    Inserted
}

#[no_mangle]
pub unsafe extern "system" fn KeAreApcsDisabled() -> BOOLEAN {
    let thread = PsGetCurrentThread();
    if *kt_CombinedApcDisable(thread) != 0 {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeAreAllApcsDisabled() -> BOOLEAN {
    let thread = KeGetCurrentThread();
    if *kt_SpecialApcDisable(thread) != 0
        || KeGetCurrentIrql() >= APC_LEVEL
    {
        1u8
    } else {
        0u8
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/apc.c
//  * PURPOSE:         Implements the Asynchronous Procedure Call mechanism
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* ARM64 verbose APC traces removed -- APC delivery confirmed working */
// #define _APC_TRACE(s) ((void)0)
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /*++
//  * @name KiCheckForKernelApcDelivery
//  * @implemented NT 5.2
//  *
//  *     The KiCheckForKernelApcDelivery routine is called whenever APCs have
//  *     just been re-enabled in Kernel Mode, such as after leaving a Critical or
//  *     Guarded Region. It delivers APCs if the environment is right.
//  *
//  * @param None.
//  *
//  * @return None.
//  *
//  * @remarks This routine allows KeLeave/EnterCritical/GuardedRegion to be used
//  *          as macros from inside WIN32K or other Drivers, which will then only
//  *          have to do an Import API call in the case where APCs are enabled again.
//  *
//  *--*/
// VOID
// NTAPI
// KiCheckForKernelApcDelivery(VOID)
// {
//     KIRQL OldIrql;
//
//     /* We should only deliver at passive */
//     if (KeGetCurrentIrql() == PASSIVE_LEVEL)
//     {
//         /* Raise to APC and Deliver APCs, then lower back to Passive */
//         KeRaiseIrql(APC_LEVEL, &OldIrql);
//         KiDeliverApc(KernelMode, 0, 0);
//         KeLowerIrql(PASSIVE_LEVEL);
//     }
//     else
//     {
//         /*
//          * If we're not at passive level it means someone raised IRQL
//          * to APC level before the critical or guarded section was entered
//          * (e.g) by a fast mutex). This implies that the APCs shouldn't
//          * be delivered now, but after the IRQL is lowered to passive
//          * level again.
//          */
//         KeGetCurrentThread()->ApcState.KernelApcPending = TRUE;
//         HalRequestSoftwareInterrupt(APC_LEVEL);
//     }
// }
//
// /*++
//  * @name KiInsertQueueApc
//  *
//  *     The KiInsertQueueApc routine queues a APC for execution when the right
//  *     scheduler environment exists.
//  *
//  * @param Apc
//  *        Pointer to an initialized control object of type APC for which the
//  *        caller provides the storage.
//  *
//  * @param PriorityBoost
//  *        Priority Boost to apply to the Thread.
//  *
//  * @return None
//  *
//  * @remarks The APC will execute at APC_LEVEL for the KernelRoutine registered,
//  *          and at PASSIVE_LEVEL for the NormalRoutine registered.
//  *
//  *          Callers of this routine must have locked the dipatcher database.
//  *
//  *--*/
// VOID
// FASTCALL
// KiInsertQueueApc(IN PKAPC Apc,
//                  IN KPRIORITY PriorityBoost)
// {
//     PKTHREAD Thread = Apc->Thread;
//     PKAPC_STATE ApcState;
//     KPROCESSOR_MODE ApcMode;
//     PLIST_ENTRY ListHead, NextEntry;
//     PKAPC QueuedApc;
//     PKGATE Gate;
//     NTSTATUS Status;
//     BOOLEAN RequestInterrupt = FALSE;
// }
