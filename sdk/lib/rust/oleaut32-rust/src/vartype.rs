use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn _wcsnicmp(s1: *const u16, s2: *const u16, n: u32) -> i32;
    fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32;
    fn iswdigit(wc: u32) -> i32;
    fn iswalpha(wc: u32) -> i32;
    fn iswspace(wc: u32) -> i32;
    // CRT math/string used directly by vartype.c
    fn ceil(x: f64) -> f64;
    fn floor(x: f64) -> f64;
    fn wcscmp(a: *const u16, b: *const u16) -> i32;
    fn _wcsicmp(a: *const u16, b: *const u16) -> i32; // C calls wcsicmp (msvcrt alias)
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcsspn(s: *const u16, accept: *const u16) -> usize;
    fn swprintf(buf: *mut u16, format: *const u16, ...) -> i32; // legacy msvcrt 2-arg form
    // hProxyDll: defined by the widl proxy unit (rpcproxy.h DLLDATA routines,
    // proxy.dlldata.c) — same DLL, C side (vartype.c:36 extern HMODULE hProxyDll).
    static mut hProxyDll: *mut c_void;
}

// winnls.h NUMBERFMTW / CURRENCYFMTW (local mirrors; kernel32 NLS formatting)
#[repr(C)]
pub struct NUMBERFMTW {
    pub NumDigits: u32,
    pub LeadingZero: u32,
    pub Grouping: u32,
    pub lpDecimalSep: *mut u16,
    pub lpThousandSep: *mut u16,
    pub NegativeOrder: u32,
}
#[repr(C)]
pub struct CURRENCYFMTW {
    pub NumDigits: u32,
    pub LeadingZero: u32,
    pub Grouping: u32,
    pub lpDecimalSep: *mut u16,
    pub lpThousandSep: *mut u16,
    pub NegativeOrder: u32,
    pub PositiveOrder: u32,
    pub lpCurrencySymbol: *mut u16,
}

extern "system" {
    fn FindResourceExW(hModule: *mut c_void, lpType: *const u16, lpName: *const u16, wLanguage: u16) -> *mut c_void;
    fn LoadResource(hModule: *mut c_void, hResInfo: *mut c_void) -> *mut c_void;
    fn LockResource(hResData: *mut c_void) -> *mut c_void;
    fn GetNumberFormatW(Locale: LCID, dwFlags: u32, lpValue: *const u16, lpFormat: *const NUMBERFMTW, lpNumberStr: *mut u16, cchNumber: i32) -> i32;
    fn GetCurrencyFormatW(Locale: LCID, dwFlags: u32, lpValue: *const u16, lpFormat: *const CURRENCYFMTW, lpCurrencyStr: *mut u16, cchCurrency: i32) -> i32;
    fn GetTimeFormatW(Locale: LCID, dwFlags: u32, lpTime: *const SYSTEMTIME, lpFormat: *const u16, lpTimeStr: *mut u16, cchTime: i32) -> i32;
    fn CompareStringW(Locale: LCID, dwCmpFlags: u32, lpString1: *const u16, cchCount1: i32, lpString2: *const u16, cchCount2: i32) -> i32;
    fn GetSystemTime(lpSystemTime: *mut SYSTEMTIME);
}

/// winuser.h MAKEINTRESOURCEW
#[inline]
fn MAKEINTRESOURCEW(i: u16) -> *const u16 {
    i as usize as *const u16
}

// variant.h size constraints / flags missing from the shared consts surface
const DATE_MAX: i64 = 2958465; // variant.h:69
const DEC_MAX_SCALE: u8 = 28; // variant.h:99
const VAR_BOOLONOFF: u32 = 0x0400; // variant.h:102
const VAR_BOOLYESNO: u32 = 0x0800; // variant.h:103
const VAR_NEGATIVE: u32 = 0x1000; // variant.h:104
// oleaut32 resource.h string ids
const IDS_TRUE: u32 = 100;
const IDS_FALSE: u32 = 101;
const IDS_YES: u32 = 102;
const IDS_ON: u32 = 104;
// winnt.h / winnls.h
const LANG_NEUTRAL: u16 = 0x00;
const SUBLANG_DEFAULT: u16 = 0x01;
const CSTR_LESS_THAN: i32 = 1;
const LOCALE_SSHORTDATE: u32 = 31;
// VARIANT_BOOL values (shadow consts.rs's u32 VARIANT_TRUE with the C-typed one)
const VARIANT_TRUE: VARIANT_BOOL = -1;
const VARIANT_FALSE: VARIANT_BOOL = 0;


// ---- file-scope preamble (translated) ----
pub const CY_MULTIPLIER: i32 = 10000;
pub const CY_MULTIPLIER_F: f64 = 10000.0;
pub const CY_HALF: i32 = CY_MULTIPLIER / 2;
pub const CY_HALF_F: f64 = CY_MULTIPLIER_F / 2.0;

pub static szFloatFormatW: [u16; 5] = [
    '%' as u16,
    '.' as u16,
    '7' as u16,
    'G' as u16,
    '\0' as u16,
];

pub static szDoubleFormatW: [u16; 6] = [
    '%' as u16,
    '.' as u16,
    '1' as u16,
    '5' as u16,
    'G' as u16,
    '\0' as u16,
];

pub static CY_Divisors: [i32; 5] = [
    CY_MULTIPLIER / 10000,
    CY_MULTIPLIER / 1000,
    CY_MULTIPLIER / 100,
    CY_MULTIPLIER / 10,
    CY_MULTIPLIER,
];

#[repr(C)]
pub struct DECIMAL_internal {
    pub bitsnum: [u32; 3],
    pub scale: u8,
    pub sign: u32, /* single bitfield, only bit 0 used */
}
pub type VARIANT_DI = DECIMAL_internal;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct R4_FIELDS_Bits {
    pub _bitfield: u32,
}
#[repr(C)]
pub union R4_FIELDS {
    pub i: R4_FIELDS_Bits,
    pub f: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct R8_FIELDS_Bits {
    pub _bitfield: u64,
}
#[repr(C)]
pub union R8_FIELDS {
    pub i: R8_FIELDS_Bits,
    pub d: f64,
}

pub const DP_TIMESEP: u32 = 0x01;
pub const DP_DATESEP: u32 = 0x02;
pub const DP_MONTH: u32 = 0x04;
pub const DP_AM: u32 = 0x08;
pub const DP_PM: u32 = 0x10;

#[repr(C)]
pub struct tagDATEPARSE {
    pub dwCount: u32,
    pub dwParseFlags: u32,
    pub dwFlags: [u32; 6],
    pub dwValues: [u32; 6],
}
pub type DATEPARSE = tagDATEPARSE;

pub const ORDER_MDY: u32 = 0x01;
pub const ORDER_YMD: u32 = 0x02;
pub const ORDER_YDM: u32 = 0x04;
pub const ORDER_DMY: u32 = 0x08;
pub const ORDER_MYD: u32 = 0x10;

const I2_MAX: i32 = 32767;

// Declare external functions and constants needed

const VAR_TIMEVALUEONLY: u32 = 1;
const VAR_DATEVALUEONLY: u32 = 2;
const LOCALE_RETURN_NUMBER: u32 = 0x20000000;
const LOCALE_NOUSEROVERRIDE: u32 = 0x80000000;
const LOCALE_SMONTHNAME1: u32 = 0x38;
const LOCALE_SMONTHNAME2: u32 = 0x39;
const LOCALE_SMONTHNAME3: u32 = 0x3A;
const LOCALE_SMONTHNAME4: u32 = 0x3B;
const LOCALE_SMONTHNAME5: u32 = 0x3C;
const LOCALE_SMONTHNAME6: u32 = 0x3D;
const LOCALE_SMONTHNAME7: u32 = 0x3E;
const LOCALE_SMONTHNAME8: u32 = 0x3F;
const LOCALE_SMONTHNAME9: u32 = 0x40;
const LOCALE_SMONTHNAME10: u32 = 0x41;
const LOCALE_SMONTHNAME11: u32 = 0x42;
const LOCALE_SMONTHNAME12: u32 = 0x43;
const LOCALE_SMONTHNAME13: u32 = 0x100E;
const LOCALE_SABBREVMONTHNAME1: u32 = 0x44;
const LOCALE_SABBREVMONTHNAME2: u32 = 0x45;
const LOCALE_SABBREVMONTHNAME3: u32 = 0x46;
const LOCALE_SABBREVMONTHNAME4: u32 = 0x47;
const LOCALE_SABBREVMONTHNAME5: u32 = 0x48;
const LOCALE_SABBREVMONTHNAME6: u32 = 0x49;
const LOCALE_SABBREVMONTHNAME7: u32 = 0x4A;
const LOCALE_SABBREVMONTHNAME8: u32 = 0x4B;
const LOCALE_SABBREVMONTHNAME9: u32 = 0x4C;
const LOCALE_SABBREVMONTHNAME10: u32 = 0x4D;
const LOCALE_SABBREVMONTHNAME11: u32 = 0x4E;
const LOCALE_SABBREVMONTHNAME12: u32 = 0x4F;
const LOCALE_SABBREVMONTHNAME13: u32 = 0x100F;
const LOCALE_SDAYNAME1: u32 = 0x2A;
const LOCALE_SDAYNAME2: u32 = 0x2B;
const LOCALE_SDAYNAME3: u32 = 0x2C;
const LOCALE_SDAYNAME4: u32 = 0x2D;
const LOCALE_SDAYNAME5: u32 = 0x2E;
const LOCALE_SDAYNAME6: u32 = 0x2F;
const LOCALE_SDAYNAME7: u32 = 0x30;
const LOCALE_SABBREVDAYNAME1: u32 = 0x31;
const LOCALE_SABBREVDAYNAME2: u32 = 0x32;
const LOCALE_SABBREVDAYNAME3: u32 = 0x33;
const LOCALE_SABBREVDAYNAME4: u32 = 0x34;
const LOCALE_SABBREVDAYNAME5: u32 = 0x35;
const LOCALE_SABBREVDAYNAME6: u32 = 0x36;
const LOCALE_SABBREVDAYNAME7: u32 = 0x37;
const LOCALE_S1159: u32 = 0x28;
const LOCALE_S2359: u32 = 0x29;
const LOCALE_SDATE: u32 = 0x1D;
const LOCALE_IDATE: u32 = 0x21;
const TOKEN_COUNT: usize = 43; // ARRAY_SIZE(ParseDateTokens)

static PARSE_DATE_TOKENS: [u32; TOKEN_COUNT] = [
    LOCALE_SMONTHNAME1, LOCALE_SMONTHNAME2, LOCALE_SMONTHNAME3, LOCALE_SMONTHNAME4,
    LOCALE_SMONTHNAME5, LOCALE_SMONTHNAME6, LOCALE_SMONTHNAME7, LOCALE_SMONTHNAME8,
    LOCALE_SMONTHNAME9, LOCALE_SMONTHNAME10, LOCALE_SMONTHNAME11, LOCALE_SMONTHNAME12,
    LOCALE_SMONTHNAME13,
    LOCALE_SABBREVMONTHNAME1, LOCALE_SABBREVMONTHNAME2, LOCALE_SABBREVMONTHNAME3,
    LOCALE_SABBREVMONTHNAME4, LOCALE_SABBREVMONTHNAME5, LOCALE_SABBREVMONTHNAME6,
    LOCALE_SABBREVMONTHNAME7, LOCALE_SABBREVMONTHNAME8, LOCALE_SABBREVMONTHNAME9,
    LOCALE_SABBREVMONTHNAME10, LOCALE_SABBREVMONTHNAME11, LOCALE_SABBREVMONTHNAME12,
    LOCALE_SABBREVMONTHNAME13,
    LOCALE_SDAYNAME1, LOCALE_SDAYNAME2, LOCALE_SDAYNAME3, LOCALE_SDAYNAME4,
    LOCALE_SDAYNAME5, LOCALE_SDAYNAME6, LOCALE_SDAYNAME7,
    LOCALE_SABBREVDAYNAME1, LOCALE_SABBREVDAYNAME2, LOCALE_SABBREVDAYNAME3,
    LOCALE_SABBREVDAYNAME4, LOCALE_SABBREVDAYNAME5, LOCALE_SABBREVDAYNAME6,
    LOCALE_SABBREVDAYNAME7,
    LOCALE_S1159, LOCALE_S2359,
    LOCALE_SDATE,
];

static PARSE_DATE_MONTHS: [u8; 26] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
];

unsafe fn VARIANT_CopyData(srcVar: *const VARIANT, vt: u16, pOut: *mut core::ffi::c_void) {
    match vt {
        VT_I1 | VT_UI1 => core::ptr::copy_nonoverlapping(
            &(*srcVar).n1.n2.n3.bVal as *const u8,
            pOut as *mut u8,
            1,
        ),
        VT_BOOL | VT_I2 | VT_UI2 => core::ptr::copy_nonoverlapping(
            &(*srcVar).n1.n2.n3.iVal as *const i16,
            pOut as *mut i16,
            1,
        ),
        VT_R4 | VT_INT | VT_I4 | VT_UINT | VT_UI4 => core::ptr::copy_nonoverlapping(
            &(*srcVar).n1.n2.n3.lVal as *const i32,
            pOut as *mut i32,
            1,
        ),
        VT_R8 | VT_DATE | VT_CY | VT_I8 | VT_UI8 => core::ptr::copy_nonoverlapping(
            &(*srcVar).n1.n2.n3.llVal as *const i64,
            pOut as *mut i64,
            1,
        ),
        VT_INT_PTR => core::ptr::copy_nonoverlapping(
            &(*srcVar).n1.n2.n3.llVal as *const i64 as *const u8,
            pOut as *mut u8,
            core::mem::size_of::<isize>(),
        ),
        VT_DECIMAL => core::ptr::copy_nonoverlapping(
            &(*srcVar).n1.decVal as *const DECIMAL,
            pOut as *mut DECIMAL,
            1,
        ),
        VT_BSTR => core::ptr::copy_nonoverlapping(
            &(*srcVar).n1.n2.n3.bstrVal as *const BSTR,
            pOut as *mut BSTR,
            1,
        ),
        _ => {
            FIXME!(b"VT_ type %d unhandled, please report!\0".as_ptr() as *const i8, vt as i32);
        }
    }
}

unsafe fn VARIANT_NumberFromBstr(
    pStrIn: *mut u16,
    lcid: u32,
    ulFlags: u32,
    pOut: *mut core::ffi::c_void,
    vt: u16,
) -> HRESULT {
    let mut dstVar: VARIANT = core::mem::zeroed();
    let mut np: NUMPARSE = core::mem::zeroed();
    let mut rgb: [u8; 1024] = [0; 1024];

    np.cDig = 1024;
    np.dwInFlags = 0x01FFFu32; // NUMPRS_STD (oleauto.h:748 — all parse flags)

    let mut hRet = VarParseNumFromStr(
        pStrIn,
        lcid,
        ulFlags,
        &mut np as *mut NUMPARSE,
        rgb.as_mut_ptr(),
    );

    if hRet >= 0 {
        hRet = VarNumFromParseNum(
            &mut np as *mut NUMPARSE,
            rgb.as_mut_ptr(),
            1u32 << vt,
            &mut dstVar as *mut VARIANT,
        );
        if hRet >= 0 {
            VARIANT_CopyData(&dstVar as *const VARIANT, vt, pOut);
        }
    }

    hRet
}

unsafe fn VARIANT_FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pOut: *mut core::ffi::c_void,
    vt: VARTYPE,
    dwFlags: DWORD,
) -> HRESULT {
    // C: static DISPPARAMS emptyParams = { NULL, NULL, 0, 0 }; (read-only —
    // a per-call local is semantically identical and avoids Sync on raw ptrs)
    let mut emptyParams: DISPPARAMS = DISPPARAMS {
        rgvarg: core::ptr::null_mut(),
        rgdispidNamedArgs: core::ptr::null_mut(),
        cArgs: 0,
        cNamedArgs: 0,
    };
    let mut srcVar: VARIANT = core::mem::zeroed();
    let mut dstVar: VARIANT = core::mem::zeroed();
    let mut hRet: HRESULT;

    if pdispIn.is_null() {
        return DISP_E_BADVARTYPE;
    }

    VariantInit(&mut srcVar);
    hRet = IDispatch_Invoke(
        pdispIn,
        DISPID_VALUE as DISPID,
        &IID_NULL,
        lcid,
        DISPATCH_PROPERTYGET as u16,
        &mut emptyParams,
        &mut srcVar,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if hRet >= 0 {
        VariantInit(&mut dstVar);
        hRet = VariantChangeTypeEx(&mut dstVar, &mut srcVar, lcid, dwFlags as u16, vt);
        VariantClear(&mut srcVar);

        if hRet >= 0 {
            VARIANT_CopyData(&dstVar, vt, pOut);
        }
    } else {
        hRet = DISP_E_TYPEMISMATCH;
    }

    hRet
}
/* Exported conversion wrappers — each just calls its _Var* inline helper,
   exactly as the C exports do (vartype.c:282 onward). */

/* I1 */
#[no_mangle]
pub unsafe extern "system" fn VarI1FromUI1(bIn: u8, pcOut: *mut i8) -> HRESULT {
    _VarI1FromUI1(bIn, pcOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI1FromI2(sIn: i16, pcOut: *mut i8) -> HRESULT {
    _VarI1FromI2(sIn, pcOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI1FromI4(lIn: i32, pcOut: *mut i8) -> HRESULT {
    _VarI1FromI4(lIn, pcOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI1FromBool(boolIn: VARIANT_BOOL, pcOut: *mut i8) -> HRESULT {
    _VarI1FromBool(boolIn, pcOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI1FromUI2(usIn: u16, pcOut: *mut i8) -> HRESULT {
    _VarI1FromUI2(usIn, pcOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI1FromUI4(ulIn: u32, pcOut: *mut i8) -> HRESULT {
    _VarI1FromUI4(ulIn, pcOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI1FromI8(llIn: i64, pcOut: *mut i8) -> HRESULT {
    _VarI1FromI8(llIn, pcOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI1FromUI8(ullIn: u64, pcOut: *mut i8) -> HRESULT {
    _VarI1FromUI8(ullIn, pcOut)
}

/* UI1 */
#[no_mangle]
pub unsafe extern "system" fn VarUI1FromI2(sIn: i16, pbOut: *mut u8) -> HRESULT {
    _VarUI1FromI2(sIn, pbOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI1FromBool(boolIn: VARIANT_BOOL, pbOut: *mut u8) -> HRESULT {
    _VarUI1FromBool(boolIn, pbOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI1FromI1(cIn: i8, pbOut: *mut u8) -> HRESULT {
    _VarUI1FromI1(cIn, pbOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI1FromUI2(usIn: u16, pbOut: *mut u8) -> HRESULT {
    _VarUI1FromUI2(usIn, pbOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI1FromI4(lIn: i32, pbOut: *mut u8) -> HRESULT {
    _VarUI1FromI4(lIn, pbOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI1FromUI4(ulIn: u32, pbOut: *mut u8) -> HRESULT {
    _VarUI1FromUI4(ulIn, pbOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI1FromI8(llIn: i64, pbOut: *mut u8) -> HRESULT {
    _VarUI1FromI8(llIn, pbOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI1FromUI8(ullIn: u64, pbOut: *mut u8) -> HRESULT {
    _VarUI1FromUI8(ullIn, pbOut)
}

/* I2 */
#[no_mangle]
pub unsafe extern "system" fn VarI2FromUI1(bIn: u8, psOut: *mut i16) -> HRESULT {
    _VarI2FromUI1(bIn, psOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI2FromI4(lIn: i32, psOut: *mut i16) -> HRESULT {
    _VarI2FromI4(lIn, psOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI2FromBool(boolIn: VARIANT_BOOL, psOut: *mut i16) -> HRESULT {
    _VarI2FromBool(boolIn, psOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI2FromI1(cIn: i8, psOut: *mut i16) -> HRESULT {
    _VarI2FromI1(cIn, psOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI2FromUI2(usIn: u16, psOut: *mut i16) -> HRESULT {
    _VarI2FromUI2(usIn, psOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI2FromUI4(ulIn: u32, psOut: *mut i16) -> HRESULT {
    _VarI2FromUI4(ulIn, psOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI2FromI8(llIn: i64, psOut: *mut i16) -> HRESULT {
    _VarI2FromI8(llIn, psOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI2FromUI8(ullIn: u64, psOut: *mut i16) -> HRESULT {
    _VarI2FromUI8(ullIn, psOut)
}

/* UI2 */
#[no_mangle]
pub unsafe extern "system" fn VarUI2FromUI1(bIn: u8, pusOut: *mut u16) -> HRESULT {
    _VarUI2FromUI1(bIn, pusOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI2FromI2(sIn: i16, pusOut: *mut u16) -> HRESULT {
    _VarUI2FromI2(sIn, pusOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI2FromI4(lIn: i32, pusOut: *mut u16) -> HRESULT {
    _VarUI2FromI4(lIn, pusOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI2FromBool(boolIn: VARIANT_BOOL, pusOut: *mut u16) -> HRESULT {
    _VarUI2FromBool(boolIn, pusOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI2FromI1(cIn: i8, pusOut: *mut u16) -> HRESULT {
    _VarUI2FromI1(cIn, pusOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI2FromUI4(ulIn: u32, pusOut: *mut u16) -> HRESULT {
    _VarUI2FromUI4(ulIn, pusOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI2FromI8(llIn: i64, pusOut: *mut u16) -> HRESULT {
    _VarUI2FromI8(llIn, pusOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI2FromUI8(ullIn: u64, pusOut: *mut u16) -> HRESULT {
    _VarUI2FromUI8(ullIn, pusOut)
}

/* I4 */
#[no_mangle]
pub unsafe extern "system" fn VarI4FromUI1(bIn: u8, piOut: *mut i32) -> HRESULT {
    _VarI4FromUI1(bIn, piOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI4FromI2(sIn: i16, piOut: *mut i32) -> HRESULT {
    _VarI4FromI2(sIn, piOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI4FromBool(boolIn: VARIANT_BOOL, piOut: *mut i32) -> HRESULT {
    _VarI4FromBool(boolIn, piOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI4FromI1(cIn: i8, piOut: *mut i32) -> HRESULT {
    _VarI4FromI1(cIn, piOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI4FromUI2(usIn: u16, piOut: *mut i32) -> HRESULT {
    _VarI4FromUI2(usIn, piOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI4FromUI4(ulIn: u32, piOut: *mut i32) -> HRESULT {
    _VarI4FromUI4(ulIn, piOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI4FromI8(llIn: i64, piOut: *mut i32) -> HRESULT {
    _VarI4FromI8(llIn, piOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI4FromUI8(ullIn: u64, piOut: *mut i32) -> HRESULT {
    _VarI4FromUI8(ullIn, piOut)
}

/* UI4 */
#[no_mangle]
pub unsafe extern "system" fn VarUI4FromUI1(bIn: u8, pulOut: *mut u32) -> HRESULT {
    _VarUI4FromUI1(bIn, pulOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI4FromI2(sIn: i16, pulOut: *mut u32) -> HRESULT {
    _VarUI4FromI2(sIn, pulOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI4FromI4(lIn: i32, pulOut: *mut u32) -> HRESULT {
    _VarUI4FromI4(lIn, pulOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI4FromBool(boolIn: VARIANT_BOOL, pulOut: *mut u32) -> HRESULT {
    _VarUI4FromBool(boolIn, pulOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI4FromI1(cIn: i8, pulOut: *mut u32) -> HRESULT {
    _VarUI4FromI1(cIn, pulOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI4FromUI2(usIn: u16, pulOut: *mut u32) -> HRESULT {
    _VarUI4FromUI2(usIn, pulOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI4FromI8(llIn: i64, pulOut: *mut u32) -> HRESULT {
    _VarUI4FromI8(llIn, pulOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI4FromUI8(ullIn: u64, pulOut: *mut u32) -> HRESULT {
    _VarUI4FromUI8(ullIn, pulOut)
}

/* I8 */
#[no_mangle]
pub unsafe extern "system" fn VarI8FromUI1(bIn: u8, pi64Out: *mut i64) -> HRESULT {
    _VarI8FromUI1(bIn, pi64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarI8FromI2(sIn: i16, pi64Out: *mut i64) -> HRESULT {
    _VarI8FromI2(sIn, pi64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarI8FromI1(cIn: i8, pi64Out: *mut i64) -> HRESULT {
    _VarI8FromI1(cIn, pi64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarI8FromUI2(usIn: u16, pi64Out: *mut i64) -> HRESULT {
    _VarI8FromUI2(usIn, pi64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarI8FromUI4(ulIn: u32, pi64Out: *mut i64) -> HRESULT {
    _VarI8FromUI4(ulIn, pi64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarI8FromUI8(ullIn: u64, pi64Out: *mut i64) -> HRESULT {
    _VarI8FromUI8(ullIn, pi64Out)
}

/* UI8 */
#[no_mangle]
pub unsafe extern "system" fn VarUI8FromUI1(bIn: u8, pui64Out: *mut u64) -> HRESULT {
    _VarUI8FromUI1(bIn, pui64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI8FromI2(sIn: i16, pui64Out: *mut u64) -> HRESULT {
    _VarUI8FromI2(sIn, pui64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI8FromI1(cIn: i8, pui64Out: *mut u64) -> HRESULT {
    _VarUI8FromI1(cIn, pui64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI8FromUI2(usIn: u16, pui64Out: *mut u64) -> HRESULT {
    _VarUI8FromUI2(usIn, pui64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI8FromUI4(ulIn: u32, pui64Out: *mut u64) -> HRESULT {
    _VarUI8FromUI4(ulIn, pui64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI8FromI8(llIn: i64, pui64Out: *mut u64) -> HRESULT {
    _VarUI8FromI8(llIn, pui64Out)
}

/* R4 */
#[no_mangle]
pub unsafe extern "system" fn VarR4FromUI1(bIn: u8, pFltOut: *mut f32) -> HRESULT {
    _VarR4FromUI1(bIn, pFltOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR4FromI2(sIn: i16, pFltOut: *mut f32) -> HRESULT {
    _VarR4FromI2(sIn, pFltOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR4FromI1(cIn: i8, pFltOut: *mut f32) -> HRESULT {
    _VarR4FromI1(cIn, pFltOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR4FromUI2(usIn: u16, pFltOut: *mut f32) -> HRESULT {
    _VarR4FromUI2(usIn, pFltOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR4FromI4(lIn: i32, pFltOut: *mut f32) -> HRESULT {
    _VarR4FromI4(lIn, pFltOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR4FromUI4(ulIn: u32, pFltOut: *mut f32) -> HRESULT {
    _VarR4FromUI4(ulIn, pFltOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR4FromI8(llIn: i64, pFltOut: *mut f32) -> HRESULT {
    _VarR4FromI8(llIn, pFltOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR4FromUI8(ullIn: u64, pFltOut: *mut f32) -> HRESULT {
    _VarR4FromUI8(ullIn, pFltOut)
}

/* R8 */
#[no_mangle]
pub unsafe extern "system" fn VarR8FromUI1(bIn: u8, pDblOut: *mut f64) -> HRESULT {
    _VarR8FromUI1(bIn, pDblOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR8FromI2(sIn: i16, pDblOut: *mut f64) -> HRESULT {
    _VarR8FromI2(sIn, pDblOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarR8FromR4(fltIn: f32, pDblOut: *mut f64) -> HRESULT {
    _VarR8FromR4(fltIn, pDblOut)
}

unsafe fn _VarR8FromCy(i: CY, o: *mut f64) -> i32 {
    *o = (i.int64 as f64) / CY_MULTIPLIER_F;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarI1FromR4(fltIn: f32, pcOut: *mut i8) -> HRESULT {
    VarI1FromR8(fltIn as f64, pcOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarI1FromR8(dblIn: f64, pcOut: *mut i8) -> HRESULT {
    if dblIn < -128.5 || dblIn >= 127.5 {
        return DISP_E_OVERFLOW;
    }
    let whole = if dblIn < 0.0 { ceil(dblIn) } else { floor(dblIn) };
    let fract = dblIn - whole;
    let result = if fract > 0.5 {
        (whole as i8).wrapping_add(1)
    } else if fract == 0.5 {
        let is_odd = (whole as i8) & 1;
        if is_odd != 0 { (whole as i8).wrapping_add(1) } else { whole as i8 }
    } else if fract >= 0.0 {
        whole as i8
    } else if fract == -0.5 {
        let is_odd = (whole as i8) & 1;
        if is_odd != 0 { (whole as i8).wrapping_sub(1) } else { whole as i8 }
    } else if fract > -0.5 {
        whole as i8
    } else {
        (whole as i8).wrapping_sub(1)
    };
    *pcOut = result;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarI1FromDate(dateIn: f64, pcOut: *mut i8) -> HRESULT {
    VarI1FromR8(dateIn, pcOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarI1FromCy(cyIn: CY, pcOut: *mut i8) -> HRESULT {
    let mut i: i32 = 128; // I1_MAX + 1
    VarI4FromCy(cyIn, &mut i);
    _VarI1FromI4(i, pcOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarI1FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pcOut: *mut i8,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pcOut as *mut core::ffi::c_void, 16u16)
}

#[no_mangle]
pub unsafe extern "system" fn VarI1FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pcOut: *mut i8,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pcOut as *mut core::ffi::c_void, VT_I1, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarI1FromDec(pdecIn: *const DECIMAL, pcOut: *mut i8) -> i32 {
    let mut i64: i64 = 0;
    let hret = VarI8FromDec(pdecIn, &raw mut i64);
    if hret >= 0 {
        _VarI1FromI8(i64, pcOut)
    } else {
        hret
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarUI1FromR4(fltIn: f32, pbOut: *mut u8) -> HRESULT {
    VarUI1FromR8(fltIn as f64, pbOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI1FromR8(dblIn: f64, pbOut: *mut u8) -> HRESULT {
    const UI1_MAX: f64 = 255.0;
    if dblIn < -0.5 || dblIn >= UI1_MAX + 0.5 {
        return DISP_E_OVERFLOW;
    }
    let whole = if dblIn < 0.0 { ceil(dblIn) } else { floor(dblIn) };
    let fract = dblIn - whole;
    let whole_int = whole as i32;
    let is_odd = (whole_int as u8) & 1 != 0;
    let res = if fract > 0.5 {
        whole + 1.0
    } else if fract == 0.5 {
        if is_odd { whole + 1.0 } else { whole }
    } else if fract >= 0.0 {
        whole
    } else if fract == -0.5 {
        if is_odd { whole - 1.0 } else { whole }
    } else if fract > -0.5 {
        whole
    } else {
        whole - 1.0
    };
    *pbOut = res as u8;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarUI1FromCy(cyIn: _CY, pbOut: *mut u8) -> i32 {
    let mut i: u32 = 0x100; /* UI1_MAX + 1 */
    VarUI4FromCy(cyIn, &mut i as *mut u32);
    _VarUI1FromUI4(i, pbOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI1FromDate(dateIn: f64, pbOut: *mut u8) -> HRESULT {
    VarUI1FromR8(dateIn, pbOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI1FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pbOut: *mut u8,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pbOut as *mut core::ffi::c_void, VT_UI1)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI1FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pbOut: *mut u8,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pbOut as *mut core::ffi::c_void, VT_UI1, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI1FromDec(pdecIn: *const DECIMAL, pbOut: *mut u8) -> i32 {
    let mut i64: i64 = 0;
    let hRet = VarI8FromDec(pdecIn, &mut i64);
    if hRet >= 0 {
        _VarUI1FromI8(i64, pbOut)
    } else {
        hRet
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarI2FromR4(fltIn: f32, psOut: *mut i16) -> HRESULT {
    VarI2FromR8(fltIn as f64, psOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarI2FromR8(dblIn: f64, psOut: *mut i16) -> HRESULT {
    if dblIn < -32768.5 || dblIn >= 32767.5 {
        return DISP_E_OVERFLOW;
    }
    let value = dblIn;
    let whole = if value < 0.0 { ceil(value) } else { floor(value) };
    let fract = value - whole;
    if fract > 0.5 {
        *psOut = (whole as i16) + 1;
    } else if fract == 0.5 {
        let is_odd: i16 = (whole as i16) & 1;
        *psOut = (whole as i16) + is_odd;
    } else if fract >= 0.0 {
        *psOut = whole as i16;
    } else if fract == -0.5 {
        let is_odd: i16 = (whole as i16) & 1;
        *psOut = (whole as i16) - is_odd;
    } else if fract > -0.5 {
        *psOut = whole as i16;
    } else {
        *psOut = (whole as i16) - 1;
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarI2FromCy(cyIn: CY, psOut: *mut SHORT) -> HRESULT {
    let mut i: LONG = I2_MAX + 1;
    VarI4FromCy(cyIn, &mut i);
    _VarI2FromI4(i, psOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarI2FromDate(dateIn: f64, psOut: *mut i16) -> HRESULT {
    VarI2FromR8(dateIn, psOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI2FromStr(strIn: *mut u16, lcid: u32, dwFlags: u32, psOut: *mut i16) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, psOut as *mut core::ffi::c_void, VT_I2)
}

#[no_mangle]
pub unsafe extern "system" fn VarI2FromDisp(
    pdispIn: *mut IDispatch,
    lcid: u32,
    psOut: *mut i16,
) -> i32 {
    VARIANT_FromDisp(
        pdispIn,
        lcid,
        psOut as *mut core::ffi::c_void,
        VT_I2,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn VarI2FromDec(pdecIn: *const DECIMAL, psOut: *mut i16) -> i32 {
    let mut i64: i64 = 0;
    let mut hRet: i32;

    hRet = VarI8FromDec(pdecIn, &raw mut i64);
    if hRet >= 0 {
        hRet = _VarI2FromI8(i64, psOut);
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarUI2FromR4(fltIn: f32, pusOut: *mut u16) -> HRESULT {
    VarUI2FromR8(fltIn as f64, pusOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI2FromR8(dblIn: f64, pusOut: *mut u16) -> HRESULT {
    if dblIn < -0.5 || dblIn >= (UI2_MAX as f64 + 0.5) {
        return DISP_E_OVERFLOW;
    }
    let whole = if dblIn < 0.0 { ceil(dblIn) } else { floor(dblIn) };
    let fract = dblIn - whole;
    let result_f64 = if fract > 0.5 {
        whole + 1.0
    } else if fract == 0.5 {
        let is_odd = (whole as i32 as u16) & 1;
        whole + (is_odd as f64)
    } else if fract >= 0.0 {
        whole
    } else if fract == -0.5 {
        let is_odd = (whole as i32 as u16) & 1;
        whole - (is_odd as f64)
    } else if fract > -0.5 {
        whole
    } else {
        whole - 1.0
    };
    *pusOut = result_f64 as u16;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarUI2FromDate(dateIn: f64, pusOut: *mut u16) -> HRESULT {
    VarUI2FromR8(dateIn, pusOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI2FromCy(cyIn: _CY, pusOut: *mut u16) -> i32 {
    const UI2_MAX: u32 = 0xFFFF;
    let mut i: u32 = UI2_MAX + 1;
    VarUI4FromCy(cyIn, &mut i);
    _VarUI2FromUI4(i, pusOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI2FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pusOut: *mut u16,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pusOut as *mut core::ffi::c_void, VT_UI2)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI2FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pusOut: *mut USHORT,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pusOut as *mut core::ffi::c_void, VT_UI2, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI2FromDec(pdecIn: *const DECIMAL, pusOut: *mut USHORT) -> HRESULT {
    let mut i64_out: i64 = 0;
    let hret = VarI8FromDec(pdecIn, core::ptr::addr_of_mut!(i64_out));
    if hret >= 0 {
        _VarUI2FromI8(i64_out, pusOut)
    } else {
        hret
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarI4FromR4(fltIn: f32, piOut: *mut i32) -> HRESULT {
    VarI4FromR8(fltIn as f64, piOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarI4FromR8(dblIn: f64, piOut: *mut i32) -> HRESULT {
    if dblIn < (core::i32::MIN as f64) - 0.5 || dblIn >= (core::i32::MAX as f64) + 0.5 {
        return DISP_E_OVERFLOW;
    }
    let value = dblIn;
    let whole = if value < 0.0 { ceil(value) } else { floor(value) };
    let fract = value - whole;
    let res = if fract > 0.5 {
        (whole as i32).wrapping_add(1)
    } else if fract == 0.5 {
        let is_odd = (whole as i32) & 1;
        (whole as i32).wrapping_add(is_odd)
    } else if fract >= 0.0 {
        whole as i32
    } else if fract == -0.5 {
        let is_odd = (whole as i32) & 1;
        (whole as i32).wrapping_sub(is_odd)
    } else if fract > -0.5 {
        whole as i32
    } else {
        (whole as i32).wrapping_sub(1)
    };
    *piOut = res;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarI4FromCy(cyIn: CY, piOut: *mut i32) -> HRESULT {
    let d = cyIn.int64 as f64 / 10000.0_f64;
    VarI4FromR8(d, piOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarI4FromDate(dateIn: f64, piOut: *mut i32) -> HRESULT {
    VarI4FromR8(dateIn, piOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarI4FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    piOut: *mut i32,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, piOut as *mut core::ffi::c_void, _VARENUM_VT_I4)
}

#[no_mangle]
pub unsafe extern "system" fn VarI4FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    piOut: *mut LONG,
) -> HRESULT {
    VARIANT_FromDisp(
        pdispIn,
        lcid,
        piOut as *mut core::ffi::c_void,
        VT_I4, /* VT_I4 = 3 */
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn VarI4FromDec(pdecIn: *const DECIMAL, piOut: *mut i32) -> i32 {
    let mut i64: i64 = 0;
    let hRet = VarI8FromDec(pdecIn, &mut i64);
    if hRet >= 0 {
        return _VarI4FromI8(i64, piOut);
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarUI4FromR4(fltIn: f32, pulOut: *mut u32) -> i32 {
    VarUI4FromR8(fltIn as f64, pulOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI4FromR8(dblIn: f64, pulOut: *mut u32) -> i32 {
    if dblIn < -0.5 || dblIn >= u32::MAX as f64 + 0.5 {
        return DISP_E_OVERFLOW;
    }
    let whole = if dblIn < 0.0 { ceil(dblIn) } else { floor(dblIn) };
    let fract = dblIn - whole;
    let res = if fract > 0.5 {
        (whole as u32) + 1u32
    } else if fract == 0.5 {
        let is_odd = ((whole as u32) & 1) != 0;
        let is_odd_val = if is_odd { 1.0 } else { 0.0 };
        (whole + is_odd_val) as u32
    } else if fract >= 0.0 {
        whole as u32
    } else if fract == -0.5 {
        let is_odd = ((whole as u32) & 1) != 0;
        let is_odd_val = if is_odd { 1.0 } else { 0.0 };
        (whole - is_odd_val) as u32
    } else if fract > -0.5 {
        whole as u32
    } else {
        (whole as u32) - 1u32
    };
    *pulOut = res;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarUI4FromDate(dateIn: f64, pulOut: *mut u32) -> i32 {
    VarUI4FromR8(dateIn, pulOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI4FromCy(cyIn: _CY, pulOut: *mut u32) -> i32 {
    let d = (cyIn.int64 as f64) / 10000.0;
    VarUI4FromR8(d, pulOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI4FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pulOut: *mut u32,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pulOut as *mut core::ffi::c_void, VT_UI4)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI4FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pulOut: *mut ULONG,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pulOut as *mut core::ffi::c_void, VT_UI4, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI4FromDec(pdecIn: *const DECIMAL, pulOut: *mut u32) -> i32 {
    let mut i64: i64 = 0;
    let hRet = VarI8FromDec(pdecIn, &mut i64);
    if hRet >= 0 {
        return _VarUI4FromI8(i64, pulOut);
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarI8FromR4(fltIn: f32, pi64Out: *mut i64) -> i32 {
    VarI8FromR8(fltIn as f64, pi64Out)
}

#[no_mangle]
pub unsafe extern "system" fn VarI8FromR8(dblIn: f64, pi64Out: *mut i64) -> i32 {
    if dblIn < -4611686018427387904.0 || dblIn >= 4611686018427387904.0 {
        return DISP_E_OVERFLOW;
    }
    let whole = if dblIn < 0.0 { ceil(dblIn) } else { floor(dblIn) };
    let fract = dblIn - whole;
    let out = if fract > 0.5 {
        (whole as i64) + 1i64
    } else if fract == 0.5 {
        let is_odd = (whole as i64) & 1;
        whole as i64 + is_odd
    } else if fract >= 0.0 {
        whole as i64
    } else if fract == -0.5 {
        let is_odd = (whole as i64) & 1;
        whole as i64 - is_odd
    } else if fract > -0.5 {
        whole as i64
    } else {
        (whole as i64) - 1i64
    };
    *pi64Out = out;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarI8FromCy(cyIn: CY, pi64Out: *mut LONG64) -> HRESULT {
    let mut cy_local = cyIn;
    *pi64Out = cy_local.int64 / 10000;
    if cy_local.int64 < 0 {
        *pi64Out -= 1;
    } else {
        cy_local.int64 -= *pi64Out * 10000;
        let lo = cy_local.__bindgen_anon_1.Lo;
        if lo > 5000 || (lo == 5000 && (*pi64Out & 1) != 0) {
            *pi64Out += 1;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarI8FromDate(dateIn: f64, pi64Out: *mut i64) -> i32 {
    VarI8FromR8(dateIn, pi64Out)
}
#[no_mangle]
 pub unsafe extern "system" fn VarI8FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pi64Out: *mut i64,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pi64Out as *mut core::ffi::c_void, VT_I8)
}

#[no_mangle]
pub unsafe extern "system" fn VarI8FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pi64Out: *mut LONG64,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pi64Out as *mut core::ffi::c_void, VT_I8, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarI8FromBool(boolIn: i16, pi64Out: *mut i64) -> i32 {
    VarI8FromI2(boolIn, pi64Out)
}

#[no_mangle]
pub unsafe extern "system" fn VarI8FromDec(pdecIn: *const DECIMAL, pi64Out: *mut i64) -> i32 {
    let scale = (*pdecIn).__bindgen_anon_1.__bindgen_anon_1.scale;
    if scale == 0 {
        let sign = (*pdecIn).__bindgen_anon_1.__bindgen_anon_1.sign;
        if (sign & !0x80u8) != 0 {
            return E_INVALIDARG;
        }
        if (*pdecIn).Hi32 != 0 || ((*pdecIn).__bindgen_anon_2.__bindgen_anon_1.Mid32 & 0x80000000u32) != 0 {
            return DISP_E_OVERFLOW;
        }
        let lo = (*pdecIn).__bindgen_anon_2.Lo64;
        if (sign & 0x80u8) != 0 {
            *pi64Out = -(lo as i64);
        } else {
            *pi64Out = lo as i64;
        }
        S_OK
    } else {
        let mut dbl: f64 = 0.0;
        let hRet = VarR8FromDec(pdecIn, &mut dbl);
        if hRet >= 0 {
            VarI8FromR8(dbl, pi64Out)
        } else {
            hRet
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn VarUI8FromR4(fltIn: f32, pui64Out: *mut ULONG64) -> HRESULT {
    VarUI8FromR8(fltIn as f64, pui64Out)
}
#[no_mangle]
pub unsafe extern "system" fn VarUI8FromR8(dblIn: f64, pui64Out: *mut ULONG64) -> HRESULT {
    if dblIn < -0.5 || dblIn > 1.844674407370955e19 {
        return (-2147352566i32) as HRESULT; // DISP_E_OVERFLOW
    }

    let value = dblIn;
    let whole = if value < 0.0 { ceil(value) } else { floor(value) };
    let fract = value - whole;
    let res = if fract > 0.5 {
        (whole as ULONG64).wrapping_add(1)
    } else if fract == 0.5 {
        let is_odd = (whole as ULONG64) & 1;
        (whole as ULONG64) + is_odd
    } else if fract >= 0.0 {
        whole as ULONG64
    } else if fract == -0.5 {
        let is_odd = (whole as ULONG64) & 1;
        (whole as ULONG64) - is_odd
    } else if fract > -0.5 {
        whole as ULONG64
    } else {
        (whole as ULONG64).wrapping_sub(1)
    };

    *pui64Out = res;
    0i32 as HRESULT // S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarUI8FromCy(mut cyIn: CY, pui64Out: *mut ULONG64) -> HRESULT {
    if cyIn.int64 < 0 {
        if cyIn.int64 < -(CY_HALF as LONGLONG) {
            return DISP_E_OVERFLOW;
        }
        *pui64Out = 0;
    } else {
        *pui64Out = (cyIn.int64 / CY_MULTIPLIER as LONGLONG) as ULONG64;
        cyIn.int64 -= (*pui64Out as LONGLONG) * (CY_MULTIPLIER as LONGLONG);
        let lo = cyIn.int64 as ULONG;
        if lo > CY_HALF as ULONG || (lo == CY_HALF as ULONG && (*pui64Out & 1) != 0) {
            *pui64Out += 1;
        }
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarUI8FromDate(dateIn: f64, pui64Out: *mut ULONG64) -> HRESULT {
    VarUI8FromR8(dateIn, pui64Out)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI8FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pui64Out: *mut u64,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pui64Out as *mut core::ffi::c_void, VT_UI8)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI8FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pui64Out: *mut ULONG64,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pui64Out as *mut core::ffi::c_void, VT_UI8, 0u32)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI8FromBool(boolIn: i16, pui64Out: *mut u64) -> i32 {
    VarI8FromI2(boolIn, pui64Out as *mut i64)
}

#[no_mangle]
pub unsafe extern "system" fn VarUI8FromDec(pdecIn: *mut DECIMAL, pui64Out: *mut ULONG64) -> HRESULT {
    let scale = (*pdecIn).__bindgen_anon_1.__bindgen_anon_1.scale;
    if scale == 0 {
        let sign = (*pdecIn).__bindgen_anon_1.__bindgen_anon_1.sign;
        if sign & !0x80u8 != 0 {
            return E_INVALIDARG;
        }

        if (*pdecIn).Hi32 != 0 {
            return DISP_E_OVERFLOW;
        }

        if sign != 0 {
            // WARN("Sign would be ignored under Win32!\n");  /* debug output omitted */
            return DISP_E_OVERFLOW;
        }

        *pui64Out = (*pdecIn).__bindgen_anon_2.__bindgen_anon_1.Lo32 as ULONG64
            | ((*pdecIn).__bindgen_anon_2.__bindgen_anon_1.Mid32 as ULONG64) << 32;
        return S_OK;
    } else {
        let mut dbl: f64 = 0.0;
        let hRet = VarR8FromDec(pdecIn, &mut dbl);
        if hRet >= 0 {
            return VarUI8FromR8(dbl, pui64Out);
        }
        return hRet;
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarR4FromR8(dblIn: f64, pFltOut: *mut f32) -> HRESULT {
    let d = if dblIn < 0.0 { -dblIn } else { dblIn };
    if d > R4_MAX {
        return DISP_E_OVERFLOW;
    }
    *pFltOut = dblIn as f32;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarR4FromCy(cyIn: CY, pFltOut: *mut FLOAT) -> HRESULT {
    *pFltOut = (cyIn.int64 as f64 / 10000.0f64) as f32;
    0 // S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarR4FromDate(dateIn: f64, pFltOut: *mut f32) -> HRESULT {
    VarR4FromR8(dateIn, pFltOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR4FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pFltOut: *mut f32,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pFltOut as *mut core::ffi::c_void, 4u16)
}
#[no_mangle]
pub unsafe extern "system" fn VarR4FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pFltOut: *mut f32,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pFltOut as *mut core::ffi::c_void, VT_R4, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarR4FromBool(boolIn: VARIANT_BOOL, pFltOut: *mut f32) -> HRESULT {
    VarR4FromI2(boolIn, pFltOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR4FromDec(pDecIn: *mut DECIMAL, pFltOut: *mut f32) -> HRESULT {
    let scale: u8 = DEC_SCALE(pDecIn);
    let mut divisor: f64 = 1.0;

    if scale > DEC_MAX_SCALE || (DEC_SIGN(pDecIn) & !DECIMAL_NEG) != 0 {
        return E_INVALIDARG;
    }

    let mut remaining = scale;
    while remaining > 0 {
        divisor *= 10.0;
        remaining -= 1;
    }

    if DEC_SIGN(pDecIn) != 0 {
        divisor = -divisor;
    }

    let hi32 = *DEC_HI32_ref(pDecIn) ;
    let high_part = if hi32 != 0 {
        let mut hp = (hi32 as f64) / divisor;
        hp *= 4294967296.0;
        hp *= 4294967296.0;
        hp
    } else {
        0.0
    };

    *pFltOut = (((*DEC_LO64_ref(pDecIn)) as f64) / divisor + high_part) as f32;
    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn VarR4CmpR8(fltLeft: f32, dblRight: f64) -> HRESULT {
    if (fltLeft as f64) < dblRight {
        VARCMP_LT
    } else if (fltLeft as f64) > dblRight {
        VARCMP_GT
    } else {
        VARCMP_EQ
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromI4(lIn: i32, pDblOut: *mut f64) -> HRESULT {
    _VarR8FromI4(lIn, pDblOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromCy(cyIn: CY, pDblOut: *mut f64) -> i32 {
    _VarR8FromCy(cyIn, pDblOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromDate(dateIn: f64, pDblOut: *mut f64) -> HRESULT {
    _VarR8FromDate(dateIn, pDblOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pDblOut: *mut f64,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pDblOut as *mut core::ffi::c_void, VT_R8)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pDblOut: *mut f64,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pDblOut as *mut core::ffi::c_void, VT_R8, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromBool(boolIn: i16, pDblOut: *mut f64) -> HRESULT {
    VarR8FromI2(boolIn, pDblOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromI1(cIn: i8, pDblOut: *mut f64) -> i32 {
    _VarR8FromI1(cIn, pDblOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromUI2(usIn: u16, pDblOut: *mut f64) -> i32 {
    _VarR8FromUI2(usIn, pDblOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromUI4(ulIn: u32, pDblOut: *mut f64) -> i32 {
    *pDblOut = ulIn as f64;
    0
}

// DeepSeek emitted this body without its signature (orphaned block); wrapped
// by hand against the C: HRESULT WINAPI VarR8FromDec(const DECIMAL*, double*).
#[no_mangle]
pub unsafe extern "system" fn VarR8FromDec(pDecIn: *const DECIMAL, pDblOut: *mut f64) -> HRESULT {
    const DEC_MAX_SCALE: u8 = 28;
    const DECIMAL_NEG: u8 = 0x80;

    let mut scale = (*(pDecIn)).__bindgen_anon_1.__bindgen_anon_1.scale;
    let sign = (*(pDecIn)).__bindgen_anon_1.__bindgen_anon_1.sign;
    if scale > DEC_MAX_SCALE || (sign & !DECIMAL_NEG) != 0 {
        return 0x80070057u32 as HRESULT; // E_INVALIDARG
    }

    let mut divisor: f64 = 1.0;
    while scale > 0 {
        divisor *= 10.0;
        scale -= 1;
    }

    if sign != 0 {
        divisor = -divisor;
    }

    let mut high_part: f64;
    let hi32 = (*(pDecIn)).Hi32;
    if hi32 != 0 {
        high_part = (hi32 as f64) / divisor;
        high_part *= 4294967296.0f64;
        high_part *= 4294967296.0f64;
    } else {
        high_part = 0.0;
    }

    let lo64 = (*(pDecIn)).__bindgen_anon_2.Lo64;
    *pDblOut = (lo64 as f64) / divisor + high_part;

    0 // S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromI8(llIn: i64, pDblOut: *mut f64) -> i32 {
    _VarR8FromI8(llIn, pDblOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8FromUI8(ullIn: ULONG64, pDblOut: *mut f64) -> HRESULT {
    _VarR8FromUI8(ullIn, pDblOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarR8Pow(dblLeft: f64, dblPow: f64, pDblOut: *mut f64) -> HRESULT {
    *pDblOut = pow(dblLeft, dblPow);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarR8Round(dblIn: f64, nDig: i32, pDblOut: *mut f64) -> HRESULT {
    if nDig < 0 {
        return E_INVALIDARG;
    }
    let scale = pow(10.0, nDig as f64);
    let mut dbl = dblIn * scale;
    let whole = if dbl < 0.0 { ceil(dbl) } else { floor(dbl) };
    let fract = dbl - whole;
    dbl = if fract > 0.5 {
        whole + 1.0
    } else if fract == 0.5 {
        whole + fmod(whole, 2.0)
    } else if fract >= 0.0 {
        whole
    } else if fract == -0.5 {
        // Wine 10.0 (upstream e05d78ede06): `+`, not `-`. fmod keeps the sign
        // of `whole`, so for -3.5 this is -3 + -1 = -4 (round half to even);
        // the old `-` gave -2.
        whole + fmod(whole, 2.0)
    } else if fract > -0.5 {
        whole
    } else {
        whole - 1.0
    };
    *pDblOut = dbl / scale;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromUI1(bIn: u8, pCyOut: *mut CY) -> HRESULT {
    (*pCyOut).int64 = (bIn as u64 * CY_MULTIPLIER as u64) as i64;
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromI2(sIn: i16, pCyOut: *mut CY) -> i32 {
    (*pCyOut).int64 = (sIn as i64) * 10000;
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromI4(lIn: i32, pCyOut: *mut CY) -> HRESULT {
    (*pCyOut).int64 = (lIn as i64).wrapping_mul(10000);
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromR4(fltIn: f32, pCyOut: *mut CY) -> HRESULT {
    VarCyFromR8(fltIn as f64, pCyOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromR8(dblIn: f64, pCyOut: *mut CY) -> HRESULT {
    if dblIn < -922337203685477.5807_f64 || dblIn >= 922337203685477.5807_f64 {
        return DISP_E_OVERFLOW;
    }
    let value = dblIn * 10000.0_f64;
    let whole = if value < 0.0_f64 { ceil(value) } else { floor(value) };
    let fract = value - whole;
    let res: i64;
    if fract > 0.5 {
        res = (whole as i64) + 1;
    } else if fract == 0.5 {
        let is_odd = (whole as i64) & 1;
        res = (whole as i64) + is_odd;
    } else if fract >= 0.0 {
        res = whole as i64;
    } else if fract == -0.5 {
        let is_odd = (whole as i64) & 1;
        res = (whole as i64) - is_odd;
    } else if fract > -0.5 {
        res = whole as i64;
    } else {
        res = (whole as i64) - 1;
    }
    (*pCyOut).int64 = res;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromDate(dateIn: f64, pCyOut: *mut CY) -> HRESULT {
    VarCyFromR8(dateIn, pCyOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromStr(
    strIn: *mut u16,
    lcid: LCID,
    dwFlags: u32,
    pCyOut: *mut CY,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pCyOut as *mut core::ffi::c_void, VT_CY)
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pCyOut: *mut CY,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pCyOut as *mut core::ffi::c_void, VT_CY, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromBool(
    boolIn: VARIANT_BOOL,
    pCyOut: *mut CY,
) -> HRESULT {
    (*pCyOut).int64 = (boolIn as i64) * 10000i64;
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromI1(cIn: i8, pCyOut: *mut CY) -> HRESULT {
    (*pCyOut).int64 = (cIn as i64) * CY_MULTIPLIER as i64;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromUI4(ulIn: ULONG, pCyOut: *mut CY) -> HRESULT {
    (*pCyOut).int64 = ((ulIn as u64) * CY_MULTIPLIER as u64) as i64;
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromUI2(usIn: USHORT, pCyOut: *mut CY) -> HRESULT {
    (*pCyOut).int64 = ((usIn as ULONG64) * CY_MULTIPLIER as ULONG64) as i64;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromDec(pdecIn: *const DECIMAL, pCyOut: *mut CY) -> HRESULT {
    let mut rounded: DECIMAL = core::mem::zeroed();
    let hret = VarDecRound(pdecIn, 4, &mut rounded as *mut DECIMAL);
    if hret >= 0 {
        if rounded.Hi32 != 0 {
            return DISP_E_OVERFLOW;
        }
        let signscale = rounded.__bindgen_anon_1.signscale;
        let scale = (signscale & 0x7F) as usize;
        let sign = (signscale & 0x80) != 0;
        let lo64 = rounded.__bindgen_anon_2.Lo64 as f64;
        let divisor = CY_Divisors[scale] as f64;
        let d = if sign { -lo64 / divisor } else { lo64 / divisor };
        return VarCyFromR8(d, pCyOut);
    }
    hret
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromI8(llIn: i64, pCyOut: *mut CY) -> HRESULT {
    const CY_MULTIPLIER: i64 = 10000;
    if llIn <= (i64::MIN / CY_MULTIPLIER) || llIn >= (i64::MAX / CY_MULTIPLIER) {
        return DISP_E_OVERFLOW;
    }
    (*pCyOut).int64 = llIn * CY_MULTIPLIER;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFromUI8(ullIn: u64, pCyOut: *mut CY) -> i32 {
    if ullIn > (i64::MAX as u64 / 10000u64) {
        return DISP_E_OVERFLOW; // DISP_E_OVERFLOW
    }
    (*pCyOut).int64 = (ullIn as i64) * 10000i64;
    0i32 // S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyAdd(cyLeft: CY, cyRight: CY, pCyOut: *mut CY) -> HRESULT {
    let mut l: f64 = 0.0;
    let mut r: f64 = 0.0;
    _VarR8FromCy(cyLeft, &mut l);
    _VarR8FromCy(cyRight, &mut r);
    VarCyFromR8(l + r, pCyOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarCyMul(cyLeft: CY, cyRight: CY, pCyOut: *mut CY) -> HRESULT {
    let mut l: f64 = 0.0;
    let mut r: f64 = 0.0;
    _VarR8FromCy(cyLeft, &mut l);
    _VarR8FromCy(cyRight, &mut r);
    l *= r;
    VarCyFromR8(l, pCyOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarCyMulI4(cyLeft: CY, lRight: i32, pCyOut: *mut CY) -> HRESULT {
    let mut d: f64 = 0.0;
    _VarR8FromCy(cyLeft, &mut d);
    d = d * lRight as f64;
    VarCyFromR8(d, pCyOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarCySub(cyLeft: CY, cyRight: CY, pCyOut: *mut CY) -> HRESULT {
    let mut l: f64 = 0.0;
    let mut r: f64 = 0.0;
    _VarR8FromCy(cyLeft, core::ptr::addr_of_mut!(l));
    _VarR8FromCy(cyRight, core::ptr::addr_of_mut!(r));
    l = l - r;
    VarCyFromR8(l, pCyOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarCyAbs(cyIn: CY, pCyOut: *mut CY) -> HRESULT {
    if cyIn.__bindgen_anon_1.Hi == 0x80000000u32 as i32 && cyIn.__bindgen_anon_1.Lo == 0u32 {
        return DISP_E_OVERFLOW;
    }
    (*pCyOut).int64 = if cyIn.int64 < 0 { -cyIn.int64 } else { cyIn.int64 };
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyFix(cyIn: CY, pCyOut: *mut CY) -> HRESULT {
    let cy_multiplier: i64 = 10000;
    (*pCyOut).int64 = (cyIn.int64 / cy_multiplier) * cy_multiplier;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyInt(cyIn: CY, pCyOut: *mut CY) -> HRESULT {
    const CY_MULTIPLIER: i64 = 10000;
    (*pCyOut).int64 = cyIn.int64 / CY_MULTIPLIER;
    (*pCyOut).int64 *= CY_MULTIPLIER;
    if cyIn.int64 < 0 && cyIn.int64 % CY_MULTIPLIER != 0 {
        (*pCyOut).int64 -= CY_MULTIPLIER;
    }
    0i32
}

#[no_mangle]
pub unsafe extern "system" fn VarCyNeg(cyIn: CY, pCyOut: *mut CY) -> HRESULT {
    if (cyIn.__bindgen_anon_1.Hi == 0x80000000u32 as i32) && (cyIn.__bindgen_anon_1.Lo == 0) {
        return -2147352566i32; // DISP_E_OVERFLOW
    }
    (*pCyOut).int64 = -cyIn.int64;
    0i32 // S_OK
}
#[no_mangle]
pub unsafe extern "system" fn VarCyRound(cyIn: CY, cDecimals: i32, pCyOut: *mut CY) -> i32 {
    if cDecimals < 0 {
        return 0x80070057u32 as i32; // E_INVALIDARG
    }

    if cDecimals > 3 {
        core::ptr::write(pCyOut, cyIn);
        return 0i32; // S_OK
    }

    let div = CY_Divisors[cDecimals as usize] as f64;
    let mut d: f64 = 0.0;
    _VarR8FromCy(cyIn, &mut d);
    d = d * div;

    // VARIANT_DutchRound(LONGLONG, d, pCyOut->int64) #1
    let whole1 = if d < 0.0 { ceil(d) } else { floor(d) };
    let fract1 = d - whole1;
    let rounded1: i64;
    if fract1 > 0.5 {
        rounded1 = whole1 as i64 + 1;
    } else if fract1 == 0.5 {
        let is_odd = (whole1 as i64) & 1;
        rounded1 = whole1 as i64 + is_odd;
    } else if fract1 >= 0.0 {
        rounded1 = whole1 as i64;
    } else if fract1 == -0.5 {
        let is_odd = (whole1 as i64) & 1;
        rounded1 = whole1 as i64 - is_odd;
    } else if fract1 > -0.5 {
        rounded1 = whole1 as i64;
    } else {
        rounded1 = whole1 as i64 - 1;
    }
    (*pCyOut).int64 = rounded1;

    d = (rounded1 as f64) / div * CY_MULTIPLIER_F;

    // VARIANT_DutchRound(LONGLONG, d, pCyOut->int64) #2
    let whole2 = if d < 0.0 { ceil(d) } else { floor(d) };
    let fract2 = d - whole2;
    let rounded2: i64;
    if fract2 > 0.5 {
        rounded2 = whole2 as i64 + 1;
    } else if fract2 == 0.5 {
        let is_odd = (whole2 as i64) & 1;
        rounded2 = whole2 as i64 + is_odd;
    } else if fract2 >= 0.0 {
        rounded2 = whole2 as i64;
    } else if fract2 == -0.5 {
        let is_odd = (whole2 as i64) & 1;
        rounded2 = whole2 as i64 - is_odd;
    } else if fract2 > -0.5 {
        rounded2 = whole2 as i64;
    } else {
        rounded2 = whole2 as i64 - 1;
    }
    (*pCyOut).int64 = rounded2;

    0i32 // S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarCyCmp(cyLeft: CY, cyRight: CY) -> HRESULT {
    let mut hRet: HRESULT;
    let mut result: CY = unsafe { core::mem::zeroed() };

    hRet = VarCySub(cyLeft, cyRight, &mut result);
    if hRet >= 0 {
        if result.int64 < 0 {
            hRet = 0; // VARCMP_LT
        } else if result.int64 > 0 {
            hRet = 2; // VARCMP_GT
        } else {
            hRet = 1; // VARCMP_EQ
        }
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarCyCmpR8(cyLeft: CY, dblRight: f64) -> HRESULT {
    let mut hRet: HRESULT = 0;
    let mut cyRight: CY = unsafe { core::mem::zeroed() };

    hRet = VarCyFromR8(dblRight, &mut cyRight);

    if SUCCEEDED(hRet) {
        hRet = VarCyCmp(cyLeft, cyRight);
    }

    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarCyMulI8(cyLeft: CY, llRight: LONG64, pCyOut: *mut CY) -> HRESULT {
    let mut d: f64 = 0.0;
    _VarR8FromCy(cyLeft, core::ptr::addr_of_mut!(d));
    d = d * (llRight as f64);
    VarCyFromR8(d, pCyOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromUI1(bIn: BYTE, pDecOut: *mut DECIMAL) -> HRESULT {
    VarDecFromUI4(bIn as ULONG, pDecOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromI2(sIn: i16, pDecOut: *mut DECIMAL) -> i32 {
    VarDecFromI4(sIn as i32, pDecOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromI4(lIn: i32, pDecOut: *mut DECIMAL) -> i32 {
    (*pDecOut).Hi32 = 0;
    (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = 0;
    if lIn < 0 {
        (*pDecOut).__bindgen_anon_1.signscale = (0x80u16 << 8) | (0u16 & 0xff);
        (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = lIn.wrapping_neg() as u32;
    } else {
        (*pDecOut).__bindgen_anon_1.signscale = (0u16 << 8) | (0u16 & 0xff);
        (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = lIn as u32;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromR4(fltIn: FLOAT, pDecOut: *mut DECIMAL) -> HRESULT {
    let mut di: VARIANT_DI = core::mem::zeroed();
    let hres = VARIANT_DI_FromR4(fltIn, &mut di);
    if hres == 0 {
        VARIANT_DecFromDI(&di, pDecOut);
    }
    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromR8(dblIn: f64, pDecOut: *mut DECIMAL) -> HRESULT {
    let mut di: VARIANT_DI = core::mem::zeroed();
    let hres: HRESULT = VARIANT_DI_FromR8(dblIn, core::ptr::addr_of_mut!(di));
    if hres == S_OK {
        VARIANT_DecFromDI(core::ptr::addr_of!(di), pDecOut);
    }
    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromDate(dateIn: f64, pDecOut: *mut DECIMAL) -> HRESULT {
    VarDecFromR8(dateIn, pDecOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarDecFromCy(cyIn: CY, pDecOut: *mut DECIMAL) -> HRESULT {
    (*pDecOut).Hi32 = 0;
    if cyIn.__bindgen_anon_1.Hi < 0 {
        (*pDecOut).__bindgen_anon_1.signscale = ((DECIMAL_NEG as u16) << 8) | 4u16;
        let int64_val: i64 = cyIn.int64;
        (*pDecOut).__bindgen_anon_2.Lo64 = (-int64_val) as u64;
    } else {
        (*pDecOut).__bindgen_anon_1.signscale = ((DECIMAL_POS as u16) << 8) | 4u16;
        (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = cyIn.__bindgen_anon_1.Hi as u32;
        (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = cyIn.__bindgen_anon_1.Lo;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromStr(
    strIn: *mut u16,
    lcid: LCID,
    dwFlags: u32,
    pDecOut: *mut DECIMAL,
) -> HRESULT {
    VARIANT_NumberFromBstr(strIn, lcid, dwFlags, pDecOut as *mut core::ffi::c_void, VT_DECIMAL)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pDecOut: *mut DECIMAL,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pDecOut as *mut core::ffi::c_void, VT_DECIMAL, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromBool(
    b_in: VARIANT_BOOL,
    p_dec_out: *mut DECIMAL,
) -> HRESULT {
    (*p_dec_out).Hi32 = 0;
    (*p_dec_out).__bindgen_anon_2.__bindgen_anon_1.Mid32 = 0;
    if b_in != 0i16 {
        (*p_dec_out).__bindgen_anon_1.signscale = 0x8000u16;
        (*p_dec_out).__bindgen_anon_2.__bindgen_anon_1.Lo32 = 1;
    } else {
        (*p_dec_out).__bindgen_anon_1.signscale = 0u16;
        (*p_dec_out).__bindgen_anon_2.__bindgen_anon_1.Lo32 = 0;
    }
    0i32
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromI1(cIn: i8, pDecOut: *mut DECIMAL) -> i32 {
    VarDecFromI4(cIn as i32, pDecOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromUI2(usIn: u16, pDecOut: *mut DECIMAL) -> HRESULT {
    VarDecFromUI4(usIn as u32, pDecOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromUI4(ulIn: ULONG, pDecOut: *mut DECIMAL) -> HRESULT {
    (*DEC_SIGNSCALE_ref(pDecOut)) = SIGNSCALE(0, 0);
    (*DEC_HI32_ref(pDecOut)) = 0;
    (*DEC_MID32_ref(pDecOut)) = 0;
    (*DEC_LO32_ref(pDecOut)) = ulIn;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromI8(llIn: i64, pDecOut: *mut DECIMAL) -> i32 {
    (*pDecOut).Hi32 = 0;

    if llIn < 0 {
        (*pDecOut).__bindgen_anon_1.signscale = (0x80u16 << 8) | 0u16;
        (*pDecOut).__bindgen_anon_2.Lo64 = (-llIn) as u64;
    } else {
        (*pDecOut).__bindgen_anon_1.signscale = (0u16 << 8) | 0u16;
        (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = (llIn >> 32) as u32;
        (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = llIn as u32;
    }

    0i32 // S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFromUI8(ullIn: ULONGLONG, pDecOut: *mut DECIMAL) -> HRESULT {
    (*pDecOut).__bindgen_anon_1.signscale = 0;
    (*pDecOut).Hi32 = 0;
    (*pDecOut).__bindgen_anon_2.Lo64 = ullIn;
    0
}

unsafe fn VARIANT_DecScale(
    ppDecLeft: *mut *const DECIMAL,
    ppDecRight: *mut *const DECIMAL,
    pDecOut: *mut DECIMAL,
) -> HRESULT {
    static mut scaleFactor: DECIMAL = DECIMAL {
        wReserved: 0,
        __bindgen_anon_1: tagDEC__bindgen_ty_1 { signscale: 0 },
        Hi32: 0,
        __bindgen_anon_2: tagDEC__bindgen_ty_2 { Lo64: 0 },
    };
    if core::ptr::read_volatile(&scaleFactor.__bindgen_anon_2.__bindgen_anon_1.Lo32) == 0 {
        scaleFactor.__bindgen_anon_2.__bindgen_anon_1.Lo32 = 10u32;
    }

    let dec_left = *ppDecLeft;
    let dec_right = *ppDecRight;
    if ((*dec_left).__bindgen_anon_1.__bindgen_anon_1.sign & 0x7F) != 0 || ((*dec_right).__bindgen_anon_1.__bindgen_anon_1.sign & 0x7F) != 0 {
        return E_INVALIDARG;
    }

    let scale_amount = ((*dec_left).__bindgen_anon_1.__bindgen_anon_1.scale as i32) - ((*dec_right).__bindgen_anon_1.__bindgen_anon_1.scale as i32);
    let mut i = scale_amount;

    if i == 0 {
        return S_OK;
    }

    let mut dec_temp: DECIMAL;
    if i > 0 {
        dec_temp = *dec_right;
        *ppDecRight = pDecOut; // &pDecOut[0]
    } else {
        dec_temp = *dec_left;
        *ppDecLeft = pDecOut;
        i = -i;
    }

    // Multiply up to match scales
    while i > 0 {
        let res = VarDecMul(&dec_temp, &scaleFactor, pDecOut);
        if res >= 0 {
            // SUCCEEDED
            dec_temp = *pDecOut;
            i -= 1;
        } else {
            break;
        }
    }

    if i == 0 {
        if scale_amount > 0 {
            (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale = ((*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale as i32 + scale_amount) as u8;
        } else {
            (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale = ((*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale as i32 + (-scale_amount)) as u8;
        }
        return S_OK;
    }

    // Further scaling not possible, reduce accuracy of other argument
    *pDecOut = dec_temp;
    if scale_amount > 0 {
        (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale = ((*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale as i32 + scale_amount - i) as u8;
        let mut di: VARIANT_DI = core::mem::zeroed();
        VARIANT_DIFromDec(*ppDecLeft, &mut di);
        *ppDecLeft = pDecOut.offset(1); // &pDecOut[1]
        di.scale = (di.scale as i32 - i) as u8;
        let mut remainder: u8 = 0;
        while i > 0 {
            i -= 1;
            if VARIANT_int_iszero(di.bitsnum.as_ptr(), 3) != 0 {
                break;
            }
            remainder = VARIANT_int_divbychar(di.bitsnum.as_mut_ptr(), 3, 10);
            // Dropping WARN log call
        }
        // Round up
        if remainder >= 5 {
            remainder = 1;
            let mut j = 0;
            while j < 3 && remainder != 0 {
                let digit = (di.bitsnum[j] as u64) + 1u64;
                remainder = if digit > 0xFFFFFFFF { 1 } else { 0 };
                di.bitsnum[j] = digit as u32;
                j += 1;
            }
        }
        VARIANT_DecFromDI(&di, pDecOut.offset(1));
    } else {
        (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale = ((*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale as i32 + (-scale_amount) - i) as u8;
        let mut di: VARIANT_DI = core::mem::zeroed();
        VARIANT_DIFromDec(*ppDecRight, &mut di);
        *ppDecRight = pDecOut.offset(1);
        di.scale = (di.scale as i32 - i) as u8;
        let mut remainder: u8 = 0;
        while i > 0 {
            i -= 1;
            if VARIANT_int_iszero(di.bitsnum.as_ptr(), 3) != 0 {
                break;
            }
            remainder = VARIANT_int_divbychar(di.bitsnum.as_mut_ptr(), 3, 10);
        }
        if remainder >= 5 {
            remainder = 1;
            let mut j = 0;
            while j < 3 && remainder != 0 {
                let digit = (di.bitsnum[j] as u64) + 1u64;
                remainder = if digit > 0xFFFFFFFF { 1 } else { 0 };
                di.bitsnum[j] = digit as u32;
                j += 1;
            }
        }
        VARIANT_DecFromDI(&di, pDecOut.offset(1));
    }

    S_OK
}

unsafe fn VARIANT_Add(ulLeft: ULONG, ulRight: ULONG, pulHigh: *mut ULONG) -> ULONG {
    let sum = (ulLeft as u64) + (ulRight as u64) + (*pulHigh as u64);
    *pulHigh = (sum >> 32) as ULONG;
    sum as ULONG
}

unsafe fn VARIANT_Sub(ulLeft: ULONG, ulRight: ULONG, pulHigh: *mut ULONG) -> ULONG {
    let left = ulLeft as u64;
    let right = ulRight as u64;
    let mut quad = left.wrapping_sub(right);
    let mut invert: u8 = if (ulLeft as u32) < (ulRight as u32) { 1 } else { 0 };
    let high_in = (*pulHigh) as u64;
    if quad > high_in {
        quad -= high_in;
    } else {
        quad = quad.wrapping_sub(high_in);
        invert = 1;
    }
    let mut high_part = (quad >> 32) as u32;
    if invert != 0 {
        high_part = high_part.wrapping_neg();
    }
    *pulHigh = high_part;
    (quad & 0xFFFFFFFF) as u32
}

unsafe fn VARIANT_Mul(ulLeft: u32, ulRight: u32, pulHigh: *mut u32) -> u32 {
    let high = *pulHigh;
    let result = (ulLeft as u64) * (ulRight as u64) + (high as u64);
    *pulHigh = (result >> 32) as u32;
    result as u32
}

unsafe fn VARIANT_DecCmp(pDecLeft: *const DECIMAL, pDecRight: *const DECIMAL) -> i32 {
    let left_hi = *DEC_HI32_ref(pDecLeft as *mut DECIMAL) ;
    let right_hi = *DEC_HI32_ref(pDecRight as *mut DECIMAL) ;
    let left_lo = *DEC_LO64_ref(pDecLeft as *mut DECIMAL) ;
    let right_lo = *DEC_LO64_ref(pDecRight as *mut DECIMAL) ;
    if left_hi < right_hi || (left_hi <= right_hi && left_lo < right_lo) {
        -1
    } else if left_hi == right_hi && left_lo == right_lo {
        0
    } else {
        1
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarDecAdd(
    pDecLeft: *const DECIMAL,
    pDecRight: *const DECIMAL,
    pDecOut: *mut DECIMAL,
) -> HRESULT {
    let mut scaled: [DECIMAL; 2] = [core::mem::zeroed(), core::mem::zeroed()];
    let mut pLeft: *const DECIMAL = pDecLeft;
    let mut pRight: *const DECIMAL = pDecRight;
    let hRet = VARIANT_DecScale(&mut pLeft, &mut pRight, scaled.as_mut_ptr());

    if hRet < 0 {
        return hRet;
    }

    'calc: loop {
        let mut overflow: ULONG = 0;
        let mut sign: u8 = 0; /* DECIMAL_POS */
        let left_sign_neg = (*pLeft).__bindgen_anon_1.__bindgen_anon_1.sign != 0;
        let right_sign_neg = (*pRight).__bindgen_anon_1.__bindgen_anon_1.sign != 0;

        if left_sign_neg && right_sign_neg {
            sign = 0x80; /* DECIMAL_NEG */
            /* Fall through to addition */
            (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = VARIANT_Add((*pLeft).__bindgen_anon_2.__bindgen_anon_1.Lo32, (*pRight).__bindgen_anon_2.__bindgen_anon_1.Lo32, &mut overflow);
            (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = VARIANT_Add((*pLeft).__bindgen_anon_2.__bindgen_anon_1.Mid32, (*pRight).__bindgen_anon_2.__bindgen_anon_1.Mid32, &mut overflow);
            (*pDecOut).Hi32 = VARIANT_Add((*pLeft).Hi32, (*pRight).Hi32, &mut overflow);
        } else if left_sign_neg && !right_sign_neg {
            let cmp = VARIANT_DecCmp(pLeft, pRight);
            if cmp > 0 {
                sign = 0x80;
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = VARIANT_Sub((*pLeft).__bindgen_anon_2.__bindgen_anon_1.Lo32, (*pRight).__bindgen_anon_2.__bindgen_anon_1.Lo32, &mut overflow);
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = VARIANT_Sub((*pLeft).__bindgen_anon_2.__bindgen_anon_1.Mid32, (*pRight).__bindgen_anon_2.__bindgen_anon_1.Mid32, &mut overflow);
                (*pDecOut).Hi32 = VARIANT_Sub((*pLeft).Hi32, (*pRight).Hi32, &mut overflow);
            } else {
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = VARIANT_Sub((*pRight).__bindgen_anon_2.__bindgen_anon_1.Lo32, (*pLeft).__bindgen_anon_2.__bindgen_anon_1.Lo32, &mut overflow);
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = VARIANT_Sub((*pRight).__bindgen_anon_2.__bindgen_anon_1.Mid32, (*pLeft).__bindgen_anon_2.__bindgen_anon_1.Mid32, &mut overflow);
                (*pDecOut).Hi32 = VARIANT_Sub((*pRight).Hi32, (*pLeft).Hi32, &mut overflow);
            }
        } else if !left_sign_neg && right_sign_neg {
            let cmp = VARIANT_DecCmp(pLeft, pRight);
            if cmp <= 0 {
                sign = 0x80;
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = VARIANT_Sub((*pRight).__bindgen_anon_2.__bindgen_anon_1.Lo32, (*pLeft).__bindgen_anon_2.__bindgen_anon_1.Lo32, &mut overflow);
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = VARIANT_Sub((*pRight).__bindgen_anon_2.__bindgen_anon_1.Mid32, (*pLeft).__bindgen_anon_2.__bindgen_anon_1.Mid32, &mut overflow);
                (*pDecOut).Hi32 = VARIANT_Sub((*pRight).Hi32, (*pLeft).Hi32, &mut overflow);
            } else {
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = VARIANT_Sub((*pLeft).__bindgen_anon_2.__bindgen_anon_1.Lo32, (*pRight).__bindgen_anon_2.__bindgen_anon_1.Lo32, &mut overflow);
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = VARIANT_Sub((*pLeft).__bindgen_anon_2.__bindgen_anon_1.Mid32, (*pRight).__bindgen_anon_2.__bindgen_anon_1.Mid32, &mut overflow);
                (*pDecOut).Hi32 = VARIANT_Sub((*pLeft).Hi32, (*pRight).Hi32, &mut overflow);
            }
        } else {
            /* Both positive */
            (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = VARIANT_Add((*pLeft).__bindgen_anon_2.__bindgen_anon_1.Lo32, (*pRight).__bindgen_anon_2.__bindgen_anon_1.Lo32, &mut overflow);
            (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = VARIANT_Add((*pLeft).__bindgen_anon_2.__bindgen_anon_1.Mid32, (*pRight).__bindgen_anon_2.__bindgen_anon_1.Mid32, &mut overflow);
            (*pDecOut).Hi32 = VARIANT_Add((*pLeft).Hi32, (*pRight).Hi32, &mut overflow);
        }

        if overflow != 0 {
            // Wine 10.0: a SUM (equal signs) that overflows 96 bits drops one
            // decimal digit of scale, rounding half up, instead of failing;
            // only a scale-0 sum has nowhere to go. (Differences cannot carry.)
            if left_sign_neg == right_sign_neg {
                let scale = (*pLeft).__bindgen_anon_1.__bindgen_anon_1.scale;
                if scale == 0 {
                    break 'calc -2147352566i32; /* DISP_E_OVERFLOW */
                }
                let mut n: [u32; 4] = [
                    (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32,
                    (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32,
                    (*pDecOut).Hi32,
                    overflow,
                ];
                let remainder = VARIANT_int_divbychar(n.as_mut_ptr(), 4, 10);
                if remainder >= 5 {
                    let mut carry = true;
                    for digit in n.iter_mut() {
                        if !carry {
                            break;
                        }
                        let (v, c) = digit.overflowing_add(1);
                        *digit = v;
                        carry = c;
                    }
                }
                (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale = scale - 1;
                (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.sign = sign;
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Lo32 = n[0];
                (*pDecOut).__bindgen_anon_2.__bindgen_anon_1.Mid32 = n[1];
                (*pDecOut).Hi32 = n[2];
                break 'calc 0i32; /* S_OK */
            }
            break 'calc -2147352566i32; /* DISP_E_OVERFLOW */
        }

        (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale = (*pLeft).__bindgen_anon_1.__bindgen_anon_1.scale;
        (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.sign = if sign == 0x80 { 0x80u8 } else { 0u8 };
        break 'calc 0i32; /* S_OK */
    }
}

unsafe fn VARIANT_DIFromDec(from: *const DECIMAL, to: *mut VARIANT_DI) {
    (*to).scale = DEC_SCALE(from as *mut DECIMAL);
    (*to).sign = if DEC_SIGN(from as *mut DECIMAL) != 0 { 1 } else { 0 };
    (*to).bitsnum[0] = *DEC_LO32_ref(from as *mut DECIMAL) ;
    (*to).bitsnum[1] = *DEC_MID32_ref(from as *mut DECIMAL) ;
    (*to).bitsnum[2] = *DEC_HI32_ref(from as *mut DECIMAL) ;
}

unsafe fn VARIANT_DecFromDI(from: *const VARIANT_DI, to: *mut DECIMAL) {
    let sign_byte: u8 = if (*from).sign != 0 { 0x80 } else { 0 };
    let scale = (*from).scale;
    (*to).__bindgen_anon_1.signscale = ((scale as u16) << 8) | sign_byte as u16;
    (*to).__bindgen_anon_2.__bindgen_anon_1.Lo32 = (*from).bitsnum[0];
    (*to).__bindgen_anon_2.__bindgen_anon_1.Mid32 = (*from).bitsnum[1];
    (*to).Hi32 = (*from).bitsnum[2];
}

unsafe fn VARIANT_DI_clear(i: *mut VARIANT_DI) {
    core::ptr::write_bytes(i, 0, 1);
}

unsafe fn VARIANT_int_divbychar(p: *mut u32, n: u32, divisor: u8) -> u8 {
    if divisor == 0 {
        return 0xFF;
    }
    if divisor == 1 {
        return 0;
    }

    let mut remainder: u8 = 0;
    let mut i: i32 = (n as i32) - 1;

    // skip leading zero DWORDs
    while i >= 0 && *p.add(i as usize) == 0 {
        i -= 1;
    }

    // process remaining DWORDs from most significant to least
    while i >= 0 {
        let dividend: u64 = ((remainder as u64) << 32) | (*p.add(i as usize) as u64);
        remainder = (dividend % (divisor as u64)) as u8;
        *p.add(i as usize) = (dividend / (divisor as u64)) as u32;
        i -= 1;
    }

    remainder
}

unsafe fn VARIANT_int_iszero(p: *const u32, mut n: u32) -> i32 {
    let mut ptr = p;
    while n > 0 {
        if *ptr != 0 {
            return 0;
        }
        ptr = ptr.add(1);
        n -= 1;
    }
    1
}

unsafe fn VARIANT_DI_mul(a: *const VARIANT_DI, b: *const VARIANT_DI, result: *mut VARIANT_DI) -> i32 {
    let mut r_overflow: i32 = 0;
    let mut running: [u32; 6] = [0; 6];
    let mut mulstart: i32 = -1;

    VARIANT_DI_clear(result);
    (*result).sign = if ((*a).sign ^ (*b).sign) != 0 { 1u32 } else { 0u32 };
    (*result).scale = (*a).scale + (*b).scale;
    core::ptr::write_bytes(running.as_mut_ptr() as *mut u8, 0, core::mem::size_of::<[u32; 6]>());

    for i in (0..3).rev() {
        if (*a).bitsnum[i as usize] != 0 {
            mulstart = i;
            break;
        }
    }
    if mulstart < 0 {
        (*result).scale = 0;
        (*result).sign = 0;
    } else {
        let mut remainder: u8 = 0;
        for iA in 0..=mulstart as usize {
            let mut iOverflowMul: u32 = 0;
            for iB in 0..3 {
                let iRV = VARIANT_Mul((*b).bitsnum[iB], (*a).bitsnum[iA], &mut iOverflowMul);
                let mut rv = iRV;
                let mut idx = iA + iB;
                loop {
                    running[idx] = VARIANT_Add(running[idx], 0, &mut rv);
                    idx += 1;
                    if rv == 0 { break; }
                }
            }
        }

        while (*result).scale > 0 && VARIANT_int_iszero(running.as_ptr().add(3), 3) == 0 {
            remainder = VARIANT_int_divbychar(running.as_mut_ptr(), 6, 10);
            if remainder > 0 {
                WARN!(b"losing significant digits (remainder %u)...\n\0".as_ptr() as *const i8, remainder);
            }
            (*result).scale -= 1;
        }

        if remainder >= 5 {
            let mut carry: u8 = 1;
            for i in 0..6 {
                if carry == 0 { break; }
                let digit = running[i] as u64 + 1;
                carry = if digit > 0xFFFFFFFF { 1 } else { 0 };
                running[i] = (digit & 0xFFFFFFFF) as u32;
            }
        }

        r_overflow = if VARIANT_int_iszero(running.as_ptr().add(3), 3) == 0 { 1 } else { 0 };
        core::ptr::copy_nonoverlapping(running.as_ptr(), (*result).bitsnum.as_mut_ptr(), 3);
    }
    r_overflow
}

unsafe fn VARIANT_DI_tostringW(a: *const VARIANT_DI, s: *mut u16, n: u32) -> i32 {
    let mut overflow = false;
    let mut quotient: [u32; 3] = [0; 3];
    let mut remainder: u8;
    let mut i: u32;
    let mut n = n;
    let mut s = s;

    /* place negative sign */
    if VARIANT_int_iszero((*a).bitsnum.as_ptr() as *const u32, 3) == 0 && (*a).sign != 0 {
        if n > 0 {
            *s = b'-' as u16;
            s = s.offset(1);
            n -= 1;
        } else {
            overflow = true;
        }
    }

    /* prepare initial 0 */
    if !overflow {
        if n >= 2 {
            *s = b'0' as u16;
            *s.offset(1) = 0u16;
        } else {
            overflow = true;
        }
    }

    i = 0;
    core::ptr::copy_nonoverlapping((*a).bitsnum.as_ptr(), quotient.as_mut_ptr(), 3);
    while !overflow && VARIANT_int_iszero(quotient.as_ptr(), 3) == 0 {
        remainder = VARIANT_int_divbychar(quotient.as_mut_ptr(), 3, 10);
        if i + 2 > n {
            overflow = true;
        } else {
            *s.offset(i as isize) = b'0' as u16 + remainder as u16;
            *s.offset(i as isize + 1) = 0u16;
            i += 1;
        }
    }

    if !overflow && VARIANT_int_iszero((*a).bitsnum.as_ptr() as *const u32, 3) == 0 {
        /* reverse order of digits */
        let mut x = s;
        let mut y = s.offset((i - 1) as isize);
        while x < y {
            let tmp = *x;
            *x = *y;
            *y = tmp;
            x = x.offset(1);
            y = y.offset(-1);
        }

        let scale = (*a).scale as u32;
        /* check for decimal point. "i" now has string length */
        if i <= scale {
            let numzeroes = scale + 1 - i;
            if i + 1 + numzeroes >= n {
                overflow = true;
            } else {
                core::ptr::copy(s, s.offset(numzeroes as isize), (i + 1) as usize);
                i += numzeroes;
                for j in 0..numzeroes {
                    *s.offset(j as isize) = b'0' as u16;
                }
            }
        }

        /* place decimal point */
        if scale > 0 {
            let periodpos = i - scale;
            if i + 2 >= n {
                overflow = true;
            } else {
                core::ptr::copy(s.offset(periodpos as isize), s.offset(periodpos as isize + 1), (i + 1 - periodpos) as usize);
                *s.offset(periodpos as isize) = b'.' as u16;
                i += 1;

                /* remove extra zeros at the end, if any */
                while i > 0 && *s.offset(i as isize - 1) == b'0' as u16 {
                    i -= 1;
                    *s.offset(i as isize) = 0u16;
                }
                if i > 0 && *s.offset(i as isize - 1) == b'.' as u16 {
                    i -= 1;
                    *s.offset(i as isize) = 0u16;
                }
            }
        }
    }

    if !overflow { 1 } else { 0 }
}

unsafe fn VARIANT_int_shiftleft(p: *mut u32, n: u32, shift: u32) {
    let n_usize = n as usize;
    let mut shift = shift;

    /* shift whole DWORDs to the left */
    while shift >= 32 {
        if n_usize > 1 {
            core::ptr::copy(p, p.add(1), n_usize - 1);
        }
        *p = 0;
        shift -= 32;
    }

    /* shift remainder (1..31 bits) */
    let mut shifted: u32 = 0;
    if shift > 0 {
        for i in 0..n_usize {
            let b = (*p.add(i)) >> (32 - shift);
            *p.add(i) = ((*p.add(i)) << shift) | shifted;
            shifted = b;
        }
    }
}

unsafe fn VARIANT_int_add(v: *mut u32, nv: u32, p: *const u32, np: u32) -> u8 {
    let mut carry: u8 = 0;
    if nv >= np {
        let mut sum: u64;
        let mut i = 0usize;
        while i < np as usize {
            sum = (*v.add(i) as u64) + (*p.add(i) as u64) + (carry as u64);
            *v.add(i) = (sum & 0xffffffff) as u32;
            carry = (sum >> 32) as u8;
            i += 1;
        }
        while i < nv as usize && carry != 0 {
            sum = (*v.add(i) as u64) + (carry as u64);
            *v.add(i) = (sum & 0xffffffff) as u32;
            carry = (sum >> 32) as u8;
            i += 1;
        }
    }
    carry
}

unsafe fn VARIANT_int_div(p: *mut u32, n: u32, divisor: *const u32, dn: u32) {
    let mut tempsub: [u32; 8] = [0u32; 8];
    let negdivisor = tempsub.as_mut_ptr().add(n as usize);

    let mut i: u32 = 0;
    while i < n {
        *negdivisor.add(i as usize) = if i < dn {
            !*divisor.add(i as usize)
        } else {
            0xFFFFFFFF
        };
        i += 1;
    }
    *p.add(n as usize) = 1;
    VARIANT_int_add(negdivisor, n, p.add(n as usize), 1);
    core::ptr::write_bytes(p.add(n as usize), 0, n as usize);

    i = 0;
    while i < n && *p.add((n as usize) - 1) == 0 {
        VARIANT_int_shiftleft(p, n, 32);
        i += 1;
    }
    i <<= 5;
    while i < (n << 5) {
        VARIANT_int_shiftleft(p, n << 1, 1);
        core::ptr::copy_nonoverlapping(p.add(n as usize), tempsub.as_mut_ptr(), n as usize);
        VARIANT_int_add(tempsub.as_mut_ptr(), n, negdivisor, n);
        if (tempsub[(n as usize) - 1] & 0x80000000) == 0 {
            core::ptr::copy_nonoverlapping(tempsub.as_ptr(), p.add(n as usize), n as usize);
            *p |= 1;
        }
        i += 1;
    }
}

unsafe fn VARIANT_int_mulbychar(p: *mut u32, n: u32, m: u8) -> u8 {
    let mut i_overflow_mul: u32 = 0;
    for i in 0..n {
        let val = *p.add(i as usize);
        *p.add(i as usize) = VARIANT_Mul(val, m as u32, &mut i_overflow_mul as *mut u32);
    }
    i_overflow_mul as u8
}

unsafe fn VARIANT_int_addlossy(
    a: *mut u32,
    ascale: *mut i32,
    an: u32,
    b: *mut u32,
    bscale: *mut i32,
    bn: u32,
) -> i32 {
    let mut underflow: i32 = 0;

    if VARIANT_int_iszero(a, an) != 0 {
        let bn_local = bn;
        let an_local = an;
        while bn_local > an_local && VARIANT_int_iszero(b.add(an_local as usize), bn_local - an_local) == 0 {
            VARIANT_int_divbychar(b, bn_local, 10);
            *bscale -= 1;
        }
        core::ptr::copy_nonoverlapping(b, a, an_local as usize);
        *ascale = *bscale;
    } else if VARIANT_int_iszero(b, bn) == 0 {
        let mut tn: u32 = an + 1;
        if bn + 1 > tn {
            tn = bn + 1;
        }
        let mut t: [u32; 5] = [0u32; 5];

        if *ascale != *bscale {
            let digitchosen: *mut u32;
            let nchosen: u32;
            let scalechosen: *mut i32;
            let targetscale: i32;

            if *ascale < *bscale {
                targetscale = *ascale;
                scalechosen = bscale;
                digitchosen = b;
                nchosen = bn;
            } else {
                targetscale = *bscale;
                scalechosen = ascale;
                digitchosen = a;
                nchosen = an;
            }
            core::ptr::write_bytes(t.as_mut_ptr() as *mut u32, 0, tn as usize);
            core::ptr::copy_nonoverlapping(digitchosen, t.as_mut_ptr(), nchosen as usize);

            while *scalechosen > targetscale {
                let remainder = VARIANT_int_divbychar(t.as_mut_ptr(), tn, 10);
                if remainder == 0 {
                    *scalechosen -= 1;
                    core::ptr::copy_nonoverlapping(t.as_ptr(), digitchosen, nchosen as usize);
                } else {
                    break;
                }
            }
        }

        if *ascale != *bscale {
            let digitchosen: *mut u32;
            let nchosen: u32;
            let scalechosen: *mut i32;
            let targetscale: i32;

            if *ascale > *bscale {
                targetscale = *ascale;
                scalechosen = bscale;
                digitchosen = b;
                nchosen = bn;
            } else {
                targetscale = *bscale;
                scalechosen = ascale;
                digitchosen = a;
                nchosen = an;
            }
            core::ptr::write_bytes(t.as_mut_ptr() as *mut u32, 0, tn as usize);
            core::ptr::copy_nonoverlapping(digitchosen, t.as_mut_ptr(), nchosen as usize);

            while *scalechosen < targetscale && *t.as_ptr().add(nchosen as usize) == 0 {
                VARIANT_int_mulbychar(t.as_mut_ptr(), tn, 10);
                if *t.as_ptr().add(nchosen as usize) == 0 {
                    *scalechosen += 1;
                    core::ptr::copy_nonoverlapping(t.as_ptr(), digitchosen, nchosen as usize);
                }
            }
        }

        if *ascale != *bscale {
            let digitchosen: *mut u32;
            let nchosen: u32;
            let scalechosen: *mut i32;
            let targetscale: i32;

            if *ascale < *bscale {
                targetscale = *ascale;
                scalechosen = bscale;
                digitchosen = b;
                nchosen = bn;
            } else {
                targetscale = *bscale;
                scalechosen = ascale;
                digitchosen = a;
                nchosen = an;
            }
            core::ptr::write_bytes(t.as_mut_ptr() as *mut u32, 0, tn as usize);
            core::ptr::copy_nonoverlapping(digitchosen, t.as_mut_ptr(), nchosen as usize);

            while *scalechosen > targetscale {
                VARIANT_int_divbychar(t.as_mut_ptr(), tn, 10);
                *scalechosen -= 1;
                core::ptr::copy_nonoverlapping(t.as_ptr(), digitchosen, nchosen as usize);
            }
        }

        if VARIANT_int_iszero(a, an) != 0 || VARIANT_int_iszero(b, bn) != 0 {
            underflow = 1;
        } else {
            loop {
                if underflow != 0 {
                    break;
                }
                core::ptr::write_bytes(t.as_mut_ptr() as *mut u32, 0, tn as usize);
                core::ptr::copy_nonoverlapping(a, t.as_mut_ptr(), an as usize);

                VARIANT_int_add(t.as_mut_ptr(), tn, b, bn);
                if VARIANT_int_iszero(t.as_ptr().add(an as usize), tn - an) != 0 {
                    core::ptr::copy_nonoverlapping(t.as_ptr(), a, an as usize);
                    break;
                } else {
                    VARIANT_int_divbychar(a, an, 10);
                    *ascale -= 1;
                    VARIANT_int_divbychar(b, bn, 10);
                    *bscale -= 1;
                    underflow = if VARIANT_int_iszero(a, an) != 0 || VARIANT_int_iszero(b, bn) != 0 { 1 } else { 0 };
                }
            }
        }
    }
    underflow
}

unsafe fn VARIANT_DI_div(
    dividend: *const VARIANT_DI,
    divisor: *const VARIANT_DI,
    quotient: *mut VARIANT_DI,
    round_remainder: u8,
) -> i32 {
    let mut r_overflow: i32 = 0; // S_OK
    if VARIANT_int_iszero((*divisor).bitsnum.as_ptr(), 3) != 0 {
        r_overflow = 0x80020012_u32 as i32; // DISP_E_DIVBYZERO
    } else if VARIANT_int_iszero((*dividend).bitsnum.as_ptr(), 3) != 0 {
        VARIANT_DI_clear(quotient);
    } else {
        let mut quotientscale: i32;
        let mut remainderscale: i32;
        let mut tempquotientscale: i32;
        let mut remainderplusquotient: [u32; 8] = [0; 8];
        let mut underflow: i32;

        quotientscale = (*dividend).scale as i32 - (*divisor).scale as i32;
        remainderscale = quotientscale;
        tempquotientscale = quotientscale;
        VARIANT_DI_clear(quotient);
        (*quotient).sign = if ((*dividend).sign ^ (*divisor).sign) != 0 { 1 } else { 0 };

        core::ptr::write_bytes(
            remainderplusquotient.as_mut_ptr() as *mut u8,
            0,
            core::mem::size_of::<[u32; 8]>(),
        );
        core::ptr::copy_nonoverlapping(
            (*dividend).bitsnum.as_ptr(),
            remainderplusquotient.as_mut_ptr(),
            3,
        );

        loop {
            VARIANT_int_div(
                remainderplusquotient.as_mut_ptr(),
                4,
                (*divisor).bitsnum.as_ptr(),
                3,
            );
            underflow = VARIANT_int_addlossy(
                (*quotient).bitsnum.as_mut_ptr(),
                &mut quotientscale,
                3,
                remainderplusquotient.as_mut_ptr(),
                &mut tempquotientscale,
                4,
            );
            if round_remainder != 0 {
                if remainderplusquotient[4] >= 5 {
                    let mut i: u32 = 0;
                    let mut remainder: u8 = 1;
                    while i < 3 && remainder != 0 {
                        let digit = (*quotient).bitsnum[i as usize] as u64 + 1;
                        remainder = if digit > 0xFFFF_FFFF { 1 } else { 0 };
                        (*quotient).bitsnum[i as usize] = (digit & 0xFFFF_FFFF) as u32;
                        i += 1;
                    }
                }
                core::ptr::write_bytes(
                    remainderplusquotient.as_mut_ptr() as *mut u8,
                    0,
                    core::mem::size_of::<[u32; 8]>(),
                );
            } else {
                VARIANT_int_mulbychar(remainderplusquotient.as_mut_ptr().add(4), 4, 10);
                core::ptr::copy_nonoverlapping(
                    remainderplusquotient.as_ptr().add(4),
                    remainderplusquotient.as_mut_ptr(),
                    4,
                );
            }
            remainderscale += 1;
            tempquotientscale = remainderscale;
            if underflow != 0 || VARIANT_int_iszero(remainderplusquotient.as_ptr().add(4), 4) != 0 {
                break;
            }
        }

        while r_overflow == 0 && quotientscale < 0 {
            core::ptr::write_bytes(
                remainderplusquotient.as_mut_ptr() as *mut u8,
                0,
                core::mem::size_of::<[u32; 8]>(),
            );
            core::ptr::copy_nonoverlapping(
                (*quotient).bitsnum.as_ptr(),
                remainderplusquotient.as_mut_ptr(),
                3,
            );
            VARIANT_int_mulbychar(remainderplusquotient.as_mut_ptr(), 8, 10);
            if VARIANT_int_iszero(remainderplusquotient.as_ptr().add(3), 5) != 0 {
                quotientscale += 1;
                core::ptr::copy_nonoverlapping(
                    remainderplusquotient.as_ptr(),
                    (*quotient).bitsnum.as_mut_ptr(),
                    3,
                );
            } else {
                r_overflow = 0x8002000A_u32 as i32; // DISP_E_OVERFLOW
            }
        }
        if r_overflow == 0 {
            if quotientscale <= 255 {
                (*quotient).scale = quotientscale as u8;
            } else {
                VARIANT_DI_clear(quotient);
            }
        }
    }
    r_overflow
}

/* This procedure receives a VARIANT_DI with a defined mantissa and sign, but
   with an undefined scale, which will be assigned to (if possible). It also
   receives an exponent of 2. This procedure will then manipulate the mantissa
   and calculate a corresponding scale, so that the exponent2 value is assimilated
   into the VARIANT_DI and is therefore no longer necessary. Returns S_OK if
   successful, or DISP_E_OVERFLOW if the represented value is too big to fit into
   a DECIMAL. */
unsafe fn VARIANT_DI_normalize(val: *mut VARIANT_DI, exponent2: i32, isDouble: u8) -> HRESULT {
    let mut hres: HRESULT = S_OK;

    /* A factor of 2^exponent2 is equivalent to (10^exponent2)/(5^exponent2), and
       thus equal to (5^-exponent2)*(10^exponent2). */
    let mut exponent5: i32 = -exponent2;
    let mut exponent10: i32 = exponent2;

    /* Handle exponent5 > 0 */
    while exponent5 > 0 {
        let bPrevCarryBit: u32;
        let bCurrCarryBit: u32;

        if ((*val).bitsnum[0] & 1) == 0 {
            /* The mantissa is divisible by 2. */
            exponent10 += 1;
            exponent5 -= 1;

            /* Shift right */
            bPrevCarryBit = (*val).bitsnum[2] & 1;
            (*val).bitsnum[2] >>= 1;
            bCurrCarryBit = (*val).bitsnum[1] & 1;
            (*val).bitsnum[1] = ((*val).bitsnum[1] >> 1) | (if bPrevCarryBit != 0 { 0x80000000 } else { 0 });
            (*val).bitsnum[0] = ((*val).bitsnum[0] >> 1) | (if bCurrCarryBit != 0 { 0x80000000 } else { 0 });
        } else {
            /* The mantissa is NOT divisible by 2. Multiply by 5 unless it overflows. */
            let mut temp_bitsnum: [u32; 3] = [0; 3];

            exponent5 -= 1;

            core::ptr::copy_nonoverlapping((*val).bitsnum.as_ptr(), temp_bitsnum.as_mut_ptr(), 3);
            if 0 == VARIANT_int_mulbychar(temp_bitsnum.as_mut_ptr(), 3, 5) {
                core::ptr::copy_nonoverlapping(temp_bitsnum.as_ptr(), (*val).bitsnum.as_mut_ptr(), 3);
            } else {
                /* Multiplication by 5 overflows. Divide by 2 instead. */
                exponent10 += 1;

                bPrevCarryBit = (*val).bitsnum[2] & 1;
                (*val).bitsnum[2] >>= 1;
                bCurrCarryBit = (*val).bitsnum[1] & 1;
                (*val).bitsnum[1] = ((*val).bitsnum[1] >> 1) | (if bPrevCarryBit != 0 { 0x80000000 } else { 0 });
                (*val).bitsnum[0] = ((*val).bitsnum[0] >> 1) | (if bCurrCarryBit != 0 { 0x80000000 } else { 0 });
            }
        }
    }

    /* Handle exponent5 < 0 */
    while exponent5 < 0 {
        if ((*val).bitsnum[2] & 0x80000000) == 0 {
            /* The mantissa can withstand a shift-left without overflowing */
            exponent10 -= 1;
            exponent5 += 1;
            VARIANT_int_shiftleft((*val).bitsnum.as_mut_ptr(), 3, 1);
        } else {
            /* Directly divide by 5. */
            exponent5 += 1;
            VARIANT_int_divbychar((*val).bitsnum.as_mut_ptr(), 3, 5);
        }
    }

    /* The exponent10 must be in the range [-DEC_MAX_SCALE..0]. */
    while hres == S_OK && exponent10 > 0 {
        if 0 == VARIANT_int_mulbychar((*val).bitsnum.as_mut_ptr(), 3, 10) {
            exponent10 -= 1;
        } else {
            hres = DISP_E_OVERFLOW;
        }
    }
    while exponent10 < -(DEC_MAX_SCALE as i32) {
        let rem10 = VARIANT_int_divbychar((*val).bitsnum.as_mut_ptr(), 3, 10);
        exponent10 += 1;
        if VARIANT_int_iszero((*val).bitsnum.as_ptr(), 3) != 0 {
            /* Underflow, unable to keep dividing */
            exponent10 = 0;
        } else if rem10 >= 5 {
            let x: u32 = 1;
            VARIANT_int_add((*val).bitsnum.as_mut_ptr(), 3, &x as *const u32, 1);
        }
    }
    /* Remove excess bits of precision down to the precision guaranteed by the FP number. */
    if isDouble != 0 {
        while exponent10 < 0 && ((*val).bitsnum[2] != 0 || ((*val).bitsnum[1] & 0xFFE00000) != 0) {
            let rem10 = VARIANT_int_divbychar((*val).bitsnum.as_mut_ptr(), 3, 10);
            exponent10 += 1;
            if rem10 >= 5 {
                let x: u32 = 1;
                VARIANT_int_add((*val).bitsnum.as_mut_ptr(), 3, &x as *const u32, 1);
            }
        }
    } else {
        while exponent10 < 0
            && ((*val).bitsnum[2] != 0
                || (*val).bitsnum[1] != 0
                || ((*val).bitsnum[2] == 0
                    && (*val).bitsnum[1] == 0
                    && ((*val).bitsnum[0] & 0xFF000000) != 0))
        {
            let rem10 = VARIANT_int_divbychar((*val).bitsnum.as_mut_ptr(), 3, 10);
            exponent10 += 1;
            if rem10 >= 5 {
                let x: u32 = 1;
                VARIANT_int_add((*val).bitsnum.as_mut_ptr(), 3, &x as *const u32, 1);
            }
        }
    }
    /* Remove multiples of 10 from the representation */
    while exponent10 < 0 {
        let mut temp_bitsnum: [u32; 3] = [0; 3];

        core::ptr::copy_nonoverlapping((*val).bitsnum.as_ptr(), temp_bitsnum.as_mut_ptr(), 3);
        if 0 == VARIANT_int_divbychar(temp_bitsnum.as_mut_ptr(), 3, 10) {
            exponent10 += 1;
            core::ptr::copy_nonoverlapping(temp_bitsnum.as_ptr(), (*val).bitsnum.as_mut_ptr(), 3);
        } else {
            break;
        }
    }

    /* Scale assignment */
    if hres == S_OK {
        (*val).scale = (-exponent10) as u8;
    }

    hres
}

unsafe fn VARIANT_DI_FromR4(source: f32, dest: *mut VARIANT_DI) -> HRESULT {
    let bits = source.to_bits();
    let sign = (bits >> 31) & 0x1;
    let exp = ((bits >> 23) & 0xFF) as u8;
    let mantissa = bits & 0x7FFFFF;

    // Special cases
    if mantissa == 0 && exp == 0 {
        // Zero
        VARIANT_DI_clear(dest);
        return S_OK;
    } else if mantissa == 0 && exp == 0xFF {
        // Infinity
        return DISP_E_OVERFLOW;
    } else if exp == 0xFF {
        // NaN
        return DISP_E_BADVARTYPE;
    }

    // Normal or denormalized number
    VARIANT_DI_clear(dest);

    (*dest).sign = sign as u32; // bitfield copy

    let mut bitsnum0 = mantissa;
    let mut exponent2: i32 = (exp as i32) - 127;

    if exp == 0 {
        // Denormalized: correct exponent
        exponent2 += 1;
    } else {
        // Normalized: add hidden bit
        bitsnum0 |= 0x00800000;
    }

    (*dest).bitsnum[0] = bitsnum0;

    // Adjust exponent as if mantissa was shifted left 23 bits
    exponent2 -= 23;

    VARIANT_DI_normalize(dest, exponent2, 0)
}

unsafe fn VARIANT_DI_FromR8(source: f64, dest: *mut VARIANT_DI) -> HRESULT {
    let mut hres: HRESULT = S_OK;
    let bits: u64 = source.to_bits();

    let mantissa_lo: u32 = (bits & 0xFFFF_FFFF) as u32;
    let mantissa_hi: u32 = ((bits >> 32) & 0x000F_FFFF) as u32;
    let exp_bias: u32 = ((bits >> 52) & 0x7FF) as u32;
    let sign: u32 = (bits >> 63) as u32;

    /* Detect special cases */
    if mantissa_lo == 0 && mantissa_hi == 0 && exp_bias == 0 {
        /* Floating-point zero */
        VARIANT_DI_clear(dest);
    } else if mantissa_lo == 0 && mantissa_hi == 0 && exp_bias == 0x7FF {
        /* Floating-point infinity */
        hres = DISP_E_OVERFLOW;
    } else if exp_bias == 0x7FF {
        /* Floating-point NaN */
        hres = DISP_E_BADVARTYPE;
    } else {
        let mut exponent2: i32;
        VARIANT_DI_clear(dest);

        exponent2 = (exp_bias as i32) - 1023;   /* Get unbiased exponent */
        (*dest).sign = sign;                    /* Sign is simply copied */

        /* Copy significant bits to VARIANT_DI mantissa */
        (*dest).bitsnum[0] = mantissa_lo;
        (*dest).bitsnum[1] = mantissa_hi;
        (*dest).bitsnum[1] &= 0x000F_FFFF;
        if exp_bias == 0 {
            /* Denormalized number - correct exponent */
            exponent2 += 1;
        } else {
            /* Add hidden bit to mantissa */
            (*dest).bitsnum[1] |= 0x0010_0000;
        }

        /* The act of copying a FP mantissa as integer bits is equivalent to
           shifting left the mantissa 52 bits. The exponent2 is reduced to
           compensate. */
        exponent2 -= 52;

        hres = VARIANT_DI_normalize(dest, exponent2, 1u8);
    }

    hres
}

unsafe fn VARIANT_do_division(
    pDecLeft: *const DECIMAL,
    pDecRight: *const DECIMAL,
    pDecOut: *mut DECIMAL,
    round: i32,
) -> HRESULT {
    let mut hRet: HRESULT = S_OK;
    let mut di_left: VARIANT_DI = core::mem::zeroed();
    let mut di_right: VARIANT_DI = core::mem::zeroed();
    let mut di_result: VARIANT_DI = core::mem::zeroed();

    VARIANT_DIFromDec(pDecLeft, &mut di_left);
    VARIANT_DIFromDec(pDecRight, &mut di_right);
    let divresult: HRESULT = VARIANT_DI_div(&di_left, &di_right, &mut di_result, round as u8);
    if divresult != S_OK {
        hRet = divresult;
    } else {
        hRet = S_OK;
        if di_result.scale > DEC_MAX_SCALE {
            let mut remainder: u8 = 0;
            let _warn_fmt1 = b"result scale is %u, scaling (with loss of significant digits)...\n\0";
            WARN!(warn_fmt1.as_ptr() as *const i8, di_result.scale);
            while di_result.scale > DEC_MAX_SCALE
                && VARIANT_int_iszero(di_result.bitsnum.as_ptr(), 3) == 0
            {
                remainder = VARIANT_int_divbychar(di_result.bitsnum.as_mut_ptr(), 3, 10);
                di_result.scale -= 1;
            }
            if di_result.scale > DEC_MAX_SCALE {
                let _warn_fmt2 = b"result underflowed, setting to 0\n\0";
                WARN!(warn_fmt2.as_ptr() as *const i8);
                di_result.scale = 0;
                di_result.sign = 0;
            } else if remainder >= 5 {
                let mut carry: u8 = 1;
                for i in 0..3 {
                    if carry == 0 {
                        break;
                    }
                    let digit = di_result.bitsnum[i] as u64 + 1;
                    carry = if digit > 0xFFFFFFFFu64 { 1 } else { 0 };
                    di_result.bitsnum[i] = (digit & 0xFFFFFFFF) as u32;
                }
            }
        }
        VARIANT_DecFromDI(&mut di_result, pDecOut);
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarDecDiv(
    pDecLeft: *const DECIMAL,
    pDecRight: *const DECIMAL,
    pDecOut: *mut DECIMAL,
) -> HRESULT {
    if pDecLeft.is_null() || pDecRight.is_null() || pDecOut.is_null() {
        return E_INVALIDARG;
    }
    VARIANT_do_division(pDecLeft, pDecRight, pDecOut, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecMul(
    pDecLeft: *const DECIMAL,
    pDecRight: *const DECIMAL,
    pDecOut: *mut DECIMAL,
) -> HRESULT {
    let mut hRet: HRESULT = S_OK;
    let mut di_left: VARIANT_DI = core::mem::zeroed();
    let mut di_right: VARIANT_DI = core::mem::zeroed();
    let mut di_result: VARIANT_DI = core::mem::zeroed();

    unsafe {
        VARIANT_DIFromDec(pDecLeft, core::ptr::addr_of_mut!(di_left));
        VARIANT_DIFromDec(pDecRight, core::ptr::addr_of_mut!(di_right));
        let mulresult = VARIANT_DI_mul(
            &di_left as *const VARIANT_DI,
            &di_right as *const VARIANT_DI,
            core::ptr::addr_of_mut!(di_result),
        );

        if mulresult != 0 {
            hRet = DISP_E_OVERFLOW;
        } else {
            if di_result.scale > DEC_MAX_SCALE as u8 {
                WARN!(
                    b"result scale is %u, scaling (with loss of significant digits)...\n\0"
                        .as_ptr() as *const u8,
                    di_result.scale as u32,
                );
                while di_result.scale > DEC_MAX_SCALE as u8
                    && VARIANT_int_iszero(di_result.bitsnum.as_ptr(), 3) == 0
                {
                    VARIANT_int_divbychar(di_result.bitsnum.as_mut_ptr(), 3, 10u8);
                    di_result.scale -= 1;
                }
                if di_result.scale > DEC_MAX_SCALE as u8 {
                    WARN!(
                        b"result underflowed, setting to 0\n\0".as_ptr() as *const u8,
                    );
                    di_result.scale = 0;
                    di_result.sign = 0;
                }
            }
            VARIANT_DecFromDI(core::ptr::addr_of_mut!(di_result), pDecOut);
        }
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarDecSub(
    pDecLeft: *const DECIMAL,
    pDecRight: *const DECIMAL,
    pDecOut: *mut DECIMAL,
) -> HRESULT {
    let mut decRight: DECIMAL = core::mem::zeroed();
    VarDecNeg(pDecRight, &mut decRight as *mut DECIMAL);
    VarDecAdd(pDecLeft, &mut decRight as *mut DECIMAL, pDecOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDecAbs(pDecIn: *const DECIMAL, pDecOut: *mut DECIMAL) -> HRESULT {
    *pDecOut = *pDecIn;
    (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.sign &= !0x80u8;
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarDecFix(pDecIn: *const _DECIMAL, pDecOut: *mut _DECIMAL) -> HRESULT {
    const DECIMAL_NEG: u8 = 0x80;
    let sign_byte = (*pDecIn).__bindgen_anon_1.__bindgen_anon_1.sign;
    if sign_byte & !DECIMAL_NEG != 0 {
        return E_INVALIDARG;
    }
    let scale = (*pDecIn).__bindgen_anon_1.__bindgen_anon_1.scale;
    if scale == 0 {
        core::ptr::copy_nonoverlapping(pDecIn, pDecOut, 1);
        return S_OK;
    }
    let mut dbl: f64 = 0.0;
    let mut hr: HRESULT = VarR8FromDec(pDecIn, &mut dbl);
    if hr >= 0 {
        let rounded: i64 = dbl as i64;
        hr = VarDecFromI8(rounded, pDecOut);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn VarDecInt(pDecIn: *const DECIMAL, pDecOut: *mut DECIMAL) -> HRESULT {
    if (DEC_SIGN(pDecIn as *mut DECIMAL) & !DECIMAL_NEG) != 0 {
        return E_INVALIDARG;
    }

    if (DEC_SIGN(pDecIn as *mut DECIMAL) & DECIMAL_NEG) == 0
        || DEC_SCALE(pDecIn as *mut DECIMAL) == 0
    {
        return VarDecFix(pDecIn, pDecOut);
    }

    let mut dbl: f64 = 0.0;
    let mut hr = VarR8FromDec(pDecIn, core::ptr::addr_of_mut!(dbl));
    if hr >= 0 {
        let rounded: i64 = (if dbl >= 0.0 { dbl + 0.5 } else { dbl - 0.5 }) as i64;
        hr = VarDecFromI8(rounded, pDecOut);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn VarDecNeg(pDecIn: *const DECIMAL, pDecOut: *mut DECIMAL) -> HRESULT {
    *pDecOut = *pDecIn;
    (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.sign ^= DECIMAL_NEG;
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarDecRound(
    pDecIn: *const DECIMAL,
    cDecimals: i32,
    pDecOut: *mut DECIMAL,
) -> HRESULT {
    if cDecimals < 0
        || (DEC_SIGN(pDecIn as *mut DECIMAL) & !DECIMAL_NEG) != 0
        || DEC_SCALE(pDecIn as *mut DECIMAL) > DEC_MAX_SCALE
    {
        return E_INVALIDARG;
    }

    if cDecimals >= DEC_SCALE(pDecIn as *mut DECIMAL) as i32 {
        core::ptr::copy_nonoverlapping(pDecIn, pDecOut, 1);
        return S_OK;
    }

    let mut divisor: DECIMAL = core::mem::zeroed();
    let mut tmp: DECIMAL = core::mem::zeroed();
    *DEC_LO64_ref(&mut divisor) = 1;
    *DEC_LO64_ref(&mut tmp) = 10;

    let count = DEC_SCALE(pDecIn as *mut DECIMAL) as i32 - cDecimals;
    for _ in 0..count {
        let hr = VarDecMul(&divisor, &tmp, &mut divisor);
        if hr < 0 {
            return hr;
        }
    }

    let hr = VARIANT_do_division(pDecIn, &divisor, pDecOut, 1);
    if hr < 0 {
        return hr;
    }

    (*pDecOut).__bindgen_anon_1.__bindgen_anon_1.scale = cDecimals as u8;

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarDecCmp(
    pDecLeft: *const DECIMAL,
    pDecRight: *const DECIMAL,
) -> HRESULT {
    if pDecLeft.is_null() || pDecRight.is_null() {
        return VARCMP_NULL;
    }

    let left_neg = (DEC_SIGN(pDecLeft as *mut DECIMAL) & DECIMAL_NEG) != 0;
    let right_neg = (DEC_SIGN(pDecRight as *mut DECIMAL) & DECIMAL_NEG) != 0;
    let left_nonzero = ((*DEC_HI32_ref(pDecLeft as *mut DECIMAL))
        | (*DEC_MID32_ref(pDecLeft as *mut DECIMAL))
        | (*DEC_LO32_ref(pDecLeft as *mut DECIMAL)))
        != 0;

    if !left_neg && right_neg && left_nonzero {
        return VARCMP_GT;
    }
    if left_neg && !right_neg && left_nonzero {
        return VARCMP_LT;
    }

    let mut result: DECIMAL = core::mem::zeroed();
    let hRet = VarDecSub(pDecLeft, pDecRight, &mut result);

    if hRet >= 0 { /* SUCCEEDED */
        let non_zero = ((*DEC_HI32_ref(&mut result))
            | (*DEC_MID32_ref(&mut result))
            | (*DEC_LO32_ref(&mut result)))
            != 0;
        if (DEC_SIGN(&mut result) & DECIMAL_NEG) != 0 && non_zero {
            VARCMP_LT
        } else if non_zero {
            VARCMP_GT
        } else {
            VARCMP_EQ
        }
    } else {
        hRet
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarDecCmpR8(pDecLeft: *const DECIMAL, dblRight: f64) -> HRESULT {
    let mut decRight: DECIMAL = core::mem::zeroed();
    let hRet = VarDecFromR8(dblRight, &mut decRight);
    if hRet >= 0 {
        VarDecCmp(pDecLeft, &decRight)
    } else {
        hRet
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromUI1(bIn: u8, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    *pBoolOut = if bIn != 0 { VARIANT_TRUE } else { VARIANT_FALSE };
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromI2(sIn: i16, pBoolOut: *mut i16) -> i32 {
    *pBoolOut = if sIn != 0 { -1i16 } else { 0i16 };
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromI4(lIn: i32, pBoolOut: *mut i16) -> i32 {
    *pBoolOut = if lIn != 0 { -1i16 } else { 0i16 };
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromR4(fltIn: FLOAT, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    *pBoolOut = if fltIn != 0.0 { VARIANT_TRUE } else { VARIANT_FALSE };
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromR8(dblIn: f64, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    *pBoolOut = if dblIn != 0.0 { VARIANT_TRUE } else { VARIANT_FALSE };
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromDate(dateIn: f64, pBoolOut: *mut i16) -> i32 {
    *pBoolOut = if dateIn != 0.0 { -1i16 } else { 0i16 };
    0 // S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromCy(cyIn: CY, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    *pBoolOut = if cyIn.int64 != 0 { VARIANT_TRUE } else { VARIANT_FALSE };
    S_OK
}

unsafe fn VARIANT_GetLocalisedText(langId: LANGID, dwId: DWORD, lpszDest: *mut WCHAR) -> BOOL {
    let hrsrc = FindResourceExW(
        hProxyDll,
        MAKEINTRESOURCEW(6u16) as *const u16,
        MAKEINTRESOURCEW(((dwId >> 4) + 1) as u16) as *const u16,
        langId,
    );
    if hrsrc.is_null() {
        return 0;
    }
    let hmem = LoadResource(hProxyDll, hrsrc);
    if hmem.is_null() {
        return 0;
    }
    let p = LockResource(hmem) as *const u16;
    let mut p = p;
    let index = (dwId & 0x0f) as usize;
    for _ in 0..index {
        let len = *p as usize;
        p = p.wrapping_offset((len + 1) as isize);
    }
    let len = *p as usize;
    core::ptr::copy_nonoverlapping(
        p.wrapping_offset(1) as *const u8,
        lpszDest as *mut u8,
        len * core::mem::size_of::<u16>(),
    );
    lpszDest.add(len).write(0u16);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromStr(
    strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pBoolOut: *mut VARIANT_BOOL,
) -> HRESULT {
    static SZ_FALSE: [u16; 8] = [
        b'#' as u16, b'F' as u16, b'A' as u16, b'L' as u16,
        b'S' as u16, b'E' as u16, b'#' as u16, 0u16,
    ];
    static SZ_TRUE: [u16; 7] = [
        b'#' as u16, b'T' as u16, b'R' as u16, b'U' as u16,
        b'E' as u16, b'#' as u16, 0u16,
    ];

    let mut szBuff: [u16; 64] = [0u16; 64];
    let mut langId: LANGID = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
    let mut hRes: HRESULT = S_OK;

    if strIn.is_null() || pBoolOut.is_null() {
        return DISP_E_TYPEMISMATCH;
    }

    if (dwFlags & VAR_LOCALBOOL) != 0 {
        let convertedLcid = ConvertDefaultLocale(lcid);
        langId = LANGIDFROMLCID(convertedLcid);
        if PRIMARYLANGID(langId) == LANG_NEUTRAL {
            langId = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
        }
    }

    'check_localised: loop {
        if VARIANT_GetLocalisedText(langId, IDS_TRUE, szBuff.as_mut_ptr()) != 0 {
            if _wcsicmp(strIn, szBuff.as_ptr()) == 0 {
                *pBoolOut = VARIANT_TRUE;
                return hRes;
            }
            VARIANT_GetLocalisedText(langId, IDS_FALSE, szBuff.as_mut_ptr());
            if _wcsicmp(strIn, szBuff.as_ptr()) == 0 {
                *pBoolOut = VARIANT_FALSE;
                return hRes;
            }
        }

        if langId != MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT) {
            langId = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
            continue 'check_localised;
        }
        break;
    }

    if wcscmp(strIn, SZ_FALSE.as_ptr()) == 0 {
        *pBoolOut = VARIANT_FALSE;
    } else if wcscmp(strIn, SZ_TRUE.as_ptr()) == 0 {
        *pBoolOut = VARIANT_TRUE;
    } else {
        let mut d: f64 = 0.0;
        hRes = VarR8FromStr(strIn, lcid, dwFlags, &mut d);
        if SUCCEEDED(hRes) {
            *pBoolOut = if d != 0.0 { VARIANT_TRUE } else { VARIANT_FALSE };
        }
    }

    hRes
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pBoolOut: *mut VARIANT_BOOL,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pBoolOut as *mut core::ffi::c_void, VT_BOOL, 0u32)
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromI1(cIn: i8, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    *pBoolOut = if cIn != 0 { VARIANT_TRUE } else { VARIANT_FALSE };
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromUI2(usIn: u16, pBoolOut: *mut i16) -> i32 {
    *pBoolOut = if usIn != 0 { -1i16 } else { 0i16 };
    0
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromUI4(ulIn: ULONG, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    *pBoolOut = if ulIn != 0 { VARIANT_TRUE } else { VARIANT_FALSE };
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromDec(pDecIn: *mut DECIMAL, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    if DEC_SCALE(pDecIn) > DEC_MAX_SCALE || (DEC_SIGN(pDecIn) & !DECIMAL_NEG) != 0 {
        return E_INVALIDARG;
    }

    if (*DEC_HI32_ref(pDecIn)) != 0 || (*DEC_MID32_ref(pDecIn)) != 0 || (*DEC_LO32_ref(pDecIn)) != 0 {
        *pBoolOut = VARIANT_TRUE;
    } else {
        *pBoolOut = VARIANT_FALSE;
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromI8(llIn: LONG64, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    *pBoolOut = if llIn != 0 { VARIANT_TRUE } else { VARIANT_FALSE };
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarBoolFromUI8(ullIn: ULONG64, pBoolOut: *mut VARIANT_BOOL) -> HRESULT {
    *pBoolOut = if ullIn != 0 { VARIANT_TRUE } else { VARIANT_FALSE };
    S_OK
}

unsafe fn VARIANT_WriteNumber(mut ulVal: u64, mut szOut: *mut u16) -> *mut u16 {
    loop {
        let ulNextDigit = (ulVal % 10) as u16;
        core::ptr::write(szOut, b'0' as u16 + ulNextDigit);
        szOut = szOut.offset(-1);
        ulVal = (ulVal - ulNextDigit as u64) / 10;
        if ulVal == 0 {
            break;
        }
    }
    szOut.offset(1)
}

unsafe fn VARIANT_MakeBstr(lcid: u32, dwFlags: u32, mut szOut: *mut u16) -> *mut u16 {
    unsafe {
        let mut szConverted: [u16; 256] = [0; 256];
        if (dwFlags & VAR_NEGATIVE) != 0 {
            szOut = szOut.sub(1);
            *szOut = '-' as u16;
        }
        if (dwFlags & LOCALE_USE_NLS) != 0 {
            GetNumberFormatW(
                lcid,
                dwFlags & LOCALE_NOUSEROVERRIDE,
                szOut,
                core::ptr::null_mut(),
                szConverted.as_mut_ptr(),
                256,
            );
            szOut = szConverted.as_mut_ptr();
        }
        let len = (lstrlenW(szOut) as usize) * 2;
        SysAllocStringByteLen(szOut as *const i8, len as u32)
    }
}

unsafe fn VARIANT_BstrFromUInt(ulVal: u64, lcid: u32, dwFlags: u32, pbstrOut: *mut *mut u16) -> i32 {
    let mut szBuff: [u16; 64] = [0u16; 64];
    let mut szOut = core::ptr::addr_of_mut!(szBuff[63]);

    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }

    *szOut = 0;
    szOut = szOut.sub(1);

    let szStart = VARIANT_WriteNumber(ulVal, szOut);
    *pbstrOut = VARIANT_MakeBstr(lcid, dwFlags, szStart);

    // TRACE("returning %s\n", debugstr_w(*pbstrOut));
    if (*pbstrOut).is_null() { E_OUTOFMEMORY } else { S_OK }
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromUI1(
    bIn: u8,
    lcid: u32,
    dwFlags: u32,
    pbstrOut: *mut *mut u16,
) -> i32 {
    VARIANT_BstrFromUInt(bIn as u64, lcid, dwFlags, pbstrOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromI2(sIn: i16, lcid: u32, dwFlags: u32, pbstrOut: *mut *mut u16) -> i32 {
    let ul64: u64 = (sIn as i32).unsigned_abs() as u64;
    let mut flags = dwFlags;
    if sIn < 0 {
        flags |= VAR_NEGATIVE;
    }
    VARIANT_BstrFromUInt(ul64, lcid, flags, pbstrOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromI4(lIn: i32, lcid: u32, dwFlags: u32, pbstrOut: *mut *mut u16) -> i32 {
    let ul64: u64;
    let mut flags = dwFlags;
    if lIn < 0 {
        ul64 = (-(lIn as i64)) as u64;
        flags |= VAR_NEGATIVE;
    } else {
        ul64 = lIn as u64;
    }
    VARIANT_BstrFromUInt(ul64, lcid, flags, pbstrOut)
}

unsafe fn VARIANT_BstrReplaceDecimal(buff: *const u16, lcid: u32, dwFlags: u32) -> *mut u16 {
    let bstrOut;
    let mut lpDecimalSep: [u16; 16] = [0u16; 16];
    let empty: [u16; 1] = [0u16];

    GetLocaleInfoW(
        lcid,
        0x0000000E_u32 | (dwFlags & 0x80000000_u32),
        lpDecimalSep.as_mut_ptr(),
        16,
    );

    if lpDecimalSep[0] == b'.' as u16 && lpDecimalSep[1] == 0u16 {
        bstrOut = SysAllocString(buff);
    } else {
        let mut numbuff: [u16; 256] = [0u16; 256];
        let mut minFormat: NUMBERFMTW = core::mem::zeroed();
        minFormat.NumDigits = 0;
        minFormat.LeadingZero = 0;
        minFormat.Grouping = 0;
        minFormat.lpDecimalSep = lpDecimalSep.as_mut_ptr();
        minFormat.lpThousandSep = empty.as_ptr() as *mut u16;
        minFormat.NegativeOrder = 1;

        // Wine 10.0: the locale's leading-zero setting (was hard-coded 0), and
        // an exponent is split off before formatting and re-appended after,
        // so "1,5E+20" does not count "5E+20" as decimal digits.
        GetLocaleInfoW(
            lcid,
            LOCALE_ILZERO | LOCALE_RETURN_NUMBER | (dwFlags & LOCALE_NOUSEROVERRIDE),
            &mut minFormat.LeadingZero as *mut u32 as *mut u16,
            (core::mem::size_of::<u32>() / core::mem::size_of::<u16>()) as i32,
        );

        // buff is the caller's local buffer; copy so the exponent can be cut
        let mut work: [u16; 256] = [0u16; 256];
        let n = (lstrlenW(buff) as usize).min(255);
        core::ptr::copy_nonoverlapping(buff, work.as_mut_ptr(), n);
        let p = wcschr(work.as_ptr(), b'.' as u16);
        let e = wcschr(if p.is_null() { work.as_ptr() } else { p.add(1) }, b'E' as u16);
        if !p.is_null() {
            minFormat.NumDigits = if e.is_null() {
                lstrlenW(p.add(1)) as u32
            } else {
                e.offset_from(p.add(1)) as u32
            };
        }
        let mut exponent: [u16; 16] = [0u16; 16];
        if !e.is_null() {
            let en = (lstrlenW(e) as usize).min(15);
            core::ptr::copy_nonoverlapping(e, exponent.as_mut_ptr(), en);
            *(e as *mut u16) = 0;
        }
        let buff = work.as_ptr();

        numbuff[0] = 0u16;
        if GetNumberFormatW(lcid, 0, buff, &minFormat, numbuff.as_mut_ptr(), 256) == 0 {
            WARN!(
                b"GetNumberFormatW() failed, returning raw number string instead\0"
                    .as_ptr() as *const i8,
            );
            bstrOut = SysAllocString(buff);
        } else {
            if exponent[0] != 0 {
                let l = lstrlenW(numbuff.as_ptr()) as usize;
                let el = lstrlenW(exponent.as_ptr()) as usize;
                if l + el < numbuff.len() {
                    core::ptr::copy_nonoverlapping(exponent.as_ptr(), numbuff.as_mut_ptr().add(l), el + 1);
                }
            }
            TRACE!(
                b"created minimal NLS string %s\n\0".as_ptr() as *const i8,
                debugstr_w(numbuff.as_ptr()),
            );
            bstrOut = SysAllocString(numbuff.as_mut_ptr());
        }
    }

    bstrOut
}

unsafe fn VARIANT_BstrFromReal(
    dblIn: f64,
    lcid: u32,
    dwFlags: u32,
    pbstrOut: *mut *mut u16,
    ndigits: i32,
) -> HRESULT {
    let mut buff: [u16; 256] = [0; 256];

    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }

    // Wine 10.0: %.*G first; if that chose an exponent but the number fits in
    // `ndigits` significant digits as plain decimals, print it that way
    // instead (VarBstrFromR8(0.00001) is "0.00001", not "1E-05"). %.*f is only
    // reached for magnitudes below 10^ndigits, so it cannot overrun buff.
    let fmt_g: [u16; 5] = [b'%' as u16, b'.' as u16, b'*' as u16, b'G' as u16, 0];
    let mut len = swprintf(buff.as_mut_ptr(), fmt_g.as_ptr(), ndigits, dblIn);
    let e = wcschr(buff.as_ptr(), b'E' as u16);
    if !e.is_null() {
        let dot = wcschr(buff.as_ptr(), b'.' as u16);
        let extra_decimals = if dot.is_null() { 0 } else { e.offset_from(dot) as i32 - 2 };
        // the exponent: optional sign, then decimal digits (wcstol(e+1, NULL, 10))
        let mut q = e.add(1);
        let neg = *q == b'-' as u16;
        if neg || *q == b'+' as u16 {
            q = q.add(1);
        }
        let mut exp: i32 = 0;
        while (b'0' as u16..=b'9' as u16).contains(&*q) {
            exp = exp * 10 + (*q - b'0' as u16) as i32;
            q = q.add(1);
        }
        if exp + extra_decimals < ndigits {
            let fmt_f: [u16; 5] = [b'%' as u16, b'.' as u16, b'*' as u16, b'f' as u16, 0];
            len = swprintf(buff.as_mut_ptr(), fmt_f.as_ptr(), ndigits, dblIn);
            while len > 0 && buff[len as usize - 1] == b'0' as u16 {
                len -= 1;
            }
        }
    }
    if len >= 0 && (len as usize) < buff.len() {
        buff[len as usize] = 0;
    }

    if buff[0] == b'-' as u16 {
        const SZ_ACCEPT: [u16; 3] = [b'0' as u16, b'.' as u16, 0];
        let remain_ptr = buff.as_ptr().add(1);
        if lstrlenW(remain_ptr) == wcsspn(remain_ptr, SZ_ACCEPT.as_ptr()) as i32 {
            buff[0] = b'0' as u16;
            buff[1] = 0;
        }
    }

    if (dwFlags & LOCALE_USE_NLS) != 0 {
        let mut numbuff: [u16; 256] = [0; 256];
        numbuff[0] = 0;
        GetNumberFormatW(
            lcid,
            dwFlags & LOCALE_NOUSEROVERRIDE,
            buff.as_ptr(),
            core::ptr::null_mut(),
            numbuff.as_mut_ptr(),
            numbuff.len() as i32,
        );
        *pbstrOut = SysAllocString(numbuff.as_ptr());
    } else {
        *pbstrOut = VARIANT_BstrReplaceDecimal(buff.as_ptr(), lcid, dwFlags);
    }

    if !(*pbstrOut).is_null() {
        S_OK
    } else {
        E_OUTOFMEMORY
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromR4(
    fltIn: f32,
    lcid: u32,
    dwFlags: u32,
    pbstrOut: *mut *mut u16,
) -> HRESULT {
    VARIANT_BstrFromReal(fltIn as f64, lcid, dwFlags, pbstrOut, 7)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromR8(
    dblIn: f64,
    lcid: u32,
    dwFlags: u32,
    pbstrOut: *mut *mut u16,
) -> HRESULT {
    VARIANT_BstrFromReal(dblIn, lcid, dwFlags, pbstrOut, 15)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromCy(
    cyIn: CY,
    lcid: u32,
    dwFlags: u32,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }

    let mut buff: [u16; 256] = [0; 256];
    let mut decVal: VARIANT_DI = core::mem::zeroed();

    decVal.scale = 4;
    decVal.sign = 0;
    decVal.bitsnum[0] = cyIn.__bindgen_anon_1.Lo;
    decVal.bitsnum[1] = cyIn.__bindgen_anon_1.Hi as u32;

    if (cyIn.__bindgen_anon_1.Hi as u32 & 0x80000000u32) != 0 {
        let one: u32 = 1;
        decVal.sign = 1;
        decVal.bitsnum[0] = !decVal.bitsnum[0];
        decVal.bitsnum[1] = !decVal.bitsnum[1];
        VARIANT_int_add(decVal.bitsnum.as_mut_ptr(), 3, &one as *const u32, 1);
    }
    decVal.bitsnum[2] = 0;

    VARIANT_DI_tostringW(&decVal as *const VARIANT_DI, buff.as_mut_ptr(), 256);

    if (dwFlags & LOCALE_USE_NLS) != 0 {
        let mut cybuff: [u16; 256] = [0; 256];
        cybuff[0] = 0;

        GetCurrencyFormatW(
            lcid,
            dwFlags & LOCALE_NOUSEROVERRIDE,
            buff.as_ptr(),
            core::ptr::null(),
            cybuff.as_mut_ptr(),
            256,
        );
        *pbstrOut = SysAllocString(cybuff.as_ptr());
    } else {
        *pbstrOut = VARIANT_BstrReplaceDecimal(buff.as_ptr(), lcid, dwFlags);
    }

    if !(*pbstrOut).is_null() {
        S_OK
    } else {
        E_OUTOFMEMORY
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromDec(
    pDecIn: *mut DECIMAL,
    lcid: LCID,
    dwFlags: ULONG,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    let mut buff: [u16; 256] = [0; 256];
    let mut temp: VARIANT_DI = core::mem::zeroed();

    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }

    VARIANT_DIFromDec(pDecIn, &mut temp);
    VARIANT_DI_tostringW(&temp as *const VARIANT_DI, buff.as_mut_ptr(), 256);

    if (dwFlags & LOCALE_USE_NLS) != 0 {
        let mut numbuff: [u16; 256] = [0; 256];

        /* Format the number for the locale */
        numbuff[0] = 0;
        GetNumberFormatW(
            lcid,
            dwFlags & LOCALE_NOUSEROVERRIDE,
            buff.as_ptr(),
            core::ptr::null(),
            numbuff.as_mut_ptr(),
            256,
        );
        *pbstrOut = SysAllocString(numbuff.as_ptr());
    } else {
        *pbstrOut = VARIANT_BstrReplaceDecimal(buff.as_ptr(), lcid, dwFlags);
    }

    if !(*pbstrOut).is_null() {
        S_OK
    } else {
        E_OUTOFMEMORY
    }
}

unsafe fn output_int_len(o: i32, min_len: i32, date: *mut u16, date_len: i32) -> i32 {
    if min_len >= date_len {
        return -1;
    }
    let mut len: i32 = 0;
    let mut tmp = o;
    while tmp != 0 {
        len += 1;
        tmp /= 10;
    }
    if len == 0 {
        len = 1;
    }
    if len >= date_len {
        return -1;
    }
    let mut p = date;
    let mut pad = min_len - len;
    while pad > 0 {
        *p = '0' as u16;
        p = p.offset(1);
        pad -= 1;
    }
    let mut value = o;
    let mut i = len;
    while i > 0 {
        let digit = (value % 10) as u32;
        *p.offset((i - 1) as isize) = b'0' as u16 + digit as u16;
        value /= 10;
        i -= 1;
    }
    if min_len > len {
        min_len
    } else {
        len
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_date_format(lcid: LCID, flags: DWORD, st: *const SYSTEMTIME, fmt: *const u16, date: *mut u16, date_len: i32) -> i32 {
    const LOCALE_NOUSEROVERRIDE: u32 = 0x80000000;
    const VAR_DATEVALUEONLY: u32 = 0x00000002;
    const dayname: [u32; 7] = [
        0x00000030, // LOCALE_SDAYNAME7
        0x0000002A, // LOCALE_SDAYNAME1
        0x0000002B,
        0x0000002C,
        0x0000002D,
        0x0000002E,
        0x0000002F,
    ];
    const sdayname: [u32; 7] = [
        0x00000037,
        0x00000031,
        0x00000032,
        0x00000033,
        0x00000034,
        0x00000035,
        0x00000036,
    ];
    const monthname: [u32; 12] = [
        0x00000038,
        0x00000039,
        0x0000003A,
        0x0000003B,
        0x0000003C,
        0x0000003D,
        0x0000003E,
        0x0000003F,
        0x00000040,
        0x00000041,
        0x00000042,
        0x00000043,
    ];
    const smonthname: [u32; 12] = [
        0x00000044,
        0x00000045,
        0x00000046,
        0x00000047,
        0x00000048,
        0x00000049,
        0x0000004A,
        0x0000004B,
        0x0000004C,
        0x0000004D,
        0x0000004E,
        0x0000004F,
    ];

    let mut flags = flags;
    if flags & !(LOCALE_NOUSEROVERRIDE | VAR_DATEVALUEONLY) != 0 {
        FIXME!(b"ignoring flags %x\n\0".as_ptr() as *const i8, flags);
    }
    flags &= LOCALE_NOUSEROVERRIDE;

    let mut pfmt = fmt;
    let mut pdate = date;
    let mut date_len = date_len;

    while *pfmt != 0 && date_len > 0 {
        let mut count: i32 = 1;

        // detect sequences of identical characters
        match *pfmt {
            c if c == 'd' as u16 || c == 'M' as u16 || c == 'y' as u16 || c == 'g' as u16 => {
                let c = *pfmt;
                let mut cnt = 1;
                while *pfmt.add(cnt as usize) == c {
                    cnt += 1;
                }
                count = cnt;
                pfmt = pfmt.add((count - 1) as usize);
            }
            _ => {}
        }

        match *pfmt {
            c if c == 'd' as u16 => {
                if count >= 4 {
                    count = GetLocaleInfoW(lcid, dayname[(*st).wDayOfWeek as usize] | flags, pdate, date_len) - 1;
                } else if count == 3 {
                    count = GetLocaleInfoW(lcid, sdayname[(*st).wDayOfWeek as usize] | flags, pdate, date_len) - 1;
                } else {
                    count = output_int_len((*st).wDay as i32, count, pdate, date_len);
                }
            }
            c if c == 'M' as u16 => {
                if count >= 4 {
                    count = GetLocaleInfoW(lcid, monthname[((*st).wMonth - 1) as usize] | flags, pdate, date_len) - 1;
                } else if count == 3 {
                    count = GetLocaleInfoW(lcid, smonthname[((*st).wMonth - 1) as usize] | flags, pdate, date_len) - 1;
                } else {
                    count = output_int_len((*st).wMonth as i32, count, pdate, date_len);
                }
            }
            c if c == 'y' as u16 => {
                if count >= 3 {
                    count = output_int_len((*st).wYear as i32, 0, pdate, date_len);
                } else {
                    count = output_int_len((*st).wYear as i32 % 100, count, pdate, date_len);
                }
            }
            c if c == 'g' as u16 => {
                if count == 2 {
                    FIXME!(b"Should be using GetCalendarInfo(CAL_SERASTRING), defaulting to 'AD'\n\0".as_ptr() as *const i8);

                    *pdate = 'A' as u16;
                    pdate = pdate.add(1);
                    date_len -= 1;
                    if date_len > 0 {
                        *pdate = 'D' as u16;
                    } else {
                        count = -1;
                    }
                } else {
                    *pdate = *pfmt;
                }
            }
            _ => {
                *pdate = *pfmt;
            }
        }

        if count < 0 {
            break;
        }
        pfmt = pfmt.add(1);
        pdate = pdate.add(count as usize);
        date_len -= count;
    }

    if date_len <= 0 {
        return 0i32;
    }
    *pdate = 0;
    1i32
}
#[no_mangle]
pub unsafe extern "system" fn VarBstrFromDate(
    dateIn: f64,
    lcid: u32,
    dwFlags: u32,
    pbstrOut: *mut *mut u16,
) -> i32 {
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut date: [u16; 128] = [0u16; 128];
    let mut fmt_buff: [u16; 80] = [0u16; 80];
    let mut dwFlags = dwFlags;
    let dwFormatFlags = dwFlags & LOCALE_NOUSEROVERRIDE;

    TRACE!(
        b"(%g,0x%08x,0x%08x,%p)\n\0".as_ptr() as *const i8,
        dateIn,
        lcid,
        dwFlags,
        pbstrOut,
    );

    if pbstrOut.is_null() || VariantTimeToSystemTime(dateIn, &mut st) == 0 {
        return E_INVALIDARG;
    }
    *pbstrOut = core::ptr::null_mut();

    if (dwFlags & VAR_CALENDAR_THAI) != 0 {
        st.wYear = st.wYear.wrapping_add(553);
    } else if (dwFlags & (VAR_CALENDAR_HIJRI | VAR_CALENDAR_GREGORIAN)) != 0 {
        FIXME!(
            b"VAR_CALENDAR_HIJRI/VAR_CALENDAR_GREGORIAN not handled\n\0".as_ptr()
                as *const i8,
        );
    }

    if (dwFlags & LOCALE_USE_NLS) != 0 {
        dwFlags &= !(VAR_TIMEVALUEONLY | VAR_DATEVALUEONLY);
    } else {
        let whole = if dateIn < 0.0 { ceil(dateIn) } else { floor(dateIn) };
        let partial = dateIn - whole;
        if whole == 0.0 {
            dwFlags |= VAR_TIMEVALUEONLY;
        } else if partial > -1e-12 && partial < 1e-12 {
            dwFlags |= VAR_DATEVALUEONLY;
        }
    }

    if (dwFlags & VAR_TIMEVALUEONLY) != 0 {
        date[0] = 0;
    } else {
        if GetLocaleInfoW(
            lcid,
            LOCALE_SSHORTDATE,
            fmt_buff.as_mut_ptr(),
            fmt_buff.len() as i32,
        ) == 0
            || get_date_format(
                lcid,
                dwFlags,
                &st,
                fmt_buff.as_ptr(),
                date.as_mut_ptr(),
                date.len() as i32,
            ) == 0
        {
            return E_INVALIDARG;
        }
    }

    if (dwFlags & VAR_DATEVALUEONLY) == 0 {
        let date_len = lstrlenW(date.as_ptr());
        let time_raw = date.as_mut_ptr().offset(date_len as isize);
        let time = if time_raw != date.as_mut_ptr() {
            *time_raw = ' ' as u16;
            time_raw.add(1)
        } else {
            time_raw
        };
        let offset = (time as usize - date.as_ptr() as usize) / core::mem::size_of::<u16>();
        let remaining = date.len() as isize - offset as isize;
        if GetTimeFormatW(
            lcid,
            dwFormatFlags,
            &st,
            core::ptr::null(),
            time,
            remaining as i32,
        ) == 0
        {
            return E_INVALIDARG;
        }
    }

    *pbstrOut = SysAllocString(date.as_ptr() as *const u16);
    if !(*pbstrOut).is_null() {
        TRACE!(
            b"returning %s\n\0".as_ptr() as *const i8,
            debugstr_w(*pbstrOut),
        );
    }
    if (*pbstrOut).is_null() {
        E_OUTOFMEMORY
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromBool(
    boolIn: VARIANT_BOOL,
    lcid: LCID,
    dwFlags: ULONG,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    let mut szBuff: [u16; 64] = [0u16; 64];
    let mut dwResId: DWORD = IDS_TRUE;
    let mut lcid = lcid;

    TRACE!(
        b"%d,0x%08x,0x%08x,%p\n\0".as_ptr() as *const i8,
        boolIn as i32,
        lcid as u32,
        dwFlags as u32,
        pbstrOut as *const core::ffi::c_void,
    );

    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }

    let flags = dwFlags & (VAR_LOCALBOOL | VAR_BOOLONOFF | VAR_BOOLYESNO);
    if flags == VAR_BOOLONOFF {
        dwResId = IDS_ON;
    } else if flags == VAR_BOOLYESNO {
        dwResId = IDS_YES;
    } else if flags != VAR_LOCALBOOL {
        lcid = MAKELCID(MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT), SORT_DEFAULT);
    }

    lcid = ConvertDefaultLocale(lcid);
    let mut langId = LANGIDFROMLCID(lcid);
    if PRIMARYLANGID(langId) == LANG_NEUTRAL {
        langId = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
    }

    if boolIn == VARIANT_FALSE {
        dwResId += 1;
    }

    'try_again: loop {
        if VARIANT_GetLocalisedText(langId, dwResId, szBuff.as_mut_ptr()) != 0 {
            *pbstrOut = SysAllocString(szBuff.as_ptr());
            return if (*pbstrOut).is_null() {
                E_OUTOFMEMORY
            } else {
                S_OK
            };
        }

        if langId != MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT) {
            langId = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
            continue 'try_again;
        }
        break;
    }

    WARN!(b"Failed to load bool text!\n\0".as_ptr() as *const i8);
    E_OUTOFMEMORY
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromI1(
    cIn: i8,
    lcid: u32,
    mut dwFlags: u32,
    pbstrOut: *mut *mut u16,
) -> i32 {
    let ul64: u64 = if cIn < 0 {
        dwFlags |= VAR_NEGATIVE;
        (-(cIn as i32)) as u64
    } else {
        cIn as u64
    };
    VARIANT_BstrFromUInt(ul64, lcid, dwFlags, pbstrOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromUI2(
    usIn: u16,
    lcid: u32,
    dwFlags: u32,
    pbstrOut: *mut *mut u16,
) -> i32 {
    VARIANT_BstrFromUInt(usIn as u64, lcid, dwFlags, pbstrOut)
}
#[no_mangle]
pub unsafe extern "system" fn VarBstrFromUI4(ulIn: u32, lcid: LCID, dwFlags: u32, pbstrOut: *mut BSTR) -> HRESULT {
    VARIANT_BstrFromUInt(ulIn as u64, lcid, dwFlags, pbstrOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromI8(llIn: i64, lcid: u32, dwFlags: u32, pbstrOut: *mut *mut u16) -> i32 {
    let mut flags = dwFlags;
    let ul64 = if llIn < 0 {
        flags |= VAR_NEGATIVE;
        llIn.wrapping_neg() as u64
    } else {
        llIn as u64
    };
    VARIANT_BstrFromUInt(ul64, lcid, flags, pbstrOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromUI8(ullIn: u64, lcid: u32, dwFlags: u32, pbstrOut: *mut *mut u16) -> HRESULT {
    VARIANT_BstrFromUInt(ullIn, lcid, dwFlags, pbstrOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrFromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    dwFlags: u32,
    pbstrOut: *mut *mut u16,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pbstrOut as *mut core::ffi::c_void, VT_BSTR, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrCat(
    pbstrLeft: BSTR,
    pbstrRight: BSTR,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    TRACE!(
        b"%s,%s,%p\n\0".as_ptr() as *const i8,
        debugstr_wn(pbstrLeft, SysStringLen(pbstrLeft)) as *const core::ffi::c_void,
        debugstr_wn(pbstrRight, SysStringLen(pbstrRight)) as *const core::ffi::c_void,
        pbstrOut as *const core::ffi::c_void,
    );

    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }

    let lenLeft = if pbstrLeft.is_null() {
        0u32
    } else {
        SysStringByteLen(pbstrLeft)
    };
    let lenRight = if pbstrRight.is_null() {
        0u32
    } else {
        SysStringByteLen(pbstrRight)
    };

    *pbstrOut = SysAllocStringByteLen(core::ptr::null_mut(), lenLeft + lenRight);
    if (*pbstrOut).is_null() {
        return E_OUTOFMEMORY;
    }

    // C: (*pbstrOut)[0] = '\0'. The translation wrote
    // `(*(*pbstrOut) as *mut u8).write_unaligned(0)`, which casts the first
    // WCHAR's VALUE (0, just memset) to a pointer and stores through it: every
    // VarBstrCat call faulted on address 0.
    *(*pbstrOut) = 0;

    if !pbstrLeft.is_null() {
        core::ptr::copy_nonoverlapping(
            pbstrLeft as *const u8,
            *pbstrOut as *mut u8,
            lenLeft as usize,
        );
    }
    if !pbstrRight.is_null() {
        let dst = (*pbstrOut as *mut u8).add(lenLeft as usize);
        core::ptr::copy_nonoverlapping(
            pbstrRight as *const u8,
            dst,
            lenRight as usize,
        );
    }

    TRACE!(
        b"%s\n\0".as_ptr() as *const i8,
        debugstr_wn(*pbstrOut, SysStringLen(*pbstrOut)) as *const core::ffi::c_void,
    );

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarBstrCmp(
    pbstrLeft: BSTR,
    pbstrRight: BSTR,
    lcid: LCID,
    dwFlags: DWORD,
) -> HRESULT {
    TRACE!(
        b"%s,%s,%d,%08x\n\0".as_ptr() as *const i8,
        debugstr_wn(pbstrLeft, SysStringLen(pbstrLeft)),
        debugstr_wn(pbstrRight, SysStringLen(pbstrRight)),
        lcid,
        dwFlags,
    );

    if pbstrLeft.is_null() || *pbstrLeft == 0u16 {
        if !pbstrRight.is_null() && *pbstrRight != 0u16 {
            return VARCMP_LT;
        }
    } else if pbstrRight.is_null() || *pbstrRight == 0u16 {
        return VARCMP_GT;
    }

    if lcid == 0 {
        let lenLeft = SysStringByteLen(pbstrLeft) as usize;
        let lenRight = SysStringByteLen(pbstrRight) as usize;
        let cmp_len = if lenLeft < lenRight { lenLeft } else { lenRight };
        let ret = memcmp(
            pbstrLeft as *const core::ffi::c_void,
            pbstrRight as *const core::ffi::c_void,
            cmp_len,
        );
        if ret < 0 {
            return VARCMP_LT;
        }
        if ret > 0 {
            return VARCMP_GT;
        }
        if lenLeft < lenRight {
            return VARCMP_LT;
        }
        if lenLeft > lenRight {
            return VARCMP_GT;
        }
        return VARCMP_EQ;
    } else {
        let lenLeft = SysStringLen(pbstrLeft) as usize;
        let lenRight = SysStringLen(pbstrRight) as usize;
        if lenLeft == 0 || lenRight == 0 {
            if lenLeft == 0 && lenRight == 0 {
                return VARCMP_EQ;
            }
            return if lenLeft < lenRight {
                VARCMP_LT
            } else {
                VARCMP_GT
            };
        }
        let hres: HRESULT = CompareStringW(
            lcid,
            dwFlags,
            pbstrLeft,
            lenLeft as i32,
            pbstrRight,
            lenRight as i32,
        ) - CSTR_LESS_THAN;
        TRACE!(b"%d\n\0".as_ptr() as *const i8, hres);
        hres
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromUI1(bIn: u8, pdateOut: *mut f64) -> i32 {
    VarR8FromUI1(bIn, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromI2(sIn: i16, pdateOut: *mut f64) -> HRESULT {
    VarR8FromI2(sIn, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromI4(lIn: i32, pdateOut: *mut f64) -> HRESULT {
    VarDateFromR8(lIn as f64, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromR4(fltIn: FLOAT, pdateOut: *mut f64) -> HRESULT {
    VarR8FromR4(fltIn, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromR8(dblIn: f64, pdateOut: *mut f64) -> HRESULT {
    if dblIn <= (DATE_MIN as f64 - 1.0) || dblIn >= (DATE_MAX as f64 + 1.0) {
        return DISP_E_OVERFLOW;
    }
    *pdateOut = dblIn;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromDisp(
    pdispIn: *mut IDispatch,
    lcid: LCID,
    pdateOut: *mut f64,
) -> HRESULT {
    VARIANT_FromDisp(pdispIn, lcid, pdateOut as *mut core::ffi::c_void, VT_DATE as VARTYPE, 0)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromBool(boolIn: i16, pdateOut: *mut f64) -> HRESULT {
    VarR8FromBool(boolIn, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromCy(cyIn: CY, pdateOut: *mut DATE) -> HRESULT {
    VarR8FromCy(cyIn, pdateOut)
}

unsafe fn VARIANT_IsValidMonthDay(day: u32, month: u32, year: u32) -> u8 {
    const DAYS: [u8; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    if day != 0 && month != 0 && month < 13 {
        if day <= DAYS[month as usize] as u32 || (month == 2 && day == 29 && IsLeapYear(year as i32)) {
            return 1u8;
        }
    }
    0u8
}

unsafe fn VARIANT_MakeDate(
    dp: *mut DATEPARSE,
    iDate: u32,
    offset: u32,
    st: *mut SYSTEMTIME,
) -> HRESULT {

    let mut v1: u32;
    let mut v2: u32;
    let mut v3: u32;

    if (*dp).dwCount == 0 {
        v1 = 30;
        v2 = 12;
        v3 = 1899;
    } else {
        v1 = (*dp).dwValues[(offset + 0) as usize];
        v2 = (*dp).dwValues[(offset + 1) as usize];
        if (*dp).dwCount == 2 {
            let mut current: SYSTEMTIME = core::mem::zeroed();
            GetSystemTime(&mut current);
            v3 = current.wYear as u32;
        } else {
            v3 = (*dp).dwValues[(offset + 2) as usize];
        }

        let mut dwAllOrders: u32;
        if ((*dp).dwFlags[(offset + 0) as usize] & DP_MONTH) != 0 {
            dwAllOrders = ORDER_MDY;
        } else if ((*dp).dwFlags[(offset + 1) as usize] & DP_MONTH) != 0 {
            dwAllOrders = ORDER_DMY;
            if (*dp).dwCount > 2 {
                dwAllOrders |= ORDER_YMD;
            }
        } else if (*dp).dwCount > 2 && ((*dp).dwFlags[(offset + 2) as usize] & DP_MONTH) != 0 {
            dwAllOrders = ORDER_YDM;
        } else {
            dwAllOrders = ORDER_MDY | ORDER_DMY;
            if (*dp).dwCount > 2 {
                dwAllOrders |= ORDER_YMD | ORDER_YDM;
            }
        }

        let mut dwCount: u32 = 0;

        'start: loop {
            TRACE!(
                b"dwAllOrders is 0x%08x\n\0".as_ptr() as *const u8,
                dwAllOrders,
            );

            while dwAllOrders != 0 {
                let dwTry: u32;
                if dwCount == 0 {
                    dwTry = match iDate {
                        0 => dwAllOrders & ORDER_MDY,
                        1 => dwAllOrders & ORDER_DMY,
                        _ => dwAllOrders & ORDER_YMD,
                    };
                } else if dwCount == 1 {
                    dwTry = match iDate {
                        0 => dwAllOrders & !(ORDER_DMY | ORDER_YDM),
                        1 => dwAllOrders & !(ORDER_MDY | ORDER_YDM | ORDER_MYD),
                        _ => dwAllOrders & !(ORDER_DMY | ORDER_YDM),
                    };
                } else {
                    dwTry = dwAllOrders;
                }

                TRACE!(
                    b"Attempt %d, dwTry is 0x%08x\n\0".as_ptr() as *const u8,
                    dwCount,
                    dwTry,
                );
                dwCount += 1;
                if dwTry == 0 {
                    continue;
                }

                if (dwTry & ORDER_MDY) != 0 {
                    if VARIANT_IsValidMonthDay(v2, v1, v3) != 0 {
                        core::mem::swap(&mut v1, &mut v2);
                        break 'start;
                    }
                    dwAllOrders &= !ORDER_MDY;
                }
                if (dwTry & ORDER_YMD) != 0 {
                    if VARIANT_IsValidMonthDay(v3, v2, v1) != 0 {
                        core::mem::swap(&mut v1, &mut v3);
                        break 'start;
                    }
                    dwAllOrders &= !ORDER_YMD;
                }
                if (dwTry & ORDER_YDM) != 0 {
                    if VARIANT_IsValidMonthDay(v2, v3, v1) != 0 {
                        core::mem::swap(&mut v1, &mut v2);
                        core::mem::swap(&mut v2, &mut v3);
                        break 'start;
                    }
                    dwAllOrders &= !ORDER_YDM;
                }
                if (dwTry & ORDER_DMY) != 0 {
                    if VARIANT_IsValidMonthDay(v1, v2, v3) != 0 {
                        break 'start;
                    }
                    dwAllOrders &= !ORDER_DMY;
                }
                if (dwTry & ORDER_MYD) != 0 {
                    if VARIANT_IsValidMonthDay(v3, v1, v2) != 0 {
                        core::mem::swap(&mut v1, &mut v3);
                        core::mem::swap(&mut v2, &mut v3);
                        break 'start;
                    }
                    dwAllOrders &= !ORDER_MYD;
                }
            }

            if (*dp).dwCount == 2 {
                v3 = 1;
                dwAllOrders = ORDER_YMD | ORDER_MYD;
                (*dp).dwCount = 0;
                dwCount = 0;
                continue 'start;
            }

            return DISP_E_TYPEMISMATCH;
        }
    }

    // VARIANT_MakeDate_OK
    if (*st).wHour > 23 || (*st).wMinute > 59 || (*st).wSecond > 59 {
        return DISP_E_TYPEMISMATCH;
    }

    TRACE!(
        b"Time %d %d %d\n\0".as_ptr() as *const u8,
        (*st).wHour,
        (*st).wMinute,
        (*st).wSecond,
    );
    if (*st).wHour < 12 && ((*dp).dwParseFlags & DP_PM) != 0 {
        (*st).wHour += 12;
    } else if (*st).wHour == 12 && ((*dp).dwParseFlags & DP_AM) != 0 {
        (*st).wHour = 0;
    }
    TRACE!(
        b"Time %d %d %d\n\0".as_ptr() as *const u8,
        (*st).wHour,
        (*st).wMinute,
        (*st).wSecond,
    );

    (*st).wDay = v1 as u16;
    (*st).wMonth = v2 as u16;
    // Wine 10.0, as current Windows: two-digit years 00-49 are 20xx (was 00-29).
    let year: u32 = if v3 <= 49 {
        2000 + v3
    } else if v3 < 100 {
        1900 + v3
    } else {
        v3
    };
    (*st).wYear = year as u16;
    TRACE!(
        b"Returning date %d/%d/%d\n\0".as_ptr() as *const u8,
        v1,
        v2,
        (*st).wYear,
    );

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromStr(
    mut strIn: *mut u16,
    lcid: u32,
    dwFlags: u32,
    pdateOut: *mut f64,
) -> i32 {
    if (dwFlags & (VAR_TIMEVALUEONLY | VAR_DATEVALUEONLY)) == (VAR_TIMEVALUEONLY | VAR_DATEVALUEONLY) {
        return E_INVALIDARG; // E_INVALIDARG
    }
    if strIn.is_null() {
        return DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
    }

    *pdateOut = 0.0;

    TRACE!(
        b"(%s,0x%08x,0x%08x,%p)\n\0".as_ptr() as *const i8,
        debugstr_w(strIn),
        lcid,
        dwFlags,
        pdateOut,
    );

    let mut dp: DATEPARSE = core::mem::zeroed();
    let mut iDate: u32 = 0;
    GetLocaleInfoW(
        lcid,
        LOCALE_IDATE | LOCALE_RETURN_NUMBER | (dwFlags & LOCALE_NOUSEROVERRIDE),
        &mut iDate as *mut u32 as *mut u16,
        2,
    );
    TRACE!(b"iDate is %d\n\0".as_ptr() as *const i8, iDate);

    // Get the month/day/am/pm tokens for this locale
    let mut tokens: [*mut u16; TOKEN_COUNT] = [core::ptr::null_mut(); TOKEN_COUNT];
    for i in 0..TOKEN_COUNT {
        let mut buff: [u16; 128] = [0; 128];
        let lctype = PARSE_DATE_TOKENS[i] | (dwFlags & LOCALE_NOUSEROVERRIDE);
        GetLocaleInfoW(lcid, lctype, buff.as_mut_ptr(), 128);
        tokens[i] = SysAllocString(buff.as_ptr());
        TRACE!(
            b"token %d is %s\n\0".as_ptr() as *const i8,
            i as i32,
            debugstr_w(tokens[i]),
        );
    }

    let mut hRet: i32 = 0; // S_OK
    let mut dwDateSeps: u32 = 0;

    // Parse the string into our structure
    while *strIn != 0 {
        let ch = *strIn as u32;
        // ASCII digits only (Wine 10.0): iswdigit also accepts other scripts'
        // digits, which wcstoul below cannot convert.
        if (b'0' as u32..=b'9' as u32).contains(&ch) {
            if dp.dwCount >= 6 {
                hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
                break;
            }
            dp.dwValues[dp.dwCount as usize] = wcstoul(strIn, &mut strIn as *mut *mut u16, 10);
            dp.dwCount += 1;
            strIn = strIn.offset(-1);
        } else if iswalpha(ch) != 0 {
            let mut bFound = false;
            for i in 0..TOKEN_COUNT {
                let dwLen = lstrlenW(tokens[i]) as u32;
                if dwLen > 0 && _wcsnicmp(strIn, tokens[i], dwLen) == 0 {
                    if i <= 25 {
                        if dp.dwCount >= 6 {
                            hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
                        } else {
                            dp.dwValues[dp.dwCount as usize] = PARSE_DATE_MONTHS[i] as u32;
                            dp.dwFlags[dp.dwCount as usize] |= DP_MONTH | DP_DATESEP;
                            dp.dwCount += 1;
                        }
                    } else if i > 39 && i < 42 {
                        if dp.dwCount == 0 || (dp.dwParseFlags & (DP_AM | DP_PM)) != 0 {
                            hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
                        } else {
                            let flag = if i == 40 { DP_AM } else { DP_PM };
                            dp.dwFlags[(dp.dwCount - 1) as usize] |= flag;
                            dp.dwParseFlags |= flag;
                        }
                    }
                    strIn = strIn.offset((dwLen - 1) as isize);
                    bFound = true;
                    break;
                }
            }

            if !bFound {
                let c = *strIn as u32;
                if (c == b'a' as u32 || c == b'A' as u32 || c == b'p' as u32 || c == b'P' as u32)
                    && dp.dwCount != 0
                    && (dp.dwParseFlags & (DP_AM | DP_PM)) == 0
                {
                    if c == b'a' as u32 || c == b'A' as u32 {
                        dp.dwFlags[(dp.dwCount - 1) as usize] |= DP_AM;
                        dp.dwParseFlags |= DP_AM;
                    } else {
                        dp.dwFlags[(dp.dwCount - 1) as usize] |= DP_PM;
                        dp.dwParseFlags |= DP_PM;
                    }
                    strIn = strIn.offset(1);
                } else {
                    TRACE!(
                        b"No matching token for %s\n\0".as_ptr() as *const i8,
                        debugstr_w(strIn),
                    );
                    hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
                    break;
                }
            }
        } else if *strIn == b':' as u16 || *strIn == b'.' as u16 {
            if dp.dwCount == 0 || *strIn.offset(1) == 0 {
                hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
            } else {
                // tokens[42] is LOCALE_SDATE index
                if *(tokens[42]) == *strIn {
                    dwDateSeps += 1;
                    if dwDateSeps > 2 {
                        hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
                    } else {
                        dp.dwFlags[(dp.dwCount - 1) as usize] |= DP_DATESEP;
                    }
                } else {
                    dp.dwFlags[(dp.dwCount - 1) as usize] |= DP_TIMESEP;
                }
            }
        } else if *strIn == b'-' as u16 || *strIn == b'/' as u16 {
            dwDateSeps += 1;
            if dwDateSeps > 2 || dp.dwCount == 0 || *strIn.offset(1) == 0 {
                hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
            } else {
                dp.dwFlags[(dp.dwCount - 1) as usize] |= DP_DATESEP;
            }
        } else if *strIn == b',' as u16 || iswspace(ch) != 0 {
            if *strIn == b',' as u16 && *strIn.offset(1) == 0 {
                hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
            }
        } else {
            hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
        }
        strIn = strIn.offset(1);
    }

    if dp.dwCount == 0
        || dp.dwCount > 6
        || (dp.dwCount == 1 && (dp.dwParseFlags & (DP_AM | DP_PM)) == 0)
    {
        hRet = DISP_E_TYPEMISMATCH; // DISP_E_TYPEMISMATCH
    }

    if hRet >= 0 {
        let mut st: SYSTEMTIME = core::mem::zeroed();
        let mut dwOffset: u32 = 0;

        // Compute TIMEFLAG values
        let timeflags = ((dp.dwFlags[0] & DP_TIMESEP) << 0)
            | ((dp.dwFlags[1] & DP_TIMESEP) << 1)
            | ((dp.dwFlags[2] & DP_TIMESEP) << 2)
            | ((dp.dwFlags[3] & DP_TIMESEP) << 3)
            | ((dp.dwFlags[4] & DP_TIMESEP) << 4);

        TRACE!(b"0x%08x\n\0".as_ptr() as *const i8, timeflags);

        match timeflags {
            0x1 => {
                // TT, TTDD, TTDDD
                if dp.dwCount > 3
                    && ((dp.dwFlags[2] & (DP_AM | DP_PM)) != 0
                        || (dp.dwFlags[3] & (DP_AM | DP_PM)) != 0
                        || (dp.dwFlags[4] & (DP_AM | DP_PM)) != 0)
                {
                    hRet = DISP_E_TYPEMISMATCH;
                } else if dp.dwCount != 2 && dp.dwCount != 4 && dp.dwCount != 5 {
                    hRet = DISP_E_TYPEMISMATCH;
                } else {
                    st.wHour = dp.dwValues[0] as u16;
                    st.wMinute = dp.dwValues[1] as u16;
                    dp.dwCount -= 2;
                    dwOffset = 2;
                }
            }
            0x3 => {
                // TTT, TTTDD, TTTDDD
                if dp.dwCount > 4
                    && ((dp.dwFlags[3] & (DP_AM | DP_PM)) != 0
                        || (dp.dwFlags[4] & (DP_AM | DP_PM)) != 0
                        || (dp.dwFlags[5] & (DP_AM | DP_PM)) != 0)
                {
                    hRet = DISP_E_TYPEMISMATCH;
                } else if dp.dwCount != 3 && dp.dwCount != 5 && dp.dwCount != 6 {
                    hRet = DISP_E_TYPEMISMATCH;
                } else {
                    st.wHour = dp.dwValues[0] as u16;
                    st.wMinute = dp.dwValues[1] as u16;
                    st.wSecond = dp.dwValues[2] as u16;
                    dwOffset = 3;
                    dp.dwCount -= 3;
                }
            }
            0x4 => {
                // DDTT
                if dp.dwCount != 4
                    || (dp.dwFlags[0] & (DP_AM | DP_PM)) != 0
                    || (dp.dwFlags[1] & (DP_AM | DP_PM)) != 0
                {
                    hRet = DISP_E_TYPEMISMATCH;
                } else {
                    st.wHour = dp.dwValues[2] as u16;
                    st.wMinute = dp.dwValues[3] as u16;
                    dp.dwCount -= 2;
                }
            }
            0x0 | 0x8 => {
                // 0x0: T, DD, DDD, TDDD, etc. 0x8: DDDTT
                if timeflags == 0x0 {
                    if dp.dwCount == 1 && (dp.dwParseFlags & (DP_AM | DP_PM)) != 0 {
                        st.wHour = dp.dwValues[0] as u16;
                        dp.dwCount = 0;
                    } else if dp.dwCount > 4
                        || (dp.dwCount < 3 && (dp.dwParseFlags & (DP_AM | DP_PM)) != 0)
                    {
                        hRet = DISP_E_TYPEMISMATCH;
                    } else if dp.dwCount == 3 {
                        if (dp.dwFlags[0] & (DP_AM | DP_PM)) != 0 {
                            // TDD
                            dp.dwCount = 2;
                            st.wHour = dp.dwValues[0] as u16;
                            dwOffset = 1;
                        } else if (dp.dwFlags[2] & (DP_AM | DP_PM)) != 0 {
                            // DDT
                            dp.dwCount = 2;
                            st.wHour = dp.dwValues[2] as u16;
                        } else if (dp.dwParseFlags & (DP_AM | DP_PM)) != 0 {
                            hRet = DISP_E_TYPEMISMATCH;
                        }
                    } else if dp.dwCount == 4 {
                        dp.dwCount = 3;
                        if (dp.dwFlags[0] & (DP_AM | DP_PM)) != 0 {
                            // TDDD
                            st.wHour = dp.dwValues[0] as u16;
                            dwOffset = 1;
                        } else if (dp.dwFlags[3] & (DP_AM | DP_PM)) != 0 {
                            // DDDT
                            st.wHour = dp.dwValues[3] as u16;
                        } else {
                            hRet = DISP_E_TYPEMISMATCH;
                        }
                        // fall through to 0x8 handling logic below (but we broke out)
                    }
                    // for dwCount == 2 or other cases without time separators,
                    // VARIANT_MakeDate will handle the date fields normally.
                    // No additional break, proceed to end of match.
                } else {
                    // 0x8: DDDTT
                    if (dp.dwCount == 2 && (dp.dwParseFlags & (DP_AM | DP_PM)) != 0)
                        || (dp.dwCount == 5
                            && ((dp.dwFlags[0] & (DP_AM | DP_PM)) != 0
                                || (dp.dwFlags[1] & (DP_AM | DP_PM)) != 0
                                || (dp.dwFlags[2] & (DP_AM | DP_PM)) != 0))
                        || dp.dwCount == 4
                        || dp.dwCount == 6
                    {
                        hRet = DISP_E_TYPEMISMATCH;
                    } else {
                        st.wHour = dp.dwValues[3] as u16;
                        st.wMinute = dp.dwValues[4] as u16;
                        if dp.dwCount == 5 {
                            dp.dwCount -= 2;
                        }
                    }
                }
            }
            0xC => {
                // DDTTT
                if dp.dwCount != 5
                    || (dp.dwFlags[0] & (DP_AM | DP_PM)) != 0
                    || (dp.dwFlags[1] & (DP_AM | DP_PM)) != 0
                {
                    hRet = DISP_E_TYPEMISMATCH;
                } else {
                    st.wHour = dp.dwValues[2] as u16;
                    st.wMinute = dp.dwValues[3] as u16;
                    st.wSecond = dp.dwValues[4] as u16;
                    dp.dwCount -= 3;
                }
            }
            0x18 => {
                // DDDTTT
                if (dp.dwFlags[0] & (DP_AM | DP_PM)) != 0
                    || (dp.dwFlags[1] & (DP_AM | DP_PM)) != 0
                    || (dp.dwFlags[2] & (DP_AM | DP_PM)) != 0
                {
                    hRet = DISP_E_TYPEMISMATCH;
                } else {
                    st.wHour = dp.dwValues[3] as u16;
                    st.wMinute = dp.dwValues[4] as u16;
                    st.wSecond = dp.dwValues[5] as u16;
                    dp.dwCount -= 3;
                }
            }
            _ => {
                hRet = DISP_E_TYPEMISMATCH;
            }
        }

        if hRet >= 0 {
            hRet = VARIANT_MakeDate(&mut dp, iDate, dwOffset, &mut st);

            if (dwFlags & VAR_TIMEVALUEONLY) != 0 {
                st.wYear = 1899;
                st.wMonth = 12;
                st.wDay = 30;
            } else if (dwFlags & VAR_DATEVALUEONLY) != 0 {
                st.wHour = 0;
                st.wMinute = 0;
                st.wSecond = 0;
            }

            // Finally, convert the value to a VT_DATE
            if hRet >= 0 {
                hRet = if SystemTimeToVariantTime(&mut st, pdateOut) != 0 {
                    0 // S_OK
                } else {
                    DISP_E_TYPEMISMATCH // DISP_E_TYPEMISMATCH
                };
            }
        }
    }

    // Free allocated tokens
    for i in 0..TOKEN_COUNT {
        SysFreeString(tokens[i]);
    }

    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromI1(cIn: i8, pdateOut: *mut f64) -> HRESULT {
    VarR8FromI1(cIn, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromUI2(uiIn: u16, pdateOut: *mut f64) -> i32 {
    VarR8FromUI2(uiIn, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromUI4(ulIn: u32, pdateOut: *mut f64) -> HRESULT {
    VarDateFromR8(ulIn as f64, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromDec(pdecIn: *mut DECIMAL, pdateOut: *mut DATE) -> HRESULT {
    VarR8FromDec(pdecIn, pdateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromI8(llIn: LONG64, pdateOut: *mut DATE) -> HRESULT {
    if llIn < DATE_MIN || llIn > DATE_MAX {
        return DISP_E_OVERFLOW;
    }
    *pdateOut = llIn as DATE;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromUI8(ullIn: u64, pdateOut: *mut f64) -> HRESULT {
    if ullIn > DATE_MAX as u64 {
        return DISP_E_OVERFLOW;
    }
    *pdateOut = ullIn as f64;
    S_OK
}

// --------------------------------------------------------------------------
// The _Var* inline conversion family, generated faithfully from the C macro
// table (vartype.c:150-262, SIMPLE/NEGTST/POSTST/BOTHTST). The C compares
// use usual-arithmetic-conversions; every instance is integer<=64bit, so an
// i128 comparison reproduces each C result exactly (verified per instance:
// all bounds are non-negative or compared signed-vs-signed in C).
// --------------------------------------------------------------------------
/// vartype.c POSTST(signed char, BYTE, VarI1FromUI1, I1_MAX)
#[inline]
pub unsafe fn _VarI1FromUI1(in_: u8, out: *mut i8) -> HRESULT {
    if (in_ as i128) > 127i128 /* (signed char)I1_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i8;
    S_OK
}
/// vartype.c BOTHTST(signed char, SHORT, VarI1FromI2, I1_MIN, I1_MAX)
#[inline]
pub unsafe fn _VarI1FromI2(in_: i16, out: *mut i8) -> HRESULT {
    if (in_ as i128) < -128i128 /* (signed char)I1_MIN */ || (in_ as i128) > 127i128 /* I1_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as i8;
    S_OK
}
/// vartype.c BOTHTST(signed char, LONG, VarI1FromI4, I1_MIN, I1_MAX)
#[inline]
pub unsafe fn _VarI1FromI4(in_: i32, out: *mut i8) -> HRESULT {
    if (in_ as i128) < -128i128 /* (signed char)I1_MIN */ || (in_ as i128) > 127i128 /* I1_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as i8;
    S_OK
}
/// vartype.c SIMPLE(signed char, VARIANT_BOOL, VarI1FromBool)
#[inline]
pub unsafe fn _VarI1FromBool(in_: i16, out: *mut i8) -> HRESULT {
    *out = in_ as i8;
    S_OK
}
/// vartype.c POSTST(signed char, USHORT, VarI1FromUI2, I1_MAX)
#[inline]
pub unsafe fn _VarI1FromUI2(in_: u16, out: *mut i8) -> HRESULT {
    if (in_ as i128) > 127i128 /* (signed char)I1_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i8;
    S_OK
}
/// vartype.c POSTST(signed char, ULONG, VarI1FromUI4, I1_MAX)
#[inline]
pub unsafe fn _VarI1FromUI4(in_: u32, out: *mut i8) -> HRESULT {
    if (in_ as i128) > 127i128 /* (signed char)I1_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i8;
    S_OK
}
/// vartype.c BOTHTST(signed char, LONG64, VarI1FromI8, I1_MIN, I1_MAX)
#[inline]
pub unsafe fn _VarI1FromI8(in_: i64, out: *mut i8) -> HRESULT {
    if (in_ as i128) < -128i128 /* (signed char)I1_MIN */ || (in_ as i128) > 127i128 /* I1_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as i8;
    S_OK
}
/// vartype.c POSTST(signed char, ULONG64, VarI1FromUI8, I1_MAX)
#[inline]
pub unsafe fn _VarI1FromUI8(in_: u64, out: *mut i8) -> HRESULT {
    if (in_ as i128) > 127i128 /* (signed char)I1_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i8;
    S_OK
}
/// vartype.c BOTHTST(BYTE, SHORT, VarUI1FromI2, UI1_MIN, UI1_MAX)
#[inline]
pub unsafe fn _VarUI1FromI2(in_: i16, out: *mut u8) -> HRESULT {
    if (in_ as i128) < 0i128 /* (BYTE)UI1_MIN */ || (in_ as i128) > 255i128 /* UI1_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as u8;
    S_OK
}
/// vartype.c SIMPLE(BYTE, VARIANT_BOOL, VarUI1FromBool)
#[inline]
pub unsafe fn _VarUI1FromBool(in_: i16, out: *mut u8) -> HRESULT {
    *out = in_ as u8;
    S_OK
}
/// vartype.c NEGTST(BYTE, signed char, VarUI1FromI1)
#[inline]
pub unsafe fn _VarUI1FromI1(in_: i8, out: *mut u8) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u8;
    S_OK
}
/// vartype.c POSTST(BYTE, USHORT, VarUI1FromUI2, UI1_MAX)
#[inline]
pub unsafe fn _VarUI1FromUI2(in_: u16, out: *mut u8) -> HRESULT {
    if (in_ as i128) > 255i128 /* (BYTE)UI1_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as u8;
    S_OK
}
/// vartype.c BOTHTST(BYTE, LONG, VarUI1FromI4, UI1_MIN, UI1_MAX)
#[inline]
pub unsafe fn _VarUI1FromI4(in_: i32, out: *mut u8) -> HRESULT {
    if (in_ as i128) < 0i128 /* (BYTE)UI1_MIN */ || (in_ as i128) > 255i128 /* UI1_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as u8;
    S_OK
}
/// vartype.c POSTST(BYTE, ULONG, VarUI1FromUI4, UI1_MAX)
#[inline]
pub unsafe fn _VarUI1FromUI4(in_: u32, out: *mut u8) -> HRESULT {
    if (in_ as i128) > 255i128 /* (BYTE)UI1_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as u8;
    S_OK
}
/// vartype.c BOTHTST(BYTE, LONG64, VarUI1FromI8, UI1_MIN, UI1_MAX)
#[inline]
pub unsafe fn _VarUI1FromI8(in_: i64, out: *mut u8) -> HRESULT {
    if (in_ as i128) < 0i128 /* (BYTE)UI1_MIN */ || (in_ as i128) > 255i128 /* UI1_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as u8;
    S_OK
}
/// vartype.c POSTST(BYTE, ULONG64, VarUI1FromUI8, UI1_MAX)
#[inline]
pub unsafe fn _VarUI1FromUI8(in_: u64, out: *mut u8) -> HRESULT {
    if (in_ as i128) > 255i128 /* (BYTE)UI1_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as u8;
    S_OK
}
/// vartype.c SIMPLE(SHORT, BYTE, VarI2FromUI1)
#[inline]
pub unsafe fn _VarI2FromUI1(in_: u8, out: *mut i16) -> HRESULT {
    *out = in_ as i16;
    S_OK
}
/// vartype.c BOTHTST(SHORT, LONG, VarI2FromI4, I2_MIN, I2_MAX)
#[inline]
pub unsafe fn _VarI2FromI4(in_: i32, out: *mut i16) -> HRESULT {
    if (in_ as i128) < -32768i128 /* (SHORT)I2_MIN */ || (in_ as i128) > 32767i128 /* I2_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as i16;
    S_OK
}
/// vartype.c SIMPLE(SHORT, VARIANT_BOOL, VarI2FromBool)
#[inline]
pub unsafe fn _VarI2FromBool(in_: i16, out: *mut i16) -> HRESULT {
    *out = in_ as i16;
    S_OK
}
/// vartype.c SIMPLE(SHORT, signed char, VarI2FromI1)
#[inline]
pub unsafe fn _VarI2FromI1(in_: i8, out: *mut i16) -> HRESULT {
    *out = in_ as i16;
    S_OK
}
/// vartype.c POSTST(SHORT, USHORT, VarI2FromUI2, I2_MAX)
#[inline]
pub unsafe fn _VarI2FromUI2(in_: u16, out: *mut i16) -> HRESULT {
    if (in_ as i128) > 32767i128 /* (SHORT)I2_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i16;
    S_OK
}
/// vartype.c POSTST(SHORT, ULONG, VarI2FromUI4, I2_MAX)
#[inline]
pub unsafe fn _VarI2FromUI4(in_: u32, out: *mut i16) -> HRESULT {
    if (in_ as i128) > 32767i128 /* (SHORT)I2_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i16;
    S_OK
}
/// vartype.c BOTHTST(SHORT, LONG64, VarI2FromI8, I2_MIN, I2_MAX)
#[inline]
pub unsafe fn _VarI2FromI8(in_: i64, out: *mut i16) -> HRESULT {
    if (in_ as i128) < -32768i128 /* (SHORT)I2_MIN */ || (in_ as i128) > 32767i128 /* I2_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as i16;
    S_OK
}
/// vartype.c POSTST(SHORT, ULONG64, VarI2FromUI8, I2_MAX)
#[inline]
pub unsafe fn _VarI2FromUI8(in_: u64, out: *mut i16) -> HRESULT {
    if (in_ as i128) > 32767i128 /* (SHORT)I2_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i16;
    S_OK
}
/// vartype.c SIMPLE(USHORT, BYTE, VarUI2FromUI1)
#[inline]
pub unsafe fn _VarUI2FromUI1(in_: u8, out: *mut u16) -> HRESULT {
    *out = in_ as u16;
    S_OK
}
/// vartype.c NEGTST(USHORT, SHORT, VarUI2FromI2)
#[inline]
pub unsafe fn _VarUI2FromI2(in_: i16, out: *mut u16) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u16;
    S_OK
}
/// vartype.c BOTHTST(USHORT, LONG, VarUI2FromI4, UI2_MIN, UI2_MAX)
#[inline]
pub unsafe fn _VarUI2FromI4(in_: i32, out: *mut u16) -> HRESULT {
    if (in_ as i128) < 0i128 /* (USHORT)UI2_MIN */ || (in_ as i128) > 65535i128 /* UI2_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as u16;
    S_OK
}
/// vartype.c SIMPLE(USHORT, VARIANT_BOOL, VarUI2FromBool)
#[inline]
pub unsafe fn _VarUI2FromBool(in_: i16, out: *mut u16) -> HRESULT {
    *out = in_ as u16;
    S_OK
}
/// vartype.c NEGTST(USHORT, signed char, VarUI2FromI1)
#[inline]
pub unsafe fn _VarUI2FromI1(in_: i8, out: *mut u16) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u16;
    S_OK
}
/// vartype.c POSTST(USHORT, ULONG, VarUI2FromUI4, UI2_MAX)
#[inline]
pub unsafe fn _VarUI2FromUI4(in_: u32, out: *mut u16) -> HRESULT {
    if (in_ as i128) > 65535i128 /* (USHORT)UI2_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as u16;
    S_OK
}
/// vartype.c BOTHTST(USHORT, LONG64, VarUI2FromI8, UI2_MIN, UI2_MAX)
#[inline]
pub unsafe fn _VarUI2FromI8(in_: i64, out: *mut u16) -> HRESULT {
    if (in_ as i128) < 0i128 /* (USHORT)UI2_MIN */ || (in_ as i128) > 65535i128 /* UI2_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as u16;
    S_OK
}
/// vartype.c POSTST(USHORT, ULONG64, VarUI2FromUI8, UI2_MAX)
#[inline]
pub unsafe fn _VarUI2FromUI8(in_: u64, out: *mut u16) -> HRESULT {
    if (in_ as i128) > 65535i128 /* (USHORT)UI2_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as u16;
    S_OK
}
/// vartype.c SIMPLE(LONG, BYTE, VarI4FromUI1)
#[inline]
pub unsafe fn _VarI4FromUI1(in_: u8, out: *mut i32) -> HRESULT {
    *out = in_ as i32;
    S_OK
}
/// vartype.c SIMPLE(LONG, SHORT, VarI4FromI2)
#[inline]
pub unsafe fn _VarI4FromI2(in_: i16, out: *mut i32) -> HRESULT {
    *out = in_ as i32;
    S_OK
}
/// vartype.c SIMPLE(LONG, VARIANT_BOOL, VarI4FromBool)
#[inline]
pub unsafe fn _VarI4FromBool(in_: i16, out: *mut i32) -> HRESULT {
    *out = in_ as i32;
    S_OK
}
/// vartype.c SIMPLE(LONG, signed char, VarI4FromI1)
#[inline]
pub unsafe fn _VarI4FromI1(in_: i8, out: *mut i32) -> HRESULT {
    *out = in_ as i32;
    S_OK
}
/// vartype.c SIMPLE(LONG, USHORT, VarI4FromUI2)
#[inline]
pub unsafe fn _VarI4FromUI2(in_: u16, out: *mut i32) -> HRESULT {
    *out = in_ as i32;
    S_OK
}
/// vartype.c POSTST(LONG, ULONG, VarI4FromUI4, I4_MAX)
#[inline]
pub unsafe fn _VarI4FromUI4(in_: u32, out: *mut i32) -> HRESULT {
    if (in_ as i128) > 2147483647i128 /* (LONG)I4_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i32;
    S_OK
}
/// vartype.c BOTHTST(LONG, LONG64, VarI4FromI8, I4_MIN, I4_MAX)
#[inline]
pub unsafe fn _VarI4FromI8(in_: i64, out: *mut i32) -> HRESULT {
    if (in_ as i128) < -2147483648i128 /* (LONG)I4_MIN */ || (in_ as i128) > 2147483647i128 /* I4_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as i32;
    S_OK
}
/// vartype.c POSTST(LONG, ULONG64, VarI4FromUI8, I4_MAX)
#[inline]
pub unsafe fn _VarI4FromUI8(in_: u64, out: *mut i32) -> HRESULT {
    if (in_ as i128) > 2147483647i128 /* (LONG)I4_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i32;
    S_OK
}
/// vartype.c SIMPLE(ULONG, BYTE, VarUI4FromUI1)
#[inline]
pub unsafe fn _VarUI4FromUI1(in_: u8, out: *mut u32) -> HRESULT {
    *out = in_ as u32;
    S_OK
}
/// vartype.c NEGTST(ULONG, SHORT, VarUI4FromI2)
#[inline]
pub unsafe fn _VarUI4FromI2(in_: i16, out: *mut u32) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u32;
    S_OK
}
/// vartype.c NEGTST(ULONG, LONG, VarUI4FromI4)
#[inline]
pub unsafe fn _VarUI4FromI4(in_: i32, out: *mut u32) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u32;
    S_OK
}
/// vartype.c SIMPLE(ULONG, VARIANT_BOOL, VarUI4FromBool)
#[inline]
pub unsafe fn _VarUI4FromBool(in_: i16, out: *mut u32) -> HRESULT {
    *out = in_ as u32;
    S_OK
}
/// vartype.c NEGTST(ULONG, signed char, VarUI4FromI1)
#[inline]
pub unsafe fn _VarUI4FromI1(in_: i8, out: *mut u32) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u32;
    S_OK
}
/// vartype.c SIMPLE(ULONG, USHORT, VarUI4FromUI2)
#[inline]
pub unsafe fn _VarUI4FromUI2(in_: u16, out: *mut u32) -> HRESULT {
    *out = in_ as u32;
    S_OK
}
/// vartype.c BOTHTST(ULONG, LONG64, VarUI4FromI8, UI4_MIN, UI4_MAX)
#[inline]
pub unsafe fn _VarUI4FromI8(in_: i64, out: *mut u32) -> HRESULT {
    if (in_ as i128) < 0i128 /* (ULONG)UI4_MIN */ || (in_ as i128) > 4294967295i128 /* UI4_MAX */ {
        return DISP_E_OVERFLOW;
    }
    *out = in_ as u32;
    S_OK
}
/// vartype.c POSTST(ULONG, ULONG64, VarUI4FromUI8, UI4_MAX)
#[inline]
pub unsafe fn _VarUI4FromUI8(in_: u64, out: *mut u32) -> HRESULT {
    if (in_ as i128) > 4294967295i128 /* (ULONG)UI4_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as u32;
    S_OK
}
/// vartype.c SIMPLE(LONG64, BYTE, VarI8FromUI1)
#[inline]
pub unsafe fn _VarI8FromUI1(in_: u8, out: *mut i64) -> HRESULT {
    *out = in_ as i64;
    S_OK
}
/// vartype.c SIMPLE(LONG64, SHORT, VarI8FromI2)
#[inline]
pub unsafe fn _VarI8FromI2(in_: i16, out: *mut i64) -> HRESULT {
    *out = in_ as i64;
    S_OK
}
/// vartype.c SIMPLE(LONG64, signed char, VarI8FromI1)
#[inline]
pub unsafe fn _VarI8FromI1(in_: i8, out: *mut i64) -> HRESULT {
    *out = in_ as i64;
    S_OK
}
/// vartype.c SIMPLE(LONG64, USHORT, VarI8FromUI2)
#[inline]
pub unsafe fn _VarI8FromUI2(in_: u16, out: *mut i64) -> HRESULT {
    *out = in_ as i64;
    S_OK
}
/// vartype.c SIMPLE(LONG64, ULONG, VarI8FromUI4)
#[inline]
pub unsafe fn _VarI8FromUI4(in_: u32, out: *mut i64) -> HRESULT {
    *out = in_ as i64;
    S_OK
}
/// vartype.c POSTST(LONG64, ULONG64, VarI8FromUI8, I8_MAX)
#[inline]
pub unsafe fn _VarI8FromUI8(in_: u64, out: *mut i64) -> HRESULT {
    if (in_ as i128) > 9223372036854775807i128 /* (LONG64)I8_MAX */ { return DISP_E_OVERFLOW; }
    *out = in_ as i64;
    S_OK
}
/// vartype.c SIMPLE(ULONG64, BYTE, VarUI8FromUI1)
#[inline]
pub unsafe fn _VarUI8FromUI1(in_: u8, out: *mut u64) -> HRESULT {
    *out = in_ as u64;
    S_OK
}
/// vartype.c NEGTST(ULONG64, SHORT, VarUI8FromI2)
#[inline]
pub unsafe fn _VarUI8FromI2(in_: i16, out: *mut u64) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u64;
    S_OK
}
/// vartype.c NEGTST(ULONG64, signed char, VarUI8FromI1)
#[inline]
pub unsafe fn _VarUI8FromI1(in_: i8, out: *mut u64) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u64;
    S_OK
}
/// vartype.c SIMPLE(ULONG64, USHORT, VarUI8FromUI2)
#[inline]
pub unsafe fn _VarUI8FromUI2(in_: u16, out: *mut u64) -> HRESULT {
    *out = in_ as u64;
    S_OK
}
/// vartype.c SIMPLE(ULONG64, ULONG, VarUI8FromUI4)
#[inline]
pub unsafe fn _VarUI8FromUI4(in_: u32, out: *mut u64) -> HRESULT {
    *out = in_ as u64;
    S_OK
}
/// vartype.c NEGTST(ULONG64, LONG64, VarUI8FromI8)
#[inline]
pub unsafe fn _VarUI8FromI8(in_: i64, out: *mut u64) -> HRESULT {
    if in_ < 0 { return DISP_E_OVERFLOW; }
    *out = in_ as u64;
    S_OK
}
/// vartype.c SIMPLE(float, BYTE, VarR4FromUI1)
#[inline]
pub unsafe fn _VarR4FromUI1(in_: u8, out: *mut f32) -> HRESULT {
    *out = in_ as f32;
    S_OK
}
/// vartype.c SIMPLE(float, SHORT, VarR4FromI2)
#[inline]
pub unsafe fn _VarR4FromI2(in_: i16, out: *mut f32) -> HRESULT {
    *out = in_ as f32;
    S_OK
}
/// vartype.c SIMPLE(float, signed char, VarR4FromI1)
#[inline]
pub unsafe fn _VarR4FromI1(in_: i8, out: *mut f32) -> HRESULT {
    *out = in_ as f32;
    S_OK
}
/// vartype.c SIMPLE(float, USHORT, VarR4FromUI2)
#[inline]
pub unsafe fn _VarR4FromUI2(in_: u16, out: *mut f32) -> HRESULT {
    *out = in_ as f32;
    S_OK
}
/// vartype.c SIMPLE(float, LONG, VarR4FromI4)
#[inline]
pub unsafe fn _VarR4FromI4(in_: i32, out: *mut f32) -> HRESULT {
    *out = in_ as f32;
    S_OK
}
/// vartype.c SIMPLE(float, ULONG, VarR4FromUI4)
#[inline]
pub unsafe fn _VarR4FromUI4(in_: u32, out: *mut f32) -> HRESULT {
    *out = in_ as f32;
    S_OK
}
/// vartype.c SIMPLE(float, LONG64, VarR4FromI8)
#[inline]
pub unsafe fn _VarR4FromI8(in_: i64, out: *mut f32) -> HRESULT {
    *out = in_ as f32;
    S_OK
}
/// vartype.c SIMPLE(float, ULONG64, VarR4FromUI8)
#[inline]
pub unsafe fn _VarR4FromUI8(in_: u64, out: *mut f32) -> HRESULT {
    *out = in_ as f32;
    S_OK
}
/// vartype.c SIMPLE(double, BYTE, VarR8FromUI1)
#[inline]
pub unsafe fn _VarR8FromUI1(in_: u8, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, SHORT, VarR8FromI2)
#[inline]
pub unsafe fn _VarR8FromI2(in_: i16, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, float, VarR8FromR4)
#[inline]
pub unsafe fn _VarR8FromR4(in_: f32, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, DATE, VarR8FromDate)
#[inline]
pub unsafe fn _VarR8FromDate(in_: f64, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, signed char, VarR8FromI1)
#[inline]
pub unsafe fn _VarR8FromI1(in_: i8, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, USHORT, VarR8FromUI2)
#[inline]
pub unsafe fn _VarR8FromUI2(in_: u16, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, LONG, VarR8FromI4)
#[inline]
pub unsafe fn _VarR8FromI4(in_: i32, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, ULONG, VarR8FromUI4)
#[inline]
pub unsafe fn _VarR8FromUI4(in_: u32, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, LONG64, VarR8FromI8)
#[inline]
pub unsafe fn _VarR8FromI8(in_: i64, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
/// vartype.c SIMPLE(double, ULONG64, VarR8FromUI8)
#[inline]
pub unsafe fn _VarR8FromUI8(in_: u64, out: *mut f64) -> HRESULT {
    *out = in_ as f64;
    S_OK
}
