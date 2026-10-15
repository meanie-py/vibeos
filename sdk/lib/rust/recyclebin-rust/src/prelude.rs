//! recyclebin-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
// recyclebin.h's COBJMACROS block: `IRecycleBin_DeleteFile(This, x)` etc. Generated from the
// island's vtable structs, so a helper's parameter list cannot drift from the slot it
// dispatches through. Re-exported here because the translated sources call them unqualified,
// exactly as the C does.
pub use crate::commacros::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// `NDEBUG` silences DPRINT only. DPRINT1 sits OUTSIDE that guard in
// sdk/include/reactos/debug.h, so it is live whenever DBG is set — and our media is
// built -DCMAKE_BUILD_TYPE=Debug, i.e. -DDBG=1. Both stay MACROS, never out-of-line
// fns (field manual §9.65: a no-op fn is an optimization barrier). The C passes
// __RELFILE__/__LINE__ as two extra varargs; file!()/line!() fold into the literal.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
// ntdll exports DbgPrint; every usermode DLL links it.
extern "C" {
    pub fn DbgPrint(Format: *const c_char, ...) -> u32;
}
#[macro_export]
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::prelude::DbgPrint(
                concat!("(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
}
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
pub fn HRESULT_FROM_WIN32(x: DWORD) -> HRESULT {
    // winerror.h:197 `((HRESULT)(x) > 0 ? ((HRESULT)(((x) & 0xFFFF) | (FACILITY_WIN32 << 16)
    // | 0x80000000)) : (HRESULT)(x))`. The macro is UNTYPED and its callers pass a DWORD --
    // `GetLastError()` above all -- so the parameter is DWORD, matching `explorer-sys` and
    // `oleaut32-rust`. Declaring it LONG made every `HRESULT_FROM_WIN32(GetLastError())` a
    // type error, and rustc's suggested `.try_into().unwrap()` is exactly the fix this tree
    // refuses: it turns C's silent truncation into a panic in a live service.
    // Note the C casts to HRESULT only for the COMPARISON and uses raw `x` for the bits.
    const FACILITY_WIN32: u32 = 7;
    if (x as HRESULT) > 0 {
        ((x & 0x0000FFFF) | (FACILITY_WIN32 << 16) | 0x80000000) as HRESULT
    } else {
        x as HRESULT
    }
}

/// `recyclebin_private.h:17` — the module's own inline wrapper. It exists precisely because
/// `HRESULT_FROM_WIN32` is a macro that EVALUATES ITS ARGUMENT TWICE; keeping it a separate
/// function preserves that, since the call sites pass `GetLastError()`.
#[inline]
pub fn HResultFromWin32(hr: DWORD) -> HRESULT {
    HRESULT_FROM_WIN32(hr)
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
//
// `unwrap_unchecked`, not `unwrap`: bindgen types every vtable slot as `Option<fn>`, but the C
// these replace is `(This)->lpVtbl->Release(This)` with no check at all, so a PANIC here would
// be less faithful than the wild call C would make — and this is a no_std staticlib linked into
// shell32, where a panic is not a diagnostic. Option<fn> carries the null-pointer optimization,
// so this is also the identical codegen. Same spelling as avltable/src/lib.rs:130.
#[inline]
pub unsafe fn IRecycleBin_Release(This: *mut IRecycleBin) -> ULONG {
    ((*(*This).lpVtbl).Release.unwrap_unchecked())(This)
}
#[inline]
pub unsafe fn IRecycleBin_AddRef(This: *mut IRecycleBin) -> ULONG {
    ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This)
}
#[inline]
pub unsafe fn IRecycleBinEnumList_Release(This: *mut IRecycleBinEnumList) -> ULONG {
    ((*(*This).lpVtbl).Release.unwrap_unchecked())(This)
}
#[inline]
pub unsafe fn IRecycleBinFile_Release(This: *mut IRecycleBinFile) -> ULONG {
    ((*(*This).lpVtbl).Release.unwrap_unchecked())(This)
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
// The zeroing macros live in `win32macros-rust`, ported once from the header and
// known-answer tested there. Re-exported under the same names, so no call site changed.
pub use win32macros_rust::ZeroMemory;
/// `_countof(a)` (stdlib.h) — element count of a fixed C array, as a MACRO so the array is
/// never moved by value.
#[macro_export]
macro_rules! _countof {
    ($a:expr $(,)?) => {
        ($a.len())
    };
}
pub use crate::_countof;

// ================= more gap fillers ===========================================================
// guiddef.h spells REFIID as a MACRO (`#define REFIID const IID* const`), so bindgen drops it
// while the emitted vtable slots already say `*const IID`. Same identity, under the name the
// translated bodies use.
pub type REFIID = *const IID;

/// `INVALID_HANDLE_VALUE` — `((HANDLE)(LONG_PTR)-1)` (handleapi.h), a cast macro.
/// Spelled as in shell32-rust/src/island_extra.rs:28.
pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE;
/// `UNICODE_NULL` (ntdef.h).
pub const UNICODE_NULL: WCHAR = 0;
/// `INT_MAX` (limits.h).
pub const INT_MAX: c_int = 2147483647;

/// C's `max(a, b)` macro. `core::cmp::max` IS that function and is generic, so there is no
/// reason to hand-roll one — and no double-evaluation, which the C macro does have.
pub use core::cmp::max;

// ================= CRT string/memory ==========================================================
extern "C" {
    pub fn wcscpy(dest: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcscat(dest: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcsstr(haystack: *const WCHAR, needle: *const WCHAR) -> *mut WCHAR;
    pub fn _ultow(value: ULONG, buf: *mut WCHAR, radix: c_int) -> *mut WCHAR;
    pub fn tolower(c: c_int) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn _wcsicmp(a: LPCWSTR, b: LPCWSTR) -> c_int;
}
// `extern "C"`, NOT `extern "system"`. winuser.h declares these `WINAPIV`, which is `__cdecl`
// on every arch precisely BECAUSE they are variadic — stdcall cannot express a caller-cleaned
// stack. So cdecl is the correct ABI here, and it is also the only one Rust accepts on stable:
// a C-variadic with the "system" convention is a nightly feature (E0658), which is how the
// wrong spelling announced itself.
extern "C" {
    pub fn wsprintfA(buf: *mut c_char, fmt: *const c_char, ...) -> c_int;
    pub fn wsprintfW(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int;
}

// guiddef.h: `#define REFGUID const GUID * const`. bindgen emits REFIID (it is used in an
// interface signature) but never REFGUID, which only appears in ones this module does not use.
pub type REFGUID = *const GUID;

// winbase.h maps InterlockedIncrement/Decrement onto COMPILER INTRINSICS, not library symbols,
// and which library happens to carry a `_Interlocked*` differs per arch — aclui's i386 link
// found one and amd64 did not. Implemented here rather than declared, so no arch depends on
// someone else exporting it. Semantics per winbase.h: atomic, returns the NEW value.
#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        .wrapping_add(1)
}
#[inline(always)]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        .wrapping_sub(1)
}

/// `MoveMemory` (winbase.h) is a MACRO for `memmove`; `swprintf` here is only ever the
/// count-bounded form the two call sites use.
#[macro_export]
macro_rules! MoveMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {{
        $crate::prelude::memmove($dst as *mut _, $src as *const _, $len as usize)
    }};
}
pub use crate::MoveMemory;

extern "C" {
    // The C's count-form `swprintf(buf, n, fmt, ...)` is a static inline in <stdio.h> over
    // `_vsnwprintf`; msvcrt's EXPORTED `swprintf` is the legacy `(buf, fmt, ...)`, which
    // read `n` as the format pointer (cscript crashed reporting a script error).
    #[link_name = "_snwprintf"]
    pub fn swprintf(buf: *mut WCHAR, count: usize, fmt: *const WCHAR, ...) -> c_int;
}

// ================= sibling TU / shell32 externs ===============================================
// GetDefaultRecycleBin lives in this crate's recyclebin.rs; the island already declares it, but
// SHUpdateRecycleBinIcon is shell32's and recyclebin_v5.cpp declares it EXTERN_C by hand.
extern "system" {
    pub fn SHUpdateRecycleBinIcon() -> HRESULT;
}

/// `recyclebin.h:62` `static inline` — an inline, so bindgen emits nothing for it.
#[inline]
pub unsafe fn InitializeRecycleBinStringRef(pRBS: *mut RECYCLEBINSTRING, String: LPCWSTR) {
    (*pRBS).String = String;
    (*pRBS).Alloc = core::ptr::null_mut();
}
