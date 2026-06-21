// power.rs — MouHid_Power, MouHid_SystemControl
// Translated from drivers/hid/mouhid/mouhid.c

use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::{PoCallDriver, PoStartNextPowerIrp, IoCallDriver};
use ntkrnl_sys::inline::tier1::{IoSkipCurrentIrpStackLocation};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT,
};

use crate::types::PMOUHID_DEVICE_EXTENSION;

// ---------------------------------------------------------------------------
// MouHid_Power — IRP_MJ_POWER (pass-through with power sequencing)
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_Power(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
) -> NTSTATUS {
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let ext = &*DeviceExtension;

    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    PoCallDriver(ext.NextDeviceObject, Irp)
}

// ---------------------------------------------------------------------------
// MouHid_SystemControl — IRP_MJ_SYSTEM_CONTROL (pass-through)
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_SystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
) -> NTSTATUS {
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let ext = &*DeviceExtension;

    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver(ext.NextDeviceObject, Irp)
}
