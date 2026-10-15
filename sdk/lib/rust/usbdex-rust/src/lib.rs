//! Rust translation of `sdk/lib/drivers/usbdex/usbdex.c` — the ReactOS
//! "UsbdEx lib" (the USBD handle / URB-allocation helpers declared under the
//! `/* UsbdEx lib */` section of `sdk/include/ddk/usbdlib.h`).
//!
//! The C was `UNIMPLEMENTED` throughout and returned `STATUS_NOT_IMPLEMENTED`, which fails
//! every KMDF USB client created with `WdfUsbTargetDeviceCreateWithParameters` (any Windows
//! 8+ driver: KMDF calls `USBD_CreateHandle` and gives up on the device). On Windows these
//! are thin bookkeeping over the USB stack, so they are implemented here: a handle that
//! records its client, URBs from nonpaged pool, and `AssignUrbToIoStackLocation` placing the
//! URB where `IOCTL_INTERNAL_USB_SUBMIT_URB` carries it. Capabilities answer
//! `STATUS_NOT_SUPPORTED`, as Windows does for one the hub lacks.
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

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC00000BBu32 as NTSTATUS;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as NTSTATUS;
const NON_PAGED_POOL: u32 = 0;

extern "system" {
    fn ExAllocatePoolWithTag(PoolType: u32, NumberOfBytes: usize, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

/// What a USBD_HANDLE points at.
#[repr(C)]
struct UsbdHandle {
    device_object: PVOID,
    target_device_object: PVOID,
    contract_version: ULONG,
    pool_tag: ULONG,
}

/// Every URB this library hands out starts with this header, so `UrbFree` knows the tag.
#[repr(C)]
struct UrbBlock {
    pool_tag: ULONG,
    _pad: [u32; 3],
    // the URB follows, 16-byte aligned
}

/// sizeof(URB) is under 0x100 on every arch; the isoch variant adds 12-byte packets.
const URB_BYTES: usize = 0x100;
const ISO_PACKET_BYTES: usize = 12;

unsafe fn alloc_urb(handle: USBD_HANDLE, bytes: usize, urb: *mut PVOID) -> NTSTATUS {
    if handle.is_null() || urb.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let tag = (*(handle as *mut UsbdHandle)).pool_tag;
    let total = core::mem::size_of::<UrbBlock>() + bytes;
    let blk = ExAllocatePoolWithTag(NON_PAGED_POOL, total, tag) as *mut UrbBlock;
    if blk.is_null() {
        *urb = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(blk as *mut u8, 0, total);
    (*blk).pool_tag = tag;
    *urb = blk.add(1) as PVOID;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn USBD_QueryUsbCapability(
    _USBDHandle: USBD_HANDLE,
    _CapabilityType: *const c_void, // const GUID*
    _OutputBufferLength: ULONG,
    _OutputBuffer: PUCHAR,
    ResultLength: PULONG,
) -> NTSTATUS {
    // Static streams, chained MDLs, selective suspend and the rest: our usbport offers none.
    if !ResultLength.is_null() {
        *ResultLength = 0;
    }
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "C" fn USBD_AssignUrbToIoStackLocation(
    _USBDHandle: USBD_HANDLE,
    IoStackLocation: PVOID, // PIO_STACK_LOCATION
    Urb: PVOID,             // PURB
) {
    // IO_STACK_LOCATION: four UCHARs, then the Parameters union at pointer alignment;
    // Parameters.Others.Argument1 is its first pointer, where SUBMIT_URB reads the URB.
    let argument1 = (IoStackLocation as *mut u8).add(core::mem::size_of::<usize>()) as *mut PVOID;
    *argument1 = Urb;
}

#[no_mangle]
pub unsafe extern "C" fn USBD_CreateHandle(
    DeviceObject: PVOID,       // PDEVICE_OBJECT
    TargetDeviceObject: PVOID, // PDEVICE_OBJECT
    USBDClientContractVersion: ULONG,
    PoolTag: ULONG,
    USBDHandle: *mut USBD_HANDLE,
) -> NTSTATUS {
    if USBDHandle.is_null() || TargetDeviceObject.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let h = ExAllocatePoolWithTag(NON_PAGED_POOL, core::mem::size_of::<UsbdHandle>(), PoolTag)
        as *mut UsbdHandle;
    if h.is_null() {
        *USBDHandle = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    h.write(UsbdHandle {
        device_object: DeviceObject,
        target_device_object: TargetDeviceObject,
        contract_version: USBDClientContractVersion,
        pool_tag: PoolTag,
    });
    *USBDHandle = h as USBD_HANDLE;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn USBD_CloseHandle(USBDHandle: USBD_HANDLE) {
    if !USBDHandle.is_null() {
        ExFreePoolWithTag(USBDHandle, (*(USBDHandle as *mut UsbdHandle)).pool_tag);
    }
}

#[no_mangle]
pub unsafe extern "C" fn USBD_UrbAllocate(
    USBDHandle: USBD_HANDLE,
    Urb: *mut PVOID, // PURB*
) -> NTSTATUS {
    alloc_urb(USBDHandle, URB_BYTES, Urb)
}

#[no_mangle]
pub unsafe extern "C" fn USBD_IsochUrbAllocate(
    USBDHandle: USBD_HANDLE,
    NumberOfIsochPacket: ULONG,
    Urb: *mut PVOID, // PURB*
) -> NTSTATUS {
    alloc_urb(USBDHandle, URB_BYTES + NumberOfIsochPacket as usize * ISO_PACKET_BYTES, Urb)
}

#[no_mangle]
pub unsafe extern "C" fn USBD_UrbFree(
    _USBDHandle: USBD_HANDLE,
    Urb: PVOID, // PURB
) {
    if !Urb.is_null() {
        let blk = (Urb as *mut UrbBlock).sub(1);
        ExFreePoolWithTag(blk as PVOID, (*blk).pool_tag);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
