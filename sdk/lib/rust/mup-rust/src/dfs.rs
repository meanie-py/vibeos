//! dfs.rs — Rust translation of drivers/filesystems/mup/dfs.c. ReactOS does not support
//! DFS, so every entry point is an UNIMPLEMENTED stub returning STATUS_NOT_IMPLEMENTED;
//! DfsDriverEntry failing is exactly what keeps DFS disabled (see mup::DriverEntry).
//! Called only from within the crate (mup.rs), so these are plain internal fns.
#![allow(non_snake_case, unused_variables)]

use ntkrnl_sys::base::NTSTATUS;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, PUNICODE_STRING};

use crate::glue::STATUS_NOT_IMPLEMENTED;

pub unsafe fn DfsVolumePassThrough(_DeviceObject: PDEVICE_OBJECT, _Irp: PIRP) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

pub unsafe fn DfsFsdFileSystemControl(_DeviceObject: PDEVICE_OBJECT, _Irp: PIRP) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

pub unsafe fn DfsFsdCreate(_DeviceObject: PDEVICE_OBJECT, _Irp: PIRP) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

pub unsafe fn DfsFsdCleanup(_DeviceObject: PDEVICE_OBJECT, _Irp: PIRP) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

pub unsafe fn DfsFsdClose(_DeviceObject: PDEVICE_OBJECT, _Irp: PIRP) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

pub unsafe fn DfsUnload(_DriverObject: PDRIVER_OBJECT) {}

pub unsafe fn DfsDriverEntry(
    _DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    /* We don't support DFS yet, so fail to make sure it remains disabled. */
    STATUS_NOT_IMPLEMENTED
}
