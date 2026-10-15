//! Power setting callbacks (Windows Vista+): PoRegisterPowerSettingCallback and
//! PoUnregisterPowerSettingCallback.
//!
//! Not a translation: ReactOS has no power-setting store, so there is no C to port. A
//! registration is accepted and handed back a real handle, but the callback is never run.
//! Windows runs it once at registration with the setting's current value; with no store
//! there is no value to report, and a driver must already cope with a setting that never
//! changes. Drivers that only want to hear about changes (Realtek's WDI Wi-Fi driver
//! registers for the console display state) work unchanged.

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{GUID, PDEVICE_OBJECT};
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::mm::NonPagedPool;

const TAG_POWER_SETTING: u32 = u32::from_le_bytes(*b"PoSc");

#[repr(C)]
struct PowerSettingRegistration {
    device: PDEVICE_OBJECT,
    setting: GUID,
    callback: PVOID,
    context: PVOID,
}

#[no_mangle]
pub unsafe extern "system" fn PoRegisterPowerSettingCallback(
    DeviceObject: PDEVICE_OBJECT,
    SettingGuid: *const GUID,
    Callback: PVOID,
    Context: PVOID,
    Handle: *mut PVOID,
) -> NTSTATUS {
    if SettingGuid.is_null() || Callback.is_null() || Handle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let reg = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<PowerSettingRegistration>(), TAG_POWER_SETTING)
        as *mut PowerSettingRegistration;
    if reg.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    reg.write(PowerSettingRegistration { device: DeviceObject, setting: *SettingGuid, callback: Callback, context: Context });
    let g = &*SettingGuid;
    DbgPrint(
        b"PoRegisterPowerSettingCallback: {%08lx-%04x-%04x-...} registered; no power-setting store, so it is never called\n\0".as_ptr() as *const i8,
        g.Data1, g.Data2 as u32, g.Data3 as u32,
    );
    core::ptr::write_volatile(Handle, reg as PVOID);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PoUnregisterPowerSettingCallback(Handle: PVOID) -> NTSTATUS {
    if Handle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    ExFreePoolWithTag(Handle, TAG_POWER_SETTING);
    STATUS_SUCCESS
}
