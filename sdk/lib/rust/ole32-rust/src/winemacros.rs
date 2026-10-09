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
// ERR!/FIXME! are the shared winedebug-rust ones: the C format interpolated through
// DbgPrint. They used to print "(see source)" and swallow their arguments as raw
// tokens, so no argument here had ever been type-checked.
#[macro_export]
macro_rules! TRACE { ($($t:tt)*) => {{}} }
#[macro_export]
macro_rules! WARN { ($($t:tt)*) => {{}} }
#[macro_export]
macro_rules! ERR { ($($t:tt)*) => { ::winedebug_rust::ERR!($($t)*) } }
#[macro_export]
macro_rules! FIXME { ($($t:tt)*) => { ::winedebug_rust::FIXME!($($t)*) } }

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
        unsafe extern "system" fn $name ( $($arg : $ty),* ) -> $ret $body
    };
    ( $(#[$m:meta])* fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) $body:block ) => {
        $(#[$m])*
        unsafe extern "system" fn $name ( $($arg : $ty),* ) $body
    };
}

/// `CONTAINING_RECORD(address, type, field)` — recover the containing struct from a
/// pointer to one of its fields. Goes through `offset_of!` so the offset can never
/// drift from the struct (a hand-written number here is exactly the magic-offset bug
/// class `tools/scan_magic_offsets.py` exists to catch).
#[macro_export]
macro_rules! CONTAINING_RECORD {
    ($addr:expr, $ty:ty, $field:ident) => {
        (($addr as *const u8).sub(core::mem::offset_of!($ty, $field))) as *mut $ty
    };
}

// ---- NDR alignment macros (usrmarshal.c) ----
// These are ASSIGNMENT macros in the C (`ALIGN_LENGTH(len, 3)` mutates `len`), so they
// must be macros here too, not functions — a function taking the value by copy would
// silently drop the update. (rustconv emitted them as calls to a function; the callers
// then discarded the result.)

/// usrmarshal.c: `ALIGN_LENGTH(_Len, _Align)` → `_Len = (_Len + _Align) & ~_Align`
#[macro_export]
macro_rules! ALIGN_LENGTH {
    ($len:expr, $align:expr) => {
        $len = (($len) + ($align)) & !($align)
    };
}

/// usrmarshal.c: `ALIGN_POINTER(_Ptr, _Align)` → round the pointer up to _Align+1 bytes
#[macro_export]
macro_rules! ALIGN_POINTER {
    ($ptr:expr, $align:expr) => {
        $ptr = ((($ptr) as usize + ($align) as usize) & !(($align) as usize)) as _
    };
}

// ---- macro spellings of the WORD/BYTE extractors ----
// rustconv emits both `LOWORD(x)` (function) and `LOWORD!(x)` (macro) call syntax,
// depending on which C site it was translating. Rust keeps macros and functions in
// SEPARATE namespaces, so a macro and a fn may share a name — define both spellings
// and neither call site has to be rewritten.

/// windef.h: `HIWORD(l)`
#[macro_export]
macro_rules! HIWORD { ($v:expr) => { (((($v) as u32) >> 16) & 0xffff) as u16 } }
/// windef.h: `LOWORD(l)`
#[macro_export]
macro_rules! LOWORD { ($v:expr) => { ((($v) as u32) & 0xffff) as u16 } }
/// windef.h: `HIBYTE(w)`
#[macro_export]
macro_rules! HIBYTE { ($v:expr) => { (((($v) as u16) >> 8) & 0xff) as u8 } }
/// windef.h: `LOBYTE(w)`
#[macro_export]
macro_rules! LOBYTE { ($v:expr) => { ((($v) as u16) & 0xff) as u8 } }
/// windef.h: `MAKELONG(a, b)`
#[macro_export]
macro_rules! MAKELONG {
    ($lo:expr, $hi:expr) => { ((($lo) as u32 & 0xffff) | ((($hi) as u32 & 0xffff) << 16)) };
}

// ---- C `min` / `assert` ----

/// The C `min(a, b)` macro. Generic so it works at every width the translations use;
/// a call that mixes widths (the C relied on integer promotion) becomes a compile
/// ERROR here rather than a silent truncation — fix those with an explicit cast.
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

/// C `assert(cond)`, as a LOUD but non-fatal check.
///
/// It cannot `panic!`: this is a `no_std` staticlib inside a DLL, so a panic is a
/// process-killing abort in code paths (storage transactions) the C would have simply
/// continued through with NDEBUG. It must not be a silent no-op either — a tripped
/// assert is a real invariant violation we want to see. So: print to the debugger and
/// keep going, which is strictly louder than the shipping C.
///
/// `CBool` exists because the C macro is applied to ints, pointers-as-ints and BOOLs
/// alike, and rustconv preserved each site's own type.
pub trait CBool {
    fn is_true(self) -> bool;
}
impl CBool for bool {
    #[inline]
    fn is_true(self) -> bool { self }
}
impl CBool for i32 {
    #[inline]
    fn is_true(self) -> bool { self != 0 }
}
impl CBool for u32 {
    #[inline]
    fn is_true(self) -> bool { self != 0 }
}

#[inline]
pub fn assert<T: CBool>(cond: T) {
    if !cond.is_true() {
        // nolint: silent-failure (loud print + continue is deliberate; see doc comment)
        unsafe { crate::debug_shims::ERR(b"ole32-rust: ASSERTION FAILED\0".as_ptr() as *const core::ffi::c_char) };
    }
}

/// The C `max(a, b)` macro — the sibling of `min` above; same width rules apply.
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// ---- LCID packing (ntbasedef.h) ----
// `const fn`, not macros: the harvested LOCALE_SYSTEM_DEFAULT / LANG_SYSTEM_DEFAULT are
// defined in terms of these, and a `pub const` initializer can call a const fn but cannot
// expand a macro that isn't in scope at that point.

/// ntbasedef.h:939 `MAKELANGID(p, s)` → `((((USHORT)(s)) << 10) | (USHORT)(p))`
pub const fn MAKELANGID(p: u32, s: u32) -> u32 {
    (((s as u16) as u32) << 10) | ((p as u16) as u32)
}

/// ntbasedef.h:942 `MAKELCID(lgid, srtid)` → `(ULONG)((((ULONG)(USHORT)srtid) << 16) | (ULONG)(USHORT)lgid)`
pub const fn MAKELCID(lgid: u32, srtid: u32) -> u32 {
    (((srtid as u16) as u32) << 16) | ((lgid as u16) as u32)
}
