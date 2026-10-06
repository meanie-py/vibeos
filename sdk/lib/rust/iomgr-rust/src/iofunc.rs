use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, _HANDLE_TABLE_ENTRY, _OBJECT_HEADER, _OBJECT_TYPE, _KPRCB,
    _FILE_OBJECT, _DEVICE_OBJECT, _DRIVER_OBJECT, _IRP, _IO_STACK_LOCATION, _KEVENT,
    _FAST_IO_DISPATCH, _MDL, _OBJECT_ATTRIBUTES, _UNICODE_STRING,
    _FILE_OBJECT_EXTENSION, _IO_STATUS_BLOCK__bindgen_ty_1, _VPB,
    PEPROCESS, PETHREAD, POBJECT_HEADER, POBJECT_TYPE, PHANDLE_TABLE, PHANDLE_TABLE_ENTRY,
    PFILE_OBJECT, PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, PIO_STACK_LOCATION, PKEVENT,
    PFAST_IO_DISPATCH, PMDL, POBJECT_ATTRIBUTES, PUNICODE_STRING, PVPB,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE, OB_OPEN_REASON, CREATE_FILE_TYPE, PKNORMAL_ROUTINE,
    _OBJECT_HANDLE_INFORMATION, _IO_STATUS_BLOCK, PIO_STATUS_BLOCK,
    LARGE_INTEGER, PLARGE_INTEGER, ULARGE_INTEGER,
    FILE_INFORMATION_CLASS, FS_INFORMATION_CLASS,
    FILE_BASIC_INFORMATION, PFILE_BASIC_INFORMATION,
    FILE_POSITION_INFORMATION, PFILE_POSITION_INFORMATION,
    FILE_ACCESS_INFORMATION, PFILE_ACCESS_INFORMATION,
    FILE_MODE_INFORMATION, PFILE_MODE_INFORMATION,
    FILE_ALIGNMENT_INFORMATION, PFILE_ALIGNMENT_INFORMATION,
    FILE_ALL_INFORMATION, PFILE_ALL_INFORMATION,
    FILE_RENAME_INFORMATION, PFILE_RENAME_INFORMATION,
    FILE_COMPLETION_INFORMATION, PFILE_COMPLETION_INFORMATION,
    FILE_SEGMENT_ELEMENT,
    FILE_FS_DEVICE_INFORMATION, PFILE_FS_DEVICE_INFORMATION,
    FILE_FS_DRIVER_PATH_INFORMATION, PFILE_FS_DRIVER_PATH_INFORMATION,
    FILE_DIRECTORY_INFORMATION, FILE_FULL_DIR_INFORMATION,
    FILE_ID_FULL_DIR_INFORMATION, FILE_NAMES_INFORMATION,
    FILE_BOTH_DIR_INFORMATION, FILE_ID_BOTH_DIR_INFORMATION,
    IO_COMPLETION_CONTEXT, PIO_COMPLETION_CONTEXT,
    PIO_APC_ROUTINE,
    TARGET_DEVICE_CUSTOM_NOTIFICATION,
    IoAllocateIrp, IoFreeIrp, IoFreeMdl,
    IoGetRelatedDeviceObject, IoGetAttachedDevice, IoAllocateMdl,
    IoGetNextIrpStackLocation, IoAcquireVpbSpinLock, IoReleaseVpbSpinLock,
    IopAbortInterruptedIrp, IopCompleteRequest,
    IopCleanupFailedIrp, IopGetDeviceAttachmentBase, IopAllocateReserveIrp,
    MmIsFileObjectAPagingFile,
    KeWaitForSingleObject, KeClearEvent, KeSetEvent,
    KeInitializeEvent, KeAcquireQueuedSpinLock, KeReleaseQueuedSpinLock,
    ExGetPreviousMode,
    ObReferenceObjectByHandle, ObCloseHandle, ObReferenceObjectByName,
    IoCreateFile, IoCreateFileSpecifyDeviceObjectHint,
    IoSetIoCompletion, IoReportTargetDeviceChange, IoGetRelatedTargetDevice,
    ObReferenceFileObjectForWrite,
    PsGetCurrentThread,
};

use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// C macros inlined
#[inline(always)]
unsafe fn MmGetMdlVirtualAddress(Mdl: PMDL) -> PVOID {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as PVOID
}
#[inline(always)]
unsafe fn MmGetMdlByteCount(Mdl: PMDL) -> ULONG {
    (*Mdl).ByteCount
}

// Extern declarations for FORCEINLINE / macro symbols not in bindgen
// NOTE: Calling conventions must match their definitions in the crate.
// On i386 stdcall != cdecl != fastcall — mismatches cause LLVM ud2.
extern "system" {
    fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn ObDereferenceObject(Object: PVOID);
    fn ExFreePool(P: PVOID);
    fn ExRaiseStatus(Status: NTSTATUS);
    static mut IoFileObjectType: *mut _OBJECT_TYPE;
    static mut ExEventObjectType: *mut _OBJECT_TYPE;
    static mut IoCompletionType: *mut _OBJECT_TYPE;
    static mut IoDriverObjectType: *mut _OBJECT_TYPE;
    #[link_name = "IopLockFileObject_shim"]
    fn IopLockFileObject(FileObject: PFILE_OBJECT, PreviousMode: KPROCESSOR_MODE) -> NTSTATUS;
    #[link_name = "IopUnlockFileObject_shim"]
    fn IopUnlockFileObject(FileObject: PFILE_OBJECT);
    #[link_name = "IopUpdateOperationCount_shim"]
    fn IopUpdateOperationCount(TransferType: ULONG);
    #[link_name = "IopUpdateTransferCount_shim"]
    fn IopUpdateTransferCount(TransferType: ULONG, TransferCount: ULONG);
    #[link_name = "IopQueueIrpToThread_shim"]
    fn IopQueueIrpToThread(Irp: PIRP);
    fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: usize, Tag: ULONG) -> PVOID;
    fn ExAllocatePoolWithQuotaTag(PoolType: ULONG, NumberOfBytes: usize, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

// These are defined as extern "C" in shim_exports.rs — must match.
extern "C" {
    fn ObReferenceObject(Object: PVOID);
    fn InterlockedCompareExchangePointer(Destination: *mut PVOID, Exchange: PVOID, Comparand: PVOID) -> PVOID;
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
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

// Type aliases for missing types
type CCHAR = i8;
type KEVENT = _KEVENT;
type PSID = PVOID;

// IO constants
const FO_SYNCHRONOUS_IO: ULONG = 0x0000002;
const FO_ALERTABLE_IO: ULONG = 0x0000004;
const FO_NO_INTERMEDIATE_BUFFERING: ULONG = 0x0000008;
const FO_WRITE_THROUGH: ULONG = 0x0000010;
const FO_SEQUENTIAL_ONLY: ULONG = 0x0000020;
const FO_NAMED_PIPE: ULONG = 0x00000080;
const FO_MAILSLOT: ULONG = 0x00000200;
const FO_DIRECT_DEVICE_OPEN: ULONG = 0x00000800;
const FO_DELETE_ON_CLOSE: ULONG = 0x00010000;
const FO_OPENED_CASE_SENSITIVE: ULONG = 0x00020000;
const FO_FILE_OBJECT_HAS_EXTENSION: ULONG = 0x0800000;

// IRP constants
const IRP_MJ_DEVICE_CONTROL: u32 = 0x0E;
const IRP_MJ_FILE_SYSTEM_CONTROL: u32 = 0x0D;
const IRP_MJ_QUERY_INFORMATION: u32 = 0x05;
const IRP_MJ_SET_INFORMATION: u32 = 0x06;
const IRP_MJ_QUERY_VOLUME_INFORMATION: u32 = 0x0A;
const IRP_MJ_READ: u32 = 0x03;
const IRP_MJ_WRITE: u32 = 0x04;
const IRP_MJ_LOCK_CONTROL: u32 = 0x11;
const IRP_MJ_DIRECTORY_CONTROL: u32 = 0x0C;
const IRP_MJ_FLUSH_BUFFERS: u32 = 0x09;

const IRP_MN_LOCK: u32 = 0x01;
const IRP_MN_NOTIFY_CHANGE_DIRECTORY: u32 = 0x02;
const IRP_MN_QUERY_DIRECTORY: u32 = 0x01;

const IRP_BUFFERED_IO: u32 = 0x00000010;
const IRP_DEALLOCATE_BUFFER: u32 = 0x00000020;
const IRP_INPUT_OPERATION: u32 = 0x00000040;
const IRP_SYNCHRONOUS_API: u32 = 0x00000004;
const IRP_OB_QUERY_NAME: u32 = 0x00001000;
const IRP_DEFER_IO_COMPLETION: u32 = 0x00000800;
const IRP_PAGING_IO: u32 = 0x00000002;
const IRP_NOCACHE: u32 = 0x00000001;
const IRP_SYNCHRONOUS_PAGING_IO: u32 = 0x00000040;
const IRP_READ_OPERATION: u32 = 0x00000100;

/* wow64-plan M15b: kernel-side IO_STATUS_BLOCK32 completion. A WoW64 guest's
 * IOSB is 8 bytes {Status, Information}; the host kernel writes 16 — an
 * 8-byte overflow past the guest struct plus Information at the wrong offset.
 * wow64.dll latches the guest IOSB pointer into TEB.Instrumentation[0] right
 * before an IOSB-bearing syscall; the Nt* entry consumes the latch (read-and-
 * clear, honored only when it matches the IoStatusBlock argument exactly) and
 * marks the IRP so IopCompleteRequest writes the 32-bit layout at completion —
 * sync AND async, which lets the wow64 thunk stop refusing Event-based I/O.
 *
 * Consumers: NtReadFile / NtWriteFile (M15b) and NtDeviceIoControlFile /
 * NtFsControlFile via IopDeviceFsIoControl (M15c). Each has three obligations,
 * and skipping any one of them is silent rather than loud: probe the guest's
 * block at 8 bytes, mark the IRP, and narrow the write on whatever inline
 * completion path bypasses the IRP entirely (fast I/O). */
pub(crate) const IRP_WOW64_IOSB32: u32 = 0x8000_0000;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct IoStatusBlock32 {
    pub Status: NTSTATUS,
    pub Information: u32,
}

/// A 32-bit kernel never hosts WoW64 guests — compile the capture out on i386
/// (its bindgen EPROCESS has no Wow64Process field).
#[cfg(target_arch = "x86")]
unsafe fn IopCaptureWow64Iosb32(
    _IoStatusBlock: PIO_STATUS_BLOCK,
    _PreviousMode: KPROCESSOR_MODE,
) -> bool {
    false
}

#[cfg(not(target_arch = "x86"))]
unsafe fn IopCaptureWow64Iosb32(
    IoStatusBlock: PIO_STATUS_BLOCK,
    PreviousMode: KPROCESSOR_MODE,
) -> bool {
    if PreviousMode == 0i8 {
        return false;
    }
    let Process = PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS;
    if Process.is_null() || (*Process).Wow64Process.is_null() {
        return false;
    }
    let Teb = ntkrnl_sys::ddk::PsGetCurrentThreadTeb() as *mut ntkrnl_sys::ddk::_TEB;
    if Teb.is_null() {
        return false;
    }
    let Slot = core::ptr::addr_of_mut!((*Teb).Instrumentation) as *mut usize;
    let mut Latch: usize = 0;
    if ntkrnl_sys::probe::safe_copy_raw(
        core::ptr::addr_of_mut!(Latch) as *mut core::ffi::c_void,
        Slot as *const core::ffi::c_void,
        core::mem::size_of::<usize>(),
    )
    .is_err()
        || Latch == 0
    {
        return false;
    }
    // Consume: the latch is strictly per-call — nested I/O (e.g. a future
    // guest-APC path issuing its own read) must never inherit it.
    let Zero: usize = 0;
    let _ = ntkrnl_sys::probe::safe_copy_raw(
        Slot as *mut core::ffi::c_void,
        core::ptr::addr_of!(Zero) as *const core::ffi::c_void,
        core::mem::size_of::<usize>(),
    );
    if Latch != IoStatusBlock as usize {
        extern "C" {
            fn DbgPrint(Format: *const i8, ...) -> u32;
        }
        DbgPrint(
            b"[IOSB32] TEB latch %p != IOSB arg %p - ignoring (wow64.dll bug?)\n\0".as_ptr()
                as *const i8,
            Latch as *const core::ffi::c_void,
            IoStatusBlock,
        );
        return false;
    }
    true
}

// Method constants (from IoControlCode)
const METHOD_BUFFERED: u32 = 0;
const METHOD_IN_DIRECT: u32 = 1;
const METHOD_OUT_DIRECT: u32 = 2;
const METHOD_NEITHER: u32 = 3;

// File access / attribute / option constants
const FILE_ANY_ACCESS: ACCESS_MASK = 0;
const FILE_READ_DATA: ACCESS_MASK = 0x0001;
const FILE_WRITE_DATA: ACCESS_MASK = 0x0002;
const FILE_LIST_DIRECTORY: ACCESS_MASK = 0x0001;
const FILE_ADD_SUBDIRECTORY: ACCESS_MASK = 0x0004;
const FILE_APPEND_DATA: ACCESS_MASK = 0x0004;
const FILE_ATTRIBUTE_DIRECTORY: ULONG = 0x00000010;
const FILE_SHARE_READ: ULONG = 0x00000001;
const FILE_SHARE_WRITE: ULONG = 0x00000002;
const FILE_OPEN: ULONG = 0x00000001;
const FILE_OPEN_FOR_BACKUP_INTENT: ULONG = 0x00004000;
const FILE_SYNCHRONOUS_IO_ALERT: ULONG = 0x00000010;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;
const FILE_NO_INTERMEDIATE_BUFFERING: ULONG = 0x00000008;
const FILE_SEQUENTIAL_ONLY: ULONG = 0x00000004;
const FILE_DELETE_ON_CLOSE: ULONG = 0x00001000;
const FILE_WRITE_THROUGH: ULONG = 0x00000002;
const FILE_USE_FILE_POINTER_POSITION: ULONG = 0xFFFFFFFE;
const FILE_EXISTS: ULONG = 0x00000004;
const FILE_NOTIFY_VALID_MASK: ULONG = 0x00000FFF;

// IO flags
const IO_FORCE_ACCESS_CHECK: ULONG = 0x0001;
const IO_OPEN_TARGET_DIRECTORY: ULONG = 0x0004;
const IO_NO_PARAMETER_CHECKING: ULONG = 0x0100;

// DO flags
const DO_BUFFERED_IO: ULONG = 0x00000004;
const DO_DIRECT_IO: ULONG = 0x00000010;

// SL flags
const SL_RESTART_SCAN: u32 = 0x01;
const SL_RETURN_SINGLE_ENTRY: u32 = 0x02;
const SL_WATCH_TREE: u32 = 0x01;
const SL_FAIL_IMMEDIATELY: u32 = 0x01;
const SL_EXCLUSIVE_LOCK: u32 = 0x02;

// EVENT_MODIFY_STATE
const EVENT_MODIFY_STATE: ACCESS_MASK = 0x0002;

// VPB flags
const VPB_MOUNTED: USHORT = 0x0001;

// Pool tags
const TAG_IO: ULONG = 0x20206F49;       // '  OI'
const TAG_IOBUF: ULONG = 0x55424F49;    // 'fbOI'
const TAG_LOCK: ULONG = 0x6B636C46;     // 'kcLI'

// Status constants
const STATUS_USER_APC: NTSTATUS = 0xC0 as NTSTATUS;
const STATUS_NOT_SAME_DEVICE: NTSTATUS = 0xC00000D4_u32 as i32;

// FSCTL
const FSCTL_DISMOUNT_VOLUME: ULONG = 0x00090020;

// Lock queue
const LockQueueIoDatabaseLock: u32 = 0x0000000A;

// IO Operation Transfer types
const IopOtherTransfer: ULONG = 0x00000002;
const IopReadTransfer: ULONG = 0x00000000;
const IopWriteTransfer: ULONG = 0x00000001;

// IO access/length arrays — declared as single-element to get correct base address;
// actual sizes are in C data segment. Access via raw pointer arithmetic.
// Length arrays are UCHAR in C; Access arrays are ACCESS_MASK (ULONG).
extern "C" {
    static IopQueryOperationAccess: ACCESS_MASK;
    static IopQueryOperationLength: u8;
    static IopSetOperationAccess: ACCESS_MASK;
    static IopSetOperationLength: u8;
    static IopQueryFsOperationAccess: ACCESS_MASK;
    static IopQueryFsOperationLength: u8;
    static IopSetFsOperationAccess: ACCESS_MASK;
    static IopSetFsOperationLength: u8;
}

// Accessor for extern C arrays (flexible arrays declared as single element in Rust)
#[inline(always)]
unsafe fn c_array_get<T: Copy>(base: *const T, index: usize) -> T {
    *base.add(index)
}
// KeGetPreviousMode is extern "system" (stdcall on i386)
extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
}

// Create file type enum values
const CreateFileTypeNone: CREATE_FILE_TYPE = 0;

// File information class enum values
const FileDirectoryInformation: u32 = 1;
const FileFullDirectoryInformation: u32 = 2;
const FileIdFullDirectoryInformation: u32 = 38;
const FileBothDirectoryInformation: u32 = 3;
const FileIdBothDirectoryInformation: u32 = 37;
const FileNamesInformation: u32 = 12;
const FileBasicInformation: u32 = 4;
const FileStandardInformation: u32 = 5;
const FilePositionInformation: u32 = 14;
const FileAccessInformation: u32 = 8;
const FileModeInformation: u32 = 16;
const FileAlignmentInformation: u32 = 17;
const FileAllInformation: u32 = 18;
const FileMaximumInformation: u32 = 64;
const FileRenameInformation: u32 = 10;
const FileLinkInformation: u32 = 11;
const FileCompletionInformation: u32 = 30;
const FileMoveClusterInformation: u32 = 31;

// FS information class enum values
const FileFsDeviceInformation: u32 = 4;
const FileFsDriverPathInformation: u32 = 0x00000009;
const FileFsMaximumInformation: u32 = 9;

// Additional IRP constants
const IRP_MJ_SET_VOLUME_INFORMATION: u32 = 0x0B;
const IRP_MN_UNLOCK_SINGLE: u32 = 0x02;
const IRP_WRITE_OPERATION: u32 = 0x00000200;
const SL_WRITE_THROUGH: u8 = 0x04;

// Additional file/device constants
const FILE_WRITE_TO_END_OF_FILE: ULONG = 0xFFFFFFFF;
const FILE_DEVICE_IS_MOUNTED: ULONG = 0x00000020;
const FILE_DEVICE_NETWORK_FILE_SYSTEM: ULONG = 0x00000014;
const IO_COMPLETION_MODIFY_STATE: ACCESS_MASK = 0x0002;
const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;

// Pool tags
const IOC_TAG: ULONG = 0x54434F49; // 'TCOI'

// GUID_IO_VOLUME_NAME_CHANGE
const GUID_IO_VOLUME_NAME_CHANGE: ntkrnl_sys::ddk::_GUID = ntkrnl_sys::ddk::_GUID {
    Data1: 0x2DE97F83,
    Data2: 0x4C06,
    Data3: 0x11D2,
    Data4: [0xA5, 0x32, 0x00, 0x60, 0x97, 0x13, 0x05, 0x5A],
};

// IO read/write access
const IoReadAccess: u32 = 0;
const IoWriteAccess: u32 = 1;

// CC globals
extern "system" {
    static mut CcDataFlushes: ULONG;
    static mut CcDataPages: LONG;
}

#[no_mangle]
#[used]
pub static mut IoPageReadIrpAllocationFailure: LONG = 0;
#[no_mangle]
#[used]
pub static mut IoPageReadNonPagefileIrpAllocationFailure: LONG = 0;

unsafe fn IopCleanupAfterException(
    FileObject: PFILE_OBJECT,
    Irp: PIRP,
    Event: PKEVENT,
    LocalEvent: PKEVENT,
) {
    if !Irp.is_null() {
        if !(*Irp).AssociatedIrp.SystemBuffer.is_null() {
            ExFreePool((*Irp).AssociatedIrp.SystemBuffer);
        }
        if !(*Irp).MdlAddress.is_null() {
            IoFreeMdl((*Irp).MdlAddress);
        }
        IoFreeIrp(Irp);
    }
    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        IopUnlockFileObject(FileObject);
    }
    if !Event.is_null() {
        ObDereferenceObject(Event as PVOID);
    }
    if !LocalEvent.is_null() {
        ExFreePool(LocalEvent as PVOID);
    }
    ObDereferenceObject(FileObject as PVOID);
}

unsafe fn IopFinalizeAsynchronousIo(
    SynchStatus: NTSTATUS,
    Event: PKEVENT,
    Irp: PIRP,
    PreviousMode: KPROCESSOR_MODE,
    KernelIosb: PIO_STATUS_BLOCK,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let mut FinalStatus = SynchStatus;
    if FinalStatus == STATUS_PENDING {
        FinalStatus = KeWaitForSingleObject(
            Event as *mut core::ffi::c_void,
            0,
            PreviousMode,
            0u8,
            core::ptr::null_mut(),
        );
        if FinalStatus == STATUS_USER_APC {
            IopAbortInterruptedIrp(Event, Irp);
        }
        FinalStatus = (*KernelIosb).__bindgen_anon_1.Status;
    }
    write_user_nofail(IoStatusBlock, *KernelIosb, PreviousMode);
    ExFreePool(Event as PVOID);
    FinalStatus
}

unsafe fn IopPerformSynchronousRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    mut FileObject: PFILE_OBJECT,
    Deferred: BOOLEAN,
    PreviousMode: KPROCESSOR_MODE,
    SynchIo: BOOLEAN,
    TransferType: u32,
) -> NTSTATUS {
    IopQueueIrpToThread(Irp);
    IopUpdateOperationCount(TransferType);
    let mut Status = IoCallDriver(DeviceObject, Irp);
    if Deferred != 0 {
        if Status != STATUS_PENDING {
            let mut NormalRoutine: PKNORMAL_ROUTINE = None;
            let mut NormalContext: PVOID = core::ptr::null_mut();
            let mut OldIrql: KIRQL = 0;
            // [guard] KeRaiseIrql(APC_LEVEL, core::ptr::addr_of_mut!(OldIrql));
            let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL) };
            IopCompleteRequest(
                core::ptr::addr_of_mut!((*Irp).Tail.Apc),
                core::ptr::addr_of_mut!(NormalRoutine),
                core::ptr::addr_of_mut!(NormalContext),
                core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
                core::ptr::addr_of_mut!(NormalContext),
            );
            drop(irql_guard);
        }
    }
    if SynchIo != 0 {
        if Status == STATUS_PENDING {
            Status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as *mut core::ffi::c_void,
                0,
                PreviousMode,
                ((*FileObject).Flags & FO_ALERTABLE_IO != 0) as u8,
                core::ptr::null_mut(),
            );
            if Status == STATUS_ALERTED || Status == STATUS_USER_APC {
                IopAbortInterruptedIrp(core::ptr::addr_of_mut!((*FileObject).Event), Irp);
            }
            Status = (*FileObject).FinalStatus;
        }
        IopUnlockFileObject(FileObject);
    }
    Status
}

unsafe fn IopDeviceFsIoControl(
    DeviceHandle: HANDLE,
    Event: HANDLE,
    UserApcRoutine: PIO_APC_ROUTINE,
    UserApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    IoControlCode: ULONG,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    IsDevIoCtl: BOOLEAN,
) -> NTSTATUS {
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut StackPtr: PIO_STACK_LOCATION;
    let mut EventObject: PKEVENT = core::ptr::null_mut();
    let mut LockedForSynch: BOOLEAN = 0u8;
    let AccessType: ULONG = IoControlCode & 3;
    let mut HandleInformation: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let PoolType: u32 = if IsDevIoCtl != 0 {
        NonPagedPoolCacheAligned
    } else {
        NonPagedPool
    };

    // wow64-plan M15c: same latch NtReadFile/NtWriteFile consume. Must run
    // before any early return -- it is read-and-clear, so leaving it set would
    // let the NEXT IOSB-bearing syscall on this thread inherit a stale address.
    let Iosb32 = IopCaptureWow64Iosb32(IoStatusBlock, PreviousMode);

    if PreviousMode != 0i8 {
        // A WoW64 guest's IOSB is 8 bytes, not 16. Probing for 16 would reject
        // a guest block sitting in the last 8 bytes of a valid page.
        let (IosbSize, IosbAlign) = if Iosb32 {
            (
                core::mem::size_of::<IoStatusBlock32>(),
                core::mem::align_of::<IoStatusBlock32>(),
            )
        } else {
            (
                core::mem::size_of::<_IO_STATUS_BLOCK>(),
                core::mem::align_of::<_IO_STATUS_BLOCK>(),
            )
        };
        if let Err(e) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            IosbSize,
            IosbAlign,
            PreviousMode,
        ) { return e; }
        if AccessType == METHOD_BUFFERED {
            if !OutputBuffer.is_null() {
                if let Err(e) = probe_user_buffer(
                    OutputBuffer,
                    OutputBufferLength as usize,
                    core::mem::size_of::<CHAR>(),
                    PreviousMode,
                ) { return e; }
            }
        }
        if AccessType != METHOD_NEITHER {
            if !InputBuffer.is_null() {
                if let Err(e) = probe_user_buffer(
                    InputBuffer,
                    InputBufferLength as usize,
                    core::mem::size_of::<CHAR>(),
                    PreviousMode,
                ) { return e; }
            }
        }
    }

    let mut Status = ObReferenceObjectByHandle(
        DeviceHandle,
        0,
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::addr_of_mut!(HandleInformation),
    );
    if Status < 0 {
        return Status;
    }

    if !(*FileObject).CompletionContext.is_null() && UserApcRoutine.is_some() {
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    if PreviousMode != 0i8 {
        let DesiredAccess: ACCESS_MASK = (IoControlCode >> 14) & 3;
        if DesiredAccess != FILE_ANY_ACCESS
            && (HandleInformation.GrantedAccess & DesiredAccess) != DesiredAccess
        {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_ACCESS_DENIED;
        }
    }

    if Event != core::ptr::null_mut() {
        Status = ObReferenceObjectByHandle(
            Event,
            EVENT_MODIFY_STATE,
            ExEventObjectType,
            PreviousMode,
            core::ptr::addr_of_mut!(EventObject) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        KeClearEvent(EventObject);
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            if !EventObject.is_null() {
                ObDereferenceObject(EventObject as PVOID);
            }
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        LockedForSynch = 1u8;
    }

    if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
        DeviceObject = IoGetAttachedDevice((*FileObject).DeviceObject);
    } else {
        DeviceObject = IoGetRelatedDeviceObject(FileObject);
    }

    if IsDevIoCtl != 0 {
        let FastIoDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;
        if !FastIoDispatch.is_null()
            && (*FastIoDispatch).FastIoDeviceControl.is_some()
        {
            let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
            if PreviousMode != 0i8 && !OutputBuffer.is_null() {
                if AccessType == METHOD_IN_DIRECT {
                    if let Err(e) = probe_user_buffer(
                        OutputBuffer,
                        OutputBufferLength as usize,
                        core::mem::size_of::<CHAR>(),
                        PreviousMode,
                    ) {
                        IopCleanupAfterException(FileObject, core::ptr::null_mut(), EventObject, core::ptr::null_mut());
                        return e;
                    }
                } else if AccessType == METHOD_OUT_DIRECT {
                    if let Err(e) = probe_user_buffer(
                        OutputBuffer,
                        OutputBufferLength as usize,
                        core::mem::size_of::<CHAR>(),
                        PreviousMode,
                    ) {
                        IopCleanupAfterException(FileObject, core::ptr::null_mut(), EventObject, core::ptr::null_mut());
                        return e;
                    }
                }
            }

            if IoControlCode == FSCTL_DISMOUNT_VOLUME {
                InterlockedIncrement(core::ptr::addr_of_mut!((*SharedUserData).DismountCount) as *mut LONG);
            }

            if ((*FastIoDispatch).FastIoDeviceControl.unwrap())(
                FileObject,
                1u8,
                InputBuffer,
                InputBufferLength,
                OutputBuffer,
                OutputBufferLength,
                IoControlCode,
                core::ptr::addr_of_mut!(KernelIosb),
                DeviceObject,
            ) != 0
            {
                let mut CompletionInfo: IO_COMPLETION_CONTEXT = core::mem::zeroed();
                // The fast-io path completes inline and never builds an IRP, so
                // the IRP_WOW64_IOSB32 flag below cannot reach it -- it has to
                // narrow the block itself.
                if Iosb32 {
                    write_user_nofail(
                        IoStatusBlock as *mut IoStatusBlock32,
                        IoStatusBlock32 {
                            Status: KernelIosb.__bindgen_anon_1.Status,
                            Information: KernelIosb.Information as u32,
                        },
                        PreviousMode,
                    );
                } else {
                    write_user_nofail(IoStatusBlock, KernelIosb, PreviousMode);
                }

                if !(*FileObject).CompletionContext.is_null() {
                    CompletionInfo = *((*FileObject).CompletionContext);
                }
                if !Event.is_null() {
                    KeSetEvent(EventObject, 0, 0u8);
                    ObDereferenceObject(EventObject as PVOID);
                }
                if LockedForSynch != 0 {
                    IopUnlockFileObject(FileObject);
                }
                if !CompletionInfo.Port.is_null() && !UserApcContext.is_null() {
                    if IoSetIoCompletion(
                        CompletionInfo.Port,
                        CompletionInfo.Key,
                        UserApcContext,
                        KernelIosb.__bindgen_anon_1.Status,
                        KernelIosb.Information,
                        1u8,
                    ) < 0
                    {
                        KernelIosb.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
                    }
                }
                ObDereferenceObject(FileObject as PVOID);
                return KernelIosb.__bindgen_anon_1.Status;
            }
        }
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, EventObject, core::ptr::null_mut());
    }

    (*Irp).UserIosb = IoStatusBlock;
    (*Irp).UserEvent = EventObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = UserApcRoutine;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = UserApcContext;
    (*Irp).Cancel = 0u8;
    (*Irp).CancelRoutine = None;
    (*Irp).PendingReturned = 0u8;
    (*Irp).RequestorMode = PreviousMode;
    (*Irp).MdlAddress = core::ptr::null_mut();
    (*Irp).AssociatedIrp.SystemBuffer = core::ptr::null_mut();
    (*Irp).Flags = 0;
    (*Irp).Tail.Overlay.AuxiliaryBuffer = core::ptr::null_mut();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).FileObject = FileObject;
    (*StackPtr).MajorFunction = if IsDevIoCtl != 0 {
        IRP_MJ_DEVICE_CONTROL
    } else {
        IRP_MJ_FILE_SYSTEM_CONTROL
    } as u8;
    (*StackPtr).MinorFunction = 0;
    (*StackPtr).Control = 0;
    (*StackPtr).Flags = 0;
    (*StackPtr).Parameters.DeviceIoControl.Type3InputBuffer = core::ptr::null_mut();
    (*StackPtr).Parameters.DeviceIoControl.IoControlCode = IoControlCode;
    (*StackPtr).Parameters.DeviceIoControl.InputBufferLength = InputBufferLength;
    (*StackPtr).Parameters.DeviceIoControl.OutputBufferLength = OutputBufferLength;

    match AccessType {
        METHOD_BUFFERED => {
            let BufferLength = if InputBufferLength > OutputBufferLength {
                InputBufferLength
            } else {
                OutputBufferLength
            };
            if BufferLength != 0 {
                let alloc_result: Result<_, NTSTATUS> = (|| {
                    let buffer = ExAllocatePoolWithQuotaTag(
                        PoolType,
                        BufferLength as SIZE_T,
                        TAG_IOBUF,
                    );
                    if buffer.is_null() {
                        return Err(STATUS_INSUFFICIENT_RESOURCES);
                    }
                    (*Irp).AssociatedIrp.SystemBuffer = buffer;
                    if !InputBuffer.is_null() {
                        core::ptr::copy_nonoverlapping(
                            InputBuffer as *const u8,
                            buffer as *mut u8,
                            InputBufferLength as usize,
                        );
                    }
                    (*Irp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
                    if !OutputBuffer.is_null() {
                        (*Irp).Flags |= IRP_INPUT_OPERATION;
                    }
                    (*Irp).UserBuffer = OutputBuffer;
                    Ok(())
                })();
                if let Err(e) = alloc_result {
                    IopCleanupAfterException(FileObject, Irp, EventObject, core::ptr::null_mut());
                    return e;
                }
            } else {
                (*Irp).UserBuffer = core::ptr::null_mut();
            }
        }
        METHOD_IN_DIRECT | METHOD_OUT_DIRECT => {
            let alloc_result: Result<_, NTSTATUS> = (|| {
                if InputBufferLength != 0 && !InputBuffer.is_null() {
                    let buffer = ExAllocatePoolWithQuotaTag(
                        PoolType,
                        InputBufferLength as SIZE_T,
                        TAG_IOBUF,
                    );
                    if buffer.is_null() {
                        return Err(STATUS_INSUFFICIENT_RESOURCES);
                    }
                    (*Irp).AssociatedIrp.SystemBuffer = buffer;
                    core::ptr::copy_nonoverlapping(
                        InputBuffer as *const u8,
                        buffer as *mut u8,
                        InputBufferLength as usize,
                    );
                    (*Irp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
                }
                if OutputBufferLength != 0 {
                    let mdl = IoAllocateMdl(
                        OutputBuffer,
                        OutputBufferLength,
                        0u8,
                        0u8,
                        Irp,
                    );
                    if mdl.is_null() {
                        return Err(STATUS_INSUFFICIENT_RESOURCES);
                    }
                    (*Irp).MdlAddress = mdl;
                    let st = ntkrnl_sys::probe::MmTryProbeAndLockPages(
                        mdl,
                        PreviousMode,
                        if AccessType == METHOD_IN_DIRECT {
                            IoReadAccess
                        } else {
                            IoWriteAccess
                        },
                    );
                    if st < 0 {
                        return Err(st);
                    }
                }
                Ok(())
            })();
            if let Err(e) = alloc_result {
                IopCleanupAfterException(FileObject, Irp, EventObject, core::ptr::null_mut());
                return e;
            }
        }
        METHOD_NEITHER => {
            (*Irp).UserBuffer = OutputBuffer;
            (*StackPtr).Parameters.DeviceIoControl.Type3InputBuffer = InputBuffer;
        }
        _ => {}
    }

    if IsDevIoCtl == 0 {
        (*Irp).Flags |= IRP_DEFER_IO_COMPLETION;
    }

    // After the METHOD_* arms above, which ASSIGN Flags rather than OR into it.
    if Iosb32 {
        (*Irp).Flags |= IRP_WOW64_IOSB32;
    }

    if IoControlCode == FSCTL_DISMOUNT_VOLUME {
        InterlockedIncrement(core::ptr::addr_of_mut!((*SharedUserData).DismountCount) as *mut LONG);
    }

    IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        (IsDevIoCtl == 0) as u8,
        PreviousMode,
        LockedForSynch,
        IopOtherTransfer,
    )
}

unsafe fn IopQueryDeviceInformation(
    FileObject: PFILE_OBJECT,
    InformationClass: ULONG,
    Length: ULONG,
    Information: PVOID,
    ReturnedLength: *mut ULONG,
    File: BOOLEAN,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Irp: PIRP;
    let DeviceObject: PDEVICE_OBJECT;
    let mut StackPtr: PIO_STACK_LOCATION;
    let mut LocalEvent: BOOLEAN = 0u8;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    ObReferenceObject(FileObject as PVOID);

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        IopLockFileObject(FileObject, 0i8);
        KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    } else {
        KeInitializeEvent(
            core::ptr::addr_of_mut!(Event),
            1,
            0u8,
        );
        LocalEvent = 1u8;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), core::ptr::null_mut());
    }

    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).RequestorMode = 0i8;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).UserIosb = core::ptr::addr_of_mut!(IoStatusBlock);
    (*Irp).UserEvent = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(Event)
    } else {
        core::ptr::null_mut()
    };
    (*Irp).Flags = if LocalEvent != 0 {
        IRP_SYNCHRONOUS_API
    } else {
        0
    };
    (*Irp).Flags |= IRP_BUFFERED_IO;
    (*Irp).AssociatedIrp.SystemBuffer = Information;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = if File != 0 {
        IRP_MJ_QUERY_INFORMATION
    } else {
        IRP_MJ_QUERY_VOLUME_INFORMATION
    } as u8;
    (*StackPtr).FileObject = FileObject;

    if File != 0 {
        (*StackPtr).Parameters.QueryFile.FileInformationClass = InformationClass as u32;
        (*StackPtr).Parameters.QueryFile.Length = Length;
    } else {
        (*StackPtr).Parameters.QueryVolume.FsInformationClass = InformationClass as u32;
        (*StackPtr).Parameters.QueryVolume.Length = Length;
    }

    IopQueueIrpToThread(Irp);
    Status = IoCallDriver(DeviceObject, Irp);

    if LocalEvent == 0 {
        if Status == STATUS_PENDING {
            Status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as *mut core::ffi::c_void,
                0,
                0i8,
                ((*FileObject).Flags & FO_ALERTABLE_IO != 0) as u8,
                core::ptr::null_mut(),
            );
            if Status == STATUS_ALERTED {
                IopAbortInterruptedIrp(core::ptr::addr_of_mut!((*FileObject).Event), Irp);
            }
            Status = (*FileObject).FinalStatus;
        }
        IopUnlockFileObject(FileObject);
    } else if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            0,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    *ReturnedLength = IoStatusBlock.Information as ULONG;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopGetFileInformation(
    FileObject: PFILE_OBJECT,
    Length: ULONG,
    FileInfoClass: FILE_INFORMATION_CLASS,
    Buffer: PVOID,
    ReturnedLength: *mut ULONG,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let Buffer = opaque_ptr(Buffer);
    let ReturnedLength = opaque_ptr(ReturnedLength);
    let Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let Stack: PIO_STACK_LOCATION;
    let DeviceObject: PDEVICE_OBJECT;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    ObReferenceObject(FileObject as PVOID);
    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), 1, 0u8);

    (*Irp).UserIosb = core::ptr::addr_of_mut!(IoStatusBlock);
    (*Irp).UserEvent = core::ptr::addr_of_mut!(Event);
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).RequestorMode = 0i8;
    (*Irp).AssociatedIrp.SystemBuffer = Buffer;
    (*Irp).Flags = IRP_SYNCHRONOUS_API | IRP_BUFFERED_IO | IRP_OB_QUERY_NAME;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();

    Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MajorFunction = IRP_MJ_QUERY_INFORMATION as u8;
    (*Stack).FileObject = FileObject;
    (*Stack).Parameters.QueryFile.FileInformationClass = FileInfoClass as u32;
    (*Stack).Parameters.QueryFile.Length = Length;

    IopQueueIrpToThread(Irp);
    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            0,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    *ReturnedLength = IoStatusBlock.Information as ULONG;
    Status
}

unsafe fn IopGetBasicInformationFile(
    FileObject: PFILE_OBJECT,
    BasicInfo: PFILE_BASIC_INFORMATION,
) -> NTSTATUS {
    let mut ReturnedLength: ULONG = 0;
    let DeviceObject: PDEVICE_OBJECT = IoGetRelatedDeviceObject(FileObject);
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if !(*DeviceObject).DriverObject.is_null()
        && !(*(*DeviceObject).DriverObject).FastIoDispatch.is_null()
        && (*(*(*DeviceObject).DriverObject).FastIoDispatch).FastIoQueryBasicInfo.is_some()
        && ((*(*(*DeviceObject).DriverObject).FastIoDispatch).FastIoQueryBasicInfo.unwrap())(
            FileObject,
            (((*FileObject).Flags & FO_SYNCHRONOUS_IO) != 0) as u8,
            BasicInfo,
            core::ptr::addr_of_mut!(IoStatusBlock),
            DeviceObject,
        ) != 0
    {
        return IoStatusBlock.__bindgen_anon_1.Status;
    }

    IopGetFileInformation(
        FileObject,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
        BasicInfo as PVOID,
        core::ptr::addr_of_mut!(ReturnedLength),
    )
}

unsafe fn IopOpenLinkOrRenameTarget(
    Handle: *mut HANDLE,
    Irp: PIRP,
    RenameInfo: PFILE_RENAME_INFORMATION,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut TargetHandle: HANDLE = core::ptr::null_mut();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let Stack: PIO_STACK_LOCATION;
    let mut TargetFileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BasicInfo: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut HandleInformation: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut DesiredAccess: ACCESS_MASK = FILE_WRITE_DATA;

    if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN == 0 {
        Status = IopGetBasicInformationFile(FileObject, core::ptr::addr_of_mut!(BasicInfo));
        if Status < 0 {
            return Status;
        }
        if BasicInfo.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            DesiredAccess = FILE_ADD_SUBDIRECTORY;
        }
    }

    FileName.Buffer = (*RenameInfo).FileName.as_mut_ptr();
    FileName.Length = (*RenameInfo).FileNameLength as u16;
    FileName.MaximumLength = (*RenameInfo).FileNameLength as u16;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = (*RenameInfo).RootDirectory;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(FileName);
    ObjectAttributes.Attributes = if (*FileObject).Flags & FO_OPENED_CASE_SENSITIVE != 0 {
        0
    } else {
        OBJ_CASE_INSENSITIVE
    } | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    if (*FileObject).Flags & FO_FILE_OBJECT_HAS_EXTENSION != 0 {
        let FileObjectExtension = (*FileObject).FileObjectExtension as *mut _FILE_OBJECT_EXTENSION;
        Status = IoCreateFileSpecifyDeviceObjectHint(
            core::ptr::addr_of_mut!(TargetHandle),
            DesiredAccess | SYNCHRONIZE,
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(IoStatusBlock),
            core::ptr::null_mut(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_OPEN,
            FILE_OPEN_FOR_BACKUP_INTENT,
            core::ptr::null_mut(),
            0,
            CreateFileTypeNone,
            core::ptr::null_mut(),
            IO_FORCE_ACCESS_CHECK | IO_OPEN_TARGET_DIRECTORY | IO_NO_PARAMETER_CHECKING,
            (*FileObjectExtension).TopDeviceObjectHint as PVOID,
        );
    } else {
        Status = IoCreateFile(
            core::ptr::addr_of_mut!(TargetHandle),
            DesiredAccess | SYNCHRONIZE,
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(IoStatusBlock),
            core::ptr::null_mut(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_OPEN,
            FILE_OPEN_FOR_BACKUP_INTENT,
            core::ptr::null_mut(),
            0,
            CreateFileTypeNone,
            core::ptr::null_mut(),
            IO_FORCE_ACCESS_CHECK | IO_OPEN_TARGET_DIRECTORY | IO_NO_PARAMETER_CHECKING,
        );
    }

    if Status < 0 {
        return Status;
    }

    Stack = IoGetNextIrpStackLocation(Irp);
    if (*Stack).Parameters.SetFile.FileInformationClass == FileLinkInformation
        && (*RenameInfo).ReplaceIfExists == 0
        && IoStatusBlock.Information == FILE_EXISTS as _
    {
        ObCloseHandle(TargetHandle, 0i8);
        return STATUS_OBJECT_NAME_COLLISION;
    }

    Status = ObReferenceObjectByHandle(
        TargetHandle,
        FILE_WRITE_DATA,
        IoFileObjectType,
        0i8,
        core::ptr::addr_of_mut!(TargetFileObject) as *mut PVOID,
        core::ptr::addr_of_mut!(HandleInformation),
    );
    if Status < 0 {
        ObCloseHandle(TargetHandle, 0i8);
        return Status;
    }

    ObDereferenceObject(TargetFileObject as PVOID);
    if IoGetRelatedDeviceObject(TargetFileObject) != IoGetRelatedDeviceObject(FileObject) {
        ObCloseHandle(TargetHandle, 0i8);
        return STATUS_NOT_SAME_DEVICE;
    }

    (*Stack).Parameters.SetFile.FileObject = TargetFileObject;
    *Handle = TargetHandle;

    STATUS_SUCCESS
}

unsafe fn IopGetFileMode(FileObject: PFILE_OBJECT) -> ULONG {
    let mut Mode: ULONG = 0;
    if (*FileObject).Flags & FO_WRITE_THROUGH != 0 {
        Mode |= FILE_WRITE_THROUGH;
    }
    if (*FileObject).Flags & FO_SEQUENTIAL_ONLY != 0 {
        Mode |= FILE_SEQUENTIAL_ONLY;
    }
    if (*FileObject).Flags & FO_NO_INTERMEDIATE_BUFFERING != 0 {
        Mode |= FILE_NO_INTERMEDIATE_BUFFERING;
    }
    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        if (*FileObject).Flags & FO_ALERTABLE_IO != 0 {
            Mode |= FILE_SYNCHRONOUS_IO_ALERT;
        } else {
            Mode |= FILE_SYNCHRONOUS_IO_NONALERT;
        }
    }
    if (*FileObject).Flags & FO_DELETE_ON_CLOSE != 0 {
        Mode |= FILE_DELETE_ON_CLOSE;
    }
    Mode
}

unsafe fn IopGetMountFlag(DeviceObject: PDEVICE_OBJECT) -> BOOLEAN {
    let mut OldIrql: KIRQL = 0;
    let Vpb: PVPB;
    let mut Mounted: BOOLEAN = 0u8;

    IoAcquireVpbSpinLock(core::ptr::addr_of_mut!(OldIrql));
    Vpb = (*DeviceObject).Vpb;
    if !Vpb.is_null() && ((*Vpb).Flags & VPB_MOUNTED != 0) {
        Mounted = 1u8;
    }
    IoReleaseVpbSpinLock(OldIrql);
    Mounted
}

unsafe fn IopVerifyDriverObjectOnStack(
    DeviceObject: PDEVICE_OBJECT,
    DriverObject: PDRIVER_OBJECT,
) -> BOOLEAN {
    let mut StackDO: PDEVICE_OBJECT = IopGetDeviceAttachmentBase(DeviceObject);
    while !StackDO.is_null() {
        if (*StackDO).DriverObject == DriverObject {
            return 1u8;
        }
        StackDO = (*StackDO).AttachedDevice;
    }
    0u8
}

unsafe fn IopGetDriverPathInformation(
    FileObject: PFILE_OBJECT,
    DriverPathInfo: PFILE_FS_DRIVER_PATH_INFORMATION,
    Length: ULONG,
) -> NTSTATUS {
    let mut OldIrql: KIRQL;
    let mut Status: NTSTATUS;
    let mut DriverName: _UNICODE_STRING = core::mem::zeroed();
    let mut DriverObject: PDRIVER_OBJECT = core::ptr::null_mut();

    if (Length as usize)
        - core::mem::offset_of!(FILE_FS_DRIVER_PATH_INFORMATION, DriverName)
        < (*DriverPathInfo).DriverNameLength as usize
    {
        return STATUS_INVALID_PARAMETER;
    }

    DriverName.Length = (*DriverPathInfo).DriverNameLength as USHORT;
    DriverName.MaximumLength = (*DriverPathInfo).DriverNameLength as USHORT;
    DriverName.Buffer = (*DriverPathInfo).DriverName.as_mut_ptr();

    Status = ObReferenceObjectByName(
        core::ptr::addr_of_mut!(DriverName),
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        0,
        IoDriverObjectType,
        0i8,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(DriverObject) as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock as _);
    if !(*FileObject).Vpb.is_null() && !(*(*FileObject).Vpb).DeviceObject.is_null() {
        (*DriverPathInfo).DriverInPath =
            IopVerifyDriverObjectOnStack((*(*FileObject).Vpb).DeviceObject, DriverObject);
    } else {
        (*DriverPathInfo).DriverInPath =
            IopVerifyDriverObjectOnStack((*FileObject).DeviceObject, DriverObject);
    }
    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock as _, OldIrql);

    ObDereferenceObject(DriverObject as PVOID);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoSynchronousPageWrite(
    FileObject: PFILE_OBJECT,
    Mdl: PMDL,
    Offset: PLARGE_INTEGER,
    Event: PKEVENT,
    StatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let Mdl = opaque_ptr(Mdl);
    let Offset = opaque_ptr(Offset);
    let Event = opaque_ptr(Event);
    let StatusBlock = opaque_ptr(StatusBlock);
    let Irp: PIRP;
    let StackPtr: PIO_STACK_LOCATION;
    let DeviceObject: PDEVICE_OBJECT;

    if !(*FileObject).SectionObjectPointer.is_null()
        && !(*(*FileObject).SectionObjectPointer).SharedCacheMap.is_null()
    {
        InterlockedIncrement(core::ptr::addr_of_mut!(CcDataFlushes) as *mut LONG);
        let pages = ((MmGetMdlByteCount(Mdl) + 0xFFF) >> 12) as LONG;
        InterlockedExchangeAdd(core::ptr::addr_of_mut!(CcDataPages) as *mut LONG, pages);
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*Irp).MdlAddress = Mdl;
    (*Irp).UserBuffer = MmGetMdlVirtualAddress(Mdl);
    (*Irp).UserIosb = StatusBlock;
    (*Irp).UserEvent = Event;
    (*Irp).RequestorMode = 0i8;
    (*Irp).Flags = IRP_PAGING_IO | IRP_NOCACHE | IRP_SYNCHRONOUS_PAGING_IO;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();

    (*StackPtr).Parameters.Write.Length = MmGetMdlByteCount(Mdl);
    (*StackPtr).Parameters.Write.ByteOffset = *Offset;
    (*StackPtr).MajorFunction = IRP_MJ_WRITE as u8;
    (*StackPtr).FileObject = FileObject;

    IoCallDriver(DeviceObject, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn IoPageRead(
    FileObject: PFILE_OBJECT,
    Mdl: PMDL,
    Offset: PLARGE_INTEGER,
    Event: PKEVENT,
    StatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let Mdl = opaque_ptr(Mdl);
    let Offset = opaque_ptr(Offset);
    let Event = opaque_ptr(Event);
    let StatusBlock = opaque_ptr(StatusBlock);
    let mut Irp: PIRP;
    let StackPtr: PIO_STACK_LOCATION;
    let DeviceObject: PDEVICE_OBJECT;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        if MmIsFileObjectAPagingFile(FileObject) != 0 {
            InterlockedExchangeAdd(
                core::ptr::addr_of_mut!(IoPageReadIrpAllocationFailure) as *mut LONG,
                1,
            );
            Irp = IopAllocateReserveIrp((*DeviceObject).StackSize as CCHAR);
        } else {
            InterlockedExchangeAdd(
                core::ptr::addr_of_mut!(IoPageReadNonPagefileIrpAllocationFailure) as *mut LONG,
                1,
            );
        }
        if Irp.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*Irp).MdlAddress = Mdl;
    (*Irp).UserBuffer = MmGetMdlVirtualAddress(Mdl);
    (*Irp).UserIosb = StatusBlock;
    (*Irp).UserEvent = Event;
    (*Irp).RequestorMode = 0i8;
    (*Irp).Flags = IRP_PAGING_IO | IRP_NOCACHE | IRP_SYNCHRONOUS_PAGING_IO | IRP_INPUT_OPERATION;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();

    (*StackPtr).Parameters.Read.Length = MmGetMdlByteCount(Mdl);
    (*StackPtr).Parameters.Read.ByteOffset = *Offset;
    (*StackPtr).MajorFunction = IRP_MJ_READ as u8;
    (*StackPtr).FileObject = FileObject;

    IoCallDriver(DeviceObject, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn IoQueryFileInformation(
    FileObject: PFILE_OBJECT,
    FileInformationClass: FILE_INFORMATION_CLASS,
    Length: ULONG,
    FileInformation: PVOID,
    ReturnedLength: *mut ULONG,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let FileInformation = opaque_ptr(FileInformation);
    let ReturnedLength = opaque_ptr(ReturnedLength);
    IopQueryDeviceInformation(
        FileObject,
        FileInformationClass as ULONG,
        Length,
        FileInformation,
        ReturnedLength,
        1u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IoQueryVolumeInformation(
    FileObject: PFILE_OBJECT,
    FsInformationClass: FS_INFORMATION_CLASS,
    Length: ULONG,
    FsInformation: PVOID,
    ReturnedLength: *mut ULONG,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let FsInformation = opaque_ptr(FsInformation);
    let ReturnedLength = opaque_ptr(ReturnedLength);
    IopQueryDeviceInformation(
        FileObject,
        FsInformationClass as ULONG,
        Length,
        FsInformation,
        ReturnedLength,
        0u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IoSetInformation(
    FileObject: PFILE_OBJECT,
    FileInformationClass: FILE_INFORMATION_CLASS,
    Length: ULONG,
    FileInformation: PVOID,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let FileInformation = opaque_ptr(FileInformation);
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Irp: PIRP;
    let DeviceObject: PDEVICE_OBJECT;
    let StackPtr: PIO_STACK_LOCATION;
    let mut LocalEvent: BOOLEAN = 0u8;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    ObReferenceObject(FileObject as PVOID);

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        IopLockFileObject(FileObject, 0i8);
        KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    } else {
        KeInitializeEvent(core::ptr::addr_of_mut!(Event), 1, 0u8);
        LocalEvent = 1u8;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), core::ptr::null_mut());
    }

    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).RequestorMode = 0i8;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).UserIosb = core::ptr::addr_of_mut!(IoStatusBlock);
    (*Irp).UserEvent = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(Event)
    } else {
        core::ptr::null_mut()
    };
    (*Irp).Flags = if LocalEvent != 0 {
        IRP_SYNCHRONOUS_API
    } else {
        0
    };
    (*Irp).Flags |= IRP_BUFFERED_IO;
    (*Irp).AssociatedIrp.SystemBuffer = FileInformation;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_SET_INFORMATION as u8;
    (*StackPtr).FileObject = FileObject;
    (*StackPtr).Parameters.SetFile.FileInformationClass = FileInformationClass as u32;
    (*StackPtr).Parameters.SetFile.Length = Length;

    IopQueueIrpToThread(Irp);
    Status = IoCallDriver(DeviceObject, Irp);

    if LocalEvent == 0 {
        if Status == STATUS_PENDING {
            Status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as *mut core::ffi::c_void,
                0,
                0i8,
                ((*FileObject).Flags & FO_ALERTABLE_IO != 0) as u8,
                core::ptr::null_mut(),
            );
            if Status == STATUS_ALERTED {
                IopAbortInterruptedIrp(core::ptr::addr_of_mut!((*FileObject).Event), Irp);
            }
            Status = (*FileObject).FinalStatus;
        }
        IopUnlockFileObject(FileObject);
    } else if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            0,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtDeviceIoControlFile(
    DeviceHandle: HANDLE,
    Event: HANDLE,
    UserApcRoutine: PIO_APC_ROUTINE,
    UserApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    IoControlCode: ULONG,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
) -> NTSTATUS {
    let DeviceHandle = opaque_ptr(DeviceHandle);
    let Event = opaque_ptr(Event);
    let UserApcContext = opaque_ptr(UserApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let InputBuffer = opaque_ptr(InputBuffer);
    let OutputBuffer = opaque_ptr(OutputBuffer);
    IopDeviceFsIoControl(
        DeviceHandle,
        Event,
        UserApcRoutine,
        UserApcContext,
        IoStatusBlock,
        IoControlCode,
        InputBuffer,
        InputBufferLength,
        OutputBuffer,
        OutputBufferLength,
        1u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtFsControlFile(
    DeviceHandle: HANDLE,
    Event: HANDLE,
    UserApcRoutine: PIO_APC_ROUTINE,
    UserApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    IoControlCode: ULONG,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
) -> NTSTATUS {
    let DeviceHandle = opaque_ptr(DeviceHandle);
    let Event = opaque_ptr(Event);
    let UserApcContext = opaque_ptr(UserApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let InputBuffer = opaque_ptr(InputBuffer);
    let OutputBuffer = opaque_ptr(OutputBuffer);
    IopDeviceFsIoControl(
        DeviceHandle,
        Event,
        UserApcRoutine,
        UserApcContext,
        IoStatusBlock,
        IoControlCode,
        InputBuffer,
        InputBufferLength,
        OutputBuffer,
        OutputBufferLength,
        0u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtFlushBuffersFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let Irp: PIRP;
    let StackPtr: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let DeviceObject: PDEVICE_OBJECT;
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut LocalEvent: BOOLEAN = 0u8;
    let mut ObjectHandleInfo: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            core::mem::align_of::<_IO_STATUS_BLOCK>(),
            PreviousMode,
        ) { return e; }
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        0,
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::addr_of_mut!(ObjectHandleInfo),
    );
    if Status < 0 {
        return Status;
    }

    if (ObjectHandleInfo.GrantedAccess
        & ((if (*FileObject).Flags & FO_NAMED_PIPE == 0 {
            FILE_APPEND_DATA
        } else {
            0
        }) | FILE_WRITE_DATA))
        == 0
    {
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_ACCESS_DENIED;
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
    } else {
        Event = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<KEVENT>(), TAG_IO) as PKEVENT;
        if Event.is_null() {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        KeInitializeEvent(Event, 1, 0u8);
        LocalEvent = 1u8;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 1u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), Event as PVOID);
    }

    (*Irp).Flags = if LocalEvent != 0 {
        IRP_SYNCHRONOUS_API
    } else {
        0
    };
    (*Irp).UserIosb = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(KernelIosb)
    } else {
        IoStatusBlock
    };
    (*Irp).UserEvent = if LocalEvent != 0 { Event } else { core::ptr::null_mut() };
    (*Irp).RequestorMode = PreviousMode;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_FLUSH_BUFFERS as u8;
    (*StackPtr).FileObject = FileObject;

    Status = IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        0u8,
        PreviousMode,
        (LocalEvent == 0) as u8,
        IopOtherTransfer,
    );

    if LocalEvent != 0 {
        Status = IopFinalizeAsynchronousIo(
            Status,
            Event,
            Irp,
            PreviousMode,
            core::ptr::addr_of_mut!(KernelIosb),
            IoStatusBlock,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtNotifyChangeDirectoryFile(
    FileHandle: HANDLE,
    EventHandle: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    BufferSize: ULONG,
    CompletionFilter: ULONG,
    WatchTree: BOOLEAN,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let EventHandle = opaque_ptr(EventHandle);
    let ApcContext = opaque_ptr(ApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Buffer = opaque_ptr(Buffer);
    let Irp: PIRP;
    let mut Event: PKEVENT = core::ptr::null_mut();
    let DeviceObject: PDEVICE_OBJECT;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut LockedForSync: BOOLEAN = 0u8;

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            core::mem::align_of::<_IO_STATUS_BLOCK>(),
            PreviousMode,
        ) { return e; }
        if BufferSize != 0 {
            if let Err(e) = probe_user_buffer(
                Buffer,
                BufferSize as usize,
                core::mem::size_of::<ULONG>(),
                PreviousMode,
            ) { return e; }
        }
        if CompletionFilter == 0 || (CompletionFilter & !FILE_NOTIFY_VALID_MASK != 0) {
            return STATUS_INVALID_PARAMETER;
        }
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        FILE_LIST_DIRECTORY,
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if !(*FileObject).CompletionContext.is_null() && ApcRoutine.is_some() {
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    if EventHandle != core::ptr::null_mut() {
        Status = ObReferenceObjectByHandle(
            EventHandle,
            EVENT_MODIFY_STATE,
            ExEventObjectType,
            PreviousMode,
            core::ptr::addr_of_mut!(Event) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        KeClearEvent(Event);
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            if !Event.is_null() {
                ObDereferenceObject(Event as PVOID);
            }
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        LockedForSync = 1u8;
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, Event, core::ptr::null_mut());
    }

    (*Irp).RequestorMode = PreviousMode;
    (*Irp).UserIosb = IoStatusBlock;
    (*Irp).UserEvent = Event;
    (*Irp).UserBuffer = Buffer;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = ApcRoutine;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = ApcContext;

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).MajorFunction = IRP_MJ_DIRECTORY_CONTROL as u8;
    (*IoStack).MinorFunction = IRP_MN_NOTIFY_CHANGE_DIRECTORY as u8;
    (*IoStack).FileObject = FileObject;
    (*IoStack).Parameters.NotifyDirectory.CompletionFilter = CompletionFilter;
    (*IoStack).Parameters.NotifyDirectory.Length = BufferSize;
    if WatchTree != 0 {
        (*IoStack).Flags = SL_WATCH_TREE as u8;
    }

    IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        0u8,
        PreviousMode,
        LockedForSync,
        IopOtherTransfer,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtLockFile(
    FileHandle: HANDLE,
    EventHandle: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    ByteOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    Key: ULONG,
    FailImmediately: BOOLEAN,
    ExclusiveLock: BOOLEAN,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let EventHandle = opaque_ptr(EventHandle);
    let ApcContext = opaque_ptr(ApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let ByteOffset = opaque_ptr(ByteOffset);
    let Length = opaque_ptr(Length);
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut LocalLength: PLARGE_INTEGER = core::ptr::null_mut();
    let Irp: PIRP;
    let StackPtr: PIO_STACK_LOCATION;
    let DeviceObject: PDEVICE_OBJECT;
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut LockedForSync: BOOLEAN = 0u8;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut CapturedByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut CapturedLength: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut HandleInformation: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let FastIoDispatch: PFAST_IO_DISPATCH;

    Status = ObReferenceObjectByHandle(
        FileHandle,
        0,
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::addr_of_mut!(HandleInformation),
    );
    if Status < 0 {
        return Status;
    }

    if PreviousMode != 0i8 {
        if !(*FileObject).CompletionContext.is_null() && ApcRoutine.is_some() {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_INVALID_PARAMETER;
        }
        if (HandleInformation.GrantedAccess & (FILE_WRITE_DATA | FILE_READ_DATA)) == 0 {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_ACCESS_DENIED;
        }
        if let Err(e) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            core::mem::align_of::<_IO_STATUS_BLOCK>(),
            PreviousMode,
        ) {
            ObDereferenceObject(FileObject as PVOID);
            return e;
        }
        match probe_read_user::<LARGE_INTEGER>(ByteOffset, PreviousMode) {
            Ok(v) => CapturedByteOffset = v,
            Err(e) => { ObDereferenceObject(FileObject as PVOID); return e; }
        }
        match probe_read_user::<LARGE_INTEGER>(Length, PreviousMode) {
            Ok(v) => CapturedLength = v,
            Err(e) => { ObDereferenceObject(FileObject as PVOID); return e; }
        }
    } else {
        CapturedByteOffset = *ByteOffset;
        CapturedLength = *Length;
    }

    if EventHandle != core::ptr::null_mut() {
        Status = ObReferenceObjectByHandle(
            EventHandle,
            EVENT_MODIFY_STATE,
            ExEventObjectType,
            PreviousMode,
            core::ptr::addr_of_mut!(Event) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status != STATUS_SUCCESS {
            return Status;
        }
        KeClearEvent(Event);
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastIoDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoLock.is_some() {
        let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        if ((*FastIoDispatch).FastIoLock.unwrap())(
            FileObject,
            core::ptr::addr_of_mut!(CapturedByteOffset),
            core::ptr::addr_of_mut!(CapturedLength),
            PsGetCurrentProcess() as PEPROCESS,
            Key,
            FailImmediately,
            ExclusiveLock,
            core::ptr::addr_of_mut!(KernelIosb),
            DeviceObject,
        ) != 0
        {
            write_user_nofail(IoStatusBlock, KernelIosb, PreviousMode);
            if EventHandle != core::ptr::null_mut() {
                KeSetEvent(Event, 0, 0u8);
                ObDereferenceObject(Event as PVOID);
            }
            if !(*FileObject).CompletionContext.is_null() && !ApcContext.is_null() {
                if IoSetIoCompletion(
                    (*(*FileObject).CompletionContext).Port,
                    (*(*FileObject).CompletionContext).Key,
                    ApcContext,
                    KernelIosb.__bindgen_anon_1.Status,
                    KernelIosb.Information,
                    1u8,
                ) < 0
                {
                    KernelIosb.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
                }
            }
            (*FileObject).LockOperation = 1u8;
            ObDereferenceObject(FileObject as PVOID);
            return KernelIosb.__bindgen_anon_1.Status;
        }
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            if !Event.is_null() {
                ObDereferenceObject(Event as PVOID);
            }
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        LockedForSync = 1u8;
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    (*FileObject).LockOperation = 1u8;
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, Event, core::ptr::null_mut());
    }

    (*Irp).RequestorMode = PreviousMode;
    (*Irp).UserIosb = IoStatusBlock;
    (*Irp).UserEvent = Event;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = ApcRoutine;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = ApcContext;

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_LOCK_CONTROL as u8;
    (*StackPtr).MinorFunction = IRP_MN_LOCK as u8;
    (*StackPtr).FileObject = FileObject;

    LocalLength =
        ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<LARGE_INTEGER>(), TAG_LOCK)
            as PLARGE_INTEGER;
    if LocalLength.is_null() {
        IopCleanupAfterException(FileObject, Irp, Event, core::ptr::null_mut());
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *LocalLength = CapturedLength;
    (*Irp).Tail.Overlay.AuxiliaryBuffer = LocalLength as *mut i8;
    (*StackPtr).Parameters.LockControl.Length = LocalLength;
    (*StackPtr).Parameters.LockControl.ByteOffset = CapturedByteOffset;
    (*StackPtr).Parameters.LockControl.Key = Key;

    if FailImmediately != 0 {
        (*StackPtr).Flags = SL_FAIL_IMMEDIATELY as u8;
    }
    if ExclusiveLock != 0 {
        (*StackPtr).Flags |= SL_EXCLUSIVE_LOCK as u8;
    }

    IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        0u8,
        PreviousMode,
        LockedForSync,
        IopOtherTransfer,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryDirectoryFile(
    FileHandle: HANDLE,
    EventHandle: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FileInformation: PVOID,
    Length: ULONG,
    FileInformationClass: FILE_INFORMATION_CLASS,
    ReturnSingleEntry: BOOLEAN,
    FileName: PUNICODE_STRING,
    RestartScan: BOOLEAN,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let EventHandle = opaque_ptr(EventHandle);
    let ApcContext = opaque_ptr(ApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let FileInformation = opaque_ptr(FileInformation);
    let FileName = opaque_ptr(FileName);
    let Irp: PIRP;
    let DeviceObject: PDEVICE_OBJECT;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let StackPtr: PIO_STACK_LOCATION;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut LockedForSynch: BOOLEAN = 0u8;
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut AuxBuffer: PVOID = core::ptr::null_mut();
    let Mdl: PMDL;
    let mut CapturedFileName: _UNICODE_STRING = core::mem::zeroed();
    let SearchPattern: PUNICODE_STRING;

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            core::mem::align_of::<_IO_STATUS_BLOCK>(),
            PreviousMode,
        ) {
            if !AuxBuffer.is_null() { ExFreePoolWithTag(AuxBuffer, TAG_IOBUF); }
            return e;
        }
        if let Err(e) = probe_user_buffer(
            FileInformation,
            Length as usize,
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) {
            if !AuxBuffer.is_null() { ExFreePoolWithTag(AuxBuffer, TAG_IOBUF); }
            return e;
        }
        if !FileName.is_null() {
            match probe_capture_unicode_string_header(FileName as *const ntkrnl_sys::UNICODE_STRING, PreviousMode) {
                Ok(v) => CapturedFileName = core::mem::transmute(v),
                Err(e) => {
                    if !AuxBuffer.is_null() { ExFreePoolWithTag(AuxBuffer, TAG_IOBUF); }
                    return e;
                }
            }
            if CapturedFileName.Length != 0 {
                if let Err(e) = probe_user_buffer(
                    CapturedFileName.Buffer as *const core::ffi::c_void,
                    CapturedFileName.Length as usize,
                    1,
                    PreviousMode,
                ) {
                    if !AuxBuffer.is_null() { ExFreePoolWithTag(AuxBuffer, TAG_IOBUF); }
                    return e;
                }
            }
            AuxBuffer = ExAllocatePoolWithTag(
                NonPagedPool,
                CapturedFileName.Length as usize + core::mem::size_of::<_UNICODE_STRING>(),
                TAG_IOBUF,
            );
            if AuxBuffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            // C: the capture copy sits in the same _SEH2_TRY as the probes; its
            // handler frees AuxBuffer and returns the code.
            if let Err(e) = safe_copy_raw(
                (AuxBuffer as usize + core::mem::size_of::<_UNICODE_STRING>()) as PVOID,
                CapturedFileName.Buffer as *const core::ffi::c_void,
                CapturedFileName.Length as usize,
            ) {
                ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
                return e;
            }
            SearchPattern = AuxBuffer as PUNICODE_STRING;
            (*SearchPattern).Buffer =
                (AuxBuffer as usize + core::mem::size_of::<_UNICODE_STRING>()) as *mut u16;
            (*SearchPattern).Length = CapturedFileName.Length;
            (*SearchPattern).MaximumLength = CapturedFileName.Length;
        }
    }

    match FileInformationClass {
        FileDirectoryInformation => {
            if Length < core::mem::size_of::<FILE_DIRECTORY_INFORMATION>() as ULONG {
                if !AuxBuffer.is_null() {
                    ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
                }
                return STATUS_INFO_LENGTH_MISMATCH;
            }
        }
        FileFullDirectoryInformation => {
            if Length < core::mem::size_of::<FILE_FULL_DIR_INFORMATION>() as ULONG {
                if !AuxBuffer.is_null() {
                    ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
                }
                return STATUS_INFO_LENGTH_MISMATCH;
            }
        }
        FileIdFullDirectoryInformation => {
            if Length < core::mem::size_of::<FILE_ID_FULL_DIR_INFORMATION>() as ULONG {
                if !AuxBuffer.is_null() {
                    ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
                }
                return STATUS_INFO_LENGTH_MISMATCH;
            }
        }
        FileNamesInformation => {
            if Length < core::mem::size_of::<FILE_NAMES_INFORMATION>() as ULONG {
                if !AuxBuffer.is_null() {
                    ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
                }
                return STATUS_INFO_LENGTH_MISMATCH;
            }
        }
        FileBothDirectoryInformation => {
            if Length < core::mem::size_of::<FILE_BOTH_DIR_INFORMATION>() as ULONG {
                if !AuxBuffer.is_null() {
                    ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
                }
                return STATUS_INFO_LENGTH_MISMATCH;
            }
        }
        FileIdBothDirectoryInformation => {
            if Length < core::mem::size_of::<FILE_ID_BOTH_DIR_INFORMATION>() as ULONG {
                if !AuxBuffer.is_null() {
                    ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
                }
                return STATUS_INFO_LENGTH_MISMATCH;
            }
        }
        _ => {}
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        FILE_LIST_DIRECTORY,
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        if !AuxBuffer.is_null() {
            ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
        }
        return Status;
    }

    if !(*FileObject).CompletionContext.is_null() && ApcRoutine.is_some() {
        ObDereferenceObject(FileObject as PVOID);
        if !AuxBuffer.is_null() {
            ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
        }
        return STATUS_INVALID_PARAMETER;
    }

    if EventHandle != core::ptr::null_mut() {
        Status = ObReferenceObjectByHandle(
            EventHandle,
            EVENT_MODIFY_STATE,
            ExEventObjectType,
            PreviousMode,
            core::ptr::addr_of_mut!(Event) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            if !AuxBuffer.is_null() {
                ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
            }
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        KeClearEvent(Event);
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            if !Event.is_null() {
                ObDereferenceObject(Event as PVOID);
            }
            ObDereferenceObject(FileObject as PVOID);
            if !AuxBuffer.is_null() {
                ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
            }
            return Status;
        }
        LockedForSynch = 1u8;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 1u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, Event, AuxBuffer);
    }

    (*Irp).RequestorMode = PreviousMode;
    (*Irp).UserIosb = IoStatusBlock;
    (*Irp).UserEvent = Event;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = ApcRoutine;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = ApcContext;
    (*Irp).MdlAddress = core::ptr::null_mut();
    (*Irp).Tail.Overlay.AuxiliaryBuffer = AuxBuffer as *mut i8;
    (*Irp).AssociatedIrp.SystemBuffer = core::ptr::null_mut();

    if (*DeviceObject).Flags & DO_BUFFERED_IO != 0 {
        let alloc_result: Result<_, NTSTATUS> = (|| {
            let buffer = ExAllocatePoolWithQuotaTag(NonPagedPool, Length as SIZE_T, TAG_IOBUF);
            if buffer.is_null() {
                return Err(STATUS_INSUFFICIENT_RESOURCES);
            }
            (*Irp).AssociatedIrp.SystemBuffer = buffer;
            (*Irp).UserBuffer = FileInformation;
            (*Irp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_INPUT_OPERATION;
            Ok(())
        })();
        if let Err(e) = alloc_result {
            IopCleanupAfterException(FileObject, Irp, Event, core::ptr::null_mut());
            if !AuxBuffer.is_null() {
                ExFreePoolWithTag(AuxBuffer, TAG_IOBUF);
            }
            return e;
        }
    } else if (*DeviceObject).Flags & DO_DIRECT_IO != 0 {
        let alloc_result: Result<_, NTSTATUS> = (|| {
            let mdl = IoAllocateMdl(FileInformation, Length, 0u8, 1u8, Irp);
            if mdl.is_null() {
                return Err(STATUS_INSUFFICIENT_RESOURCES);
            }
            let st = ntkrnl_sys::probe::MmTryProbeAndLockPages(mdl, PreviousMode, IoWriteAccess);
            if st < 0 {
                return Err(st);
            }
            Ok(())
        })();
        if let Err(e) = alloc_result {
            IopCleanupAfterException(FileObject, Irp, Event, core::ptr::null_mut());
            return e;
        }
    } else {
        (*Irp).UserBuffer = FileInformation;
    }

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).FileObject = FileObject;
    (*StackPtr).MajorFunction = IRP_MJ_DIRECTORY_CONTROL as u8;
    (*StackPtr).MinorFunction = IRP_MN_QUERY_DIRECTORY as u8;
    (*StackPtr).Parameters.QueryDirectory.FileInformationClass = FileInformationClass as u32;
    (*StackPtr).Parameters.QueryDirectory.FileName = AuxBuffer as PUNICODE_STRING;
    (*StackPtr).Parameters.QueryDirectory.FileIndex = 0;
    (*StackPtr).Parameters.QueryDirectory.Length = Length;
    (*StackPtr).Flags = 0;
    if RestartScan != 0 {
        (*StackPtr).Flags = SL_RESTART_SCAN as u8;
    }
    if ReturnSingleEntry != 0 {
        (*StackPtr).Flags |= SL_RETURN_SINGLE_ENTRY as u8;
    }

    (*Irp).Flags |= IRP_DEFER_IO_COMPLETION;

    IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        1u8,
        PreviousMode,
        LockedForSynch,
        IopOtherTransfer,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryEaFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    Length: ULONG,
    ReturnSingleEntry: BOOLEAN,
    EaList: PVOID,
    EaListLength: ULONG,
    EaIndex: *mut ULONG,
    RestartScan: BOOLEAN,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Buffer = opaque_ptr(Buffer);
    let EaList = opaque_ptr(EaList);
    let EaIndex = opaque_ptr(EaIndex);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FileInformation: PVOID,
    Length: ULONG,
    FileInformationClass: FILE_INFORMATION_CLASS,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let FileInformation = opaque_ptr(FileInformation);
    let mut HandleInformation: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let Irp: PIRP;
    let DeviceObject: PDEVICE_OBJECT;
    let StackPtr: PIO_STACK_LOCATION;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut LocalEvent: BOOLEAN = 0u8;
    let mut NormalRoutine: PKNORMAL_ROUTINE = None;
    let mut NormalContext: PVOID = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut CallDriver: BOOLEAN = 1u8;
    let AccessBuffer: PFILE_ACCESS_INFORMATION;
    let ModeBuffer: PFILE_MODE_INFORMATION;
    let AlignmentBuffer: PFILE_ALIGNMENT_INFORMATION;
    let AllBuffer: PFILE_ALL_INFORMATION;
    let FastIoDispatch: PFAST_IO_DISPATCH;

    // Class + length validation runs for EVERY caller, not just user mode.
    //
    // The C guarded the kernel-mode copy of these two checks behind `#if DBG`, and
    // the translation dropped that arm — leaving kernel callers unvalidated. That is
    // not a cosmetic gap: `Length` sizes the TAG_IOBUF system buffer below, so a
    // kernel caller asking for FileAllInformation with Length=0 got a 0-byte pool
    // block, and then the I/O manager wrote AccessInformation/ModeInformation/
    // AlignmentInformation into it and the FSD wrote a FILE_BASIC_INFORMATION on top
    // — smashing the next pool header with a FILETIME (BUGCHECK 0x19 subcode 5).
    // Our own NtSetInformationFile already validates unconditionally; the two
    // siblings disagreeing is what gave this away.
    //
    // Unconditional is also the honest reading of the C for this kernel: we build
    // checked (HalpBuildType = PRCB_BUILD_DEBUG), so `#if DBG` is live here. We keep
    // it unconditional rather than re-deriving DBG, because the C guards it for SPEED,
    // not correctness — and the unguarded path corrupts pool. Its sibling
    // NtSetInformationFile validates in every build for the same reason.
    //
    // KNOWN CONSEQUENCE, deliberately accepted: NpfsFileInfo.c:95 queries
    // FileAllInformation with sizeof(FILE_ALL_INFORMATION)-4 and expects
    // STATUS_BUFFER_OVERFLOW. On a checked build that is INFO_LENGTH_MISMATCH — real
    // Windows and upstream ReactOS behave the same way, because the C table
    // (ntoskrnl/include/internal/io_i.h) also uses sizeof(FILE_ALL_INFORMATION). The
    // test is missing the `if (KmtIsCheckedBuild)` guard its IoFilesystem sibling has.
    // That is a test-side gap; do NOT weaken this check to turn it green.
    if (FileInformationClass as i32) < 0
        || (FileInformationClass as u32) >= FileMaximumInformation
        || c_array_get(&IopQueryOperationLength, FileInformationClass as usize) as ULONG == 0
    {
        return STATUS_INVALID_INFO_CLASS;
    }
    if Length < c_array_get(&IopQueryOperationLength, FileInformationClass as usize) as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            core::mem::align_of::<_IO_STATUS_BLOCK>(),
            PreviousMode,
        ) { return e; }
        if let Err(e) = probe_user_buffer(
            FileInformation,
            Length as usize,
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) { return e; }
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        c_array_get(&IopQueryOperationAccess, FileInformationClass as usize),
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::addr_of_mut!(HandleInformation),
    );
    if Status < 0 {
        return Status;
    }

    if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
        DeviceObject = IoGetAttachedDevice((*FileObject).DeviceObject);
    } else {
        DeviceObject = IoGetRelatedDeviceObject(FileObject);
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        if FileInformationClass == FilePositionInformation {
            let pos_result = write_user(
                core::ptr::addr_of_mut!(
                    (*(FileInformation as PFILE_POSITION_INFORMATION)).CurrentByteOffset
                ),
                (*FileObject).CurrentByteOffset,
                PreviousMode,
            );
            if let Err(e) = pos_result {
                IopUnlockFileObject(FileObject);
                ObDereferenceObject(FileObject as PVOID);
                return e;
            }
            let iosb = _IO_STATUS_BLOCK {
                __bindgen_anon_1: _IO_STATUS_BLOCK__bindgen_ty_1 {
                    Status: STATUS_SUCCESS,
                },
                Information: core::mem::size_of::<FILE_POSITION_INFORMATION>() as _,
            };
            let write_result = write_user(IoStatusBlock, iosb, PreviousMode);
            Status = match write_result {
                Ok(_) => STATUS_SUCCESS,
                Err(e) => e,
            };
            IopUnlockFileObject(FileObject);
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
    } else {
        Event = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<KEVENT>(), TAG_IO) as PKEVENT;
        if Event.is_null() {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        KeInitializeEvent(Event, 1, 0u8);
        LocalEvent = 1u8;
    }

    FastIoDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;
    if !FastIoDispatch.is_null()
        && ((FileInformationClass == FileBasicInformation
            && (*FastIoDispatch).FastIoQueryBasicInfo.is_some())
            || (FileInformationClass == FileStandardInformation
                && (*FastIoDispatch).FastIoQueryStandardInfo.is_some()))
    {
        let mut Success: BOOLEAN = 0u8;
        if FileInformationClass == FileBasicInformation {
            Success = ((*FastIoDispatch).FastIoQueryBasicInfo.unwrap())(
                FileObject,
                1u8,
                FileInformation as PFILE_BASIC_INFORMATION,
                core::ptr::addr_of_mut!(KernelIosb),
                DeviceObject,
            );
        } else {
            Success = ((*FastIoDispatch).FastIoQueryStandardInfo.unwrap())(
                FileObject,
                1u8,
                FileInformation as *mut ntkrnl_sys::ddk::_FILE_STANDARD_INFORMATION,
                core::ptr::addr_of_mut!(KernelIosb),
                DeviceObject,
            );
        }
        if Success != 0 {
            write_user_nofail(IoStatusBlock, KernelIosb, PreviousMode);
            if LocalEvent != 0 {
                ExFreePoolWithTag(Event as PVOID, TAG_IO);
            }
            if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
                IopUnlockFileObject(FileObject);
            }
            ObDereferenceObject(FileObject as PVOID);
            return KernelIosb.__bindgen_anon_1.Status;
        }
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), Event as PVOID);
    }

    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).RequestorMode = PreviousMode;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).Flags = if LocalEvent != 0 {
        IRP_SYNCHRONOUS_API
    } else {
        0
    };
    (*Irp).UserIosb = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(KernelIosb)
    } else {
        IoStatusBlock
    };
    (*Irp).UserEvent = if LocalEvent != 0 { Event } else { core::ptr::null_mut() };
    (*Irp).AssociatedIrp.SystemBuffer = core::ptr::null_mut();
    (*Irp).MdlAddress = core::ptr::null_mut();
    (*Irp).UserBuffer = FileInformation;

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_QUERY_INFORMATION as u8;
    (*StackPtr).FileObject = FileObject;

    let alloc_result: Result<_, NTSTATUS> = (|| {
        let buffer = ExAllocatePoolWithQuotaTag(NonPagedPool, Length as SIZE_T, TAG_IOBUF);
        if buffer.is_null() {
            return Err(STATUS_INSUFFICIENT_RESOURCES);
        }
        (*Irp).AssociatedIrp.SystemBuffer = buffer;
        Ok(())
    })();
    if let Err(e) = alloc_result {
        IopCleanupAfterException(FileObject, Irp, core::ptr::null_mut(), Event);
        return e;
    }

    (*Irp).Flags |= IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_INPUT_OPERATION | IRP_DEFER_IO_COMPLETION;
    (*StackPtr).Parameters.QueryFile.FileInformationClass = FileInformationClass as u32;
    (*StackPtr).Parameters.QueryFile.Length = Length;

    IopQueueIrpToThread(Irp);
    IopUpdateOperationCount(IopOtherTransfer);

    if FileInformationClass == FileAccessInformation {
        AccessBuffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_ACCESS_INFORMATION;
        (*AccessBuffer).AccessFlags = HandleInformation.GrantedAccess;
        (*Irp).IoStatus.Information = core::mem::size_of::<FILE_ACCESS_INFORMATION>() as _;
        CallDriver = 0u8;
    } else if FileInformationClass == FileModeInformation {
        ModeBuffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_MODE_INFORMATION;
        (*ModeBuffer).Mode = IopGetFileMode(FileObject);
        (*Irp).IoStatus.Information = core::mem::size_of::<FILE_MODE_INFORMATION>() as _;
        CallDriver = 0u8;
    } else if FileInformationClass == FileAlignmentInformation {
        AlignmentBuffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_ALIGNMENT_INFORMATION;
        (*AlignmentBuffer).AlignmentRequirement = (*DeviceObject).AlignmentRequirement;
        (*Irp).IoStatus.Information = core::mem::size_of::<FILE_ALIGNMENT_INFORMATION>() as _;
        CallDriver = 0u8;
    } else if FileInformationClass == FileAllInformation {
        AllBuffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_ALL_INFORMATION;
        (*AllBuffer).AccessInformation.AccessFlags = HandleInformation.GrantedAccess;
        (*AllBuffer).ModeInformation.Mode = IopGetFileMode(FileObject);
        (*AllBuffer).AlignmentInformation.AlignmentRequirement = (*DeviceObject).AlignmentRequirement;
        (*Irp).IoStatus.Information = (core::mem::size_of::<FILE_ACCESS_INFORMATION>()
            + core::mem::size_of::<FILE_MODE_INFORMATION>()
            + core::mem::size_of::<FILE_ALIGNMENT_INFORMATION>()) as _;
    }

    if CallDriver != 0 {
        Status = IoCallDriver(DeviceObject, Irp);
    } else {
        Status = STATUS_SUCCESS;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }

    if Status == STATUS_PENDING {
        if LocalEvent != 0 {
            Status = KeWaitForSingleObject(
                Event as *mut core::ffi::c_void,
                0,
                PreviousMode,
                0u8,
                core::ptr::null_mut(),
            );
            if Status == STATUS_USER_APC {
                IopAbortInterruptedIrp(Event, Irp);
            }
            Status = KernelIosb.__bindgen_anon_1.Status;
            let write_result = write_user(IoStatusBlock, KernelIosb, PreviousMode);
            if let Err(e) = write_result {
                Status = e;
            }
            ExFreePoolWithTag(Event as PVOID, TAG_IO);
        } else {
            Status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as *mut core::ffi::c_void,
                0,
                PreviousMode,
                ((*FileObject).Flags & FO_ALERTABLE_IO != 0) as u8,
                core::ptr::null_mut(),
            );
            if Status == STATUS_USER_APC || Status == STATUS_ALERTED {
                IopAbortInterruptedIrp(core::ptr::addr_of_mut!((*FileObject).Event), Irp);
            }
            Status = (*FileObject).FinalStatus;
            IopUnlockFileObject(FileObject);
        }
    } else {
        if LocalEvent != 0 {
            (*Irp).UserEvent = core::ptr::null_mut();
            ExFreePoolWithTag(Event as PVOID, TAG_IO);
        }
        (*Irp).UserIosb = IoStatusBlock;
        NormalRoutine = None;
        NormalContext = core::ptr::null_mut();
        // [guard] KeRaiseIrql(APC_LEVEL, core::ptr::addr_of_mut!(OldIrql));
        let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL) };
        IopCompleteRequest(
            core::ptr::addr_of_mut!((*Irp).Tail.Apc),
            core::ptr::addr_of_mut!(NormalRoutine),
            core::ptr::addr_of_mut!(NormalContext),
            core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
            core::ptr::addr_of_mut!(NormalContext),
        );
        drop(irql_guard);
        if LocalEvent == 0 {
            IopUnlockFileObject(FileObject);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryQuotaInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    Length: ULONG,
    ReturnSingleEntry: BOOLEAN,
    SidList: PVOID,
    SidListLength: ULONG,
    StartSid: PSID,
    RestartScan: BOOLEAN,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Buffer = opaque_ptr(Buffer);
    let SidList = opaque_ptr(SidList);
    let StartSid = opaque_ptr(StartSid);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtReadFile(
    FileHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    Length: ULONG,
    ByteOffset: PLARGE_INTEGER,
    Key: *mut ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let Event = opaque_ptr(Event);
    let ApcContext = opaque_ptr(ApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Buffer = opaque_ptr(Buffer);
    let ByteOffset = opaque_ptr(ByteOffset);
    let Key = opaque_ptr(Key);
    let mut Status: NTSTATUS;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let Irp: PIRP;
    let DeviceObject: PDEVICE_OBJECT;
    let StackPtr: PIO_STACK_LOCATION;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut EventObject: PKEVENT = core::ptr::null_mut();
    let mut CapturedByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut CapturedKey: ULONG = 0;
    let mut Synchronous: BOOLEAN = 0u8;
    let Mdl: PMDL;
    let FastIoDispatch: PFAST_IO_DISPATCH;
    let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Success: BOOLEAN;

    let Iosb32 = IopCaptureWow64Iosb32(IoStatusBlock, PreviousMode);

    Status = ObReferenceObjectByHandle(
        FileHandle,
        FILE_READ_DATA,
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);

    if PreviousMode != 0i8 {
        let (IosbSize, IosbAlign) = if Iosb32 {
            (
                core::mem::size_of::<IoStatusBlock32>(),
                core::mem::align_of::<IoStatusBlock32>(),
            )
        } else {
            (
                core::mem::size_of::<_IO_STATUS_BLOCK>(),
                core::mem::align_of::<_IO_STATUS_BLOCK>(),
            )
        };
        if let Err(e) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            IosbSize,
            IosbAlign,
            PreviousMode,
        ) {
            ObDereferenceObject(FileObject as PVOID);
            return e;
        }
        if let Err(e) = probe_user_buffer(Buffer, Length as usize, 1, PreviousMode) {
            ObDereferenceObject(FileObject as PVOID);
            return e;
        }
        if !ByteOffset.is_null() {
            match probe_read_user::<LARGE_INTEGER>(ByteOffset, PreviousMode) {
                Ok(v) => CapturedByteOffset = v,
                Err(e) => { ObDereferenceObject(FileObject as PVOID); return e; }
            }
        }
        if !(*FileObject).CompletionContext.is_null() && ApcRoutine.is_some() {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_INVALID_PARAMETER;
        }
        if (*FileObject).Flags & FO_NO_INTERMEDIATE_BUFFERING != 0 {
            if (*DeviceObject).SectorSize != 0
                && ((*DeviceObject).SectorSize as ULONG - 1) & Length != 0
            {
                if Length % (*DeviceObject).SectorSize as ULONG != 0 {
                    ObDereferenceObject(FileObject as PVOID);
                    return STATUS_INVALID_PARAMETER;
                }
            }
            if (Buffer as ULONG_PTR) & (*DeviceObject).AlignmentRequirement as ULONG_PTR != 0 {
                ObDereferenceObject(FileObject as PVOID);
                return STATUS_INVALID_PARAMETER;
            }
            if !ByteOffset.is_null() {
                if (*DeviceObject).SectorSize != 0
                    && CapturedByteOffset.QuadPart % (*DeviceObject).SectorSize as i64 != 0
                {
                    ObDereferenceObject(FileObject as PVOID);
                    return STATUS_INVALID_PARAMETER;
                }
            }
        }
        if !Key.is_null() {
            match probe_read_user::<ULONG>(Key, PreviousMode) {
                Ok(v) => CapturedKey = v,
                Err(e) => { ObDereferenceObject(FileObject as PVOID); return e; }
            }
        }
    } else {
        if !ByteOffset.is_null() {
            CapturedByteOffset = *ByteOffset;
        }
        if !Key.is_null() {
            CapturedKey = *Key;
        }
    }

    if CapturedByteOffset.QuadPart < 0 && CapturedByteOffset.QuadPart != -2 {
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    if Event != core::ptr::null_mut() {
        Status = ObReferenceObjectByHandle(
            Event,
            EVENT_MODIFY_STATE,
            ExEventObjectType,
            PreviousMode,
            core::ptr::addr_of_mut!(EventObject) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        KeClearEvent(EventObject);
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            if !EventObject.is_null() {
                ObDereferenceObject(EventObject as PVOID);
            }
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        if ByteOffset.is_null()
            || (CapturedByteOffset.u.LowPart == FILE_USE_FILE_POINTER_POSITION
                && CapturedByteOffset.u.HighPart == -1i32)
        {
            CapturedByteOffset = (*FileObject).CurrentByteOffset;
        }
        if !(*FileObject).PrivateCacheMap.is_null() {
            FastIoDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;
            Success = ((*FastIoDispatch).FastIoRead.unwrap())(
                FileObject,
                core::ptr::addr_of_mut!(CapturedByteOffset),
                Length,
                1u8,
                CapturedKey,
                Buffer,
                core::ptr::addr_of_mut!(KernelIosb),
                DeviceObject,
            );
            if Success != 0
                && (KernelIosb.__bindgen_anon_1.Status == STATUS_SUCCESS
                    || KernelIosb.__bindgen_anon_1.Status == STATUS_BUFFER_OVERFLOW
                    || KernelIosb.__bindgen_anon_1.Status == STATUS_END_OF_FILE)
            {
                IopUpdateOperationCount(IopReadTransfer);
                IopUpdateTransferCount(IopReadTransfer, KernelIosb.Information as ULONG);
                let write_result = if Iosb32 {
                    write_user(
                        IoStatusBlock as *mut IoStatusBlock32,
                        IoStatusBlock32 {
                            Status: KernelIosb.__bindgen_anon_1.Status,
                            Information: KernelIosb.Information as u32,
                        },
                        PreviousMode,
                    )
                } else {
                    write_user(IoStatusBlock, KernelIosb, PreviousMode)
                };
                if let Err(e) = write_result {
                    if !EventObject.is_null() {
                        ObDereferenceObject(EventObject as PVOID);
                    }
                    IopUnlockFileObject(FileObject);
                    ObDereferenceObject(FileObject as PVOID);
                    return e;
                }
                if !EventObject.is_null() {
                    KeSetEvent(EventObject, 0, 0u8);
                    ObDereferenceObject(EventObject as PVOID);
                }
                IopUnlockFileObject(FileObject);
                ObDereferenceObject(FileObject as PVOID);
                return KernelIosb.__bindgen_anon_1.Status;
            }
        }
        Synchronous = 1u8;
    } else if ByteOffset.is_null()
        && ((*FileObject).Flags & (FO_NAMED_PIPE | FO_MAILSLOT)) == 0
    {
        if !EventObject.is_null() {
            ObDereferenceObject(EventObject as PVOID);
        }
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));
    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, EventObject, core::ptr::null_mut());
    }

    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).RequestorMode = PreviousMode;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = ApcRoutine;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = ApcContext;
    (*Irp).UserIosb = IoStatusBlock;
    (*Irp).UserEvent = EventObject;
    (*Irp).PendingReturned = 0u8;
    (*Irp).Cancel = 0u8;
    (*Irp).CancelRoutine = None;
    (*Irp).AssociatedIrp.SystemBuffer = core::ptr::null_mut();
    (*Irp).MdlAddress = core::ptr::null_mut();

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_READ as u8;
    (*StackPtr).FileObject = FileObject;
    (*StackPtr).Parameters.Read.Key = CapturedKey;
    (*StackPtr).Parameters.Read.Length = Length;
    (*StackPtr).Parameters.Read.ByteOffset = CapturedByteOffset;

    if (*DeviceObject).Flags & DO_BUFFERED_IO != 0 {
        if Length != 0 {
            let alloc_result: Result<_, NTSTATUS> = (|| {
                let buffer = ExAllocatePoolWithQuotaTag(NonPagedPool, Length as SIZE_T, TAG_IOBUF);
                if buffer.is_null() {
                    return Err(STATUS_INSUFFICIENT_RESOURCES);
                }
                (*Irp).AssociatedIrp.SystemBuffer = buffer;
                (*Irp).UserBuffer = Buffer;
                (*Irp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_INPUT_OPERATION;
                Ok(())
            })();
            if let Err(e) = alloc_result {
                IopCleanupAfterException(FileObject, Irp, EventObject, core::ptr::null_mut());
                return e;
            }
        } else {
            (*Irp).Flags = IRP_BUFFERED_IO | IRP_INPUT_OPERATION;
        }
    } else if (*DeviceObject).Flags & DO_DIRECT_IO != 0 {
        if Length != 0 {
            let alloc_result: Result<_, NTSTATUS> = (|| {
                let mdl = IoAllocateMdl(Buffer, Length, 0u8, 1u8, Irp);
                if mdl.is_null() {
                    return Err(STATUS_INSUFFICIENT_RESOURCES);
                }
                let st = ntkrnl_sys::probe::MmTryProbeAndLockPages(mdl, PreviousMode, IoWriteAccess);
                if st < 0 {
                    return Err(st);
                }
                Ok(())
            })();
            if let Err(e) = alloc_result {
                IopCleanupAfterException(FileObject, Irp, EventObject, core::ptr::null_mut());
                return e;
            }
        }
        (*Irp).Flags = 0;
    } else {
        (*Irp).Flags = 0;
        (*Irp).UserBuffer = Buffer;
    }

    (*Irp).Flags |= IRP_READ_OPERATION | IRP_DEFER_IO_COMPLETION;
    if Iosb32 {
        (*Irp).Flags |= IRP_WOW64_IOSB32;
    }
    if (*FileObject).Flags & FO_NO_INTERMEDIATE_BUFFERING != 0 {
        (*Irp).Flags |= IRP_NOCACHE;
    }

    IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        1u8,
        PreviousMode,
        Synchronous,
        IopReadTransfer,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtReadFileScatter(
    FileHandle: HANDLE,
    Event: HANDLE,
    UserApcRoutine: PIO_APC_ROUTINE,
    UserApcContext: PVOID,
    UserIoStatusBlock: PIO_STATUS_BLOCK,
    BufferDescription: *const FILE_SEGMENT_ELEMENT,
    BufferLength: ULONG,
    ByteOffset: PLARGE_INTEGER,
    Key: *mut ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let Event = opaque_ptr(Event);
    let UserApcContext = opaque_ptr(UserApcContext);
    let UserIoStatusBlock = opaque_ptr(UserIoStatusBlock);
    let ByteOffset = opaque_ptr(ByteOffset);
    let Key = opaque_ptr(Key);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtSetEaFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    EaBuffer: PVOID,
    EaBufferSize: ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let EaBuffer = opaque_ptr(EaBuffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtSetInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FileInformation: PVOID,
    Length: ULONG,
    FileInformationClass: FILE_INFORMATION_CLASS,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let FileInformation = opaque_ptr(FileInformation);
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut StackPtr: PIO_STACK_LOCATION;
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut LocalEvent: BOOLEAN = 0;
    let mut NormalRoutine: PKNORMAL_ROUTINE = None;
    let mut NormalContext: PVOID = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Queue: PVOID = core::ptr::null_mut();
    let mut CompletionInfo: PFILE_COMPLETION_INFORMATION;
    let mut Context: PIO_COMPLETION_CONTEXT;
    let mut RenameInfo: PFILE_RENAME_INFORMATION;
    let mut TargetHandle: HANDLE = core::ptr::null_mut();

    if FileInformationClass >= FileMaximumInformation
        || c_array_get(&IopSetOperationLength, FileInformationClass as usize) as ULONG == 0
    {
        return STATUS_INVALID_INFO_CLASS;
    }

    if Length < c_array_get(&IopSetOperationLength, FileInformationClass as usize) as ULONG as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    if PreviousMode != 0i8 {
        // C: under _SEH2_TRY, whose handler returns the exception code.
        if let Err(status) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            1,
            PreviousMode,
        ) {
            return status;
        }
        let align = if Length == core::mem::size_of::<BOOLEAN>() as ULONG { 1 } else { 4 };
        if let Err(status) = probe_user_buffer(FileInformation as *const core::ffi::c_void, Length as usize, align, PreviousMode) {
            return status;
        }
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        c_array_get(&IopSetOperationAccess, FileInformationClass as usize),
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
        DeviceObject = IoGetAttachedDevice((*FileObject).DeviceObject);
    } else {
        DeviceObject = IoGetRelatedDeviceObject(FileObject);
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }

        if FileInformationClass == FilePositionInformation {
            // C: the read of FileInformation and the IoStatusBlock stores are one
            // _SEH2_TRY whose handler sets Status to the code.
            Status = match probe_read_generic(FileInformation as *const FILE_POSITION_INFORMATION, 1, 0) {
                Ok(position) => {
                    (*FileObject).CurrentByteOffset = position.CurrentByteOffset;
                    match write_back(IoStatusBlock, _IO_STATUS_BLOCK {
                        __bindgen_anon_1: _IO_STATUS_BLOCK__bindgen_ty_1 { Status: STATUS_SUCCESS },
                        Information: 0,
                    }) {
                        Ok(()) => STATUS_SUCCESS,
                        Err(e) => e,
                    }
                }
                Err(e) => e,
            };
            IopUpdateTransferCount(IopOtherTransfer, Length);
            IopUnlockFileObject(FileObject);
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
    } else {
        Event = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<KEVENT>(),
            TAG_IO,
        ) as PKEVENT;
        if Event.is_null() {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        KeInitializeEvent(Event, 1, 0u8);
        LocalEvent = 1u8;
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));

    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 1u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), Event as PVOID);
    }

    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).RequestorMode = PreviousMode;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).Flags = if LocalEvent != 0 { IRP_SYNCHRONOUS_API } else { 0 };
    (*Irp).UserIosb = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(KernelIosb)
    } else {
        IoStatusBlock
    };
    (*Irp).UserEvent = if LocalEvent != 0 { Event } else { core::ptr::null_mut() };
    (*Irp).AssociatedIrp.SystemBuffer = core::ptr::null_mut();
    (*Irp).MdlAddress = core::ptr::null_mut();
    (*Irp).UserBuffer = FileInformation;

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_SET_INFORMATION as u8;
    (*StackPtr).FileObject = FileObject;

    (*Irp).AssociatedIrp.SystemBuffer =
        ExAllocatePoolWithQuotaTag(NonPagedPool, Length as _, TAG_IOBUF);
    core::ptr::copy_nonoverlapping(
        FileInformation as *const u8,
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        Length as usize,
    );

    (*Irp).Flags |= IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_DEFER_IO_COMPLETION;
    (*StackPtr).Parameters.SetFile.FileInformationClass = FileInformationClass as u32;
    (*StackPtr).Parameters.SetFile.Length = Length;

    IopQueueIrpToThread(Irp);
    IopUpdateOperationCount(IopOtherTransfer);

    if FileInformationClass == FileCompletionInformation {
        if ((*FileObject).Flags & FO_SYNCHRONOUS_IO != 0)
            || !(*FileObject).CompletionContext.is_null()
        {
            Status = STATUS_INVALID_PARAMETER;
        } else {
            CompletionInfo = (*Irp).AssociatedIrp.SystemBuffer as PFILE_COMPLETION_INFORMATION;
            Status = ObReferenceObjectByHandle(
                (*CompletionInfo).Port,
                IO_COMPLETION_MODIFY_STATE,
                IoCompletionType,
                PreviousMode,
                core::ptr::addr_of_mut!(Queue),
                core::ptr::null_mut(),
            );
            if Status >= 0 {
                Context = ExAllocatePoolWithTag(
                    PagedPool,
                    core::mem::size_of::<IO_COMPLETION_CONTEXT>(),
                    IOC_TAG,
                ) as PIO_COMPLETION_CONTEXT;
                if !Context.is_null() {
                    (*Context).Key = (*CompletionInfo).Key;
                    (*Context).Port = Queue;
                    if !InterlockedCompareExchangePointer(
                        core::ptr::addr_of_mut!((*FileObject).CompletionContext) as *mut PVOID,
                        Context as PVOID,
                        core::ptr::null_mut(),
                    )
                    .is_null()
                    {
                        ExFreePoolWithTag(Context as PVOID, IOC_TAG);
                        ObDereferenceObject(Queue);
                        Status = STATUS_INVALID_PARAMETER;
                    }
                } else {
                    ObDereferenceObject(Queue);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                }
            }
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
    } else if FileInformationClass == FileRenameInformation
        || FileInformationClass == FileLinkInformation
        || FileInformationClass == FileMoveClusterInformation
    {
        RenameInfo = (*Irp).AssociatedIrp.SystemBuffer as PFILE_RENAME_INFORMATION;

        if (*RenameInfo).FileNameLength != 0
            && ((*RenameInfo).FileNameLength & 1) == 0
            && (Length - 12 >= (*RenameInfo).FileNameLength)
        {
            if FileInformationClass == FileMoveClusterInformation {
                (*StackPtr).Parameters.SetFile.__bindgen_anon_1.ClusterCount =
                    (*((*Irp).AssociatedIrp.SystemBuffer
                        as *const ntkrnl_sys::ddk::_FILE_MOVE_CLUSTER_INFORMATION))
                    .ClusterCount;
            } else {
                (*StackPtr).Parameters.SetFile.__bindgen_anon_1.__bindgen_anon_1.ReplaceIfExists =
                    (*RenameInfo).ReplaceIfExists;
            }

            if *(*RenameInfo).FileName.as_ptr() == OBJ_NAME_PATH_SEPARATOR
                || !(*RenameInfo).RootDirectory.is_null()
            {
                Status = IopOpenLinkOrRenameTarget(
                    core::ptr::addr_of_mut!(TargetHandle),
                    Irp,
                    RenameInfo,
                    FileObject,
                );
                if Status < 0 {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                } else {
                    Status = IoCallDriver(DeviceObject, Irp);
                }
            } else {
                Status = IoCallDriver(DeviceObject, Irp);
            }
        } else {
            Status = STATUS_INVALID_PARAMETER;
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        }
    } else {
        Status = IoCallDriver(DeviceObject, Irp);
    }

    if Status == STATUS_PENDING {
        if LocalEvent != 0 {
            Status = KeWaitForSingleObject(
                Event as *mut core::ffi::c_void,
                0,
                PreviousMode,
                0u8,
                core::ptr::null_mut(),
            );
            if Status == STATUS_USER_APC {
                IopAbortInterruptedIrp(Event, Irp);
            }

            Status = KernelIosb.__bindgen_anon_1.Status;
            // C: _SEH2_TRY { *IoStatusBlock = KernelIosb; } _SEH2_EXCEPT { Status = code; }
            if let Err(e) = write_back(IoStatusBlock, KernelIosb) {
                Status = e;
            }
            ExFreePoolWithTag(Event as PVOID, TAG_IO);
        } else {
            Status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*FileObject).Event) as *mut core::ffi::c_void,
                0,
                PreviousMode,
                ((*FileObject).Flags & FO_ALERTABLE_IO != 0) as u8,
                core::ptr::null_mut(),
            );
            if Status == STATUS_USER_APC || Status == STATUS_ALERTED {
                IopAbortInterruptedIrp(core::ptr::addr_of_mut!((*FileObject).Event), Irp);
            }
            Status = (*FileObject).FinalStatus;
            IopUnlockFileObject(FileObject);
        }
    } else {
        if LocalEvent != 0 {
            (*Irp).UserEvent = core::ptr::null_mut();
            ExFreePoolWithTag(Event as PVOID, TAG_IO);
        }

        (*Irp).UserIosb = IoStatusBlock;
        NormalRoutine = None;
        NormalContext = core::ptr::null_mut();
        // [guard] KeRaiseIrql(APC_LEVEL, core::ptr::addr_of_mut!(OldIrql));
        let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL) };
        IopCompleteRequest(
            core::ptr::addr_of_mut!((*Irp).Tail.Apc),
            core::ptr::addr_of_mut!(NormalRoutine),
            core::ptr::addr_of_mut!(NormalContext),
            core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
            core::ptr::addr_of_mut!(NormalContext),
        );
        drop(irql_guard);

        if LocalEvent == 0 {
            IopUnlockFileObject(FileObject);
        }
    }

    if !TargetHandle.is_null() {
        ObCloseHandle(TargetHandle, 0i8);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetQuotaInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    BufferLength: ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Buffer = opaque_ptr(Buffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtUnlockFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    ByteOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    Key: ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let ByteOffset = opaque_ptr(ByteOffset);
    let Length = opaque_ptr(Length);
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut LocalLength: PLARGE_INTEGER = core::ptr::null_mut();
    let mut Irp: PIRP;
    let mut StackPtr: PIO_STACK_LOCATION;
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut LocalEvent: BOOLEAN = 0;
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut CapturedByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut CapturedLength: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut HandleInformation: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let FastIoDispatch: PFAST_IO_DISPATCH;

    CapturedByteOffset.QuadPart = 0;
    CapturedLength.QuadPart = 0;

    Status = ObReferenceObjectByHandle(
        FileHandle,
        0,
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::addr_of_mut!(HandleInformation),
    );
    if Status < 0 {
        return Status;
    }

    if PreviousMode != 0i8 {
        if (HandleInformation.GrantedAccess & (FILE_WRITE_DATA | FILE_READ_DATA)) == 0 {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_ACCESS_DENIED;
        }

        // C: _SEH2_TRY around these, except handler dereferences FileObject and
        // returns the exception code. No SEH here, so Result probes instead.
        let probed = (|| -> Result<(LARGE_INTEGER, LARGE_INTEGER), NTSTATUS> {
            ntkrnl_sys::probe::probe_write_generic(IoStatusBlock, 1, PreviousMode)?;
            Ok((
                ntkrnl_sys::probe::probe_read_generic(ByteOffset as *const LARGE_INTEGER, 1, PreviousMode)?,
                ntkrnl_sys::probe::probe_read_generic(Length as *const LARGE_INTEGER, 1, PreviousMode)?,
            ))
        })();
        match probed {
            Ok((offset, length)) => {
                CapturedByteOffset = offset;
                CapturedLength = length;
            }
            Err(status) => {
                ObDereferenceObject(FileObject as PVOID);
                return status;
            }
        }
    } else {
        CapturedByteOffset = *ByteOffset;
        CapturedLength = *Length;
    }

    if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0 {
        DeviceObject = IoGetAttachedDevice((*FileObject).DeviceObject);
    } else {
        DeviceObject = IoGetRelatedDeviceObject(FileObject);
    }

    FastIoDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoUnlockSingle.is_some() {
        if ((*FastIoDispatch).FastIoUnlockSingle.unwrap())(
            FileObject,
            core::ptr::addr_of_mut!(CapturedByteOffset),
            core::ptr::addr_of_mut!(CapturedLength),
            PsGetCurrentProcess() as _,
            Key,
            core::ptr::addr_of_mut!(KernelIosb),
            DeviceObject,
        ) != 0
        {
            // C: _SEH2_TRY { *IoStatusBlock = KernelIosb; } _SEH2_EXCEPT { Status = code; }
            let Status = match write_back(IoStatusBlock, KernelIosb) {
                Ok(()) => KernelIosb.__bindgen_anon_1.Status,
                Err(e) => e,
            };
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
    } else {
        Event = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<KEVENT>(),
            TAG_IO,
        ) as PKEVENT;
        if Event.is_null() {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        KeInitializeEvent(Event, 1, 0u8);
        LocalEvent = 1u8;
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));

    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), Event as PVOID);
    }

    (*Irp).RequestorMode = PreviousMode;
    (*Irp).Flags = if LocalEvent != 0 { IRP_SYNCHRONOUS_API } else { 0 };
    (*Irp).UserIosb = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(KernelIosb)
    } else {
        IoStatusBlock
    };
    (*Irp).UserEvent = if LocalEvent != 0 { Event } else { core::ptr::null_mut() };
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_LOCK_CONTROL as u8;
    (*StackPtr).MinorFunction = IRP_MN_UNLOCK_SINGLE as u8;
    (*StackPtr).FileObject = FileObject;

    LocalLength = ExAllocatePoolWithQuotaTag(
        NonPagedPool,
        core::mem::size_of::<LARGE_INTEGER>(),
        TAG_LOCK,
    ) as PLARGE_INTEGER;
    *LocalLength = CapturedLength;
    (*Irp).Tail.Overlay.AuxiliaryBuffer = LocalLength as *mut i8;
    (*StackPtr).Parameters.LockControl.Length = LocalLength;
    (*StackPtr).Parameters.LockControl.ByteOffset = CapturedByteOffset;
    (*StackPtr).Parameters.LockControl.Key = Key;

    Status = IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        0u8,
        PreviousMode,
        (LocalEvent == 0) as u8,
        IopOtherTransfer,
    );

    if LocalEvent != 0 {
        Status = IopFinalizeAsynchronousIo(
            Status,
            Event,
            Irp,
            PreviousMode,
            core::ptr::addr_of_mut!(KernelIosb),
            IoStatusBlock,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtWriteFile(
    FileHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    Length: ULONG,
    ByteOffset: PLARGE_INTEGER,
    Key: *mut ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let Event = opaque_ptr(Event);
    let ApcContext = opaque_ptr(ApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Buffer = opaque_ptr(Buffer);
    let ByteOffset = opaque_ptr(ByteOffset);
    let Key = opaque_ptr(Key);
    let mut Status: NTSTATUS;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Irp: PIRP;
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut StackPtr: PIO_STACK_LOCATION;
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut EventObject: PKEVENT = core::ptr::null_mut();
    let mut CapturedByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut CapturedKey: ULONG = 0;
    let mut Synchronous: BOOLEAN = 0;
    let mut Mdl: PMDL;
    let mut ObjectHandleInfo: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Success: BOOLEAN;
    let FastIoDispatch: PFAST_IO_DISPATCH;

    CapturedByteOffset.QuadPart = 0;

    let Iosb32 = IopCaptureWow64Iosb32(IoStatusBlock, PreviousMode);

    Status = ObReferenceFileObjectForWrite(
        FileHandle,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject),
        core::ptr::addr_of_mut!(ObjectHandleInfo),
    );
    if Status < 0 {
        return Status;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);

    if PreviousMode != 0i8 {
        if Iosb32 {
            if let Err(e) = probe_user_buffer(
                IoStatusBlock as *const core::ffi::c_void,
                core::mem::size_of::<IoStatusBlock32>(),
                core::mem::align_of::<IoStatusBlock32>(),
                PreviousMode,
            ) {
                ObDereferenceObject(FileObject as PVOID);
                return e;
            }
        } else if let Err(status) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            1,
            PreviousMode,
        ) {
            ObDereferenceObject(FileObject as PVOID);
            return status;
        }
        if let Err(status) = probe_user_buffer(Buffer as *const core::ffi::c_void, Length as usize, 1, PreviousMode) {
            ObDereferenceObject(FileObject as PVOID);
            return status;
        }

        if !ByteOffset.is_null() {
            // C: inside _SEH2_TRY; the handler dereferences FileObject and
            // returns the exception code.
            match ntkrnl_sys::probe::probe_read_generic(ByteOffset as *const LARGE_INTEGER, 1, PreviousMode) {
                Ok(offset) => CapturedByteOffset = offset,
                Err(status) => {
                    ObDereferenceObject(FileObject as PVOID);
                    return status;
                }
            }
        }

        if !(*FileObject).CompletionContext.is_null() && ApcRoutine.is_some() {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_INVALID_PARAMETER;
        }

        if (*FileObject).Flags & FO_NO_INTERMEDIATE_BUFFERING != 0 {
            if (*DeviceObject).SectorSize != 0
                && (((*DeviceObject).SectorSize as ULONG - 1) & Length) != 0
            {
                if Length % (*DeviceObject).SectorSize as ULONG != 0 {
                    ObDereferenceObject(FileObject as PVOID);
                    return STATUS_INVALID_PARAMETER;
                }
            }

            if ((Buffer as ULONG_PTR) & (*DeviceObject).AlignmentRequirement as ULONG_PTR) != 0 {
                ObDereferenceObject(FileObject as PVOID);
                return STATUS_INVALID_PARAMETER;
            }

            if !ByteOffset.is_null() {
                if (*DeviceObject).SectorSize != 0
                    && (CapturedByteOffset.QuadPart % (*DeviceObject).SectorSize as i64 != 0)
                {
                    if CapturedByteOffset.QuadPart != FILE_WRITE_TO_END_OF_FILE as i64
                        && (CapturedByteOffset.QuadPart != FILE_USE_FILE_POINTER_POSITION as i64
                            || (*FileObject).Flags & FO_SYNCHRONOUS_IO == 0)
                    {
                        ObDereferenceObject(FileObject as PVOID);
                        return STATUS_INVALID_PARAMETER;
                    }
                }
            }
        }

        if !Key.is_null() {
            CapturedKey = *Key;
        }
    } else {
        if !ByteOffset.is_null() {
            CapturedByteOffset = *ByteOffset;
        }
        if !Key.is_null() {
            CapturedKey = *Key;
        }
    }

    if CapturedByteOffset.QuadPart < -2 {
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    if (ObjectHandleInfo.GrantedAccess & (FILE_APPEND_DATA | FILE_WRITE_DATA)) == FILE_APPEND_DATA {
        CapturedByteOffset.u.LowPart = FILE_WRITE_TO_END_OF_FILE;
        CapturedByteOffset.u.HighPart = -1;
    }

    if !Event.is_null() {
        Status = ObReferenceObjectByHandle(
            Event,
            EVENT_MODIFY_STATE,
            ExEventObjectType,
            PreviousMode,
            core::ptr::addr_of_mut!(EventObject) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
        KeClearEvent(EventObject);
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            if !EventObject.is_null() {
                ObDereferenceObject(EventObject as PVOID);
            }
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }

        if ByteOffset.is_null()
            || (CapturedByteOffset.u.LowPart == FILE_USE_FILE_POINTER_POSITION
                && CapturedByteOffset.u.HighPart == -1)
        {
            CapturedByteOffset = (*FileObject).CurrentByteOffset;
        }

        if !(*FileObject).PrivateCacheMap.is_null() {
            FastIoDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;
            Success = ((*FastIoDispatch).FastIoWrite.unwrap())(
                FileObject,
                core::ptr::addr_of_mut!(CapturedByteOffset),
                Length,
                1u8,
                CapturedKey,
                Buffer,
                core::ptr::addr_of_mut!(KernelIosb),
                DeviceObject,
            );

            if Success != 0 && KernelIosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
                IopUpdateOperationCount(IopWriteTransfer);
                IopUpdateTransferCount(IopWriteTransfer, KernelIosb.Information as ULONG);
                // C: _SEH2_TRY { store the IOSB } _SEH2_EXCEPT { KernelIosb.Status = code; }
                let written = if Iosb32 {
                    let P = IoStatusBlock as *mut IoStatusBlock32;
                    write_back(core::ptr::addr_of_mut!((*P).Status), KernelIosb.__bindgen_anon_1.Status)
                        .and_then(|()| write_back(core::ptr::addr_of_mut!((*P).Information), KernelIosb.Information as u32))
                } else {
                    write_back(IoStatusBlock, KernelIosb)
                };
                if let Err(e) = written {
                    KernelIosb.__bindgen_anon_1.Status = e;
                }

                if !EventObject.is_null() {
                    KeSetEvent(EventObject, 0, 0u8);
                    ObDereferenceObject(EventObject as PVOID);
                }

                IopUnlockFileObject(FileObject);
                ObDereferenceObject(FileObject as PVOID);
                return KernelIosb.__bindgen_anon_1.Status;
            }
        }

        Synchronous = 1u8;
    } else if ByteOffset.is_null()
        && ((*FileObject).Flags & (FO_NAMED_PIPE | FO_MAILSLOT)) == 0
    {
        if !EventObject.is_null() {
            ObDereferenceObject(EventObject as PVOID);
        }
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));

    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, EventObject, core::ptr::null_mut());
    }

    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).RequestorMode = PreviousMode;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = ApcRoutine;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = ApcContext;
    (*Irp).UserIosb = IoStatusBlock;
    (*Irp).UserEvent = EventObject;
    (*Irp).PendingReturned = 0u8;
    (*Irp).Cancel = 0u8;
    (*Irp).CancelRoutine = None;
    (*Irp).AssociatedIrp.SystemBuffer = core::ptr::null_mut();
    (*Irp).MdlAddress = core::ptr::null_mut();

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_WRITE as u8;
    (*StackPtr).FileObject = FileObject;
    (*StackPtr).Flags = if (*FileObject).Flags & FO_WRITE_THROUGH != 0 {
        SL_WRITE_THROUGH
    } else {
        0
    };
    (*StackPtr).Parameters.Write.Key = CapturedKey;
    (*StackPtr).Parameters.Write.Length = Length;
    (*StackPtr).Parameters.Write.ByteOffset = CapturedByteOffset;

    if (*DeviceObject).Flags & DO_BUFFERED_IO != 0 {
        if Length != 0 {
            (*Irp).AssociatedIrp.SystemBuffer =
                ExAllocatePoolWithQuotaTag(NonPagedPool, Length as _, TAG_IOBUF);
            // Fault-tolerant capture of the caller's write buffer. The C original
            // wrapped this copy in _SEH2_TRY with an IopCleanupAfterException
            // handler (iofunc.c:295-310); under DUMMY_PSEH that handler is dead, so
            // a hostile Buffer (kmtest IoReadWrite) faulted memcpy+0x40. This is the
            // third member of the memcpy+0x40 cluster — the *write* side only, which
            // is why reads never crashed: the read path allocates SystemBuffer but
            // copies data OUT, never from a caller-supplied source.
            if let Err(s) = safe_copy_from_user(
                (*Irp).AssociatedIrp.SystemBuffer,
                Buffer,
                Length as usize,
                1,
                PreviousMode,
            ) {
                IopCleanupAfterException(FileObject, Irp, EventObject, core::ptr::null_mut());
                return s;
            }
            (*Irp).Flags = IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
        } else {
            (*Irp).Flags = IRP_BUFFERED_IO;
        }
    } else if (*DeviceObject).Flags & DO_DIRECT_IO != 0 {
        if Length != 0 {
            // C: both the allocation failure (an explicit ExRaiseStatus) and the
            // probe raise into a handler that runs IopCleanupAfterException and
            // returns the code. Without SEH either raise was a bugcheck.
            Mdl = IoAllocateMdl(Buffer, Length, 0u8, 1u8, Irp);
            let st = if Mdl.is_null() {
                STATUS_INSUFFICIENT_RESOURCES
            } else {
                ntkrnl_sys::probe::MmTryProbeAndLockPages(Mdl, PreviousMode, IoReadAccess)
            };
            if st < 0 {
                IopCleanupAfterException(FileObject, Irp, EventObject, core::ptr::null_mut());
                return st;
            }
        }
        (*Irp).Flags = 0;
    } else {
        (*Irp).Flags = 0;
        (*Irp).UserBuffer = Buffer;
    }

    (*Irp).Flags |= IRP_WRITE_OPERATION | IRP_DEFER_IO_COMPLETION;
    if Iosb32 {
        (*Irp).Flags |= IRP_WOW64_IOSB32;
    }
    if (*FileObject).Flags & FO_NO_INTERMEDIATE_BUFFERING != 0 {
        (*Irp).Flags |= IRP_NOCACHE;
    }

    IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        1u8,
        PreviousMode,
        Synchronous,
        IopWriteTransfer,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtWriteFileGather(
    FileHandle: HANDLE,
    Event: HANDLE,
    UserApcRoutine: PIO_APC_ROUTINE,
    UserApcContext: PVOID,
    UserIoStatusBlock: PIO_STATUS_BLOCK,
    BufferDescription: *const FILE_SEGMENT_ELEMENT,
    BufferLength: ULONG,
    ByteOffset: PLARGE_INTEGER,
    Key: *mut ULONG,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let Event = opaque_ptr(Event);
    let UserApcContext = opaque_ptr(UserApcContext);
    let UserIoStatusBlock = opaque_ptr(UserIoStatusBlock);
    let ByteOffset = opaque_ptr(ByteOffset);
    let Key = opaque_ptr(Key);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryVolumeInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FsInformation: PVOID,
    Length: ULONG,
    FsInformationClass: FS_INFORMATION_CLASS,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let FsInformation = opaque_ptr(FsInformation);
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Irp: PIRP;
    let mut StackPtr: PIO_STACK_LOCATION;
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut LocalEvent: BOOLEAN = 0u8;
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    if PreviousMode != 0i8 {
        if FsInformationClass >= FileFsMaximumInformation
            || c_array_get(&IopQueryFsOperationLength, FsInformationClass as usize) as ULONG == 0
        {
            return STATUS_INVALID_INFO_CLASS;
        }

        if Length < c_array_get(&IopQueryFsOperationLength, FsInformationClass as usize) as ULONG as ULONG {
            return STATUS_INFO_LENGTH_MISMATCH;
        }

        // C: under _SEH2_TRY, whose handler returns the exception code.
        if let Err(status) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            1,
            PreviousMode,
        ) {
            return status;
        }
        if let Err(status) = probe_user_buffer(FsInformation as *const core::ffi::c_void, Length as usize, 4, PreviousMode) {
            return status;
        }
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        c_array_get(&IopQueryFsOperationAccess, FsInformationClass as usize),
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if (*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0
        && FsInformationClass != FileFsDeviceInformation
    {
        ObDereferenceObject(FileObject as PVOID);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            return Status;
        }
    }

    if FsInformationClass == FileFsDeviceInformation
        && ((*FileObject).Flags & FO_DIRECT_DEVICE_OPEN != 0
            || (*(*FileObject).DeviceObject).DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM)
    {
        DeviceObject = (*FileObject).DeviceObject;

        let mut FsDeviceInfo: FILE_FS_DEVICE_INFORMATION = core::mem::zeroed();
        FsDeviceInfo.DeviceType = (*DeviceObject).DeviceType;
        FsDeviceInfo.Characteristics = (*DeviceObject).Characteristics;
        if IopGetMountFlag(DeviceObject) != 0 {
            FsDeviceInfo.Characteristics |= FILE_DEVICE_IS_MOUNTED;
        }

        // C: the stores into FsInformation and IoStatusBlock are one _SEH2_TRY whose
        // handler returns the code.
        let iosb = _IO_STATUS_BLOCK {
            __bindgen_anon_1: _IO_STATUS_BLOCK__bindgen_ty_1 { Status: STATUS_SUCCESS },
            Information: core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as _,
        };
        let Status = match write_user(FsInformation as PFILE_FS_DEVICE_INFORMATION, FsDeviceInfo, 0)
            .and_then(|()| write_user(IoStatusBlock, iosb, 0))
        {
            Ok(()) => STATUS_SUCCESS,
            Err(e) => e,
        };

        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            IopUnlockFileObject(FileObject);
        }

        ObDereferenceObject(FileObject as PVOID);
        return Status;
    } else if FsInformationClass == FileFsDriverPathInformation {
        let mut DriverPathInfo: PVOID = core::ptr::null_mut();

        // C: the capture, the DriverInPath store and the IoStatusBlock stores are one
        // _SEH2_TRY whose handler returns the code. (The quota allocation raises on
        // failure in the C; here it is checked.)
        DriverPathInfo = ExAllocatePoolWithQuotaTag(NonPagedPool, Length as _, TAG_IO);
        Status = if DriverPathInfo.is_null() {
            STATUS_INSUFFICIENT_RESOURCES
        } else {
            match safe_copy_raw(DriverPathInfo, FsInformation as *const core::ffi::c_void, Length as usize) {
                Ok(()) => IopGetDriverPathInformation(
                    FileObject,
                    DriverPathInfo as PFILE_FS_DRIVER_PATH_INFORMATION,
                    Length,
                ),
                Err(e) => e,
            }
        };

        if Status >= 0 {
            let iosb = _IO_STATUS_BLOCK {
                __bindgen_anon_1: _IO_STATUS_BLOCK__bindgen_ty_1 { Status: STATUS_SUCCESS },
                Information: core::mem::size_of::<FILE_FS_DRIVER_PATH_INFORMATION>() as _,
            };
            let written = write_user(
                core::ptr::addr_of_mut!((*(FsInformation as PFILE_FS_DRIVER_PATH_INFORMATION)).DriverInPath),
                (*(DriverPathInfo as PFILE_FS_DRIVER_PATH_INFORMATION)).DriverInPath,
                0,
            )
            .and_then(|()| write_user(IoStatusBlock, iosb, 0));
            if let Err(e) = written {
                Status = e;
            }
        }

        if !DriverPathInfo.is_null() {
            ExFreePoolWithTag(DriverPathInfo, TAG_IO);
        }

        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            IopUnlockFileObject(FileObject);
        }

        ObDereferenceObject(FileObject as PVOID);
        return Status;
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO == 0 {
        Event = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<KEVENT>(),
            TAG_IO,
        ) as PKEVENT;
        if Event.is_null() {
            ObDereferenceObject(FileObject as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        KeInitializeEvent(Event, 1, 0u8);
        LocalEvent = 1u8;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));

    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), Event as PVOID);
    }

    (*Irp).RequestorMode = PreviousMode;
    (*Irp).Flags = if LocalEvent != 0 { IRP_SYNCHRONOUS_API } else { 0 };
    (*Irp).UserIosb = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(KernelIosb)
    } else {
        IoStatusBlock
    };
    (*Irp).UserEvent = if LocalEvent != 0 { Event } else { core::ptr::null_mut() };
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).UserBuffer = FsInformation;
    (*Irp).AssociatedIrp.SystemBuffer = core::ptr::null_mut();
    (*Irp).MdlAddress = core::ptr::null_mut();

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_QUERY_VOLUME_INFORMATION as u8;
    (*StackPtr).FileObject = FileObject;

    (*Irp).AssociatedIrp.SystemBuffer =
        ExAllocatePoolWithQuotaTag(NonPagedPool, Length as _, TAG_IOBUF);
    (*Irp).Flags |=
        IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_INPUT_OPERATION | IRP_DEFER_IO_COMPLETION;
    (*StackPtr).Parameters.QueryVolume.Length = Length;
    (*StackPtr).Parameters.QueryVolume.FsInformationClass = FsInformationClass as u32;

    Status = IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        1u8,
        PreviousMode,
        (LocalEvent == 0) as u8,
        IopOtherTransfer,
    );

    if LocalEvent != 0 {
        Status = IopFinalizeAsynchronousIo(
            Status,
            Event,
            Irp,
            PreviousMode,
            core::ptr::addr_of_mut!(KernelIosb),
            IoStatusBlock,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetVolumeInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FsInformation: PVOID,
    Length: ULONG,
    FsInformationClass: FS_INFORMATION_CLASS,
) -> NTSTATUS {
    let FileHandle = opaque_ptr(FileHandle);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let FsInformation = opaque_ptr(FsInformation);
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Irp: PIRP;
    let mut StackPtr: PIO_STACK_LOCATION;
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut TargetDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut LocalEvent: BOOLEAN = 0u8;
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut KernelIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut NotificationStructure: TARGET_DEVICE_CUSTOM_NOTIFICATION = core::mem::zeroed();

    if PreviousMode != 0i8 {
        if FsInformationClass >= FileFsMaximumInformation
            || c_array_get(&IopSetFsOperationLength, FsInformationClass as usize) as ULONG == 0
        {
            return STATUS_INVALID_INFO_CLASS;
        }

        if Length < c_array_get(&IopSetFsOperationLength, FsInformationClass as usize) as ULONG as ULONG {
            return STATUS_INFO_LENGTH_MISMATCH;
        }

        // C: under _SEH2_TRY, whose handler returns the exception code.
        if let Err(status) = probe_user_buffer(
            IoStatusBlock as *const core::ffi::c_void,
            core::mem::size_of::<_IO_STATUS_BLOCK>(),
            1,
            PreviousMode,
        ) {
            return status;
        }
        if let Err(status) = probe_user_buffer(FsInformation as *const core::ffi::c_void, Length as usize, 4, PreviousMode) {
            return status;
        }
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        c_array_get(&IopSetFsOperationAccess, FsInformationClass as usize),
        IoFileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    Status = IoGetRelatedTargetDevice(FileObject, core::ptr::addr_of_mut!(TargetDeviceObject));
    if Status < 0 {
        TargetDeviceObject = core::ptr::null_mut();
    }

    if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
        Status = IopLockFileObject(FileObject, PreviousMode);
        if Status != STATUS_SUCCESS {
            ObDereferenceObject(FileObject as PVOID);
            if !TargetDeviceObject.is_null() {
                ObDereferenceObject(TargetDeviceObject as PVOID);
            }
            return Status;
        }
    } else {
        Event = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<KEVENT>(),
            TAG_IO,
        ) as PKEVENT;
        if Event.is_null() {
            ObDereferenceObject(FileObject as PVOID);
            if !TargetDeviceObject.is_null() {
                ObDereferenceObject(TargetDeviceObject as PVOID);
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        KeInitializeEvent(Event, 1, 0u8);
        LocalEvent = 1u8;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    KeClearEvent(core::ptr::addr_of_mut!((*FileObject).Event));

    Irp = IoAllocateIrp((*DeviceObject).StackSize as CCHAR, 0u8);
    if Irp.is_null() {
        if !TargetDeviceObject.is_null() {
            ObDereferenceObject(TargetDeviceObject as PVOID);
        }
        return IopCleanupFailedIrp(FileObject, core::ptr::null_mut(), Event as PVOID);
    }

    (*Irp).RequestorMode = PreviousMode;
    (*Irp).Flags = if LocalEvent != 0 { IRP_SYNCHRONOUS_API } else { 0 };
    (*Irp).UserIosb = if LocalEvent != 0 {
        core::ptr::addr_of_mut!(KernelIosb)
    } else {
        IoStatusBlock
    };
    (*Irp).UserEvent = if LocalEvent != 0 { Event } else { core::ptr::null_mut() };
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Overlay.AsynchronousParameters.__bindgen_anon_1.UserApcRoutine = None;
    (*Irp).UserBuffer = FsInformation;
    (*Irp).AssociatedIrp.SystemBuffer = core::ptr::null_mut();
    (*Irp).MdlAddress = core::ptr::null_mut();

    StackPtr = IoGetNextIrpStackLocation(Irp);
    (*StackPtr).MajorFunction = IRP_MJ_SET_VOLUME_INFORMATION as u8;
    (*StackPtr).FileObject = FileObject;

    (*Irp).AssociatedIrp.SystemBuffer =
        ExAllocatePoolWithQuotaTag(NonPagedPool, Length as _, TAG_IOBUF);
    core::ptr::copy_nonoverlapping(
        FsInformation as *const u8,
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        Length as usize,
    );
    (*Irp).Flags |= IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER;
    (*StackPtr).Parameters.SetVolume.Length = Length;
    (*StackPtr).Parameters.SetVolume.FsInformationClass = FsInformationClass as u32;

    Status = IopPerformSynchronousRequest(
        DeviceObject,
        Irp,
        FileObject,
        0u8,
        PreviousMode,
        (LocalEvent == 0) as u8,
        IopOtherTransfer,
    );

    if LocalEvent != 0 {
        Status = IopFinalizeAsynchronousIo(
            Status,
            Event,
            Irp,
            PreviousMode,
            core::ptr::addr_of_mut!(KernelIosb),
            IoStatusBlock,
        );
    }

    if !TargetDeviceObject.is_null() && Status >= 0 {
        NotificationStructure.Version = 1;
        NotificationStructure.Size = core::mem::size_of::<TARGET_DEVICE_CUSTOM_NOTIFICATION>() as u16;
        NotificationStructure.Event = GUID_IO_VOLUME_NAME_CHANGE;
        NotificationStructure.FileObject = core::ptr::null_mut();
        NotificationStructure.NameBufferOffset = -1;
        Status = IoReportTargetDeviceChange(
            TargetDeviceObject,
            core::ptr::addr_of_mut!(NotificationStructure) as PVOID,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtCancelDeviceWakeupRequest(
    DeviceHandle: HANDLE,
) -> NTSTATUS {
    let DeviceHandle = opaque_ptr(DeviceHandle);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtRequestDeviceWakeup(
    DeviceHandle: HANDLE,
) -> NTSTATUS {
    let DeviceHandle = opaque_ptr(DeviceHandle);
    STATUS_NOT_IMPLEMENTED
}
