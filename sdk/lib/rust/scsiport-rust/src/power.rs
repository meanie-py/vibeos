#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{SCSI_PORT_COMMON_EXTENSION};

#[no_mangle]
pub unsafe extern "system" fn ScsiPortDispatchPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let comExt = (*DeviceObject).DeviceExtension as *mut SCSI_PORT_COMMON_EXTENSION;
    if (*comExt).IsFDO != 0 {
        PoStartNextPowerIrp(Irp);
        IoSkipCurrentIrpStackLocation(Irp);
        return PoCallDriver((*comExt).LowerDevice, Irp);
    } else {
        let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
        match (*ioStack).MinorFunction as i32 {
            (IRP_MN_SET_POWER) as i32 | (IRP_MN_QUERY_POWER) as i32 => {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            }
            _ => {}
        }
        let status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;
        PoStartNextPowerIrp(Irp);
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return status;
    }
}
