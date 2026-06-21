//! flushbuf.rs — flushbuf.c: Buffers Flushing Support (NpCommonFlushBuffers / NpFsdFlushBuffers).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::glue::*;
use crate::globals::*;
use crate::helpers::*;
use crate::types::*;
use crate::CONTAINING_RECORD;

// cross-module calls
use crate::datasup::NpAddDataQueueEntry;
use crate::fileobsup::NpDecodeFileObject;

// File ID number for NPFS bugchecking support
// flushbuf.c: #define NPFS_BUGCHECK_FILE_ID (NPFS_BUGCHECK_FLUSHBUF)
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_FLUSHBUF;

pub unsafe extern "system" fn NpCommonFlushBuffers(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let NodeTypeCode: NODE_TYPE_CODE;
    let mut Ccb: PNP_CCB = null_mut();
    let mut NamedPipeEnd: ULONG = 0;
    let Status: NTSTATUS;
    let FlushQueue: PNP_DATA_QUEUE;
    // PAGED_CODE();

    NodeTypeCode = NpDecodeFileObject(
        (*IoGetCurrentIrpStackLocation(Irp)).FileObject,
        null_mut(),
        addr_of_mut!(Ccb),
        addr_of_mut!(NamedPipeEnd),
    );
    if NodeTypeCode != NPFS_NTC_CCB {
        return STATUS_PIPE_DISCONNECTED;
    }

    ExAcquireResourceExclusiveLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock), TRUE);

    if NamedPipeEnd == FILE_PIPE_SERVER_END {
        FlushQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]);
    } else {
        FlushQueue = addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]);
    }

    if (*FlushQueue).QueueState == WriteEntries {
        Status = NpAddDataQueueEntry(
            NamedPipeEnd,
            Ccb,
            FlushQueue,
            WriteEntries,
            2,
            0,
            Irp,
            null_mut(),
            0,
        );
    } else {
        Status = STATUS_SUCCESS;
    }

    ExReleaseResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
    Status
}

pub unsafe extern "system" fn NpFsdFlushBuffers(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    // PAGED_CODE();

    FsRtlEnterFileSystem();
    NpAcquireSharedVcb();

    Status = NpCommonFlushBuffers(DeviceObject, Irp);

    NpReleaseVcb();
    FsRtlExitFileSystem();

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    Status
}

/* EOF */
