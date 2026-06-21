//! statesup.rs — statesup.c: Pipes State Support.
//! The named-pipe state machine: NpSetConnected/Listening/Disconnected/ClosingPipeState
//! transitions + the NpCancelListeningQueueIrp DRIVER_CANCEL routine.
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::ke::{KeBugCheckEx, KeSetEvent};
// IoMarkIrpPending is a C macro (sets SL_PENDING_RETURNED on the current stack
// location). Use the inline tier1 version (explicit import shadows the ddk::* glob).
use ntkrnl_sys::inline::tier1::{IoMarkIrpPending, IoSetCancelRoutine};
// FsRtlEnterFileSystem / FsRtlExitFileSystem are fltkernel.h macros (Ke{Enter,Leave}
// CriticalRegion) — not in ddk::*; come from inline::common.
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// cross-module fns
use crate::datasup::NpRemoveDataQueueEntry;
use crate::fileobsup::NpSetFileObject;
use crate::secursup::NpUninitializeSecurity;
use crate::strucsup::{NpDeleteCcb, NpDeleteEventTableEntry, NpDeleteFcb};
use crate::waitsup::NpCancelWaiter;

// File ID number for NPFS bugchecking support
// npfs.h: #define NPFS_BUGCHECK_FILE_ID (NPFS_BUGCHECK_STATESUP)
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_STATESUP;

// The fn-pointer slot type for IoSetCancelRoutine. PDRIVER_CANCEL is bindgen
// Option<unsafe extern "C" fn(PDEVICE_OBJECT, PIRP)>; our cancel routine is
// extern "system". Transmute the fn item into the slot type (contract §9).
type CancelFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP);

/// VOID NTAPI DRIVER_CANCEL cancel routine for a parked listening IRP.
pub unsafe extern "system" fn NpCancelListeningQueueIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    FsRtlEnterFileSystem();
    NpAcquireExclusiveVcb();

    RemoveEntryList(irp_list_entry(Irp));

    NpReleaseVcb();
    FsRtlExitFileSystem();

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
}

pub unsafe extern "system" fn NpSetConnectedPipeState(
    Ccb: PNP_CCB,
    FileObject: PFILE_OBJECT,
    List: PLIST_ENTRY,
) -> NTSTATUS {
    let mut NextEntry: PLIST_ENTRY;
    let mut Irp: PIRP;

    // ASSERT(Ccb->NamedPipeState == FILE_PIPE_LISTENING_STATE);

    (*Ccb).ReadMode[FILE_PIPE_CLIENT_END as usize] = FILE_PIPE_BYTE_STREAM_MODE as UCHAR;
    (*Ccb).CompletionMode[FILE_PIPE_CLIENT_END as usize] = FILE_PIPE_QUEUE_OPERATION as UCHAR;
    (*Ccb).NamedPipeState = FILE_PIPE_CONNECTED_STATE as UCHAR;
    (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize] = FileObject;

    NpSetFileObject(FileObject, Ccb as PVOID, (*Ccb).NonPagedCcb as PVOID, FALSE as ULONG);

    while !IsListEmpty(addr_of_mut!((*Ccb).IrpList)) {
        NextEntry = RemoveHeadList(addr_of_mut!((*Ccb).IrpList));

        Irp = irp_from_list_entry(NextEntry);

        // if (IoSetCancelRoutine(Irp, NULL)) — truthy when a previous routine was set.
        if IoSetCancelRoutine(Irp, None).is_some() {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            InsertTailList(List, NextEntry);
        } else {
            InitializeListHead(NextEntry);
        }
    }

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpSetDisconnectedPipeState(
    Ccb: PNP_CCB,
    List: PLIST_ENTRY,
) -> NTSTATUS {
    let mut Irp: PIRP;
    let NonPagedCcb: PNP_NONPAGED_CCB;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut NextEntry: PLIST_ENTRY;
    let mut EventBuffer: PNP_EVENT_BUFFER;

    NonPagedCcb = (*Ccb).NonPagedCcb;

    match (*Ccb).NamedPipeState as ULONG {
        FILE_PIPE_DISCONNECTED_STATE => {
            Status = STATUS_PIPE_DISCONNECTED;
        }

        FILE_PIPE_LISTENING_STATE => {
            while !IsListEmpty(addr_of_mut!((*Ccb).IrpList)) {
                NextEntry = RemoveHeadList(addr_of_mut!((*Ccb).IrpList));

                Irp = irp_from_list_entry(NextEntry);

                if IoSetCancelRoutine(Irp, None).is_some() {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_DISCONNECTED;
                    InsertTailList(List, NextEntry);
                } else {
                    InitializeListHead(NextEntry);
                }
            }

            Status = STATUS_SUCCESS;
        }

        // FILE_PIPE_CONNECTED_STATE falls through into FILE_PIPE_CLOSING_STATE on
        // purpose (C drops down); replicate by running the connected block then the
        // closing block straight-line.
        FILE_PIPE_CONNECTED_STATE => {
            EventBuffer = (*NonPagedCcb).EventBuffer[FILE_PIPE_CLIENT_END as usize];

            while (*Ccb).DataQueue[FILE_PIPE_INBOUND as usize].QueueState != Empty {
                Irp = NpRemoveDataQueueEntry(
                    addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]),
                    FALSE,
                    List,
                );
                if !Irp.is_null() {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_DISCONNECTED;
                    InsertTailList(List, irp_list_entry(Irp));
                }
            }

            while (*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize].QueueState != Empty {
                Irp = NpRemoveDataQueueEntry(
                    addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]),
                    FALSE,
                    List,
                );
                if !Irp.is_null() {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_DISCONNECTED;
                    InsertTailList(List, irp_list_entry(Irp));
                }
            }

            if !EventBuffer.is_null() {
                KeSetEvent((*EventBuffer).Event as PVOID as PVOID, IO_NO_INCREMENT as LONG, FALSE);
            }

            // drop down on purpose... queue will be empty so flush code is nop
            // ASSERT(Ccb->DataQueue[FILE_PIPE_OUTBOUND].QueueState == Empty);

            // --- FILE_PIPE_CLOSING_STATE body (fall-through) ---
            EventBuffer = (*NonPagedCcb).EventBuffer[FILE_PIPE_CLIENT_END as usize];

            while (*Ccb).DataQueue[FILE_PIPE_INBOUND as usize].QueueState != Empty {
                Irp = NpRemoveDataQueueEntry(
                    addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]),
                    FALSE,
                    List,
                );
                if !Irp.is_null() {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_DISCONNECTED;
                    InsertTailList(List, irp_list_entry(Irp));
                }
            }

            // ASSERT(Ccb->DataQueue[FILE_PIPE_OUTBOUND].QueueState == Empty);

            NpDeleteEventTableEntry(addr_of_mut!((*NpVcb).EventTable), EventBuffer as PVOID);
            (*NonPagedCcb).EventBuffer[FILE_PIPE_CLIENT_END as usize] = null_mut();

            NpSetFileObject(
                (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize],
                null_mut(),
                null_mut(),
                FALSE as ULONG,
            );
            (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize] = null_mut();

            NpUninitializeSecurity(Ccb);

            if !(*Ccb).ClientSession.is_null() {
                ExFreePool((*Ccb).ClientSession);
                (*Ccb).ClientSession = null_mut();
            }

            Status = STATUS_SUCCESS;
        }

        FILE_PIPE_CLOSING_STATE => {
            EventBuffer = (*NonPagedCcb).EventBuffer[FILE_PIPE_CLIENT_END as usize];

            while (*Ccb).DataQueue[FILE_PIPE_INBOUND as usize].QueueState != Empty {
                Irp = NpRemoveDataQueueEntry(
                    addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]),
                    FALSE,
                    List,
                );
                if !Irp.is_null() {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_DISCONNECTED;
                    InsertTailList(List, irp_list_entry(Irp));
                }
            }

            // ASSERT(Ccb->DataQueue[FILE_PIPE_OUTBOUND].QueueState == Empty);

            NpDeleteEventTableEntry(addr_of_mut!((*NpVcb).EventTable), EventBuffer as PVOID);
            (*NonPagedCcb).EventBuffer[FILE_PIPE_CLIENT_END as usize] = null_mut();

            NpSetFileObject(
                (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize],
                null_mut(),
                null_mut(),
                FALSE as ULONG,
            );
            (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize] = null_mut();

            NpUninitializeSecurity(Ccb);

            if !(*Ccb).ClientSession.is_null() {
                ExFreePool((*Ccb).ClientSession);
                (*Ccb).ClientSession = null_mut();
            }

            Status = STATUS_SUCCESS;
        }

        _ => {
            // NpBugCheck(Ccb->NamedPipeState, 0, 0); — statesup.c:178
            KeBugCheckEx(
                NPFS_FILE_SYSTEM,
                ((NPFS_BUGCHECK_FILE_ID << 16) | 178u32) as _,
                (*Ccb).NamedPipeState as _,
                0 as _,
                0 as _,
            );
        }
    }

    (*Ccb).NamedPipeState = FILE_PIPE_DISCONNECTED_STATE as UCHAR;
    Status
}

pub unsafe extern "system" fn NpSetListeningPipeState(
    Ccb: PNP_CCB,
    Irp: PIRP,
    List: PLIST_ENTRY,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    match (*Ccb).NamedPipeState as ULONG {
        // FILE_PIPE_DISCONNECTED_STATE drops down on purpose into
        // FILE_PIPE_LISTENING_STATE: run the disconnected prologue then the
        // listening block straight-line (with an early-return guard).
        FILE_PIPE_DISCONNECTED_STATE => {
            Status = NpCancelWaiter(
                addr_of_mut!((*NpVcb).WaitQueue),
                addr_of_mut!((*(*Ccb).Fcb).FullName),
                STATUS_SUCCESS,
                List,
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }

            // --- FILE_PIPE_LISTENING_STATE body (fall-through) ---
            if (*Ccb).CompletionMode[FILE_PIPE_SERVER_END as usize]
                == FILE_PIPE_COMPLETE_OPERATION as UCHAR
            {
                (*Ccb).NamedPipeState = FILE_PIPE_LISTENING_STATE as UCHAR;
                return STATUS_PIPE_LISTENING;
            }

            IoSetCancelRoutine(
                Irp,
                core::mem::transmute::<CancelFn, PDRIVER_CANCEL>(NpCancelListeningQueueIrp),
            );
            if (*Irp).Cancel != 0 && IoSetCancelRoutine(Irp, None).is_some() {
                return STATUS_CANCELLED;
            }

            (*Ccb).NamedPipeState = FILE_PIPE_LISTENING_STATE as UCHAR;
            IoMarkIrpPending(Irp);
            InsertTailList(addr_of_mut!((*Ccb).IrpList), irp_list_entry(Irp));
            return STATUS_PENDING;
        }

        FILE_PIPE_LISTENING_STATE => {
            if (*Ccb).CompletionMode[FILE_PIPE_SERVER_END as usize]
                == FILE_PIPE_COMPLETE_OPERATION as UCHAR
            {
                (*Ccb).NamedPipeState = FILE_PIPE_LISTENING_STATE as UCHAR;
                return STATUS_PIPE_LISTENING;
            }

            IoSetCancelRoutine(
                Irp,
                core::mem::transmute::<CancelFn, PDRIVER_CANCEL>(NpCancelListeningQueueIrp),
            );
            if (*Irp).Cancel != 0 && IoSetCancelRoutine(Irp, None).is_some() {
                return STATUS_CANCELLED;
            }

            (*Ccb).NamedPipeState = FILE_PIPE_LISTENING_STATE as UCHAR;
            IoMarkIrpPending(Irp);
            InsertTailList(addr_of_mut!((*Ccb).IrpList), irp_list_entry(Irp));
            return STATUS_PENDING;
        }

        FILE_PIPE_CONNECTED_STATE => {
            Status = STATUS_PIPE_CONNECTED;
        }

        FILE_PIPE_CLOSING_STATE => {
            Status = STATUS_PIPE_CLOSING;
        }

        _ => {
            // NpBugCheck(Ccb->NamedPipeState, 0, 0); — statesup.c:236
            KeBugCheckEx(
                NPFS_FILE_SYSTEM,
                ((NPFS_BUGCHECK_FILE_ID << 16) | 236u32) as _,
                (*Ccb).NamedPipeState as _,
                0 as _,
                0 as _,
            );
        }
    }

    Status
}

pub unsafe extern "system" fn NpSetClosingPipeState(
    Ccb: PNP_CCB,
    Irp: PIRP,
    NamedPipeEnd: ULONG,
    List: PLIST_ENTRY,
) -> NTSTATUS {
    let NonPagedCcb: PNP_NONPAGED_CCB;
    let Fcb: PNP_FCB;
    let mut NextEntry: PLIST_ENTRY;
    let mut ReadQueue: PNP_DATA_QUEUE;
    let mut WriteQueue: PNP_DATA_QUEUE;
    let mut DataQueue: PNP_DATA_QUEUE;
    let mut EventBuffer: PNP_EVENT_BUFFER;
    let mut ListIrp: PIRP;

    NonPagedCcb = (*Ccb).NonPagedCcb;
    Fcb = (*Ccb).Fcb;

    match (*Ccb).NamedPipeState as ULONG {
        // FILE_PIPE_LISTENING_STATE drops on purpose into FILE_PIPE_DISCONNECTED_STATE:
        // run the listening cleanup loop then the disconnected block straight-line.
        FILE_PIPE_LISTENING_STATE => {
            // ASSERT(NamedPipeEnd == FILE_PIPE_SERVER_END);

            while !IsListEmpty(addr_of_mut!((*Ccb).IrpList)) {
                NextEntry = RemoveHeadList(addr_of_mut!((*Ccb).IrpList));

                ListIrp = irp_from_list_entry(NextEntry);

                if IoSetCancelRoutine(ListIrp, None).is_some() {
                    (*ListIrp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_BROKEN;
                    InsertTailList(List, NextEntry);
                } else {
                    InitializeListHead(NextEntry);
                }
            }

            // Drop on purpose into FILE_PIPE_DISCONNECTED_STATE body
            // ASSERT(NamedPipeEnd == FILE_PIPE_SERVER_END);

            NpSetFileObject(
                (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize],
                null_mut(),
                null_mut(),
                TRUE as ULONG,
            );
            (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize] = null_mut();

            NpSetFileObject(
                (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize],
                null_mut(),
                null_mut(),
                FALSE as ULONG,
            );
            (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize] = null_mut();

            NpDeleteCcb(Ccb, List);
            if (*Fcb).CurrentInstances == 0 {
                NpDeleteFcb(Fcb, List);
            }
        }

        FILE_PIPE_DISCONNECTED_STATE => {
            // ASSERT(NamedPipeEnd == FILE_PIPE_SERVER_END);

            NpSetFileObject(
                (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize],
                null_mut(),
                null_mut(),
                TRUE as ULONG,
            );
            (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize] = null_mut();

            NpSetFileObject(
                (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize],
                null_mut(),
                null_mut(),
                FALSE as ULONG,
            );
            (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize] = null_mut();

            NpDeleteCcb(Ccb, List);
            if (*Fcb).CurrentInstances == 0 {
                NpDeleteFcb(Fcb, List);
            }
        }

        FILE_PIPE_CLOSING_STATE => {
            if NamedPipeEnd == FILE_PIPE_SERVER_END {
                DataQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
            } else {
                DataQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
            }

            NpSetFileObject(
                (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize],
                null_mut(),
                null_mut(),
                TRUE as ULONG,
            );
            (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize] = null_mut();

            NpSetFileObject(
                (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize],
                null_mut(),
                null_mut(),
                FALSE as ULONG,
            );
            (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize] = null_mut();

            while (*DataQueue).QueueState != Empty {
                ListIrp = NpRemoveDataQueueEntry(DataQueue, FALSE, List);
                if !ListIrp.is_null() {
                    (*ListIrp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_BROKEN;
                    InsertTailList(List, irp_list_entry(ListIrp));
                }
            }

            NpUninitializeSecurity(Ccb);

            if !(*Ccb).ClientSession.is_null() {
                ExFreePool((*Ccb).ClientSession);
                (*Ccb).ClientSession = null_mut();
            }

            NpDeleteCcb(Ccb, List);
            if (*Fcb).CurrentInstances == 0 {
                NpDeleteFcb(Fcb, List);
            }
        }

        FILE_PIPE_CONNECTED_STATE => {
            if NamedPipeEnd == FILE_PIPE_SERVER_END {
                ReadQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
                WriteQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);

                NpSetFileObject(
                    (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize],
                    null_mut(),
                    null_mut(),
                    TRUE as ULONG,
                );
                (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize] = null_mut();
            } else {
                ReadQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
                WriteQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);

                NpSetFileObject(
                    (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize],
                    null_mut(),
                    null_mut(),
                    FALSE as ULONG,
                );
                (*Ccb).FileObject[FILE_PIPE_CLIENT_END as usize] = null_mut();
            }

            EventBuffer = (*NonPagedCcb).EventBuffer[NamedPipeEnd as usize];

            (*Ccb).NamedPipeState = FILE_PIPE_CLOSING_STATE as UCHAR;

            while (*ReadQueue).QueueState != Empty {
                ListIrp = NpRemoveDataQueueEntry(ReadQueue, FALSE, List);
                if !ListIrp.is_null() {
                    (*ListIrp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_BROKEN;
                    InsertTailList(List, irp_list_entry(ListIrp));
                }
            }

            while (*WriteQueue).QueueState == ReadEntries {
                ListIrp = NpRemoveDataQueueEntry(WriteQueue, FALSE, List);
                if !ListIrp.is_null() {
                    (*ListIrp).IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_BROKEN;
                    InsertTailList(List, irp_list_entry(ListIrp));
                }
            }

            if !EventBuffer.is_null() {
                KeSetEvent((*EventBuffer).Event as PVOID as PVOID, IO_NO_INCREMENT as LONG, FALSE);
            }
        }

        _ => {
            // NpBugCheck(Ccb->NamedPipeState, 0, 0); — statesup.c:385
            KeBugCheckEx(
                NPFS_FILE_SYSTEM,
                ((NPFS_BUGCHECK_FILE_ID << 16) | 385u32) as _,
                (*Ccb).NamedPipeState as _,
                0 as _,
                0 as _,
            );
        }
    }

    STATUS_SUCCESS
}

/* EOF */
