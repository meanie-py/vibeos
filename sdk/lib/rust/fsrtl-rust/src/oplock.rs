use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    ACCESS_MASK,
    ExAcquireFastMutex,
    ExAcquireFastMutexUnsafe,
    ExInitializeFastMutex,
    ExReleaseFastMutex,
    ExReleaseFastMutexUnsafe,
    FAST_MUTEX,
    IRP,
    IoAcquireCancelSpinLock,
    IoGetCurrentIrpStackLocation,
    IoIsOperationSynchronous,
    IoMarkIrpPending,
    IoReleaseCancelSpinLock,
    IoSetCancelRoutine,
    IofCompleteRequest,
    KEVENT,
    KeInitializeEvent,
    KeWaitForSingleObject,
    PDEVICE_OBJECT,
    PIO_STACK_LOCATION,
    PIRP,
    PKEVENT,
    POPLOCK,
    POPLOCK_FS_PREPOST_IRP,
    POPLOCK_WAIT_COMPLETE_ROUTINE,
    _ETHREAD,
    PFILE_OBJECT,
};
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::ke::KeSetEvent;
use ntkrnl_sys::mm::{NonPagedPool, PagedPool};
use ntkrnl_sys::ob::{ObDereferenceObject, ObReferenceObject};
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn KeGetCurrentThread() -> ntkrnl_sys::ddk::PKTHREAD;
}

#[inline(always)]
unsafe fn IoCompleteRequest(Irp: PIRP, PriorityBoost: i32) {
    IofCompleteRequest(Irp, PriorityBoost as i8);
}

const NO_OPLOCK: ULONG = 0x1;
const LEVEL_1_OPLOCK: ULONG = 0x2;
const BATCH_OPLOCK: ULONG = 0x4;
const FILTER_OPLOCK: ULONG = 0x8;
const LEVEL_2_OPLOCK: ULONG = 0x10;

const EXCLUSIVE_LOCK: ULONG = 0x40;
const PENDING_LOCK: ULONG = 0x80;

const BROKEN_TO_LEVEL_2: ULONG = 0x100;
const BROKEN_TO_NONE: ULONG = 0x200;
const BROKEN_TO_NONE_FROM_LEVEL_2: ULONG = 0x400;
const BROKEN_TO_CLOSE_PENDING: ULONG = 0x800;
const BROKEN_ANY: ULONG = BROKEN_TO_LEVEL_2 | BROKEN_TO_NONE | BROKEN_TO_NONE_FROM_LEVEL_2 | BROKEN_TO_CLOSE_PENDING;

const TAG_OPLOCK: ULONG = 0x6F725346;
const IO_DISK_INCREMENT: i32 = 1;
const IO_NO_INCREMENT: i32 = 0;

#[repr(C)]
struct INTERNAL_OPLOCK {
    ExclusiveIrp: PIRP,
    FileObject: PFILE_OBJECT,
    SharedListHead: LIST_ENTRY,
    WaitListHead: LIST_ENTRY,
    Flags: ULONG,
    IntLock: *mut FAST_MUTEX,
}

type PINTERNAL_OPLOCK = *mut INTERNAL_OPLOCK;

#[repr(C)]
struct WAIT_CONTEXT {
    WaitListEntry: LIST_ENTRY,
    Irp: PIRP,
    CompletionRoutine: POPLOCK_WAIT_COMPLETE_ROUTINE,
    CompletionContext: PVOID,
    Reserved: ULONG,
    SavedInformation: ULONG_PTR,
}

type PWAIT_CONTEXT = *mut WAIT_CONTEXT;

unsafe extern "system" fn FsRtlNotifyCompletion(Context: PVOID, Irp: PIRP) {
    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
}

unsafe extern "system" fn FsRtlCompletionRoutinePriv(Context: PVOID, Irp: PIRP) {
    let wait_event = Context as PKEVENT;
    KeSetEvent(wait_event as PVOID, IO_NO_INCREMENT, 0u8);
}

unsafe fn FsRtlRemoveAndCompleteWaitIrp(wait_ctx: PWAIT_CONTEXT) {
    RemoveEntryList(core::ptr::addr_of_mut!((*wait_ctx).WaitListEntry) as PLIST_ENTRY);
    let irp = (*wait_ctx).Irp;

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
    IoSetCancelRoutine(irp, None);
    IoReleaseCancelSpinLock((*irp).CancelIrql);

    (*irp).IoStatus.Information = (*wait_ctx).SavedInformation as _;
    (*irp).IoStatus.__bindgen_anon_1.Status = if (*irp).Cancel != 0 {
        STATUS_CANCELLED
    } else {
        STATUS_SUCCESS
    };

    ((*wait_ctx).CompletionRoutine.unwrap())((*wait_ctx).CompletionContext, irp);

    ExFreePoolWithTag(wait_ctx as PVOID, TAG_OPLOCK);
}

unsafe extern "system" fn FsRtlCancelWaitIrp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let oplock = (*Irp).IoStatus.Information as PINTERNAL_OPLOCK;

    IoSetCancelRoutine(Irp, None);
    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    ExAcquireFastMutex((*oplock).IntLock);
    let mut next_entry = (*oplock).WaitListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*oplock).WaitListHead) as PLIST_ENTRY {
        let wait_ctx = ((next_entry as usize) - core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry)) as PWAIT_CONTEXT;

        if (*(*wait_ctx).Irp).Cancel != 0 {
            FsRtlRemoveAndCompleteWaitIrp(wait_ctx);
        }

        next_entry = (*next_entry).Flink;
    }
    ExReleaseFastMutex((*oplock).IntLock);
}

unsafe fn FsRtlWaitOnIrp(
    oplock: PINTERNAL_OPLOCK,
    irp: PIRP,
    completion_context: PVOID,
    completion_routine: POPLOCK_WAIT_COMPLETE_ROUTINE,
    post_irp_routine: POPLOCK_FS_PREPOST_IRP,
    wait_event: PKEVENT,
) {
    let mut locked = true;

    let wait_ctx = ExAllocatePoolWithTag(
        PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
        core::mem::size_of::<WAIT_CONTEXT>(),
        TAG_OPLOCK,
    ) as PWAIT_CONTEXT;
    (*wait_ctx).Irp = irp;
    (*wait_ctx).SavedInformation = (*irp).IoStatus.Information as usize;

    if completion_routine.is_some() {
        (*wait_ctx).CompletionRoutine = completion_routine;
        (*wait_ctx).CompletionContext = completion_context;
    } else {
        (*wait_ctx).CompletionRoutine = Some(abi_cast!(FsRtlCompletionRoutinePriv));
        (*wait_ctx).CompletionContext = wait_event as PVOID;
        KeInitializeEvent(wait_event, 0, 0u8);
    }

    if let Some(post_routine) = post_irp_routine {
        post_routine(completion_context, irp);
    }

    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

    InsertHeadList(
        core::ptr::addr_of_mut!((*oplock).WaitListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*wait_ctx).WaitListEntry) as PLIST_ENTRY,
    );

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
    (*irp).IoStatus.Information = oplock as _;

    if (*irp).Cancel != 0 {
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        locked = false;

        if completion_routine.is_some() {
            IoMarkIrpPending(irp);
        }
        FsRtlCancelWaitIrp(core::ptr::null_mut(), irp);
    } else {
        IoSetCancelRoutine(irp, Some(abi_cast!(FsRtlCancelWaitIrp)));
        IoReleaseCancelSpinLock((*irp).CancelIrql);
        if completion_routine.is_some() {
            IoMarkIrpPending(irp);
        } else {
            ExReleaseFastMutexUnsafe((*oplock).IntLock);
            locked = false;
            KeWaitForSingleObject(
                wait_event as PVOID,
                0,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
        }
    }

    if locked {
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
    }
}

unsafe fn FsRtlOplockBreakNotify(
    oplock: PINTERNAL_OPLOCK,
    stack: PIO_STACK_LOCATION,
    irp: PIRP,
) -> NTSTATUS {
    if oplock.is_null() {
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        return STATUS_SUCCESS;
    }

    ExAcquireFastMutexUnsafe((*oplock).IntLock);
    if ((*oplock).Flags & BROKEN_TO_LEVEL_2) == 0 {
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_SUCCESS;
    }

    if ((*oplock).Flags & PENDING_LOCK) != 0 {
        (*oplock).FileObject = core::ptr::null_mut();
        (*oplock).Flags = NO_OPLOCK;
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_SUCCESS;
    }

    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    FsRtlWaitOnIrp(
        oplock,
        irp,
        core::ptr::null_mut(),
        Some(abi_cast!(FsRtlNotifyCompletion)),
        None,
        core::ptr::null_mut(),
    );
    STATUS_SUCCESS
}

unsafe fn FsRtlRemoveAndCompleteIrp(irp: PIRP) {
    let stack = IoGetCurrentIrpStackLocation(irp);

    ObDereferenceObject((*stack).FileObject as PVOID);

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
    IoSetCancelRoutine(irp, None);
    IoReleaseCancelSpinLock((*irp).CancelIrql);

    RemoveEntryList(core::ptr::addr_of_mut!((*irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY);

    (*irp).IoStatus.Information = 3;
    (*irp).IoStatus.__bindgen_anon_1.Status = if (*irp).Cancel != 0 {
        STATUS_CANCELLED
    } else {
        STATUS_SUCCESS
    };

    IoCompleteRequest(irp, IO_DISK_INCREMENT);
}

unsafe extern "system" fn FsRtlCancelOplockIIIrp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let oplock = (*Irp).IoStatus.Information as PINTERNAL_OPLOCK;

    IoSetCancelRoutine(Irp, None);
    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    let mut removed = false;
    ExAcquireFastMutex((*oplock).IntLock);
    let mut next_entry = (*oplock).SharedListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*oplock).SharedListHead) as PLIST_ENTRY {
        let list_irp = ((next_entry as usize) - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;

        if (*list_irp).Cancel != 0 {
            FsRtlRemoveAndCompleteIrp(list_irp);
            removed = true;
        }

        next_entry = (*next_entry).Flink;
    }

    if removed && IsListEmpty(core::ptr::addr_of_mut!((*oplock).SharedListHead) as PLIST_ENTRY) {
        (*oplock).Flags = NO_OPLOCK;
    }
    ExReleaseFastMutex((*oplock).IntLock);
}

unsafe fn FsRtlAcknowledgeOplockBreak(
    oplock: PINTERNAL_OPLOCK,
    stack: PIO_STACK_LOCATION,
    irp: PIRP,
    switch_to_level2: BOOLEAN,
) -> NTSTATUS {
    if oplock.is_null() {
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_OPLOCK_PROTOCOL;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        return STATUS_INVALID_OPLOCK_PROTOCOL;
    }

    ExAcquireFastMutexUnsafe((*oplock).IntLock);
    let mut locked = true;
    if (*oplock).FileObject != (*stack).FileObject {
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_OPLOCK_PROTOCOL;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_INVALID_OPLOCK_PROTOCOL;
    }

    let mut deref = true;

    if switch_to_level2 != 0 && ((*oplock).Flags & BROKEN_TO_LEVEL_2) != 0 {
        IoMarkIrpPending(irp);
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        InsertTailList(
            core::ptr::addr_of_mut!((*oplock).SharedListHead) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY,
        );

        deref = false;
        (*oplock).Flags = LEVEL_2_OPLOCK;
        (*irp).IoStatus.Information = oplock as _;

        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
        if (*irp).Cancel != 0 {
            ExReleaseFastMutexUnsafe((*oplock).IntLock);
            locked = false;
            FsRtlCancelOplockIIIrp(core::ptr::null_mut(), irp);
        } else {
            IoSetCancelRoutine(irp, Some(abi_cast!(FsRtlCancelOplockIIIrp)));
            IoReleaseCancelSpinLock((*irp).CancelIrql);
        }
    } else if ((*oplock).Flags & (BROKEN_TO_NONE | BROKEN_TO_LEVEL_2)) != 0 {
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IofCompleteRequest(irp, IO_DISK_INCREMENT as i8);
        (*oplock).Flags = NO_OPLOCK;
    } else if ((*oplock).Flags & BROKEN_TO_NONE_FROM_LEVEL_2) != 0 {
        (*irp).IoStatus.Information = 3;
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IofCompleteRequest(irp, IO_DISK_INCREMENT as i8);
        (*oplock).Flags = NO_OPLOCK;
    }

    let mut next_entry = (*oplock).WaitListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*oplock).WaitListHead) as PLIST_ENTRY {
        let wait_ctx = ((next_entry as usize) - core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry)) as PWAIT_CONTEXT;
        FsRtlRemoveAndCompleteWaitIrp(wait_ctx);
        next_entry = (*next_entry).Flink;
    }

    if deref {
        ObDereferenceObject((*oplock).FileObject as PVOID);
    }
    (*oplock).FileObject = core::ptr::null_mut();

    if locked {
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
    }

    STATUS_SUCCESS
}

unsafe fn FsRtlOpBatchBreakClosePending(
    oplock: PINTERNAL_OPLOCK,
    stack: PIO_STACK_LOCATION,
    irp: PIRP,
) -> NTSTATUS {
    if oplock.is_null() {
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_OPLOCK_PROTOCOL;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        return STATUS_INVALID_OPLOCK_PROTOCOL;
    }

    let mut status = STATUS_SUCCESS;
    ExAcquireFastMutexUnsafe((*oplock).IntLock);

    if (*oplock).FileObject == (*stack).FileObject
        && ((*oplock).Flags & (BROKEN_TO_LEVEL_2 | BROKEN_TO_NONE | BROKEN_TO_NONE_FROM_LEVEL_2)) != 0
    {
        if ((*oplock).Flags & (PENDING_LOCK | LEVEL_1_OPLOCK)) != 0 {
            if ((*oplock).Flags & LEVEL_1_OPLOCK) != 0 {
                ObDereferenceObject((*oplock).FileObject as PVOID);
            }

            (*oplock).Flags = NO_OPLOCK;
            (*oplock).FileObject = core::ptr::null_mut();

            let mut next_entry = (*oplock).WaitListHead.Flink;
            while next_entry != core::ptr::addr_of_mut!((*oplock).WaitListHead) as PLIST_ENTRY {
                let wait_ctx = ((next_entry as usize) - core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry)) as PWAIT_CONTEXT;
                FsRtlRemoveAndCompleteWaitIrp(wait_ctx);
                next_entry = (*next_entry).Flink;
            }
        } else {
            (*oplock).Flags &= !BROKEN_ANY;
            (*oplock).Flags |= BROKEN_TO_CLOSE_PENDING;
        }
    } else {
        status = STATUS_INVALID_OPLOCK_PROTOCOL;
    }

    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(irp, IO_DISK_INCREMENT);
    ExReleaseFastMutexUnsafe((*oplock).IntLock);

    status
}

unsafe fn FsRtlAllocateOplock() -> PINTERNAL_OPLOCK {
    let mut oplock: PINTERNAL_OPLOCK = core::ptr::null_mut();

    oplock = ExAllocatePoolWithTag(
        PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE | 0x00000100,
        core::mem::size_of::<INTERNAL_OPLOCK>(),
        TAG_OPLOCK,
    ) as PINTERNAL_OPLOCK;
    core::ptr::write_bytes(oplock as *mut u8, 0, core::mem::size_of::<INTERNAL_OPLOCK>());
    (*oplock).IntLock = ExAllocatePoolWithTag(
        NonPagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
        core::mem::size_of::<FAST_MUTEX>(),
        TAG_OPLOCK,
    ) as *mut FAST_MUTEX;
    ExInitializeFastMutex((*oplock).IntLock);
    InitializeListHead(core::ptr::addr_of_mut!((*oplock).SharedListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!((*oplock).WaitListHead) as PLIST_ENTRY);
    (*oplock).Flags = NO_OPLOCK;

    oplock
}

unsafe extern "system" fn FsRtlCancelExclusiveIrp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let int_oplock = (*Irp).IoStatus.Information as PINTERNAL_OPLOCK;

    IoSetCancelRoutine(Irp, None);
    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    ExAcquireFastMutex((*int_oplock).IntLock);
    if !(*int_oplock).ExclusiveIrp.is_null() && (*(*int_oplock).ExclusiveIrp).Cancel != 0 {
        (*(*int_oplock).ExclusiveIrp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        IoCompleteRequest((*int_oplock).ExclusiveIrp, IO_DISK_INCREMENT);
        (*int_oplock).ExclusiveIrp = core::ptr::null_mut();

        ObDereferenceObject((*int_oplock).FileObject as PVOID);
        (*int_oplock).FileObject = core::ptr::null_mut();
        (*int_oplock).Flags = NO_OPLOCK;

        let mut next_entry = (*int_oplock).WaitListHead.Flink;
        while next_entry != core::ptr::addr_of_mut!((*int_oplock).WaitListHead) as PLIST_ENTRY {
            let wait_ctx = ((next_entry as usize) - core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry)) as PWAIT_CONTEXT;
            FsRtlRemoveAndCompleteWaitIrp(wait_ctx);
            next_entry = (*next_entry).Flink;
        }
    }

    ExReleaseFastMutexUnsafe((*int_oplock).IntLock);
}

unsafe fn FsRtlRequestExclusiveOplock(
    oplock: POPLOCK,
    stack: PIO_STACK_LOCATION,
    irp: PIRP,
    flags: ULONG,
) -> NTSTATUS {
    let mut int_oplock = *oplock as PINTERNAL_OPLOCK;
    let mut locked = false;
    let mut status = STATUS_SUCCESS;

    if int_oplock.is_null() {
        *oplock = FsRtlAllocateOplock() as PVOID;
        int_oplock = *oplock as PINTERNAL_OPLOCK;
    }

    ExAcquireFastMutexUnsafe((*int_oplock).IntLock);
    locked = true;

    if flags == (EXCLUSIVE_LOCK | PENDING_LOCK | FILTER_OPLOCK) {
        (*int_oplock).ExclusiveIrp = irp;
        (*int_oplock).FileObject = (*stack).FileObject;
        (*int_oplock).Flags = EXCLUSIVE_LOCK | PENDING_LOCK | FILTER_OPLOCK;
    } else {
        if ((*int_oplock).Flags & (LEVEL_2_OPLOCK | PENDING_LOCK | NO_OPLOCK)) != 0 {
            if (*int_oplock).Flags == LEVEL_2_OPLOCK {
                let list_irp = (((*int_oplock).SharedListHead.Flink as usize)
                    - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry))
                    as PIRP;
                FsRtlRemoveAndCompleteIrp(list_irp);
            }

            (*int_oplock).ExclusiveIrp = irp;
            (*int_oplock).FileObject = (*stack).FileObject;
            (*int_oplock).Flags = flags;

            IoMarkIrpPending(irp);
            ObReferenceObject((*stack).FileObject as PVOID);
            (*irp).IoStatus.Information = int_oplock as _;

            IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
            if (*irp).Cancel != 0 {
                ExReleaseFastMutexUnsafe((*int_oplock).IntLock);
                locked = false;
                FsRtlCancelExclusiveIrp(core::ptr::null_mut(), irp);
                status = STATUS_CANCELLED;
            } else {
                IoSetCancelRoutine(irp, Some(abi_cast!(FsRtlCancelExclusiveIrp)));
                IoReleaseCancelSpinLock((*irp).CancelIrql);
            }
        } else {
            if !irp.is_null() {
                (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_OPLOCK_NOT_GRANTED;
                IoCompleteRequest(irp, IO_DISK_INCREMENT);
                status = STATUS_OPLOCK_NOT_GRANTED;
            }
        }
    }

    if locked {
        ExReleaseFastMutexUnsafe((*int_oplock).IntLock);
    }

    status
}

unsafe fn FsRtlRequestOplockII(oplock: POPLOCK, stack: PIO_STACK_LOCATION, irp: PIRP) -> NTSTATUS {
    let mut int_oplock = *oplock as PINTERNAL_OPLOCK;
    let mut locked = false;
    let mut status = STATUS_SUCCESS;

    if int_oplock.is_null() {
        *oplock = FsRtlAllocateOplock() as PVOID;
        int_oplock = *oplock as PINTERNAL_OPLOCK;
    }

    ExAcquireFastMutexUnsafe((*int_oplock).IntLock);
    locked = true;

    if ((*int_oplock).Flags & (LEVEL_2_OPLOCK | NO_OPLOCK)) != 0 {
        IoMarkIrpPending(irp);
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

        InsertTailList(
            core::ptr::addr_of_mut!((*int_oplock).SharedListHead) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY,
        );

        (*irp).IoStatus.Information = int_oplock as _;

        (*int_oplock).Flags = LEVEL_2_OPLOCK;

        ObReferenceObject((*stack).FileObject as PVOID);

        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
        if (*irp).Cancel != 0 {
            ExReleaseFastMutexUnsafe((*int_oplock).IntLock);
            locked = false;
            FsRtlCancelOplockIIIrp(core::ptr::null_mut(), irp);
            status = STATUS_CANCELLED;
        } else {
            IoSetCancelRoutine(irp, Some(abi_cast!(FsRtlCancelOplockIIIrp)));
            IoReleaseCancelSpinLock((*irp).CancelIrql);
        }
    } else {
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_OPLOCK_NOT_GRANTED;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        status = STATUS_OPLOCK_NOT_GRANTED;
    }

    if locked {
        ExReleaseFastMutexUnsafe((*int_oplock).IntLock);
    }

    status
}

unsafe fn FsRtlOplockCleanup(oplock: PINTERNAL_OPLOCK, stack: PIO_STACK_LOCATION) {
    ExAcquireFastMutexUnsafe((*oplock).IntLock);
    if (*oplock).Flags != NO_OPLOCK {
        if (*oplock).Flags == LEVEL_2_OPLOCK {
            let mut next_entry = (*oplock).SharedListHead.Flink;
            while next_entry != core::ptr::addr_of_mut!((*oplock).SharedListHead) as PLIST_ENTRY {
                let list_irp = ((next_entry as usize)
                    - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry))
                    as PIRP;
                let list_stack = IoGetCurrentIrpStackLocation(list_irp);

                if (*stack).FileObject == (*list_stack).FileObject {
                    FsRtlRemoveAndCompleteIrp(list_irp);
                }

                next_entry = (*next_entry).Flink;
            }

            if IsListEmpty(core::ptr::addr_of_mut!((*oplock).SharedListHead) as PLIST_ENTRY) {
                (*oplock).Flags = NO_OPLOCK;
            }
        } else {
            if (*oplock).FileObject == (*stack).FileObject {
                if ((*oplock).Flags & (BROKEN_ANY | PENDING_LOCK)) == 0 {
                    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*(*oplock).ExclusiveIrp).CancelIrql));
                    IoSetCancelRoutine((*oplock).ExclusiveIrp, None);
                    IoReleaseCancelSpinLock((*(*oplock).ExclusiveIrp).CancelIrql);

                    (*(*oplock).ExclusiveIrp).IoStatus.Information = 3;
                    (*(*oplock).ExclusiveIrp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

                    IoCompleteRequest((*oplock).ExclusiveIrp, IO_DISK_INCREMENT);
                    (*oplock).ExclusiveIrp = core::ptr::null_mut();
                }

                if ((*oplock).Flags & PENDING_LOCK) == 0 {
                    ObDereferenceObject((*oplock).FileObject as PVOID);
                }

                (*oplock).FileObject = core::ptr::null_mut();
                (*oplock).Flags = NO_OPLOCK;

                let mut next_entry = (*oplock).WaitListHead.Flink;
                while next_entry != core::ptr::addr_of_mut!((*oplock).WaitListHead) as PLIST_ENTRY {
                    let wait_ctx = ((next_entry as usize) - core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry)) as PWAIT_CONTEXT;
                    FsRtlRemoveAndCompleteWaitIrp(wait_ctx);
                    next_entry = (*next_entry).Flink;
                }
            }
        }
    }
    ExReleaseFastMutexUnsafe((*oplock).IntLock);
}

unsafe fn FsRtlOplockBreakToNone(
    oplock: PINTERNAL_OPLOCK,
    stack: PIO_STACK_LOCATION,
    irp: PIRP,
    context: PVOID,
    completion_routine: POPLOCK_WAIT_COMPLETE_ROUTINE,
    post_irp_routine: POPLOCK_FS_PREPOST_IRP,
) -> NTSTATUS {
    let mut wait_event: KEVENT = core::mem::zeroed();

    ExAcquireFastMutexUnsafe((*oplock).IntLock);

    if (*oplock).Flags == NO_OPLOCK {
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_SUCCESS;
    }

    if ((*oplock).Flags & (BROKEN_ANY | PENDING_LOCK | LEVEL_2_OPLOCK)) == 0 {
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*(*oplock).ExclusiveIrp).CancelIrql));
        IoSetCancelRoutine((*oplock).ExclusiveIrp, None);
        IoReleaseCancelSpinLock((*(*oplock).ExclusiveIrp).CancelIrql);

        if (*(*oplock).ExclusiveIrp).Cancel != 0 {
            (*(*oplock).ExclusiveIrp).IoStatus.Information = 3;
            (*(*oplock).ExclusiveIrp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
            IoCompleteRequest((*oplock).ExclusiveIrp, IO_DISK_INCREMENT);

            (*oplock).Flags = NO_OPLOCK;
            (*oplock).ExclusiveIrp = core::ptr::null_mut();

            ObDereferenceObject((*oplock).FileObject as PVOID);
            (*oplock).FileObject = core::ptr::null_mut();

            let mut next_entry = (*oplock).WaitListHead.Flink;
            while next_entry != core::ptr::addr_of_mut!((*oplock).WaitListHead) as PLIST_ENTRY {
                let wait_ctx = ((next_entry as usize) - core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry)) as PWAIT_CONTEXT;
                FsRtlRemoveAndCompleteWaitIrp(wait_ctx);
                next_entry = (*next_entry).Flink;
            }

            ExReleaseFastMutexUnsafe((*oplock).IntLock);

            return STATUS_SUCCESS;
        }

        (*(*oplock).ExclusiveIrp).IoStatus.Information = 3;
        (*(*oplock).ExclusiveIrp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest((*oplock).ExclusiveIrp, IO_DISK_INCREMENT);

        (*oplock).ExclusiveIrp = core::ptr::null_mut();
        (*oplock).Flags |= BROKEN_TO_NONE;
    } else if (*oplock).Flags == LEVEL_2_OPLOCK {
        let mut next_entry = (*oplock).SharedListHead.Flink;
        while next_entry != core::ptr::addr_of_mut!((*oplock).SharedListHead) as PLIST_ENTRY {
            let list_irp = ((next_entry as usize)
                - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry))
                as PIRP;
            FsRtlRemoveAndCompleteIrp(list_irp);
            next_entry = (*next_entry).Flink;
        }

        (*oplock).Flags = NO_OPLOCK;

        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_SUCCESS;
    } else if ((*oplock).Flags & BROKEN_TO_LEVEL_2) != 0 {
        (*oplock).Flags &= !BROKEN_TO_LEVEL_2;
        (*oplock).Flags |= BROKEN_TO_NONE_FROM_LEVEL_2;
    } else if ((*oplock).Flags & PENDING_LOCK) != 0 {
        (*oplock).Flags = NO_OPLOCK;
        (*oplock).FileObject = core::ptr::null_mut();

        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_SUCCESS;
    }

    if (*oplock).FileObject == (*stack).FileObject {
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_SUCCESS;
    }

    if (*stack).MajorFunction != 0x00 || ((*stack).Parameters.Create.Options & 0x00000400) == 0 {
        FsRtlWaitOnIrp(oplock, irp, context, completion_routine, post_irp_routine, &mut wait_event);

        ExReleaseFastMutexUnsafe((*oplock).IntLock);

        return STATUS_SUCCESS;
    } else {
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_OPLOCK_BREAK_IN_PROGRESS;
    }
}

unsafe fn FsRtlOplockBreakToII(
    oplock: PINTERNAL_OPLOCK,
    stack: PIO_STACK_LOCATION,
    irp: PIRP,
    context: PVOID,
    completion_routine: POPLOCK_WAIT_COMPLETE_ROUTINE,
    post_irp_routine: POPLOCK_FS_PREPOST_IRP,
) -> NTSTATUS {
    let mut wait_event: KEVENT = core::mem::zeroed();

    ExAcquireFastMutexUnsafe((*oplock).IntLock);

    if ((*oplock).Flags & EXCLUSIVE_LOCK) == 0
        || (*oplock).FileObject == (*stack).FileObject
    {
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_SUCCESS;
    }

    if ((*oplock).Flags & (BROKEN_ANY | PENDING_LOCK)) != 0 {
        if ((*oplock).Flags & PENDING_LOCK) != 0 {
            (*oplock).Flags = NO_OPLOCK;
            (*oplock).FileObject = core::ptr::null_mut();

            ExReleaseFastMutexUnsafe((*oplock).IntLock);
            return STATUS_SUCCESS;
        }
    } else {
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*(*oplock).ExclusiveIrp).CancelIrql));
        IoSetCancelRoutine((*oplock).ExclusiveIrp, None);
        IoReleaseCancelSpinLock((*(*oplock).ExclusiveIrp).CancelIrql);

        if (*(*oplock).ExclusiveIrp).Cancel != 0 {
            (*(*oplock).ExclusiveIrp).IoStatus.Information = 3;
            (*(*oplock).ExclusiveIrp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
            IoCompleteRequest((*oplock).ExclusiveIrp, IO_DISK_INCREMENT);

            (*oplock).Flags = NO_OPLOCK;
            (*oplock).ExclusiveIrp = core::ptr::null_mut();
            ObDereferenceObject((*oplock).FileObject as PVOID);
            (*oplock).FileObject = core::ptr::null_mut();

            let mut next_entry = (*oplock).WaitListHead.Flink;
            while next_entry != core::ptr::addr_of_mut!((*oplock).WaitListHead) as PLIST_ENTRY {
                let wait_ctx = ((next_entry as usize) - core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry)) as PWAIT_CONTEXT;
                FsRtlRemoveAndCompleteWaitIrp(wait_ctx);
                next_entry = (*next_entry).Flink;
            }

            ExReleaseFastMutexUnsafe((*oplock).IntLock);

            return STATUS_SUCCESS;
        }

        (*(*oplock).ExclusiveIrp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        if ((*oplock).Flags & (BATCH_OPLOCK | LEVEL_1_OPLOCK)) != 0 {
            (*oplock).Flags |= BROKEN_TO_LEVEL_2;
            (*(*oplock).ExclusiveIrp).IoStatus.Information = 2;
        } else {
            (*oplock).Flags |= BROKEN_TO_NONE;
            (*(*oplock).ExclusiveIrp).IoStatus.Information = 3;
        }
        IoCompleteRequest((*oplock).ExclusiveIrp, IO_DISK_INCREMENT);
        (*oplock).ExclusiveIrp = core::ptr::null_mut();
    }

    if (*stack).MajorFunction != 0x00 || ((*stack).Parameters.Create.Options & 0x00000400) == 0 {
        FsRtlWaitOnIrp(oplock, irp, context, completion_routine, post_irp_routine, &mut wait_event);

        ExReleaseFastMutexUnsafe((*oplock).IntLock);

        return STATUS_SUCCESS;
    } else {
        ExReleaseFastMutexUnsafe((*oplock).IntLock);
        return STATUS_OPLOCK_BREAK_IN_PROGRESS;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlCheckOplock(
    Oplock: POPLOCK,
    Irp: PIRP,
    Context: PVOID,
    CompletionRoutine: POPLOCK_WAIT_COMPLETE_ROUTINE,
    PostIrpRoutine: POPLOCK_FS_PREPOST_IRP,
) -> NTSTATUS {
    let Irp = opaque_ptr(Irp);
    let Context = opaque_ptr(Context);
    let int_oplock = *Oplock as PINTERNAL_OPLOCK;

    if int_oplock.is_null() {
        return STATUS_SUCCESS;
    }

    if ((*Irp).Flags & 0x00000001) != 0 {
        return STATUS_SUCCESS;
    }

    if (*int_oplock).Flags == NO_OPLOCK {
        return STATUS_SUCCESS;
    }

    let stack = IoGetCurrentIrpStackLocation(Irp);

    if (*stack).MajorFunction == 0x12 {
        FsRtlOplockCleanup(int_oplock, stack);
        return STATUS_SUCCESS;
    } else if (*stack).MajorFunction == 0x11 {
        if ((*int_oplock).Flags & FILTER_OPLOCK) != 0 {
            return STATUS_SUCCESS;
        }

        if (*int_oplock).Flags == LEVEL_2_OPLOCK
            || (*int_oplock).FileObject != (*stack).FileObject
        {
            return FsRtlOplockBreakToNone(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
        }

        return STATUS_SUCCESS;
    } else if (*stack).MajorFunction == 0x0D {
        if (*stack).Parameters.FileSystemControl.FsControlCode != 0x00098098 {
            return STATUS_SUCCESS;
        }

        if (*int_oplock).Flags == LEVEL_2_OPLOCK
            || (*int_oplock).FileObject != (*stack).FileObject
        {
            return FsRtlOplockBreakToNone(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
        }

        return STATUS_SUCCESS;
    } else if (*stack).MajorFunction == 0x04 {
        if (*int_oplock).Flags == LEVEL_2_OPLOCK
            || (*int_oplock).FileObject != (*stack).FileObject
        {
            return FsRtlOplockBreakToNone(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
        }

        return STATUS_SUCCESS;
    } else if (*stack).MajorFunction == 0x03 {
        if ((*int_oplock).Flags & FILTER_OPLOCK) != 0 {
            return STATUS_SUCCESS;
        }

        if (*int_oplock).Flags != LEVEL_2_OPLOCK
            || (*int_oplock).FileObject != (*stack).FileObject
        {
            return FsRtlOplockBreakToII(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
        }

        return STATUS_SUCCESS;
    } else if (*stack).MajorFunction == 0x00 {
        let desired_access = (*(*stack).Parameters.Create.SecurityContext).DesiredAccess;

        if ((desired_access & !(0x00100000 | 0x00000100 | 0x00000080 | 0x00000001)) == 0
            && ((*stack).Parameters.Create.Options & 0x10000000) == 0)
            || (((*int_oplock).Flags & FILTER_OPLOCK) != 0
                && (desired_access & !(0x00100000 | 0x00020000 | 0x00000100 | 0x00000080 | 0x00000020 | 0x00000008 | 0x00000002)) == 0
                && ((*stack).Parameters.Create.ShareAccess & 0x00000001) != 0)
        {
            return STATUS_SUCCESS;
        }

        let create_disposition = ((*stack).Parameters.Create.Options >> 24) & 0xFF;
        if create_disposition == 0
            || create_disposition == 4
            || create_disposition == 5
            || ((*stack).Parameters.Create.Options & 0x10000000) != 0
        {
            if (*int_oplock).Flags == LEVEL_2_OPLOCK
                || (*int_oplock).FileObject != (*stack).FileObject
            {
                return FsRtlOplockBreakToNone(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
            }

            return STATUS_SUCCESS;
        }

        if (*int_oplock).Flags != LEVEL_2_OPLOCK
            || (*int_oplock).FileObject != (*stack).FileObject
        {
            return FsRtlOplockBreakToII(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
        }

        return STATUS_SUCCESS;
    } else if (*stack).MajorFunction == 0x09 {
        if (*int_oplock).Flags != LEVEL_2_OPLOCK
            || (*int_oplock).FileObject != (*stack).FileObject
        {
            return FsRtlOplockBreakToII(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
        }

        return STATUS_SUCCESS;
    } else if (*stack).MajorFunction == 0x06 {
        let file_info = (*stack).Parameters.SetFile.FileInformationClass;
        if file_info == 10 || file_info == 11 || file_info == 64 {
            if ((*int_oplock).Flags & (FILTER_OPLOCK | BATCH_OPLOCK)) == 0 {
                return STATUS_SUCCESS;
            } else {
                if (*int_oplock).Flags == LEVEL_2_OPLOCK
                    || (*int_oplock).FileObject != (*stack).FileObject
                {
                    return FsRtlOplockBreakToNone(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
                }

                return STATUS_SUCCESS;
            }
        } else if file_info == 19 {
            if (*int_oplock).Flags == LEVEL_2_OPLOCK
                || (*int_oplock).FileObject != (*stack).FileObject
            {
                return FsRtlOplockBreakToNone(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
            }

            return STATUS_SUCCESS;
        } else if file_info == 20 {
            if (*stack).Parameters.SetFile.__bindgen_anon_1.__bindgen_anon_1.AdvanceOnly != 0 {
                return STATUS_SUCCESS;
            }

            if (*int_oplock).Flags == LEVEL_2_OPLOCK
                || (*int_oplock).FileObject != (*stack).FileObject
            {
                return FsRtlOplockBreakToNone(int_oplock, stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
            }

            return STATUS_SUCCESS;
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlCurrentBatchOplock(Oplock: POPLOCK) -> BOOLEAN {
    let int_oplock = *Oplock as PINTERNAL_OPLOCK;

    if !int_oplock.is_null() && ((*int_oplock).Flags & (FILTER_OPLOCK | BATCH_OPLOCK)) != 0 {
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeOplock(Oplock: POPLOCK) {}

#[no_mangle]
pub unsafe extern "system" fn FsRtlOplockFsctrl(
    Oplock: POPLOCK,
    Irp: PIRP,
    OpenCount: ULONG,
) -> NTSTATUS {
    let Irp = opaque_ptr(Irp);
    let int_oplock = *Oplock as PINTERNAL_OPLOCK;
    let stack = IoGetCurrentIrpStackLocation(Irp);

    if (*stack).MajorFunction != 0x00 {
        match (*stack).Parameters.FileSystemControl.FsControlCode {
            0x00090098 => {
                return FsRtlOplockBreakNotify(int_oplock, stack, Irp);
            }
            0x00090084 => {
                return FsRtlAcknowledgeOplockBreak(int_oplock, stack, Irp, 0u8);
            }
            0x00090090 => {
                return FsRtlOpBatchBreakClosePending(int_oplock, stack, Irp);
            }
            0x00090000 => {
                if OpenCount == 1
                    && IoIsOperationSynchronous(Irp) == 0
                    && ((*Irp).Flags & 0x00000040) == 0
                    && ((*(*stack).FileObject).Flags & 0x00004000) == 0
                {
                    return FsRtlRequestExclusiveOplock(Oplock, stack, Irp, EXCLUSIVE_LOCK | LEVEL_1_OPLOCK);
                } else {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_OPLOCK_NOT_GRANTED;
                    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
                    return STATUS_OPLOCK_NOT_GRANTED;
                }
            }
            0x00090008 => {
                if OpenCount == 0
                    && IoIsOperationSynchronous(Irp) == 0
                    && ((*Irp).Flags & 0x00000040) == 0
                    && ((*(*stack).FileObject).Flags & 0x00004000) == 0
                {
                    return FsRtlRequestOplockII(Oplock, stack, Irp);
                } else {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_OPLOCK_NOT_GRANTED;
                    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
                    return STATUS_OPLOCK_NOT_GRANTED;
                }
            }
            0x00090094 => {
                return FsRtlAcknowledgeOplockBreak(int_oplock, stack, Irp, 1u8);
            }
            0x00090004 => {
                if OpenCount == 1
                    && IoIsOperationSynchronous(Irp) == 0
                    && ((*Irp).Flags & 0x00000040) == 0
                    && ((*(*stack).FileObject).Flags & 0x00004000) == 0
                {
                    return FsRtlRequestExclusiveOplock(Oplock, stack, Irp, EXCLUSIVE_LOCK | BATCH_OPLOCK);
                } else {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_OPLOCK_NOT_GRANTED;
                    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
                    return STATUS_OPLOCK_NOT_GRANTED;
                }
            }
            0x0009008C => {
                if OpenCount == 1
                    && IoIsOperationSynchronous(Irp) == 0
                    && ((*Irp).Flags & 0x00000040) == 0
                    && ((*(*stack).FileObject).Flags & 0x00004000) == 0
                {
                    return FsRtlRequestExclusiveOplock(Oplock, stack, Irp, EXCLUSIVE_LOCK | FILTER_OPLOCK);
                } else {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_OPLOCK_NOT_GRANTED;
                    IoCompleteRequest(Irp, IO_DISK_INCREMENT);
                    return STATUS_OPLOCK_NOT_GRANTED;
                }
            }
            _ => {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                IoCompleteRequest(Irp, IO_DISK_INCREMENT);
                return STATUS_INVALID_PARAMETER;
            }
        }
    }

    if OpenCount == 1
        && ((*(*stack).Parameters.Create.SecurityContext).DesiredAccess & !(0x00000080 | 0x00000001)) == 0
        && ((*stack).Parameters.Create.ShareAccess & 0x00000007) == 0x00000007
    {
        return FsRtlRequestExclusiveOplock(Oplock, stack, core::ptr::null_mut(), EXCLUSIVE_LOCK | PENDING_LOCK | FILTER_OPLOCK);
    }

    STATUS_OPLOCK_NOT_GRANTED
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlOplockIsFastIoPossible(Oplock: POPLOCK) -> BOOLEAN {
    let int_oplock = *Oplock as PINTERNAL_OPLOCK;

    if !int_oplock.is_null() && ((*int_oplock).Flags & (BROKEN_ANY | LEVEL_2_OPLOCK)) != 0 {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlUninitializeOplock(Oplock: POPLOCK) {
    let int_oplock = *Oplock as PINTERNAL_OPLOCK;

    if int_oplock.is_null() {
        return;
    }

    *Oplock = core::ptr::null_mut();

    ExAcquireFastMutexUnsafe((*int_oplock).IntLock);

    // Both walks below capture the next link BEFORE the entry is consumed. The C reads
    // `NextEntry->Flink` in the for-increment, after `ExFreePoolWithTag(WaitCtx)` and after
    // `IoCompleteRequest(Irp)` -- each of which can release the memory `NextEntry` points
    // into. That is a use-after-free in upstream oplock.c; this is the same bug
    // RtlDeleteOwnersRanges shipped, and it bugchecked kmtest RtlRangeList.
    let mut next_entry = (*int_oplock).WaitListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*int_oplock).WaitListHead) as PLIST_ENTRY {
        let following = (*next_entry).Flink;
        let wait_ctx = ((next_entry as usize) - core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry)) as PWAIT_CONTEXT;
        let irp = (*wait_ctx).Irp;

        RemoveEntryList(core::ptr::addr_of_mut!((*wait_ctx).WaitListEntry) as PLIST_ENTRY);
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
        IoSetCancelRoutine(irp, None);
        IoReleaseCancelSpinLock((*irp).CancelIrql);

        (*irp).IoStatus.Information = 0;
        ((*wait_ctx).CompletionRoutine.unwrap())((*wait_ctx).CompletionContext, (*wait_ctx).Irp);

        ExFreePoolWithTag(wait_ctx as PVOID, TAG_OPLOCK);

        next_entry = following;
    }

    next_entry = (*int_oplock).SharedListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*int_oplock).SharedListHead) as PLIST_ENTRY {
        let following = (*next_entry).Flink;
        let irp = ((next_entry as usize)
            - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry))
            as PIRP;

        RemoveEntryList(core::ptr::addr_of_mut!((*irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY);

        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
        IoSetCancelRoutine(irp, None);
        IoReleaseCancelSpinLock((*irp).CancelIrql);

        let stack = IoGetCurrentIrpStackLocation(irp);
        ObDereferenceObject((*stack).FileObject as PVOID);

        (*irp).IoStatus.Information = 3;
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);

        next_entry = following;
    }

    let irp = (*int_oplock).ExclusiveIrp;
    if !irp.is_null() {
        IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));
        IoSetCancelRoutine(irp, None);
        IoReleaseCancelSpinLock((*irp).CancelIrql);

        (*irp).IoStatus.Information = 3;
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        (*int_oplock).ExclusiveIrp = core::ptr::null_mut();

        if !(*int_oplock).FileObject.is_null() {
            ObDereferenceObject((*int_oplock).FileObject as PVOID);
        }
    }

    ExReleaseFastMutexUnsafe((*int_oplock).IntLock);

    ExFreePoolWithTag((*int_oplock).IntLock as PVOID, TAG_OPLOCK);
    ExFreePoolWithTag(int_oplock as PVOID, TAG_OPLOCK);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/oplock.c
//  * PURPOSE:         Provides an Opportunistic Lock for file system drivers.
//  * PROGRAMMERS:     Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define NO_OPLOCK                   0x1
// #define LEVEL_1_OPLOCK              0x2
// #define BATCH_OPLOCK                0x4
// #define FILTER_OPLOCK               0x8
// #define LEVEL_2_OPLOCK              0x10
//
// #define EXCLUSIVE_LOCK              0x40
// #define PENDING_LOCK                0x80
//
// #define BROKEN_TO_LEVEL_2           0x100
// #define BROKEN_TO_NONE              0x200
// #define BROKEN_TO_NONE_FROM_LEVEL_2 0x400
// #define BROKEN_TO_CLOSE_PENDING     0x800
// #define BROKEN_ANY                  (BROKEN_TO_LEVEL_2 | BROKEN_TO_NONE  | BROKEN_TO_NONE_FROM_LEVEL_2 | BROKEN_TO_CLOSE_PENDING)
//
// typedef struct _INTERNAL_OPLOCK
// {
//     /* Level I IRP */
//     PIRP ExclusiveIrp;
//     /* Level I FILE_OBJECT */
//     PFILE_OBJECT FileObject;
//     /* Level II IRPs */
//     LIST_ENTRY SharedListHead;
//     /* IRPs waiting on level I */
//     LIST_ENTRY WaitListHead;
//     ULONG Flags;
//     PFAST_MUTEX IntLock;
// } INTERNAL_OPLOCK, *PINTERNAL_OPLOCK;
//
// typedef struct _WAIT_CONTEXT
// {
//     LIST_ENTRY WaitListEntry;
//     PIRP Irp;
//     POPLOCK_WAIT_COMPLETE_ROUTINE CompletionRoutine;
//     PVOID CompletionContext;
//     ULONG Reserved;
//     ULONG_PTR SavedInformation;
// } WAIT_CONTEXT, *PWAIT_CONTEXT;
//
// VOID
// NTAPI
// FsRtlNotifyCompletion(IN PVOID Context,
//                       IN PIRP Irp)
// {
//     PAGED_CODE();
//
//     DPRINT("FsRtlNotifyCompletion(%p, %p)\n", Context, Irp);
//
//     /* Just complete the IRP */
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
// }
//
// VOID
// NTAPI
// FsRtlCompletionRoutinePriv(IN PVOID Context,
//                            IN PIRP Irp)
// {
//     PKEVENT WaitEvent;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlCompletionRoutinePriv(%p, %p)\n", Context, Irp);
//
//     /* Set the event */
//     WaitEvent = (PKEVENT)Context;
//     KeSetEvent(WaitEvent, IO_NO_INCREMENT, FALSE);
// }
//
// VOID
// FsRtlRemoveAndCompleteWaitIrp(IN PWAIT_CONTEXT WaitCtx)
// {
//     PIRP Irp;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlRemoveAndCompleteWaitIrp(%p)\n", WaitCtx);
//
//     RemoveEntryList(&WaitCtx->WaitListEntry);
//     Irp = WaitCtx->Irp;
//
//     /* No cancel routine anymore */
//     IoAcquireCancelSpinLock(&Irp->CancelIrql);
//     IoSetCancelRoutine(Irp, NULL);
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     /* Set the information */
//     Irp->IoStatus.Information = WaitCtx->SavedInformation;
//     /* Set the status according to the fact it got cancel or not */
//     Irp->IoStatus.Status = (Irp->Cancel ? STATUS_CANCELLED : STATUS_SUCCESS);
//
//     /* Call the completion routine */
//     WaitCtx->CompletionRoutine(WaitCtx->CompletionContext, Irp);
//
//     /* And get rid of the now useless wait context */
//     ExFreePoolWithTag(WaitCtx, TAG_OPLOCK);
// }
//
// VOID
// NTAPI
// FsRtlCancelWaitIrp(IN PDEVICE_OBJECT DeviceObject,
//                    IN PIRP Irp)
// {
//     PINTERNAL_OPLOCK Oplock;
//     PLIST_ENTRY NextEntry;
//     PWAIT_CONTEXT WaitCtx;
//
//     DPRINT("FsRtlCancelWaitIrp(%p, %p)\n", DeviceObject, Irp);
//
//     /* Get the associated oplock */
//     Oplock = (PINTERNAL_OPLOCK)Irp->IoStatus.Information;
//
//     /* Remove the cancel routine (we're being called!) */
//     IoSetCancelRoutine(Irp, NULL);
//     /* And release the cancel spin lock (always locked when cancel routine is called) */
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     /* Now, remove and complete any associated waiter */
//     ExAcquireFastMutex(Oplock->IntLock);
//     for (NextEntry = Oplock->WaitListHead.Flink;
//          NextEntry != &Oplock->WaitListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         WaitCtx = CONTAINING_RECORD(NextEntry, WAIT_CONTEXT, WaitListEntry);
//
//         if (WaitCtx->Irp->Cancel)
//         {
//             FsRtlRemoveAndCompleteWaitIrp(WaitCtx);
//         }
//     }
//     ExReleaseFastMutex(Oplock->IntLock);
// }
//
// VOID
// FsRtlWaitOnIrp(IN PINTERNAL_OPLOCK Oplock,
//                IN PIRP Irp,
//                IN PVOID CompletionContext,
//                IN POPLOCK_WAIT_COMPLETE_ROUTINE CompletionRoutine,
//                IN POPLOCK_FS_PREPOST_IRP PostIrpRoutine,
//                IN PKEVENT WaitEvent)
// {
//     BOOLEAN Locked;
//     PWAIT_CONTEXT WaitCtx;
//
//     DPRINT("FsRtlWaitOnIrp(%p, %p, %p, %p, %p, %p)\n", Oplock, Irp, CompletionContext, CompletionRoutine, PostIrpRoutine, WaitEvent);
//
//     /* We must always be called with IntLock locked! */
//     Locked = TRUE;
//     /* Dirty check for above statement */
//     ASSERT(Oplock->IntLock->Owner == KeGetCurrentThread());
//
//     /* Allocate a wait context for the IRP */
//     WaitCtx = ExAllocatePoolWithTag(PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE, sizeof(WAIT_CONTEXT), TAG_OPLOCK);
//     WaitCtx->Irp = Irp;
//     WaitCtx->SavedInformation = Irp->IoStatus.Information;
//     /* If caller provided everything required, us it */
//     if (CompletionRoutine != NULL)
//     {
//         WaitCtx->CompletionRoutine = CompletionRoutine;
//         WaitCtx->CompletionContext = CompletionContext;
//     }
//     /* Otherwise, put ourselves */
//     else
//     {
//         WaitCtx->CompletionRoutine = FsRtlCompletionRoutinePriv;
//         WaitCtx->CompletionContext = WaitEvent;
//         KeInitializeEvent(WaitEvent, NotificationEvent, FALSE);
//     }
//
//     /* If we got a prepost routine, call it now! */
//     if (PostIrpRoutine != NULL)
//     {
//         PostIrpRoutine(CompletionContext, Irp);
//     }
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//
//     /* Queue the IRP - it's OK, we're locked */
//     InsertHeadList(&Oplock->WaitListHead, &WaitCtx->WaitListEntry);
//
//     /* Set the oplock as information of the IRP (for the cancel routine)
//      * And lock the cancel routine lock for setting it
//      */
//     IoAcquireCancelSpinLock(&Irp->CancelIrql);
//     Irp->IoStatus.Information = (ULONG_PTR)Oplock;
//
//     /* If there's already a cancel routine
//      * Cancel the IRP
//      */
//     if (Irp->Cancel)
//     {
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         Locked = FALSE;
//
//         if (CompletionRoutine != NULL)
//         {
//             IoMarkIrpPending(Irp);
//         }
//         FsRtlCancelWaitIrp(NULL, Irp);
//     }
//     /* Otherwise, put ourselves as the cancel routine and start waiting */
//     else
//     {
//         IoSetCancelRoutine(Irp, FsRtlCancelWaitIrp);
//         IoReleaseCancelSpinLock(Irp->CancelIrql);
//         if (CompletionRoutine != NULL)
//         {
//             IoMarkIrpPending(Irp);
//         }
//         else
//         {
//             ExReleaseFastMutexUnsafe(Oplock->IntLock);
//             Locked = FALSE;
//             KeWaitForSingleObject(WaitEvent, Executive, KernelMode, FALSE, NULL);
//         }
//     }
//
//     /* If we didn't unlock yet, do it now */
//     if (Locked)
//     {
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//     }
// }
//
// NTSTATUS
// FsRtlOplockBreakNotify(IN PINTERNAL_OPLOCK Oplock,
//                        IN PIO_STACK_LOCATION Stack,
//                        IN PIRP Irp)
// {
//     PAGED_CODE();
//
//     DPRINT("FsRtlOplockBreakNotify(%p, %p, %p)\n", Oplock, Stack, Irp);
//
//     /* No oplock, no break to notify */
//     if (Oplock == NULL)
//     {
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return STATUS_SUCCESS;
//     }
//
//     /* Notify by completing the IRP, unless we have broken to shared */
//     ExAcquireFastMutexUnsafe(Oplock->IntLock);
//     if (!BooleanFlagOn(Oplock->Flags, BROKEN_TO_LEVEL_2))
//     {
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_SUCCESS;
//     }
//
//     /* If it's pending, just complete the IRP and get rid of the oplock */
//     if (BooleanFlagOn(Oplock->Flags, PENDING_LOCK))
//     {
//         Oplock->FileObject = NULL;
//         Oplock->Flags = NO_OPLOCK;
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_SUCCESS;
//     }
//
//     /* Otherwise, wait on the IRP */
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     FsRtlWaitOnIrp(Oplock, Irp, NULL, FsRtlNotifyCompletion, NULL, NULL);
//     return STATUS_SUCCESS;
// }
//
// VOID
// FsRtlRemoveAndCompleteIrp(IN PIRP Irp)
// {
//     PIO_STACK_LOCATION Stack;
//
//     DPRINT("FsRtlRemoveAndCompleteIrp(%p)\n", Irp);
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Remove our extra ref */
//     ObDereferenceObject(Stack->FileObject);
//
//     /* Remove our cancel routine */
//     IoAcquireCancelSpinLock(&Irp->CancelIrql);
//     IoSetCancelRoutine(Irp, NULL);
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     /* Remove the IRP from the list it may be in (wait or shared) */
//     RemoveEntryList(&Irp->Tail.Overlay.ListEntry);
//
//     /* And complete! */
//     Irp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//     Irp->IoStatus.Status = (Irp->Cancel ? STATUS_CANCELLED : STATUS_SUCCESS);
//
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
// }
//
// VOID
// NTAPI
// FsRtlCancelOplockIIIrp(IN PDEVICE_OBJECT DeviceObject,
//                        IN PIRP Irp)
// {
//     PINTERNAL_OPLOCK Oplock;
//     PLIST_ENTRY NextEntry;
//     PIRP ListIrp;
//     BOOLEAN Removed;
//
//     DPRINT("FsRtlCancelOplockIIIrp(%p, %p)\n", DeviceObject, Irp);
//
//     /* Get the associated oplock */
//     Oplock = (PINTERNAL_OPLOCK)Irp->IoStatus.Information;
//
//     /* Remove the cancel routine (it's OK, we're the cancel routine! )*/
//     IoSetCancelRoutine(Irp, NULL);
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     /* Nothing removed yet */
//     Removed = FALSE;
//     ExAcquireFastMutex(Oplock->IntLock);
//     /* Browse all the IRPs associated to the shared lock */
//     for (NextEntry = Oplock->SharedListHead.Flink;
//          NextEntry != &Oplock->SharedListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         ListIrp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.ListEntry);
//
//         /* If canceled, remove it */
//         if (ListIrp->Cancel)
//         {
//             FsRtlRemoveAndCompleteIrp(ListIrp);
//             Removed = TRUE;
//         }
//     }
//
//     /* If no IRP left, the oplock is gone */
//     if (Removed && IsListEmpty(&Oplock->SharedListHead))
//     {
//         Oplock->Flags = NO_OPLOCK;
//     }
//     /* Don't forget to release the mutex */
//     ExReleaseFastMutex(Oplock->IntLock);
// }
//
// NTSTATUS
// FsRtlAcknowledgeOplockBreak(IN PINTERNAL_OPLOCK Oplock,
//                             IN PIO_STACK_LOCATION Stack,
//                             IN PIRP Irp,
//                             IN BOOLEAN SwitchToLevel2)
// {
//     PLIST_ENTRY NextEntry;
//     PWAIT_CONTEXT WaitCtx;
//     BOOLEAN Deref;
//     BOOLEAN Locked;
//
//     DPRINT("FsRtlAcknowledgeOplockBreak(%p, %p, %p, %u)\n", Oplock, Stack, Irp, Unknown);
//
//     /* No oplock, nothing to acknowledge */
//     if (Oplock == NULL)
//     {
//         Irp->IoStatus.Status = STATUS_INVALID_OPLOCK_PROTOCOL;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return STATUS_INVALID_OPLOCK_PROTOCOL;
//     }
//
//     /* Acquire oplock internal lock */
//     ExAcquireFastMutexUnsafe(Oplock->IntLock);
//     Locked = TRUE;
//     /* Does it match the file? */
//     if (Oplock->FileObject != Stack->FileObject)
//     {
//         Irp->IoStatus.Status = STATUS_INVALID_OPLOCK_PROTOCOL;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_INVALID_OPLOCK_PROTOCOL;
//     }
//
//     /* Assume we'll have to deref our extra ref (level I) */
//     Deref = TRUE;
//
//     /* If we got broken to level 2 and asked for a shared lock
//      * switch the oplock to shared
//      */
//     if (SwitchToLevel2 && BooleanFlagOn(Oplock->Flags, BROKEN_TO_LEVEL_2))
//     {
//         /* The IRP cannot be synchronous, we'll move it to the LEVEL_2 IRPs */
//         ASSERT(!IoIsOperationSynchronous(Irp));
//
//         /* Mark the IRP pending, and queue it for the shared IRPs */
//         IoMarkIrpPending(Irp);
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         InsertTailList(&Oplock->SharedListHead, &Irp->Tail.Overlay.ListEntry);
//
//         /* Don't deref, we're not done yet */
//         Deref = FALSE;
//         /* And mark we've got a shared lock */
//         Oplock->Flags = LEVEL_2_OPLOCK;
//         /* To find the lock back on cancel */
//         Irp->IoStatus.Information = (ULONG_PTR)Oplock;
//
//         /* Acquire the spinlock to set the cancel routine */
//         IoAcquireCancelSpinLock(&Irp->CancelIrql);
//         /* If IRP got canceled, call it immediately */
//         if (Irp->Cancel)
//         {
//             ExReleaseFastMutexUnsafe(Oplock->IntLock);
//             Locked = FALSE;
//             FsRtlCancelOplockIIIrp(NULL, Irp);
//         }
//         /* Otherwise, just set our cancel routine */
//         else
//         {
//             IoSetCancelRoutine(Irp, FsRtlCancelOplockIIIrp);
//             IoReleaseCancelSpinLock(Irp->CancelIrql);
//         }
//     }
//     /* If oplock got broken, remove it */
//     else if (BooleanFlagOn(Oplock->Flags, (BROKEN_TO_NONE | BROKEN_TO_LEVEL_2)))
//     {
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IofCompleteRequest(Irp, IO_DISK_INCREMENT);
//         Oplock->Flags = NO_OPLOCK;
//     }
//     /* Same, but precise we got broken from none to shared */
//     else if (BooleanFlagOn(Oplock->Flags, BROKEN_TO_NONE_FROM_LEVEL_2))
//     {
//         Irp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         IofCompleteRequest(Irp, IO_DISK_INCREMENT);
//         Oplock->Flags = NO_OPLOCK;
//     }
//
//     /* Now, complete any IRP waiting */
//     for (NextEntry = Oplock->WaitListHead.Flink;
//          NextEntry != &Oplock->WaitListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         WaitCtx = CONTAINING_RECORD(NextEntry, WAIT_CONTEXT, WaitListEntry);
//         FsRtlRemoveAndCompleteWaitIrp(WaitCtx);
//     }
//
//     /* If we dropped oplock, remove our extra ref */
//     if (Deref)
//     {
//         ObDereferenceObject(Oplock->FileObject);
//     }
//     /* And unset FO: no oplock left or shared */
//     Oplock->FileObject = NULL;
//
//     /* Don't leak the mutex! */
//     if (Locked)
//     {
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// FsRtlOpBatchBreakClosePending(IN PINTERNAL_OPLOCK Oplock,
//                               IN PIO_STACK_LOCATION Stack,
//                               IN PIRP Irp)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     PWAIT_CONTEXT WaitCtx;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlOpBatchBreakClosePending(%p, %p, %p)\n", Oplock, Stack, Irp);
//
//     /* No oplock, that's not legit! */
//     if (Oplock == NULL)
//     {
//         Irp->IoStatus.Status = STATUS_INVALID_OPLOCK_PROTOCOL;
//         IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         return STATUS_INVALID_OPLOCK_PROTOCOL;
//     }
//
//     Status = STATUS_SUCCESS;
//     ExAcquireFastMutexUnsafe(Oplock->IntLock);
//
//     /* First of all, check if all conditions are met:
//      * Correct FO + broken oplock
//      */
//     if (Oplock->FileObject == Stack->FileObject && (BooleanFlagOn(Oplock->Flags, (BROKEN_TO_LEVEL_2 | BROKEN_TO_NONE | BROKEN_TO_NONE_FROM_LEVEL_2))))
//     {
//         /* If we have a pending or level 1 oplock... */
//         if (BooleanFlagOn(Oplock->Flags, (PENDING_LOCK | LEVEL_1_OPLOCK)))
//         {
//             /* Remove our extra ref from the FO */
//             if (Oplock->Flags & LEVEL_1_OPLOCK)
//             {
//                 ObDereferenceObject(Oplock->FileObject);
//             }
//
//             /* And remove the oplock */
//             Oplock->Flags = NO_OPLOCK;
//             Oplock->FileObject = NULL;
//
//             /* Complete any waiting IRP */
//             for (NextEntry = Oplock->WaitListHead.Flink;
//                  NextEntry != &Oplock->WaitListHead;
//                  NextEntry = NextEntry->Flink)
//             {
//                 WaitCtx = CONTAINING_RECORD(NextEntry, WAIT_CONTEXT, WaitListEntry);
//                 FsRtlRemoveAndCompleteWaitIrp(WaitCtx);
//             }
//         }
//         /* Otherwise, mark the oplock as close pending */
//         else
//         {
//             ClearFlag(Oplock->Flags, BROKEN_ANY);
//             SetFlag(Oplock->Flags, BROKEN_TO_CLOSE_PENDING);
//         }
//     }
//     /* Oplock is in invalid state */
//     else
//     {
//         Status = STATUS_INVALID_OPLOCK_PROTOCOL;
//     }
//
//     /* And complete */
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//     ExReleaseFastMutexUnsafe(Oplock->IntLock);
//
//     return Status;
// }
//
// PINTERNAL_OPLOCK
// FsRtlAllocateOplock(VOID)
// {
//     PINTERNAL_OPLOCK Oplock = NULL;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlAllocateOplock()\n");
//
//     _SEH2_TRY
//     {
//         /* Allocate and initialize the oplock */
//         Oplock = ExAllocatePoolWithTag(PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE | POOL_COLD_ALLOCATION, sizeof(INTERNAL_OPLOCK), TAG_OPLOCK);
//         RtlZeroMemory(Oplock, sizeof(INTERNAL_OPLOCK));
//         /* We allocate the fast mutex separately to have it non paged (while the rest of the oplock can be paged) */
//         Oplock->IntLock = ExAllocatePoolWithTag(NonPagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE, sizeof(FAST_MUTEX), TAG_OPLOCK);
//         ExInitializeFastMutex(Oplock->IntLock);
//         /* Initialize the IRP list for level 2 oplock */
//         InitializeListHead(&Oplock->SharedListHead);
//         /* And for the wait IRPs */
//         InitializeListHead(&Oplock->WaitListHead);
//         Oplock->Flags = NO_OPLOCK;
//     }
//     _SEH2_FINALLY
//     {
//         /* In case of abnormal termination, it means either OPLOCK or FAST_MUTEX allocation failed */
//         if (_SEH2_AbnormalTermination())
//         {
//             /* That FAST_MUTEX, free OPLOCK */
//             if (Oplock != NULL)
//             {
//                 ExFreePoolWithTag(Oplock, TAG_OPLOCK);
//                 Oplock = NULL;
//             }
//         }
//     }
//     _SEH2_END;
//
//     return Oplock;
// }
//
// VOID
// NTAPI
// FsRtlCancelExclusiveIrp(IN PDEVICE_OBJECT DeviceObject,
//                         IN PIRP Irp)
// {
//     PINTERNAL_OPLOCK IntOplock;
//     PLIST_ENTRY NextEntry;
//     PWAIT_CONTEXT WaitCtx;
//
//     DPRINT("FsRtlCancelExclusiveIrp(%p, %p)\n", DeviceObject, Irp);
//
//     /* Get the associated oplock */
//     IntOplock = (PINTERNAL_OPLOCK)Irp->IoStatus.Information;
//
//     /* Remove the cancel routine (us!) and release the cancel spinlock */
//     IoSetCancelRoutine(Irp, NULL);
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     /* Acquire our internal FAST_MUTEX */
//     ExAcquireFastMutex(IntOplock->IntLock);
//     /* If we had an exclusive IRP */
//     if (IntOplock->ExclusiveIrp != NULL && IntOplock->ExclusiveIrp->Cancel)
//     {
//         /* Cancel it, and remove it from the oplock */
//         IntOplock->ExclusiveIrp->IoStatus.Status = STATUS_CANCELLED;
//         IoCompleteRequest(IntOplock->ExclusiveIrp, IO_DISK_INCREMENT);
//         IntOplock->ExclusiveIrp = NULL;
//
//         /* Dereference the fileobject and remove the oplock */
//         ObDereferenceObject(IntOplock->FileObject);
//         IntOplock->FileObject = NULL;
//         IntOplock->Flags = NO_OPLOCK;
//
//         /* And complete any waiting IRP */
//         for (NextEntry = IntOplock->WaitListHead.Flink;
//              NextEntry != &IntOplock->WaitListHead;
//              NextEntry = NextEntry->Flink)
//         {
//             WaitCtx = CONTAINING_RECORD(NextEntry, WAIT_CONTEXT, WaitListEntry);
//             FsRtlRemoveAndCompleteWaitIrp(WaitCtx);
//         }
//     }
//
//     /* Done! */
//     ExReleaseFastMutexUnsafe(IntOplock->IntLock);
// }
//
// NTSTATUS
// FsRtlRequestExclusiveOplock(IN POPLOCK Oplock,
//                             IN PIO_STACK_LOCATION Stack,
//                             IN PIRP Irp,
//                             IN ULONG Flags)
// {
//     PINTERNAL_OPLOCK IntOplock;
//     PIRP ListIrp;
//     BOOLEAN Locked;
//     NTSTATUS Status;
//
//     DPRINT("FsRtlRequestExclusiveOplock(%p, %p, %p, %lu)\n", Oplock, Stack, Irp, Flags);
//
//     IntOplock = *Oplock;
//     Locked = FALSE;
//     Status = STATUS_SUCCESS;
//
//     /* Time to work! */
//     _SEH2_TRY
//     {
//         /* Was the oplock already allocated? If not, do it now! */
//         if (IntOplock == NULL)
//         {
//             *Oplock = FsRtlAllocateOplock();
//             IntOplock = *Oplock;
//         }
//
//         /* Acquire our internal lock */
//         ExAcquireFastMutexUnsafe(IntOplock->IntLock);
//         Locked = TRUE;
//
//         /* If we request exclusiveness, a filter or a pending oplock, grant it */
//         if (Flags == (EXCLUSIVE_LOCK | PENDING_LOCK | FILTER_OPLOCK))
//         {
//             /* Either no oplock, or pending */
//             ASSERT(BooleanFlagOn(IntOplock->Flags, (NO_OPLOCK | PENDING_LOCK)));
//             IntOplock->ExclusiveIrp = Irp;
//             IntOplock->FileObject = Stack->FileObject;
//             IntOplock->Flags = (EXCLUSIVE_LOCK | PENDING_LOCK | FILTER_OPLOCK);
//         }
//         else
//         {
//             /* Otherwise, shared or no effective oplock */
//             if (BooleanFlagOn(IntOplock->Flags, (LEVEL_2_OPLOCK | PENDING_LOCK | NO_OPLOCK)))
//             {
//                 /* The shared IRPs list should contain a single entry! */
//                 if (IntOplock->Flags == LEVEL_2_OPLOCK)
//                 {
//                     ListIrp = CONTAINING_RECORD(IntOplock->SharedListHead.Flink, IRP, Tail.Overlay.ListEntry);
//                     ASSERT(IntOplock->SharedListHead.Flink == IntOplock->SharedListHead.Blink);
//                     FsRtlRemoveAndCompleteIrp(ListIrp);
//                 }
//
//                 /* Set the exclusiveness */
//                 IntOplock->ExclusiveIrp = Irp;
//                 IntOplock->FileObject = Stack->FileObject;
//                 IntOplock->Flags = Flags;
//
//                 /* Mark the IRP pending and reference our file object */
//                 IoMarkIrpPending(Irp);
//                 ObReferenceObject(Stack->FileObject);
//                 Irp->IoStatus.Information = (ULONG_PTR)IntOplock;
//
//                 /* Now, set ourselves as cancel routine */
//                 IoAcquireCancelSpinLock(&Irp->CancelIrql);
//                 /* Unless IRP got canceled, then, just give up */
//                 if (Irp->Cancel)
//                 {
//                     ExReleaseFastMutexUnsafe(IntOplock->IntLock);
//                     Locked = FALSE;
//                     FsRtlCancelExclusiveIrp(NULL, Irp);
//                     Status = STATUS_CANCELLED;
//                 }
//                 else
//                 {
//                     IoSetCancelRoutine(Irp, FsRtlCancelExclusiveIrp);
//                     IoReleaseCancelSpinLock(Irp->CancelIrql);
//                 }
//             }
//             /* Cannot set exclusiveness, fail */
//             else
//             {
//                 if (Irp != NULL)
//                 {
//                     Irp->IoStatus.Status = STATUS_OPLOCK_NOT_GRANTED;
//                     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//                     Status = STATUS_OPLOCK_NOT_GRANTED;
//                 }
//             }
//         }
//     }
//     /* If locked, release */
//     _SEH2_FINALLY
//     {
//         if (Locked)
//         {
//             ExReleaseFastMutexUnsafe(IntOplock->IntLock);
//         }
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
// NTSTATUS
// FsRtlRequestOplockII(IN POPLOCK Oplock,
//                      IN PIO_STACK_LOCATION Stack,
//                      IN PIRP Irp)
// {
//     BOOLEAN Locked;
//     NTSTATUS Status;
//     PINTERNAL_OPLOCK IntOplock;
//
//     DPRINT("FsRtlRequestOplockII(%p, %p, %p)\n", Oplock, Stack, Irp);
//
//     IntOplock = *Oplock;
//     Locked = FALSE;
//     Status = STATUS_SUCCESS;
//
//     _SEH2_TRY
//     {
//         /* No oplock yet? Allocate it */
//         if (IntOplock == NULL)
//         {
//             *Oplock = FsRtlAllocateOplock();
//             IntOplock = *Oplock;
//         }
//
//         /* Acquire the oplock */
//         ExAcquireFastMutexUnsafe(IntOplock->IntLock);
//         Locked = TRUE;
//
//         /* If already shared, or no oplock that's fine! */
//         if (BooleanFlagOn(IntOplock->Flags, (LEVEL_2_OPLOCK | NO_OPLOCK)))
//         {
//             IoMarkIrpPending(Irp);
//             /* Granted! */
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//
//             /* Insert in the shared list */
//             InsertTailList(&IntOplock->SharedListHead, &Irp->Tail.Overlay.ListEntry);
//
//             /* Save the associated oplock */
//             Irp->IoStatus.Information = (ULONG_PTR)IntOplock;
//
//             /* The oplock is shared */
//             IntOplock->Flags = LEVEL_2_OPLOCK;
//
//             /* Reference the fileobject */
//             ObReferenceObject(Stack->FileObject);
//
//             /* Set our cancel routine, unless the IRP got canceled in-between */
//             IoAcquireCancelSpinLock(&Irp->CancelIrql);
//             if (Irp->Cancel)
//             {
//                 ExReleaseFastMutexUnsafe(IntOplock->IntLock);
//                 Locked = FALSE;
//                 FsRtlCancelOplockIIIrp(NULL, Irp);
//                 Status = STATUS_CANCELLED;
//             }
//             else
//             {
//                 IoSetCancelRoutine(Irp, FsRtlCancelOplockIIIrp);
//                 IoReleaseCancelSpinLock(Irp->CancelIrql);
//             }
//         }
//         /* Otherwise, just fail */
//         else
//         {
//             Irp->IoStatus.Status = STATUS_OPLOCK_NOT_GRANTED;
//             IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//             Status = STATUS_OPLOCK_NOT_GRANTED;
//         }
//     }
//     _SEH2_FINALLY
//     {
//         if (Locked)
//         {
//             ExReleaseFastMutexUnsafe(IntOplock->IntLock);
//         }
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
// VOID
// FsRtlOplockCleanup(IN PINTERNAL_OPLOCK Oplock,
//                    IN PIO_STACK_LOCATION Stack)
// {
//     PIO_STACK_LOCATION ListStack;
//     PLIST_ENTRY NextEntry;
//     PIRP ListIrp;
//     PWAIT_CONTEXT WaitCtx;
//
//     DPRINT("FsRtlOplockCleanup(%p, %p)\n", Oplock, Stack);
//
//     ExAcquireFastMutexUnsafe(Oplock->IntLock);
//     /* oplock cleaning only makes sense if there's an oplock */
//     if (Oplock->Flags != NO_OPLOCK)
//     {
//         /* Shared lock */
//         if (Oplock->Flags == LEVEL_2_OPLOCK)
//         {
//             /* Complete any associated IRP */
//             for (NextEntry = Oplock->SharedListHead.Flink;
//                  NextEntry != &Oplock->SharedListHead;
//                  NextEntry = NextEntry->Flink)
//             {
//                 ListIrp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.ListEntry);
//                 ListStack = IoGetCurrentIrpStackLocation(ListIrp);
//
//                 if (Stack->FileObject == ListStack->FileObject)
//                 {
//                     FsRtlRemoveAndCompleteIrp(ListIrp);
//                 }
//             }
//
//             /* If, in the end, no IRP is left, then the lock is gone */
//             if (IsListEmpty(&Oplock->SharedListHead))
//             {
//                 Oplock->Flags = NO_OPLOCK;
//             }
//         }
//         else
//         {
//             /* If we have matching file */
//             if (Oplock->FileObject == Stack->FileObject)
//             {
//                 /* Oplock wasn't broken (still exclusive), easy case */
//                 if (!BooleanFlagOn(Oplock->Flags, (BROKEN_ANY | PENDING_LOCK)))
//                 {
//                     /* Remove the cancel routine we set previously */
//                     IoAcquireCancelSpinLock(&Oplock->ExclusiveIrp->CancelIrql);
//                     IoSetCancelRoutine(Oplock->ExclusiveIrp, NULL);
//                     IoReleaseCancelSpinLock(Oplock->ExclusiveIrp->CancelIrql);
//
//                     /* And return the fact we broke the oplock to no oplock */
//                     Oplock->ExclusiveIrp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//                     Oplock->ExclusiveIrp->IoStatus.Status = STATUS_SUCCESS;
//
//                     /* And complete! */
//                     IoCompleteRequest(Oplock->ExclusiveIrp, IO_DISK_INCREMENT);
//                     Oplock->ExclusiveIrp = NULL;
//                 }
//
//                 /* If  no pending, we can safely dereference the file object */
//                 if (!BooleanFlagOn(Oplock->Flags, PENDING_LOCK))
//                 {
//                     ObDereferenceObject(Oplock->FileObject);
//                 }
//
//                 /* Now, remove the oplock */
//                 Oplock->FileObject = NULL;
//                 Oplock->Flags = NO_OPLOCK;
//
//                 /* And complete any waiting IRP */
//                 for (NextEntry = Oplock->WaitListHead.Flink;
//                      NextEntry != &Oplock->WaitListHead;
//                      NextEntry = NextEntry->Flink)
//                 {
//                     WaitCtx = CONTAINING_RECORD(NextEntry, WAIT_CONTEXT, WaitListEntry);
//                     FsRtlRemoveAndCompleteWaitIrp(WaitCtx);
//                 }
//             }
//         }
//     }
//     ExReleaseFastMutexUnsafe(Oplock->IntLock);
// }
//
// NTSTATUS
// NTAPI
// FsRtlOplockBreakToNone(IN PINTERNAL_OPLOCK Oplock,
//                        IN PIO_STACK_LOCATION Stack,
//                        IN PIRP Irp,
//                        IN PVOID Context,
//                        IN POPLOCK_WAIT_COMPLETE_ROUTINE CompletionRoutine OPTIONAL,
//                        IN POPLOCK_FS_PREPOST_IRP PostIrpRoutine OPTIONAL)
// {
//     PLIST_ENTRY NextEntry;
//     PWAIT_CONTEXT WaitCtx;
//     PIRP ListIrp;
//     KEVENT WaitEvent;
//
//     DPRINT("FsRtlOplockBreakToNone(%p, %p, %p, %p, %p, %p)\n", Oplock, Stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
//
//     ExAcquireFastMutexUnsafe(Oplock->IntLock);
//
//     /* No oplock to break! */
//     if (Oplock->Flags == NO_OPLOCK)
//     {
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_SUCCESS;
//     }
//
//     /* Not broken yet, but set... Let's do it!
//      * Also, we won't break a shared oplock
//      */
//     if (!BooleanFlagOn(Oplock->Flags, (BROKEN_ANY | PENDING_LOCK | LEVEL_2_OPLOCK)))
//     {
//         /* Remove our cancel routine, no longer needed */
//         IoAcquireCancelSpinLock(&Oplock->ExclusiveIrp->CancelIrql);
//         IoSetCancelRoutine(Oplock->ExclusiveIrp, NULL);
//         IoReleaseCancelSpinLock(Oplock->ExclusiveIrp->CancelIrql);
//
//         /* If the IRP got canceled, we need to cleanup a bit */
//         if (Oplock->ExclusiveIrp->Cancel)
//         {
//             /* Return cancelation */
//             Oplock->ExclusiveIrp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//             Oplock->ExclusiveIrp->IoStatus.Status = STATUS_CANCELLED;
//             IoCompleteRequest(Oplock->ExclusiveIrp, IO_DISK_INCREMENT);
//
//             /* No oplock left */
//             Oplock->Flags = NO_OPLOCK;
//             Oplock->ExclusiveIrp = NULL;
//
//             /* No need for the FO anymore */
//             ObDereferenceObject(Oplock->FileObject);
//             Oplock->FileObject = NULL;
//
//             /* And complete any waiting IRP */
//             for (NextEntry = Oplock->WaitListHead.Flink;
//                  NextEntry != &Oplock->WaitListHead;
//                  NextEntry = NextEntry->Flink)
//             {
//                 WaitCtx = CONTAINING_RECORD(NextEntry, WAIT_CONTEXT, WaitListEntry);
//                 FsRtlRemoveAndCompleteWaitIrp(WaitCtx);
//             }
//
//             /* Done! */
//             ExReleaseFastMutexUnsafe(Oplock->IntLock);
//
//             return STATUS_SUCCESS;
//         }
//
//         /* Easier break, just complete :-) */
//         Oplock->ExclusiveIrp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//         Oplock->ExclusiveIrp->IoStatus.Status = STATUS_SUCCESS;
//         IoCompleteRequest(Oplock->ExclusiveIrp, IO_DISK_INCREMENT);
//
//         /* And remove our exclusive IRP */
//         Oplock->ExclusiveIrp = NULL;
//         SetFlag(Oplock->Flags, BROKEN_TO_NONE);
//     }
//     /* Shared lock */
//     else if (Oplock->Flags == LEVEL_2_OPLOCK)
//     {
//         /* Complete any IRP in the shared lock */
//         for (NextEntry = Oplock->SharedListHead.Flink;
//              NextEntry != &Oplock->SharedListHead;
//              NextEntry = NextEntry->Flink)
//         {
//             ListIrp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.ListEntry);
//             FsRtlRemoveAndCompleteIrp(ListIrp);
//         }
//
//         /* No lock left */
//         Oplock->Flags = NO_OPLOCK;
//
//         /* Done */
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_SUCCESS;
//     }
//     /* If it was broken to level 2, break it to none from level 2 */
//     else if (Oplock->Flags & BROKEN_TO_LEVEL_2)
//     {
//         ClearFlag(Oplock->Flags, BROKEN_TO_LEVEL_2);
//         SetFlag(Oplock->Flags, BROKEN_TO_NONE_FROM_LEVEL_2);
//     }
//     /* If it was pending, just drop the lock */
//     else if (BooleanFlagOn(Oplock->Flags, PENDING_LOCK))
//     {
//         Oplock->Flags = NO_OPLOCK;
//         Oplock->FileObject = NULL;
//
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_SUCCESS;
//     }
//
//     /* If that's ours, job done */
//     if (Oplock->FileObject == Stack->FileObject)
//     {
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_SUCCESS;
//     }
//
//     /* Otherwise, wait on the IRP */
//     if (Stack->MajorFunction != IRP_MJ_CREATE || !BooleanFlagOn(Stack->Parameters.Create.Options, FILE_COMPLETE_IF_OPLOCKED))
//     {
//         FsRtlWaitOnIrp(Oplock, Irp, Context, CompletionRoutine, PostIrpRoutine, &WaitEvent);
//
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//
//         return STATUS_SUCCESS;
//     }
//     /* Done */
//     else
//     {
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_OPLOCK_BREAK_IN_PROGRESS;
//     }
// }
//
// NTSTATUS
// NTAPI
// FsRtlOplockBreakToII(IN PINTERNAL_OPLOCK Oplock,
//                      IN PIO_STACK_LOCATION Stack,
//                      IN PIRP Irp,
//                      IN PVOID Context,
//                      IN POPLOCK_WAIT_COMPLETE_ROUTINE CompletionRoutine OPTIONAL,
//                      IN POPLOCK_FS_PREPOST_IRP PostIrpRoutine OPTIONAL)
// {
//     PLIST_ENTRY NextEntry;
//     PWAIT_CONTEXT WaitCtx;
//     KEVENT WaitEvent;
//
//     DPRINT("FsRtlOplockBreakToII(%p, %p, %p, %p, %p, %p)\n", Oplock, Stack, Irp, Context, CompletionRoutine, PostIrpRoutine);
//
//     ExAcquireFastMutexUnsafe(Oplock->IntLock);
//
//     /* If our lock, or if not exclusively locked, nothing to break! */
//     if (!BooleanFlagOn(Oplock->Flags, EXCLUSIVE_LOCK) || Oplock->FileObject == Stack->FileObject)
//     {
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_SUCCESS;
//     }
//
//     /* If already broken or not set yet */
//     if (BooleanFlagOn(Oplock->Flags, (BROKEN_ANY | PENDING_LOCK)))
//     {
//         /* Drop oplock if pending */
//         if (BooleanFlagOn(Oplock->Flags, PENDING_LOCK))
//         {
//             Oplock->Flags = NO_OPLOCK;
//             Oplock->FileObject = NULL;
//
//             ExReleaseFastMutexUnsafe(Oplock->IntLock);
//             return STATUS_SUCCESS;
//         }
//
//     }
//     /* To break! */
//     else
//     {
//         /* Drop the cancel routine of the exclusive IRP */
//         IoAcquireCancelSpinLock(&Oplock->ExclusiveIrp->CancelIrql);
//         IoSetCancelRoutine(Oplock->ExclusiveIrp, NULL);
//         IoReleaseCancelSpinLock(Oplock->ExclusiveIrp->CancelIrql);
//
//         /* If it was canceled in between, break to no oplock */
//         if (Oplock->ExclusiveIrp->Cancel)
//         {
//             /* Complete the IRP with cancellation */
//             Oplock->ExclusiveIrp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//             Oplock->ExclusiveIrp->IoStatus.Status = STATUS_CANCELLED;
//             IoCompleteRequest(Oplock->ExclusiveIrp, IO_DISK_INCREMENT);
//
//             /* And mark we have no longer lock */
//             Oplock->Flags = NO_OPLOCK;
//             Oplock->ExclusiveIrp = NULL;
//             ObDereferenceObject(Oplock->FileObject);
//             Oplock->FileObject = NULL;
//
//             /* Finally, complete any waiter */
//             for (NextEntry = Oplock->WaitListHead.Flink;
//                  NextEntry != &Oplock->WaitListHead;
//                  NextEntry = NextEntry->Flink)
//             {
//                 WaitCtx = CONTAINING_RECORD(NextEntry, WAIT_CONTEXT, WaitListEntry);
//                 FsRtlRemoveAndCompleteWaitIrp(WaitCtx);
//             }
//
//             ExReleaseFastMutexUnsafe(Oplock->IntLock);
//
//             return STATUS_SUCCESS;
//         }
//
//         /* It wasn't canceled, so break to shared unless we were alone, in that case we break to no lock! */
//         Oplock->ExclusiveIrp->IoStatus.Status = STATUS_SUCCESS;
//         if (BooleanFlagOn(Oplock->Flags, (BATCH_OPLOCK | LEVEL_1_OPLOCK)))
//         {
//             SetFlag(Oplock->Flags, BROKEN_TO_LEVEL_2);
//             Oplock->ExclusiveIrp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_LEVEL_2;
//         }
//         else
//         {
//             SetFlag(Oplock->Flags, BROKEN_TO_NONE);
//             Oplock->ExclusiveIrp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//         }
//         /* And complete */
//         IoCompleteRequest(Oplock->ExclusiveIrp, IO_DISK_INCREMENT);
//         Oplock->ExclusiveIrp = NULL;
//     }
//
//     /* Wait if required */
//     if (Stack->MajorFunction != IRP_MJ_CREATE || !BooleanFlagOn(Stack->Parameters.Create.Options, FILE_COMPLETE_IF_OPLOCKED))
//     {
//         FsRtlWaitOnIrp(Oplock, Irp, Context, CompletionRoutine, PostIrpRoutine, &WaitEvent);
//
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         ExReleaseFastMutexUnsafe(Oplock->IntLock);
//         return STATUS_OPLOCK_BREAK_IN_PROGRESS;
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlCheckOplock
//  * @unimplemented
//  *
//  * FILLME
//  *
//  * @param Oplock
//  *        FILLME
//  *
//  * @param Irp
//  *        FILLME
//  *
//  * @param Context
//  *        FILLME
//  *
//  * @param CompletionRoutine
//  *        FILLME
//  *
//  * @param PostIrpRoutine
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// FsRtlCheckOplock(IN POPLOCK Oplock,
//                  IN PIRP Irp,
//                  IN PVOID Context,
//                  IN POPLOCK_WAIT_COMPLETE_ROUTINE CompletionRoutine OPTIONAL,
//                  IN POPLOCK_FS_PREPOST_IRP PostIrpRoutine OPTIONAL)
// {
//     PINTERNAL_OPLOCK IntOplock;
//     PIO_STACK_LOCATION Stack;
//     ACCESS_MASK DesiredAccess;
//     FILE_INFORMATION_CLASS FileInfo;
//     ULONG CreateDisposition;
//
// #define BreakToIIIfRequired                                                               \
//     if (IntOplock->Flags != LEVEL_2_OPLOCK || IntOplock->FileObject != Stack->FileObject) \
//         return FsRtlOplockBreakToII(IntOplock, Stack, Irp, Context, CompletionRoutine, PostIrpRoutine)
//
// #define BreakToNoneIfRequired                                                             \
//     if (IntOplock->Flags == LEVEL_2_OPLOCK || IntOplock->FileObject != Stack->FileObject) \
//         return FsRtlOplockBreakToNone(IntOplock, Stack, Irp, Context, CompletionRoutine, PostIrpRoutine)
//
//     DPRINT("FsRtlCheckOplock(%p, %p, %p, %p, %p)\n", Oplock, Irp, Context, CompletionRoutine, PostIrpRoutine);
//
//     IntOplock = *Oplock;
//
//     /* No oplock, easy! */
//     if (IntOplock == NULL)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* No sense on paging */
//     if (Irp->Flags & IRP_PAGING_IO)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* No oplock, easy (bis!) */
//     if (IntOplock->Flags == NO_OPLOCK)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* If cleanup, cleanup the associated oplock & return */
//     if (Stack->MajorFunction == IRP_MJ_CLEANUP)
//     {
//         FsRtlOplockCleanup(IntOplock, Stack);
//         return STATUS_SUCCESS;
//     }
//     else if (Stack->MajorFunction == IRP_MJ_LOCK_CONTROL)
//     {
//         /* OK for filter */
//         if (BooleanFlagOn(IntOplock->Flags, FILTER_OPLOCK))
//         {
//             return STATUS_SUCCESS;
//         }
//
//         /* Lock operation, we will have to break to no lock if shared or not us */
//         BreakToNoneIfRequired;
//
//         return STATUS_SUCCESS;
//     }
//     else if (Stack->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL)
//     {
//         /* FSCTL should be safe, unless user wants a write FSCTL */
//         if (Stack->Parameters.FileSystemControl.FsControlCode != FSCTL_SET_ZERO_DATA)
//         {
//             return STATUS_SUCCESS;
//         }
//
//         /* We will have to break for write if shared or not us! */
//         BreakToNoneIfRequired;
//
//         return STATUS_SUCCESS;
//     }
//     else if (Stack->MajorFunction == IRP_MJ_WRITE)
//     {
//         /* Write operation, we will have to break if shared or not us */
//         BreakToNoneIfRequired;
//
//         return STATUS_SUCCESS;
//     }
//     else if (Stack->MajorFunction == IRP_MJ_READ)
//     {
//         /* If that's filter oplock, it's alright */
//         if (BooleanFlagOn(IntOplock->Flags, FILTER_OPLOCK))
//         {
//             return STATUS_SUCCESS;
//         }
//
//         /* Otherwise, we need to break to shared oplock */
//         BreakToIIIfRequired;
//
//         return STATUS_SUCCESS;
//     }
//     else if (Stack->MajorFunction == IRP_MJ_CREATE)
//     {
//         DesiredAccess = Stack->Parameters.Create.SecurityContext->DesiredAccess;
//
//         /* If that's just for reading, the oplock is fine */
//         if ((!(DesiredAccess & ~(SYNCHRONIZE | FILE_WRITE_ATTRIBUTES | FILE_READ_ATTRIBUTES | FILE_READ_DATA)) && !(Stack->Parameters.Create.Options & FILE_RESERVE_OPFILTER))
//             || (BooleanFlagOn(IntOplock->Flags, FILTER_OPLOCK) && !(DesiredAccess & ~(SYNCHRONIZE | READ_CONTROL | FILE_WRITE_ATTRIBUTES | FILE_READ_ATTRIBUTES | FILE_EXECUTE | FILE_READ_EA | FILE_WRITE_DATA)) && BooleanFlagOn(Stack->Parameters.Create.ShareAccess, FILE_SHARE_READ)))
//         {
//             return STATUS_SUCCESS;
//         }
//
//         /* Otherwise, check the disposition */
//         CreateDisposition = (Stack->Parameters.Create.Options >> 24) & 0x000000FF;
//         if (!CreateDisposition || CreateDisposition == FILE_OVERWRITE ||
//             CreateDisposition == FILE_OVERWRITE_IF ||
//             BooleanFlagOn(Stack->Parameters.Create.Options, FILE_RESERVE_OPFILTER))
//         {
//             /* Not us, we have to break the oplock! */
//             BreakToNoneIfRequired;
//
//             return STATUS_SUCCESS;
//         }
//
//         /* It's fine, we can have the oplock shared */
//         BreakToIIIfRequired;
//
//         return STATUS_SUCCESS;
//     }
//     else if (Stack->MajorFunction == IRP_MJ_FLUSH_BUFFERS)
//     {
//         /* We need to share the lock, if not done yet! */
//         BreakToIIIfRequired;
//
//         return STATUS_SUCCESS;
//     }
//     else if (Stack->MajorFunction == IRP_MJ_SET_INFORMATION)
//     {
//         /* Only deal with really specific classes */
//         FileInfo = Stack->Parameters.SetFile.FileInformationClass;
//         if (FileInfo == FileRenameInformation || FileInfo == FileLinkInformation ||
//             FileInfo == FileShortNameInformation)
//         {
//             /* No need to break */
//             if (!BooleanFlagOn(IntOplock->Flags, (FILTER_OPLOCK | BATCH_OPLOCK)))
//             {
//                 return STATUS_SUCCESS;
//             }
//             /* Otherwise break to none */
//             else
//             {
//                 BreakToNoneIfRequired;
//
//                 return STATUS_SUCCESS;
//             }
//         }
//         else if (FileInfo == FileAllocationInformation)
//         {
//             BreakToNoneIfRequired;
//
//             return STATUS_SUCCESS;
//         }
//         else if (FileInfo == FileEndOfFileInformation)
//         {
//             /* Advance only, nothing to do */
//             if (Stack->Parameters.SetFile.AdvanceOnly)
//             {
//                 return STATUS_SUCCESS;
//             }
//
//             /* Otherwise, attempt to break to none */
//             BreakToNoneIfRequired;
//
//             return STATUS_SUCCESS;
//         }
//     }
//
// #undef BreakToIIIfRequired
// #undef BreakToNoneIfRequired
//
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name FsRtlCurrentBatchOplock
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Oplock
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlCurrentBatchOplock(IN POPLOCK Oplock)
// {
//     PINTERNAL_OPLOCK IntOplock;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlCurrentBatchOplock(%p)\n", Oplock);
//
//     IntOplock = *Oplock;
//
//     /* Only return true if batch or filter oplock */
//     if (IntOplock != NULL &&
//         BooleanFlagOn(IntOplock->Flags, (FILTER_OPLOCK | BATCH_OPLOCK)))
//     {
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// /*++
//  * @name FsRtlInitializeOplock
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Oplock
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlInitializeOplock(IN OUT POPLOCK Oplock)
// {
//     PAGED_CODE();
//
//     /* Nothing to do */
//     DPRINT("FsRtlInitializeOplock(%p)\n", Oplock);
// }
//
// /*++
//  * @name FsRtlOplockFsctrl
//  * @unimplemented
//  *
//  * FILLME
//  *
//  * @param Oplock
//  *        FILLME
//  *
//  * @param Irp
//  *        FILLME
//  *
//  * @param OpenCount
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// FsRtlOplockFsctrl(IN POPLOCK Oplock,
//                   IN PIRP Irp,
//                   IN ULONG OpenCount)
// {
//     PIO_STACK_LOCATION Stack;
//     PINTERNAL_OPLOCK IntOplock;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlOplockFsctrl(%p, %p, %lu)\n", Oplock, Irp, OpenCount);
//
//     IntOplock = *Oplock;
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     /* Make sure it's not called on create */
//     if (Stack->MajorFunction != IRP_MJ_CREATE)
//     {
//         switch (Stack->Parameters.FileSystemControl.FsControlCode)
//         {
//             case FSCTL_OPLOCK_BREAK_NOTIFY:
//                 return FsRtlOplockBreakNotify(IntOplock, Stack, Irp);
//
//             case FSCTL_OPLOCK_BREAK_ACK_NO_2:
//                 return FsRtlAcknowledgeOplockBreak(IntOplock, Stack, Irp, FALSE);
//
//             case FSCTL_OPBATCH_ACK_CLOSE_PENDING:
//                 return FsRtlOpBatchBreakClosePending(IntOplock, Stack, Irp);
//
//             case FSCTL_REQUEST_OPLOCK_LEVEL_1:
//                 /* We can only grant level 1 if synchronous, and only a single handle to it
//                  * (plus, not a paging IO - obvious, and not cleanup done...)
//                  */
//                 if (OpenCount == 1 && !IoIsOperationSynchronous(Irp) &&
//                     !BooleanFlagOn(Irp->Flags, IRP_SYNCHRONOUS_PAGING_IO) && !BooleanFlagOn(Stack->FileObject->Flags, FO_CLEANUP_COMPLETE))
//                 {
//                     return FsRtlRequestExclusiveOplock(Oplock, Stack, Irp, EXCLUSIVE_LOCK | LEVEL_1_OPLOCK);
//                 }
//                 /* Not matching, fail */
//                 else
//                 {
//                     Irp->IoStatus.Status = STATUS_OPLOCK_NOT_GRANTED;
//                     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//                     return STATUS_OPLOCK_NOT_GRANTED;
//                 }
//
//             case FSCTL_REQUEST_OPLOCK_LEVEL_2:
//                 /* Shared can only be granted if no byte-range lock, and async operation
//                  * (plus, not a paging IO - obvious, and not cleanup done...)
//                  */
//                 if (OpenCount == 0 && !IoIsOperationSynchronous(Irp) &&
//                     !BooleanFlagOn(Irp->Flags, IRP_SYNCHRONOUS_PAGING_IO) && !BooleanFlagOn(Stack->FileObject->Flags, FO_CLEANUP_COMPLETE))
//                 {
//                     return FsRtlRequestOplockII(Oplock, Stack, Irp);
//                 }
//                 /* Not matching, fail */
//                 else
//                 {
//                     Irp->IoStatus.Status = STATUS_OPLOCK_NOT_GRANTED;
//                     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//                     return STATUS_OPLOCK_NOT_GRANTED;
//                 }
//
//             case FSCTL_OPLOCK_BREAK_ACKNOWLEDGE:
//                 return FsRtlAcknowledgeOplockBreak(IntOplock, Stack, Irp, TRUE);
//
//             case FSCTL_REQUEST_BATCH_OPLOCK:
//                 /* Batch oplock can only be granted if there's a byte-range lock and async operation
//                  * (plus, not a paging IO - obvious, and not cleanup done...)
//                  */
//                 if (OpenCount == 1 && !IoIsOperationSynchronous(Irp) &&
//                     !BooleanFlagOn(Irp->Flags, IRP_SYNCHRONOUS_PAGING_IO) && !BooleanFlagOn(Stack->FileObject->Flags, FO_CLEANUP_COMPLETE))
//                 {
//                     return FsRtlRequestExclusiveOplock(Oplock, Stack, Irp, EXCLUSIVE_LOCK | BATCH_OPLOCK);
//                 }
//                 /* Not matching, fail */
//                 else
//                 {
//                     Irp->IoStatus.Status = STATUS_OPLOCK_NOT_GRANTED;
//                     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//                     return STATUS_OPLOCK_NOT_GRANTED;
//                 }
//
//             case FSCTL_REQUEST_FILTER_OPLOCK:
//                 /* Filter oplock can only be granted if there's a byte-range lock and async operation
//                  * (plus, not a paging IO - obvious, and not cleanup done...)
//                  */
//                 if (OpenCount == 1 && !IoIsOperationSynchronous(Irp) &&
//                     !BooleanFlagOn(Irp->Flags, IRP_SYNCHRONOUS_PAGING_IO) && !BooleanFlagOn(Stack->FileObject->Flags, FO_CLEANUP_COMPLETE))
//                 {
//                     return FsRtlRequestExclusiveOplock(Oplock, Stack, Irp, EXCLUSIVE_LOCK | FILTER_OPLOCK);
//                 }
//                 /* Not matching, fail */
//                 else
//                 {
//                     Irp->IoStatus.Status = STATUS_OPLOCK_NOT_GRANTED;
//                     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//                     return STATUS_OPLOCK_NOT_GRANTED;
//                 }
//
//             default:
//                 Irp->IoStatus.Status = STATUS_INVALID_PARAMETER;
//                 IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//                 return STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     /* That's a create operation! Only grant exclusive if there's a single user handle opened
//      * and we're only performing reading operations.
//      */
//     if (OpenCount == 1 &&
//         !(Stack->Parameters.Create.SecurityContext->DesiredAccess & ~(FILE_READ_ATTRIBUTES | FILE_READ_DATA)) &&
//         (Stack->Parameters.Create.ShareAccess & FILE_SHARE_VALID_FLAGS) == FILE_SHARE_VALID_FLAGS)
//     {
//         return FsRtlRequestExclusiveOplock(Oplock, Stack, NULL, EXCLUSIVE_LOCK | PENDING_LOCK | FILTER_OPLOCK);
//     }
//
//     return STATUS_OPLOCK_NOT_GRANTED;
// }
//
// /*++
//  * @name FsRtlOplockIsFastIoPossible
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Oplock
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlOplockIsFastIoPossible(IN POPLOCK Oplock)
// {
//     PINTERNAL_OPLOCK IntOplock;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlOplockIsFastIoPossible(%p)\n", Oplock);
//
//     IntOplock = *Oplock;
//
//     /* If there's a shared oplock or if it was used for write operation, deny FastIO */
//     if (IntOplock != NULL &&
//         BooleanFlagOn(IntOplock->Flags, (BROKEN_ANY | LEVEL_2_OPLOCK)))
//     {
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*++
//  * @name FsRtlUninitializeOplock
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Oplock
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlUninitializeOplock(IN POPLOCK Oplock)
// {
//     PINTERNAL_OPLOCK IntOplock;
//     PLIST_ENTRY NextEntry;
//     PWAIT_CONTEXT WaitCtx;
//     PIRP Irp;
//     PIO_STACK_LOCATION Stack;
//
//     DPRINT("FsRtlUninitializeOplock(%p)\n", Oplock);
//
//     IntOplock = *Oplock;
//
//     /* No oplock, nothing to do */
//     if (IntOplock == NULL)
//     {
//         return;
//     }
//
//     /* Caller won't have the oplock anymore */
//     *Oplock = NULL;
//
//     _SEH2_TRY
//     {
//         ExAcquireFastMutexUnsafe(IntOplock->IntLock);
//
//         /* If we had IRPs waiting for the lock, complete them */
//         for (NextEntry = IntOplock->WaitListHead.Flink;
//              NextEntry != &IntOplock->WaitListHead;
//             NextEntry = NextEntry->Flink)
//         {
//             WaitCtx = CONTAINING_RECORD(NextEntry, WAIT_CONTEXT, WaitListEntry);
//             Irp = WaitCtx->Irp;
//
//             RemoveEntryList(&WaitCtx->WaitListEntry);
//             /* Remove the cancel routine */
//             IoAcquireCancelSpinLock(&Irp->CancelIrql);
//             IoSetCancelRoutine(Irp, NULL);
//             IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//             /* And complete */
//             Irp->IoStatus.Information = 0;
//             WaitCtx->CompletionRoutine(WaitCtx->CompletionContext, WaitCtx->Irp);
//
//             ExFreePoolWithTag(WaitCtx, TAG_OPLOCK);
//         }
//
//         /* If we had shared IRPs (LEVEL_2), complete them */
//         for (NextEntry = IntOplock->SharedListHead.Flink;
//              NextEntry != &IntOplock->SharedListHead;
//              NextEntry = NextEntry->Flink)
//         {
//             Irp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.ListEntry);
//
//             RemoveEntryList(&Irp->Tail.Overlay.ListEntry);
//
//             /* Remvoe the cancel routine */
//             IoAcquireCancelSpinLock(&Irp->CancelIrql);
//             IoSetCancelRoutine(Irp, NULL);
//             IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//             /* Dereference the file object */
//             Stack = IoGetCurrentIrpStackLocation(Irp);
//             ObDereferenceObject(Stack->FileObject);
//
//             /* And complete */
//             Irp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//         }
//
//         /* If we have an exclusive IRP, complete it */
//         Irp = IntOplock->ExclusiveIrp;
//         if (Irp != NULL)
//         {
//             /* Remvoe the cancel routine */
//             IoAcquireCancelSpinLock(&Irp->CancelIrql);
//             IoSetCancelRoutine(Irp, NULL);
//             IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//             /* And complete */
//             Irp->IoStatus.Information = FILE_OPLOCK_BROKEN_TO_NONE;
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//             IntOplock->ExclusiveIrp = NULL;
//
//             /* If still referenced, dereference */
//             if (IntOplock->FileObject != NULL)
//             {
//                 ObDereferenceObject(IntOplock->FileObject);
//             }
//         }
//     }
//     _SEH2_FINALLY
//     {
//         ExReleaseFastMutexUnsafe(IntOplock->IntLock);
//     }
//     _SEH2_END;
//
//     ExFreePoolWithTag(IntOplock->IntLock, TAG_OPLOCK);
//     ExFreePoolWithTag(IntOplock, TAG_OPLOCK);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<INTERNAL_OPLOCK>() == 64);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, ExclusiveIrp) == 0);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, FileObject) == 8);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, SharedListHead) == 16);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, WaitListHead) == 32);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, Flags) == 48);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, IntLock) == 56);
    assert!(core::mem::size_of::<WAIT_CONTEXT>() == 56);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry) == 0);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, Irp) == 16);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, CompletionRoutine) == 24);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, CompletionContext) == 32);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, Reserved) == 40);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, SavedInformation) == 48);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<INTERNAL_OPLOCK>() == 32);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, ExclusiveIrp) == 0);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, FileObject) == 4);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, SharedListHead) == 8);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, WaitListHead) == 16);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, Flags) == 24);
    assert!(core::mem::offset_of!(INTERNAL_OPLOCK, IntLock) == 28);
    assert!(core::mem::size_of::<WAIT_CONTEXT>() == 28);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, WaitListEntry) == 0);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, Irp) == 8);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, CompletionRoutine) == 12);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, CompletionContext) == 16);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, Reserved) == 20);
    assert!(core::mem::offset_of!(WAIT_CONTEXT, SavedInformation) == 24);
};
// END OFFSET CHECKS
