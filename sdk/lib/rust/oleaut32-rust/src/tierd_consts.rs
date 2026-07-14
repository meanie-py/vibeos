//! Constants for the Tier-D modules (connpt/olefont/olepicture/olepropframe/recinfo/
//! varformat), HARVESTED from the real SDK headers — every value carries a `file:line`
//! citation and nothing here was typed from memory.
//!
//! Why the paranoia: a wrong constant compiles, links, boots, and is silently wrong. The
//! ole32 pass shipped seven of these (e.g. `DISCARDCACHE_SAVEIFDIRTY` as 1 where the header
//! says 0, inverting save-on-discard). Regenerate/extend with:
//!
//!     python3 tools/harvest_consts.py NAME1 NAME2 ...
//!
//! The HRESULT families (`CONNECT_E_*`, `CTL_E_*`) are **derived with the header's own
//! macros** (`MAKE_SCODE`/`STD_CTL_SCODE`, below) rather than transcribed as hex, so the
//! derivation is visible and checked by the compiler instead of by my eyesight.
//!
//! TYPES MATTER AS MUCH AS VALUES. `E_*`/`DISP_E_*`/`CTL_E_*`/`TYPE_E_*`/`CONNECT_E_*`/
//! `OLE_E_*` are HRESULT (`i32`) — if they were `DWORD`, every `hr == E_NOINTERFACE` would
//! fail to typecheck. Integer constants are typed to the C expression they land in (a UINT
//! field, an `int` bitmask, a BYTE LOGFONT member), because C's implicit conversions have
//! no equivalent in Rust.
//!
//! Nothing here duplicates `consts.rs`, `winemacros.rs` or com-sys — a redefinition would be
//! an ambiguous-glob error (E0659), and the overlap was checked before writing.
#![allow(clippy::all)]
use crate::prelude::*;

// ---------------------------------------------------------------------------
// HRESULT/SCODE construction (winerror.h:182-183, olectl.h:264-266)
// ---------------------------------------------------------------------------
/// winerror.h:178
pub const SEVERITY_ERROR: u32 = 1;
/// winerror.h:28
pub const FACILITY_ITF: u32 = 4;
/// olectl.h:264 (`#ifndef FACILITY_CONTROL` → 0xa; winerror.h:33 agrees: 10)
pub const FACILITY_CONTROL: u32 = 0xa;

/// winerror.h:182 — `((SCODE)(((unsigned int)(sev)<<31) | ((unsigned int)(fac)<<16) | ((unsigned int)(code))))`
#[inline]
pub const fn MAKE_SCODE(sev: u32, fac: u32, code: u32) -> HRESULT {
    ((sev << 31) | (fac << 16) | code) as HRESULT
}
/// winerror.h:180 — MAKE_HRESULT has the identical body (differs only in the cast type).
#[inline]
pub const fn MAKE_HRESULT(sev: u32, fac: u32, code: u32) -> HRESULT {
    ((sev << 31) | (fac << 16) | code) as HRESULT
}
/// olectl.h:266 — `MAKE_SCODE(SEVERITY_ERROR, FACILITY_CONTROL, n)`
#[inline]
pub const fn STD_CTL_SCODE(n: u32) -> HRESULT {
    MAKE_SCODE(SEVERITY_ERROR, FACILITY_CONTROL, n)
}

// ---------------------------------------------------------------------------
// HRESULTs (i32). Derived, not transcribed.
// ---------------------------------------------------------------------------
/// olectl.h:246 — MAKE_SCODE(SEVERITY_ERROR, FACILITY_ITF, 0x0200) == 0x80040200
pub const CONNECT_E_FIRST: HRESULT = MAKE_SCODE(SEVERITY_ERROR, FACILITY_ITF, 0x0200);
/// olectl.h:251 — CONNECT_E_FIRST+0
pub const CONNECT_E_NOCONNECTION: HRESULT = CONNECT_E_FIRST;
/// olectl.h:253 — CONNECT_E_FIRST+2
pub const CONNECT_E_CANNOTCONNECT: HRESULT = CONNECT_E_FIRST + 2;

/// olectl.h:274 — STD_CTL_SCODE(53) == 0x800A0035
pub const CTL_E_FILENOTFOUND: HRESULT = STD_CTL_SCODE(53);
/// olectl.h:287 — STD_CTL_SCODE(75) == 0x800A004B
pub const CTL_E_PATHFILEACCESSERROR: HRESULT = STD_CTL_SCODE(75);
/// olectl.h:292 — STD_CTL_SCODE(380) == 0x800A017C
pub const CTL_E_INVALIDPROPERTYVALUE: HRESULT = STD_CTL_SCODE(380);

/// winerror.h:3479
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
/// winerror.h:3614
pub const DISP_E_UNKNOWNINTERFACE: HRESULT = 0x80020001u32 as HRESULT;
/// winerror.h:3615
pub const DISP_E_MEMBERNOTFOUND: HRESULT = 0x80020003u32 as HRESULT;
/// winerror.h:3618
pub const DISP_E_UNKNOWNNAME: HRESULT = 0x80020006u32 as HRESULT;
/// winerror.h:3626
pub const DISP_E_BADPARAMCOUNT: HRESULT = 0x8002000Eu32 as HRESULT;
/// winerror.h:3627
pub const DISP_E_PARAMNOTOPTIONAL: HRESULT = 0x8002000Fu32 as HRESULT;
/// winerror.h:3632
pub const TYPE_E_BUFFERTOOSMALL: HRESULT = 0x80028016u32 as HRESULT;
/// winerror.h:3633
pub const TYPE_E_FIELDNOTFOUND: HRESULT = 0x80028017u32 as HRESULT;
/// winerror.h:3729
pub const OLE_E_NOCONNECTION: HRESULT = 0x80040004u32 as HRESULT;

// ---------------------------------------------------------------------------
// DISPIDs (DISPID = LONG = i32). olectl.h:427-439, oaidl.h:757.
// ---------------------------------------------------------------------------
/// oaidl.h:757 — DISPID_UNKNOWN (-1)
pub const DISPID_UNKNOWN: DISPID = -1;
/// oleauto.h:1003 — `#define MEMBERID_NIL DISPID_UNKNOWN`
pub const MEMBERID_NIL: DISPID = DISPID_UNKNOWN;

/// olectl.h:427
pub const DISPID_FONT_NAME: DISPID = 0;
/// olectl.h:428
pub const DISPID_FONT_SIZE: DISPID = 2;
/// olectl.h:429
pub const DISPID_FONT_BOLD: DISPID = 3;
/// olectl.h:430
pub const DISPID_FONT_ITALIC: DISPID = 4;
/// olectl.h:431
pub const DISPID_FONT_UNDER: DISPID = 5;
/// olectl.h:432
pub const DISPID_FONT_STRIKE: DISPID = 6;
/// olectl.h:433
pub const DISPID_FONT_WEIGHT: DISPID = 7;
/// olectl.h:434
pub const DISPID_FONT_CHARSET: DISPID = 8;
/// olectl.h:435
pub const DISPID_FONT_CHANGED: DISPID = 9;
/// olectl.h:438
pub const DISPID_PICT_HANDLE: DISPID = 0;
/// olectl.h:439
pub const DISPID_PICT_HPAL: DISPID = 2;

// ---------------------------------------------------------------------------
// PICTYPE_* — olectl.h:131-136.
//
// Typed UINT, not int, ON PURPOSE: every use site scrutinises `PICTDESC.picType`,
// which the header declares `UINT` (win_sys: tagPICTDESC.picType: UINT). The C compares
// the `(-1)` literal against that UINT after the usual arithmetic conversions, i.e. as
// 0xFFFFFFFF — so PICTYPE_UNINITIALIZED must be `0xFFFF_FFFF: UINT` here for the match
// arms to both typecheck AND mean what the C means.
// ---------------------------------------------------------------------------
/// olectl.h:131 — `(-1)`, compared against the UINT `PICTDESC.picType`
pub const PICTYPE_UNINITIALIZED: UINT = 0xFFFF_FFFF;
/// olectl.h:132
pub const PICTYPE_NONE: UINT = 0;
/// olectl.h:133
pub const PICTYPE_BITMAP: UINT = 1;
/// olectl.h:134
pub const PICTYPE_METAFILE: UINT = 2;
/// olectl.h:135
pub const PICTYPE_ICON: UINT = 3;
/// olectl.h:136
pub const PICTYPE_ENHMETAFILE: UINT = 4;

/// ocidl.h (enum tagPICTUREATTRIBUTES) — PICTURE_SCALABLE
pub const PICTURE_SCALABLE: DWORD = 0x1;
/// ocidl.h (enum tagPICTUREATTRIBUTES) — PICTURE_TRANSPARENT
pub const PICTURE_TRANSPARENT: DWORD = 0x2;

// ---------------------------------------------------------------------------
// Automation enums (oaidl.h). Typed to the C expression they land in.
// ---------------------------------------------------------------------------
/// oaidl.h (enum tagINVOKEKIND) — INVOKE_FUNC.
/// NOTE: recinfo.rs compares these against a `wFlags: ULONG` (recinfo.rs:847), so ULONG is
/// the fit for 3 of the 4 call sites. olefont.rs:490 passes INVOKE_FUNC into an
/// `IFontEventsDisp_Invoke(.., wFlags: WORD, ..)` slot, where the C narrowed implicitly —
/// that one site needs an explicit cast in olefont.rs (flagged; not fixable from here).
pub const INVOKE_FUNC: ULONG = 1;
/// oaidl.h (enum tagINVOKEKIND) — INVOKE_PROPERTYPUT
pub const INVOKE_PROPERTYPUT: ULONG = 4;
/// oaidl.h (enum tagINVOKEKIND) — INVOKE_PROPERTYPUTREF
pub const INVOKE_PROPERTYPUTREF: ULONG = 8;

/// oaidl.h (enum tagTYPEKIND) — TKIND_RECORD (TKIND_INTERFACE=3 already in consts.rs)
pub const TKIND_RECORD: u32 = 1;
/// oaidl.h (enum tagTYPEKIND) — TKIND_ALIAS
pub const TKIND_ALIAS: u32 = 6;
/// oaidl.h (enum tagVARKIND) — VAR_PERINSTANCE
pub const VAR_PERINSTANCE: u32 = 0;

/// oleauto.h:1006 — `DISPATCH_METHOD 0x1`. Typed `int`: the use site is
/// `wFlags as c_int & DISPATCH_METHOD` (olepicture.rs:5484), the C's integer-promoted form.
pub const DISPATCH_METHOD: c_int = 0x1;
/// oleauto.h:1008 — `DISPATCH_PROPERTYPUT 0x4`; use site `wFlags as c_int & ..` (olefont.rs:1530)
pub const DISPATCH_PROPERTYPUT: c_int = 0x4;

/// oleauto.h:748 — `NUMPRS_STD 0x01FFF`
pub const NUMPRS_STD: u32 = 0x01FFF;
/// oleauto.h:331 — `VAR_FORMAT_NOSUBSTITUTE 0x020`
pub const VAR_FORMAT_NOSUBSTITUTE: u32 = 0x020;
/// dll/win32/oleaut32/variant.h:102 — "Convert bool to On/Off" (oleaut32-private, not in the PSDK)
pub const VAR_BOOLONOFF: u32 = 0x0400;
/// dll/win32/oleaut32/variant.h:103 — "Convert bool to Yes/No" (oleaut32-private, not in the PSDK)
pub const VAR_BOOLYESNO: u32 = 0x0800;

/// wtypes.h:646 — `VARIANT_FALSE ((VARIANT_BOOL)0x0000)`. VARIANT_BOOL is i16.
/// (VARIANT_TRUE already lives in consts.rs; note it is 0xFFFF = -1, never 1.)
pub const VARIANT_FALSE: VARIANT_BOOL = 0;

/// wtypes.h (enum tagCLSCTX) — CLSCTX_INPROC_SERVER
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;

// ---------------------------------------------------------------------------
// winnls.h — locale info IDs (LCTYPE = DWORD). Values are the header's, verbatim.
// ---------------------------------------------------------------------------
/// winnls.h:22
pub const LOCALE_RETURN_NUMBER: LCTYPE = 0x20000000;
/// winnls.h:51
pub const LOCALE_SGROUPING: LCTYPE = 16;
/// winnls.h:52
pub const LOCALE_IDIGITS: LCTYPE = 17;
/// winnls.h:53
pub const LOCALE_ILZERO: LCTYPE = 18;
/// winnls.h:54
pub const LOCALE_INEGNUMBER: LCTYPE = 0x1010;
/// winnls.h:63
pub const LOCALE_ICURRENCY: LCTYPE = 27;
/// winnls.h:64
pub const LOCALE_INEGCURR: LCTYPE = 28;
/// winnls.h:65
pub const LOCALE_SDATE: LCTYPE = 29;
/// winnls.h:66
pub const LOCALE_STIME: LCTYPE = 30;
/// winnls.h:67
pub const LOCALE_SSHORTDATE: LCTYPE = 31;
/// winnls.h:68
pub const LOCALE_SLONGDATE: LCTYPE = 32;
/// winnls.h:78
pub const LOCALE_S1159: LCTYPE = 40;
/// winnls.h:79
pub const LOCALE_S2359: LCTYPE = 41;
/// winnls.h:82
pub const LOCALE_IFIRSTDAYOFWEEK: LCTYPE = 0x100C;
/// winnls.h:84
pub const LOCALE_SDAYNAME1: LCTYPE = 42;
/// winnls.h:91
pub const LOCALE_SABBREVDAYNAME1: LCTYPE = 49;
/// winnls.h:98
pub const LOCALE_SMONTHNAME1: LCTYPE = 56;
/// winnls.h:111
pub const LOCALE_SABBREVMONTHNAME1: LCTYPE = 68;

/// ntbasedef.h:1395 — `MAKELCID(MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL), SORT_DEFAULT)`
/// = MAKELCID(0, 0) = 0. (LANG_NEUTRAL=0 nls.h:22, SUBLANG_NEUTRAL=0 nls.h:167,
/// SORT_DEFAULT=0 ntbasedef.h:1368.)
pub const LOCALE_NEUTRAL: LCID = 0;

/// winnls.h:245
pub const CP_ACP: UINT = 0;
/// winnls.h:500 — CompareStringW's "equal" return, an `int`
pub const CSTR_EQUAL: c_int = 2;

// ---------------------------------------------------------------------------
// wingdi.h — LOGFONTW members are BYTE/LONG, so these are typed `int` (the C's
// promoted literal type) and cast at the store, exactly as the C does.
// ---------------------------------------------------------------------------
/// wingdi.h:40
pub const LF_FACESIZE: c_int = 32;
/// wingdi.h:373
pub const FW_NORMAL: c_int = 400;
/// wingdi.h:378
pub const FW_BOLD: c_int = 700;
/// wingdi.h:384
pub const DEFAULT_CHARSET: c_int = 1;
/// wingdi.h:417
pub const OUT_CHARACTER_PRECIS: c_int = 2;
/// wingdi.h:426
pub const CLIP_DEFAULT_PRECIS: c_int = 0;
/// wingdi.h:436
pub const DEFAULT_QUALITY: c_int = 0;
/// wingdi.h:443
pub const DEFAULT_PITCH: c_int = 0;
/// wingdi.h:695 — OBJ_BITMAP (GetCurrentObject's UINT type selector)
pub const OBJ_BITMAP: UINT = 7;
/// wingdi.h:719 — GetDeviceCaps index (int)
pub const LOGPIXELSY: c_int = 90;
/// wingdi.h:867 — SetMapMode mode (int)
pub const MM_ANISOTROPIC: c_int = 8;

// ---------------------------------------------------------------------------
// winuser.h
// ---------------------------------------------------------------------------
/// winuser.h:779
pub const SW_HIDE: c_int = 0;
/// winuser.h:786
pub const SW_SHOW: c_int = 5;

/// winuser.h:881-883 — `DWLP_MSGRESULT 0`, `DWLP_DLGPROC (DWLP_MSGRESULT + sizeof(LRESULT))`,
/// `DWLP_USER (DWLP_DLGPROC + sizeof(DLGPROC))`. **Pointer-width dependent**: 0+4+4 = 8 on
/// i386, 0+8+8 = 16 on amd64/arm64. Computed, not transcribed, so it cannot drift per arch.
pub const DWLP_MSGRESULT: usize = 0;
pub const DWLP_DLGPROC: usize = DWLP_MSGRESULT + core::mem::size_of::<LRESULT>();
pub const DWLP_USER: usize = DWLP_DLGPROC + core::mem::size_of::<*mut c_void>(); // sizeof(DLGPROC)

/// winuser.h:604 — `RT_DIALOG MAKEINTRESOURCE(5)`, i.e. the integer 5 cast to a resource
/// pointer at the call site (`RT_DIALOG as LPWSTR`, olepropframe.rs:311).
pub const RT_DIALOG: usize = 5;

// ---------------------------------------------------------------------------
// kernel32 / winerror
// ---------------------------------------------------------------------------
/// setypes.h:27
pub const GENERIC_READ: DWORD = 0x80000000;
/// winbase.h:228
pub const OPEN_EXISTING: DWORD = 3;
/// winbase.h:528
pub const INVALID_FILE_SIZE: DWORD = 0xFFFFFFFF;
/// winerror.h:227
pub const ERROR_FILE_NOT_FOUND: DWORD = 2;
/// ntrxdef.h:6 — `((HANDLE)(LONG_PTR)-1)`
pub const INVALID_HANDLE_VALUE: HANDLE = !0usize as HANDLE;

/// typedefs.h:43-44 — BOOL is `int`
pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;

// ---- olectl.h DISPIDs for IPicture's IDispatch::Invoke ----
//
// These are NOT cosmetic. `OLEPictureImpl_Invoke` switches on `dispIdMember`, and the
// translation lowered that `switch` to a Rust `match`. In Rust an identifier that is not
// a `const` in scope is an irrefutable BINDING, not a comparison — so with these missing,
// the FIRST arm swallowed every dispid and the rest were dead code. IPicture would have
// returned the wrong property for every request, and it compiles clean.
// Caught by `tools/verify_match_consts.py`; the identical shape shipped in ole32
// (StgOpenStorageEx treated every stgfmt as STGFMT_ANY). Field manual §9.72.
//
// Typed `DISPID` (= LONG = i32, com-sys bindgen_i386.rs:7431), matching the scrutinee —
// as DWORD they would not typecheck against `dispIdMember`, and "fixing" that with a cast
// would just re-hide the bug.
// (HANDLE and HPAL are already above; only the four the `match` actually swallowed were
// missing. The two definitions were derived independently and agree — 0 and 2.)
/// sdk/include/psdk/olectl.h:440
pub const DISPID_PICT_TYPE: DISPID = 3;
/// sdk/include/psdk/olectl.h:441
pub const DISPID_PICT_WIDTH: DISPID = 4;
/// sdk/include/psdk/olectl.h:442
pub const DISPID_PICT_HEIGHT: DISPID = 5;
/// sdk/include/psdk/olectl.h:443
pub const DISPID_PICT_RENDER: DISPID = 6;
