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

use ntkrnl_sys::base::{LIST_ENTRY, ULONG, UCHAR, USHORT, BOOLEAN, SIZE_T, PVOID, PFN_NUMBER};
use core::ffi::c_void;

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
    pub fn UiMessageBoxCritical(msg: *const i8);
    pub static __ImageBase: u8;
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

/// C `L"..."` wide-string literal -> `*const u16` ('static; NUL kept as
/// written). ASCII-only (all freeldr service names are). #[macro_export]
/// puts it at crate root -> `crate::wide!`.
#[macro_export]
macro_rules! wide {
    ($s:expr) => {{
        static W: [u16; { $s.len() }] = {
            let b = $s.as_bytes();
            let mut a = [0u16; { $s.len() }];
            let mut i = 0;
            while i < $s.len() { a[i] = b[i] as u16; i += 1; }
            a
        };
        W.as_ptr()
    }};
}

/// C `min()` / `max()` macros — generic.
#[inline(always)]
pub fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline(always)]
pub fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }

// xdk/mmfuncs.h ALIGN_* / reactos ROUND_* macros. Type-generic over the
// unsigned integer width (freeldr calls these with ULONG *and* usize — the
// C macros are width-agnostic, so the keystone helpers must be too).
pub trait IntAlign:
    Copy
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::BitAnd<Output = Self>
    + core::ops::Not<Output = Self>
    + From<u8>
{
}
impl<T> IntAlign for T where
    T: Copy
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::BitAnd<Output = T>
        + core::ops::Not<Output = T>
        + From<u8>
{
}

#[inline(always)]
pub fn ALIGN_DOWN_BY<T: IntAlign>(size: T, align: T) -> T {
    size & !(align - T::from(1u8))
}
#[inline(always)]
pub fn ALIGN_UP_BY<T: IntAlign>(size: T, align: T) -> T {
    ALIGN_DOWN_BY(size + align - T::from(1u8), align)
}
#[inline(always)]
pub unsafe fn ALIGN_DOWN_POINTER_BY<T>(p: *mut T, align: usize) -> *mut T {
    ALIGN_DOWN_BY(p as usize, align) as *mut T
}
#[inline(always)]
pub unsafe fn ALIGN_UP_POINTER_BY<T>(p: *mut T, align: usize) -> *mut T {
    ALIGN_UP_BY(p as usize, align) as *mut T
}
#[inline(always)]
pub fn ROUND_DOWN<T: IntAlign>(n: T, align: T) -> T {
    n & !(align - T::from(1u8))
}
#[inline(always)]
pub fn ROUND_UP<T: IntAlign>(n: T, align: T) -> T {
    ROUND_DOWN(n + align - T::from(1u8), align)
}

/// `RtlZeroMemory` — C macro (memset(d,0,n)).
#[inline(always)]
pub unsafe fn RtlZeroMemory(dst: PVOID, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len)
}

/// `ASSERT(cond)` — no-op in the freeldr release build (NDEBUG).
#[inline(always)]
pub fn ASSERT<T>(_cond: T) {}



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

// ===========================================================================
// arc.h enum _TYPE_OF_MEMORY (sequential 0..) + bindgen-prefixed aliases
// ===========================================================================
pub const LoaderExceptionBlock: TYPE_OF_MEMORY = 0;
pub const _TYPE_OF_MEMORY_LoaderExceptionBlock: TYPE_OF_MEMORY = 0;
pub const LoaderSystemBlock: TYPE_OF_MEMORY = 1;
pub const _TYPE_OF_MEMORY_LoaderSystemBlock: TYPE_OF_MEMORY = 1;
pub const LoaderFree: TYPE_OF_MEMORY = 2;
pub const _TYPE_OF_MEMORY_LoaderFree: TYPE_OF_MEMORY = 2;
pub const LoaderBad: TYPE_OF_MEMORY = 3;
pub const _TYPE_OF_MEMORY_LoaderBad: TYPE_OF_MEMORY = 3;
pub const LoaderLoadedProgram: TYPE_OF_MEMORY = 4;
pub const _TYPE_OF_MEMORY_LoaderLoadedProgram: TYPE_OF_MEMORY = 4;
pub const LoaderFirmwareTemporary: TYPE_OF_MEMORY = 5;
pub const _TYPE_OF_MEMORY_LoaderFirmwareTemporary: TYPE_OF_MEMORY = 5;
pub const LoaderFirmwarePermanent: TYPE_OF_MEMORY = 6;
pub const _TYPE_OF_MEMORY_LoaderFirmwarePermanent: TYPE_OF_MEMORY = 6;
pub const LoaderOsloaderHeap: TYPE_OF_MEMORY = 7;
pub const _TYPE_OF_MEMORY_LoaderOsloaderHeap: TYPE_OF_MEMORY = 7;
pub const LoaderOsloaderStack: TYPE_OF_MEMORY = 8;
pub const _TYPE_OF_MEMORY_LoaderOsloaderStack: TYPE_OF_MEMORY = 8;
pub const LoaderSystemCode: TYPE_OF_MEMORY = 9;
pub const _TYPE_OF_MEMORY_LoaderSystemCode: TYPE_OF_MEMORY = 9;
pub const LoaderHalCode: TYPE_OF_MEMORY = 10;
pub const _TYPE_OF_MEMORY_LoaderHalCode: TYPE_OF_MEMORY = 10;
pub const LoaderBootDriver: TYPE_OF_MEMORY = 11;
pub const _TYPE_OF_MEMORY_LoaderBootDriver: TYPE_OF_MEMORY = 11;
pub const LoaderConsoleInDriver: TYPE_OF_MEMORY = 12;
pub const _TYPE_OF_MEMORY_LoaderConsoleInDriver: TYPE_OF_MEMORY = 12;
pub const LoaderConsoleOutDriver: TYPE_OF_MEMORY = 13;
pub const _TYPE_OF_MEMORY_LoaderConsoleOutDriver: TYPE_OF_MEMORY = 13;
pub const LoaderStartupDpcStack: TYPE_OF_MEMORY = 14;
pub const _TYPE_OF_MEMORY_LoaderStartupDpcStack: TYPE_OF_MEMORY = 14;
pub const LoaderStartupKernelStack: TYPE_OF_MEMORY = 15;
pub const _TYPE_OF_MEMORY_LoaderStartupKernelStack: TYPE_OF_MEMORY = 15;
pub const LoaderStartupPanicStack: TYPE_OF_MEMORY = 16;
pub const _TYPE_OF_MEMORY_LoaderStartupPanicStack: TYPE_OF_MEMORY = 16;
pub const LoaderStartupPcrPage: TYPE_OF_MEMORY = 17;
pub const _TYPE_OF_MEMORY_LoaderStartupPcrPage: TYPE_OF_MEMORY = 17;
pub const LoaderStartupPdrPage: TYPE_OF_MEMORY = 18;
pub const _TYPE_OF_MEMORY_LoaderStartupPdrPage: TYPE_OF_MEMORY = 18;
pub const LoaderRegistryData: TYPE_OF_MEMORY = 19;
pub const _TYPE_OF_MEMORY_LoaderRegistryData: TYPE_OF_MEMORY = 19;
pub const LoaderMemoryData: TYPE_OF_MEMORY = 20;
pub const _TYPE_OF_MEMORY_LoaderMemoryData: TYPE_OF_MEMORY = 20;
pub const LoaderNlsData: TYPE_OF_MEMORY = 21;
pub const _TYPE_OF_MEMORY_LoaderNlsData: TYPE_OF_MEMORY = 21;
pub const LoaderSpecialMemory: TYPE_OF_MEMORY = 22;
pub const _TYPE_OF_MEMORY_LoaderSpecialMemory: TYPE_OF_MEMORY = 22;
pub const LoaderBBTMemory: TYPE_OF_MEMORY = 23;
pub const _TYPE_OF_MEMORY_LoaderBBTMemory: TYPE_OF_MEMORY = 23;
pub const LoaderReserve: TYPE_OF_MEMORY = 24;
pub const _TYPE_OF_MEMORY_LoaderReserve: TYPE_OF_MEMORY = 24;
pub const LoaderXIPRom: TYPE_OF_MEMORY = 25;
pub const _TYPE_OF_MEMORY_LoaderXIPRom: TYPE_OF_MEMORY = 25;
pub const LoaderHALCachedMemory: TYPE_OF_MEMORY = 26;
pub const _TYPE_OF_MEMORY_LoaderHALCachedMemory: TYPE_OF_MEMORY = 26;
pub const LoaderLargePageFiller: TYPE_OF_MEMORY = 27;
pub const _TYPE_OF_MEMORY_LoaderLargePageFiller: TYPE_OF_MEMORY = 27;
pub const LoaderErrorLogMemory: TYPE_OF_MEMORY = 28;
pub const _TYPE_OF_MEMORY_LoaderErrorLogMemory: TYPE_OF_MEMORY = 28;
pub const LoaderMaximum: TYPE_OF_MEMORY = 29;
pub const _TYPE_OF_MEMORY_LoaderMaximum: TYPE_OF_MEMORY = 29;

// freeldr layout / PE / heap constants (i386 smoke target values).
pub type POOL_TYPE = u32;
pub const FREELDR_BASE: usize = 0xF800;
pub const FREELDR_PE_BASE: usize = 0x10000;
pub const FREELDR_SECTION_COUNT: usize = 4;
pub const MM_MAX_PAGE_LOADER: PFN_NUMBER = 0xFFFFF;
pub const MAX_FREELDR_PE_SIZE: usize = 0x70000 - FREELDR_PE_BASE - MM_PAGE_SIZE;
pub const REDZONE_MARK: u64 = 0xCCCCCCCCCCCCCCCC;
pub const REDZONE_ALLOCATION: usize = 24;
pub const HEAP_ZERO_MEMORY: ULONG = 0x00000008;
pub const MAXUSHORT: USHORT = 0xffff;
pub const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x10b;
pub const IMAGE_FILE_MACHINE_NATIVE: u16 = 0x014c;
pub const IMAGE_SUBSYSTEM_NATIVE: u16 = 1;
pub const FREELDR_IMAGE_CORRUPTION: ULONG = 1;
pub const MEMORY_INIT_FAILURE: ULONG = 2;

// meminit.c globals the rustconv pass dropped (faithful #[no_mangle]).
#[no_mangle] pub static mut MmLowestPhysicalPage: PFN_NUMBER = 0xFFFFFFFF;
#[no_mangle] pub static mut MmHighestPhysicalPage: PFN_NUMBER = 0;
#[no_mangle] pub static mut BiosMemoryMapEntryCount: ULONG = 0;
#[no_mangle] pub static mut BiosMemoryMap: PFREELDR_MEMORY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle] pub static mut LoaderPagesSpanned: PFN_NUMBER = 0;
// MachVtbl is real C (machpc.c / machine.h) — declared, not owned.
#[repr(C)]
pub struct MACHVTBL {
    _before: [*const core::ffi::c_void; 22], // Cons*/Video*/Beep/Prepare/GetMemoryDescriptor
    pub GetMemoryMap: Option<unsafe extern "C" fn(*mut ULONG) -> PFREELDR_MEMORY_DESCRIPTOR>,
    _after: [*const core::ffi::c_void; 14],
}
extern "C" { pub static MachVtbl: MACHVTBL; }

// debug.h — DPRINT channels + levels (shared).
pub const DPRINT_NONE: ULONG = 0;
pub const DPRINT_WARNING: ULONG = 1;
pub const DPRINT_MEMORY: ULONG = 2;
pub const DPRINT_FILESYSTEM: ULONG = 3;
pub const DPRINT_INIFILE: ULONG = 4;
pub const DPRINT_UI: ULONG = 5;
pub const DPRINT_DISK: ULONG = 6;
pub const DPRINT_CACHE: ULONG = 7;
pub const DPRINT_REGISTRY: ULONG = 8;
pub const DPRINT_REACTOS: ULONG = 9;
pub const DPRINT_LINUX: ULONG = 10;
pub const DPRINT_HWDETECT: ULONG = 11;
pub const DPRINT_WINDOWS: ULONG = 12;
pub const DPRINT_PELOADER: ULONG = 13;
pub const DPRINT_SCSIPORT: ULONG = 14;
pub const DPRINT_HEAP: ULONG = 15;
pub const DBG_CHANNELS_COUNT: usize = 16;
pub const ERR_LEVEL: ULONG = 0x1;
pub const FIXME_LEVEL: ULONG = 0x2;
pub const WARN_LEVEL: ULONG = 0x4;
pub const TRACE_LEVEL: ULONG = 0x8;
pub const MAX_LEVEL: ULONG = 0xF;
pub type ULONG64 = u64;

/// Debug-path intrinsics (no-op for boot-to-shell; DBG off in release).
#[no_mangle] extern "C" fn WRITE_PORT_UCHAR(_port: *mut u8, _val: u8) {}
#[no_mangle] extern "C" fn DbgBreakPoint() {}

// fs.h DEVVTBL — vtable; ARC_* are ddk Option<extern "C" fn> (bindgen-exact).
use ntkrnl_sys::ddk::{ARC_CLOSE, ARC_GET_FILE_INFORMATION, ARC_OPEN, ARC_READ, ARC_SEEK};
use ntkrnl_sys::base::PCWSTR;
#[repr(C)]
pub struct DEVVTBL {
    pub Close: ARC_CLOSE,
    pub GetFileInformation: ARC_GET_FILE_INFORMATION,
    pub Open: ARC_OPEN,
    pub Read: ARC_READ,
    pub Seek: ARC_SEEK,
    pub ServiceName: PCWSTR,
}
pub type PFS_MOUNT = unsafe extern "C" fn(ULONG) -> *const DEVVTBL;
// vtables are immutable read-only; raw fn/wstr ptrs are Send/Sync-safe here.
unsafe impl Sync for DEVVTBL {}

// arc.h _ARC_CODES — sequential from 0. ARC_STATUS is ULONG (ddk).
pub const ESUCCESS: ULONG = 0;
pub const E2BIG: ULONG = 1;
pub const EACCES: ULONG = 2;
pub const EAGAIN: ULONG = 3;
pub const EBADF: ULONG = 4;
pub const EBUSY: ULONG = 5;
pub const EFAULT: ULONG = 6;
pub const EINVAL: ULONG = 7;
pub const EIO: ULONG = 8;
pub const EISDIR: ULONG = 9;
pub const EMFILE: ULONG = 10;
pub const EMLINK: ULONG = 11;
pub const ENAMETOOLONG: ULONG = 12;
pub const ENODEV: ULONG = 13;
pub const ENOENT: ULONG = 14;
pub const ENOEXEC: ULONG = 15;
pub const ENOMEM: ULONG = 16;
pub const ENOSPC: ULONG = 17;
pub const ENOTDIR: ULONG = 18;
pub const ENOTTY: ULONG = 19;
pub const ENXIO: ULONG = 20;
pub const EROFS: ULONG = 21;
pub const EMAXIMUM: ULONG = 22;
pub type SEEKMODE = u32;
pub const SeekAbsolute: u32 = 0;
pub const SeekRelative: u32 = 1;
// arc.h _OPENMODE
pub type OPENMODE = u32;
pub const OpenReadOnly: u32 = 0;
pub const OpenWriteOnly: u32 = 1;
pub const OpenReadWrite: u32 = 2;
pub const CreateWriteOnly: u32 = 3;
pub const CreateReadWrite: u32 = 4;
pub const SupersedeWriteOnly: u32 = 5;
pub const SupersedeReadWrite: u32 = 6;
pub const OpenDirectory: u32 = 7;
pub const CreateDirectory: u32 = 8;
// arc.h _FILEATTRIBUTES
pub const ReadOnlyFile: u32 = 0x01;
pub const HiddenFile: u32 = 0x02;
pub const SystemFile: u32 = 0x04;
pub const ArchiveFile: u32 = 0x08;
pub const DirectoryFile: u32 = 0x10;
pub const DeleteFile: u32 = 0x20;
// shared CRT-ish aliases the fs/ui translations expect
pub type VOID = c_void;
pub type UINT8 = u8;
pub type UINT16 = u16;
pub type UINT32 = u32;
pub const MAX_PATH: usize = 260;
extern "C" {
    pub fn strrchr(s: *const i8, c: i32) -> *mut i8;
    pub fn strchr(s: *const i8, c: i32) -> *mut i8;
    pub fn strcmp(a: *const i8, b: *const i8) -> i32;
    pub fn strcpy(d: *mut i8, s: *const i8) -> *mut i8;
    pub fn strcat(d: *mut i8, s: *const i8) -> *mut i8;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn RtlStringCbCopyA(dst: *mut i8, cb: usize, src: *const i8) -> i32;
    pub fn RtlStringCbCatA(dst: *mut i8, cb: usize, src: *const i8) -> i32;
}

// C `_countof(arr)` — the translator emits `_countof(addr_of!(arr))`.
#[inline(always)]
pub fn _countof<T, const N: usize>(_p: *const [T; N]) -> ULONG {
    N as ULONG
}

pub const MAX_FDS: usize = 60;
pub const INVALID_FILE_ID: ULONG = ULONG::MAX; // ((ULONG)-1)

// preamble.py over-strips the `ntkrnl_sys::rtl::*` glob; provide RtlCopyMemory
// (memmove semantics; freeldr fs callers always copy disjoint regions).
#[inline(always)]
pub unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy(src as *const u8, dst as *mut u8, len);
}
