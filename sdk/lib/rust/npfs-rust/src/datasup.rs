//! datasup.rs — datasup.c: Data Queues Support (NP_DATA_QUEUE engine + cancel routine).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};
use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// PLIST_ENTRY/LIST_ENTRY exist in both base::* and ddk::* globs (same target, distinct items) —
// explicit imports shadow the globs and resolve the ambiguity.
use ntkrnl_sys::ddk::{LIST_ENTRY, PLIST_ENTRY};

// cross-module fns
use crate::secursup::{NpFreeClientSecurityContext, NpGetClientSecurityContext};
use ntkrnl_sys::driver_ddk::IoCompleteRequest;

// pool — datasup.c uses ExAllocatePoolWithQuotaTag + the tagless ExFreePool (bindgen exports both)
use ntkrnl_sys::ddk::{ExAllocatePoolWithQuotaTag, ExFreePool};

// FsRtlEnterFileSystem/FsRtlExitFileSystem are inline (not bindgen externs); PsGetCurrentThread,
// IoMarkIrpPending, IoSetCancelRoutine, IoReleaseCancelSpinLock come via the ddk glob (bindgen).
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::inline::tier1::{IoSetCancelRoutine, IoMarkIrpPending};

// iotypes.h: POOL_QUOTA_FAIL_INSTEAD_OF_RAISE
const POOL_QUOTA_FAIL_INSTEAD_OF_RAISE: ULONG = 0x8; // sdk/include/xdk/iotypes.h:17

// excpt.h: EXCEPTION_EXECUTE_HANDLER (used only in the dropped SEH filter; kept for fidelity)
const EXCEPTION_EXECUTE_HANDLER: i32 = 1; // sdk/include/vcruntime/excpt.h:90

// DRIVER_CANCEL slot is bindgen Option<extern "C" fn>; our cancel routine is extern "system".
type CancelFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP);

/// &Irp->Tail.Overlay.DriverContext[i]  (bindgen anon-union path)
#[inline(always)]
unsafe fn irp_driver_context(Irp: PIRP, i: usize) -> *mut PVOID {
    addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[i])
}

pub unsafe extern "system" fn NpUninitializeDataQueue(DataQueue: PNP_DATA_QUEUE) -> NTSTATUS {
    // PAGED_CODE();
    // ASSERT(DataQueue->QueueState == Empty);
    core::ptr::write_bytes(DataQueue as *mut u8, 0, core::mem::size_of::<NP_DATA_QUEUE>());
    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpInitializeDataQueue(
    DataQueue: PNP_DATA_QUEUE,
    Quota: ULONG,
) -> NTSTATUS {
    // PAGED_CODE();
    (*DataQueue).BytesInQueue = 0;
    (*DataQueue).EntriesInQueue = 0;
    (*DataQueue).QuotaUsed = 0;
    (*DataQueue).ByteOffset = 0;
    (*DataQueue).QueueState = Empty;
    (*DataQueue).Quota = Quota;
    InitializeListHead(addr_of_mut!((*DataQueue).Queue));
    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpCompleteStalledWrites(
    DataQueue: PNP_DATA_QUEUE,
    List: PLIST_ENTRY,
) {
    let mut QuotaLeft: ULONG;
    let mut ByteOffset: ULONG;
    let mut DataLeft: ULONG;
    let mut NewQuotaLeft: ULONG;
    let mut DataQueueEntry: PNP_DATA_QUEUE_ENTRY;
    let mut Irp: PIRP;
    let mut NextEntry: PLIST_ENTRY;

    QuotaLeft = (*DataQueue).Quota - (*DataQueue).QuotaUsed;
    ByteOffset = (*DataQueue).ByteOffset;

    NextEntry = (*DataQueue).Queue.Flink;
    while NextEntry != addr_of_mut!((*DataQueue).Queue) {
        if QuotaLeft == 0 {
            break;
        }

        DataQueueEntry = CONTAINING_RECORD!(NextEntry, NP_DATA_QUEUE_ENTRY, QueueEntry);

        Irp = (*DataQueueEntry).Irp;

        if (*DataQueueEntry).DataEntryType == Buffered && !Irp.is_null() {
            DataLeft = (*DataQueueEntry).DataSize - ByteOffset;

            if (*DataQueueEntry).QuotaInEntry < DataLeft {
                NewQuotaLeft = DataLeft - (*DataQueueEntry).QuotaInEntry;
                if NewQuotaLeft > QuotaLeft {
                    NewQuotaLeft = QuotaLeft;
                }

                QuotaLeft -= NewQuotaLeft;
                (*DataQueueEntry).QuotaInEntry += NewQuotaLeft;

                if (*DataQueueEntry).QuotaInEntry == DataLeft
                    && !IoSetCancelRoutine(Irp, None).is_none()
                {
                    (*DataQueueEntry).Irp = null_mut();

                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                    (*Irp).IoStatus.Information = (*DataQueueEntry).DataSize as _;

                    InsertTailList(List, irp_list_entry(Irp));
                }
            }
        }

        NextEntry = (*NextEntry).Flink;
        ByteOffset = 0;
    }

    (*DataQueue).QuotaUsed = (*DataQueue).Quota - QuotaLeft;
}

pub unsafe extern "system" fn NpRemoveDataQueueEntry(
    DataQueue: PNP_DATA_QUEUE,
    Flag: BOOLEAN,
    List: PLIST_ENTRY,
) -> PIRP {
    let mut Irp: PIRP;
    let mut QueueEntry: PNP_DATA_QUEUE_ENTRY;
    let mut HasWrites: BOOLEAN;

    if (*DataQueue).QueueState == Empty {
        Irp = null_mut();
        // ASSERT(IsListEmpty(&DataQueue->Queue));
        // ASSERT(DataQueue->EntriesInQueue == 0);
        // ASSERT(DataQueue->BytesInQueue == 0);
        // ASSERT(DataQueue->QuotaUsed == 0);
    } else {
        QueueEntry = CONTAINING_RECORD!(
            RemoveHeadList(addr_of_mut!((*DataQueue).Queue)),
            NP_DATA_QUEUE_ENTRY,
            QueueEntry
        );

        (*DataQueue).BytesInQueue -= (*QueueEntry).DataSize;
        (*DataQueue).EntriesInQueue -= 1;

        HasWrites = TRUE;
        if (*DataQueue).QueueState != WriteEntries
            || (*DataQueue).QuotaUsed < (*DataQueue).Quota
            || (*QueueEntry).QuotaInEntry == 0
        {
            HasWrites = FALSE;
        }

        (*DataQueue).QuotaUsed -= (*QueueEntry).QuotaInEntry;

        if IsListEmpty(addr_of!((*DataQueue).Queue)) {
            (*DataQueue).QueueState = Empty;
            HasWrites = FALSE;
        }

        Irp = (*QueueEntry).Irp;
        NpFreeClientSecurityContext((*QueueEntry).ClientSecurityContext);

        if !Irp.is_null() && IoSetCancelRoutine(Irp, None).is_none() {
            *irp_driver_context(Irp, 3) = null_mut();
            Irp = null_mut();
        }

        ExFreePool(QueueEntry as PVOID);

        if Flag != 0 {
            NpGetNextRealDataQueueEntry(DataQueue, List);
        }

        if HasWrites != 0 {
            NpCompleteStalledWrites(DataQueue, List);
        }
    }

    (*DataQueue).ByteOffset = 0;
    Irp
}

pub unsafe extern "system" fn NpGetNextRealDataQueueEntry(
    DataQueue: PNP_DATA_QUEUE,
    List: PLIST_ENTRY,
) -> PLIST_ENTRY {
    let mut DataEntry: PNP_DATA_QUEUE_ENTRY;
    let mut Type: ULONG;
    let mut Irp: PIRP;
    let mut NextEntry: PLIST_ENTRY;
    // PAGED_CODE();

    NextEntry = (*DataQueue).Queue.Flink;
    while NextEntry != addr_of_mut!((*DataQueue).Queue) {
        DataEntry = CONTAINING_RECORD!(NextEntry, NP_DATA_QUEUE_ENTRY, QueueEntry);

        Type = (*DataEntry).DataEntryType;
        if Type == Buffered || Type == Unbuffered {
            break;
        }

        Irp = NpRemoveDataQueueEntry(DataQueue, FALSE, List);
        if !Irp.is_null() {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            InsertTailList(List, irp_list_entry(Irp));
        }

        // for-loop reinit: NextEntry = DataQueue->Queue.Flink
        NextEntry = (*DataQueue).Queue.Flink;
    }

    NextEntry
}

/// DRIVER_CANCEL callback. DeviceObject == NULL means the caller already holds the locks and
/// is invoking the cancel path inline (NpAddDataQueueEntry's race handling).
pub unsafe extern "system" fn NpCancelDataQueueIrp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let mut DataQueue: PNP_DATA_QUEUE;
    let mut DataEntry: PNP_DATA_QUEUE_ENTRY;
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    let mut ClientSecurityContext: PSECURITY_CLIENT_CONTEXT;
    let mut CompleteWrites: BOOLEAN;
    let mut FirstEntry: BOOLEAN = FALSE;

    if !DeviceObject.is_null() {
        IoReleaseCancelSpinLock((*Irp).CancelIrql);
    }

    InitializeListHead(addr_of_mut!(DeferredList));

    DataQueue = *irp_driver_context(Irp, 2) as PNP_DATA_QUEUE;
    ClientSecurityContext = null_mut();

    if !DeviceObject.is_null() {
        FsRtlEnterFileSystem();
        NpAcquireExclusiveVcb();
    }

    DataEntry = *irp_driver_context(Irp, 3) as PNP_DATA_QUEUE_ENTRY;
    if !DataEntry.is_null() {
        if (*DataEntry).QueueEntry.Blink == addr_of_mut!((*DataQueue).Queue) {
            (*DataQueue).ByteOffset = 0;
            FirstEntry = TRUE;
        } else {
            FirstEntry = FALSE;
        }

        RemoveEntryList(addr_of_mut!((*DataEntry).QueueEntry));

        ClientSecurityContext = (*DataEntry).ClientSecurityContext;

        CompleteWrites = TRUE;
        if (*DataQueue).QueueState != WriteEntries
            || (*DataQueue).QuotaUsed < (*DataQueue).Quota
            || (*DataEntry).QuotaInEntry == 0
        {
            CompleteWrites = FALSE;
        }

        (*DataQueue).BytesInQueue -= (*DataEntry).DataSize;
        (*DataQueue).QuotaUsed -= (*DataEntry).QuotaInEntry;
        (*DataQueue).EntriesInQueue -= 1;

        if IsListEmpty(addr_of!((*DataQueue).Queue)) {
            (*DataQueue).QueueState = Empty;
            // ASSERT(DataQueue->BytesInQueue == 0);
            // ASSERT(DataQueue->EntriesInQueue == 0);
            // ASSERT(DataQueue->QuotaUsed == 0);
        } else {
            if FirstEntry != 0 {
                NpGetNextRealDataQueueEntry(DataQueue, addr_of_mut!(DeferredList));
            }
            if CompleteWrites != 0 {
                NpCompleteStalledWrites(DataQueue, addr_of_mut!(DeferredList));
            }
        }
    }

    if !DeviceObject.is_null() {
        NpReleaseVcb();
        FsRtlExitFileSystem();
    }

    if !DataEntry.is_null() {
        ExFreePool(DataEntry as PVOID);
    }

    NpFreeClientSecurityContext(ClientSecurityContext);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);

    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));
}

pub unsafe extern "system" fn NpAddDataQueueEntry(
    NamedPipeEnd: ULONG,
    Ccb: PNP_CCB,
    DataQueue: PNP_DATA_QUEUE,
    Who: ULONG,
    Type: ULONG,
    DataSize: ULONG,
    Irp: PIRP,
    Buffer: PVOID,
    ByteOffset: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DataEntry: PNP_DATA_QUEUE_ENTRY;
    let mut EntrySize: SIZE_T;
    let mut QuotaInEntry: ULONG;
    let mut ClientContext: PSECURITY_CLIENT_CONTEXT;
    let mut HasSpace: BOOLEAN = FALSE;

    ClientContext = null_mut();
    // ASSERT((DataQueue->QueueState == Empty) || (DataQueue->QueueState == Who));

    Status = STATUS_SUCCESS;

    if Type != 2 && Who == WriteEntries {
        Status = NpGetClientSecurityContext(
            NamedPipeEnd,
            Ccb,
            if !Irp.is_null() {
                (*Irp).Tail.Overlay.Thread
            } else {
                PsGetCurrentThread()
            },
            addr_of_mut!(ClientContext),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    match Type {
        // Unbuffered (1) | 2 | 3
        x if x == Unbuffered || x == 2 || x == 3 => {
            // ASSERT(Irp != NULL);
            DataEntry = ExAllocatePoolWithQuotaTag(
                NonPagedPool | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE as POOL_TYPE,
                core::mem::size_of::<NP_DATA_QUEUE_ENTRY>() as SIZE_T,
                NPFS_DATA_ENTRY_TAG,
            ) as PNP_DATA_QUEUE_ENTRY;
            if DataEntry.is_null() {
                NpFreeClientSecurityContext(ClientContext);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*DataEntry).DataEntryType = Type;
            (*DataEntry).QuotaInEntry = 0;
            (*DataEntry).Irp = Irp;
            (*DataEntry).DataSize = DataSize;
            (*DataEntry).ClientSecurityContext = ClientContext;
            // ASSERT((DataQueue->QueueState == Empty) || (DataQueue->QueueState == Who));
            Status = STATUS_PENDING;
        }

        // Buffered (0)
        x if x == Buffered => {
            EntrySize = core::mem::size_of::<NP_DATA_QUEUE_ENTRY>() as _;
            if Who != ReadEntries {
                EntrySize += DataSize as SIZE_T;
                if EntrySize < DataSize as SIZE_T {
                    NpFreeClientSecurityContext(ClientContext);
                    return STATUS_INVALID_PARAMETER;
                }
            }

            QuotaInEntry = DataSize - ByteOffset;
            if (*DataQueue).Quota - (*DataQueue).QuotaUsed < QuotaInEntry {
                QuotaInEntry = (*DataQueue).Quota - (*DataQueue).QuotaUsed;
                HasSpace = TRUE;
            } else {
                HasSpace = FALSE;
            }

            DataEntry = ExAllocatePoolWithQuotaTag(
                NonPagedPool | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE as POOL_TYPE,
                EntrySize,
                NPFS_DATA_ENTRY_TAG,
            ) as PNP_DATA_QUEUE_ENTRY;
            if DataEntry.is_null() {
                NpFreeClientSecurityContext(ClientContext);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*DataEntry).QuotaInEntry = QuotaInEntry;
            (*DataEntry).Irp = Irp;
            (*DataEntry).DataEntryType = Buffered;
            (*DataEntry).ClientSecurityContext = ClientContext;
            (*DataEntry).DataSize = DataSize;

            if Who == ReadEntries {
                // ASSERT(Irp);
                Status = STATUS_PENDING;
                // ASSERT((DataQueue->QueueState == Empty) || (DataQueue->QueueState == Who));
            } else {
                // _SEH2_TRY { RtlCopyMemory(DataEntry + 1, Irp ? Irp->UserBuffer : Buffer, DataSize) }
                // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { NpFreeClientSecurityContext(ClientContext);
                //   _SEH2_YIELD(return _SEH2_GetExceptionCode()); } _SEH2_END;
                // Under USE_DUMMY_PSEH the EXCEPT is dead code: emit the TRY body only.
                // DataEntry + 1 = byte-after-the-entry (the inline data area).
                let src = if !Irp.is_null() { (*Irp).UserBuffer } else { Buffer };
                core::ptr::copy_nonoverlapping(
                    src as *const u8,
                    DataEntry.add(1) as *mut u8,
                    DataSize as usize,
                );

                if HasSpace != 0 && !Irp.is_null() {
                    Status = STATUS_PENDING;
                } else {
                    (*DataEntry).Irp = null_mut();
                    Status = STATUS_SUCCESS;
                }

                // ASSERT((DataQueue->QueueState == Empty) || (DataQueue->QueueState == Who));
            }
        }

        _ => {
            // ASSERT(FALSE);
            NpFreeClientSecurityContext(ClientContext);
            return STATUS_INVALID_PARAMETER;
        }
    }

    // ASSERT((DataQueue->QueueState == Empty) || (DataQueue->QueueState == Who));
    // (the Empty/non-Empty asserts are debug-only; no runtime effect)

    (*DataQueue).QuotaUsed += (*DataEntry).QuotaInEntry;
    (*DataQueue).QueueState = Who;
    (*DataQueue).BytesInQueue += (*DataEntry).DataSize;
    (*DataQueue).EntriesInQueue += 1;

    if ByteOffset != 0 {
        (*DataQueue).ByteOffset = ByteOffset;
        // ASSERT(Who == WriteEntries);
        // ASSERT(ByteOffset < DataEntry->DataSize);
        // ASSERT(DataQueue->EntriesInQueue == 1);
    }

    InsertTailList(addr_of_mut!((*DataQueue).Queue), addr_of_mut!((*DataEntry).QueueEntry));

    if Status == STATUS_PENDING {
        IoMarkIrpPending(Irp);
        *irp_driver_context(Irp, 2) = DataQueue as PVOID;
        *irp_driver_context(Irp, 3) = DataEntry as PVOID;

        IoSetCancelRoutine(
            Irp,
            core::mem::transmute::<CancelFn, PDRIVER_CANCEL>(NpCancelDataQueueIrp),
        );

        if (*Irp).Cancel != 0 && !IoSetCancelRoutine(Irp, None).is_none() {
            NpCancelDataQueueIrp(null_mut(), Irp);
        }
    }

    Status
}

/* EOF */
