// power.rs — KbdHid_Power, KbdHid_SystemControl.
// Translated from drivers/hid/kbdhid/kbdhid.c

use ntkrnl_sys::base::NTSTATUS;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP};
use ntkrnl_sys::driver_ddk::{IoCallDriver, PoCallDriver, PoStartNextPowerIrp};
use ntkrnl_sys::inline::tier1::{IoSkipCurrentIrpStackLocation};
use crate::types::PKBDHID_DEVICE_EXTENSION;

/// KbdHid_Power — IRP_MJ_POWER: start next power IRP, skip location, PoCallDriver.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_Power(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    PoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
}

/// KbdHid_SystemControl — IRP_MJ_SYSTEM_CONTROL: skip and forward.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_SystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
}
