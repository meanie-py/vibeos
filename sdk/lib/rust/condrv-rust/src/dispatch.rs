//! Rust translation of `drivers/base/condrv/dispatch.c` — the (skeleton)
//! dispatch interface. The C only DPRINTs each major function (no-op under
//! NDEBUG) and completes every IRP with STATUS_SUCCESS.

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP};

pub unsafe fn CompleteRequest(Irp: PIRP, Status: NTSTATUS, Information: ULONG_PTR) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = Information as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    Status
}

pub unsafe extern "system" fn ConDrvDispatch(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    /* The C just traced every IRP major function (debug-only) and completed it. */
    CompleteRequest(Irp, STATUS_SUCCESS, 0)
}
