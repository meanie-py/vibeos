//! seinfo.rs — seinfo.c: Pipes Security Information (query/set security descriptor).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ex::ExFreePoolWithTag;
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// Cross-module call: NpDecodeFileObject lives in fileobsup.c (fileobsup.rs).
use crate::fileobsup::NpDecodeFileObject;

// File ID number for NPFS bugchecking support
// npfs.h: #define NPFS_BUGCHECK_FILE_ID (NPFS_BUGCHECK_SEINFO)
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_SEINFO;

/* FUNCTIONS ******************************************************************/

pub unsafe extern "system" fn NpCommonQuerySecurityInfo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let NodeTypeCode: NODE_TYPE_CODE;
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut Fcb: PNP_FCB = null_mut();
    let mut Ccb: PNP_CCB = null_mut();
    let mut NamedPipeEnd: ULONG = 0;
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    NodeTypeCode = NpDecodeFileObject(
        (*IoStack).FileObject,
        addr_of_mut!(Fcb) as *mut PVOID,
        addr_of_mut!(Ccb),
        addr_of_mut!(NamedPipeEnd),
    );
    if NodeTypeCode == 0 {
        return STATUS_PIPE_DISCONNECTED;
    }
    if NodeTypeCode != NPFS_NTC_CCB {
        return STATUS_INVALID_PARAMETER;
    }

    Status = SeQuerySecurityDescriptorInfo(
        addr_of_mut!((*IoStack).Parameters.QuerySecurity.SecurityInformation) as PSECURITY_INFORMATION,
        (*Irp).UserBuffer,
        addr_of_mut!((*IoStack).Parameters.QuerySecurity.Length),
        addr_of_mut!((*Fcb).SecurityDescriptor),
    );
    if Status == STATUS_BUFFER_TOO_SMALL {
        (*Irp).IoStatus.Information = (*IoStack).Parameters.QuerySecurity.Length as _;
        Status = STATUS_BUFFER_OVERFLOW;
    }

    Status
}

pub unsafe extern "system" fn NpCommonSetSecurityInfo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let NodeTypeCode: NODE_TYPE_CODE;
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut Fcb: PNP_FCB = null_mut();
    let mut Ccb: PNP_CCB = null_mut();
    let mut NamedPipeEnd: ULONG = 0;
    let OldSecurityDescriptor: PSECURITY_DESCRIPTOR;
    let mut TempSecurityDescriptor: PSECURITY_DESCRIPTOR;
    let mut NewSecurityDescriptor: PSECURITY_DESCRIPTOR = null_mut();
    // PAGED_CODE();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    NodeTypeCode = NpDecodeFileObject(
        (*IoStack).FileObject,
        addr_of_mut!(Fcb) as *mut PVOID,
        addr_of_mut!(Ccb),
        addr_of_mut!(NamedPipeEnd),
    );
    if NodeTypeCode == 0 {
        return STATUS_PIPE_DISCONNECTED;
    }
    if NodeTypeCode != NPFS_NTC_CCB {
        return STATUS_INVALID_PARAMETER;
    }

    TempSecurityDescriptor = (*Fcb).SecurityDescriptor;
    OldSecurityDescriptor = TempSecurityDescriptor;
    Status = SeSetSecurityDescriptorInfo(
        null_mut(),
        addr_of_mut!((*IoStack).Parameters.SetSecurity.SecurityInformation) as PSECURITY_INFORMATION,
        (*IoStack).Parameters.SetSecurity.SecurityDescriptor,
        addr_of_mut!(TempSecurityDescriptor),
        PagedPool,
        IoGetFileObjectGenericMapping(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ObLogSecurityDescriptor(TempSecurityDescriptor, addr_of_mut!(NewSecurityDescriptor), 1);
    // ASSERT(TempSecurityDescriptor != OldSecurityDescriptor);
    ExFreePoolWithTag(TempSecurityDescriptor, 0);

    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*Fcb).SecurityDescriptor = NewSecurityDescriptor;
    ObDereferenceSecurityDescriptor(OldSecurityDescriptor, 1);
    Status
}

pub unsafe extern "system" fn NpFsdQuerySecurityInfo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    // PAGED_CODE();

    FsRtlEnterFileSystem();
    NpAcquireExclusiveVcb();

    Status = NpCommonQuerySecurityInfo(DeviceObject, Irp);

    NpReleaseVcb();
    FsRtlExitFileSystem();

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    Status
}

pub unsafe extern "system" fn NpFsdSetSecurityInfo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    // PAGED_CODE();

    FsRtlEnterFileSystem();
    NpAcquireExclusiveVcb();

    Status = NpCommonSetSecurityInfo(DeviceObject, Irp);

    NpReleaseVcb();
    FsRtlExitFileSystem();

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    Status
}

/* EOF */
