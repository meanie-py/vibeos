//! usbohci glue — cohab shims not in the bindgen types.rs. DbgPrint/RtlAssert and the
//! USBPORT_*/READ_PORT_*/WRITE_PORT_* registrations resolve from the ntoskrnl/hal/usbport
//! import libs (modules declare them in their own extern blocks). Mirrors usbehci glue.
#![allow(unused_imports)]

pub use crate::types::*;

// ---- kernel type aliases the modules name bare but don't import from ddk ----
pub use ntkrnl_sys::ddk::{PCSTR, ULONG64};

// ---- USB_DEVICE_SPEED enum members (bindgen names them `_USB_DEVICE_SPEED_*`) ----
pub use crate::types::_USB_DEVICE_SPEED_UsbLowSpeed as UsbLowSpeed;
pub use crate::types::_USB_DEVICE_SPEED_UsbFullSpeed as UsbFullSpeed;
pub use crate::types::_USB_DEVICE_SPEED_UsbHighSpeed as UsbHighSpeed;

// ---- USBD_STATUS codes (usb.h). USBD_STATUS = LONG. Values verified vs the C header. ----
use ntkrnl_sys::ddk::LONG;
pub const USBD_STATUS_SUCCESS: LONG = 0x00000000u32 as LONG;
pub const USBD_STATUS_HALTED: LONG = 0xC0000000u32 as LONG;
pub const USBD_STATUS_CRC: LONG = 0xC0000001u32 as LONG;
pub const USBD_STATUS_STALL_PID: LONG = 0xC0000004u32 as LONG;
pub const USBD_STATUS_DEV_NOT_RESPONDING: LONG = 0xC0000005u32 as LONG;
pub const USBD_STATUS_DATA_OVERRUN: LONG = 0xC0000008u32 as LONG;
pub const USBD_STATUS_BUFFER_OVERRUN: LONG = 0xC000000Cu32 as LONG;
pub const USBD_STATUS_BUFFER_UNDERRUN: LONG = 0xC000000Du32 as LONG;
pub const USBD_STATUS_XACT_ERROR: LONG = 0xC0000011u32 as LONG;
pub const USBD_STATUS_BABBLE_DETECTED: LONG = 0xC0000012u32 as LONG;
pub const USBD_STATUS_DATA_BUFFER_ERROR: LONG = 0xC0000013u32 as LONG;
pub const USBD_STATUS_INTERNAL_HC_ERROR: LONG = 0x80000800u32 as LONG;
pub const USBD_STATUS_ERROR_SHORT_TRANSFER: LONG = 0x80000900u32 as LONG;

// ---- HAL memory-mapped register access (READ/WRITE_REGISTER_*) — wdm.h __inline volatile
// MMIO loads/stores. Faithful inline (no exported symbol; matches the DDK macros). ----
use ntkrnl_sys::ddk::{UCHAR, ULONG, USHORT};
#[inline] pub unsafe fn READ_REGISTER_ULONG(r: *mut ULONG) -> ULONG { core::ptr::read_volatile(r) }
#[inline] pub unsafe fn WRITE_REGISTER_ULONG(r: *mut ULONG, v: ULONG) { core::ptr::write_volatile(r, v) }
#[inline] pub unsafe fn READ_REGISTER_UCHAR(r: *mut UCHAR) -> UCHAR { core::ptr::read_volatile(r) }
#[inline] pub unsafe fn WRITE_REGISTER_UCHAR(r: *mut UCHAR, v: UCHAR) { core::ptr::write_volatile(r, v) }
#[inline] pub unsafe fn READ_REGISTER_USHORT(r: *mut USHORT) -> USHORT { core::ptr::read_volatile(r) }
#[inline] pub unsafe fn WRITE_REGISTER_USHORT(r: *mut USHORT, v: USHORT) { core::ptr::write_volatile(r, v) }

// ---- KeQuerySystemTime — wdm.h macro reading KI_USER_SHARED_DATA.SystemTime (i386 only;
// ntkrnl-sys provides it for non-x86). ----
#[inline]
pub unsafe fn KeQuerySystemTime(current_time: *mut ntkrnl_sys::ddk::_LARGE_INTEGER) {
    let SharedUserData = ntkrnl_sys::mm::KI_USER_SHARED_DATA as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
    loop {
        let high1 = core::ptr::addr_of!((*SharedUserData).SystemTime.High1Time).read_volatile();
        let low = core::ptr::addr_of!((*SharedUserData).SystemTime.LowPart).read_volatile();
        let high2 = core::ptr::addr_of!((*SharedUserData).SystemTime.High2Time).read_volatile();
        if high1 == high2 {
            (*current_time).QuadPart = ((high1 as i64) << 32) | (low as u32 as i64);
            break;
        }
    }
}

// DPRINT_OHCI — NDEBUG-disabled DbgPrint wrapper (dbg_ohci.h); cdecl no-op link symbol.
#[no_mangle]
unsafe extern "C" fn DPRINT_OHCI(_fmt: *const ::core::ffi::c_char) {}

// KeQueryTickCount — ntoskrnl export (PLARGE_INTEGER out). Body passes *mut i64 (== PLARGE_INTEGER).
#[cfg(target_arch = "x86")]
extern "system" { pub fn KeQueryTickCount(CurrentCount: *mut i64); }
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn KeQueryTickCount(CurrentCount: *mut i64) {
    // KUSER_SHARED_DATA.TickCountQuad lives at offset 0x320 on amd64/arm64.
    let tcq = (ntkrnl_sys::mm::KI_USER_SHARED_DATA + 0x320) as *const i64;
    *CurrentCount = core::ptr::read_volatile(tcq);
}

// RtlSecureZeroMemory — winnt.h FORCEINLINE (volatile memset, not optimized away); no export.
#[no_mangle]
unsafe extern "C" fn RtlSecureZeroMemory(ptr: *mut core::ffi::c_void, cnt: usize) -> *mut core::ffi::c_void {
    let mut p = ptr as *mut u8;
    let mut i = 0usize;
    while i < cnt { core::ptr::write_volatile(p, 0); p = p.add(1); i += 1; }
    ptr
}
