//! FreeLDR FFI keystone for the peloader translation — the C surface
//! rustconv/the LLM can't synthesize: freeldr-internal externs (exact
//! ABI from peloader.c.obj), the PE/address macros, the debug-channel
//! macros, and the two peloader.c-defined globals.
//!
//! ABI split is authoritative (nm of build-i386 peloader.c.obj):
//!  - cdecl  (`extern "C"`):  Arc*, DbgPrint2, FrLdrHeap*, Mm*, mem*,
//!                            str*, toupper
//!  - stdcall(`extern "system"`): RtlImageNtHeader@4,
//!    RtlImageDirectoryEntryToData@16, LdrRelocateImageWithBias@28,
//!    RtlAssert@16  (the AI imported these cdecl from ntkrnl_sys — the
//!    classic i386 ABI bug; see feedback_extern_c_abi).

#![allow(non_snake_case, non_upper_case_globals)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{LIST_ENTRY, PIMAGE_NT_HEADERS};
use core::ffi::c_void;

// ---------------------------------------------------------------------
// conversion.h FORCEINLINE VaToPa/PaToVa (non-_ZOOM2_ variant):
//   VaToPa(Va) = Va & ~KSEG0_BASE ;  PaToVa(Pa) = Pa | KSEG0_BASE
// KSEG0_BASE: i386 0x8000_0000 ; amd64/arm64 0xFFFF_F800_0000_0000.
// ---------------------------------------------------------------------
#[cfg(target_arch = "x86")]
pub const KSEG0_BASE: usize = 0x8000_0000;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const KSEG0_BASE: usize = 0xFFFF_F800_0000_0000;

// Generic over pointer type — C's `VaToPa(PVOID)` accepts/returns any
// pointer implicitly; the mask is pointer-width-agnostic. Generic keeps
// the caller's pointee type (no per-call-site cast churn).
#[inline]
pub unsafe fn VaToPa<T>(va: *const T) -> *mut T {
    ((va as usize) & !KSEG0_BASE) as *mut T
}
#[inline]
pub unsafe fn PaToVa<T>(pa: *const T) -> *mut T {
    ((pa as usize) | KSEG0_BASE) as *mut T
}

/// ntimage.h `IMAGE_FIRST_SECTION(NtHeader)` macro.
#[inline]
pub unsafe fn IMAGE_FIRST_SECTION(nt: PIMAGE_NT_HEADERS) -> ntkrnl_sys::ddk::PIMAGE_SECTION_HEADER {
    // (ULONG_PTR)nt + FIELD_OFFSET(IMAGE_NT_HEADERS, OptionalHeader)
    //            + FileHeader.SizeOfOptionalHeader
    let opt_off = core::mem::offset_of!(
        ntkrnl_sys::ddk::_IMAGE_NT_HEADERS,
        OptionalHeader
    );
    ((nt as usize)
        + opt_off
        + (*nt).FileHeader.SizeOfOptionalHeader as usize)
        as ntkrnl_sys::ddk::PIMAGE_SECTION_HEADER
}

/// freeldr.h `#define RVA(m, b) ((PVOID)((ULONG_PTR)(b)+(ULONG_PTR)(m)))`
/// — commutative add; callers pass (base, rva).
#[inline]
pub unsafe fn RVA(a: PVOID, b: u32) -> PVOID {
    ((a as usize).wrapping_add(b as usize)) as PVOID
}

// ntimage.h PE ordinal macros (arch-width).
#[cfg(target_arch = "x86")]
pub const IMAGE_ORDINAL_FLAG: usize = 0x8000_0000;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const IMAGE_ORDINAL_FLAG: usize = 0x8000_0000_0000_0000;

#[inline]
pub fn IMAGE_SNAP_BY_ORDINAL(ordinal: usize) -> u8 {
    ((ordinal & IMAGE_ORDINAL_FLAG) != 0) as u8
}
#[inline]
pub fn IMAGE_ORDINAL(ordinal: usize) -> usize {
    ordinal & 0xffff
}

// ---------------------------------------------------------------------
// Debug-channel macros. TRACE/WARN/FIXME are DbgPrint2 calls gated on a
// per-file channel mask (compiled out entirely in the NDEBUG freeldr
// build); ERR is the error path. None affects boot behavior — model as
// no-ops that swallow the varargs (faithful to the NDEBUG build;
// functionally inert when DBG since DbgPrint2 is mask-gated anyway).
// Call sites are rewritten `TRACE(...)` → `TRACE!(...)`.
// ---------------------------------------------------------------------
#[macro_export]
macro_rules! TRACE { ($($a:tt)*) => {{}} }
#[macro_export]
macro_rules! WARN  { ($($a:tt)*) => {{}} }
#[macro_export]
macro_rules! ERR   { ($($a:tt)*) => {{}} }
#[macro_export]
macro_rules! FIXME { ($($a:tt)*) => {{}} }

// ---------------------------------------------------------------------
// freeldr-internal C externs — cdecl. (ARC_STATUS/OPENMODE/SEEKMODE/
// TYPE_OF_MEMORY are C enums == i32-width → u32.)
// ---------------------------------------------------------------------
pub type ARC_STATUS = u32;

// --- constants the AI used but rustconv didn't import/define ---
// ARC_CODES (arc.h enum order): ESUCCESS=0 … ENOEXEC=15.
pub const ESUCCESS: ARC_STATUS = 0;
pub const ENOEXEC: ARC_STATUS = 15;
// SEEKMODE / OPENMODE (enums start at 0).
pub const SeekAbsolute: u32 = 0;
pub const SeekRelative: u32 = 1;
pub const OpenReadOnly: u32 = 0;
// TYPE_OF_MEMORY: LoaderExceptionBlock=0 … LoaderLoadedProgram=4.
pub const LoaderLoadedProgram: u32 = 4;
// ldrtypes.h LDRP flags.
pub const LDRP_IMAGE_INTEGRITY_FORCED: ULONG = 0x0000_0020;
pub const LDRP_ENTRY_PROCESSED: ULONG = 0x0000_4000;
// ntimage.h.
pub const IMAGE_FILE_EXECUTABLE_IMAGE: USHORT = 0x0002;
pub const IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY: USHORT = 0x0080;
pub const IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG: USHORT = 10;
// fs.h.
pub const SECTOR_SIZE: ULONG = 512;
// peloader.c `#define DEFAULT_SECURITY_COOKIE` (arch-split; rustconv
// dropped the #define).
#[cfg(target_arch = "x86")]
pub const DEFAULT_SECURITY_COOKIE: usize = 0xBB40_E64E;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const DEFAULT_SECURITY_COOKIE: usize = 0x0000_2B99_2DDF_A232;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const COOKIE_MAX: usize = 0x0000_FFFF_FFFF_FFFF;

extern "C" {
    pub fn ArcOpen(path: *mut CHAR, open_mode: u32, file_id: *mut ULONG) -> ARC_STATUS;
    pub fn ArcClose(file_id: ULONG) -> ARC_STATUS;
    pub fn ArcRead(file_id: ULONG, buffer: PVOID, n: ULONG, count: *mut ULONG) -> ARC_STATUS;
    pub fn ArcSeek(file_id: ULONG, position: *mut i64, seek_mode: u32) -> ARC_STATUS;
    pub fn ArcGetRelativeTime() -> ULONG;

    pub fn DbgPrint2(mask: ULONG, level: ULONG, file: *const CHAR, line: ULONG, fmt: *const CHAR, ...) -> ULONG;

    pub fn FrLdrHeapAlloc(size: usize, tag: ULONG) -> PVOID;
    pub fn FrLdrHeapFree(p: PVOID, tag: ULONG);
    pub fn MmAllocateMemoryWithType(size: usize, mem_type: u32) -> PVOID;
    pub fn MmAllocateMemoryAtAddress(size: usize, desired: PVOID, mem_type: u32) -> PVOID;
    pub fn MmFreeMemory(p: PVOID);

    pub fn memmove(dst: PVOID, src: *const c_void, n: usize) -> PVOID;
    pub fn memset(dst: PVOID, c: i32, n: usize) -> PVOID;
    pub fn strchr(s: *const CHAR, c: i32) -> *mut CHAR;
    pub fn strrchr(s: *const CHAR, c: i32) -> *mut CHAR;
    pub fn strcmp(a: *const CHAR, b: *const CHAR) -> i32;
    pub fn strlen(s: *const CHAR) -> usize;
    pub fn toupper(c: i32) -> i32;
}

// stdcall (NTAPI, i386 `_Name@N`). MUST be `extern "system"`.
extern "system" {
    pub fn RtlImageNtHeader(base: PVOID) -> PIMAGE_NT_HEADERS;
    pub fn RtlImageDirectoryEntryToData(
        base: PVOID,
        mapped_as_image: u8,
        directory_entry: USHORT,
        size: *mut ULONG,
    ) -> PVOID;
    pub fn LdrRelocateImageWithBias(
        new_base: PVOID,
        additional_bias: i64,
        loader_name: *mut CHAR,
        success: ULONG,
        conflict: ULONG,
        invalid: ULONG,
    ) -> ULONG;
    pub fn RtlAssert(failed: PVOID, file: PVOID, line: ULONG, message: *mut CHAR);
}

/// FreeLDR PE image base (linker symbol; i386 GNU `___ImageBase`).
extern "C" {
    pub static __ImageBase: u8;
}

// ---------------------------------------------------------------------
// peloader.c-defined globals (the AI mis-declared these `extern`).
// ---------------------------------------------------------------------
/// `LIST_ENTRY FrLdrModuleList;`
#[no_mangle]
pub static mut FrLdrModuleList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

/// `PELDR_IMPORTDLL_LOAD_CALLBACK PeLdrImportDllLoadCallback = NULL;`
/// (`typedef VOID (*)(PCCH)`).
pub type PELDR_IMPORTDLL_LOAD_CALLBACK = Option<unsafe extern "C" fn(*const CHAR)>;
#[no_mangle]
pub static mut PeLdrImportDllLoadCallback: PELDR_IMPORTDLL_LOAD_CALLBACK = None;

// ---------------------------------------------------------------------
// ntstrsafe RtlStringCbCopyA/CatA — peloader.c emits these locally
// (static inline from ntstrsafe.h; rustconv dropped them). Faithful
// bounded ANSI copy/cat; `cb` is a BYTE count. NTSTATUS return.
// ---------------------------------------------------------------------
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x8000_0005u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;

#[inline]
pub unsafe fn RtlStringCbCopyA(dest: *mut CHAR, cb_dest: usize, src: *const CHAR) -> NTSTATUS {
    if dest.is_null() || src.is_null() || cb_dest == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    while i + 1 < cb_dest {
        let ch = *src.add(i);
        *dest.add(i) = ch;
        if ch == 0 {
            return STATUS_SUCCESS;
        }
        i += 1;
    }
    *dest.add(i) = 0;
    if *src.add(i) == 0 { STATUS_SUCCESS } else { STATUS_BUFFER_OVERFLOW }
}

#[inline]
pub unsafe fn RtlStringCbCatA(dest: *mut CHAR, cb_dest: usize, src: *const CHAR) -> NTSTATUS {
    if dest.is_null() || src.is_null() || cb_dest == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let mut d = 0usize;
    while d < cb_dest && *dest.add(d) != 0 {
        d += 1;
    }
    if d >= cb_dest {
        return STATUS_INVALID_PARAMETER;
    }
    let mut s = 0usize;
    while d + 1 < cb_dest {
        let ch = *src.add(s);
        *dest.add(d) = ch;
        if ch == 0 {
            return STATUS_SUCCESS;
        }
        d += 1;
        s += 1;
    }
    *dest.add(d) = 0;
    if *src.add(s) == 0 { STATUS_SUCCESS } else { STATUS_BUFFER_OVERFLOW }
}
