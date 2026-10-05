use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ExGetPreviousMode, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IRP, IoAcquireCancelSpinLock, IoBuildSynchronousFsdRequest, IoGetCurrentIrpStackLocation, IoGetFileObjectGenericMapping, IoGetNextIrpStackLocation, IoGetRelatedDeviceObject, IoReleaseCancelSpinLock, IoSetCancelRoutine, IoSetCompletionRoutine, KEVENT, KPROCESSOR_MODE, KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, LPWSTR, MmMapLockedPages, MmProbeAndLockPages, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PDRIVER_CANCEL, PDRIVER_DISPATCH, PDRIVER_OBJECT, PFAST_IO_DISPATCH, PFILE_BASIC_INFORMATION, PFILE_OBJECT, PFILE_STANDARD_INFORMATION, PGENERIC_MAPPING, PIO_COMPLETION_ROUTINE, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PKEVENT, PLARGE_INTEGER, PMDL, PSECURITY_DESCRIPTOR, PUNICODE_STRING, RtlCompareUnicodeString, SeQuerySecurityDescriptorInfo, SeSetSecurityDescriptorInfo};
use core::ffi::c_void;

// ---- cross-module helpers + bindgen-name aliases (used only in this module) ----
use crate::misc::{CompleteRequest, AllocateItem, FreeItem};
use crate::device::{IKsDevice_Create, IKsDevice_Power, IKsDevice_Pnp};
use ntkrnl_sys::ddk::{FILE_INFORMATION_CLASS, IoAllocateMdl, LOCK_OPERATION};
use ntkrnl_sys::ddk::{
    _FILE_INFORMATION_CLASS_FileBasicInformation as FileBasicInformation,
    _FILE_INFORMATION_CLASS_FileStandardInformation as FileStandardInformation,
    _LOCK_OPERATION_IoReadAccess as IoReadAccess,
    _LOCK_OPERATION_IoWriteAccess as IoWriteAccess,
};
use crate::types::{
    KSCOMPLETION_INVOCATION_KsInvokeOnSuccess as KsInvokeOnSuccess,
    KSCOMPLETION_INVOCATION_KsInvokeOnError as KsInvokeOnError,
    KSCOMPLETION_INVOCATION_KsInvokeOnCancel as KsInvokeOnCancel,
    KSLIST_ENTRY_LOCATION_KsListEntryTail as KsListEntryTail,
    KSLIST_ENTRY_LOCATION_KsListEntryHead as KsListEntryHead,
    KSIRP_REMOVAL_OPERATION_KsAcquireAndRemove as KsAcquireAndRemove,
    KSIRP_REMOVAL_OPERATION_KsAcquireOnlySingleItem as KsAcquireOnlySingleItem,
    KSIRP_REMOVAL_OPERATION_KsAcquireAndRemoveOnlySingleItem as KsAcquireAndRemoveOnlySingleItem,
};

// ProbeForRead/ProbeForWrite are real ntoskrnl exports (not in the bindgen layer).
extern "system" {
}

// KS #defines used only here (header-sourced: sdk/include/psdk/ks.h, xdk/iotypes.h)
const KSCREATE_ITEM_SECURITYCHANGED: ULONG = 0x1;
const KSCREATE_ITEM_WILDCARD: ULONG = 0x2;
const IRP_MJ_QUERY_SECURITY: u8 = 0x14;
const IRP_MJ_SET_SECURITY: u8 = 0x15;

// KSCREATE_ITEM_IRP_STORAGE!(Irp) == *(PKSOBJECT_CREATE_ITEM*)&Irp->Tail.Overlay.DriverContext[0]
macro_rules! KSCREATE_ITEM_IRP_STORAGE { ($irp:expr) => {
    *(&mut (*$irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0]
        as *mut PVOID as *mut PKSOBJECT_CREATE_ITEM)
}; }
// KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp) == *(PKSPIN_LOCK*)&Irp->Tail.Overlay.DriverContext[1]
macro_rules! KSQUEUE_SPINLOCK_IRP_STORAGE { ($irp:expr) => {
    *(&mut (*$irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1]
        as *mut PVOID as *mut PKSPIN_LOCK)
}; }

// ---- file-scope preamble (translated) ----
// The C preamble contains only function declarations (not definitions) and no file-scope
// types, constants, or tables. There are no Rust declarations to translate from it.

#[no_mangle]
pub unsafe extern "system" fn KsDispatchQuerySecurity(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let CreateItem = KSCREATE_ITEM_IRP_STORAGE!(Irp);

    if CreateItem.is_null() || (*CreateItem).SecurityDescriptor.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_SECURITY_ON_OBJECT;
        CompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NO_SECURITY_ON_OBJECT;
    }

    let mut Length: ULONG = (*IoStack).Parameters.QuerySecurity.Length;

    let Status = SeQuerySecurityDescriptorInfo(
        &mut (*IoStack).Parameters.QuerySecurity.SecurityInformation,
        (*Irp).UserBuffer as PSECURITY_DESCRIPTOR,
        &mut Length,
        &mut (*CreateItem).SecurityDescriptor,
    );

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = Length as _;

    CompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsDispatchSetSecurity(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let CreateItem: PKSOBJECT_CREATE_ITEM = KSCREATE_ITEM_IRP_STORAGE!(Irp);

    if CreateItem.is_null() || (*CreateItem).SecurityDescriptor.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_SECURITY_ON_OBJECT;
        CompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NO_SECURITY_ON_OBJECT;
    }

    let Descriptor: PSECURITY_DESCRIPTOR = (*CreateItem).SecurityDescriptor;
    let Mapping: PGENERIC_MAPPING = IoGetFileObjectGenericMapping();

    let Status: NTSTATUS = SeSetSecurityDescriptorInfo(
        core::ptr::null_mut(),
        &mut (*IoStack).Parameters.SetSecurity.SecurityInformation,
        (*IoStack).Parameters.SetSecurity.SecurityDescriptor,
        &mut (*CreateItem).SecurityDescriptor,
        NonPagedPool,
        Mapping,
    );

    if NT_SUCCESS(Status) {
        FreeItem(Descriptor);
        (*CreateItem).Flags |= KSCREATE_ITEM_SECURITYCHANGED;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    CompleteRequest(Irp, IO_NO_INCREMENT);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsDispatchSpecificMethod(
    Irp: PIRP,
    Handler: PFNKSHANDLER,
) -> NTSTATUS {
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn KsReadFile(
    FileObject: PFILE_OBJECT,
    mut Event: PKEVENT,
    PortContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    Length: ULONG,
    Key: ULONG,
    RequestorMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let DeviceObject: PDEVICE_OBJECT;
    let IoStack: PIO_STACK_LOCATION;
    let Irp: PIRP;
    let mut Status: NTSTATUS;
    let Result: BOOLEAN;
    let mut LocalEvent: KEVENT = core::mem::zeroed();

    if !Event.is_null() {
        KeClearEvent(Event as _);
    }

    if RequestorMode == 1i8 {
        // C: _SEH2_TRY { ProbeForWrite } whose handler returns the exception code.
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(Buffer, Length as usize, 1, 1) {
            return status;
        }
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    if RequestorMode == 0i8 && ExGetPreviousMode() == 0i8 &&
        !(*((*DeviceObject).DriverObject)).FastIoDispatch.is_null() &&
        !(*(*((*DeviceObject).DriverObject)).FastIoDispatch).FastIoRead.is_none()
    {
        Result = (*(*((*DeviceObject).DriverObject)).FastIoDispatch).FastIoRead.unwrap()(
            FileObject,
            &mut (*FileObject).CurrentByteOffset,
            Length,
            1u8,
            Key,
            Buffer,
            IoStatusBlock,
            DeviceObject,
        );

        if Result != 0 && NT_SUCCESS((*IoStatusBlock).__bindgen_anon_1.Status) {
            return STATUS_SUCCESS;
        }
    }

    if Event.is_null() {
        KeInitializeEvent(&mut LocalEvent, _EVENT_TYPE_NotificationEvent, 0u8);
        Event = &mut LocalEvent as PKEVENT;
    }

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_READ as ULONG,
        DeviceObject,
        Buffer,
        Length,
        &mut (*FileObject).CurrentByteOffset,
        Event,
        IoStatusBlock,
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).RequestorMode = RequestorMode;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = PortContext;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).FileObject = FileObject;
    (*IoStack).Parameters.Read.Key = Key;

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            KeWaitForSingleObject(Event as _, _KWAIT_REASON_Executive, RequestorMode, 0u8, core::ptr::null_mut());
            Status = (*IoStatusBlock).__bindgen_anon_1.Status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsWriteFile(
    FileObject: PFILE_OBJECT,
    mut Event: PKEVENT,
    PortContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    Length: ULONG,
    Key: ULONG,
    RequestorMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let DeviceObject: PDEVICE_OBJECT;
    let IoStack: PIO_STACK_LOCATION;
    let Irp: PIRP;
    let mut Status: NTSTATUS;
    let Result: BOOLEAN;
    let mut LocalEvent: KEVENT = core::mem::zeroed();

    if !Event.is_null() {
        KeClearEvent(Event as _);
    }

    if RequestorMode == 1i8 {
        // C: _SEH2_TRY { ProbeForRead } whose handler returns the exception code.
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(Buffer, Length as usize, 1, 1) {
            return status;
        }
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    if RequestorMode == 0i8 && ExGetPreviousMode() == 0i8 &&
        !(*((*DeviceObject).DriverObject)).FastIoDispatch.is_null() &&
        !(*(*((*DeviceObject).DriverObject)).FastIoDispatch).FastIoWrite.is_none()
    {
        Result = (*(*((*DeviceObject).DriverObject)).FastIoDispatch).FastIoWrite.unwrap()(
            FileObject,
            &mut (*FileObject).CurrentByteOffset,
            Length,
            1u8,
            Key,
            Buffer,
            IoStatusBlock,
            DeviceObject,
        );

        if Result != 0 && NT_SUCCESS((*IoStatusBlock).__bindgen_anon_1.Status) {
            return STATUS_SUCCESS;
        }
    }

    if Event.is_null() {
        KeInitializeEvent(&mut LocalEvent, _EVENT_TYPE_NotificationEvent, 0u8);
        Event = &mut LocalEvent;
    }

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_WRITE as ULONG,
        DeviceObject,
        Buffer,
        Length,
        &mut (*FileObject).CurrentByteOffset,
        Event,
        IoStatusBlock,
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).RequestorMode = RequestorMode;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = PortContext;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).FileObject = FileObject;
    (*IoStack).Parameters.Write.Key = Key;

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        if (*FileObject).Flags & 0x00000001 != 0 {
            KeWaitForSingleObject(Event as _, _KWAIT_REASON_Executive, RequestorMode, 0u8, core::ptr::null_mut());
            Status = (*IoStatusBlock).__bindgen_anon_1.Status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsQueryInformationFile(
    FileObject: PFILE_OBJECT,
    FileInformation: PVOID,
    Length: ULONG,
    FileInformationClass: FILE_INFORMATION_CLASS,
) -> NTSTATUS {
    let DeviceObject: PDEVICE_OBJECT;
    let FastIoDispatch: PFAST_IO_DISPATCH;
    let Irp: PIRP;
    let IoStack: PIO_STACK_LOCATION;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastIoDispatch = (*((*DeviceObject).DriverObject)).FastIoDispatch;

    if !FastIoDispatch.is_null() {
        if FileInformationClass == FileBasicInformation {
            if !(*FastIoDispatch).FastIoQueryBasicInfo.is_none()
                && ((*FastIoDispatch).FastIoQueryBasicInfo).unwrap()(
                    FileObject,
                    1u8,
                    FileInformation as PFILE_BASIC_INFORMATION,
                    &mut IoStatusBlock,
                    DeviceObject,
                ) != 0
            {
                return IoStatusBlock.__bindgen_anon_1.Status;
            }
        } else if FileInformationClass == FileStandardInformation {
            if !(*FastIoDispatch).FastIoQueryStandardInfo.is_none()
                && ((*FastIoDispatch).FastIoQueryStandardInfo).unwrap()(
                    FileObject,
                    1u8,
                    FileInformation as PFILE_STANDARD_INFORMATION,
                    &mut IoStatusBlock,
                    DeviceObject,
                ) != 0
            {
                return IoStatusBlock.__bindgen_anon_1.Status;
            }
        }
    }

    KeClearEvent(&mut (*FileObject).Event as *mut _ as _);
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    Offset.QuadPart = 0i64;

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_QUERY_INFORMATION as ULONG,
        IoGetRelatedDeviceObject(FileObject),
        core::ptr::null_mut(),
        0,
        &mut Offset,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.QueryFile.FileInformationClass = FileInformationClass;
    (*IoStack).Parameters.QueryFile.Length = Length;
    (*Irp).AssociatedIrp.SystemBuffer = FileInformation;

    Status = IoCallDriver(IoGetRelatedDeviceObject(FileObject), Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(&mut Event as *mut _ as _, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            Status = (*FileObject).FinalStatus;
        } else {
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsSetInformationFile(
    FileObject: PFILE_OBJECT,
    FileInformation: PVOID,
    Length: ULONG,
    FileInformationClass: FILE_INFORMATION_CLASS,
) -> NTSTATUS {
    let DeviceObject: PDEVICE_OBJECT;
    let Irp: PIRP;
    let Buffer: PVOID;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);

    Buffer = AllocateItem(NonPagedPool, Length as usize);
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Divergence: the C runs `ProbeForRead(Buffer, ...)` on the pool buffer it
    // just allocated, not on FileInformation. A kernel address always fails that
    // probe, so the C returned STATUS_ACCESS_VIOLATION for every non-empty call.
    // What it meant is a guarded capture of the caller's data; its _SEH2_EXCEPT
    // stores the code, and the failure path below frees the buffer.
    Status = match ntkrnl_sys::probe::safe_copy_raw(Buffer, FileInformation, Length as usize) {
        Ok(()) => STATUS_SUCCESS,
        Err(status) => status,
    };

    if !NT_SUCCESS(Status) {
        FreeItem(Buffer);
        return Status;
    }

    KeInitializeEvent(&mut Event, _EVENT_TYPE_SynchronizationEvent, FALSE);
    Offset.QuadPart = 0i64;

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_SET_INFORMATION as ULONG,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        &mut Offset,
        &mut Event,
        &mut IoStatus,
    );
    if Irp.is_null() {
        FreeItem(Buffer);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).Parameters.SetFile.FileInformationClass = FileInformationClass;
    (*IoStack).Parameters.SetFile.Length = Length;
    (*IoStack).Parameters.SetFile.FileObject = FileObject;
    (*Irp).AssociatedIrp.SystemBuffer = Buffer;
    (*Irp).UserBuffer = FileInformation;

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut _ as _,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE,
            core::ptr::null_mut(),
        );
        if (*FileObject).Flags & FO_SYNCHRONOUS_IO != 0 {
            Status = (*FileObject).FinalStatus;
        } else {
            Status = IoStatus.__bindgen_anon_1.Status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamIo(
    FileObject: PFILE_OBJECT,
    Event: PKEVENT,
    PortContext: PVOID,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
    CompletionInvocationFlags: KSCOMPLETION_INVOCATION,
    IoStatusBlock: PIO_STATUS_BLOCK,
    StreamHeaders: PVOID,
    Length: ULONG,
    Flags: ULONG,
    RequestorMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let DeviceObject = IoGetRelatedDeviceObject(FileObject);
    if !Event.is_null() {
        KeClearEvent(Event as _);
    }
    if RequestorMode != 0 || ExGetPreviousMode() == 0i8 {
        let ObjectHeader = (*FileObject).FsContext2 as *mut KSIOBJECT_HEADER;
        if !ObjectHeader.is_null() {
            if Flags == KSSTREAM_READ {
                if !(*ObjectHeader).DispatchTable.FastRead.is_none() {
                    let Ret = ((*ObjectHeader).DispatchTable.FastRead).unwrap()(
                        FileObject,
                        core::ptr::null_mut(),
                        Length,
                        0u8,
                        0,
                        StreamHeaders,
                        IoStatusBlock,
                        DeviceObject,
                    );
                    if Ret != 0u8 {
                        return (*IoStatusBlock).__bindgen_anon_1.Status;
                    }
                }
            } else if Flags == KSSTREAM_WRITE {
                if !(*ObjectHeader).DispatchTable.FastWrite.is_none() {
                    let Ret = ((*ObjectHeader).DispatchTable.FastWrite).unwrap()(
                        FileObject,
                        core::ptr::null_mut(),
                        Length,
                        0u8,
                        0,
                        StreamHeaders,
                        IoStatusBlock,
                        DeviceObject,
                    );
                    if Ret != 0u8 {
                        return (*IoStatusBlock).__bindgen_anon_1.Status;
                    }
                }
            }
        }
    }
    KeClearEvent(&mut (*FileObject).Event as *mut _ as _);
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    Offset.QuadPart = 0i64;
    let Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_DEVICE_CONTROL as ULONG,
        DeviceObject,
        StreamHeaders,
        Length,
        &mut Offset,
        Event,
        IoStatusBlock,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*Irp).RequestorMode = RequestorMode;
    (*Irp).Overlay.AsynchronousParameters.UserApcContext = PortContext;
    (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
    (*Irp).UserBuffer = StreamHeaders;
    let IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).FileObject = FileObject;
    (*IoStack).Parameters.DeviceIoControl.OutputBufferLength = Length;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = if Flags == KSSTREAM_READ {
        IOCTL_KS_READ_STREAM
    } else {
        IOCTL_KS_WRITE_STREAM
    };
    if CompletionRoutine.is_some() {
        IoSetCompletionRoutine(
            Irp,
            CompletionRoutine,
            CompletionContext,
            if (CompletionInvocationFlags & KsInvokeOnSuccess) != 0 {
                1u8
            } else {
                0u8
            },
            if (CompletionInvocationFlags & KsInvokeOnError) != 0 {
                1u8
            } else {
                0u8
            },
            if (CompletionInvocationFlags & KsInvokeOnCancel) != 0 {
                1u8
            } else {
                0u8
            },
        );
    }
    let Status = IoCallDriver(DeviceObject, Irp);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsProbeStreamIrp(
    Irp: PIRP,
    ProbeFlags: ULONG,
    HeaderSize: ULONG,
) -> NTSTATUS {
    let mut Mdl: PMDL;
    let mut Buffer: *mut core::ffi::c_void;
    let Operation: LOCK_OPERATION;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let IoStack: PIO_STACK_LOCATION;
    let Length: ULONG;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Length = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;

    if (*Irp).RequestorMode == 0i8 || !(*Irp).AssociatedIrp.SystemBuffer.is_null() {
        if (*Irp).RequestorMode == 0i8 {
            /* no need to allocate stream header */
            (*Irp).AssociatedIrp.SystemBuffer = (*Irp).UserBuffer;
        }
    } else {
        /* probe user mode buffers */
        if !(Length != 0
            && (HeaderSize == 0
                || Length % HeaderSize == 0
                || ((ProbeFlags & KSPROBE_ALLOWFORMATCHANGE) != 0
                    && Length as usize == core::mem::size_of::<KSSTREAM_HEADER>())))
        {
            return STATUS_INVALID_BUFFER_SIZE;
        }

        /* allocate stream header buffer */
        (*Irp).AssociatedIrp.SystemBuffer = AllocateItem(NonPagedPool, Length as usize);
        if (*Irp).AssociatedIrp.SystemBuffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        /* mark irp as buffered so that changes the stream headers are propagated back */
        (*Irp).Flags = IRP_DEALLOCATE_BUFFER | IRP_BUFFERED_IO;

        // C: the probes and the copy are one _SEH2_TRY whose handler stores the
        // exception code; failure returns it. ProbeForRead and ProbeForWrite make
        // the same range check, so one Result probe covers each branch.
        if let Err(status) = ntkrnl_sys::probe::safe_copy_from_user(
            (*Irp).AssociatedIrp.SystemBuffer,
            (*Irp).UserBuffer,
            Length as usize,
            1,
            1,
        ) {
            return status;
        }
        if (ProbeFlags & KSPROBE_STREAMWRITE) == 0 {
            /* stream reads means writing */
            (*Irp).Flags |= IRP_INPUT_OPERATION;
        }

        if (ProbeFlags & KSPROBE_ALLOCATEMDL) == 0 {
            /* validate the captured headers; with no MDL requested there is
             * nothing more to do (the C's `goto AllocMdl` returns at once) */
            return ks_check_stream_headers(Irp, ProbeFlags, HeaderSize, Length, false);
        }
        /* else fall through to AllocMdl, which validates and allocates */
    }

    /* AllocMdl: */
    if (ProbeFlags & KSPROBE_ALLOCATEMDL) == 0 {
        /* nothing more to do */
        return STATUS_SUCCESS;
    }
    if (*Irp).MdlAddress.is_null() {
        /* check all stream headers, allocating an MDL for each data frame */
        Status = ks_check_stream_headers(Irp, ProbeFlags, HeaderSize, Length, true);
        if !NT_SUCCESS(Status) {
            return Status;
        }
        if (*Irp).MdlAddress.is_null() {
            /* No header carried a frame, so there is nothing to probe. The C
             * jumps to ProbeMdl regardless and dereferences the NULL MdlAddress. */
            return Status;
        }
    }

    /* ProbeMdl: */
    if (ProbeFlags & KSPROBE_PROBEANDLOCK) != 0 {
        if (*(*Irp).MdlAddress).MdlFlags as u32 & (MDL_PAGES_LOCKED | MDL_SOURCE_IS_NONPAGED_POOL as u32) != 0 {
            if (ProbeFlags & KSPROBE_SYSTEMADDRESS) != 0 {
                Mdl = (*Irp).MdlAddress;
                loop {
                    (*Mdl).MdlFlags |= MDL_MAPPING_CAN_FAIL as i16;
                    if (*Mdl).MdlFlags as u16 & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
                        Buffer = (*Mdl).MappedSystemVa;
                    } else {
                        Buffer = MmMapLockedPages(Mdl, 0i8);
                    }
                    if Buffer.is_null() {
                        /* the C raises this into its own handler, which only
                         * records it as the status (dead under DUMMY_PSEH) */
                        return STATUS_INSUFFICIENT_RESOURCES;
                    }
                    Mdl = (*Mdl).Next;
                    if Mdl.is_null() {
                        break;
                    }
                }
            }
        } else {
            Mdl = (*Irp).MdlAddress;
            if (ProbeFlags & KSPROBE_STREAMWRITE) == 0 || (ProbeFlags & KSPROBE_MODIFY) != 0 {
                Operation = IoWriteAccess;
            } else {
                Operation = IoReadAccess;
            }
            loop {
                MmProbeAndLockPages(Mdl, (*Irp).RequestorMode, Operation);
                if (ProbeFlags & KSPROBE_SYSTEMADDRESS) != 0 {
                    (*Mdl).MdlFlags |= MDL_MAPPING_CAN_FAIL as i16;
                    if (*Mdl).MdlFlags as u16 & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
                        Buffer = (*Mdl).MappedSystemVa;
                    } else {
                        Buffer = MmMapLockedPages(Mdl, 0i8);
                    }
                    if Buffer.is_null() {
                        return STATUS_INSUFFICIENT_RESOURCES;
                    }
                }
                Mdl = (*Mdl).Next;
                if Mdl.is_null() {
                    break;
                }
            }
        }
    }
    Status
}

/// The two "check all stream headers" loops of KsProbeStreamIrp (irp.c). They
/// differ in one place: the AllocMdl pass (`allocate_frames`) also allocates an
/// MDL for every header's data frame; the user-buffer pass only for a format
/// change packet. Each C `ExRaiseStatus(X)` lands in a handler that just records
/// X as the status, so it is returned directly here: under DUMMY_PSEH that
/// handler is dead and the raise would bugcheck instead of failing the IRP.
unsafe fn ks_check_stream_headers(
    Irp: PIRP,
    ProbeFlags: ULONG,
    HeaderSize: ULONG,
    mut Length: ULONG,
    allocate_frames: bool,
) -> NTSTATUS {
    let header_len = core::mem::size_of::<KSSTREAM_HEADER>() as ULONG;
    let mut StreamHeader = (*Irp).AssociatedIrp.SystemBuffer as PKSSTREAM_HEADER;
    if StreamHeader.is_null() {
        return STATUS_INVALID_BUFFER_SIZE;
    }

    loop {
        let size = (*StreamHeader).Size;
        if HeaderSize != 0 {
            /* does the supplied header size match stream header size and no type changed */
            if size != HeaderSize && ((*StreamHeader).OptionsFlags & KSSTREAM_HEADER_OPTIONSF_TYPECHANGED) == 0 {
                return STATUS_INVALID_BUFFER_SIZE;
            }
        } else if size < header_len || (size & 7) != 0 {
            /* stream must be at least of size KSSTREAM_HEADER and 8-byte block aligned */
            return STATUS_INVALID_BUFFER_SIZE;
        }

        if Length < size {
            return STATUS_INVALID_BUFFER_SIZE;
        }
        if size == 0 {
            /* A zero-sized format-change header passes the HeaderSize test and
             * would never advance; the C spins here forever. */
            return STATUS_INVALID_BUFFER_SIZE;
        }

        if (ProbeFlags & KSPROBE_STREAMWRITE) != 0 {
            if (*StreamHeader).DataUsed > (*StreamHeader).FrameExtent {
                /* frame extend can never be smaller */
                return STATUS_INVALID_BUFFER_SIZE;
            }

            /* is this stream change packet */
            if ((*StreamHeader).OptionsFlags & KSSTREAM_HEADER_OPTIONSF_TYPECHANGED) != 0 {
                if Length != header_len || StreamHeader as PVOID != (*Irp).AssociatedIrp.SystemBuffer {
                    /* stream changed - must be send in a single packet */
                    return STATUS_INVALID_BUFFER_SIZE;
                }
                if (ProbeFlags & KSPROBE_ALLOWFORMATCHANGE) == 0 {
                    /* caller does not permit format changes */
                    return STATUS_INVALID_PARAMETER;
                }
                if (*StreamHeader).FrameExtent != 0 {
                    if IoAllocateMdl((*StreamHeader).Data, (*StreamHeader).FrameExtent, 0, 1, Irp).is_null() {
                        return STATUS_INSUFFICIENT_RESOURCES;
                    }
                    /* break-out to probe for the irp */
                    return STATUS_SUCCESS;
                }
            }
        } else {
            if (*StreamHeader).DataUsed != 0 {
                /* DataUsed must be zero for stream read operation */
                return STATUS_INVALID_BUFFER_SIZE;
            }
            if (*StreamHeader).OptionsFlags != 0 {
                /* no flags supported for reading */
                return STATUS_INVALID_PARAMETER;
            }
        }

        if allocate_frames && (*StreamHeader).FrameExtent != 0 {
            /* IoAllocateMdl chains each MDL onto Irp->MdlAddress. (The C asserts
             * MdlAddress == NULL here, which the second header always trips.) */
            if IoAllocateMdl((*StreamHeader).Data, (*StreamHeader).FrameExtent, 0, 1, Irp).is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        /* move to next stream header */
        Length -= size;
        if Length == 0 {
            return STATUS_SUCCESS;
        }
        StreamHeader = (StreamHeader as ULONG_PTR + size as ULONG_PTR) as PKSSTREAM_HEADER;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsAllocateExtraData(
    Irp: PIRP,
    ExtraSize: ULONG,
    ExtraBuffer: *mut PVOID,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Count = (*IoStack).Parameters.DeviceIoControl.InputBufferLength / core::mem::size_of::<KSSTREAM_HEADER>() as ULONG;
    let Buffer = AllocateItem(NonPagedPool, (Count * (core::mem::size_of::<KSSTREAM_HEADER>() as ULONG + ExtraSize)) as usize);
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let BufferOrg = Buffer;
    let mut Status = STATUS_SUCCESS;
    let Header = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as *const KSSTREAM_HEADER;
    let mut Index = 0;
    while Index < Count {
        core::ptr::copy_nonoverlapping(Header.add(Index as usize) as *const u8, Buffer as *mut u8, core::mem::size_of::<KSSTREAM_HEADER>());
        Index += 1;
    }
    if !NT_SUCCESS(Status) {
        FreeItem(Buffer);
        return Status;
    }
    *ExtraBuffer = BufferOrg;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsNullDriverUnload(
    DriverObject: PDRIVER_OBJECT,
) {
}

#[no_mangle]
pub unsafe extern "system" fn KsDispatchInvalidDeviceRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
    CompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_INVALID_DEVICE_REQUEST
}

#[no_mangle]
pub unsafe extern "system" fn KsDefaultDeviceIoCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Status: NTSTATUS;

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_PROPERTY
        && (*IoStack).Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_METHOD
        && (*IoStack).Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_ENABLE_EVENT
        && (*IoStack).Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_DISABLE_EVENT
    {
        if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_RESET_STATE {
            Status = STATUS_SUCCESS;
        } else {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    } else {
        Status = STATUS_PROPSET_NOT_FOUND;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    CompleteRequest(Irp, IO_NO_INCREMENT);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsDispatchFastIoDeviceControlFailure(
    FileObject: PFILE_OBJECT,
    Wait: BOOLEAN,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    IoControlCode: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    return 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn KsDispatchFastReadFailure(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    Buffer: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    return 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn KsCancelIo(
    QueueHead: PLIST_ENTRY,
    SpinLock: PKSPIN_LOCK,
) {
    let mut OldDriverCancel: PDRIVER_CANCEL;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Entry: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut Irp: PIRP;
    let mut OldLevel: KIRQL = 0;

    KeAcquireSpinLock(SpinLock, &mut OldLevel);
    Entry = (*QueueHead).Flink;
    while Entry != QueueHead {
        Irp = CONTAINING_RECORD!(Entry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as PIRP;
        NextEntry = (*Entry).Flink;
        (*Irp).Cancel = 1u8;
        OldDriverCancel = IoSetCancelRoutine(Irp, None);
        if OldDriverCancel.is_some() {
            KeReleaseSpinLock(SpinLock, OldLevel);
            IoStack = IoGetCurrentIrpStackLocation(Irp);
            IoAcquireCancelSpinLock(&mut (*Irp).CancelIrql);
            OldDriverCancel.unwrap()((*IoStack).DeviceObject, Irp);
            KeAcquireSpinLock(SpinLock, &mut OldLevel);
        }
        Entry = NextEntry;
    }
    KeReleaseSpinLock(SpinLock, OldLevel);
}

#[no_mangle]
pub unsafe extern "system" fn KsReleaseIrpOnCancelableQueue(
    Irp: PIRP,
    DriverCancel: PDRIVER_CANCEL,
) {
    let SpinLock: PKSPIN_LOCK;
    let OldDriverCancel: PDRIVER_CANCEL;
    let IoStack: PIO_STACK_LOCATION;
    let mut OldLevel: KIRQL = 0;

    if Irp.is_null() {
        return;
    }

    let DriverCancel: PDRIVER_CANCEL = if DriverCancel.is_some() {
        DriverCancel
    } else {
        crate::cb!(KsCancelRoutine)
    };

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    SpinLock = KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp);

    KeAcquireSpinLock(SpinLock, &mut OldLevel);

    OldDriverCancel = IoSetCancelRoutine(Irp, DriverCancel);

    if (*Irp).Cancel != 0 && OldDriverCancel.is_none() {
        KeReleaseSpinLock(SpinLock, OldLevel);
        IoAcquireCancelSpinLock(&mut (*Irp).CancelIrql);
        DriverCancel.unwrap()((*IoStack).DeviceObject, Irp);
    } else {
        KeReleaseSpinLock(SpinLock, OldLevel);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsRemoveIrpFromCancelableQueue(
    QueueHead: PLIST_ENTRY,
    SpinLock: PKSPIN_LOCK,
    ListLocation: KSLIST_ENTRY_LOCATION,
    RemovalOperation: KSIRP_REMOVAL_OPERATION,
) -> PIRP {
    let mut Irp: PIRP;
    let mut CurEntry: PLIST_ENTRY;
    let mut OldIrql: KIRQL = 0;

    if QueueHead.is_null() || SpinLock.is_null() {
        return core::ptr::null_mut();
    }

    if ListLocation != KsListEntryTail && ListLocation != KsListEntryHead {
        return core::ptr::null_mut();
    }

    KeAcquireSpinLock(SpinLock, &mut OldIrql);

    CurEntry = QueueHead;

    loop {
        Irp = core::ptr::null_mut();

        if ListLocation == KsListEntryHead {
            CurEntry = (*CurEntry).Flink;
        } else {
            CurEntry = (*CurEntry).Blink;
        }

        if CurEntry == QueueHead {
            break;
        }

        Irp = CONTAINING_RECORD!(CurEntry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as PIRP;

        if (*Irp).Cancel != 0 {
            break;
        }

        if (*Irp).CancelRoutine.is_some() {
            (*Irp).CancelRoutine = None;

            if RemovalOperation == KsAcquireAndRemove || RemovalOperation == KsAcquireAndRemoveOnlySingleItem {
                RemoveEntryList(&mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);
            }

            if RemovalOperation == KsAcquireAndRemoveOnlySingleItem || RemovalOperation == KsAcquireOnlySingleItem {
                break;
            }
        }
    }

    KeReleaseSpinLock(SpinLock, OldIrql);

    if Irp.is_null() || (*Irp).CancelRoutine.is_none() {
        return Irp;
    }

    IoAcquireCancelSpinLock(&mut OldIrql);
    KeAcquireSpinLockAtDpcLevel(SpinLock);

    if RemovalOperation == KsAcquireAndRemove || RemovalOperation == KsAcquireAndRemoveOnlySingleItem {
        RemoveEntryList(&mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);
    }

    KeReleaseSpinLockFromDpcLevel(SpinLock);
    IoReleaseCancelSpinLock(OldIrql);

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn KsMoveIrpsOnCancelableQueue(
    SourceList: PLIST_ENTRY,
    SourceLock: PKSPIN_LOCK,
    DestinationList: PLIST_ENTRY,
    DestinationLock: PKSPIN_LOCK,
    ListLocation: KSLIST_ENTRY_LOCATION,
    ListCallback: PFNKSIRPLISTCALLBACK,
    Context: PVOID,
) -> NTSTATUS {
    let mut OldLevel: KIRQL = 0;
    let mut SrcEntry: PLIST_ENTRY;
    let mut Irp: PIRP;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if DestinationLock.is_null() {
        KeAcquireSpinLock(SourceLock, &mut OldLevel);
    } else {
        IoAcquireCancelSpinLock(&mut OldLevel);
        KeAcquireSpinLockAtDpcLevel(SourceLock);
        KeAcquireSpinLockAtDpcLevel(DestinationLock);
    }

    SrcEntry = SourceList;

    loop {
        if ListLocation == KsListEntryTail {
            SrcEntry = (*SrcEntry).Flink;
        } else {
            SrcEntry = (*SrcEntry).Blink;
        }

        if SrcEntry == SourceList {
            break;
        }

        Irp = CONTAINING_RECORD!(
            SrcEntry,
            IRP,
            Tail.Overlay.__bindgen_anon_2.ListEntry,
        );

        Status = ListCallback.unwrap()(Irp, Context);

        if Status == STATUS_SUCCESS {
            RemoveEntryList(&mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);

            if ListLocation == KsListEntryTail {
                InsertTailList(DestinationList, &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);
            } else {
                InsertHeadList(DestinationList, &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);
            }

            if !DestinationLock.is_null() {
                KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp) = DestinationLock;
            }
        } else {
            if Status != STATUS_NO_MATCH {
                break;
            }
            Status = STATUS_SUCCESS;
        }
    }

    if DestinationLock.is_null() {
        KeReleaseSpinLock(SourceLock, OldLevel);
    } else {
        KeReleaseSpinLockFromDpcLevel(DestinationLock);
        KeReleaseSpinLockFromDpcLevel(SourceLock);
        IoReleaseCancelSpinLock(OldLevel);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsRemoveSpecificIrpFromCancelableQueue(
    Irp: PIRP,
) {
    let SpinLock: PKSPIN_LOCK;
    let mut OldLevel: KIRQL = 0;

    SpinLock = KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp);

    KeAcquireSpinLock(SpinLock, &mut OldLevel);

    RemoveEntryList(&mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);

    KeReleaseSpinLock(SpinLock, OldLevel);
}

#[no_mangle]
pub unsafe extern "system" fn KsAddIrpToCancelableQueue(
    QueueHead: PLIST_ENTRY,
    SpinLock: PKSPIN_LOCK,
    Irp: PIRP,
    ListLocation: KSLIST_ENTRY_LOCATION,
    DriverCancel: PDRIVER_CANCEL,
) {
    let OldDriverCancel: PDRIVER_CANCEL;
    let IoStack: PIO_STACK_LOCATION;
    let mut OldLevel: KIRQL = 0;

    if QueueHead.is_null() || SpinLock.is_null() || Irp.is_null() {
        return;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).MajorFunction == IRP_MJ_CREATE {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        CompleteRequest(Irp, IO_NO_INCREMENT);
        return;
    }

    let DriverCancel: PDRIVER_CANCEL = if DriverCancel.is_none() {
        crate::cb!(KsCancelRoutine)
    } else {
        DriverCancel
    };

    KeAcquireSpinLock(SpinLock, &mut OldLevel);

    if ListLocation == KsListEntryTail {
        InsertTailList(QueueHead, &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);
    } else {
        InsertHeadList(QueueHead, &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);
    }

    KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp) = SpinLock;

    OldDriverCancel = IoSetCancelRoutine(Irp, DriverCancel);

    if (*Irp).Cancel != 0 && OldDriverCancel.is_none() {
        KeReleaseSpinLock(SpinLock, OldLevel);
        IoAcquireCancelSpinLock(&mut (*Irp).CancelIrql);
        DriverCancel.unwrap()((*IoStack).DeviceObject, Irp);
    } else {
        KeReleaseSpinLock(SpinLock, OldLevel);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsCancelRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let SpinLock: PKSPIN_LOCK;

    SpinLock = KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp);

    KeAcquireSpinLockAtDpcLevel(SpinLock);

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    RemoveEntryList(&mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);

    KeReleaseSpinLock(SpinLock, (*Irp).CancelIrql);

    if (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_CANCELLED {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        CompleteRequest(Irp, IO_NO_INCREMENT);
    }
}
#[no_mangle]
pub unsafe extern "C" fn FindMatchingCreateItem(
    ListHead: PLIST_ENTRY,
    String: PUNICODE_STRING,
    OutCreateItem: *mut PCREATE_ITEM_ENTRY,
) -> NTSTATUS {
    let mut Entry: PLIST_ENTRY;
    let mut CreateItemEntry: PCREATE_ITEM_ENTRY;
    let mut RefString: _UNICODE_STRING;
    let mut pStr: LPWSTR;
    let mut Count: ULONG;

    RefString = *String;

    if *(*String).Buffer as u16 == b'\\' as u16 {
        RefString.Buffer = RefString.Buffer.offset(1);
        RefString.Length -= core::mem::size_of::<u16>() as u16;
    } else {
        pStr = (*String).Buffer;
        Count = (*String).Length as u32 / core::mem::size_of::<u16>() as u32;
        while (Count > 0) && (*pStr as u16 != b'\\' as u16) {
            pStr = pStr.offset(1);
            Count -= 1;
        }

        RefString.Length = (pStr as *const u8).offset_from((*String).Buffer as *const u8) as u16;
    }

    Entry = (*ListHead).Flink;

    while Entry != ListHead {
        CreateItemEntry = CONTAINING_RECORD!(Entry, CREATE_ITEM_ENTRY, Entry) as PCREATE_ITEM_ENTRY;

        if (*((*CreateItemEntry).CreateItem)).Flags & KSCREATE_ITEM_WILDCARD != 0 {
            *OutCreateItem = CreateItemEntry;
            return STATUS_SUCCESS;
        }

        if (*((*CreateItemEntry).CreateItem)).Create.is_none() {
            Entry = (*Entry).Flink;
            continue;
        }

        if (*((*CreateItemEntry).CreateItem)).ObjectClass.Length > RefString.Length {
            Entry = (*Entry).Flink;
            continue;
        }

        if RtlCompareUnicodeString(
            &mut (*((*CreateItemEntry).CreateItem)).ObjectClass,
            &RefString as *const _UNICODE_STRING as *mut _UNICODE_STRING,
            TRUE,
        ) == 0
        {
            *OutCreateItem = CreateItemEntry;
            return STATUS_SUCCESS;
        }

        Entry = (*Entry).Flink;
    }

    STATUS_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn KspCreate(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut CreateItemEntry: PCREATE_ITEM_ENTRY = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;
    let DeviceExtension: PDEVICE_EXTENSION;
    let DeviceHeader: PKSIDEVICE_HEADER;
    let ObjectHeader: PKSIOBJECT_HEADER;
    let mut Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    DeviceHeader = (*DeviceExtension).DeviceHeader;

    if (*(*IoStack).FileObject).FileName.Buffer.is_null() {
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        CompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    if !(*(*IoStack).FileObject).RelatedFileObject.is_null() {
        ObjectHeader = (*(*(*IoStack).FileObject).RelatedFileObject).FsContext2 as PKSIOBJECT_HEADER;
        Status = FindMatchingCreateItem(
            &mut (*ObjectHeader).ItemList,
            &mut (*(*IoStack).FileObject).FileName,
            &mut CreateItemEntry,
        );
    } else {
        Status = FindMatchingCreateItem(
            &mut (*DeviceHeader).ItemList,
            &mut (*(*IoStack).FileObject).FileName,
            &mut CreateItemEntry,
        );
    }

    if NT_SUCCESS(Status) {
        KSCREATE_ITEM_IRP_STORAGE!(Irp) = (*CreateItemEntry).CreateItem;
        Status = ((*(*CreateItemEntry).CreateItem).Create).unwrap()(DeviceObject, Irp);
        if NT_SUCCESS(Status) {
            InterlockedIncrement(&mut (*CreateItemEntry).ReferenceCount);
        }
        return Status;
    }

    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    CompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn KspDispatchIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let ObjectHeader: PKSIOBJECT_HEADER;
    let Dispatch: PDRIVER_DISPATCH;
    let Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    ObjectHeader = (*((*IoStack).FileObject)).FsContext2 as PKSIOBJECT_HEADER;

    if ObjectHeader.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = 0;
        CompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    Dispatch = match (*IoStack).MajorFunction {
        IRP_MJ_CLOSE => (*ObjectHeader).DispatchTable.Close,
        IRP_MJ_DEVICE_CONTROL => (*ObjectHeader).DispatchTable.DeviceIoControl,
        IRP_MJ_READ => (*ObjectHeader).DispatchTable.Read,
        IRP_MJ_WRITE => (*ObjectHeader).DispatchTable.Write,
        IRP_MJ_FLUSH_BUFFERS => (*ObjectHeader).DispatchTable.Flush,
        IRP_MJ_QUERY_SECURITY => (*ObjectHeader).DispatchTable.QuerySecurity,
        IRP_MJ_SET_SECURITY => (*ObjectHeader).DispatchTable.SetSecurity,
        IRP_MJ_PNP => Some(KsDefaultDispatchPnp),
        _ => None,
    };

    if Dispatch.is_some() {
        Status = Dispatch.unwrap()(DeviceObject, Irp);
    } else {
        Status = KsDispatchInvalidDeviceRequest(DeviceObject, Irp);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsSetMajorFunctionHandler(
    DriverObject: PDRIVER_OBJECT,
    MajorFunction: ULONG,
) -> NTSTATUS {
    match MajorFunction as u8 {
        IRP_MJ_CREATE => {
            (*DriverObject).MajorFunction[MajorFunction as usize] = crate::cb!(KspCreate);
        }
        IRP_MJ_DEVICE_CONTROL | IRP_MJ_CLOSE | IRP_MJ_READ | IRP_MJ_WRITE | IRP_MJ_FLUSH_BUFFERS | IRP_MJ_QUERY_SECURITY | IRP_MJ_SET_SECURITY => {
            (*DriverObject).MajorFunction[MajorFunction as usize] = crate::cb!(KspDispatchIrp);
        }
        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsDispatchIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: *mut DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as *mut DEVICE_EXTENSION;
    let DeviceHeader: PKSIDEVICE_HEADER = (*DeviceExtension).DeviceHeader;
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).MajorFunction <= IRP_MJ_DEVICE_CONTROL {
        if (*IoStack).MajorFunction == IRP_MJ_CREATE {
            if !(*DeviceHeader).BasicHeader.OuterUnknown.is_null() {
                return IKsDevice_Create(DeviceObject, Irp);
            } else {
                return KspCreate(DeviceObject, Irp);
            }
        }

        match (*IoStack).MajorFunction {
            IRP_MJ_CLOSE | IRP_MJ_READ | IRP_MJ_WRITE | IRP_MJ_FLUSH_BUFFERS
            | IRP_MJ_QUERY_SECURITY | IRP_MJ_SET_SECURITY | IRP_MJ_PNP
            | IRP_MJ_DEVICE_CONTROL => {
                return KspDispatchIrp(DeviceObject, Irp);
            }
            _ => {
                return KsDispatchInvalidDeviceRequest(DeviceObject, Irp);
            }
        }
    }

    if (*IoStack).MajorFunction == IRP_MJ_POWER {
        if !(*DeviceHeader).BasicHeader.OuterUnknown.is_null() {
            return IKsDevice_Power(DeviceObject, Irp);
        } else {
            return KsDefaultDispatchPower(DeviceObject, Irp);
        }
    } else if (*IoStack).MajorFunction == IRP_MJ_PNP {
        if !(*DeviceHeader).BasicHeader.OuterUnknown.is_null() {
            return IKsDevice_Pnp(DeviceObject, Irp);
        } else {
            return KsDefaultDispatchPnp(DeviceObject, Irp);
        }
    } else if (*IoStack).MajorFunction == IRP_MJ_SYSTEM_CONTROL {
        return KsDefaultForwardIrp(DeviceObject, Irp);
    } else {
        return KsDispatchInvalidDeviceRequest(DeviceObject, Irp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsGetNodeIdFromIrp(
    Irp: PIRP,
) -> ULONG {
    // UNIMPLEMENTED
    return KSFILTER_NODE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/factory.c
//  * PURPOSE:         KS Allocator functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #include <ntifs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsDispatchQuerySecurity(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PKSOBJECT_CREATE_ITEM CreateItem;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     ULONG Length;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get create item */
//     CreateItem = KSCREATE_ITEM_IRP_STORAGE!(Irp);
//
//     if (!CreateItem || !CreateItem->SecurityDescriptor)
//     {
//         /* no create item */
//         Irp->IoStatus.Status = STATUS_NO_SECURITY_ON_OBJECT;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_NO_SECURITY_ON_OBJECT;
//     }
//
//
//     /* get input length */
//     Length = IoStack->Parameters.QuerySecurity.Length;
//
//     /* clone the security descriptor */
//     Status = SeQuerySecurityDescriptorInfo(&IoStack->Parameters.QuerySecurity.SecurityInformation, (PSECURITY_DESCRIPTOR)Irp->UserBuffer, &Length, &CreateItem->SecurityDescriptor);
//
//     DPRINT("SeQuerySecurityDescriptorInfo Status %x\n", Status);
//     /* store result */
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = Length;
//
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsDispatchSetSecurity(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PKSOBJECT_CREATE_ITEM CreateItem;
//     PIO_STACK_LOCATION IoStack;
//     PGENERIC_MAPPING Mapping;
//     PSECURITY_DESCRIPTOR Descriptor;
//     NTSTATUS Status;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get create item */
//     CreateItem = KSCREATE_ITEM_IRP_STORAGE!(Irp);
//
//     if (!CreateItem || !CreateItem->SecurityDescriptor)
//     {
//         /* no create item */
//         Irp->IoStatus.Status = STATUS_NO_SECURITY_ON_OBJECT;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_NO_SECURITY_ON_OBJECT;
//     }
//
//     /* backup old descriptor */
//     Descriptor = CreateItem->SecurityDescriptor;
//
//     /* get generic mapping */
//     Mapping = IoGetFileObjectGenericMapping();
//
//     /* change security descriptor */
//     Status = SeSetSecurityDescriptorInfo(NULL, /*FIXME */
//                                          &IoStack->Parameters.SetSecurity.SecurityInformation,
//                                          IoStack->Parameters.SetSecurity.SecurityDescriptor,
//                                          &CreateItem->SecurityDescriptor,
//                                          NonPagedPool,
//                                          Mapping);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* free old descriptor */
//         FreeItem(Descriptor);
//
//        /* mark create item as changed */
//        CreateItem->Flags |= KSCREATE_ITEM_SECURITYCHANGED;
//     }
//
//     /* store result */
//     Irp->IoStatus.Status = Status;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsDispatchSpecificMethod(
//     IN  PIRP Irp,
//     IN  PFNKSHANDLER Handler)
// {
//     // UNIMPLEMENTED
//     return STATUS_UNSUCCESSFUL;
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsReadFile(
//     IN  PFILE_OBJECT FileObject,
//     IN  PKEVENT Event OPTIONAL,
//     IN  PVOID PortContext OPTIONAL,
//     OUT PIO_STATUS_BLOCK IoStatusBlock,
//     OUT PVOID Buffer,
//     IN  ULONG Length,
//     IN  ULONG Key OPTIONAL,
//     IN  KPROCESSOR_MODE RequestorMode)
// {
//     PDEVICE_OBJECT DeviceObject;
//     PIO_STACK_LOCATION IoStack;
//     PIRP Irp;
//     NTSTATUS Status;
//     BOOLEAN Result;
//     KEVENT LocalEvent;
//
//     if (Event)
//     {
//         /* make sure event is reset */
//         KeClearEvent(Event as _);
//     }
//
//     if (RequestorMode == UserMode)
//     {
//         /* probe the user buffer */
//         _SEH2_TRY
//         {
//             ProbeForWrite(Buffer, Length, sizeof(UCHAR));
//             Status = STATUS_SUCCESS;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Exception, get the error code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//          if (!NT_SUCCESS(Status))
//          {
//              DPRINT1("Invalid user buffer provided\n");
//              return Status;
//          }
//     }
//
//     /* get corresponding device object */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//
//     /* fast-io read is only available for kernel mode clients */
//     if (RequestorMode == KernelMode && ExGetPreviousMode() == KernelMode &&
//         DeviceObject->DriverObject->FastIoDispatch->FastIoRead)
//     {
//         /* call fast io write */
//         Result = DeviceObject->DriverObject->FastIoDispatch->FastIoRead(FileObject, &FileObject->CurrentByteOffset, Length, TRUE, Key, Buffer, IoStatusBlock, DeviceObject);
//
//         if (Result && NT_SUCCESS(IoStatusBlock->Status))
//         {
//             /* request was handled and succeeded */
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* do the slow way */
//     if (!Event)
//     {
//         /* initialize temp event */
//         KeInitializeEvent(&LocalEvent, NotificationEvent, FALSE);
//         Event = &LocalEvent;
//     }
//
//     /* build the irp packet */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ, DeviceObject, Buffer, Length, &FileObject->CurrentByteOffset, Event, IoStatusBlock);
//     if (!Irp)
//     {
//         /* not enough resources */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* setup the rest of irp */
//     Irp->RequestorMode = RequestorMode;
//     Irp->Overlay.AsynchronousParameters.UserApcContext = PortContext;
//     Irp->Tail.Overlay.OriginalFileObject = FileObject;
//
//     /* setup irp stack */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->FileObject = FileObject;
//     IoStack->Parameters.Read.Key = Key;
//
//     /* send the packet */
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         /* operation is pending, is sync file object */
//         if (FileObject->Flags & FO_SYNCHRONOUS_IO)
//         {
//             /* it is so wait */
//             KeWaitForSingleObject(Event as _, Executive, RequestorMode, FALSE, NULL);
//             Status = IoStatusBlock->Status;
//         }
//     }
//     /* return result */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsWriteFile(
//     IN  PFILE_OBJECT FileObject,
//     IN  PKEVENT Event OPTIONAL,
//     IN  PVOID PortContext OPTIONAL,
//     OUT PIO_STATUS_BLOCK IoStatusBlock,
//     IN  PVOID Buffer,
//     IN  ULONG Length,
//     IN  ULONG Key OPTIONAL,
//     IN  KPROCESSOR_MODE RequestorMode)
// {
//     PDEVICE_OBJECT DeviceObject;
//     PIO_STACK_LOCATION IoStack;
//     PIRP Irp;
//     NTSTATUS Status;
//     BOOLEAN Result;
//     KEVENT LocalEvent;
//
//     if (Event)
//     {
//         /* make sure event is reset */
//         KeClearEvent(Event as _);
//     }
//
//     if (RequestorMode == UserMode)
//     {
//         /* probe the user buffer */
//         _SEH2_TRY
//         {
//             ProbeForRead(Buffer, Length, sizeof(UCHAR));
//             Status = STATUS_SUCCESS;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Exception, get the error code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//          if (!NT_SUCCESS(Status))
//          {
//              DPRINT1("Invalid user buffer provided\n");
//              return Status;
//          }
//     }
//
//     /* get corresponding device object */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//
//     /* fast-io write is only available for kernel mode clients */
//     if (RequestorMode == KernelMode && ExGetPreviousMode() == KernelMode &&
//         DeviceObject->DriverObject->FastIoDispatch->FastIoWrite)
//     {
//         /* call fast io write */
//         Result = DeviceObject->DriverObject->FastIoDispatch->FastIoWrite(FileObject, &FileObject->CurrentByteOffset, Length, TRUE, Key, Buffer, IoStatusBlock, DeviceObject);
//
//         if (Result && NT_SUCCESS(IoStatusBlock->Status))
//         {
//             /* request was handled and succeeded */
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* do the slow way */
//     if (!Event)
//     {
//         /* initialize temp event */
//         KeInitializeEvent(&LocalEvent, NotificationEvent, FALSE);
//         Event = &LocalEvent;
//     }
//
//     /* build the irp packet */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_WRITE, DeviceObject, Buffer, Length, &FileObject->CurrentByteOffset, Event, IoStatusBlock);
//     if (!Irp)
//     {
//         /* not enough resources */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* setup the rest of irp */
//     Irp->RequestorMode = RequestorMode;
//     Irp->Overlay.AsynchronousParameters.UserApcContext = PortContext;
//     Irp->Tail.Overlay.OriginalFileObject = FileObject;
//
//     /* setup irp stack */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->FileObject = FileObject;
//     IoStack->Parameters.Write.Key = Key;
//
//     /* send the packet */
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         /* operation is pending, is sync file object */
//         if (FileObject->Flags & FO_SYNCHRONOUS_IO)
//         {
//             /* it is so wait */
//             KeWaitForSingleObject(Event as _, Executive, RequestorMode, FALSE, NULL);
//             Status = IoStatusBlock->Status;
//         }
//     }
//     /* return result */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsQueryInformationFile(
//     IN  PFILE_OBJECT FileObject,
//     OUT PVOID FileInformation,
//     IN  ULONG Length,
//     IN  FILE_INFORMATION_CLASS FileInformationClass)
// {
//     PDEVICE_OBJECT DeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     KEVENT Event;
//     LARGE_INTEGER Offset;
//     NTSTATUS Status;
//
//     /* get related file object */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//
//     /* get fast i/o table */
//     FastIoDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* is there a fast table */
//     if (FastIoDispatch)
//     {
//         /* check the class */
//         if (FileInformationClass == FileBasicInformation)
//         {
//             /* use FastIoQueryBasicInfo routine */
//             if (FastIoDispatch->FastIoQueryBasicInfo != NULL &&
//                 FastIoDispatch->FastIoQueryBasicInfo(
//                   FileObject,
//                   TRUE,
//                   (PFILE_BASIC_INFORMATION)FileInformation,
//                   &IoStatusBlock,
//                   DeviceObject))
//             {
//                 /* request was handled */
//                 return IoStatusBlock.Status;
//             }
//         }
//         else if (FileInformationClass == FileStandardInformation)
//         {
//             /* use FastIoQueryStandardInfo routine */
//             if (FastIoDispatch->FastIoQueryStandardInfo != NULL &&
//                 FastIoDispatch->FastIoQueryStandardInfo(
//                   FileObject,
//                   TRUE,
//                   (PFILE_STANDARD_INFORMATION)FileInformation,
//                   &IoStatusBlock,
//                   DeviceObject))
//             {
//                 /* request was handled */
//                 return IoStatusBlock.Status;
//             }
//         }
//     }
//
//     /* clear event */
//     KeClearEvent(&FileObject->Event);
//
//     /* initialize event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* set offset to zero */
//     Offset.QuadPart = 0L;
//
//     /* build the request */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_QUERY_INFORMATION, IoGetRelatedDeviceObject(FileObject), NULL, 0, &Offset, &Event, &IoStatusBlock);
//     if (!Irp)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* get next stack location */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     /* setup parameters */
//     IoStack->Parameters.QueryFile.FileInformationClass = FileInformationClass;
//     IoStack->Parameters.QueryFile.Length = Length;
//     Irp->AssociatedIrp.SystemBuffer = FileInformation;
//
//
//     /* call the driver */
//     Status = IoCallDriver(IoGetRelatedDeviceObject(FileObject), Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         /* wait for the operation to complete */
//         KeWaitForSingleObject(&mut Event as *mut _ as _, Executive, KernelMode, FALSE, NULL);
//
//        /* is object sync */
//        if (FileObject->Flags & FO_SYNCHRONOUS_IO)
//            Status = FileObject->FinalStatus;
//        else
//            Status = IoStatusBlock.Status;
//     }
//
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsSetInformationFile(
//     IN  PFILE_OBJECT FileObject,
//     IN  PVOID FileInformation,
//     IN  ULONG Length,
//     IN  FILE_INFORMATION_CLASS FileInformationClass)
// {
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_OBJECT DeviceObject;
//     PIRP Irp;
//     PVOID Buffer;
//     KEVENT Event;
//     LARGE_INTEGER Offset;
//     IO_STATUS_BLOCK IoStatus;
//     NTSTATUS Status;
//
//     /* get related device object */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//
//     /* copy file information */
//     Buffer = AllocateItem(NonPagedPool, Length as usize);
//     if (!Buffer)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     _SEH2_TRY
//     {
//         ProbeForRead(Buffer, Length, sizeof(UCHAR));
//         RtlMoveMemory(Buffer, FileInformation, Length);
//         Status = STATUS_SUCCESS;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Exception, get the error code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* invalid user buffer */
//         FreeItem(Buffer);
//         return Status;
//     }
//
//     /* initialize the event */
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//     /* zero offset */
//     Offset.QuadPart = 0LL;
//
//     /* build the irp */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_SET_INFORMATION, DeviceObject, NULL, 0, &Offset, &Event, &IoStatus);
//     if (!Irp)
//     {
//         /* failed to allocate irp */
//         FreeItem(Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* get next stack location */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     /* set irp parameters */
//     IoStack->Parameters.SetFile.FileInformationClass = FileInformationClass;
//     IoStack->Parameters.SetFile.Length = Length;
//     IoStack->Parameters.SetFile.FileObject = FileObject;
//     Irp->AssociatedIrp.SystemBuffer = Buffer;
//     Irp->UserBuffer = FileInformation;
//
//     /* dispatch the irp */
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         /* wait untill the operation has completed */
//         KeWaitForSingleObject(&mut Event as *mut _ as _, Executive, KernelMode, FALSE, NULL);
//         /* is a sync file object */
//         if (FileObject->Flags & FO_SYNCHRONOUS_IO)
//             Status = FileObject->FinalStatus;
//         else
//             Status = IoStatus.Status;
//     }
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsStreamIo(
//     IN  PFILE_OBJECT FileObject,
//     IN  PKEVENT Event OPTIONAL,
//     IN  PVOID PortContext OPTIONAL,
//     IN  PIO_COMPLETION_ROUTINE CompletionRoutine OPTIONAL,
//     IN  PVOID CompletionContext OPTIONAL,
//     IN  KSCOMPLETION_INVOCATION CompletionInvocationFlags OPTIONAL,
//     OUT PIO_STATUS_BLOCK IoStatusBlock,
//     IN  OUT PVOID StreamHeaders,
//     IN  ULONG Length,
//     IN  ULONG Flags,
//     IN  KPROCESSOR_MODE RequestorMode)
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_OBJECT DeviceObject;
//     NTSTATUS Status;
//     LARGE_INTEGER Offset;
//     PKSIOBJECT_HEADER ObjectHeader;
//     BOOLEAN Ret;
//
//     /* get related device object */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     /* sanity check */
//     ASSERT(DeviceObject != NULL);
//
//     /* is there a event provided */
//     if (Event)
//     {
//         /* reset event */
//         KeClearEvent(Event as _);
//     }
//
//     if (RequestorMode || ExGetPreviousMode() == KernelMode)
//     {
//         /* requestor is from kernel land */
//         ObjectHeader = (PKSIOBJECT_HEADER)FileObject->FsContext2;
//
//         if (ObjectHeader)
//         {
//             /* there is a object header */
//             if (Flags == KSSTREAM_READ)
//             {
//                 /* is fast read supported */
//                 if (ObjectHeader->DispatchTable.FastRead)
//                 {
//                     /* call fast read dispatch routine */
//                     Ret = ObjectHeader->DispatchTable.FastRead(FileObject, NULL, Length, FALSE, 0, StreamHeaders, IoStatusBlock, DeviceObject);
//
//                     if (Ret)
//                     {
//                         /* the request was handled */
//                         return IoStatusBlock->Status;
//                     }
//                 }
//             }
//             else if (Flags == KSSTREAM_WRITE)
//             {
//                 /* is fast write supported */
//                 if (ObjectHeader->DispatchTable.FastWrite)
//                 {
//                     /* call fast write dispatch routine */
//                     Ret = ObjectHeader->DispatchTable.FastWrite(FileObject, NULL, Length, FALSE, 0, StreamHeaders, IoStatusBlock, DeviceObject);
//
//                     if (Ret)
//                     {
//                         /* the request was handled */
//                         return IoStatusBlock->Status;
//                     }
//                 }
//             }
//         }
//     }
//
//     /* clear file object event */
//     KeClearEvent(&FileObject->Event);
//
//     /* set the offset to zero */
//     Offset.QuadPart = 0LL;
//
//     /* now build the irp */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_DEVICE_CONTROL,
//                                        DeviceObject, (PVOID)StreamHeaders, Length, &Offset, Event, IoStatusBlock);
//     if (!Irp)
//     {
//         /* not enough memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* setup irp parameters */
//     Irp->RequestorMode = RequestorMode;
//     Irp->Overlay.AsynchronousParameters.UserApcContext = PortContext;
//     Irp->Tail.Overlay.OriginalFileObject = FileObject;
//     Irp->UserBuffer = StreamHeaders;
//
//     /* get next irp stack location */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     /* setup stack parameters */
//     IoStack->FileObject = FileObject;
//     IoStack->Parameters.DeviceIoControl.OutputBufferLength = Length;
//     IoStack->Parameters.DeviceIoControl.IoControlCode = (Flags == KSSTREAM_READ ? IOCTL_KS_READ_STREAM : IOCTL_KS_WRITE_STREAM);
//
//     if (CompletionRoutine)
//     {
//         /* setup completion routine for async processing */
//         IoSetCompletionRoutine(Irp, CompletionRoutine, CompletionContext, (CompletionInvocationFlags & KsInvokeOnSuccess), (CompletionInvocationFlags & KsInvokeOnError), (CompletionInvocationFlags & KsInvokeOnCancel));
//     }
//
//     /* now call the driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsProbeStreamIrp(
//     IN  PIRP Irp,
//     IN  ULONG ProbeFlags,
//     IN  ULONG HeaderSize)
// {
//     PMDL Mdl;
//     PVOID Buffer;
//     LOCK_OPERATION Operation;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PKSSTREAM_HEADER StreamHeader;
//     PIO_STACK_LOCATION IoStack;
//     ULONG Length;
//     //BOOLEAN AllocateMdl = FALSE;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     Length = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     if (Irp->RequestorMode == KernelMode || Irp->AssociatedIrp.SystemBuffer)
//     {
//         if (Irp->RequestorMode == KernelMode)
//         {
//             /* no need to allocate stream header */
//             Irp->AssociatedIrp.SystemBuffer = Irp->UserBuffer;
//         }
// AllocMdl:
//         /* check if alloc mdl flag is passed */
//         if (!(ProbeFlags & KSPROBE_ALLOCATEMDL))
//         {
//             /* nothing more to do */
//             return STATUS_SUCCESS;
//         }
//         if (Irp->MdlAddress)
//         {
// ProbeMdl:
//             if (ProbeFlags & KSPROBE_PROBEANDLOCK)
//             {
//                 if (Irp->MdlAddress->MdlFlags & (MDL_PAGES_LOCKED | MDL_SOURCE_IS_NONPAGED_POOL))
//                 {
//                     if (ProbeFlags & KSPROBE_SYSTEMADDRESS)
//                     {
//                         _SEH2_TRY
//                         {
//                             /* loop through all mdls and probe them */
//                             Mdl = Irp->MdlAddress;
//                             do
//                             {
//                                 /* the mapping can fail */
//                                 Mdl->MdlFlags |= MDL_MAPPING_CAN_FAIL;
//
//                                 if (Mdl->MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL))
//                                 {
//                                     /* no need to probe these pages */
//                                     Buffer = Mdl->MappedSystemVa;
//                                 }
//                                 else
//                                 {
//                                     /* probe that mdl */
//                                     Buffer = MmMapLockedPages(Mdl, KernelMode);
//                                 }
//
//                                 /* check if the mapping succeeded */
//                                 if (!Buffer)
//                                 {
//                                     /* raise exception we'll catch */
//                                     ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
//                                 }
//
//                                 /* iterate to next mdl */
//                                 Mdl = Mdl->Next;
//
//                             }while(Mdl);
//                         }
//                         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                         {
//                             /* Exception, get the error code */
//                             Status = _SEH2_GetExceptionCode();
//                         } _SEH2_END;
//                     }
//                 }
//                 else
//                 {
//                     _SEH2_TRY
//                     {
//                         /* loop through all mdls and probe them */
//                         Mdl = Irp->MdlAddress;
//
//                         /* determine operation */
//                         if (!(ProbeFlags & KSPROBE_STREAMWRITE) || (ProbeFlags & KSPROBE_MODIFY))
//                         {
//                             /* operation is read / modify stream, need write access */
//                             Operation = IoWriteAccess;
//                         }
//                         else
//                         {
//                             /* operation is write to device, so we need read access */
//                             Operation = IoReadAccess;
//                         }
//
//                         do
//                         {
//                             /* probe the pages */
//                             MmProbeAndLockPages(Mdl, Irp->RequestorMode, Operation);
//
//                             if (ProbeFlags & KSPROBE_SYSTEMADDRESS)
//                             {
//                                 /* the mapping can fail */
//                                 Mdl->MdlFlags |= MDL_MAPPING_CAN_FAIL;
//
//                                 if (Mdl->MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL))
//                                 {
//                                     /* no need to probe these pages */
//                                     Buffer = Mdl->MappedSystemVa;
//                                 }
//                                 else
//                                 {
//                                     /* probe that mdl */
//                                     Buffer = MmMapLockedPages(Mdl, KernelMode);
//                                 }
//
//                                 /* check if the mapping succeeded */
//                                 if (!Buffer)
//                                 {
//                                     /* raise exception we'll catch */
//                                     ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
//                                 }
//                             }
//
//                             /* iterate to next mdl */
//                             Mdl = Mdl->Next;
//
//                         }while(Mdl);
//                     }
//                     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                     {
//                         /* Exception, get the error code */
//                         Status = _SEH2_GetExceptionCode();
//                     } _SEH2_END;
//                 }
//             }
//             return Status;
//         }
//
//         /* check all stream headers */
//         StreamHeader = (PKSSTREAM_HEADER)Irp->AssociatedIrp.SystemBuffer;
//         ASSERT(StreamHeader);
//         _SEH2_TRY
//         {
//             do
//             {
//                 if (HeaderSize)
//                 {
//                     /* does the supplied header size match stream header size and no type changed */
//                     if (StreamHeader->Size != HeaderSize && !(StreamHeader->OptionsFlags & KSSTREAM_HEADER_OPTIONSF_TYPECHANGED))
//                     {
//                         /* invalid stream header */
//                         ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                     }
//                 }
//                 else
//                 {
//                     /* stream must be at least of size KSSTREAM_HEADER and size must be 8-byte block aligned */
//                     if (StreamHeader->Size < sizeof(KSSTREAM_HEADER) || (StreamHeader->Size & 7))
//                     {
//                         /* invalid stream header */
//                         ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                     }
//                 }
//
//                 if (Length < StreamHeader->Size)
//                 {
//                     /* length is too short */
//                     ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                 }
//
//                 if (ProbeFlags & KSPROBE_STREAMWRITE)
//                 {
//                     if (StreamHeader->DataUsed > StreamHeader->FrameExtent)
//                     {
//                         /* frame extend can never be smaller */
//                         ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                     }
//
//                     /* is this stream change packet */
//                     if (StreamHeader->OptionsFlags & KSSTREAM_HEADER_OPTIONSF_TYPECHANGED)
//                     {
//                         if (Length != sizeof(KSSTREAM_HEADER) || (PVOID)StreamHeader != Irp->AssociatedIrp.SystemBuffer)
//                         {
//                             /* stream changed - must be send in a single packet */
//                             ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                         }
//
//                         if (!(ProbeFlags & KSPROBE_ALLOWFORMATCHANGE))
//                         {
//                             /* caller does not permit format changes */
//                             ExRaiseStatus(STATUS_INVALID_PARAMETER);
//                         }
//
//                         if (StreamHeader->FrameExtent)
//                         {
//                             /* allocate an mdl */
//                             Mdl = IoAllocateMdl(StreamHeader->Data, StreamHeader->FrameExtent, FALSE, TRUE, Irp);
//
//                             if (!Mdl)
//                             {
//                                 /* not enough memory */
//                                 ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
//                             }
//
//                             /* break-out to probe for the irp */
//                             break;
//                         }
//                     }
//                 }
//                 else
//                 {
//                     if (StreamHeader->DataUsed)
//                     {
//                         /* DataUsed must be zero for stream read operation */
//                         ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                     }
//
//                     if (StreamHeader->OptionsFlags)
//                     {
//                         /* no flags supported for reading */
//                         ExRaiseStatus(STATUS_INVALID_PARAMETER);
//                     }
//                 }
//
//                 if (StreamHeader->FrameExtent)
//                 {
//                     /* allocate an mdl */
//                     ASSERT(Irp->MdlAddress == NULL);
//                     Mdl = IoAllocateMdl(StreamHeader->Data, StreamHeader->FrameExtent, FALSE, TRUE, Irp);
//                     if (!Mdl)
//                     {
//                         /* not enough memory */
//                         ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
//                     }
//                 }
//
//                 /* move to next stream header */
//                 Length -= StreamHeader->Size;
//                 StreamHeader = (PKSSTREAM_HEADER)((ULONG_PTR)StreamHeader + StreamHeader->Size);
//             }while(Length);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Exception, get the error code */
//             Status = _SEH2_GetExceptionCode();
//         }_SEH2_END;
//
//         /* now probe the allocated mdl's */
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("Status %x\n", Status);
//             return Status;
//         }
//         else
//             goto ProbeMdl;
//     }
//
//     /* probe user mode buffers */
//     if (Length && ( (!HeaderSize) || (Length % HeaderSize == 0) || ((ProbeFlags & KSPROBE_ALLOWFORMATCHANGE) && (Length == sizeof(KSSTREAM_HEADER))) ) )
//     {
//         /* allocate stream header buffer */
//         Irp->AssociatedIrp.SystemBuffer = AllocateItem(NonPagedPool, Length as usize);
//
//         if (!Irp->AssociatedIrp.SystemBuffer)
//         {
//             /* no memory */
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* mark irp as buffered so that changes the stream headers are propagated back */
//         Irp->Flags = IRP_DEALLOCATE_BUFFER | IRP_BUFFERED_IO;
//
//         _SEH2_TRY
//         {
//             if (ProbeFlags & KSPROBE_STREAMWRITE)
//             {
//                 if (ProbeFlags & KSPROBE_MODIFY)
//                     ProbeForWrite(Irp->UserBuffer, Length, sizeof(UCHAR));
//                 else
//                     ProbeForRead(Irp->UserBuffer, Length, sizeof(UCHAR));
//             }
//             else
//             {
//                 /* stream reads means writing */
//                 ProbeForWrite(Irp->UserBuffer, Length, sizeof(UCHAR));
//
//                 /* set input operation flags */
//                 Irp->Flags |= IRP_INPUT_OPERATION;
//             }
//
//             /* copy stream buffer */
//             RtlMoveMemory(Irp->AssociatedIrp.SystemBuffer, Irp->UserBuffer, Length);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Exception, get the error code */
//             Status = _SEH2_GetExceptionCode();
//         }_SEH2_END;
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* failed */
//             return Status;
//         }
//
//         if (ProbeFlags & KSPROBE_ALLOCATEMDL)
//         {
//             /* alloc mdls */
//             goto AllocMdl;
//         }
//
//         /* check all stream headers */
//         StreamHeader = (PKSSTREAM_HEADER)Irp->AssociatedIrp.SystemBuffer;
//
//         _SEH2_TRY
//         {
//             do
//             {
//                 if (HeaderSize)
//                 {
//                     /* does the supplied header size match stream header size and no type changed */
//                     if (StreamHeader->Size != HeaderSize && !(StreamHeader->OptionsFlags & KSSTREAM_HEADER_OPTIONSF_TYPECHANGED))
//                     {
//                         /* invalid stream header */
//                         ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                     }
//                 }
//                 else
//                 {
//                     /* stream must be at least of size KSSTREAM_HEADER and size must be 8-byte block aligned */
//                     if (StreamHeader->Size < sizeof(KSSTREAM_HEADER) || (StreamHeader->Size & 7))
//                     {
//                         /* invalid stream header */
//                         ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                     }
//                 }
//
//                 if (Length < StreamHeader->Size)
//                 {
//                     /* length is too short */
//                     ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                 }
//
//                 if (ProbeFlags & KSPROBE_STREAMWRITE)
//                 {
//                     if (StreamHeader->DataUsed > StreamHeader->FrameExtent)
//                     {
//                         /* frame extend can never be smaller */
//                         ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                     }
//
//                     /* is this stream change packet */
//                     if (StreamHeader->OptionsFlags & KSSTREAM_HEADER_OPTIONSF_TYPECHANGED)
//                     {
//                         if (Length != sizeof(KSSTREAM_HEADER) || (PVOID)StreamHeader != Irp->AssociatedIrp.SystemBuffer)
//                         {
//                             /* stream changed - must be send in a single packet */
//                             ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                         }
//
//                         if (!(ProbeFlags & KSPROBE_ALLOWFORMATCHANGE))
//                         {
//                             /* caller does not permit format changes */
//                             ExRaiseStatus(STATUS_INVALID_PARAMETER);
//                         }
//
//                         if (StreamHeader->FrameExtent)
//                         {
//                             /* allocate an mdl */
//                             Mdl = IoAllocateMdl(StreamHeader->Data, StreamHeader->FrameExtent, FALSE, TRUE, Irp);
//
//                             if (!Mdl)
//                             {
//                                 /* not enough memory */
//                                 ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
//                             }
//
//                             /* break out to probe for the irp */
//                             //AllocateMdl = TRUE;
//                             break;
//                         }
//                     }
//                 }
//                 else
//                 {
//                     if (StreamHeader->DataUsed)
//                     {
//                         /* DataUsed must be zero for stream read operation */
//                         ExRaiseStatus(STATUS_INVALID_BUFFER_SIZE);
//                     }
//
//                     if (StreamHeader->OptionsFlags)
//                     {
//                         /* no flags supported for reading */
//                         ExRaiseStatus(STATUS_INVALID_PARAMETER);
//                     }
//                 }
//
//                 /* move to next stream header */
//                 Length -= StreamHeader->Size;
//                 StreamHeader = (PKSSTREAM_HEADER)((ULONG_PTR)StreamHeader + StreamHeader->Size);
//             }while(Length);
//
//         }_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Exception, get the error code */
//             Status = _SEH2_GetExceptionCode();
//         }_SEH2_END;
//
//         /* now probe the allocated mdl's */
//         if (NT_SUCCESS(Status))
//             goto AllocMdl;
//         else
//             return Status;
//     }
//
//     return STATUS_INVALID_BUFFER_SIZE;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsAllocateExtraData(
//     IN  PIRP Irp,
//     IN  ULONG ExtraSize,
//     OUT PVOID* ExtraBuffer)
// {
//     PIO_STACK_LOCATION IoStack;
//     ULONG Count, Index;
//     PUCHAR Buffer, BufferOrg;
//     PKSSTREAM_HEADER Header;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity check */
//     ASSERT(IoStack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(KSSTREAM_HEADER));
//
//     /* get total length */
//     Count = IoStack->Parameters.DeviceIoControl.InputBufferLength / sizeof(KSSTREAM_HEADER);
//
//     /* allocate buffer */
//     Buffer = BufferOrg = AllocateItem(NonPagedPool, Count * (sizeof(KSSTREAM_HEADER) + ExtraSize));
//     if (!Buffer)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     _SEH2_TRY
//     {
//         /* get input buffer */
//         Header = (PKSSTREAM_HEADER)IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//         for(Index = 0; Index < Count; Index++)
//         {
//             /* copy stream header */
//             RtlMoveMemory(Buffer, Header, sizeof(KSSTREAM_HEADER));
//
//             /* move to next header */
//             Header++;
//             /* increment output buffer offset */
//             Buffer += sizeof(KSSTREAM_HEADER) + ExtraSize;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Exception, get the error code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* free buffer on exception */
//         FreeItem(Buffer);
//         return Status;
//     }
//
//     /* store result */
//     *ExtraBuffer = BufferOrg;
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsNullDriverUnload(
//     IN  PDRIVER_OBJECT DriverObject)
// {
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsDispatchInvalidDeviceRequest(
//     IN  PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     Irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_INVALID_DEVICE_REQUEST;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsDefaultDeviceIoCompletion(
//     IN  PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_PROPERTY &&
//         IoStack->Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_METHOD &&
//         IoStack->Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_ENABLE_EVENT &&
//         IoStack->Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_DISABLE_EVENT)
//     {
//         if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_RESET_STATE)
//         {
//             /* fake success */
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* request unsupported */
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//     else
//     {
//         /* property / method / event not found */
//         Status = STATUS_PROPSET_NOT_FOUND;
//     }
//
//     /* complete request */
//     Irp->IoStatus.Status = Status;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// BOOLEAN
// NTAPI
// KsDispatchFastIoDeviceControlFailure(
//     IN  PFILE_OBJECT FileObject,
//     IN  BOOLEAN Wait,
//     IN  PVOID InputBuffer  OPTIONAL,
//     IN  ULONG InputBufferLength,
//     OUT PVOID OutputBuffer  OPTIONAL,
//     IN  ULONG OutputBufferLength,
//     IN  ULONG IoControlCode,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN  PDEVICE_OBJECT DeviceObject)
// {
//     return FALSE;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// BOOLEAN
// NTAPI
// KsDispatchFastReadFailure(
//     IN  PFILE_OBJECT FileObject,
//     IN  PLARGE_INTEGER FileOffset,
//     IN  ULONG Length,
//     IN  BOOLEAN Wait,
//     IN  ULONG LockKey,
//     OUT PVOID Buffer,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN  PDEVICE_OBJECT DeviceObject)
// {
//     return FALSE;
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsCancelIo(
//     IN  OUT PLIST_ENTRY QueueHead,
//     IN  PKSPIN_LOCK SpinLock)
// {
//     PDRIVER_CANCEL OldDriverCancel;
//     PIO_STACK_LOCATION IoStack;
//     PLIST_ENTRY Entry;
//     PLIST_ENTRY NextEntry;
//     PIRP Irp;
//     KIRQL OldLevel;
//
//     /* acquire spinlock */
//     KeAcquireSpinLock(SpinLock, &mut OldLevel);
//     /* point to first entry */
//     Entry = QueueHead->Flink;
//     /* loop all items */
//     while(Entry != QueueHead)
//     {
//         /* get irp offset */
//         Irp = (PIRP)CONTAINING_RECORD!(Entry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry);
//
//         /* get next entry */
//         NextEntry = Entry->Flink;
//
//         /* set cancelled bit */
//         Irp->Cancel = TRUE;
//
//         /* now set the cancel routine */
//         OldDriverCancel = IoSetCancelRoutine(Irp, NULL);
//         if (OldDriverCancel)
//         {
//             /* this irp hasnt been yet used, so free to cancel */
//             KeReleaseSpinLock(SpinLock, OldLevel);
//
//             /* get current irp stack */
//             IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//             /* acquire cancel spinlock */
//             IoAcquireCancelSpinLock(&Irp->CancelIrql);
//
//             /* call provided cancel routine */
//             OldDriverCancel(IoStack->DeviceObject, Irp);
//
//             /* re-acquire spinlock */
//             KeAcquireSpinLock(SpinLock, &mut OldLevel);
//         }
//
//         /* move on to next entry */
//         Entry = NextEntry;
//     }
//
//     /* the irp has already been canceled */
//     KeReleaseSpinLock(SpinLock, OldLevel);
//
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsReleaseIrpOnCancelableQueue(
//     IN  PIRP Irp,
//     IN  PDRIVER_CANCEL DriverCancel OPTIONAL)
// {
//     PKSPIN_LOCK SpinLock;
//     PDRIVER_CANCEL OldDriverCancel;
//     PIO_STACK_LOCATION IoStack;
//     KIRQL OldLevel;
//
//     /* check for required parameters */
//     if (!Irp)
//         return;
//
//     if (!DriverCancel)
//     {
//         /* default to KsCancelRoutine */
//         DriverCancel = KsCancelRoutine;
//     }
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get internal queue lock */
//     SpinLock = KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp);
//
//     /* acquire spinlock */
//     KeAcquireSpinLock(SpinLock, &mut OldLevel);
//
//     /* now set the cancel routine */
//     OldDriverCancel = IoSetCancelRoutine(Irp, DriverCancel);
//
//     if (Irp->Cancel && OldDriverCancel == NULL)
//     {
//         /* the irp has already been canceled */
//         KeReleaseSpinLock(SpinLock, OldLevel);
//
//         /* cancel routine requires that cancel spinlock is held */
//         IoAcquireCancelSpinLock(&Irp->CancelIrql);
//
//         /* cancel irp */
//         DriverCancel(IoStack->DeviceObject, Irp);
//     }
//     else
//     {
//         /* done */
//         KeReleaseSpinLock(SpinLock, OldLevel);
//     }
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PIRP
// NTAPI
// KsRemoveIrpFromCancelableQueue(
//     IN  OUT PLIST_ENTRY QueueHead,
//     IN  PKSPIN_LOCK SpinLock,
//     IN  KSLIST_ENTRY_LOCATION ListLocation,
//     IN  KSIRP_REMOVAL_OPERATION RemovalOperation)
// {
//     PIRP Irp;
//     PLIST_ENTRY CurEntry;
//     KIRQL OldIrql;
//
//     DPRINT("KsRemoveIrpFromCancelableQueue ListHead %p SpinLock %p ListLocation %x RemovalOperation %x\n", QueueHead, SpinLock, ListLocation, RemovalOperation);
//
//     /* check parameters */
//     if (!QueueHead || !SpinLock)
//         return NULL;
//
//     /* check if parameter ListLocation is valid */
//     if (ListLocation != KsListEntryTail && ListLocation != KsListEntryHead)
//         return NULL;
//
//     /* acquire list lock */
//     KeAcquireSpinLock(SpinLock, &OldIrql);
//
//     /* point to queue head */
//     CurEntry = QueueHead;
//
//     do
//     {
//         /* reset irp to null */
//         Irp = NULL;
//
//         /* iterate to next entry */
//         if (ListLocation == KsListEntryHead)
//             CurEntry = CurEntry->Flink;
//         else
//             CurEntry = CurEntry->Blink;
//
//         /* is the end of list reached */
//         if (CurEntry == QueueHead)
//         {
//             /* reached end of list */
//             break;
//         }
//
//         /* get irp offset */
//         Irp = (PIRP)CONTAINING_RECORD!(CurEntry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry);
//
//         if (Irp->Cancel)
//         {
//             /* irp has been canceled */
//             break;
//         }
//
//         if (Irp->CancelRoutine)
//         {
//             /* remove cancel routine */
//             Irp->CancelRoutine = NULL;
//
//             if (RemovalOperation == KsAcquireAndRemove || RemovalOperation == KsAcquireAndRemoveOnlySingleItem)
//             {
//                 /* remove irp from list */
//                 RemoveEntryList(&Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//             }
//
//             if (RemovalOperation == KsAcquireAndRemoveOnlySingleItem || RemovalOperation == KsAcquireOnlySingleItem)
//                 break;
//         }
//
//     }while(TRUE);
//
//     /* release lock */
//     KeReleaseSpinLock(SpinLock, OldIrql);
//
//     if (!Irp || Irp->CancelRoutine == NULL)
//     {
//         /* either an irp has been acquired or nothing found */
//         return Irp;
//     }
//
//     /* time to remove the canceled irp */
//     IoAcquireCancelSpinLock(&OldIrql);
//     /* acquire list lock */
//     KeAcquireSpinLockAtDpcLevel(SpinLock);
//
//     if (RemovalOperation == KsAcquireAndRemove || RemovalOperation == KsAcquireAndRemoveOnlySingleItem)
//     {
//         /* remove it */
//         RemoveEntryList(&Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//     }
//
//     /* release list lock */
//     KeReleaseSpinLockFromDpcLevel(SpinLock);
//
//     /* release cancel spinlock */
//     IoReleaseCancelSpinLock(OldIrql);
//     /* no non canceled irp has been found */
//     return NULL;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsMoveIrpsOnCancelableQueue(
//     IN  OUT PLIST_ENTRY SourceList,
//     IN  PKSPIN_LOCK SourceLock,
//     IN  OUT PLIST_ENTRY DestinationList,
//     IN  PKSPIN_LOCK DestinationLock OPTIONAL,
//     IN  KSLIST_ENTRY_LOCATION ListLocation,
//     IN  PFNKSIRPLISTCALLBACK ListCallback,
//     IN  PVOID Context)
// {
//     KIRQL OldLevel;
//     PLIST_ENTRY SrcEntry;
//     PIRP Irp;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     if (!DestinationLock)
//     {
//         /* no destination lock just acquire the source lock */
//         KeAcquireSpinLock(SourceLock, &mut OldLevel);
//     }
//     else
//     {
//         /* acquire cancel spinlock */
//         IoAcquireCancelSpinLock(&OldLevel);
//
//         /* now acquire source lock */
//         KeAcquireSpinLockAtDpcLevel(SourceLock);
//
//         /* now acquire destination lock */
//         KeAcquireSpinLockAtDpcLevel(DestinationLock);
//     }
//
//     /* point to list head */
//     SrcEntry = SourceList;
//
//     /* now move all irps */
//     while(TRUE)
//     {
//         if (ListLocation == KsListEntryTail)
//         {
//             /* move queue downwards */
//             SrcEntry = SrcEntry->Flink;
//         }
//         else
//         {
//             /* move queue upwards */
//             SrcEntry = SrcEntry->Blink;
//         }
//
//         if (SrcEntry == SourceList)
//         {
//             /* eof list reached */
//             break;
//         }
//
//         /* get irp offset */
//         Irp = (PIRP)CONTAINING_RECORD!(SrcEntry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry);
//
//         /* now check if irp can be moved */
//         Status = ListCallback.unwrap()(Irp, Context);
//
//         /* check if irp can be moved */
//         if (Status == STATUS_SUCCESS)
//         {
//             /* remove irp from src list */
//             RemoveEntryList(&Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//
//             if (ListLocation == KsListEntryTail)
//             {
//                 /* insert irp end of list */
//                 InsertTailList(DestinationList, &Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//             }
//             else
//             {
//                 /* insert irp head of list */
//                 InsertHeadList(DestinationList, &Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//             }
//
//             /* do we need to update the irp lock */
//             if (DestinationLock)
//             {
//                 /* update irp lock */
//                 KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp) = DestinationLock;
//             }
//         }
//         else
//         {
//             if (Status != STATUS_NO_MATCH)
//             {
//                 /* callback decided to stop enumeration */
//                 break;
//             }
//
//             /* reset return value */
//             Status = STATUS_SUCCESS;
//         }
//     }
//
//     if (!DestinationLock)
//     {
//         /* release source lock */
//         KeReleaseSpinLock(SourceLock, OldLevel);
//     }
//     else
//     {
//         /* now release destination lock */
//         KeReleaseSpinLockFromDpcLevel(DestinationLock);
//
//         /* now release source lock */
//         KeReleaseSpinLockFromDpcLevel(SourceLock);
//
//
//         /* now release cancel spinlock */
//         IoReleaseCancelSpinLock(OldLevel);
//     }
//
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsRemoveSpecificIrpFromCancelableQueue(
//     IN  PIRP Irp)
// {
//     PKSPIN_LOCK SpinLock;
//     KIRQL OldLevel;
//
//     DPRINT("KsRemoveSpecificIrpFromCancelableQueue %p\n", Irp);
//
//     /* get internal queue lock */
//     SpinLock = KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp);
//
//     /* acquire spinlock */
//     KeAcquireSpinLock(SpinLock, &mut OldLevel);
//
//     /* remove the irp from the list */
//     RemoveEntryList(&Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//
//     /* release spinlock */
//     KeReleaseSpinLock(SpinLock, OldLevel);
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsAddIrpToCancelableQueue(
//     IN  OUT PLIST_ENTRY QueueHead,
//     IN  PKSPIN_LOCK SpinLock,
//     IN  PIRP Irp,
//     IN  KSLIST_ENTRY_LOCATION ListLocation,
//     IN  PDRIVER_CANCEL DriverCancel OPTIONAL)
// {
//     PDRIVER_CANCEL OldDriverCancel;
//     PIO_STACK_LOCATION IoStack;
//     KIRQL OldLevel;
//
//     /* check for required parameters */
//     if (!QueueHead || !SpinLock || !Irp)
//         return;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("KsAddIrpToCancelableQueue QueueHead %p SpinLock %p Irp %p ListLocation %x DriverCancel %p\n", QueueHead, SpinLock, Irp, ListLocation, DriverCancel);
//
//     // HACK for ms portcls
//     if (IoStack->MajorFunction == IRP_MJ_CREATE)
//     {
//         // complete the request
//         DPRINT1("MS HACK\n");
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//
//         return;
//     }
//
//
//     if (!DriverCancel)
//     {
//         /* default to KsCancelRoutine */
//         DriverCancel = KsCancelRoutine;
//     }
//
//
//     /* acquire spinlock */
//     KeAcquireSpinLock(SpinLock, &mut OldLevel);
//
//     if (ListLocation == KsListEntryTail)
//     {
//         /* insert irp to tail of list */
//         InsertTailList(QueueHead, &Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//     }
//     else
//     {
//         /* insert irp to head of list */
//         InsertHeadList(QueueHead, &Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//     }
//
//     /* store internal queue lock */
//     KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp) = SpinLock;
//
//     /* now set the cancel routine */
//     OldDriverCancel = IoSetCancelRoutine(Irp, DriverCancel);
//
//     if (Irp->Cancel && OldDriverCancel == NULL)
//     {
//         /* the irp has already been canceled */
//         KeReleaseSpinLock(SpinLock, OldLevel);
//
//         /* cancel routine requires that cancel spinlock is held */
//         IoAcquireCancelSpinLock(&Irp->CancelIrql);
//
//         /* cancel irp */
//         DriverCancel(IoStack->DeviceObject, Irp);
//     }
//     else
//     {
//         /* done */
//         KeReleaseSpinLock(SpinLock, OldLevel);
//     }
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsCancelRoutine(
//     IN  PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     PKSPIN_LOCK SpinLock;
//
//     /* get internal queue lock */
//     SpinLock = KSQUEUE_SPINLOCK_IRP_STORAGE!(Irp);
//
//     /* acquire spinlock */
//     KeAcquireSpinLockAtDpcLevel(SpinLock);
//
//     /* sanity check */
//     ASSERT(KeGetCurrentIrql() == DISPATCH_LEVEL);
//
//     /* release cancel spinlock */
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     /* remove the irp from the list */
//     RemoveEntryList(&Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//
//     /* release spinlock */
//     KeReleaseSpinLock(SpinLock, Irp->CancelIrql);
//
//     /* has the irp already been canceled */
//     if (Irp->IoStatus.Status != STATUS_CANCELLED)
//     {
//         /* let's complete it */
//         Irp->IoStatus.Status = STATUS_CANCELLED;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//     }
// }
//
// NTSTATUS
// FindMatchingCreateItem(
//     PLIST_ENTRY ListHead,
//     PUNICODE_STRING String,
//     OUT PCREATE_ITEM_ENTRY *OutCreateItem)
// {
//     PLIST_ENTRY Entry;
//     PCREATE_ITEM_ENTRY CreateItemEntry;
//     UNICODE_STRING RefString;
//     LPWSTR pStr;
//     ULONG Count;
//
//     /* Copy the input string */
//     RefString = *String;
//
//     /* Check if the string starts with a backslash */
//     if (String->Buffer[0] == L'\\')
//     {
//         /* Skip backslash */
//         RefString.Buffer++;
//         RefString.Length -= sizeof(WCHAR);
//     }
//     else
//     {
//         /* get terminator */
//         pStr = String->Buffer;
//         Count = String->Length / sizeof(WCHAR);
//         while ((Count > 0) && (*pStr != L'\\'))
//         {
//             pStr++;
//             Count--;
//         }
//
//         /* sanity check */
//         ASSERT(Count != 0);
//
//         // request is for pin / node / allocator
//         RefString.Length = (USHORT)((PCHAR)pStr - (PCHAR)String->Buffer);
//     }
//
//     /* point to first entry */
//     Entry = ListHead->Flink;
//
//     /* loop all device items */
//     while (Entry != ListHead)
//     {
//         /* get create item entry */
//         CreateItemEntry = (PCREATE_ITEM_ENTRY)CONTAINING_RECORD!(Entry,
//                                                                 CREATE_ITEM_ENTRY,
//                                                                 Entry);
//
//         ASSERT(CreateItemEntry->CreateItem);
//
//         if(CreateItemEntry->CreateItem->Flags & KSCREATE_ITEM_WILDCARD)
//         {
//             /* create item is default */
//             *OutCreateItem = CreateItemEntry;
//             return STATUS_SUCCESS;
//         }
//
//         if (!CreateItemEntry->CreateItem->Create)
//         {
//             /* skip free create item */
//             Entry = Entry->Flink;
//             continue;
//         }
//
//         DPRINT("CreateItem %S Length %u Request %wZ %u\n",
//                CreateItemEntry->CreateItem->ObjectClass.Buffer,
//                CreateItemEntry->CreateItem->ObjectClass.Length,
//                &RefString,
//                RefString.Length);
//
//         if (CreateItemEntry->CreateItem->ObjectClass.Length > RefString.Length)
//         {
//             /* create item doesnt match in length */
//             Entry = Entry->Flink;
//             continue;
//         }
//
//          /* now check if the object class is the same */
//         if (!RtlCompareUnicodeString(&CreateItemEntry->CreateItem->ObjectClass,
//                                      &RefString,
//                                      TRUE))
//         {
//             /* found matching create item */
//             *OutCreateItem = CreateItemEntry;
//             return STATUS_SUCCESS;
//         }
//         /* iterate to next */
//         Entry = Entry->Flink;
//     }
//
//     return STATUS_NOT_FOUND;
// }
//
// NTSTATUS
// NTAPI
// KspCreate(
//     IN  PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     PCREATE_ITEM_ENTRY CreateItemEntry;
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_EXTENSION DeviceExtension;
//     PKSIDEVICE_HEADER DeviceHeader;
//     PKSIOBJECT_HEADER ObjectHeader;
//     NTSTATUS Status;
//
//     DPRINT("KS / CREATE\n");
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     /* get device header */
//     DeviceHeader = DeviceExtension->DeviceHeader;
//
//
//     if (IoStack->FileObject->FileName.Buffer == NULL)
//     {
//         /* FIXME Pnp-Issue */
//         DPRINT("Using reference string hack\n");
//         Irp->IoStatus.Information = 0;
//         /* set return status */
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_SUCCESS;
//     }
//
//     if (IoStack->FileObject->RelatedFileObject != NULL)
//     {
//         /* request is to instantiate a pin / node / clock / allocator */
//         ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->RelatedFileObject->FsContext2;
//
//         /* sanity check */
//         ASSERT(ObjectHeader);
//
//         /* find a matching a create item */
//         Status = FindMatchingCreateItem(&ObjectHeader->ItemList,
//                                         &IoStack->FileObject->FileName,
//                                         &CreateItemEntry);
//     }
//     else
//     {
//         /* request to create a filter */
//         Status = FindMatchingCreateItem(&DeviceHeader->ItemList,
//                                         &IoStack->FileObject->FileName,
//                                         &CreateItemEntry);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         /* set object create item */
//         KSCREATE_ITEM_IRP_STORAGE!(Irp) = CreateItemEntry->CreateItem;
//
//         /* call create function */
//         Status = CreateItemEntry->CreateItem->Create(DeviceObject, Irp);
//
//         if (NT_SUCCESS(Status))
//         {
//             /* increment create item reference count */
//             InterlockedIncrement(&CreateItemEntry->ReferenceCount);
//         }
//         return Status;
//     }
//
//     Irp->IoStatus.Information = 0;
//     /* set return status */
//     Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_UNSUCCESSFUL;
// }
//
// NTSTATUS
// NTAPI
// KspDispatchIrp(
//     IN  PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     //PDEVICE_EXTENSION DeviceExtension;
//     PKSIOBJECT_HEADER ObjectHeader;
//     //PKSIDEVICE_HEADER DeviceHeader;
//     PDRIVER_DISPATCH Dispatch;
//     NTSTATUS Status;
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get device extension */
//     //DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     /* get device header */
//     //DeviceHeader = DeviceExtension->DeviceHeader;
//
//     ASSERT(IoStack->FileObject);
//
//     /* get object header */
//     ObjectHeader = (PKSIOBJECT_HEADER) IoStack->FileObject->FsContext2;
//
//     if (!ObjectHeader)
//     {
//         /* FIXME Pnp-Issue*/
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         Irp->IoStatus.Information = 0;
//         /* complete and forget */
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_SUCCESS;
//     }
//
//     /* sanity check */
//     ASSERT(ObjectHeader);
//     /* store create item */
//     //KSCREATE_ITEM_IRP_STORAGE!(Irp) = (PKSOBJECT_CREATE_ITEM)0x12345678; //ObjectHeader->CreateItem;
//
//     /* retrieve matching dispatch function */
//     switch(IoStack->MajorFunction)
//     {
//         case IRP_MJ_CLOSE:
//             Dispatch = ObjectHeader->DispatchTable.Close;
//             break;
//         case IRP_MJ_DEVICE_CONTROL:
//             Dispatch = ObjectHeader->DispatchTable.DeviceIoControl;
//             break;
//         case IRP_MJ_READ:
//             Dispatch = ObjectHeader->DispatchTable.Read;
//             break;
//         case IRP_MJ_WRITE:
//             Dispatch = ObjectHeader->DispatchTable.Write;
//             break;
//         case IRP_MJ_FLUSH_BUFFERS :
//             Dispatch = ObjectHeader->DispatchTable.Flush;
//             break;
//         case IRP_MJ_QUERY_SECURITY:
//             Dispatch = ObjectHeader->DispatchTable.QuerySecurity;
//             break;
//         case IRP_MJ_SET_SECURITY:
//             Dispatch = ObjectHeader->DispatchTable.SetSecurity;
//             break;
//         case IRP_MJ_PNP:
//             Dispatch = KsDefaultDispatchPnp;
//             break;
//         default:
//             Dispatch = NULL;
//     }
//
//     /* is the request supported */
//     if (Dispatch)
//     {
//         /* now call the dispatch function */
//         Status = Dispatch(DeviceObject, Irp);
//     }
//     else
//     {
//         /* not supported request */
//         Status = KsDispatchInvalidDeviceRequest(DeviceObject, Irp);
//     }
//
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsSetMajorFunctionHandler(
//     IN  PDRIVER_OBJECT DriverObject,
//     IN  ULONG MajorFunction)
// {
//     DPRINT("KsSetMajorFunctionHandler Function %x\n", MajorFunction);
//
//     switch ( MajorFunction )
//     {
//         case IRP_MJ_CREATE:
//             DriverObject->MajorFunction[MajorFunction] = KspCreate;
//             break;
//         case IRP_MJ_DEVICE_CONTROL:
//         case IRP_MJ_CLOSE:
//         case IRP_MJ_READ:
//         case IRP_MJ_WRITE:
//         case IRP_MJ_FLUSH_BUFFERS :
//         case IRP_MJ_QUERY_SECURITY:
//         case IRP_MJ_SET_SECURITY:
//             DriverObject->MajorFunction[MajorFunction] = KspDispatchIrp;
//             break;
//         default:
//             DPRINT1("NotSupported %x\n", MajorFunction);
//             return STATUS_INVALID_PARAMETER;
//     };
//
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsDispatchIrp(
//     IN  PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIDEVICE_HEADER DeviceHeader;
//     PDEVICE_EXTENSION DeviceExtension;
//
//     DPRINT("KsDispatchIrp DeviceObject %p Irp %p\n", DeviceObject, Irp);
//
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* get device header */
//     DeviceHeader = DeviceExtension->DeviceHeader;
//
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->MajorFunction <= IRP_MJ_DEVICE_CONTROL)
//     {
//         if (IoStack->MajorFunction == IRP_MJ_CREATE)
//         {
//             /* check internal type */
//             if (DeviceHeader->BasicHeader.OuterUnknown) /* FIXME improve check */
//             {
//                 /* AVStream client */
//                 return IKsDevice_Create(DeviceObject, Irp);
//             }
//             else
//             {
//                 /* external client (portcls) */
//                 return KspCreate(DeviceObject, Irp);
//             }
//         }
//
//         switch (IoStack->MajorFunction)
//         {
//             case IRP_MJ_CLOSE:
//             case IRP_MJ_READ:
//             case IRP_MJ_WRITE:
//             case IRP_MJ_FLUSH_BUFFERS:
//             case IRP_MJ_QUERY_SECURITY:
//             case IRP_MJ_SET_SECURITY:
//             case IRP_MJ_PNP:
//             case IRP_MJ_DEVICE_CONTROL:
//                 return KspDispatchIrp(DeviceObject, Irp);
//             default:
//                 return KsDispatchInvalidDeviceRequest(DeviceObject, Irp);
//         }
//     }
//
//     /* dispatch power */
//     if (IoStack->MajorFunction == IRP_MJ_POWER)
//     {
//         /* check internal type */
//         if (DeviceHeader->BasicHeader.OuterUnknown) /* FIXME improve check */
//         {
//             /* AVStream client */
//             return IKsDevice_Power(DeviceObject, Irp);
//         }
//         else
//         {
//             /* external client (portcls) */
//             return KsDefaultDispatchPower(DeviceObject, Irp);
//         }
//     }
//     else if (IoStack->MajorFunction == IRP_MJ_PNP) /* dispatch pnp */
//     {
//         /* check internal type */
//         if (DeviceHeader->BasicHeader.OuterUnknown) /* FIXME improve check */
//         {
//             /* AVStream client */
//             return IKsDevice_Pnp(DeviceObject, Irp);
//         }
//         else
//         {
//             /* external client (portcls) */
//             return KsDefaultDispatchPnp(DeviceObject, Irp);
//         }
//     }
//     else if (IoStack->MajorFunction == IRP_MJ_SYSTEM_CONTROL)
//     {
//         /* forward irp */
//         return KsDefaultForwardIrp(DeviceObject, Irp);
//     }
//     else
//     {
//         /* not supported */
//         return KsDispatchInvalidDeviceRequest(DeviceObject, Irp);
//     }
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// ULONG
// NTAPI
// KsGetNodeIdFromIrp(
//     IN PIRP Irp)
// {
//     // UNIMPLEMENTED
//     return KSFILTER_NODE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
