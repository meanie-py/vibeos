//! readsup.rs — readsup.c: Pipes Reading Support (NpReadDataQueue).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::rtl::RtlCopyMemory;

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// File ID number for NPFS bugchecking support: NPFS_BUGCHECK_READSUP (glue.rs)

// cross-module helpers
use crate::datasup::{NpCompleteStalledWrites, NpGetNextRealDataQueueEntry, NpRemoveDataQueueEntry};
use crate::secursup::NpCopyClientContext;

/// IO_STATUS_BLOCK NTAPI NpReadDataQueue(...)
///
/// Walks the NP_DATA_QUEUE entries copying buffered/unbuffered data into Buffer, and
/// returns an IO_STATUS_BLOCK by value (Status + Information = total bytes copied).
pub unsafe extern "system" fn NpReadDataQueue(
    DataQueue: PNP_DATA_QUEUE,
    mut Peek: BOOLEAN,
    ReadOverflowOperation: BOOLEAN,
    Buffer: PVOID,
    BufferSize: ULONG,
    Mode: ULONG,
    Ccb: PNP_CCB,
    List: *mut LIST_ENTRY,
) -> IO_STATUS_BLOCK {
    let mut DataEntry: PNP_DATA_QUEUE_ENTRY;
    let mut TempDataEntry: PNP_DATA_QUEUE_ENTRY;
    let mut DataBuffer: PVOID;
    let mut DataSize: ULONG;
    let mut DataLength: ULONG;
    let mut TotalBytesCopied: ULONG;
    let mut RemainingSize: ULONG;
    let mut Offset: ULONG;
    let mut Irp: PIRP;
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut CompleteWrites: BOOLEAN = FALSE;
    // PAGED_CODE();

    if ReadOverflowOperation != 0 {
        Peek = TRUE;
    }

    RemainingSize = BufferSize;
    IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    TotalBytesCopied = 0;

    if Peek != 0 {
        DataEntry = CONTAINING_RECORD!(
            (*DataQueue).Queue.Flink,
            NP_DATA_QUEUE_ENTRY,
            QueueEntry
        );
    } else {
        DataEntry = CONTAINING_RECORD!(
            NpGetNextRealDataQueueEntry(DataQueue, List),
            NP_DATA_QUEUE_ENTRY,
            QueueEntry
        );
    }

    while (addr_of_mut!((*DataEntry).QueueEntry) != addr_of_mut!((*DataQueue).Queue))
        && (RemainingSize != 0)
    {
        if Peek == 0
            || (*DataEntry).DataEntryType == Buffered
            || (*DataEntry).DataEntryType == Unbuffered
        {
            if (*DataEntry).DataEntryType == Unbuffered {
                DataBuffer = (*(*DataEntry).Irp).AssociatedIrp.SystemBuffer;
            } else {
                // &DataEntry[1] — the inline buffered data that follows the entry struct
                DataBuffer = DataEntry.add(1) as PVOID;
            }

            DataSize = (*DataEntry).DataSize;
            Offset = DataSize;

            if addr_of_mut!((*DataEntry).QueueEntry) as *mut LIST_ENTRY == (*DataQueue).Queue.Flink {
                Offset -= (*DataQueue).ByteOffset;
            }

            DataLength = Offset;
            if DataLength >= RemainingSize {
                DataLength = RemainingSize;
            }

            // _SEH2_TRY { RtlCopyMemory(...) } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { ASSERT(FALSE); }
            // USE_DUMMY_PSEH: emit the TRY body only; the EXCEPT filter+handler are dead code.
            RtlCopyMemory(
                ((Buffer as ULONG_PTR) + BufferSize as ULONG_PTR - RemainingSize as ULONG_PTR)
                    as PVOID,
                ((DataBuffer as ULONG_PTR) + DataSize as ULONG_PTR - Offset as ULONG_PTR) as PVOID,
                DataLength as _,
            );

            RemainingSize -= DataLength;
            Offset -= DataLength;
            TotalBytesCopied += DataLength;

            if Peek == 0 {
                (*DataEntry).QuotaInEntry -= DataLength;
                (*DataQueue).QuotaUsed -= DataLength;
                (*DataQueue).ByteOffset += DataLength;
                CompleteWrites = TRUE;
            }

            NpCopyClientContext(Ccb, DataEntry);

            if (Offset != 0) || (ReadOverflowOperation != 0 && TotalBytesCopied == 0) {
                if Mode == FILE_PIPE_MESSAGE_MODE {
                    IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_OVERFLOW;
                    break;
                }
            } else {
                if Peek == 0 || ReadOverflowOperation != 0 {
                    if ReadOverflowOperation != 0 {
                        TempDataEntry = CONTAINING_RECORD!(
                            NpGetNextRealDataQueueEntry(DataQueue, List),
                            NP_DATA_QUEUE_ENTRY,
                            QueueEntry
                        );
                        // ASSERT(TempDataEntry == DataEntry);
                    }

                    Irp = NpRemoveDataQueueEntry(DataQueue, TRUE, List);
                    if !Irp.is_null() {
                        (*Irp).IoStatus.Information = DataSize as _;
                        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                        InsertTailList(List, irp_list_entry(Irp));
                    }
                }

                if Mode == FILE_PIPE_MESSAGE_MODE {
                    IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                    break;
                }

                // ASSERT(!ReadOverflowOperation);
            }
        }

        if Peek != 0 {
            DataEntry = CONTAINING_RECORD!(
                (*DataEntry).QueueEntry.Flink,
                NP_DATA_QUEUE_ENTRY,
                QueueEntry
            );
        } else {
            DataEntry = CONTAINING_RECORD!(
                NpGetNextRealDataQueueEntry(DataQueue, List),
                NP_DATA_QUEUE_ENTRY,
                QueueEntry
            );
        }
    }

    IoStatus.Information = TotalBytesCopied as _;
    if CompleteWrites != 0 {
        NpCompleteStalledWrites(DataQueue, List);
    }
    IoStatus
}

/* EOF */
