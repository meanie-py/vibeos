//! null-rust — Rust translation of `drivers/base/null/null.c`, the Null Device
//! Driver (`\Device\Null`). Produces `null.sys`; the final link is driven by
//! `null-cargo` (reactos-link `ModuleKind::KernelDll`, entry = `DriverEntry`,
//! no exports — a kernelmodedriver is a kerneldll whose PE entry point is
//! `DriverEntry@8`). The body is arch-generic, exactly like the C, so it
//! builds for i386 + amd64 + arm64.
//!
//! ABI note: NTAPI == `extern "system"` (stdcall on i386, cdecl on
//! amd64/arm64). The IRP MajorFunction / FastIo / Unload callback slots are
//! bindgen `Option<extern "stdcall" fn>` on i386 but `Option<extern "C" fn>`
//! on amd64/arm64; assigning our `extern "system"` fn items into them is
//! ABI-correct on every arch but the *Rust type strings* differ per arch, so
//! the assignments go through a size-checked `transmute` (the fn pointer and
//! the `Option<fn>` niche share a representation).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_unsafe, dead_code, unused_variables)]

use core::mem::{size_of, transmute, MaybeUninit};
use ntkrnl_sys::ddk::*;

// STATUS_SUCCESS isn't in the constants module; the error codes are.
const STATUS_SUCCESS: NTSTATUS = 0;
use ntkrnl_sys::constants::{STATUS_END_OF_FILE, STATUS_INVALID_INFO_CLASS};

// IRP major function codes + device flags — xdk #define macros, not bindgen.
const IRP_MJ_CREATE: UCHAR = 0x00;
const IRP_MJ_CLOSE: UCHAR = 0x02;
const IRP_MJ_READ: UCHAR = 0x03;
const IRP_MJ_WRITE: UCHAR = 0x04;
const IRP_MJ_QUERY_INFORMATION: UCHAR = 0x05;
const IRP_MJ_LOCK_CONTROL: UCHAR = 0x11;
const FO_SYNCHRONOUS_IO: ULONG = 0x0000_0002;
const FILE_DEVICE_NULL: ULONG = 0x0000_0015;
const FILE_DEVICE_SECURE_OPEN: ULONG = 0x0000_0100;
const IO_NO_INCREMENT: CCHAR = 0;
const FileStandardInformation: FILE_INFORMATION_CLASS =
    _FILE_INFORMATION_CLASS_FileStandardInformation;

// NTAPI imports from ntoskrnl (resolved via the ntoskrnl import lib).
extern "system" {
    fn IoCreateDevice(
        DriverObject: *mut _DRIVER_OBJECT,
        DeviceExtensionSize: ULONG,
        DeviceName: *mut _UNICODE_STRING,
        DeviceType: ULONG,
        DeviceCharacteristics: ULONG,
        Exclusive: BOOLEAN,
        DeviceObject: *mut *mut _DEVICE_OBJECT,
    ) -> NTSTATUS;
    fn IoDeleteDevice(DeviceObject: *mut _DEVICE_OBJECT);
    fn IoCompleteRequest(Irp: *mut _IRP, PriorityBoost: CCHAR);
    fn MmPageEntireDriver(AddressWithinSection: PVOID) -> PVOID;
}

/// FORCEINLINE `IoGetCurrentIrpStackLocation` — reads
/// `Irp->Tail.Overlay.CurrentStackLocation` (bindgen anon-union path).
#[inline(always)]
unsafe fn IoGetCurrentIrpStackLocation(Irp: *mut _IRP) -> *mut _IO_STACK_LOCATION {
    (*Irp)
        .Tail
        .Overlay
        .__bindgen_anon_2
        .__bindgen_anon_1
        .CurrentStackLocation
}

#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

/// Global fast-I/O dispatch table. Zero-initialised at load (the C global lives
/// in BSS, then DriverEntry RtlZeroMemory's it again + fills two slots).
static mut FAST_IO_DISPATCH: MaybeUninit<_FAST_IO_DISPATCH> = MaybeUninit::zeroed();

unsafe extern "system" fn NullQueryFileInformation(
    Buffer: PVOID,
    Length: PULONG,
    InformationClass: FILE_INFORMATION_CLASS,
) -> NTSTATUS {
    let StandardInfo = Buffer as *mut _FILE_STANDARD_INFORMATION;

    // PAGED_CODE();

    // We only support one class.
    if InformationClass != FileStandardInformation {
        return STATUS_INVALID_INFO_CLASS;
    }

    // Fill out the information.
    core::ptr::write_bytes(
        StandardInfo as *mut u8,
        0,
        size_of::<_FILE_STANDARD_INFORMATION>(),
    );
    (*StandardInfo).NumberOfLinks = 1;

    // Return the length and success.
    core::ptr::write_volatile(Length, size_of::<_FILE_STANDARD_INFORMATION>() as ULONG);
    STATUS_SUCCESS
}

unsafe extern "system" fn NullRead(
    FileObject: *mut _FILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    Buffer: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: *mut _DEVICE_OBJECT,
) -> BOOLEAN {
    // PAGED_CODE();

    // Complete successfully (read at EOF).
    (*IoStatus).__bindgen_anon_1.Status = STATUS_END_OF_FILE;
    (*IoStatus).Information = 0;
    1 as BOOLEAN
}

unsafe extern "system" fn NullWrite(
    FileObject: *mut _FILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    Buffer: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: *mut _DEVICE_OBJECT,
) -> BOOLEAN {
    // PAGED_CODE();

    // Complete successfully (swallow the whole write).
    (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*IoStatus).Information = Length as ULONG_PTR;
    1 as BOOLEAN
}

unsafe extern "system" fn NullDispatch(
    DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // PAGED_CODE();

    // Get the file object and check what kind of request this is.
    let FileObject = (*IoStack).FileObject;
    match (*IoStack).MajorFunction {
        IRP_MJ_CREATE | IRP_MJ_CLOSE => {
            // Check if this is synch I/O.
            if ((*FileObject).Flags & FO_SYNCHRONOUS_IO) != 0 {
                // Set distinguished value for Cc.
                (*FileObject).PrivateCacheMap = 1usize as PVOID;
            }

            // Complete successfully.
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information = 0;
        }
        IRP_MJ_READ => {
            // Return as if we read the entire file.
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_END_OF_FILE;
            (*Irp).IoStatus.Information = 0;
        }
        IRP_MJ_WRITE => {
            // Return as if we wrote the entire request.
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information =
                (*IoStack).Parameters.Write.Length as ULONG_PTR;
        }
        IRP_MJ_LOCK_CONTROL => {
            // Dummy.
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information = 0;
        }
        IRP_MJ_QUERY_INFORMATION => {
            // Get the length inputted and do the request.
            let mut Length = (*IoStack).Parameters.QueryFile.Length;
            (*Irp).IoStatus.__bindgen_anon_1.Status = NullQueryFileInformation(
                (*Irp).AssociatedIrp.SystemBuffer,
                &mut Length,
                (*IoStack).Parameters.QueryFile.FileInformationClass,
            );

            // Return the actual length.
            (*Irp).IoStatus.Information = Length as ULONG_PTR;
        }
        _ => {}
    }

    // Complete the request.
    let Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}

unsafe extern "system" fn NullUnload(DriverObject: *mut _DRIVER_OBJECT) {
    let DeviceObject = (*DriverObject).DeviceObject;

    // Delete the Null device.
    IoDeleteDevice(DeviceObject);
}

// L"\Device\Null" + NUL — the RTL_CONSTANT_STRING backing store.
static DEVICE_NAME_BUF: [u16; 13] = [
    b'\\' as u16,
    b'D' as u16,
    b'e' as u16,
    b'v' as u16,
    b'i' as u16,
    b'c' as u16,
    b'e' as u16,
    b'\\' as u16,
    b'N' as u16,
    b'u' as u16,
    b'l' as u16,
    b'l' as u16,
    0,
];

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: *mut _DRIVER_OBJECT,
    RegistryPath: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let mut DeviceName = _UNICODE_STRING {
        Length: 24,         // 12 wchars * 2
        MaximumLength: 26,  // incl. NUL
        Buffer: DEVICE_NAME_BUF.as_ptr() as PWCH,
    };
    let mut DeviceObject: *mut _DEVICE_OBJECT = core::ptr::null_mut();

    // PAGED_CODE();
    let _ = RegistryPath;

    // Page the driver.
    MmPageEntireDriver(DriverEntry as *const () as PVOID);

    // Create the Null device.
    let Status = IoCreateDevice(
        DriverObject,
        0,
        &mut DeviceName,
        FILE_DEVICE_NULL,
        FILE_DEVICE_SECURE_OPEN,
        0 as BOOLEAN,
        &mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // Register driver routines. The MajorFunction slots / DriverUnload are
    // per-arch bindgen Option<fn> types; transmute our extern "system" fn
    // items in (ABI-identical, only the Rust type string differs by arch).
    let dispatch: PDRIVER_DISPATCH = transmute::<
        unsafe extern "system" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS,
        PDRIVER_DISPATCH,
    >(NullDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = dispatch;
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = dispatch;
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] = dispatch;
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = dispatch;
    (*DriverObject).MajorFunction[IRP_MJ_LOCK_CONTROL as usize] = dispatch;
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_INFORMATION as usize] = dispatch;
    (*DriverObject).DriverUnload = transmute::<
        unsafe extern "system" fn(*mut _DRIVER_OBJECT),
        PDRIVER_UNLOAD,
    >(NullUnload);

    // Initialize the fast I/O dispatch table. Go through a raw pointer (not a
    // `&mut` to the `static mut`) to avoid the nightly static_mut_refs lint.
    let fid = core::ptr::addr_of_mut!(FAST_IO_DISPATCH).cast::<_FAST_IO_DISPATCH>();
    core::ptr::write_bytes(fid as *mut u8, 0, size_of::<_FAST_IO_DISPATCH>());
    (*fid).SizeOfFastIoDispatch = size_of::<_FAST_IO_DISPATCH>() as ULONG;

    // Setup our pointers.
    (*fid).FastIoRead = transmute::<
        unsafe extern "system" fn(
            *mut _FILE_OBJECT,
            PLARGE_INTEGER,
            ULONG,
            BOOLEAN,
            ULONG,
            PVOID,
            PIO_STATUS_BLOCK,
            *mut _DEVICE_OBJECT,
        ) -> BOOLEAN,
        PFAST_IO_READ,
    >(NullRead);
    (*fid).FastIoWrite = transmute::<
        unsafe extern "system" fn(
            *mut _FILE_OBJECT,
            PLARGE_INTEGER,
            ULONG,
            BOOLEAN,
            ULONG,
            PVOID,
            PIO_STATUS_BLOCK,
            *mut _DEVICE_OBJECT,
        ) -> BOOLEAN,
        PFAST_IO_WRITE,
    >(NullWrite);
    (*DriverObject).FastIoDispatch = fid;

    // Return success.
    STATUS_SUCCESS
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
