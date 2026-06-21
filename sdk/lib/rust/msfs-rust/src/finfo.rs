//! finfo.rs — finfo.c: query/set mailslot information. Only valid on the server side
//! (Fcb->ServerCcb == Ccb). MAILSLOT_NO_MESSAGE == MAXULONG.
#![allow(non_snake_case)]

use core::mem::size_of;
use core::ptr::addr_of_mut;

use ntkrnl_sys::base::{ULONG, ULONG_PTR};
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, FILE_MAILSLOT_QUERY_INFORMATION,
    FILE_MAILSLOT_SET_INFORMATION, KIRQL, NTSTATUS, PFILE_MAILSLOT_QUERY_INFORMATION,
    PFILE_MAILSLOT_SET_INFORMATION,
};
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::glue::*;
use crate::types::*;
use crate::CONTAINING_RECORD;

const MAILSLOT_NO_MESSAGE: ULONG = MAXULONG;

unsafe fn MsfsQueryMailslotInformation(
    Fcb: PMSFS_FCB,
    Buffer: PFILE_MAILSLOT_QUERY_INFORMATION,
    BufferLength: *mut ULONG,
) -> NTSTATUS {
    if *BufferLength < size_of::<FILE_MAILSLOT_QUERY_INFORMATION>() as ULONG {
        return STATUS_BUFFER_OVERFLOW;
    }

    (*Buffer).MaximumMessageSize = (*Fcb).MaxMessageSize;
    (*Buffer).ReadTimeout = (*Fcb).TimeOut;

    let mut oldIrql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, addr_of_mut!(oldIrql));
    (*Buffer).MessagesAvailable = (*Fcb).MessageCount;
    if (*Fcb).MessageCount == 0 {
        (*Buffer).NextMessageSize = MAILSLOT_NO_MESSAGE;
    } else {
        let Message =
            CONTAINING_RECORD!((*Fcb).MessageListHead.Flink, MSFS_MESSAGE, MessageListEntry);
        (*Buffer).NextMessageSize = (*Message).Size;
    }
    KeReleaseSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, oldIrql);

    *BufferLength -= size_of::<FILE_MAILSLOT_QUERY_INFORMATION>() as ULONG;

    STATUS_SUCCESS
}

unsafe fn MsfsSetMailslotInformation(
    Fcb: PMSFS_FCB,
    Buffer: PFILE_MAILSLOT_SET_INFORMATION,
    BufferLength: *mut ULONG,
) -> NTSTATUS {
    if *BufferLength < size_of::<FILE_MAILSLOT_SET_INFORMATION>() as ULONG {
        return STATUS_BUFFER_OVERFLOW;
    }

    (*Fcb).TimeOut = *(*Buffer).ReadTimeout;

    STATUS_SUCCESS
}

pub unsafe extern "system" fn MsfsQueryInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject;
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let FileInformationClass = (*IoStack).Parameters.QueryFile.FileInformationClass;
    let FileObject = (*IoStack).FileObject;
    let Fcb = (*FileObject).FsContext as PMSFS_FCB;
    let Ccb = (*FileObject).FsContext2 as PMSFS_CCB;

    /* querying information is not permitted on client side */
    if (*Fcb).ServerCcb != Ccb {
        let Status = STATUS_ACCESS_DENIED;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    let SystemBuffer = (*Irp).AssociatedIrp.SystemBuffer;
    let mut BufferLength = (*IoStack).Parameters.QueryFile.Length;

    let Status = if FileInformationClass == FileMailslotQueryInformation {
        MsfsQueryMailslotInformation(Fcb, SystemBuffer as _, addr_of_mut!(BufferLength))
    } else {
        STATUS_NOT_IMPLEMENTED
    };

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    if NT_SUCCESS(Status) {
        (*Irp).IoStatus.Information =
            ((*IoStack).Parameters.QueryFile.Length - BufferLength) as _;
    } else {
        (*Irp).IoStatus.Information = 0;
    }
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    Status
}

pub unsafe extern "system" fn MsfsSetInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject;
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let FileInformationClass = (*IoStack).Parameters.QueryFile.FileInformationClass;
    let FileObject = (*IoStack).FileObject;
    let Fcb = (*FileObject).FsContext as PMSFS_FCB;
    let Ccb = (*FileObject).FsContext2 as PMSFS_CCB;

    /* setting information is not permitted on client side */
    if (*Fcb).ServerCcb != Ccb {
        let Status = STATUS_ACCESS_DENIED;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    let SystemBuffer = (*Irp).AssociatedIrp.SystemBuffer;
    let mut BufferLength = (*IoStack).Parameters.QueryFile.Length;

    let Status = if FileInformationClass == FileMailslotSetInformation {
        MsfsSetMailslotInformation(Fcb, SystemBuffer as _, addr_of_mut!(BufferLength))
    } else {
        STATUS_NOT_IMPLEMENTED
    };

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    Status
}
