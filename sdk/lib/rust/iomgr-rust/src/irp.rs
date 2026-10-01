use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _IRP, _IO_STACK_LOCATION, _IO_STATUS_BLOCK, _FILE_OBJECT, _DEVICE_OBJECT, _DRIVER_OBJECT,
    _ETHREAD, _KAPC, _KEVENT, _MDL, _KPRCB, _NPAGED_LOOKASIDE_LIST,
    _PP_NPAGED_LOOKASIDE_NUMBER, _REPARSE_DATA_BUFFER, _IO_ERROR_LOG_PACKET,
    _EPROCESS, _KTHREAD,
    PIRP, PIO_STACK_LOCATION, PFILE_OBJECT, PDEVICE_OBJECT, PDRIVER_OBJECT,
    PETHREAD, PKAPC, PKEVENT, PMDL, PKPRCB, PNPAGED_LOOKASIDE_LIST,
    PP_NPAGED_LOOKASIDE_NUMBER, PREPARSE_DATA_BUFFER, PIO_ERROR_LOG_PACKET,
    PEPROCESS, PIO_STATUS_BLOCK, PCHAR, CCHAR,
    KPROCESSOR_MODE, HANDLE, PKNORMAL_ROUTINE, PDRIVER_CANCEL, PIO_COMPLETION_ROUTINE,
    IoAllocateErrorLogEntry, IoWriteErrorLogEntry, IoFreeMdl,
    KeReadStateEvent, KeDelayExecutionThread, KeWaitForSingleObject,
    KeInitializeApc, KeInsertQueueApc, MmUnlockPages, MmProbeAndLockPages,
    MmGetSessionId, IoAllocateMdl, IopDoNameTransmogrify,
    ObDereferenceObjectDeferDelete,
    KeInitializeEvent,
    ExFreePool, PsGetCurrentThread,
    _IO_PAGING_PRIORITY, IO_PAGING_PRIORITY,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList, _PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList,
    _EVENT_TYPE_NotificationEvent, _EVENT_TYPE_SynchronizationEvent,
    _KWAIT_REASON_Executive, _KWAIT_REASON_Suspended,
    _KAPC_ENVIRONMENT_CurrentApcEnvironment, _KAPC_ENVIRONMENT_OriginalApcEnvironment,
    _KAPC_ENVIRONMENT_AttachedApcEnvironment,
    _LOCK_OPERATION_IoWriteAccess, _LOCK_OPERATION_IoReadAccess,
    _IO_PAGING_PRIORITY_IoPagingPriorityHigh, _IO_PAGING_PRIORITY_IoPagingPriorityNormal,
    _IO_PAGING_PRIORITY_IoPagingPriorityInvalid,
    _COMPLETION_PACKET_TYPE_IopCompletionPacketIrp,
    _IOP_TRANSFER_TYPE_IopWriteTransfer, _IOP_TRANSFER_TYPE_IopReadTransfer,
    _IOP_TRANSFER_TYPE_IopOtherTransfer,
};

// Alias prefixed enum constants to short names used throughout the file
const CurrentApcEnvironment: u32 = _KAPC_ENVIRONMENT_CurrentApcEnvironment;
const OriginalApcEnvironment: u32 = _KAPC_ENVIRONMENT_OriginalApcEnvironment;
const AttachedApcEnvironment: u32 = _KAPC_ENVIRONMENT_AttachedApcEnvironment;
const IoWriteAccess: u32 = _LOCK_OPERATION_IoWriteAccess;
const IoReadAccess: u32 = _LOCK_OPERATION_IoReadAccess;
const IoPagingPriorityHigh: IO_PAGING_PRIORITY = _IO_PAGING_PRIORITY_IoPagingPriorityHigh;
const IoPagingPriorityNormal: IO_PAGING_PRIORITY = _IO_PAGING_PRIORITY_IoPagingPriorityNormal;
const IoPagingPriorityInvalid: IO_PAGING_PRIORITY = _IO_PAGING_PRIORITY_IoPagingPriorityInvalid;
const IopCompletionPacketIrp: u32 = _COMPLETION_PACKET_TYPE_IopCompletionPacketIrp;
const IopWriteTransfer: u32 = _IOP_TRANSFER_TYPE_IopWriteTransfer;
const IopReadTransfer: u32 = _IOP_TRANSFER_TYPE_IopReadTransfer;
const IopOtherTransfer: u32 = _IOP_TRANSFER_TYPE_IopOtherTransfer;

// IRP flags (from iotypes.h)
const IRP_PAGING_IO: ULONG = 0x00000002;
const IRP_SYNCHRONOUS_API: ULONG = 0x00000004;
const IRP_ASSOCIATED_IRP: ULONG = 0x00000008;
const IRP_BUFFERED_IO: ULONG = 0x00000010;
const IRP_DEALLOCATE_BUFFER: ULONG = 0x00000020;
const IRP_INPUT_OPERATION: ULONG = 0x00000040;
const IRP_SYNCHRONOUS_PAGING_IO: ULONG = 0x00000040;
const IRP_CREATE_OPERATION: ULONG = 0x00000080;
const IRP_READ_OPERATION: ULONG = 0x00000100;
const IRP_WRITE_OPERATION: ULONG = 0x00000200;
const IRP_CLOSE_OPERATION: ULONG = 0x00000400;
const IRP_DEFER_IO_COMPLETION: ULONG = 0x00000800;
const IRP_OB_QUERY_NAME: ULONG = 0x00001000;
const IRP_CLASS_CACHE_OPERATION: ULONG = 0x00008000;

// IRP allocation flags (UCHAR)
const IRP_QUOTA_CHARGED: UCHAR = 0x01;
const IRP_ALLOCATED_FIXED_SIZE: UCHAR = 0x04;
const IRP_LOOKASIDE_ALLOCATION: UCHAR = 0x08;

// Stack location control flags
const SL_PENDING_RETURNED: UCHAR = 0x01;
const SL_ERROR_RETURNED: UCHAR = 0x02;
const SL_INVOKE_ON_CANCEL: UCHAR = 0x20;
const SL_INVOKE_ON_SUCCESS: UCHAR = 0x40;
const SL_INVOKE_ON_ERROR: UCHAR = 0x80;

// File object flags
const FO_SYNCHRONOUS_IO: ULONG = 0x00000002;

// Device object flags
const DO_BUFFERED_IO: ULONG = 0x00000004;
const DO_DIRECT_IO: ULONG = 0x00000010;

// IO type
const IO_TYPE_IRP: USHORT = 6;

// IO increment
const IO_NO_INCREMENT: i32 = 0;

// IO reparse/remount
const IO_REPARSE_TAG_MOUNT_POINT: ULONG = 0xA0000003;
const IO_REMOUNT: ULONG = 0x1;
const IO_DRIVER_CANCEL_TIMEOUT: NTSTATUS = 0x80040036_u32 as i32;

// IRP major function codes
const IRP_MJ_READ: ULONG = 0x03;
const IRP_MJ_WRITE: ULONG = 0x04;
const IRP_MJ_FLUSH_BUFFERS: ULONG = 0x09;
const IRP_MJ_DEVICE_CONTROL: ULONG = 0x0e;
const IRP_MJ_INTERNAL_DEVICE_CONTROL: ULONG = 0x0f;
const IRP_MJ_SHUTDOWN: ULONG = 0x10;
const IRP_MJ_POWER: ULONG = 0x16;
const IRP_MJ_PNP: ULONG = 0x1b;

// IO control method
const METHOD_BUFFERED: ULONG = 0;
const METHOD_IN_DIRECT: ULONG = 1;
const METHOD_OUT_DIRECT: ULONG = 2;
const METHOD_NEITHER: ULONG = 3;

// Pool tags
const TAG_IRP: ULONG = 0x20707249;
const TAG_IOBUF: ULONG = 0x55424F49;

// Reparse buffer size
const MAXIMUM_REPARSE_DATA_BUFFER_SIZE: ULONG = 16 * 1024;

// Bugcheck codes
const NO_MORE_IRP_STACK_LOCATIONS: ULONG = 0x35;
const MULTIPLE_IRP_COMPLETE_REQUESTS: ULONG = 0x44;
const CANCEL_STATE_IN_COMPLETED_IRP: ULONG = 0x48;

// Missing status codes
const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x80000016_u32 as i32;
const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016_u32 as i32;
const STATUS_IO_REPARSE_TAG_NOT_HANDLED: NTSTATUS = 0xC0000279_u32 as i32;

// Pool flag
const POOL_QUOTA_FAIL_INSTEAD_OF_RAISE: ULONG = 0x8;

// Lock queue index
const LockQueueIoCompletionLock: ULONG = 11;

// Inline function: IoSizeOfIrp
#[inline]
unsafe fn IoSizeOfIrp(StackSize: CCHAR) -> USHORT {
    (size_of::<_IRP>() + (StackSize as usize) * size_of::<_IO_STACK_LOCATION>()) as USHORT
}

// NOTE (2026-07-09): the aarch64 "C-truth" shims that used to live here
// (IO_STACK_LOCATION stride 0x44, CurrentStackLocation slot at IRP+0xb8) are
// GONE. They compensated for the arm64 header bug where the iotypes.h pack(4)
// guard missed _ARM64_ (clang aarch64-w64-mingw32 ignores per-field
// __attribute__((aligned)) under #pragma pack, so C and bindgen genuinely
// disagreed). The guard is fixed, bindgen is regenerated, C == bindgen on all
// arches — plain field access / pointer arithmetic is correct everywhere.
// Leaving the shims in produced MIXED strides (68 vs 72) inside one kernel and
// wedged boot at the first synchronous PnP IRP (CompletionRoutine read as None).

#[inline]
unsafe fn irp_current_stack_location_slot(irp: PIRP) -> *mut PIO_STACK_LOCATION {
    core::ptr::addr_of_mut!(
        (*irp)
            .Tail
            .Overlay
            .__bindgen_anon_2
            .__bindgen_anon_1
            .CurrentStackLocation
    )
}

#[inline]
unsafe fn get_current_stack_location(irp: PIRP) -> PIO_STACK_LOCATION {
    *irp_current_stack_location_slot(irp)
}

#[inline]
unsafe fn set_current_stack_location(irp: PIRP, stack: PIO_STACK_LOCATION) {
    *irp_current_stack_location_slot(irp) = stack;
}

#[inline]
unsafe fn stack_location_add(stack: PIO_STACK_LOCATION, count: isize) -> PIO_STACK_LOCATION {
    stack.offset(count)
}

#[inline]
unsafe fn first_stack_location(irp: PIRP) -> PIO_STACK_LOCATION {
    (irp as *mut u8).add(size_of::<_IRP>()) as PIO_STACK_LOCATION
}

// FORCEINLINE shim declarations (implemented in io_rust_shims.c)
extern "system" {
    #[link_name = "IopUnlockFileObject_shim"]
    fn IopUnlockFileObject(FileObject: PFILE_OBJECT);
    #[link_name = "IopUpdateTransferCount_shim"]
    fn IopUpdateTransferCount(Type: ULONG, TransferCount: ULONG);
    #[link_name = "IopQueueIrpToThread_shim"]
    fn IopQueueIrpToThread(Irp: PIRP);
    #[link_name = "IopUnQueueIrpFromThread_shim"]
    fn IopUnQueueIrpFromThread(Irp: PIRP);
}
// InterlockedExchange is fastcall on i386, cdecl on others
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
}

// KeRaiseIrql: i386 uses KfRaiseIrql (FASTCALL macro), amd64/arm64 have real symbol
#[cfg(target_arch = "x86")]
unsafe fn KeRaiseIrql(new_irql: KIRQL, old_irql: *mut KIRQL) {
    extern "fastcall" {
        fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    }
    *old_irql = KfRaiseIrql(new_irql);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
}

// KeLowerIrql: i386 uses KfLowerIrql (FASTCALL macro), amd64/arm64 have real symbol
#[cfg(target_arch = "x86")]
unsafe fn KeLowerIrql(new_irql: KIRQL) {
    extern "fastcall" {
        fn KfLowerIrql(NewIrql: KIRQL);
    }
    KfLowerIrql(new_irql);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KeLowerIrql(NewIrql: KIRQL);
}

use core::ptr::{null_mut, addr_of, addr_of_mut};
use core::mem::{size_of, zeroed};
use core::ffi::c_void;

static mut IopDeadIrp: PIRP = null_mut();

#[repr(C)]
pub struct RESERVE_IRP_ALLOCATOR {
    pub ReserveIrp: PIRP,
    pub ReserveIrpInUse: LONG,
    pub WaitEvent: _KEVENT,
    pub StackSize: CCHAR,
}

#[no_mangle]
pub static mut IopReserveIrpAllocator: RESERVE_IRP_ALLOCATOR = unsafe { zeroed() };

unsafe fn CONTAINING_RECORD(ptr: *mut c_void, offset: usize) -> *mut c_void {
    (ptr as usize - offset) as *mut c_void
}

unsafe fn IRP_FROM_APC(apc: PKAPC) -> PIRP {
    let offset = core::mem::offset_of!(_IRP, Tail);
    CONTAINING_RECORD(apc as *mut c_void, offset) as PIRP
}

unsafe fn NT_ERROR(status: NTSTATUS) -> bool {
    ((status as u32) >> 30) == 3
}

unsafe fn IsIrpSynchronous(irp: PIRP, file_object: PFILE_OBJECT) -> bool {
    ((*irp).Flags & IRP_SYNCHRONOUS_API != 0)
        || (!file_object.is_null() && ((*file_object).Flags & FO_SYNCHRONOUS_IO) != 0)
}

unsafe fn IO_METHOD_FROM_CTL_CODE(code: ULONG) -> ULONG {
    code & 3
}

unsafe fn IopClearStackLocation(stack: PIO_STACK_LOCATION) {
    (*stack).MinorFunction = 0;
    (*stack).Flags = 0;
    (*stack).Control &= SL_ERROR_RETURNED;
    (*stack).Parameters.Others.Argument1 = null_mut();
    (*stack).Parameters.Others.Argument2 = null_mut();
    (*stack).Parameters.Others.Argument3 = null_mut();
    (*stack).FileObject = null_mut();
}

unsafe fn IoGetCurrentIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION {
    get_current_stack_location(irp)
}

unsafe fn IoGetNextIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION {
    stack_location_add(get_current_stack_location(irp), -1)
}

unsafe fn IoMarkIrpPending(irp: PIRP) {
    let stack = IoGetCurrentIrpStackLocation(irp);
    (*stack).Control |= SL_PENDING_RETURNED;
}

unsafe fn IoSetCancelRoutine(irp: PIRP, routine: PDRIVER_CANCEL) -> PDRIVER_CANCEL {
    let old: PDRIVER_CANCEL = core::mem::transmute((*irp).CancelRoutine);
    (*irp).CancelRoutine = core::mem::transmute(routine);
    old
}

unsafe fn IoAcquireCancelSpinLock(irql: *mut KIRQL) {
    extern "system" {
        fn IoAcquireCancelSpinLock(Irql: *mut KIRQL);
    }
    IoAcquireCancelSpinLock(irql)
}

unsafe fn IoReleaseCancelSpinLock(irql: KIRQL) {
    extern "system" {
        fn IoReleaseCancelSpinLock(Irql: KIRQL);
    }
    IoReleaseCancelSpinLock(irql)
}

unsafe fn IoCopyCurrentIrpStackLocationToNext(irp: PIRP) {
    let src = IoGetCurrentIrpStackLocation(irp);
    let dst = IoGetNextIrpStackLocation(irp);
    core::ptr::copy_nonoverlapping(src, dst, 1);
}

unsafe fn IoSetCompletionRoutine(
    irp: PIRP,
    routine: PIO_COMPLETION_ROUTINE,
    context: PVOID,
    on_success: bool,
    on_error: bool,
    on_cancel: bool,
) {
    let stack = IoGetNextIrpStackLocation(irp);
    (*stack).CompletionRoutine = routine;
    (*stack).Context = context;
    (*stack).Control = 0;
    if on_success {
        (*stack).Control |= SL_INVOKE_ON_SUCCESS;
    }
    if on_error {
        (*stack).Control |= SL_INVOKE_ON_ERROR;
    }
    if on_cancel {
        (*stack).Control |= SL_INVOKE_ON_CANCEL;
    }
}

unsafe fn PtrToUlong(p: PVOID) -> ULONG {
    p as usize as ULONG
}

unsafe fn UlongToPtr(u: ULONG) -> PVOID {
    u as usize as PVOID
}

unsafe extern "system" fn IopFreeIrpKernelApc(
    Apc: PKAPC,
    _NormalRoutine: *mut PKNORMAL_ROUTINE,
    _NormalContext: *mut PVOID,
    _SystemArgument1: *mut PVOID,
    _SystemArgument2: *mut PVOID,
) {
    IoFreeIrp(IRP_FROM_APC(Apc));
}

unsafe extern "system" fn IopAbortIrpKernelApc(Apc: PKAPC) {
    IoFreeIrp(IRP_FROM_APC(Apc));
}

#[no_mangle]
pub unsafe extern "system" fn IopCleanupFailedIrp(
    FileObject: PFILE_OBJECT,
    EventObject: PKEVENT,
    Buffer: PVOID,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let EventObject = opaque_ptr(EventObject);
    let Buffer = opaque_ptr(Buffer);
    if !EventObject.is_null() {
        ObDereferenceObject(EventObject as PVOID);
    }

    if !Buffer.is_null() {
        ExFreePool(Buffer);
    }

    if !FileObject.is_null() && ((*FileObject).Flags & FO_SYNCHRONOUS_IO) != 0 {
        IopUnlockFileObject(FileObject);
    }

    if !FileObject.is_null() {
        ObDereferenceObject(FileObject as PVOID);
    }
    STATUS_INSUFFICIENT_RESOURCES
}

#[no_mangle]
pub unsafe extern "system" fn IopAbortInterruptedIrp(EventObject: PKEVENT, Irp: PIRP) {
    let EventObject = opaque_ptr(EventObject);
    let Irp = opaque_ptr(Irp);
    let mut OldIrql: KIRQL = 0;
    let CancelResult: BOOLEAN;
    let mut Wait = LARGE_INTEGER { QuadPart: -100000 };

    KeRaiseIrql(APC_LEVEL, addr_of_mut!(OldIrql));

    if KeReadStateEvent(EventObject) == 0 {
        CancelResult = IoCancelIrp(Irp);
        KeLowerIrql(OldIrql);

        if CancelResult != 0 {
            while KeReadStateEvent(EventObject) == 0 {
                Wait.QuadPart = -100000;
                KeDelayExecutionThread(0i8, 0u8, addr_of_mut!(Wait) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER);
            }
        } else {
            KeWaitForSingleObject(
                EventObject as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                null_mut(),
            );
        }
    } else {
        KeLowerIrql(OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopDisassociateThreadIrp() {
    let mut OldIrql: KIRQL = 0;
    let LockIrql: KIRQL;
    let IrpThread: PETHREAD;
    let IrpEntry: PLIST_ENTRY;
    let mut ErrorLogEntry: PIO_ERROR_LOG_PACKET;
    let mut DeviceObject: PDEVICE_OBJECT = null_mut();
    let IoStackLocation: PIO_STACK_LOCATION;

    KeRaiseIrql(APC_LEVEL, addr_of_mut!(OldIrql));

    IrpThread = PsGetCurrentThread();
    if IsListEmpty(addr_of_mut!((*IrpThread).IrpList)) {
        KeLowerIrql(OldIrql);
        return;
    }

    LockIrql = KeAcquireQueuedSpinLock(LockQueueIoCompletionLock);

    IrpEntry = (*IrpThread).IrpList.Flink;
    IopDeadIrp = CONTAINING_RECORD(
        IrpEntry as *mut c_void,
        core::mem::offset_of!(_IRP, ThreadListEntry),
    ) as PIRP;

    if (*IopDeadIrp).CurrentLocation == ((*IopDeadIrp).StackCount + 2) as i8 {
        KeReleaseQueuedSpinLock(LockQueueIoCompletionLock, LockIrql);
        KeLowerIrql(OldIrql);
        return;
    }

    (*IopDeadIrp).Tail.Overlay.Thread = null_mut();
    RemoveHeadList(addr_of_mut!((*IrpThread).IrpList));
    InitializeListHead(addr_of_mut!((*IopDeadIrp).ThreadListEntry));

    IoStackLocation = IoGetCurrentIrpStackLocation(IopDeadIrp);
    if (*IopDeadIrp).CurrentLocation <= (*IopDeadIrp).StackCount as i8 {
        DeviceObject = (*IoStackLocation).DeviceObject;
    }

    KeReleaseQueuedSpinLock(LockQueueIoCompletionLock, LockIrql);
    KeLowerIrql(OldIrql);

    if !DeviceObject.is_null() {
        ErrorLogEntry = IoAllocateErrorLogEntry(
            DeviceObject as PVOID,
            size_of::<_IO_ERROR_LOG_PACKET>() as UCHAR,
        ) as PIO_ERROR_LOG_PACKET;
        if !ErrorLogEntry.is_null() {
            (*ErrorLogEntry).ErrorCode = IO_DRIVER_CANCEL_TIMEOUT;
            IoWriteErrorLogEntry(ErrorLogEntry as PVOID);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopCleanupIrp(Irp: PIRP, FileObject: PFILE_OBJECT) {
    let Irp = opaque_ptr(Irp);
    let FileObject = opaque_ptr(FileObject);
    let mut Mdl: PMDL = (*Irp).MdlAddress;

    while !Mdl.is_null() {
        let next = (*Mdl).Next;
        (*Irp).MdlAddress = next;
        IoFreeMdl(Mdl);
        Mdl = next;
    }

    if ((*Irp).Flags & IRP_DEALLOCATE_BUFFER) != 0 {
        ExFreePoolWithTag((*Irp).AssociatedIrp.SystemBuffer, TAG_IOBUF);
    }

    if !(*Irp).UserEvent.is_null()
        && ((*Irp).Flags & IRP_SYNCHRONOUS_API) == 0
        && !FileObject.is_null()
    {
        ObDereferenceObject((*Irp).UserEvent as PVOID);
    }

    if !FileObject.is_null() && ((*Irp).Flags & IRP_CREATE_OPERATION) == 0 {
        ObDereferenceObject(FileObject as PVOID);
    }

    IoFreeIrp(Irp);
}

/// Write the IRP's status to the caller's I/O status block. The C does this inside
/// _SEH2_TRY with an empty handler ("make sure we don't write somewhere invalid"); under
/// DUMMY_PSEH that handler is dead, so a process that freed or unmapped its IOSB before an
/// async completion took the kernel down instead of losing the status. Same fault-tolerant
/// copy as the buffered copy-back above. wow64-plan M15b: a guest IO_STATUS_BLOCK32 gets
/// the 8-byte layout.
unsafe fn IopWriteUserIosb(Irp: PIRP) {
    if ((*Irp).Flags & crate::iofunc::IRP_WOW64_IOSB32) != 0 {
        let mut Iosb32: crate::iofunc::IoStatusBlock32 = core::mem::zeroed();
        Iosb32.Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        Iosb32.Information = (*Irp).IoStatus.Information as u32;
        let _ = ntkrnl_sys::probe::safe_copy_raw(
            (*Irp).UserIosb as *mut core::ffi::c_void,
            &Iosb32 as *const _ as *const core::ffi::c_void,
            core::mem::size_of::<crate::iofunc::IoStatusBlock32>(),
        );
    } else {
        let _ = ntkrnl_sys::probe::safe_copy_raw(
            (*Irp).UserIosb as *mut core::ffi::c_void,
            addr_of!((*Irp).IoStatus) as *const core::ffi::c_void,
            core::mem::size_of_val(&(*Irp).IoStatus),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopCompleteRequest(
    Apc: PKAPC,
    _NormalRoutine: *mut PKNORMAL_ROUTINE,
    _NormalContext: *mut PVOID,
    SystemArgument1: *mut PVOID,
    SystemArgument2: *mut PVOID,
) {
    let _NormalRoutine = opaque_ptr(_NormalRoutine);
    let _NormalContext = opaque_ptr(_NormalContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    IopCompleteRequestInner(Apc, _NormalRoutine, _NormalContext, SystemArgument1, SystemArgument2);
}
#[inline(always)]
unsafe fn IopCompleteRequestInner(
    Apc: PKAPC,
    _NormalRoutine: *mut PKNORMAL_ROUTINE,
    _NormalContext: *mut PVOID,
    SystemArgument1: *mut PVOID,
    SystemArgument2: *mut PVOID,
) {
    let FileObject: PFILE_OBJECT = (*SystemArgument1) as PFILE_OBJECT;
    let Irp: PIRP = IRP_FROM_APC(Apc);
    let mut Mdl: PMDL;
    let mut NextMdl: PMDL;
    let mut Port: PVOID = null_mut();
    let mut Key: PVOID = null_mut();
    let mut SignaledCreateRequest = false;

    if !(*SystemArgument2).is_null()
        && (*Irp).IoStatus.__bindgen_anon_1.Status == STATUS_REPARSE
        && (*Irp).IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT as _
    {
        let ReparseData = (*SystemArgument2) as PREPARSE_DATA_BUFFER;
        IopDoNameTransmogrify(Irp, FileObject, ReparseData);
    }

    if ((*Irp).Flags & IRP_BUFFERED_IO) != 0 {
        // TEMPORARY INSTRUMENT — remove with the fix (approved 2026-09-13).
        //
        // Chasing an intermittent arm64 defect: kmtest `Example`'s IOCTL round-trip
        // (Example.c:39/:40) came back `{0, ""}` while DeviceIoControl returned
        // ERROR_SUCCESS. The handler provably RAN — both the failing and the clean
        // round executed exactly 46 assertions, so its own ok_* calls are in the
        // total — and it did its RtlCopyMemory. So the bytes were lost between the
        // driver and the user buffer, on this path.
        //
        // Source alone rules out the copy FAILING: the only error exit below sets
        // STATUS_ACCESS_VIOLATION, and the caller saw success. What is left is a
        // copy that never happened or copied nothing, which is a question about
        // FLAGS and Information, not about buffer contents — so the instrument
        // reads four scalars already in the IRP and never touches the buffers.
        //
        // Deliberately silent in the normal case: it fires only on a completion
        // that is internally inconsistent about whether it has output for the
        // caller. A quiet boot is the instrument working, not the instrument off.
        //
        // The filter was NARROWED by measuring it, not by reasoning about it. The
        // first version also flagged "produced output but is not flagged to copy it
        // back" (!IRP_INPUT_OPERATION with Information != 0), which sounded equally
        // anomalous and fired 58 times on a clean arm64 boot -- it is just an
        // ordinary buffered WRITE reporting the bytes it consumed, plus
        // STATUS_BUFFER_OVERFLOW completions legitimately returning partial data.
        // The condition below fired ZERO times on that same boot. An instrument
        // whose filter has not been run against a HEALTHY system is an instrument
        // whose silence means nothing.
        {
            let status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            let info = (*Irp).IoStatus.Information as usize;
            let is_input = ((*Irp).Flags & IRP_INPUT_OPERATION) != 0;
            let sysbuf = (*Irp).AssociatedIrp.SystemBuffer;
            // Promised the caller output, succeeded OUTRIGHT, delivered zero bytes.
            //
            // `status == STATUS_SUCCESS`, not `!NT_ERROR(status)`, even though the copy
            // branch below uses the latter. The reported symptom is DeviceIoControl
            // returning ERROR_SUCCESS, and the looser test also admits the statuses that
            // succeed WITHOUT promising delivery -- it left one benign hit per boot,
            // STATUS_USER_APC (0xC0) on an alertable buffered read torn down by
            // PspExitThread. One known-benign hit per boot is worse than none: it teaches
            // the reader to skip the line that matters.
            if status == STATUS_SUCCESS && is_input && info == 0 && !(*Irp).UserBuffer.is_null() {
                crate::dbg_print!(
                    b"[IOBUFCOPY] empty-input flags=%lx status=%lx user=%p sys=%p\n\0",
                    (*Irp).Flags as u64,
                    status as u64,
                    (*Irp).UserBuffer,
                    sysbuf,
                );
            }
        }

        if ((*Irp).Flags & IRP_INPUT_OPERATION) != 0
            && (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_VERIFY_REQUIRED
            && !NT_ERROR((*Irp).IoStatus.__bindgen_anon_1.Status)
        {
            // Fault-tolerant copy-back to the caller's buffer. The C wrapped this in
            // _SEH2_TRY and failed the IRP with the exception code (irp.c:292-304);
            // under DUMMY_PSEH that handler is dead, so a NULL/hostile UserBuffer
            // faulted memcpy+0x40 -> BUGCHECK 0x1E. This is the completion-path member
            // of that cluster: kmtest IoReadWrite issues NtReadFile(Buffer=NULL,
            // Length=32) expecting STATUS_ACCESS_VIOLATION — the range probe at the
            // syscall passes (NULL is below MmUserProbeAddress), so the only thing that
            // ever protected this copy was the SEH.
            if ntkrnl_sys::probe::safe_copy_raw(
                (*Irp).UserBuffer,
                (*Irp).AssociatedIrp.SystemBuffer,
                (*Irp).IoStatus.Information as usize,
            )
            .is_err()
            {
                (*Irp).IoStatus.__bindgen_anon_1.Status =
                    ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION;
            }
        }

        if ((*Irp).Flags & IRP_DEALLOCATE_BUFFER) != 0 {
            ExFreePool((*Irp).AssociatedIrp.SystemBuffer);
        }
    }

    (*Irp).Flags &= !(IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER);

    Mdl = (*Irp).MdlAddress;
    while !Mdl.is_null() {
        NextMdl = (*Mdl).Next;
        IoFreeMdl(Mdl);
        Mdl = NextMdl;
    }
    (*Irp).MdlAddress = null_mut();

    if !NT_ERROR((*Irp).IoStatus.__bindgen_anon_1.Status)
        || ((*Irp).PendingReturned != 0 && !IsIrpSynchronous(Irp, FileObject))
    {
        if !FileObject.is_null() && !(*FileObject).CompletionContext.is_null() {
            Port = (*(*FileObject).CompletionContext).Port;
            Key = (*(*FileObject).CompletionContext).Key;
        }

        if !(*Irp).UserIosb.is_null() {
            IopWriteUserIosb(Irp);
        }

        if !(*Irp).UserEvent.is_null() {
            KeSetEvent((*Irp).UserEvent as PVOID, 0, 0u8);

            if !FileObject.is_null() {
                if ((*Irp).Flags & IRP_SYNCHRONOUS_API) == 0 {
                    ObDereferenceObject((*Irp).UserEvent as PVOID);
                }

                if ((*FileObject).Flags & FO_SYNCHRONOUS_IO) != 0
                    && ((*Irp).Flags & IRP_OB_QUERY_NAME) == 0
                {
                    KeSetEvent(addr_of_mut!((*FileObject).Event) as PVOID, 0, 0u8);
                    (*FileObject).FinalStatus = (*Irp).IoStatus.__bindgen_anon_1.Status;
                }

                if ((*Irp).Flags & IRP_CREATE_OPERATION) != 0 {
                    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
                    SignaledCreateRequest = true;
                }
            }
        } else if !FileObject.is_null() {
            KeSetEvent(addr_of_mut!((*FileObject).Event) as PVOID, 0, 0u8);
            (*FileObject).FinalStatus = (*Irp).IoStatus.__bindgen_anon_1.Status;

            if ((*Irp).Flags & IRP_CREATE_OPERATION) != 0 {
                (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
                SignaledCreateRequest = true;
            }
        }

        if ((*Irp).Flags & IRP_CREATE_OPERATION) == 0 {
            if ((*Irp).Flags & IRP_WRITE_OPERATION) != 0 {
                IopUpdateTransferCount(IopWriteTransfer, (*Irp).IoStatus.Information as ULONG);
            } else if ((*Irp).Flags & IRP_READ_OPERATION) != 0 {
                IopUpdateTransferCount(IopReadTransfer, (*Irp).IoStatus.Information as ULONG);
            } else {
                IopUpdateTransferCount(IopOtherTransfer, (*Irp).IoStatus.Information as ULONG);
            }
        }

        IopUnQueueIrpFromThread(Irp);

        if (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine.is_some() {
            KeInitializeApc(
                addr_of_mut!((*Irp).Tail.Apc),
                &mut (*(PsGetCurrentThread())).Tcb as *mut _ as *mut ntkrnl_sys::ddk::_KTHREAD,
                CurrentApcEnvironment as u32,
                Some(IopFreeIrpKernelApc),
                Some(IopAbortIrpKernelApc),
                core::mem::transmute((*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine),
                (*Irp).RequestorMode,
                (*Irp).Overlay.AsynchronousParameters.UserApcContext,
            );
            KeInsertQueueApc(
                addr_of_mut!((*Irp).Tail.Apc),
                (*Irp).UserIosb as PVOID,
                null_mut(),
                2,
            );
        } else if !Port.is_null() && !(*Irp).Overlay.AsynchronousParameters.UserApcContext.is_null() {
            (*Irp).Tail.CompletionKey = Key;
            (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.PacketType = 0; // IopCompletionPacketIrp
            extern "system" {
    fn KeInsertQueue(Queue: PVOID, Entry: PLIST_ENTRY) -> LONG;
}
            KeInsertQueue(Port, addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry));
        } else {
            IoFreeIrp(Irp);
        }

        if !FileObject.is_null() && !SignaledCreateRequest {
            ObDereferenceObjectDeferDelete(FileObject as PVOID);
        }
    } else {
        if (*Irp).PendingReturned != 0 && !FileObject.is_null() {
            if ((*Irp).Flags & IRP_SYNCHRONOUS_API) != 0 {
                IopWriteUserIosb(Irp);

                if !(*Irp).UserEvent.is_null() {
                    KeSetEvent((*Irp).UserEvent as PVOID, 0, 0u8);
                } else {
                    KeSetEvent(addr_of_mut!((*FileObject).Event) as PVOID, 0, 0u8);
                }
            } else {
                (*FileObject).FinalStatus = (*Irp).IoStatus.__bindgen_anon_1.Status;
                KeSetEvent(addr_of_mut!((*FileObject).Event) as PVOID, 0, 0u8);
            }
        }

        if !FileObject.is_null() && ((*Irp).Flags & IRP_CREATE_OPERATION) == 0 {
            ObDereferenceObjectDeferDelete(FileObject as PVOID);
        }

        if !(*Irp).UserEvent.is_null()
            && !FileObject.is_null()
            && ((*Irp).Flags & IRP_SYNCHRONOUS_API) == 0
        {
            ObDereferenceObject((*Irp).UserEvent as PVOID);
        }

        IopUnQueueIrpFromThread(Irp);
        IoFreeIrp(Irp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopInitializeReserveIrp(ReserveIrpAllocator: *mut RESERVE_IRP_ALLOCATOR) -> BOOLEAN {
    let ReserveIrpAllocator = opaque_ptr(ReserveIrpAllocator);
    (*ReserveIrpAllocator).StackSize = 20;

    (*ReserveIrpAllocator).ReserveIrp = IoAllocateIrp((*ReserveIrpAllocator).StackSize, 0u8);
    if (*ReserveIrpAllocator).ReserveIrp.is_null() {
        return 0u8;
    }

    (*ReserveIrpAllocator).ReserveIrpInUse = 0;
    KeInitializeEvent(
        addr_of_mut!((*ReserveIrpAllocator).WaitEvent),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn IopAllocateReserveIrp(StackSize: CCHAR) -> PIRP {
    if StackSize > IopReserveIrpAllocator.StackSize {
        return null_mut();
    }

    while InterlockedExchange(addr_of_mut!(IopReserveIrpAllocator.ReserveIrpInUse), 1) == 1 {
        KeWaitForSingleObject(
            addr_of_mut!(IopReserveIrpAllocator.WaitEvent) as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            null_mut(),
        );
    }

    IoInitializeIrp(
        IopReserveIrpAllocator.ReserveIrp,
        IoSizeOfIrp(StackSize),
        StackSize,
    );

    IopReserveIrpAllocator.ReserveIrp
}

unsafe fn IopFreeReserveIrp(PriorityBoost: CCHAR) {
    InterlockedExchange(addr_of_mut!(IopReserveIrpAllocator.ReserveIrpInUse), 0);
    KeSetEvent(
        addr_of_mut!(IopReserveIrpAllocator.WaitEvent) as PVOID,
        PriorityBoost as i32,
        0u8,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IoAllocateIrp(StackSize: CCHAR, ChargeQuota: BOOLEAN) -> PIRP {
    let mut Irp: PIRP = null_mut();
    let mut Size: USHORT = IoSizeOfIrp(StackSize);
    let Prcb: PKPRCB;
    let mut Flags: UCHAR = 0;
    let mut List: PNPAGED_LOOKASIDE_LIST = null_mut();
    let mut ListType: PP_NPAGED_LOOKASIDE_NUMBER = _PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList;

    if ChargeQuota != 0 {
        Flags |= IRP_QUOTA_CHARGED;
    }

    Prcb = KeGetCurrentPrcb();

    if StackSize <= 8 && (ChargeQuota == 0 || (*Prcb).LookasideIrpFloat > 0) {
        Flags |= IRP_ALLOCATED_FIXED_SIZE;

        if StackSize != 1 {
            Size = IoSizeOfIrp(8);
            ListType = _PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList;
        }

        List = (*Prcb).PPLookasideList[ListType as usize].P as PNPAGED_LOOKASIDE_LIST;
        (*List).L.TotalAllocates = (*List).L.TotalAllocates.wrapping_add(1);
        Irp = InterlockedPopEntrySList(addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) as PIRP;

        if Irp.is_null() {
            (*List).L.__bindgen_anon_2.AllocateMisses = (*List).L.__bindgen_anon_2.AllocateMisses.wrapping_add(1);
            List = (*Prcb).PPLookasideList[ListType as usize].L as PNPAGED_LOOKASIDE_LIST;
            (*List).L.TotalAllocates = (*List).L.TotalAllocates.wrapping_add(1);
            Irp = InterlockedPopEntrySList(addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) as PIRP;
        }
    }

    if Irp.is_null() {
        if (Flags & IRP_ALLOCATED_FIXED_SIZE) != 0 {
            (*List).L.__bindgen_anon_2.AllocateMisses = (*List).L.__bindgen_anon_2.AllocateMisses.wrapping_add(1);
        }

        if ChargeQuota != 0 {
            extern "system" {
    fn ExAllocatePoolWithQuotaTag( PoolType: ULONG, NumberOfBytes: SIZE_T, Tag: ULONG, ) -> PVOID;
}
            Irp = ExAllocatePoolWithQuotaTag(
                NonPagedPool | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE,
                Size as SIZE_T,
                TAG_IRP,
            ) as PIRP;
        } else {
            Irp = ExAllocatePoolWithTag(NonPagedPool, Size as SIZE_T, TAG_IRP) as PIRP;
        }

        if Irp.is_null() {
            return null_mut();
        }
    } else if (Flags & IRP_QUOTA_CHARGED) != 0 {
        InterlockedDecrement(addr_of_mut!((*Prcb).LookasideIrpFloat));
        Flags |= IRP_LOOKASIDE_ALLOCATION;
        Flags &= !IRP_QUOTA_CHARGED;
    }

    IoInitializeIrp(Irp, Size, StackSize);
    (*Irp).AllocationFlags = Flags;

    Irp
}

#[no_mangle]
pub unsafe extern "system" fn IopAllocateIrpMustSucceed(StackSize: CCHAR) -> PIRP {
    let mut i: LONG = LONG::MAX;
    let mut Irp: PIRP;
    let mut Sleep = LARGE_INTEGER { QuadPart: -10 * 1000 * 10 };

    Irp = IoAllocateIrp(StackSize, 0u8);
    if !Irp.is_null() {
        return Irp;
    }

    loop {
        i -= 1;
        if i <= 0 {
            break;
        }

        Sleep.QuadPart = -10 * 1000 * 10;
        KeDelayExecutionThread(0i8, 0u8, addr_of_mut!(Sleep) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER);

        Irp = IoAllocateIrp(StackSize, 0u8);
        if !Irp.is_null() {
            return Irp;
        }
    }

    Irp
}

#[no_mangle]
pub unsafe extern "system" fn IoBuildAsynchronousFsdRequest(
    MajorFunction: ULONG,
    DeviceObject: PDEVICE_OBJECT,
    Buffer: PVOID,
    Length: ULONG,
    StartingOffset: *mut LARGE_INTEGER,
    IoStatusBlock: *mut _IO_STATUS_BLOCK,
) -> PIRP {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Buffer = opaque_ptr(Buffer);
    let StartingOffset = opaque_ptr(StartingOffset);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Irp: PIRP;
    let StackPtr: PIO_STACK_LOCATION;

    Irp = IoAllocateIrp((*DeviceObject).StackSize, 0u8);
    if Irp.is_null() {
        return null_mut();
    }

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = MajorFunction as UCHAR;

    if MajorFunction != IRP_MJ_FLUSH_BUFFERS
        && MajorFunction != IRP_MJ_SHUTDOWN
        && MajorFunction != IRP_MJ_PNP
        && MajorFunction != IRP_MJ_POWER
    {
        if ((*DeviceObject).Flags & DO_BUFFERED_IO) != 0 {
            (*Irp).AssociatedIrp.SystemBuffer =
                ExAllocatePoolWithTag(NonPagedPool, Length as SIZE_T, TAG_IOBUF);
            if (*Irp).AssociatedIrp.SystemBuffer.is_null() {
                IoFreeIrp(Irp);
                return null_mut();
            }

            (*Irp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;

            if MajorFunction == IRP_MJ_WRITE {
                core::ptr::copy_nonoverlapping(
                    Buffer as *const u8,
                    (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
                    Length as usize,
                );
            } else {
                (*Irp).Flags |= IRP_INPUT_OPERATION;
                (*Irp).UserBuffer = Buffer;
            }
        } else if ((*DeviceObject).Flags & DO_DIRECT_IO) != 0 {
            (*Irp).MdlAddress = IoAllocateMdl(Buffer, Length, 0u8, 0u8, null_mut());
            if (*Irp).MdlAddress.is_null() {
                IoFreeIrp(Irp);
                return null_mut();
            }

            MmProbeAndLockPages(
                (*Irp).MdlAddress,
                0i8,
                if MajorFunction == IRP_MJ_READ {
                    IoWriteAccess
                } else {
                    IoReadAccess
                },
            );
        } else {
            (*Irp).UserBuffer = Buffer;
        }

        if MajorFunction == IRP_MJ_READ {
            (*StackPtr).Parameters.Read.Length = Length;
            (*StackPtr).Parameters.Read.ByteOffset = *(StartingOffset as *const ntkrnl_sys::ddk::_LARGE_INTEGER);
        } else if MajorFunction == IRP_MJ_WRITE {
            (*StackPtr).Parameters.Write.Length = Length;
            (*StackPtr).Parameters.Write.ByteOffset = *(StartingOffset as *const ntkrnl_sys::ddk::_LARGE_INTEGER);
        }
    }

    (*Irp).UserIosb = IoStatusBlock;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();

    Irp
}

#[no_mangle]
pub unsafe extern "system" fn IoBuildDeviceIoControlRequest(
    IoControlCode: ULONG,
    DeviceObject: PDEVICE_OBJECT,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    InternalDeviceIoControl: BOOLEAN,
    Event: PKEVENT,
    IoStatusBlock: *mut _IO_STATUS_BLOCK,
) -> PIRP {
    let DeviceObject = opaque_ptr(DeviceObject);
    let InputBuffer = opaque_ptr(InputBuffer);
    let OutputBuffer = opaque_ptr(OutputBuffer);
    let Event = opaque_ptr(Event);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Irp: PIRP;
    let StackPtr: PIO_STACK_LOCATION;
    let BufferLength: ULONG;

    Irp = IoAllocateIrp((*DeviceObject).StackSize, 0u8);
    if Irp.is_null() {
        return null_mut();
    }

    StackPtr = IoGetNextIrpStackLocation(Irp);

    (*StackPtr).MajorFunction = if InternalDeviceIoControl != 0 {
        IRP_MJ_INTERNAL_DEVICE_CONTROL as UCHAR
    } else {
        IRP_MJ_DEVICE_CONTROL as UCHAR
    };

    (*StackPtr).Parameters.DeviceIoControl.IoControlCode = IoControlCode;
    (*StackPtr).Parameters.DeviceIoControl.InputBufferLength = InputBufferLength;
    (*StackPtr).Parameters.DeviceIoControl.OutputBufferLength = OutputBufferLength;

    match IO_METHOD_FROM_CTL_CODE(IoControlCode) {
        METHOD_BUFFERED => {
            BufferLength = if InputBufferLength > OutputBufferLength {
                InputBufferLength
            } else {
                OutputBufferLength
            };

            if BufferLength != 0 {
                (*Irp).AssociatedIrp.SystemBuffer =
                    ExAllocatePoolWithTag(NonPagedPool, BufferLength as SIZE_T, TAG_IOBUF);
                if (*Irp).AssociatedIrp.SystemBuffer.is_null() {
                    IoFreeIrp(Irp);
                    return null_mut();
                }

                if !InputBuffer.is_null() {
                    core::ptr::copy_nonoverlapping(
                        InputBuffer as *const u8,
                        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
                        InputBufferLength as usize,
                    );
                }

                (*Irp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
                if !OutputBuffer.is_null() {
                    (*Irp).Flags |= IRP_INPUT_OPERATION;
                }
                (*Irp).UserBuffer = OutputBuffer;
            } else {
                (*Irp).Flags = 0;
                (*Irp).UserBuffer = null_mut();
            }
        }
        METHOD_IN_DIRECT | METHOD_OUT_DIRECT => {
            if !InputBuffer.is_null() {
                (*Irp).AssociatedIrp.SystemBuffer =
                    ExAllocatePoolWithTag(NonPagedPool, InputBufferLength as SIZE_T, TAG_IOBUF);
                if (*Irp).AssociatedIrp.SystemBuffer.is_null() {
                    IoFreeIrp(Irp);
                    return null_mut();
                }

                core::ptr::copy_nonoverlapping(
                    InputBuffer as *const u8,
                    (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
                    InputBufferLength as usize,
                );

                (*Irp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
            } else {
                (*Irp).Flags = 0;
            }

            if !OutputBuffer.is_null() {
                (*Irp).MdlAddress = IoAllocateMdl(OutputBuffer, OutputBufferLength, 0u8, 0u8, Irp);
                if (*Irp).MdlAddress.is_null() {
                    IoFreeIrp(Irp);
                    return null_mut();
                }

                MmProbeAndLockPages(
                    (*Irp).MdlAddress,
                    0i8,
                    if IO_METHOD_FROM_CTL_CODE(IoControlCode) == METHOD_IN_DIRECT {
                        IoReadAccess
                    } else {
                        IoWriteAccess
                    },
                );
            }
        }
        METHOD_NEITHER | _ => {
            (*Irp).UserBuffer = OutputBuffer;
            (*StackPtr).Parameters.DeviceIoControl.Type3InputBuffer = InputBuffer;
        }
    }

    (*Irp).UserIosb = IoStatusBlock;
    (*Irp).UserEvent = Event;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    IoQueueThreadIrp(Irp);

    Irp
}

#[no_mangle]
pub unsafe extern "system" fn IoBuildSynchronousFsdRequest(
    MajorFunction: ULONG,
    DeviceObject: PDEVICE_OBJECT,
    Buffer: PVOID,
    Length: ULONG,
    StartingOffset: *mut LARGE_INTEGER,
    Event: PKEVENT,
    IoStatusBlock: *mut _IO_STATUS_BLOCK,
) -> PIRP {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Buffer = opaque_ptr(Buffer);
    let StartingOffset = opaque_ptr(StartingOffset);
    let Event = opaque_ptr(Event);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Irp: PIRP = IoBuildAsynchronousFsdRequest(
        MajorFunction,
        DeviceObject,
        Buffer,
        Length,
        StartingOffset,
        IoStatusBlock,
    );
    if Irp.is_null() {
        return null_mut();
    }

    (*Irp).UserEvent = Event;
    IoQueueThreadIrp(Irp);
    Irp
}

#[no_mangle]
pub unsafe extern "system" fn IoCancelIrp(Irp: PIRP) -> BOOLEAN {
    let Irp = opaque_ptr(Irp);
    let mut OldIrql: KIRQL = 0;
    let CancelRoutine: PDRIVER_CANCEL;

    IoAcquireCancelSpinLock(addr_of_mut!(OldIrql));
    (*Irp).Cancel = 1u8;

    CancelRoutine = IoSetCancelRoutine(Irp, None);
    if CancelRoutine.is_some() {
        if (*Irp).CurrentLocation > ((*Irp).StackCount + 1) as i8 {
            KeBugCheckEx(
                CANCEL_STATE_IN_COMPLETED_IRP,
                Irp as usize,
                CancelRoutine.unwrap() as usize,
                0,
                0,
            );
        }

        (*Irp).CancelIrql = OldIrql;
        (CancelRoutine.unwrap())(
            (*IoGetCurrentIrpStackLocation(Irp)).DeviceObject,
            Irp,
        );
        return 1u8;
    }

    IoReleaseCancelSpinLock(OldIrql);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn IoCancelThreadIo(Thread: PETHREAD) {
    let Thread = opaque_ptr(Thread);
    let mut OldIrql: KIRQL = 0;
    let mut Retries: ULONG = 3000;
    let mut Interval = LARGE_INTEGER { QuadPart: -1000000 };
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let Irp: PIRP;

    let Thread = PsGetCurrentThread();

    KeRaiseIrql(APC_LEVEL, addr_of_mut!(OldIrql));

    ListHead = addr_of_mut!((*Thread).IrpList);
    NextEntry = (*ListHead).Flink;
    while ListHead != NextEntry {
        let Irp = CONTAINING_RECORD(
            NextEntry as *mut c_void,
            core::mem::offset_of!(_IRP, ThreadListEntry),
        ) as PIRP;

        IoCancelIrp(Irp);
        NextEntry = (*NextEntry).Flink;
    }

    Interval.QuadPart = -1000000;

    while !IsListEmpty(addr_of_mut!((*Thread).IrpList)) {
        KeLowerIrql(OldIrql);
        KeDelayExecutionThread(0i8, 0u8, addr_of_mut!(Interval) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER);

        if Retries == 0 {
            IopDisassociateThreadIrp();
        } else {
            Retries -= 1;
        }

        KeRaiseIrql(APC_LEVEL, addr_of_mut!(OldIrql));
    }

    KeLowerIrql(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    IofCallDriver(DeviceObject, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn IoCompleteRequest(Irp: PIRP, PriorityBoost: CCHAR) {
    let Irp = opaque_ptr(Irp);
    IofCompleteRequest(Irp, PriorityBoost);
}

#[no_mangle]
pub unsafe extern "system" fn IoEnqueueIrp(Irp: PIRP) {
    let Irp = opaque_ptr(Irp);
    IoQueueThreadIrp(Irp);
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IofCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    IofCallDriverInner(DeviceObject, Irp)
}
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IofCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    IofCallDriverInner(DeviceObject, Irp)
}
#[inline(always)]
unsafe fn IofCallDriverInner(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DriverObject: PDRIVER_OBJECT;
    let StackPtr: PIO_STACK_LOCATION;

    DriverObject = (*DeviceObject).DriverObject;

    (*Irp).CurrentLocation -= 1;
    if (*Irp).CurrentLocation <= 0 {
        KeBugCheckEx(NO_MORE_IRP_STACK_LOCATIONS, Irp as usize, 0, 0, 0);
    }

    StackPtr = IoGetNextIrpStackLocation(Irp);
    set_current_stack_location(Irp, StackPtr);
    (*StackPtr).DeviceObject = DeviceObject;

    ((*DriverObject).MajorFunction[(*StackPtr).MajorFunction as usize].unwrap())(DeviceObject, Irp)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IofCompleteRequest(Irp: PIRP, PriorityBoost: CCHAR) {
    let Irp = opaque_ptr(Irp);
    IofCompleteRequestInner(Irp, PriorityBoost);
}
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IofCompleteRequest(Irp: PIRP, PriorityBoost: CCHAR) {
    let Irp = opaque_ptr(Irp);
    IofCompleteRequestInner(Irp, PriorityBoost);
}
#[inline(always)]
unsafe fn IofCompleteRequestInner(Irp: PIRP, PriorityBoost: CCHAR) {
    let mut StackPtr: PIO_STACK_LOCATION;
    let LastStackPtr: PIO_STACK_LOCATION;
    let mut DeviceObject: PDEVICE_OBJECT;
    let FileObject: PFILE_OBJECT;
    let Thread: PETHREAD;
    let mut Status: NTSTATUS;
    let mut Mdl: PMDL;
    let mut NextMdl: PMDL;
    let MasterCount: ULONG;
    let MasterIrp: PIRP;
    let Flags: ULONG;
    let mut ErrorCode: NTSTATUS = STATUS_SUCCESS;
    let mut DataBuffer: PREPARSE_DATA_BUFFER = null_mut();

    if (*Irp).CurrentLocation > ((*Irp).StackCount + 1) as i8 {
        KeBugCheckEx(MULTIPLE_IRP_COMPLETE_REQUESTS, Irp as usize, 0, 0, 0);
    }

    LastStackPtr = first_stack_location(Irp);
    if ((*LastStackPtr).Control & SL_ERROR_RETURNED) != 0 {
        ErrorCode = PtrToUlong((*LastStackPtr).Parameters.Others.Argument4) as NTSTATUS;
    }

    StackPtr = IoGetCurrentIrpStackLocation(Irp);
    (*Irp).CurrentLocation += 1;
    set_current_stack_location(Irp, stack_location_add(get_current_stack_location(Irp), 1));

    while (*Irp).CurrentLocation <= ((*Irp).StackCount + 1) as i8 {
        (*Irp).PendingReturned = if ((*StackPtr).Control & SL_PENDING_RETURNED) != 0 { 1u8 } else { 0u8 };

        if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
            if (*Irp).IoStatus.__bindgen_anon_1.Status != ErrorCode {
                ErrorCode = (*Irp).IoStatus.__bindgen_anon_1.Status;
                (*StackPtr).Control |= SL_ERROR_RETURNED;
                (*LastStackPtr).Parameters.Others.Argument4 = UlongToPtr(ErrorCode as ULONG);
                (*LastStackPtr).Control |= SL_ERROR_RETURNED;
            }
        }

        if (NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) && ((*StackPtr).Control & SL_INVOKE_ON_SUCCESS) != 0)
            || (!NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) && ((*StackPtr).Control & SL_INVOKE_ON_ERROR) != 0)
            || ((*Irp).Cancel != 0 && ((*StackPtr).Control & SL_INVOKE_ON_CANCEL) != 0)
        {
            IopClearStackLocation(StackPtr);

            if (*Irp).CurrentLocation == ((*Irp).StackCount + 1) as i8 {
                DeviceObject = null_mut();
            } else {
                DeviceObject = (*IoGetCurrentIrpStackLocation(Irp)).DeviceObject;
            }

            Status = ((*StackPtr).CompletionRoutine.unwrap())(DeviceObject, Irp, (*StackPtr).Context);

            if Status == STATUS_MORE_PROCESSING_REQUIRED {
                return;
            }
        } else {
            if (*Irp).CurrentLocation <= (*Irp).StackCount as i8 && (*Irp).PendingReturned != 0 {
                IoMarkIrpPending(Irp);
            }
            IopClearStackLocation(StackPtr);
        }

        StackPtr = stack_location_add(StackPtr, 1);
        (*Irp).CurrentLocation += 1;
        set_current_stack_location(Irp, stack_location_add(get_current_stack_location(Irp), 1));
    }

    if ((*Irp).Flags & IRP_ASSOCIATED_IRP) != 0 {
        MasterIrp = (*Irp).AssociatedIrp.MasterIrp;
        let MasterCount = InterlockedDecrement(addr_of_mut!((*MasterIrp).AssociatedIrp.IrpCount));

        Mdl = (*Irp).MdlAddress;
        while !Mdl.is_null() {
            NextMdl = (*Mdl).Next;
            IoFreeMdl(Mdl);
            Mdl = NextMdl;
        }

        IoFreeIrp(Irp);

        if MasterCount == 0 {
            IofCompleteRequest(MasterIrp, PriorityBoost);
        }
        return;
    }

    if (*Irp).IoStatus.__bindgen_anon_1.Status == STATUS_REPARSE {
        if (*Irp).IoStatus.Information > IO_REMOUNT as _ {
            if (*Irp).IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT as _ {
                DataBuffer = (*Irp).Tail.Overlay.AuxiliaryBuffer as PREPARSE_DATA_BUFFER;
                (*Irp).Tail.Overlay.AuxiliaryBuffer = null_mut();
            } else {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_REPARSE_TAG_NOT_HANDLED;
            }
        }
    }

    if !(*Irp).Tail.Overlay.AuxiliaryBuffer.is_null() {
        ExFreePool((*Irp).Tail.Overlay.AuxiliaryBuffer as PVOID);
        (*Irp).Tail.Overlay.AuxiliaryBuffer = null_mut();
    }

    if ((*Irp).Flags & (IRP_PAGING_IO | IRP_CLOSE_OPERATION)) != 0 {
        if ((*Irp).Flags & (IRP_SYNCHRONOUS_PAGING_IO | IRP_CLOSE_OPERATION)) != 0 {
            Flags = (*Irp).Flags & (IRP_SYNCHRONOUS_PAGING_IO | IRP_PAGING_IO);
            *(*Irp).UserIosb = (*Irp).IoStatus;
            KeSetEvent((*Irp).UserEvent as PVOID, PriorityBoost as i32, 0u8);

            if Flags != 0 {
                if Irp == IopReserveIrpAllocator.ReserveIrp {
                    IopFreeReserveIrp(PriorityBoost);
                } else {
                    IoFreeIrp(Irp);
                }
            }
        }
        return;
    }

    Mdl = (*Irp).MdlAddress;
    while !Mdl.is_null() {
        MmUnlockPages(Mdl);
        Mdl = (*Mdl).Next;
    }

    if ((*Irp).Flags & IRP_DEFER_IO_COMPLETION) != 0 && (*Irp).PendingReturned == 0 {
        if (*Irp).IoStatus.__bindgen_anon_1.Status == STATUS_REPARSE
            && (*Irp).IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT as _
        {
            (*Irp).Tail.Overlay.AuxiliaryBuffer = DataBuffer as *mut i8;
        }
        return;
    }

    Thread = (*Irp).Tail.Overlay.Thread;
    FileObject = (*Irp).Tail.Overlay.OriginalFileObject;

    if (*Irp).Cancel == 0 {
        KeInitializeApc(
            addr_of_mut!((*Irp).Tail.Apc),
            &mut (*Thread).Tcb as *mut _ as *mut ntkrnl_sys::ddk::_KTHREAD,
            (*Irp).ApcEnvironment as u32,
            Some(IopCompleteRequest),
            None,
            None,
            0i8,
            null_mut(),
        );
        KeInsertQueueApc(
            addr_of_mut!((*Irp).Tail.Apc),
            FileObject as PVOID,
            DataBuffer as PVOID,
            PriorityBoost as i32,
        );
    } else {
        if !Thread.is_null() {
            KeInitializeApc(
                addr_of_mut!((*Irp).Tail.Apc),
                &mut (*Thread).Tcb as *mut _ as *mut ntkrnl_sys::ddk::_KTHREAD,
                (*Irp).ApcEnvironment as u32,
                Some(IopCompleteRequest),
                None,
                None,
                0i8,
                null_mut(),
            );
            KeInsertQueueApc(
                addr_of_mut!((*Irp).Tail.Apc),
                FileObject as PVOID,
                DataBuffer as PVOID,
                PriorityBoost as i32,
            );
        } else {
            IopCleanupIrp(Irp, FileObject);
        }
    }
}

unsafe extern "system" fn IopSynchronousCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    if (*Irp).PendingReturned != 0 {
        KeSetEvent(Context as PVOID, IO_NO_INCREMENT, 0u8);
    }
    STATUS_MORE_PROCESSING_REQUIRED
}

#[no_mangle]
pub unsafe extern "system" fn IoForwardIrpSynchronously(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> BOOLEAN {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let mut Event: _KEVENT = zeroed();
    let mut Status: NTSTATUS;

    if (*Irp).CurrentLocation > (*Irp).StackCount as i8 || (*Irp).CurrentLocation <= 1 {
        return 0u8;
    }

    KeInitializeEvent(addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
    IoCopyCurrentIrpStackLocationToNext(Irp);
    IoSetCompletionRoutine(Irp, Some(IopSynchronousCompletion), addr_of_mut!(Event) as PVOID, true, true, true);

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Suspended,
            0i8,
            0u8,
            null_mut(),
        );
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn IoFreeIrp(Irp: PIRP) {
    let Irp = opaque_ptr(Irp);
    let mut List: PNPAGED_LOOKASIDE_LIST;
    let mut ListType: PP_NPAGED_LOOKASIDE_NUMBER = _PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList;
    let Prcb: PKPRCB;

    Prcb = KeGetCurrentPrcb();

    if ((*Irp).AllocationFlags & IRP_LOOKASIDE_ALLOCATION) != 0 {
        (*Irp).AllocationFlags &= !IRP_LOOKASIDE_ALLOCATION;
        InterlockedIncrement(addr_of_mut!((*Prcb).LookasideIrpFloat));
    }

    if ((*Irp).AllocationFlags & IRP_ALLOCATED_FIXED_SIZE) == 0 {
        ExFreePoolWithTag(Irp as PVOID, TAG_IRP);
    } else {
        if (*Irp).StackCount != 1 {
            ListType = _PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList;
        }

        List = (*Prcb).PPLookasideList[ListType as usize].P as PNPAGED_LOOKASIDE_LIST;
        (*List).L.TotalFrees = (*List).L.TotalFrees.wrapping_add(1);

        if ExQueryDepthSList(addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) >= (*List).L.Depth {
            (*List).L.__bindgen_anon_3.FreeMisses = (*List).L.__bindgen_anon_3.FreeMisses.wrapping_add(1);
            List = (*Prcb).PPLookasideList[ListType as usize].L as PNPAGED_LOOKASIDE_LIST;
            (*List).L.TotalFrees = (*List).L.TotalFrees.wrapping_add(1);

            if ExQueryDepthSList(addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) >= (*List).L.Depth {
                (*List).L.__bindgen_anon_3.FreeMisses = (*List).L.__bindgen_anon_3.FreeMisses.wrapping_add(1);
                ExFreePoolWithTag(Irp as PVOID, TAG_IRP);
                return;
            }
        }

        if ((*Irp).AllocationFlags & IRP_QUOTA_CHARGED) != 0 {
            extern "system" {
    fn ExReturnPoolQuota(P: PVOID);
}
            ExReturnPoolQuota(Irp as PVOID);
            (*Irp).AllocationFlags &= !IRP_QUOTA_CHARGED;
        }

        InterlockedPushEntrySList(
            addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER,
            Irp as PSLIST_ENTRY,
        );
    }
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IoGetPagingIoPriority(Irp: PIRP) -> IO_PAGING_PRIORITY {
    let Irp = opaque_ptr(Irp);
    IoGetPagingIoPriorityInner(Irp)
}
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IoGetPagingIoPriority(Irp: PIRP) -> IO_PAGING_PRIORITY {
    let Irp = opaque_ptr(Irp);
    IoGetPagingIoPriorityInner(Irp)
}
#[inline(always)]
unsafe fn IoGetPagingIoPriorityInner(Irp: PIRP) -> IO_PAGING_PRIORITY {
    let Priority: IO_PAGING_PRIORITY;
    let Flags: ULONG = (*Irp).Flags;

    if (Flags & IRP_CLASS_CACHE_OPERATION) != 0 {
        Priority = IoPagingPriorityHigh;
    } else if (Flags & IRP_PAGING_IO) != 0 {
        Priority = IoPagingPriorityNormal;
    } else {
        Priority = IoPagingPriorityInvalid;
    }

    Priority
}

#[no_mangle]
pub unsafe extern "system" fn IoGetRequestorProcess(Irp: PIRP) -> PEPROCESS {
    let Irp = opaque_ptr(Irp);
    if !(*Irp).Tail.Overlay.Thread.is_null() {
        if (*Irp).ApcEnvironment as u32 == OriginalApcEnvironment {
            return (*(*Irp).Tail.Overlay.Thread).ThreadsProcess;
        } else if (*Irp).ApcEnvironment as u32 == AttachedApcEnvironment {
            return (*(*Irp).Tail.Overlay.Thread).Tcb.__bindgen_anon_1.ApcState.Process as PEPROCESS;
        }
    }

    null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn IoGetRequestorProcessId(Irp: PIRP) -> ULONG {
    let Irp = opaque_ptr(Irp);
    let Process: PEPROCESS = IoGetRequestorProcess(Irp);
    if !Process.is_null() {
        return PtrToUlong((*Process).UniqueProcessId);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn IoGetRequestorSessionId(
    Irp: PIRP,
    pSessionId: *mut ULONG,
) -> NTSTATUS {
    let Irp = opaque_ptr(Irp);
    let pSessionId = opaque_ptr(pSessionId);
    if !(*Irp).Tail.Overlay.Thread.is_null() {
        let Process = (*(*Irp).Tail.Overlay.Thread).ThreadsProcess;
        *pSessionId = MmGetSessionId(Process);
        return STATUS_SUCCESS;
    }

    *pSessionId = (-1i32) as ULONG;
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn IoGetTopLevelIrp() -> PIRP {
    (*(PsGetCurrentThread() as *mut _ETHREAD)).TopLevelIrp as PIRP
}

#[no_mangle]
pub unsafe extern "system" fn IoInitializeIrp(Irp: PIRP, PacketSize: USHORT, StackSize: CCHAR) {
    let Irp = opaque_ptr(Irp);
    core::ptr::write_bytes(Irp as *mut u8, 0, PacketSize as usize);

    (*Irp).Type = IO_TYPE_IRP as i16;
    (*Irp).Size = PacketSize;
    (*Irp).StackCount = StackSize;
    (*Irp).CurrentLocation = (StackSize + 1) as CCHAR;
    (*Irp).ApcEnvironment = (*(PsGetCurrentThread() as *mut _KTHREAD)).ApcStateIndex as i8;
    set_current_stack_location(
        Irp,
        stack_location_add(first_stack_location(Irp), StackSize as isize),
    );

    InitializeListHead(addr_of_mut!((*Irp).ThreadListEntry));
}

#[no_mangle]
pub unsafe extern "system" fn IoIsOperationSynchronous(Irp: PIRP) -> BOOLEAN {
    let Irp = opaque_ptr(Irp);
    let SynchIO: bool;
    let ForceAsync: bool;

    SynchIO = ((*(*IoGetCurrentIrpStackLocation(Irp)).FileObject).Flags & FO_SYNCHRONOUS_IO) != 0
        || ((*Irp).Flags & IRP_SYNCHRONOUS_API) != 0
        || ((*Irp).Flags & IRP_SYNCHRONOUS_PAGING_IO) != 0;

    ForceAsync = ((*Irp).Flags & IRP_PAGING_IO) != 0
        && ((*Irp).Flags & IRP_SYNCHRONOUS_PAGING_IO) == 0;

    if SynchIO && !ForceAsync {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoIsValidNameGraftingBuffer(
    _Irp: PIRP,
    _ReparseBuffer: PREPARSE_DATA_BUFFER,
) -> BOOLEAN {
    let _Irp = opaque_ptr(_Irp);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn IoMakeAssociatedIrp(Irp: PIRP, StackSize: CCHAR) -> PIRP {
    let Irp = opaque_ptr(Irp);
    let AssocIrp: PIRP = IoAllocateIrp(StackSize, 0u8);
    if AssocIrp.is_null() {
        return null_mut();
    }

    (*AssocIrp).Flags |= IRP_ASSOCIATED_IRP;
    (*AssocIrp).Tail.Overlay.Thread = (*Irp).Tail.Overlay.Thread;
    (*AssocIrp).AssociatedIrp.MasterIrp = Irp;

    AssocIrp
}

#[no_mangle]
pub unsafe extern "system" fn IoQueueThreadIrp(Irp: PIRP) {
    let Irp = opaque_ptr(Irp);
    IopQueueIrpToThread(Irp);
}

#[no_mangle]
pub unsafe extern "system" fn IoReuseIrp(Irp: PIRP, Status: NTSTATUS) {
    let Irp = opaque_ptr(Irp);
    let AllocationFlags: UCHAR = (*Irp).AllocationFlags;

    IoInitializeIrp(Irp, (*Irp).Size, (*Irp).StackCount as i8);
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).AllocationFlags = AllocationFlags;
}

#[no_mangle]
pub unsafe extern "system" fn IoSetTopLevelIrp(Irp: PIRP) {
    let Irp = opaque_ptr(Irp);
    (*(PsGetCurrentThread() as *mut _ETHREAD)).TopLevelIrp = Irp as usize as ntkrnl_sys::ddk::ULONG_PTR;
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "system" fn IoIs32bitProcess(_Irp: PIRP) -> BOOLEAN {
    let _Irp = opaque_ptr(_Irp);
    0u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/irp.c
//  * PURPOSE:         IRP Handling Functions
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Gunnar Dalsnes
//  *                  Filip Navara (navaraf@reactos.org)
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
//
// /* ARM64 verbose IRP traces removed — IRP completion confirmed working */
// #define _IRP_TRACE2(s) ((void)0)
// #include <debug.h>
//
// PIRP IopDeadIrp;
// RESERVE_IRP_ALLOCATOR IopReserveIrpAllocator;
//
// /* PRIVATE FUNCTIONS  ********************************************************/
//
// VOID
// NTAPI
// IopFreeIrpKernelApc(IN PKAPC Apc,
//                     IN PKNORMAL_ROUTINE *NormalRoutine,
//                     IN PVOID *NormalContext,
//                     IN PVOID *SystemArgument1,
//                     IN PVOID *SystemArgument2)
// {
//     /* Free the IRP */
//     IoFreeIrp(CONTAINING_RECORD(Apc, IRP, Tail.Apc));
// }
//
// VOID
// NTAPI
// IopAbortIrpKernelApc(IN PKAPC Apc)
// {
//     /* Free the IRP */
//     IoFreeIrp(CONTAINING_RECORD(Apc, IRP, Tail.Apc));
// }
//
// NTSTATUS
// NTAPI
// IopCleanupFailedIrp(IN PFILE_OBJECT FileObject,
//                     IN PKEVENT EventObject OPTIONAL,
//                     IN PVOID Buffer OPTIONAL)
// {
//     PAGED_CODE();
//
//     /* Dereference the event */
//     if (EventObject) ObDereferenceObject(EventObject);
//
//     /* Free a buffer, if any */
//     if (Buffer) ExFreePool(Buffer);
//
//     /* If this was a file opened for synch I/O, then unlock it */
//     if (FileObject->Flags & FO_SYNCHRONOUS_IO) IopUnlockFileObject(FileObject);
//
//     /* Now dereference it and return */
//     ObDereferenceObject(FileObject);
//     return STATUS_INSUFFICIENT_RESOURCES;
// }
//
// VOID
// NTAPI
// IopAbortInterruptedIrp(IN PKEVENT EventObject,
//                        IN PIRP Irp)
// {
//     KIRQL OldIrql;
//     BOOLEAN CancelResult;
//     LARGE_INTEGER Wait;
//     PAGED_CODE();
//
//     /* Raise IRQL to APC */
//     KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Check if nobody completed it yet */
//     if (!KeReadStateEvent(EventObject))
//     {
//         /* First, cancel it */
//         CancelResult = IoCancelIrp(Irp);
//         KeLowerIrql(OldIrql);
//
//         /* Check if we cancelled it */
//         if (CancelResult)
//         {
//             /* Wait for the IRP to be cancelled */
//             Wait.QuadPart = -100000;
//             while (!KeReadStateEvent(EventObject))
//             {
//                 /* Delay indefintely */
//                 KeDelayExecutionThread(KernelMode, FALSE, &Wait);
//             }
//         }
//         else
//         {
//             /* No cancellation done, so wait for the I/O system to kill it */
//             KeWaitForSingleObject(EventObject,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//         }
//     }
//     else
//     {
//         /* We got preempted, so give up */
//         KeLowerIrql(OldIrql);
//     }
// }
//
// VOID
// NTAPI
// IopDisassociateThreadIrp(VOID)
// {
//     KIRQL OldIrql, LockIrql;
//     PETHREAD IrpThread;
//     PLIST_ENTRY IrpEntry;
//     PIO_ERROR_LOG_PACKET ErrorLogEntry;
//     PDEVICE_OBJECT DeviceObject = NULL;
//     PIO_STACK_LOCATION IoStackLocation;
//
//     /* First, raise to APC to protect IrpList */
//     KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Get the Thread and check the list */
//     IrpThread = PsGetCurrentThread();
//     if (IsListEmpty(&IrpThread->IrpList))
//     {
//         /* It got completed now, so quit */
//         KeLowerIrql(OldIrql);
//         return;
//     }
//
//     /* Ensure no one will come disturb */
//     LockIrql = KeAcquireQueuedSpinLock(LockQueueIoCompletionLock);
//
//     /* Get the misbehaving IRP */
//     IrpEntry = IrpThread->IrpList.Flink;
//     IopDeadIrp = CONTAINING_RECORD(IrpEntry, IRP, ThreadListEntry);
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Deassociating IRP %p for %p\n",
//             __FUNCTION__,
//             IopDeadIrp,
//             IrpThread);
//
//     /* Don't cancel the IRP if it's already been completed far */
//     if (IopDeadIrp->CurrentLocation == (IopDeadIrp->StackCount + 2))
//     {
//         /* Return */
//         KeReleaseQueuedSpinLock(LockQueueIoCompletionLock, LockIrql);
//         KeLowerIrql(OldIrql);
//         return;
//     }
//
//     /* Disown the IRP! */
//     IopDeadIrp->Tail.Overlay.Thread = NULL;
//     RemoveHeadList(&IrpThread->IrpList);
//     InitializeListHead(&IopDeadIrp->ThreadListEntry);
//
//     /* Get the stack location and check if it's valid */
//     IoStackLocation = IoGetCurrentIrpStackLocation(IopDeadIrp);
//     if (IopDeadIrp->CurrentLocation <= IopDeadIrp->StackCount)
//     {
//         /* Get the device object */
//         DeviceObject = IoStackLocation->DeviceObject;
//     }
//
//     KeReleaseQueuedSpinLock(LockQueueIoCompletionLock, LockIrql);
//     /* Lower IRQL now, since we have the pointers we need */
//     KeLowerIrql(OldIrql);
//
//     /* Check if we can send an Error Log Entry*/
//     if (DeviceObject)
//     {
//         /* Allocate an entry */
//         ErrorLogEntry = IoAllocateErrorLogEntry(DeviceObject,
//                                                 sizeof(IO_ERROR_LOG_PACKET));
//         if (ErrorLogEntry)
//         {
//             /* Write the entry */
//             ErrorLogEntry->ErrorCode = IO_DRIVER_CANCEL_TIMEOUT;
//             IoWriteErrorLogEntry(ErrorLogEntry);
//         }
//     }
// }
//
// VOID
// NTAPI
// IopCleanupIrp(IN PIRP Irp,
//               IN PFILE_OBJECT FileObject)
// {
//     PMDL Mdl;
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Cleaning IRP %p for %p\n",
//             __FUNCTION__,
//             Irp,
//             FileObject);
//
//     /* Check if there's an MDL */
//     while ((Mdl = Irp->MdlAddress))
//     {
//         /* Clear all of them */
//         Irp->MdlAddress = Mdl->Next;
//         IoFreeMdl(Mdl);
//     }
//
//     /* Check if the IRP has system buffer */
//     if (Irp->Flags & IRP_DEALLOCATE_BUFFER)
//     {
//         /* Free the buffer */
//         ExFreePoolWithTag(Irp->AssociatedIrp.SystemBuffer, TAG_IOBUF);
//     }
//
//     /* Check if this IRP has a user event, a file object, and is async */
//     if ((Irp->UserEvent) &&
//         !(Irp->Flags & IRP_SYNCHRONOUS_API) &&
//         (FileObject))
//     {
//         /* Dereference the User Event */
//         ObDereferenceObject(Irp->UserEvent);
//     }
//
//     /* Check if we have a file object and this isn't a create operation */
//     if ((FileObject) && !(Irp->Flags & IRP_CREATE_OPERATION))
//     {
//         /* Dereference the file object */
//         ObDereferenceObject(FileObject);
//     }
//
//     /* Free the IRP */
//     IoFreeIrp(Irp);
// }
//
// VOID
// NTAPI
// IopCompleteRequest(IN PKAPC Apc,
//                    IN PKNORMAL_ROUTINE* NormalRoutine,
//                    IN PVOID* NormalContext,
//                    IN PVOID* SystemArgument1,
//                    IN PVOID* SystemArgument2)
// {
//     PFILE_OBJECT FileObject;
//     PIRP Irp;
//     PMDL Mdl, NextMdl;
//     PVOID Port = NULL, Key = NULL;
//     BOOLEAN SignaledCreateRequest = FALSE;
//
//     /* Get data from the APC */
//     FileObject = (PFILE_OBJECT)*SystemArgument1;
//     Irp = CONTAINING_RECORD(Apc, IRP, Tail.Apc);
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Completing IRP %p for %p\n",
//             __FUNCTION__,
//             Irp,
//             FileObject);
//
//     /* Sanity check */
//     ASSERT(Irp->IoStatus.Status != (NTSTATUS)0xFFFFFFFF);
//
//     /* Check if we have a file object */
//     if (*SystemArgument2)
//     {
//         /* Check if we're reparsing */
//         if ((Irp->IoStatus.Status == STATUS_REPARSE) &&
//             (Irp->IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT))
//         {
//             PREPARSE_DATA_BUFFER ReparseData;
//
//             ReparseData = (PREPARSE_DATA_BUFFER)*SystemArgument2;
//
//             ASSERT(ReparseData->ReparseTag == IO_REPARSE_TAG_MOUNT_POINT);
//             ASSERT(ReparseData->ReparseDataLength < MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//             ASSERT(ReparseData->Reserved < MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//
//             IopDoNameTransmogrify(Irp, FileObject, ReparseData);
//         }
//     }
//
//     /* Handle Buffered case first */
//     if (Irp->Flags & IRP_BUFFERED_IO)
//     {
//         /* Check if we have an input buffer and if we succeeded */
//         if ((Irp->Flags & IRP_INPUT_OPERATION) &&
//             (Irp->IoStatus.Status != STATUS_VERIFY_REQUIRED) &&
//             !(NT_ERROR(Irp->IoStatus.Status)))
//         {
//             _SEH2_TRY
//             {
//                 /* Copy the buffer back to the user */
//                 RtlCopyMemory(Irp->UserBuffer,
//                               Irp->AssociatedIrp.SystemBuffer,
//                               Irp->IoStatus.Information);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Fail the IRP */
//                 Irp->IoStatus.Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//
//         /* Also check if we should de-allocate it */
//         if (Irp->Flags & IRP_DEALLOCATE_BUFFER)
//         {
//             /* Deallocate it */
//             ExFreePool(Irp->AssociatedIrp.SystemBuffer);
//         }
//     }
//
//     /* Now we got rid of these two... */
//     Irp->Flags &= ~(IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER);
//
//     /* Check if there's an MDL */
//     for (Mdl = Irp->MdlAddress; Mdl; Mdl = NextMdl)
//     {
//         /* Free it */
//         NextMdl = Mdl->Next;
//         IoFreeMdl(Mdl);
//     }
//
//     /* No MDLs left */
//     Irp->MdlAddress = NULL;
//
//     /*
//      * Check if either the request was completed without any errors
//      * (but warnings are OK!), or if it was completed with an error, but
//      * did return from a pending I/O Operation and is not synchronous.
//      */
//     if (!NT_ERROR(Irp->IoStatus.Status) ||
//         (Irp->PendingReturned &&
//          !IsIrpSynchronous(Irp, FileObject)))
//     {
//         /* Get any information we need from the FO before we kill it */
//         if ((FileObject) && (FileObject->CompletionContext))
//         {
//             /* Save Completion Data */
//             Port = FileObject->CompletionContext->Port;
//             Key = FileObject->CompletionContext->Key;
//         }
//
//         /* Check for UserIos */
//         if (Irp->UserIosb != NULL)
//         {
//             /* Use SEH to make sure we don't write somewhere invalid */
//             _SEH2_TRY
//             {
//                 /*  Save the IOSB Information */
//                 *Irp->UserIosb = Irp->IoStatus;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Ignore any error */
//             }
//             _SEH2_END;
//         }
//
//         /* Check if we have an event or a file object */
//         if (Irp->UserEvent)
//         {
//             /* At the very least, this is a PKEVENT, so signal it always */
//             KeSetEvent(Irp->UserEvent, 0, FALSE);
//
//             /* Check if we also have a File Object */
//             if (FileObject)
//             {
//                 /* Check if this is an Asynch API */
//                 if (!(Irp->Flags & IRP_SYNCHRONOUS_API))
//                 {
//                     /* Dereference the event */
//                     ObDereferenceObject(Irp->UserEvent);
//                 }
//
//                 /*
//                  * Now, if this is a Synch I/O File Object, then this event is
//                  * NOT an actual Executive Event, so we won't dereference it,
//                  * and instead, we will signal the File Object
//                  */
//                 if ((FileObject->Flags & FO_SYNCHRONOUS_IO) &&
//                     !(Irp->Flags & IRP_OB_QUERY_NAME))
//                 {
//                     /* Signal the file object and set the status */
//                     KeSetEvent(&FileObject->Event, 0, FALSE);
//                     FileObject->FinalStatus = Irp->IoStatus.Status;
//                 }
//
//                 /*
//                  * This could also be a create operation, in which case we want
//                  * to make sure there's no APC fired.
//                  */
//                 if (Irp->Flags & IRP_CREATE_OPERATION)
//                 {
//                     /* Clear the APC Routine and remember this */
//                     Irp->Overlay.AsynchronousParameters.UserApcRoutine = NULL;
//                     SignaledCreateRequest = TRUE;
//                 }
//             }
//         }
//         else if (FileObject)
//         {
//             /* Signal the file object and set the status */
//             KeSetEvent(&FileObject->Event, 0, FALSE);
//             FileObject->FinalStatus = Irp->IoStatus.Status;
//
//             /*
//             * This could also be a create operation, in which case we want
//             * to make sure there's no APC fired.
//             */
//             if (Irp->Flags & IRP_CREATE_OPERATION)
//             {
//                 /* Clear the APC Routine and remember this */
//                 Irp->Overlay.AsynchronousParameters.UserApcRoutine = NULL;
//                 SignaledCreateRequest = TRUE;
//             }
//         }
//
//         /* Update transfer count for everything but create operation */
//         if (!(Irp->Flags & IRP_CREATE_OPERATION))
//         {
//             if (Irp->Flags & IRP_WRITE_OPERATION)
//             {
//                 /* Update write transfer count */
//                 IopUpdateTransferCount(IopWriteTransfer,
//                                        (ULONG)Irp->IoStatus.Information);
//             }
//             else if (Irp->Flags & IRP_READ_OPERATION)
//             {
//                 /* Update read transfer count */
//                 IopUpdateTransferCount(IopReadTransfer,
//                                        (ULONG)Irp->IoStatus.Information);
//             }
//             else
//             {
//                 /* Update other transfer count */
//                 IopUpdateTransferCount(IopOtherTransfer,
//                                        (ULONG)Irp->IoStatus.Information);
//             }
//         }
//
//         /* Now that we've signaled the events, de-associate the IRP */
//         IopUnQueueIrpFromThread(Irp);
//
//         /* Now check if a User APC Routine was requested */
//         if (Irp->Overlay.AsynchronousParameters.UserApcRoutine)
//         {
//             /* Initialize it */
//             KeInitializeApc(&Irp->Tail.Apc,
//                             KeGetCurrentThread(),
//                             CurrentApcEnvironment,
//                             IopFreeIrpKernelApc,
//                             IopAbortIrpKernelApc,
//                             (PKNORMAL_ROUTINE)Irp->
//                             Overlay.AsynchronousParameters.UserApcRoutine,
//                             Irp->RequestorMode,
//                             Irp->
//                             Overlay.AsynchronousParameters.UserApcContext);
//
//             /* Queue it */
//             KeInsertQueueApc(&Irp->Tail.Apc, Irp->UserIosb, NULL, 2);
//         }
//         else if ((Port) &&
//                  (Irp->Overlay.AsynchronousParameters.UserApcContext))
//         {
//             /* We have an I/O Completion setup... create the special Overlay */
//             Irp->Tail.CompletionKey = Key;
//             Irp->Tail.Overlay.PacketType = IopCompletionPacketIrp;
//             KeInsertQueue(Port, &Irp->Tail.Overlay.ListEntry);
//         }
//         else
//         {
//             /* Free the IRP since we don't need it anymore */
//             IoFreeIrp(Irp);
//         }
//
//         /* Check if we have a file object that wasn't part of a create */
//         if ((FileObject) && !(SignaledCreateRequest))
//         {
//             /* Dereference it, since it's not needed anymore either */
//             ObDereferenceObjectDeferDelete(FileObject);
//         }
//     }
//     else
//     {
//         /*
//          * Either we didn't return from the request, or we did return but this
//          * request was synchronous.
//          */
//         if ((Irp->PendingReturned) && (FileObject))
//         {
//             /* So we did return with a synch operation, was it the IRP? */
//             if (Irp->Flags & IRP_SYNCHRONOUS_API)
//             {
//                 /* Yes, this IRP was synchronous, so return the I/O Status */
//                 *Irp->UserIosb = Irp->IoStatus;
//
//                 /* Now check if the user gave an event */
//                 if (Irp->UserEvent)
//                 {
//                     /* Signal it */
//                     KeSetEvent(Irp->UserEvent, 0, FALSE);
//                 }
//                 else
//                 {
//                     /* No event was given, so signal the FO instead */
//                     KeSetEvent(&FileObject->Event, 0, FALSE);
//                 }
//             }
//             else
//             {
//                 /*
//                  * It's not the IRP that was synchronous, it was the FO
//                  * that was opened this way. Signal its event.
//                  */
//                 FileObject->FinalStatus = Irp->IoStatus.Status;
//                 KeSetEvent(&FileObject->Event, 0, FALSE);
//             }
//         }
//
//         /* Now that we got here, we do this for incomplete I/Os as well */
//         if ((FileObject) && !(Irp->Flags & IRP_CREATE_OPERATION))
//         {
//             /* Dereference the File Object unless this was a create */
//             ObDereferenceObjectDeferDelete(FileObject);
//         }
//
//         /*
//          * Check if this was an Executive Event (remember that we know this
//          * by checking if the IRP is synchronous)
//          */
//         if ((Irp->UserEvent) &&
//             (FileObject) &&
//             !(Irp->Flags & IRP_SYNCHRONOUS_API))
//         {
//             /* This isn't a PKEVENT, so dereference it */
//             ObDereferenceObject(Irp->UserEvent);
//         }
//
//         /* Now that we've signaled the events, de-associate the IRP */
//         IopUnQueueIrpFromThread(Irp);
//
//         /* Free the IRP as well */
//         IoFreeIrp(Irp);
//     }
// }
//
// BOOLEAN
// NTAPI
// IopInitializeReserveIrp(IN PRESERVE_IRP_ALLOCATOR ReserveIrpAllocator)
// {
//     /* Our allocated stack size */
//     ReserveIrpAllocator->StackSize = 20;
//
//     /* Allocate the IRP now */
//     ReserveIrpAllocator->ReserveIrp = IoAllocateIrp(ReserveIrpAllocator->StackSize, FALSE);
//     /* If we cannot, abort system boot */
//     if (ReserveIrpAllocator->ReserveIrp == NULL)
//     {
//         return FALSE;
//     }
//
//     /* It's not in use */
//     ReserveIrpAllocator->ReserveIrpInUse = 0;
//     /* And init the event */
//     KeInitializeEvent(&ReserveIrpAllocator->WaitEvent, SynchronizationEvent, FALSE);
//
//     /* All good, keep booting */
//     return TRUE;
// }
//
// PIRP
// NTAPI
// IopAllocateReserveIrp(IN CCHAR StackSize)
// {
//     /* If we need a stack size higher than what was allocated, then fail */
//     if (StackSize > IopReserveIrpAllocator.StackSize)
//     {
//         return NULL;
//     }
//
//     /* Now, wait until the IRP becomes available and reserve it immediately */
//     while (InterlockedExchange(&IopReserveIrpAllocator.ReserveIrpInUse, 1) == 1)
//     {
//         KeWaitForSingleObject(&IopReserveIrpAllocator.WaitEvent,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     /* It's ours! Initialize it */
//     IoInitializeIrp(IopReserveIrpAllocator.ReserveIrp, IoSizeOfIrp(StackSize), StackSize);
//
//     /* And return it to the caller */
//     return IopReserveIrpAllocator.ReserveIrp;
// }
//
// VOID
// IopFreeReserveIrp(IN CCHAR PriorityBoost)
// {
//     /* Mark we don't use the IRP anymore */
//     InterlockedExchange(&IopReserveIrpAllocator.ReserveIrpInUse, 0);
//
//     /* And set the event if someone is waiting on the IRP */
//     KeSetEvent(&IopReserveIrpAllocator.WaitEvent, PriorityBoost, FALSE);
// }
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// PIRP
// NTAPI
// IoAllocateIrp(IN CCHAR StackSize,
//               IN BOOLEAN ChargeQuota)
// {
//     PIRP Irp = NULL;
//     USHORT Size = IoSizeOfIrp(StackSize);
//     PKPRCB Prcb;
//     UCHAR Flags = 0;
//     PNPAGED_LOOKASIDE_LIST List = NULL;
//     PP_NPAGED_LOOKASIDE_NUMBER ListType = LookasideSmallIrpList;
//
//     /* Set Charge Quota Flag */
//     if (ChargeQuota) Flags |= IRP_QUOTA_CHARGED;
//
//     /* Get the PRCB */
//     Prcb = KeGetCurrentPrcb();
//
//     /* Figure out which Lookaside List to use */
//     if ((StackSize <= 8) && (ChargeQuota == FALSE || Prcb->LookasideIrpFloat > 0))
//     {
//         /* Set Fixed Size Flag */
//         Flags |= IRP_ALLOCATED_FIXED_SIZE;
//
//         /* See if we should use big list */
//         if (StackSize != 1)
//         {
//             Size = IoSizeOfIrp(8);
//             ListType = LookasideLargeIrpList;
//         }
//
//         /* Get the P List First */
//         List = (PNPAGED_LOOKASIDE_LIST)Prcb->PPLookasideList[ListType].P;
//
//         /* Attempt allocation */
//         List->L.TotalAllocates++;
//         Irp = (PIRP)InterlockedPopEntrySList(&List->L.ListHead);
//
//         /* Check if the P List failed */
//         if (!Irp)
//         {
//             /* Let the balancer know */
//             List->L.AllocateMisses++;
//
//             /* Try the L List */
//             List = (PNPAGED_LOOKASIDE_LIST)Prcb->PPLookasideList[ListType].L;
//             List->L.TotalAllocates++;
//             Irp = (PIRP)InterlockedPopEntrySList(&List->L.ListHead);
//         }
//     }
//
//     /* Check if we have to use the pool */
//     if (!Irp)
//     {
//         /* Did we try lookaside and fail? */
//         if (Flags & IRP_ALLOCATED_FIXED_SIZE) List->L.AllocateMisses++;
//
//         /* Check if we should charge quota */
//         if (ChargeQuota)
//         {
//             Irp = ExAllocatePoolWithQuotaTag(NonPagedPool | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE,
//                                              Size,
//                                              TAG_IRP);
//         }
//         else
//         {
//             /* Allocate the IRP with no quota charge */
//             Irp = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_IRP);
//         }
//
//         /* Make sure it was sucessful */
//         if (!Irp) return NULL;
//     }
//     else if (Flags & IRP_QUOTA_CHARGED)
//     {
//         /* Decrement lookaside float */
//         InterlockedDecrement(&Prcb->LookasideIrpFloat);
//         Flags |= IRP_LOOKASIDE_ALLOCATION;
//
//         /* In this case there is no charge quota */
//         Flags &= ~IRP_QUOTA_CHARGED;
//     }
//
//     /* Now Initialize it */
//     IoInitializeIrp(Irp, Size, StackSize);
//
//     /* Set the Allocation Flags */
//     Irp->AllocationFlags = Flags;
//
//     /* Return it */
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Allocated IRP %p with allocation flags %lx\n",
//             __FUNCTION__,
//             Irp,
//             Flags);
//     return Irp;
// }
//
// /*
//  * @implemented
//  */
// PIRP
// NTAPI
// IopAllocateIrpMustSucceed(IN CCHAR StackSize)
// {
//     LONG i;
//     PIRP Irp;
//     LARGE_INTEGER Sleep;
//
//     /* Try to get an IRP */
//     Irp = IoAllocateIrp(StackSize, FALSE);
//     if (Irp)
//         return Irp;
//
//     /* If we fail, start looping till we may get one */
//     i = LONG_MAX;
//     do {
//         i--;
//
//         /* First, sleep for 10ms */
//         Sleep.QuadPart = -10 * 1000 * 10;
//         KeDelayExecutionThread(KernelMode, FALSE, &Sleep);
//
//         /* Then, retry allocation */
//         Irp = IoAllocateIrp(StackSize, FALSE);
//         if (Irp)
//             return Irp;
//     } while (i > 0);
//
//     return Irp;
// }
//
// /*
//  * @implemented
//  */
// PIRP
// NTAPI
// IoBuildAsynchronousFsdRequest(IN ULONG MajorFunction,
//                               IN PDEVICE_OBJECT DeviceObject,
//                               IN PVOID Buffer,
//                               IN ULONG Length,
//                               IN PLARGE_INTEGER StartingOffset,
//                               IN PIO_STATUS_BLOCK IoStatusBlock)
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION StackPtr;
//
//     /* Allocate IRP */
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//     if (!Irp) return NULL;
//
//     /* Get the Stack */
//     StackPtr = IoGetNextIrpStackLocation(Irp);
//
//     /* Write the Major function and then deal with it */
//     StackPtr->MajorFunction = (UCHAR)MajorFunction;
//
//     /* Do not handle the following here */
//     if ((MajorFunction != IRP_MJ_FLUSH_BUFFERS) &&
//         (MajorFunction != IRP_MJ_SHUTDOWN) &&
//         (MajorFunction != IRP_MJ_PNP) &&
//         (MajorFunction != IRP_MJ_POWER))
//     {
//         /* Check if this is Buffered IO */
//         if (DeviceObject->Flags & DO_BUFFERED_IO)
//         {
//             /* Allocate the System Buffer */
//             Irp->AssociatedIrp.SystemBuffer =
//                 ExAllocatePoolWithTag(NonPagedPool, Length, TAG_IOBUF);
//             if (!Irp->AssociatedIrp.SystemBuffer)
//             {
//                 /* Free the IRP and fail */
//                 IoFreeIrp(Irp);
//                 return NULL;
//             }
//
//             /* Set flags */
//             Irp->Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
//
//             /* Handle special IRP_MJ_WRITE Case */
//             if (MajorFunction == IRP_MJ_WRITE)
//             {
//                 /* Copy the buffer data */
//                 RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer, Buffer, Length);
//             }
//             else
//             {
//                 /* Set the Input Operation flag and set this as a User Buffer */
//                 Irp->Flags |= IRP_INPUT_OPERATION;
//                 Irp->UserBuffer = Buffer;
//             }
//         }
//         else if (DeviceObject->Flags & DO_DIRECT_IO)
//         {
//             /* Use an MDL for Direct I/O */
//             Irp->MdlAddress = IoAllocateMdl(Buffer,
//                                             Length,
//                                             FALSE,
//                                             FALSE,
//                                             NULL);
//             if (!Irp->MdlAddress)
//             {
//                 /* Free the IRP and fail */
//                 IoFreeIrp(Irp);
//                 return NULL;
//             }
//
//             /* Probe and Lock */
//             _SEH2_TRY
//             {
//                 /* Do the probe */
//                 MmProbeAndLockPages(Irp->MdlAddress,
//                                     KernelMode,
//                                     MajorFunction == IRP_MJ_READ ?
//                                     IoWriteAccess : IoReadAccess);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Free the IRP and its MDL */
//                 IoFreeMdl(Irp->MdlAddress);
//                 IoFreeIrp(Irp);
//
//                 /* Fail */
//                 _SEH2_YIELD(return NULL);
//             }
//             _SEH2_END;
//         }
//         else
//         {
//             /* Neither, use the buffer */
//             Irp->UserBuffer = Buffer;
//         }
//
//         /* Check if this is a read */
//         if (MajorFunction == IRP_MJ_READ)
//         {
//             /* Set the parameters for a read */
//             StackPtr->Parameters.Read.Length = Length;
//             StackPtr->Parameters.Read.ByteOffset = *StartingOffset;
//         }
//         else if (MajorFunction == IRP_MJ_WRITE)
//         {
//             /* Otherwise, set write parameters */
//             StackPtr->Parameters.Write.Length = Length;
//             StackPtr->Parameters.Write.ByteOffset = *StartingOffset;
//         }
//     }
//
//     /* Set the Current Thread and IOSB */
//     Irp->UserIosb = IoStatusBlock;
//     Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//
//     /* Return the IRP */
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Built IRP %p with Major, Buffer, DO %lx %p %p\n",
//             __FUNCTION__,
//             Irp,
//             MajorFunction,
//             Buffer,
//             DeviceObject);
//     return Irp;
// }
//
// /*
//  * @implemented
//  */
// PIRP
// NTAPI
// IoBuildDeviceIoControlRequest(IN ULONG IoControlCode,
//                               IN PDEVICE_OBJECT DeviceObject,
//                               IN PVOID InputBuffer,
//                               IN ULONG InputBufferLength,
//                               IN PVOID OutputBuffer,
//                               IN ULONG OutputBufferLength,
//                               IN BOOLEAN InternalDeviceIoControl,
//                               IN PKEVENT Event,
//                               IN PIO_STATUS_BLOCK IoStatusBlock)
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION StackPtr;
//     ULONG BufferLength;
//
//     /* Allocate IRP */
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//     if (!Irp) return NULL;
//
//     /* Get the Stack */
//     StackPtr = IoGetNextIrpStackLocation(Irp);
//
//     /* Set the DevCtl Type */
//     StackPtr->MajorFunction = InternalDeviceIoControl ?
//                               IRP_MJ_INTERNAL_DEVICE_CONTROL :
//                               IRP_MJ_DEVICE_CONTROL;
//
//     /* Set the IOCTL Data */
//     StackPtr->Parameters.DeviceIoControl.IoControlCode = IoControlCode;
//     StackPtr->Parameters.DeviceIoControl.InputBufferLength = InputBufferLength;
//     StackPtr->Parameters.DeviceIoControl.OutputBufferLength =
//         OutputBufferLength;
//
//     /* Handle the Methods */
//     switch (IO_METHOD_FROM_CTL_CODE(IoControlCode))
//     {
//         /* Buffered I/O */
//         case METHOD_BUFFERED:
//
//             /* Select the right Buffer Length */
//             BufferLength = InputBufferLength > OutputBufferLength ?
//                            InputBufferLength : OutputBufferLength;
//
//             /* Make sure there is one */
//             if (BufferLength)
//             {
//                 /* Allocate the System Buffer */
//                 Irp->AssociatedIrp.SystemBuffer =
//                     ExAllocatePoolWithTag(NonPagedPool,
//                                           BufferLength,
//                                           TAG_IOBUF);
//                 if (!Irp->AssociatedIrp.SystemBuffer)
//                 {
//                     /* Free the IRP and fail */
//                     IoFreeIrp(Irp);
//                     return NULL;
//                 }
//
//                 /* Check if we got a buffer */
//                 if (InputBuffer)
//                 {
//                     /* Copy into the System Buffer */
//                     RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer,
//                                   InputBuffer,
//                                   InputBufferLength);
//                 }
//
//                 /* Write the flags */
//                 Irp->Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
//                 if (OutputBuffer) Irp->Flags |= IRP_INPUT_OPERATION;
//
//                 /* Save the Buffer */
//                 Irp->UserBuffer = OutputBuffer;
//             }
//             else
//             {
//                 /* Clear the Flags and Buffer */
//                 Irp->Flags = 0;
//                 Irp->UserBuffer = NULL;
//             }
//             break;
//
//         /* Direct I/O */
//         case METHOD_IN_DIRECT:
//         case METHOD_OUT_DIRECT:
//
//             /* Check if we got an input buffer */
//             if (InputBuffer)
//             {
//                 /* Allocate the System Buffer */
//                 Irp->AssociatedIrp.SystemBuffer =
//                     ExAllocatePoolWithTag(NonPagedPool,
//                                           InputBufferLength,
//                                           TAG_IOBUF);
//                 if (!Irp->AssociatedIrp.SystemBuffer)
//                 {
//                     /* Free the IRP and fail */
//                     IoFreeIrp(Irp);
//                     return NULL;
//                 }
//
//                 /* Copy into the System Buffer */
//                 RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer,
//                               InputBuffer,
//                               InputBufferLength);
//
//                 /* Write the flags */
//                 Irp->Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
//             }
//             else
//             {
//                 /* Clear the flags */
//                 Irp->Flags = 0;
//             }
//
//             /* Check if we got an output buffer */
//             if (OutputBuffer)
//             {
//                 /* Allocate the System Buffer */
//                 Irp->MdlAddress = IoAllocateMdl(OutputBuffer,
//                                                 OutputBufferLength,
//                                                 FALSE,
//                                                 FALSE,
//                                                 Irp);
//                 if (!Irp->MdlAddress)
//                 {
//                     /* Free the IRP and fail */
//                     IoFreeIrp(Irp);
//                     return NULL;
//                 }
//
//                 /* Probe and Lock */
//                 _SEH2_TRY
//                 {
//                     /* Do the probe */
//                     MmProbeAndLockPages(Irp->MdlAddress,
//                                         KernelMode,
//                                         IO_METHOD_FROM_CTL_CODE(IoControlCode) ==
//                                         METHOD_IN_DIRECT ?
//                                         IoReadAccess : IoWriteAccess);
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     /* Free the MDL */
//                     IoFreeMdl(Irp->MdlAddress);
//
//                     /* Free the input buffer and IRP */
//                     if (InputBuffer) ExFreePool(Irp->AssociatedIrp.SystemBuffer);
//                     IoFreeIrp(Irp);
//
//                     /* Fail */
//                     _SEH2_YIELD(return NULL);
//                 }
//                 _SEH2_END;
//             }
//             break;
//
//         case METHOD_NEITHER:
//
//             /* Just save the Buffer */
//             Irp->UserBuffer = OutputBuffer;
//             StackPtr->Parameters.DeviceIoControl.Type3InputBuffer = InputBuffer;
//     }
//
//     /* Now write the Event and IoSB */
//     Irp->UserIosb = IoStatusBlock;
//     Irp->UserEvent = Event;
//
//     /* Sync IRPs are queued to requestor thread's irp cancel/cleanup list */
//     Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//     IoQueueThreadIrp(Irp);
//
//     /* Return the IRP */
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Built IRP %p with IOCTL, Buffers, DO %lx %p %p %p\n",
//             __FUNCTION__,
//             Irp,
//             IoControlCode,
//             InputBuffer,
//             OutputBuffer,
//             DeviceObject);
//     return Irp;
// }
//
// /*
//  * @implemented
//  */
// PIRP
// NTAPI
// IoBuildSynchronousFsdRequest(IN ULONG MajorFunction,
//                              IN PDEVICE_OBJECT DeviceObject,
//                              IN PVOID Buffer,
//                              IN ULONG Length,
//                              IN PLARGE_INTEGER StartingOffset,
//                              IN PKEVENT Event,
//                              IN PIO_STATUS_BLOCK IoStatusBlock)
// {
//     PIRP Irp;
//
//     /* Do the big work to set up the IRP */
//     Irp = IoBuildAsynchronousFsdRequest(MajorFunction,
//                                         DeviceObject,
//                                         Buffer,
//                                         Length,
//                                         StartingOffset,
//                                         IoStatusBlock );
//     if (!Irp) return NULL;
//
//     /* Set the Event which makes it Syncronous */
//     Irp->UserEvent = Event;
//
//     /* Sync IRPs are queued to requestor thread's irp cancel/cleanup list */
//     IoQueueThreadIrp(Irp);
//     return Irp;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoCancelIrp(IN PIRP Irp)
// {
//     KIRQL OldIrql;
//     PDRIVER_CANCEL CancelRoutine;
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Canceling IRP %p\n",
//             __FUNCTION__,
//             Irp);
//     ASSERT(Irp->Type == IO_TYPE_IRP);
//
//     /* Acquire the cancel lock and cancel the IRP */
//     IoAcquireCancelSpinLock(&OldIrql);
//     Irp->Cancel = TRUE;
//
//     /* Clear the cancel routine and get the old one */
//     CancelRoutine = IoSetCancelRoutine(Irp, NULL);
//     if (CancelRoutine)
//     {
//         /* We had a routine, make sure the IRP isn't completed */
//         if (Irp->CurrentLocation > (Irp->StackCount + 1))
//         {
//             /* It is, bugcheck */
//             KeBugCheckEx(CANCEL_STATE_IN_COMPLETED_IRP,
//                          (ULONG_PTR)Irp,
//                          (ULONG_PTR)CancelRoutine,
//                          0,
//                          0);
//         }
//
//         /* Set the cancel IRQL And call the routine */
//         Irp->CancelIrql = OldIrql;
//         CancelRoutine(IoGetCurrentIrpStackLocation(Irp)->DeviceObject, Irp);
//         return TRUE;
//     }
//
//     /* Otherwise, release the cancel lock and fail */
//     IoReleaseCancelSpinLock(OldIrql);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoCancelThreadIo(IN PETHREAD Thread)
// {
//     KIRQL OldIrql;
//     ULONG Retries = 3000;
//     LARGE_INTEGER Interval;
//     PLIST_ENTRY ListHead, NextEntry;
//     PIRP Irp;
//     PAGED_CODE();
//
//     /* Windows isn't using given thread, but using current. */
//     Thread = PsGetCurrentThread();
//
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Canceling IRPs for Thread %p\n",
//             __FUNCTION__,
//             Thread);
//
//     /* Raise to APC to protect the IrpList */
//     KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Start by cancelling all the IRPs in the current thread queue. */
//     ListHead = &Thread->IrpList;
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         /* Get the IRP */
//         Irp = CONTAINING_RECORD(NextEntry, IRP, ThreadListEntry);
//
//         /* Cancel it */
//         IoCancelIrp(Irp);
//
//         /* Move to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//      /* Wait 100 milliseconds */
//     Interval.QuadPart = -1000000;
//
//     /* Wait till all the IRPs are completed or cancelled. */
//     while (!IsListEmpty(&Thread->IrpList))
//     {
//         /* Now we can lower */
//         KeLowerIrql(OldIrql);
//
//         /* Wait a short while and then look if all our IRPs were completed. */
//         KeDelayExecutionThread(KernelMode, FALSE, &Interval);
//
//         /*
//          * Don't stay here forever if some broken driver doesn't complete
//          * the IRP.
//          */
//         if (!(Retries--))
//         {
//             /* Print out a message and remove the IRP */
//             DPRINT1("Broken driver did not complete!\n");
//             IopDisassociateThreadIrp();
//         }
//
//         /* Raise the IRQL Again */
//         KeRaiseIrql(APC_LEVEL, &OldIrql);
//     }
//
//     /* We're done, lower the IRQL */
//     KeLowerIrql(OldIrql);
// }
//
// /*
//  * @implemented
//  */
// #undef IoCallDriver
// NTSTATUS
// NTAPI
// IoCallDriver(IN PDEVICE_OBJECT DeviceObject,
//              IN PIRP Irp)
// {
//     /* Call fastcall */
//     return IofCallDriver(DeviceObject, Irp);
// }
//
// #define IoCallDriver IofCallDriver
//
// /*
//  * @implemented
//  */
// #undef IoCompleteRequest
// VOID
// NTAPI
// IoCompleteRequest(IN PIRP Irp,
//                   IN CCHAR PriorityBoost)
// {
//     /* Call the fastcall */
//     IofCompleteRequest(Irp, PriorityBoost);
// }
//
// #define IoCompleteRequest IofCompleteRequest
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoEnqueueIrp(IN PIRP Irp)
// {
//     /* This is the same as calling IoQueueThreadIrp */
//     IoQueueThreadIrp(Irp);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// FASTCALL
// IofCallDriver(IN PDEVICE_OBJECT DeviceObject,
//               IN PIRP Irp)
// {
//     PDRIVER_OBJECT DriverObject;
//     PIO_STACK_LOCATION StackPtr;
//
//     /* Make sure this is a valid IRP */
//     ASSERT(Irp->Type == IO_TYPE_IRP);
//
//     /* Get the Driver Object */
//     DriverObject = DeviceObject->DriverObject;
//
//     /* Decrease the current location and check if */
//     Irp->CurrentLocation--;
//     if (Irp->CurrentLocation <= 0)
//     {
//         /* This IRP ran out of stack, bugcheck */
//         KeBugCheckEx(NO_MORE_IRP_STACK_LOCATIONS, (ULONG_PTR)Irp, 0, 0, 0);
//     }
//
//     /* Now update the stack location */
//     StackPtr = IoGetNextIrpStackLocation(Irp);
//     Irp->Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation = StackPtr;
//
//     /* Get the Device Object */
//     StackPtr->DeviceObject = DeviceObject;
//
//     /* Call it */
//     return DriverObject->MajorFunction[StackPtr->MajorFunction](DeviceObject,
//                                                                 Irp);
// }
//
// FORCEINLINE
// VOID
// IopClearStackLocation(IN PIO_STACK_LOCATION IoStackLocation)
// {
//     IoStackLocation->MinorFunction = 0;
//     IoStackLocation->Flags = 0;
//     IoStackLocation->Control &= SL_ERROR_RETURNED;
//     IoStackLocation->Parameters.Others.Argument1 = 0;
//     IoStackLocation->Parameters.Others.Argument2 = 0;
//     IoStackLocation->Parameters.Others.Argument3 = 0;
//     IoStackLocation->FileObject = NULL;
// }
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// IofCompleteRequest(IN PIRP Irp,
//                    IN CCHAR PriorityBoost)
// {
//     PIO_STACK_LOCATION StackPtr, LastStackPtr;
//     PDEVICE_OBJECT DeviceObject;
//     PFILE_OBJECT FileObject;
//     PETHREAD Thread;
//     NTSTATUS Status;
//     PMDL Mdl, NextMdl;
//     ULONG MasterCount;
//     PIRP MasterIrp;
//     ULONG Flags;
//     NTSTATUS ErrorCode = STATUS_SUCCESS;
//     PREPARSE_DATA_BUFFER DataBuffer = NULL;
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Completing IRP %p\n",
//             __FUNCTION__,
//             Irp);
//
//     _IRP_TRACE2("[IRP2] IofCompleteRequest\r\n");
//
//     /* Make sure this IRP isn't getting completed twice or is invalid */
//     if ((Irp->CurrentLocation) > (Irp->StackCount + 1))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(MULTIPLE_IRP_COMPLETE_REQUESTS, (ULONG_PTR)Irp, 0, 0, 0);
//     }
//
//     /* Some sanity checks */
//     ASSERT(Irp->Type == IO_TYPE_IRP);
//     ASSERT(!Irp->CancelRoutine);
//     ASSERT(Irp->IoStatus.Status != STATUS_PENDING);
//     ASSERT(Irp->IoStatus.Status != (NTSTATUS)0xFFFFFFFF);
//
//     /* Get the last stack */
//     LastStackPtr = (PIO_STACK_LOCATION)(Irp + 1);
//     if (LastStackPtr->Control & SL_ERROR_RETURNED)
//     {
//         /* Get the error code */
//         ErrorCode = PtrToUlong(LastStackPtr->Parameters.Others.Argument4);
//     }
//
//     /*
//      * Start the loop with the current stack and point the IRP to the next stack
//      * and then keep incrementing the stack as we loop through. The IRP should
//      * always point to the next stack location w.r.t the one currently being
//      * analyzed, so completion routine code will see the appropriate value.
//      * Because of this, we must loop until the current stack location is +1 of
//      * the stack count, because when StackPtr is at the end, CurrentLocation is +1.
//      */
//     for (StackPtr = IoGetCurrentIrpStackLocation(Irp),
//          Irp->CurrentLocation++,
//          Irp->Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation++;
//          Irp->CurrentLocation <= (Irp->StackCount + 1);
//          StackPtr++,
//          Irp->CurrentLocation++,
//          Irp->Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation++)
//     {
//         /* Set Pending Returned */
//         Irp->PendingReturned = StackPtr->Control & SL_PENDING_RETURNED;
//
//         /* Check if we failed */
//         if (!NT_SUCCESS(Irp->IoStatus.Status))
//         {
//             /* Check if it was changed by a completion routine */
//             if (Irp->IoStatus.Status != ErrorCode)
//             {
//                 /* Update the error for the current stack */
//                 ErrorCode = Irp->IoStatus.Status;
//                 StackPtr->Control |= SL_ERROR_RETURNED;
//                 LastStackPtr->Parameters.Others.Argument4 = UlongToPtr(ErrorCode);
//                 LastStackPtr->Control |= SL_ERROR_RETURNED;
//             }
//         }
//
//         /* Check if there is a Completion Routine to Call */
//         if ((NT_SUCCESS(Irp->IoStatus.Status) &&
//              (StackPtr->Control & SL_INVOKE_ON_SUCCESS)) ||
//             (!NT_SUCCESS(Irp->IoStatus.Status) &&
//              (StackPtr->Control & SL_INVOKE_ON_ERROR)) ||
//             (Irp->Cancel &&
//              (StackPtr->Control & SL_INVOKE_ON_CANCEL)))
//         {
//             /* Clear the stack location */
//             IopClearStackLocation(StackPtr);
//
//             /* Check for highest-level device completion routines */
//             if (Irp->CurrentLocation == (Irp->StackCount + 1))
//             {
//                 /* Clear the DO, since the current stack location is invalid */
//                 DeviceObject = NULL;
//             }
//             else
//             {
//                 /* Otherwise, return the real one */
//                 DeviceObject = IoGetCurrentIrpStackLocation(Irp)->DeviceObject;
//             }
//
//             /* Call the completion routine */
//             Status = StackPtr->CompletionRoutine(DeviceObject,
//                                                  Irp,
//                                                  StackPtr->Context);
//
//             /* Don't touch the Packet in this case, since it might be gone! */
//             if (Status == STATUS_MORE_PROCESSING_REQUIRED) return;
//         }
//         else
//         {
//             /* Otherwise, check if this is a completed IRP */
//             if ((Irp->CurrentLocation <= Irp->StackCount) &&
//                 (Irp->PendingReturned))
//             {
//                 /* Mark it as pending */
//                 IoMarkIrpPending(Irp);
//             }
//
//             /* Clear the stack location */
//             IopClearStackLocation(StackPtr);
//         }
//     }
//
//     _IRP_TRACE2("[IRP2] loop done\r\n");
//
//     /* Check if the IRP is an associated IRP */
//     if (Irp->Flags & IRP_ASSOCIATED_IRP)
//     {
//         /* Get the master IRP and count */
//         MasterIrp = Irp->AssociatedIrp.MasterIrp;
//         MasterCount = InterlockedDecrement(&MasterIrp->AssociatedIrp.IrpCount);
//
//         /* Free the MDLs */
//         for (Mdl = Irp->MdlAddress; Mdl; Mdl = NextMdl)
//         {
//             /* Go to the next one */
//             NextMdl = Mdl->Next;
//             IoFreeMdl(Mdl);
//         }
//
//         /* Free the IRP itself */
//         IoFreeIrp(Irp);
//
//         /* Complete the Master IRP */
//         if (!MasterCount) IofCompleteRequest(MasterIrp, PriorityBoost);
//         return;
//     }
//
//     /* Check whether we have to reparse */
//     if (Irp->IoStatus.Status == STATUS_REPARSE)
//     {
//         if (Irp->IoStatus.Information > IO_REMOUNT)
//         {
//             /* If that's a reparse tag we understand, save the buffer from deletion */
//             if (Irp->IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT)
//             {
//                 ASSERT(Irp->Tail.Overlay.AuxiliaryBuffer != NULL);
//                 DataBuffer = (PREPARSE_DATA_BUFFER)Irp->Tail.Overlay.AuxiliaryBuffer;
//                 Irp->Tail.Overlay.AuxiliaryBuffer = NULL;
//             }
//             else
//             {
//                 Irp->IoStatus.Status = STATUS_IO_REPARSE_TAG_NOT_HANDLED;
//             }
//         }
//     }
//
//     /* Check if we have an auxiliary buffer */
//     if (Irp->Tail.Overlay.AuxiliaryBuffer)
//     {
//         /* Free it */
//         ExFreePool(Irp->Tail.Overlay.AuxiliaryBuffer);
//         Irp->Tail.Overlay.AuxiliaryBuffer = NULL;
//     }
//
//     /* Check if this is a Paging I/O or Close Operation */
//     if (Irp->Flags & (IRP_PAGING_IO | IRP_CLOSE_OPERATION))
//     {
//         /* Handle a Close Operation or Sync Paging I/O */
//         if (Irp->Flags & (IRP_SYNCHRONOUS_PAGING_IO | IRP_CLOSE_OPERATION))
//         {
//             /* Set the I/O Status and Signal the Event */
//             Flags = Irp->Flags & (IRP_SYNCHRONOUS_PAGING_IO | IRP_PAGING_IO);
//             *Irp->UserIosb = Irp->IoStatus;
//             KeSetEvent(Irp->UserEvent, PriorityBoost, FALSE);
//
//             /* Free the IRP for a Paging I/O Only, Close is handled by us */
//             if (Flags)
//             {
//                 /* If we were using the reserve IRP, then call the appropriate
//                  * free function (to make the IRP available again)
//                  */
//                 if (Irp == IopReserveIrpAllocator.ReserveIrp)
//                 {
//                     IopFreeReserveIrp(PriorityBoost);
//                 }
//                 /* Otherwise, free for real! */
//                 else
//                 {
//                     IoFreeIrp(Irp);
//                 }
//             }
//         }
//         else
//         {
// #if 0
//             /* Page 166 */
//             KeInitializeApc(&Irp->Tail.Apc
//                             &Irp->Tail.Overlay.Thread->Tcb,
//                             Irp->ApcEnvironment,
//                             IopCompletePageWrite,
//                             NULL,
//                             NULL,
//                             KernelMode,
//                             NULL);
//             KeInsertQueueApc(&Irp->Tail.Apc,
//                              NULL,
//                              NULL,
//                              PriorityBoost);
// #else
//             /* Not implemented yet. */
//             UNIMPLEMENTED_DBGBREAK("Not supported!\n");
// #endif
//         }
//
//         /* Get out of here */
//         return;
//     }
//
//     /* Unlock MDL Pages, page 167. */
//     Mdl = Irp->MdlAddress;
//     while (Mdl)
//     {
//         MmUnlockPages(Mdl);
//         Mdl = Mdl->Next;
//     }
//
//     _IRP_TRACE2("[IRP2] mdl done\r\n");
//     /* Check if we should exit because of a Deferred I/O (page 168) */
//     if ((Irp->Flags & IRP_DEFER_IO_COMPLETION) && !(Irp->PendingReturned))
//     {
//         /* Restore the saved reparse buffer for the caller */
//         if (Irp->IoStatus.Status == STATUS_REPARSE &&
//             Irp->IoStatus.Information == IO_REPARSE_TAG_MOUNT_POINT)
//         {
//             Irp->Tail.Overlay.AuxiliaryBuffer = (PCHAR)DataBuffer;
//         }
//
//         /*
//          * Return without queuing the completion APC, since the caller will
//          * take care of doing its own optimized completion at PASSIVE_LEVEL.
//          */
//         return;
//     }
//
//     /* Get the thread and file object */
//     _IRP_TRACE2("[IRP2] APC path\r\n");
//     Thread = Irp->Tail.Overlay.Thread;
//     FileObject = Irp->Tail.Overlay.OriginalFileObject;
//
//     /* Make sure the IRP isn't canceled */
//     if (!Irp->Cancel)
//     {
//         _IRP_TRACE2("[IRP2] KeInitializeApc\r\n");
//         /* Initialize the APC */
//         KeInitializeApc(&Irp->Tail.Apc,
//                         &Thread->Tcb,
//                         Irp->ApcEnvironment,
//                         IopCompleteRequest,
//                         NULL,
//                         NULL,
//                         KernelMode,
//                         NULL);
//
//         _IRP_TRACE2("[IRP2] KeInsertQueueApc\r\n");
//         /* Queue it */
//         KeInsertQueueApc(&Irp->Tail.Apc,
//                          FileObject,
//                          DataBuffer,
//                          PriorityBoost);
//         _IRP_TRACE2("[IRP2] KeInsertQueueApc done\r\n");
//     }
//     else
//     {
//         /* The IRP just got canceled... does a thread still own it? */
//         if (Thread)
//         {
//             /* Yes! There is still hope! Initialize the APC */
//             KeInitializeApc(&Irp->Tail.Apc,
//                             &Thread->Tcb,
//                             Irp->ApcEnvironment,
//                             IopCompleteRequest,
//                             NULL,
//                             NULL,
//                             KernelMode,
//                             NULL);
//
//             /* Queue it */
//             KeInsertQueueApc(&Irp->Tail.Apc,
//                              FileObject,
//                              DataBuffer,
//                              PriorityBoost);
//         }
//         else
//         {
//             /* Nothing left for us to do, kill it */
//             ASSERT(Irp->Cancel);
//             IopCleanupIrp(Irp, FileObject);
//         }
//     }
// }
//
// NTSTATUS
// NTAPI
// IopSynchronousCompletion(IN PDEVICE_OBJECT DeviceObject,
//                          IN PIRP Irp,
//                          IN PVOID Context)
// {
//     if (Irp->PendingReturned)
//         KeSetEvent((PKEVENT)Context, IO_NO_INCREMENT, FALSE);
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoForwardIrpSynchronously(IN PDEVICE_OBJECT DeviceObject,
//                           IN PIRP Irp)
// {
//     KEVENT Event;
//     NTSTATUS Status;
//
//     /* Check if next stack location is available */
//     if (Irp->CurrentLocation > Irp->StackCount || Irp->CurrentLocation <= 1)
//     {
//         /* No more stack location */
//         return FALSE;
//     }
//
//     /* Initialize event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Copy stack location for next driver */
//     IoCopyCurrentIrpStackLocationToNext(Irp);
//
//     /* Set a completion routine, which will signal the event */
//     IoSetCompletionRoutine(Irp, IopSynchronousCompletion, &Event, TRUE, TRUE, TRUE);
//
//     /* Call next driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     /* Check if irp is pending */
//     if (Status == STATUS_PENDING)
//     {
//         /* Yes, wait for its completion */
//         KeWaitForSingleObject(&Event, Suspended, KernelMode, FALSE, NULL);
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoFreeIrp(IN PIRP Irp)
// {
//     PNPAGED_LOOKASIDE_LIST List;
//     PP_NPAGED_LOOKASIDE_NUMBER ListType = LookasideSmallIrpList;
//     PKPRCB Prcb;
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Freeing IRPs %p\n",
//             __FUNCTION__,
//             Irp);
//
//     /* Make sure the Thread IRP list is empty and that it OK to free it */
//     ASSERT(Irp->Type == IO_TYPE_IRP);
//     ASSERT(IsListEmpty(&Irp->ThreadListEntry));
//     ASSERT(Irp->CurrentLocation >= Irp->StackCount);
//
//     /* Get the PRCB */
//     Prcb = KeGetCurrentPrcb();
//
//     /* If this was a lookaside alloc, increment lookaside float */
//     if (Irp->AllocationFlags & IRP_LOOKASIDE_ALLOCATION)
//     {
//         Irp->AllocationFlags &= ~IRP_LOOKASIDE_ALLOCATION;
//         InterlockedIncrement(&Prcb->LookasideIrpFloat);
//     }
//
//     /* If this was a pool alloc, free it with the pool */
//     if (!(Irp->AllocationFlags & IRP_ALLOCATED_FIXED_SIZE))
//     {
//         /* Free it */
//         ExFreePoolWithTag(Irp, TAG_IRP);
//     }
//     else
//     {
//         /* Check if this was a Big IRP */
//         if (Irp->StackCount != 1) ListType = LookasideLargeIrpList;
//
//         /* Use the P List */
//         List = (PNPAGED_LOOKASIDE_LIST)Prcb->PPLookasideList[ListType].P;
//         List->L.TotalFrees++;
//
//         /* Check if the Free was within the Depth or not */
//         if (ExQueryDepthSList(&List->L.ListHead) >= List->L.Depth)
//         {
//             /* Let the balancer know */
//             List->L.FreeMisses++;
//
//             /* Use the L List */
//             List = (PNPAGED_LOOKASIDE_LIST)Prcb->PPLookasideList[ListType].L;
//             List->L.TotalFrees++;
//
//             /* Check if the Free was within the Depth or not */
//             if (ExQueryDepthSList(&List->L.ListHead) >= List->L.Depth)
//             {
//                 /* All lists failed, use the pool */
//                 List->L.FreeMisses++;
//                 ExFreePoolWithTag(Irp, TAG_IRP);
//                 Irp = NULL;
//             }
//         }
//
//         /* The free was within the Depth */
//         if (Irp)
//         {
//             /* Remove the association with the process */
//             if (Irp->AllocationFlags & IRP_QUOTA_CHARGED)
//             {
//                 ExReturnPoolQuota(Irp);
//                 Irp->AllocationFlags &= ~IRP_QUOTA_CHARGED;
//             }
//
//             /* Add it to the lookaside list */
//             InterlockedPushEntrySList(&List->L.ListHead,
//                                       (PSLIST_ENTRY)Irp);
//         }
//     }
// }
//
// /*
//  * @implemented
//  */
// IO_PAGING_PRIORITY
// FASTCALL
// IoGetPagingIoPriority(IN PIRP Irp)
// {
//     IO_PAGING_PRIORITY Priority;
//     ULONG Flags;
//
//     /* Get the flags */
//     Flags = Irp->Flags;
//
//     /* Check what priority it has */
//     if (Flags & IRP_CLASS_CACHE_OPERATION)
//     {
//         /* High priority */
//         Priority = IoPagingPriorityHigh;
//     }
//     else if (Flags & IRP_PAGING_IO)
//     {
//         /* Normal priority */
//         Priority = IoPagingPriorityNormal;
//     }
//     else
//     {
//         /* Invalid -- not a paging IRP */
//         Priority = IoPagingPriorityInvalid;
//     }
//
//     /* Return the priority */
//     return Priority;
// }
//
// /*
//  * @implemented
//  */
// PEPROCESS
// NTAPI
// IoGetRequestorProcess(IN PIRP Irp)
// {
//     /* Return the requestor process */
//     if (Irp->Tail.Overlay.Thread)
//     {
//         if (Irp->ApcEnvironment == OriginalApcEnvironment)
//         {
//             return Irp->Tail.Overlay.Thread->ThreadsProcess;
//         }
//         else if (Irp->ApcEnvironment == AttachedApcEnvironment)
//         {
//             return (PEPROCESS)Irp->Tail.Overlay.Thread->Tcb.ApcState.Process;
//         }
//     }
//
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// IoGetRequestorProcessId(IN PIRP Irp)
// {
//     PEPROCESS Process;
//
//     /* Return the requestor process' id */
//     Process = IoGetRequestorProcess(Irp);
//     if (Process) return PtrToUlong(Process->UniqueProcessId);
//
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoGetRequestorSessionId(IN PIRP Irp,
//                         OUT PULONG pSessionId)
// {
//     PEPROCESS Process;
//
//     /* Return the session */
//     if (Irp->Tail.Overlay.Thread)
//     {
//         Process = Irp->Tail.Overlay.Thread->ThreadsProcess;
//         *pSessionId = MmGetSessionId(Process);
//         return STATUS_SUCCESS;
//     }
//
//     *pSessionId = (ULONG)-1;
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//  * @implemented
//  */
// PIRP
// NTAPI
// IoGetTopLevelIrp(VOID)
// {
//     /* Return the IRP */
//     return (PIRP)PsGetCurrentThread()->TopLevelIrp;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoInitializeIrp(IN PIRP Irp,
//                 IN USHORT PacketSize,
//                 IN CCHAR StackSize)
// {
//     /* Clear it */
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Initializing IRP %p\n",
//             __FUNCTION__,
//             Irp);
//     RtlZeroMemory(Irp, PacketSize);
//
//     /* Set the Header and other data */
//     Irp->Type = IO_TYPE_IRP;
//     Irp->Size = PacketSize;
//     Irp->StackCount = StackSize;
//     Irp->CurrentLocation = StackSize + 1;
//     Irp->ApcEnvironment =  KeGetCurrentThread()->ApcStateIndex;
//     Irp->Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation = (PIO_STACK_LOCATION)(Irp + 1) + StackSize;
//
//     /* Initialize the Thread List */
//     InitializeListHead(&Irp->ThreadListEntry);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// IoIsOperationSynchronous(IN PIRP Irp)
// {
//     BOOLEAN SynchIO;
//     BOOLEAN ForceAsync;
//
//     /* If the IRP requests synchronous paging I/O, if the file object was opened
//        for synchronous I/O, if the IRP_SYNCHRONOUS_API flag is set in the IRP
//        the operation is synchronous */
//     SynchIO = (IoGetCurrentIrpStackLocation(Irp)->FileObject->Flags & FO_SYNCHRONOUS_IO) ||
//               (Irp->Flags & IRP_SYNCHRONOUS_API) || (Irp->Flags & IRP_SYNCHRONOUS_PAGING_IO);
//
//     /* If the IRP requests asynchronous paging I/O, the operation is asynchronous,
//        even if one of the above conditions is true */
//     ForceAsync = (Irp->Flags & IRP_PAGING_IO) && !(Irp->Flags & IRP_SYNCHRONOUS_PAGING_IO);
//
//     /* Check the flags */
//     if (SynchIO && !ForceAsync)
//     {
//         /* Synch API or Paging I/O is OK, as is Sync File I/O */
//         return TRUE;
//     }
//
//     /* Otherwise, it is an asynchronous operation. */
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// IoIsValidNameGraftingBuffer(IN PIRP Irp,
//                             IN PREPARSE_DATA_BUFFER ReparseBuffer)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// PIRP
// NTAPI
// IoMakeAssociatedIrp(IN PIRP Irp,
//                     IN CCHAR StackSize)
// {
//     PIRP AssocIrp;
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Associating IRP %p\n",
//             __FUNCTION__,
//             Irp);
//
//     /* Allocate the IRP */
//     AssocIrp = IoAllocateIrp(StackSize, FALSE);
//     if (!AssocIrp) return NULL;
//
//     /* Set the Flags */
//     AssocIrp->Flags |= IRP_ASSOCIATED_IRP;
//
//     /* Set the Thread */
//     AssocIrp->Tail.Overlay.Thread = Irp->Tail.Overlay.Thread;
//
//     /* Associate them */
//     AssocIrp->AssociatedIrp.MasterIrp = Irp;
//     return AssocIrp;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoQueueThreadIrp(IN PIRP Irp)
// {
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Queueing IRP %p\n",
//             __FUNCTION__,
//             Irp);
//
//     /* Use our inlined routine */
//     IopQueueIrpToThread(Irp);
// }
//
// /*
//  * @implemented
//  * Reference: Chris Cant's "Writing WDM Device Drivers"
//  */
// VOID
// NTAPI
// IoReuseIrp(IN OUT PIRP Irp,
//            IN NTSTATUS Status)
// {
//     UCHAR AllocationFlags;
//     IOTRACE(IO_IRP_DEBUG,
//             "%s - Reusing IRP %p\n",
//             __FUNCTION__,
//             Irp);
//
//     /* Make sure it's OK to reuse it */
//     ASSERT(!Irp->CancelRoutine);
//     ASSERT(IsListEmpty(&Irp->ThreadListEntry));
//
//     /* Get the old flags */
//     AllocationFlags = Irp->AllocationFlags;
//
//     /* Reinitialize the IRP */
//     IoInitializeIrp(Irp, Irp->Size, Irp->StackCount);
//
//     /* Duplicate the data */
//     Irp->IoStatus.Status = Status;
//     Irp->AllocationFlags = AllocationFlags;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoSetTopLevelIrp(IN PIRP Irp)
// {
//     /* Set the IRP */
//     PsGetCurrentThread()->TopLevelIrp = (ULONG_PTR)Irp;
// }
//
// #if defined (_WIN64)
// BOOLEAN
// NTAPI
// IoIs32bitProcess(
//     IN PIRP Irp OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
// #endif
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
