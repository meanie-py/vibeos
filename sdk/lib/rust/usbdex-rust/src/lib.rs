//! Rust translation of `sdk/lib/drivers/usbdex/usbdex.c` — the ReactOS
//! "UsbdEx lib" (the USBD handle / URB-allocation helpers declared under the
//! `/* UsbdEx lib */` section of `sdk/include/ddk/usbdlib.h`).
//!
//! Faithful port of the C: every function is `UNIMPLEMENTED` and returns
//! `STATUS_NOT_IMPLEMENTED` (the `NTSTATUS` ones) or nothing (the `VOID` ones),
//! exactly as the C did. No state, no allocation, no DDK calls — so the crate
//! is fully self-contained (no ntkrnl-sys dependency).
//!
//! ABI: the usbdlib.h prototypes carry NO `NTAPI` — they are plain **cdecl**
//! on every arch (unlike the NTAPI cohab crates which use `extern "system"`).
//! `#[no_mangle] extern "C"` gives the C cdecl symbol (`_USBD_*` on i386 PE,
//! `USBD_*` on amd64/arm64), matching the C object these replace.
//!
//! crate-type = staticlib → `libusbdex_rust.a`, linked PUBLIC into the
//! `usbdex` cmake target (cohabitation, usbdex.c kept on disk per the
//! never-delete rule).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

use core::ffi::c_void;

type NTSTATUS = i32;
type ULONG = u32;
type PVOID = *mut c_void;
type PUCHAR = *mut u8;
type PULONG = *mut u32;
type USBD_HANDLE = PVOID; // DECLARE_HANDLE(USBD_HANDLE)

// STATUS_NOT_IMPLEMENTED (ntstatus.h)
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "C" fn USBD_QueryUsbCapability(
    _USBDHandle: USBD_HANDLE,
    _CapabilityType: *const c_void, // const GUID*
    _OutputBufferLength: ULONG,
    _OutputBuffer: PUCHAR,
    _ResultLength: PULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "C" fn USBD_AssignUrbToIoStackLocation(
    _USBDHandle: USBD_HANDLE,
    _IoStackLocation: PVOID, // PIO_STACK_LOCATION
    _Urb: PVOID,             // PURB
) {
}

#[no_mangle]
pub unsafe extern "C" fn USBD_CreateHandle(
    _DeviceObject: PVOID,       // PDEVICE_OBJECT
    _TargetDeviceObject: PVOID, // PDEVICE_OBJECT
    _USBDClientContractVersion: ULONG,
    _PoolTag: ULONG,
    _USBDHandle: *mut USBD_HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "C" fn USBD_CloseHandle(_USBDHandle: USBD_HANDLE) {}

#[no_mangle]
pub unsafe extern "C" fn USBD_UrbAllocate(
    _USBDHandle: USBD_HANDLE,
    _Urb: *mut PVOID, // PURB*
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "C" fn USBD_IsochUrbAllocate(
    _USBDHandle: USBD_HANDLE,
    _NumberOfIsochPacket: ULONG,
    _Urb: *mut PVOID, // PURB*
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "C" fn USBD_UrbFree(
    _USBDHandle: USBD_HANDLE,
    _Urb: PVOID, // PURB
) {
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
