//! close.rs — close.c: Pipes closing (IRP_MJ_CLOSE dispatch + common handler).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty}; // ULONG, NTSTATUS, PVOID, list ops, InitializeListHead...
use ntkrnl_sys::ddk::*; // DEVICE_OBJECT, IRP, FILE_OBJECT, IO_STACK_LOCATION...
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::glue::*;
use crate::globals::*;
use crate::helpers::*;
use crate::types::*;
use crate::CONTAINING_RECORD;

// File ID number for NPFS bugchecking support (close.c: NPFS_BUGCHECK_FILE_ID = NPFS_BUGCHECK_CLOSE)
// Cross-module helpers this file calls (function -> module map):
use crate::fileobsup::NpDecodeFileObject;
use crate::strucsup::NpDeleteCcb;

/// close.c:18 — NpCommonClose: common close handler for the NPFS device.
pub unsafe extern "system" fn NpCommonClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut NodeTypeCode: NODE_TYPE_CODE;
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    let mut Fcb: PNP_FCB = null_mut();
    let mut Ccb: PNP_CCB = null_mut();
    let mut NamedPipeEnd: ULONG = 0;
    // PAGED_CODE();  // debug-only no-op

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    InitializeListHead(addr_of_mut!(DeferredList));

    NpAcquireExclusiveVcb();
    NodeTypeCode = NpDecodeFileObject(
        (*IoStack).FileObject,
        addr_of_mut!(Fcb) as *mut PVOID,
        addr_of_mut!(Ccb),
        addr_of_mut!(NamedPipeEnd),
    );
    if NodeTypeCode == NPFS_NTC_ROOT_DCB {
        (*Fcb).CurrentInstances -= 1;
        NpDeleteCcb(Ccb, addr_of_mut!(DeferredList));
    } else if NodeTypeCode == NPFS_NTC_VCB {
        (*NpVcb).ReferenceCount -= 1;
    }

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    STATUS_SUCCESS
}

/// close.c:57 — NpFsdClose: IRP_MJ_CLOSE FSD dispatch entry.
pub unsafe extern "system" fn NpFsdClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    // PAGED_CODE();  // debug-only no-op

    FsRtlEnterFileSystem();

    Status = NpCommonClose(DeviceObject, Irp);

    FsRtlExitFileSystem();

    Status
}

/* EOF */
