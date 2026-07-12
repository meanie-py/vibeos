//! Hand-written companions to winemacros_gen.rs: the wine/Win32 function-like
//! macros the rustconv translations call as plain functions, plus the handful
//! of constants the generator couldn't evaluate mechanically (each value
//! cross-checked against its defining header — cited inline).
#![allow(clippy::all)]
use crate::prelude::*;

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
/// variant.h: `SIGNSCALE(sign,scale) (((scale) << 8) | sign)` (LE branch)
#[inline]
pub fn SIGNSCALE(sign: u8, scale: u8) -> u16 { ((scale as u16) << 8) | sign as u16 }
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
pub unsafe fn V_ISBYREF(v: *mut VARIANT) -> bool { (((*v).n1.n2.vt & VT_BYREF) != 0) }
#[inline]
pub unsafe fn V_ISARRAY(v: *mut VARIANT) -> bool { (((*v).n1.n2.vt & VT_ARRAY) != 0) }
#[inline]
pub unsafe fn V_ISVECTOR(v: *mut VARIANT) -> bool { (((*v).n1.n2.vt & VT_VECTOR) != 0) }
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
