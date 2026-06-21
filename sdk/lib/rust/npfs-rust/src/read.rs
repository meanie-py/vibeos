//! read.rs — read.c: Pipes Reading (NpCommonRead, NpFsdRead, NpFastRead).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{
    BOOLEAN, InitializeListHead, LONG, NTSTATUS, NT_SUCCESS, PVOID, SIZE_T, UCHAR, ULONG,
    ULONG_PTR, USHORT,
};
use ntkrnl_sys::ddk::*;

use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use ntkrnl_sys::ke::KeSetEvent;

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// cross-module fns
use crate::datasup::NpAddDataQueueEntry;
use crate::fileobsup::NpDecodeFileObject;
use crate::readsup::NpReadDataQueue;

// File ID number for NPFS bugchecking support: NPFS_BUGCHECK_READ (glue.rs)

// ntddk.h: IO_NO_INCREMENT — CCHAR PriorityBoost passed to KeSetEvent here.
const IO_NO_INCREMENT: LONG = 0; // (KeSetEvent's Increment arg is KPRIORITY = LONG)

/* GLOBALS ********************************************************************/

pub static mut NpSlowReadCalls: LONG = 0;
pub static mut NpFastReadTrue: ULONG = 0;
pub static mut NpFastReadFalse: ULONG = 0;

/* FUNCTIONS ******************************************************************/

pub unsafe extern "system" fn NpCommonRead(
    FileObject: PFILE_OBJECT,
    Buffer: PVOID,
    BufferSize: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> BOOLEAN {
    let NodeType: NODE_TYPE_CODE;
    let ReadQueue: PNP_DATA_QUEUE;
    let EventBuffer: PNP_EVENT_BUFFER;
    let Status: NTSTATUS;
    let mut NamedPipeEnd: ULONG = 0;
    let mut Ccb: PNP_CCB = null_mut();
    let mut ReadOk: BOOLEAN;
    let NonPagedCcb: PNP_NONPAGED_CCB;
    // PAGED_CODE();

    (*IoStatus).Information = 0;
    NodeType = NpDecodeFileObject(FileObject, null_mut(), &mut Ccb, &mut NamedPipeEnd);

    if NodeType == 0 {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_PIPE_DISCONNECTED;
        return TRUE;
    }

    if NodeType != NPFS_NTC_CCB {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return TRUE;
    }

    NonPagedCcb = (*Ccb).NonPagedCcb;
    ExAcquireResourceExclusiveLite(addr_of_mut!((*NonPagedCcb).Lock), TRUE);

    if (*Ccb).NamedPipeState == FILE_PIPE_DISCONNECTED_STATE as UCHAR
        || (*Ccb).NamedPipeState == FILE_PIPE_LISTENING_STATE as UCHAR
    {
        (*IoStatus).__bindgen_anon_1.Status =
            if (*Ccb).NamedPipeState != FILE_PIPE_DISCONNECTED_STATE as UCHAR {
                STATUS_PIPE_LISTENING
            } else {
                STATUS_PIPE_DISCONNECTED
            };
        ReadOk = TRUE;
        // goto Quickie;
        ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        return ReadOk;
    }

    // ASSERT((Ccb->NamedPipeState == FILE_PIPE_CONNECTED_STATE) || (Ccb->NamedPipeState == FILE_PIPE_CLOSING_STATE));

    if (NamedPipeEnd == FILE_PIPE_SERVER_END
        && (*(*Ccb).Fcb).NamedPipeConfiguration == FILE_PIPE_OUTBOUND as USHORT)
        || (NamedPipeEnd == FILE_PIPE_CLIENT_END
            && (*(*Ccb).Fcb).NamedPipeConfiguration == FILE_PIPE_INBOUND as USHORT)
    {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        ReadOk = TRUE;
        // goto Quickie;
        ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        return ReadOk;
    }

    if NamedPipeEnd == FILE_PIPE_SERVER_END {
        ReadQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
    } else {
        ReadQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
    }

    EventBuffer = (*NonPagedCcb).EventBuffer[NamedPipeEnd as usize];

    if (*ReadQueue).QueueState == WriteEntries {
        *IoStatus = NpReadDataQueue(
            ReadQueue,
            FALSE,
            FALSE,
            Buffer,
            BufferSize,
            (*Ccb).ReadMode[NamedPipeEnd as usize] as ULONG,
            Ccb,
            List,
        );
        if !NT_SUCCESS((*IoStatus).__bindgen_anon_1.Status) {
            ReadOk = TRUE;
            // goto Quickie;
            ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
            return ReadOk;
        }

        ReadOk = TRUE;
        if !EventBuffer.is_null() {
            KeSetEvent((*EventBuffer).Event as PVOID, IO_NO_INCREMENT, FALSE);
        }
        // goto Quickie;
        ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        return ReadOk;
    }

    if (*Ccb).NamedPipeState == FILE_PIPE_CLOSING_STATE as UCHAR {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_PIPE_BROKEN;
        ReadOk = TRUE;
        if !EventBuffer.is_null() {
            KeSetEvent((*EventBuffer).Event as PVOID, IO_NO_INCREMENT, FALSE);
        }
        // goto Quickie;
        ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        return ReadOk;
    }

    if (*Ccb).CompletionMode[NamedPipeEnd as usize] == FILE_PIPE_COMPLETE_OPERATION as UCHAR {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_PIPE_EMPTY;
        ReadOk = TRUE;
        if !EventBuffer.is_null() {
            KeSetEvent((*EventBuffer).Event as PVOID, IO_NO_INCREMENT, FALSE);
        }
        // goto Quickie;
        ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        return ReadOk;
    }

    if Irp.is_null() {
        ReadOk = FALSE;
        // goto Quickie;
        ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        return ReadOk;
    }

    Status = NpAddDataQueueEntry(
        NamedPipeEnd,
        Ccb,
        ReadQueue,
        ReadEntries,
        Buffered,
        BufferSize,
        Irp,
        null_mut(),
        0,
    );
    (*IoStatus).__bindgen_anon_1.Status = Status;
    if !NT_SUCCESS(Status) {
        ReadOk = FALSE;
    } else {
        ReadOk = TRUE;
        if !EventBuffer.is_null() {
            KeSetEvent((*EventBuffer).Event as PVOID, IO_NO_INCREMENT, FALSE);
        }
    }

    // Quickie:
    ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
    ReadOk
}

pub unsafe extern "system" fn NpFsdRead(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    // PAGED_CODE();
    NpSlowReadCalls += 1;

    InitializeListHead(addr_of_mut!(DeferredList));
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    FsRtlEnterFileSystem();
    NpAcquireSharedVcb();

    NpCommonRead(
        (*IoStack).FileObject,
        (*Irp).UserBuffer,
        (*IoStack).Parameters.Read.Length,
        addr_of_mut!(IoStatus),
        Irp,
        addr_of_mut!(DeferredList),
    );

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));
    FsRtlExitFileSystem();

    if IoStatus.__bindgen_anon_1.Status != STATUS_PENDING {
        (*Irp).IoStatus.Information = IoStatus.Information;
        (*Irp).IoStatus.__bindgen_anon_1.Status = IoStatus.__bindgen_anon_1.Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    IoStatus.__bindgen_anon_1.Status
}

// _Function_class_(FAST_IO_READ) _IRQL_requires_same_
pub unsafe extern "system" fn NpFastRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    Buffer: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    let Result: BOOLEAN;
    // PAGED_CODE();

    InitializeListHead(addr_of_mut!(DeferredList));

    FsRtlEnterFileSystem();
    NpAcquireSharedVcb();

    Result = NpCommonRead(
        FileObject,
        Buffer,
        Length,
        IoStatus,
        null_mut(),
        addr_of_mut!(DeferredList),
    );
    if Result != 0 {
        NpFastReadTrue += 1;
    } else {
        NpFastReadFalse += 1;
    }

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));
    FsRtlExitFileSystem();

    Result
}

/* EOF */
