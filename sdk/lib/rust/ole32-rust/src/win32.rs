//! The kernel32/user32 import surface the `ring` modules call, declared ONCE with
//! the correct ABI.
//!
//! This module exists because of a specific, load-bearing hazard: rustconv's raw
//! output declares every Win32 API in a per-file `extern "C"` block. On amd64/arm64
//! that happens to be right, so it links and boots — but on i386 WINAPI is
//! __stdcall, whose symbols are decorated `_Foo@N`. An `extern "C"` decl asks the
//! linker for undecorated `_Foo`, which does not exist → i386 link failure (or,
//! worse, a silent mismatch). `extern "system"` resolves to stdcall on i386 and C
//! elsewhere, which is exactly what WINAPI compiles to on each target.
//!
//! So: the ring's local `extern "C"` blocks are stripped at integration and every
//! Win32 call routes here. `scan_cc_mismatch` (in `vibeos-cli check`) is the
//! tripwire that catches any that creep back.
#![allow(clippy::all)]
use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
pub use crate::compobj::ffi::HeapAlloc;
use core::ffi::c_void;

// Base types com-sys doesn't carry (they're kernel32, not COM).
pub type LPCVOID = *const c_void;
pub type FARPROC = *mut c_void;
pub type HLOCAL = *mut c_void;

/// kernel32 CRITICAL_SECTION — opaque to us: the ring only ever passes a pointer
/// to one it owns (`static mut`), never inspects fields. Sized to the real
/// RTL_CRITICAL_SECTION so a `static mut` reserves enough storage:
/// { PRTL_CRITICAL_SECTION_DEBUG, LONG, LONG, HANDLE, HANDLE, ULONG_PTR }.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CRITICAL_SECTION {
    pub DebugInfo: *mut c_void,
    pub LockCount: i32,
    pub RecursionCount: i32,
    pub OwningThread: *mut c_void,
    pub LockSemaphore: *mut c_void,
    pub SpinCount: usize,
}
unsafe impl Sync for CRITICAL_SECTION {}
impl CRITICAL_SECTION {
    pub const fn new() -> Self {
        Self {
            DebugInfo: core::ptr::null_mut(),
            LockCount: 0,
            RecursionCount: 0,
            OwningThread: core::ptr::null_mut(),
            LockSemaphore: core::ptr::null_mut(),
            SpinCount: 0,
        }
    }
}

// GlobalAlloc/LocalAlloc flags (minwinbase.h)
pub const GMEM_FIXED: u32 = 0x0000;
pub const GMEM_MOVEABLE: u32 = 0x0002;
pub const GMEM_ZEROINIT: u32 = 0x0040;
pub const GMEM_NODISCARD: u32 = 0x0020;
pub const GMEM_SHARE: u32 = 0x2000;
pub const GHND: u32 = GMEM_MOVEABLE | GMEM_ZEROINIT;
pub const GPTR: u32 = GMEM_FIXED | GMEM_ZEROINIT;
pub const LMEM_FIXED: u32 = 0x0000;
pub const LMEM_MOVEABLE: u32 = 0x0002;
pub const LMEM_ZEROINIT: u32 = 0x0040;
pub const LPTR: u32 = LMEM_FIXED | LMEM_ZEROINIT;
pub const HEAP_ZERO_MEMORY: u32 = 0x0000_0008;

extern "system" {
    // --- heap ---
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn HeapSize(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPCVOID) -> SIZE_T;
    pub fn HeapValidate(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPCVOID) -> BOOL;

    // --- local/global memory (the OLE clipboard + hglobalstream paths) ---
    pub fn LocalAlloc(uFlags: u32, uBytes: SIZE_T) -> HLOCAL;
    pub fn LocalReAlloc(hMem: HLOCAL, uBytes: SIZE_T, uFlags: u32) -> HLOCAL;
    pub fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    pub fn GlobalAlloc(uFlags: u32, dwBytes: SIZE_T) -> HGLOBAL;
    pub fn GlobalReAlloc(hMem: HGLOBAL, dwBytes: SIZE_T, uFlags: u32) -> HGLOBAL;
    pub fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    pub fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    pub fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    pub fn GlobalSize(hMem: HGLOBAL) -> SIZE_T;

    // --- critical sections ---
    pub fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);

    // --- strings (WINAPI — stdcall-decorated on i386, hence extern "system") ---
    pub fn lstrlenW(lpString: *const u16) -> c_int;
    pub fn lstrcpyW(lpString1: *mut u16, lpString2: *const u16) -> *mut u16;
    pub fn lstrcmpW(lpString1: *const u16, lpString2: *const u16) -> c_int;
    pub fn lstrcmpiW(lpString1: *const u16, lpString2: *const u16) -> c_int;

    // --- misc ---
    pub fn GetThreadLocale() -> LCID;
    pub fn IsBadReadPtr(lp: LPCVOID, ucb: SIZE_T) -> BOOL;
    pub fn IsBadWritePtr(lp: LPVOID, ucb: SIZE_T) -> BOOL;
    pub fn IsBadCodePtr(lpfn: FARPROC) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn SetLastError(dwErrCode: DWORD);
    // ---- Tier-D (storage32 / stg_prop / datacache) additions ----
    pub fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    pub fn GetTempPathW(nBufferLength: DWORD, lpBuffer: *mut u16) -> DWORD;
    pub fn GetTempFileNameW(
        lpPathName: *const u16,
        lpPrefixString: *const u16,
        uUnique: UINT,
        lpTempFileName: *mut u16,
    ) -> UINT;
    /// Deliberately the ANSI/legacy pair — stg_prop.c compares codepage-tagged names.
    pub fn lstrcmpA(lpString1: *const c_char, lpString2: *const c_char) -> c_int;
    pub fn lstrcmpiA(lpString1: *const c_char, lpString2: *const c_char) -> c_int;
    pub fn GetACP() -> UINT;
    /// stg_prop.c stamps the property-set header's originator OS with this.
    pub fn GetVersion() -> DWORD;

    // advapi32
    pub fn RegQueryValueA(
        hKey: HKEY,
        lpSubKey: *const c_char,
        lpData: *mut c_char,
        lpcbData: *mut c_long,
    ) -> c_long;
}

// ---- wine/heap.h inlines (the C had them as static inline in a header, so
// rustconv emitted extern decls for symbols that don't exist) ----

/// wine/heap.h: `heap_alloc(len)` → HeapAlloc(GetProcessHeap(), 0, len)
#[inline]
pub unsafe fn heap_alloc(len: SIZE_T) -> LPVOID {
    HeapAlloc(GetProcessHeap(), 0, (len) as usize)
}

/// wine/heap.h: `heap_alloc_zero(len)` → HeapAlloc(.., HEAP_ZERO_MEMORY, len)
#[inline]
pub unsafe fn heap_alloc_zero(len: SIZE_T) -> LPVOID {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (len) as usize)
}

/// wine/heap.h: `heap_realloc(mem, len)` — NULL mem means a fresh alloc (HeapReAlloc
/// does NOT accept NULL, unlike realloc(3); the C helper handles that).
#[inline]
pub unsafe fn heap_realloc(mem: LPVOID, len: SIZE_T) -> LPVOID {
    if mem.is_null() {
        HeapAlloc(GetProcessHeap(), 0, (len) as usize)
    } else {
        HeapReAlloc(GetProcessHeap(), 0, mem, len)
    }
}

/// wine/heap.h: `heap_free(mem)` → HeapFree(GetProcessHeap(), 0, mem)
#[inline]
pub unsafe fn heap_free(mem: LPVOID) -> BOOL {
    HeapFree(GetProcessHeap(), 0, mem)

}
