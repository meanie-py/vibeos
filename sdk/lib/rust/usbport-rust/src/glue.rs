//! usbport glue — cohab shims not in the bindgen types.rs. DbgPrint/RtlAssert and the
//! kernel/HAL imports resolve from the ntoskrnl/hal import libs (modules declare them in
//! their own extern blocks). Mirrors the usbehci/ohci/uhci miniport glue.
#![allow(unused_imports)]

pub use crate::types::*;

// ---- kernel type aliases the modules name bare but don't import from ddk ----
pub use ntkrnl_sys::ddk::PCSTR;
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
pub const MP_STATUS_FAILURE: ntkrnl_sys::ddk::ULONG = 1;
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
