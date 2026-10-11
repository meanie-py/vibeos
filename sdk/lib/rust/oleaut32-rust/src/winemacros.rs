//! Hand-written companions to winemacros_gen.rs: the wine/Win32 function-like
//! macros the rustconv translations call as plain functions, plus the handful
//! of constants the generator couldn't evaluate mechanically (each value
//! cross-checked against its defining header — cited inline).
#![allow(clippy::all)]
use crate::prelude::*;

// A COM vtbl method is STDMETHODCALLTYPE = __stdcall on i386, cdecl elsewhere.
// com-sys/bindgen spells the vtbl slot with the CONCRETE ABI (`unsafe extern
// "stdcall" fn(..)` on i386), and rustc keeps `extern "system"` a NOMINALLY
// DISTINCT fn type from `extern "stdcall"` even where they lower identically —
// so a method declared `extern "system"` will NOT unify with its own slot
// (`error[E0308]: expected "stdcall" fn, found "system" fn`). Declare vtbl
// methods with this macro so the ABI is spelled the way com-sys spells it.
//
// Hoisted here (was a file-local macro_rules in oleaut.rs, hence invisible to
// every other module) so newly translated COM files can see it; mirrors
// ole32-rust/src/winemacros.rs. rustconv emits this form — see abi_enforce.py's
// `com_method_macro`.
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

// ---- Win32 word/byte/lang macros (minwindef.h / ntbasedef.h) ----
#[inline]
pub fn HIWORD(v: u32) -> u16 { (v >> 16) as u16 }
#[inline]
pub fn LOWORD(v: u32) -> u16 { v as u16 }
#[inline]
pub fn HIBYTE(v: u16) -> u8 { (v >> 8) as u8 }
#[inline]
pub fn LOBYTE(v: u16) -> u8 { v as u8 }
#[inline]
pub fn MAKELANGID(p: u16, s: u16) -> LANGID { ((s << 10) | p) as LANGID }
#[inline]
pub fn PRIMARYLANGID(l: LANGID) -> LANGID { l & 0x3ff }
#[inline]
pub fn SUBLANGID(l: LANGID) -> LANGID { l >> 10 }
#[inline]
pub fn MAKELCID(lgid: u16, srtid: u16) -> LCID { ((srtid as LCID) << 16) | lgid as LCID }
#[inline]
pub fn LANGIDFROMLCID(lcid: LCID) -> LANGID { lcid as LANGID }

// ---- wingdi.h color macros ----
#[inline]
pub fn GetRValue(rgb: u32) -> u8 { rgb as u8 }
#[inline]
pub fn GetGValue(rgb: u32) -> u8 { (rgb >> 8) as u8 }
#[inline]
pub fn GetBValue(rgb: u32) -> u8 { (rgb >> 16) as u8 }
#[inline]
pub fn PALETTERGB(r: u8, g: u8, b: u8) -> u32 {
    0x0200_0000 | (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

// ---- interlocked / memory (the C called the kernel32 macros/intrinsics) ----
#[inline]
pub unsafe fn InterlockedIncrement(p: *mut i32) -> i32 {
    (*(p as *mut core::sync::atomic::AtomicI32)).fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
#[inline]
pub unsafe fn InterlockedDecrement(p: *mut i32) -> i32 {
    (*(p as *mut core::sync::atomic::AtomicI32)).fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}
#[inline]
pub unsafe fn ZeroMemory(p: *mut core::ffi::c_void, len: usize) {
    core::ptr::write_bytes(p as *mut u8, 0, len);
}

// ---- wine variant.h DECIMAL access macros (dll/win32/oleaut32/variant.h) ----
// DECIMAL layout (com-sys): { wReserved, u{ s{scale,sign}, signscale }, Hi32, u2{ s2{Lo32,Mid32}, Lo64 } }
// bindgen flattens the named DUMMYSTRUCTNAMEs; field paths verified against bindgen_amd64.rs.
#[inline]
pub unsafe fn DEC_SCALE(d: *mut DECIMAL) -> u8 { (*d).__bindgen_anon_1.__bindgen_anon_1.scale }
#[inline]
pub unsafe fn DEC_SIGN(d: *mut DECIMAL) -> u8 { (*d).__bindgen_anon_1.__bindgen_anon_1.sign }
#[inline]
pub unsafe fn DEC_SIGNSCALE_ref(d: *mut DECIMAL) -> *mut u16 { core::ptr::addr_of_mut!((*d).__bindgen_anon_1.signscale) }
#[inline]
pub unsafe fn DEC_HI32_ref(d: *mut DECIMAL) -> *mut u32 { core::ptr::addr_of_mut!((*d).Hi32) }
#[inline]
pub unsafe fn DEC_MID32_ref(d: *mut DECIMAL) -> *mut u32 { core::ptr::addr_of_mut!((*d).__bindgen_anon_2.__bindgen_anon_1.Mid32) }
#[inline]
pub unsafe fn DEC_LO32_ref(d: *mut DECIMAL) -> *mut u32 { core::ptr::addr_of_mut!((*d).__bindgen_anon_2.__bindgen_anon_1.Lo32) }
#[inline]
pub unsafe fn DEC_LO64_ref(d: *mut DECIMAL) -> *mut u64 { core::ptr::addr_of_mut!((*d).__bindgen_anon_2.Lo64) }
/// variant.h little-endian branch: `SIGNSCALE(sign,scale) (((sign) << 8) | scale)`
/// (DECIMAL keeps `scale` in the low byte). The translation took the
/// WORDS_BIGENDIAN branch, `(scale << 8) | sign`, so SIGNSCALE(DECIMAL_NEG, 0)
/// stored scale 128 / sign 0: every negative DECIMAL made this way was garbage.
#[inline]
pub fn SIGNSCALE(sign: u8, scale: u8) -> u16 { ((sign as u16) << 8) | scale as u16 }
#[inline]
pub unsafe fn DECIMAL_SETZERO(d: *mut DECIMAL) {
    core::ptr::write_bytes(d as *mut u8, 0, core::mem::size_of::<DECIMAL>());
}

// ---- variant.c local date/dos macros (verbatim from the C) ----
#[inline]
pub fn IsLeapYear(y: i32) -> bool { ((y % 4) == 0) && ((y % 100) != 0 || (y % 400) == 0) }
#[inline]
pub fn DOS_YEAR(x: u16) -> i32 { 1980 + (x >> 9) as i32 }
#[inline]
pub fn DOS_MONTH(x: u16) -> i32 { ((x >> 5) & 0xf) as i32 }
#[inline]
pub fn DOS_DAY(x: u16) -> i32 { (x & 0x1f) as i32 }
#[inline]
pub fn DOS_HOUR(x: u16) -> i32 { (x >> 11) as i32 }
#[inline]
pub fn DOS_MINUTE(x: u16) -> i32 { ((x >> 5) & 0x3f) as i32 }
#[inline]
pub fn DOS_SECOND(x: u16) -> i32 { (2 * (x & 0x1f)) as i32 }
// variant.c: FITS_AS_In(x) ((x) >> n*8 == 0)
#[inline]
pub fn FITS_AS_I1(x: u32) -> bool { x >> 8 == 0 }
#[inline]
pub fn FITS_AS_I2(x: u32) -> bool { x >> 16 == 0 }
#[inline]
pub fn FITS_AS_I4(x: u64) -> bool { x >> 32 == 0 }

// ---- V_* predicates (oleauto.h; rvalue-only, not lvalue accessors) ----
#[inline]
pub unsafe fn V_TYPE(v: *mut VARIANT) -> VARTYPE { (*v).n1.n2.vt & VT_TYPEMASK }
#[inline]
pub unsafe fn V_ISBYREF(v: *mut VARIANT) -> bool { ((*v).n1.n2.vt & VT_BYREF) != 0 }
#[inline]
pub unsafe fn V_ISARRAY(v: *mut VARIANT) -> bool { ((*v).n1.n2.vt & VT_ARRAY) != 0 }
#[inline]
pub unsafe fn V_ISVECTOR(v: *mut VARIANT) -> bool { ((*v).n1.n2.vt & VT_VECTOR) != 0 }
#[inline]
pub unsafe fn V_EXTRA_TYPE(v: *mut VARIANT) -> VARTYPE {
    (*v).n1.n2.vt & !VT_TYPEMASK
}

// ---- constants the generator couldn't evaluate (cited) ----
/// wtypes.h DECIMAL sign byte: negative = 0x80
pub const DECIMAL_NEG: u8 = 0x80;
pub const DECIMAL_POS: u8 = 0;
/// variant.h: I8_MAX/I8_MIN/UI8_MAX
pub const I8_MAX: i64 = i64::MAX;
pub const I8_MIN: i64 = i64::MIN;
pub const UI8_MAX: u64 = u64::MAX;
/// ntbasedef.h: MAKELCID(MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), SORT_DEFAULT) = 0x0400
pub const LOCALE_USER_DEFAULT: u32 = 0x0400;
/// rpcndr.h: NDR_LITTLE_ENDIAN = 0x10
pub const NDR_LOCAL_DATA_REPRESENTATION: u32 = 0x10;
/// objbase.h ROTFLAGS
pub const ROTFLAGS_REGISTRATIONKEEPSALIVE: u32 = 1;
/// oaidl.h SF_TYPE (= the matching VT_ values; SF_HAVEIID = VT_UNKNOWN|VT_RESERVED)
pub const SF_ERROR: u32 = 10;
pub const SF_I1: u32 = 16;
pub const SF_I2: u32 = 2;
pub const SF_I4: u32 = 3;
pub const SF_I8: u32 = 20;
pub const SF_BSTR: u32 = 8;
pub const SF_UNKNOWN: u32 = 13;
pub const SF_DISPATCH: u32 = 9;
pub const SF_VARIANT: u32 = 12;
pub const SF_RECORD: u32 = 36;
pub const SF_HAVEIID: u32 = 13 | 0x8000;
/// oaidl.h TYPEFLAGS
pub const TYPEFLAG_FDUAL: u32 = 0x40;
/// oleauto.h VTBIT_* (1 << VT_x)
pub const VTBIT_NULL: u32 = 1 << 1;
pub const VTBIT_DECIMAL: u32 = 1 << 14;
/// bindgen tagSYSKIND_SYS_MAC = 2 (deepseek emitted the windows-sys spelling)
pub const _SYSKIND_SYS_MAC: u32 = 2;

// ---- V_RECORD accessors (oleauto.h:208-209, __STDC__ branch) ----
#[inline]
pub unsafe fn V_RECORD_ref(v: *mut VARIANT) -> *mut PVOID {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.brecVal.pvRecord)
}
#[inline]
pub unsafe fn V_RECORDINFO_ref(v: *mut VARIANT) -> *mut *mut IRecordInfo {
    core::ptr::addr_of_mut!((*v).n1.n2.n3.brecVal.pRecInfo)
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

// ---- minwindef.h MAKELONG ----
#[inline]
pub fn MAKELONG(lo: u16, hi: u16) -> u32 { (lo as u32) | ((hi as u32) << 16) }

// ---- winuser.h: #define CopyCursor(pcur) ((HCURSOR)CopyIcon((HICON)(pcur))) ----
#[inline]
pub unsafe fn CopyCursor(pcur: HCURSOR) -> HCURSOR { CopyIcon(pcur as HICON) as HCURSOR }

// ---- variant.c:1104-1105 DOS date/time packing ----
#[inline]
pub fn DOS_DATE(d: i32, m: i32, y: i32) -> i32 { d | (m << 5) | ((y - 1980) << 9) }
#[inline]
pub fn DOS_TIME(h: i32, m: i32, s: i32) -> i32 { (s >> 1) | (m << 5) | (h << 11) }

// ---- well-known IIDs (uuidgen-stable, from the SDK) ----
pub static IID_NULL: GUID = GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] };
pub static IID_IUnknown: GUID = GUID {
    Data1: 0x0000_0000, Data2: 0x0000, Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};
pub static IID_IDispatch: GUID = GUID {
    Data1: 0x0002_0400, Data2: 0x0000, Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

// ---- usrmarshal.c:42-45 NDR alignment macros (assignment macros → macro_rules) ----
#[macro_export]
macro_rules! ALIGN_LENGTH {
    ($len:expr, $align:expr) => {
        $len = (($len) + ($align)) & !($align)
    };
}
#[macro_export]
macro_rules! ALIGN_POINTER {
    ($ptr:expr, $align:expr) => {
        $ptr = ((($ptr) as usize + ($align) as usize) & !(($align) as usize)) as _
    };
}

// ---- winnt.h ARRAY_SIZE ----
// `#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))` (winnt_old.h:4762). A macro, not a
// fn, because it must take the ARRAY (a `[T; N]` place) and not decay it to a pointer —
// `.len()` on the array type is the exact Rust equivalent and is const-folded identically.
// (varformat.rs re-declares a function-local `macro_rules! ARRAY_SIZE` further down the
// file; macro_rules are textually scoped, so that one shadows this from its definition
// point onward and the earlier call sites — varformat.rs:1810/1829/1836 — need this one.)
#[macro_export]
macro_rules! ARRAY_SIZE {
    ($arr:expr) => {
        ($arr).len()
    };
}

/// Same `ARRAY_SIZE`, in the VALUE namespace. varformat.rs is inconsistent about which
/// spelling it uses — some sites are `ARRAY_SIZE!(buff)` (macro, handled above), others are
/// `ARRAY_SIZE(buff)` (function-call syntax: varformat.rs:964/1431/2242/2268/2275/2284), and
/// function-call syntax cannot invoke a macro_rules. A generic const fn renders
/// `sizeof(x)/sizeof(x[0])` exactly and type-safely: `N` comes from the array's own type, so
/// it cannot disagree with the buffer the way a hand-written literal count could.
///
/// The array is taken BY VALUE because that is how the call sites pass it. Every such buffer
/// is `[WCHAR; N]` (Copy), so this is a copy the optimiser deletes — `_a` is never read and
/// `N` is a compile-time constant. (A `&[T; N]` signature would be tidier but would not
/// resolve the existing call sites, which is the entire point.)
#[inline(always)]
pub const fn ARRAY_SIZE<T: Copy, const N: usize>(_a: [T; N]) -> usize {
    N
}

/// C `assert(expr)` called with function-call syntax (connpt.rs:296 —
/// `assert(nextslot == (*This).nSinks)`).
///
/// Rust's `assert` is a macro, so `assert(x)` finds nothing in the value namespace and fails
/// with E0423. This is the value-namespace companion. `debug_assert!` is the faithful
/// rendering, not `assert!`: ReactOS builds the DLLs with NDEBUG, under which
/// `<assert.h>`'s `assert` expands to nothing — so an unconditional panic here would abort
/// on an invariant the C deliberately does not check in a shipping build. Gating on
/// `debug_assertions` reproduces NDEBUG exactly.
#[inline(always)]
pub fn assert(cond: bool) {
    debug_assert!(cond, "oleaut32-rust: C assert() failed");
}

// ---- wine/debug.h:382 `#define WINE_MESSAGE wine_dbg_printf` (→ MESSAGE, debug.h:415) ----
// Unlike TRACE/WARN (no-ops) and ERR/FIXME (raw-format prints), MESSAGE is used for
// *interpolated* output — olepicture.rs:4505-4514 hex-dumps an unrecognised image header a
// byte at a time. `wine_dbg_printf` is a REAL exported symbol (wine/debug.h:180, defined in
// sdk/lib/rtl/wine_debug.c), so forward the varargs to it verbatim and keep the dump live
// rather than stubbing it into silence.
#[macro_export]
macro_rules! MESSAGE {
    ($($t:tt)*) => {{
        unsafe { $crate::debug_shims::wine_dbg_printf($($t)*) }
    }};
}

// ---- wine/list.h — the doubly-linked list olefont.c keeps its HFONT cache in ----
// `struct list` and its operations are `static inline` in the header (list.h:37-40, 78-113):
// they are NOT exported symbols; winelist-rust is the shared port of them.
pub use winelist_rust::{
    list, list_add_after, list_add_before, list_add_head, list_add_tail, list_head, list_init,
    list_next, list_remove,
};
/// rustconv spells `struct list *` as `*mut list_entry` in olefont.rs's extern block;
/// same type, different name. (Not to be confused with winnt's `LIST_ENTRY`, which is a
/// *different* struct — Flink/Blink — already mirrored in prelude.rs.)
pub type list_entry = list;

// ---- C `min(a, b)` (windef.h macro) → generic free fn (the `max` above's twin) ----
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}

// ---- winuser.h:589 `MAKEINTRESOURCEW(i) ((LPWSTR)(ULONG_PTR)(WORD)(i))` ----
#[inline]
pub fn MAKEINTRESOURCEW(i: WORD) -> LPWSTR {
    i as usize as LPWSTR
}

// ---- wingdi.h `RGB(r,g,b) ((COLORREF)(((BYTE)(r)|((WORD)((BYTE)(g))<<8))|(((DWORD)(BYTE)(b))<<16))) ----
#[inline]
pub fn RGB(r: u8, g: u8, b: u8) -> COLORREF {
    (r as COLORREF) | ((g as COLORREF) << 8) | ((b as COLORREF) << 16)
}

// ---- V_* rvalue accessors (oleauto.h V_UNION map; the `*_ref` lvalue forms are in
// winemacros_gen.rs — these are the read-only spellings the C uses in expressions) ----
#[inline]
pub unsafe fn V_VT(v: *const VARIANT) -> VARTYPE {
    (*v).n1.n2.vt
}
#[inline]
pub unsafe fn V_BSTR(v: *const VARIANT) -> BSTR {
    (*v).n1.n2.n3.bstrVal
}
#[inline]
pub unsafe fn V_R8(v: *const VARIANT) -> f64 {
    (*v).n1.n2.n3.dblVal
}
#[inline]
pub unsafe fn V_BOOL(v: *const VARIANT) -> VARIANT_BOOL {
    (*v).n1.n2.n3.boolVal
}
#[inline]
pub unsafe fn V_I2(v: *const VARIANT) -> SHORT {
    (*v).n1.n2.n3.iVal
}
#[inline]
pub unsafe fn V_I4(v: *const VARIANT) -> LONG {
    (*v).n1.n2.n3.lVal
}
#[inline]
pub unsafe fn V_UINT(v: *const VARIANT) -> UINT {
    (*v).n1.n2.n3.uintVal
}
#[inline]
pub unsafe fn V_CY(v: *const VARIANT) -> CY {
    (*v).n1.n2.n3.cyVal
}
#[inline]
pub unsafe fn V_DATE(v: *const VARIANT) -> DATE {
    (*v).n1.n2.n3.date
}

// ---- varformat.c:164-167 — the FMT_HEADER `starts[]` selectors ----
// These are file-local C macros, not SDK ones: each picks a section offset out of the parsed
// format header, falling back to the positive section when the specific one is absent. The
// fallback is the whole point ("#,##0" formats a negative number with the positive section),
// so it is transcribed exactly rather than simplified.
/// varformat.c:164 — `FmtGetPositive(x) (x->starts[0])`
#[inline]
pub unsafe fn FmtGetPositive(x: *const FMT_HEADER) -> u8 {
    (*x).starts[0]
}
/// varformat.c:165 — `FmtGetNegative(x) (x->starts[1] ? x->starts[1] : x->starts[0])`
#[inline]
pub unsafe fn FmtGetNegative(x: *const FMT_HEADER) -> u8 {
    if (*x).starts[1] != 0 { (*x).starts[1] } else { (*x).starts[0] }
}
/// varformat.c:166 — `FmtGetZero(x) (x->starts[2] ? x->starts[2] : x->starts[0])`
#[inline]
pub unsafe fn FmtGetZero(x: *const FMT_HEADER) -> u8 {
    if (*x).starts[2] != 0 { (*x).starts[2] } else { (*x).starts[0] }
}
/// varformat.c:167 — `FmtGetNull(x) (x->starts[3] ? x->starts[3] : x->starts[0])`
#[inline]
pub unsafe fn FmtGetNull(x: *const FMT_HEADER) -> u8 {
    if (*x).starts[3] != 0 { (*x).starts[3] } else { (*x).starts[0] }
}

// ---- wine debug channel macro forms (TRACE!(..) style call sites) ----
// ERR!/FIXME! are the shared winedebug-rust ones: the C format interpolated through
// DbgPrint. They used to print "(see source)" and swallow their arguments as raw
// tokens, so no argument here had ever been type-checked. `PCHAR` is what the
// translated format casts are spelled with (winnt.h's `typedef CHAR *PCHAR`).
pub type PCHAR = *mut core::ffi::c_char;
#[macro_export]
macro_rules! TRACE { ($($t:tt)*) => {{}} }
#[macro_export]
macro_rules! WARN { ($($t:tt)*) => {{}} }
#[macro_export]
macro_rules! ERR { ($($t:tt)*) => { ::winedebug_rust::ERR!($($t)*) } }
#[macro_export]
macro_rules! FIXME { ($($t:tt)*) => { ::winedebug_rust::FIXME!($($t)*) } }
