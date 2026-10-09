//! usbuhci glue — cohab shims not in the bindgen types.rs. DbgPrint/RtlAssert and the
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

// ---- KeQuerySystemTime — wdm.h macro reading KI_USER_SHARED_DATA.SystemTime (i386 only). ----
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

// DPRINT_IMPL / DPRINT_UHCI — NDEBUG-disabled DbgPrint wrappers (dbg_uhci.h); cdecl no-op symbols.
// DPRINT_IMPL def relocated to drvtrace-shim (§9.99 fold-proofing)
// DPRINT_UHCI def relocated to drvtrace-shim (§9.99 fold-proofing)

// ---- uhci-specific shims ----
pub const MAXUSHORT: ::core::ffi::c_int = 0xFFFF;
#[inline] pub unsafe fn SharedSystemTimePtr() -> *mut ULONG64 {
    let SharedUserData = ntkrnl_sys::mm::KI_USER_SHARED_DATA as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
    core::ptr::addr_of_mut!((*SharedUserData).SystemTime) as *mut ULONG64
}
// InterlockedExchange — atomic swap intrinsic (no ntoskrnl export); inline, no symbol.
#[inline] pub unsafe fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    (*(target as *const core::sync::atomic::AtomicI32)).swap(value, core::sync::atomic::Ordering::SeqCst)
}

// ---- USB_CONTROLLER_FLAVOR enum members (bindgen names them `_USB_CONTROLLER_FLAVOR_*`) ----
pub use crate::types::_USB_CONTROLLER_FLAVOR_UHCI_Piix4 as UHCI_Piix4;
pub use crate::types::_USB_CONTROLLER_FLAVOR_UHCI_VIA as UHCI_VIA;
pub use crate::types::_USB_CONTROLLER_FLAVOR_UHCI_VIA_x01 as UHCI_VIA_x01;
pub use crate::types::_USB_CONTROLLER_FLAVOR_UHCI_VIA_x02 as UHCI_VIA_x02;
pub use crate::types::_USB_CONTROLLER_FLAVOR_UHCI_VIA_x03 as UHCI_VIA_x03;
pub use crate::types::_USB_CONTROLLER_FLAVOR_UHCI_VIA_x04 as UHCI_VIA_x04;

// READ/WRITE_PORT_* — real HAL exports on i386; on amd64 they are inline x86 port I/O;
// on arm64 UHCI (x86-era) is inert, so a 0/no-op stub satisfies the link.
#[cfg(target_arch = "x86_64")]
mod uhci_port_io {
    #[no_mangle] unsafe extern "system" fn READ_PORT_UCHAR(p: *mut u8) -> u8 {
        let v: u8; core::arch::asm!("in al, dx", out("al") v, in("dx") p as u16, options(nomem, nostack, preserves_flags)); v }
    #[no_mangle] unsafe extern "system" fn READ_PORT_USHORT(p: *mut u16) -> u16 {
        let v: u16; core::arch::asm!("in ax, dx", out("ax") v, in("dx") p as u16, options(nomem, nostack, preserves_flags)); v }
    #[no_mangle] unsafe extern "system" fn READ_PORT_ULONG(p: *mut u32) -> u32 {
        let v: u32; core::arch::asm!("in eax, dx", out("eax") v, in("dx") p as u16, options(nomem, nostack, preserves_flags)); v }
    #[no_mangle] unsafe extern "system" fn WRITE_PORT_UCHAR(p: *mut u8, v: u8) {
        core::arch::asm!("out dx, al", in("dx") p as u16, in("al") v, options(nomem, nostack, preserves_flags)); }
    #[no_mangle] unsafe extern "system" fn WRITE_PORT_USHORT(p: *mut u16, v: u16) {
        core::arch::asm!("out dx, ax", in("dx") p as u16, in("ax") v, options(nomem, nostack, preserves_flags)); }
    #[no_mangle] unsafe extern "system" fn WRITE_PORT_ULONG(p: *mut u32, v: u32) {
        core::arch::asm!("out dx, eax", in("dx") p as u16, in("eax") v, options(nomem, nostack, preserves_flags)); }
}
#[cfg(target_arch = "aarch64")]
mod uhci_port_io {
    #[no_mangle] unsafe extern "system" fn READ_PORT_UCHAR(_p: *mut u8) -> u8 { 0 }
    #[no_mangle] unsafe extern "system" fn READ_PORT_USHORT(_p: *mut u16) -> u16 { 0 }
    #[no_mangle] unsafe extern "system" fn READ_PORT_ULONG(_p: *mut u32) -> u32 { 0 }
    #[no_mangle] unsafe extern "system" fn WRITE_PORT_UCHAR(_p: *mut u8, _v: u8) {}
    #[no_mangle] unsafe extern "system" fn WRITE_PORT_USHORT(_p: *mut u16, _v: u16) {}
    #[no_mangle] unsafe extern "system" fn WRITE_PORT_ULONG(_p: *mut u32, _v: u32) {}
}
