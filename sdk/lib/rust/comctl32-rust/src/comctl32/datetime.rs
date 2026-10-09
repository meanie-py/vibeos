use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::CreateUpDownControl;
use crate::comctl32::comctl32undoc::Free;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
crate::wine_default_debug_channel!(__wine_dbch_datetime);
pub static RUSTCONV_WSTR_1871D60992D7: [u16; 5] = [0x0032u16, 0x0032, 0x0032, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_66039A3ABDA7: [u16; 3] = [0x0032u16, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_664E177C91D5: [u16; 10] = [0x0053u16, 0x0065, 0x0070, 0x0074, 0x0065, 0x006D, 0x0062, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_6AB1762C9894: [u16; 5] = [0x0025u16, 0x002E, 0x0032, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_85DD751867E3: [u16; 2] = [0x0032u16, 0x0000];
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_AC9F9795F0E6: [u16; 4] = [0x0044u16, 0x0065, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_D09E79007215: [u16; 4] = [0x0057u16, 0x0065, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_D5174DA8E065: [u16; 6] = [0x0025u16, 0x0073, 0x002E, 0x0033, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_DAAF62DE76F0: [u16; 2] = [0x0041u16, 0x0000];
pub static RUSTCONV_WSTR_E1DBD501A112: [u16; 10] = [0x0057u16, 0x0065, 0x0064, 0x006E, 0x0065, 0x0073, 0x0064, 0x0061, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_FE60D71583A9: [u16; 3] = [0x0041u16, 0x004D, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const MONTHCAL_CLASSW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"S\0y\0s\0M\0o\0n\0t\0h\0C\0a\0l\x003\x002\0\0\0",
    )
};

pub const DATETIMEPICK_CLASSW: [::core::ffi::c_ushort; 18] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_ushort; 18]>(
        *b"S\0y\0s\0D\0a\0t\0e\0T\0i\0m\0e\0P\0i\0c\0k\x003\x002\0\0\0",
    )
};

pub const WC_BUTTONW: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"B\0u\0t\0t\0o\0n\0\0\0")
};

static mut allowedformatchars: [WCHAR; 10] =
    unsafe { ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(*b"d\0h\0H\0m\0M\0s\0t\0y\0X\0\0\0") };

static mut maxrepetition: [::core::ffi::c_int; 9] = [
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    -(1 as ::core::ffi::c_int),
];

static mut max_allowed_date: SYSTEMTIME = _SYSTEMTIME {
    wYear: 9999 as WORD,
    wMonth: 12 as WORD,
    wDayOfWeek: 0 as WORD,
    wDay: 31 as WORD,
    wHour: 0,
    wMinute: 0,
    wSecond: 0,
    wMilliseconds: 0,
};

static mut min_allowed_date: SYSTEMTIME = _SYSTEMTIME {
    wYear: 1752 as WORD,
    wMonth: 9 as WORD,
    wDayOfWeek: 0 as WORD,
    wDay: 14 as WORD,
    wHour: 0,
    wMinute: 0,
    wSecond: 0,
    wMilliseconds: 0,
};

#[repr(C)]
pub struct DATETIME_INFO {
    pub hwndSelf: HWND,
    pub hMonthCal: HWND,
    pub hwndNotify: HWND,
    pub hUpdown: HWND,
    pub dwStyle: DWORD,
    pub date: SYSTEMTIME,
    pub dateValid: BOOL,
    pub hwndCheckbut: HWND,
    pub rcClient: RECT,
    pub rcDraw: RECT,
    pub checkbox: RECT,
    pub calbutton: RECT,
    pub bCalDepressed: BOOL,
    pub bDropdownEnabled: BOOL,
    pub select: i32,
    pub charsEntered: [u16; 4],
    pub nCharsEntered: i32,
    pub hFont: HFONT,
    pub nrFieldsAllocated: i32,
    pub nrFields: i32,
    pub haveFocus: i32,
    pub fieldspec: *mut i32,
    pub fieldRect: *mut RECT,
    pub buflen: *mut i32,
    pub textbuf: [u16; 256],
    pub monthcal_pos: POINT,
    pub pendingUpdown: i32,
}

pub type LPDATETIME_INFO = *mut DATETIME_INFO;

pub const DT_END_FORMAT: i32 = 0;

pub const ONEDIGITDAY: i32 = 0x01;

pub const TWODIGITDAY: i32 = 0x02;

pub const THREECHARDAY: i32 = 0x03;

pub const FULLDAY: i32 = 0x04;

pub const ONEDIGIT12HOUR: i32 = 0x11;

pub const TWODIGIT12HOUR: i32 = 0x12;

pub const ONEDIGIT24HOUR: i32 = 0x21;

pub const TWODIGIT24HOUR: i32 = 0x22;

pub const ONEDIGITMINUTE: i32 = 0x31;

pub const TWODIGITMINUTE: i32 = 0x32;

pub const ONEDIGITMONTH: i32 = 0x41;

pub const TWODIGITMONTH: i32 = 0x42;

pub const THREECHARMONTH: i32 = 0x43;

pub const FULLMONTH: i32 = 0x44;

pub const ONEDIGITSECOND: i32 = 0x51;

pub const TWODIGITSECOND: i32 = 0x52;

pub const ONELETTERAMPM: i32 = 0x61;

pub const TWOLETTERAMPM: i32 = 0x62;

pub const ONEDIGITYEAR: i32 = 0x71;

pub const TWODIGITYEAR: i32 = 0x72;

// FIXME - yyy is not valid - we'll treat it as yyyy
pub const INVALIDFULLYEAR: i32 = 0x73;

pub const FULLYEAR: i32 = 0x74;

// -> maximum of 0x80 callbacks possible
pub const FORMATCALLBACK: i32 = 0x81;

pub const FORMATCALLMASK: i32 = 0x80;

pub const DT_STRING: i32 = 0x0100;

// for hit-testing
pub const DTHT_DATEFIELD: i32 = 0xff;

pub const DTHT_NONE: i32 = 0x1000;

// these should end at '00' , to make
pub const DTHT_CHECKBOX: i32 = 0x2000;

// & DTHT_DATEFIELD 0 when DATETIME_KeyDown
pub const DTHT_MCPOPUP: i32 = 0x3000;

// tests for date-fields
pub const DTHT_GOTFOCUS: i32 = 0x4000;

// to mask check and drop down from others
pub const DTHT_NODATEMASK: i32 = 0xf000;

unsafe extern "C" fn DATETIME_GetSystemTime(infoPtr: *const DATETIME_INFO, systime: *mut SYSTEMTIME) -> DWORD {
    if systime.is_null() {
        return (GDT_NONE) as DWORD;
    }
    if ((*infoPtr).dwStyle & (DTS_SHOWNONE) as DWORD != 0)
        && (SendMessageW((*infoPtr).hwndCheckbut, (BM_GETCHECK) as UINT, (0) as WPARAM, (0) as LPARAM) == (BST_UNCHECKED) as LRESULT)
    {
        return (GDT_NONE) as DWORD;
    }

    *systime = (*infoPtr).date;

    return (GDT_VALID) as DWORD;
}

unsafe extern "C" fn DATETIME_IsDateInValidRange(infoPtr: *const DATETIME_INFO, date: *const SYSTEMTIME) -> BOOL {
    let mut range: [SYSTEMTIME; 2] = core::mem::zeroed();
    let mut limits: DWORD;

    if (MONTHCAL_CompareSystemTime(date, core::ptr::addr_of!(max_allowed_date)) == 1)
        || (MONTHCAL_CompareSystemTime(date, core::ptr::addr_of!(min_allowed_date)) == -1)
    {
        return FALSE;
    }

    limits = SendMessageW(
        (*infoPtr).hMonthCal,
        MCM_GETRANGE as UINT,
        0 as WPARAM,
        range.as_mut_ptr() as LPARAM,
    ) as DWORD;

    if limits & GDTR_MAX != 0 {
        if MONTHCAL_CompareSystemTime(date, core::ptr::addr_of_mut!(range[1 as usize])) == 1 {
            return FALSE;
        }
    }

    if limits & GDTR_MIN != 0 {
        if MONTHCAL_CompareSystemTime(date, core::ptr::addr_of_mut!(range[0 as usize])) == -1 {
            return FALSE;
        }
    }

    return TRUE;
}

unsafe extern "C" fn DATETIME_SetSystemTime(
    infoPtr: *mut DATETIME_INFO,
    flag: DWORD,
    systime: *const SYSTEMTIME,
) -> BOOL {
    if systime.is_null() {
        return FALSE;
    }
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DATETIME_SetSystemTime\0" as *const u8 as *const ::core::ffi::c_char,
            200 as ::core::ffi::c_int,
            b"%04d/%02d/%02d %02d:%02d:%02d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*systime).wYear as ::core::ffi::c_int,
            (*systime).wMonth as ::core::ffi::c_int,
            (*systime).wDay as ::core::ffi::c_int,
            (*systime).wHour as ::core::ffi::c_int,
            (*systime).wMinute as ::core::ffi::c_int,
            (*systime).wSecond as ::core::ffi::c_int,
        );
    }

    if flag == GDT_VALID as DWORD {
        if (*systime).wYear as i32 == 0
            || (((*systime).wMonth as i32) < 1)
            || ((*systime).wMonth as i32 > 12)
            || (((*systime).wDay as i32) < 1)
            || ((*systime).wDay as i32
                > MONTHCAL_MonthLength(
                    (*systime).wMonth as i32,
                    (*systime).wYear as i32,
                ))
            || ((*systime).wHour as i32 > 23)
            || ((*systime).wMinute as i32 > 59)
            || ((*systime).wSecond as i32 > 59)
            || ((*systime).wMilliseconds as i32 > 999)
        {
            return FALSE;
        }

        /* Windows returns true if the date is valid but outside the limits set */
        if DATETIME_IsDateInValidRange(infoPtr, systime) == 0 {
            return TRUE;
        }

        (*infoPtr).dateValid = TRUE;
        (*infoPtr).date = *systime;
        /* always store a valid day of week */
        MONTHCAL_CalculateDayOfWeek(&raw mut (*infoPtr).date, TRUE);

        SendMessageW(
            (*infoPtr).hMonthCal,
            MCM_SETCURSEL as UINT,
            0 as WPARAM,
            &raw mut (*infoPtr).date as LPARAM,
        );
        SendMessageW(
            (*infoPtr).hwndCheckbut,
            BM_SETCHECK as UINT,
            BST_CHECKED as WPARAM,
            0 as LPARAM,
        );
    } else if ((*infoPtr).dwStyle & (DTS_SHOWNONE as DWORD)) != 0
        && (flag == GDT_NONE as DWORD)
    {
        (*infoPtr).dateValid = FALSE;
        SendMessageW(
            (*infoPtr).hwndCheckbut,
            BM_SETCHECK as UINT,
            BST_UNCHECKED as WPARAM,
            0 as LPARAM,
        );
    } else {
        return FALSE;
    }

    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null_mut::<RECT>(),
        TRUE,
    );
    return TRUE;
}

unsafe extern "C" fn DATETIME_UseFormat(infoPtr: *mut DATETIME_INFO, formattxt: LPCWSTR) {
    let mut i: ::core::ffi::c_uint;
    let mut j: ::core::ffi::c_int;
    let mut k: ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int;
    let mut inside_literal: BOOL = FALSE; /* inside 0x002E2E2E /* 0x002E2E2E /* 0x002E2E2E /* 0x002E2E2E /* 0x002E2E2E /* 0x002E2E2E /* '...' */ */ */ */ */ */ */
    let nrFields: *mut ::core::ffi::c_int = core::ptr::addr_of_mut!((*infoPtr).nrFields);

    *nrFields = 0;
    *(*infoPtr).fieldspec.add(*nrFields as usize) = 0;
    len = lstrlenW(core::ptr::addr_of!(allowedformatchars) as LPCWSTR);
    k = 0;

    i = 0;
    while *formattxt.add(i as usize) != 0 {
        if inside_literal == 0 {
            j = 0;
            while j < len {
                if allowedformatchars[j as usize] == *formattxt.add(i as usize) {
                    if (*nrFields == 0)
                        && (*(*infoPtr).fieldspec.add(*nrFields as usize) == 0)
                    {
                        *(*infoPtr).fieldspec.add(*nrFields as usize) =
                            (j << 4) + 1;
                        break;
                    }
                    if ((*(*infoPtr).fieldspec.add(*nrFields as usize)) >> 4) != j {
                        *nrFields += 1;
                        *(*infoPtr).fieldspec.add(*nrFields as usize) =
                            (j << 4) + 1;
                        break;
                    }
                    if ((*(*infoPtr).fieldspec.add(*nrFields as usize)) & 0x0f)
                        == maxrepetition[j as usize]
                    {
                        *nrFields += 1;
                        *(*infoPtr).fieldspec.add(*nrFields as usize) =
                            (j << 4) + 1;
                        break;
                    }
                    *(*infoPtr).fieldspec.add(*nrFields as usize) += 1;
                    break;
                } else {
                    j += 1;
                }
            }
        } else {
            j = len;
        }

        if *formattxt.add(i as usize) == '\'' as u16 {
            inside_literal = if inside_literal == 0 { 1 } else { 0 };
            i = i.wrapping_add(1);
            continue;
        }

        /* char is not a specifier: handle char like a string */
        if j == len {
            if (*nrFields == 0)
                && (*(*infoPtr).fieldspec.add(*nrFields as usize) == 0)
            {
                *(*infoPtr).fieldspec.add(*nrFields as usize) = DT_STRING + k;
                *(*infoPtr).buflen.add(*nrFields as usize) = 0;
            } else if (*(*infoPtr).fieldspec.add(*nrFields as usize) & DT_STRING)
                != DT_STRING
            {
                *nrFields += 1;
                *(*infoPtr).fieldspec.add(*nrFields as usize) = DT_STRING + k;
                *(*infoPtr).buflen.add(*nrFields as usize) = 0;
            }
            (*infoPtr).textbuf[k as usize] = *formattxt.add(i as usize);
            k += 1;
            *(*infoPtr).buflen.add(*nrFields as usize) += 1;
        }

        if *nrFields == (*infoPtr).nrFieldsAllocated {
            /* out of memory; should reallocate. crash ahead. */
        }

        i = i.wrapping_add(1);
    }

    if *(*infoPtr).fieldspec.add(*nrFields as usize) != 0 {
        *nrFields += 1;
    }
}

unsafe extern "C" fn DATETIME_SetFormatW(infoPtr: *mut DATETIME_INFO, mut format: LPCWSTR) -> BOOL {
    let mut format_buf: [WCHAR; 80] = [0; 80];

    if format.is_null() {
        let mut format_item: DWORD;

        if ((*infoPtr).dwStyle & DTS_SHORTDATECENTURYFORMAT as DWORD) == DTS_SHORTDATECENTURYFORMAT as DWORD {
            format_item = LOCALE_SSHORTDATE as DWORD;
        } else if ((*infoPtr).dwStyle & DTS_LONGDATEFORMAT as DWORD) == DTS_LONGDATEFORMAT as DWORD {
            format_item = LOCALE_SLONGDATE as DWORD;
        } else if ((*infoPtr).dwStyle & DTS_TIMEFORMAT as DWORD) == DTS_TIMEFORMAT as DWORD {
            format_item = LOCALE_STIMEFORMAT as DWORD;
        } else {
            /* DTS_SHORTDATEFORMAT */
            format_item = LOCALE_SSHORTDATE as DWORD;
        }

        GetLocaleInfoW(
            LOCALE_USER_DEFAULT as LCID,
            format_item as LCTYPE,
            &raw mut format_buf as LPWSTR,
            ARRAY_SIZE!(format_buf) as ::core::ffi::c_int,
        );
        format = &raw mut format_buf as *mut WCHAR as LPCWSTR;
    }

    DATETIME_UseFormat(infoPtr, format);
    InvalidateRect(
        (*infoPtr).hwndSelf,
        ::core::ptr::null::<RECT>(),
        TRUE as BOOL,
    );

    return TRUE as BOOL;
}

unsafe extern "C" fn DATETIME_SetFormatA(
    mut infoPtr: *mut DATETIME_INFO,
    mut lpszFormat: LPCSTR,
) -> BOOL {
    if !lpszFormat.is_null() {
        let mut retval: BOOL = 0;
        let mut len: INT = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszFormat,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as INT;
        let mut wstr: LPWSTR = Alloc(
            (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ) as LPWSTR;
        if !wstr.is_null() {
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                lpszFormat,
                -(1 as ::core::ffi::c_int),
                wstr,
                len as ::core::ffi::c_int,
            );
        }
        retval = DATETIME_SetFormatW(infoPtr, wstr as LPCWSTR);
        Free(wstr as LPVOID);
        return retval;
    } else {
        return DATETIME_SetFormatW(infoPtr, 0 as LPCWSTR);
    };
}

unsafe extern "C" fn DATETIME_ReturnTxt(
    infoPtr: *const DATETIME_INFO,
    count: i32,
    result: LPWSTR,
    resultSize: i32,
) {
    let mut date: SYSTEMTIME = (*infoPtr).date;
    let mut spec: i32;
    let mut buffer: [WCHAR; 80] = [0 as WCHAR; 80];

    *result = 0 as WCHAR;

    if count > (*infoPtr).nrFields || count < 0 {
        return;
    }

    if (*infoPtr).fieldspec.is_null() {
        return;
    }

    spec = *(*infoPtr).fieldspec.offset(count as isize);
    if (spec & DT_STRING) != 0 {
        let mut txtlen: i32 = *(*infoPtr).buflen.offset(count as isize);

        if txtlen > resultSize {
            txtlen = resultSize - 1;
        }
        memcpy(
            result as *mut core::ffi::c_void,
            (&raw const (*infoPtr).textbuf as *const WCHAR).offset((spec & !DT_STRING) as isize)
                as *const core::ffi::c_void,
            (txtlen as usize) * core::mem::size_of::<WCHAR>(),
        );
        *result.offset(txtlen as isize) = 0 as WCHAR;

        return;
    }

    match spec {
        DT_END_FORMAT => {
            *result = 0 as WCHAR;
        }
        ONEDIGITDAY => {
            wsprintfW(result, RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), date.wDay as i32);
        }
        TWODIGITDAY => {
            wsprintfW(result, RUSTCONV_WSTR_6AB1762C9894.as_ptr(), date.wDay as i32);
        }
        THREECHARDAY => {
            GetLocaleInfoW(
                LOCALE_USER_DEFAULT,
                (LOCALE_SABBREVDAYNAME1 + (((date.wDayOfWeek as i32 + 6) % 7)) as u32) as LCTYPE,
                result,
                4,
            );
            /* wsprintfW (result,"%.3s",days[date.wDayOfWeek]); */
        }
        FULLDAY => {
            GetLocaleInfoW(
                LOCALE_USER_DEFAULT,
                (LOCALE_SDAYNAME1 + (((date.wDayOfWeek as i32 + 6) % 7)) as u32) as LCTYPE,
                result,
                resultSize,
            );
        }
        ONEDIGIT12HOUR => {
            if date.wHour as i32 == 0 {
                *result.offset(0 as isize) = '1' as WCHAR;
                *result.offset(1 as isize) = '2' as WCHAR;
                *result.offset(2 as isize) = 0 as WCHAR;
            } else {
                wsprintfW(
                    result,
                    RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                    date.wHour as i32
                        - (if date.wHour as i32 > 12 {
                            12
                        } else {
                            0
                        }),
                );
            }
        }
        TWODIGIT12HOUR => {
            if date.wHour as i32 == 0 {
                *result.offset(0 as isize) = '1' as WCHAR;
                *result.offset(1 as isize) = '2' as WCHAR;
                *result.offset(2 as isize) = 0 as WCHAR;
            } else {
                wsprintfW(
                    result,
                    RUSTCONV_WSTR_6AB1762C9894.as_ptr(),
                    date.wHour as i32
                        - (if date.wHour as i32 > 12 {
                            12
                        } else {
                            0
                        }),
                );
            }
        }
        ONEDIGIT24HOUR => {
            wsprintfW(result, RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), date.wHour as i32);
        }
        TWODIGIT24HOUR => {
            wsprintfW(result, RUSTCONV_WSTR_6AB1762C9894.as_ptr(), date.wHour as i32);
        }
        ONEDIGITSECOND => {
            wsprintfW(result, RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), date.wSecond as i32);
        }
        TWODIGITSECOND => {
            wsprintfW(result, RUSTCONV_WSTR_6AB1762C9894.as_ptr(), date.wSecond as i32);
        }
        ONEDIGITMINUTE => {
            wsprintfW(result, RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), date.wMinute as i32);
        }
        TWODIGITMINUTE => {
            wsprintfW(result, RUSTCONV_WSTR_6AB1762C9894.as_ptr(), date.wMinute as i32);
        }
        ONEDIGITMONTH => {
            wsprintfW(result, RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), date.wMonth as i32);
        }
        TWODIGITMONTH => {
            wsprintfW(result, RUSTCONV_WSTR_6AB1762C9894.as_ptr(), date.wMonth as i32);
        }
        THREECHARMONTH => {
            GetLocaleInfoW(
                LOCALE_USER_DEFAULT,
                (LOCALE_SMONTHNAME1 + (date.wMonth as i32) as u32 - 1) as LCTYPE,
                (&raw mut buffer) as LPWSTR,
                ARRAY_SIZE!(buffer) as i32,
            );
            wsprintfW(result, RUSTCONV_WSTR_D5174DA8E065.as_ptr(), (&raw mut buffer) as *mut WCHAR);
        }
        FULLMONTH => {
            GetLocaleInfoW(
                LOCALE_USER_DEFAULT,
                (LOCALE_SMONTHNAME1 + (date.wMonth as i32) as u32 - 1) as LCTYPE,
                result,
                resultSize,
            );
        }
        ONELETTERAMPM => {
            *result.offset(0 as isize) =
                (if (date.wHour as i32) < 12 { 'A' as i32 } else { 'P' as i32 }) as WCHAR;
            *result.offset(1 as isize) = 0 as WCHAR;
        }
        TWOLETTERAMPM => {
            *result.offset(0 as isize) =
                (if (date.wHour as i32) < 12 { 'A' as i32 } else { 'P' as i32 }) as WCHAR;
            *result.offset(1 as isize) = 'M' as WCHAR;
            *result.offset(2 as isize) = 0 as WCHAR;
        }
        FORMATCALLBACK => {
            /* FIXME ("Not implemented\n"); */
            *result.offset(0 as isize) = 'x' as WCHAR;
            *result.offset(1 as isize) = 0 as WCHAR;
        }
        ONEDIGITYEAR => {
            wsprintfW(result, RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), date.wYear as i32 % 10);
        }
        TWODIGITYEAR => {
            wsprintfW(result, RUSTCONV_WSTR_6AB1762C9894.as_ptr(), date.wYear as i32 % 100);
        }
        INVALIDFULLYEAR | FULLYEAR => {
            wsprintfW(result, RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), date.wYear as i32);
        }
        _ => {}
    }
}

unsafe extern "C" fn wrap(
    mut val: ::core::ffi::c_int,
    mut delta: ::core::ffi::c_int,
    mut minVal: ::core::ffi::c_int,
    mut maxVal: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    val += delta;
    if delta == INT_MIN || val < minVal {
        return maxVal;
    }
    if delta == (INT_MAX as i32) || val > maxVal {
        return minVal;
    }
    return val;
}

unsafe extern "C" fn DATETIME_IncreaseField(
    infoPtr: *mut DATETIME_INFO,
    number: ::core::ffi::c_int,
    mut delta: ::core::ffi::c_int,
) {
    let date: *mut SYSTEMTIME = &raw mut (*infoPtr).date;
    let mut range: [SYSTEMTIME; 2] = core::mem::zeroed();
    let mut limits: DWORD = 0;
    let mut min: BOOL = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DATETIME_IncreaseField\0" as *const u8 as *const ::core::ffi::c_char,
            507 as ::core::ffi::c_int,
            b"%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            number,
        );
    }

    if (number > (*infoPtr).nrFields) || (number < 0) {
        return;
    }

    if (*(*infoPtr).fieldspec.offset(number as isize) & DTHT_DATEFIELD) == 0 {
        return;
    }

    // cb: 0 = end of switch, 1 = day section, 2 = hour section
    let mut cb: u8;
    match *(*infoPtr).fieldspec.offset(number as isize) {
        ONEDIGITYEAR | TWODIGITYEAR | FULLYEAR => {
            if delta == INT_MIN {
                (*date).wYear = 1752;
            } else if delta == (INT_MAX as i32) {
                (*date).wYear = 9999;
            } else {
                // max(min(date->wYear + delta, FACT_0011), FACT_0012) expanded inline
                let t: ::core::ffi::c_int = (*date).wYear as ::core::ffi::c_int + delta;
                let m: ::core::ffi::c_int = if t < 9999 {
                    t
                } else {
                    9999
                };
                (*date).wYear = (if m > 1752 {
                    m
                } else {
                    1752
                }) as WORD;
            }

            if ((*date).wDay as ::core::ffi::c_int)
                > MONTHCAL_MonthLength(
                    (*date).wMonth as ::core::ffi::c_int,
                    (*date).wYear as ::core::ffi::c_int,
                )
            {
                /* This can happen when moving away from a leap year. */
                (*date).wDay = MONTHCAL_MonthLength(
                    (*date).wMonth as ::core::ffi::c_int,
                    (*date).wYear as ::core::ffi::c_int,
                ) as WORD;
            }
            MONTHCAL_CalculateDayOfWeek(date, TRUE);
            cb = 0;
        }
        ONEDIGITMONTH | TWODIGITMONTH | THREECHARMONTH | FULLMONTH => {
            (*date).wMonth = wrap(
                (*date).wMonth as ::core::ffi::c_int,
                delta,
                1,
                12,
            ) as WORD;
            MONTHCAL_CalculateDayOfWeek(date, TRUE);
            delta = 0;
            /* fall through */
            cb = 1;
        }
        ONEDIGITDAY | TWODIGITDAY | THREECHARDAY | FULLDAY => {
            cb = 1;
        }
        ONELETTERAMPM | TWOLETTERAMPM => {
            delta *= 12;
            /* fall through */
            cb = 2;
        }
        ONEDIGIT12HOUR | TWODIGIT12HOUR | ONEDIGIT24HOUR | TWODIGIT24HOUR => {
            cb = 2;
        }
        ONEDIGITMINUTE | TWODIGITMINUTE => {
            (*date).wMinute = wrap(
                (*date).wMinute as ::core::ffi::c_int,
                delta,
                0,
                59,
            ) as WORD;
            cb = 0;
        }
        ONEDIGITSECOND | TWODIGITSECOND => {
            (*date).wSecond = wrap(
                (*date).wSecond as ::core::ffi::c_int,
                delta,
                0,
                59,
            ) as WORD;
            cb = 0;
        }
        FORMATCALLBACK => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/datetime.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"DATETIME_IncreaseField\0" as *const u8 as *const ::core::ffi::c_char,
                    562 as ::core::ffi::c_int,
                    b"Not implemented\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            cb = 0;
        }
        _ => {
            cb = 0;
        }
    }

    match cb {
        1 => {
            (*date).wDay = wrap(
                (*date).wDay as ::core::ffi::c_int,
                delta,
                1,
                MONTHCAL_MonthLength(
                    (*date).wMonth as ::core::ffi::c_int,
                    (*date).wYear as ::core::ffi::c_int,
                ),
            ) as WORD;
            MONTHCAL_CalculateDayOfWeek(date, TRUE);
        }
        2 => {
            (*date).wHour = wrap(
                (*date).wHour as ::core::ffi::c_int,
                delta,
                0,
                23,
            ) as WORD;
        }
        _ => {}
    }

    /* FYI: On 1752/9/14 the calendar changed and England and the
     * American colonies changed to the Gregorian calendar. This change
     * involved having September 14th follow September 2nd. So no date
     * algorithm works before that date.
     */
    if (10000 * (*date).wYear as ::core::ffi::c_int
        + 100 * (*date).wMonth as ::core::ffi::c_int
        + (*date).wDay as ::core::ffi::c_int)
        < 17520914
    {
        (*date).wYear = 1752;
        (*date).wMonth = 9;
        (*date).wDay = 14;
        (*date).wSecond = 0;
        (*date).wMinute = 0;
        (*date).wHour = 0;
    }

    /* Ensure time is within bounds */
    limits = SendMessageW(
        (*infoPtr).hMonthCal,
        MCM_GETRANGE,
        0,
        core::ptr::addr_of_mut!(range) as *mut SYSTEMTIME as LPARAM,
    ) as DWORD;
    min = (delta < 0) as ::core::ffi::c_int as BOOL;

    if (limits & (if min != 0 { GDTR_MIN } else { GDTR_MAX }) as DWORD) != 0
    {
        let i: ::core::ffi::c_int = if min != 0 {
            0
        } else {
            1
        };

        if MONTHCAL_CompareSystemTime(
            date,
            (core::ptr::addr_of_mut!(range) as *mut SYSTEMTIME).offset(i as isize) as *mut SYSTEMTIME,
        ) == (if min != 0 {
            -1
        } else {
            1
        })
        {
            (*date).wYear = range[i as usize].wYear;
            (*date).wMonth = range[i as usize].wMonth;
            (*date).wDayOfWeek = range[i as usize].wDayOfWeek;
            (*date).wDay = range[i as usize].wDay;
            (*date).wHour = range[i as usize].wHour;
            (*date).wMinute = range[i as usize].wMinute;
            (*date).wSecond = range[i as usize].wSecond;
            (*date).wMilliseconds = range[i as usize].wMilliseconds;
        }
    }
}

unsafe extern "C" fn DATETIME_GetFieldWidth(infoPtr: *const DATETIME_INFO, hdc: HDC, count: ::core::ffi::c_int) -> ::core::ffi::c_int {
    /* fields are a fixed width, determined by the largest possible string */
    /* presumably, these widths should be language dependent */
    let mut spec: ::core::ffi::c_int = 0;
    let mut buffer: [WCHAR; 80] = [0; 80];
    let mut bufptr: LPCWSTR = core::ptr::null();
    let mut size: SIZE = SIZE { cx: 0, cy: 0 };

    if (*infoPtr).fieldspec.is_null() {
        return 0;
    }

    spec = *(*infoPtr).fieldspec.offset(count as isize);
    if (spec & DT_STRING) != 0 {
        let mut txtlen: ::core::ffi::c_int = *(*infoPtr).buflen.offset(count as isize);

        if txtlen > 79 {
            txtlen = 79;
        }
        memcpy(
            &raw mut buffer as *mut WCHAR as *mut ::core::ffi::c_void,
            (&raw const (*infoPtr).textbuf as *const WCHAR).offset((spec & !DT_STRING) as isize)
                as *const ::core::ffi::c_void,
            (txtlen as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        buffer[txtlen as usize] = 0 as WCHAR;
        bufptr = &raw mut buffer as *mut WCHAR as LPCWSTR;
    } else {
        match spec {
            ONEDIGITDAY
            | ONEDIGIT12HOUR
            | ONEDIGIT24HOUR
            | ONEDIGITSECOND
            | ONEDIGITMINUTE
            | ONEDIGITMONTH
            | ONEDIGITYEAR
            | TWODIGITDAY
            | TWODIGIT12HOUR
            | TWODIGIT24HOUR
            | TWODIGITSECOND
            | TWODIGITMINUTE
            | TWODIGITMONTH
            | TWODIGITYEAR => {
                /* these seem to use a two byte field */
                bufptr = RUSTCONV_WSTR_66039A3ABDA7.as_ptr();
            }
            INVALIDFULLYEAR | FULLYEAR => {
                bufptr = RUSTCONV_WSTR_1871D60992D7.as_ptr();
            }
            THREECHARMONTH | FULLMONTH | THREECHARDAY | FULLDAY => {
                let mut fall: *const WCHAR;
                let mut lctype: LCTYPE;
                let mut i: INT;
                let mut max_count: INT;
                let mut cx: LONG;

                /* choose locale data type and fallback string */
                match spec {
                    THREECHARDAY => {
                        fall = RUSTCONV_WSTR_D09E79007215.as_ptr();
                        lctype = LOCALE_SABBREVDAYNAME1 as LCTYPE;
                        max_count = 7 as INT;
                    }
                    FULLDAY => {
                        fall = RUSTCONV_WSTR_E1DBD501A112.as_ptr();
                        lctype = LOCALE_SDAYNAME1 as LCTYPE;
                        max_count = 7 as INT;
                    }
                    THREECHARMONTH => {
                        fall = RUSTCONV_WSTR_AC9F9795F0E6.as_ptr();
                        lctype = LOCALE_SABBREVMONTHNAME1 as LCTYPE;
                        max_count = 12 as INT;
                    }
                    _ => {
                        /* FULLMONTH */
                        fall = RUSTCONV_WSTR_664E177C91D5.as_ptr();
                        lctype = LOCALE_SMONTHNAME1 as LCTYPE;
                        max_count = 12 as INT;
                    }
                }

                cx = 0 as LONG;
                i = 0 as INT;
                while i < max_count {
                    if GetLocaleInfoW(
                        LOCALE_USER_DEFAULT as LCTYPE,
                        lctype.wrapping_add(i as LCTYPE),
                        &raw mut buffer as *mut WCHAR as LPWSTR,
                        ARRAY_SIZE!(buffer) as ::core::ffi::c_int,
                    ) != 0
                    {
                        GetTextExtentPoint32W(
                            hdc,
                            &raw mut buffer as *mut WCHAR as LPCWSTR,
                            lstrlenW(&raw mut buffer as *mut WCHAR as LPCWSTR),
                            &raw mut size,
                        );
                        if size.cx > cx {
                            cx = size.cx;
                        }
                        i += 1;
                    } else {
                        /* locale independent fallback on failure */
                        GetTextExtentPoint32W(
                            hdc,
                            fall as LPCWSTR,
                            lstrlenW(fall as LPCWSTR),
                            &raw mut size,
                        );
                        cx = size.cx;
                        break;
                    }
                }
                return cx as ::core::ffi::c_int;
            }
            ONELETTERAMPM => {
                bufptr = RUSTCONV_WSTR_DAAF62DE76F0.as_ptr();
            }
            TWOLETTERAMPM => {
                bufptr = RUSTCONV_WSTR_FE60D71583A9.as_ptr();
            }
            _ => {
                bufptr = RUSTCONV_WSTR_85DD751867E3.as_ptr();
            }
        }
    }
    GetTextExtentPoint32W(hdc, bufptr, lstrlenW(bufptr), &raw mut size);
    return size.cx as ::core::ffi::c_int;
}

unsafe extern "C" fn DATETIME_Refresh(infoPtr: *mut DATETIME_INFO, hdc: HDC) {
    if (*infoPtr).dateValid != 0 {
        let mut i: ::core::ffi::c_int = 0;
        let mut prevright: ::core::ffi::c_int = 0;
        let mut field: *mut RECT = ::core::ptr::null_mut::<RECT>();
        let mut rcDraw: *mut RECT = &raw mut (*infoPtr).rcDraw;
        let mut size: SIZE = tagSIZE { cx: 0, cy: 0 };
        let mut oldTextColor: COLORREF = 0;
        let mut oldFont: HFONT = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
        let mut oldBkMode: INT = SetBkMode(hdc, (TRANSPARENT as i32)) as INT;
        let mut txt: [WCHAR; 80] = [0; 80];

        DATETIME_ReturnTxt(
            infoPtr,
            0,
            &raw mut txt as LPWSTR,
            ARRAY_SIZE!(txt) as ::core::ffi::c_int,
        );
        GetTextExtentPoint32W(
            hdc,
            &raw mut txt as *mut WCHAR as LPCWSTR,
            lstrlenW(&raw mut txt as *mut WCHAR as LPCWSTR),
            &raw mut size,
        );
        (*rcDraw).bottom = (size.cy as ::core::ffi::c_int + 2) as LONG;

        (*infoPtr).checkbox.right = (if (*infoPtr).dwStyle & DTS_SHOWNONE as DWORD != 0 {
            18
        } else {
            2
        }) as LONG;
        prevright = (*infoPtr).checkbox.right as ::core::ffi::c_int;

        i = 0 as ::core::ffi::c_int;
        while i < (*infoPtr).nrFields {
            DATETIME_ReturnTxt(
                infoPtr,
                i,
                &raw mut txt as LPWSTR,
                ARRAY_SIZE!(txt) as ::core::ffi::c_int,
            );
            GetTextExtentPoint32W(
                hdc,
                &raw mut txt as *mut WCHAR as LPCWSTR,
                lstrlenW(&raw mut txt as *mut WCHAR as LPCWSTR),
                &raw mut size,
            );
            field = (*infoPtr).fieldRect.offset(i as isize) as *mut RECT;
            (*field).left = prevright as LONG;
            (*field).right = (prevright + DATETIME_GetFieldWidth(infoPtr, hdc, i)) as LONG;
            (*field).top = (*rcDraw).top;
            (*field).bottom = (*rcDraw).bottom;
            prevright = (*field).right as ::core::ffi::c_int;

            if (*infoPtr).dwStyle & WS_DISABLED as DWORD != 0 {
                oldTextColor = SetTextColor(hdc, comctl32_color.clrGrayText);
            } else if (*infoPtr).haveFocus != 0 && i == (*infoPtr).select {
                let mut selection: RECT = tagRECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                let mut hbr: HBRUSH = CreateSolidBrush(comctl32_color.clrActiveCaption);

                if (*infoPtr).nCharsEntered != 0 {
                    memcpy(
                        &raw mut txt as *mut WCHAR as *mut ::core::ffi::c_void,
                        &raw mut (*infoPtr).charsEntered as *mut WCHAR as *const ::core::ffi::c_void,
                        ((*infoPtr).nCharsEntered as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
                    );
                    txt[(*infoPtr).nCharsEntered as usize] = 0 as WCHAR;
                    GetTextExtentPoint32W(
                        hdc,
                        &raw mut txt as *mut WCHAR as LPCWSTR,
                        lstrlenW(&raw mut txt as *mut WCHAR as LPCWSTR),
                        &raw mut size,
                    );
                }

                SetRect(
                    &raw mut selection,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    size.cx as ::core::ffi::c_int,
                    size.cy as ::core::ffi::c_int,
                );
                /* center rectangle */
                OffsetRect(
                    &raw mut selection,
                    ((*field).right as ::core::ffi::c_int
                        + (*field).left as ::core::ffi::c_int
                        - size.cx as ::core::ffi::c_int)
                        / 2 as ::core::ffi::c_int,
                    ((*field).bottom as ::core::ffi::c_int - size.cy as ::core::ffi::c_int)
                        / 2 as ::core::ffi::c_int,
                );

                FillRect(hdc, &raw mut selection as LPCRECT, hbr);
                DeleteObject(hbr as HGDIOBJ);
                oldTextColor = SetTextColor(hdc, comctl32_color.clrWindow);
            } else {
                oldTextColor = SetTextColor(hdc, comctl32_color.clrWindowText);
            }

            /* draw the date text using the colour set above */
            DrawTextW(
                hdc,
                &raw mut txt as *mut WCHAR as LPCWSTR,
                lstrlenW(&raw mut txt as *mut WCHAR as LPCWSTR),
                field as LPRECT,
                (DT_CENTER | DT_VCENTER | DT_SINGLELINE) as UINT,
            );
            SetTextColor(hdc, oldTextColor);
            i += 1;
        }
        SetBkMode(hdc, oldBkMode as ::core::ffi::c_int);
        SelectObject(hdc, oldFont as HGDIOBJ);
    }

    if (*infoPtr).dwStyle & DTS_UPDOWN as DWORD == 0 {
        DrawFrameControl(
            hdc,
            &raw mut (*infoPtr).calbutton,
            DFC_SCROLL as UINT,
            (DFCS_SCROLLDOWN
                | (if (*infoPtr).bCalDepressed != 0 {
                    DFCS_PUSHED
                } else {
                    0
                })
                | (if (*infoPtr).dwStyle & WS_DISABLED as DWORD != 0 {
                    DFCS_INACTIVE
                } else {
                    0
                })) as UINT,
        );
    }
}

unsafe extern "C" fn DATETIME_HitTest(mut infoPtr: *const DATETIME_INFO, mut pt: POINT) -> INT {
    let mut i: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DATETIME_HitTest\0" as *const u8 as *const ::core::ffi::c_char,
            792 as ::core::ffi::c_int,
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_point(&raw mut pt),
        );
    }
    if PtInRect(&raw const (*infoPtr).calbutton, pt) != 0 {
        return DTHT_MCPOPUP;
    }
    if PtInRect(&raw const (*infoPtr).checkbox, pt) != 0 {
        return DTHT_CHECKBOX;
    }
    i = 0;
    while i < (*infoPtr).nrFields {
        if PtInRect(
            (*infoPtr).fieldRect.offset(i as isize) as *const RECT,
            pt,
        ) != 0
        {
            return i as INT;
        }
        i += 1;
    }
    return DTHT_NONE;
}

unsafe extern "C" fn DATETIME_GetPrevDateField(infoPtr: *const DATETIME_INFO, mut i: ::core::ffi::c_int) -> ::core::ffi::c_int {
    i -= 1;
    while i >= 0 {
        if *(*infoPtr).fieldspec.offset(i as isize) & DTHT_DATEFIELD != 0 {
            return i;
        }
        i -= 1;
    }
    return -1;
}

unsafe extern "C" fn DATETIME_ApplySelectedField(mut infoPtr: *mut DATETIME_INFO) {
    let mut fieldNum: ::core::ffi::c_int = (*infoPtr).select & DTHT_DATEFIELD;
    let mut i: ::core::ffi::c_int = 0;
    let mut val: ::core::ffi::c_int = 0;
    let mut clamp_day: BOOL = FALSE;
    let mut date: SYSTEMTIME = (*infoPtr).date;
    let mut oldyear: ::core::ffi::c_int;
    if (*infoPtr).select == -(1)
        || (*infoPtr).nCharsEntered == 0
    {
        return;
    }
    if *(*infoPtr).fieldspec.offset(fieldNum as isize) == ONELETTERAMPM
        || *(*infoPtr).fieldspec.offset(fieldNum as isize) == TWOLETTERAMPM
    {
        val = (*infoPtr).charsEntered[0 as usize] as ::core::ffi::c_int;
    } else {
        i = 0;
        while i < (*infoPtr).nCharsEntered {
            val = val * 10
                + (*infoPtr).charsEntered[i as usize] as ::core::ffi::c_int
                - '0' as ::core::ffi::c_int;
            i += 1;
        }
    }
    (*infoPtr).nCharsEntered = 0;
    match *(*infoPtr).fieldspec.offset(fieldNum as isize) {
        ONEDIGITYEAR | TWODIGITYEAR => {
            oldyear = date.wYear as ::core::ffi::c_int;
            date.wYear = (date.wYear as ::core::ffi::c_int
                - date.wYear as ::core::ffi::c_int % 100
                + val) as WORD;
            if DATETIME_IsDateInValidRange(infoPtr, core::ptr::addr_of_mut!(date)) != 0 {
                clamp_day = TRUE;
            } else {
                date.wYear = oldyear as WORD;
            }
        }
        INVALIDFULLYEAR | FULLYEAR => {
            oldyear = date.wYear as ::core::ffi::c_int;
            date.wYear = val as WORD;
            if DATETIME_IsDateInValidRange(infoPtr, core::ptr::addr_of_mut!(date)) != 0 {
                clamp_day = TRUE;
            } else {
                date.wYear = oldyear as WORD;
            }
        }
        ONEDIGITMONTH | TWODIGITMONTH => {
            date.wMonth = val as WORD;
            clamp_day = TRUE;
        }
        ONEDIGITDAY | TWODIGITDAY => {
            date.wDay = val as WORD;
        }
        ONEDIGIT12HOUR | TWODIGIT12HOUR => {
            if val >= 24 {
                val -= 20;
            }
            if val >= 13 {
                date.wHour = val as WORD;
            } else if val != 0 {
                if date.wHour as ::core::ffi::c_int >= 12 {
                    date.wHour = (if val == 12 {
                        12
                    } else {
                        val + 12
                    }) as WORD;
                } else {
                    date.wHour = (if val == 12 {
                        0
                    } else {
                        val
                    }) as WORD;
                }
            }
        }
        ONEDIGIT24HOUR | TWODIGIT24HOUR => {
            date.wHour = val as WORD;
        }
        ONEDIGITMINUTE | TWODIGITMINUTE => {
            date.wMinute = val as WORD;
        }
        ONEDIGITSECOND | TWODIGITSECOND => {
            date.wSecond = val as WORD;
        }
        ONELETTERAMPM | TWOLETTERAMPM => {
            if val == 'a' as ::core::ffi::c_int || val == 'A' as ::core::ffi::c_int {
                if date.wHour as ::core::ffi::c_int >= 12 {
                    date.wHour = (date.wHour as ::core::ffi::c_int - 12) as WORD;
                }
            } else if val == 'p' as ::core::ffi::c_int || val == 'P' as ::core::ffi::c_int {
                if (date.wHour as ::core::ffi::c_int) < 12 {
                    date.wHour = (date.wHour as ::core::ffi::c_int + 12) as WORD;
                }
            }
        }
        _ => {}
    }
    if clamp_day != 0
        && date.wDay as ::core::ffi::c_int
            > MONTHCAL_MonthLength(
                date.wMonth as ::core::ffi::c_int,
                date.wYear as ::core::ffi::c_int,
            )
    {
        date.wDay = MONTHCAL_MonthLength(
            date.wMonth as ::core::ffi::c_int,
            date.wYear as ::core::ffi::c_int,
        ) as WORD;
    }
    if DATETIME_SetSystemTime(
        infoPtr,
        GDT_VALID as DWORD,
        core::ptr::addr_of_mut!(date),
    ) != 0
    {
        DATETIME_SendDateTimeChangeNotify(infoPtr);
    }
}

unsafe extern "C" fn DATETIME_SetSelectedField(
    infoPtr: *mut DATETIME_INFO,
    select: core::ffi::c_int,
) {
    DATETIME_ApplySelectedField(infoPtr);

    (*infoPtr).select = select;
    (*infoPtr).nCharsEntered = 0;
}

unsafe extern "C" fn DATETIME_LButtonDown(infoPtr: *mut DATETIME_INFO, x: INT, y: INT) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut new: i32 = 0;

    pt.x = x;
    pt.y = y;
    new = DATETIME_HitTest(infoPtr, pt);

    SetFocus((*infoPtr).hwndSelf);

    if (new & DTHT_NODATEMASK) == 0 || new == DTHT_NONE {
        if new == DTHT_NONE {
            new = (*infoPtr).nrFields - 1;
        } else {
            /* hitting string part moves selection to next date field to left */
            if (*(*infoPtr).fieldspec.offset(new as isize) & DT_STRING) != 0 {
                new = DATETIME_GetPrevDateField(infoPtr, new);
                if new == -1 {
                    return 0;
                }
            }
            /* never select full day of week */
            if *(*infoPtr).fieldspec.offset(new as isize) == FULLDAY {
                return 0;
            }
        }
    }

    DATETIME_SetSelectedField(infoPtr, new);

    if (*infoPtr).select == DTHT_MCPOPUP {
        let mut rcMonthCal: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let mut pos: POINT = POINT { x: 0, y: 0 };
        SendMessageW(
            (*infoPtr).hMonthCal,
            MCM_GETMINREQRECT as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(rcMonthCal) as LPARAM,
        );

        /* FIXME: button actually is only depressed during dropdown of the */
        /* calendar control and when the mouse is over the button window */
        (*infoPtr).bCalDepressed = TRUE;

        /* recalculate the position of the monthcal popup */
        if ((*infoPtr).dwStyle & DTS_RIGHTALIGN) != 0 {
            pos.x = (*infoPtr).calbutton.left - (rcMonthCal.right - rcMonthCal.left);
        } else {
            /* FIXME: this should be after the area reserved for the checkbox */
            pos.x = (*infoPtr).rcDraw.left;
        }

        pos.y = (*infoPtr).rcClient.bottom;
        OffsetRect(
            core::ptr::addr_of_mut!(rcMonthCal),
            pos.x as i32,
            pos.y as i32,
        );
        MapWindowPoints(
            (*infoPtr).hwndSelf,
            0 as HWND,
            core::ptr::addr_of_mut!(rcMonthCal) as LPPOINT,
            2 as UINT,
        );
        SetWindowPos(
            (*infoPtr).hMonthCal,
            0 as HWND,
            rcMonthCal.left as i32,
            rcMonthCal.top as i32,
            rcMonthCal.right as i32 - rcMonthCal.left as i32,
            rcMonthCal.bottom as i32 - rcMonthCal.top as i32,
            0 as UINT,
        );

        if IsWindowVisible((*infoPtr).hMonthCal) != 0 {
            ShowWindow((*infoPtr).hMonthCal, (SW_HIDE) as i32);
            (*infoPtr).bDropdownEnabled = FALSE;
            DATETIME_SendSimpleNotify(infoPtr, DTN_CLOSEUP as UINT);
        } else {
            let lprgSysTimeArray: *const SYSTEMTIME = core::ptr::addr_of!((*infoPtr).date);

            SendMessageW(
                (*infoPtr).hMonthCal,
                MCM_SETCURSEL as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!((*infoPtr).date) as LPARAM,
            );

            if (*infoPtr).bDropdownEnabled != 0 {
                ShowWindow((*infoPtr).hMonthCal, (SW_SHOW) as i32);
                DATETIME_SendSimpleNotify(infoPtr, DTN_DROPDOWN as UINT);
            }
            (*infoPtr).bDropdownEnabled = TRUE;
        }
    }

    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null(),
        TRUE as BOOL,
    );

    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_LButtonUp(infoPtr: *mut DATETIME_INFO) -> LRESULT {
    if (*infoPtr).bCalDepressed != 0 {
        (*infoPtr).bCalDepressed = FALSE;
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!((*infoPtr).calbutton) as LPCRECT, TRUE);
    }

    return 0;
}

unsafe extern "C" fn DATETIME_Paint(infoPtr: *mut DATETIME_INFO, mut hdc: HDC) -> LRESULT {
    if hdc.is_null() {
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        hdc = BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
        DATETIME_Refresh(infoPtr, hdc);
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    } else {
        DATETIME_Refresh(infoPtr, hdc);
    }

    /* Not a click on the dropdown box, enabled it */
    (*infoPtr).bDropdownEnabled = TRUE;

    return 0;
}

unsafe extern "C" fn DATETIME_Button_Command(
    mut infoPtr: *mut DATETIME_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
        let mut state: DWORD = SendMessageW(
            lParam as HWND,
            BM_GETCHECK as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as DWORD;
        (*infoPtr).dateValid = (state == BST_CHECKED as DWORD) as ::core::ffi::c_int as BOOL;
        InvalidateRect(
            (*infoPtr).hwndSelf,
            ::core::ptr::null::<RECT>(),
            TRUE,
        );
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_Command(
    infoPtr: *mut DATETIME_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if (*infoPtr).hwndCheckbut == lParam as HWND {
        return DATETIME_Button_Command(infoPtr, wParam, lParam);
    }
    return 0;
}

unsafe extern "C" fn DATETIME_Enable(infoPtr: *mut DATETIME_INFO, bEnable: BOOL) -> LRESULT {
    if bEnable != 0 {
        (*infoPtr).dwStyle &= !(WS_DISABLED as DWORD);
    } else {
        (*infoPtr).dwStyle |= WS_DISABLED as DWORD;
    }

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE);

    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_EraseBackground(
    infoPtr: *const DATETIME_INFO,
    hdc: HDC,
) -> LRESULT {
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut hSolidBrush: HBRUSH = core::ptr::null_mut();
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if (*infoPtr).dwStyle & WS_DISABLED as DWORD != 0 {
        hBrush = CreateSolidBrush(comctl32_color.clrBtnFace);
        hSolidBrush = hBrush;
    } else {
        hBrush = SendMessageW(
            (*infoPtr).hwndNotify,
            WM_CTLCOLOREDIT as UINT,
            hdc as WPARAM,
            (*infoPtr).hwndSelf as LPARAM,
        ) as HBRUSH;
        if hBrush.is_null() {
            hBrush = CreateSolidBrush(comctl32_color.clrWindow);
            hSolidBrush = hBrush;
        }
    }

    GetClientRect((*infoPtr).hwndSelf, &raw mut rc);

    FillRect(hdc, &raw mut rc as LPCRECT, hBrush);

    if !hSolidBrush.is_null() {
        DeleteObject(hSolidBrush as HGDIOBJ);
    }

    return -(1 as LRESULT);
}

unsafe extern "C" fn DATETIME_Notify(
    mut infoPtr: *mut DATETIME_INFO,
    mut lpnmh: *const NMHDR,
) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DATETIME_Notify\0" as *const u8 as *const ::core::ffi::c_char,
            1102 as ::core::ffi::c_int,
            b"Got notification %x from %p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*lpnmh).code,
            (*lpnmh).hwndFrom,
        );
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DATETIME_Notify\0" as *const u8 as *const ::core::ffi::c_char,
            1103 as ::core::ffi::c_int,
            b"info: %p %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            (*infoPtr).hMonthCal,
            (*infoPtr).hUpdown,
        );
    }
    if (*lpnmh).code == (MCN_SELECT) as u32 {
        ShowWindow((*infoPtr).hMonthCal, (SW_HIDE) as i32);
        (*infoPtr).dateValid = TRUE as BOOL;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/datetime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DATETIME_Notify\0" as *const u8 as *const ::core::ffi::c_char,
                1110 as ::core::ffi::c_int,
                b"got from calendar %04d/%02d/%02d day of week %d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*infoPtr).date.wYear as ::core::ffi::c_int,
                (*infoPtr).date.wMonth as ::core::ffi::c_int,
                (*infoPtr).date.wDay as ::core::ffi::c_int,
                (*infoPtr).date.wDayOfWeek as ::core::ffi::c_int,
            );
        }
        SendMessageW(
            (*infoPtr).hMonthCal,
            MCM_GETCURSEL as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!((*infoPtr).date) as LPARAM,
        );
        SendMessageW(
            (*infoPtr).hwndCheckbut,
            BM_SETCHECK as UINT,
            BST_CHECKED as WPARAM,
            0 as LPARAM,
        );
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE as BOOL);
        DATETIME_SendDateTimeChangeNotify(infoPtr);
        DATETIME_SendSimpleNotify(infoPtr, DTN_CLOSEUP as UINT);
    }
    if (*lpnmh).hwndFrom == (*infoPtr).hUpdown && (*lpnmh).code == (UDN_DELTAPOS) as u32 {
        let mut lpnmud: *const NM_UPDOWN = lpnmh as *const NM_UPDOWN;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/datetime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DATETIME_Notify\0" as *const u8 as *const ::core::ffi::c_char,
                1118 as ::core::ffi::c_int,
                b"Delta pos %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*lpnmud).iDelta,
            );
        }
        (*infoPtr).pendingUpdown = (*lpnmud).iDelta;
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_KeyDown(
    mut infoPtr: *mut DATETIME_INFO,
    mut vkCode: DWORD,
) -> LRESULT {
    let mut fieldNum: ::core::ffi::c_int = (*infoPtr).select & DTHT_DATEFIELD;
    let mut wrap_0: ::core::ffi::c_int = 0;
    let mut new: ::core::ffi::c_int = 0;
    if (*infoPtr).haveFocus == 0 {
        return 0 as LRESULT;
    }
    if fieldNum == 0 && (*infoPtr).select != 0 {
        return 0 as LRESULT;
    }
    if (*infoPtr).select & FORMATCALLMASK != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/datetime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DATETIME_KeyDown\0" as *const u8 as *const ::core::ffi::c_char,
                1136 as ::core::ffi::c_int,
                b"Callbacks not implemented yet\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    }
    match vkCode {
        VK_ADD | VK_UP => {
            (*infoPtr).nCharsEntered = 0;
            DATETIME_IncreaseField(infoPtr, fieldNum, 1);
            DATETIME_SendDateTimeChangeNotify(infoPtr);
        }
        VK_SUBTRACT | VK_DOWN => {
            (*infoPtr).nCharsEntered = 0;
            DATETIME_IncreaseField(infoPtr, fieldNum, -1);
            DATETIME_SendDateTimeChangeNotify(infoPtr);
        }
        VK_HOME => {
            (*infoPtr).nCharsEntered = 0;
            DATETIME_IncreaseField(infoPtr, fieldNum, INT_MIN);
            DATETIME_SendDateTimeChangeNotify(infoPtr);
        }
        VK_END => {
            (*infoPtr).nCharsEntered = 0;
            DATETIME_IncreaseField(infoPtr, fieldNum, (INT_MAX as i32));
            DATETIME_SendDateTimeChangeNotify(infoPtr);
        }
        VK_LEFT => {
            new = (*infoPtr).select;
            loop {
                if new == 0 {
                    new = new - 1;
                    wrap_0 += 1;
                } else {
                    new -= 1;
                }
                if !((*(*infoPtr).fieldspec.offset(new as isize)) & DT_STRING != 0
                    && wrap_0 < 2)
                {
                    break;
                }
            }
            if new != (*infoPtr).select {
                DATETIME_SetSelectedField(infoPtr, new);
            }
        }
        VK_RIGHT => {
            new = (*infoPtr).select;
            loop {
                new += 1;
                if new == (*infoPtr).nrFields {
                    new = 0;
                    wrap_0 += 1;
                }
                if !((*(*infoPtr).fieldspec.offset(new as isize)) & DT_STRING != 0
                    && wrap_0 < 2)
                {
                    break;
                }
            }
            if new != (*infoPtr).select {
                DATETIME_SetSelectedField(infoPtr, new);
            }
        }
        _ => {}
    }
    InvalidateRect((*infoPtr).hwndSelf, ::core::ptr::null::<RECT>(), TRUE);
    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_Char(infoPtr: *mut DATETIME_INFO, vkCode: WPARAM) -> LRESULT {
    let mut fieldNum: ::core::ffi::c_int = 0;
    let mut fieldSpec: ::core::ffi::c_int = 0;

    fieldNum = (*infoPtr).select & DTHT_DATEFIELD;
    fieldSpec = *(*infoPtr).fieldspec.offset(fieldNum as isize);

    if fieldSpec == ONELETTERAMPM || fieldSpec == TWOLETTERAMPM {
        (*infoPtr).charsEntered[0 as usize] = vkCode as WCHAR;
        (*infoPtr).nCharsEntered = 1;

        DATETIME_ApplySelectedField(infoPtr);
    } else if vkCode >= '0' as WPARAM && vkCode <= '9' as WPARAM {
        let mut maxChars: ::core::ffi::c_int = 0;

        let fresh17 = (*infoPtr).nCharsEntered;
        (*infoPtr).nCharsEntered = fresh17 + 1;
        (*infoPtr).charsEntered[fresh17 as usize] = vkCode as WCHAR;

        if fieldSpec == INVALIDFULLYEAR || fieldSpec == FULLYEAR {
            maxChars = 4;
        } else {
            maxChars = 2;
        }

        if (fieldSpec == ONEDIGIT12HOUR
            || fieldSpec == TWODIGIT12HOUR
            || fieldSpec == ONEDIGIT24HOUR
            || fieldSpec == TWODIGIT24HOUR)
            && (*infoPtr).nCharsEntered == 1
        {
            if vkCode >= '3' as WPARAM {
                maxChars = 1;
            }
        }

        if maxChars == (*infoPtr).nCharsEntered {
            DATETIME_ApplySelectedField(infoPtr);
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_VScroll(infoPtr: *mut DATETIME_INFO, wScroll: WORD) -> LRESULT {
    let mut fieldNum: ::core::ffi::c_int = (*infoPtr).select & DTHT_DATEFIELD;

    if ((LOWORD!(wScroll as DWORD) as SHORT) as ::core::ffi::c_int) as u32 != SB_THUMBPOSITION {
        return 0 as LRESULT;
    }
    if !((*infoPtr).haveFocus != 0) {
        return 0 as LRESULT;
    }
    if (fieldNum == 0) && ((*infoPtr).select != 0) {
        return 0 as LRESULT;
    }

    if (*infoPtr).pendingUpdown >= 0 {
        DATETIME_IncreaseField(infoPtr, fieldNum, 1);
        DATETIME_SendDateTimeChangeNotify(infoPtr);
    } else {
        DATETIME_IncreaseField(infoPtr, fieldNum, -1);
        DATETIME_SendDateTimeChangeNotify(infoPtr);
    }

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE);

    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_KillFocus(
    mut infoPtr: *mut DATETIME_INFO,
    mut lostFocus: HWND,
) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DATETIME_KillFocus\0" as *const u8 as *const ::core::ffi::c_char,
            1263 as ::core::ffi::c_int,
            b"lost focus to %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            lostFocus,
        );
    }
    if (*infoPtr).haveFocus != 0 {
        DATETIME_SendSimpleNotify(infoPtr, NM_KILLFOCUS);
        (*infoPtr).haveFocus = 0;
        DATETIME_SetSelectedField(infoPtr, -1);
    }
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE);
    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_NCCreate(hwnd: HWND, lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut dwExStyle: DWORD = GetWindowLongW(hwnd, GWL_EXSTYLE) as DWORD;
    /* force control to have client edge */
    dwExStyle |= WS_EX_CLIENTEDGE as DWORD;
    SetWindowLongW(hwnd, GWL_EXSTYLE, dwExStyle as LONG);

    return 1 as LRESULT;
}

unsafe extern "C" fn DATETIME_SetFocus(infoPtr: *mut DATETIME_INFO, lostFocus: HWND) -> LRESULT {
    if !(*infoPtr).hMonthCal.is_null()
        && (lostFocus == (*infoPtr).hMonthCal)
        && IsWindowVisible((*infoPtr).hMonthCal) != 0
    {
        ShowWindow((*infoPtr).hMonthCal, (SW_HIDE) as i32);
        DATETIME_SendSimpleNotify(infoPtr, (DTN_CLOSEUP) as UINT);
        (*infoPtr).bDropdownEnabled = FALSE;
        return 0;
    }

    if (*infoPtr).haveFocus == 0 {
        DATETIME_SendSimpleNotify(infoPtr, NM_SETFOCUS);
        (*infoPtr).haveFocus = DTHT_GOTFOCUS;
    }

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), FALSE);

    return 0;
}

unsafe extern "C" fn DATETIME_SendDateTimeChangeNotify(infoPtr: *const DATETIME_INFO) -> BOOL {
    let mut dtdtc: NMDATETIMECHANGE = core::mem::zeroed();

    dtdtc.nmhdr.hwndFrom = (*infoPtr).hwndSelf;
    dtdtc.nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
    dtdtc.nmhdr.code = DTN_DATETIMECHANGE as UINT;

    dtdtc.dwFlags = (if (*infoPtr).dateValid != 0 {
        GDT_VALID
    } else {
        GDT_NONE
    }) as DWORD;

    dtdtc.st = (*infoPtr).date;
    return SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        dtdtc.nmhdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(dtdtc) as LPARAM,
    ) as BOOL;
}

unsafe extern "C" fn DATETIME_SendSimpleNotify(infoPtr: *const DATETIME_INFO, code: UINT) -> BOOL {
    let mut nmhdr: NMHDR = core::mem::zeroed();
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const i8,
            b"DATETIME_SendSimpleNotify\0" as *const u8 as *const i8,
            1340 as i32,
            b"%x\n\0" as *const u8 as *const i8,
            code,
        );
    }
    nmhdr.hwndFrom = (*infoPtr).hwndSelf;
    nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
    nmhdr.code = code;
    return SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nmhdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmhdr) as LPARAM,
    ) as BOOL;
}

unsafe extern "C" fn DATETIME_Size(infoPtr: *mut DATETIME_INFO, width: INT, height: INT) -> LRESULT {
    /* set size */
    (*infoPtr).rcClient.bottom = height as LONG;
    (*infoPtr).rcClient.right = width as LONG;

    (*infoPtr).rcDraw = (*infoPtr).rcClient;

    if ((*infoPtr).dwStyle & DTS_UPDOWN) != 0 {
        SetWindowPos(
            (*infoPtr).hUpdown,
            core::ptr::null_mut(),
            ((*infoPtr).rcClient.right as INT - 14),
            0,
            15,
            ((*infoPtr).rcClient.bottom as INT - (*infoPtr).rcClient.top as INT),
            (SWP_NOACTIVATE | SWP_NOZORDER) as UINT,
        );
    } else {
        /* set the size of the button that drops the calendar down */
        /* FIXME: account for style that allows button on left side */
        (*infoPtr).calbutton.top = (*infoPtr).rcDraw.top;
        (*infoPtr).calbutton.bottom = (*infoPtr).rcDraw.bottom;
        (*infoPtr).calbutton.left = ((*infoPtr).rcDraw.right - 15) as LONG;
        (*infoPtr).calbutton.right = (*infoPtr).rcDraw.right;
    }

    /* set enable/disable button size for show none style being enabled */
    /* FIXME: these dimensions are completely incorrect */
    (*infoPtr).checkbox.top = (*infoPtr).rcDraw.top;
    (*infoPtr).checkbox.bottom = (*infoPtr).rcDraw.bottom;
    (*infoPtr).checkbox.left = (*infoPtr).rcDraw.left;
    (*infoPtr).checkbox.right = ((*infoPtr).rcDraw.left + 10) as LONG;

    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null::<RECT>(),
        FALSE,
    );

    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_StyleChanging(
    mut infoPtr: *mut DATETIME_INFO,
    mut wStyleType: WPARAM,
    mut lpss: *mut STYLESTRUCT,
) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DATETIME_StyleChanging\0" as *const u8 as *const ::core::ffi::c_char,
            1391 as ::core::ffi::c_int,
            b"(styletype=%lx, styleOld=0x%08x, styleNew=0x%08x)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wStyleType,
            (*lpss).styleOld,
            (*lpss).styleNew,
        );
    }

    /* block DTS_SHOWNONE change */
    if (((*lpss).styleNew ^ (*lpss).styleOld) & DTS_SHOWNONE as DWORD) != 0 {
        if (*lpss).styleOld & DTS_SHOWNONE as DWORD != 0 {
            (*lpss).styleNew |= DTS_SHOWNONE as DWORD;
        } else {
            (*lpss).styleNew &= !(DTS_SHOWNONE as DWORD);
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn DATETIME_StyleChanged(mut infoPtr: *mut DATETIME_INFO, mut wStyleType: WPARAM, mut lpss: *const STYLESTRUCT) -> LRESULT {
    if wStyleType != ((GWL_STYLE) as u32) as _ {
        return 0;
    }
    (*infoPtr).dwStyle = (*lpss).styleNew;

    if !((*lpss).styleOld & DTS_SHOWNONE != 0) && ((*lpss).styleNew & DTS_SHOWNONE != 0) {
        (*infoPtr).hwndCheckbut = CreateWindowExW(
            0,
            (WC_BUTTONW).as_ptr(),
            core::ptr::null(),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
            2,
            2,
            13,
            13,
            (*infoPtr).hwndSelf,
            core::ptr::null_mut(),
            GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_HINSTANCE) as HINSTANCE,
            core::ptr::null_mut(),
        );
        SendMessageW(
            (*infoPtr).hwndCheckbut,
            BM_SETCHECK,
            if (*infoPtr).dateValid != 0 { 1 } else { 0 },
            0,
        );
    }
    if ((*lpss).styleOld & DTS_SHOWNONE != 0) && !((*lpss).styleNew & DTS_SHOWNONE != 0) {
        DestroyWindow((*infoPtr).hwndCheckbut);
        (*infoPtr).hwndCheckbut = core::ptr::null_mut();
    }
    if !((*lpss).styleOld & DTS_UPDOWN != 0) && ((*lpss).styleNew & DTS_UPDOWN != 0) {
        (*infoPtr).hUpdown = CreateUpDownControl(
            WS_CHILD | WS_BORDER | WS_VISIBLE,
            120,
            1,
            20,
            20,
            (*infoPtr).hwndSelf,
            1,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (UD_MAXVAL) as INT,
            UD_MINVAL,
            0,
        );
    }
    if ((*lpss).styleOld & DTS_UPDOWN != 0) && !((*lpss).styleNew & DTS_UPDOWN != 0) {
        DestroyWindow((*infoPtr).hUpdown);
        (*infoPtr).hUpdown = core::ptr::null_mut();
    }

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);
    return 0;
}

unsafe extern "C" fn DATETIME_GetIdealSize(infoPtr: *mut DATETIME_INFO, size: *mut SIZE) -> BOOL {
    let mut field_size: SIZE = SIZE { cx: 0, cy: 0 };
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut txt: [WCHAR; 80] = [0; 80];
    let mut hdc: HDC = core::ptr::null_mut();
    let mut oldFont: HFONT = core::ptr::null_mut();
    let mut i: i32;

    (*size).cy = 0 as LONG;
    (*size).cx = (*size).cy;

    hdc = GetDC((*infoPtr).hwndSelf);
    oldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    /* Get text font height */
    DATETIME_ReturnTxt(
        infoPtr,
        0,
        core::ptr::addr_of_mut!(txt) as LPWSTR,
        ARRAY_SIZE!(txt) as i32,
    );
    GetTextExtentPoint32W(
        hdc,
        core::ptr::addr_of_mut!(txt) as *mut WCHAR as LPCWSTR,
        lstrlenW(core::ptr::addr_of_mut!(txt) as *mut WCHAR as LPCWSTR),
        core::ptr::addr_of_mut!(field_size),
    );
    (*size).cy = field_size.cy;

    /* Get text font width */
    i = 0;
    while i < (*infoPtr).nrFields {
        (*size).cx += DATETIME_GetFieldWidth(infoPtr, hdc, i);
        i += 1;
    }

    SelectObject(hdc, oldFont as HGDIOBJ);
    ReleaseDC((*infoPtr).hwndSelf, hdc);

    if (*infoPtr).dwStyle & DTS_UPDOWN as DWORD != 0 {
        GetWindowRect((*infoPtr).hUpdown, core::ptr::addr_of_mut!(rect));
        (*size).cx += rect.right - rect.left;
    } else {
        (*size).cx += (*infoPtr).calbutton.right - (*infoPtr).calbutton.left;
    }

    if (*infoPtr).dwStyle & DTS_SHOWNONE as DWORD != 0 {
        (*size).cx += (*infoPtr).checkbox.right - (*infoPtr).checkbox.left;
    }

    /* Add space between controls for them not to get too close */
    (*size).cx += 12;
    (*size).cy += 4;

    return TRUE;
}

unsafe extern "C" fn DATETIME_SetFont(infoPtr: *mut DATETIME_INFO, font: HFONT, repaint: BOOL) -> LRESULT {
    (*infoPtr).hFont = font;
    if repaint != 0 { InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE); }
    return 0;
}

unsafe extern "C" fn DATETIME_Create(hwnd: HWND, lpcs: *const CREATESTRUCTW) -> LRESULT {
    let infoPtr: *mut DATETIME_INFO = Alloc(core::mem::size_of::<DATETIME_INFO>() as DWORD) as *mut DATETIME_INFO;
    let mut ss: STYLESTRUCT = STYLESTRUCT {
        styleOld: 0,
        styleNew: (*lpcs).style as DWORD,
    };

    if infoPtr.is_null() {
        return -1;
    }

    (*infoPtr).hwndSelf = hwnd;
    (*infoPtr).dwStyle = (*lpcs).style as DWORD;

    (*infoPtr).nrFieldsAllocated = 32;
    (*infoPtr).fieldspec = Alloc(
        ((*infoPtr).nrFieldsAllocated as usize)
            .wrapping_mul(core::mem::size_of::<::core::ffi::c_int>() as usize) as DWORD,
    ) as *mut ::core::ffi::c_int;
    (*infoPtr).fieldRect = Alloc(
        ((*infoPtr).nrFieldsAllocated as usize)
            .wrapping_mul(core::mem::size_of::<RECT>() as usize) as DWORD,
    ) as *mut RECT;
    (*infoPtr).buflen = Alloc(
        ((*infoPtr).nrFieldsAllocated as usize)
            .wrapping_mul(core::mem::size_of::<::core::ffi::c_int>() as usize) as DWORD,
    ) as *mut ::core::ffi::c_int;
    (*infoPtr).hwndNotify = (*lpcs).hwndParent;
    (*infoPtr).select = -1; /* initially, nothing is selected */
    (*infoPtr).bDropdownEnabled = TRUE;

    DATETIME_StyleChanged(infoPtr, (GWL_STYLE) as WPARAM, core::ptr::addr_of_mut!(ss));
    DATETIME_SetFormatW(infoPtr, core::ptr::null());

    /* create the monthcal control */
    (*infoPtr).hMonthCal = CreateWindowExW(
        0,
        (MONTHCAL_CLASSW).as_ptr(),
        core::ptr::null(),
        WS_BORDER | WS_POPUP | WS_CLIPSIBLINGS,
        0,
        0,
        0,
        0,
        (*infoPtr).hwndSelf,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    /* initialize info structure */
    GetLocalTime(core::ptr::addr_of_mut!((*infoPtr).date));
    (*infoPtr).dateValid = TRUE;
    (*infoPtr).hFont = GetStockObject((DEFAULT_GUI_FONT as i32)) as HFONT;

    SetWindowLongPtrW(hwnd, 0, infoPtr as DWORD_PTR as LONG_PTR);

    return 0;
}

unsafe extern "C" fn DATETIME_Destroy(infoPtr: *mut DATETIME_INFO) -> LRESULT {
    if !(*infoPtr).hwndCheckbut.is_null() {
        DestroyWindow((*infoPtr).hwndCheckbut);
    }
    if !(*infoPtr).hUpdown.is_null() {
        DestroyWindow((*infoPtr).hUpdown);
    }
    if !(*infoPtr).hMonthCal.is_null() {
        DestroyWindow((*infoPtr).hMonthCal);
    }
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);
    Free((*infoPtr).buflen as LPVOID);
    Free((*infoPtr).fieldRect as LPVOID);
    Free((*infoPtr).fieldspec as LPVOID);
    Free(infoPtr as LPVOID);
    return 0;
}

unsafe extern "C" fn DATETIME_GetText(infoPtr: *const DATETIME_INFO, count: INT, dst: LPWSTR) -> INT {
    let mut buf: [WCHAR; 80] = [0; 80];
    let mut i: INT;

    if dst.is_null() || count <= 0 {
        return 0;
    }

    *dst.offset(0 as isize) = 0;
    i = 0;
    while i < (*infoPtr).nrFields {
        DATETIME_ReturnTxt(infoPtr, i, buf.as_mut_ptr(), (ARRAY_SIZE!(buf)) as i32);
        if (lstrlenW(dst as LPCWSTR) + lstrlenW(buf.as_ptr())) < count {
            lstrcatW(dst, buf.as_ptr());
        } else {
            break;
        }
        i += 1;
    }
    return lstrlenW(dst as LPCWSTR);
}

unsafe extern "C" fn DATETIME_GetTextLength(info: *const DATETIME_INFO) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut length: ::core::ffi::c_int = 0;
    let mut buffer: [WCHAR; 80] = [0; 80];
    i = 0;
    while i < (*info).nrFields {
        DATETIME_ReturnTxt(
            info,
            i,
            core::ptr::addr_of_mut!(buffer) as LPWSTR,
            ARRAY_SIZE!(buffer) as ::core::ffi::c_int,
        );
        length += lstrlenW(core::ptr::addr_of_mut!(buffer) as LPCWSTR);
        i += 1;
    }
    return length;
}

unsafe extern "system" fn DATETIME_WindowProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut DATETIME_INFO =
        GetWindowLongPtrW(hwnd, 0) as *mut DATETIME_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/datetime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DATETIME_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            1593 as ::core::ffi::c_int,
            b"%x, %lx, %lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            uMsg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && uMsg != WM_CREATE && uMsg != WM_NCCREATE {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    if uMsg == DTM_GETSYSTEMTIME {
        return DATETIME_GetSystemTime(infoPtr, lParam as *mut SYSTEMTIME) as LRESULT;
    } else if uMsg == DTM_SETSYSTEMTIME {
        return DATETIME_SetSystemTime(infoPtr, wParam as DWORD, lParam as *mut SYSTEMTIME)
            as LRESULT;
    } else if uMsg == DTM_GETRANGE {
        return SendMessageW(
            (*infoPtr).hMonthCal,
            MCM_GETRANGE as UINT,
            wParam,
            lParam,
        );
    } else if uMsg == DTM_SETRANGE {
        return SendMessageW(
            (*infoPtr).hMonthCal,
            MCM_SETRANGE as UINT,
            wParam,
            lParam,
        );
    } else if uMsg == DTM_SETFORMATA {
        return DATETIME_SetFormatA(infoPtr, lParam as LPCSTR) as LRESULT;
    } else if uMsg == DTM_SETFORMATW {
        return DATETIME_SetFormatW(infoPtr, lParam as LPCWSTR) as LRESULT;
    } else if uMsg == DTM_GETMONTHCAL {
        return (*infoPtr).hMonthCal as LRESULT;
    } else if uMsg == DTM_SETMCCOLOR {
        return SendMessageW(
            (*infoPtr).hMonthCal,
            MCM_SETCOLOR as UINT,
            wParam,
            lParam,
        );
    } else if uMsg == DTM_GETMCCOLOR {
        return SendMessageW(
            (*infoPtr).hMonthCal,
            MCM_GETCOLOR as UINT,
            wParam,
            0 as LPARAM,
        );
    } else if uMsg == DTM_SETMCFONT {
        return SendMessageW(
            (*infoPtr).hMonthCal,
            WM_SETFONT as UINT,
            wParam,
            lParam,
        );
    } else if uMsg == DTM_GETMCFONT {
        return SendMessageW(
            (*infoPtr).hMonthCal,
            WM_GETFONT as UINT,
            wParam,
            lParam,
        );
    } else if uMsg == DTM_GETIDEALSIZE {
        return DATETIME_GetIdealSize(infoPtr, lParam as *mut SIZE) as LRESULT;
    } else if uMsg == WM_NOTIFY {
        return DATETIME_Notify(infoPtr, lParam as LPNMHDR as *const NMHDR);
    } else if uMsg == WM_ENABLE {
        return DATETIME_Enable(infoPtr, wParam as BOOL);
    } else if uMsg == WM_ERASEBKGND {
        return DATETIME_EraseBackground(infoPtr, wParam as HDC);
    } else if uMsg == WM_GETDLGCODE {
        return (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT;
    } else if uMsg == WM_PRINTCLIENT || uMsg == WM_PAINT {
        return DATETIME_Paint(infoPtr, wParam as HDC);
    } else if uMsg == WM_KEYDOWN {
        return DATETIME_KeyDown(infoPtr, wParam as DWORD);
    } else if uMsg == WM_CHAR {
        return DATETIME_Char(infoPtr, wParam);
    } else if uMsg == WM_KILLFOCUS {
        return DATETIME_KillFocus(infoPtr, wParam as HWND);
    } else if uMsg == WM_NCCREATE {
        return DATETIME_NCCreate(hwnd, lParam as LPCREATESTRUCTW as *const CREATESTRUCTW);
    } else if uMsg == WM_SETFOCUS {
        return DATETIME_SetFocus(infoPtr, wParam as HWND);
    } else if uMsg == WM_SIZE {
        return DATETIME_Size(
            infoPtr,
            LOWORD!(lParam) as SHORT as INT,
            HIWORD!(lParam) as SHORT as INT,
        );
    } else if uMsg == WM_LBUTTONDOWN {
        return DATETIME_LButtonDown(
            infoPtr,
            LOWORD!(lParam) as SHORT as INT,
            HIWORD!(lParam) as SHORT as INT,
        );
    } else if uMsg == WM_LBUTTONUP {
        return DATETIME_LButtonUp(infoPtr);
    } else if uMsg == WM_VSCROLL {
        return DATETIME_VScroll(infoPtr, wParam as WORD);
    } else if uMsg == WM_CREATE {
        return DATETIME_Create(hwnd, lParam as LPCREATESTRUCTW as *const CREATESTRUCTW);
    } else if uMsg == WM_DESTROY {
        return DATETIME_Destroy(infoPtr);
    } else if uMsg == WM_COMMAND {
        return DATETIME_Command(infoPtr, wParam, lParam);
    } else if uMsg == WM_STYLECHANGING {
        return DATETIME_StyleChanging(infoPtr, wParam, lParam as *mut STYLESTRUCT);
    } else if uMsg == WM_STYLECHANGED {
        return DATETIME_StyleChanged(
            infoPtr,
            wParam,
            lParam as LPSTYLESTRUCT as *const STYLESTRUCT,
        );
    } else if uMsg == WM_SETFONT {
        return DATETIME_SetFont(infoPtr, wParam as HFONT, lParam as BOOL);
    } else if uMsg == WM_GETFONT {
        return (*infoPtr).hFont as LRESULT;
    } else if uMsg == WM_GETTEXT {
        return DATETIME_GetText(infoPtr, wParam as INT, lParam as LPWSTR) as LRESULT;
    } else if uMsg == WM_GETTEXTLENGTH {
        return DATETIME_GetTextLength(infoPtr) as LRESULT;
    } else if uMsg == WM_SETTEXT {
        return CB_ERR as LRESULT;
    } else {
        if uMsg >= WM_USER as UINT
            && uMsg < WM_APP as UINT
            && COMCTL32_IsReflectedMessage(uMsg) == 0
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/datetime.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"DATETIME_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                    1712 as ::core::ffi::c_int,
                    b"unknown msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    uMsg,
                    wParam,
                    lParam,
                );
            }
        }
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DATETIME_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = CS_GLOBALCLASS as UINT;
    wndClass.lpfnWndProc =
        Some(DATETIME_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut DATETIME_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        0 as HINSTANCE,
        IDC_ARROW as LPCWSTR,
    );
    wndClass.hbrBackground =
        (COLOR_WINDOW + 1) as HBRUSH;
    wndClass.lpszClassName = DATETIMEPICK_CLASSW.as_ptr() as LPCWSTR;
    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn DATETIME_Unregister() {
    UnregisterClassW(
        DATETIMEPICK_CLASSW.as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
    );
}
