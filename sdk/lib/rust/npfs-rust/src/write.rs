//! write.rs — write.c: Pipes Writing (NpCommonWrite / NpFsdWrite / NpFastWrite).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};
use ntkrnl_sys::base::{
    ULONG, USHORT, UCHAR, BOOLEAN, NTSTATUS, NT_SUCCESS, PVOID, SIZE_T, ULONG_PTR, LONG,
    InitializeListHead,
};
use ntkrnl_sys::ddk::*; // bindgen structs/extern fns: DEVICE_OBJECT, IRP, FILE_OBJECT, Io*, Ke*, Ex*, ...
use ntkrnl_sys::ke::KeSetEvent;
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem, PsGetCurrentThread};
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// cross-module fns
use crate::fileobsup::NpDecodeFileObject;
use crate::datasup::NpAddDataQueueEntry;
use crate::writesup::NpWriteDataQueue;

// xdk/iotypes.h: IO_NO_INCREMENT
const IO_NO_INCREMENT: LONG = 0; // xdk/iotypes.h: #define IO_NO_INCREMENT 0

// write.c globals
pub static mut NpSlowWriteCalls: LONG = 0;
pub static mut NpFastWriteTrue: ULONG = 0;
pub static mut NpFastWriteFalse: ULONG = 0;

pub unsafe extern "system" fn NpCommonWrite(
    FileObject: PFILE_OBJECT,
    Buffer: PVOID,
    DataSize: ULONG,
    Thread: PETHREAD,
    IoStatus: PIO_STATUS_BLOCK,
    Irp: PIRP,
    List: *mut LIST_ENTRY,
) -> BOOLEAN {
    let mut NodeType: NODE_TYPE_CODE;
    let mut WriteOk: BOOLEAN = FALSE;
    let mut Ccb: PNP_CCB = null_mut();
    let mut NonPagedCcb: PNP_NONPAGED_CCB;
    let mut WriteQueue: PNP_DATA_QUEUE;
    let mut Status: NTSTATUS;
    let mut EventBuffer: PNP_EVENT_BUFFER;
    let mut BytesWritten: ULONG = 0;
    let mut NamedPipeEnd: ULONG = 0;
    let mut ReadMode: ULONG;
    // PAGED_CODE();

    (*IoStatus).Information = 0 as _;
    NodeType = NpDecodeFileObject(FileObject, null_mut(), addr_of_mut!(Ccb), addr_of_mut!(NamedPipeEnd));

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

    // The `Quickie:` label releases the NonPagedCcb lock then returns WriteOk. Model the
    // C `goto Quickie` with a labeled block: break 'quickie reaches the release-and-return.
    'quickie: {
        if (*Ccb).NamedPipeState as ULONG != FILE_PIPE_CONNECTED_STATE {
            if (*Ccb).NamedPipeState as ULONG == FILE_PIPE_DISCONNECTED_STATE {
                (*IoStatus).__bindgen_anon_1.Status = STATUS_PIPE_DISCONNECTED;
            } else if (*Ccb).NamedPipeState as ULONG == FILE_PIPE_LISTENING_STATE {
                (*IoStatus).__bindgen_anon_1.Status = STATUS_PIPE_LISTENING;
            } else {
                // ASSERT(Ccb->NamedPipeState == FILE_PIPE_CLOSING_STATE);
                (*IoStatus).__bindgen_anon_1.Status = STATUS_PIPE_CLOSING;
            }

            WriteOk = TRUE;
            break 'quickie;
        }

        if (NamedPipeEnd == FILE_PIPE_SERVER_END
            && (*(*Ccb).Fcb).NamedPipeConfiguration as ULONG == FILE_PIPE_INBOUND)
            || (NamedPipeEnd == FILE_PIPE_CLIENT_END
                && (*(*Ccb).Fcb).NamedPipeConfiguration as ULONG == FILE_PIPE_OUTBOUND)
        {
            (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
            WriteOk = TRUE;
            break 'quickie;
        }

        (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*IoStatus).Information = DataSize as _;

        if NamedPipeEnd == FILE_PIPE_SERVER_END {
            WriteQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
            ReadMode = (*Ccb).ReadMode[FILE_PIPE_CLIENT_END as usize] as ULONG;
        } else {
            WriteQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
            ReadMode = (*Ccb).ReadMode[FILE_PIPE_SERVER_END as usize] as ULONG;
        }

        EventBuffer = (*NonPagedCcb).EventBuffer[NamedPipeEnd as usize];

        if ((*WriteQueue).QueueState == ReadEntries
            && (*WriteQueue).BytesInQueue < DataSize
            && (*WriteQueue).Quota < DataSize - (*WriteQueue).BytesInQueue)
            || ((*WriteQueue).QueueState != ReadEntries
                && (*WriteQueue).Quota - (*WriteQueue).QuotaUsed < DataSize)
        {
            if (*(*Ccb).Fcb).NamedPipeType as ULONG == FILE_PIPE_MESSAGE_TYPE
                && (*Ccb).CompletionMode[NamedPipeEnd as usize] as ULONG
                    == FILE_PIPE_COMPLETE_OPERATION
            {
                (*IoStatus).Information = 0 as _;
                (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
                WriteOk = TRUE;
                break 'quickie;
            }

            if Irp.is_null() {
                WriteOk = FALSE;
                break 'quickie;
            }
        }

        Status = NpWriteDataQueue(
            WriteQueue,
            ReadMode,
            Buffer,
            DataSize,
            (*(*Ccb).Fcb).NamedPipeType as ULONG,
            addr_of_mut!(BytesWritten),
            Ccb,
            NamedPipeEnd,
            Thread,
            List,
        );
        (*IoStatus).__bindgen_anon_1.Status = Status;

        if Status == STATUS_MORE_PROCESSING_REQUIRED {
            // ASSERT(WriteQueue->QueueState != ReadEntries);
            if ((*Ccb).CompletionMode[NamedPipeEnd as usize] as ULONG
                == FILE_PIPE_COMPLETE_OPERATION
                || Irp.is_null())
                && ((*WriteQueue).Quota - (*WriteQueue).QuotaUsed) < BytesWritten
            {
                (*IoStatus).Information = (DataSize - BytesWritten) as _;
                (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
            } else {
                // ASSERT(WriteQueue->QueueState != ReadEntries);

                (*IoStatus).__bindgen_anon_1.Status = NpAddDataQueueEntry(
                    NamedPipeEnd,
                    Ccb,
                    WriteQueue,
                    WriteEntries,
                    Buffered,
                    DataSize,
                    Irp,
                    Buffer,
                    DataSize - BytesWritten,
                );
            }
        }

        if !EventBuffer.is_null() {
            KeSetEvent((*EventBuffer).Event as PVOID as PVOID, IO_NO_INCREMENT, FALSE);
        }
        WriteOk = TRUE;
    } // 'quickie

    // Quickie:
    ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
    WriteOk
}

pub unsafe extern "system" fn NpFsdWrite(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: *mut IO_STACK_LOCATION;
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    // PAGED_CODE();
    NpSlowWriteCalls += 1;

    InitializeListHead(addr_of_mut!(DeferredList));
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    FsRtlEnterFileSystem();
    NpAcquireSharedVcb();

    NpCommonWrite(
        (*IoStack).FileObject,
        (*Irp).UserBuffer,
        (*IoStack).Parameters.Write.Length,
        (*Irp).Tail.Overlay.Thread,
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

pub unsafe extern "system" fn NpFastWrite(
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
    let mut Result: BOOLEAN;
    // PAGED_CODE();

    InitializeListHead(addr_of_mut!(DeferredList));

    FsRtlEnterFileSystem();
    NpAcquireSharedVcb();

    Result = NpCommonWrite(
        FileObject,
        Buffer,
        Length,
        PsGetCurrentThread(),
        IoStatus,
        null_mut(),
        addr_of_mut!(DeferredList),
    );
    if Result != FALSE {
        NpFastWriteTrue += 1;
    } else {
        NpFastWriteFalse += 1;
    }

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));
    FsRtlExitFileSystem();

    Result
}

/* EOF */
