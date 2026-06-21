//! writesup.rs — writesup.c: Pipes Writing Support (NpWriteDataQueue).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};
use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty}; // ULONG, USHORT, UCHAR, BOOLEAN, NTSTATUS, NT_SUCCESS, PVOID, SIZE_T, ULONG_PTR, list ops...
use ntkrnl_sys::ddk::*; // bindgen structs/extern fns
use ntkrnl_sys::ex::ExAllocatePoolWithTag;
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use ntkrnl_sys::rtl::RtlCopyMemory;

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// cross-module helpers
use crate::datasup::{NpGetNextRealDataQueueEntry, NpRemoveDataQueueEntry};
use crate::secursup::{NpFreeClientSecurityContext, NpGetClientSecurityContext};

// xdk/iotypes.h: FSCTL_PIPE_INTERNAL_READ_OVFLOW
//   CTL_CODE(FILE_DEVICE_NAMED_PIPE=0x11, 2048, METHOD_BUFFERED=0, FILE_READ_DATA=1)
//   = (0x11<<16) | (1<<14) | (2048<<2) | 0 = 0x00116000
const FSCTL_PIPE_INTERNAL_READ_OVFLOW: ULONG = 0x0011_6000;

// xdk/iotypes.h: IRP flag bits
const IRP_BUFFERED_IO: ULONG = 0x0000_0010; // xdk/iotypes.h: IRP_BUFFERED_IO
const IRP_DEALLOCATE_BUFFER: ULONG = 0x0000_0020; // xdk/iotypes.h: IRP_DEALLOCATE_BUFFER
const IRP_INPUT_OPERATION: ULONG = 0x0000_0040; // xdk/iotypes.h: IRP_INPUT_OPERATION

#[allow(non_snake_case)]
pub unsafe extern "system" fn NpWriteDataQueue(
    WriteQueue: PNP_DATA_QUEUE,
    Mode: ULONG,
    OutBuffer: PVOID,
    OutBufferSize: ULONG,
    PipeType: ULONG,
    BytesNotWritten: *mut ULONG,
    Ccb: PNP_CCB,
    NamedPipeEnd: ULONG,
    Thread: PETHREAD,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    let mut HaveContext: BOOLEAN = FALSE;
    let mut MoreProcessing: BOOLEAN;
    let mut AllocatedBuffer: BOOLEAN = FALSE;
    let mut DataEntry: PNP_DATA_QUEUE_ENTRY;
    let mut DataSize: ULONG;
    let mut BufferSize: ULONG;
    let mut WriteIrp: PIRP;
    let mut IoStack: *mut IO_STACK_LOCATION;
    let mut Buffer: PVOID;
    let mut Status: NTSTATUS;
    let mut ClientContext: PSECURITY_CLIENT_CONTEXT = null_mut();
    // PAGED_CODE();

    *BytesNotWritten = OutBufferSize;

    MoreProcessing = TRUE;
    if (PipeType != FILE_PIPE_MESSAGE_MODE) || (OutBufferSize != 0) {
        MoreProcessing = FALSE;
    }

    DataEntry = CONTAINING_RECORD!(
        NpGetNextRealDataQueueEntry(WriteQueue, List),
        NP_DATA_QUEUE_ENTRY,
        QueueEntry
    );
    while ((*WriteQueue).QueueState == ReadEntries)
        && ((*BytesNotWritten > 0) || (MoreProcessing != FALSE))
    {
        DataSize = (*DataEntry).DataSize;

        IoStack = IoGetCurrentIrpStackLocation((*DataEntry).Irp);

        if (*IoStack).MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL as UCHAR
            && (*IoStack).Parameters.FileSystemControl.FsControlCode
                == FSCTL_PIPE_INTERNAL_READ_OVFLOW
            && (DataSize < OutBufferSize || MoreProcessing != FALSE)
        {
            WriteIrp = NpRemoveDataQueueEntry(WriteQueue, TRUE, List);
            if !WriteIrp.is_null() {
                (*WriteIrp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_OVERFLOW;
                InsertTailList(List, irp_list_entry(WriteIrp));
            }

            DataEntry = CONTAINING_RECORD!(
                NpGetNextRealDataQueueEntry(WriteQueue, List),
                NP_DATA_QUEUE_ENTRY,
                QueueEntry
            );
            continue;
        }

        if (*DataEntry).DataEntryType == Unbuffered {
            (*(*DataEntry).Irp).Overlay.AllocationSize.QuadPart = 0;
        }

        BufferSize = *BytesNotWritten;
        if BufferSize >= DataSize {
            BufferSize = DataSize;
        }

        if (*DataEntry).DataEntryType != Unbuffered && BufferSize != 0 {
            Buffer = ExAllocatePoolWithTag(NonPagedPool, BufferSize as _, NPFS_DATA_ENTRY_TAG);
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            AllocatedBuffer = TRUE;
        } else {
            Buffer = (*(*DataEntry).Irp).AssociatedIrp.SystemBuffer;
            AllocatedBuffer = FALSE;
        }

        // _SEH2_TRY { RtlCopyMemory(...) }
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { free + return exception code }
        // Under USE_DUMMY_PSEH the EXCEPT is dead code; the copy reads from OutBuffer
        // (a kernel buffer already validated by the caller) so emit straight-line.
        RtlCopyMemory(
            Buffer,
            ((OutBuffer as ULONG_PTR) + OutBufferSize as ULONG_PTR
                - *BytesNotWritten as ULONG_PTR) as PVOID,
            BufferSize as _,
        );

        if HaveContext == FALSE {
            HaveContext = TRUE;
            Status = NpGetClientSecurityContext(NamedPipeEnd, Ccb, Thread, addr_of_mut!(ClientContext));
            if !NT_SUCCESS(Status) {
                if AllocatedBuffer != FALSE {
                    ExFreePool(Buffer);
                }
                return Status;
            }

            if !ClientContext.is_null() {
                NpFreeClientSecurityContext((*Ccb).ClientContext);
                (*Ccb).ClientContext = ClientContext;
            }
        }

        WriteIrp = NpRemoveDataQueueEntry(WriteQueue, TRUE, List);
        if !WriteIrp.is_null() {
            *BytesNotWritten -= BufferSize;
            (*WriteIrp).IoStatus.Information = BufferSize as _;

            if AllocatedBuffer != FALSE {
                (*WriteIrp).AssociatedIrp.SystemBuffer = Buffer;
                (*WriteIrp).Flags |= IRP_DEALLOCATE_BUFFER | IRP_BUFFERED_IO | IRP_INPUT_OPERATION;
            }

            if *BytesNotWritten == 0 {
                MoreProcessing = FALSE;
                (*WriteIrp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                InsertTailList(List, irp_list_entry(WriteIrp));

                DataEntry = CONTAINING_RECORD!(
                    NpGetNextRealDataQueueEntry(WriteQueue, List),
                    NP_DATA_QUEUE_ENTRY,
                    QueueEntry
                );
                continue;
            }

            if Mode == FILE_PIPE_MESSAGE_MODE {
                (*WriteIrp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_OVERFLOW;
            } else {
                (*WriteIrp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            }

            InsertTailList(List, irp_list_entry(WriteIrp));
        } else if AllocatedBuffer != FALSE {
            ExFreePool(Buffer);
        }

        DataEntry = CONTAINING_RECORD!(
            NpGetNextRealDataQueueEntry(WriteQueue, List),
            NP_DATA_QUEUE_ENTRY,
            QueueEntry
        );
    }

    if *BytesNotWritten > 0 || MoreProcessing != FALSE {
        // ASSERT(WriteQueue->QueueState != ReadEntries);
        Status = STATUS_MORE_PROCESSING_REQUIRED;
    } else {
        Status = STATUS_SUCCESS;
    }

    Status
}

/* EOF */
