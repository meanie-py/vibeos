//! virtdisk-rust: `virtdisk.dll` for an OS that has NO virtual-disk provider (nothing can attach
//! a VHD / VHDX / ISO). Exports OpenVirtualDisk, DetachVirtualDisk and
//! GetStorageDependencyInformation — the three Raspberry Pi Imager imports — and answers each
//! with what Windows 11 answers for the NON-virtual cases, which is every case this OS can have.
//!
//! GROUND TRUTH (measured on Windows 11 x64; the first six rows are measurements, the rest are
//! extrapolations from them or from Wine's tests and are marked "unmeasured"):
//!  * OpenVirtualDisk(&{VHD, Microsoft}, L"C:\\nonexistent.vhd", ALL, NONE, NULL, &h) = 2
//!    (ERROR_FILE_NOT_FOUND) and *h is left untouched; NULL type = 87.
//!  * DetachVirtualDisk(INVALID_HANDLE_VALUE or NULL, 0, 0) = 6.
//!  * GetStorageDependencyInformation on a `\\.\C:` handle: HOST_VOLUMES = 0xC03A0015
//!    (ERROR_VIRTDISK_NOT_VIRTUAL_DISK); NONE = 0 with NumberEntries 0 and *SizeUsed 72
//!    (Version 2 request, 4096-byte buffer); DISK_HANDLE = 87 (also on `\\.\PhysicalDrive0`);
//!    INVALID_HANDLE_VALUE = 6; NULL info = 87.
//!  * Wine's virtdisk test (dlls/virtdisk/tests/virtdisk.c), which passes on Windows 10+: a
//!    Version 2/3 parameter block is ACCEPTED, access mask 0xffffff = 87, size 0 = 87.
//!
//! DECISION for the one case that cannot be measured away: a path that EXISTS. Windows would
//! parse and mount it; we cannot. We return ERROR_VIRTDISK_PROVIDER_NOT_FOUND (0xC03A0014, "the
//! virtual disk support provider for the specified file was not found" — winerror.h) rather than
//! Wine's ERROR_CALL_NOT_IMPLEMENTED (120), because that is the error Windows itself uses for
//! "no provider is registered for this storage type", which is literally our situation, and
//! callers treat an NTSTATUS-range failure as "not a usable virtual disk" without special
//! casing. Errors from the open itself (not found, path not found, access denied, sharing
//! violation) are passed through unchanged, so the observable behaviour for everything that is
//! not a readable file matches Windows exactly.
//!
//! The "72": STORAGE_DEPENDENCY_INFO = {u32 Version; u32 NumberEntries; union { Type1 Version1
//! Entries[1]; Type2 Version2Entries[1]; }}. Type2 = {u32 flags, u32 providerFlags, GUID-bearing
//! VIRTUAL_STORAGE_TYPE (20 bytes), u32 AncestorLevel, 4 x PWSTR}. On x64 the pointers pad Type2
//! to 64 (offset 32, +4*8) and the union to offset 8, so 8 + 64 = 72 — i.e. SizeUsed is
//! sizeof(STORAGE_DEPENDENCY_INFO). On i386 the same layout is 8 + 48 = 56. The value here is
//! `size_of::<StorageDependencyInfo>()`, never a literal. (A Version 1 request is unmeasured; it
//! gets the same sizeof, which is also what Wine's stub reports.)
//!
//! Unmeasured choices, all conservative: unknown GET_STORAGE_DEPENDENCY flag bits = 87; Version
//! other than 1/2 = 87; a too-small buffer = ERROR_INSUFFICIENT_BUFFER with *SizeUsed set to the
//! required size (Wine's behaviour); HOST_VOLUMES on a non-volume valid handle also = NOT_VIRTUAL;
//! DetachVirtualDisk on a valid but non-virtual handle = NOT_VIRTUAL, non-zero flags = 87.
//!
//! Every other virtdisk export is `@ stub` exactly as in Wine's virtdisk.spec.

#![cfg_attr(not(test), no_std)]
#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

use core::ffi::c_void;

mod plat;
#[cfg(test)]
mod tests;

pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_FILE_NOT_FOUND: u32 = 2;
pub const ERROR_INVALID_HANDLE: u32 = 6;
pub const ERROR_INVALID_PARAMETER: u32 = 87;
pub const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
pub const ERROR_VIRTDISK_PROVIDER_NOT_FOUND: u32 = 0xC03A_0014;
pub const ERROR_VIRTDISK_NOT_VIRTUAL_DISK: u32 = 0xC03A_0015;

pub const VIRTUAL_DISK_ACCESS_ALL: u32 = 0x003f_0000;
/// OPEN_VIRTUAL_DISK_FLAG_NONE .. NO_WRITE_HARDENING (virtdisk.h, Windows 10).
pub const OPEN_VIRTUAL_DISK_FLAG_MASK: u32 = 0x1ff;
pub const OPEN_VIRTUAL_DISK_VERSION_1: u32 = 1;
pub const OPEN_VIRTUAL_DISK_VERSION_3: u32 = 3;

pub const GET_STORAGE_DEPENDENCY_FLAG_HOST_VOLUMES: u32 = 1;
pub const GET_STORAGE_DEPENDENCY_FLAG_DISK_HANDLE: u32 = 2;
pub const STORAGE_DEPENDENCY_INFO_VERSION_1: u32 = 1;
pub const STORAGE_DEPENDENCY_INFO_VERSION_2: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtualStorageType {
    pub device_id: u32,
    pub vendor_id: Guid,
}

/// Only the leading `Version` is read; the union that follows is irrelevant to validation.
#[repr(C)]
pub struct OpenVirtualDiskParameters {
    pub version: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StorageDependencyInfoType1 {
    pub dependency_type_flags: u32,
    pub provider_specific_flags: u32,
    pub virtual_storage_type: VirtualStorageType,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StorageDependencyInfoType2 {
    pub dependency_type_flags: u32,
    pub provider_specific_flags: u32,
    pub virtual_storage_type: VirtualStorageType,
    pub ancestor_level: u32,
    pub dependency_device_name: *mut u16,
    pub host_volume_name: *mut u16,
    pub dependent_volume_name: *mut u16,
    pub dependent_volume_relative_path: *mut u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union StorageDependencyEntries {
    pub version1: [StorageDependencyInfoType1; 1],
    pub version2: [StorageDependencyInfoType2; 1],
}

#[repr(C)]
pub struct StorageDependencyInfo {
    pub version: u32,
    pub number_entries: u32,
    pub entries: StorageDependencyEntries,
}

/// SizeUsed / required buffer for an empty answer: 72 on amd64+arm64, 56 on i386.
pub const STORAGE_DEPENDENCY_INFO_SIZE: u32 = core::mem::size_of::<StorageDependencyInfo>() as u32;

// Compile-time proof of the layout the doc comment derives, on every target we build.
const _: () = assert!(
    STORAGE_DEPENDENCY_INFO_SIZE == if cfg!(target_pointer_width = "64") { 72 } else { 56 }
);
const _: () = assert!(core::mem::size_of::<VirtualStorageType>() == 20);
const _: () = assert!(core::mem::offset_of!(StorageDependencyInfo, entries) == 8);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// staticlib needs an eh personality (no_std cohab).
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

/// `DWORD WINAPI OpenVirtualDisk(PVIRTUAL_STORAGE_TYPE, PCWSTR, VIRTUAL_DISK_ACCESS_MASK,
/// OPEN_VIRTUAL_DISK_FLAG, POPEN_VIRTUAL_DISK_PARAMETERS, PHANDLE)`.
///
/// Parameter is OPTIONAL (measured: NULL succeeds past validation on Windows; Wine wrongly
/// rejects it). `*handle` is only ever written on success, and success is impossible here.
#[no_mangle]
pub unsafe extern "system" fn OpenVirtualDisk(
    storage_type: *const VirtualStorageType,
    path: *const u16,
    access_mask: u32,
    flags: u32,
    parameters: *const OpenVirtualDiskParameters,
    handle: *mut *mut c_void,
) -> u32 {
    if storage_type.is_null() || path.is_null() || handle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if access_mask & !VIRTUAL_DISK_ACCESS_ALL != 0 || flags & !OPEN_VIRTUAL_DISK_FLAG_MASK != 0 {
        return ERROR_INVALID_PARAMETER;
    }
    if !parameters.is_null() {
        let v = (*parameters).version;
        if !(OPEN_VIRTUAL_DISK_VERSION_1..=OPEN_VIRTUAL_DISK_VERSION_3).contains(&v) {
            return ERROR_INVALID_PARAMETER;
        }
    }
    // Existence/permission is decided by the real open, so every pre-provider failure carries
    // the code Windows would report.
    let e = plat::probe_file(path);
    if e != 0 {
        return e;
    }
    ERROR_VIRTDISK_PROVIDER_NOT_FOUND
}

/// `DWORD WINAPI DetachVirtualDisk(HANDLE, DETACH_VIRTUAL_DISK_FLAG, ULONG)`.
#[no_mangle]
pub unsafe extern "system" fn DetachVirtualDisk(
    handle: *mut c_void,
    flags: u32,
    provider_specific_flags: u32,
) -> u32 {
    let _ = provider_specific_flags;
    if !plat::handle_valid(handle) {
        return ERROR_INVALID_HANDLE;
    }
    if flags != 0 {
        return ERROR_INVALID_PARAMETER; // DETACH_VIRTUAL_DISK_FLAG_NONE is the only defined value
    }
    ERROR_VIRTDISK_NOT_VIRTUAL_DISK
}

/// `DWORD WINAPI GetStorageDependencyInformation(HANDLE, GET_STORAGE_DEPENDENCY_FLAG, ULONG,
/// PSTORAGE_DEPENDENCY_INFO, PULONG)`. `size_used` is optional.
#[no_mangle]
pub unsafe extern "system" fn GetStorageDependencyInformation(
    handle: *mut c_void,
    flags: u32,
    size: u32,
    info: *mut StorageDependencyInfo,
    size_used: *mut u32,
) -> u32 {
    if flags & !(GET_STORAGE_DEPENDENCY_FLAG_HOST_VOLUMES | GET_STORAGE_DEPENDENCY_FLAG_DISK_HANDLE) != 0
        || info.is_null()
        || size == 0
    {
        return ERROR_INVALID_PARAMETER;
    }
    if !plat::handle_valid(handle) {
        return ERROR_INVALID_HANDLE;
    }
    // Nothing here is a virtual disk, so the virtual-disk-specific queries all fail the way
    // Windows fails them for an ordinary volume / physical drive.
    if flags & GET_STORAGE_DEPENDENCY_FLAG_DISK_HANDLE != 0 {
        return ERROR_INVALID_PARAMETER;
    }
    if flags & GET_STORAGE_DEPENDENCY_FLAG_HOST_VOLUMES != 0 {
        return ERROR_VIRTDISK_NOT_VIRTUAL_DISK;
    }
    if size < STORAGE_DEPENDENCY_INFO_SIZE {
        if !size_used.is_null() {
            *size_used = STORAGE_DEPENDENCY_INFO_SIZE;
        }
        return ERROR_INSUFFICIENT_BUFFER;
    }
    let v = (*info).version;
    if v != STORAGE_DEPENDENCY_INFO_VERSION_1 && v != STORAGE_DEPENDENCY_INFO_VERSION_2 {
        return ERROR_INVALID_PARAMETER;
    }
    (*info).number_entries = 0;
    if !size_used.is_null() {
        *size_used = STORAGE_DEPENDENCY_INFO_SIZE;
    }
    ERROR_SUCCESS
}
