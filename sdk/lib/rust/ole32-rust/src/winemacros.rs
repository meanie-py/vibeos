//! The wine/Win32 function-like macros the rustconv translations call as plain
//! functions, plus the constants the translations reference as bare names. Each
//! value is cross-checked against its defining SDK header (cited inline).
//!
//! Scope: the `ring` modules only (ifs/streams/monikers/bindctx/ole2/clipboard/
//! …). compobj/marshal predate this and carry equivalents in their own ffi.rs /
//! types.rs — that duplication is intentional and harmless (all inline, no
//! symbols), and collapsing it is the queued cleanup.
#![allow(clippy::all)]
use crate::prelude::*;

// ---- Win32 word/byte macros (minwindef.h) ----
#[inline]
pub fn HIWORD(v: u32) -> u16 { (v >> 16) as u16 }
#[inline]
pub fn LOWORD(v: u32) -> u16 { v as u16 }
#[inline]
pub fn HIBYTE(v: u16) -> u8 { (v >> 8) as u8 }
#[inline]
pub fn LOBYTE(v: u16) -> u8 { v as u8 }
#[inline]
pub fn MAKELONG(lo: u16, hi: u16) -> u32 { (lo as u32) | ((hi as u32) << 16) }

// ---- interlocked / memory (the C called the kernel32 macros/intrinsics) ----
#[inline]
pub unsafe fn InterlockedIncrement(p: *mut i32) -> i32 {
    (*(p as *mut core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
#[inline]
pub unsafe fn InterlockedDecrement(p: *mut i32) -> i32 {
    (*(p as *mut core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}
#[inline]
pub unsafe fn InterlockedExchange(p: *mut i32, v: i32) -> i32 {
    (*(p as *mut core::sync::atomic::AtomicI32)).swap(v, core::sync::atomic::Ordering::SeqCst)
}
#[inline]
pub unsafe fn ZeroMemory(p: *mut core::ffi::c_void, len: usize) {
    core::ptr::write_bytes(p as *mut u8, 0, len);
}

// ---- guiddef.h GUID equality (C inline: !memcmp(a, b, sizeof(GUID))) ----
#[inline]
pub unsafe fn IsEqualGUID(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1 && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3 && (*a).Data4 == (*b).Data4
}
#[inline]
pub unsafe fn IsEqualIID(a: *const IID, b: *const IID) -> bool { IsEqualGUID(a, b) }
#[inline]
pub unsafe fn IsEqualCLSID(a: *const CLSID, b: *const CLSID) -> bool { IsEqualGUID(a, b) }

// ---- winerror.h HRESULT helpers ----
/// winerror.h: HRESULT_FROM_WIN32(x) — already-an-HRESULT values pass through.
#[inline]
pub fn HRESULT_FROM_WIN32(x: u32) -> HRESULT {
    if x as i32 <= 0 { x as HRESULT } else { ((x & 0x0000_FFFF) | 0x8007_0000) as HRESULT }
}

// ---- IIDs/CLSIDs: DELIBERATELY NOT DEFINED HERE ----
// Every IID_* / CLSID_* the ring needs is already a real data symbol at ole32.dll
// link time: the moniker CLSIDs come from ole32's own `guid.c` (an `#include
// <initguid.h>` TU we keep compiled), and the interface IIDs come from the linked
// `uuid` static lib (`nm libuuid.a` → `R IID_IMalloc`, `R IID_IMoniker`, …).
// So the ring modules DECLARE them (`extern "C" { static IID_IMalloc: GUID; }`,
// which is exactly what rustconv emits) and never redefine them. Hand-transcribing
// GUID values here would be a silent-corruption source — a wrong byte makes
// QueryInterface miss and the object "just doesn't work" with no link error.

// ---- wine debug channel macro forms (TRACE!(..) style call sites) ----
#[macro_export]
macro_rules! TRACE { ($($t:tt)*) => {{}} }
#[macro_export]
macro_rules! WARN { ($($t:tt)*) => {{}} }
#[macro_export]
macro_rules! ERR { ($($t:tt)*) => {{
    unsafe { $crate::debug_shims::ERR(b"(see source)\0".as_ptr() as *const core::ffi::c_char) }
}} }
#[macro_export]
macro_rules! FIXME { ($($t:tt)*) => {{
    unsafe { $crate::debug_shims::FIXME(b"(see source)\0".as_ptr() as *const core::ffi::c_char) }
}} }

// ---- STDMETHODCALLTYPE vtbl thunks ----
// com-sys's bindgen vtbl slots spell the COM calling convention CONCRETELY:
// `extern "stdcall"` on i386, `extern "C"` on amd64/arm64 — which is exactly what
// STDMETHODCALLTYPE compiles to per target. rustc keeps `extern "system"` a
// NOMINALLY DISTINCT fn type from `extern "C"` even where they lower identically, so
// a method declared `extern "system"` will NOT unify with the slot's type
// (`error[E0308]: expected "C" fn, found "system" fn`). Declare vtbl methods with this
// macro so the ABI is spelled the same way com-sys spells it.
//
// Same pattern as oleaut32-rust/src/oleaut.rs. Three separate ring burn-downs hit this
// wall independently before it was hoisted here — if you are writing a new COM object
// in this crate, use this.
#[macro_export]
macro_rules! stdmethod {
    ( $(#[$m:meta])* fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty $body:block ) => {
        $(#[$m])*
        #[cfg(target_arch = "x86")]
        unsafe extern "stdcall" fn $name ( $($arg : $ty),* ) -> $ret $body
        $(#[$m])*
        #[cfg(not(target_arch = "x86"))]
        unsafe extern "C" fn $name ( $($arg : $ty),* ) -> $ret $body
    };
    ( $(#[$m:meta])* fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) $body:block ) => {
        $(#[$m])*
        #[cfg(target_arch = "x86")]
        unsafe extern "stdcall" fn $name ( $($arg : $ty),* ) $body
        $(#[$m])*
        #[cfg(not(target_arch = "x86"))]
        unsafe extern "C" fn $name ( $($arg : $ty),* ) $body
    };
}
