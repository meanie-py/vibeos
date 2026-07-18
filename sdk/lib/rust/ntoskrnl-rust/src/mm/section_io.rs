//! Translated from `ntoskrnl/cache/section/io.c`.
//!
//! Synchronous paging-or-buffered read used by the legacy MM section/sysldr
//! during PE image load. The `#ifdef NEWCC` writeback tail of the original
//! (`_MiSimpleWrite`, `_MiWriteBackPage`) is intentionally omitted — NewCC
//! is dead code in this tree (see `ntos.cmake` `NEWCC` gate).

use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PFILE_OBJECT, PIRP, PMDL, PIO_STACK_LOCATION, PIO_STATUS_BLOCK,
    PLARGE_INTEGER, KEVENT, _IRP, _DEVICE_OBJECT,
    IoBuildAsynchronousFsdRequest, IoFreeMdl, IoGetNextIrpStackLocation,
    IoGetRelatedDeviceObject, KeInitializeEvent, KeWaitForSingleObject,
    MmUnlockPages,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Suspended,
    _MODE_KernelMode,
    KWAIT_REASON, KPROCESSOR_MODE,
};

// IoCallDriver is a FORCEINLINE/macro in C — provide our own extern decl matching
// the kernel ABI (stdcall on i386). Same pattern as iomgr/iofunc.rs:75.
extern "system" {
    fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn PsGetCurrentThread() -> *mut core::ffi::c_void;
}

extern "C" {
    fn ObReferenceObject(Object: PVOID) -> isize;
}

// Match the canonical Windows DDK / WDM IRP flag bits (also defined locally in iomgr/iofunc.rs).
const IRP_NOCACHE: u32                 = 0x00000001;
const IRP_PAGING_IO: u32               = 0x00000002;
const IRP_SYNCHRONOUS_API: u32         = 0x00000004;
const IRP_SYNCHRONOUS_PAGING_IO: u32   = 0x00000040;

const IRP_MJ_READ: u32          = 0x03;
const SL_INVOKE_ON_SUCCESS: u8  = 0x40;
const SL_INVOKE_ON_ERROR: u8    = 0x80;

// STATUS_INTERNAL_ERROR (0xC00000E5) is not in ntkrnl-sys/base or constants — define locally.
const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5_u32 as i32;
const STATUS_END_OF_FILE: NTSTATUS    = 0xC0000011_u32 as i32;
const STATUS_PENDING: NTSTATUS        = 0x00000103;

#[no_mangle]
pub unsafe extern "system" fn MmGetDeviceObjectForFile(FileObject: PFILE_OBJECT) -> PDEVICE_OBJECT {
    IoGetRelatedDeviceObject(FileObject)
}

/// Paging-IO completion routine. Required because paging-IO completion does
/// almost nothing and won't free the MDL chain on its own.
unsafe extern "system" fn MiSimpleReadComplete(
    _DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
    _Context: PVOID,
) -> NTSTATUS {
    // Unlock every MDL in the chain.
    let mut Mdl: PMDL = (*Irp).MdlAddress;
    while !Mdl.is_null() {
        MmUnlockPages(Mdl);
        Mdl = (*Mdl).Next;
    }

    // Free every MDL in the chain.
    loop {
        let head: PMDL = (*Irp).MdlAddress;
        if head.is_null() { break; }
        (*Irp).MdlAddress = (*head).Next;
        IoFreeMdl(head);
    }

    STATUS_SUCCESS
}

/// Convenience function for paging-or-buffered reads. The `Paging` flag picks
/// between an `IRP_PAGING_IO` and an ordinary buffered read; the caller's
/// `ReadStatus` is filled with the final _IO_STATUS_BLOCK.
#[no_mangle]
pub unsafe extern "system" fn MiSimpleRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Buffer: PVOID,
    Length: ULONG,
    Paging: BOOLEAN,
    ReadStatus: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    debug_assert!(!FileObject.is_null());
    debug_assert!(!FileOffset.is_null());
    debug_assert!(!Buffer.is_null());
    debug_assert!(!ReadStatus.is_null());

    let DeviceObject: PDEVICE_OBJECT = MmGetDeviceObjectForFile(FileObject);
    debug_assert!(!DeviceObject.is_null());

    // Pre-seed an internal-error sentinel so callers see a non-success status
    // unless completion fills the block in.
    (*ReadStatus).__bindgen_anon_1.Status = STATUS_INTERNAL_ERROR;
    (*ReadStatus).Information = 0;

    let mut ReadWait: KEVENT = core::mem::zeroed();
    KeInitializeEvent(
        core::ptr::addr_of_mut!(ReadWait),
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );

    let Irp: PIRP = IoBuildAsynchronousFsdRequest(
        IRP_MJ_READ,
        DeviceObject,
        Buffer,
        Length,
        FileOffset,
        ReadStatus,
    );
    if Irp.is_null() {
        return STATUS_NO_MEMORY;
    }

    let paging_flags: u32 = if Paging != 0 {
        IRP_PAGING_IO | IRP_SYNCHRONOUS_PAGING_IO | IRP_NOCACHE
    } else {
        0
    };
    (*Irp).Flags |= paging_flags | IRP_SYNCHRONOUS_API;

    (*Irp).UserEvent = core::ptr::addr_of_mut!(ReadWait);
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as _;

    let IrpSp: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);
    (*IrpSp).Control |= SL_INVOKE_ON_SUCCESS | SL_INVOKE_ON_ERROR;
    (*IrpSp).FileObject = FileObject;
    (*IrpSp).CompletionRoutine = Some(abi_cast!(MiSimpleReadComplete));

    // Non-paging case: the file object will be dereferenced at completion.
    if Paging == 0 {
        ObReferenceObject(FileObject as PVOID);
    }

    let mut Status: NTSTATUS = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        let wait = KeWaitForSingleObject(
            core::ptr::addr_of_mut!(ReadWait) as PVOID,
            _KWAIT_REASON_Suspended as KWAIT_REASON,
            _MODE_KernelMode as KPROCESSOR_MODE,
            0u8,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(wait) {
            // Original C asserts FALSE here; in DUMMY_PSEH builds we just
            // surface the IRP status to the caller.
            return Status;
        }
    }

    // When `Information > 0` is false and `Status == STATUS_END_OF_FILE` is
    // true, the read pointer is past EOF — propagate the EOF status. Otherwise
    // a partial read at EOF is reported as success.
    Status = (*ReadStatus).__bindgen_anon_1.Status;
    if Status == STATUS_END_OF_FILE && (*ReadStatus).Information > 0 {
        STATUS_SUCCESS
    } else {
        Status
    }
}
