use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use super::ke_defs::*;
#[cfg(feature = "smp")]
use crate::ke::krnlinit::{KeNodeBlock, KeNumberNodes, KeProcessNodeSeed};
#[cfg(feature = "smp")]
use super::thrdobj::KeFindNextRightSetAffinity;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::KeGetCurrentThread;

use ntkrnl_sys::ddk::{
    _KPROCESS, _KTHREAD, _KSERVICE_TABLE_DESCRIPTOR,
    KEVENT, PROCESS_VALUES,
    PKPROCESS, PKTHREAD, PRKAPC_STATE,
    KAFFINITY, KPRIORITY,
    HalRequestSoftwareInterrupt,
    KeBugCheck,
    KiAcquireApcLockRaiseToSynch, KiAcquireDispatcherLock,
    KiAcquireDispatcherLockAtSynchLevel, KiAcquireProcessLockRaiseToSynch,
    KiAcquireThreadLock, KiExitDispatcher,
    KiMoveApcState, KiReleaseApcLock,
    KiReleaseApcLockFromSynchLevel, KiReleaseDispatcherLock,
    KiReleaseDispatcherLockFromSynchLevel, KiReleaseProcessLock,
    KiReleaseProcessLockFromSynchLevel, KiReleaseThreadLock,
    KiSetAffinityThread, KiSetPriorityThread, KiSwapProcess,
    KeIsExecutingDpc,
};

// ---------------------------------------------------------------------------
// Architecture-specific KTHREAD field access helpers.
//
// On i386 the union numbering differs from amd64/arm64:
//   SavedApcState:  i386 = __bindgen_anon_8,  amd64/arm64 = __bindgen_anon_6
//   Quantum:        i386 = __bindgen_anon_9.__bindgen_anon_1,  amd64/arm64 = __bindgen_anon_7.__bindgen_anon_1
//   QuantumReset:   i386 = __bindgen_anon_9.__bindgen_anon_2,  amd64/arm64 = __bindgen_anon_7.__bindgen_anon_2
//   KernelTime:     i386 = __bindgen_anon_9.__bindgen_anon_3,  amd64/arm64 = __bindgen_anon_7.__bindgen_anon_3
//   UserTime:       i386 = direct field,       amd64/arm64 = __bindgen_anon_7.__bindgen_anon_6
// ---------------------------------------------------------------------------

/// Read Thread->QuantumReset (UCHAR on all arches).
#[inline(always)]
unsafe fn kthread_get_quantum_reset(t: *const _KTHREAD) -> u8 {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_9.__bindgen_anon_2.QuantumReset }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_7.__bindgen_anon_2.QuantumReset }
}

/// Write Thread->QuantumReset.
#[inline(always)]
unsafe fn kthread_set_quantum_reset(t: *mut _KTHREAD, val: u8) {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_9.__bindgen_anon_2.QuantumReset = val; }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_7.__bindgen_anon_2.QuantumReset = val; }
}

/// Read Thread->Quantum (SCHAR on all arches).
#[inline(always)]
unsafe fn kthread_get_quantum(t: *const _KTHREAD) -> i8 {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_9.__bindgen_anon_1.Quantum }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_7.__bindgen_anon_1.Quantum }
}

/// Write Thread->Quantum.
#[inline(always)]
unsafe fn kthread_set_quantum(t: *mut _KTHREAD, val: i8) {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_9.__bindgen_anon_1.Quantum = val; }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_7.__bindgen_anon_1.Quantum = val; }
}

/// Read Thread->KernelTime (ULONG).
#[inline(always)]
unsafe fn kthread_get_kernel_time(t: *const _KTHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_9.__bindgen_anon_3.KernelTime }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_7.__bindgen_anon_3.KernelTime }
}

/// Read Thread->UserTime (ULONG).
#[inline(always)]
unsafe fn kthread_get_user_time(t: *const _KTHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).UserTime }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_7.__bindgen_anon_6.UserTime }
}

/// Get pointer to Thread->SavedApcState.
#[inline(always)]
unsafe fn kthread_saved_apc_state_ptr(t: *mut _KTHREAD) -> *mut ntkrnl_sys::ddk::_KAPC_STATE {
    #[cfg(target_arch = "x86")]
    { core::ptr::addr_of_mut!((*t).__bindgen_anon_8.SavedApcState) }
    #[cfg(not(target_arch = "x86"))]
    { core::ptr::addr_of_mut!((*t).__bindgen_anon_6.SavedApcState) }
}

/// Read SavedApcState.Process.
#[inline(always)]
unsafe fn kthread_saved_apc_state_process(t: *const _KTHREAD) -> PKPROCESS {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_8.SavedApcState.Process }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_6.SavedApcState.Process }
}

/// Write SavedApcState.Process.
#[inline(always)]
unsafe fn kthread_set_saved_apc_state_process(t: *mut _KTHREAD, val: PKPROCESS) {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_8.SavedApcState.Process = val; }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_6.SavedApcState.Process = val; }
}

/// Read Thread->KernelApcDisable (SHORT).
#[inline(always)]
unsafe fn kthread_get_kernel_apc_disable(t: *const _KTHREAD) -> i16 {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_4.__bindgen_anon_1.KernelApcDisable }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.KernelApcDisable }
}

/// C's CONTAINING_RECORD macro, inlined (verify_ffi C_MACRO rule). The
/// type-size argument is unused — the record base is ptr - field_offset;
/// kept in the signature so the translated call sites read like the C.
#[inline(always)]
unsafe fn CONTAINING_RECORD(ptr: PVOID, _type_size: usize, field_offset: usize) -> PVOID {
    (ptr as usize - field_offset) as PVOID
}

#[no_mangle]
pub static mut KiProcessListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut KiProcessInSwapListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut KiProcessOutSwapListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut KiStackInSwapListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut KiSwapEvent: KEVENT = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KeServiceDescriptorTable: [_KSERVICE_TABLE_DESCRIPTOR; SSDT_MAX_ENTRIES as usize] =
    unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KeServiceDescriptorTableShadow: [_KSERVICE_TABLE_DESCRIPTOR; SSDT_MAX_ENTRIES as usize] =
    unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KeUserApcDispatcher: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut KeUserCallbackDispatcher: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut KeUserExceptionDispatcher: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut KeRaiseUserExceptionDispatcher: PVOID = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn KeInitializeProcess(
    Process: PKPROCESS,
    Priority: KPRIORITY,
    Affinity: KAFFINITY,
    DirectoryTableBase: PULONG_PTR,
    Enable: BOOLEAN,
) {
    let Process = opaque_ptr(Process);
    let DirectoryTableBase = opaque_ptr(DirectoryTableBase);
    #[cfg(target_arch = "x86")]
    {
        extern "system" {
    fn KiComputeIopmOffset(map: USHORT) -> USHORT;
}
        const IO_ACCESS_MAP_NONE: USHORT = 0;
    }

    #[cfg(feature = "smp")]
    {
        use super::krnlinit::{KeNumberNodes, KeProcessNodeSeed, KeNodeBlock};
        use ntkrnl_sys::ddk::{PKNODE, KeFindNextRightSetAffinity};
    }

    (*Process).Header.__bindgen_anon_1.__bindgen_anon_1.Type = ProcessObject;
    (*Process).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size = (core::mem::size_of::<_KPROCESS>() / core::mem::size_of::<ULONG>()) as UCHAR;
    (*Process).Header.SignalState = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Process).Header.WaitListHead));

    (*Process).Affinity = Affinity;
    (*Process).BasePriority = Priority as i8;
    (*Process).QuantumReset = 6;
    (*Process).DirectoryTableBase[0] = *DirectoryTableBase.add(0) as ntkrnl_sys::ddk::ULONG_PTR;
    (*Process).DirectoryTableBase[1] = *DirectoryTableBase.add(1) as ntkrnl_sys::ddk::ULONG_PTR;
    (*Process).__bindgen_anon_1.__bindgen_anon_1.set_AutoAlignment(Enable as LONG);

    #[cfg(target_arch = "x86")]
    {
        (*Process).IopmOffset = KiComputeIopmOffset(IO_ACCESS_MAP_NONE as u32);
    }

    InitializeListHead(core::ptr::addr_of_mut!((*Process).ThreadListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*Process).ProfileListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*Process).ReadyListHead));

    (*Process).State = ProcessInMemory as u8;

    #[cfg(feature = "smp")]
    {
        let mut i: ULONG = 0;
        let mut IdealNode: UCHAR = 0;

        if KeNumberNodes > 1 {
            KeProcessNodeSeed = (KeProcessNodeSeed + 1) % KeNumberNodes as UCHAR;
            IdealNode = KeProcessNodeSeed;

            loop {
                let node_mask = (*KeNodeBlock[IdealNode as usize]).ProcessorMask;
                if node_mask != Affinity {
                    break;
                }

                IdealNode += 1;
                i += 1;

                if IdealNode >= KeNumberNodes as UCHAR {
                    IdealNode -= KeNumberNodes as UCHAR;
                }

                if i >= KeNumberNodes as ULONG {
                    break;
                }
            }
        }

        (*Process).IdealNode = IdealNode;
        let Node = KeNodeBlock[IdealNode as usize];

        let affinity_masked = Affinity & (*Node).ProcessorMask;
        (*Process).ThreadSeed = KeFindNextRightSetAffinity((*Node).Seed, affinity_masked);
        (*Node).Seed = (*Process).ThreadSeed;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeSetProcess(
    Process: PKPROCESS,
    Increment: KPRIORITY,
    InWait: BOOLEAN,
) -> ULONG {
    let Process = opaque_ptr(Process);
    extern "C" {
    fn KxUnwaitThread(header: PVOID, increment: KPRIORITY);
}

    let OldIrql = KiAcquireDispatcherLock();
    let OldState = (*Process).Header.SignalState;

    (*Process).Header.SignalState = 1;

    if OldState == 0 && !IsListEmpty(core::ptr::addr_of_mut!((*Process).Header.WaitListHead)) {
        KxUnwaitThread(core::ptr::addr_of_mut!((*Process).Header) as PVOID, Increment);
    }

    KiReleaseDispatcherLock(OldIrql);

    OldState as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn KeSetQuantumProcess(Process: PKPROCESS, Quantum: UCHAR) {
    let Process = opaque_ptr(Process);
    let mut ProcessLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(ProcessLock));

    (*Process).QuantumReset = Quantum as i8;

    let ListHead = core::ptr::addr_of_mut!((*Process).ThreadListHead);
    let mut NextEntry = (*ListHead).Flink;

    while NextEntry != ListHead {
        let Thread = CONTAINING_RECORD(
            NextEntry as PVOID,
            core::mem::size_of::<_KTHREAD>(),
            core::mem::offset_of!(_KTHREAD, ThreadListEntry),
        ) as PKTHREAD;

        kthread_set_quantum_reset(Thread, Quantum);

        NextEntry = (*NextEntry).Flink;
    }

    KiReleaseProcessLock(core::ptr::addr_of_mut!(ProcessLock));
}

#[no_mangle]
pub unsafe extern "system" fn KeSetAffinityProcess(
    Process: PKPROCESS,
    Affinity: KAFFINITY,
) -> KAFFINITY {
    let Process = opaque_ptr(Process);
    let mut ProcessLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(ProcessLock));
    KiAcquireDispatcherLockAtSynchLevel();

    let OldAffinity = (*Process).Affinity;
    (*Process).Affinity = Affinity;

    let ListHead = core::ptr::addr_of_mut!((*Process).ThreadListHead);
    let mut NextEntry = (*ListHead).Flink;

    while NextEntry != ListHead {
        let Thread = CONTAINING_RECORD(
            NextEntry as PVOID,
            core::mem::size_of::<_KTHREAD>(),
            core::mem::offset_of!(_KTHREAD, ThreadListEntry),
        ) as PKTHREAD;

        KiSetAffinityThread(Thread, Affinity);
        NextEntry = (*NextEntry).Flink;
    }

    KiReleaseDispatcherLockFromSynchLevel();
    KiReleaseProcessLockFromSynchLevel(core::ptr::addr_of_mut!(ProcessLock));
    KiExitDispatcher(core::ptr::read_volatile(&ProcessLock.OldIrql));

    OldAffinity
}

#[no_mangle]
pub unsafe extern "system" fn KeSetAutoAlignmentProcess(
    Process: PKPROCESS,
    Enable: BOOLEAN,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    const KPSF_AUTO_ALIGNMENT_BIT: LONG = 0;

    if Enable != 0 {
        InterlockedBitTestAndSet(
            core::ptr::addr_of_mut!((*Process).__bindgen_anon_1.ProcessFlags) as *mut LONG,
            KPSF_AUTO_ALIGNMENT_BIT,
        ) as BOOLEAN
    } else {
        InterlockedBitTestAndReset(
            core::ptr::addr_of_mut!((*Process).__bindgen_anon_1.ProcessFlags) as *mut LONG,
            KPSF_AUTO_ALIGNMENT_BIT,
        ) as BOOLEAN
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeSetDisableBoostProcess(
    Process: PKPROCESS,
    Disable: BOOLEAN,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    const KPSF_DISABLE_BOOST_BIT: LONG = 1;

    if Disable != 0 {
        InterlockedBitTestAndSet(
            core::ptr::addr_of_mut!((*Process).__bindgen_anon_1.ProcessFlags) as *mut LONG,
            KPSF_DISABLE_BOOST_BIT,
        ) as BOOLEAN
    } else {
        InterlockedBitTestAndReset(
            core::ptr::addr_of_mut!((*Process).__bindgen_anon_1.ProcessFlags) as *mut LONG,
            KPSF_DISABLE_BOOST_BIT,
        ) as BOOLEAN
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeSetPriorityAndQuantumProcess(
    Process: PKPROCESS,
    Priority: KPRIORITY,
    Quantum: UCHAR,
) -> KPRIORITY {
    let Process = opaque_ptr(Process);
    const LOW_PRIORITY: KPRIORITY = 0;
    const LOW_REALTIME_PRIORITY: KPRIORITY = 16;
    const HIGH_PRIORITY: KPRIORITY = 31;

    if (*Process).BasePriority == Priority as i8 {
        return (*Process).BasePriority as KPRIORITY;
    }

    let mut Priority = Priority;
    if Priority == 0 {
        Priority = LOW_PRIORITY + 1;
    }

    let mut ProcessLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(ProcessLock));
    KiAcquireDispatcherLockAtSynchLevel();

    if Quantum != 0 {
        (*Process).QuantumReset = Quantum as i8;
    }

    let OldPriority = (*Process).BasePriority as KPRIORITY;
    (*Process).BasePriority = Priority as i8;

    let Delta = Priority as i32 - OldPriority as i32;

    let ListHead = core::ptr::addr_of_mut!((*Process).ThreadListHead);
    let mut NextEntry = (*ListHead).Flink;

    if Priority >= LOW_REALTIME_PRIORITY {
        while NextEntry != ListHead {
            let Thread = CONTAINING_RECORD(
                NextEntry as PVOID,
                core::mem::size_of::<_KTHREAD>(),
                core::mem::offset_of!(_KTHREAD, ThreadListEntry),
            ) as PKTHREAD;

            if Quantum != 0 {
                kthread_set_quantum_reset(Thread, Quantum);
            }

            KiAcquireThreadLock(Thread);

            let mut NewPriority = (*Thread).BasePriority as i32 + Delta;
            if NewPriority < LOW_REALTIME_PRIORITY as i32 {
                NewPriority = LOW_REALTIME_PRIORITY as i32;
            } else if NewPriority > HIGH_PRIORITY as i32 {
                NewPriority = HIGH_PRIORITY as i32;
            }

            if (*Thread).Saturation == 0 || OldPriority < LOW_REALTIME_PRIORITY {
                if (*Thread).Saturation > 0 {
                    NewPriority = HIGH_PRIORITY as i32;
                } else if (*Thread).Saturation < 0 {
                    NewPriority = LOW_REALTIME_PRIORITY as i32;
                }

                (*Thread).BasePriority = NewPriority as i8;
                kthread_set_quantum(Thread, kthread_get_quantum_reset(Thread) as i8);
                (*Thread).PriorityDecrement = 0;
                KiSetPriorityThread(Thread, NewPriority as KPRIORITY);
            }

            KiReleaseThreadLock(Thread);

            NextEntry = (*NextEntry).Flink;
        }
    } else {
        while NextEntry != ListHead {
            let Thread = CONTAINING_RECORD(
                NextEntry as PVOID,
                core::mem::size_of::<_KTHREAD>(),
                core::mem::offset_of!(_KTHREAD, ThreadListEntry),
            ) as PKTHREAD;

            if Quantum != 0 {
                kthread_set_quantum_reset(Thread, Quantum);
            }

            KiAcquireThreadLock(Thread);

            let mut NewPriority = (*Thread).BasePriority as i32 + Delta;
            if NewPriority >= LOW_REALTIME_PRIORITY as i32 {
                NewPriority = LOW_REALTIME_PRIORITY as i32 - 1;
            } else if NewPriority <= LOW_PRIORITY as i32 {
                NewPriority = 1;
            }

            if (*Thread).Saturation == 0 || OldPriority >= LOW_REALTIME_PRIORITY {
                if (*Thread).Saturation > 0 {
                    NewPriority = LOW_REALTIME_PRIORITY as i32 - 1;
                } else if (*Thread).Saturation < 0 {
                    NewPriority = 1;
                }

                (*Thread).BasePriority = NewPriority as i8;
                kthread_set_quantum(Thread, kthread_get_quantum_reset(Thread) as i8);
                (*Thread).PriorityDecrement = 0;
                KiSetPriorityThread(Thread, NewPriority as KPRIORITY);
            }

            KiReleaseThreadLock(Thread);

            NextEntry = (*NextEntry).Flink;
        }
    }

    KiReleaseDispatcherLockFromSynchLevel();
    KiReleaseProcessLockFromSynchLevel(core::ptr::addr_of_mut!(ProcessLock));
    KiExitDispatcher(core::ptr::read_volatile(&ProcessLock.OldIrql));

    OldPriority
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryValuesProcess(
    Process: PKPROCESS,
    Values: *mut PROCESS_VALUES,
) {
    let Process = opaque_ptr(Process);
    let Values = opaque_ptr(Values);
    use ntkrnl_sys::ddk::_EPROCESS;
    // Use actual definition from clock.rs, not extern (lto=false splits them)
    use crate::ke::clock::KeMaximumIncrement;

    let mut ProcessLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(ProcessLock));

    let mut TotalKernel = (*Process).KernelTime;
    let mut TotalUser = (*Process).UserTime;

    let EProcess = Process as *mut _EPROCESS;
    (*Values).IoInfo.ReadOperationCount = (*EProcess).ReadOperationCount.QuadPart as u64;
    (*Values).IoInfo.WriteOperationCount = (*EProcess).WriteOperationCount.QuadPart as u64;
    (*Values).IoInfo.OtherOperationCount = (*EProcess).OtherOperationCount.QuadPart as u64;
    (*Values).IoInfo.ReadTransferCount = (*EProcess).ReadTransferCount.QuadPart as u64;
    (*Values).IoInfo.WriteTransferCount = (*EProcess).WriteTransferCount.QuadPart as u64;
    (*Values).IoInfo.OtherTransferCount = (*EProcess).OtherTransferCount.QuadPart as u64;

    let mut NextEntry = (*Process).ThreadListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*Process).ThreadListHead) {
        let Thread = CONTAINING_RECORD(
            NextEntry as PVOID,
            core::mem::size_of::<_KTHREAD>(),
            core::mem::offset_of!(_KTHREAD, ThreadListEntry),
        ) as PKTHREAD;

        TotalKernel += kthread_get_kernel_time(Thread);
        TotalUser += kthread_get_user_time(Thread);

        NextEntry = (*NextEntry).Flink;
    }

    KiReleaseProcessLock(core::ptr::addr_of_mut!(ProcessLock));

    (*Values).TotalKernelTime.QuadPart = TotalKernel as i64 * KeMaximumIncrement as i64;
    (*Values).TotalUserTime.QuadPart = TotalUser as i64 * KeMaximumIncrement as i64;
}

/// Originally NTAPI in procobj.c. Splices a new process into the calling
/// thread's APC environment after the caller has acquired the thread's APC
/// lock and the dispatcher lock at SYNCH_LEVEL. The release pair runs as
/// part of the swap so callers see the dispatcher already lowered when we
/// return.
#[no_mangle]
pub unsafe extern "system" fn KiAttachProcess(
    Thread: PKTHREAD,
    Process: PKPROCESS,
    ApcLock: PKLOCK_QUEUE_HANDLE,
    SavedApcState: PRKAPC_STATE,
) {
    const KernelMode: usize = 0;
    const UserMode: usize = 1;
    const OriginalApcEnvironment: UCHAR = 0;
    const AttachedApcEnvironment: UCHAR = 1;
    const ProcessInMemory: UCHAR = 0;

    (*Process).StackCount = (*Process).StackCount.wrapping_add(1);

    KiMoveApcState(
        core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState),
        SavedApcState,
    );

    InitializeListHead(core::ptr::addr_of_mut!(
        (*Thread).__bindgen_anon_1.ApcState.ApcListHead[KernelMode]
    ));
    InitializeListHead(core::ptr::addr_of_mut!(
        (*Thread).__bindgen_anon_1.ApcState.ApcListHead[UserMode]
    ));
    (*Thread).__bindgen_anon_1.ApcState.Process = Process;
    (*Thread).__bindgen_anon_1.ApcState.KernelApcInProgress = 0;
    (*Thread).__bindgen_anon_1.ApcState.KernelApcPending = 0;
    (*Thread).__bindgen_anon_1.ApcState.UserApcPending = 0;

    if SavedApcState == kthread_saved_apc_state_ptr(Thread) {
        (*Thread).ApcStatePointer[OriginalApcEnvironment as usize] =
            kthread_saved_apc_state_ptr(Thread);
        (*Thread).ApcStatePointer[AttachedApcEnvironment as usize] =
            core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState);
        (*Thread).ApcStateIndex = AttachedApcEnvironment;
    }

    if (*Process).State == ProcessInMemory {
        KiReleaseDispatcherLockFromSynchLevel();
        KiReleaseApcLockFromSynchLevel(ApcLock);
        KiSwapProcess(Process, (*SavedApcState).Process);
        KiExitDispatcher(core::ptr::read_volatile(&(*ApcLock).OldIrql));
    } else {
        // ReactOS doesn't support paging out processes yet — match the C ASSERT(FALSE).
        KeBugCheck(INVALID_PROCESS_ATTACH_ATTEMPT);
    }
}

#[no_mangle]
#[cfg_attr(target_arch = "aarch64", inline(never))]
pub unsafe extern "system" fn KeAttachProcess(Process: PKPROCESS) {
    let Process = opaque_ptr(Process);
    const OriginalApcEnvironment: UCHAR = 0;

    let Thread = KeGetCurrentThread() as PKTHREAD;
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    if (*Thread).__bindgen_anon_1.ApcState.Process == Process {
        return;
    }

    if (*Thread).ApcStateIndex != OriginalApcEnvironment || KeIsExecutingDpc() != 0 {
        KeBugCheckEx(
            INVALID_PROCESS_ATTACH_ATTEMPT,
            Process as ULONG_PTR,
            (*Thread).__bindgen_anon_1.ApcState.Process as ULONG_PTR,
            (*Thread).ApcStateIndex as ULONG_PTR,
            KeIsExecutingDpc() as ULONG_PTR,
        );
    } else {
        KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));
        KiAcquireDispatcherLockAtSynchLevel();

        KiAttachProcess(
            Thread,
            Process,
            core::ptr::addr_of_mut!(ApcLock),
            kthread_saved_apc_state_ptr(Thread),
        );
    }
}

#[no_mangle]
#[cfg_attr(target_arch = "aarch64", inline(never))]
pub unsafe extern "system" fn KeDetachProcess() {
    const OriginalApcEnvironment: UCHAR = 0;

    let Thread = KeGetCurrentThread() as PKTHREAD;
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    if (*Thread).ApcStateIndex == OriginalApcEnvironment {
        return;
    }

    KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));

    if (*Thread).__bindgen_anon_1.ApcState.KernelApcInProgress != 0
        || !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState.ApcListHead[0]))
        || !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState.ApcListHead[1]))
    {
        KeBugCheck(INVALID_PROCESS_DETACH_ATTEMPT);
    }

    let Process = (*Thread).__bindgen_anon_1.ApcState.Process;

    KiAcquireDispatcherLockAtSynchLevel();

    (*Process).StackCount -= 1;

    if (*Process).StackCount == 0 {
    }

    KiReleaseDispatcherLockFromSynchLevel();

    KiMoveApcState(
        kthread_saved_apc_state_ptr(Thread),
        core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState),
    );
    kthread_set_saved_apc_state_process(Thread, core::ptr::null_mut());
    (*Thread).ApcStatePointer[OriginalApcEnvironment as usize] =
        core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState);
    (*Thread).ApcStatePointer[1] = kthread_saved_apc_state_ptr(Thread);
    (*Thread).ApcStateIndex = OriginalApcEnvironment;

    KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));

    KiSwapProcess((*Thread).__bindgen_anon_1.ApcState.Process, Process);

    // read_volatile prevents LLVM from proving OldIrql is the zeroed init value
    // (C shim KiAcquireApcLockRaiseToSynch writes it through a pointer)
    KiExitDispatcher(core::ptr::read_volatile(&ApcLock.OldIrql));

    if !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState.ApcListHead[0])) {
        (*Thread).__bindgen_anon_1.ApcState.KernelApcPending = 1;
        HalRequestSoftwareInterrupt(APC_LEVEL);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeIsAttachedProcess() -> BOOLEAN {
    (*(KeGetCurrentThread() as PKTHREAD)).ApcStateIndex
}

#[no_mangle]
pub unsafe extern "system" fn KeStackAttachProcess(
    Process: PKPROCESS,
    ApcState: PRKAPC_STATE,
) {
    let Process = opaque_ptr(Process);
    const OriginalApcEnvironment: UCHAR = 0;

    let Thread = KeGetCurrentThread() as PKTHREAD;
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    if KeIsExecutingDpc() != 0 {
        KeBugCheckEx(
            INVALID_PROCESS_ATTACH_ATTEMPT,
            Process as ULONG_PTR,
            (*Thread).__bindgen_anon_1.ApcState.Process as ULONG_PTR,
            (*Thread).ApcStateIndex as ULONG_PTR,
            KeIsExecutingDpc() as ULONG_PTR,
        );
    }

    if (*Thread).__bindgen_anon_1.ApcState.Process == Process {
        (*ApcState).Process = 1 as PKPROCESS;
        return;
    }

    KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));
    KiAcquireDispatcherLockAtSynchLevel();

    if (*Thread).ApcStateIndex != OriginalApcEnvironment {
        KiAttachProcess(Thread, Process, core::ptr::addr_of_mut!(ApcLock), ApcState);
    } else {
        KiAttachProcess(
            Thread,
            Process,
            core::ptr::addr_of_mut!(ApcLock),
            kthread_saved_apc_state_ptr(Thread),
        );
        (*ApcState).Process = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeUnstackDetachProcess(ApcState: PRKAPC_STATE) {
    const OriginalApcEnvironment: UCHAR = 0;
    // (removed broken `const 0i32` / `const 1i32` declarations)

    let Thread = KeGetCurrentThread() as PKTHREAD;
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    if (*ApcState).Process == 1 as PKPROCESS {
        return;
    }

    loop {
        KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));

        if (*Thread).__bindgen_anon_1.ApcState.KernelApcPending != 0 {
            if kthread_get_kernel_apc_disable(Thread) == 0 && ApcLock.OldIrql <= APC_LEVEL {
                KiReleaseApcLock(core::ptr::addr_of_mut!(ApcLock));
                continue;
            }
        }

        break;
    }

    if (*Thread).ApcStateIndex == OriginalApcEnvironment
        || (*Thread).__bindgen_anon_1.ApcState.KernelApcInProgress != 0
        || !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState.ApcListHead[0]))
        || !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState.ApcListHead[1]))
    {
        KeBugCheck(INVALID_PROCESS_DETACH_ATTEMPT);
    }

    let Process = (*Thread).__bindgen_anon_1.ApcState.Process;

    KiAcquireDispatcherLockAtSynchLevel();

    (*Process).StackCount -= 1;

    if (*Process).StackCount == 0 {
    }

    KiReleaseDispatcherLockFromSynchLevel();

    if !(*ApcState).Process.is_null() {
        KiMoveApcState(ApcState, core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState));
    } else {
        KiMoveApcState(
            kthread_saved_apc_state_ptr(Thread),
            core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState),
        );
        kthread_set_saved_apc_state_process(Thread, core::ptr::null_mut());
        (*Thread).ApcStateIndex = OriginalApcEnvironment;
        (*Thread).ApcStatePointer[OriginalApcEnvironment as usize] =
            core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState);
        (*Thread).ApcStatePointer[1] = kthread_saved_apc_state_ptr(Thread);
    }

    KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));

    KiSwapProcess((*Thread).__bindgen_anon_1.ApcState.Process, Process);

    // read_volatile prevents LLVM from proving OldIrql is the zeroed init value
    // (C shim KiAcquireApcLockRaiseToSynch writes it through a pointer)
    KiExitDispatcher(core::ptr::read_volatile(&ApcLock.OldIrql));

    if !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.ApcState.ApcListHead[0])) {
        (*Thread).__bindgen_anon_1.ApcState.KernelApcPending = 1;
        HalRequestSoftwareInterrupt(APC_LEVEL);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryRuntimeProcess(
    Process: PKPROCESS,
    UserTime: *mut ULONG,
) -> ULONG {
    let Process = opaque_ptr(Process);
    let UserTime = opaque_ptr(UserTime);
    let mut ProcessLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    let mut TotalUser = (*Process).UserTime;
    let mut TotalKernel = (*Process).KernelTime;

    KiAcquireProcessLockRaiseToSynch(Process, core::ptr::addr_of_mut!(ProcessLock));

    let ListHead = core::ptr::addr_of_mut!((*Process).ThreadListHead);
    let mut NextEntry = (*ListHead).Flink;

    while NextEntry != ListHead {
        let Thread = CONTAINING_RECORD(
            NextEntry as PVOID,
            core::mem::size_of::<_KTHREAD>(),
            core::mem::offset_of!(_KTHREAD, ThreadListEntry),
        ) as PKTHREAD;

        TotalKernel += kthread_get_kernel_time(Thread);
        TotalUser += kthread_get_user_time(Thread);

        NextEntry = (*NextEntry).Flink;
    }

    KiReleaseProcessLock(core::ptr::addr_of_mut!(ProcessLock));

    core::ptr::write_volatile(UserTime, TotalUser);

    TotalKernel
}

#[no_mangle]
pub unsafe extern "system" fn KeAddSystemServiceTable(
    Base: PULONG_PTR,
    Count: *mut ULONG,
    Limit: ULONG,
    Number: *mut UCHAR,
    Index: ULONG,
) -> BOOLEAN {
    let Base = opaque_ptr(Base);
    let Count = opaque_ptr(Count);
    let Number = opaque_ptr(Number);
    if Index > (SSDT_MAX_ENTRIES - 1) as ULONG
        || !KeServiceDescriptorTable[Index as usize].Base.is_null()
        || !KeServiceDescriptorTableShadow[Index as usize].Base.is_null()
    {
        return 0u8;
    }

    KeServiceDescriptorTableShadow[Index as usize].Base = Base as ntkrnl_sys::ddk::PULONG_PTR;
    KeServiceDescriptorTableShadow[Index as usize].Limit = Limit;
    KeServiceDescriptorTableShadow[Index as usize].Number = Number;
    KeServiceDescriptorTableShadow[Index as usize].Count = Count;

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn KeRemoveSystemServiceTable(Index: ULONG) -> BOOLEAN {
    if Index > (SSDT_MAX_ENTRIES - 1) as ULONG {
        return 0u8;
    }

    if KeServiceDescriptorTable[Index as usize].Base.is_null() {
        if KeServiceDescriptorTableShadow[Index as usize].Base.is_null() {
            return 0u8;
        }
    }

    KeServiceDescriptorTableShadow[Index as usize].Base = core::ptr::null_mut();
    KeServiceDescriptorTableShadow[Index as usize].Number = core::ptr::null_mut();
    KeServiceDescriptorTableShadow[Index as usize].Limit = 0;
    KeServiceDescriptorTableShadow[Index as usize].Count = core::ptr::null_mut();

    if Index == 1 {
        KeServiceDescriptorTable[Index as usize].Base = core::ptr::null_mut();
        KeServiceDescriptorTable[Index as usize].Number = core::ptr::null_mut();
        KeServiceDescriptorTable[Index as usize].Limit = 0;
        KeServiceDescriptorTable[Index as usize].Count = core::ptr::null_mut();
    }

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/procobj.c
//  * PURPOSE:         Kernel Process Management and System Call Tables
//  * PROGRAMMERS:     Alex Ionescu
//  *                  Gregor Anich
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
// LIST_ENTRY KiProcessListHead;
// LIST_ENTRY KiProcessInSwapListHead, KiProcessOutSwapListHead;
// LIST_ENTRY KiStackInSwapListHead;
// KEVENT KiSwapEvent;
//
// KSERVICE_TABLE_DESCRIPTOR KeServiceDescriptorTable[SSDT_MAX_ENTRIES];
// KSERVICE_TABLE_DESCRIPTOR KeServiceDescriptorTableShadow[SSDT_MAX_ENTRIES];
//
// PVOID KeUserApcDispatcher;
// PVOID KeUserCallbackDispatcher;
// PVOID KeUserExceptionDispatcher;
// PVOID KeRaiseUserExceptionDispatcher;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// KiAttachProcess(IN PKTHREAD Thread,
//                 IN PKPROCESS Process,
//                 IN PKLOCK_QUEUE_HANDLE ApcLock,
//                 IN PRKAPC_STATE SavedApcState)
// {
// #if 0
//     PLIST_ENTRY ListHead, NextEntry;
//     PKTHREAD CurrentThread;
// #endif
//     ASSERT(Process != Thread->ApcState.Process);
//
//     /* Increase Stack Count */
//     ASSERT(Process->StackCount != MAXULONG_PTR);
//     Process->StackCount++;
//
//     /* Swap the APC Environment */
//     KiMoveApcState(&Thread->ApcState, SavedApcState);
//
//     /* Reinitialize Apc State */
//     InitializeListHead(&Thread->ApcState.ApcListHead[KernelMode]);
//     InitializeListHead(&Thread->ApcState.ApcListHead[UserMode]);
//     Thread->ApcState.Process = Process;
//     Thread->ApcState.KernelApcInProgress = FALSE;
//     Thread->ApcState.KernelApcPending = FALSE;
//     Thread->ApcState.UserApcPending = FALSE;
//
//     /* Update Environment Pointers if needed*/
//     if (SavedApcState == &Thread->SavedApcState)
//     {
//         Thread->ApcStatePointer[OriginalApcEnvironment] = &Thread->
//                                                           SavedApcState;
//         Thread->ApcStatePointer[AttachedApcEnvironment] = &Thread->ApcState;
//         Thread->ApcStateIndex = AttachedApcEnvironment;
//     }
//
//     /* Check if the process is paged in */
//     if (Process->State == ProcessInMemory)
//     {
//         /* Scan the ready list */
// #if 0
//         ListHead = &Process->ReadyListHead;
//         NextEntry = ListHead->Flink;
//         while (NextEntry != ListHead)
//         {
//             /* Get the thread */
//             CurrentThread = CONTAINING_RECORD(NextEntry, KTHREAD, WaitListEntry);
//
//             /* Remove it */
//             RemoveEntryList(NextEntry);
//             CurrentThread->ProcessReadyQueue = FALSE;
//
//             /* Mark it ready */
//             KiReadyThread(CurrentThread);
//
//             /* Go to the next one */
//             NextEntry = ListHead->Flink;
//         }
// #endif
//
//         /* Release dispatcher lock */
//         KiReleaseDispatcherLockFromSynchLevel();
//
//         /* Release lock */
//         KiReleaseApcLockFromSynchLevel(ApcLock);
//
//         /* Swap Processes */
//         KiSwapProcess(Process, SavedApcState->Process);
//
//         /* Exit the dispatcher */
//         KiExitDispatcher(ApcLock->OldIrql);
//     }
//     else
//     {
//         DPRINT1("Errr. ReactOS doesn't support paging out processes yet...\n");
//         ASSERT(FALSE);
//     }
// }
//
// VOID
// NTAPI
// KeInitializeProcess(IN OUT PKPROCESS Process,
//                     IN KPRIORITY Priority,
//                     IN KAFFINITY Affinity,
//                     IN PULONG_PTR DirectoryTableBase,
//                     IN BOOLEAN Enable)
// {
// #ifdef CONFIG_SMP
//     ULONG i = 0;
//     UCHAR IdealNode = 0;
//     PKNODE Node;
// #endif
//
//     /* Initialize the Dispatcher Header */
//     Process->Header.Type = ProcessObject;
//     Process->Header.Size = sizeof(KPROCESS) / sizeof(ULONG);
//     Process->Header.SignalState = 0;
//     InitializeListHead(&(Process->Header.WaitListHead));
//
//     /* Initialize Scheduler Data, Alignment Faults and Set the PDE */
//     Process->Affinity = Affinity;
//     Process->BasePriority = (CHAR)Priority;
//     Process->QuantumReset = 6;
//     Process->DirectoryTableBase[0] = DirectoryTableBase[0];
//     Process->DirectoryTableBase[1] = DirectoryTableBase[1];
//     Process->AutoAlignment = Enable;
// #if defined(_M_IX86)
//     Process->IopmOffset = KiComputeIopmOffset(IO_ACCESS_MAP_NONE);
// #endif
//
//     /* Initialize the lists */
//     InitializeListHead(&Process->ThreadListHead);
//     InitializeListHead(&Process->ProfileListHead);
//     InitializeListHead(&Process->ReadyListHead);
//
//     /* Initialize the current State */
//     Process->State = ProcessInMemory;
//
//     /* Check how many Nodes there are on the system */
// #ifdef CONFIG_SMP
//     if (KeNumberNodes > 1)
//     {
//         /* Set the new seed */
//         KeProcessNodeSeed = (KeProcessNodeSeed + 1) / KeNumberNodes;
//         IdealNode = KeProcessNodeSeed;
//
//         /* Loop every node */
//         do
//         {
//             /* Check if the affinity matches */
//             if (KeNodeBlock[IdealNode]->ProcessorMask != Affinity) break;
//
//             /* No match, try next Ideal Node and increase node loop index */
//             IdealNode++;
//             i++;
//
//             /* Check if the Ideal Node is beyond the total number of nodes */
//             if (IdealNode >= KeNumberNodes)
//             {
//                 /* Normalize the Ideal Node */
//                 IdealNode -= KeNumberNodes;
//             }
//         } while (i < KeNumberNodes);
//     }
//
//     /* Set the ideal node and get the ideal node block */
//     Process->IdealNode = IdealNode;
//     Node = KeNodeBlock[IdealNode];
//     ASSERT(Node->ProcessorMask & Affinity);
//
//     /* Find the matching affinity set to calculate the thread seed */
//     Affinity &= Node->ProcessorMask;
//     Process->ThreadSeed = KeFindNextRightSetAffinity(Node->Seed,
//                                                      (ULONG)Affinity);
//     Node->Seed = Process->ThreadSeed;
// #endif
// }
//
// ULONG
// NTAPI
// KeSetProcess(IN PKPROCESS Process,
//              IN KPRIORITY Increment,
//              IN BOOLEAN InWait)
// {
//     KIRQL OldIrql;
//     ULONG OldState;
//     ASSERT_PROCESS(Process);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Lock Dispatcher */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Get Old State */
//     OldState = Process->Header.SignalState;
//
//     /* Signal the Process */
//     Process->Header.SignalState = TRUE;
//
//     /* Check if was unsignaled and has waiters */
//     if (!(OldState) &&
//         !(IsListEmpty(&Process->Header.WaitListHead)))
//     {
//         /* Unwait the threads */
//         KxUnwaitThread(&Process->Header, Increment);
//     }
//
//     /* Release Dispatcher Database */
//     KiReleaseDispatcherLock(OldIrql);
//
//     /* Return the previous State */
//     return OldState;
// }
//
// VOID
// NTAPI
// KeSetQuantumProcess(IN PKPROCESS Process,
//                     IN UCHAR Quantum)
// {
//     KLOCK_QUEUE_HANDLE ProcessLock;
//     PLIST_ENTRY NextEntry, ListHead;
//     PKTHREAD Thread;
//     ASSERT_PROCESS(Process);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Lock the process */
//     KiAcquireProcessLockRaiseToSynch(Process, &ProcessLock);
//
//     /* Set new quantum */
//     Process->QuantumReset = Quantum;
//
//     /* Loop all child threads */
//     ListHead = &Process->ThreadListHead;
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         /* Get the thread */
//         Thread = CONTAINING_RECORD(NextEntry, KTHREAD, ThreadListEntry);
//
//         /* Set quantum */
//         Thread->QuantumReset = Quantum;
//
//         /* Go to the next one */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release lock */
//     KiReleaseProcessLock(&ProcessLock);
// }
//
// KAFFINITY
// NTAPI
// KeSetAffinityProcess(IN PKPROCESS Process,
//                      IN KAFFINITY Affinity)
// {
//
//     KLOCK_QUEUE_HANDLE ProcessLock;
//     PLIST_ENTRY NextEntry, ListHead;
//     KAFFINITY OldAffinity;
//     PKTHREAD Thread;
//     ASSERT_PROCESS(Process);
//     ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL);
//     ASSERT((Affinity & KeActiveProcessors) != 0);
//
//     /* Lock the process */
//     KiAcquireProcessLockRaiseToSynch(Process, &ProcessLock);
//
//     /* Acquire the dispatcher lock */
//     KiAcquireDispatcherLockAtSynchLevel();
//
//     /* Capture old affinity and update it */
//     OldAffinity = Process->Affinity;
//     Process->Affinity = Affinity;
//
//     /* Loop all child threads */
//     ListHead = &Process->ThreadListHead;
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         /* Get the thread */
//         Thread = CONTAINING_RECORD(NextEntry, KTHREAD, ThreadListEntry);
//
//         /* Set affinity on it */
//         KiSetAffinityThread(Thread, Affinity);
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release Dispatcher Database */
//     KiReleaseDispatcherLockFromSynchLevel();
//
//     /* Release the process lock */
//     KiReleaseProcessLockFromSynchLevel(&ProcessLock);
//     KiExitDispatcher(ProcessLock.OldIrql);
//
//     /* Return previous affinity */
//     return OldAffinity;
// }
//
// BOOLEAN
// NTAPI
// KeSetAutoAlignmentProcess(IN PKPROCESS Process,
//                           IN BOOLEAN Enable)
// {
//     /* Set or reset the bit depending on what the enable flag says */
//     if (Enable)
//     {
//         return InterlockedBitTestAndSet(&Process->ProcessFlags,
//                                         KPSF_AUTO_ALIGNMENT_BIT);
//     }
//     else
//     {
//         return InterlockedBitTestAndReset(&Process->ProcessFlags,
//                                           KPSF_AUTO_ALIGNMENT_BIT);
//     }
// }
//
// BOOLEAN
// NTAPI
// KeSetDisableBoostProcess(IN PKPROCESS Process,
//                          IN BOOLEAN Disable)
// {
//     /* Set or reset the bit depending on what the disable flag says */
//     if (Disable)
//     {
//         return InterlockedBitTestAndSet(&Process->ProcessFlags,
//                                         KPSF_DISABLE_BOOST_BIT);
//     }
//     else
//     {
//         return InterlockedBitTestAndReset(&Process->ProcessFlags,
//                                           KPSF_DISABLE_BOOST_BIT);
//     }
// }
//
// KPRIORITY
// NTAPI
// KeSetPriorityAndQuantumProcess(IN PKPROCESS Process,
//                                IN KPRIORITY Priority,
//                                IN UCHAR Quantum OPTIONAL)
// {
//     KLOCK_QUEUE_HANDLE ProcessLock;
//     KPRIORITY Delta;
//     PLIST_ENTRY NextEntry, ListHead;
//     KPRIORITY NewPriority, OldPriority;
//     PKTHREAD Thread;
//     ASSERT_PROCESS(Process);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Check if the process already has this priority */
//     if (Process->BasePriority == Priority) return Process->BasePriority;
//
//     /* If the caller gave priority 0, normalize to 1 */
//     if (!Priority) Priority = LOW_PRIORITY + 1;
//
//     /* Lock the process */
//     KiAcquireProcessLockRaiseToSynch(Process, &ProcessLock);
//
//     /* Acquire the dispatcher lock */
//     KiAcquireDispatcherLockAtSynchLevel();
//
//     /* Check if we are modifying the quantum too */
//     if (Quantum) Process->QuantumReset = Quantum;
//
//     /* Save the current base priority and update it */
//     OldPriority = Process->BasePriority;
//     Process->BasePriority = (SCHAR)Priority;
//
//     /* Calculate the priority delta */
//     Delta = Priority - OldPriority;
//
//     /* Set the list head and list entry */
//     ListHead = &Process->ThreadListHead;
//     NextEntry = ListHead->Flink;
//
//     /* Check if this is a real-time priority */
//     if (Priority >= LOW_REALTIME_PRIORITY)
//     {
//         /* Loop the thread list */
//         while (NextEntry != ListHead)
//         {
//             /* Get the thread */
//             Thread = CONTAINING_RECORD(NextEntry, KTHREAD, ThreadListEntry);
//
//             /* Update the quantum if we had one */
//             if (Quantum) Thread->QuantumReset = Quantum;
//
//             /* Acquire the thread lock */
//             KiAcquireThreadLock(Thread);
//
//             /* Calculate the new priority */
//             NewPriority = Thread->BasePriority + Delta;
//             if (NewPriority < LOW_REALTIME_PRIORITY)
//             {
//                 /* We're in real-time range, don't let it go below */
//                 NewPriority = LOW_REALTIME_PRIORITY;
//             }
//             else if (NewPriority > HIGH_PRIORITY)
//             {
//                 /* We're going beyond the maximum priority, normalize */
//                 NewPriority = HIGH_PRIORITY;
//             }
//
//             /*
//              * If priority saturation occured or the old priority was still in
//              * the real-time range, don't do anything.
//              */
//             if (!(Thread->Saturation) || (OldPriority < LOW_REALTIME_PRIORITY))
//             {
//                 /* Check if we had priority saturation */
//                 if (Thread->Saturation > 0)
//                 {
//                     /* Boost priority to maximum */
//                     NewPriority = HIGH_PRIORITY;
//                 }
//                 else if (Thread->Saturation < 0)
//                 {
//                     /* If we had negative saturation, set minimum priority */
//                     NewPriority = LOW_REALTIME_PRIORITY;
//                 }
//
//                 /* Update priority and quantum */
//                 Thread->BasePriority = (SCHAR)NewPriority;
//                 Thread->Quantum = Thread->QuantumReset;
//
//                 /* Disable decrements and update priority */
//                 Thread->PriorityDecrement = 0;
//                 KiSetPriorityThread(Thread, NewPriority);
//             }
//
//             /* Release the thread lock */
//             KiReleaseThreadLock(Thread);
//
//             /* Go to the next thread */
//             NextEntry = NextEntry->Flink;
//         }
//     }
//     else
//     {
//         /* Loop the thread list */
//         while (NextEntry != ListHead)
//         {
//             /* Get the thread */
//             Thread = CONTAINING_RECORD(NextEntry, KTHREAD, ThreadListEntry);
//
//             /* Update the quantum if we had one */
//             if (Quantum) Thread->QuantumReset = Quantum;
//
//             /* Lock the thread */
//             KiAcquireThreadLock(Thread);
//
//             /* Calculate the new priority */
//             NewPriority = Thread->BasePriority + Delta;
//             if (NewPriority >= LOW_REALTIME_PRIORITY)
//             {
//                 /* We're not real-time range, don't let it enter RT range */
//                 NewPriority = LOW_REALTIME_PRIORITY - 1;
//             }
//             else if (NewPriority <= LOW_PRIORITY)
//             {
//                 /* We're going below the minimum priority, normalize */
//                 NewPriority = 1;
//             }
//
//             /*
//              * If priority saturation occured or the old priority was still in
//              * the real-time range, don't do anything.
//              */
//             if (!(Thread->Saturation) ||
//                 (OldPriority >= LOW_REALTIME_PRIORITY))
//             {
//                 /* Check if we had priority saturation */
//                 if (Thread->Saturation > 0)
//                 {
//                     /* Boost priority to maximum */
//                     NewPriority = LOW_REALTIME_PRIORITY - 1;
//                 }
//                 else if (Thread->Saturation < 0)
//                 {
//                     /* If we had negative saturation, set minimum priority */
//                     NewPriority = 1;
//                 }
//
//                 /* Update priority and quantum */
//                 Thread->BasePriority = (SCHAR)NewPriority;
//                 Thread->Quantum = Thread->QuantumReset;
//
//                 /* Disable decrements and update priority */
//                 Thread->PriorityDecrement = 0;
//                 KiSetPriorityThread(Thread, NewPriority);
//             }
//
//             /* Release the thread lock */
//             KiReleaseThreadLock(Thread);
//
//             /* Go to the next thread */
//             NextEntry = NextEntry->Flink;
//         }
//     }
//
//     /* Release Dispatcher Database */
//     KiReleaseDispatcherLockFromSynchLevel();
//
//     /* Release the process lock */
//     KiReleaseProcessLockFromSynchLevel(&ProcessLock);
//     KiExitDispatcher(ProcessLock.OldIrql);
//
//     /* Return previous priority */
//     return OldPriority;
// }
//
// VOID
// NTAPI
// KeQueryValuesProcess(IN PKPROCESS Process,
//                      PPROCESS_VALUES Values)
// {
//     PEPROCESS EProcess;
//     PLIST_ENTRY NextEntry;
//     ULONG TotalKernel, TotalUser;
//     KLOCK_QUEUE_HANDLE ProcessLock;
//
//     ASSERT_PROCESS(Process);
//     ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL);
//
//     /* Lock the process */
//     KiAcquireProcessLockRaiseToSynch(Process, &ProcessLock);
//
//     /* Initialize user and kernel times */
//     TotalKernel = Process->KernelTime;
//     TotalUser = Process->UserTime;
//
//     /* Copy the IO_COUNTERS from the process */
//     EProcess = (PEPROCESS)Process;
//     Values->IoInfo.ReadOperationCount = EProcess->ReadOperationCount.QuadPart;
//     Values->IoInfo.WriteOperationCount = EProcess->WriteOperationCount.QuadPart;
//     Values->IoInfo.OtherOperationCount = EProcess->OtherOperationCount.QuadPart;
//     Values->IoInfo.ReadTransferCount = EProcess->ReadTransferCount.QuadPart;
//     Values->IoInfo.WriteTransferCount = EProcess->WriteTransferCount.QuadPart;
//     Values->IoInfo.OtherTransferCount = EProcess->OtherTransferCount.QuadPart;
//
//     /* Loop all child threads and sum up their times */
//     for (NextEntry = Process->ThreadListHead.Flink;
//          NextEntry != &Process->ThreadListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         PKTHREAD Thread;
//
//         /* Get the thread */
//         Thread = CONTAINING_RECORD(NextEntry, KTHREAD, ThreadListEntry);
//
//         /* Sum up times */
//         TotalKernel += Thread->KernelTime;
//         TotalUser += Thread->UserTime;
//     }
//
//     /* Release the process lock */
//     KiReleaseProcessLock(&ProcessLock);
//
//     /* Compute total times */
//     Values->TotalKernelTime.QuadPart = TotalKernel * (LONGLONG)KeMaximumIncrement;
//     Values->TotalUserTime.QuadPart = TotalUser * (LONGLONG)KeMaximumIncrement;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeAttachProcess(IN PKPROCESS Process)
// {
//     KLOCK_QUEUE_HANDLE ApcLock;
//     PKTHREAD Thread = KeGetCurrentThread();
//     ASSERT_PROCESS(Process);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Check if we're already in that process */
//     if (Thread->ApcState.Process == Process) return;
//
//     /* Check if a DPC is executing or if we're already attached */
//     if ((Thread->ApcStateIndex != OriginalApcEnvironment) ||
//         (KeIsExecutingDpc()))
//     {
//         /* Invalid attempt */
//         KeBugCheckEx(INVALID_PROCESS_ATTACH_ATTEMPT,
//                      (ULONG_PTR)Process,
//                      (ULONG_PTR)Thread->ApcState.Process,
//                      Thread->ApcStateIndex,
//                      KeIsExecutingDpc());
//     }
//     else
//     {
//         /* Acquire APC Lock */
//         KiAcquireApcLockRaiseToSynch(Thread, &ApcLock);
//
//         /* Acquire the dispatcher lock */
//         KiAcquireDispatcherLockAtSynchLevel();
//
//         /* Legit attach attempt: do it! */
//         KiAttachProcess(Thread, Process, &ApcLock, &Thread->SavedApcState);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeDetachProcess(VOID)
// {
//     PKTHREAD Thread = KeGetCurrentThread();
//     KLOCK_QUEUE_HANDLE ApcLock;
//     PKPROCESS Process;
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Check if it's attached */
//     if (Thread->ApcStateIndex == OriginalApcEnvironment) return;
//
//     /* Acquire APC Lock */
//     KiAcquireApcLockRaiseToSynch(Thread, &ApcLock);
//
//     /* Check for invalid attach attempts */
//     if ((Thread->ApcState.KernelApcInProgress) ||
//         !(IsListEmpty(&Thread->ApcState.ApcListHead[KernelMode])) ||
//         !(IsListEmpty(&Thread->ApcState.ApcListHead[UserMode])))
//     {
//         /* Crash the system */
//         KeBugCheck(INVALID_PROCESS_DETACH_ATTEMPT);
//     }
//
//     /* Get the process */
//     Process = Thread->ApcState.Process;
//
//     /* Acquire dispatcher lock */
//     KiAcquireDispatcherLockAtSynchLevel();
//
//     /* Decrease the stack count */
//     ASSERT(Process->StackCount != 0);
//     ASSERT(Process->State == ProcessInMemory);
//     Process->StackCount--;
//
//     /* Check if we can swap the process out */
//     if (!Process->StackCount)
//     {
//         /* FIXME: Swap the process out */
//     }
//
//     /* Release dispatcher lock */
//     KiReleaseDispatcherLockFromSynchLevel();
//
//     /* Restore the APC State */
//     KiMoveApcState(&Thread->SavedApcState, &Thread->ApcState);
//     Thread->SavedApcState.Process = NULL;
//     Thread->ApcStatePointer[OriginalApcEnvironment] = &Thread->ApcState;
//     Thread->ApcStatePointer[AttachedApcEnvironment] = &Thread->SavedApcState;
//     Thread->ApcStateIndex = OriginalApcEnvironment;
//
//     /* Release lock */
//     KiReleaseApcLockFromSynchLevel(&ApcLock);
//
//     /* Swap Processes */
//     KiSwapProcess(Thread->ApcState.Process, Process);
//
//     /* Exit the dispatcher */
//     KiExitDispatcher(ApcLock.OldIrql);
//
//     /* Check if we have pending APCs */
//     if (!(IsListEmpty(&Thread->ApcState.ApcListHead[KernelMode])))
//     {
//         /* What do you know, we do! Request them to be delivered */
//         Thread->ApcState.KernelApcPending = TRUE;
//         HalRequestSoftwareInterrupt(APC_LEVEL);
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeIsAttachedProcess(VOID)
// {
//     /* Return the APC State */
//     return KeGetCurrentThread()->ApcStateIndex;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeStackAttachProcess(IN PKPROCESS Process,
//                      OUT PRKAPC_STATE ApcState)
// {
//     KLOCK_QUEUE_HANDLE ApcLock;
//     PKTHREAD Thread = KeGetCurrentThread();
//     ASSERT_PROCESS(Process);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Crash system if DPC is being executed! */
//     if (KeIsExecutingDpc())
//     {
//         /* Executing a DPC, crash! */
//         KeBugCheckEx(INVALID_PROCESS_ATTACH_ATTEMPT,
//                      (ULONG_PTR)Process,
//                      (ULONG_PTR)Thread->ApcState.Process,
//                      Thread->ApcStateIndex,
//                      KeIsExecutingDpc());
//     }
//
//     /* Check if we are already in the target process */
//     if (Thread->ApcState.Process == Process)
//     {
//         /* Set magic value so we don't crash later when detaching */
//         ApcState->Process = (PKPROCESS)1;
//         return;
//     }
//
//     /* Acquire APC Lock */
//     KiAcquireApcLockRaiseToSynch(Thread, &ApcLock);
//
//     /* Acquire dispatcher lock */
//     KiAcquireDispatcherLockAtSynchLevel();
//
//     /* Check if the Current Thread is already attached */
//     if (Thread->ApcStateIndex != OriginalApcEnvironment)
//     {
//         /* We're already attached, so save the APC State into what we got */
//         KiAttachProcess(Thread, Process, &ApcLock, ApcState);
//     }
//     else
//     {
//         /* We're not attached, so save the APC State into SavedApcState */
//         KiAttachProcess(Thread, Process, &ApcLock, &Thread->SavedApcState);
//         ApcState->Process = NULL;
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeUnstackDetachProcess(IN PRKAPC_STATE ApcState)
// {
//     KLOCK_QUEUE_HANDLE ApcLock;
//     PKTHREAD Thread = KeGetCurrentThread();
//     PKPROCESS Process;
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Check for magic value meaning we were already in the same process */
//     if (ApcState->Process == (PKPROCESS)1) return;
//
//     /* Loop to make sure no APCs are pending  */
//     for (;;)
//     {
//         /* Acquire APC Lock */
//         KiAcquireApcLockRaiseToSynch(Thread, &ApcLock);
//
//         /* Check if a kernel APC is pending */
//         if (Thread->ApcState.KernelApcPending)
//         {
//             /* Check if kernel APC should be delivered */
//             if (!(Thread->KernelApcDisable) && (ApcLock.OldIrql <= APC_LEVEL))
//             {
//                 /* Release the APC lock so that the APC can be delivered */
//                 KiReleaseApcLock(&ApcLock);
//                 continue;
//             }
//         }
//
//         /* Otherwise, break out */
//         break;
//     }
//
//     /*
//      * Check if the process isn't attacked, or has a Kernel APC in progress
//      * or has pending APC of any kind.
//      */
//     if ((Thread->ApcStateIndex == OriginalApcEnvironment) ||
//         (Thread->ApcState.KernelApcInProgress) ||
//         (!IsListEmpty(&Thread->ApcState.ApcListHead[KernelMode])) ||
//         (!IsListEmpty(&Thread->ApcState.ApcListHead[UserMode])))
//     {
//         /* Bugcheck the system */
//         KeBugCheck(INVALID_PROCESS_DETACH_ATTEMPT);
//     }
//
//     /* Get the process */
//     Process = Thread->ApcState.Process;
//
//     /* Acquire dispatcher lock */
//     KiAcquireDispatcherLockAtSynchLevel();
//
//     /* Decrease the stack count */
//     ASSERT(Process->StackCount != 0);
//     ASSERT(Process->State == ProcessInMemory);
//     Process->StackCount--;
//
//     /* Check if we can swap the process out */
//     if (!Process->StackCount)
//     {
//         /* FIXME: Swap the process out */
//     }
//
//     /* Release dispatcher lock */
//     KiReleaseDispatcherLockFromSynchLevel();
//
//     /* Check if there's an APC state to restore */
//     if (ApcState->Process)
//     {
//         /* Restore the APC State */
//         KiMoveApcState(ApcState, &Thread->ApcState);
//     }
//     else
//     {
//         /* The ApcState parameter is useless, so use the saved data and reset it */
//         KiMoveApcState(&Thread->SavedApcState, &Thread->ApcState);
//         Thread->SavedApcState.Process = NULL;
//         Thread->ApcStateIndex = OriginalApcEnvironment;
//         Thread->ApcStatePointer[OriginalApcEnvironment] = &Thread->ApcState;
//         Thread->ApcStatePointer[AttachedApcEnvironment] = &Thread->SavedApcState;
//     }
//
//     /* Release lock */
//     KiReleaseApcLockFromSynchLevel(&ApcLock);
//
//     /* Swap Processes */
//     KiSwapProcess(Thread->ApcState.Process, Process);
//
//     /* Exit the dispatcher */
//     KiExitDispatcher(ApcLock.OldIrql);
//
//     /* Check if we have pending APCs */
//     if (!(IsListEmpty(&Thread->ApcState.ApcListHead[KernelMode])))
//     {
//         /* What do you know, we do! Request them to be delivered */
//         Thread->ApcState.KernelApcPending = TRUE;
//         HalRequestSoftwareInterrupt(APC_LEVEL);
//     }
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// KeQueryRuntimeProcess(IN PKPROCESS Process,
//                       OUT PULONG UserTime)
// {
//     ULONG TotalUser, TotalKernel;
//     KLOCK_QUEUE_HANDLE ProcessLock;
//     PLIST_ENTRY NextEntry, ListHead;
//     PKTHREAD Thread;
//
//     ASSERT_PROCESS(Process);
//
//     /* Initialize user and kernel times */
//     TotalUser = Process->UserTime;
//     TotalKernel = Process->KernelTime;
//
//     /* Lock the process */
//     KiAcquireProcessLockRaiseToSynch(Process, &ProcessLock);
//
//     /* Loop all child threads and sum up their times */
//     ListHead = &Process->ThreadListHead;
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         /* Get the thread */
//         Thread = CONTAINING_RECORD(NextEntry, KTHREAD, ThreadListEntry);
//
//         /* Sum up times */
//         TotalKernel += Thread->KernelTime;
//         TotalUser += Thread->UserTime;
//
//         /* Go to the next one */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release lock */
//     KiReleaseProcessLock(&ProcessLock);
//
//     /* Return the user time */
//     *UserTime = TotalUser;
//
//     /* Return the kernel time */
//     return TotalKernel;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeAddSystemServiceTable(IN PULONG_PTR Base,
//                         IN PULONG Count OPTIONAL,
//                         IN ULONG Limit,
//                         IN PUCHAR Number,
//                         IN ULONG Index)
// {
//     PAGED_CODE();
//
//     /* Check if descriptor table entry is free */
//     if ((Index > SSDT_MAX_ENTRIES - 1) ||
//         (KeServiceDescriptorTable[Index].Base) ||
//         (KeServiceDescriptorTableShadow[Index].Base))
//     {
//         /* It's not, fail */
//         return FALSE;
//     }
//
//     /* Initialize the shadow service descriptor table */
//     KeServiceDescriptorTableShadow[Index].Base = Base;
//     KeServiceDescriptorTableShadow[Index].Limit = Limit;
//     KeServiceDescriptorTableShadow[Index].Number = Number;
//     KeServiceDescriptorTableShadow[Index].Count = Count;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeRemoveSystemServiceTable(IN ULONG Index)
// {
//     PAGED_CODE();
//
//     /* Make sure the Index is valid */
//     if (Index > (SSDT_MAX_ENTRIES - 1)) return FALSE;
//
//     /* Is there a Normal Descriptor Table? */
//     if (!KeServiceDescriptorTable[Index].Base)
//     {
//         /* Not with the index, is there a shadow at least? */
//         if (!KeServiceDescriptorTableShadow[Index].Base) return FALSE;
//     }
//
//     /* Now clear from the Shadow Table. */
//     KeServiceDescriptorTableShadow[Index].Base = NULL;
//     KeServiceDescriptorTableShadow[Index].Number = NULL;
//     KeServiceDescriptorTableShadow[Index].Limit = 0;
//     KeServiceDescriptorTableShadow[Index].Count = NULL;
//
//     /* Check if we should clean from the Master one too */
//     if (Index == 1)
//     {
//         KeServiceDescriptorTable[Index].Base = NULL;
//         KeServiceDescriptorTable[Index].Number = NULL;
//         KeServiceDescriptorTable[Index].Limit = 0;
//         KeServiceDescriptorTable[Index].Count = NULL;
//     }
//
//     /* Return success */
//     return TRUE;
// }
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
