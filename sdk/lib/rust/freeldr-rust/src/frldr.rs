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
#[derive(Clone, Copy)]
pub struct _GEOMETRY {
    pub Cylinders: ULONG,
    pub Heads: ULONG,
    pub SectorsPerTrack: ULONG,
    pub BytesPerSector: ULONG,
    pub Sectors: u64,
}
pub type GEOMETRY = _GEOMETRY;
pub type PGEOMETRY = *mut _GEOMETRY;

// ntldr/winldr.h _LOADER_SYSTEM_BLOCK — shared across winldr/wlregistry
// TUs. #[repr(C)], exact field order; i386 includes HeadlessLoaderBlock.
// MAX_OPTIONS_LENGTH=255, MAX_PATH=260 (so [+1] arrays are 256/261).
#[repr(C)]
pub struct _LOADER_SYSTEM_BLOCK {
    pub LoaderBlock: ntkrnl_sys::ddk::_LOADER_PARAMETER_BLOCK,
    pub Extension: ntkrnl_sys::ddk::_LOADER_PARAMETER_EXTENSION,
    pub SetupBlock: ntkrnl_sys::ddk::_SETUP_LOADER_BLOCK,
    #[cfg(target_arch = "x86")]
    pub HeadlessLoaderBlock: ntkrnl_sys::ddk::_HEADLESS_LOADER_BLOCK,
    pub NlsDataBlock: ntkrnl_sys::ddk::_NLS_DATA_BLOCK,
    pub LoadOptions: [::core::ffi::c_char; 256],
    pub ArcBootDeviceName: [::core::ffi::c_char; 261],
    pub NtBootPathName: [::core::ffi::c_char; 261],
    pub NtHalPathName: [::core::ffi::c_char; 261],
    pub ArcDiskInformation: ntkrnl_sys::ddk::_ARC_DISK_INFORMATION,
    pub LoaderPerformanceData: ntkrnl_sys::ddk::_LOADER_PERFORMANCE_DATA,
}
pub type LOADER_SYSTEM_BLOCK = _LOADER_SYSTEM_BLOCK;
pub type PLOADER_SYSTEM_BLOCK = *mut _LOADER_SYSTEM_BLOCK;

// ---------------------------------------------------------------------------
// Constants (verified against the C headers — these are the AI's blind spots)
// ---------------------------------------------------------------------------

// C multi-char literals are big-endian: 'DcaC' == 0x44_63_61_43.
pub const TAG_CACHE_DATA: ULONG = u32::from_be_bytes(*b"DcaC") as ULONG;
pub const TAG_CACHE_BLOCK: ULONG = u32::from_be_bytes(*b"BcaC") as ULONG;
// mm.h
pub const MM_PAGE_SIZE: SIZE_T = 4096;
pub const MM_PAGE_MASK: ULONG = 0xFFF;
pub const MM_PAGE_SHIFT: ULONG = 12;
// i386 non-PAE: 20 bits for the PFN. (MM_MAX_PAGE_LOADER already defined
// below as PFN_NUMBER — reused, not redefined.)
pub const MM_MAX_PAGE: ULONG = 0xFFFFF;
pub const TEMP_HEAP_SIZE: SIZE_T = 32 * 1024 * 1024;
// ndk ketypes.h: KI_USER_SHARED_DATA fixed VA. Per-arch:
//   i386  (ndk/i386/ketypes.h):   0xffdf0000
//   amd64 (xdk/amd64/ke.h):       0xFFFFF78000000000
//   arm64 (ndk/arm64/ketypes.h):  0xFFFFF78000000000ULL
#[cfg(target_arch = "x86")]
pub const KI_USER_SHARED_DATA: usize = 0xffdf0000;
#[cfg(target_arch = "x86_64")]
pub const KI_USER_SHARED_DATA: usize = 0xFFFFF78000000000;
#[cfg(target_arch = "aarch64")]
pub const KI_USER_SHARED_DATA: usize = 0xFFFFF78000000000;
// ndk i386 ketypes.h: KSEG0_BASE. Reuse ffi.rs's per-arch definition
// (single source of truth) so the bare name resolves via `use frldr::*`.
pub use crate::ffi::KSEG0_BASE;

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

// `RtlZeroMemory` — re-exported from `ntkrnl_sys::rtl` (NTAPI / stdcall).
//
// We DO NOT define a Rust body here. `libblrtl.a` (sdk/lib/rtl/i386/rtlmem.s)
// provides the real `_RtlZeroMemory@8` symbol; both freeldr_pe.exe and
// uefildr.efi link against it.
//
// History (preserved as a load-bearing memo — see feedback_ud2_abi_vs_captures.md):
// this file used to define `pub unsafe extern "C" fn RtlZeroMemory(...)` so
// callers importing `crate::frldr::*` had it in scope. That `extern "C"` was
// cdecl, which exported the (undecorated) symbol `_RtlZeroMemory`, distinct
// from libblrtl.a's stdcall `_RtlZeroMemory@8`. The linker was happy.
//
// But at the LLVM IR layer the IR-level name `@RtlZeroMemory` was shared
// with the `ntkrnl_sys::rtl::RtlZeroMemory` extern decl (`extern "system"` =
// stdcall). LLVM marked our local cdecl body as `available_externally` and
// used it for IPO. The next time a function in this crate called the
// stdcall `RtlZeroMemory`, LLVM compared the call convention to the visible
// local body, proved contradiction → emitted `unreachable` for the
// remainder of the caller. That `unreachable` quietly folded
// `PeLdrAllocateDataTableEntry` / `PeLdrLoadImage` into 0x30-byte `ud2`
// stubs that triple-faulted i386 boot before SeaBIOS even printed.
//
// The re-export below keeps the `crate::frldr::*` callers compiling without
// re-introducing a duplicate definition.
pub use ntkrnl_sys::rtl::RtlZeroMemory;

/// `ASSERT(cond)` — no-op in the freeldr release build (NDEBUG).
#[inline(always)]
pub fn ASSERT<T>(_cond: T) {}

/// Debug-only: write a single character to QEMU's debugcon (port 0xe9). Lets
/// us trace boot progress past the screen-clear in PcVideoInit without
/// needing serial init. Cheap; safe to leave in.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn dbgcon(c: u8) {
    core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") c, options(nostack, preserves_flags));
}
// ARM64 uses the same firmware-discovered UART as the serial debug path.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn dbgcon(c: u8) {
    extern "C" { fn Rs232PortPutByte(byte: u8); }
    Rs232PortPutByte(c);
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
#[inline(always)]
pub unsafe fn dbgcon(_c: u8) {}

#[inline(always)]
pub unsafe fn dbgcon_str(s: &[u8]) {
    for &c in s { dbgcon(c); }
}

/// Print one byte as 2 hex chars to debugcon.
#[inline(always)]
pub unsafe fn dbgcon_hex8(v: u32) {
    let nib = |n: u32| if n < 10 { b'0' + n as u8 } else { b'A' + (n - 10) as u8 };
    dbgcon(nib((v >> 4) & 0xF));
    dbgcon(nib(v & 0xF));
}

/// Print four bytes (u32) as 8 hex chars to debugcon.
#[inline(always)]
pub unsafe fn dbgcon_hex32(v: u32) {
    dbgcon_hex8(v >> 24);
    dbgcon_hex8(v >> 16);
    dbgcon_hex8(v >> 8);
    dbgcon_hex8(v);
}

/// Print eight bytes (u64) as 16 hex chars to debugcon.
#[inline(always)]
pub unsafe fn dbgcon_hex64(v: u64) {
    dbgcon_hex32((v >> 32) as u32);
    dbgcon_hex32(v as u32);
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

// freeldr layout / PE / heap constants.
pub type POOL_TYPE = u32;
pub const FREELDR_BASE: usize = 0xF800;
pub const FREELDR_PE_BASE: usize = 0x10000;
pub const FREELDR_SECTION_COUNT: usize = 4;
pub const MM_MAX_PAGE_LOADER: PFN_NUMBER = 0xFFFFF;
// x86common.h: MEMORY_MARGIN = HEX(88000), MAX_FREELDR_PE_SIZE = MEMORY_MARGIN
// - FREELDR_PE_BASE - PAGE_SIZE = 0x77000. Earlier rustconv emitted the literal
// 0x70000 instead of the MEMORY_MARGIN macro, capping the PE at 0x5f000 = 380KB.
// The Rust-integrated freeldr_pe.exe is 426KB (.eh_frame/.reloc/.buildid from
// the Rust toolchain), bugchecking on OptionalHeader.SizeOfImage check.
pub const MEMORY_MARGIN: usize = 0x88000;
pub const MAX_FREELDR_PE_SIZE: usize = MEMORY_MARGIN - FREELDR_PE_BASE - MM_PAGE_SIZE;
pub const REDZONE_MARK: u64 = 0xCCCCCCCCCCCCCCCC;
pub const REDZONE_ALLOCATION: usize = 24;
pub const HEAP_ZERO_MEMORY: ULONG = 0x00000008;
pub const MAXUSHORT: USHORT = 0xffff;
// IMAGE_NT_OPTIONAL_HDR_MAGIC: 0x10b = PE32 (i386), 0x20b = PE32+ (amd64/arm64).
// IMAGE_FILE_MACHINE_NATIVE: 0x014c = I386, 0x8664 = AMD64, 0xaa64 = ARM64.
#[cfg(target_arch = "x86")]
pub const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x10b;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const IMAGE_NT_OPTIONAL_HDR_MAGIC: u16 = 0x20b;
#[cfg(target_arch = "x86")]
pub const IMAGE_FILE_MACHINE_NATIVE: u16 = 0x014c;
#[cfg(target_arch = "x86_64")]
pub const IMAGE_FILE_MACHINE_NATIVE: u16 = 0x8664;
#[cfg(target_arch = "aarch64")]
pub const IMAGE_FILE_MACHINE_NATIVE: u16 = 0xaa64;
pub const IMAGE_SUBSYSTEM_NATIVE: u16 = 1;
// _FRLDR_BUGCHECK_CODES enum (debug.h:143). Off-by-one in earlier rustconv
// output dropped TEST_BUGCHECK/MISSING_HARDWARE_REQUIREMENTS — shifted all
// subsequent values down by 2. C: TEST=0, MISSING_HARDWARE=1,
// IMAGE_CORRUPTION=2, MEMORY_INIT_FAILURE=3, EXIT_BOOTSERVICES_FAILURE=4.
pub const TEST_BUGCHECK: ULONG = 0;
pub const MISSING_HARDWARE_REQUIREMENTS: ULONG = 1;
pub const FREELDR_IMAGE_CORRUPTION: ULONG = 2;
pub const MEMORY_INIT_FAILURE: ULONG = 3;
pub const EXIT_BOOTSERVICES_FAILURE: ULONG = 4;

// meminit.c globals the rustconv pass dropped (faithful #[no_mangle]).
#[no_mangle] pub static mut MmLowestPhysicalPage: PFN_NUMBER = 0xFFFFFFFF;
#[no_mangle] pub static mut MmHighestPhysicalPage: PFN_NUMBER = 0;
#[no_mangle] pub static mut BiosMemoryMapEntryCount: ULONG = 0;
#[no_mangle] pub static mut BiosMemoryMap: PFREELDR_MEMORY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle] pub static mut LoaderPagesSpanned: PFN_NUMBER = 0;
// MachVtbl is real C (machpc.c / machine.h) — declared, not owned. C MACHVTBL
// has 30 slots; GetMemoryMap is at index 19. The old _before=22/_after=14
// (=37 slots) was wrong for the current 30-slot machine.h, causing meminit
// to read DiskReadLogicalSectors (slot 22) instead of GetMemoryMap.
// arcemul.c.obj .bss MachVtbl = 0x78 = 120 bytes = 30 * 4.
#[repr(C)]
pub struct MACHVTBL {
    _before: [*const core::ffi::c_void; 19], // 3×Cons + 13×Video + Beep + Prepare + GetMemoryDescriptor
    pub GetMemoryMap: Option<unsafe extern "C" fn(*mut ULONG) -> PFREELDR_MEMORY_DESCRIPTOR>,
    _after: [*const core::ffi::c_void; 10],  // GetExtendedBIOSData..HwIdle
}
const _: () = assert!(
    core::mem::size_of::<MACHVTBL>() == 30 * core::mem::size_of::<*const ()>(),
    "MACHVTBL must hold exactly 30 pointers to match C arcemul.c");
extern "C" { pub static MachVtbl: MACHVTBL; }

// MachInitializeBootDevices() is a macro in C (machine.h):
//   #define MachInitializeBootDevices() MachVtbl.InitializeBootDevices()
// MACHVTBL above only names GetMemoryMap; reach InitializeBootDevices
// (slot 27 — 3 Cons + 13 Video + Beep + Prepare + GetMemoryDescriptor +
// GetMemoryMap + GetExtendedBIOSData + GetFloppyCount + 3 Disk + GetTime +
// GetRelativeTime → InitializeBootDevices) via raw slot index. Shared by
// freeldr.rs (BIOS BootMain) and uefildr.rs (EfiEntry).
#[inline]
pub unsafe fn MachInitializeBootDevices() -> BOOLEAN {
    let vtbl = core::ptr::addr_of!(MachVtbl) as *const *const core::ffi::c_void;
    let slot = *vtbl.add(27);
    let f: extern "C" fn() -> BOOLEAN = core::mem::transmute(slot);
    f()
}

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

/// x86 port-I/O intrinsics. xdk defines READ_PORT_UCHAR/WRITE_PORT_UCHAR as
/// compiler intrinsics (no C symbol); rustconv emitted `extern { fn ... }` +
/// call sites. Real cdecl wrappers — UNIQUE (no C/asm defines them), so
/// keystone-safe. Used by PIT 8254 timer (pchw), CMOS A20/mem probe (pcmem),
/// serial detect (machpc), floppy reset (pcdisk) — must be real `in`/`out`,
/// NOT a no-op (silently dropping these breaks hardware init).
// x86 port-I/O wrappers. arm64 freeldr is UEFI-only and never issues x86
// `in`/`out`; keep the #[no_mangle] symbol (any cross-module reference still
// links) with an unreachable arm64 stub so the asm only compiles for x86.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn WRITE_PORT_UCHAR(port: *mut u8, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port as u16, in("al") val, options(nomem, nostack, preserves_flags));
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn WRITE_PORT_UCHAR(_port: *mut u8, _val: u8) {
    unreachable!("x86 port I/O is not used on the arm64 UEFI boot path");
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn READ_PORT_UCHAR(port: *mut u8) -> u8 {
    let v: u8;
    core::arch::asm!("in al, dx", out("al") v, in("dx") port as u16, options(nomem, nostack, preserves_flags));
    v
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn READ_PORT_UCHAR(_port: *mut u8) -> u8 {
    unreachable!("x86 port I/O is not used on the arm64 UEFI boot path");
}
/// xdk WRITE_PORT_USHORT intrinsic (16-bit `out`). rustconv emitted it
/// as `extern { fn ... }` (pcvideo VGA palette path). UNIQUE — keystone.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn WRITE_PORT_USHORT(port: *mut u16, val: u16) {
    core::arch::asm!("out dx, ax", in("dx") port as u16, in("ax") val, options(nomem, nostack, preserves_flags));
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn WRITE_PORT_USHORT(_port: *mut u16, _val: u16) {
    unreachable!("x86 port I/O is not used on the arm64 UEFI boot path");
}
/// MSVC `__cpuid(int info[4], int leaf)` intrinsic. rustconv emitted it
/// as `extern { fn __cpuid(info: *mut i32, function: i32) }` (winldr
/// CPU detect). LLVM reserves rbx/ebx → save/restore via xchg.
/// **The xchg MUST be full-width on amd64** (`xchg rbx, ...`): a 32-bit
/// `xchg ebx, ...` zero-extends rbx in 64-bit mode, wiping any pointer
/// the caller had stashed there. See feedback_cpuid_xchg_32bit_zext.md.
/// UNIQUE — keystone.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn __cpuid(info: *mut i32, function: i32) {
    let (mut a, mut b, mut c, mut d): (i32, i32, i32, i32);
    core::arch::asm!(
        "xchg {ebx_save:e}, ebx",
        "cpuid",
        "xchg {ebx_save:e}, ebx",
        ebx_save = inout(reg) 0i32 => b,
        inout("eax") function => a,
        out("ecx") c,
        out("edx") d,
        options(nostack, preserves_flags),
    );
    *info.offset(0) = a;
    *info.offset(1) = b;
    *info.offset(2) = c;
    *info.offset(3) = d;
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn __cpuid(info: *mut i32, function: i32) {
    let (mut a, mut c, mut d): (i32, i32, i32);
    let mut b_save: i64;
    core::arch::asm!(
        "xchg {rbx_save}, rbx",
        "cpuid",
        "xchg {rbx_save}, rbx",
        rbx_save = inout(reg) 0i64 => b_save,
        inout("eax") function => a,
        out("ecx") c,
        out("edx") d,
        options(nostack, preserves_flags),
    );
    *info.offset(0) = a;
    *info.offset(1) = b_save as i32; // cpuid only writes low 32 of rbx
    *info.offset(2) = c;
    *info.offset(3) = d;
}
// arm64 has no CPUID; freeldr CPU-detect is x86-only. Keep the symbol.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn __cpuid(_info: *mut i32, _function: i32) {
    unreachable!("__cpuid is not used on the arm64 UEFI boot path");
}
/// xdk/rtlfuncs.h `#define RtlEqualMemory(d,s,l) (!memcmp(d,s,l))`.
/// rustconv emitted it as `extern { fn RtlEqualMemory }` + call sites
/// (iso.rs/fat.rs). Real cdecl wrapper (byte compare; returns TRUE iff
/// equal). UNIQUE — keystone (C macro, no symbol).
#[no_mangle]
pub unsafe extern "C" fn RtlEqualMemory(
    s1: *const core::ffi::c_void,
    s2: *const core::ffi::c_void,
    len: usize,
) -> BOOLEAN {
    let (a, b) = (s1 as *const u8, s2 as *const u8);
    let mut i = 0usize;
    while i < len {
        if *a.add(i) != *b.add(i) {
            return 0;
        }
        i += 1;
    }
    1
}
/// xdk intype.h `UlongToPtr(ul) = ((void*)(ULONG_PTR)ul)` (and the
/// `ULongToPtr` casing variant). rustconv emitted both as
/// `extern { fn ... }`. Real cdecl wrappers. UNIQUE — keystone.
#[no_mangle]
pub unsafe extern "C" fn UlongToPtr(ul: ULONG) -> *mut core::ffi::c_void {
    ul as usize as *mut core::ffi::c_void
}
#[no_mangle]
pub unsafe extern "C" fn ULongToPtr(ul: ULONG) -> *mut core::ffi::c_void {
    ul as usize as *mut core::ffi::c_void
}
// DbgBreakPoint: i386/amd64 lib/debug.c is replaced by Rust, so this empty
// stub is the sole provider there. arm64's still-active C arch/arm64/debug.c
// owns the real `brk #0xF000` implementation (uefi.cmake:41) — exclude the
// Rust stub on aarch64 to avoid a duplicate-symbol cold-link collision.
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle] extern "system" fn DbgBreakPoint() {}

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
// These are the ARC status codes, NOT POSIX errno: sdk/include/reactos/arc/arc.h:30-54
// #undefs the errno names and re-declares them as this enum (EIO=8, ENOEXEC=15, ...),
// so they intentionally differ from errno.h's values in the islands.
pub const ESUCCESS: ULONG = 0;
pub const E2BIG: ULONG = 1; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EACCES: ULONG = 2; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EAGAIN: ULONG = 3; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EBADF: ULONG = 4; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EBUSY: ULONG = 5; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EFAULT: ULONG = 6; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EINVAL: ULONG = 7;  // nolint: const-contradiction (ARC_STATUS enumerator; arc.h #undefs the C EINVAL)
pub const EIO: ULONG = 8; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EISDIR: ULONG = 9; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EMFILE: ULONG = 10; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EMLINK: ULONG = 11; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const ENAMETOOLONG: ULONG = 12; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const ENODEV: ULONG = 13; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const ENOENT: ULONG = 14; // nolint: const-contradiction (ARC_STATUS enumerator; arc.h #undefs the C ENOENT)
pub const ENOEXEC: ULONG = 15; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const ENOMEM: ULONG = 16; // nolint: const-contradiction (ARC_STATUS enumerator; arc.h #undefs the C ENOMEM)
pub const ENOSPC: ULONG = 17; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const ENOTDIR: ULONG = 18; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const ENOTTY: ULONG = 19; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const ENXIO: ULONG = 20; // nolint: const-contradiction (ARC_STATUS, arc.h)
pub const EROFS: ULONG = 21; // nolint: const-contradiction (ARC_STATUS, arc.h)
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

// ===========================================================================
// disk unit — shared partition/MBR types (include/disk.h). rustconv drops
// non-fn header content at scale; restored verbatim. Layout is packed
// (disk.h wraps these in pshpack1/poppack — MBR field offsets prove it).
// ===========================================================================
pub use ntkrnl_sys::ddk::_PARTITION_STYLE as PARTITION_STYLE;
pub const PARTITION_STYLE_MBR: PARTITION_STYLE = 0;
pub const PARTITION_STYLE_GPT: PARTITION_STYLE = 1;
pub const PARTITION_STYLE_BRFR: PARTITION_STYLE = 128;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PARTITION_TABLE_ENTRY {
    pub BootIndicator: UCHAR,
    pub StartHead: UCHAR,
    pub StartSector: UCHAR,
    pub StartCylinder: UCHAR,
    pub SystemIndicator: UCHAR,
    pub EndHead: UCHAR,
    pub EndSector: UCHAR,
    pub EndCylinder: UCHAR,
    pub SectorCountBeforePartition: ULONG,
    pub PartitionSectorCount: ULONG,
}
pub type PPARTITION_TABLE_ENTRY = *mut PARTITION_TABLE_ENTRY;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct MASTER_BOOT_RECORD {
    pub MasterBootRecordCodeAndData: [UCHAR; 0x1b8],
    pub Signature: ULONG,
    pub Reserved: USHORT,
    pub PartitionTable: [PARTITION_TABLE_ENTRY; 4],
    pub MasterBootRecordMagic: USHORT,
}
pub type PMASTER_BOOT_RECORD = *mut MASTER_BOOT_RECORD;

pub const PARTITION_ENTRY_UNUSED: UCHAR = 0x00;
pub const PARTITION_FAT_12: UCHAR = 0x01;
pub const PARTITION_XENIX_1: UCHAR = 0x02;
pub const PARTITION_XENIX_2: UCHAR = 0x03;
pub const PARTITION_FAT_16: UCHAR = 0x04;
pub const PARTITION_EXTENDED: UCHAR = 0x05;
pub const PARTITION_HUGE: UCHAR = 0x06;
pub const PARTITION_IFS: UCHAR = 0x07;
pub const PARTITION_OS2BOOTMGR: UCHAR = 0x0A;
pub const PARTITION_FAT32: UCHAR = 0x0B;
pub const PARTITION_FAT32_XINT13: UCHAR = 0x0C;
pub const PARTITION_XINT13: UCHAR = 0x0E;
pub const PARTITION_XINT13_EXTENDED: UCHAR = 0x0F;
pub const PARTITION_NTFS: UCHAR = 0x17;
pub const PARTITION_PREP: UCHAR = 0x41;
pub const PARTITION_LDM: UCHAR = 0x42;
pub const PARTITION_UNIX: UCHAR = 0x63;
pub const VALID_NTFT: UCHAR = 0xC0;
pub const PARTITION_NTFT: UCHAR = 0x80;
pub const PARTITION_GPT: UCHAR = 0xEE;
pub const PARTITION_STYLE_RAW: PARTITION_STYLE = 2;

// CONFIGURATION_TYPE values used by FILEINFORMATION.Type (ddk enum, but
// rustconv imports the bare names — keystone provides them, unique).
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE as CONFIGURATION_TYPE;
pub const DiskPeripheral: CONFIGURATION_TYPE = 25;
pub const FloppyDiskPeripheral: CONFIGURATION_TYPE = 26;

// arc.h enum constants used by archwsup.c hardware-tree builder + freeldr
// configuration translations. rustconv imports the bare unprefixed names
// (`SystemClass`, `MaximumType`, `Failed`, …) — bindgen prefixes them
// (`_CONFIGURATION_CLASS_SystemClass`, …). Keystone aliases each bare name.
pub const MaximumType: CONFIGURATION_TYPE = 41;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS as CONFIGURATION_CLASS;
pub const SystemClass: CONFIGURATION_CLASS = 0;
pub const ProcessorClass: CONFIGURATION_CLASS = 1;
pub const CacheClass: CONFIGURATION_CLASS = 2;
pub const AdapterClass: CONFIGURATION_CLASS = 3;
pub const ControllerClass: CONFIGURATION_CLASS = 4;
pub const PeripheralClass: CONFIGURATION_CLASS = 5;
pub const MemoryClass: CONFIGURATION_CLASS = 6;
pub const MaximumClass: CONFIGURATION_CLASS = 7;
pub use ntkrnl_sys::ddk::_IDENTIFIER_FLAG as IDENTIFIER_FLAG;
pub const Failed: IDENTIFIER_FLAG = 0x01;
pub const ReadOnly: IDENTIFIER_FLAG = 0x02;
pub const Removable: IDENTIFIER_FLAG = 0x04;
pub const ConsoleIn: IDENTIFIER_FLAG = 0x08;
pub const ConsoleOut: IDENTIFIER_FLAG = 0x10;
pub const Input: IDENTIFIER_FLAG = 0x20;
pub const Output: IDENTIFIER_FLAG = 0x40;

// CCHAR — narrow ANSI char alias used by freeldr settings.c static buffers.
// xdk: `typedef char CCHAR;` (signed on Win32 PE-COFF).
pub type CCHAR = i8;

// arc.h _ARC_DISK_SIGNATURE_EX (freeldr's ntldr/winldr.h extension wrapping
// the kernel's ARC_DISK_SIGNATURE with the bundled ArcName buffer).
// MAX_PATH = 260. Mirror C layout exactly (#[repr(C)]).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ARC_DISK_SIGNATURE_EX {
    pub DiskSignature: ntkrnl_sys::ddk::ARC_DISK_SIGNATURE,
    pub ArcName: [i8; 260],
}
pub type PARC_DISK_SIGNATURE_EX = *mut ARC_DISK_SIGNATURE_EX;

// settings.h _BOOTMGRINFO — boot-manager settings populated by settings.c
// from the command line and freeldr.ini. C declares FrLdrSection as
// ULONG_PTR but the IniOpenSection callers pass &BootMgrInfo.FrLdrSection
// as a (PVOID*) out-param. Layout-identical to ULONG_PTR on all targets
// (sizeof(uintptr_t) == sizeof(void*)); keep as *mut c_void so callers
// don't need raw casts on every use.
#[repr(C)]
pub struct BOOTMGRINFO {
    pub DebugString: ntkrnl_sys::ddk::PCSTR,
    pub VideoOptions: ntkrnl_sys::ddk::PCSTR,
    pub DefaultOs: ntkrnl_sys::ddk::PCSTR,
    pub TimeOut: i32,
    pub FrLdrSection: *mut c_void, // ULONG_PTR (PVOID out-param holder)
}
unsafe impl Sync for BOOTMGRINFO {}
pub type PBOOTMGRINFO = *mut BOOTMGRINFO;

// ===========================================================================
// pc-hw unit — shared real-mode BIOS surface (include/arch/pc/pcbios.h +
// x86common.h). pshpack1 → packed; the Int386 asm thunk reads exact REGS
// offsets (REGS_SIZE=40). Unique names (verified not elsewhere) so
// keystone-safe; the `Int386` extern itself stays file-local per unit.
// ===========================================================================
pub const EFLAGS_CF: ULONG = 0x01;
pub const MAX_BIOS_DESCRIPTORS: usize = 80;
pub const BIOSCALLBUFFER: ULONG = 0x4000;
pub const BIOSCALLBUFSEGMENT: ULONG = 0x4000 / 16;
pub const BIOSCALLBUFOFFSET: ULONG = 0x0000;
pub const BIOSCALLBUFSIZE: ULONG = 0x1000; // PAGE_SIZE
pub const REGS_EAX: ULONG = 0;
pub const REGS_EBX: ULONG = 4;
pub const REGS_ECX: ULONG = 8;
pub const REGS_EDX: ULONG = 12;
pub const REGS_ESI: ULONG = 16;
pub const REGS_EDI: ULONG = 20;
pub const REGS_EBP: ULONG = 24;
pub const REGS_DS: ULONG = 28;
pub const REGS_ES: ULONG = 30;
pub const REGS_FS: ULONG = 32;
pub const REGS_GS: ULONG = 34;
pub const REGS_EFLAGS: ULONG = 36;
pub const REGS_SIZE: ULONG = 40;

// BIOS_MEMORY_TYPE (E820 region kinds).
pub type BIOS_MEMORY_TYPE = u32;
pub const BiosMemoryUsable: BIOS_MEMORY_TYPE = 1;
pub const BiosMemoryReserved: BIOS_MEMORY_TYPE = 2;
pub const BiosMemoryAcpiReclaim: BIOS_MEMORY_TYPE = 3;
pub const BiosMemoryAcpiNvs: BIOS_MEMORY_TYPE = 4;
pub const BiosMemoryUnusable: BIOS_MEMORY_TYPE = 5;
pub const BiosMemoryDisabled: BIOS_MEMORY_TYPE = 6;
pub const BiosMemoryPersistent: BIOS_MEMORY_TYPE = 7;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct DWORDREGS {
    pub eax: u32, pub ebx: u32, pub ecx: u32, pub edx: u32,
    pub esi: u32, pub edi: u32, pub ebp: u32,
    pub ds: u16, pub es: u16, pub fs: u16, pub gs: u16,
    pub eflags: u32,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct WORDREGS {
    pub ax: u16, pub _upper_ax: u16, pub bx: u16, pub _upper_bx: u16,
    pub cx: u16, pub _upper_cx: u16, pub dx: u16, pub _upper_dx: u16,
    pub si: u16, pub _upper_si: u16, pub di: u16, pub _upper_di: u16,
    pub bp: u16, pub _upper_bp: u16,
    pub ds: u16, pub es: u16, pub fs: u16, pub gs: u16,
    pub flags: u16, pub _upper_flags: u16,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BYTEREGS {
    pub al: u8, pub ah: u8, pub _upper_ax: u16,
    pub bl: u8, pub bh: u8, pub _upper_bx: u16,
    pub cl: u8, pub ch: u8, pub _upper_cx: u16,
    pub dl: u8, pub dh: u8, pub _upper_dx: u16,
    pub si: u16, pub _upper_si: u16, pub di: u16, pub _upper_di: u16,
    pub bp: u16, pub _upper_bp: u16,
    pub ds: u16, pub es: u16, pub fs: u16, pub gs: u16,
    pub flags: u16, pub _upper_flags: u16,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union REGS {
    pub x: DWORDREGS,
    pub d: DWORDREGS,
    pub w: WORDREGS,
    pub b: BYTEREGS,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BIOS_MEMORY_MAP {
    pub BaseAddress: u64,
    pub Length: u64,
    pub Type: ULONG,
    pub ExtendedAttributesAsULONG: ULONG,
}
pub type PBIOS_MEMORY_MAP = *mut BIOS_MEMORY_MAP;

// C macro: INT386_SUCCESS(regs) == ((regs.x.eflags & EFLAGS_CF) == 0)
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn INT386_SUCCESS(regs: REGS) -> BOOLEAN {
    ((regs.x.eflags & EFLAGS_CF) == 0) as BOOLEAN
}

// Misc shared pc/hw symbols (unique — keystone-safe).
pub type INT = i32;
pub const TAG_HW_RESOURCE_LIST: ULONG = u32::from_be_bytes(*b"lRwH");
// CM_RESOURCE_TYPE (cmtypes.h): CmResourceTypeDeviceSpecific is 5, NOT 0x81.
// 0x81 (129) is CmResourceTypeDevicePrivate — a wrong value here made
// uefihw::DetectDisplayController tag the framebuffer's DeviceSpecific
// descriptor as 0x81, so arm64vid.sys (checking ==5) skipped it, never read
// the CM_FRAMEBUF_DEVICE_DATA (ScreenWidth stayed 0), failed discovery, and
// bugchecked 0xB4 VIDEO_DRIVER_INIT_FAILURE. machpc.rs already had it right.
pub const CmResourceTypeDeviceSpecific: u8 = 5; // CM_RESOURCE_TYPE
// arc.h IDENTIFIER_FLAG bits not already pulled from ddk by consumers.
pub const IDENTIFIER_FLAG_Input: i32 = 0x20;
pub const IDENTIFIER_FLAG_Output: i32 = 0x40;

// C `FIELD_OFFSET(t,f)` macro — rustconv emits it as a call; the
// freeldr_fieldoffset.py pass rewrites call→macro!. offset_of! → ULONG.
#[macro_export]
macro_rules! FIELD_OFFSET {
    // FIELD_OFFSET(t, arr[i]) — flexible-array element offset idiom.
    ($t:ty, $f:ident [ $i:expr ]) => {{
        let _u: ::core::mem::MaybeUninit<$t> = ::core::mem::MaybeUninit::uninit();
        let _b = _u.as_ptr();
        (::core::mem::offset_of!($t, $f)
            + ($i as usize) * ::core::mem::size_of_val(unsafe { &(*_b).$f[0] })) as u32
    }};
    ($t:ty, $f:tt) => { ::core::mem::offset_of!($t, $f) as u32 };
}
// C `RTL_SIZEOF_THROUGH_FIELD(t,f)` == offsetof(t,f)+sizeof(((t*)0)->f).
#[macro_export]
macro_rules! RTL_SIZEOF_THROUGH_FIELD {
    ($t:ty, $f:tt) => {{
        // packed-safe: never take a reference to the (possibly unaligned)
        // field — derive its size from the raw-pointer pointee type.
        const fn __fsz<T>(_: *const T) -> usize { ::core::mem::size_of::<T>() }
        let _u: ::core::mem::MaybeUninit<$t> = ::core::mem::MaybeUninit::uninit();
        let _b = _u.as_ptr();
        ::core::mem::offset_of!($t, $f)
            + __fsz(unsafe { ::core::ptr::addr_of!((*_b).$f) })
    }};
}

// ===========================================================================
// EDK2 / UEFI surface — shared by all arch/uefi/* translation units.
// Mirrors vendor/edk2/*.h exactly. EFIAPI == cdecl on i386,
// so every EFI function pointer is `Option<unsafe extern "C" fn ...>`.
// Service-table slots not called by freeldr are kept as pointer-sized
// opaque `*mut c_void` so the struct field offsets remain C-exact (the
// EFI_SYSTEM_TABLE / EFI_BOOT_SERVICES are the real firmware tables).
// EFI structs are naturally aligned (NOT packed); the freeldr-local
// GPT_* structs are #include <pshpack1.h> → #[repr(C, packed(1))].
// ===========================================================================

// --- ProcessorBind.h / Base.h primitive types (i386: UINTN == UINT32) ---
// UINT8/UINT16/UINT32/VOID already defined above (CRT-ish aliases block).
pub type UINT64 = u64;
pub type INT16 = i16;
pub type INT32 = i32;
// UINTN/INTN are native UEFI pointer width — UINT64 on amd64+arm64,
// UINT32 on i386. The 32-bit-everywhere hardcoding "worked" on amd64
// (high stack-slot bytes happened to zero) but BREAKS arm64 freeldr's
// GetMemoryMap retry loop: UEFI tries to write the 8-byte required size
// into our 4-byte slot, overflowing into adjacent stack vars and corrupting
// MapKey/DescriptorSize. Root-caused 2026-05-28 via dbgcon PL011 trace.
#[cfg(target_pointer_width = "64")]
pub type UINTN = u64;
#[cfg(target_pointer_width = "32")]
pub type UINTN = u32;
#[cfg(target_pointer_width = "64")]
pub type INTN = i64;
#[cfg(target_pointer_width = "32")]
pub type INTN = i32;
pub type CHAR8 = i8;
pub type CHAR16 = u16;
pub type BOOLEAN_EFI = u8;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_GUID {
    pub Data1: UINT32,
    pub Data2: UINT16,
    pub Data3: UINT16,
    pub Data4: [UINT8; 8],
}

pub type EFI_STATUS = UINTN; // typedef RETURN_STATUS(=UINTN) EFI_STATUS;
pub type EFI_HANDLE = *mut c_void;
pub type EFI_EVENT = *mut c_void;
pub type EFI_LBA = UINT64;
pub type EFI_TPL = UINTN;
pub type EFI_PHYSICAL_ADDRESS = UINT64;
pub type EFI_VIRTUAL_ADDRESS = UINT64;

// Base.h: ENCODE_ERROR(x) = MAX_BIT | x; i386 MAX_BIT = 0x8000_0000.
pub const EFI_MAX_BIT: EFI_STATUS = 0x8000_0000;
pub const EFI_SUCCESS: EFI_STATUS = 0;
pub const EFI_UNSUPPORTED: EFI_STATUS = EFI_MAX_BIT | 3;
pub const EFI_NOT_READY: EFI_STATUS = EFI_MAX_BIT | 6;
pub const EFI_NOT_FOUND: EFI_STATUS = EFI_MAX_BIT | 14;
// EFI_ERROR(x) == high bit set.
#[inline]
pub unsafe extern "C" fn EFI_ERROR(Status: EFI_STATUS) -> BOOLEAN {
    if (Status & EFI_MAX_BIT) != 0 { 1 } else { 0 }
}

// UefiMultiPhase.h — EFI_MEMORY_TYPE (C enum, int width).
pub type EFI_MEMORY_TYPE = u32;
pub const EfiReservedMemoryType: EFI_MEMORY_TYPE = 0;
pub const EfiLoaderCode: EFI_MEMORY_TYPE = 1;
pub const EfiLoaderData: EFI_MEMORY_TYPE = 2;
pub const EfiBootServicesCode: EFI_MEMORY_TYPE = 3;
pub const EfiBootServicesData: EFI_MEMORY_TYPE = 4;
pub const EfiRuntimeServicesCode: EFI_MEMORY_TYPE = 5;
pub const EfiRuntimeServicesData: EFI_MEMORY_TYPE = 6;
pub const EfiConventionalMemory: EFI_MEMORY_TYPE = 7;
pub const EfiUnusableMemory: EFI_MEMORY_TYPE = 8;
pub const EfiACPIReclaimMemory: EFI_MEMORY_TYPE = 9;
pub const EfiACPIMemoryNVS: EFI_MEMORY_TYPE = 10;
pub const EfiMemoryMappedIO: EFI_MEMORY_TYPE = 11;
pub const EfiMemoryMappedIOPortSpace: EFI_MEMORY_TYPE = 12;
pub const EfiPalCode: EFI_MEMORY_TYPE = 13;
pub const EfiMaxMemoryType: EFI_MEMORY_TYPE = 14;

pub type EFI_ALLOCATE_TYPE = u32;
pub const AllocateAnyPages: EFI_ALLOCATE_TYPE = 0;
pub const AllocateMaxAddress: EFI_ALLOCATE_TYPE = 1;
pub const AllocateAddress: EFI_ALLOCATE_TYPE = 2;

pub type EFI_LOCATE_SEARCH_TYPE = u32;
pub const AllHandles: EFI_LOCATE_SEARCH_TYPE = 0;
pub const ByRegisterNotify: EFI_LOCATE_SEARCH_TYPE = 1;
pub const ByProtocol: EFI_LOCATE_SEARCH_TYPE = 2;

pub const EFI_PAGE_SIZE: UINTN = 0x1000;
pub const EFI_PAGE_SHIFT: UINTN = 12;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_MEMORY_DESCRIPTOR {
    pub Type: UINT32,
    pub PhysicalStart: EFI_PHYSICAL_ADDRESS,
    pub VirtualStart: EFI_VIRTUAL_ADDRESS,
    pub NumberOfPages: UINT64,
    pub Attribute: UINT64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_TABLE_HEADER {
    pub Signature: UINT64,
    pub Revision: UINT32,
    pub HeaderSize: UINT32,
    pub CRC32: UINT32,
    pub Reserved: UINT32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_TIME {
    pub Year: UINT16,
    pub Month: UINT8,
    pub Day: UINT8,
    pub Hour: UINT8,
    pub Minute: UINT8,
    pub Second: UINT8,
    pub Pad1: UINT8,
    pub Nanosecond: UINT32,
    pub TimeZone: INT16,
    pub Daylight: UINT8,
    pub Pad2: UINT8,
}

// --- BlockIo.h ---
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_BLOCK_IO_MEDIA {
    pub MediaId: UINT32,
    pub RemovableMedia: BOOLEAN_EFI,
    pub MediaPresent: BOOLEAN_EFI,
    pub LogicalPartition: BOOLEAN_EFI,
    pub ReadOnly: BOOLEAN_EFI,
    pub WriteCaching: BOOLEAN_EFI,
    pub BlockSize: UINT32,
    pub IoAlign: UINT32,
    pub LastBlock: EFI_LBA,
    pub LowestAlignedLba: EFI_LBA,
    pub LogicalBlocksPerPhysicalBlock: UINT32,
    pub OptimalTransferLengthGranularity: UINT32,
}

pub type EFI_BLOCK_RESET = Option<
    unsafe extern "C" fn(This: *mut EFI_BLOCK_IO_PROTOCOL, ExtendedVerification: BOOLEAN_EFI) -> EFI_STATUS,
>;
pub type EFI_BLOCK_READ = Option<
    unsafe extern "C" fn(
        This: *mut EFI_BLOCK_IO_PROTOCOL,
        MediaId: UINT32,
        Lba: EFI_LBA,
        BufferSize: UINTN,
        Buffer: *mut c_void,
    ) -> EFI_STATUS,
>;
pub type EFI_BLOCK_WRITE = Option<
    unsafe extern "C" fn(
        This: *mut EFI_BLOCK_IO_PROTOCOL,
        MediaId: UINT32,
        Lba: EFI_LBA,
        BufferSize: UINTN,
        Buffer: *mut c_void,
    ) -> EFI_STATUS,
>;
pub type EFI_BLOCK_FLUSH =
    Option<unsafe extern "C" fn(This: *mut EFI_BLOCK_IO_PROTOCOL) -> EFI_STATUS>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_BLOCK_IO_PROTOCOL {
    pub Revision: UINT64,
    pub Media: *mut EFI_BLOCK_IO_MEDIA,
    pub Reset: EFI_BLOCK_RESET,
    pub ReadBlocks: EFI_BLOCK_READ,
    pub WriteBlocks: EFI_BLOCK_WRITE,
    pub FlushBlocks: EFI_BLOCK_FLUSH,
}
pub type EFI_BLOCK_IO = EFI_BLOCK_IO_PROTOCOL;

// --- GraphicsOutput.h ---
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_PIXEL_BITMASK {
    pub RedMask: UINT32,
    pub GreenMask: UINT32,
    pub BlueMask: UINT32,
    pub ReservedMask: UINT32,
}
pub type PPIXEL_BITMASK = *mut EFI_PIXEL_BITMASK;
pub type PEFI_PIXEL_BITMASK = *mut EFI_PIXEL_BITMASK;

pub type EFI_GRAPHICS_PIXEL_FORMAT = u32;
pub const PixelRedGreenBlueReserved8BitPerColor: EFI_GRAPHICS_PIXEL_FORMAT = 0;
pub const PixelBlueGreenRedReserved8BitPerColor: EFI_GRAPHICS_PIXEL_FORMAT = 1;
pub const PixelBitMask: EFI_GRAPHICS_PIXEL_FORMAT = 2;
pub const PixelBltOnly: EFI_GRAPHICS_PIXEL_FORMAT = 3;
pub const PixelFormatMax: EFI_GRAPHICS_PIXEL_FORMAT = 4;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_GRAPHICS_OUTPUT_BLT_PIXEL {
    pub Blue: UINT8,
    pub Green: UINT8,
    pub Red: UINT8,
    pub Reserved: UINT8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_GRAPHICS_OUTPUT_MODE_INFORMATION {
    pub Version: UINT32,
    pub HorizontalResolution: UINT32,
    pub VerticalResolution: UINT32,
    pub PixelFormat: EFI_GRAPHICS_PIXEL_FORMAT,
    pub PixelInformation: EFI_PIXEL_BITMASK,
    pub PixelsPerScanLine: UINT32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE {
    pub MaxMode: UINT32,
    pub Mode: UINT32,
    pub Info: *mut EFI_GRAPHICS_OUTPUT_MODE_INFORMATION,
    pub SizeOfInfo: UINTN,
    pub FrameBufferBase: EFI_PHYSICAL_ADDRESS,
    pub FrameBufferSize: UINTN,
}

pub type EFI_GRAPHICS_OUTPUT_PROTOCOL_QUERY_MODE = Option<
    unsafe extern "C" fn(
        This: *mut EFI_GRAPHICS_OUTPUT_PROTOCOL,
        ModeNumber: UINT32,
        SizeOfInfo: *mut UINTN,
        Info: *mut *mut EFI_GRAPHICS_OUTPUT_MODE_INFORMATION,
    ) -> EFI_STATUS,
>;
pub type EFI_GRAPHICS_OUTPUT_PROTOCOL_SET_MODE = Option<
    unsafe extern "C" fn(This: *mut EFI_GRAPHICS_OUTPUT_PROTOCOL, ModeNumber: UINT32) -> EFI_STATUS,
>;
pub type EFI_GRAPHICS_OUTPUT_PROTOCOL_BLT = Option<
    unsafe extern "C" fn(
        This: *mut EFI_GRAPHICS_OUTPUT_PROTOCOL,
        BltBuffer: *mut EFI_GRAPHICS_OUTPUT_BLT_PIXEL,
        BltOperation: u32,
        SourceX: UINTN,
        SourceY: UINTN,
        DestinationX: UINTN,
        DestinationY: UINTN,
        Width: UINTN,
        Height: UINTN,
        Delta: UINTN,
    ) -> EFI_STATUS,
>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_GRAPHICS_OUTPUT_PROTOCOL {
    pub QueryMode: EFI_GRAPHICS_OUTPUT_PROTOCOL_QUERY_MODE,
    pub SetMode: EFI_GRAPHICS_OUTPUT_PROTOCOL_SET_MODE,
    pub Blt: EFI_GRAPHICS_OUTPUT_PROTOCOL_BLT,
    pub Mode: *mut EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE,
}

// --- SimpleTextIn.h ---
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_INPUT_KEY {
    pub ScanCode: UINT16,
    pub UnicodeChar: CHAR16,
}

pub type EFI_INPUT_RESET = Option<
    unsafe extern "C" fn(
        This: *mut EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
        ExtendedVerification: BOOLEAN_EFI,
    ) -> EFI_STATUS,
>;
pub type EFI_INPUT_READ_KEY = Option<
    unsafe extern "C" fn(
        This: *mut EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
        Key: *mut EFI_INPUT_KEY,
    ) -> EFI_STATUS,
>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_SIMPLE_TEXT_INPUT_PROTOCOL {
    pub Reset: EFI_INPUT_RESET,
    pub ReadKeyStroke: EFI_INPUT_READ_KEY,
    pub WaitForKey: EFI_EVENT,
}

// --- LoadedImage.h ---
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_LOADED_IMAGE_PROTOCOL {
    pub Revision: UINT32,
    pub ParentHandle: EFI_HANDLE,
    pub SystemTable: *mut EFI_SYSTEM_TABLE,
    pub DeviceHandle: EFI_HANDLE,
    pub FilePath: *mut c_void, // EFI_DEVICE_PATH_PROTOCOL*
    pub Reserved: *mut c_void,
    pub LoadOptionsSize: UINT32,
    pub LoadOptions: *mut c_void,
    pub ImageBase: *mut c_void,
    pub ImageSize: UINT64,
    pub ImageCodeType: EFI_MEMORY_TYPE,
    pub ImageDataType: EFI_MEMORY_TYPE,
    pub Unload: *mut c_void, // EFI_IMAGE_UNLOAD
}

// --- UefiSpec.h service tables. Called slots get real signatures;
//     the rest are pointer-sized opaques to keep the layout C-exact. ---
pub type EFI_ALLOCATE_PAGES = Option<
    unsafe extern "C" fn(
        Type: EFI_ALLOCATE_TYPE,
        MemoryType: EFI_MEMORY_TYPE,
        Pages: UINTN,
        Memory: *mut EFI_PHYSICAL_ADDRESS,
    ) -> EFI_STATUS,
>;
pub type EFI_GET_MEMORY_MAP = Option<
    unsafe extern "C" fn(
        MemoryMapSize: *mut UINTN,
        MemoryMap: *mut EFI_MEMORY_DESCRIPTOR,
        MapKey: *mut UINTN,
        DescriptorSize: *mut UINTN,
        DescriptorVersion: *mut UINT32,
    ) -> EFI_STATUS,
>;
pub type EFI_ALLOCATE_POOL = Option<
    unsafe extern "C" fn(
        PoolType: EFI_MEMORY_TYPE,
        Size: UINTN,
        Buffer: *mut *mut c_void,
    ) -> EFI_STATUS,
>;
pub type EFI_FREE_POOL = Option<unsafe extern "C" fn(Buffer: *mut c_void) -> EFI_STATUS>;
pub type EFI_EXIT_BOOT_SERVICES =
    Option<unsafe extern "C" fn(ImageHandle: EFI_HANDLE, MapKey: UINTN) -> EFI_STATUS>;
pub type EFI_HANDLE_PROTOCOL = Option<
    unsafe extern "C" fn(
        Handle: EFI_HANDLE,
        Protocol: *mut EFI_GUID,
        Interface: *mut *mut c_void,
    ) -> EFI_STATUS,
>;
pub type EFI_LOCATE_HANDLE = Option<
    unsafe extern "C" fn(
        SearchType: EFI_LOCATE_SEARCH_TYPE,
        Protocol: *mut EFI_GUID,
        SearchKey: *mut c_void,
        BufferSize: *mut UINTN,
        Buffer: *mut EFI_HANDLE,
    ) -> EFI_STATUS,
>;
pub type EFI_LOCATE_PROTOCOL = Option<
    unsafe extern "C" fn(
        Protocol: *mut EFI_GUID,
        Registration: *mut c_void,
        Interface: *mut *mut c_void,
    ) -> EFI_STATUS,
>;
pub type EFI_GET_TIME = Option<
    unsafe extern "C" fn(Time: *mut EFI_TIME, Capabilities: *mut c_void) -> EFI_STATUS,
>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_BOOT_SERVICES {
    pub Hdr: EFI_TABLE_HEADER,
    // TPL services
    pub RaiseTPL: *mut c_void,
    pub RestoreTPL: *mut c_void,
    // Memory services
    pub AllocatePages: EFI_ALLOCATE_PAGES,
    pub FreePages: *mut c_void,
    pub GetMemoryMap: EFI_GET_MEMORY_MAP,
    pub AllocatePool: EFI_ALLOCATE_POOL,
    pub FreePool: EFI_FREE_POOL,
    // Event & timer services
    pub CreateEvent: *mut c_void,
    pub SetTimer: *mut c_void,
    pub WaitForEvent: *mut c_void,
    pub SignalEvent: *mut c_void,
    pub CloseEvent: *mut c_void,
    pub CheckEvent: *mut c_void,
    // Protocol handler services
    pub InstallProtocolInterface: *mut c_void,
    pub ReinstallProtocolInterface: *mut c_void,
    pub UninstallProtocolInterface: *mut c_void,
    pub HandleProtocol: EFI_HANDLE_PROTOCOL,
    pub Reserved: *mut c_void,
    pub RegisterProtocolNotify: *mut c_void,
    pub LocateHandle: EFI_LOCATE_HANDLE,
    pub LocateDevicePath: *mut c_void,
    pub InstallConfigurationTable: *mut c_void,
    // Image services
    pub LoadImage: *mut c_void,
    pub StartImage: *mut c_void,
    pub Exit: *mut c_void,
    pub UnloadImage: *mut c_void,
    pub ExitBootServices: EFI_EXIT_BOOT_SERVICES,
    // Misc services
    pub GetNextMonotonicCount: *mut c_void,
    pub Stall: *mut c_void,
    pub SetWatchdogTimer: *mut c_void,
    // DriverSupport services
    pub ConnectController: *mut c_void,
    pub DisconnectController: *mut c_void,
    // Open & close protocol services
    pub OpenProtocol: *mut c_void,
    pub CloseProtocol: *mut c_void,
    pub OpenProtocolInformation: *mut c_void,
    // Library services
    pub ProtocolsPerHandle: *mut c_void,
    pub LocateHandleBuffer: *mut c_void,
    pub LocateProtocol: EFI_LOCATE_PROTOCOL,
    pub InstallMultipleProtocolInterfaces: *mut c_void,
    pub UninstallMultipleProtocolInterfaces: *mut c_void,
    // 32-bit CRC services
    pub CalculateCrc32: *mut c_void,
    // Misc services
    pub CopyMem: *mut c_void,
    pub SetMem: *mut c_void,
    pub CreateEventEx: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_RUNTIME_SERVICES {
    pub Hdr: EFI_TABLE_HEADER,
    // Time services
    pub GetTime: EFI_GET_TIME,
    pub SetTime: *mut c_void,
    pub GetWakeupTime: *mut c_void,
    pub SetWakeupTime: *mut c_void,
    // Virtual memory services
    pub SetVirtualAddressMap: *mut c_void,
    pub ConvertPointer: *mut c_void,
    // Variable services
    pub GetVariable: *mut c_void,
    pub GetNextVariableName: *mut c_void,
    pub SetVariable: *mut c_void,
    // Misc services
    pub GetNextHighMonotonicCount: *mut c_void,
    pub ResetSystem: *mut c_void,
    // UEFI 2.0 capsule services
    pub UpdateCapsule: *mut c_void,
    pub QueryCapsuleCapabilities: *mut c_void,
    // Misc UEFI 2.0 service
    pub QueryVariableInfo: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_CONFIGURATION_TABLE {
    pub VendorGuid: EFI_GUID,
    pub VendorTable: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EFI_SYSTEM_TABLE {
    pub Hdr: EFI_TABLE_HEADER,
    pub FirmwareVendor: *mut CHAR16,
    pub FirmwareRevision: UINT32,
    pub ConsoleInHandle: EFI_HANDLE,
    pub ConIn: *mut EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
    pub ConsoleOutHandle: EFI_HANDLE,
    pub ConOut: *mut c_void, // EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL*
    pub StandardErrorHandle: EFI_HANDLE,
    pub StdErr: *mut c_void,
    pub RuntimeServices: *mut EFI_RUNTIME_SERVICES,
    pub BootServices: *mut EFI_BOOT_SERVICES,
    pub NumberOfTableEntries: UINTN,
    pub ConfigurationTable: *mut EFI_CONFIGURATION_TABLE,
}

// Protocol GUID constants (vendor/edk2/*.h).
pub const EFI_BLOCK_IO_PROTOCOL_GUID: EFI_GUID = EFI_GUID {
    Data1: 0x964e5b21, Data2: 0x6459, Data3: 0x11d2,
    Data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};
pub const EFI_LOADED_IMAGE_PROTOCOL_GUID: EFI_GUID = EFI_GUID {
    Data1: 0x5B1B31A1, Data2: 0x9562, Data3: 0x11d2,
    Data4: [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
};
pub const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EFI_GUID = EFI_GUID {
    Data1: 0x9042a9de, Data2: 0x23dc, Data3: 0x4a38,
    Data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};
pub const EFI_ACPI_TABLE_GUID: EFI_GUID = EFI_GUID {
    Data1: 0x8868e871, Data2: 0xe4f1, Data3: 0x11d3,
    Data4: [0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81],
};
pub const EFI_ACPI_20_TABLE_GUID: EFI_GUID = EFI_ACPI_TABLE_GUID;

// winldr.h — Root System Descriptor Pointer (pshpack1).
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct RSDP_DESCRIPTOR {
    pub signature: [i8; 8],
    pub checksum: UCHAR,
    pub oem_id: [i8; 6],
    pub revision: UCHAR,
    pub rsdt_physical_address: ULONG,
    pub length: ULONG,
    pub xsdt_physical_address: u64,
    pub extended_checksum: UCHAR,
    pub reserved: [i8; 3],
}
pub type PRSDP_DESCRIPTOR = *mut RSDP_DESCRIPTOR;

// uefildr.h — ACPI_BIOS_DATA (pshpack1). BIOS_MEMORY_MAP / PBIOS_MEMORY_MAP
// already defined above (REGS/Int386 block); reuse them here.
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct ACPI_BIOS_DATA {
    // PHYSICAL_ADDRESS == LARGE_INTEGER (bindgen union: .QuadPart / .u...LowPart).
    pub RSDTAddress: ntkrnl_sys::ddk::_LARGE_INTEGER,
    pub Count: u64,
    pub MemoryMap: [BIOS_MEMORY_MAP; 1],
}
pub type PACPI_BIOS_DATA = *mut ACPI_BIOS_DATA;

// VIDEODISPLAYMODE / VideoTextMode / VideoGraphicsMode live in
// ui_keystone.rs (don't duplicate — would alias-clash in ui.rs glob).
// uefivid.rs defines them file-local (it only globs crate::frldr).

// cmtypes.h CM_VIDEO_DEVICE_DATA + bootvid/framebuf.h CM_FRAMEBUF_DEVICE_DATA.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CM_FRAMEBUF_DEVICE_DATA {
    // anonymous CM_VIDEO_DEVICE_DATA member, flattened
    pub Version: USHORT,
    pub Revision: USHORT,
    pub VideoClock: ULONG,
    pub FrameBufferOffset: ULONG,
    pub ScreenWidth: ULONG,
    pub ScreenHeight: ULONG,
    pub PixelsPerScanLine: ULONG,
    pub BitsPerPixel: ULONG,
    pub PixelMasks: CM_FRAMEBUF_PIXEL_MASKS,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CM_FRAMEBUF_PIXEL_MASKS {
    pub RedMask: ULONG,
    pub GreenMask: ULONG,
    pub BlueMask: ULONG,
    pub ReservedMask: ULONG,
}
pub type PCM_FRAMEBUF_DEVICE_DATA = *mut CM_FRAMEBUF_DEVICE_DATA;

// ---------------------------------------------------------------------
// BIOS-only realmode-thunk symbols — per-binary-owner UEFI stubs.
//
// Int386 (arch/i386/int386.S), Relocator16Boot (arch/i386/entry.S),
// PnpBios* (arch/i386/pnpbios.S), DetectApmBios (arch/i386/hwapm.c),
// PcConsGetCh (arch/i386/pc/pccons.c) are BIOS-only realmode asm/C and
// are NOT compiled into the uefildr link unit. They are referenced only
// by the PC-hardware Rust modules (pcvideo/pcvesa/pcmem/pcdisk/hwpci/
// machpc), whose `Pc*` exports are consumed solely by the BIOS MachVtbl
// installed in the already-`#[cfg(not(feature="uefi"))]`-gated
// `MachInit`. uefildr fills MachVtbl from uefisetup.c (UEFI C) instead,
// so this PC code is unreachable dead code there — but the linker still
// needs the extern symbols resolved. Same per-binary-owner pattern as
// FrldrBootDrive / PxeCallApi: under `not(uefi)` the BIOS asm/C owns
// them (extern, declared file-locally per unit); under `uefi` the
// keystone owns inert stubs (UNIQUE — verified no other #[no_mangle]
// def in freeldr-rust). All __cdecl. The pc* callers are never invoked
// at uefildr runtime (UEFI MachVtbl), so the stub bodies' return values
// are inert.
#[cfg(feature = "uefi")]
#[no_mangle]
pub unsafe extern "C" fn Int386(_interrupt: i32, _in_regs: *mut REGS, _out_regs: *mut REGS) -> i32 {
    0
}
#[cfg(feature = "uefi")]
#[no_mangle]
pub unsafe extern "C" fn Relocator16Boot(
    _Regs: *mut REGS,
    _StackSegment: u16,
    _StackPointer: u16,
    _CodeSegment: u16,
    _CodePointer: u16,
) {
}
#[cfg(feature = "uefi")]
#[no_mangle]
pub unsafe extern "C" fn DetectApmBios(_SystemKey: *mut c_void, _BusNumber: *mut ULONG) {
}
#[cfg(feature = "uefi")]
#[no_mangle]
pub unsafe extern "C" fn PnpBiosSupported() -> usize {
    0
}
#[cfg(feature = "uefi")]
#[no_mangle]
pub unsafe extern "C" fn PnpBiosGetDeviceNodeCount(
    _NodeSize: *mut ULONG,
    _NodeCount: *mut ULONG,
) -> ULONG {
    1 /* non-zero = "error/none" per pnpbios.S convention */
}
#[cfg(feature = "uefi")]
#[no_mangle]
pub unsafe extern "C" fn PnpBiosGetDeviceNode(_NodeId: *mut UCHAR, _NodeBuffer: *mut UCHAR) -> ULONG {
    1
}
#[cfg(feature = "uefi")]
#[no_mangle]
pub unsafe extern "C" fn PnpBiosGetDockStationInformation(_DockingStationInfo: *mut UCHAR) -> ULONG {
    1
}
#[cfg(feature = "uefi")]
#[no_mangle]
pub unsafe extern "C" fn PcConsGetCh() -> i32 {
    0
}
