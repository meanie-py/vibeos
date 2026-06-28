//! misc/heap.c → Rust. gdi32 private heap allocator.
//!
//! Owns the process-global `hProcessHeap` HANDLE (a DLL global is process-global) and the
//! ANSI→Unicode strdup helpers built on it. `HEAP_alloc`/`HEAP_free` are `static inline FASTCALL`
//! header helpers (gdi32p.h) — NOT real exports — so the one this file needs (`HEAP_alloc`) is
//! reimplemented locally as a private fn wrapping `RtlAllocateHeap(hProcessHeap, ...)`.
//!
//! NEEDS-IMPORT: `lstrlenA` (kernel32 WINAPI), `RtlMultiByteToUnicodeN` + `RtlAllocateHeap`
//!   (ntdll NTAPI), and `strlen` (CRT cdecl) are not in the win32k_sys bindgen surface; declared
//!   extern below. They resolve against the kernel32/ntdll importlibs / CRT of gdi32.dll.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// ── ntstatus.h values used here (NTSTATUS = LONG) ──────────────────────────────
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NO_MEMORY: NTSTATUS = 0xC000_0017u32 as NTSTATUS;
// ntdef.h
const UNICODE_NULL: WCHAR = 0;

// ── process-global heap handle (misc/heap.c: `HANDLE hProcessHeap = NULL;`) ─────
// "global variables in a dll are process-global"
#[no_mangle]
pub static mut hProcessHeap: HANDLE = core::ptr::null_mut();

// ── Cross-TU externs ───────────────────────────────────────────────────────────
extern "system" {
    // kernel32.dll (WINAPI = stdcall on i386)
    fn lstrlenA(lpString: PCSTR) -> INT;
    // ntdll.dll (NTAPI = stdcall on i386, cdecl on amd64/arm64 — `extern "system"` per-arch)
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlMultiByteToUnicodeN(
        UnicodeString: PWCH,
        MaxBytesInUnicodeString: ULONG,
        BytesInUnicodeString: PULONG,
        MultiByteString: PCSTR,
        BytesInMultiByteString: ULONG,
    ) -> NTSTATUS;
}

extern "C" {
    // CRT (cdecl)
    fn strlen(s: PCSTR) -> usize;
}

// ── gdi32p.h `static inline PVOID FASTCALL HEAP_alloc(SIZE_T len)` ──────────────
// Reimplemented locally (header static-inline, not a real export). ASSERT(hProcessHeap) → dropped.
#[inline]
unsafe fn HEAP_alloc(len: SIZE_T) -> PVOID {
    RtlAllocateHeap(hProcessHeap, 0, len)
}

#[no_mangle]
pub unsafe extern "fastcall" fn HEAP_strdupA2W(ppszW: *mut PWSTR, lpszA: PCSTR) -> NTSTATUS {
    let len: ULONG;
    let Status: NTSTATUS;

    *ppszW = core::ptr::null_mut();
    if lpszA.is_null() {
        return STATUS_SUCCESS;
    }
    len = lstrlenA(lpszA) as ULONG;

    *ppszW = HEAP_alloc(((len + 1) as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T) as PWSTR;
    if (*ppszW).is_null() {
        return STATUS_NO_MEMORY;
    }
    Status = RtlMultiByteToUnicodeN(
        *ppszW,
        len * core::mem::size_of::<WCHAR>() as ULONG,
        core::ptr::null_mut(),
        lpszA,
        len,
    );
    *(*ppszW).add(len as usize) = UNICODE_NULL;
    Status
}

#[no_mangle]
pub unsafe extern "fastcall" fn HEAP_strdupA2W_buf(
    lpszA: PCSTR,
    pszStaticBuff: PWSTR,
    cchStaticBuff: SIZE_T,
) -> PWSTR {
    if lpszA.is_null() {
        return core::ptr::null_mut();
    }

    let size: SIZE_T = strlen(lpszA) as SIZE_T + 1;
    let pszW: PWSTR = if size < cchStaticBuff {
        pszStaticBuff
    } else {
        HEAP_alloc(size * core::mem::size_of::<WCHAR>() as SIZE_T) as PWSTR
    };
    if pszW.is_null() {
        return core::ptr::null_mut();
    }

    RtlMultiByteToUnicodeN(
        pszW,
        (size * core::mem::size_of::<WCHAR>() as SIZE_T) as ULONG,
        core::ptr::null_mut(),
        lpszA,
        size as ULONG,
    );
    pszW
}
