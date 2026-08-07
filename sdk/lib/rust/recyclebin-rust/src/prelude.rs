//! recyclebin-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1 — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier).
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! DPRINT1 { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, DPRINT1};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= COM result codes ===========================================================
// winerror.h spells every one of these as a CAST macro — `#define S_OK ((HRESULT)0L)` — which
// bindgen does not fold, so the island carries none of them and the first cargo check reported
// 27 missing `S_OK` alone. Values are COPIED from the landed, in-tree definitions
// (shell32-rust/src/island_extra.rs:34-41, win32.rs:1129) rather than transcribed from memory:
// hand-typing famous constants is exactly the class that shipped 61 invented bugcheck codes.
pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
pub const E_POINTER: HRESULT = 0x80004003u32 as HRESULT;
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT;
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT;
pub const E_UNEXPECTED: HRESULT = 0x8000FFFFu32 as HRESULT;

/// `HRESULT_FROM_WIN32` (winerror.h): positive Win32 error → 0x8007xxxx; pass-through
/// otherwise. Faithful port of `__HRESULT_FROM_WIN32`.
#[inline]
pub fn HRESULT_FROM_WIN32(x: LONG) -> HRESULT {
    const FACILITY_WIN32: u32 = 7;
    if x > 0 {
        (((x as u32) & 0x0000FFFF) | (FACILITY_WIN32 << 16) | 0x80000000) as HRESULT
    } else {
        x as HRESULT
    }
}

/// `recyclebin_private.h:17` — the module's own inline wrapper. It exists precisely because
/// `HRESULT_FROM_WIN32` is a macro that EVALUATES ITS ARGUMENT TWICE; keeping it a separate
/// function preserves that, since the call sites pass `GetLastError()`.
#[inline]
pub fn HResultFromWin32(hr: DWORD) -> HRESULT {
    HRESULT_FROM_WIN32(hr as LONG)
}

/// `HRESULT_CODE` / `HRESULT_FACILITY` (winerror.h).
#[inline]
pub fn HRESULT_CODE(hr: HRESULT) -> c_int {
    (hr & 0xFFFF) as c_int
}
#[inline]
pub fn HRESULT_FACILITY(hr: HRESULT) -> c_int {
    ((hr >> 16) & 0x1fff) as c_int
}

/// `IsEqualIID` — a guiddef.h comparison MACRO, not an import. GUID does not derive PartialEq,
/// so compare the record bytes exactly as `IsEqualGUID` does.
#[inline]
pub unsafe fn IsEqualIID(a: *const GUID, b: *const GUID) -> bool {
    core::slice::from_raw_parts(a as *const u8, core::mem::size_of::<GUID>())
        == core::slice::from_raw_parts(b as *const u8, core::mem::size_of::<GUID>())
}

extern "C" {
    /// Defined by uuid.lib, which shell32 already links.
    pub static IID_IUnknown: IID;
}

// ================= COBJMACROS vtable helpers ==================================================
// recyclebin.h / recyclebin_v5.h define these under `#ifdef COBJMACROS` (which
// recyclebin_private.h sets) as `(This)->lpVtbl->Method(This, ...)`. They are macros, so the
// island cannot carry them, and the C bodies call them by name throughout. Each one below is
// that expansion and nothing more — the island's C-mode vtables are what make it spellable.
#[inline]
pub unsafe fn IRecycleBin_Release(This: *mut IRecycleBin) -> ULONG {
    ((*(*This).lpVtbl).Release.unwrap())(This)
}
#[inline]
pub unsafe fn IRecycleBin_AddRef(This: *mut IRecycleBin) -> ULONG {
    ((*(*This).lpVtbl).AddRef.unwrap())(This)
}
#[inline]
pub unsafe fn IRecycleBinEnumList_Release(This: *mut IRecycleBinEnumList) -> ULONG {
    ((*(*This).lpVtbl).Release.unwrap())(This)
}
#[inline]
pub unsafe fn IRecycleBinFile_Release(This: *mut IRecycleBinFile) -> ULONG {
    ((*(*This).lpVtbl).Release.unwrap())(This)
}

/// `recyclebin.h:50` `#define IRecycleBinFileFromHDELFILE(hDF) ((IRecycleBinFile*)(hDF))` — an
/// HDELFILE IS an IRecycleBinFile*, handed out opaquely by the enumeration callback.
#[inline]
pub unsafe fn IRecycleBinFileFromHDELFILE(hDF: HDELFILE) -> *mut IRecycleBinFile {
    hDF as *mut IRecycleBinFile
}

// ================= Win32 / CRT gap fillers ====================================================
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF;

extern "system" {
    pub fn InterlockedIncrement(Addend: *mut LONG) -> LONG;
    pub fn InterlockedDecrement(Addend: *mut LONG) -> LONG;
    pub fn SHStrDupW(psz: LPCWSTR, ppwsz: *mut LPWSTR) -> HRESULT;
}
/// `SHStrDup` is the A/W macro; under UNICODE it is `SHStrDupW`.
#[inline]
pub unsafe fn SHStrDup(psz: LPCWSTR, ppwsz: *mut LPWSTR) -> HRESULT {
    SHStrDupW(psz, ppwsz)
}

extern "C" {
    pub fn wcslen(s: *const WCHAR) -> usize;
}

/// `ZeroMemory(dst, len)` (winbase.h) — a MACRO for `memset(dst, 0, len)`. Kept a macro here:
/// an out-of-line no-op-shaped wrapper is the optimization barrier §9.65 warns about.
#[macro_export]
macro_rules! ZeroMemory {
    ($dst:expr, $len:expr) => {{
        core::ptr::write_bytes($dst as *mut u8, 0u8, $len as usize)
    }};
}
/// `_countof(a)` (stdlib.h) — element count of a fixed C array, as a MACRO so the array is
/// never moved by value.
#[macro_export]
macro_rules! _countof {
    ($a:expr) => {
        ($a.len())
    };
}
pub use crate::{ZeroMemory, _countof};
