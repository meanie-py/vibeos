//! frldr — FreeLDR-internal keystone (types / globals / constants / stubs).
//!
//! Distinct from `ffi.rs` (which is the peloader PE-loader FFI surface).
//! This module carries the freeldr-internal C surface that the generic
//! lib/* translations need: the cache-manager structs and globals, the
//! pool tags, heap/disk Mach* externs, and no-op debug stubs (TRACE/ERR
//! are DBG_DEFAULT_CHANNEL macros in C — no real symbol — so the kernel
//! never links them; we provide cdecl-safe no-op symbols instead).
//!
//! Grows one bucket at a time as the 🟢 sweep lands (cache unit first).

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntkrnl_sys::base::{LIST_ENTRY, ULONG, UCHAR, BOOLEAN, SIZE_T, PVOID, PFN_NUMBER};

// ---------------------------------------------------------------------------
// cache.h — cache-manager types (mirror C layout exactly, #[repr(C)])
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct _CACHE_BLOCK {
    pub ListEntry: LIST_ENTRY,
    pub BlockNumber: ULONG,
    pub LockedInCache: BOOLEAN,
    pub AccessCount: ULONG,
    pub BlockData: PVOID,
}
pub type CACHE_BLOCK = _CACHE_BLOCK;
pub type PCACHE_BLOCK = *mut _CACHE_BLOCK;

#[repr(C)]
pub struct _CACHE_DRIVE {
    pub DriveNumber: UCHAR,
    pub BytesPerSector: ULONG,
    pub BlockSize: ULONG,
    pub CacheBlockHead: LIST_ENTRY,
}
pub type CACHE_DRIVE = _CACHE_DRIVE;
pub type PCACHE_DRIVE = *mut _CACHE_DRIVE;

// disk.h — _GEOMETRY
#[repr(C)]
pub struct _GEOMETRY {
    pub Cylinders: ULONG,
    pub Heads: ULONG,
    pub SectorsPerTrack: ULONG,
    pub BytesPerSector: ULONG,
    pub Sectors: u64,
}
pub type GEOMETRY = _GEOMETRY;
pub type PGEOMETRY = *mut _GEOMETRY;

// ---------------------------------------------------------------------------
// Constants (verified against the C headers — these are the AI's blind spots)
// ---------------------------------------------------------------------------

// C multi-char literals are big-endian: 'DcaC' == 0x44_63_61_43.
pub const TAG_CACHE_DATA: ULONG = u32::from_be_bytes(*b"DcaC") as ULONG;
pub const TAG_CACHE_BLOCK: ULONG = u32::from_be_bytes(*b"BcaC") as ULONG;
// mm.h
pub const MM_PAGE_SIZE: SIZE_T = 4096;
pub const TEMP_HEAP_SIZE: SIZE_T = 32 * 1024 * 1024;

// ---------------------------------------------------------------------------
// cache.c globals (C-defined data; faithful #[no_mangle] so any remaining
// C TU that externs them via cache.h still links).
// ---------------------------------------------------------------------------

#[no_mangle]
pub static mut CacheManagerDrive: _CACHE_DRIVE = _CACHE_DRIVE {
    DriveNumber: 0,
    BytesPerSector: 0,
    BlockSize: 0,
    CacheBlockHead: LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() },
};
#[no_mangle]
pub static mut CacheManagerInitialized: BOOLEAN = 0;
#[no_mangle]
pub static mut CacheManagerDataInvalid: BOOLEAN = 0;
#[no_mangle]
pub static mut CacheBlockCount: ULONG = 0;
#[no_mangle]
pub static mut CacheSizeLimit: SIZE_T = 0;
#[no_mangle]
pub static mut CacheSizeCurrent: SIZE_T = 0;

// ---------------------------------------------------------------------------
// FreeLDR-internal C symbols the cache unit calls (cdecl — FLDRAPI is empty
// on the non-MSVC build, so every freeldr API is plain cdecl `_Name`).
// ---------------------------------------------------------------------------

extern "C" {
    pub fn FrLdrTempAlloc(size: usize, tag: ULONG) -> *mut core::ffi::c_void;
    pub fn FrLdrTempFree(ptr: *mut core::ffi::c_void, tag: ULONG);
    pub fn MachDiskReadLogicalSectors(
        DriveNumber: UCHAR,
        SectorNumber: u64,
        SectorCount: ULONG,
        Buffer: PVOID,
    ) -> BOOLEAN;
    pub fn MachDiskGetDriveGeometry(DriveNumber: UCHAR, Geometry: PGEOMETRY) -> BOOLEAN;
    pub fn MachDiskGetCacheableBlockCount(DriveNumber: UCHAR) -> ULONG;

    pub static mut DiskReadBuffer: PVOID;
}

// ---------------------------------------------------------------------------
// Debug no-op symbols. C `TRACE`/`BugCheck` are DBG_DEFAULT_CHANNEL macros
// (no real symbol). The translated code declares them as variadic
// `extern "C" fn` so its call sites typecheck; we satisfy the link with a
// non-variadic cdecl definition — caller-cleans-stack makes ignoring the
// extra varargs ABI-safe on i386/amd64/arm64. THE systemic sweep fix.
// ---------------------------------------------------------------------------

#[no_mangle]
extern "C" fn TRACE(_fmt: *const i8) { /* no-op (DBG channel disabled) */ }
#[no_mangle]
extern "C" fn ERR(_fmt: *const i8) { /* no-op */ }
#[no_mangle]
extern "C" fn WARN(_fmt: *const i8) { /* no-op */ }
#[no_mangle]
extern "C" fn FIXME(_fmt: *const i8) { /* no-op */ }
#[no_mangle]
extern "C" fn DbgDumpBuffer(_level: ULONG, _buffer: *const u8, _size: ULONG) {}

#[no_mangle]
extern "C" fn BugCheck(_message: *const i8) -> ! {
    // FrLdrBugCheckWithMessage analogue — freeldr halts here in C too.
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe { core::arch::asm!("hlt"); }
        #[cfg(target_arch = "aarch64")]
        unsafe { core::arch::asm!("wfe", options(nomem, nostack)); }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        core::hint::spin_loop();
    }
}

/// C `min()` macro — generic, used by CacheInitializeDrive.
#[inline(always)]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

// ===========================================================================
// inifile.h — .ini parser unit (ini_init / inifile / parse)
// ===========================================================================

use ntkrnl_sys::base::PCHAR;

#[repr(C)]
pub struct INI_SECTION_ITEM {
    pub ListEntry: LIST_ENTRY,
    pub ItemName: PCHAR,
    pub ItemValue: PCHAR,
}
pub type PINI_SECTION_ITEM = *mut INI_SECTION_ITEM;
pub type _INI_SECTION_ITEM = INI_SECTION_ITEM;

#[repr(C)]
pub struct INI_SECTION {
    pub ListEntry: LIST_ENTRY,
    pub SectionName: PCHAR,
    pub SectionItemCount: ULONG,
    pub SectionItemList: LIST_ENTRY,
}
pub type PINI_SECTION = *mut INI_SECTION;
pub type _INI_SECTION = INI_SECTION;

// C multi-char literals are big-endian: 'FinI' == 0x46_69_6E_49.
pub const TAG_INI_FILE: ULONG = u32::from_be_bytes(*b"FinI") as ULONG;
pub const TAG_INI_SECTION: ULONG = u32::from_be_bytes(*b"SinI") as ULONG;
pub const TAG_INI_SECTION_ITEM: ULONG = u32::from_be_bytes(*b"IinI") as ULONG;
pub const TAG_INI_NAME: ULONG = u32::from_be_bytes(*b"NinI") as ULONG;
pub const TAG_INI_VALUE: ULONG = u32::from_be_bytes(*b"VinI") as ULONG;
pub const INI_FILE_COMMENT_CHAR: u8 = b';';

// inifile.c globals (faithful #[no_mangle]).
#[no_mangle]
pub static mut IniFileSectionListHead: LIST_ENTRY =
    LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
#[no_mangle]
pub static mut IniFileSectionInitialized: BOOLEAN = 0;
#[no_mangle]
pub static mut IniFileSectionCount: ULONG = 0;
#[no_mangle]
pub static mut IniFileSettingCount: ULONG = 0;

// ===========================================================================
// mm.h — FreeLDR memory manager (meminit / mm / heap)
// ===========================================================================

// arc.h `enum _TYPE_OF_MEMORY` — C enum, default int width.
pub type TYPE_OF_MEMORY = u32;

#[repr(C)]
pub struct _FREELDR_MEMORY_DESCRIPTOR {
    pub MemoryType: TYPE_OF_MEMORY,
    pub BasePage: PFN_NUMBER,
    pub PageCount: PFN_NUMBER,
}
pub type FREELDR_MEMORY_DESCRIPTOR = _FREELDR_MEMORY_DESCRIPTOR;
pub type PFREELDR_MEMORY_DESCRIPTOR = *mut _FREELDR_MEMORY_DESCRIPTOR;

// Packed (mm.h wraps it in pshpack1/poppack) — the lookup table is indexed
// by exact element stride; must match C byte-for-byte.
#[repr(C, packed)]
pub struct PAGE_LOOKUP_TABLE_ITEM {
    pub PageAllocated: TYPE_OF_MEMORY,
    pub PageAllocationLength: PFN_NUMBER,
}
pub type PPAGE_LOOKUP_TABLE_ITEM = *mut PAGE_LOOKUP_TABLE_ITEM;

pub const DEFAULT_HEAP_SIZE: SIZE_T = 1024 * 1024;
// (TEMP_HEAP_SIZE / MM_PAGE_SIZE already defined in the cache section.)
pub const HEAP_PAGES: SIZE_T = 0x400;

// mm.c / heap.c globals (faithful #[no_mangle]; mm.c is the real owner —
// the cache unit consumes TotalPagesInLookupTable via this export).
#[no_mangle]
pub static mut PageLookupTableAddress: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut TotalPagesInLookupTable: PFN_NUMBER = 0;
#[no_mangle]
pub static mut FreePagesInLookupTable: PFN_NUMBER = 0;
#[no_mangle]
pub static mut LastFreePageHint: PFN_NUMBER = 0;
#[no_mangle]
pub static mut FrLdrImageSize: SIZE_T = 0;
