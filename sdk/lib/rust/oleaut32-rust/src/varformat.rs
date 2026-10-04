use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32;
    fn wcstod(nptr: *const u16, endptr: *mut *mut u16) -> f64;
    fn wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn bsearch( key: *const core::ffi::c_void, base: *const core::ffi::c_void, num: usize, size: usize, compar: unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32, ) -> *mut core::ffi::c_void;
    fn _wcsnicmp(s1: LPCWSTR, s2: LPCWSTR, n: usize) -> i32;
    fn wcsncmp(s1: LPCWSTR, s2: LPCWSTR, n: usize) -> i32;
}
extern "system" {
    fn GetCurrencyFormatW( Locale: LCID, dwFlags: u32, lpValue: LPCWSTR, lpFormat: *const CURRENCYFMTW, lpCurrencyStr: LPWSTR, cchCurrency: i32, ) -> i32;
    fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
}

// ---- file-scope preamble (translated) ----
pub const LCID_US: LCID = 0x0409;

pub static szPercent_d: [u16; 3] = [b'%' as u16, b'd' as u16, 0];
pub static szPercentZeroTwo_d: [u16; 5] = [b'%' as u16, b'0' as u16, b'2' as u16, b'd' as u16, 0];
pub static szPercentZeroStar_d: [u16; 5] = [b'%' as u16, b'0' as u16, b'*' as u16, b'd' as u16, 0];

pub const FMT_TYPE_UNKNOWN: u8 = 0x0;
pub const FMT_TYPE_GENERAL: u8 = 0x1;
pub const FMT_TYPE_NUMBER: u8 = 0x2;
pub const FMT_TYPE_DATE: u8 = 0x3;
pub const FMT_TYPE_STRING: u8 = 0x4;

pub const FMT_TO_STRING: u8 = 0x0;

#[repr(C)]
pub struct FMT_SHORT_HEADER {
    pub size: u8,
    pub type_: u8,
    pub offset: [u8; 1],
}
pub type PFMT_SHORT_HEADER = *mut FMT_SHORT_HEADER;

#[repr(C)]
pub struct FMT_HEADER {
    pub size: u8,
    pub type_: u8,
    pub starts: [u8; 4],
}
pub type PFMT_HEADER = *mut FMT_HEADER;

pub const FMT_FLAG_LT: u8 = 0x1;
pub const FMT_FLAG_GT: u8 = 0x2;
pub const FMT_FLAG_RTL: u8 = 0x4;

#[repr(C)]
pub struct FMT_STRING_HEADER {
    pub flags: u8,
    pub unknown1: u8,
    pub unknown2: u8,
    pub copy_chars: u8,
    pub unknown3: u8,
}
pub type PFMT_STRING_HEADER = *mut FMT_STRING_HEADER;

pub const FMT_FLAG_PERCENT: u8 = 0x1;
pub const FMT_FLAG_EXPONENT: u8 = 0x2;
pub const FMT_FLAG_THOUSANDS: u8 = 0x4;
pub const FMT_FLAG_BOOL: u8 = 0x20;

#[repr(C)]
pub struct FMT_NUMBER_HEADER {
    pub flags: u8,
    pub multiplier: u8,
    pub divisor: u8,
    pub whole: u8,
    pub fractional: u8,
}
pub type PFMT_NUMBER_HEADER = *mut FMT_NUMBER_HEADER;

#[repr(C)]
pub struct FMT_DATE_HEADER {
    pub flags: u8,
    pub unknown1: u8,
    pub unknown2: u8,
    pub unknown3: u8,
    pub unknown4: u8,
}
pub type PFMT_DATE_HEADER = *mut FMT_DATE_HEADER;

pub const FMT_GEN_COPY: u8 = 0x00;
pub const FMT_GEN_INLINE: u8 = 0x01;
pub const FMT_GEN_END: u8 = 0x02;
pub const FMT_DATE_TIME_SEP: u8 = 0x03;
pub const FMT_DATE_DATE_SEP: u8 = 0x04;
pub const FMT_DATE_GENERAL: u8 = 0x05;
pub const FMT_DATE_QUARTER: u8 = 0x06;
pub const FMT_DATE_TIME_SYS: u8 = 0x07;
pub const FMT_DATE_DAY: u8 = 0x08;
pub const FMT_DATE_DAY_0: u8 = 0x09;
pub const FMT_DATE_DAY_SHORT: u8 = 0x0A;
pub const FMT_DATE_DAY_LONG: u8 = 0x0B;
pub const FMT_DATE_SHORT: u8 = 0x0C;
pub const FMT_DATE_LONG: u8 = 0x0D;
pub const FMT_DATE_MEDIUM: u8 = 0x0E;
pub const FMT_DATE_DAY_WEEK: u8 = 0x0F;
pub const FMT_DATE_WEEK_YEAR: u8 = 0x10;
pub const FMT_DATE_MON: u8 = 0x11;
pub const FMT_DATE_MON_0: u8 = 0x12;
pub const FMT_DATE_MON_SHORT: u8 = 0x13;
pub const FMT_DATE_MON_LONG: u8 = 0x14;
pub const FMT_DATE_YEAR_DOY: u8 = 0x15;
pub const FMT_DATE_YEAR_0: u8 = 0x16;
pub const FMT_DATE_YEAR_LONG: u8 = 0x18;
pub const FMT_DATE_MIN: u8 = 0x1A;
pub const FMT_DATE_MIN_0: u8 = 0x1B;
pub const FMT_DATE_SEC: u8 = 0x1C;
pub const FMT_DATE_SEC_0: u8 = 0x1D;
pub const FMT_DATE_HOUR: u8 = 0x1E;
pub const FMT_DATE_HOUR_0: u8 = 0x1F;
pub const FMT_DATE_HOUR_12: u8 = 0x20;
pub const FMT_DATE_HOUR_12_0: u8 = 0x21;
pub const FMT_DATE_TIME_UNK2: u8 = 0x23;
pub const FMT_DATE_AMPM_SYS1: u8 = 0x2E;
pub const FMT_DATE_AMPM_UPPER: u8 = 0x2F;
pub const FMT_DATE_A_UPPER: u8 = 0x30;
pub const FMT_DATE_AMPM_SYS2: u8 = 0x31;
pub const FMT_DATE_AMPM_LOWER: u8 = 0x32;
pub const FMT_DATE_A_LOWER: u8 = 0x33;
pub const FMT_NUM_COPY_ZERO: u8 = 0x34;
pub const FMT_NUM_COPY_SKIP: u8 = 0x35;
pub const FMT_NUM_DECIMAL: u8 = 0x36;
pub const FMT_NUM_EXP_POS_U: u8 = 0x37;
pub const FMT_NUM_EXP_NEG_U: u8 = 0x38;
pub const FMT_NUM_EXP_POS_L: u8 = 0x39;
pub const FMT_NUM_EXP_NEG_L: u8 = 0x3A;
pub const FMT_NUM_CURRENCY: u8 = 0x3B;
pub const FMT_NUM_TRUE_FALSE: u8 = 0x3D;
pub const FMT_NUM_YES_NO: u8 = 0x3E;
pub const FMT_NUM_ON_OFF: u8 = 0x3F;
pub const FMT_STR_COPY_SPACE: u8 = 0x40;
pub const FMT_STR_COPY_SKIP: u8 = 0x41;

pub static szGeneralDate: [u16; 13] = [b'G' as u16, b'e' as u16, b'n' as u16, b'e' as u16, b'r' as u16, b'a' as u16, b'l' as u16, b' ' as u16, b'D' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0];
pub static fmtGeneralDate: [u8; 0x0a] = [
    0x0a, FMT_TYPE_DATE, 3,
    0x0, 0x0, 0x0, 0x0, 0x0,
    FMT_DATE_GENERAL, FMT_GEN_END
];

pub static szShortDate: [u16; 11] = [b'S' as u16, b'h' as u16, b'o' as u16, b'r' as u16, b't' as u16, b' ' as u16, b'D' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0];
pub static fmtShortDate: [u8; 0x0a] = [
    0x0a, FMT_TYPE_DATE, 3,
    0x0, 0x0, 0x0, 0x0, 0x0,
    FMT_DATE_SHORT, FMT_GEN_END
];

pub static szMediumDate: [u16; 12] = [b'M' as u16, b'e' as u16, b'd' as u16, b'i' as u16, b'u' as u16, b'm' as u16, b' ' as u16, b'D' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0];
pub static fmtMediumDate: [u8; 0x0a] = [
    0x0a, FMT_TYPE_DATE, 3,
    0x0, 0x0, 0x0, 0x0, 0x0,
    FMT_DATE_MEDIUM, FMT_GEN_END
];

pub static szLongDate: [u16; 10] = [b'L' as u16, b'o' as u16, b'n' as u16, b'g' as u16, b' ' as u16, b'D' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0];
pub static fmtLongDate: [u8; 0x0a] = [
    0x0a, FMT_TYPE_DATE, 3,
    0x0, 0x0, 0x0, 0x0, 0x0,
    FMT_DATE_LONG, FMT_GEN_END
];

pub static szShortTime: [u16; 11] = [b'S' as u16, b'h' as u16, b'o' as u16, b'r' as u16, b't' as u16, b' ' as u16, b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16, 0];
pub static fmtShortTime: [u8; 0x0c] = [
    0x0c, FMT_TYPE_DATE, 3,
    0x0, 0x0, 0x0, 0x0, 0x0,
    FMT_DATE_TIME_UNK2, FMT_DATE_TIME_SEP, FMT_DATE_MIN_0, FMT_GEN_END
];

pub static szMediumTime: [u16; 12] = [b'M' as u16, b'e' as u16, b'd' as u16, b'i' as u16, b'u' as u16, b'm' as u16, b' ' as u16, b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16, 0];
pub static fmtMediumTime: [u8; 0x11] = [
    0x11, FMT_TYPE_DATE, 3,
    0x0, 0x0, 0x0, 0x0, 0x0,
    FMT_DATE_HOUR_12_0, FMT_DATE_TIME_SEP, FMT_DATE_MIN_0,
    FMT_GEN_INLINE, 0x01, b' ', 0, FMT_DATE_AMPM_SYS1, FMT_GEN_END
];

pub static szLongTime: [u16; 10] = [b'L' as u16, b'o' as u16, b'n' as u16, b'g' as u16, b' ' as u16, b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16, 0];
// The C declares [0x0d] but writes only 10 bytes and lets the compiler zero-fill
// the tail; Rust has no such rule, so the three trailing zeros must be spelled.
// (The translation copied the 10 written bytes and kept the [u8; 0x0d] type —
// caught by rustconv's table_audit, which is the only oracle on this file:
// c2rust panics on varformat.c's AST, so there is no compiler-exact anchor.)
pub static fmtLongTime: [u8; 0x0d] = [
    0x0a, FMT_TYPE_DATE, 3,
    0x0, 0x0, 0x0, 0x0, 0x0,
    FMT_DATE_TIME_SYS, FMT_GEN_END,
    0x0, 0x0, 0x0
];

pub static szTrueFalse: [u16; 11] = [b'T' as u16, b'r' as u16, b'u' as u16, b'e' as u16, b'/' as u16, b'F' as u16, b'a' as u16, b'l' as u16, b's' as u16, b'e' as u16, 0];
pub static fmtTrueFalse: [u8; 0x0d] = [
    0x0d, FMT_TYPE_NUMBER, 6, 0x0, 0x0, 0x0,
    FMT_FLAG_BOOL, 0x0, 0x0, 0x0, 0x0,
    FMT_NUM_TRUE_FALSE, FMT_GEN_END
];

pub static szYesNo: [u16; 7] = [b'Y' as u16, b'e' as u16, b's' as u16, b'/' as u16, b'N' as u16, b'o' as u16, 0];
pub static fmtYesNo: [u8; 0x0d] = [
    0x0d, FMT_TYPE_NUMBER, 6, 0x0, 0x0, 0x0,
    FMT_FLAG_BOOL, 0x0, 0x0, 0x0, 0x0,
    FMT_NUM_YES_NO, FMT_GEN_END
];

pub static szOnOff: [u16; 7] = [b'O' as u16, b'n' as u16, b'/' as u16, b'O' as u16, b'f' as u16, b'f' as u16, 0];
pub static fmtOnOff: [u8; 0x0d] = [
    0x0d, FMT_TYPE_NUMBER, 6, 0x0, 0x0, 0x0,
    FMT_FLAG_BOOL, 0x0, 0x0, 0x0, 0x0,
    FMT_NUM_ON_OFF, FMT_GEN_END
];

pub static szGeneralNumber: [u16; 15] = [b'G' as u16, b'e' as u16, b'n' as u16, b'e' as u16, b'r' as u16, b'a' as u16, b'l' as u16, b' ' as u16, b'N' as u16, b'u' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'r' as u16, 0];
pub static fmtGeneralNumber: [u8; 6] = [
    6, FMT_TYPE_GENERAL, 6, 0x0, 0x0, 0x0
];

pub static szCurrency: [u16; 9] = [b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b'c' as u16, b'y' as u16, 0];
pub static fmtCurrency: [u8; 0x26] = [
    0x26, FMT_TYPE_NUMBER, 6, 0x12, 0x0, 0x0,
    FMT_FLAG_THOUSANDS, 0xcc, 0x0, 0x1, 0x2,
    FMT_NUM_CURRENCY, FMT_NUM_COPY_ZERO, 0x1, FMT_NUM_DECIMAL, FMT_NUM_COPY_ZERO, 0x2,
    FMT_GEN_END,
    FMT_FLAG_THOUSANDS, 0xcc, 0x0, 0x1, 0x2,
    FMT_GEN_INLINE, 0x1, b'(', 0, FMT_NUM_CURRENCY, FMT_NUM_COPY_ZERO, 0x1,
    FMT_NUM_DECIMAL, FMT_NUM_COPY_ZERO, 0x2, FMT_GEN_INLINE, 0x1, b')', 0,
    FMT_GEN_END
];

pub static szFixed: [u16; 6] = [b'F' as u16, b'i' as u16, b'x' as u16, b'e' as u16, b'd' as u16, 0];
pub static fmtFixed: [u8; 0x11] = [
    0x11, FMT_TYPE_NUMBER, 6, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x1, 0x2,
    FMT_NUM_COPY_ZERO, 0x1, FMT_NUM_DECIMAL, FMT_NUM_COPY_ZERO, 0x2, FMT_GEN_END
];

pub static szStandard: [u16; 9] = [b'S' as u16, b't' as u16, b'a' as u16, b'n' as u16, b'd' as u16, b'a' as u16, b'r' as u16, b'd' as u16, 0];
pub static fmtStandard: [u8; 0x11] = [
    0x11, FMT_TYPE_NUMBER, 6, 0x0, 0x0, 0x0,
    FMT_FLAG_THOUSANDS, 0x0, 0x0, 0x1, 0x2,
    FMT_NUM_COPY_ZERO, 0x1, FMT_NUM_DECIMAL, FMT_NUM_COPY_ZERO, 0x2, FMT_GEN_END
];

pub static szPercent: [u16; 8] = [b'P' as u16, b'e' as u16, b'r' as u16, b'c' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0];
pub static fmtPercent: [u8; 0x15] = [
    0x15, FMT_TYPE_NUMBER, 6, 0x0, 0x0, 0x0,
    FMT_FLAG_PERCENT, 0x1, 0x0, 0x1, 0x2,
    FMT_NUM_COPY_ZERO, 0x1, FMT_NUM_DECIMAL, FMT_NUM_COPY_ZERO, 0x2,
    FMT_GEN_INLINE, 0x1, b'%', 0, FMT_GEN_END
];

pub static szScientific: [u16; 11] = [b'S' as u16, b'c' as u16, b'i' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'i' as u16, b'f' as u16, b'i' as u16, b'c' as u16, 0];
pub static fmtScientific: [u8; 0x13] = [
    0x13, FMT_TYPE_NUMBER, 6, 0x0, 0x0, 0x0,
    FMT_FLAG_EXPONENT, 0x0, 0x0, 0x1, 0x2,
    FMT_NUM_COPY_ZERO, 0x1, FMT_NUM_DECIMAL, FMT_NUM_COPY_ZERO, 0x2, FMT_NUM_EXP_POS_U, 0x2, FMT_GEN_END
];

#[repr(C)]
pub struct NAMED_FORMAT {
    pub name: LPCWSTR,
    pub format: *const u8,
}
pub type PNAMED_FORMAT = *mut NAMED_FORMAT;
pub type LPCNAMED_FORMAT = *const NAMED_FORMAT;

// C's `static const NAMED_FORMAT VARIANT_NamedFormats[]` is immutable data holding pointers to
// other immutable data. Rust requires `Sync` for a `static`, and raw pointers are not `Sync` by
// default. The table is never mutated and the pointees are `static`s, so sharing it across
// threads is sound.
unsafe impl Sync for NAMED_FORMAT {}

pub static VARIANT_NamedFormats: [NAMED_FORMAT; 16] = [
    NAMED_FORMAT { name: szCurrency.as_ptr(), format: fmtCurrency.as_ptr() },
    NAMED_FORMAT { name: szFixed.as_ptr(), format: fmtFixed.as_ptr() },
    NAMED_FORMAT { name: szGeneralDate.as_ptr(), format: fmtGeneralDate.as_ptr() },
    NAMED_FORMAT { name: szGeneralNumber.as_ptr(), format: fmtGeneralNumber.as_ptr() },
    NAMED_FORMAT { name: szLongDate.as_ptr(), format: fmtLongDate.as_ptr() },
    NAMED_FORMAT { name: szLongTime.as_ptr(), format: fmtLongTime.as_ptr() },
    NAMED_FORMAT { name: szMediumDate.as_ptr(), format: fmtMediumDate.as_ptr() },
    NAMED_FORMAT { name: szMediumTime.as_ptr(), format: fmtMediumTime.as_ptr() },
    NAMED_FORMAT { name: szOnOff.as_ptr(), format: fmtOnOff.as_ptr() },
    NAMED_FORMAT { name: szPercent.as_ptr(), format: fmtPercent.as_ptr() },
    NAMED_FORMAT { name: szScientific.as_ptr(), format: fmtScientific.as_ptr() },
    NAMED_FORMAT { name: szShortDate.as_ptr(), format: fmtShortDate.as_ptr() },
    NAMED_FORMAT { name: szShortTime.as_ptr(), format: fmtShortTime.as_ptr() },
    NAMED_FORMAT { name: szStandard.as_ptr(), format: fmtStandard.as_ptr() },
    NAMED_FORMAT { name: szTrueFalse.as_ptr(), format: fmtTrueFalse.as_ptr() },
    NAMED_FORMAT { name: szYesNo.as_ptr(), format: fmtYesNo.as_ptr() }
];

pub const FMT_STATE_OPEN_COPY: u32 = 0x1;
pub const FMT_STATE_WROTE_DECIMAL: u32 = 0x2;
pub const FMT_STATE_SEEN_HOURS: u32 = 0x4;
pub const FMT_STATE_WROTE_MINUTES: u32 = 0x8;

pub const NUM_WROTE_DEC: u32 = 0x01;
pub const NUM_WRITE_ON: u32 = 0x02;
pub const NUM_WROTE_SIGN: u32 = 0x04;

pub const NUMBER_VTBITS: u32 = 0x1FFE07F;

unsafe extern "C" fn FormatCompareFn(l: *const core::ffi::c_void, r: *const core::ffi::c_void) -> i32 {
    
    wcsicmp(
        (*(l as *const NAMED_FORMAT)).name,
        (*(r as *const NAMED_FORMAT)).name
    )
}
unsafe extern "C" fn VARIANT_GetNamedFormat(lpszFormat: LPCWSTR) -> *const u8 {

    let mut key: NAMED_FORMAT = core::mem::zeroed();
    key.name = lpszFormat;
    
    let fmt = bsearch(
        core::ptr::addr_of!(key) as *const core::ffi::c_void,
        VARIANT_NamedFormats.as_ptr() as *const core::ffi::c_void,
        VARIANT_NamedFormats.len(),
        core::mem::size_of::<NAMED_FORMAT>(),
        FormatCompareFn,
    ) as *const NAMED_FORMAT;
    
    if !fmt.is_null() {
        (*fmt).format
    } else {
        core::ptr::null()
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarTokenizeFormatString(
    lpszFormat: LPWSTR,
    rgbTok: LPBYTE,
    cbTok: i32,
    nFirstDay: i32,
    nFirstWeek: i32,
    _lcid: LCID,
    pcbActual: *mut i32,
) -> HRESULT {
    static szTTTTT: [u16; 5] = [b't' as u16, b't' as u16, b't' as u16, b't' as u16, b't' as u16];
    static szAMPM: [u16; 4] = [b'A' as u16, b'M' as u16, b'P' as u16, b'M' as u16];
    static szampm: [u16; 4] = [b'a' as u16, b'm' as u16, b'p' as u16, b'm' as u16];
    static szAMSlashPM: [u16; 5] = [b'A' as u16, b'M' as u16, b'/' as u16, b'P' as u16, b'M' as u16];
    static szamSlashpm: [u16; 5] = [b'a' as u16, b'm' as u16, b'/' as u16, b'p' as u16, b'm' as u16];

    let mut cbTok = cbTok;
    let header = rgbTok as *mut FMT_HEADER;
    let str_header = rgbTok.add(core::mem::size_of::<FMT_HEADER>()) as *mut FMT_STRING_HEADER;
    let num_header = str_header as *mut FMT_NUMBER_HEADER;
    let mut pOut = rgbTok.add(core::mem::size_of::<FMT_HEADER>() + core::mem::size_of::<FMT_STRING_HEADER>());
    let mut pLastHours: *mut u8 = core::ptr::null_mut();
    let mut fmt_number: u8 = 0;
    let mut fmt_state: u32 = 0;
    let mut pFormat = lpszFormat;

    if rgbTok.is_null() || nFirstDay < 0 || nFirstDay > 7 || nFirstWeek < 0 || nFirstWeek > 3 {
        return E_INVALIDARG as HRESULT;
    }

    if lpszFormat.is_null() || *lpszFormat == 0 {
        if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
        cbTok -= 1;
        *rgbTok = FMT_TO_STRING as u8;
        if !pcbActual.is_null() {
            *pcbActual = FMT_TO_STRING as i32;
        }
        return S_OK as HRESULT;
    }

    if cbTok > 255 {
        cbTok = 255;
    }

    let namedFmt = VARIANT_GetNamedFormat(lpszFormat);
    if !namedFmt.is_null() {
        let size = *namedFmt as i32;
        if cbTok < size { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
        core::ptr::copy_nonoverlapping(namedFmt, rgbTok, size as usize);
        return S_OK as HRESULT;
    }

    if cbTok < (core::mem::size_of::<FMT_HEADER>() + core::mem::size_of::<FMT_STRING_HEADER>()) as i32 {
        return TYPE_E_BUFFERTOOSMALL as HRESULT;
    }
    cbTok -= (core::mem::size_of::<FMT_HEADER>() + core::mem::size_of::<FMT_STRING_HEADER>()) as i32;
    core::ptr::write_bytes(header, 0, 1);
    core::ptr::write_bytes(str_header, 0, 1);

    (*header).starts[fmt_number as usize] = core::mem::size_of::<FMT_HEADER>() as u8;

    while *pFormat != 0 {
        if *pFormat == b';' as u16 {
            while *pFormat == b';' as u16 {
                fmt_number += 1;
                if fmt_number > 3 {
                    return E_INVALIDARG as HRESULT;
                }
                pFormat = pFormat.add(1);
            }
            if *pFormat != 0 {
                if cbTok < (1 + core::mem::size_of::<FMT_STRING_HEADER>()) as i32 {
                    return TYPE_E_BUFFERTOOSMALL as HRESULT;
                }
                cbTok -= (1 + core::mem::size_of::<FMT_STRING_HEADER>()) as i32;
                *pOut = FMT_GEN_END as u8;
                pOut = pOut.add(1);

                (*header).starts[fmt_number as usize] = pOut.offset_from(rgbTok) as u8;
                let str_header = pOut as *mut FMT_STRING_HEADER;
                let _num_header = pOut as *mut FMT_NUMBER_HEADER;
                core::ptr::write_bytes(str_header, 0, 1);
                pOut = pOut.add(core::mem::size_of::<FMT_STRING_HEADER>());
                fmt_state = 0;
                pLastHours = core::ptr::null_mut();
            }
        } else if *pFormat == b'\\' as u16 {
            if *pFormat.add(1) != 0 {
                if cbTok < 3 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
                cbTok -= 3;
                pFormat = pFormat.add(1);
                *pOut = FMT_GEN_COPY as u8;
                pOut = pOut.add(1);
                *pOut = pFormat.offset_from(lpszFormat) as u8;
                pOut = pOut.add(1);
                *pOut = 0x1;
                pOut = pOut.add(1);
                fmt_state |= FMT_STATE_OPEN_COPY;
            } else {
                fmt_state &= !FMT_STATE_OPEN_COPY;
            }
            pFormat = pFormat.add(1);
        } else if *pFormat == b'"' as u16 {
            if *pFormat.add(1) == 0 {
                pFormat = pFormat.add(1);
            } else if *pFormat.add(1) == b'"' as u16 {
                pFormat = pFormat.add(2);
            } else {
                pFormat = pFormat.add(1);
                let start = pFormat;
                while *pFormat != 0 && *pFormat != b'"' as u16 {
                    pFormat = pFormat.add(1);
                }
                if cbTok < 3 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
                cbTok -= 3;
                *pOut = FMT_GEN_COPY as u8;
                pOut = pOut.add(1);
                *pOut = start.offset_from(lpszFormat) as u8;
                pOut = pOut.add(1);
                *pOut = pFormat.offset_from(start) as u8;
                pOut = pOut.add(1);
                if *pFormat == b'"' as u16 {
                    pFormat = pFormat.add(1);
                }
            }
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if *pFormat == b'0' as u16 && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_NUMBER) {
            (*header).type_ = FMT_TYPE_NUMBER;
            if cbTok < 2 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 2;
            *pOut = FMT_NUM_COPY_ZERO as u8;
            pOut = pOut.add(1);
            *pOut = 0x0;
            while *pFormat == b'0' as u16 {
                *pOut = (*pOut).wrapping_add(1);
                pFormat = pFormat.add(1);
            }
            if fmt_state & FMT_STATE_WROTE_DECIMAL != 0 {
                (*num_header).fractional = (*num_header).fractional.wrapping_add(*pOut);
            } else {
                (*num_header).whole = (*num_header).whole.wrapping_add(*pOut);
            }
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if *pFormat == b'#' as u16 && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_NUMBER) {
            (*header).type_ = FMT_TYPE_NUMBER;
            if cbTok < 2 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 2;
            *pOut = FMT_NUM_COPY_SKIP as u8;
            pOut = pOut.add(1);
            *pOut = 0x0;
            while *pFormat == b'#' as u16 {
                *pOut = (*pOut).wrapping_add(1);
                pFormat = pFormat.add(1);
            }
            if fmt_state & FMT_STATE_WROTE_DECIMAL != 0 {
                (*num_header).fractional = (*num_header).fractional.wrapping_add(*pOut);
            } else {
                (*num_header).whole = (*num_header).whole.wrapping_add(*pOut);
            }
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if *pFormat == b'.' as u16 && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_NUMBER) && fmt_state & FMT_STATE_WROTE_DECIMAL == 0 {
            (*header).type_ = FMT_TYPE_NUMBER;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            *pOut = FMT_NUM_DECIMAL as u8;
            pOut = pOut.add(1);
            fmt_state |= FMT_STATE_WROTE_DECIMAL;
            fmt_state &= !FMT_STATE_OPEN_COPY;
            pFormat = pFormat.add(1);
        } else if (*pFormat == b'e' as u16 || *pFormat == b'E' as u16) && (*pFormat.add(1) == b'-' as u16 || *pFormat.add(1) == b'+' as u16) && (*header).type_ == FMT_TYPE_NUMBER {
            (*num_header).flags |= FMT_FLAG_EXPONENT;
            if cbTok < 2 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 2;
            if *pFormat == b'e' as u16 {
                if *pFormat.add(1) == b'+' as u16 {
                    *pOut = FMT_NUM_EXP_POS_L as u8;
                } else {
                    *pOut = FMT_NUM_EXP_NEG_L as u8;
                }
            } else {
                if *pFormat.add(1) == b'+' as u16 {
                    *pOut = FMT_NUM_EXP_POS_U as u8;
                } else {
                    *pOut = FMT_NUM_EXP_NEG_U as u8;
                }
            }
            pFormat = pFormat.add(2);
            pOut = pOut.add(1);
            *pOut = 0x0;
            while *pFormat == b'0' as u16 {
                *pOut = (*pOut).wrapping_add(1);
                pFormat = pFormat.add(1);
            }
            pOut = pOut.add(1);
        } else if *pFormat == b',' as u16 && (*header).type_ == FMT_TYPE_NUMBER {
            (*num_header).flags |= FMT_FLAG_THOUSANDS;
            pFormat = pFormat.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if *pFormat == b'/' as u16 && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            *pOut = FMT_DATE_DATE_SEP as u8;
            pOut = pOut.add(1);
            pFormat = pFormat.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if *pFormat == b':' as u16 && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            *pOut = FMT_DATE_TIME_SEP as u8;
            pOut = pOut.add(1);
            pFormat = pFormat.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if (*pFormat == b'a' as u16 || *pFormat == b'A' as u16) && _wcsnicmp(pFormat, szAMPM.as_ptr(), 4) == 0 {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(4);
            if wcsncmp(pFormat, szampm.as_ptr(), 4) == 0 {
                *pOut = FMT_DATE_AMPM_SYS2 as u8;
            } else {
                *pOut = FMT_DATE_AMPM_SYS1 as u8;
            }
            pOut = pOut.add(1);
            if !pLastHours.is_null() {
                *pLastHours = (*pLastHours).wrapping_add(2);
            }
        } else if *pFormat == b'a' as u16 && *pFormat.add(1) == b'/' as u16 && (*pFormat.add(2) == b'p' as u16 || *pFormat.add(2) == b'P' as u16) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(3);
            *pOut = FMT_DATE_A_LOWER as u8;
            pOut = pOut.add(1);
            if !pLastHours.is_null() {
                *pLastHours = (*pLastHours).wrapping_add(2);
            }
        } else if *pFormat == b'A' as u16 && *pFormat.add(1) == b'/' as u16 && (*pFormat.add(2) == b'p' as u16 || *pFormat.add(2) == b'P' as u16) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(3);
            *pOut = FMT_DATE_A_UPPER as u8;
            pOut = pOut.add(1);
            if !pLastHours.is_null() {
                *pLastHours = (*pLastHours).wrapping_add(2);
            }
        } else if *pFormat == b'a' as u16 && wcsncmp(pFormat, szamSlashpm.as_ptr(), 5) == 0 {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(5);
            *pOut = FMT_DATE_AMPM_LOWER as u8;
            pOut = pOut.add(1);
            if !pLastHours.is_null() {
                *pLastHours = (*pLastHours).wrapping_add(2);
            }
        } else if *pFormat == b'A' as u16 && wcsncmp(pFormat, szAMSlashPM.as_ptr(), 5) == 0 {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(5);
            *pOut = FMT_DATE_AMPM_UPPER as u8;
            pOut = pOut.add(1);
        } else if (*pFormat == b'c' as u16 || *pFormat == b'C' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(5);
            *pOut = FMT_DATE_GENERAL as u8;
            pOut = pOut.add(1);
        } else if (*pFormat == b'd' as u16 || *pFormat == b'D' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            let mut count: i32 = -1;
            (*header).type_ = FMT_TYPE_DATE;
            while (*pFormat == b'd' as u16 || *pFormat == b'D' as u16) && count < 6 {
                pFormat = pFormat.add(1);
                count += 1;
            }
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            *pOut = (FMT_DATE_DAY as i32 + count) as u8;
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
            fmt_state &= !FMT_STATE_SEEN_HOURS;
        } else if (*pFormat == b'h' as u16 || *pFormat == b'H' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(1);
            pLastHours = pOut;
            if *pFormat == b'h' as u16 || *pFormat == b'H' as u16 {
                pFormat = pFormat.add(1);
                *pOut = FMT_DATE_HOUR_0 as u8;
            } else {
                *pOut = FMT_DATE_HOUR as u8;
            }
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
            fmt_state |= FMT_STATE_SEEN_HOURS;
        } else if (*pFormat == b'm' as u16 || *pFormat == b'M' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            let mut count: i32 = -1;
            (*header).type_ = FMT_TYPE_DATE;
            while (*pFormat == b'm' as u16 || *pFormat == b'M' as u16) && count < 4 {
                pFormat = pFormat.add(1);
                count += 1;
            }
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            if count <= 1 && fmt_state & FMT_STATE_SEEN_HOURS != 0 && fmt_state & FMT_STATE_WROTE_MINUTES == 0 {
                *pOut = if count == 1 { FMT_DATE_MIN_0 } else { FMT_DATE_MIN } as u8;
                fmt_state |= FMT_STATE_WROTE_MINUTES;
            } else {
                *pOut = (FMT_DATE_MON as i32 + count) as u8;
            }
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if (*pFormat == b'n' as u16 || *pFormat == b'N' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(1);
            if *pFormat == b'n' as u16 || *pFormat == b'N' as u16 {
                pFormat = pFormat.add(1);
                *pOut = FMT_DATE_MIN_0 as u8;
            } else {
                *pOut = FMT_DATE_MIN as u8;
            }
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if (*pFormat == b'q' as u16 || *pFormat == b'Q' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            *pOut = FMT_DATE_QUARTER as u8;
            pOut = pOut.add(1);
            pFormat = pFormat.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if (*pFormat == b's' as u16 || *pFormat == b'S' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            (*header).type_ = FMT_TYPE_DATE;
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            pFormat = pFormat.add(1);
            if *pFormat == b's' as u16 || *pFormat == b'S' as u16 {
                pFormat = pFormat.add(1);
                *pOut = FMT_DATE_SEC_0 as u8;
            } else {
                *pOut = FMT_DATE_SEC as u8;
            }
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if (*pFormat == b't' as u16 || *pFormat == b'T' as u16) && _wcsnicmp(pFormat, szTTTTT.as_ptr(), 5) == 0 {
            (*header).type_ = FMT_TYPE_DATE;
            pFormat = pFormat.add(5);
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            *pOut = FMT_DATE_TIME_SYS as u8;
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if (*pFormat == b'w' as u16 || *pFormat == b'W' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            (*header).type_ = FMT_TYPE_DATE;
            pFormat = pFormat.add(1);
            if *pFormat == b'w' as u16 || *pFormat == b'W' as u16 {
                if cbTok < 3 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
                cbTok -= 3;
                pFormat = pFormat.add(1);
                *pOut = FMT_DATE_WEEK_YEAR as u8;
                pOut = pOut.add(1);
                *pOut = nFirstDay as u8;
                pOut = pOut.add(1);
                *pOut = nFirstWeek as u8;
                pOut = pOut.add(1);
            } else {
                if cbTok < 2 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
                cbTok -= 2;
                *pOut = FMT_DATE_DAY_WEEK as u8;
                pOut = pOut.add(1);
                *pOut = nFirstDay as u8;
                pOut = pOut.add(1);
            }
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if (*pFormat == b'y' as u16 || *pFormat == b'Y' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_DATE) {
            let mut count: i32 = -1;
            (*header).type_ = FMT_TYPE_DATE;
            while (*pFormat == b'y' as u16 || *pFormat == b'Y' as u16) && count < 4 {
                pFormat = pFormat.add(1);
                count += 1;
            }
            if count == 2 {
                count -= 1;
                pFormat = pFormat.sub(1);
            }
            if cbTok < 1 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 1;
            *pOut = (FMT_DATE_YEAR_DOY as i32 + count) as u8;
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if *pFormat == b'@' as u16 && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_STRING) {
            (*header).type_ = FMT_TYPE_STRING;
            if cbTok < 2 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 2;
            *pOut = FMT_STR_COPY_SPACE as u8;
            pOut = pOut.add(1);
            *pOut = 0x0;
            while *pFormat == b'@' as u16 {
                *pOut = (*pOut).wrapping_add(1);
                (*str_header).copy_chars = (*str_header).copy_chars.wrapping_add(1);
                pFormat = pFormat.add(1);
            }
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if *pFormat == b'&' as u16 && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_STRING) {
            (*header).type_ = FMT_TYPE_STRING;
            if cbTok < 2 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
            cbTok -= 2;
            *pOut = FMT_STR_COPY_SKIP as u8;
            pOut = pOut.add(1);
            *pOut = 0x0;
            while *pFormat == b'&' as u16 {
                *pOut = (*pOut).wrapping_add(1);
                (*str_header).copy_chars = (*str_header).copy_chars.wrapping_add(1);
                pFormat = pFormat.add(1);
            }
            pOut = pOut.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if (*pFormat == b'<' as u16 || *pFormat == b'>' as u16) && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_STRING) {
            (*header).type_ = FMT_TYPE_STRING;
            if *pFormat == b'<' as u16 {
                (*str_header).flags |= FMT_FLAG_LT;
            } else {
                (*str_header).flags |= FMT_FLAG_GT;
            }
            pFormat = pFormat.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else if *pFormat == b'!' as u16 && (fmt_number == 0 && (*header).type_ == FMT_TYPE_UNKNOWN || (*header).type_ == FMT_TYPE_STRING) {
            (*header).type_ = FMT_TYPE_STRING;
            (*str_header).flags |= FMT_FLAG_RTL;
            pFormat = pFormat.add(1);
            fmt_state &= !FMT_STATE_OPEN_COPY;
        } else {
            if *pFormat == b'%' as u16 && (*header).type_ == FMT_TYPE_NUMBER {
                (*num_header).flags |= FMT_FLAG_PERCENT;
            }

            if fmt_state & FMT_STATE_OPEN_COPY != 0 {
                *pOut.sub(1) = (*pOut.sub(1)).wrapping_add(1);
            } else {
                if cbTok < 3 { return TYPE_E_BUFFERTOOSMALL as HRESULT; }
                cbTok -= 3;
                *pOut = FMT_GEN_COPY as u8;
                pOut = pOut.add(1);
                *pOut = pFormat.offset_from(lpszFormat) as u8;
                pOut = pOut.add(1);
                *pOut = 0x1;
                pOut = pOut.add(1);
                fmt_state |= FMT_STATE_OPEN_COPY;
            }
            pFormat = pFormat.add(1);
        }
    }

    *pOut = FMT_GEN_END as u8;
    pOut = pOut.add(1);

    (*header).size = pOut.offset_from(rgbTok) as u8;
    if !pcbActual.is_null() {
        *pcbActual = (*header).size as i32;
    }

    S_OK as HRESULT
}
unsafe extern "C" fn VARIANT_FormatNumber(
    pVarIn: *mut VARIANT,
    lpszFormat: LPOLESTR,
    rgbTok: LPBYTE,
    _dwFlags: ULONG,
    pbstrOut: *mut BSTR,
    lcid: LCID,
) -> HRESULT {
    let mut rgbDig: [BYTE; 256] = [0; 256];
    let mut prgbDig: *mut BYTE;
    let mut np: NUMPARSE = core::mem::zeroed();
    let mut have_int: i32;
    let mut need_int: i32 = 0;
    let mut have_frac: i32;
    let need_frac: i32;
    let mut exponent: i32 = 0;
    let mut pad: i32 = 0;
    let mut buff: [WCHAR; 256] = [0; 256];
    let mut pBuff: *mut WCHAR = buff.as_mut_ptr();
    let mut thousandSeparator: [WCHAR; 32] = [0; 32];
    let mut vString: VARIANT = core::mem::zeroed();
    let mut vBool: VARIANT = core::mem::zeroed();
    let mut dwState: DWORD = 0;
    let header: *mut FMT_HEADER = rgbTok as *mut FMT_HEADER;
    let numHeader: *mut FMT_NUMBER_HEADER;
    let mut pToken: *const BYTE = core::ptr::null();
    let mut hRes: HRESULT = S_OK;

    TRACE!(
        b"(%s,%s,%p,0x%08x,%p,0x%08x)\n\0".as_ptr() as PCHAR,
        debugstr_variant(pVarIn),
        debugstr_w(lpszFormat),
        rgbTok,
        dwFlags,
        pbstrOut,
        lcid,
    );

    *V_VT_ref(&mut vString) = VT_EMPTY as VARTYPE;
    *V_VT_ref(&mut vBool) = VT_BOOL as VARTYPE;

    if V_TYPE(pVarIn) == VT_EMPTY as u16 || V_TYPE(pVarIn) == VT_NULL as u16 {
        have_int = 0;
        have_frac = 0;
        numHeader = (rgbTok as usize + FmtGetNull(header) as usize) as *mut FMT_NUMBER_HEADER;
        *V_BOOL_ref(&mut vBool) = VARIANT_FALSE as VARIANT_BOOL;
    } else {
        hRes = VariantChangeTypeEx(&mut vString, pVarIn, lcid, VARIANT_NOUSEROVERRIDE as USHORT, VT_BSTR as VARTYPE);
        if FAILED(hRes) {
            return hRes;
        }

        np.cDig = core::mem::size_of_val(&rgbDig) as i32;
        np.dwInFlags = NUMPRS_STD;
        hRes = VarParseNumFromStr(V_BSTR(&mut vString), lcid, 0, &mut np, rgbDig.as_mut_ptr());
        if FAILED(hRes) {
            return hRes;
        }

        have_int = np.cDig;
        have_frac = 0;
        exponent = np.nPwr10;

        if (np.dwOutFlags & NUMPRS_NEG) != 0 {
            numHeader = (rgbTok as usize + FmtGetNegative(header) as usize) as *mut FMT_NUMBER_HEADER;
            *V_BOOL_ref(&mut vBool) = VARIANT_TRUE as VARIANT_BOOL;
        } else if have_int == 1 && exponent == 0 && rgbDig[0] == 0 {
            numHeader = (rgbTok as usize + FmtGetZero(header) as usize) as *mut FMT_NUMBER_HEADER;
            *V_BOOL_ref(&mut vBool) = VARIANT_FALSE as VARIANT_BOOL;
        } else {
            numHeader = (rgbTok as usize + FmtGetPositive(header) as usize) as *mut FMT_NUMBER_HEADER;
            *V_BOOL_ref(&mut vBool) = VARIANT_TRUE as VARIANT_BOOL;
        }

        TRACE!(
            b"num header: flags = 0x%x, mult=%d, div=%d, whole=%d, fract=%d\n\0".as_ptr() as PCHAR,
            (*numHeader).flags,
            (*numHeader).multiplier,
            (*numHeader).divisor,
            (*numHeader).whole,
            (*numHeader).fractional,
        );

        need_int = (*numHeader).whole as i32;
        need_frac = (*numHeader).fractional as i32;

        if ((*numHeader).flags & FMT_FLAG_PERCENT) != 0
            && !(have_int == 1 && exponent == 0 && rgbDig[0] == 0)
        {
            exponent += 2;
        }

        if ((*numHeader).flags & FMT_FLAG_EXPONENT) != 0 {
            pad = need_int - have_int;
            exponent -= pad;
            if pad < 0 {
                have_int = need_int;
                have_frac -= pad;
                pad = 0;
            }
        } else {
            pad = max(exponent, -have_int);
            exponent -= pad;
            if pad < 0 {
                have_int += pad;
                have_frac = -pad;
                pad = 0;
            }
            if exponent < 0 && exponent > (-256 + have_int + have_frac) {
                memmove(
                    rgbDig.as_mut_ptr().offset(-exponent as isize) as *mut core::ffi::c_void,
                    rgbDig.as_ptr() as *const core::ffi::c_void,
                    (have_int + have_frac) as usize,
                );
                ZeroMemory(rgbDig.as_mut_ptr() as *mut core::ffi::c_void, (-exponent) as usize);
                have_frac -= exponent;
                exponent = 0;
            }
        }

        if have_frac > need_frac {
            prgbDig = rgbDig.as_mut_ptr().offset((have_int + need_frac) as isize);
            have_frac = need_frac;
            if *prgbDig >= 5 {
                loop {
                    prgbDig = prgbDig.offset(-1);
                    if !(prgbDig >= rgbDig.as_mut_ptr() && *prgbDig == 9) {
                        break;
                    }
                    *prgbDig = 0;
                }
                if prgbDig < rgbDig.as_mut_ptr() {
                    rgbDig[0] = 1;
                    if ((*numHeader).flags & FMT_FLAG_EXPONENT) != 0 {
                        exponent += 1;
                    } else {
                        rgbDig[(have_int + need_frac) as usize] = 0;
                        if exponent < 0 {
                            exponent += 1;
                        } else {
                            have_int += 1;
                        }
                    }
                } else {
                    *prgbDig = (*prgbDig).wrapping_add(1);
                }
            }
            while have_frac > 0 && rgbDig[(have_int + have_frac - 1) as usize] == 0 {
                have_frac -= 1;
            }
        }
        TRACE!(
            b"have_int=%d,need_int=%d,have_frac=%d,need_frac=%d,pad=%d,exp=%d\n\0".as_ptr() as PCHAR,
            have_int,
            need_int,
            have_frac,
            need_frac,
            pad,
            exponent,
        );
    }

    if ((*numHeader).flags & FMT_FLAG_THOUSANDS) != 0 {
        if GetLocaleInfoW(lcid, LOCALE_STHOUSAND, thousandSeparator.as_mut_ptr(), ARRAY_SIZE(thousandSeparator) as i32) == 0 {
            thousandSeparator[0] = ',' as u16;
            thousandSeparator[1] = 0;
        }
    }

    pToken = (numHeader as usize + core::mem::size_of::<FMT_NUMBER_HEADER>()) as *const BYTE;
    prgbDig = rgbDig.as_mut_ptr();

    while SUCCEEDED(hRes) && *pToken != FMT_GEN_END {
        let mut defaultChar: WCHAR = '?' as u16;
        let mut boolFlag: DWORD = 0;
        let mut localeValue: DWORD = 0;
        let mut shouldAdvance: BOOL = 1;

        if (pToken as usize - rgbTok as usize) > (*header).size as usize {
            ERR!(b"Ran off the end of the format!\n\0".as_ptr() as PCHAR);
            hRes = E_INVALIDARG;
            break;
        }

        match *pToken {
            FMT_GEN_COPY => {
                TRACE!(
                    b"copy %s\n\0".as_ptr() as PCHAR,
                    debugstr_wn(lpszFormat.offset(*pToken.offset(1) as isize), *pToken.offset(2) as i32),
                );
                core::ptr::copy_nonoverlapping(
                    lpszFormat.offset(*pToken.offset(1) as isize),
                    pBuff,
                    *pToken.offset(2) as usize,
                );
                pBuff = pBuff.offset(*pToken.offset(2) as isize);
                pToken = pToken.offset(2);
            }
            FMT_GEN_INLINE => {
                pToken = pToken.offset(2);
                TRACE!(b"copy %s\n\0".as_ptr() as PCHAR, debugstr_a(pToken as LPCSTR));
                while *pToken != 0 {
                    *pBuff = *pToken as WCHAR;
                    pBuff = pBuff.offset(1);
                    pToken = pToken.offset(1);
                }
            }
            FMT_NUM_YES_NO => {
                boolFlag = VAR_BOOLYESNO;
                let mut boolStr: BSTR = core::ptr::null_mut();

                if *pToken.offset(1) != FMT_GEN_END {
                    ERR!(b"Boolean token not at end of format!\n\0".as_ptr() as PCHAR);
                    hRes = E_INVALIDARG;
                    break;
                }
                hRes = VarBstrFromBool(V_BOOL(&mut vBool), lcid, boolFlag, &mut boolStr);
                if SUCCEEDED(hRes) {
                    lstrcpyW(pBuff, boolStr);
                    SysFreeString(boolStr);
                    while *pBuff != 0 {
                        pBuff = pBuff.offset(1);
                    }
                }
            }
            FMT_NUM_ON_OFF => {
                boolFlag = VAR_BOOLONOFF;
                let mut boolStr: BSTR = core::ptr::null_mut();

                if *pToken.offset(1) != FMT_GEN_END {
                    ERR!(b"Boolean token not at end of format!\n\0".as_ptr() as PCHAR);
                    hRes = E_INVALIDARG;
                    break;
                }
                hRes = VarBstrFromBool(V_BOOL(&mut vBool), lcid, boolFlag, &mut boolStr);
                if SUCCEEDED(hRes) {
                    lstrcpyW(pBuff, boolStr);
                    SysFreeString(boolStr);
                    while *pBuff != 0 {
                        pBuff = pBuff.offset(1);
                    }
                }
            }
            FMT_NUM_TRUE_FALSE => {
                boolFlag = VAR_LOCALBOOL;
                let mut boolStr: BSTR = core::ptr::null_mut();

                if *pToken.offset(1) != FMT_GEN_END {
                    ERR!(b"Boolean token not at end of format!\n\0".as_ptr() as PCHAR);
                    hRes = E_INVALIDARG;
                    break;
                }
                hRes = VarBstrFromBool(V_BOOL(&mut vBool), lcid, boolFlag, &mut boolStr);
                if SUCCEEDED(hRes) {
                    lstrcpyW(pBuff, boolStr);
                    SysFreeString(boolStr);
                    while *pBuff != 0 {
                        pBuff = pBuff.offset(1);
                    }
                }
            }
            FMT_NUM_DECIMAL => {
                if (np.dwOutFlags & NUMPRS_NEG) != 0
                    && (dwState & NUM_WROTE_SIGN) == 0
                    && (*header).starts[1] == 0
                {
                    TRACE!(b"write negative sign\n\0".as_ptr() as PCHAR);
                    localeValue = LOCALE_SNEGATIVESIGN;
                    defaultChar = '-' as u16;
                    dwState |= NUM_WROTE_SIGN;
                    shouldAdvance = 0;
                } else {
                    TRACE!(b"write decimal separator\n\0".as_ptr() as PCHAR);
                    localeValue = LOCALE_SDECIMAL;
                    defaultChar = '.' as u16;
                    dwState |= NUM_WROTE_DEC;
                }
            }
            FMT_NUM_CURRENCY => {
                TRACE!(b"write currency symbol\n\0".as_ptr() as PCHAR);
                localeValue = LOCALE_SCURRENCY;
                defaultChar = '$' as u16;
            }
            FMT_NUM_EXP_POS_U | FMT_NUM_EXP_POS_L | FMT_NUM_EXP_NEG_U | FMT_NUM_EXP_NEG_L => {
                if *pToken == FMT_NUM_EXP_POS_L || *pToken == FMT_NUM_EXP_NEG_L {
                    *pBuff = 'e' as u16;
                    pBuff = pBuff.offset(1);
                } else {
                    *pBuff = 'E' as u16;
                    pBuff = pBuff.offset(1);
                }
                // Wine 10.0 bounds these: the width comes from the caller's
                // format string ("0E+000..."), so an unbounded write could run
                // past buff. Count leaves room for _snwprintf's NUL (buff is zeroed).
                if exponent < 0 {
                    *pBuff = '-' as u16;
                    pBuff = pBuff.offset(1);
                    _snwprintf(pBuff, (buff.len() - 1).saturating_sub(pBuff.offset_from(buff.as_ptr()) as usize), szPercentZeroStar_d.as_ptr(), *pToken.offset(1) as i32, -exponent);
                } else {
                    if *pToken == FMT_NUM_EXP_POS_L || *pToken == FMT_NUM_EXP_POS_U {
                        *pBuff = '+' as u16;
                        pBuff = pBuff.offset(1);
                    }
                    _snwprintf(pBuff, (buff.len() - 1).saturating_sub(pBuff.offset_from(buff.as_ptr()) as usize), szPercentZeroStar_d.as_ptr(), *pToken.offset(1) as i32, exponent);
                }
                while *pBuff != 0 {
                    pBuff = pBuff.offset(1);
                }
                pToken = pToken.offset(1);
            }
            FMT_NUM_COPY_ZERO => {
                dwState |= NUM_WRITE_ON;

                TRACE!(
                    b"write %d %sdigits or %s\n\0".as_ptr() as PCHAR,
                    *pToken.offset(1),
                    if (dwState & NUM_WROTE_DEC) != 0 {
                        b"fractional \0".as_ptr()
                    } else {
                        b"\0".as_ptr()
                    },
                    if *pToken == FMT_NUM_COPY_ZERO {
                        b"0\0".as_ptr()
                    } else {
                        b"skip\0".as_ptr()
                    },
                );

                if (dwState & NUM_WROTE_DEC) != 0 {
                    let mut count: i32;
                    let mut i: i32;

                    if ((*numHeader).flags & FMT_FLAG_EXPONENT) == 0 && exponent < 0 {
                        pad = max(exponent, -(*pToken.offset(1) as i32));
                        exponent -= pad;
                        count = min(have_frac, *pToken.offset(1) as i32 + pad);
                        i = 0;
                        while i > pad {
                            *pBuff = '0' as u16;
                            pBuff = pBuff.offset(1);
                            i -= 1;
                        }
                    } else {
                        count = min(have_frac, *pToken.offset(1) as i32);
                    }

                    pad += *pToken.offset(1) as i32 - count;
                    have_frac -= count;
                    while count > 0 {
                        *pBuff = ('0' as u8 + *prgbDig) as u16;
                        pBuff = pBuff.offset(1);
                        prgbDig = prgbDig.offset(1);
                        count -= 1;
                    }
                    if *pToken == FMT_NUM_COPY_ZERO {
                        while pad > 0 {
                            *pBuff = '0' as u16;
                            pBuff = pBuff.offset(1);
                            pad -= 1;
                        }
                    }
                } else {
                    let mut count: i32;
                    let mut count_max: i32;
                    let mut position: i32;

                    if (np.dwOutFlags & NUMPRS_NEG) != 0
                        && (dwState & NUM_WROTE_SIGN) == 0
                        && (*header).starts[1] == 0
                    {
                        TRACE!(b"write negative sign\n\0".as_ptr() as PCHAR);
                        localeValue = LOCALE_SNEGATIVESIGN;
                        defaultChar = '-' as u16;
                        dwState |= NUM_WROTE_SIGN;
                        shouldAdvance = 0;
                    } else {
                        position = have_int + pad;
                        if (dwState & NUM_WRITE_ON) != 0 {
                            position = max(position, need_int);
                        }
                        need_int -= *pToken.offset(1) as i32;
                        count_max = have_int + pad - need_int;
                        if count_max < 0 {
                            count_max = 0;
                        }
                        if (dwState & NUM_WRITE_ON) != 0 {
                            count = *pToken.offset(1) as i32 - count_max;
                            TRACE!(b"write %d leading zeros\n\0".as_ptr() as PCHAR, count);
                            while count > 0 {
                                *pBuff = '0' as u16;
                                pBuff = pBuff.offset(1);
                                if ((*numHeader).flags & FMT_FLAG_THOUSANDS) != 0
                                    && position > 1
                                    && {
                                        position -= 1;
                                        position % 3
                                    } == 0
                                {
                                    let mut k: i32 = 0;
                                    TRACE!(b"write thousand separator\n\0".as_ptr() as PCHAR);
                                    while thousandSeparator[k as usize] != 0 {
                                        *pBuff = thousandSeparator[k as usize];
                                        pBuff = pBuff.offset(1);
                                        k += 1;
                                    }
                                }
                                count -= 1;
                            }
                        }
                        if *pToken == FMT_NUM_COPY_ZERO
                            || have_int > 1
                            || (have_int > 0 && *prgbDig > 0)
                        {
                            count = min(count_max, have_int);
                            count_max -= count;
                            have_int -= count;
                            TRACE!(b"write %d whole number digits\n\0".as_ptr() as PCHAR, count);
                            while count > 0 {
                                dwState |= NUM_WRITE_ON;
                                *pBuff = ('0' as u8 + *prgbDig) as u16;
                                pBuff = pBuff.offset(1);
                                prgbDig = prgbDig.offset(1);
                                if ((*numHeader).flags & FMT_FLAG_THOUSANDS) != 0
                                    && position > 1
                                    && {
                                        position -= 1;
                                        position % 3
                                    } == 0
                                {
                                    let mut k: i32 = 0;
                                    TRACE!(b"write thousand separator\n\0".as_ptr() as PCHAR);
                                    while thousandSeparator[k as usize] != 0 {
                                        *pBuff = thousandSeparator[k as usize];
                                        pBuff = pBuff.offset(1);
                                        k += 1;
                                    }
                                }
                                count -= 1;
                            }
                        }
                        count = min(count_max, pad);
                        pad -= count;
                        TRACE!(b"write %d whole trailing 0's\n\0".as_ptr() as PCHAR, count);
                        while count > 0 {
                            *pBuff = '0' as u16;
                            pBuff = pBuff.offset(1);
                            if ((*numHeader).flags & FMT_FLAG_THOUSANDS) != 0
                                && position > 1
                                && {
                                    position -= 1;
                                    position % 3
                                } == 0
                            {
                                let mut k: i32 = 0;
                                TRACE!(b"write thousand separator\n\0".as_ptr() as PCHAR);
                                while thousandSeparator[k as usize] != 0 {
                                    *pBuff = thousandSeparator[k as usize];
                                    pBuff = pBuff.offset(1);
                                    k += 1;
                                }
                            }
                            count -= 1;
                        }
                    }
                }
                pToken = pToken.offset(1);
            }
            FMT_NUM_COPY_SKIP => {
                TRACE!(
                    b"write %d %sdigits or %s\n\0".as_ptr() as PCHAR,
                    *pToken.offset(1),
                    if (dwState & NUM_WROTE_DEC) != 0 {
                        b"fractional \0".as_ptr()
                    } else {
                        b"\0".as_ptr()
                    },
                    if *pToken == FMT_NUM_COPY_ZERO {
                        b"0\0".as_ptr()
                    } else {
                        b"skip\0".as_ptr()
                    },
                );

                if (dwState & NUM_WROTE_DEC) != 0 {
                    let mut count: i32;
                    let mut i: i32;

                    if ((*numHeader).flags & FMT_FLAG_EXPONENT) == 0 && exponent < 0 {
                        pad = max(exponent, -(*pToken.offset(1) as i32));
                        exponent -= pad;
                        count = min(have_frac, *pToken.offset(1) as i32 + pad);
                        i = 0;
                        while i > pad {
                            *pBuff = '0' as u16;
                            pBuff = pBuff.offset(1);
                            i -= 1;
                        }
                    } else {
                        count = min(have_frac, *pToken.offset(1) as i32);
                    }

                    pad += *pToken.offset(1) as i32 - count;
                    have_frac -= count;
                    while count > 0 {
                        *pBuff = ('0' as u8 + *prgbDig) as u16;
                        pBuff = pBuff.offset(1);
                        prgbDig = prgbDig.offset(1);
                        count -= 1;
                    }
                    if *pToken == FMT_NUM_COPY_ZERO {
                        while pad > 0 {
                            *pBuff = '0' as u16;
                            pBuff = pBuff.offset(1);
                            pad -= 1;
                        }
                    }
                } else {
                    let mut count: i32;
                    let mut count_max: i32;
                    let mut position: i32;

                    if (np.dwOutFlags & NUMPRS_NEG) != 0
                        && (dwState & NUM_WROTE_SIGN) == 0
                        && (*header).starts[1] == 0
                    {
                        TRACE!(b"write negative sign\n\0".as_ptr() as PCHAR);
                        localeValue = LOCALE_SNEGATIVESIGN;
                        defaultChar = '-' as u16;
                        dwState |= NUM_WROTE_SIGN;
                        shouldAdvance = 0;
                    } else {
                        position = have_int + pad;
                        if (dwState & NUM_WRITE_ON) != 0 {
                            position = max(position, need_int);
                        }
                        need_int -= *pToken.offset(1) as i32;
                        count_max = have_int + pad - need_int;
                        if count_max < 0 {
                            count_max = 0;
                        }
                        if (dwState & NUM_WRITE_ON) != 0 {
                            count = *pToken.offset(1) as i32 - count_max;
                            TRACE!(b"write %d leading zeros\n\0".as_ptr() as PCHAR, count);
                            while count > 0 {
                                *pBuff = '0' as u16;
                                pBuff = pBuff.offset(1);
                                if ((*numHeader).flags & FMT_FLAG_THOUSANDS) != 0
                                    && position > 1
                                    && {
                                        position -= 1;
                                        position % 3
                                    } == 0
                                {
                                    let mut k: i32 = 0;
                                    TRACE!(b"write thousand separator\n\0".as_ptr() as PCHAR);
                                    while thousandSeparator[k as usize] != 0 {
                                        *pBuff = thousandSeparator[k as usize];
                                        pBuff = pBuff.offset(1);
                                        k += 1;
                                    }
                                }
                                count -= 1;
                            }
                        }
                        if *pToken == FMT_NUM_COPY_ZERO
                            || have_int > 1
                            || (have_int > 0 && *prgbDig > 0)
                        {
                            count = min(count_max, have_int);
                            count_max -= count;
                            have_int -= count;
                            TRACE!(b"write %d whole number digits\n\0".as_ptr() as PCHAR, count);
                            while count > 0 {
                                dwState |= NUM_WRITE_ON;
                                *pBuff = ('0' as u8 + *prgbDig) as u16;
                                pBuff = pBuff.offset(1);
                                prgbDig = prgbDig.offset(1);
                                if ((*numHeader).flags & FMT_FLAG_THOUSANDS) != 0
                                    && position > 1
                                    && {
                                        position -= 1;
                                        position % 3
                                    } == 0
                                {
                                    let mut k: i32 = 0;
                                    TRACE!(b"write thousand separator\n\0".as_ptr() as PCHAR);
                                    while thousandSeparator[k as usize] != 0 {
                                        *pBuff = thousandSeparator[k as usize];
                                        pBuff = pBuff.offset(1);
                                        k += 1;
                                    }
                                }
                                count -= 1;
                            }
                        }
                        count = min(count_max, pad);
                        pad -= count;
                        TRACE!(b"write %d whole trailing 0's\n\0".as_ptr() as PCHAR, count);
                        while count > 0 {
                            *pBuff = '0' as u16;
                            pBuff = pBuff.offset(1);
                            if ((*numHeader).flags & FMT_FLAG_THOUSANDS) != 0
                                && position > 1
                                && {
                                    position -= 1;
                                    position % 3
                                } == 0
                            {
                                let mut k: i32 = 0;
                                TRACE!(b"write thousand separator\n\0".as_ptr() as PCHAR);
                                while thousandSeparator[k as usize] != 0 {
                                    *pBuff = thousandSeparator[k as usize];
                                    pBuff = pBuff.offset(1);
                                    k += 1;
                                }
                            }
                            count -= 1;
                        }
                    }
                }
                pToken = pToken.offset(1);
            }
            _ => {
                ERR!(b"Unknown token 0x%02x!\n\0".as_ptr() as PCHAR, *pToken);
                hRes = E_INVALIDARG;
                break;
            }
        }
        if localeValue != 0 {
            if GetLocaleInfoW(
                lcid,
                localeValue,
                pBuff,
                (ARRAY_SIZE(buff) - (pBuff as usize - buff.as_ptr() as usize) / core::mem::size_of::<WCHAR>()) as i32,
            ) != 0
            {
                TRACE!(b"added %s\n\0".as_ptr() as PCHAR, debugstr_w(pBuff));
                while *pBuff != 0 {
                    pBuff = pBuff.offset(1);
                }
            } else {
                TRACE!(b"added %d '%c'\n\0".as_ptr() as PCHAR, defaultChar, defaultChar);
                *pBuff = defaultChar;
                pBuff = pBuff.offset(1);
            }
        }
        if shouldAdvance != 0 {
            pToken = pToken.offset(1);
        }
    }

    VariantClear(&mut vString);
    *pBuff = 0;
    TRACE!(b"buff is %s\n\0".as_ptr() as PCHAR, debugstr_w(buff.as_ptr()));
    if SUCCEEDED(hRes) {
        *pbstrOut = SysAllocString(buff.as_ptr());
        if (*pbstrOut).is_null() {
            hRes = E_OUTOFMEMORY;
        }
    }
    hRes
}
unsafe extern "C" fn VARIANT_FormatDate(
    pVarIn: *mut VARIANT,
    lpszFormat: LPOLESTR,
    rgbTok: LPBYTE,
    dwFlags: ULONG,
    pbstrOut: *mut BSTR,
    lcid: LCID,
) -> HRESULT {
    let mut buff: [WCHAR; 256] = [0; 256];
    let mut pBuff = buff.as_mut_ptr();
    let mut vDate: VARIANT = core::mem::zeroed();
    let mut udate: UDATE = core::mem::zeroed();
    let header = rgbTok as *const FMT_HEADER;
    let dateHeader: *const FMT_DATE_HEADER;
    let mut pToken: *const BYTE;
    let mut hRes: HRESULT;

    TRACE!(
        b"(%s,%s,%p,0x%08x,%p,0x%08x)\n\0".as_ptr() as PCHAR,
        debugstr_variant(pVarIn),
        debugstr_w(lpszFormat),
        rgbTok,
        dwFlags,
        pbstrOut,
        lcid,
    );

    *V_VT_ref(&mut vDate) = VT_EMPTY as VARTYPE;

    if V_TYPE(pVarIn) == VT_EMPTY as u16 || V_TYPE(pVarIn) == VT_NULL as u16 {
        dateHeader = rgbTok.offset(FmtGetNegative(header) as isize) as *const FMT_DATE_HEADER;
        *V_DATE_ref(&mut vDate) = 0.0;
    } else {
        let usFlags: USHORT = if dwFlags & VARIANT_CALENDAR_HIJRI != 0 {
            VAR_CALENDAR_HIJRI as USHORT
        } else {
            0
        };

        hRes = VariantChangeTypeEx(&mut vDate, pVarIn, lcid, usFlags, VT_DATE as u16);
        if FAILED(hRes) {
            // Wine 10.0: a BSTR that is a DATE in numeric form ("31809.40")
            // fails the locale date parse but is still a date: read it as a
            // double. Anything else keeps the failure.
            if *V_VT_ref(pVarIn) != VT_BSTR as VARTYPE {
                return hRes;
            }
            let mut endptr: *mut u16 = core::ptr::null_mut();
            let tmp = wcstod(*V_BSTR_ref(pVarIn), &mut endptr);
            if endptr.is_null() || *endptr != 0 {
                return hRes; /* not a double in string form */
            }
            let mut out: f64 = 0.0;
            hRes = crate::vartype::VarDateFromR8(tmp, &mut out);
            if FAILED(hRes) {
                return hRes;
            }
            *V_VT_ref(&mut vDate) = VT_DATE as VARTYPE;
            *V_DATE_ref(&mut vDate) = out;
        }
        dateHeader = rgbTok.offset(FmtGetPositive(header) as isize) as *const FMT_DATE_HEADER;
    }

    hRes = VarUdateFromDate(V_DATE(&mut vDate), 0, &mut udate);
    if FAILED(hRes) {
        return hRes;
    }
    pToken = (dateHeader as *const BYTE).offset(core::mem::size_of::<FMT_DATE_HEADER>() as isize);

    'main_loop: loop {
        if *pToken == FMT_GEN_END {
            break 'main_loop;
        }

        let mut dwVal: DWORD = 0;
        let mut localeValue: DWORD = 0;
        let mut dwFmt: DWORD = 0;
        let mut szPrintFmt: LPCWSTR = core::ptr::null();
        let mut defaultChar: WCHAR = '?' as WCHAR;

        if pToken.offset_from(rgbTok) > (*header).size as isize {
            ERR!(b"Ran off the end of the format!\n\0".as_ptr() as PCHAR);
            hRes = E_INVALIDARG;
            break 'main_loop;
        }

        match *pToken {
            FMT_GEN_COPY => {
                TRACE!(
                    b"copy %s\n\0".as_ptr() as PCHAR,
                    debugstr_wn(lpszFormat.offset(*pToken.offset(1) as isize), *pToken.offset(2) as i32),
                );
                core::ptr::copy_nonoverlapping(
                    lpszFormat.offset(*pToken.offset(1) as isize),
                    pBuff,
                    *pToken.offset(2) as usize,
                );
                pBuff = pBuff.offset(*pToken.offset(2) as isize);
                pToken = pToken.offset(2);
            }

            FMT_GEN_INLINE => {
                pToken = pToken.offset(2);
                TRACE!(b"copy %s\n\0".as_ptr() as PCHAR, debugstr_a(pToken as LPCSTR));
                while *pToken != 0 {
                    *pBuff = *pToken as WCHAR;
                    pBuff = pBuff.offset(1);
                    pToken = pToken.offset(1);
                }
            }

            FMT_DATE_TIME_SEP => {
                TRACE!(b"time separator\n\0".as_ptr() as PCHAR);
                localeValue = LOCALE_STIME;
                defaultChar = ':' as WCHAR;
            }

            FMT_DATE_DATE_SEP => {
                TRACE!(b"date separator\n\0".as_ptr() as PCHAR);
                localeValue = LOCALE_SDATE;
                defaultChar = '/' as WCHAR;
            }

            FMT_DATE_GENERAL => {
                let mut date: BSTR = core::ptr::null_mut();
                hRes = VarBstrFromDate(V_DATE(&mut vDate), lcid, 0, &mut date);
                if FAILED(hRes) {
                    break 'main_loop;
                }
                let mut pDate = date;
                while *pDate != 0 {
                    *pBuff = *pDate;
                    pBuff = pBuff.offset(1);
                    pDate = pDate.offset(1);
                }
                SysFreeString(date);
            }

            FMT_DATE_QUARTER => {
                if udate.st.wMonth <= 3 {
                    *pBuff = '1' as WCHAR;
                } else if udate.st.wMonth <= 6 {
                    *pBuff = '2' as WCHAR;
                } else if udate.st.wMonth <= 9 {
                    *pBuff = '3' as WCHAR;
                } else {
                    *pBuff = '4' as WCHAR;
                }
                pBuff = pBuff.offset(1);
            }

            FMT_DATE_TIME_SYS => {
                let mut date: BSTR = core::ptr::null_mut();
                hRes = VarBstrFromDate(V_DATE(&mut vDate), lcid, VAR_TIMEVALUEONLY, &mut date);
                if FAILED(hRes) {
                    break 'main_loop;
                }
                let mut pDate = date;
                while *pDate != 0 {
                    *pBuff = *pDate;
                    pBuff = pBuff.offset(1);
                    pDate = pDate.offset(1);
                }
                SysFreeString(date);
            }

            FMT_DATE_DAY => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = udate.st.wDay as DWORD;
            }

            FMT_DATE_DAY_0 => {
                szPrintFmt = szPercentZeroTwo_d.as_ptr();
                dwVal = udate.st.wDay as DWORD;
            }

            FMT_DATE_DAY_SHORT => {
                TRACE!(b"short day\n\0".as_ptr() as PCHAR);
                localeValue = LOCALE_SABBREVDAYNAME1 + (udate.st.wDayOfWeek as u32 + 6) % 7;
                defaultChar = '?' as WCHAR;
            }

            FMT_DATE_DAY_LONG => {
                TRACE!(b"long day\n\0".as_ptr() as PCHAR);
                localeValue = LOCALE_SDAYNAME1 + (udate.st.wDayOfWeek as u32 + 6) % 7;
                defaultChar = '?' as WCHAR;
            }

            FMT_DATE_SHORT => {
                dwFmt = LOCALE_SSHORTDATE;
            }

            FMT_DATE_LONG => {
                dwFmt = LOCALE_SLONGDATE;
            }

            FMT_DATE_MEDIUM => {
                FIXME!(b"Medium date treated as long date\n\0".as_ptr() as PCHAR);
                dwFmt = LOCALE_SLONGDATE;
            }

            FMT_DATE_DAY_WEEK => {
                szPrintFmt = szPercent_d.as_ptr();
                if *pToken.offset(1) != 0 {
                    dwVal = udate.st.wDayOfWeek as DWORD + 2 - *pToken.offset(1) as DWORD;
                } else {
                    GetLocaleInfoW(
                        lcid,
                        LOCALE_RETURN_NUMBER | LOCALE_IFIRSTDAYOFWEEK,
                        &mut dwVal as *mut DWORD as LPWSTR,
                        (core::mem::size_of::<DWORD>() / core::mem::size_of::<WCHAR>()) as i32,
                    );
                    dwVal = udate.st.wDayOfWeek as DWORD + 1 - dwVal;
                }
                pToken = pToken.offset(1);
            }

            FMT_DATE_WEEK_YEAR => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = udate.wDayOfYear as DWORD / 7 + 1;
                pToken = pToken.offset(2);
                FIXME!(
                    b"Ignoring nFirstDay of %d, nFirstWeek of %d\n\0".as_ptr() as PCHAR,
                    *pToken.offset(0) as i32,
                    *pToken.offset(1) as i32,
                );
            }

            FMT_DATE_MON => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = udate.st.wMonth as DWORD;
            }

            FMT_DATE_MON_0 => {
                szPrintFmt = szPercentZeroTwo_d.as_ptr();
                dwVal = udate.st.wMonth as DWORD;
            }

            FMT_DATE_MON_SHORT => {
                TRACE!(b"short month\n\0".as_ptr() as PCHAR);
                localeValue = LOCALE_SABBREVMONTHNAME1 + udate.st.wMonth as u32 - 1;
                defaultChar = '?' as WCHAR;
            }

            FMT_DATE_MON_LONG => {
                TRACE!(b"long month\n\0".as_ptr() as PCHAR);
                localeValue = LOCALE_SMONTHNAME1 + udate.st.wMonth as u32 - 1;
                defaultChar = '?' as WCHAR;
            }

            FMT_DATE_YEAR_DOY => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = udate.wDayOfYear as DWORD;
            }

            FMT_DATE_YEAR_0 => {
                szPrintFmt = szPercentZeroTwo_d.as_ptr();
                dwVal = (udate.st.wYear as DWORD % 100) as DWORD;
            }

            FMT_DATE_YEAR_LONG => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = udate.st.wYear as DWORD;
            }

            FMT_DATE_MIN => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = udate.st.wMinute as DWORD;
            }

            FMT_DATE_MIN_0 => {
                szPrintFmt = szPercentZeroTwo_d.as_ptr();
                dwVal = udate.st.wMinute as DWORD;
            }

            FMT_DATE_SEC => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = udate.st.wSecond as DWORD;
            }

            FMT_DATE_SEC_0 => {
                szPrintFmt = szPercentZeroTwo_d.as_ptr();
                dwVal = udate.st.wSecond as DWORD;
            }

            FMT_DATE_HOUR => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = udate.st.wHour as DWORD;
            }

            FMT_DATE_HOUR_0 | FMT_DATE_TIME_UNK2 => {
                szPrintFmt = szPercentZeroTwo_d.as_ptr();
                dwVal = udate.st.wHour as DWORD;
            }

            FMT_DATE_HOUR_12 => {
                szPrintFmt = szPercent_d.as_ptr();
                dwVal = if udate.st.wHour != 0 {
                    if udate.st.wHour > 12 {
                        udate.st.wHour as DWORD - 12
                    } else {
                        udate.st.wHour as DWORD
                    }
                } else {
                    12
                };
            }

            FMT_DATE_HOUR_12_0 => {
                szPrintFmt = szPercentZeroTwo_d.as_ptr();
                dwVal = if udate.st.wHour != 0 {
                    if udate.st.wHour > 12 {
                        udate.st.wHour as DWORD - 12
                    } else {
                        udate.st.wHour as DWORD
                    }
                } else {
                    12
                };
            }

            FMT_DATE_AMPM_SYS1 | FMT_DATE_AMPM_SYS2 => {
                localeValue = if udate.st.wHour < 12 {
                    LOCALE_S1159
                } else {
                    LOCALE_S2359
                };
                defaultChar = '?' as WCHAR;
            }

            FMT_DATE_AMPM_UPPER => {
                *pBuff = if udate.st.wHour < 12 { 'A' } else { 'P' } as WCHAR;
                pBuff = pBuff.offset(1);
                *pBuff = 'M' as WCHAR;
                pBuff = pBuff.offset(1);
            }

            FMT_DATE_A_UPPER => {
                *pBuff = if udate.st.wHour < 12 { 'A' } else { 'P' } as WCHAR;
                pBuff = pBuff.offset(1);
            }

            FMT_DATE_AMPM_LOWER => {
                *pBuff = if udate.st.wHour < 12 { 'a' } else { 'p' } as WCHAR;
                pBuff = pBuff.offset(1);
                *pBuff = 'm' as WCHAR;
                pBuff = pBuff.offset(1);
            }

            FMT_DATE_A_LOWER => {
                *pBuff = if udate.st.wHour < 12 { 'a' } else { 'p' } as WCHAR;
                pBuff = pBuff.offset(1);
            }

            _ => {
                ERR!(b"Unknown token 0x%02x!\n\0".as_ptr() as PCHAR, *pToken as i32);
                hRes = E_INVALIDARG;
                break 'main_loop;
            }
        }

        if localeValue != 0 {
            *pBuff = 0;
            if GetLocaleInfoW(
                lcid,
                localeValue,
                pBuff,
                (ARRAY_SIZE!(buff) - pBuff.offset_from(buff.as_mut_ptr()) as usize) as i32,
            ) != 0
            {
                TRACE!(b"added %s\n\0".as_ptr() as PCHAR, debugstr_w(pBuff));
                while *pBuff != 0 {
                    pBuff = pBuff.offset(1);
                }
            } else {
                TRACE!(
                    b"added %d %c\n\0".as_ptr() as PCHAR,
                    defaultChar as i32,
                    defaultChar as i32,
                );
                *pBuff = defaultChar;
                pBuff = pBuff.offset(1);
            }
        } else if dwFmt != 0 {
            let mut fmt_buff: [WCHAR; 80] = [0; 80];

            if GetLocaleInfoW(lcid, dwFmt, fmt_buff.as_mut_ptr(), ARRAY_SIZE!(fmt_buff) as i32) == 0
                || get_date_format(
                    lcid,
                    0,
                    // `udate.st` is com-sys's bindgen `_SYSTEMTIME`; `get_date_format` takes
                    // prelude's `SYSTEMTIME`. Two distinct Rust types, provably identical
                    // layout (8 x WORD), so the pointer cast is sound.
                    core::ptr::addr_of!(udate.st).cast(),
                    fmt_buff.as_ptr(),
                    pBuff,
                    (ARRAY_SIZE!(buff) - pBuff.offset_from(buff.as_mut_ptr()) as usize) as i32,
                ) == 0
            {
                hRes = E_INVALIDARG;
                break 'main_loop;
            }
            while *pBuff != 0 {
                pBuff = pBuff.offset(1);
            }
        } else if !szPrintFmt.is_null() {
            _snwprintf(pBuff, (buff.len() - 1).saturating_sub(pBuff.offset_from(buff.as_ptr()) as usize), szPrintFmt, dwVal);
            while *pBuff != 0 {
                pBuff = pBuff.offset(1);
            }
        }
        pToken = pToken.offset(1);
    }

    *pBuff = 0;
    TRACE!(b"buff is %s\n\0".as_ptr() as PCHAR, debugstr_w(buff.as_ptr()));
    if SUCCEEDED(hRes) {
        *pbstrOut = SysAllocString(buff.as_ptr());
        if (*pbstrOut).is_null() {
            hRes = E_OUTOFMEMORY;
        }
    }
    hRes
}
unsafe extern "C" fn VARIANT_FormatString(
    pVarIn: *mut VARIANT,
    lpszFormat: LPWSTR,
    rgbTok: LPBYTE,
    _dwFlags: ULONG,
    pbstrOut: *mut BSTR,
    lcid: LCID,
) -> HRESULT {
    static mut szEmpty: [WCHAR; 1] = [0];
    let mut buff: [WCHAR; 256] = [0; 256];
    let mut pBuff = buff.as_mut_ptr();
    let mut pSrc: LPWSTR;
    let header = rgbTok as *mut FMT_HEADER;
    let strHeader: *mut FMT_STRING_HEADER;
    let mut pToken: *const BYTE = core::ptr::null();
    let mut vStr: VARIANT = core::mem::zeroed();
    let mut blanks_first: i32;
    let mut bUpper: BOOL = 0; // FALSE
    let mut hRes: HRESULT = S_OK;

    TRACE!(
        b"%s,%s,%p,0x%08x,%p,0x%08x)\n\0".as_ptr() as LPCSTR,
        debugstr_variant(pVarIn),
        debugstr_w(lpszFormat),
        rgbTok,
        dwFlags,
        pbstrOut,
        lcid,
    );

    *V_VT_ref(&mut vStr) = VT_EMPTY as VARTYPE;

    if V_TYPE(pVarIn) == VT_EMPTY as u16 || V_TYPE(pVarIn) == VT_NULL as u16 {
        strHeader = rgbTok.offset(FmtGetNegative(header) as isize) as *mut FMT_STRING_HEADER;
        *V_BSTR_ref(&mut vStr) = core::ptr::addr_of_mut!(szEmpty) as *mut WCHAR;
    } else {
        hRes = VariantChangeTypeEx(
            &mut vStr,
            pVarIn,
            lcid,
            VARIANT_NOUSEROVERRIDE as USHORT,
            VT_BSTR as VARTYPE,
        );
        if FAILED(hRes) {
            return hRes;
        }

        if *V_BSTR(&mut vStr) == 0 {
            strHeader = rgbTok.offset(FmtGetNegative(header) as isize) as *mut FMT_STRING_HEADER;
        } else {
            strHeader = rgbTok.offset(FmtGetPositive(header) as isize) as *mut FMT_STRING_HEADER;
        }
    }

    pSrc = V_BSTR(&mut vStr);
    if ((*strHeader).flags & (FMT_FLAG_LT | FMT_FLAG_GT)) == FMT_FLAG_GT {
        bUpper = 1; // TRUE
    }
    blanks_first = (*strHeader).copy_chars as i32 - lstrlenW(pSrc);
    pToken = (strHeader as *const BYTE).offset(5);

    while *pToken != 0x02 {
        let mut dwCount: i32 = 0;

        if pToken.offset_from(rgbTok as *const BYTE) > (*header).size as isize {
            ERR!(b"Ran off the end of the format!\n\0".as_ptr() as LPCSTR);
            hRes = 0x80070057u32 as i32;
            break;
        }

        match *pToken {
            0x00 => {
                TRACE!(
                    b"copy %s\n\0".as_ptr() as LPCSTR,
                    debugstr_wn(lpszFormat.offset(*pToken.offset(1) as isize), *pToken.offset(2) as i32),
                );
                core::ptr::copy_nonoverlapping(
                    lpszFormat.offset(*pToken.offset(1) as isize),
                    pBuff,
                    *pToken.offset(2) as usize,
                );
                pBuff = pBuff.offset(*pToken.offset(2) as isize);
                pToken = pToken.offset(2);
            }
            0x40 | 0x41 => {
                dwCount = *pToken.offset(1) as i32;
                if *pToken == 0x40 && blanks_first > 0 {
                    TRACE!(b"insert %d initial spaces\n\0".as_ptr() as LPCSTR, blanks_first);
                    while dwCount > 0 && blanks_first > 0 {
                        *pBuff = 0x20;
                        pBuff = pBuff.offset(1);
                        dwCount -= 1;
                        blanks_first -= 1;
                    }
                }
                TRACE!(
                    b"copy %d chars%s\n\0".as_ptr() as LPCSTR,
                    dwCount,
                    if *pToken == 0x40 {
                        b" with space\0".as_ptr() as LPCSTR
                    } else {
                        b"\0".as_ptr() as LPCSTR
                    },
                );
                while dwCount > 0 && *pSrc != 0 {
                    if bUpper != 0 {
                        *pBuff = towupper(*pSrc as wint_t) as WCHAR;
                    } else {
                        *pBuff = towlower(*pSrc as wint_t) as WCHAR;
                    }
                    pBuff = pBuff.offset(1);
                    dwCount -= 1;
                    pSrc = pSrc.offset(1);
                }
                if *pToken == 0x40 && dwCount > 0 {
                    TRACE!(b"insert %d spaces\n\0".as_ptr() as LPCSTR, dwCount);
                    while dwCount > 0 {
                        dwCount -= 1;
                        *pBuff = 0x20;
                        pBuff = pBuff.offset(1);
                    }
                }
                pToken = pToken.offset(1);
            }
            _ => {
                ERR!(b"Unknown token 0x%02x!\n\0".as_ptr() as LPCSTR, *pToken);
                hRes = 0x80070057u32 as i32;
                break;
            }
        }
        pToken = pToken.offset(1);
    }

    while *pSrc != 0 {
        if bUpper != 0 {
            *pBuff = towupper(*pSrc as wint_t) as WCHAR;
        } else {
            *pBuff = towlower(*pSrc as wint_t) as WCHAR;
        }
        pBuff = pBuff.offset(1);
        pSrc = pSrc.offset(1);
    }
    VariantClear(&mut vStr);
    *pBuff = 0;
    TRACE!(b"buff is %s\n\0".as_ptr() as LPCSTR, debugstr_w(buff.as_ptr()));
    if SUCCEEDED(hRes) {
        *pbstrOut = SysAllocString(buff.as_ptr());
        if (*pbstrOut).is_null() {
            hRes = 0x8007000Eu32 as i32;
        }
    }
    hRes
}

#[no_mangle]
pub unsafe extern "system" fn VarFormatFromTokens(
    pVarIn: *mut VARIANT,
    lpszFormat: LPOLESTR,
    rgbTok: LPBYTE,
    dwFlags: ULONG,
    pbstrOut: *mut BSTR,
    lcid: LCID,
) -> HRESULT {
    let header = rgbTok as *mut FMT_SHORT_HEADER;
    let mut vTmp: VARIANT = core::mem::zeroed();
    let mut hres: HRESULT;

    TRACE!(
        b"(%p,%s,%p,%x,%p,0x%08x)\n\0".as_ptr() as LPCSTR,
        pVarIn,
        debugstr_w(lpszFormat),
        rgbTok,
        dwFlags,
        pbstrOut,
        lcid,
    );

    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }

    *pbstrOut = core::ptr::null_mut();

    if pVarIn.is_null() || rgbTok.is_null() {
        return E_INVALIDARG;
    }

    if V_VT(pVarIn) == VT_NULL {
        return S_OK;
    }

    if *rgbTok == FMT_TO_STRING || (*header).type_ == FMT_TYPE_GENERAL {
        *V_VT_ref(&mut vTmp) = VT_EMPTY as VARTYPE;
        hres = VariantChangeTypeEx(&mut vTmp, pVarIn, lcid, dwFlags as USHORT, VT_BSTR as VARTYPE);
        *pbstrOut = V_BSTR(&mut vTmp);
    } else {
        if (*header).type_ == FMT_TYPE_NUMBER
            || ((*header).type_ == FMT_TYPE_UNKNOWN
                && ((1 << V_TYPE(pVarIn)) & NUMBER_VTBITS) != 0)
        {
            hres = VARIANT_FormatNumber(pVarIn, lpszFormat, rgbTok, dwFlags, pbstrOut, lcid);
        } else if (*header).type_ == FMT_TYPE_DATE
            || ((*header).type_ == FMT_TYPE_UNKNOWN && V_TYPE(pVarIn) == VT_DATE)
        {
            hres = VARIANT_FormatDate(pVarIn, lpszFormat, rgbTok, dwFlags, pbstrOut, lcid);
        } else if (*header).type_ == FMT_TYPE_STRING || V_TYPE(pVarIn) == VT_BSTR {
            hres = VARIANT_FormatString(pVarIn, lpszFormat, rgbTok, dwFlags, pbstrOut, lcid);
        } else {
            ERR!(
                b"unrecognised format type 0x%02x\n\0".as_ptr() as LPCSTR,
                (*header).type_,
            );
            return E_INVALIDARG;
        }

        if (hres == DISP_E_OVERFLOW || hres == DISP_E_TYPEMISMATCH)
            && (dwFlags & VAR_FORMAT_NOSUBSTITUTE) == 0
        {
            *V_VT_ref(&mut vTmp) = VT_EMPTY as VARTYPE;
            hres = VariantChangeTypeEx(&mut vTmp, pVarIn, lcid, dwFlags as USHORT, VT_BSTR as VARTYPE);
            *pbstrOut = V_BSTR(&mut vTmp);
        }
    }

    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarFormat(
    pVarIn: LPVARIANT,
    lpszFormat: LPOLESTR,
    nFirstDay: i32,
    nFirstWeek: i32,
    dwFlags: ULONG,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    let mut buff: [u8; 256] = [0; 256];
    let mut hres: HRESULT;

    TRACE!(
        b"(%s,%s,%d,%d,0x%08x,%p)\n\0".as_ptr() as LPCSTR,
        debugstr_variant(pVarIn),
        debugstr_w(lpszFormat),
        nFirstDay,
        nFirstWeek,
        dwFlags,
        pbstrOut,
    );

    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }
    *pbstrOut = core::ptr::null_mut();

    hres = VarTokenizeFormatString(
        lpszFormat,
        buff.as_mut_ptr(),
        core::mem::size_of_val(&buff) as i32,
        nFirstDay,
        nFirstWeek,
        LOCALE_USER_DEFAULT,
        core::ptr::null_mut(),
    );
    if SUCCEEDED(hres) {
        hres = VarFormatFromTokens(
            pVarIn,
            lpszFormat,
            buff.as_mut_ptr(),
            dwFlags,
            pbstrOut,
            LOCALE_USER_DEFAULT,
        );
    }
    TRACE!(
        b"returning 0x%08x, %s\n\0".as_ptr() as LPCSTR,
        hres,
        debugstr_w(*pbstrOut),
    );
    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarFormatDateTime(
    pVarIn: *mut VARIANT,
    nFormat: i32,
    dwFlags: u32,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    static mut szEmpty: [u16; 1] = [0];
    let mut lpFmt: *const u8 = core::ptr::null();

    TRACE!(
        b"%s,%d,0x%08x,%p)\n\0".as_ptr() as *const i8,
        debugstr_variant(pVarIn),
        nFormat,
        dwFlags,
        pbstrOut,
    );

    if pVarIn.is_null() || pbstrOut.is_null() || nFormat < 0 || nFormat > 4 {
        return 0x80070057u32 as i32;
    }

    match nFormat {
        0 => lpFmt = fmtGeneralDate.as_ptr(),
        1 => lpFmt = fmtLongDate.as_ptr(),
        2 => lpFmt = fmtShortDate.as_ptr(),
        3 => lpFmt = fmtLongTime.as_ptr(),
        4 => lpFmt = fmtShortTime.as_ptr(),
        _ => {}
    }

    VarFormatFromTokens(
        pVarIn,
        core::ptr::addr_of_mut!(szEmpty) as LPOLESTR,
        lpFmt as *mut u8,
        dwFlags,
        pbstrOut,
        LOCALE_USER_DEFAULT,
    )
}

/// varformat.c:2212 —
/// ```c
/// #define GETLOCALENUMBER(type,field) GetLocaleInfoW(LOCALE_USER_DEFAULT, \
///                                                    type|LOCALE_RETURN_NUMBER, \
///                                                    (LPWSTR)&numfmt.field, \
///                                                    sizeof(numfmt.field)/sizeof(WCHAR))
/// ```
/// A file-scope macro used by BOTH `VarFormatNumber` (on a `NUMBERFMTW`) and
/// `VarFormatCurrency` (on a `CURRENCYFMTW`) — hence the struct-generic `$numfmt` parameter.
///
/// The C macro captures the caller's local `numfmt` by name; `macro_rules!` is hygienic and
/// cannot, so the struct is passed in explicitly. `LOCALE_RETURN_NUMBER` makes GetLocaleInfoW
/// write a DWORD through the `LPWSTR` out-pointer, which is why the C casts and divides by
/// sizeof(WCHAR) — reproduced verbatim.
macro_rules! GETLOCALENUMBER {
    ($numfmt:ident, $typ:expr, $field:ident) => {{
        let _ = GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            ($typ) | LOCALE_RETURN_NUMBER,
            core::ptr::addr_of_mut!($numfmt.$field) as LPWSTR,
            (core::mem::size_of_val(&$numfmt.$field) / core::mem::size_of::<WCHAR>()) as i32,
        );
    }};
}

#[no_mangle]
pub unsafe extern "system" fn VarFormatNumber(
    pVarIn: LPVARIANT,
    nDigits: INT,
    nLeading: INT,
    nParens: INT,
    nGrouping: INT,
    _dwFlags: ULONG,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    TRACE!(
        b"(%s,%d,%d,%d,%d,0x%08x,%p)\n\0".as_ptr() as LPCSTR,
        debugstr_variant(pVarIn),
        nDigits,
        nLeading,
        nParens,
        nGrouping,
        dwFlags,
        pbstrOut,
    );

    if pVarIn.is_null() || pbstrOut.is_null() || nDigits > 9 {
        return E_INVALIDARG;
    }

    *pbstrOut = core::ptr::null_mut();

    let mut vStr: VARIANT = core::mem::zeroed();
    *V_VT_ref(&mut vStr) = VT_EMPTY as VARTYPE;
    let mut hRet = VariantCopyInd(&mut vStr, pVarIn);

    if SUCCEEDED(hRet) {
        // C passes &vStr as BOTH dest and src; take one raw pointer so Rust does not need two
        // simultaneous &mut borrows of the same local.
        let pvStr: *mut VARIANT = &mut vStr;
        hRet = VariantChangeTypeEx(pvStr, pvStr, LCID_US, 0, VT_BSTR as VARTYPE);
    }

    if SUCCEEDED(hRet) {
        let mut buff: [WCHAR; 256] = [0; 256];
        let mut decimal: [WCHAR; 8] = [0; 8];
        let mut thousands: [WCHAR; 8] = [0; 8];
        let mut numfmt: NUMBERFMTW = core::mem::zeroed();

        if nDigits < 0 {
            GETLOCALENUMBER!(numfmt, LOCALE_IDIGITS, NumDigits);
        } else {
            numfmt.NumDigits = nDigits as UINT;
        }

        if nLeading == -2 {
            GETLOCALENUMBER!(numfmt, LOCALE_ILZERO, LeadingZero);
        } else if nLeading == -1 {
            numfmt.LeadingZero = 1;
        } else {
            numfmt.LeadingZero = 0;
        }

        if nGrouping == -2 {
            let mut grouping: [WCHAR; 10] = [0; 10];
            grouping[2] = 0;
            GetLocaleInfoW(
                LOCALE_USER_DEFAULT,
                LOCALE_SGROUPING,
                grouping.as_mut_ptr(),
                ARRAY_SIZE(grouping) as i32,
            );
            numfmt.Grouping = if grouping[2] == '2' as WCHAR {
                32
            } else {
                (grouping[0] - '0' as WCHAR) as UINT
            };
        } else if nGrouping == -1 {
            numfmt.Grouping = 3;
        } else {
            numfmt.Grouping = 0;
        }

        if nParens == -2 {
            GETLOCALENUMBER!(numfmt, LOCALE_INEGNUMBER, NegativeOrder);
        } else if nParens == -1 {
            numfmt.NegativeOrder = 0;
        } else {
            numfmt.NegativeOrder = 1;
        }

        numfmt.lpDecimalSep = decimal.as_mut_ptr();
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_SDECIMAL,
            decimal.as_mut_ptr(),
            ARRAY_SIZE(decimal) as i32,
        );
        numfmt.lpThousandSep = thousands.as_mut_ptr();
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_STHOUSAND,
            thousands.as_mut_ptr(),
            ARRAY_SIZE(thousands) as i32,
        );

        if GetNumberFormatW(
            LOCALE_USER_DEFAULT,
            0,
            V_BSTR(&mut vStr),
            core::ptr::addr_of!(numfmt) as *const c_void,
            buff.as_mut_ptr(),
            ARRAY_SIZE(buff) as i32,
        ) != 0
        {
            *pbstrOut = SysAllocString(buff.as_ptr());
            if (*pbstrOut).is_null() {
                hRet = E_OUTOFMEMORY;
            }
        } else {
            hRet = DISP_E_TYPEMISMATCH;
        }

        SysFreeString(V_BSTR(&mut vStr));
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarFormatPercent(
    pVarIn: LPVARIANT,
    nDigits: INT,
    nLeading: INT,
    nParens: INT,
    nGrouping: INT,
    dwFlags: ULONG,
    pbstrOut: *mut BSTR,
) -> HRESULT {

    static szPercent: [WCHAR; 2] = ['%' as u16, 0];
    static szPercentBracket: [WCHAR; 3] = ['%' as u16, ')' as u16, 0];
    
    let mut buff: [WCHAR; 256] = [0; 256];
    let mut hRet: HRESULT;
    let mut vDbl: VARIANT = core::mem::zeroed();

    TRACE!(
        b"(%s,%d,%d,%d,%d,0x%08x,%p)\n\0".as_ptr() as LPCSTR,
        debugstr_variant(pVarIn),
        nDigits,
        nLeading,
        nParens,
        nGrouping,
        dwFlags,
        pbstrOut,
    );

    if pVarIn.is_null() || pbstrOut.is_null() || nDigits > 9 {
        return E_INVALIDARG;
    }

    *pbstrOut = core::ptr::null_mut();

    *V_VT_ref(&mut vDbl) = VT_EMPTY as VARTYPE;
    hRet = VariantCopyInd(&mut vDbl, pVarIn);

    if SUCCEEDED(hRet) {
        // C passes &vDbl as BOTH dest and src.
        let pvDbl: *mut VARIANT = &mut vDbl;
        hRet = VariantChangeTypeEx(pvDbl, pvDbl, LOCALE_USER_DEFAULT, 0, VT_R8 as VARTYPE);

        if SUCCEEDED(hRet) {
            if V_R8(&vDbl) > (R8_MAX / 100.0) {
                return DISP_E_OVERFLOW;
            }

            *V_R8_ref(&mut vDbl) *= 100.0;
            hRet = VarFormatNumber(&mut vDbl, nDigits, nLeading, nParens, nGrouping, dwFlags, pbstrOut);

            if SUCCEEDED(hRet) {
                let dwLen = lstrlenW(*pbstrOut) as DWORD;
                // NOTE: verbatim from C (varformat.c:2380) — `(*pbstrOut)[dwLen]` indexes the
                // NUL terminator, so bBracket is effectively always FALSE. Upstream
                // Wine/ReactOS quirk; preserved deliberately rather than "corrected" to
                // [dwLen - 1], which would change observable behaviour.
                let bBracket = *(*pbstrOut).offset(dwLen as isize) == ')' as u16;

                let dwLen = dwLen - bBracket as DWORD;
                memcpy(
                    buff.as_mut_ptr() as *mut core::ffi::c_void,
                    *pbstrOut as *const core::ffi::c_void,
                    (dwLen as usize) * core::mem::size_of::<WCHAR>(),
                );
                lstrcpyW(
                    buff.as_mut_ptr().offset(dwLen as isize),
                    if bBracket { szPercentBracket.as_ptr() } else { szPercent.as_ptr() },
                );
                SysFreeString(*pbstrOut);
                *pbstrOut = SysAllocString(buff.as_ptr());
                if (*pbstrOut).is_null() {
                    hRet = E_OUTOFMEMORY;
                }
            }
        }
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarFormatCurrency(
    pVarIn: LPVARIANT,
    nDigits: INT,
    nLeading: INT,
    nParens: INT,
    nGrouping: INT,
    _dwFlags: ULONG,
    pbstrOut: *mut BSTR,
) -> HRESULT {

    TRACE!(
        b"(%s,%d,%d,%d,%d,0x%08x,%p)\n\0".as_ptr() as LPCSTR,
        debugstr_variant(pVarIn),
        nDigits,
        nLeading,
        nParens,
        nGrouping,
        dwFlags,
        pbstrOut
    );

    if pVarIn.is_null() || pbstrOut.is_null() || nDigits > 9 {
        return E_INVALIDARG;
    }

    *pbstrOut = core::ptr::null_mut();

    let mut vStr: VARIANT = core::mem::zeroed();
    // Wine 10.0: a currency string is PARSED as currency first (it may carry
    // the locale's currency symbol and separators, which GetCurrencyFormatW
    // would refuse in the raw string).
    let mut hRet;
    let vt_in = *V_VT_ref(pVarIn);
    if vt_in == VT_BSTR as VARTYPE || vt_in == (VT_BSTR | VT_BYREF) as VARTYPE {
        let bstr = if vt_in & VT_BYREF as VARTYPE != 0 {
            *((*pVarIn).n1.n2.n3.byref as *mut BSTR)
        } else {
            *V_BSTR_ref(pVarIn)
        };
        let mut cy: CY = core::mem::zeroed();
        hRet = crate::vartype::VarCyFromStr(bstr, LOCALE_USER_DEFAULT, 0, &mut cy);
        if FAILED(hRet) {
            return hRet;
        }
        *V_VT_ref(&mut vStr) = VT_CY as VARTYPE;
        *V_CY_ref(&mut vStr) = cy;
    } else {
        *V_VT_ref(&mut vStr) = VT_EMPTY as VARTYPE;
        hRet = VariantCopyInd(&mut vStr, pVarIn);
    }

    if SUCCEEDED(hRet) {
        // C passes &vStr as BOTH dest and src.
        let pvStr: *mut VARIANT = &mut vStr;
        hRet = VariantChangeTypeEx(pvStr, pvStr, LOCALE_USER_DEFAULT, 0, VT_BSTR as VARTYPE);
    }

    if SUCCEEDED(hRet) {
        let mut buff: [u16; 256] = [0; 256];
        let mut decimal: [u16; 8] = [0; 8];
        let mut thousands: [u16; 4] = [0; 4];
        let mut currency: [u16; 13] = [0; 13];
        let mut numfmt: CURRENCYFMTW = core::mem::zeroed();

        if nDigits < 0 {
            GETLOCALENUMBER!(numfmt, LOCALE_IDIGITS, NumDigits);
        } else {
            numfmt.NumDigits = nDigits as u32;
        }

        if nLeading == -2 {
            GETLOCALENUMBER!(numfmt, LOCALE_ILZERO, LeadingZero);
        } else if nLeading == -1 {
            numfmt.LeadingZero = 1;
        } else {
            numfmt.LeadingZero = 0;
        }

        if nGrouping == -2 {
            let mut grouping: [u16; 10] = [0; 10];
            grouping[2] = 0;
            GetLocaleInfoW(
                LOCALE_USER_DEFAULT,
                LOCALE_SGROUPING,
                grouping.as_mut_ptr(),
                ARRAY_SIZE!(grouping) as i32,
            );
            numfmt.Grouping = if grouping[2] == b'2' as u16 {
                32
            } else {
                (grouping[0] - b'0' as u16) as UINT
            };
        } else if nGrouping == -1 {
            numfmt.Grouping = 3;
        } else {
            numfmt.Grouping = 0;
        }

        if nParens == -2 {
            GETLOCALENUMBER!(numfmt, LOCALE_INEGCURR, NegativeOrder);
        } else if nParens == -1 {
            numfmt.NegativeOrder = 0;
        } else {
            numfmt.NegativeOrder = 1;
        }

        GETLOCALENUMBER!(numfmt, LOCALE_ICURRENCY, PositiveOrder);

        numfmt.lpDecimalSep = decimal.as_mut_ptr();
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_SDECIMAL,
            decimal.as_mut_ptr(),
            ARRAY_SIZE!(decimal) as i32,
        );
        numfmt.lpThousandSep = thousands.as_mut_ptr();
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_STHOUSAND,
            thousands.as_mut_ptr(),
            ARRAY_SIZE!(thousands) as i32,
        );
        numfmt.lpCurrencySymbol = currency.as_mut_ptr();
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_SCURRENCY,
            currency.as_mut_ptr(),
            ARRAY_SIZE!(currency) as i32,
        );

        if GetCurrencyFormatW(
            LOCALE_USER_DEFAULT,
            0,
            V_BSTR(&mut vStr),
            core::ptr::addr_of!(numfmt),
            buff.as_mut_ptr(),
            ARRAY_SIZE!(buff) as i32,
        ) != 0
        {
            *pbstrOut = SysAllocString(buff.as_ptr());
            if (*pbstrOut).is_null() {
                hRet = E_OUTOFMEMORY;
            }
        } else {
            hRet = DISP_E_TYPEMISMATCH;
        }

        SysFreeString(V_BSTR(&mut vStr));
    }

    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarMonthName(
    iMonth: INT,
    fAbbrev: INT,
    dwFlags: ULONG,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    let localeValue: DWORD;
    let mut size: INT;

    if (iMonth < 1) || (iMonth > 12) {
        return E_INVALIDARG;
    }

    if dwFlags != 0 {
        FIXME!(
            b"Does not support dwFlags 0x%x, ignoring.\n\0".as_ptr() as LPCSTR,
            dwFlags,
        );
    }

    if fAbbrev != 0 {
        localeValue = LOCALE_SABBREVMONTHNAME1 + (iMonth - 1) as u32;
    } else {
        localeValue = LOCALE_SMONTHNAME1 + (iMonth - 1) as u32;
    }

    size = GetLocaleInfoW(LOCALE_USER_DEFAULT, localeValue, core::ptr::null_mut(), 0);
    if size == 0 {
        ERR!(
            b"GetLocaleInfo 0x%x failed.\n\0".as_ptr() as LPCSTR,
            localeValue,
        );
        return HRESULT_FROM_WIN32(GetLastError());
    }

    *pbstrOut = SysAllocStringLen(core::ptr::null(), (size - 1) as u32);
    if (*pbstrOut).is_null() {
        return E_OUTOFMEMORY;
    }

    size = GetLocaleInfoW(LOCALE_USER_DEFAULT, localeValue, *pbstrOut, size);
    if size == 0 {
        ERR!(
            b"GetLocaleInfo of 0x%x failed in 2nd stage?!\n\0".as_ptr() as LPCSTR,
            localeValue,
        );
        SysFreeString(*pbstrOut);
        return HRESULT_FROM_WIN32(GetLastError());
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarWeekdayName(
    iWeekday: i32,
    fAbbrev: i32,
    iFirstDay: i32,
    dwFlags: u32,
    pbstrOut: *mut BSTR,
) -> HRESULT {
    let mut localeValue: u32;
    let mut size: i32;

    if iWeekday < 1 || iWeekday > 7 {
        return E_INVALIDARG;
    }
    if iFirstDay < 0 || iFirstDay > 7 {
        return E_INVALIDARG;
    }
    if pbstrOut.is_null() {
        return E_INVALIDARG;
    }

    if dwFlags != 0 {
        FIXME!(
            b"Does not support dwFlags 0x%x, ignoring.\n\0".as_ptr() as LPCSTR,
            dwFlags,
        );
    }

    let mut iFirstDay = iFirstDay;
    if iFirstDay == 0 {
        let mut firstDay: u32 = 0;
        localeValue = LOCALE_RETURN_NUMBER | LOCALE_IFIRSTDAYOFWEEK;
        size = GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            localeValue,
            &mut firstDay as *mut u32 as LPWSTR,
            (core::mem::size_of::<u32>() / core::mem::size_of::<u16>()) as i32,
        );
        if size == 0 {
            ERR!(
                b"GetLocaleInfo 0x%x failed.\n\0".as_ptr() as LPCSTR,
                localeValue,
            );
            return HRESULT_FROM_WIN32(GetLastError());
        }
        iFirstDay = firstDay as i32 + 2;
    }

    localeValue = if fAbbrev != 0 {
        LOCALE_SABBREVDAYNAME1
    } else {
        LOCALE_SDAYNAME1
    };
    localeValue += ((7 + iWeekday - 1 + iFirstDay - 2) % 7) as u32;

    size = GetLocaleInfoW(LOCALE_USER_DEFAULT, localeValue, core::ptr::null_mut(), 0);
    if size == 0 {
        ERR!(
            b"GetLocaleInfo 0x%x failed.\n\0".as_ptr() as LPCSTR,
            localeValue,
        );
        return HRESULT_FROM_WIN32(GetLastError());
    }
    *pbstrOut = SysAllocStringLen(core::ptr::null(), (size - 1) as u32);
    if (*pbstrOut).is_null() {
        return E_OUTOFMEMORY;
    }
    size = GetLocaleInfoW(LOCALE_USER_DEFAULT, localeValue, *pbstrOut, size);
    if size == 0 {
        ERR!(
            b"GetLocaleInfo 0x%x failed in 2nd stage?!\n\0".as_ptr() as LPCSTR,
            localeValue,
        );
        SysFreeString(*pbstrOut);
        return HRESULT_FROM_WIN32(GetLastError());
    }
    S_OK
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Variant formatting functions
//  *
//  * Copyright 2008 Damjan Jovanovic
//  * Copyright 2003 Jon Griffiths
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  * NOTES
//  *  Since the formatting functions aren't properly documented, I used the
//  *  Visual Basic documentation as a guide to implementing these functions. This
//  *  means that some named or user-defined formats may work slightly differently.
//  *  Please submit a test case if you find a difference.
//  */
//
// #include <string.h>
// #include <stdlib.h>
// #include <stdarg.h>
// #include <stdio.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "winerror.h"
// #include "variant.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(variant);
//
// /* Make sure internal conversions to strings use the '.','+'/'-' and ','
//  * format chars from the US locale. This enables us to parse the created
//  * strings to determine the number of decimal places, exponent, etc.
//  */
// #define LCID_US MAKELCID(MAKELANGID(LANG_ENGLISH,SUBLANG_ENGLISH_US),SORT_DEFAULT)
//
// static const WCHAR szPercent_d[] = { '%','d','\0' };
// static const WCHAR szPercentZeroTwo_d[] = { '%','0','2','d','\0' };
// static const WCHAR szPercentZeroStar_d[] = { '%','0','*','d','\0' };
//
// /******************************************************************************
//  * Variant-Formats {OLEAUT32}
//  *
//  * NOTES
//  *  When formatting a variant a variety of format strings may be used to generate
//  *  different kinds of formatted output. A format string consists of either a named
//  *  format, or a user-defined format.
//  *
//  *  The following named formats are defined:
//  *| Name           Description
//  *| ----           -----------
//  *| General Date   Display Date, and time for non-integer values
//  *| Short Date     Short date format as defined by locale settings
//  *| Medium Date    Medium date format as defined by locale settings
//  *| Long Date      Long date format as defined by locale settings
//  *| Short Time     Short Time format as defined by locale settings
//  *| Medium Time    Medium time format as defined by locale settings
//  *| Long Time      Long time format as defined by locale settings
//  *| True/False     Localised text of "True" or "False"
//  *| Yes/No         Localised text of "Yes" or "No"
//  *| On/Off         Localised text of "On" or "Off"
//  *| General Number No thousands separator. No decimal points for integers
//  *| Currency       General currency format using localised characters
//  *| Fixed          At least one whole and two fractional digits
//  *| Standard       Same as 'Fixed', but including decimal separators
//  *| Percent        Multiply by 100 and display a trailing '%' character
//  *| Scientific     Display with exponent
//  *
//  *  User-defined formats consist of a combination of tokens and literal
//  *  characters. Literal characters are copied unmodified to the formatted
//  *  output at the position they occupy in the format string. Any character
//  *  that is not recognised as a token is treated as a literal. A literal can
//  *  also be specified by preceding it with a backslash character
//  *  (e.g. "\L\i\t\e\r\a\l") or enclosing it in double quotes.
//  *
//  *  A user-defined format can have up to 4 sections, depending on the type of
//  *  format. The following table lists sections and their meaning:
//  *| Format Type  Sections Meaning
//  *| -----------  -------- -------
//  *| Number       1        Use the same format for all numbers
//  *| Number       2        Use format 1 for positive and 2 for negative numbers
//  *| Number       3        Use format 1 for positive, 2 for zero, and 3
//  *|                       for negative numbers.
//  *| Number       4        Use format 1 for positive, 2 for zero, 3 for
//  *|                       negative, and 4 for null numbers.
//  *| String       1        Use the same format for all strings
//  *| String       2        Use format 2 for null and empty strings, otherwise
//  *|                       use format 1.
//  *| Date         1        Use the same format for all dates
//  *
//  *  The formatting tokens fall into several categories depending on the type
//  *  of formatted output. For more information on each type, see
//  *  VarFormat-Dates(), VarFormat-Strings() and VarFormat-Numbers().
//  *
//  *  SEE ALSO
//  *  VarTokenizeFormatString(), VarFormatFromTokens(), VarFormat(),
//  *  VarFormatDateTime(), VarFormatNumber(), VarFormatCurrency().
//  */
//
// /******************************************************************************
//  * VarFormat-Strings {OLEAUT32}
//  *
//  * NOTES
//  *  When formatting a variant as a string, it is first converted to a VT_BSTR.
//  *  The user-format string defines which characters are copied into which
//  *  positions in the output string. Literals may be inserted in the format
//  *  string. When creating the formatted string, excess characters in the string
//  *  (those not consumed by a token) are appended to the end of the output. If
//  *  there are more tokens than characters in the string to format, spaces will
//  *  be inserted at the start of the string if the '@' token was used.
//  *
//  *  By default strings are converted to lowercase, or uppercase if the '>' token
//  *  is encountered. This applies to the whole string: it is not possible to
//  *  generate a mixed-case output string.
//  *
//  *  In user-defined string formats, the following tokens are recognised:
//  *| Token  Description
//  *| -----  -----------
//  *| '@'    Copy a char from the source, or a space if no chars are left.
//  *| '&'    Copy a char from the source, or write nothing if no chars are left.
//  *| '<'    Output the whole string as lower-case (the default).
//  *| '>'    Output the whole string as upper-case.
//  *| '!'    MSDN indicates that this character should cause right-to-left
//  *|        copying, however tests show that it is tokenised but not processed.
//  */
//
// /*
//  * Common format definitions
//  */
//
//  /* Format types */
// #define FMT_TYPE_UNKNOWN 0x0
// #define FMT_TYPE_GENERAL 0x1
// #define FMT_TYPE_NUMBER  0x2
// #define FMT_TYPE_DATE    0x3
// #define FMT_TYPE_STRING  0x4
//
// #define FMT_TO_STRING    0x0 /* If header->size == this, act like VB's Str() fn */
//
// typedef struct tagFMT_SHORT_HEADER
// {
//   BYTE   size;      /* Size of tokenised block (including header), or FMT_TO_STRING */
//   BYTE   type;      /* Allowable types (FMT_TYPE_*) */
//   BYTE   offset[1]; /* Offset of the first (and only) format section */
// } FMT_SHORT_HEADER;
//
// typedef struct tagFMT_HEADER
// {
//   BYTE   size;      /* Total size of the whole tokenised block (including header) */
//   BYTE   type;      /* Allowable types (FMT_TYPE_*) */
//   BYTE   starts[4]; /* Offset of each of the 4 format sections, or 0 if none */
// } FMT_HEADER;
//
// #define FmtGetPositive(x)  (x->starts[0])
// #define FmtGetNegative(x)  (x->starts[1] ? x->starts[1] : x->starts[0])
// #define FmtGetZero(x)      (x->starts[2] ? x->starts[2] : x->starts[0])
// #define FmtGetNull(x)      (x->starts[3] ? x->starts[3] : x->starts[0])
//
// /*
//  * String formats
//  */
//
// #define FMT_FLAG_LT  0x1 /* Has '<' (lower case) */
// #define FMT_FLAG_GT  0x2 /* Has '>' (upper case) */
// #define FMT_FLAG_RTL 0x4 /* Has '!' (Copy right to left) */
//
// typedef struct tagFMT_STRING_HEADER
// {
//   BYTE   flags;      /* LT, GT, RTL */
//   BYTE   unknown1;
//   BYTE   unknown2;
//   BYTE   copy_chars; /* Number of chars to be copied */
//   BYTE   unknown3;
// } FMT_STRING_HEADER;
//
// /*
//  * Number formats
//  */
//
// #define FMT_FLAG_PERCENT   0x1  /* Has '%' (Percentage) */
// #define FMT_FLAG_EXPONENT  0x2  /* Has 'e' (Exponent/Scientific notation) */
// #define FMT_FLAG_THOUSANDS 0x4  /* Has ',' (Standard use of the thousands separator) */
// #define FMT_FLAG_BOOL      0x20 /* Boolean format */
//
// typedef struct tagFMT_NUMBER_HEADER
// {
//   BYTE   flags;      /* PERCENT, EXPONENT, THOUSANDS, BOOL */
//   BYTE   multiplier; /* Multiplier, 100 for percentages */
//   BYTE   divisor;    /* Divisor, 1000 if '%%' was used */
//   BYTE   whole;      /* Number of digits before the decimal point */
//   BYTE   fractional; /* Number of digits after the decimal point */
// } FMT_NUMBER_HEADER;
//
// /*
//  * Date Formats
//  */
// typedef struct tagFMT_DATE_HEADER
// {
//   BYTE   flags;
//   BYTE   unknown1;
//   BYTE   unknown2;
//   BYTE   unknown3;
//   BYTE   unknown4;
// } FMT_DATE_HEADER;
//
// /*
//  * Format token values
//  */
// #define FMT_GEN_COPY        0x00 /* \n, "lit" => 0,pos,len: Copy len chars from input+pos */
// #define FMT_GEN_INLINE      0x01 /*      => 1,len,[chars]: Copy len chars from token stream */
// #define FMT_GEN_END         0x02 /* \0,; => 2: End of the tokenised format */
// #define FMT_DATE_TIME_SEP   0x03 /* Time separator char */
// #define FMT_DATE_DATE_SEP   0x04 /* Date separator char */
// #define FMT_DATE_GENERAL    0x05 /* General format date */
// #define FMT_DATE_QUARTER    0x06 /* Quarter of the year from 1-4 */
// #define FMT_DATE_TIME_SYS   0x07 /* System long time format */
// #define FMT_DATE_DAY        0x08 /* Day with no leading 0 */
// #define FMT_DATE_DAY_0      0x09 /* Day with leading 0 */
// #define FMT_DATE_DAY_SHORT  0x0A /* Short day name */
// #define FMT_DATE_DAY_LONG   0x0B /* Long day name */
// #define FMT_DATE_SHORT      0x0C /* Short date format */
// #define FMT_DATE_LONG       0x0D /* Long date format */
// #define FMT_DATE_MEDIUM     0x0E /* Medium date format */
// #define FMT_DATE_DAY_WEEK   0x0F /* First day of the week */
// #define FMT_DATE_WEEK_YEAR  0x10 /* First week of the year */
// #define FMT_DATE_MON        0x11 /* Month with no leading 0 */
// #define FMT_DATE_MON_0      0x12 /* Month with leading 0 */
// #define FMT_DATE_MON_SHORT  0x13 /* Short month name */
// #define FMT_DATE_MON_LONG   0x14 /* Long month name */
// #define FMT_DATE_YEAR_DOY   0x15 /* Day of the year with no leading 0 */
// #define FMT_DATE_YEAR_0     0x16 /* 2 digit year with leading 0 */
// /* NOTE: token 0x17 is not defined, 'yyy' is not valid */
// #define FMT_DATE_YEAR_LONG  0x18 /* 4 digit year */
// #define FMT_DATE_MIN        0x1A /* Minutes with no leading 0 */
// #define FMT_DATE_MIN_0      0x1B /* Minutes with leading 0 */
// #define FMT_DATE_SEC        0x1C /* Seconds with no leading 0 */
// #define FMT_DATE_SEC_0      0x1D /* Seconds with leading 0 */
// #define FMT_DATE_HOUR       0x1E /* Hours with no leading 0 */
// #define FMT_DATE_HOUR_0     0x1F /* Hours with leading 0 */
// #define FMT_DATE_HOUR_12    0x20 /* Hours with no leading 0, 12 hour clock */
// #define FMT_DATE_HOUR_12_0  0x21 /* Hours with leading 0, 12 hour clock */
// #define FMT_DATE_TIME_UNK2  0x23 /* same as FMT_DATE_HOUR_0, for "short time" format */
// /* FIXME: probably missing some here */
// #define FMT_DATE_AMPM_SYS1  0x2E /* AM/PM as defined by system settings */
// #define FMT_DATE_AMPM_UPPER 0x2F /* Upper-case AM or PM */
// #define FMT_DATE_A_UPPER    0x30 /* Upper-case A or P */
// #define FMT_DATE_AMPM_SYS2  0x31 /* AM/PM as defined by system settings */
// #define FMT_DATE_AMPM_LOWER 0x32 /* Lower-case AM or PM */
// #define FMT_DATE_A_LOWER    0x33 /* Lower-case A or P */
// #define FMT_NUM_COPY_ZERO   0x34 /* Copy 1 digit or 0 if no digit */
// #define FMT_NUM_COPY_SKIP   0x35 /* Copy 1 digit or skip if no digit */
// #define FMT_NUM_DECIMAL     0x36 /* Decimal separator */
// #define FMT_NUM_EXP_POS_U   0x37 /* Scientific notation, uppercase, + sign */
// #define FMT_NUM_EXP_NEG_U   0x38 /* Scientific notation, uppercase, - sign */
// #define FMT_NUM_EXP_POS_L   0x39 /* Scientific notation, lowercase, + sign */
// #define FMT_NUM_EXP_NEG_L   0x3A /* Scientific notation, lowercase, - sign */
// #define FMT_NUM_CURRENCY    0x3B /* Currency symbol */
// #define FMT_NUM_TRUE_FALSE  0x3D /* Convert to "True" or "False" */
// #define FMT_NUM_YES_NO      0x3E /* Convert to "Yes" or "No" */
// #define FMT_NUM_ON_OFF      0x3F /* Convert to "On" or "Off"  */
// #define FMT_STR_COPY_SPACE  0x40 /* Copy len chars with space if no char */
// #define FMT_STR_COPY_SKIP   0x41 /* Copy len chars or skip if no char */
//
// /* Named Formats and their tokenised values */
// static const WCHAR szGeneralDate[] = { 'G','e','n','e','r','a','l',' ','D','a','t','e','\0' };
// static const BYTE fmtGeneralDate[0x0a] =
// {
//   0x0a,FMT_TYPE_DATE,sizeof(FMT_SHORT_HEADER),
//   0x0,0x0,0x0,0x0,0x0,
//   FMT_DATE_GENERAL,FMT_GEN_END
// };
//
// static const WCHAR szShortDate[] = { 'S','h','o','r','t',' ','D','a','t','e','\0' };
// static const BYTE fmtShortDate[0x0a] =
// {
//   0x0a,FMT_TYPE_DATE,sizeof(FMT_SHORT_HEADER),
//   0x0,0x0,0x0,0x0,0x0,
//   FMT_DATE_SHORT,FMT_GEN_END
// };
//
// static const WCHAR szMediumDate[] = { 'M','e','d','i','u','m',' ','D','a','t','e','\0' };
// static const BYTE fmtMediumDate[0x0a] =
// {
//   0x0a,FMT_TYPE_DATE,sizeof(FMT_SHORT_HEADER),
//   0x0,0x0,0x0,0x0,0x0,
//   FMT_DATE_MEDIUM,FMT_GEN_END
// };
//
// static const WCHAR szLongDate[] = { 'L','o','n','g',' ','D','a','t','e','\0' };
// static const BYTE fmtLongDate[0x0a] =
// {
//   0x0a,FMT_TYPE_DATE,sizeof(FMT_SHORT_HEADER),
//   0x0,0x0,0x0,0x0,0x0,
//   FMT_DATE_LONG,FMT_GEN_END
// };
//
// static const WCHAR szShortTime[] = { 'S','h','o','r','t',' ','T','i','m','e','\0' };
// static const BYTE fmtShortTime[0x0c] =
// {
//   0x0c,FMT_TYPE_DATE,sizeof(FMT_SHORT_HEADER),
//   0x0,0x0,0x0,0x0,0x0,
//   FMT_DATE_TIME_UNK2,FMT_DATE_TIME_SEP,FMT_DATE_MIN_0,FMT_GEN_END
// };
//
// static const WCHAR szMediumTime[] = { 'M','e','d','i','u','m',' ','T','i','m','e','\0' };
// static const BYTE fmtMediumTime[0x11] =
// {
//   0x11,FMT_TYPE_DATE,sizeof(FMT_SHORT_HEADER),
//   0x0,0x0,0x0,0x0,0x0,
//   FMT_DATE_HOUR_12_0,FMT_DATE_TIME_SEP,FMT_DATE_MIN_0,
//   FMT_GEN_INLINE,0x01,' ','\0',FMT_DATE_AMPM_SYS1,FMT_GEN_END
// };
//
// static const WCHAR szLongTime[] = { 'L','o','n','g',' ','T','i','m','e','\0' };
// static const BYTE fmtLongTime[0x0d] =
// {
//   0x0a,FMT_TYPE_DATE,sizeof(FMT_SHORT_HEADER),
//   0x0,0x0,0x0,0x0,0x0,
//   FMT_DATE_TIME_SYS,FMT_GEN_END
// };
//
// static const WCHAR szTrueFalse[] = { 'T','r','u','e','/','F','a','l','s','e','\0' };
// static const BYTE fmtTrueFalse[0x0d] =
// {
//   0x0d,FMT_TYPE_NUMBER,sizeof(FMT_HEADER),0x0,0x0,0x0,
//   FMT_FLAG_BOOL,0x0,0x0,0x0,0x0,
//   FMT_NUM_TRUE_FALSE,FMT_GEN_END
// };
//
// static const WCHAR szYesNo[] = { 'Y','e','s','/','N','o','\0' };
// static const BYTE fmtYesNo[0x0d] =
// {
//   0x0d,FMT_TYPE_NUMBER,sizeof(FMT_HEADER),0x0,0x0,0x0,
//   FMT_FLAG_BOOL,0x0,0x0,0x0,0x0,
//   FMT_NUM_YES_NO,FMT_GEN_END
// };
//
// static const WCHAR szOnOff[] = { 'O','n','/','O','f','f','\0' };
// static const BYTE fmtOnOff[0x0d] =
// {
//   0x0d,FMT_TYPE_NUMBER,sizeof(FMT_HEADER),0x0,0x0,0x0,
//   FMT_FLAG_BOOL,0x0,0x0,0x0,0x0,
//   FMT_NUM_ON_OFF,FMT_GEN_END
// };
//
// static const WCHAR szGeneralNumber[] = { 'G','e','n','e','r','a','l',' ','N','u','m','b','e','r','\0' };
// static const BYTE fmtGeneralNumber[sizeof(FMT_HEADER)] =
// {
//   sizeof(FMT_HEADER),FMT_TYPE_GENERAL,sizeof(FMT_HEADER),0x0,0x0,0x0
// };
//
// static const WCHAR szCurrency[] = { 'C','u','r','r','e','n','c','y','\0' };
// static const BYTE fmtCurrency[0x26] =
// {
//   0x26,FMT_TYPE_NUMBER,sizeof(FMT_HEADER),0x12,0x0,0x0,
//   /* Positive numbers */
//   FMT_FLAG_THOUSANDS,0xcc,0x0,0x1,0x2,
//   FMT_NUM_CURRENCY,FMT_NUM_COPY_ZERO,0x1,FMT_NUM_DECIMAL,FMT_NUM_COPY_ZERO,0x2,
//   FMT_GEN_END,
//   /* Negative numbers */
//   FMT_FLAG_THOUSANDS,0xcc,0x0,0x1,0x2,
//   FMT_GEN_INLINE,0x1,'(','\0',FMT_NUM_CURRENCY,FMT_NUM_COPY_ZERO,0x1,
//   FMT_NUM_DECIMAL,FMT_NUM_COPY_ZERO,0x2,FMT_GEN_INLINE,0x1,')','\0',
//   FMT_GEN_END
// };
//
// static const WCHAR szFixed[] = { 'F','i','x','e','d','\0' };
// static const BYTE fmtFixed[0x11] =
// {
//   0x11,FMT_TYPE_NUMBER,sizeof(FMT_HEADER),0x0,0x0,0x0,
//   0x0,0x0,0x0,0x1,0x2,
//   FMT_NUM_COPY_ZERO,0x1,FMT_NUM_DECIMAL,FMT_NUM_COPY_ZERO,0x2,FMT_GEN_END
// };
//
// static const WCHAR szStandard[] = { 'S','t','a','n','d','a','r','d','\0' };
// static const BYTE fmtStandard[0x11] =
// {
//   0x11,FMT_TYPE_NUMBER,sizeof(FMT_HEADER),0x0,0x0,0x0,
//   FMT_FLAG_THOUSANDS,0x0,0x0,0x1,0x2,
//   FMT_NUM_COPY_ZERO,0x1,FMT_NUM_DECIMAL,FMT_NUM_COPY_ZERO,0x2,FMT_GEN_END
// };
//
// static const WCHAR szPercent[] = { 'P','e','r','c','e','n','t','\0' };
// static const BYTE fmtPercent[0x15] =
// {
//   0x15,FMT_TYPE_NUMBER,sizeof(FMT_HEADER),0x0,0x0,0x0,
//   FMT_FLAG_PERCENT,0x1,0x0,0x1,0x2,
//   FMT_NUM_COPY_ZERO,0x1,FMT_NUM_DECIMAL,FMT_NUM_COPY_ZERO,0x2,
//   FMT_GEN_INLINE,0x1,'%','\0',FMT_GEN_END
// };
//
// static const WCHAR szScientific[] = { 'S','c','i','e','n','t','i','f','i','c','\0' };
// static const BYTE fmtScientific[0x13] =
// {
//   0x13,FMT_TYPE_NUMBER,sizeof(FMT_HEADER),0x0,0x0,0x0,
//   FMT_FLAG_EXPONENT,0x0,0x0,0x1,0x2,
//   FMT_NUM_COPY_ZERO,0x1,FMT_NUM_DECIMAL,FMT_NUM_COPY_ZERO,0x2,FMT_NUM_EXP_POS_U,0x2,FMT_GEN_END
// };
//
// typedef struct tagNAMED_FORMAT
// {
//   LPCWSTR name;
//   const BYTE* format;
// } NAMED_FORMAT;
//
// /* Format name to tokenised format. Must be kept sorted by name */
// static const NAMED_FORMAT VARIANT_NamedFormats[] =
// {
//   { szCurrency, fmtCurrency },
//   { szFixed, fmtFixed },
//   { szGeneralDate, fmtGeneralDate },
//   { szGeneralNumber, fmtGeneralNumber },
//   { szLongDate, fmtLongDate },
//   { szLongTime, fmtLongTime },
//   { szMediumDate, fmtMediumDate },
//   { szMediumTime, fmtMediumTime },
//   { szOnOff, fmtOnOff },
//   { szPercent, fmtPercent },
//   { szScientific, fmtScientific },
//   { szShortDate, fmtShortDate },
//   { szShortTime, fmtShortTime },
//   { szStandard, fmtStandard },
//   { szTrueFalse, fmtTrueFalse },
//   { szYesNo, fmtYesNo }
// };
// typedef const NAMED_FORMAT *LPCNAMED_FORMAT;
//
// static int __cdecl FormatCompareFn(const void *l, const void *r)
// {
//   return wcsicmp(((LPCNAMED_FORMAT)l)->name, ((LPCNAMED_FORMAT)r)->name);
// }
//
// static inline const BYTE *VARIANT_GetNamedFormat(LPCWSTR lpszFormat)
// {
//   NAMED_FORMAT key;
//   LPCNAMED_FORMAT fmt;
//
//   key.name = lpszFormat;
//   fmt = bsearch(&key, VARIANT_NamedFormats, ARRAY_SIZE(VARIANT_NamedFormats),
//                                  sizeof(NAMED_FORMAT), FormatCompareFn);
//   return fmt ? fmt->format : NULL;
// }
//
// /* Return an error if the token for the value will not fit in the destination */
// #define NEED_SPACE(x) if (cbTok < (int)(x)) return TYPE_E_BUFFERTOOSMALL; cbTok -= (x)
//
// /* Non-zero if the format is unknown or a given type */
// #define COULD_BE(typ) ((!fmt_number && header->type==FMT_TYPE_UNKNOWN)||header->type==typ)
//
// /* State during tokenising */
// #define FMT_STATE_OPEN_COPY     0x1 /* Last token written was a copy */
// #define FMT_STATE_WROTE_DECIMAL 0x2 /* Already wrote a decimal separator */
// #define FMT_STATE_SEEN_HOURS    0x4 /* See the hh specifier */
// #define FMT_STATE_WROTE_MINUTES 0x8 /* Wrote minutes */
//
// /**********************************************************************
//  *              VarTokenizeFormatString [OLEAUT32.140]
//  *
//  * Convert a format string into tokenised form.
//  *
//  * PARAMS
//  *  lpszFormat [I] Format string to tokenise
//  *  rgbTok     [O] Destination for tokenised format
//  *  cbTok      [I] Size of rgbTok in bytes
//  *  nFirstDay  [I] First day of the week (1-7, or 0 for current system default)
//  *  nFirstWeek [I] How to treat the first week (see notes)
//  *  lcid       [I] Locale Id of the format string
//  *  pcbActual  [O] If non-NULL, filled with the first token generated
//  *
//  * RETURNS
//  *  Success: S_OK. rgbTok contains the tokenised format.
//  *  Failure: E_INVALIDARG, if any argument is invalid.
//  *           TYPE_E_BUFFERTOOSMALL, if rgbTok is not large enough.
//  *
//  * NOTES
//  * Valid values for the nFirstWeek parameter are:
//  *| Value  Meaning
//  *| -----  -------
//  *|   0    Use the current system default
//  *|   1    The first week is that containing Jan 1
//  *|   2    Four or more days of the first week are in the current year
//  *|   3    The first week is 7 days long
//  * See Variant-Formats(), VarFormatFromTokens().
//  */
// HRESULT WINAPI VarTokenizeFormatString(LPOLESTR lpszFormat, LPBYTE rgbTok,
//                                        int cbTok, int nFirstDay, int nFirstWeek,
//                                        LCID lcid, int *pcbActual)
// {
//   /* Note: none of these strings should be NUL terminated */
//   static const WCHAR szTTTTT[] = { 't','t','t','t','t' };
//   static const WCHAR szAMPM[] = { 'A','M','P','M' };
//   static const WCHAR szampm[] = { 'a','m','p','m' };
//   static const WCHAR szAMSlashPM[] = { 'A','M','/','P','M' };
//   static const WCHAR szamSlashpm[] = { 'a','m','/','p','m' };
//   const BYTE *namedFmt;
//   FMT_HEADER *header = (FMT_HEADER*)rgbTok;
//   FMT_STRING_HEADER *str_header = (FMT_STRING_HEADER*)(rgbTok + sizeof(FMT_HEADER));
//   FMT_NUMBER_HEADER *num_header = (FMT_NUMBER_HEADER*)str_header;
//   BYTE* pOut = rgbTok + sizeof(FMT_HEADER) + sizeof(FMT_STRING_HEADER);
//   BYTE* pLastHours = NULL;
//   BYTE fmt_number = 0;
//   DWORD fmt_state = 0;
//   LPCWSTR pFormat = lpszFormat;
//
//   TRACE("(%s,%p,%d,%d,%d,0x%08x,%p)\n", debugstr_w(lpszFormat), rgbTok, cbTok,
//         nFirstDay, nFirstWeek, lcid, pcbActual);
//
//   if (!rgbTok ||
//       nFirstDay < 0 || nFirstDay > 7 || nFirstWeek < 0 || nFirstWeek > 3)
//     return E_INVALIDARG;
//
//   if (!lpszFormat || !*lpszFormat)
//   {
//     /* An empty string means 'general format' */
//     NEED_SPACE(sizeof(BYTE));
//     *rgbTok = FMT_TO_STRING;
//     if (pcbActual)
//       *pcbActual = FMT_TO_STRING;
//     return S_OK;
//   }
//
//   if (cbTok > 255)
//     cbTok = 255; /* Ensure we error instead of wrapping */
//
//   /* Named formats */
//   namedFmt = VARIANT_GetNamedFormat(lpszFormat);
//   if (namedFmt)
//   {
//     NEED_SPACE(namedFmt[0]);
//     memcpy(rgbTok, namedFmt, namedFmt[0]);
//     TRACE("Using pre-tokenised named format %s\n", debugstr_w(lpszFormat));
//     /* FIXME: pcbActual */
//     return S_OK;
//   }
//
//   /* Insert header */
//   NEED_SPACE(sizeof(FMT_HEADER) + sizeof(FMT_STRING_HEADER));
//   memset(header, 0, sizeof(FMT_HEADER));
//   memset(str_header, 0, sizeof(FMT_STRING_HEADER));
//
//   header->starts[fmt_number] = sizeof(FMT_HEADER);
//
//   while (*pFormat)
//   {
//     /* --------------
//      * General tokens
//      * --------------
//      */
//     if (*pFormat == ';')
//     {
//       while (*pFormat == ';')
//       {
//         TRACE(";\n");
//         if (++fmt_number > 3)
//           return E_INVALIDARG; /* too many formats */
//         pFormat++;
//       }
//       if (*pFormat)
//       {
//         TRACE("New header\n");
//         NEED_SPACE(sizeof(BYTE) + sizeof(FMT_STRING_HEADER));
//         *pOut++ = FMT_GEN_END;
//
//         header->starts[fmt_number] = pOut - rgbTok;
//         str_header = (FMT_STRING_HEADER*)pOut;
//         num_header = (FMT_NUMBER_HEADER*)pOut;
//         memset(str_header, 0, sizeof(FMT_STRING_HEADER));
//         pOut += sizeof(FMT_STRING_HEADER);
//         fmt_state = 0;
//         pLastHours = NULL;
//       }
//     }
//     else if (*pFormat == '\\')
//     {
//       /* Escaped character */
//       if (pFormat[1])
//       {
//         NEED_SPACE(3 * sizeof(BYTE));
//         pFormat++;
//         *pOut++ = FMT_GEN_COPY;
//         *pOut++ = pFormat - lpszFormat;
//         *pOut++ = 0x1;
//         fmt_state |= FMT_STATE_OPEN_COPY;
//         TRACE("'\\'\n");
//       }
//       else
//         fmt_state &= ~FMT_STATE_OPEN_COPY;
//       pFormat++;
//     }
//     else if (*pFormat == '"')
//     {
//       /* Escaped string
//        * Note: Native encodes "" as a copy of length zero. That's just dumb, so
//        * here we avoid encoding anything in this case.
//        */
//       if (!pFormat[1])
//         pFormat++;
//       else if (pFormat[1] == '"')
//       {
//         pFormat += 2;
//       }
//       else
//       {
//         LPCWSTR start = ++pFormat;
//         while (*pFormat && *pFormat != '"')
//           pFormat++;
//         NEED_SPACE(3 * sizeof(BYTE));
//         *pOut++ = FMT_GEN_COPY;
//         *pOut++ = start - lpszFormat;
//         *pOut++ = pFormat - start;
//         if (*pFormat == '"')
//           pFormat++;
//         TRACE("Quoted string pos %d, len %d\n", pOut[-2], pOut[-1]);
//       }
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     /* -------------
//      * Number tokens
//      * -------------
//      */
//     else if (*pFormat == '0' && COULD_BE(FMT_TYPE_NUMBER))
//     {
//       /* Number formats: Digit from number or '0' if no digits
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_NUMBER;
//       NEED_SPACE(2 * sizeof(BYTE));
//       *pOut++ = FMT_NUM_COPY_ZERO;
//       *pOut = 0x0;
//       while (*pFormat == '0')
//       {
//         *pOut = *pOut + 1;
//         pFormat++;
//       }
//       if (fmt_state & FMT_STATE_WROTE_DECIMAL)
//         num_header->fractional += *pOut;
//       else
//         num_header->whole += *pOut;
//       TRACE("%d 0's\n", *pOut);
//       pOut++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if (*pFormat == '#' && COULD_BE(FMT_TYPE_NUMBER))
//     {
//       /* Number formats: Digit from number or blank if no digits
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_NUMBER;
//       NEED_SPACE(2 * sizeof(BYTE));
//       *pOut++ = FMT_NUM_COPY_SKIP;
//       *pOut = 0x0;
//       while (*pFormat == '#')
//       {
//         *pOut = *pOut + 1;
//         pFormat++;
//       }
//       if (fmt_state & FMT_STATE_WROTE_DECIMAL)
//         num_header->fractional += *pOut;
//       else
//         num_header->whole += *pOut;
//       TRACE("%d #'s\n", *pOut);
//       pOut++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if (*pFormat == '.' && COULD_BE(FMT_TYPE_NUMBER) &&
//               !(fmt_state & FMT_STATE_WROTE_DECIMAL))
//     {
//       /* Number formats: Decimal separator when 1st seen, literal thereafter
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_NUMBER;
//       NEED_SPACE(sizeof(BYTE));
//       *pOut++ = FMT_NUM_DECIMAL;
//       fmt_state |= FMT_STATE_WROTE_DECIMAL;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       pFormat++;
//       TRACE("decimal sep\n");
//     }
//     else if ((*pFormat == 'e' || *pFormat == 'E') && (pFormat[1] == '-' ||
//               pFormat[1] == '+') && header->type == FMT_TYPE_NUMBER)
//     {
//       /* Number formats: Exponent specifier
//        * Other formats: Literal
//        */
//       num_header->flags |= FMT_FLAG_EXPONENT;
//       NEED_SPACE(2 * sizeof(BYTE));
//       if (*pFormat == 'e') {
//         if (pFormat[1] == '+')
//           *pOut = FMT_NUM_EXP_POS_L;
//         else
//           *pOut = FMT_NUM_EXP_NEG_L;
//       } else {
//         if (pFormat[1] == '+')
//           *pOut = FMT_NUM_EXP_POS_U;
//         else
//           *pOut = FMT_NUM_EXP_NEG_U;
//       }
//       pFormat += 2;
//       *++pOut = 0x0;
//       while (*pFormat == '0')
//       {
//         *pOut = *pOut + 1;
//         pFormat++;
//       }
//       pOut++;
//       TRACE("exponent\n");
//     }
//     /* FIXME: %% => Divide by 1000 */
//     else if (*pFormat == ',' && header->type == FMT_TYPE_NUMBER)
//     {
//       /* Number formats: Use the thousands separator
//        * Other formats: Literal
//        */
//       num_header->flags |= FMT_FLAG_THOUSANDS;
//       pFormat++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       TRACE("thousands sep\n");
//     }
//     /* -----------
//      * Date tokens
//      * -----------
//      */
//     else if (*pFormat == '/' && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Date separator
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       *pOut++ = FMT_DATE_DATE_SEP;
//       pFormat++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       TRACE("date sep\n");
//     }
//     else if (*pFormat == ':' && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Time separator
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       *pOut++ = FMT_DATE_TIME_SEP;
//       pFormat++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       TRACE("time sep\n");
//     }
//     else if ((*pFormat == 'a' || *pFormat == 'A') &&
//               !_wcsnicmp(pFormat, szAMPM, ARRAY_SIZE(szAMPM)))
//     {
//       /* Date formats: System AM/PM designation
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat += ARRAY_SIZE(szAMPM);
//       if (!wcsncmp(pFormat, szampm, ARRAY_SIZE(szampm)))
//         *pOut++ = FMT_DATE_AMPM_SYS2;
//       else
//         *pOut++ = FMT_DATE_AMPM_SYS1;
//       if (pLastHours)
//         *pLastHours = *pLastHours + 2;
//       TRACE("ampm\n");
//     }
//     else if (*pFormat == 'a' && pFormat[1] == '/' &&
//               (pFormat[2] == 'p' || pFormat[2] == 'P'))
//     {
//       /* Date formats: lowercase a or p designation
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat += 3;
//       *pOut++ = FMT_DATE_A_LOWER;
//       if (pLastHours)
//         *pLastHours = *pLastHours + 2;
//       TRACE("a/p\n");
//     }
//     else if (*pFormat == 'A' && pFormat[1] == '/' &&
//               (pFormat[2] == 'p' || pFormat[2] == 'P'))
//     {
//       /* Date formats: Uppercase a or p designation
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat += 3;
//       *pOut++ = FMT_DATE_A_UPPER;
//       if (pLastHours)
//         *pLastHours = *pLastHours + 2;
//       TRACE("A/P\n");
//     }
//     else if (*pFormat == 'a' && !wcsncmp(pFormat, szamSlashpm, ARRAY_SIZE(szamSlashpm)))
//     {
//       /* Date formats: lowercase AM or PM designation
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat += ARRAY_SIZE(szamSlashpm);
//       *pOut++ = FMT_DATE_AMPM_LOWER;
//       if (pLastHours)
//         *pLastHours = *pLastHours + 2;
//       TRACE("AM/PM\n");
//     }
//     else if (*pFormat == 'A' && !wcsncmp(pFormat, szAMSlashPM, ARRAY_SIZE(szAMSlashPM)))
//     {
//       /* Date formats: Uppercase AM or PM designation
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat += ARRAY_SIZE(szAMSlashPM);
//       *pOut++ = FMT_DATE_AMPM_UPPER;
//       TRACE("AM/PM\n");
//     }
//     else if ((*pFormat == 'c' || *pFormat == 'C') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: General date format
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat += ARRAY_SIZE(szAMSlashPM);
//       *pOut++ = FMT_DATE_GENERAL;
//       TRACE("gen date\n");
//     }
//     else if ((*pFormat == 'd' || *pFormat == 'D') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Day specifier
//        * Other formats: Literal
//        * Types the format if found
//        */
//       int count = -1;
//       header->type = FMT_TYPE_DATE;
//       while ((*pFormat == 'd' || *pFormat == 'D') && count < 6)
//       {
//         pFormat++;
//         count++;
//       }
//       NEED_SPACE(sizeof(BYTE));
//       *pOut++ = FMT_DATE_DAY + count;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       /* When we find the days token, reset the seen hours state so that
//        * 'mm' is again written as month when encountered.
//        */
//       fmt_state &= ~FMT_STATE_SEEN_HOURS;
//       TRACE("%d d's\n", count + 1);
//     }
//     else if ((*pFormat == 'h' || *pFormat == 'H') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Hour specifier
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat++;
//       /* Record the position of the hours specifier - if we encounter
//        * an am/pm specifier we will change the hours from 24 to 12.
//        */
//       pLastHours = pOut;
//       if (*pFormat == 'h' || *pFormat == 'H')
//       {
//         pFormat++;
//         *pOut++ = FMT_DATE_HOUR_0;
//         TRACE("hh\n");
//       }
//       else
//       {
//         *pOut++ = FMT_DATE_HOUR;
//         TRACE("h\n");
//       }
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       /* Note that now we have seen an hours token, the next occurrence of
//        * 'mm' indicates minutes, not months.
//        */
//       fmt_state |= FMT_STATE_SEEN_HOURS;
//     }
//     else if ((*pFormat == 'm' || *pFormat == 'M') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Month specifier (or Minute specifier, after hour specifier)
//        * Other formats: Literal
//        * Types the format if found
//        */
//       int count = -1;
//       header->type = FMT_TYPE_DATE;
//       while ((*pFormat == 'm' || *pFormat == 'M') && count < 4)
//       {
//         pFormat++;
//         count++;
//       }
//       NEED_SPACE(sizeof(BYTE));
//       if (count <= 1 && fmt_state & FMT_STATE_SEEN_HOURS &&
//           !(fmt_state & FMT_STATE_WROTE_MINUTES))
//       {
//         /* We have seen an hours specifier and not yet written a minutes
//          * specifier. Write this as minutes and thereafter as months.
//          */
//         *pOut++ = count == 1 ? FMT_DATE_MIN_0 : FMT_DATE_MIN;
//         fmt_state |= FMT_STATE_WROTE_MINUTES; /* Hereafter write months */
//       }
//       else
//         *pOut++ = FMT_DATE_MON + count; /* Months */
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       TRACE("%d m's\n", count + 1);
//     }
//     else if ((*pFormat == 'n' || *pFormat == 'N') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Minute specifier
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat++;
//       if (*pFormat == 'n' || *pFormat == 'N')
//       {
//         pFormat++;
//         *pOut++ = FMT_DATE_MIN_0;
//         TRACE("nn\n");
//       }
//       else
//       {
//         *pOut++ = FMT_DATE_MIN;
//         TRACE("n\n");
//       }
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if ((*pFormat == 'q' || *pFormat == 'Q') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Quarter specifier
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       *pOut++ = FMT_DATE_QUARTER;
//       pFormat++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       TRACE("quarter\n");
//     }
//     else if ((*pFormat == 's' || *pFormat == 'S') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Second specifier
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       NEED_SPACE(sizeof(BYTE));
//       pFormat++;
//       if (*pFormat == 's' || *pFormat == 'S')
//       {
//         pFormat++;
//         *pOut++ = FMT_DATE_SEC_0;
//         TRACE("ss\n");
//       }
//       else
//       {
//         *pOut++ = FMT_DATE_SEC;
//         TRACE("s\n");
//       }
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if ((*pFormat == 't' || *pFormat == 'T') &&
//               !_wcsnicmp(pFormat, szTTTTT, ARRAY_SIZE(szTTTTT)))
//     {
//       /* Date formats: System time specifier
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       pFormat += ARRAY_SIZE(szTTTTT);
//       NEED_SPACE(sizeof(BYTE));
//       *pOut++ = FMT_DATE_TIME_SYS;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if ((*pFormat == 'w' || *pFormat == 'W') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Week of the year/Day of the week
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_DATE;
//       pFormat++;
//       if (*pFormat == 'w' || *pFormat == 'W')
//       {
//         NEED_SPACE(3 * sizeof(BYTE));
//         pFormat++;
//         *pOut++ = FMT_DATE_WEEK_YEAR;
//         *pOut++ = nFirstDay;
//         *pOut++ = nFirstWeek;
//         TRACE("ww\n");
//       }
//       else
//       {
//         NEED_SPACE(2 * sizeof(BYTE));
//         *pOut++ = FMT_DATE_DAY_WEEK;
//         *pOut++ = nFirstDay;
//         TRACE("w\n");
//       }
//
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if ((*pFormat == 'y' || *pFormat == 'Y') && COULD_BE(FMT_TYPE_DATE))
//     {
//       /* Date formats: Day of year/Year specifier
//        * Other formats: Literal
//        * Types the format if found
//        */
//       int count = -1;
//       header->type = FMT_TYPE_DATE;
//       while ((*pFormat == 'y' || *pFormat == 'Y') && count < 4)
//       {
//         pFormat++;
//         count++;
//       }
//       if (count == 2)
//       {
//         count--; /* 'yyy' has no meaning, despite what MSDN says */
//         pFormat--;
//       }
//       NEED_SPACE(sizeof(BYTE));
//       *pOut++ = FMT_DATE_YEAR_DOY + count;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       TRACE("%d y's\n", count + 1);
//     }
//     /* -------------
//      * String tokens
//      * -------------
//      */
//     else if (*pFormat == '@' && COULD_BE(FMT_TYPE_STRING))
//     {
//       /* String formats: Character from string or space if no char
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_STRING;
//       NEED_SPACE(2 * sizeof(BYTE));
//       *pOut++ = FMT_STR_COPY_SPACE;
//       *pOut = 0x0;
//       while (*pFormat == '@')
//       {
//         *pOut = *pOut + 1;
//         str_header->copy_chars++;
//         pFormat++;
//       }
//       TRACE("%d @'s\n", *pOut);
//       pOut++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if (*pFormat == '&' && COULD_BE(FMT_TYPE_STRING))
//     {
//       /* String formats: Character from string or skip if no char
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_STRING;
//       NEED_SPACE(2 * sizeof(BYTE));
//       *pOut++ = FMT_STR_COPY_SKIP;
//       *pOut = 0x0;
//       while (*pFormat == '&')
//       {
//         *pOut = *pOut + 1;
//         str_header->copy_chars++;
//         pFormat++;
//       }
//       TRACE("%d &'s\n", *pOut);
//       pOut++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if ((*pFormat == '<' || *pFormat == '>') && COULD_BE(FMT_TYPE_STRING))
//     {
//       /* String formats: Use upper/lower case
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_STRING;
//       if (*pFormat == '<')
//         str_header->flags |= FMT_FLAG_LT;
//       else
//         str_header->flags |= FMT_FLAG_GT;
//       TRACE("to %s case\n", *pFormat == '<' ? "lower" : "upper");
//       pFormat++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//     }
//     else if (*pFormat == '!' && COULD_BE(FMT_TYPE_STRING))
//     {
//       /* String formats: Copy right to left
//        * Other formats: Literal
//        * Types the format if found
//        */
//       header->type = FMT_TYPE_STRING;
//       str_header->flags |= FMT_FLAG_RTL;
//       pFormat++;
//       fmt_state &= ~FMT_STATE_OPEN_COPY;
//       TRACE("copy right-to-left\n");
//     }
//     /* --------
//      * Literals
//      * --------
//      */
//     /* FIXME: [ seems to be ignored */
//     else
//     {
//       if (*pFormat == '%' && header->type == FMT_TYPE_NUMBER)
//       {
//         /* Number formats: Percentage indicator, also a literal
//          * Other formats: Literal
//          * Doesn't type the format
//          */
//         num_header->flags |= FMT_FLAG_PERCENT;
//       }
//
//       if (fmt_state & FMT_STATE_OPEN_COPY)
//       {
//         pOut[-1] = pOut[-1] + 1; /* Increase the length of the open copy */
//         TRACE("extend copy (char '%c'), length now %d\n", *pFormat, pOut[-1]);
//       }
//       else
//       {
//         /* Create a new open copy */
//         TRACE("New copy (char '%c')\n", *pFormat);
//         NEED_SPACE(3 * sizeof(BYTE));
//         *pOut++ = FMT_GEN_COPY;
//         *pOut++ = pFormat - lpszFormat;
//         *pOut++ = 0x1;
//         fmt_state |= FMT_STATE_OPEN_COPY;
//       }
//       pFormat++;
//     }
//   }
//
//   *pOut++ = FMT_GEN_END;
//
//   header->size = pOut - rgbTok;
//   if (pcbActual)
//     *pcbActual = header->size;
//
//   return S_OK;
// }
//
// /* Number formatting state flags */
// #define NUM_WROTE_DEC  0x01 /* Written the decimal separator */
// #define NUM_WRITE_ON   0x02 /* Started to write the number */
// #define NUM_WROTE_SIGN 0x04 /* Written the negative sign */
//
// /* Format a variant using a number format */
// static HRESULT VARIANT_FormatNumber(LPVARIANT pVarIn, LPOLESTR lpszFormat,
//                                     LPBYTE rgbTok, ULONG dwFlags,
//                                     BSTR *pbstrOut, LCID lcid)
// {
//   BYTE rgbDig[256], *prgbDig;
//   NUMPARSE np;
//   int have_int, need_int = 0, have_frac, need_frac, exponent = 0, pad = 0;
//   WCHAR buff[256], *pBuff = buff;
//   WCHAR thousandSeparator[32];
//   VARIANT vString, vBool;
//   DWORD dwState = 0;
//   FMT_HEADER *header = (FMT_HEADER*)rgbTok;
//   FMT_NUMBER_HEADER *numHeader;
//   const BYTE* pToken = NULL;
//   HRESULT hRes = S_OK;
//
//   TRACE("(%s,%s,%p,0x%08x,%p,0x%08x)\n", debugstr_variant(pVarIn), debugstr_w(lpszFormat),
//         rgbTok, dwFlags, pbstrOut, lcid);
//
//   V_VT(&vString) = VT_EMPTY;
//   V_VT(&vBool) = VT_BOOL;
//
//   if (V_TYPE(pVarIn) == VT_EMPTY || V_TYPE(pVarIn) == VT_NULL)
//   {
//     have_int = have_frac = 0;
//     numHeader = (FMT_NUMBER_HEADER*)(rgbTok + FmtGetNull(header));
//     V_BOOL(&vBool) = VARIANT_FALSE;
//   }
//   else
//   {
//     /* Get a number string from pVarIn, and parse it */
//     hRes = VariantChangeTypeEx(&vString, pVarIn, lcid, VARIANT_NOUSEROVERRIDE, VT_BSTR);
//     if (FAILED(hRes))
//       return hRes;
//
//     np.cDig = sizeof(rgbDig);
//     np.dwInFlags = NUMPRS_STD;
//     hRes = VarParseNumFromStr(V_BSTR(&vString), lcid, 0, &np, rgbDig);
//     if (FAILED(hRes))
//       return hRes;
//
//     have_int = np.cDig;
//     have_frac = 0;
//     exponent = np.nPwr10;
//
//     /* Figure out which format to use */
//     if (np.dwOutFlags & NUMPRS_NEG)
//     {
//       numHeader = (FMT_NUMBER_HEADER*)(rgbTok + FmtGetNegative(header));
//       V_BOOL(&vBool) = VARIANT_TRUE;
//     }
//     else if (have_int == 1 && !exponent && rgbDig[0] == 0)
//     {
//       numHeader = (FMT_NUMBER_HEADER*)(rgbTok + FmtGetZero(header));
//       V_BOOL(&vBool) = VARIANT_FALSE;
//     }
//     else
//     {
//       numHeader = (FMT_NUMBER_HEADER*)(rgbTok + FmtGetPositive(header));
//       V_BOOL(&vBool) = VARIANT_TRUE;
//     }
//
//     TRACE("num header: flags = 0x%x, mult=%d, div=%d, whole=%d, fract=%d\n",
//           numHeader->flags, numHeader->multiplier, numHeader->divisor,
//           numHeader->whole, numHeader->fractional);
//
//     need_int = numHeader->whole;
//     need_frac = numHeader->fractional;
//
//     if (numHeader->flags & FMT_FLAG_PERCENT &&
//         !(have_int == 1 && !exponent && rgbDig[0] == 0))
//       exponent += 2;
//
//     if (numHeader->flags & FMT_FLAG_EXPONENT)
//     {
//       /* Exponent format: length of the integral number part is fixed and
//          specified by the format. */
//       pad = need_int - have_int;
//       exponent -= pad;
//       if (pad < 0)
//       {
//         have_int = need_int;
//         have_frac -= pad;
//         pad = 0;
//       }
//     }
//     else
//     {
//       /* Convert the exponent */
//       pad = max(exponent, -have_int);
//       exponent -= pad;
//       if (pad < 0)
//       {
//         have_int += pad;
//         have_frac = -pad;
//         pad = 0;
//       }
//       if(exponent < 0 && exponent > (-256 + have_int + have_frac))
//       {
//         /* Remove exponent notation */
//         memmove(rgbDig - exponent, rgbDig, have_int + have_frac);
//         ZeroMemory(rgbDig, -exponent);
//         have_frac -= exponent;
//         exponent = 0;
//       }
//     }
//
//     /* Rounding the number */
//     if (have_frac > need_frac)
//     {
//       prgbDig = &rgbDig[have_int + need_frac];
//       have_frac = need_frac;
//       if (*prgbDig >= 5)
//       {
//         while (prgbDig-- > rgbDig && *prgbDig == 9)
//           *prgbDig = 0;
//         if (prgbDig < rgbDig)
//         {
//           /* We reached the first digit and that was also a 9 */
//           rgbDig[0] = 1;
//           if (numHeader->flags & FMT_FLAG_EXPONENT)
//             exponent++;
//           else
//           {
//             rgbDig[have_int + need_frac] = 0;
//             if (exponent < 0)
//               exponent++;
//             else
//               have_int++;
//           }
//         }
//         else
//           (*prgbDig)++;
//       }
//       /* We converted trailing digits to zeroes => have_frac has changed */
//       while (have_frac > 0 && rgbDig[have_int + have_frac - 1] == 0)
//         have_frac--;
//     }
//     TRACE("have_int=%d,need_int=%d,have_frac=%d,need_frac=%d,pad=%d,exp=%d\n",
//           have_int, need_int, have_frac, need_frac, pad, exponent);
//   }
//
//   if (numHeader->flags & FMT_FLAG_THOUSANDS)
//   {
//     if (!GetLocaleInfoW(lcid, LOCALE_STHOUSAND, thousandSeparator, ARRAY_SIZE(thousandSeparator)))
//     {
//       thousandSeparator[0] = ',';
//       thousandSeparator[1] = 0;
//     }
//   }
//
//   pToken = (const BYTE*)numHeader + sizeof(FMT_NUMBER_HEADER);
//   prgbDig = rgbDig;
//
//   while (SUCCEEDED(hRes) && *pToken != FMT_GEN_END)
//   {
//     WCHAR defaultChar = '?';
//     DWORD boolFlag, localeValue = 0;
//     BOOL shouldAdvance = TRUE;
//
//     if (pToken - rgbTok > header->size)
//     {
//       ERR("Ran off the end of the format!\n");
//       hRes = E_INVALIDARG;
//       goto VARIANT_FormatNumber_Exit;
//     }
//
//     switch (*pToken)
//     {
//     case FMT_GEN_COPY:
//       TRACE("copy %s\n", debugstr_wn(lpszFormat + pToken[1], pToken[2]));
//       memcpy(pBuff, lpszFormat + pToken[1], pToken[2] * sizeof(WCHAR));
//       pBuff += pToken[2];
//       pToken += 2;
//       break;
//
//     case FMT_GEN_INLINE:
//       pToken += 2;
//       TRACE("copy %s\n", debugstr_a((LPCSTR)pToken));
//       while (*pToken)
//         *pBuff++ = *pToken++;
//       break;
//
//     case FMT_NUM_YES_NO:
//       boolFlag = VAR_BOOLYESNO;
//       goto VARIANT_FormatNumber_Bool;
//
//     case FMT_NUM_ON_OFF:
//       boolFlag = VAR_BOOLONOFF;
//       goto VARIANT_FormatNumber_Bool;
//
//     case FMT_NUM_TRUE_FALSE:
//       boolFlag = VAR_LOCALBOOL;
//
// VARIANT_FormatNumber_Bool:
//       {
//         BSTR boolStr = NULL;
//
//         if (pToken[1] != FMT_GEN_END)
//         {
//           ERR("Boolean token not at end of format!\n");
//           hRes = E_INVALIDARG;
//           goto VARIANT_FormatNumber_Exit;
//         }
//         hRes = VarBstrFromBool(V_BOOL(&vBool), lcid, boolFlag, &boolStr);
//         if (SUCCEEDED(hRes))
//         {
//           lstrcpyW(pBuff, boolStr);
//           SysFreeString(boolStr);
//           while (*pBuff)
//             pBuff++;
//         }
//       }
//       break;
//
//     case FMT_NUM_DECIMAL:
//       if ((np.dwOutFlags & NUMPRS_NEG) && !(dwState & NUM_WROTE_SIGN) && !header->starts[1])
//       {
//         /* last chance for a negative sign in the .# case */
//         TRACE("write negative sign\n");
//         localeValue = LOCALE_SNEGATIVESIGN;
//         defaultChar = '-';
//         dwState |= NUM_WROTE_SIGN;
//         shouldAdvance = FALSE;
//         break;
//       }
//       TRACE("write decimal separator\n");
//       localeValue = LOCALE_SDECIMAL;
//       defaultChar = '.';
//       dwState |= NUM_WROTE_DEC;
//       break;
//
//     case FMT_NUM_CURRENCY:
//       TRACE("write currency symbol\n");
//       localeValue = LOCALE_SCURRENCY;
//       defaultChar = '$';
//       break;
//
//     case FMT_NUM_EXP_POS_U:
//     case FMT_NUM_EXP_POS_L:
//     case FMT_NUM_EXP_NEG_U:
//     case FMT_NUM_EXP_NEG_L:
//       if (*pToken == FMT_NUM_EXP_POS_L || *pToken == FMT_NUM_EXP_NEG_L)
//         *pBuff++ = 'e';
//       else
//         *pBuff++ = 'E';
//       if (exponent < 0)
//       {
//         *pBuff++ = '-';
//         swprintf(pBuff, szPercentZeroStar_d, pToken[1], -exponent);
//       }
//       else
//       {
//         if (*pToken == FMT_NUM_EXP_POS_L || *pToken == FMT_NUM_EXP_POS_U)
//           *pBuff++ = '+';
//         swprintf(pBuff, szPercentZeroStar_d, pToken[1], exponent);
//       }
//       while (*pBuff)
//         pBuff++;
//       pToken++;
//       break;
//
//     case FMT_NUM_COPY_ZERO:
//       dwState |= NUM_WRITE_ON;
//       /* Fall through */
//
//     case FMT_NUM_COPY_SKIP:
//       TRACE("write %d %sdigits or %s\n", pToken[1],
//             dwState & NUM_WROTE_DEC ? "fractional " : "",
//             *pToken == FMT_NUM_COPY_ZERO ? "0" : "skip");
//
//       if (dwState & NUM_WROTE_DEC)
//       {
//         int count, i;
//
//         if (!(numHeader->flags & FMT_FLAG_EXPONENT) && exponent < 0)
//         {
//           /* Pad with 0 before writing the fractional digits */
//           pad = max(exponent, -pToken[1]);
//           exponent -= pad;
//           count = min(have_frac, pToken[1] + pad);
//           for (i = 0; i > pad; i--)
//             *pBuff++ = '0';
//         }
//         else
//           count = min(have_frac, pToken[1]);
//
//         pad += pToken[1] - count;
//         have_frac -= count;
//         while (count--)
//           *pBuff++ = '0' + *prgbDig++;
//         if (*pToken == FMT_NUM_COPY_ZERO)
//         {
//           for (; pad > 0; pad--)
//             *pBuff++ = '0'; /* Write zeros for missing trailing digits */
//         }
//       }
//       else
//       {
//         int count, count_max, position;
//
//         if ((np.dwOutFlags & NUMPRS_NEG) && !(dwState & NUM_WROTE_SIGN) && !header->starts[1])
//         {
//           TRACE("write negative sign\n");
//           localeValue = LOCALE_SNEGATIVESIGN;
//           defaultChar = '-';
//           dwState |= NUM_WROTE_SIGN;
//           shouldAdvance = FALSE;
//           break;
//         }
//
//         position = have_int + pad;
//         if (dwState & NUM_WRITE_ON)
//           position = max(position, need_int);
//         need_int -= pToken[1];
//         count_max = have_int + pad - need_int;
//         if (count_max < 0)
//             count_max = 0;
//         if (dwState & NUM_WRITE_ON)
//         {
//           count = pToken[1] - count_max;
//           TRACE("write %d leading zeros\n", count);
//           while (count-- > 0)
//           {
//             *pBuff++ = '0';
//             if ((numHeader->flags & FMT_FLAG_THOUSANDS) &&
//                 position > 1 && (--position % 3) == 0)
//             {
//               int k;
//               TRACE("write thousand separator\n");
//               for (k = 0; thousandSeparator[k]; k++)
//                 *pBuff++ = thousandSeparator[k];
//             }
//           }
//         }
//         if (*pToken == FMT_NUM_COPY_ZERO || have_int > 1 ||
//             (have_int > 0 && *prgbDig > 0))
//         {
//           count = min(count_max, have_int);
//           count_max -= count;
//           have_int -= count;
//           TRACE("write %d whole number digits\n", count);
//           while (count--)
//           {
//             dwState |= NUM_WRITE_ON;
//             *pBuff++ = '0' + *prgbDig++;
//             if ((numHeader->flags & FMT_FLAG_THOUSANDS) &&
//                 position > 1 && (--position % 3) == 0)
//             {
//               int k;
//               TRACE("write thousand separator\n");
//               for (k = 0; thousandSeparator[k]; k++)
//                 *pBuff++ = thousandSeparator[k];
//             }
//           }
//         }
//         count = min(count_max, pad);
//         pad -= count;
//         TRACE("write %d whole trailing 0's\n", count);
//         while (count--)
//         {
//           *pBuff++ = '0';
//           if ((numHeader->flags & FMT_FLAG_THOUSANDS) &&
//               position > 1 && (--position % 3) == 0)
//           {
//             int k;
//             TRACE("write thousand separator\n");
//             for (k = 0; thousandSeparator[k]; k++)
//               *pBuff++ = thousandSeparator[k];
//           }
//         }
//       }
//       pToken++;
//       break;
//
//     default:
//       ERR("Unknown token 0x%02x!\n", *pToken);
//       hRes = E_INVALIDARG;
//       goto VARIANT_FormatNumber_Exit;
//     }
//     if (localeValue)
//     {
//       if (GetLocaleInfoW(lcid, localeValue, pBuff, ARRAY_SIZE(buff)-(pBuff-buff)))
//       {
//         TRACE("added %s\n", debugstr_w(pBuff));
//         while (*pBuff)
//           pBuff++;
//       }
//       else
//       {
//         TRACE("added %d '%c'\n", defaultChar, defaultChar);
//         *pBuff++ = defaultChar;
//       }
//     }
//     if (shouldAdvance)
//       pToken++;
//   }
//
// VARIANT_FormatNumber_Exit:
//   VariantClear(&vString);
//   *pBuff = '\0';
//   TRACE("buff is %s\n", debugstr_w(buff));
//   if (SUCCEEDED(hRes))
//   {
//     *pbstrOut = SysAllocString(buff);
//     if (!*pbstrOut)
//       hRes = E_OUTOFMEMORY;
//   }
//   return hRes;
// }
//
// /* Format a variant using a date format */
// static HRESULT VARIANT_FormatDate(LPVARIANT pVarIn, LPOLESTR lpszFormat,
//                                   LPBYTE rgbTok, ULONG dwFlags,
//                                   BSTR *pbstrOut, LCID lcid)
// {
//   WCHAR buff[256], *pBuff = buff;
//   VARIANT vDate;
//   UDATE udate;
//   FMT_HEADER *header = (FMT_HEADER*)rgbTok;
//   FMT_DATE_HEADER *dateHeader;
//   const BYTE* pToken = NULL;
//   HRESULT hRes;
//
//   TRACE("(%s,%s,%p,0x%08x,%p,0x%08x)\n", debugstr_variant(pVarIn),
//         debugstr_w(lpszFormat), rgbTok, dwFlags, pbstrOut, lcid);
//
//   V_VT(&vDate) = VT_EMPTY;
//
//   if (V_TYPE(pVarIn) == VT_EMPTY || V_TYPE(pVarIn) == VT_NULL)
//   {
//     dateHeader = (FMT_DATE_HEADER*)(rgbTok + FmtGetNegative(header));
//     V_DATE(&vDate) = 0;
//   }
//   else
//   {
//     USHORT usFlags = dwFlags & VARIANT_CALENDAR_HIJRI ? VAR_CALENDAR_HIJRI : 0;
//
//     hRes = VariantChangeTypeEx(&vDate, pVarIn, lcid, usFlags, VT_DATE);
//     if (FAILED(hRes))
//       return hRes;
//     dateHeader = (FMT_DATE_HEADER*)(rgbTok + FmtGetPositive(header));
//   }
//
//   hRes = VarUdateFromDate(V_DATE(&vDate), 0 /* FIXME: flags? */, &udate);
//   if (FAILED(hRes))
//     return hRes;
//   pToken = (const BYTE*)dateHeader + sizeof(FMT_DATE_HEADER);
//
//   while (*pToken != FMT_GEN_END)
//   {
//     DWORD dwVal = 0, localeValue = 0, dwFmt = 0;
//     LPCWSTR szPrintFmt = NULL;
//     WCHAR defaultChar = '?';
//
//     if (pToken - rgbTok > header->size)
//     {
//       ERR("Ran off the end of the format!\n");
//       hRes = E_INVALIDARG;
//       goto VARIANT_FormatDate_Exit;
//     }
//
//     switch (*pToken)
//     {
//     case FMT_GEN_COPY:
//       TRACE("copy %s\n", debugstr_wn(lpszFormat + pToken[1], pToken[2]));
//       memcpy(pBuff, lpszFormat + pToken[1], pToken[2] * sizeof(WCHAR));
//       pBuff += pToken[2];
//       pToken += 2;
//       break;
//
//     case FMT_GEN_INLINE:
//       pToken += 2;
//       TRACE("copy %s\n", debugstr_a((LPCSTR)pToken));
//       while (*pToken)
//         *pBuff++ = *pToken++;
//       break;
//
//     case FMT_DATE_TIME_SEP:
//       TRACE("time separator\n");
//       localeValue = LOCALE_STIME;
//       defaultChar = ':';
//       break;
//
//     case FMT_DATE_DATE_SEP:
//       TRACE("date separator\n");
//       localeValue = LOCALE_SDATE;
//       defaultChar = '/';
//       break;
//
//     case FMT_DATE_GENERAL:
//       {
//         BSTR date = NULL;
//         WCHAR *pDate;
//         hRes = VarBstrFromDate(V_DATE(&vDate), lcid, 0, &date);
//         if (FAILED(hRes))
//           goto VARIANT_FormatDate_Exit;
// 	pDate = date;
//         while (*pDate)
//           *pBuff++ = *pDate++;
//         SysFreeString(date);
//       }
//       break;
//
//     case FMT_DATE_QUARTER:
//       if (udate.st.wMonth <= 3)
//         *pBuff++ = '1';
//       else if (udate.st.wMonth <= 6)
//         *pBuff++ = '2';
//       else if (udate.st.wMonth <= 9)
//         *pBuff++ = '3';
//       else
//         *pBuff++ = '4';
//       break;
//
//     case FMT_DATE_TIME_SYS:
//       {
//         /* FIXME: VARIANT_CALENDAR HIJRI should cause Hijri output */
//         BSTR date = NULL;
//         WCHAR *pDate;
//         hRes = VarBstrFromDate(V_DATE(&vDate), lcid, VAR_TIMEVALUEONLY, &date);
//         if (FAILED(hRes))
//           goto VARIANT_FormatDate_Exit;
// 	pDate = date;
//         while (*pDate)
//           *pBuff++ = *pDate++;
//         SysFreeString(date);
//       }
//       break;
//
//     case FMT_DATE_DAY:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.st.wDay;
//       break;
//
//     case FMT_DATE_DAY_0:
//       szPrintFmt = szPercentZeroTwo_d;
//       dwVal = udate.st.wDay;
//       break;
//
//     case FMT_DATE_DAY_SHORT:
//       /* FIXME: VARIANT_CALENDAR HIJRI should cause Hijri output */
//       TRACE("short day\n");
//       localeValue = LOCALE_SABBREVDAYNAME1 + (udate.st.wDayOfWeek + 6)%7;
//       defaultChar = '?';
//       break;
//
//     case FMT_DATE_DAY_LONG:
//       /* FIXME: VARIANT_CALENDAR HIJRI should cause Hijri output */
//       TRACE("long day\n");
//       localeValue = LOCALE_SDAYNAME1 + (udate.st.wDayOfWeek + 6)%7;
//       defaultChar = '?';
//       break;
//
//     case FMT_DATE_SHORT:
//       /* FIXME: VARIANT_CALENDAR HIJRI should cause Hijri output */
//       dwFmt = LOCALE_SSHORTDATE;
//       break;
//
//     case FMT_DATE_LONG:
//       /* FIXME: VARIANT_CALENDAR HIJRI should cause Hijri output */
//       dwFmt = LOCALE_SLONGDATE;
//       break;
//
//     case FMT_DATE_MEDIUM:
//       FIXME("Medium date treated as long date\n");
//       dwFmt = LOCALE_SLONGDATE;
//       break;
//
//     case FMT_DATE_DAY_WEEK:
//       szPrintFmt = szPercent_d;
//       if (pToken[1])
//         dwVal = udate.st.wDayOfWeek + 2 - pToken[1];
//       else
//       {
//         GetLocaleInfoW(lcid,LOCALE_RETURN_NUMBER|LOCALE_IFIRSTDAYOFWEEK,
//                        (LPWSTR)&dwVal, sizeof(dwVal)/sizeof(WCHAR));
//         dwVal = udate.st.wDayOfWeek + 1 - dwVal;
//       }
//       pToken++;
//       break;
//
//     case FMT_DATE_WEEK_YEAR:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.wDayOfYear / 7 + 1;
//       pToken += 2;
//       FIXME("Ignoring nFirstDay of %d, nFirstWeek of %d\n", pToken[0], pToken[1]);
//       break;
//
//     case FMT_DATE_MON:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.st.wMonth;
//       break;
//
//     case FMT_DATE_MON_0:
//       szPrintFmt = szPercentZeroTwo_d;
//       dwVal = udate.st.wMonth;
//       break;
//
//     case FMT_DATE_MON_SHORT:
//       /* FIXME: VARIANT_CALENDAR HIJRI should cause Hijri output */
//       TRACE("short month\n");
//       localeValue = LOCALE_SABBREVMONTHNAME1 + udate.st.wMonth - 1;
//       defaultChar = '?';
//       break;
//
//     case FMT_DATE_MON_LONG:
//       /* FIXME: VARIANT_CALENDAR HIJRI should cause Hijri output */
//       TRACE("long month\n");
//       localeValue = LOCALE_SMONTHNAME1 + udate.st.wMonth - 1;
//       defaultChar = '?';
//       break;
//
//     case FMT_DATE_YEAR_DOY:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.wDayOfYear;
//       break;
//
//     case FMT_DATE_YEAR_0:
//       szPrintFmt = szPercentZeroTwo_d;
//       dwVal = udate.st.wYear % 100;
//       break;
//
//     case FMT_DATE_YEAR_LONG:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.st.wYear;
//       break;
//
//     case FMT_DATE_MIN:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.st.wMinute;
//       break;
//
//     case FMT_DATE_MIN_0:
//       szPrintFmt = szPercentZeroTwo_d;
//       dwVal = udate.st.wMinute;
//       break;
//
//     case FMT_DATE_SEC:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.st.wSecond;
//       break;
//
//     case FMT_DATE_SEC_0:
//       szPrintFmt = szPercentZeroTwo_d;
//       dwVal = udate.st.wSecond;
//       break;
//
//     case FMT_DATE_HOUR:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.st.wHour;
//       break;
//
//     case FMT_DATE_HOUR_0:
//     case FMT_DATE_TIME_UNK2:
//       szPrintFmt = szPercentZeroTwo_d;
//       dwVal = udate.st.wHour;
//       break;
//
//     case FMT_DATE_HOUR_12:
//       szPrintFmt = szPercent_d;
//       dwVal = udate.st.wHour ? udate.st.wHour > 12 ? udate.st.wHour - 12 : udate.st.wHour : 12;
//       break;
//
//     case FMT_DATE_HOUR_12_0:
//       szPrintFmt = szPercentZeroTwo_d;
//       dwVal = udate.st.wHour ? udate.st.wHour > 12 ? udate.st.wHour - 12 : udate.st.wHour : 12;
//       break;
//
//     case FMT_DATE_AMPM_SYS1:
//     case FMT_DATE_AMPM_SYS2:
//       localeValue = udate.st.wHour < 12 ? LOCALE_S1159 : LOCALE_S2359;
//       defaultChar = '?';
//       break;
//
//     case FMT_DATE_AMPM_UPPER:
//       *pBuff++ = udate.st.wHour < 12 ? 'A' : 'P';
//       *pBuff++ = 'M';
//       break;
//
//     case FMT_DATE_A_UPPER:
//       *pBuff++ = udate.st.wHour < 12 ? 'A' : 'P';
//       break;
//
//     case FMT_DATE_AMPM_LOWER:
//       *pBuff++ = udate.st.wHour < 12 ? 'a' : 'p';
//       *pBuff++ = 'm';
//       break;
//
//     case FMT_DATE_A_LOWER:
//       *pBuff++ = udate.st.wHour < 12 ? 'a' : 'p';
//       break;
//
//     default:
//       ERR("Unknown token 0x%02x!\n", *pToken);
//       hRes = E_INVALIDARG;
//       goto VARIANT_FormatDate_Exit;
//     }
//     if (localeValue)
//     {
//       *pBuff = '\0';
//       if (GetLocaleInfoW(lcid, localeValue, pBuff, ARRAY_SIZE(buff)-(pBuff-buff)))
//       {
//         TRACE("added %s\n", debugstr_w(pBuff));
//         while (*pBuff)
//           pBuff++;
//       }
//       else
//       {
//         TRACE("added %d %c\n", defaultChar, defaultChar);
//         *pBuff++ = defaultChar;
//       }
//     }
//     else if (dwFmt)
//     {
//       WCHAR fmt_buff[80];
//
//       if (!GetLocaleInfoW(lcid, dwFmt, fmt_buff, ARRAY_SIZE(fmt_buff)) ||
//           !get_date_format(lcid, 0, &udate.st, fmt_buff, pBuff, ARRAY_SIZE(buff)-(pBuff-buff)))
//       {
//         hRes = E_INVALIDARG;
//         goto VARIANT_FormatDate_Exit;
//       }
//       while (*pBuff)
//         pBuff++;
//     }
//     else if (szPrintFmt)
//     {
//       swprintf(pBuff, szPrintFmt, dwVal);
//       while (*pBuff)
//         pBuff++;
//     }
//     pToken++;
//   }
//
// VARIANT_FormatDate_Exit:
//   *pBuff = '\0';
//   TRACE("buff is %s\n", debugstr_w(buff));
//   if (SUCCEEDED(hRes))
//   {
//     *pbstrOut = SysAllocString(buff);
//     if (!*pbstrOut)
//       hRes = E_OUTOFMEMORY;
//   }
//   return hRes;
// }
//
// /* Format a variant using a string format */
// static HRESULT VARIANT_FormatString(LPVARIANT pVarIn, LPOLESTR lpszFormat,
//                                     LPBYTE rgbTok, ULONG dwFlags,
//                                     BSTR *pbstrOut, LCID lcid)
// {
//   static WCHAR szEmpty[] = { '\0' };
//   WCHAR buff[256], *pBuff = buff;
//   WCHAR *pSrc;
//   FMT_HEADER *header = (FMT_HEADER*)rgbTok;
//   FMT_STRING_HEADER *strHeader;
//   const BYTE* pToken = NULL;
//   VARIANT vStr;
//   int blanks_first;
//   BOOL bUpper = FALSE;
//   HRESULT hRes = S_OK;
//
//   TRACE("%s,%s,%p,0x%08x,%p,0x%08x)\n", debugstr_variant(pVarIn), debugstr_w(lpszFormat),
//         rgbTok, dwFlags, pbstrOut, lcid);
//
//   V_VT(&vStr) = VT_EMPTY;
//
//   if (V_TYPE(pVarIn) == VT_EMPTY || V_TYPE(pVarIn) == VT_NULL)
//   {
//     strHeader = (FMT_STRING_HEADER*)(rgbTok + FmtGetNegative(header));
//     V_BSTR(&vStr) = szEmpty;
//   }
//   else
//   {
//     hRes = VariantChangeTypeEx(&vStr, pVarIn, lcid, VARIANT_NOUSEROVERRIDE, VT_BSTR);
//     if (FAILED(hRes))
//       return hRes;
//
//     if (V_BSTR(&vStr)[0] == '\0')
//       strHeader = (FMT_STRING_HEADER*)(rgbTok + FmtGetNegative(header));
//     else
//       strHeader = (FMT_STRING_HEADER*)(rgbTok + FmtGetPositive(header));
//   }
//   pSrc = V_BSTR(&vStr);
//   if ((strHeader->flags & (FMT_FLAG_LT|FMT_FLAG_GT)) == FMT_FLAG_GT)
//     bUpper = TRUE;
//   blanks_first = strHeader->copy_chars - lstrlenW(pSrc);
//   pToken = (const BYTE*)strHeader + sizeof(FMT_DATE_HEADER);
//
//   while (*pToken != FMT_GEN_END)
//   {
//     int dwCount = 0;
//
//     if (pToken - rgbTok > header->size)
//     {
//       ERR("Ran off the end of the format!\n");
//       hRes = E_INVALIDARG;
//       goto VARIANT_FormatString_Exit;
//     }
//
//     switch (*pToken)
//     {
//     case FMT_GEN_COPY:
//       TRACE("copy %s\n", debugstr_wn(lpszFormat + pToken[1], pToken[2]));
//       memcpy(pBuff, lpszFormat + pToken[1], pToken[2] * sizeof(WCHAR));
//       pBuff += pToken[2];
//       pToken += 2;
//       break;
//
//     case FMT_STR_COPY_SPACE:
//     case FMT_STR_COPY_SKIP:
//       dwCount = pToken[1];
//       if (*pToken == FMT_STR_COPY_SPACE && blanks_first > 0)
//       {
//         TRACE("insert %d initial spaces\n", blanks_first);
//         while (dwCount > 0 && blanks_first > 0)
//         {
//           *pBuff++ = ' ';
//           dwCount--;
//           blanks_first--;
//         }
//       }
//       TRACE("copy %d chars%s\n", dwCount,
//             *pToken == FMT_STR_COPY_SPACE ? " with space" :"");
//       while (dwCount > 0 && *pSrc)
//       {
//         if (bUpper)
//           *pBuff++ = towupper(*pSrc);
//         else
//           *pBuff++ = towlower(*pSrc);
//         dwCount--;
//         pSrc++;
//       }
//       if (*pToken == FMT_STR_COPY_SPACE && dwCount > 0)
//       {
//         TRACE("insert %d spaces\n", dwCount);
//         while (dwCount-- > 0)
//           *pBuff++ = ' ';
//       }
//       pToken++;
//       break;
//
//     default:
//       ERR("Unknown token 0x%02x!\n", *pToken);
//       hRes = E_INVALIDARG;
//       goto VARIANT_FormatString_Exit;
//     }
//     pToken++;
//   }
//
// VARIANT_FormatString_Exit:
//   /* Copy out any remaining chars */
//   while (*pSrc)
//   {
//     if (bUpper)
//       *pBuff++ = towupper(*pSrc);
//     else
//       *pBuff++ = towlower(*pSrc);
//     pSrc++;
//   }
//   VariantClear(&vStr);
//   *pBuff = '\0';
//   TRACE("buff is %s\n", debugstr_w(buff));
//   if (SUCCEEDED(hRes))
//   {
//     *pbstrOut = SysAllocString(buff);
//     if (!*pbstrOut)
//       hRes = E_OUTOFMEMORY;
//   }
//   return hRes;
// }
//
// #define NUMBER_VTBITS (VTBIT_I1|VTBIT_UI1|VTBIT_I2|VTBIT_UI2| \
//                        VTBIT_I4|VTBIT_UI4|VTBIT_I8|VTBIT_UI8| \
//                        VTBIT_R4|VTBIT_R8|VTBIT_CY|VTBIT_DECIMAL| \
//                        VTBIT_BOOL|VTBIT_INT|VTBIT_UINT)
//
// /**********************************************************************
//  *              VarFormatFromTokens [OLEAUT32.139]
//  */
// HRESULT WINAPI VarFormatFromTokens(LPVARIANT pVarIn, LPOLESTR lpszFormat,
//                                    LPBYTE rgbTok, ULONG dwFlags,
//                                    BSTR *pbstrOut, LCID lcid)
// {
//   FMT_SHORT_HEADER *header = (FMT_SHORT_HEADER *)rgbTok;
//   VARIANT vTmp;
//   HRESULT hres;
//
//   TRACE("(%p,%s,%p,%x,%p,0x%08x)\n", pVarIn, debugstr_w(lpszFormat),
//           rgbTok, dwFlags, pbstrOut, lcid);
//
//   if (!pbstrOut)
//     return E_INVALIDARG;
//
//   *pbstrOut = NULL;
//
//   if (!pVarIn || !rgbTok)
//     return E_INVALIDARG;
//
//   if (V_VT(pVarIn) == VT_NULL)
//     return S_OK;
//
//   if (*rgbTok == FMT_TO_STRING || header->type == FMT_TYPE_GENERAL)
//   {
//     /* According to MSDN, general format acts somewhat like the 'Str'
//      * function in Visual Basic.
//      */
// VarFormatFromTokens_AsStr:
//     V_VT(&vTmp) = VT_EMPTY;
//     hres = VariantChangeTypeEx(&vTmp, pVarIn, lcid, dwFlags, VT_BSTR);
//     *pbstrOut = V_BSTR(&vTmp);
//   }
//   else
//   {
//     if (header->type == FMT_TYPE_NUMBER ||
//         (header->type == FMT_TYPE_UNKNOWN && ((1 << V_TYPE(pVarIn)) & NUMBER_VTBITS)))
//     {
//       hres = VARIANT_FormatNumber(pVarIn, lpszFormat, rgbTok, dwFlags, pbstrOut, lcid);
//     }
//     else if (header->type == FMT_TYPE_DATE ||
//              (header->type == FMT_TYPE_UNKNOWN && V_TYPE(pVarIn) == VT_DATE))
//     {
//       hres = VARIANT_FormatDate(pVarIn, lpszFormat, rgbTok, dwFlags, pbstrOut, lcid);
//     }
//     else if (header->type == FMT_TYPE_STRING || V_TYPE(pVarIn) == VT_BSTR)
//     {
//       hres = VARIANT_FormatString(pVarIn, lpszFormat, rgbTok, dwFlags, pbstrOut, lcid);
//     }
//     else
//     {
//       ERR("unrecognised format type 0x%02x\n", header->type);
//       return E_INVALIDARG;
//     }
//     /* If the coercion failed, still try to create output, unless the
//      * VAR_FORMAT_NOSUBSTITUTE flag is set.
//      */
//     if ((hres == DISP_E_OVERFLOW || hres == DISP_E_TYPEMISMATCH) &&
//         !(dwFlags & VAR_FORMAT_NOSUBSTITUTE))
//       goto VarFormatFromTokens_AsStr;
//   }
//
//   return hres;
// }
//
// /**********************************************************************
//  *              VarFormat [OLEAUT32.87]
//  *
//  * Format a variant from a format string.
//  *
//  * PARAMS
//  *  pVarIn     [I] Variant to format
//  *  lpszFormat [I] Format string (see notes)
//  *  nFirstDay  [I] First day of the week, (See VarTokenizeFormatString() for details)
//  *  nFirstWeek [I] First week of the year (See VarTokenizeFormatString() for details)
//  *  dwFlags    [I] Flags for the format (VAR_ flags from "oleauto.h")
//  *  pbstrOut   [O] Destination for formatted string.
//  *
//  * RETURNS
//  *  Success: S_OK. pbstrOut contains the formatted value.
//  *  Failure: E_INVALIDARG, if any parameter is invalid.
//  *           E_OUTOFMEMORY, if enough memory cannot be allocated.
//  *           DISP_E_TYPEMISMATCH, if the variant cannot be formatted.
//  *
//  * NOTES
//  *  - See Variant-Formats for details concerning creating format strings.
//  *  - This function uses LOCALE_USER_DEFAULT when calling VarTokenizeFormatString()
//  *    and VarFormatFromTokens().
//  */
// HRESULT WINAPI VarFormat(LPVARIANT pVarIn, LPOLESTR lpszFormat,
//                          int nFirstDay, int nFirstWeek, ULONG dwFlags,
//                          BSTR *pbstrOut)
// {
//   BYTE buff[256];
//   HRESULT hres;
//
//   TRACE("(%s,%s,%d,%d,0x%08x,%p)\n", debugstr_variant(pVarIn), debugstr_w(lpszFormat),
//         nFirstDay, nFirstWeek, dwFlags, pbstrOut);
//
//   if (!pbstrOut)
//     return E_INVALIDARG;
//   *pbstrOut = NULL;
//
//   hres = VarTokenizeFormatString(lpszFormat, buff, sizeof(buff), nFirstDay,
//                                  nFirstWeek, LOCALE_USER_DEFAULT, NULL);
//   if (SUCCEEDED(hres))
//     hres = VarFormatFromTokens(pVarIn, lpszFormat, buff, dwFlags,
//                                pbstrOut, LOCALE_USER_DEFAULT);
//   TRACE("returning 0x%08x, %s\n", hres, debugstr_w(*pbstrOut));
//   return hres;
// }
//
// /**********************************************************************
//  *              VarFormatDateTime [OLEAUT32.97]
//  *
//  * Format a variant value as a date and/or time.
//  *
//  * PARAMS
//  *  pVarIn    [I] Variant to format
//  *  nFormat   [I] Format type (see notes)
//  *  dwFlags   [I] Flags for the format (VAR_ flags from "oleauto.h")
//  *  pbstrOut  [O] Destination for formatted string.
//  *
//  * RETURNS
//  *  Success: S_OK. pbstrOut contains the formatted value.
//  *  Failure: E_INVALIDARG, if any parameter is invalid.
//  *           E_OUTOFMEMORY, if enough memory cannot be allocated.
//  *           DISP_E_TYPEMISMATCH, if the variant cannot be formatted.
//  *
//  * NOTES
//  *  This function uses LOCALE_USER_DEFAULT when determining the date format
//  *  characters to use.
//  *  Possible values for the nFormat parameter are:
//  *| Value  Meaning
//  *| -----  -------
//  *|   0    General date format
//  *|   1    Long date format
//  *|   2    Short date format
//  *|   3    Long time format
//  *|   4    Short time format
//  */
// HRESULT WINAPI VarFormatDateTime(LPVARIANT pVarIn, INT nFormat, ULONG dwFlags, BSTR *pbstrOut)
// {
//   static WCHAR szEmpty[] = { '\0' };
//   const BYTE* lpFmt = NULL;
//
//   TRACE("%s,%d,0x%08x,%p)\n", debugstr_variant(pVarIn), nFormat, dwFlags, pbstrOut);
//
//   if (!pVarIn || !pbstrOut || nFormat < 0 || nFormat > 4)
//     return E_INVALIDARG;
//
//   switch (nFormat)
//   {
//   case 0: lpFmt = fmtGeneralDate; break;
//   case 1: lpFmt = fmtLongDate; break;
//   case 2: lpFmt = fmtShortDate; break;
//   case 3: lpFmt = fmtLongTime; break;
//   case 4: lpFmt = fmtShortTime; break;
//   }
//   return VarFormatFromTokens(pVarIn, szEmpty, (BYTE*)lpFmt, dwFlags,
//                               pbstrOut, LOCALE_USER_DEFAULT);
// }
//
// #define GETLOCALENUMBER(type,field) GetLocaleInfoW(LOCALE_USER_DEFAULT, \
//                                                    type|LOCALE_RETURN_NUMBER, \
//                                                    (LPWSTR)&numfmt.field, \
//                                                    sizeof(numfmt.field)/sizeof(WCHAR))
//
// /**********************************************************************
//  *              VarFormatNumber [OLEAUT32.107]
//  *
//  * Format a variant value as a number.
//  *
//  * PARAMS
//  *  pVarIn    [I] Variant to format
//  *  nDigits   [I] Number of digits following the decimal point (-1 = user default)
//  *  nLeading  [I] Use a leading zero (-2 = user default, -1 = yes, 0 = no)
//  *  nParens   [I] Use brackets for values < 0 (-2 = user default, -1 = yes, 0 = no)
//  *  nGrouping [I] Use grouping characters (-2 = user default, -1 = yes, 0 = no)
//  *  dwFlags   [I] Currently unused, set to zero
//  *  pbstrOut  [O] Destination for formatted string.
//  *
//  * RETURNS
//  *  Success: S_OK. pbstrOut contains the formatted value.
//  *  Failure: E_INVALIDARG, if any parameter is invalid.
//  *           E_OUTOFMEMORY, if enough memory cannot be allocated.
//  *           DISP_E_TYPEMISMATCH, if the variant cannot be formatted.
//  *
//  * NOTES
//  *  This function uses LOCALE_USER_DEFAULT when determining the number format
//  *  characters to use.
//  */
// HRESULT WINAPI VarFormatNumber(LPVARIANT pVarIn, INT nDigits, INT nLeading, INT nParens,
//                                INT nGrouping, ULONG dwFlags, BSTR *pbstrOut)
// {
//   HRESULT hRet;
//   VARIANT vStr;
//
//   TRACE("(%s,%d,%d,%d,%d,0x%08x,%p)\n", debugstr_variant(pVarIn), nDigits, nLeading,
//         nParens, nGrouping, dwFlags, pbstrOut);
//
//   if (!pVarIn || !pbstrOut || nDigits > 9)
//     return E_INVALIDARG;
//
//   *pbstrOut = NULL;
//
//   V_VT(&vStr) = VT_EMPTY;
//   hRet = VariantCopyInd(&vStr, pVarIn);
//
//   if (SUCCEEDED(hRet))
//     hRet = VariantChangeTypeEx(&vStr, &vStr, LCID_US, 0, VT_BSTR);
//
//   if (SUCCEEDED(hRet))
//   {
//     WCHAR buff[256], decimal[8], thousands[8];
//     NUMBERFMTW numfmt;
//
//     /* Although MSDN makes it clear that the native versions of these functions
//      * are implemented using VarTokenizeFormatString()/VarFormatFromTokens(),
//      * using NLS gives us the same result.
//      */
//     if (nDigits < 0)
//       GETLOCALENUMBER(LOCALE_IDIGITS, NumDigits);
//     else
//       numfmt.NumDigits = nDigits;
//
//     if (nLeading == -2)
//       GETLOCALENUMBER(LOCALE_ILZERO, LeadingZero);
//     else if (nLeading == -1)
//       numfmt.LeadingZero = 1;
//     else
//       numfmt.LeadingZero = 0;
//
//     if (nGrouping == -2)
//     {
//       WCHAR grouping[10];
//       grouping[2] = '\0';
//       GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SGROUPING, grouping, ARRAY_SIZE(grouping));
//       numfmt.Grouping = grouping[2] == '2' ? 32 : grouping[0] - '0';
//     }
//     else if (nGrouping == -1)
//       numfmt.Grouping = 3; /* 3 = "n,nnn.nn" */
//     else
//       numfmt.Grouping = 0; /* 0 = No grouping */
//
//     if (nParens == -2)
//       GETLOCALENUMBER(LOCALE_INEGNUMBER, NegativeOrder);
//     else if (nParens == -1)
//       numfmt.NegativeOrder = 0; /* 0 = "(xxx)" */
//     else
//       numfmt.NegativeOrder = 1; /* 1 = "-xxx" */
//
//     numfmt.lpDecimalSep = decimal;
//     GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SDECIMAL, decimal, ARRAY_SIZE(decimal));
//     numfmt.lpThousandSep = thousands;
//     GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_STHOUSAND, thousands, ARRAY_SIZE(thousands));
//
//     if (GetNumberFormatW(LOCALE_USER_DEFAULT, 0, V_BSTR(&vStr), &numfmt, buff, ARRAY_SIZE(buff)))
//     {
//       *pbstrOut = SysAllocString(buff);
//       if (!*pbstrOut)
//         hRet = E_OUTOFMEMORY;
//     }
//     else
//       hRet = DISP_E_TYPEMISMATCH;
//
//     SysFreeString(V_BSTR(&vStr));
//   }
//   return hRet;
// }
//
// /**********************************************************************
//  *              VarFormatPercent [OLEAUT32.117]
//  *
//  * Format a variant value as a percentage.
//  *
//  * PARAMS
//  *  pVarIn    [I] Variant to format
//  *  nDigits   [I] Number of digits following the decimal point (-1 = user default)
//  *  nLeading  [I] Use a leading zero (-2 = user default, -1 = yes, 0 = no)
//  *  nParens   [I] Use brackets for values < 0 (-2 = user default, -1 = yes, 0 = no)
//  *  nGrouping [I] Use grouping characters (-2 = user default, -1 = yes, 0 = no)
//  *  dwFlags   [I] Currently unused, set to zero
//  *  pbstrOut  [O] Destination for formatted string.
//  *
//  * RETURNS
//  *  Success: S_OK. pbstrOut contains the formatted value.
//  *  Failure: E_INVALIDARG, if any parameter is invalid.
//  *           E_OUTOFMEMORY, if enough memory cannot be allocated.
//  *           DISP_E_OVERFLOW, if overflow occurs during the conversion.
//  *           DISP_E_TYPEMISMATCH, if the variant cannot be formatted.
//  *
//  * NOTES
//  *  This function uses LOCALE_USER_DEFAULT when determining the number format
//  *  characters to use.
//  */
// HRESULT WINAPI VarFormatPercent(LPVARIANT pVarIn, INT nDigits, INT nLeading, INT nParens,
//                                 INT nGrouping, ULONG dwFlags, BSTR *pbstrOut)
// {
//   static const WCHAR szPercent[] = { '%','\0' };
//   static const WCHAR szPercentBracket[] = { '%',')','\0' };
//   WCHAR buff[256];
//   HRESULT hRet;
//   VARIANT vDbl;
//
//   TRACE("(%s,%d,%d,%d,%d,0x%08x,%p)\n", debugstr_variant(pVarIn), nDigits, nLeading,
//         nParens, nGrouping, dwFlags, pbstrOut);
//
//   if (!pVarIn || !pbstrOut || nDigits > 9)
//     return E_INVALIDARG;
//
//   *pbstrOut = NULL;
//
//   V_VT(&vDbl) = VT_EMPTY;
//   hRet = VariantCopyInd(&vDbl, pVarIn);
//
//   if (SUCCEEDED(hRet))
//   {
//     hRet = VariantChangeTypeEx(&vDbl, &vDbl, LOCALE_USER_DEFAULT, 0, VT_R8);
//
//     if (SUCCEEDED(hRet))
//     {
//       if (V_R8(&vDbl) > (R8_MAX / 100.0))
//         return DISP_E_OVERFLOW;
//
//       V_R8(&vDbl) *= 100.0;
//       hRet = VarFormatNumber(&vDbl, nDigits, nLeading, nParens,
//                              nGrouping, dwFlags, pbstrOut);
//
//       if (SUCCEEDED(hRet))
//       {
//         DWORD dwLen = lstrlenW(*pbstrOut);
//         BOOL bBracket = (*pbstrOut)[dwLen] == ')';
//
//         dwLen -= bBracket;
//         memcpy(buff, *pbstrOut, dwLen * sizeof(WCHAR));
//         lstrcpyW(buff + dwLen, bBracket ? szPercentBracket : szPercent);
//         SysFreeString(*pbstrOut);
//         *pbstrOut = SysAllocString(buff);
//         if (!*pbstrOut)
//           hRet = E_OUTOFMEMORY;
//       }
//     }
//   }
//   return hRet;
// }
//
// /**********************************************************************
//  *              VarFormatCurrency [OLEAUT32.127]
//  *
//  * Format a variant value as a currency.
//  *
//  * PARAMS
//  *  pVarIn    [I] Variant to format
//  *  nDigits   [I] Number of digits following the decimal point (-1 = user default)
//  *  nLeading  [I] Use a leading zero (-2 = user default, -1 = yes, 0 = no)
//  *  nParens   [I] Use brackets for values < 0 (-2 = user default, -1 = yes, 0 = no)
//  *  nGrouping [I] Use grouping characters (-2 = user default, -1 = yes, 0 = no)
//  *  dwFlags   [I] Currently unused, set to zero
//  *  pbstrOut  [O] Destination for formatted string.
//  *
//  * RETURNS
//  *  Success: S_OK. pbstrOut contains the formatted value.
//  *  Failure: E_INVALIDARG, if any parameter is invalid.
//  *           E_OUTOFMEMORY, if enough memory cannot be allocated.
//  *           DISP_E_TYPEMISMATCH, if the variant cannot be formatted.
//  *
//  * NOTES
//  *  This function uses LOCALE_USER_DEFAULT when determining the currency format
//  *  characters to use.
//  */
// HRESULT WINAPI VarFormatCurrency(LPVARIANT pVarIn, INT nDigits, INT nLeading,
//                                  INT nParens, INT nGrouping, ULONG dwFlags,
//                                  BSTR *pbstrOut)
// {
//   HRESULT hRet;
//   VARIANT vStr;
//
//   TRACE("(%s,%d,%d,%d,%d,0x%08x,%p)\n", debugstr_variant(pVarIn), nDigits, nLeading,
//         nParens, nGrouping, dwFlags, pbstrOut);
//
//   if (!pVarIn || !pbstrOut || nDigits > 9)
//     return E_INVALIDARG;
//
//   *pbstrOut = NULL;
//
//   V_VT(&vStr) = VT_EMPTY;
//   hRet = VariantCopyInd(&vStr, pVarIn);
//
//   if (SUCCEEDED(hRet))
//     hRet = VariantChangeTypeEx(&vStr, &vStr, LOCALE_USER_DEFAULT, 0, VT_BSTR);
//
//   if (SUCCEEDED(hRet))
//   {
//     WCHAR buff[256], decimal[8], thousands[4], currency[13];
//     CURRENCYFMTW numfmt;
//
//     if (nDigits < 0)
//       GETLOCALENUMBER(LOCALE_IDIGITS, NumDigits);
//     else
//       numfmt.NumDigits = nDigits;
//
//     if (nLeading == -2)
//       GETLOCALENUMBER(LOCALE_ILZERO, LeadingZero);
//     else if (nLeading == -1)
//       numfmt.LeadingZero = 1;
//     else
//       numfmt.LeadingZero = 0;
//
//     if (nGrouping == -2)
//     {
//       WCHAR grouping[10];
//       grouping[2] = '\0';
//       GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SGROUPING, grouping, ARRAY_SIZE(grouping));
//       numfmt.Grouping = grouping[2] == '2' ? 32 : grouping[0] - '0';
//     }
//     else if (nGrouping == -1)
//       numfmt.Grouping = 3; /* 3 = "n,nnn.nn" */
//     else
//       numfmt.Grouping = 0; /* 0 = No grouping */
//
//     if (nParens == -2)
//       GETLOCALENUMBER(LOCALE_INEGCURR, NegativeOrder);
//     else if (nParens == -1)
//       numfmt.NegativeOrder = 0; /* 0 = "(xxx)" */
//     else
//       numfmt.NegativeOrder = 1; /* 1 = "-xxx" */
//
//     GETLOCALENUMBER(LOCALE_ICURRENCY, PositiveOrder);
//
//     numfmt.lpDecimalSep = decimal;
//     GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SDECIMAL, decimal, ARRAY_SIZE(decimal));
//     numfmt.lpThousandSep = thousands;
//     GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_STHOUSAND, thousands, ARRAY_SIZE(thousands));
//     numfmt.lpCurrencySymbol = currency;
//     GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SCURRENCY, currency, ARRAY_SIZE(currency));
//
//     /* use NLS as per VarFormatNumber() */
//     if (GetCurrencyFormatW(LOCALE_USER_DEFAULT, 0, V_BSTR(&vStr), &numfmt, buff, ARRAY_SIZE(buff)))
//     {
//       *pbstrOut = SysAllocString(buff);
//       if (!*pbstrOut)
//         hRet = E_OUTOFMEMORY;
//     }
//     else
//       hRet = DISP_E_TYPEMISMATCH;
//
//     SysFreeString(V_BSTR(&vStr));
//   }
//   return hRet;
// }
//
// /**********************************************************************
//  *              VarMonthName [OLEAUT32.129]
//  *
//  * Print the specified month as localized name.
//  *
//  * PARAMS
//  *  iMonth    [I] month number 1..12
//  *  fAbbrev   [I] 0 - full name, !0 - abbreviated name
//  *  dwFlags   [I] flag stuff. only VAR_CALENDAR_HIJRI possible.
//  *  pbstrOut  [O] Destination for month name
//  *
//  * RETURNS
//  *  Success: S_OK. pbstrOut contains the name.
//  *  Failure: E_INVALIDARG, if any parameter is invalid.
//  *           E_OUTOFMEMORY, if enough memory cannot be allocated.
//  */
// HRESULT WINAPI VarMonthName(INT iMonth, INT fAbbrev, ULONG dwFlags, BSTR *pbstrOut)
// {
//   DWORD localeValue;
//   INT size;
//
//   if ((iMonth < 1)  || (iMonth > 12))
//     return E_INVALIDARG;
//
//   if (dwFlags)
//     FIXME("Does not support dwFlags 0x%x, ignoring.\n", dwFlags);
//
//   if (fAbbrev)
// 	localeValue = LOCALE_SABBREVMONTHNAME1 + iMonth - 1;
//   else
// 	localeValue = LOCALE_SMONTHNAME1 + iMonth - 1;
//
//   size = GetLocaleInfoW(LOCALE_USER_DEFAULT,localeValue, NULL, 0);
//   if (!size) {
//     ERR("GetLocaleInfo 0x%x failed.\n", localeValue);
//     return HRESULT_FROM_WIN32(GetLastError());
//   }
//   *pbstrOut = SysAllocStringLen(NULL,size - 1);
//   if (!*pbstrOut)
//     return E_OUTOFMEMORY;
//   size = GetLocaleInfoW(LOCALE_USER_DEFAULT,localeValue, *pbstrOut, size);
//   if (!size) {
//     ERR("GetLocaleInfo of 0x%x failed in 2nd stage?!\n", localeValue);
//     SysFreeString(*pbstrOut);
//     return HRESULT_FROM_WIN32(GetLastError());
//   }
//   return S_OK;
// }
//
// /**********************************************************************
//  *              VarWeekdayName [OLEAUT32.129]
//  *
//  * Print the specified weekday as localized name.
//  *
//  * PARAMS
//  *  iWeekday  [I] day of week, 1..7, 1="the first day of the week"
//  *  fAbbrev   [I] 0 - full name, !0 - abbreviated name
//  *  iFirstDay [I] first day of week,
//  *                0=system default, 1=Sunday, 2=Monday, .. (contrary to MSDN)
//  *  dwFlags   [I] flag stuff. only VAR_CALENDAR_HIJRI possible.
//  *  pbstrOut  [O] Destination for weekday name.
//  *
//  * RETURNS
//  *  Success: S_OK, pbstrOut contains the name.
//  *  Failure: E_INVALIDARG, if any parameter is invalid.
//  *           E_OUTOFMEMORY, if enough memory cannot be allocated.
//  */
// HRESULT WINAPI VarWeekdayName(INT iWeekday, INT fAbbrev, INT iFirstDay,
//                               ULONG dwFlags, BSTR *pbstrOut)
// {
//   DWORD localeValue;
//   INT size;
//
//   /* Windows XP oleaut32.dll doesn't allow iWekday==0, contrary to MSDN */
//   if (iWeekday < 1 || iWeekday > 7)
//     return E_INVALIDARG;
//   if (iFirstDay < 0 || iFirstDay > 7)
//     return E_INVALIDARG;
//   if (!pbstrOut)
//     return E_INVALIDARG;
//
//   if (dwFlags)
//     FIXME("Does not support dwFlags 0x%x, ignoring.\n", dwFlags);
//
//   /* If we have to use the default firstDay, find which one it is */
//   if (iFirstDay == 0) {
//     DWORD firstDay;
//     localeValue = LOCALE_RETURN_NUMBER | LOCALE_IFIRSTDAYOFWEEK;
//     size = GetLocaleInfoW(LOCALE_USER_DEFAULT, localeValue,
//                           (LPWSTR)&firstDay, sizeof(firstDay) / sizeof(WCHAR));
//     if (!size) {
//       ERR("GetLocaleInfo 0x%x failed.\n", localeValue);
//       return HRESULT_FROM_WIN32(GetLastError());
//     }
//     iFirstDay = firstDay + 2;
//   }
//
//   /* Determine what we need to return */
//   localeValue = fAbbrev ? LOCALE_SABBREVDAYNAME1 : LOCALE_SDAYNAME1;
//   localeValue += (7 + iWeekday - 1 + iFirstDay - 2) % 7;
//
//   /* Determine the size of the data, allocate memory and retrieve the data */
//   size = GetLocaleInfoW(LOCALE_USER_DEFAULT, localeValue, NULL, 0);
//   if (!size) {
//     ERR("GetLocaleInfo 0x%x failed.\n", localeValue);
//     return HRESULT_FROM_WIN32(GetLastError());
//   }
//   *pbstrOut = SysAllocStringLen(NULL, size - 1);
//   if (!*pbstrOut)
//     return E_OUTOFMEMORY;
//   size = GetLocaleInfoW(LOCALE_USER_DEFAULT, localeValue, *pbstrOut, size);
//   if (!size) {
//     ERR("GetLocaleInfo 0x%x failed in 2nd stage?!\n", localeValue);
//     SysFreeString(*pbstrOut);
//     return HRESULT_FROM_WIN32(GetLastError());
//   }
//   return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
