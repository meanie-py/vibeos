//! cleanup.rs — cleanup.c: Named Pipe FileSystem IRP_MJ_CLEANUP dispatch.
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// cross-module fns
use crate::fileobsup::NpDecodeFileObject;
use crate::statesup::NpSetClosingPipeState;

// File ID number for NPFS bugchecking support
// npfs.h: NPFS_BUGCHECK_CLEANUP — used by the (here unused) NpBugCheck macro.
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_CLEANUP;

pub unsafe extern "system" fn NpCommonCleanup(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut NodeTypeCode: NODE_TYPE_CODE;
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    let mut Fcb: PNP_FCB = null_mut();
    let mut Ccb: PNP_CCB = null_mut();
    let mut NamedPipeEnd: ULONG = 0;
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    InitializeListHead(addr_of_mut!(DeferredList));

    NpAcquireExclusiveVcb();
    NodeTypeCode = NpDecodeFileObject(
        (*IoStack).FileObject,
        addr_of_mut!(Fcb) as *mut PVOID,
        addr_of_mut!(Ccb),
        addr_of_mut!(NamedPipeEnd),
    );
    if NodeTypeCode == NPFS_NTC_CCB {
        if NamedPipeEnd == FILE_PIPE_SERVER_END {
            // ASSERT(Ccb->Fcb->ServerOpenCount != 0);
            (*(*Ccb).Fcb).ServerOpenCount -= 1;
        }

        NpSetClosingPipeState(Ccb, Irp, NamedPipeEnd, addr_of_mut!(DeferredList));
    }

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpFsdCleanup(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    // PAGED_CODE();

    FsRtlEnterFileSystem();

    Status = NpCommonCleanup(DeviceObject, Irp);

    FsRtlExitFileSystem();

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    Status
}

/* EOF */
