//! fsctrl.rs — fsctrl.c: MsfsFileSystemControl. msfs implements no FSCTLs, so every code
//! falls through to STATUS_NOT_IMPLEMENTED.
#![allow(non_snake_case)]

use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP, NTSTATUS};
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::glue::*;
use crate::types::PMSFS_FCB;

pub unsafe extern "system" fn MsfsFileSystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject;
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let FileObject = (*IoStack).FileObject;
    let _Fcb = (*FileObject).FsContext as PMSFS_FCB;

    let Status = match (*IoStack).Parameters.FileSystemControl.FsControlCode {
        _ => STATUS_NOT_IMPLEMENTED,
    };

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    Status
}
