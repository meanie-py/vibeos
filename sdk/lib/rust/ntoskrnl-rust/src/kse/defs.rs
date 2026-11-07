//! ntoskrnl/include/internal/kse.h `#define`s and tag.h's KSE pool tags. The
//! structs (KSE_HOOK, KSE_SHIM, ...) come from ntkrnl_sys::ddk; these plain
//! macros never reach bindgen, so they live here, values copied verbatim.
#![allow(dead_code)]
use ntkrnl_sys::base::ULONG;

pub const KseHookFunction: ULONG = 0;
pub const KseHookIRPCallback: ULONG = 1;
pub const KseHookInvalid: ULONG = 2;

pub const KseHookCallbackDriverInit: ULONG = 1;
pub const KseHookCallbackDriverStartIo: ULONG = 2;
pub const KseHookCallbackDriverUnload: ULONG = 3;
pub const KseHookCallbackAddDevice: ULONG = 4;
pub const KseHookCallbackMajorFunction: ULONG = 100;

pub const KseCollectionNtExport: ULONG = 0;
pub const KseCollectionHalExport: ULONG = 1;
pub const KseCollectionDriverExport: ULONG = 2;
pub const KseCollectionCallback: ULONG = 3;
pub const KseCollectionInvalid: ULONG = 4;

// tag.h multi-character constants: 'cESK' stores as the bytes "KSEc".
pub const TAG_KSE_CALLBACKS: ULONG = u32::from_le_bytes(*b"KSEc");
pub const TAG_KSE_DATABASE: ULONG = u32::from_le_bytes(*b"KSEd");
pub const TAG_KSE_PROVIDER: ULONG = u32::from_le_bytes(*b"KSEp");
pub const TAG_KSE_REGISTRY: ULONG = u32::from_le_bytes(*b"KSEr");

// Enum members and macros the translated bodies name directly. The enum values come
// from the bindgen enum constants, so they follow the headers.
pub const IoReadAccess: ntkrnl_sys::ddk::_LOCK_OPERATION = ntkrnl_sys::ddk::_LOCK_OPERATION_IoReadAccess;
pub const MmCached: ntkrnl_sys::ddk::_MEMORY_CACHING_TYPE = ntkrnl_sys::ddk::_MEMORY_CACHING_TYPE_MmCached;
pub const NormalPagePriority: ntkrnl_sys::ddk::_MM_PAGE_PRIORITY = ntkrnl_sys::ddk::_MM_PAGE_PRIORITY_NormalPagePriority;
pub const KeyValueFullInformation: ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS =
    ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation;
pub const STATUS_INVALID_IMAGE_FORMAT: ntkrnl_sys::base::NTSTATUS = 0xC000_007Bu32 as i32;
pub const IMAGE_DIRECTORY_ENTRY_IMPORT: u16 = 1; // winnt.h
pub const VER_PLATFORM_WIN32_NT: ULONG = 2; // winnt.h
pub const ANSI_NULL: i8 = 0;
pub const UNICODE_NULL: u16 = 0;

extern "C" {
    /// ntoskrnl's own export (the C calls the CRT strcmp). Other modules declare it
    /// with *mut/PCHAR parameters; the ABI is identical.
    #[allow(clashing_extern_declarations)]
    pub fn strcmp(s1: *const i8, s2: *const i8) -> i32;
    /// Set by the loader/ExpInitialize; kernel-wide `extern ULONG InitSafeBootMode`.
    pub static mut InitSafeBootMode: ULONG;
}

/// io.h `IoGetDrvObjExtension(DriverObject)`: the driver extension viewed as the
/// EXTENDED_DRIVER_EXTENSION it really is.
#[inline(always)]
pub unsafe fn IoGetDrvObjExtension(
    DriverObject: ntkrnl_sys::ddk::PDRIVER_OBJECT,
) -> ntkrnl_sys::ddk::PEXTENDED_DRIVER_EXTENSION {
    (*DriverObject).DriverExtension as ntkrnl_sys::ddk::PEXTENDED_DRIVER_EXTENSION
}

/// LDR_DATA_TABLE_ENTRY from its InLoadOrderLinks (CONTAINING_RECORD).
#[inline(always)]
pub unsafe fn ldr_from_load_order(
    e: *mut ntkrnl_sys::ddk::LIST_ENTRY,
) -> ntkrnl_sys::ddk::PLDR_DATA_TABLE_ENTRY {
    (e as *mut u8).sub(core::mem::offset_of!(ntkrnl_sys::ddk::LDR_DATA_TABLE_ENTRY, InLoadOrderLinks))
        as ntkrnl_sys::ddk::PLDR_DATA_TABLE_ENTRY
}

/// winnt.h IMAGE_SNAP_BY_ORDINAL: the ordinal flag is the top bit of the
/// thunk, which is 32 or 64 bits wide with the image.
#[inline(always)]
pub fn image_snap_by_ordinal(ordinal: u64) -> bool {
    #[cfg(target_pointer_width = "64")]
    { ordinal & 0x8000_0000_0000_0000 != 0 }
    #[cfg(target_pointer_width = "32")]
    { ordinal & 0x8000_0000 != 0 }
}
