//! sys.rs — additional Win32/CRT import surface + types needed by the Lane A2
//! typelib.c continuation (data model, MSFT/SLTG loaders, ITypeLib2/ITypeInfo2,
//! DispCallFunc, writer). Same convention as `olesys.rs`: hand-declared,
//! self-contained, the ONE dedupe target for the future com-sys/win-sys crate.
//!
//! Everything here is `pub(crate)`-equivalent surface for the sibling
//! `typelib/*.rs` modules; the integrator swaps these decls for sys-crate
//! imports mechanically.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use core::ffi::c_void;
use super::super::olesys::*;

// ---------------------------------------------------------------------------
// Extra base types
// ---------------------------------------------------------------------------

pub type HFILE = INT;
pub type HRSRC = *mut c_void;
pub type HGLOBAL = *mut c_void;
pub type DWORD_PTR = usize;
pub type INT_PTR = isize;
pub type UINT_PTR = usize;
pub type INT16 = i16;
pub type UINT16 = u16;
pub type DATE = f64;

pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as usize as HANDLE;

// winnt.h CY / DECIMAL (sizes matter for TLB_size_instance)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CY {
    pub int64: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DECIMAL {
    pub wReserved: USHORT,
    pub scale: BYTE,
    pub sign: BYTE,
    pub Hi32: ULONG,
    pub Lo64: u64,
}
const _: () = assert!(core::mem::size_of::<CY>() == 8);
const _: () = assert!(core::mem::size_of::<DECIMAL>() == 16);

// VT_* values used by the loaders that olesys.rs doesn't carry
pub const VT_RESERVED: VARTYPE = 0x8000;

// ---------------------------------------------------------------------------
// CRITICAL_SECTION + one-time init (lazy replacement for the C's
// self-referential static initializer — see MANIFEST deviations)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct CRITICAL_SECTION {
    pub DebugInfo: *mut c_void,
    pub LockCount: LONG,
    pub RecursionCount: LONG,
    pub OwningThread: HANDLE,
    pub LockSemaphore: HANDLE,
    pub SpinCount: ULONG_PTR,
}

#[repr(C)]
pub struct INIT_ONCE {
    pub Ptr: *mut c_void,
}
pub type PINIT_ONCE_FN =
    unsafe extern "system" fn(*mut INIT_ONCE, *mut c_void, *mut *mut c_void) -> BOOL;

// ---------------------------------------------------------------------------
// File / mapping / resource constants
// ---------------------------------------------------------------------------

pub const GENERIC_READ: DWORD = 0x8000_0000;
pub const FILE_SHARE_READ: DWORD = 0x0000_0001;
pub const OPEN_EXISTING: DWORD = 3;
pub const FILE_ATTRIBUTE_NORMAL: DWORD = 0x80;
pub const PAGE_READONLY: DWORD = 0x02;
pub const SEC_COMMIT: DWORD = 0x800_0000;
pub const FILE_MAP_READ: DWORD = 0x0004;
pub const DONT_RESOLVE_DLL_REFERENCES: DWORD = 0x1;
pub const LOAD_LIBRARY_AS_DATAFILE: DWORD = 0x2;
pub const LOAD_WITH_ALTERED_SEARCH_PATH: DWORD = 0x8;
pub const ERROR_MORE_DATA: DWORD = 234;
pub const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;

// MultiByteToWideChar / WideCharToMultiByte
pub const CP_ACP: UINT = 0;
pub const MB_PRECOMPOSED: DWORD = 0x0001;
pub const MB_ERR_INVALID_CHARS: DWORD = 0x0008;

// GetFileInformationByHandleEx
pub const FileNameInfo: INT = 2; // FILE_INFO_BY_HANDLE_CLASS
#[repr(C)]
pub struct FILE_NAME_INFORMATION {
    pub FileNameLength: DWORD,
    pub FileName: [WCHAR; 1],
}

// LZ / OpenFile
pub const OF_READ: UINT = 0x0000;
pub const SEEK_SET: INT = 0;
pub const SEEK_CUR: INT = 1;
pub const OFS_MAXPATHNAME: usize = 128;
#[repr(C)]
pub struct OFSTRUCT {
    pub cBytes: BYTE,
    pub fFixedDisk: BYTE,
    pub nErrCode: WORD,
    pub Reserved1: WORD,
    pub Reserved2: WORD,
    pub szPathName: [CHAR; OFS_MAXPATHNAME],
}

// ---------------------------------------------------------------------------
// PE / NE image headers (winnt.h) — only the fields the NE path reads
// ---------------------------------------------------------------------------

pub const IMAGE_DOS_SIGNATURE: WORD = 0x5A4D; /* MZ */
pub const IMAGE_OS2_SIGNATURE: INT = 0x454E; /* NE */
pub const IMAGE_NT_SIGNATURE: INT = 0x0000_4550; /* PE00 */

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IMAGE_DOS_HEADER {
    pub e_magic: WORD,
    pub e_cblp: WORD,
    pub e_cp: WORD,
    pub e_crlc: WORD,
    pub e_cparhdr: WORD,
    pub e_minalloc: WORD,
    pub e_maxalloc: WORD,
    pub e_ss: WORD,
    pub e_sp: WORD,
    pub e_csum: WORD,
    pub e_ip: WORD,
    pub e_cs: WORD,
    pub e_lfarlc: WORD,
    pub e_ovno: WORD,
    pub e_res: [WORD; 4],
    pub e_oemid: WORD,
    pub e_oeminfo: WORD,
    pub e_res2: [WORD; 10],
    pub e_lfanew: LONG,
}
const _: () = assert!(core::mem::size_of::<IMAGE_DOS_HEADER>() == 64);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IMAGE_OS2_HEADER {
    pub ne_magic: WORD,
    pub ne_ver: CHAR,
    pub ne_rev: CHAR,
    pub ne_enttab: WORD,
    pub ne_cbenttab: WORD,
    pub ne_crc: LONG,
    pub ne_flags: WORD,
    pub ne_autodata: WORD,
    pub ne_heap: WORD,
    pub ne_stack: WORD,
    pub ne_csip: LONG,
    pub ne_sssp: LONG,
    pub ne_cseg: WORD,
    pub ne_cmod: WORD,
    pub ne_cbnrestab: WORD,
    pub ne_segtab: WORD,
    pub ne_rsrctab: WORD,
    pub ne_restab: WORD,
    pub ne_modtab: WORD,
    pub ne_imptab: WORD,
    pub ne_nrestab: LONG,
    pub ne_cmovent: WORD,
    pub ne_align: WORD,
    pub ne_cres: WORD,
    pub ne_exetyp: BYTE,
    pub ne_flagsothers: BYTE,
    pub ne_pretthunks: WORD,
    pub ne_psegrefbytes: WORD,
    pub ne_swaparea: WORD,
    pub ne_expver: WORD,
}
const _: () = assert!(core::mem::size_of::<IMAGE_OS2_HEADER>() == 64);

/// wine/winbase16-style NE resource structures (packed on disk, but all
/// members are naturally aligned WORDs/DWORDs so repr(C) matches).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NE_TYPEINFO {
    pub type_id: WORD,
    pub count: WORD,
    pub resloader: DWORD,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NE_NAMEINFO {
    pub offset: WORD,
    pub length: WORD,
    pub flags: WORD,
    pub id: WORD,
    pub handle: WORD,
    pub usage: WORD,
}

#[inline]
pub fn IS_INTRESOURCE_PTR(p: *const c_void) -> bool {
    (p as usize) >> 16 == 0
}
#[inline]
pub fn MAKEINTRESOURCEA(i: INT) -> LPCSTR {
    (i as WORD as usize) as LPCSTR
}
#[inline]
pub fn MAKEINTRESOURCEW(i: INT) -> LPCWSTR {
    (i as WORD as usize) as LPCWSTR
}

#[inline]
pub fn LOWORD(v: u32) -> u16 {
    (v & 0xffff) as u16
}
#[inline]
pub fn HIWORD(v: u32) -> u16 {
    (v >> 16) as u16
}
#[inline]
pub fn MAKELONG(lo: u16, hi: u16) -> u32 {
    (lo as u32) | ((hi as u32) << 16)
}

// ---------------------------------------------------------------------------
// System imports (extern "system")
// ---------------------------------------------------------------------------

// (clashing_extern_declarations: the .check rig also hosts the ole32 lane,
// whose ffi.rs declares InitOnceExecuteOnce against ITS nominal INIT_ONCE —
// layouts identical, ABI identical; separate crates at integration.)
#[allow(clashing_extern_declarations)]
extern "system" {
    // kernel32 — files/mappings
    pub fn CreateFileW(
        lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: PVOID,
        dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE,
    ) -> HANDLE;
    pub fn CreateFileMappingW(
        hFile: HANDLE, lpAttributes: PVOID, flProtect: DWORD, dwMaximumSizeHigh: DWORD,
        dwMaximumSizeLow: DWORD, lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn MapViewOfFile(
        hFileMappingObject: HANDLE, dwDesiredAccess: DWORD, dwFileOffsetHigh: DWORD,
        dwFileOffsetLow: DWORD, dwNumberOfBytesToMap: SIZE_T,
    ) -> PVOID;
    pub fn UnmapViewOfFile(lpBaseAddress: PVOID) -> BOOL;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn GetFileSize(hFile: HANDLE, lpFileSizeHigh: *mut DWORD) -> DWORD;
    pub fn GetFileInformationByHandleEx(
        hFile: HANDLE, FileInformationClass: INT, lpFileInformation: PVOID, dwBufferSize: DWORD,
    ) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    pub fn SearchPathW(
        lpPath: LPCWSTR, lpFileName: LPCWSTR, lpExtension: LPCWSTR, nBufferLength: DWORD,
        lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    // kernel32 — modules/resources
    pub fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn GetModuleFileNameW(hModule: HMODULE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD;
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> *mut c_void;
    pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    pub fn FindResourceW(hModule: HMODULE, lpName: LPCWSTR, lpType: LPCWSTR) -> HRSRC;
    pub fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL;
    pub fn LockResource(hResData: HGLOBAL) -> PVOID;
    pub fn SizeofResource(hModule: HMODULE, hResInfo: HRSRC) -> DWORD;
    pub fn FreeResource(hResData: HGLOBAL) -> BOOL;
    // kernel32 — strings/misc
    pub fn MultiByteToWideChar(
        CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT,
        lpWideCharStr: LPWSTR, cchWideChar: INT,
    ) -> INT;
    pub fn WideCharToMultiByte(
        CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: INT,
        lpMultiByteStr: LPSTR, cbMultiByte: INT, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut BOOL,
    ) -> INT;
    pub fn lstrlenW(lpString: LPCWSTR) -> INT;
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrcmpiW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> INT;
    pub fn DebugBreak();
    // kernel32 — sync
    pub fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn InitOnceExecuteOnce(
        InitOnce: *mut INIT_ONCE, InitFn: PINIT_ONCE_FN, Parameter: *mut c_void,
        Context: *mut *mut c_void,
    ) -> BOOL;
    // lz32
    pub fn LZOpenFileW(lpFileName: LPWSTR, lpReOpenBuf: *mut OFSTRUCT, wStyle: WORD) -> INT;
    pub fn LZSeek(hFile: INT, lOffset: LONG, iOrigin: INT) -> LONG;
    pub fn LZRead(hFile: INT, lpBuffer: LPSTR, cbRead: INT) -> INT;
    pub fn LZClose(hFile: INT);
    // oleaut32 other TUs (oleaut.c / variant.c / safearray.c — active C or
    // Phase-1 Rust at link time)
    pub fn SysAllocString(str_: LPCOLESTR) -> BSTR;
    pub fn SysAllocStringLen(str_: LPCOLESTR, len: UINT) -> BSTR;
    pub fn SysAllocStringByteLen(str_: LPCSTR, len: UINT) -> BSTR;
    pub fn SysFreeString(bstr: BSTR);
    pub fn SysStringLen(bstr: BSTR) -> UINT;
    pub fn VariantInit(pvarg: *mut VARIANT);
    pub fn VariantClear(pvarg: *mut VARIANT) -> HRESULT;
    pub fn VariantCopy(pvargDest: *mut VARIANT, pvargSrc: *const VARIANT) -> HRESULT;
    pub fn VariantCopyInd(pvarDest: *mut VARIANT, pvargSrc: *const VARIANT) -> HRESULT;
    // (signature textually matches dispatch.rs's decl — same TU family, keeps
    // the clashing_extern_declarations lint quiet in the .check rig)
    pub fn VariantChangeType(
        pvargDest: *mut VARIANTARG, pvargSrc: *mut VARIANTARG, wFlags: USHORT, vt: VARTYPE,
    ) -> HRESULT;
    pub fn SafeArrayAllocDescriptor(cDims: UINT, ppsaOut: *mut *mut c_void) -> HRESULT;
    // ole32 (CoTaskMemAlloc/Free come from olesys.rs — not redeclared here)
    pub fn CoCreateInstance(
        rclsid: REFCLSID, pUnkOuter: *mut IUnknown, dwClsContext: DWORD, riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
}

extern "C" {
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    pub fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn strlen(s: *const i8) -> usize;
    pub fn _strnicmp(a: *const i8, b: *const i8, n: usize) -> i32;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
    pub fn wcscmp(a: *const u16, b: *const u16) -> i32;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32;
    pub fn wcslen(s: *const u16) -> usize;
}

// ---------------------------------------------------------------------------
// wine/list.h — doubly linked list (byte-compatible with the C)
// ---------------------------------------------------------------------------

pub use winelist_rust::{
    list, list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty,
    list_head, list_init, list_next, list_prev, list_remove, list_tail,
};

/// `LIST_ENTRY(elem, Type, field)` — container_of.
macro_rules! list_entry {
    ($elem:expr, $ty:ty, $field:ident) => {
        ($elem as *mut u8).sub(core::mem::offset_of!($ty, $field)) as *mut $ty
    };
}
pub(crate) use list_entry;

/// `CONTAINING_RECORD(addr, Type, field)`.
macro_rules! containing_record {
    ($addr:expr, $ty:ty, $field:ident) => {
        ($addr as *mut u8).sub(core::mem::offset_of!($ty, $field)) as *mut $ty
    };
}
pub(crate) use containing_record;

// NOTE: LIST_FOR_EACH_ENTRY(_SAFE) iterations are hand-rolled while-loops at
// every use site (the C frequently breaks out with sentinel tests that don't
// map onto a closure/macro body) — deliberate, see MANIFEST.
