// Shared types for processr-rust, translated from processr.h. The grinder left
// DEVICE_EXTENSION undefined (it's a driver-private struct in processr.h, not a
// DDK type) — define it once here and import into all three modules.
use ntkrnl_sys::ddk::PDEVICE_OBJECT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _DEVICE_EXTENSION {
    pub DeviceObject: PDEVICE_OBJECT,
    pub LowerDevice: PDEVICE_OBJECT,
}
pub type DEVICE_EXTENSION = _DEVICE_EXTENSION;
pub type PDEVICE_EXTENSION = *mut _DEVICE_EXTENSION;
