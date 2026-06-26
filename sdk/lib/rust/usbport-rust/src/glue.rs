//! usbport glue — cohab shims not in the bindgen types.rs. DbgPrint/RtlAssert and the
//! kernel/HAL imports resolve from the ntoskrnl/hal import libs (modules declare them in
//! their own extern blocks). Mirrors the usbehci/ohci/uhci miniport glue.
#![allow(unused_imports)]

pub use crate::types::*;

// ---- kernel type aliases the modules name bare but don't import from ddk ----
pub use ntkrnl_sys::ddk::PCSTR;
use ntkrnl_sys::base::PVOID;
use ntkrnl_sys::base::ULONG;
pub use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyDriverKeyName as DevicePropertyDriverKeyName;
pub use ntkrnl_sys::ddk::{
    ACCESS_MASK, CCHAR, GUID, IO_ALLOCATION_ACTION, KEY_VALUE_INFORMATION_CLASS, KPRIORITY,
    KPROCESSOR_MODE, PRKTHREAD, _KSEMAPHORE, _LIST_ENTRY, _UNICODE_STRING, wchar_t,
};
pub use crate::types::_USB_PORT_STATUS_AND_CHANGE as USB_PORT_STATUS_AND_CHANGE;

// ---- misc consts ----
pub const USB_PORT_TAG: u32 = 0x70627375; // 'pbsu' (usbport.h:44)
pub const EVENT_INCREMENT: i8 = 1; // ntddk.h

// ---- USB_DEVICE_SPEED / USB_DEVICE_TYPE enum members (crate bindgen names them `_ENUM_*`) ----
pub use crate::types::_USB_DEVICE_SPEED_UsbLowSpeed as UsbLowSpeed;
pub use crate::types::_USB_DEVICE_SPEED_UsbFullSpeed as UsbFullSpeed;
pub use crate::types::_USB_DEVICE_SPEED_UsbHighSpeed as UsbHighSpeed;
pub use crate::types::_USB_DEVICE_TYPE_Usb11Device as Usb11Device;
pub use crate::types::_USB_DEVICE_TYPE_Usb20Device as Usb20Device;

// ---- kernel enum members imported by bare name (bindgen names them `_ENUM_Member`) ----
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0;
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD1 as PowerDeviceD1;
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD2 as PowerDeviceD2;
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemUnspecified as PowerSystemUnspecified;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemSleeping1 as PowerSystemSleeping1;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemSleeping2 as PowerSystemSleeping2;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemSleeping3 as PowerSystemSleeping3;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemHibernate as PowerSystemHibernate;
pub use ntkrnl_sys::ddk::_POWER_STATE_TYPE_DevicePowerState as DevicePowerState;
pub use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_CriticalWorkQueue as CriticalWorkQueue;
pub use ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation as KeyValueFullInformation;
pub use ntkrnl_sys::ddk::_MM_PAGE_PRIORITY_NormalPagePriority as NormalPagePriority;
pub use ntkrnl_sys::ddk::_IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters as DeallocateObjectKeepRegisters;

// ---- USBD_STATUS codes (usb.h). USBD_STATUS = LONG. Values verified vs the C header. ----
use ntkrnl_sys::ddk::LONG;
pub const USBD_STATUS_SUCCESS: LONG = 0x00000000u32 as LONG;
pub const USBD_STATUS_HALTED: LONG = 0xC0000000u32 as LONG;
pub const USBD_STATUS_STALL_PID: LONG = 0xC0000004u32 as LONG;
pub const USBD_STATUS_DEVICE_GONE: LONG = 0xC0000005u32 as LONG;
pub const USBD_STATUS_NOT_SUPPORTED: LONG = 0xC0000E00u32 as LONG;
pub const USBD_STATUS_INVALID_URB_FUNCTION: LONG = 0x80000200u32 as LONG;
pub const USBD_STATUS_INVALID_PARAMETER: LONG = 0x80000300u32 as LONG;
pub const USBD_STATUS_INVALID_PIPE_HANDLE: LONG = 0x80000600u32 as LONG;
pub const USBD_STATUS_INVALID_PIPE_FLAGS: LONG = 0x80000700u32 as LONG;
pub const USBD_STATUS_INSUFFICIENT_RESOURCES: LONG = 0x80000C00u32 as LONG;
pub const USBD_STATUS_INTERFACE_NOT_FOUND: LONG = 0x80000A00u32 as LONG;
pub const USBD_STATUS_DEVICE_NOT_RESPONDING: LONG = 0xC0000005u32 as LONG;

// ---- DPRINT family — NDEBUG-disabled DbgPrint wrappers (usbdebug.h). Variadic decls put the
// name in scope for modules that don't declare it themselves; the no-op symbols satisfy the
// link (cdecl, caller cleans varargs). ----
extern "C" {
    pub fn DPRINT(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_CORE(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_URB(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_QUEUE(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_USB(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_TIMER(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_INT(fmt: *const ::core::ffi::c_char, ...);
    pub fn DPRINT_MINIPORT(fmt: *const ::core::ffi::c_char, ...);
}
mod dprint_syms {
    macro_rules! noop { ($($n:ident),*) => { $(
        #[no_mangle] unsafe extern "C" fn $n(_fmt: *const ::core::ffi::c_char) {}
    )* } }
    noop!(DPRINT, DPRINT_CORE, DPRINT_URB, DPRINT_QUEUE, DPRINT_USB, DPRINT_TIMER, DPRINT_INT, DPRINT_MINIPORT);
}

// ASSERT — NT_ASSERT, no-op in retail (NDEBUG).
#[inline] pub fn ASSERT(_cond: i32) {}

// ---- KeQuerySystemTime / SharedSystemTime — read KI_USER_SHARED_DATA.SystemTime ----
#[inline]
pub unsafe fn KeQuerySystemTime(current_time: *mut ntkrnl_sys::ddk::_LARGE_INTEGER) {
    let shared = ntkrnl_sys::mm::KI_USER_SHARED_DATA as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
    loop {
        let high1 = core::ptr::addr_of!((*shared).SystemTime.High1Time).read_volatile();
        let low = core::ptr::addr_of!((*shared).SystemTime.LowPart).read_volatile();
        let high2 = core::ptr::addr_of!((*shared).SystemTime.High2Time).read_volatile();
        if high1 == high2 {
            (*current_time).QuadPart = ((high1 as i64) << 32) | (low as u32 as i64);
            break;
        }
    }
}
#[inline]
pub unsafe fn SharedSystemTimePtr() -> *mut ntkrnl_sys::ddk::ULONG64 {
    let shared = ntkrnl_sys::mm::KI_USER_SHARED_DATA as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
    core::ptr::addr_of_mut!((*shared).SystemTime) as *mut ntkrnl_sys::ddk::ULONG64
}

// MmGetMdlVirtualAddress — wdm.h FORCEINLINE: MDL->StartVa + MDL->ByteOffset.
#[inline]
pub unsafe fn MmGetMdlVirtualAddress(Mdl: *mut ntkrnl_sys::ddk::_MDL) -> *mut core::ffi::c_void {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as *mut core::ffi::c_void
}

// ---- IRQL (i386: HAL export; amd64: CR8 FORCEINLINE) — re-export ntkrnl-sys inline ----
pub use ntkrnl_sys::inline::tier1::{KeRaiseIrql, KeLowerIrql};

// ---- BM_REQUEST_TYPE — bindgen collapsed the bitfield struct to just `B: UCHAR`; supply the
// USB-spec bmRequestType bitfield accessors (Recipient[0:4], Type[5:6], Dir[7]) on the byte. ----
impl crate::types::_BM_REQUEST_TYPE {
    #[inline] pub fn Recipient(&self) -> ntkrnl_sys::ddk::UCHAR { unsafe { self.B & 0x1F } }
    #[inline] pub fn set_Recipient(&mut self, v: ntkrnl_sys::ddk::UCHAR) { unsafe { self.B = (self.B & !0x1F) | (v & 0x1F); } }
    #[inline] pub fn Type(&self) -> ntkrnl_sys::ddk::UCHAR { unsafe { (self.B >> 5) & 0x3 } }
    #[inline] pub fn set_Type(&mut self, v: ntkrnl_sys::ddk::UCHAR) { unsafe { self.B = (self.B & !0x60) | ((v & 0x3) << 5); } }
    #[inline] pub fn Dir(&self) -> ntkrnl_sys::ddk::UCHAR { unsafe { (self.B >> 7) & 0x1 } }
    #[inline] pub fn set_Dir(&mut self, v: ntkrnl_sys::ddk::UCHAR) { unsafe { self.B = (self.B & !0x80) | ((v & 0x1) << 7); } }
    #[inline] pub fn Reserved(&self) -> ntkrnl_sys::ddk::UCHAR { 0 }
    #[inline] pub fn set_Reserved(&mut self, _v: ntkrnl_sys::ddk::UCHAR) {}
}

// ---- USBPORT_MPStatusToRHStatus (roothub.c:18) — dropped by the pure-LLM pass; faithful re-add. ----
pub const RH_STATUS_SUCCESS: ntkrnl_sys::ddk::ULONG = 0;
#[no_mangle]
pub unsafe extern "system" fn USBPORT_MPStatusToRHStatus(MPStatus: MPSTATUS) -> RHSTATUS {
    let mut rh: RHSTATUS = RH_STATUS_SUCCESS;
    if MPStatus != 0 {
        rh = (MPStatus != MP_STATUS_FAILURE) as RHSTATUS;
        rh += 1;
    }
    rh
}

// ---- misc consts / inline helpers ----
pub const USBD_STATUS_BUFFER_TOO_SMALL: LONG = 0xC0003000u32 as LONG;
pub const BUGCODE_USB_DRIVER: u32 = 0xFE; // bugcheck.h
#[inline] pub fn NT_ERROR(status: ntkrnl_sys::ddk::NTSTATUS) -> bool { ((status as u32) >> 30) == 3 }
#[inline] pub unsafe fn __int2c() {} // __debugbreak-ish; no-op in retail
#[inline] pub unsafe fn wcslen(mut s: *const u16) -> usize { let mut n = 0; while *s != 0 { s = s.add(1); n += 1; } n }
#[inline] pub unsafe fn InterlockedCompareExchange(dest: *mut LONG, exch: LONG, comp: LONG) -> LONG {
    let a = &*(dest as *const core::sync::atomic::AtomicI32);
    match a.compare_exchange(comp, exch, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) {
        Ok(v) | Err(v) => v,
    }
}

// ---- usbport: types/consts bindgen inlined or omitted (faithful re-decls) ----
pub const USBD_STATUS_BAD_START_FRAME: USBD_STATUS = 0xC0000A00u32 as USBD_STATUS;

// ASYNC_TIMER_CALLBACK: usbport.h `typedef VOID ASYNC_TIMER_CALLBACK(PVOID, PVOID)` —
// bindgen inlined the field type, so the named alias is absent. Matches the
// _USBPORT_ASYNC_CALLBACK_DATA.CallbackFunction slot Option<fn(PVOID,PVOID)>.
pub type ASYNC_TIMER_CALLBACK = unsafe extern "system" fn(arg1: PVOID, arg2: PVOID);

// USB_IDLE_CALLBACK_INFO: usbiodef.h — usbiodef.h not in the bindgen allowlist, re-declared.
pub type USB_IDLE_CALLBACK = ::core::option::Option<unsafe extern "system" fn(Context: PVOID)>;
#[repr(C)]
pub struct USB_IDLE_CALLBACK_INFO {
    pub IdleCallback: USB_IDLE_CALLBACK,
    pub IdleContext: PVOID,
}
pub type PUSB_IDLE_CALLBACK_INFO = *mut USB_IDLE_CALLBACK_INFO;

// USB_HUB_STATUS_AND_CHANGE: un-prefixed alias for the bindgen `_`-prefixed union.
pub use crate::types::_USB_HUB_STATUS_AND_CHANGE as USB_HUB_STATUS_AND_CHANGE;

// ---- USB user-mode IOCTL types/consts (usbuser.h + usbioctl.h; not in bindgen allowlist) ----
pub type USB_USER_ERROR_CODE = i32;
pub const UsbUserSuccess: USB_USER_ERROR_CODE = 0;
pub const UsbUserNotSupported: USB_USER_ERROR_CODE = 1;
pub const UsbUserInvalidRequestCode: USB_USER_ERROR_CODE = 2;
pub const UsbUserFeatureDisabled: USB_USER_ERROR_CODE = 3;
pub const UsbUserInvalidHeaderParameter: USB_USER_ERROR_CODE = 4;
pub const UsbUserInvalidParameter: USB_USER_ERROR_CODE = 5;
pub const UsbUserMiniportError: USB_USER_ERROR_CODE = 6;
pub const UsbUserBufferTooSmall: USB_USER_ERROR_CODE = 7;
pub const UsbUserErrorNotMapped: USB_USER_ERROR_CODE = 8;
pub const UsbUserDeviceNotStarted: USB_USER_ERROR_CODE = 9;
pub const UsbUserNoDeviceConnected: USB_USER_ERROR_CODE = 10;

#[repr(C)]
pub struct USBUSER_REQUEST_HEADER {
    pub UsbUserRequest: ULONG,
    pub UsbUserStatusCode: USB_USER_ERROR_CODE,
    pub RequestBufferLength: ULONG,
    pub ActualBufferLength: ULONG,
}
pub type PUSBUSER_REQUEST_HEADER = *mut USBUSER_REQUEST_HEADER;

#[repr(C)]
pub struct USB_UNICODE_NAME {
    pub Length: ULONG,
    pub String: [u16; 1],
}
pub type PUSB_UNICODE_NAME = *mut USB_UNICODE_NAME;

#[repr(C)]
pub struct USBUSER_CONTROLLER_UNICODE_NAME {
    pub Header: USBUSER_REQUEST_HEADER,
    pub UnicodeName: USB_UNICODE_NAME,
}
pub type PUSBUSER_CONTROLLER_UNICODE_NAME = *mut USBUSER_CONTROLLER_UNICODE_NAME;

#[repr(C)]
pub struct USB_HCD_DRIVERKEY_NAME {
    pub ActualLength: ULONG,
    pub DriverKeyName: [u16; 1],
}
pub type PUSB_HCD_DRIVERKEY_NAME = *mut USB_HCD_DRIVERKEY_NAME;

pub const USBUSER_GET_CONTROLLER_DRIVER_KEY: ULONG = 0x00000002;
pub const USBUSER_GET_ROOTHUB_SYMBOLIC_NAME: ULONG = 0x00000007;
pub const IOCTL_GET_HCD_DRIVERKEY_NAME: ULONG = 0x00220424;

pub const USBD_STATUS_CANCELED: USBD_STATUS = 0xC0010000u32 as USBD_STATUS;
pub const USBD_STATUS_NO_BANDWIDTH: USBD_STATUS = 0x80000700u32 as USBD_STATUS;
pub const USBD_STATUS_TIMEOUT: USBD_STATUS = 0xC0006000u32 as USBD_STATUS;
pub const USBD_STATUS_PENDING: USBD_STATUS = 0x40000000u32 as USBD_STATUS;

// usb100.h direction macros (object-like, not bindgen-emitted)
pub const USB_ENDPOINT_DIRECTION_MASK: u8 = 0x80;
#[inline] pub unsafe fn USB_ENDPOINT_DIRECTION_OUT(x: u8) -> u32 { ((x & USB_ENDPOINT_DIRECTION_MASK) == 0) as u32 }
#[inline] pub unsafe fn USB_ENDPOINT_DIRECTION_IN(x: u8) -> u32 { ((x & USB_ENDPOINT_DIRECTION_MASK) != 0) as u32 }
