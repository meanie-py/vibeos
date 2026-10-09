use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_monthcal);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const UPDOWN_CLASSW: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"m\0s\0c\0t\0l\0s\0_\0u\0p\0d\0o\0w\0n\x003\x002\0\0\0",
    )
};

pub const MONTHCAL_CLASSW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"S\0y\0s\0M\0o\0n\0t\0h\0C\0a\0l\x003\x002\0\0\0",
    )
};

pub const WC_EDITW: [::core::ffi::c_ushort; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_ushort; 5]>(*b"E\0d\0i\0t\0\0\0") };

static mut themeClass: [WCHAR; 10] = [
    'S' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut st_null: SYSTEMTIME = _SYSTEMTIME {
    wYear: 0,
    wMonth: 0,
    wDayOfWeek: 0,
    wDay: 0,
    wHour: 0,
    wMinute: 0,
    wSecond: 0,
    wMilliseconds: 0,
};

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

// WINE_DEFAULT_DEBUG_CHANNEL(monthcal) is a macro that expands to a debug-channel static; it is omitted here.
// DAYSTO100NSECS is a function-like macro and is skipped.
pub const MCS_NOSELCHANGEONNAV: u32 = 0x0100;
pub const MC_SEL_LBUTUP: i32 = 1;
pub const MC_SEL_LBUTDOWN: i32 = 2;
pub const MC_PREVPRESSED: i32 = 4;
pub const MC_NEXTPRESSED: i32 = 8;
pub const MC_PREVNEXTMONTHDELAY: i32 = 350;
pub const MC_TODAYUPDATEDELAY: i32 = 120000;
pub const MC_PREVNEXTMONTHTIMER: i32 = 1;
pub const MC_TODAYUPDATETIMER: i32 = 2;
pub const MC_CALENDAR_PADDING: i32 = 6;

pub const PenRed: i32 = 0;
pub const PenText: i32 = 1;
pub const PenLast: i32 = 2;

pub const BrushTitle: i32 = 0;
pub const BrushMonth: i32 = 1;
pub const BrushBackground: i32 = 2;
pub const BrushLast: i32 = 3;

// Value from the Win32 MCSC_* enum (MCSC_TRAILINGTEXT is the last entry).
pub const MCSC_TRAILINGTEXT: usize = 5;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CALENDAR_INFO {
    pub title: RECT,
    pub titlemonth: RECT,
    pub titleyear: RECT,
    pub wdays: RECT,
    pub days: RECT,
    pub weeknums: RECT,
    pub month: SYSTEMTIME,
}

#[repr(C)]
pub struct MONTHCAL_INFO {
    pub hwndSelf: HWND,
    pub dwStyle: DWORD,
    pub colors: [COLORREF; MCSC_TRAILINGTEXT + 1],
    pub brushes: [HBRUSH; BrushLast as usize],
    pub pens: [HPEN; PenLast as usize],
    pub hFont: HFONT,
    pub hBoldFont: HFONT,
    pub textHeight: i32,
    pub height_increment: i32,
    pub width_increment: i32,
    pub delta: i32,
    pub firstDay: i32,
    pub firstDaySet: i32,
    pub isUnicode: i32,
    pub monthdayState: *mut MONTHDAYSTATE,
    pub todaysDate: SYSTEMTIME,
    pub todaySet: i32,
    pub status: i32,
    pub firstSel: SYSTEMTIME,
    pub maxSelCount: i32,
    pub minSel: SYSTEMTIME,
    pub maxSel: SYSTEMTIME,
    pub focusedSel: SYSTEMTIME,
    pub rangeValid: DWORD,
    pub minDate: SYSTEMTIME,
    pub maxDate: SYSTEMTIME,
    pub titlebtnnext: RECT,
    pub titlebtnprev: RECT,
    pub todayrect: RECT,
    pub hwndNotify: HWND,
    pub hWndYearEdit: HWND,
    pub hWndYearUpDown: HWND,
    pub EditWndProc: WNDPROC,
    pub calendars: *mut CALENDAR_INFO,
    pub dim: SIZE,
}
pub type LPMONTHCAL_INFO = *mut MONTHCAL_INFO;

pub const DIRECTION_BACKWARD: i32 = 0;
pub const DIRECTION_FORWARD: i32 = 1;

unsafe extern "C" fn MONTHCAL_GetCalCount(infoPtr: *const MONTHCAL_INFO) -> INT {
    return (*infoPtr).dim.cx * (*infoPtr).dim.cy;
}

unsafe extern "C" fn MONTHCAL_NotifySelectionChange(infoPtr: *const MONTHCAL_INFO) {
    let mut nmsc: NMSELCHANGE = core::mem::zeroed();

    nmsc.nmhdr.hwndFrom = (*infoPtr).hwndSelf;
    nmsc.nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID as i32) as UINT_PTR;
    nmsc.nmhdr.code = MCN_SELCHANGE as UINT;
    nmsc.stSelStart = (*infoPtr).minSel;
    nmsc.stSelStart.wDayOfWeek = 0 as WORD;
    if ((*infoPtr).dwStyle & (MCS_MULTISELECT as DWORD)) != 0 {
        nmsc.stSelEnd = (*infoPtr).maxSel;
        nmsc.stSelEnd.wDayOfWeek = 0 as WORD;
    } else {
        nmsc.stSelEnd = st_null;
    }

    SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nmsc.nmhdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmsc) as LPARAM,
    );
}

unsafe extern "C" fn MONTHCAL_NotifySelect(infoPtr: *const MONTHCAL_INFO) {
    let mut nmsc: NMSELCHANGE = tagNMSELCHANGE {
        nmhdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        stSelStart: _SYSTEMTIME {
            wYear: 0,
            wMonth: 0,
            wDayOfWeek: 0,
            wDay: 0,
            wHour: 0,
            wMinute: 0,
            wSecond: 0,
            wMilliseconds: 0,
        },
        stSelEnd: _SYSTEMTIME {
            wYear: 0,
            wMonth: 0,
            wDayOfWeek: 0,
            wDay: 0,
            wHour: 0,
            wMinute: 0,
            wSecond: 0,
            wMilliseconds: 0,
        },
    };
    nmsc.nmhdr.hwndFrom = (*infoPtr).hwndSelf;
    nmsc.nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID as i32) as UINT_PTR;
    nmsc.nmhdr.code = MCN_SELECT as UINT;
    nmsc.stSelStart = (*infoPtr).minSel;
    nmsc.stSelStart.wDayOfWeek = 0 as WORD;
    if ((*infoPtr).dwStyle & (MCS_MULTISELECT as DWORD)) != 0 {
        nmsc.stSelEnd = (*infoPtr).maxSel;
        nmsc.stSelEnd.wDayOfWeek = 0 as WORD;
    } else {
        nmsc.stSelEnd = st_null;
    }
    SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nmsc.nmhdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmsc) as LPARAM,
    );
}

unsafe extern "C" fn MONTHCAL_MonthDiff(left: *const SYSTEMTIME, right: *const SYSTEMTIME) -> ::core::ffi::c_int {
    return ((*right).wYear as ::core::ffi::c_int - (*left).wYear as ::core::ffi::c_int) * 12 + (*right).wMonth as ::core::ffi::c_int - (*left).wMonth as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn MONTHCAL_MonthLength(mut month: ::core::ffi::c_int, mut year: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static mdays: [::core::ffi::c_int; 12] = [31 as ::core::ffi::c_int, 28 as ::core::ffi::c_int, 31 as ::core::ffi::c_int, 30 as ::core::ffi::c_int, 31 as ::core::ffi::c_int, 30 as ::core::ffi::c_int, 31 as ::core::ffi::c_int, 31 as ::core::ffi::c_int, 30 as ::core::ffi::c_int, 31 as ::core::ffi::c_int, 30 as ::core::ffi::c_int, 31 as ::core::ffi::c_int];
    if month == 0 as ::core::ffi::c_int {
        month = 12 as ::core::ffi::c_int;
    } else if month == 13 as ::core::ffi::c_int {
        month = 1 as ::core::ffi::c_int;
    }
    if month == min_allowed_date.wMonth as ::core::ffi::c_int && year == min_allowed_date.wYear as ::core::ffi::c_int {
        return 19 as ::core::ffi::c_int;
    }
    if month == 2 as ::core::ffi::c_int {
        return mdays[(month - 1 as ::core::ffi::c_int) as usize]
            + (if year % 400 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else if (year % 100 as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
                && (year % 4 as ::core::ffi::c_int == 0 as ::core::ffi::c_int)
            {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
    } else {
        return mdays[(month - 1 as ::core::ffi::c_int) as usize];
    }
}

unsafe extern "C" fn MONTHCAL_IsDateEqual(first: *const SYSTEMTIME, second: *const SYSTEMTIME) -> BOOL {
    return ((*first).wYear as i32 == (*second).wYear as i32
        && (*first).wMonth as i32 == (*second).wMonth as i32
        && (*first).wDay as i32 == (*second).wDay as i32) as i32;
}

unsafe extern "C" fn MONTHCAL_ValidateDate(time: *const SYSTEMTIME) -> BOOL {
    if (((*time).wMonth as i32) < 1) || ((*time).wMonth as i32 > 12) {
        return FALSE;
    }
    if ((*time).wDay as i32 == 0) || ((*time).wDay as i32 > MONTHCAL_MonthLength((*time).wMonth as i32, (*time).wYear as i32)) {
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn MONTHCAL_CopyTime(from: *const SYSTEMTIME, to: *mut SYSTEMTIME) {
    (*to).wHour = (*from).wHour;
    (*to).wMinute = (*from).wMinute;
    (*to).wSecond = (*from).wSecond;
}

unsafe extern "C" fn MONTHCAL_CopyDate(from: *const SYSTEMTIME, to: *mut SYSTEMTIME) {
    (*to).wYear = (*from).wYear;
    (*to).wMonth = (*from).wMonth;
    (*to).wDay = (*from).wDay;
    (*to).wDayOfWeek = (*from).wDayOfWeek;
}
#[no_mangle]
pub unsafe extern "C" fn MONTHCAL_CompareSystemTime(first: *const SYSTEMTIME, second: *const SYSTEMTIME) -> LONG {
    let mut ft_first: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut ft_second: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

    SystemTimeToFileTime(first, core::ptr::addr_of_mut!(ft_first));
    SystemTimeToFileTime(second, core::ptr::addr_of_mut!(ft_second));

    return CompareFileTime(core::ptr::addr_of_mut!(ft_first), core::ptr::addr_of_mut!(ft_second));
}

unsafe extern "C" fn MONTHCAL_CompareMonths(first: *const SYSTEMTIME, second: *const SYSTEMTIME) -> LONG {
    let mut st_first: SYSTEMTIME = st_null;
    let mut st_second: SYSTEMTIME = st_null;
    st_second = st_null;
    st_first = st_second;
    MONTHCAL_CopyDate(first, core::ptr::addr_of_mut!(st_first));
    MONTHCAL_CopyDate(second, core::ptr::addr_of_mut!(st_second));
    st_second.wDay = 1 as WORD;
    st_first.wDay = st_second.wDay;

    return MONTHCAL_CompareSystemTime(core::ptr::addr_of_mut!(st_first), core::ptr::addr_of_mut!(st_second));
}

unsafe extern "C" fn MONTHCAL_CompareDate(first: *const SYSTEMTIME, second: *const SYSTEMTIME) -> LONG {
    let mut st_first: SYSTEMTIME = core::mem::zeroed();
    let mut st_second: SYSTEMTIME = core::mem::zeroed();

    st_first = st_null;
    st_second = st_null;
    MONTHCAL_CopyDate(first, core::ptr::addr_of_mut!(st_first));
    MONTHCAL_CopyDate(second, core::ptr::addr_of_mut!(st_second));

    return MONTHCAL_CompareSystemTime(core::ptr::addr_of_mut!(st_first), core::ptr::addr_of_mut!(st_second));
}

unsafe extern "C" fn MONTHCAL_IsDateInValidRange(infoPtr: *const MONTHCAL_INFO, date: *mut SYSTEMTIME, fix: BOOL) -> BOOL {
  let mut fix_st: *const SYSTEMTIME = core::ptr::null();

  if MONTHCAL_CompareSystemTime(date, core::ptr::addr_of!(max_allowed_date)) == 1 {
     fix_st = core::ptr::addr_of!(max_allowed_date);
  }
  else if MONTHCAL_CompareSystemTime(date, core::ptr::addr_of!(min_allowed_date)) == -1 {
     fix_st = core::ptr::addr_of!(min_allowed_date);
  }
  else {
     if (*infoPtr).rangeValid & GDTR_MAX != 0 {
        if MONTHCAL_CompareSystemTime(date, core::ptr::addr_of!((*infoPtr).maxDate)) == 1 {
           fix_st = core::ptr::addr_of!((*infoPtr).maxDate);
        }
     }

     if (*infoPtr).rangeValid & GDTR_MIN != 0 {
        if MONTHCAL_CompareSystemTime(date, core::ptr::addr_of!((*infoPtr).minDate)) == -1 {
           fix_st = core::ptr::addr_of!((*infoPtr).minDate);
        }
     }
  }

  if fix != 0 && !fix_st.is_null() {
    (*date).wYear = (*fix_st).wYear;
    (*date).wMonth = (*fix_st).wMonth;
  }

  return fix_st.is_null() as BOOL;
}

unsafe extern "C" fn MONTHCAL_IsSelRangeValid(
    infoPtr: *const MONTHCAL_INFO,
    range0: *const SYSTEMTIME,
    range1: *const SYSTEMTIME,
    adjust: *mut SYSTEMTIME,
) -> BOOL {
    let mut ul_range0: ULARGE_INTEGER = core::mem::zeroed();
    let mut ul_range1: ULARGE_INTEGER = core::mem::zeroed();
    let mut ul_diff: ULARGE_INTEGER = core::mem::zeroed();
    let mut ft_range0: FILETIME = core::mem::zeroed();
    let mut ft_range1: FILETIME = core::mem::zeroed();
    let mut cmp: LONG = 0;

    SystemTimeToFileTime(range0, core::ptr::addr_of_mut!(ft_range0));
    SystemTimeToFileTime(range1, core::ptr::addr_of_mut!(ft_range1));

    ul_range0.u.LowPart = ft_range0.dwLowDateTime;
    ul_range0.u.HighPart = ft_range0.dwHighDateTime;
    ul_range1.u.LowPart = ft_range1.dwLowDateTime;
    ul_range1.u.HighPart = ft_range1.dwHighDateTime;

    cmp = CompareFileTime(
        core::ptr::addr_of_mut!(ft_range0),
        core::ptr::addr_of_mut!(ft_range1),
    );

    if cmp == 1 {
        ul_diff.QuadPart = ul_range0.QuadPart.wrapping_sub(ul_range1.QuadPart);
    } else {
        ul_diff.QuadPart = ul_range0
            .QuadPart
            .wrapping_neg()
            .wrapping_add(ul_range1.QuadPart);
    }

    if ul_diff.QuadPart >= DAYSTO100NSECS!((*infoPtr).maxSelCount as ULONGLONG) {
        if !adjust.is_null() {
            if cmp == 1 {
                ul_range0.QuadPart = ul_range1.QuadPart.wrapping_add(DAYSTO100NSECS!(
                    ((*infoPtr).maxSelCount - 1) as ULONGLONG,
                ));
            } else {
                ul_range0.QuadPart = ul_range1.QuadPart.wrapping_sub(DAYSTO100NSECS!(
                    ((*infoPtr).maxSelCount - 1) as ULONGLONG,
                ));
            }

            ft_range0.dwLowDateTime = ul_range0.u.LowPart;
            ft_range0.dwHighDateTime = ul_range0.u.HighPart;
            FileTimeToSystemTime(core::ptr::addr_of_mut!(ft_range0), adjust);
        }

        return FALSE;
    } else {
        return TRUE;
    }
}

unsafe extern "C" fn MONTHCAL_ValidateTime(time: *const SYSTEMTIME) -> BOOL {
    if ((*time).wHour as ::core::ffi::c_int > 24)
        || ((*time).wMinute as ::core::ffi::c_int > 59)
        || ((*time).wSecond as ::core::ffi::c_int > 59)
    {
        return FALSE;
    } else {
        return TRUE;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MONTHCAL_CalculateDayOfWeek(date: *mut SYSTEMTIME, inplace: BOOL) -> i32 {
    let mut st: SYSTEMTIME = st_null;
    let mut ft: FILETIME = core::mem::zeroed();

    MONTHCAL_CopyDate(date, core::ptr::addr_of_mut!(st));

    SystemTimeToFileTime(core::ptr::addr_of_mut!(st), core::ptr::addr_of_mut!(ft));
    FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(st));

    if inplace != 0 {
        (*date).wDayOfWeek = st.wDayOfWeek;
    }

    return st.wDayOfWeek as i32;
}

unsafe extern "C" fn MONTHCAL_GetMonth(date: *mut SYSTEMTIME, months: INT) {
    let mut length: INT = 0;
    let mut m: INT = (*date).wMonth as INT + months;
    (*date).wYear = ((*date).wYear as INT + if m > 0 {
        (m - 1) / 12
    } else {
        m / 12 - 1
    }) as WORD;
    (*date).wMonth = (if m > 0 {
        (m - 1) % 12 + 1
    } else {
        12 + m % 12
    }) as WORD;
    /* fix moving from last day in a month */
    length = MONTHCAL_MonthLength((*date).wMonth as INT, (*date).wYear as INT);
    if ((*date).wDay as INT) > length {
        (*date).wDay = length as WORD;
    }
    MONTHCAL_CalculateDayOfWeek(date, TRUE);
}

unsafe extern "C" fn MONTHCAL_GetNextMonth(date: *mut SYSTEMTIME) {
    MONTHCAL_GetMonth(date, 1);
}

unsafe extern "C" fn MONTHCAL_GetPrevMonth(date: *mut SYSTEMTIME) {
    MONTHCAL_GetMonth(date, -1);
}

unsafe extern "C" fn MONTHCAL_GetMinDate(infoPtr: *const MONTHCAL_INFO, date: *mut SYSTEMTIME) {
    /* zero indexed calendar has the earliest date */
    let mut st_first: SYSTEMTIME = (*(*infoPtr).calendars.offset(0 as isize)).month;
    let mut firstDay: INT = 0;

    st_first.wDay = 1 as WORD;
    firstDay = MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!(st_first), FALSE) as INT;

    *date = (*(*infoPtr).calendars.offset(0 as isize)).month;
    MONTHCAL_GetPrevMonth(date);

    (*date).wDay = (MONTHCAL_MonthLength((*date).wMonth as INT, (*date).wYear as INT)
        + ((*infoPtr).firstDay - firstDay) % 7
        + 1) as WORD;

    if ((*date).wDay as INT) > MONTHCAL_MonthLength((*date).wMonth as INT, (*date).wYear as INT) {
        (*date).wDay = ((*date).wDay as INT - 7) as WORD;
    }

    /* fix day of week */
    MONTHCAL_CalculateDayOfWeek(date, TRUE);
}

unsafe extern "C" fn MONTHCAL_GetMaxDate(infoPtr: *const MONTHCAL_INFO, date: *mut SYSTEMTIME) {
    /* the latest date is in latest calendar */
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let lt_month: *mut SYSTEMTIME = core::ptr::addr_of_mut!(
        (*(*infoPtr).calendars.offset(
            (MONTHCAL_GetCalCount(infoPtr) - 1) as isize,
        ))
        .month
    );
    let mut first_day: INT = 0;

    *date = *lt_month;
    st = *lt_month;

    /* day of week of first day of current month */
    st.wDay = 1 as WORD;
    first_day = MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!(st), FALSE);

    MONTHCAL_GetNextMonth(date);
    MONTHCAL_GetPrevMonth(core::ptr::addr_of_mut!(st));

    /* last calendar starts with some date from previous month that not displayed */
    st.wDay = (MONTHCAL_MonthLength(st.wMonth as INT, st.wYear as INT)
        + ((*infoPtr).firstDay - first_day) % 7
        + 1) as WORD;
    if (st.wDay as INT) > MONTHCAL_MonthLength(st.wMonth as INT, st.wYear as INT) {
        st.wDay = (st.wDay as INT - 7) as WORD;
    }

    /* Use month length to get max day. 42 means max day count in calendar area */
    (*date).wDay = (42
        - (MONTHCAL_MonthLength(st.wMonth as INT, st.wYear as INT) - st.wDay as INT
            + 1)
        - MONTHCAL_MonthLength((*lt_month).wMonth as INT, (*lt_month).wYear as INT))
        as WORD;

    /* fix day of week */
    MONTHCAL_CalculateDayOfWeek(date, TRUE);
}

unsafe extern "C" fn MONTHCAL_GetDayFromPos(infoPtr: *const MONTHCAL_INFO, mut pt: POINT, calIdx: INT) -> ::core::ffi::c_int {
    let mut st: SYSTEMTIME = (*(*infoPtr).calendars.offset(calIdx as isize)).month;
    let mut firstDay: ::core::ffi::c_int = 0;
    let mut col: ::core::ffi::c_int = 0;
    let mut row: ::core::ffi::c_int = 0;
    let mut client: RECT = core::mem::zeroed();

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(client));

    /* if the point is outside the x bounds of the window put it at the boundary */
    if pt.x > client.right {
        pt.x = client.right;
    }

    col = (pt.x as ::core::ffi::c_int
        - (*(*infoPtr).calendars.offset(calIdx as isize)).days.left as ::core::ffi::c_int)
        / (*infoPtr).width_increment;
    row = (pt.y as ::core::ffi::c_int
        - (*(*infoPtr).calendars.offset(calIdx as isize)).days.top as ::core::ffi::c_int)
        / (*infoPtr).height_increment;

    st.wDay = 1 as WORD;
    firstDay = (MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!(st), FALSE as BOOL) + 6 as ::core::ffi::c_int - (*infoPtr).firstDay) % 7 as ::core::ffi::c_int;
    return col + 7 as ::core::ffi::c_int * row - firstDay;
}

unsafe extern "C" fn MONTHCAL_GetDayPos(
    infoPtr: *const MONTHCAL_INFO,
    date: *const SYSTEMTIME,
    col: *mut INT,
    row: *mut INT,
    calIdx: INT,
) {
    let mut st: SYSTEMTIME = (*(*infoPtr).calendars.offset(calIdx as isize)).month;
    let mut first: INT = 0;

    st.wDay = 1;
    first = (MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!(st), FALSE) + 6 - (*infoPtr).firstDay) % 7;

    if calIdx == 0 || calIdx == MONTHCAL_GetCalCount(infoPtr) - 1 {
        let cal: *const SYSTEMTIME = core::ptr::addr_of!((*(*infoPtr).calendars.offset(calIdx as isize)).month);
        let cmp: LONG = MONTHCAL_CompareMonths(date, core::ptr::addr_of!(st));

        /* previous month */
        if cmp == -(1) {
            *col = (first - MONTHCAL_MonthLength((*date).wMonth as INT, (*cal).wYear as INT) + (*date).wDay as INT) % 7;
            *row = 0;
            return;
        }

        /* next month calculation is same as for current, just add current month length */
        if cmp == 1 {
            first += MONTHCAL_MonthLength((*cal).wMonth as INT, (*cal).wYear as INT);
        }
    }

    *col = ((*date).wDay as INT + first) % 7;
    *row = ((*date).wDay as INT + first - *col) / 7;
}

unsafe extern "C" fn MONTHCAL_GetDayRectI(
    infoPtr: *const MONTHCAL_INFO,
    r: *mut RECT,
    col: INT,
    row: INT,
    calIdx: INT,
) {
    (*r).left = ((*(*infoPtr).calendars.offset(calIdx as isize)).days.left as i32
        + col * (*infoPtr).width_increment) as LONG;
    (*r).right = ((*r).left as i32 + (*infoPtr).width_increment) as LONG;
    (*r).top = ((*(*infoPtr).calendars.offset(calIdx as isize)).days.top as i32
        + row * (*infoPtr).height_increment) as LONG;
    (*r).bottom = ((*r).top as i32 + (*infoPtr).textHeight) as LONG;
}

unsafe extern "C" fn MONTHCAL_GetDayRect(
    infoPtr: *const MONTHCAL_INFO,
    date: *const SYSTEMTIME,
    r: *mut RECT,
    mut calIdx: INT,
) -> BOOL {
    let mut col: INT = 0;
    let mut row: INT = 0;

    if MONTHCAL_ValidateDate(date) == 0 {
        SetRectEmpty(r as LPRECT);
        return FALSE;
    }

    if calIdx == -1 {
        let mut cmp: INT = MONTHCAL_CompareMonths(
            date,
            &raw mut (*(*infoPtr).calendars.offset(0 as isize)).month,
        ) as INT;

        if cmp <= 0 {
            calIdx = 0;
        } else {
            cmp = MONTHCAL_CompareMonths(
                date,
                &raw mut (*(*infoPtr).calendars.offset(
                    (MONTHCAL_GetCalCount(infoPtr) - 1) as isize,
                ))
                .month,
            ) as INT;
            if cmp >= 0 {
                calIdx = MONTHCAL_GetCalCount(infoPtr) - 1;
            } else {
                calIdx = 1;
                while calIdx < MONTHCAL_GetCalCount(infoPtr) - 1 {
                    if MONTHCAL_CompareMonths(
                        date,
                        &raw mut (*(*infoPtr).calendars.offset(calIdx as isize)).month,
                    ) == 0
                    {
                        break;
                    }
                    calIdx += 1;
                }
            }
        }
    }

    MONTHCAL_GetDayPos(infoPtr, date, &raw mut col, &raw mut row, calIdx);
    MONTHCAL_GetDayRectI(infoPtr, r, col, row, calIdx);

    return TRUE;
}

unsafe extern "C" fn MONTHCAL_GetMonthRange(
    infoPtr: *const MONTHCAL_INFO,
    flag: DWORD,
    st: *mut SYSTEMTIME,
) -> LRESULT {
    let mut range: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_GetMonthRange\0" as *const u8 as *const ::core::ffi::c_char,
            677 as ::core::ffi::c_int,
            b"flag=%d, st=%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            flag,
            st,
        );
    }
    match flag {
        GMR_VISIBLE => {
            if !st.is_null() {
                *st.offset(0 as isize) = (*(*infoPtr)
                    .calendars
                    .offset(0 as isize))
                .month;
                *st.offset(1 as isize) = (*(*infoPtr).calendars.offset(
                    (MONTHCAL_GetCalCount(infoPtr) as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int) as isize,
                ))
                .month;
                if (*st.offset(0 as isize)).wMonth as ::core::ffi::c_int
                    == min_allowed_date.wMonth as ::core::ffi::c_int
                    && (*st.offset(0 as isize)).wYear as ::core::ffi::c_int
                        == min_allowed_date.wYear as ::core::ffi::c_int
                {
                    (*st.offset(0 as isize)).wDay = min_allowed_date.wDay;
                } else {
                    (*st.offset(0 as isize)).wDay = 1 as WORD;
                }
                MONTHCAL_CalculateDayOfWeek(
                    st.offset(0 as isize) as *mut SYSTEMTIME,
                    TRUE,
                );
                (*st.offset(1 as isize)).wDay = MONTHCAL_MonthLength(
                    (*st.offset(1 as isize)).wMonth as ::core::ffi::c_int,
                    (*st.offset(1 as isize)).wYear as ::core::ffi::c_int,
                ) as WORD;
                MONTHCAL_CalculateDayOfWeek(
                    st.offset(1 as isize) as *mut SYSTEMTIME,
                    TRUE,
                );
            }
            range = MONTHCAL_GetCalCount(infoPtr);
        }
        GMR_DAYSTATE => {
            if !st.is_null() {
                MONTHCAL_GetMinDate(
                    infoPtr,
                    st.offset(0 as isize) as *mut SYSTEMTIME,
                );
                MONTHCAL_GetMaxDate(
                    infoPtr,
                    st.offset(1 as isize) as *mut SYSTEMTIME,
                );
            }
            range = (MONTHCAL_GetCalCount(infoPtr) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as INT;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/monthcal.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"MONTHCAL_GetMonthRange\0" as *const u8
                        as *const ::core::ffi::c_char,
                    715 as ::core::ffi::c_int,
                    b"Unknown flag value, got %d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    flag,
                );
            }
            range = 0 as INT;
        }
    }
    return range as LRESULT;
}

unsafe extern "C" fn MONTHCAL_SetDayFocus(infoPtr: *mut MONTHCAL_INFO, st: *const SYSTEMTIME) -> BOOL {
    let mut r: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if !st.is_null() {
        /* there's nothing to do if it's the same date,
           mouse move within same date rectangle case */
        if MONTHCAL_IsDateEqual(&raw mut (*infoPtr).focusedSel, st) != 0 {
            return FALSE;
        }

        /* invalidate old focused day */
        if MONTHCAL_GetDayRect(infoPtr, &raw mut (*infoPtr).focusedSel, &raw mut r, -1) != 0 {
            InvalidateRect((*infoPtr).hwndSelf, &raw mut r, FALSE);
        }

        (*infoPtr).focusedSel = *st;
    }

    /* On set invalidates new day, on reset clears previous focused day. */
    if MONTHCAL_GetDayRect(infoPtr, &raw mut (*infoPtr).focusedSel, &raw mut r, -1) != 0 {
        InvalidateRect((*infoPtr).hwndSelf, &raw mut r, FALSE);
    }

    if st.is_null() && MONTHCAL_ValidateDate(&raw mut (*infoPtr).focusedSel) != 0 {
        (*infoPtr).focusedSel = st_null;
    }

    return TRUE;
}

unsafe extern "C" fn MONTHCAL_Circle(infoPtr: *const MONTHCAL_INFO, hdc: HDC, r: *const RECT) {
    let old_pen: HPEN = SelectObject(hdc, (*infoPtr).pens[PenRed as usize] as HGDIOBJ) as HPEN;
    let mut old_brush: HBRUSH;

    old_brush = SelectObject(hdc, GetStockObject((NULL_BRUSH as i32)) as HGDIOBJ) as HBRUSH;
    Rectangle(hdc, (*r).left, (*r).top, (*r).right, (*r).bottom);

    SelectObject(hdc, old_brush as HGDIOBJ);
    SelectObject(hdc, old_pen as HGDIOBJ);
}

unsafe extern "C" fn MONTHCAL_CircleDay(infoPtr: *const MONTHCAL_INFO, hdc: HDC, date: *const SYSTEMTIME) {
    let mut r: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    MONTHCAL_GetDayRect(infoPtr, date, core::ptr::addr_of_mut!(r), -1);
    MONTHCAL_Circle(infoPtr, hdc, core::ptr::addr_of!(r));
}

unsafe extern "C" fn MONTHCAL_DrawDay(infoPtr: *const MONTHCAL_INFO, hdc: HDC, st: *const SYSTEMTIME, bold: i32, ps: *const PAINTSTRUCT) {
    static fmtW: [WCHAR; 3] = [0x0025u16, 0x0064, 0x0000];
    let mut buf: [WCHAR; 10] = [0; 10];
    let mut r: RECT = core::mem::zeroed();
    let mut r_temp: RECT = core::mem::zeroed();
    let mut oldCol: COLORREF = 0;
    let mut oldBk: COLORREF = 0;
    let mut old_bkmode: INT = 0;
    let mut selection: INT = 0;

    MONTHCAL_GetDayRect(infoPtr, st, core::ptr::addr_of_mut!(r), -1);
    if IntersectRect(core::ptr::addr_of_mut!(r_temp), core::ptr::addr_of!((*ps).rcPaint), core::ptr::addr_of_mut!(r)) == 0 {
        return;
    }

    if (MONTHCAL_CompareDate(st, core::ptr::addr_of!((*infoPtr).minSel)) >= 0) &&
        (MONTHCAL_CompareDate(st, core::ptr::addr_of!((*infoPtr).maxSel)) <= 0)
    {
        oldCol = SetTextColor(hdc, (*infoPtr).colors[MCSC_MONTHBK as usize]);
        oldBk = SetBkColor(hdc, (*infoPtr).colors[MCSC_TRAILINGTEXT as usize]);
        FillRect(hdc, core::ptr::addr_of_mut!(r), (*infoPtr).brushes[BrushTitle as usize]);

        selection = 1;
    }
    else {
        selection = 0;
    }

    SelectObject(
        hdc,
        if bold != 0 { ((*infoPtr).hBoldFont).cast() } else { ((*infoPtr).hFont).cast() },
    );

    old_bkmode = SetBkMode(hdc, (TRANSPARENT as i32));
    wsprintfW(core::ptr::addr_of_mut!(buf) as *mut WCHAR, core::ptr::addr_of!(fmtW) as *const WCHAR, (*st).wDay as i32);
    DrawTextW(
        hdc,
        core::ptr::addr_of_mut!(buf) as *mut WCHAR,
        -1,
        core::ptr::addr_of_mut!(r),
        (DT_CENTER | DT_VCENTER | DT_SINGLELINE) as UINT,
    );
    SetBkMode(hdc, old_bkmode);

    if selection != 0 {
        SetTextColor(hdc, oldCol);
        SetBkColor(hdc, oldBk);
    }
}

unsafe extern "C" fn MONTHCAL_PaintButton(
    mut infoPtr: *mut MONTHCAL_INFO,
    mut hdc: HDC,
    mut button: nav_direction,
) {
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    let mut r: *mut RECT = if button as ::core::ffi::c_uint
        == DIRECTION_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        &raw mut (*infoPtr).titlebtnnext
    } else {
        &raw mut (*infoPtr).titlebtnprev
    };
    let mut pressed: BOOL = if button as ::core::ffi::c_uint
        == DIRECTION_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*infoPtr).status as BOOL & MC_NEXTPRESSED
    } else {
        (*infoPtr).status as BOOL & MC_PREVPRESSED
    };
    if !theme.is_null() {
        static mut states: [::core::ffi::c_int; 6] = [
            ABS_LEFTNORMAL as ::core::ffi::c_int,
            ABS_LEFTPRESSED as ::core::ffi::c_int,
            ABS_LEFTDISABLED as ::core::ffi::c_int,
            ABS_RIGHTNORMAL as ::core::ffi::c_int,
            ABS_RIGHTPRESSED as ::core::ffi::c_int,
            ABS_RIGHTDISABLED as ::core::ffi::c_int,
        ];
        let mut stateNum: ::core::ffi::c_int = if button as ::core::ffi::c_uint
            == DIRECTION_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            3 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        if pressed != 0 {
            stateNum += 1 as ::core::ffi::c_int;
        } else if (*infoPtr).dwStyle & WS_DISABLED as DWORD != 0 {
            stateNum += 2 as ::core::ffi::c_int;
        }
        DrawThemeBackground(
            theme,
            hdc,
            SBP_ARROWBTN as ::core::ffi::c_int,
            states[stateNum as usize],
            r,
            ::core::ptr::null::<RECT>(),
        );
    } else {
        let mut style: ::core::ffi::c_int = if button as ::core::ffi::c_uint
            == DIRECTION_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            DFCS_SCROLLRIGHT as ::core::ffi::c_int
        } else {
            DFCS_SCROLLLEFT as ::core::ffi::c_int
        };
        if pressed != 0 {
            style |= DFCS_PUSHED as ::core::ffi::c_int;
        } else if (*infoPtr).dwStyle & WS_DISABLED as DWORD != 0 {
            style |= DFCS_INACTIVE as ::core::ffi::c_int;
        }

        DrawFrameControl(
            hdc,
            r as LPRECT,
            DFC_SCROLL as UINT,
            style as UINT,
        );
    }
}

unsafe extern "C" fn MONTHCAL_PaintTitle(
    infoPtr: *mut MONTHCAL_INFO,
    hdc: HDC,
    ps: *const PAINTSTRUCT,
    calIdx: INT,
) {
    static mmmmW: [WCHAR; 5] = [0x004Du16, 0x004D, 0x004D, 0x004D, 0x0000];
    static mmmW: [WCHAR; 4] = [0x004Du16, 0x004D, 0x004D, 0x0000];
    static mmW: [WCHAR; 3] = [0x004Du16, 0x004D, 0x0000];
    static fmtyearW: [WCHAR; 4] = [0x0025u16, 0x006C, 0x0064, 0x0000];
    static fmtmmW: [WCHAR; 5] = [0x0025u16, 0x0030, 0x0032, 0x0064, 0x0000];
    static fmtmW: [WCHAR; 3] = [0x0025u16, 0x0064, 0x0000];
    let title: *mut RECT = &raw mut (*(*infoPtr).calendars.offset(calIdx as isize)).title;
    let st: *const SYSTEMTIME = &raw const (*(*infoPtr).calendars.offset(calIdx as isize)).month;
    let mut monthW: [WCHAR; 80] = [0; 80];
    let mut strW: [WCHAR; 80] = [0; 80];
    let mut fmtW: [WCHAR; 80] = [0; 80];
    let mut yearW: [WCHAR; 6] = [0; 6];
    let mut yearoffset: INT;
    let mut monthoffset: INT;
    let mut shiftX: INT;
    let mut sz: SIZE = SIZE { cx: 0, cy: 0 };

    /* fill header box */
    FillRect(hdc, title, (*infoPtr).brushes[BrushTitle as usize]);

    /* month/year string */
    SetBkColor(hdc, (*infoPtr).colors[MCSC_TITLEBK as usize]);
    SetTextColor(hdc, (*infoPtr).colors[MCSC_TITLETEXT as usize]);
    SelectObject(hdc, ((*infoPtr).hBoldFont).cast());

    /* draw formatted date string */
    GetDateFormatW(
        LOCALE_USER_DEFAULT as LCID,
        DATE_YEARMONTH as DWORD,
        st,
        core::ptr::null::<WCHAR>(),
        &raw mut strW as LPWSTR,
        (ARRAY_SIZE!(strW)) as i32,
    );
    DrawTextW(
        hdc,
        &raw mut strW as *mut WCHAR as LPCWSTR,
        lstrlenW(&raw mut strW as *mut WCHAR as LPCWSTR),
        title as LPRECT,
        (DT_CENTER | DT_VCENTER | DT_SINGLELINE) as UINT,
    );

    GetLocaleInfoW(
        LOCALE_USER_DEFAULT as LCID,
        LOCALE_SYEARMONTH as LCTYPE,
        &raw mut fmtW as LPWSTR,
        (ARRAY_SIZE!(fmtW)) as i32,
    );
    wsprintfW(&raw mut yearW as LPWSTR, &raw const fmtyearW as LPCWSTR, (*st).wYear as INT);

    /* month is trickier as it's possible to have different format pictures, we'll
       test for M, MM, MMM, and MMMM */
    if !wcsstr(&raw mut fmtW as *mut WCHAR, &raw const mmmmW as *const WCHAR).is_null() {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT as LCID,
            (LOCALE_SMONTHNAME1 + ((*st).wMonth as INT) as u32 - 1) as LCTYPE,
            &raw mut monthW as LPWSTR,
            (ARRAY_SIZE!(monthW)) as i32,
        );
    } else if !wcsstr(&raw mut fmtW as *mut WCHAR, &raw const mmmW as *const WCHAR).is_null() {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT as LCID,
            (LOCALE_SABBREVMONTHNAME1 + ((*st).wMonth as INT) as u32 - 1) as LCTYPE,
            &raw mut monthW as LPWSTR,
            (ARRAY_SIZE!(monthW)) as i32,
        );
    } else if !wcsstr(&raw mut fmtW as *mut WCHAR, &raw const mmW as *const WCHAR).is_null() {
        wsprintfW(&raw mut monthW as LPWSTR, &raw const fmtmmW as LPCWSTR, (*st).wMonth as INT);
    } else {
        wsprintfW(&raw mut monthW as LPWSTR, &raw const fmtmW as LPCWSTR, (*st).wMonth as INT);
    }

    /* update hit boxes */
    yearoffset = 0 as INT;
    while strW[yearoffset as usize] != 0 {
        if wcsncmp(
            (&raw mut strW as *mut WCHAR).offset(yearoffset as isize) as *mut WCHAR,
            &raw mut yearW as *mut WCHAR,
            lstrlenW(&raw mut yearW as *mut WCHAR as LPCWSTR) as usize,
        ) == 0
        {
            break;
        }
        yearoffset += 1;
    }

    monthoffset = 0 as INT;
    while strW[monthoffset as usize] != 0 {
        if wcsncmp(
            (&raw mut strW as *mut WCHAR).offset(monthoffset as isize) as *mut WCHAR,
            &raw mut monthW as *mut WCHAR,
            lstrlenW(&raw mut monthW as *mut WCHAR as LPCWSTR) as usize,
        ) == 0
        {
            break;
        }
        monthoffset += 1;
    }

    /* for left limits use offsets */
    sz.cx = 0 as LONG;
    if yearoffset != 0 {
        GetTextExtentPoint32W(hdc, &raw mut strW as *mut WCHAR as LPCWSTR, yearoffset, &raw mut sz);
    }
    (*(*infoPtr).calendars.offset(calIdx as isize)).titleyear.left = sz.cx;

    sz.cx = 0 as LONG;
    if monthoffset != 0 {
        GetTextExtentPoint32W(hdc, &raw mut strW as *mut WCHAR as LPCWSTR, monthoffset, &raw mut sz);
    }
    (*(*infoPtr).calendars.offset(calIdx as isize)).titlemonth.left = sz.cx;

    /* for right limits use actual string parts lengths */
    GetTextExtentPoint32W(
        hdc,
        (&raw mut strW as *mut WCHAR).offset(yearoffset as isize) as *mut WCHAR as LPCWSTR,
        lstrlenW(&raw mut yearW as *mut WCHAR as LPCWSTR),
        &raw mut sz,
    );
    (*(*infoPtr).calendars.offset(calIdx as isize)).titleyear.right =
        (*(*infoPtr).calendars.offset(calIdx as isize)).titleyear.left + sz.cx;

    GetTextExtentPoint32W(
        hdc,
        &raw mut monthW as *mut WCHAR as LPCWSTR,
        lstrlenW(&raw mut monthW as *mut WCHAR as LPCWSTR),
        &raw mut sz,
    );
    (*(*infoPtr).calendars.offset(calIdx as isize)).titlemonth.right =
        (*(*infoPtr).calendars.offset(calIdx as isize)).titlemonth.left + sz.cx;

    /* Finally translate rectangles to match center aligned string,
       hit rectangles are relative to title rectangle before translation. */
    GetTextExtentPoint32W(
        hdc,
        &raw mut strW as *mut WCHAR as LPCWSTR,
        lstrlenW(&raw mut strW as *mut WCHAR as LPCWSTR),
        &raw mut sz,
    );
    shiftX = (((*title).right - (*title).left - sz.cx) / (2 as LONG) + (*title).left) as INT;
    OffsetRect(
        &raw mut (*(*infoPtr).calendars.offset(calIdx as isize)).titleyear,
        shiftX,
        0 as INT,
    );
    OffsetRect(
        &raw mut (*(*infoPtr).calendars.offset(calIdx as isize)).titlemonth,
        shiftX,
        0 as INT,
    );
}

unsafe extern "C" fn MONTHCAL_PaintWeeknumbers(infoPtr: *const MONTHCAL_INFO, hdc: HDC, ps: *const PAINTSTRUCT, calIdx: INT) {
    let date: *const SYSTEMTIME = core::ptr::addr_of!((*(*infoPtr).calendars.offset(calIdx as isize)).month);
    static fmt_weekW: [WCHAR; 3] = [0x0025u16, 0x0064, 0x0000];
    let mut mindays: INT = 0;
    let mut weeknum: INT = 0;
    let mut weeknum1: INT = 0;
    let mut startofprescal: INT = 0;
    let mut i: INT = 0;
    let mut prev_month: INT = 0;
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut buf: [WCHAR; 80] = [0 as WCHAR; 80];
    let mut old_pen: HPEN;
    let mut r: RECT = core::mem::zeroed();

    if ((*infoPtr).dwStyle & (MCS_WEEKNUMBERS as DWORD)) == 0 {
        return;
    }

    MONTHCAL_GetMinDate(infoPtr, core::ptr::addr_of_mut!(st));
    startofprescal = st.wDay as INT;
    st = *date;

    prev_month = (*date).wMonth as INT - 1 as INT;
    if prev_month == 0 as INT {
        prev_month = 12 as INT;
    }

    /*
       Rules what week to call the first week of a new year:
       LOCALE_IFIRSTWEEKOFYEAR == 0 (e.g US?):
       The week containing Jan 1 is the first week of year
       LOCALE_IFIRSTWEEKOFYEAR == 2 (e.g. Germany):
       First week of year must contain 4 days of the new year
       LOCALE_IFIRSTWEEKOFYEAR == 1  (what countries?)
       The first week of the year must contain only days of the new year
    */
    GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_IFIRSTWEEKOFYEAR, core::ptr::addr_of_mut!(buf) as *mut WCHAR, ARRAY_SIZE!(buf) as INT);
    weeknum = wcstol(core::ptr::addr_of_mut!(buf) as *mut WCHAR, core::ptr::null_mut(), 10 as INT) as INT;
    if weeknum == 1 as INT {
        mindays = 6 as INT;
    } else if weeknum == 2 as INT {
        mindays = 3 as INT;
    } else if weeknum == 0 as INT {
        mindays = 0 as INT;
    } else {
        WARN!(b"Unknown LOCALE_IFIRSTWEEKOFYEAR value %d, defaulting to 0\n\0".as_ptr() as *const i8, weeknum);
        mindays = 0 as INT;
    }

    if (*date).wMonth as INT == 1 as INT {
        /* calculate all those exceptions for January */
        st.wMonth = 1 as WORD;
        st.wDay = st.wMonth;
        weeknum1 = MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!(st), FALSE as _) as INT;
        if ((*infoPtr).firstDay - weeknum1) % (7 as INT) > mindays {
            weeknum = 1 as INT;
        } else {
            weeknum = 0 as INT;
            i = 0 as INT;
            while i < 11 as INT {
                weeknum += MONTHCAL_MonthLength(i + 1 as INT, (*date).wYear as INT - 1 as INT);
                i += 1;
            }

            weeknum += startofprescal + 7 as INT;
            weeknum /= 7 as INT;
            st.wYear = (st.wYear as INT - 1 as INT) as WORD;
            weeknum1 = MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!(st), FALSE as _) as INT;
            if ((*infoPtr).firstDay - weeknum1) % (7 as INT) > mindays {
                weeknum += 1;
            }
        }
    } else {
        weeknum = 0 as INT;
        i = 0 as INT;
        while i < prev_month - 1 as INT {
            weeknum += MONTHCAL_MonthLength(i + 1 as INT, (*date).wYear as INT);
            i += 1;
        }

        weeknum += startofprescal + 7 as INT;
        weeknum /= 7 as INT;
        st.wMonth = 1 as WORD;
        st.wDay = st.wMonth;
        weeknum1 = MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!(st), FALSE as _) as INT;
        if ((*infoPtr).firstDay - weeknum1) % (7 as INT) > mindays {
            weeknum += 1;
        }
    }

    r = (*(*infoPtr).calendars.offset(calIdx as isize)).weeknums;

    /* erase whole week numbers area */
    FillRect(hdc, core::ptr::addr_of_mut!(r), (*infoPtr).brushes[BrushMonth as usize]);
    SetTextColor(hdc, (*infoPtr).colors[MCSC_TITLEBK as usize]);

    /* reduce rectangle to one week number */
    r.bottom = (r.top as INT + (*infoPtr).height_increment) as LONG;

    i = 0 as INT;
    while i < 6 as INT {
        if (i == 0 as INT) && (weeknum > 50 as INT) {
            wsprintfW(core::ptr::addr_of_mut!(buf) as LPWSTR, core::ptr::addr_of!(fmt_weekW) as LPCWSTR, weeknum);
            weeknum = 0 as INT;
        } else if (i == 5 as INT) && (weeknum > 47 as INT) {
            wsprintfW(core::ptr::addr_of_mut!(buf) as LPWSTR, core::ptr::addr_of!(fmt_weekW) as LPCWSTR, 1 as INT);
        } else {
            wsprintfW(core::ptr::addr_of_mut!(buf) as LPWSTR, core::ptr::addr_of!(fmt_weekW) as LPCWSTR, weeknum + i);
        }

        DrawTextW(
            hdc,
            core::ptr::addr_of_mut!(buf) as *mut WCHAR as LPCWSTR,
            -(1 as INT),
            core::ptr::addr_of_mut!(r),
            (DT_CENTER | DT_VCENTER | DT_SINGLELINE) as UINT,
        );
        OffsetRect(core::ptr::addr_of_mut!(r), 0 as INT, (*infoPtr).height_increment);
        i += 1;
    }

    /* line separator for week numbers column */
    old_pen = SelectObject(hdc, (*infoPtr).pens[PenText as usize] as HGDIOBJ) as HPEN;
    MoveToEx(
        hdc,
        (*(*infoPtr).calendars.offset(calIdx as isize)).weeknums.right as INT,
        (*(*infoPtr).calendars.offset(calIdx as isize)).weeknums.top as INT + 3 as INT,
        core::ptr::null_mut(),
    );
    LineTo(
        hdc,
        (*(*infoPtr).calendars.offset(calIdx as isize)).weeknums.right as INT,
        (*(*infoPtr).calendars.offset(calIdx as isize)).weeknums.bottom as INT,
    );
    SelectObject(hdc, old_pen as HGDIOBJ);
    let _ = ps;
}

unsafe extern "C" fn MONTHCAL_PaintTodayTitle(infoPtr: *const MONTHCAL_INFO, hdc: HDC, ps: *const PAINTSTRUCT) {
    static fmt_todayW: [WCHAR; 6] = [0x0025u16, 0x0073, 0x0020, 0x0025, 0x0073, 0x0000];
    let mut buf_todayW: [WCHAR; 30] = [0; 30];
    let mut buf_dateW: [WCHAR; 20] = [0; 20];
    let mut buf: [WCHAR; 80] = [0; 80];
    let mut text_rect: RECT = core::mem::zeroed();
    let mut box_rect: RECT = core::mem::zeroed();
    let mut old_font: HFONT = core::ptr::null_mut();
    let mut col: INT;

    if ((*infoPtr).dwStyle & MCS_NOTODAY) != 0 {
        return;
    }

    LoadStringW(COMCTL32_hModule, IDM_TODAY, buf_todayW.as_mut_ptr(), (ARRAY_SIZE!(buf_todayW)) as INT);
    col = if ((*infoPtr).dwStyle & MCS_NOTODAYCIRCLE) != 0 { 0 } else { 1 };
    if ((*infoPtr).dwStyle & MCS_WEEKNUMBERS) != 0 {
        col -= 1;
    }
    /* label is located below first calendar last row */
    MONTHCAL_GetDayRectI(infoPtr, core::ptr::addr_of_mut!(text_rect), col, 6, (*infoPtr).dim.cx * (*infoPtr).dim.cy - (*infoPtr).dim.cx);
    box_rect = text_rect;

    GetDateFormatW(LOCALE_USER_DEFAULT, DATE_SHORTDATE, core::ptr::addr_of!((*infoPtr).todaysDate), core::ptr::null(), buf_dateW.as_mut_ptr(), (ARRAY_SIZE!(buf_dateW)) as i32);
    old_font = (SelectObject(hdc, ((*infoPtr).hBoldFont).cast())).cast();
    SetTextColor(hdc, (*infoPtr).colors[MCSC_TEXT as usize]);

    wsprintfW(buf.as_mut_ptr(), fmt_todayW.as_ptr(), buf_todayW.as_mut_ptr(), buf_dateW.as_mut_ptr());
    DrawTextW(hdc, buf.as_ptr(), -(1), core::ptr::addr_of_mut!(text_rect), (DT_CALCRECT | DT_LEFT | DT_VCENTER | DT_SINGLELINE) as UINT);
    DrawTextW(hdc, buf.as_ptr(), -(1), core::ptr::addr_of_mut!(text_rect), (DT_LEFT | DT_VCENTER | DT_SINGLELINE) as UINT);

    if ((*infoPtr).dwStyle & MCS_NOTODAYCIRCLE) == 0 {
        OffsetRect(core::ptr::addr_of_mut!(box_rect), -(*infoPtr).width_increment, 0);
        MONTHCAL_Circle(infoPtr, hdc, core::ptr::addr_of_mut!(box_rect));
    }

    SelectObject(hdc, (old_font).cast());
}

unsafe extern "C" fn MONTHCAL_PaintFocusAndCircle(infoPtr: *const MONTHCAL_INFO, hdc: HDC, ps: *const PAINTSTRUCT) {
    /* circle today date if only it's in fully visible month */
    if !(((*infoPtr).dwStyle & (MCS_NOTODAYCIRCLE as DWORD)) != 0) {
        let mut i: INT;

        i = 0 as INT;
        while i < MONTHCAL_GetCalCount(infoPtr) {
            if MONTHCAL_CompareMonths(
                core::ptr::addr_of!((*infoPtr).todaysDate),
                core::ptr::addr_of!((*(*infoPtr).calendars.offset(i as isize)).month),
            ) == 0
            {
                MONTHCAL_CircleDay(infoPtr, hdc, core::ptr::addr_of!((*infoPtr).todaysDate));
                break;
            }
            i += 1;
        }
    }

    if MONTHCAL_IsDateEqual(
        core::ptr::addr_of!((*infoPtr).focusedSel),
        core::ptr::addr_of!(st_null),
    ) == 0
    {
        let mut r: RECT = core::mem::zeroed();
        MONTHCAL_GetDayRect(
            infoPtr,
            core::ptr::addr_of!((*infoPtr).focusedSel),
            core::ptr::addr_of_mut!(r),
            -(1 as INT),
        );
        DrawFocusRect(hdc, core::ptr::addr_of!(r));
    }
}

unsafe extern "C" fn MONTHCAL_PaintLeadTrailMonths(infoPtr: *const MONTHCAL_INFO, hdc: HDC, ps: *const PAINTSTRUCT) {
    let mut mask: INT = 0;
    let mut index: INT = 0;
    let mut length: UINT = 0;
    let mut st_max: SYSTEMTIME = core::mem::zeroed();
    let mut st: SYSTEMTIME = core::mem::zeroed();

    if ((*infoPtr).dwStyle & MCS_NOTRAILINGDATES) != 0 {
        return;
    }

    SetTextColor(hdc, (*infoPtr).colors[MCSC_TRAILINGTEXT as usize]);

    /* draw prev month */
    MONTHCAL_GetMinDate(infoPtr, core::ptr::addr_of_mut!(st));
    mask = (1 << (st.wDay as INT - 1)) as INT;
    /* December and January both 31 days long, so no worries if wrapped */
    length = MONTHCAL_MonthLength(
        (*(*infoPtr).calendars.offset(0 as isize)).month.wMonth as INT - 1,
        (*(*infoPtr).calendars.offset(0 as isize)).month.wYear as INT,
    ) as UINT;
    index = 0;
    while (st.wDay as UINT) <= length {
        MONTHCAL_DrawDay(
            infoPtr,
            hdc,
            core::ptr::addr_of_mut!(st),
            (*(*infoPtr).monthdayState.offset(index as isize) & mask as MONTHDAYSTATE) as INT,
            ps,
        );
        mask <<= 1;
        st.wDay = st.wDay.wrapping_add(1);
    }

    /* draw next month */
    st = (*(*infoPtr).calendars.offset(
        (MONTHCAL_GetCalCount(infoPtr) - 1) as isize,
    ))
    .month;
    st.wDay = 1 as WORD;
    MONTHCAL_GetNextMonth(core::ptr::addr_of_mut!(st));
    MONTHCAL_GetMaxDate(infoPtr, core::ptr::addr_of_mut!(st_max));
    mask = 1;
    index = (MONTHCAL_GetMonthRange(infoPtr, GMR_DAYSTATE, core::ptr::null_mut()) - 1) as INT;
    while (st.wDay as INT) <= (st_max.wDay as INT) {
        MONTHCAL_DrawDay(
            infoPtr,
            hdc,
            core::ptr::addr_of_mut!(st),
            (*(*infoPtr).monthdayState.offset(index as isize) & mask as MONTHDAYSTATE) as INT,
            ps,
        );
        mask <<= 1;
        st.wDay = st.wDay.wrapping_add(1);
    }
}

unsafe extern "C" fn get_localized_dayname(
    infoPtr: *const MONTHCAL_INFO,
    day: ::core::ffi::c_uint,
    buff: *mut WCHAR,
    count: ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    let mut lctype: LCTYPE = 0;
    if ((*infoPtr).dwStyle & MCS_SHORTDAYSOFWEEK as DWORD) != 0 {
        lctype = (LOCALE_SSHORTESTDAYNAME1 as ::core::ffi::c_uint).wrapping_add(day) as LCTYPE;
    } else {
        lctype = (LOCALE_SABBREVDAYNAME1 as ::core::ffi::c_uint).wrapping_add(day) as LCTYPE;
    }
    return GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        lctype,
        buff as LPWSTR,
        count as ::core::ffi::c_int,
    );
}

unsafe extern "C" fn MONTHCAL_PaintCalendar(
    mut infoPtr: *const MONTHCAL_INFO,
    mut hdc: HDC,
    mut ps: *const PAINTSTRUCT,
    mut calIdx: INT,
) {
    let mut date: *const SYSTEMTIME =
        &raw mut (*(*infoPtr).calendars.offset(calIdx as isize)).month;
    let mut i: INT = 0;
    let mut j: INT = 0;
    let mut length: UINT = 0;
    let mut r: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut fill_bk_rect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut st: SYSTEMTIME = _SYSTEMTIME {
        wYear: 0,
        wMonth: 0,
        wDayOfWeek: 0,
        wDay: 0,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    };
    let mut buf: [WCHAR; 80] = [0; 80];
    let mut old_pen: HPEN = ::core::ptr::null_mut::<HPEN__>();
    let mut mask: ::core::ffi::c_int = 0;

    /* fill whole days area - from week days area to today note rectangle */
    fill_bk_rect = (*(*infoPtr).calendars.offset(calIdx as isize)).wdays;
    fill_bk_rect.bottom = (*(*infoPtr).calendars.offset(calIdx as isize)).days.bottom
        + ((*infoPtr).todayrect.bottom - (*infoPtr).todayrect.top);

    FillRect(
        hdc,
        &raw mut fill_bk_rect as LPCRECT,
        (*infoPtr).brushes[BrushMonth as ::core::ffi::c_int as usize],
    );

    /* draw line under day abbreviations */
    old_pen = SelectObject(
        hdc,
        (*infoPtr).pens[PenText as ::core::ffi::c_int as usize] as HGDIOBJ,
    ) as HPEN;
    MoveToEx(
        hdc,
        (*(*infoPtr).calendars.offset(calIdx as isize)).days.left as ::core::ffi::c_int
            + 3,
        (*(*infoPtr).calendars.offset(calIdx as isize)).title.bottom as ::core::ffi::c_int
            + (*infoPtr).textHeight
            + 1,
        ::core::ptr::null_mut::<tagPOINT>(),
    );
    LineTo(
        hdc,
        (*(*infoPtr).calendars.offset(calIdx as isize)).days.right as ::core::ffi::c_int
            - 3,
        (*(*infoPtr).calendars.offset(calIdx as isize)).title.bottom as ::core::ffi::c_int
            + (*infoPtr).textHeight
            + 1,
    );
    SelectObject(hdc, old_pen as HGDIOBJ);

    let fresh15: LONG = (*(*infoPtr).calendars.offset(calIdx as isize))
        .weeknums
        .right;
    (*(*infoPtr).calendars.offset(calIdx as isize)).days.left = fresh15;
    (*(*infoPtr).calendars.offset(calIdx as isize)).wdays.left = fresh15;

    /* draw day abbreviations */
    SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ);
    SetBkColor(
        hdc,
        (*infoPtr).colors[MCSC_MONTHBK as usize],
    );
    SetTextColor(
        hdc,
        (*infoPtr).colors[MCSC_TITLEBK as usize],
    );
    /* rectangle to draw a single day abbreviation within */
    r = (*(*infoPtr).calendars.offset(calIdx as isize)).wdays;
    r.right = (r.left as ::core::ffi::c_int + (*infoPtr).width_increment) as LONG;

    i = (*infoPtr).firstDay as INT;
    j = 0 as INT;
    while j < 7 as INT {
        get_localized_dayname(
            infoPtr,
            ((i as ::core::ffi::c_int + j as ::core::ffi::c_int + 6)
                % 7) as ::core::ffi::c_uint,
            &raw mut buf as *mut WCHAR,
            ARRAY_SIZE!(buf) as ::core::ffi::c_uint,
        );
        DrawTextW(
            hdc,
            &raw mut buf as *mut WCHAR as LPCWSTR,
            lstrlenW(&raw mut buf as *mut WCHAR as LPCWSTR),
            &raw mut r,
            (DT_CENTER | DT_VCENTER | DT_SINGLELINE) as UINT,
        );
        OffsetRect(
            &raw mut r,
            (*infoPtr).width_increment,
            0 as ::core::ffi::c_int,
        );
        j += 1;
    }

    /* draw current month */
    SetTextColor(
        hdc,
        (*infoPtr).colors[MCSC_TEXT as usize],
    );
    st = *date;
    st.wDay = 1 as WORD;
    mask = 1 as ::core::ffi::c_int;
    length = MONTHCAL_MonthLength(
        (*date).wMonth as ::core::ffi::c_int,
        (*date).wYear as ::core::ffi::c_int,
    ) as UINT;
    while st.wDay as UINT <= length {
        MONTHCAL_DrawDay(
            infoPtr,
            hdc,
            &raw mut st,
            (*(*infoPtr)
                .monthdayState
                .offset((calIdx as ::core::ffi::c_int + 1) as isize)
                & mask as MONTHDAYSTATE) as ::core::ffi::c_int,
            ps,
        );
        mask <<= 1 as ::core::ffi::c_int;
        st.wDay = st.wDay.wrapping_add(1);
    }
}

unsafe extern "C" fn MONTHCAL_Refresh(infoPtr: *mut MONTHCAL_INFO, hdc: HDC, ps: *const PAINTSTRUCT) {
    let mut old_text_clr: COLORREF = 0;
    let mut old_bk_clr: COLORREF = 0;
    let mut old_font: HFONT = core::ptr::null_mut();
    let mut i: INT = 0;

    old_text_clr = SetTextColor(hdc, comctl32_color.clrWindowText);
    old_bk_clr = GetBkColor(hdc);
    old_font = GetCurrentObject(hdc, OBJ_FONT) as HFONT;

    i = 0;
    while i < MONTHCAL_GetCalCount(infoPtr) {
        let title: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(i as isize)).title);
        let mut r: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        /* draw title, redraw all its elements */
        if IntersectRect(core::ptr::addr_of_mut!(r), core::ptr::addr_of!((*ps).rcPaint), title as LPCRECT) != 0 {
            MONTHCAL_PaintTitle(infoPtr, hdc, ps, i);
        }

        /* draw calendar area */
        UnionRect(
            core::ptr::addr_of_mut!(r),
            core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(i as isize)).wdays) as LPCRECT,
            core::ptr::addr_of_mut!((*infoPtr).todayrect) as LPCRECT,
        );
        if IntersectRect(core::ptr::addr_of_mut!(r), core::ptr::addr_of!((*ps).rcPaint), core::ptr::addr_of_mut!(r) as LPCRECT) != 0 {
            MONTHCAL_PaintCalendar(infoPtr, hdc, ps, i);
        }

        /* week numbers */
        MONTHCAL_PaintWeeknumbers(infoPtr, hdc, ps, i);

        i += 1;
    }

    /* partially visible months */
    MONTHCAL_PaintLeadTrailMonths(infoPtr, hdc, ps);

    /* focus and today rectangle */
    MONTHCAL_PaintFocusAndCircle(infoPtr, hdc, ps);

    /* today at the bottom left */
    MONTHCAL_PaintTodayTitle(infoPtr, hdc, ps);

    /* navigation buttons */
    MONTHCAL_PaintButton(infoPtr, hdc, (DIRECTION_BACKWARD) as nav_direction);
    MONTHCAL_PaintButton(infoPtr, hdc, (DIRECTION_FORWARD) as nav_direction);

    /* restore context */
    SetBkColor(hdc, old_bk_clr);
    SelectObject(hdc, old_font as HGDIOBJ);
    SetTextColor(hdc, old_text_clr);
}

unsafe extern "C" fn MONTHCAL_GetMinReqRect(
    infoPtr: *const MONTHCAL_INFO,
    rect: *mut RECT,
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
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_GetMinReqRect\0" as *const u8 as *const ::core::ffi::c_char,
            1278 as ::core::ffi::c_int,
            b"rect %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            rect,
        );
    }

    if rect.is_null() {
        return FALSE as LRESULT;
    }

    *rect = (*(*infoPtr).calendars.offset(0 as isize)).title;
    (*rect).bottom = (*(*infoPtr).calendars.offset(0 as isize))
        .days
        .bottom
        + (*infoPtr).todayrect.bottom
        - (*infoPtr).todayrect.top;

    AdjustWindowRect(rect as LPRECT, (*infoPtr).dwStyle, FALSE);

    /* minimal rectangle is zero based */
    OffsetRect(
        rect as LPRECT,
        -((*rect).left as ::core::ffi::c_int),
        -((*rect).top as ::core::ffi::c_int),
    );

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_GetMinReqRect\0" as *const u8 as *const ::core::ffi::c_char,
            1291 as ::core::ffi::c_int,
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(rect),
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn MONTHCAL_GetColor(infoPtr: *const MONTHCAL_INFO, index: UINT) -> COLORREF {
    if index > MCSC_TRAILINGTEXT as UINT {
        return (-(1 as i32)) as COLORREF;
    }
    return (*infoPtr).colors[index as usize];
}

unsafe extern "C" fn MONTHCAL_SetColor(
    mut infoPtr: *mut MONTHCAL_INFO,
    mut index: UINT,
    mut color: COLORREF,
) -> LRESULT {
    let mut type_: CachedBrush = (BrushTitle as u32);
    let mut prev: COLORREF = 0;
    if index > MCSC_TRAILINGTEXT as UINT {
        return -(1 as LRESULT);
    }
    prev = (*infoPtr).colors[index as usize];
    (*infoPtr).colors[index as usize] = color;
    match index {
        MCSC_BACKGROUND => {
            type_ = (BrushBackground as u32);
        }
        MCSC_TITLEBK => {
            type_ = (BrushTitle as u32);
        }
        MCSC_MONTHBK => {
            type_ = (BrushMonth as u32);
        }
        _ => {
            type_ = (BrushLast as u32);
        }
    }
    if type_ as ::core::ffi::c_uint != BrushLast as ::core::ffi::c_int as ::core::ffi::c_uint {
        DeleteObject((*infoPtr).brushes[type_ as usize] as HGDIOBJ);
        (*infoPtr).brushes[type_ as usize] = CreateSolidBrush(color);
    }
    if index == MCSC_TEXT as UINT {
        DeleteObject((*infoPtr).pens[PenText as ::core::ffi::c_int as usize] as HGDIOBJ);
        (*infoPtr).pens[PenText as ::core::ffi::c_int as usize] = CreatePen(
            (PS_SOLID as i32),
            1,
            (*infoPtr).colors[index as usize],
        );
    }
    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null::<RECT>(),
        (index == MCSC_BACKGROUND as UINT) as ::core::ffi::c_int,
    );
    return prev as LRESULT;
}

unsafe extern "C" fn MONTHCAL_GetMonthDelta(infoPtr: *const MONTHCAL_INFO) -> LRESULT {
    if (*infoPtr).delta != 0 {
        return (*infoPtr).delta as LRESULT;
    }

    return MONTHCAL_GetMonthRange(infoPtr, GMR_VISIBLE as DWORD, core::ptr::null_mut());
}
unsafe extern "C" fn MONTHCAL_SetMonthDelta(
    mut infoPtr: *mut MONTHCAL_INFO,
    mut delta: INT,
) -> LRESULT {
    let mut prev: INT = (*infoPtr).delta;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_SetMonthDelta\0" as *const u8 as *const ::core::ffi::c_char,
            1368 as ::core::ffi::c_int,
            b"delta %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            delta,
        );
    }
    (*infoPtr).delta = delta;
    return prev as LRESULT;
}

unsafe extern "C" fn MONTHCAL_GetFirstDayOfWeek(infoPtr: *const MONTHCAL_INFO) -> LRESULT {
    let mut day: ::core::ffi::c_int;

    /* convert from SYSTEMTIME to locale format */
    day = if (*infoPtr).firstDay >= 0 {
        ((*infoPtr).firstDay + 6) % 7
    } else {
        (*infoPtr).firstDay
    };

    return MAKELONG!(day, (*infoPtr).firstDaySet) as LRESULT;
}

unsafe extern "C" fn MONTHCAL_SetFirstDayOfWeek(infoPtr: *mut MONTHCAL_INFO, day: INT) -> LRESULT {
    let mut prev: LRESULT = MONTHCAL_GetFirstDayOfWeek(infoPtr);
    let mut new_day: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_SetFirstDayOfWeek\0" as *const u8 as *const ::core::ffi::c_char,
            1409 as ::core::ffi::c_int,
            b"%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            day,
        );
    }
    if day == -1 {
        let mut buf: [WCHAR; 80] = [0; 80];
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_IFIRSTDAYOFWEEK,
            &raw mut buf as *mut WCHAR,
            (ARRAY_SIZE!(buf)) as i32,
        );
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/monthcal.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MONTHCAL_SetFirstDayOfWeek\0" as *const u8
                    as *const ::core::ffi::c_char,
                1416 as ::core::ffi::c_int,
                b"%s %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(&raw mut buf as *mut WCHAR),
                lstrlenW(&raw mut buf as *mut WCHAR as LPCWSTR),
            );
        }
        new_day = wcstol(
            &raw mut buf as *mut WCHAR,
            ::core::ptr::null_mut::<*mut WCHAR>(),
            10,
        ) as ::core::ffi::c_int;
        (*infoPtr).firstDaySet = FALSE;
    } else if day >= 7 {
        new_day = 6;
        (*infoPtr).firstDaySet = TRUE;
    } else {
        if day < -1 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/monthcal.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"MONTHCAL_SetFirstDayOfWeek\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1434 as ::core::ffi::c_int,
                    b"No bug compatibility for day=%d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    day,
                );
            }
        }
        new_day = day as ::core::ffi::c_int;
        (*infoPtr).firstDaySet = TRUE;
    }
    (*infoPtr).firstDay = if new_day >= 0 {
        new_day += 1;
        new_day % 7
    } else {
        new_day
    };
    InvalidateRect(
        (*infoPtr).hwndSelf,
        ::core::ptr::null::<RECT>(),
        FALSE,
    );
    return prev;
}

unsafe extern "C" fn MONTHCAL_GetMaxTodayWidth(infoPtr: *const MONTHCAL_INFO) -> LRESULT {
    return ((*infoPtr).todayrect.right - (*infoPtr).todayrect.left) as LRESULT;
}

unsafe extern "C" fn MONTHCAL_SetRange(
    infoPtr: *mut MONTHCAL_INFO,
    limits: SHORT,
    range: *mut SYSTEMTIME,
) -> LRESULT {
    let mut ft_min: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut ft_max: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };

    if ((limits as i32 & (GDTR_MIN as i32)) != 0
        && MONTHCAL_ValidateDate(range.offset(0 as isize)) == 0)
        || ((limits as i32 & (GDTR_MAX as i32)) != 0
            && MONTHCAL_ValidateDate(range.offset(1 as isize)) == 0)
    {
        return FALSE as LRESULT;
    }

    (*infoPtr).rangeValid = 0 as DWORD;
    (*infoPtr).maxDate = st_null;
    (*infoPtr).minDate = (*infoPtr).maxDate;

    if (limits as i32 & (GDTR_MIN as i32)) != 0 {
        if MONTHCAL_ValidateTime(range.offset(0 as isize)) == 0 {
            MONTHCAL_CopyTime(
                core::ptr::addr_of_mut!((*infoPtr).todaysDate),
                range.offset(0 as isize),
            );
        }
        (*infoPtr).minDate = *range.offset(0 as isize);
        (*infoPtr).rangeValid |= GDTR_MIN as DWORD;
    }
    if (limits as i32 & (GDTR_MAX as i32)) != 0 {
        if MONTHCAL_ValidateTime(range.offset(1 as isize)) == 0 {
            MONTHCAL_CopyTime(
                core::ptr::addr_of_mut!((*infoPtr).todaysDate),
                range.offset(1 as isize),
            );
        }
        (*infoPtr).maxDate = *range.offset(1 as isize);
        (*infoPtr).rangeValid |= GDTR_MAX as DWORD;
    }

    /* Only one limit set - we are done */
    if ((*infoPtr).rangeValid & (GDTR_MIN | GDTR_MAX) as DWORD)
        != (GDTR_MIN | GDTR_MAX) as DWORD
    {
        return TRUE as LRESULT;
    }

    SystemTimeToFileTime(
        core::ptr::addr_of_mut!((*infoPtr).maxDate),
        core::ptr::addr_of_mut!(ft_max),
    );
    SystemTimeToFileTime(
        core::ptr::addr_of_mut!((*infoPtr).minDate),
        core::ptr::addr_of_mut!(ft_min),
    );

    if CompareFileTime(
        core::ptr::addr_of_mut!(ft_min),
        core::ptr::addr_of_mut!(ft_max),
    ) >= 0
    {
        if ((limits as i32 & ((GDTR_MIN | GDTR_MAX) as i32))) as u32
            == (GDTR_MIN | GDTR_MAX)
        {
            /* Native swaps limits only when both limits are being set. */
            let mut st_tmp: SYSTEMTIME = (*infoPtr).minDate;
            (*infoPtr).minDate = (*infoPtr).maxDate;
            (*infoPtr).maxDate = st_tmp;
        } else {
            /* reset the other limit */
            if (limits as i32 & (GDTR_MIN as i32)) != 0 {
                (*infoPtr).maxDate = st_null;
            }
            if (limits as i32 & (GDTR_MAX as i32)) != 0 {
                (*infoPtr).minDate = st_null;
            }
            (*infoPtr).rangeValid &= (if (limits as i32 & (GDTR_MIN as i32)) != 0 {
                !GDTR_MAX
            } else {
                !GDTR_MIN
            }) as DWORD;
        }
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn MONTHCAL_GetRange(infoPtr: *const MONTHCAL_INFO, range: *mut SYSTEMTIME) -> LRESULT {
    if range.is_null() {
        return 0 as LRESULT;
    }
    *range.offset(1 as isize) = (*infoPtr).maxDate;
    *range.offset(0 as isize) = (*infoPtr).minDate;

    return (*infoPtr).rangeValid as LRESULT;
}

unsafe extern "C" fn MONTHCAL_SetDayState(
    infoPtr: *const MONTHCAL_INFO,
    months: INT,
    states: *mut MONTHDAYSTATE,
) -> LRESULT {
    if ((*infoPtr).dwStyle & (MCS_DAYSTATE as DWORD)) == 0 {
        return 0 as LRESULT;
    }
    if (months as LRESULT)
        != MONTHCAL_GetMonthRange(
            infoPtr,
            GMR_DAYSTATE as DWORD,
            0 as *mut SYSTEMTIME,
        )
    {
        return 0 as LRESULT;
    }

    memcpy(
        (*infoPtr).monthdayState as *mut core::ffi::c_void,
        states as *const core::ffi::c_void,
        (months as usize).wrapping_mul(core::mem::size_of::<MONTHDAYSTATE>()),
    );

    return 1 as LRESULT;
}

unsafe extern "C" fn MONTHCAL_GetCurSel(infoPtr: *const MONTHCAL_INFO, curSel: *mut SYSTEMTIME) -> LRESULT {
    if curSel.is_null() {
        return (FALSE) as LRESULT;
    }
    if ((*infoPtr).dwStyle & ((MCS_MULTISELECT) as DWORD)) != 0 {
        return (FALSE) as LRESULT;
    }

    *curSel = (*infoPtr).minSel;

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn MONTHCAL_SetCurSel(
    mut infoPtr: *mut MONTHCAL_INFO,
    mut curSel: *mut SYSTEMTIME,
) -> LRESULT {
    let mut prev: SYSTEMTIME = (*infoPtr).minSel;
    let mut selection: SYSTEMTIME = core::mem::zeroed();
    let mut diff: INT = 0;
    let mut day: WORD = 0;

    if curSel.is_null() {
        return FALSE as LRESULT;
    }
    if ((*infoPtr).dwStyle & (MCS_MULTISELECT as DWORD)) != 0 {
        return FALSE as LRESULT;
    }

    if MONTHCAL_ValidateDate(curSel) == 0 {
        return FALSE as LRESULT;
    }
    /* exit earlier if selection equals current */
    if MONTHCAL_IsDateEqual(&raw mut (*infoPtr).minSel, curSel) != 0 {
        return TRUE as LRESULT;
    }

    selection = *curSel;
    selection.wMilliseconds = 0 as WORD;
    selection.wSecond = selection.wMilliseconds;
    selection.wMinute = selection.wSecond;
    selection.wHour = selection.wMinute;
    MONTHCAL_CalculateDayOfWeek(&raw mut selection, TRUE);

    if MONTHCAL_IsDateInValidRange(infoPtr, &raw mut selection, FALSE) == 0 {
        return FALSE as LRESULT;
    }

    /* scroll calendars only if we have to */
    diff = MONTHCAL_MonthDiff(
        &raw mut (*(*infoPtr)
            .calendars
            .offset((MONTHCAL_GetCalCount(infoPtr) - 1) as isize))
        .month,
        curSel,
    ) as INT;
    if diff <= 0 {
        diff = MONTHCAL_MonthDiff(
            &raw mut (*(*infoPtr).calendars.offset(0 as isize)).month,
            curSel,
        ) as INT;
        if diff > 0 {
            diff = 0 as INT;
        }
    }

    if diff != 0 {
        let mut i: INT = 0;

        i = 0 as INT;
        while i < MONTHCAL_GetCalCount(infoPtr) {
            MONTHCAL_GetMonth(&raw mut (*(*infoPtr).calendars.offset(i as isize)).month, diff);
            i += 1;
        }
    }

    /* we need to store time part as it is */
    selection = *curSel;
    MONTHCAL_CalculateDayOfWeek(&raw mut selection, TRUE);
    (*infoPtr).maxSel = selection;
    (*infoPtr).minSel = (*infoPtr).maxSel;

    /* if selection is still in current month, reduce rectangle */
    day = prev.wDay;
    prev.wDay = (*curSel).wDay;
    if MONTHCAL_IsDateEqual(&raw mut prev, curSel) != 0 {
        let mut r_prev: RECT = core::mem::zeroed();
        let mut r_new: RECT = core::mem::zeroed();

        prev.wDay = day;
        MONTHCAL_GetDayRect(infoPtr, &raw mut prev, &raw mut r_prev, -1);
        MONTHCAL_GetDayRect(infoPtr, curSel, &raw mut r_new, -1);

        InvalidateRect((*infoPtr).hwndSelf, &raw mut r_prev as LPCRECT, FALSE);
        InvalidateRect((*infoPtr).hwndSelf, &raw mut r_new as LPCRECT, FALSE);
    } else {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null(), FALSE);
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn MONTHCAL_GetMaxSelCount(infoPtr: *const MONTHCAL_INFO) -> LRESULT {
    ((*infoPtr).maxSelCount) as _
}

unsafe extern "C" fn MONTHCAL_SetMaxSelCount(infoPtr: *mut MONTHCAL_INFO, max: INT) -> LRESULT {
    if ((*infoPtr).dwStyle & (MCS_MULTISELECT as DWORD)) == 0 {
        return FALSE as LRESULT;
    }
    if max <= (0 as INT) {
        return FALSE as LRESULT;
    }

    (*infoPtr).maxSelCount = max;

    return TRUE as LRESULT;
}

unsafe extern "C" fn MONTHCAL_GetSelRange(
    infoPtr: *const MONTHCAL_INFO,
    range: *mut SYSTEMTIME,
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
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_GetSelRange\0" as *const u8 as *const ::core::ffi::c_char,
            1640 as ::core::ffi::c_int,
            b"%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            range,
        );
    }

    if range.is_null() {
        return FALSE as LRESULT;
    }

    if ((*infoPtr).dwStyle & (MCS_MULTISELECT as DWORD)) != 0 {
        *range.offset(1 as isize) = (*infoPtr).maxSel;
        *range.offset(0 as isize) = (*infoPtr).minSel;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/monthcal.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MONTHCAL_GetSelRange\0" as *const u8 as *const ::core::ffi::c_char,
                1648 as ::core::ffi::c_int,
                b"[min,max]=[%d %d]\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*infoPtr).minSel.wDay as ::core::ffi::c_int,
                (*infoPtr).maxSel.wDay as ::core::ffi::c_int,
            );
        }
        return TRUE as LRESULT;
    }

    return FALSE as LRESULT;
}

unsafe extern "C" fn MONTHCAL_SetSelRange(mut infoPtr: *mut MONTHCAL_INFO, mut range: *mut SYSTEMTIME) -> LRESULT {
    let mut old_range: [SYSTEMTIME; 2 as usize] = core::mem::zeroed();
    let mut diff: INT = 0;

    if range.is_null() || ((*infoPtr).dwStyle & (MCS_MULTISELECT as DWORD)) == 0 {
        return FALSE as LRESULT;
    }

    /* adjust timestamps */
    if MONTHCAL_ValidateTime(range.offset(0 as isize) as *mut SYSTEMTIME) == 0 {
        MONTHCAL_CopyTime(core::ptr::addr_of_mut!((*infoPtr).todaysDate), range.offset(0 as isize) as *mut SYSTEMTIME);
    }
    if MONTHCAL_ValidateTime(range.offset(1 as isize) as *mut SYSTEMTIME) == 0 {
        MONTHCAL_CopyTime(core::ptr::addr_of_mut!((*infoPtr).todaysDate), range.offset(1 as isize) as *mut SYSTEMTIME);
    }

    /* maximum range exceeded */
    if MONTHCAL_IsSelRangeValid(
        infoPtr,
        range.offset(0 as isize) as *mut SYSTEMTIME,
        range.offset(1 as isize) as *mut SYSTEMTIME,
        core::ptr::null_mut::<SYSTEMTIME>(),
    ) == 0
    {
        return FALSE as LRESULT;
    }

    old_range[0 as usize] = (*infoPtr).minSel;
    old_range[1 as usize] = (*infoPtr).maxSel;

    /* swap if min > max */
    if MONTHCAL_CompareSystemTime(
        range.offset(0 as isize) as *mut SYSTEMTIME,
        range.offset(1 as isize) as *mut SYSTEMTIME,
    ) <= (0 as LONG)
    {
        (*infoPtr).minSel = *range.offset(0 as isize);
        (*infoPtr).maxSel = *range.offset(1 as isize);
    } else {
        (*infoPtr).minSel = *range.offset(1 as isize);
        (*infoPtr).maxSel = *range.offset(0 as isize);
    }

    diff = MONTHCAL_MonthDiff(
        core::ptr::addr_of_mut!(
            (*(*infoPtr).calendars.offset(
                (MONTHCAL_GetCalCount(infoPtr) - (1 as INT)) as isize,
            ))
            .month
        ),
        core::ptr::addr_of_mut!((*infoPtr).maxSel),
    ) as INT;
    if diff < (0 as INT) {
        diff = MONTHCAL_MonthDiff(
            core::ptr::addr_of_mut!(
                (*(*infoPtr).calendars.offset(0 as isize)).month
            ),
            core::ptr::addr_of_mut!((*infoPtr).maxSel),
        ) as INT;
        if diff > (0 as INT) {
            diff = 0 as INT;
        }
    }

    if diff != (0 as INT) {
        let mut i: INT = 0;

        i = 0 as INT;
        while i < MONTHCAL_GetCalCount(infoPtr) {
            MONTHCAL_GetMonth(core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(i as isize)).month), diff);
            i += 1;
        }
    }

    /* update day of week */
    MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!((*infoPtr).minSel), TRUE as BOOL);
    MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!((*infoPtr).maxSel), TRUE as BOOL);

    /* redraw if bounds changed */
    /* FIXME: no actual need to redraw everything */
    if MONTHCAL_IsDateEqual(
        (core::ptr::addr_of_mut!(old_range) as *mut SYSTEMTIME).offset(0 as isize) as *mut SYSTEMTIME,
        range.offset(0 as isize) as *mut SYSTEMTIME,
    ) == 0
        || MONTHCAL_IsDateEqual(
            (core::ptr::addr_of_mut!(old_range) as *mut SYSTEMTIME).offset(1 as isize) as *mut SYSTEMTIME,
            range.offset(1 as isize) as *mut SYSTEMTIME,
        ) == 0
    {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), FALSE as BOOL);
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn MONTHCAL_GetToday(
    mut infoPtr: *const MONTHCAL_INFO,
    mut today: *mut SYSTEMTIME,
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
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_GetToday\0" as *const u8 as *const ::core::ffi::c_char,
            1723 as ::core::ffi::c_int,
            b"%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            today,
        );
    }
    if today.is_null() {
        return FALSE as LRESULT;
    }
    *today = (*infoPtr).todaysDate;
    return TRUE as LRESULT;
}

unsafe extern "C" fn MONTHCAL_UpdateToday(infoPtr: *mut MONTHCAL_INFO, today: *const SYSTEMTIME) -> BOOL {
    let mut rect: RECT = core::mem::zeroed();

    if MONTHCAL_IsDateEqual(today, core::ptr::addr_of_mut!((*infoPtr).todaysDate)) != 0 {
        return FALSE;
    }

    /* Invalidate old and new today day rectangle, and today label. */
    if MONTHCAL_GetDayRect(infoPtr, core::ptr::addr_of_mut!((*infoPtr).todaysDate), core::ptr::addr_of_mut!(rect), -1) != 0 {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect) as LPCRECT, FALSE);
    }

    if MONTHCAL_GetDayRect(infoPtr, today, core::ptr::addr_of_mut!(rect), -1) != 0 {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect) as LPCRECT, FALSE);
    }

    (*infoPtr).todaysDate = *today;

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!((*infoPtr).todayrect) as LPCRECT, FALSE);
    return TRUE;
}

unsafe extern "C" fn MONTHCAL_SetToday(infoPtr: *mut MONTHCAL_INFO, today: *const SYSTEMTIME) -> LRESULT {
    if !today.is_null() {
        if MONTHCAL_UpdateToday(infoPtr, today) != 0 {
            (*infoPtr).todaySet = TRUE;
        }
    }
    return 0;
}

unsafe extern "C" fn MONTHCAL_GetCalendarFromPoint(infoPtr: *const MONTHCAL_INFO, pt: *const POINT) -> INT {
    let mut r: RECT = core::mem::zeroed();
    let mut i: INT = 0;
    i = 0 as INT;
    while i < MONTHCAL_GetCalCount(infoPtr) {
        r.left = (*(*infoPtr).calendars.offset(i as isize)).title.left;
        r.top = (*(*infoPtr).calendars.offset(i as isize)).title.top;
        r.bottom = (*(*infoPtr).calendars.offset(i as isize)).days.bottom;
        r.right = (*(*infoPtr).calendars.offset(i as isize)).days.right;
        if PtInRect(core::ptr::addr_of_mut!(r) as LPCRECT, *pt) != 0 {
            return i;
        }
        i += 1;
    }
    return -(1 as INT);
}

unsafe extern "C" fn fill_hittest_info(
    mut src: *const MCHITTESTINFO,
    mut dest: *mut MCHITTESTINFO,
) -> UINT {
    (*dest).uHit = (*src).uHit;
    (*dest).st = (*src).st;
    if (*dest).cbSize as usize == core::mem::size_of::<MCHITTESTINFO>() {
        memcpy(
            core::ptr::addr_of_mut!((*dest).rc) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*src).rc) as *const core::ffi::c_void,
            core::mem::size_of::<MCHITTESTINFO>() - MCHITTESTINFO_V1_SIZE as usize,
        );
    }
    return (*src).uHit;
}

unsafe extern "C" fn MONTHCAL_HitTest(infoPtr: *const MONTHCAL_INFO, lpht: *mut MCHITTESTINFO) -> LRESULT {
    let mut htinfo: MCHITTESTINFO = core::mem::zeroed();
    let mut ht_month: *mut SYSTEMTIME = core::ptr::null_mut();
    let mut day: INT = 0;
    let mut calIdx: INT = 0;

    if lpht.is_null() || (*lpht).cbSize < MCHITTESTINFO_V1_SIZE {
        return -(1) as LRESULT;
    }

    htinfo.st = st_null;

    /* we should preserve passed fields if hit area doesn't need them */
    if (*lpht).cbSize == core::mem::size_of::<MCHITTESTINFO>() as UINT {
        memcpy(
            core::ptr::addr_of_mut!(htinfo.rc) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*lpht).rc) as *const core::ffi::c_void,
            (core::mem::size_of::<MCHITTESTINFO>() - (MCHITTESTINFO_V1_SIZE as usize)) as usize,
        );
    }

    /* guess in what calendar we are */
    calIdx = MONTHCAL_GetCalendarFromPoint(infoPtr, core::ptr::addr_of!((*lpht).pt));
    if calIdx == -(1) {
        if PtInRect(core::ptr::addr_of!((*infoPtr).todayrect), (*lpht).pt) != 0 {
            htinfo.uHit = MCHT_TODAYLINK as UINT;
            htinfo.rc = (*infoPtr).todayrect;
        } else {
            /* outside of calendar area? What's left must be background :-) */
            htinfo.uHit = MCHT_CALENDARBK as UINT;
        }

        return fill_hittest_info(core::ptr::addr_of_mut!(htinfo), lpht) as LRESULT;
    }

    /* are we in the header? */
    if PtInRect(
        core::ptr::addr_of!((*(*infoPtr).calendars.offset(calIdx as isize)).title),
        (*lpht).pt,
    ) != 0
    {
        /* FIXME: buttons hittesting could be optimized cause maximum
                  two calendars have buttons */
        if calIdx == 0
            && PtInRect(core::ptr::addr_of!((*infoPtr).titlebtnprev), (*lpht).pt) != 0
        {
            htinfo.uHit = MCHT_TITLEBTNPREV as UINT;
            htinfo.rc = (*infoPtr).titlebtnprev;
        } else if PtInRect(core::ptr::addr_of!((*infoPtr).titlebtnnext), (*lpht).pt) != 0
        {
            htinfo.uHit = MCHT_TITLEBTNNEXT as UINT;
            htinfo.rc = (*infoPtr).titlebtnnext;
        } else if PtInRect(
            core::ptr::addr_of!((*(*infoPtr).calendars.offset(calIdx as isize)).titlemonth),
            (*lpht).pt,
        ) != 0
        {
            htinfo.uHit = MCHT_TITLEMONTH as UINT;
            htinfo.rc = (*(*infoPtr).calendars.offset(calIdx as isize)).titlemonth;
            htinfo.iOffset = calIdx;
        } else if PtInRect(
            core::ptr::addr_of!((*(*infoPtr).calendars.offset(calIdx as isize)).titleyear),
            (*lpht).pt,
        ) != 0
        {
            htinfo.uHit = MCHT_TITLEYEAR as UINT;
            htinfo.rc = (*(*infoPtr).calendars.offset(calIdx as isize)).titleyear;
            htinfo.iOffset = calIdx;
        } else {
            htinfo.uHit = MCHT_TITLE as UINT;
            htinfo.rc = (*(*infoPtr).calendars.offset(calIdx as isize)).title;
            htinfo.iOffset = calIdx;
        }

        return fill_hittest_info(core::ptr::addr_of_mut!(htinfo), lpht) as LRESULT;
    }

    ht_month = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(calIdx as isize)).month);
    /* days area (including week days and week numbers) */
    day = MONTHCAL_GetDayFromPos(infoPtr, (*lpht).pt, calIdx);
    if PtInRect(
        core::ptr::addr_of!((*(*infoPtr).calendars.offset(calIdx as isize)).wdays),
        (*lpht).pt,
    ) != 0
    {
        htinfo.uHit = MCHT_CALENDARDAY as UINT;
        htinfo.iOffset = calIdx;
        htinfo.st.wYear = (*ht_month).wYear;
        htinfo.st.wMonth = (if day < 1 {
            (*ht_month).wMonth as INT - 1
        } else {
            (*ht_month).wMonth as INT
        }) as WORD;
        htinfo.st.wDay = (if day < 1 {
            MONTHCAL_MonthLength(
                (*ht_month).wMonth as INT - 1,
                (*ht_month).wYear as INT,
            ) - day
        } else {
            day
        }) as WORD;

        MONTHCAL_GetDayPos(
            infoPtr,
            core::ptr::addr_of_mut!(htinfo.st),
            core::ptr::addr_of_mut!(htinfo.iCol),
            core::ptr::addr_of_mut!(htinfo.iRow),
            calIdx,
        );
    } else if PtInRect(
        core::ptr::addr_of!((*(*infoPtr).calendars.offset(calIdx as isize)).weeknums),
        (*lpht).pt,
    ) != 0
    {
        htinfo.uHit = MCHT_CALENDARWEEKNUM as UINT;
        htinfo.st.wYear = (*ht_month).wYear;
        htinfo.iOffset = calIdx;

        if day < 1 {
            htinfo.st.wMonth = ((*ht_month).wMonth as INT - 1) as WORD;
            htinfo.st.wDay = (MONTHCAL_MonthLength(
                (*ht_month).wMonth as INT - 1,
                (*ht_month).wYear as INT,
            ) - day) as WORD;
        } else if day > MONTHCAL_MonthLength((*ht_month).wMonth as INT, (*ht_month).wYear as INT) {
            htinfo.st.wMonth = ((*ht_month).wMonth as INT + 1) as WORD;
            htinfo.st.wDay = (day
                - MONTHCAL_MonthLength((*ht_month).wMonth as INT, (*ht_month).wYear as INT))
                as WORD;
        } else {
            htinfo.st.wMonth = (*ht_month).wMonth;
            htinfo.st.wDay = day as WORD;
        }
    } else if PtInRect(
        core::ptr::addr_of!((*(*infoPtr).calendars.offset(calIdx as isize)).days),
        (*lpht).pt,
    ) != 0
    {
        htinfo.iOffset = calIdx;
        htinfo.st.wDay = (*ht_month).wDay;
        htinfo.st.wYear = (*ht_month).wYear;
        htinfo.st.wMonth = (*ht_month).wMonth;
        /* previous month only valid for first calendar */
        if day < 1 && calIdx == 0 {
            htinfo.uHit = MCHT_CALENDARDATEPREV as UINT;
            MONTHCAL_GetPrevMonth(core::ptr::addr_of_mut!(htinfo.st));
            htinfo.st.wDay = (MONTHCAL_MonthLength(
                htinfo.st.wMonth as INT,
                htinfo.st.wYear as INT,
            ) + day) as WORD;
        }
        /* next month only valid for last calendar */
        else if day > MONTHCAL_MonthLength((*ht_month).wMonth as INT, (*ht_month).wYear as INT)
            && calIdx == MONTHCAL_GetCalCount(infoPtr) - 1
        {
            htinfo.uHit = MCHT_CALENDARDATENEXT as UINT;
            MONTHCAL_GetNextMonth(core::ptr::addr_of_mut!(htinfo.st));
            htinfo.st.wDay = (day
                - MONTHCAL_MonthLength((*ht_month).wMonth as INT, (*ht_month).wYear as INT))
                as WORD;
        }
        /* multiple calendars case - blank areas for previous/next month */
        else if day < 1
            || day > MONTHCAL_MonthLength((*ht_month).wMonth as INT, (*ht_month).wYear as INT)
        {
            htinfo.uHit = MCHT_CALENDARBK as UINT;
        } else {
            htinfo.uHit = MCHT_CALENDARDATE as UINT;
            htinfo.st.wDay = day as WORD;
        }

        MONTHCAL_GetDayPos(
            infoPtr,
            core::ptr::addr_of_mut!(htinfo.st),
            core::ptr::addr_of_mut!(htinfo.iCol),
            core::ptr::addr_of_mut!(htinfo.iRow),
            calIdx,
        );
        MONTHCAL_GetDayRectI(
            infoPtr,
            core::ptr::addr_of_mut!(htinfo.rc),
            htinfo.iCol as INT,
            htinfo.iRow as INT,
            calIdx,
        );
        /* always update day of week */
        MONTHCAL_CalculateDayOfWeek(core::ptr::addr_of_mut!(htinfo.st), TRUE);
    }

    return fill_hittest_info(core::ptr::addr_of_mut!(htinfo), lpht) as LRESULT;
}

unsafe extern "C" fn MONTHCAL_NotifyDayState(infoPtr: *mut MONTHCAL_INFO) {
    let mut state: *mut MONTHDAYSTATE = core::ptr::null_mut();
    let mut nmds: NMDAYSTATE = core::mem::zeroed();

    if ((*infoPtr).dwStyle & (MCS_DAYSTATE as DWORD)) == 0 {
        return;
    }

    nmds.nmhdr.hwndFrom = (*infoPtr).hwndSelf;
    nmds.nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
    nmds.nmhdr.code = MCN_GETDAYSTATE as UINT;
    nmds.cDayState = MONTHCAL_GetMonthRange(infoPtr, GMR_DAYSTATE as DWORD, core::ptr::null_mut()) as ::core::ffi::c_int;
    state = heap_alloc_zero(((nmds.cDayState as usize).wrapping_mul(core::mem::size_of::<MONTHDAYSTATE>())) as SIZE_T) as *mut MONTHDAYSTATE;
    nmds.prgDayState = state as LPMONTHDAYSTATE;

    MONTHCAL_GetMinDate(infoPtr, core::ptr::addr_of_mut!(nmds.stStart));
    nmds.stStart.wDay = 1 as WORD;

    SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nmds.nmhdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmds) as LPARAM,
    );
    memcpy(
        (*infoPtr).monthdayState as *mut ::core::ffi::c_void,
        nmds.prgDayState as *const ::core::ffi::c_void,
        (MONTHCAL_GetMonthRange(infoPtr, GMR_DAYSTATE as DWORD, core::ptr::null_mut()) as usize)
            .wrapping_mul(core::mem::size_of::<MONTHDAYSTATE>()),
    );

    heap_free(state as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn MONTHCAL_Scroll(infoPtr: *mut MONTHCAL_INFO, delta: INT, keep_selection: BOOL) {
    let mut i: INT = 0;
    let mut selIdx: INT = -1;

    i = 0;
    while i < MONTHCAL_GetCalCount(infoPtr) {
        /* save selection position to shift it later */
        if selIdx == -1
            && MONTHCAL_CompareMonths(
                core::ptr::addr_of_mut!((*infoPtr).minSel),
                core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(i as isize)).month),
            ) == 0
        {
            selIdx = i;
        }

        MONTHCAL_GetMonth(
            core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(i as isize)).month),
            delta,
        );
        i += 1;
    }

    if keep_selection != 0 {
        return;
    }

    /* selection is always shifted to first calendar */
    if (*infoPtr).dwStyle & MCS_MULTISELECT != 0 {
        let mut range: [SYSTEMTIME; 2] = core::mem::zeroed();

        MONTHCAL_GetSelRange(infoPtr, core::ptr::addr_of_mut!(range) as *mut SYSTEMTIME);
        MONTHCAL_GetMonth(
            core::ptr::addr_of_mut!(range[0]) as *mut SYSTEMTIME,
            delta - selIdx,
        );
        MONTHCAL_GetMonth(
            core::ptr::addr_of_mut!(range[1]) as *mut SYSTEMTIME,
            delta - selIdx,
        );
        MONTHCAL_SetSelRange(infoPtr, core::ptr::addr_of_mut!(range) as *mut SYSTEMTIME);
    } else {
        let mut st: SYSTEMTIME = (*infoPtr).minSel;

        MONTHCAL_GetMonth(core::ptr::addr_of_mut!(st), delta - selIdx);
        MONTHCAL_SetCurSel(infoPtr, core::ptr::addr_of_mut!(st));
    }
}

unsafe extern "C" fn MONTHCAL_GoToMonth(infoPtr: *mut MONTHCAL_INFO, direction: nav_direction) {
    let mut delta: INT = if (*infoPtr).delta != 0 {
        (*infoPtr).delta
    } else {
        MONTHCAL_GetCalCount(infoPtr)
    };
    let mut keep_selection: BOOL;
    let mut st: SYSTEMTIME = core::mem::zeroed();

    /* check if change allowed by range set */
    if direction as u32 == DIRECTION_BACKWARD as u32 {
        st = (*(*infoPtr).calendars.offset(0 as isize)).month;
        MONTHCAL_GetMonth(core::ptr::addr_of_mut!(st), -delta);
    } else {
        st = (*(*infoPtr).calendars.offset(
            (MONTHCAL_GetCalCount(infoPtr) as INT - 1 as INT) as isize,
        ))
        .month;
        MONTHCAL_GetMonth(core::ptr::addr_of_mut!(st), delta);
    }

    if MONTHCAL_IsDateInValidRange(infoPtr, core::ptr::addr_of_mut!(st), FALSE as BOOL) == 0 {
        return;
    }

    keep_selection = ((*infoPtr).dwStyle & MCS_NOSELCHANGEONNAV as DWORD) as BOOL;
    MONTHCAL_Scroll(
        infoPtr,
        if direction as u32 == DIRECTION_BACKWARD as u32 {
            -delta
        } else {
            delta
        },
        keep_selection,
    );
    MONTHCAL_NotifyDayState(infoPtr);
    if keep_selection == 0 {
        MONTHCAL_NotifySelectionChange(infoPtr);
    }
}

unsafe extern "C" fn MONTHCAL_RButtonUp(mut infoPtr: *mut MONTHCAL_INFO, mut lParam: LPARAM) -> LRESULT {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut menupoint: POINT = POINT { x: 0, y: 0 };
    let mut buf: [WCHAR; 32] = [0; 32];

    hMenu = CreatePopupMenu();
    LoadStringW(
        COMCTL32_hModule,
        IDM_GOTODAY,
        core::ptr::addr_of_mut!(buf) as LPWSTR,
        (ARRAY_SIZE!(buf)) as INT,
    );
    AppendMenuW(
        hMenu,
        (MF_STRING | MF_ENABLED) as UINT,
        1 as UINT_PTR,
        core::ptr::addr_of_mut!(buf) as LPCWSTR,
    );
    menupoint.x = LOWORD!(lParam) as i16 as LONG;
    menupoint.y = HIWORD!(lParam) as i16 as LONG;
    ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(menupoint));
    if TrackPopupMenu(
        hMenu,
        (TPM_RIGHTBUTTON | TPM_NONOTIFY | TPM_RETURNCMD) as UINT,
        menupoint.x as core::ffi::c_int,
        menupoint.y as core::ffi::c_int,
        0 as core::ffi::c_int,
        (*infoPtr).hwndSelf,
        core::ptr::null::<RECT>(),
    ) != 0
    {
        if ((*infoPtr).dwStyle & MCS_MULTISELECT) != 0 {
            let mut range: [SYSTEMTIME; 2] = core::mem::zeroed();

            range[1 as usize] = (*infoPtr).todaysDate;
            range[0 as usize] = range[1 as usize];
            MONTHCAL_SetSelRange(infoPtr, core::ptr::addr_of_mut!(range) as *mut SYSTEMTIME);
        } else {
            MONTHCAL_SetCurSel(infoPtr, core::ptr::addr_of_mut!((*infoPtr).todaysDate));
        }

        MONTHCAL_NotifySelectionChange(infoPtr);
        MONTHCAL_NotifySelect(infoPtr);
    }

    return 0;
}

unsafe extern "system" fn EditWndProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let infoPtr: *mut MONTHCAL_INFO = GetWindowLongPtrW(GetParent(hwnd), 0) as *mut MONTHCAL_INFO;

    'tail: {
        if uMsg == WM_GETDLGCODE {
            return (DLGC_WANTARROWS | DLGC_WANTALLKEYS) as LRESULT;
        }
        if uMsg == WM_DESTROY {
            let editProc: WNDPROC = (*infoPtr).EditWndProc;
            (*infoPtr).EditWndProc = None;
            SetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
                ::core::mem::transmute::<WNDPROC, DWORD_PTR>(editProc) as LONG_PTR,
            );
            return CallWindowProcW(editProc, hwnd, uMsg, wParam, lParam);
        }
        if uMsg == WM_KILLFOCUS {
            break 'tail;
        }
        if uMsg == WM_KEYDOWN {
            if (VK_ESCAPE == ((wParam as INT)) as u32) || (VK_RETURN == ((wParam as INT)) as u32) {
                break 'tail;
            }
        }
        return CallWindowProcW((*infoPtr).EditWndProc, hwnd, uMsg, wParam, lParam);
    }

    SendMessageW(
        (*infoPtr).hWndYearUpDown,
        WM_CLOSE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    SendMessageW(
        hwnd,
        WM_CLOSE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    return 0 as LRESULT;
}

unsafe extern "C" fn MONTHCAL_EditYear(infoPtr: *mut MONTHCAL_INFO, calIdx: INT) {
    let rc: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(calIdx as isize)).titleyear);
    let title: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(calIdx as isize)).title);

    (*infoPtr).hWndYearEdit =
        CreateWindowExW(0, (WC_EDITW).as_ptr(), core::ptr::null(), WS_VISIBLE | WS_CHILD | ES_READONLY,
            (*rc).left + 3, ((*title).bottom + (*title).top - (*infoPtr).textHeight) / 2,
            (*rc).right - (*rc).left + 4,
            (*infoPtr).textHeight, (*infoPtr).hwndSelf,
            core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    SendMessageW((*infoPtr).hWndYearEdit, WM_SETFONT, (*infoPtr).hBoldFont as WPARAM, (TRUE) as LPARAM);

    (*infoPtr).hWndYearUpDown =
        CreateWindowExW(0, (UPDOWN_CLASSW).as_ptr(), core::ptr::null(),
            WS_VISIBLE | WS_CHILD | UDS_SETBUDDYINT | UDS_NOTHOUSANDS | UDS_ARROWKEYS,
            (*rc).right + 7, ((*title).bottom + (*title).top - (*infoPtr).textHeight) / 2,
            18, (*infoPtr).textHeight, (*infoPtr).hwndSelf,
            core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    /* attach edit box */
    SendMessageW((*infoPtr).hWndYearUpDown, UDM_SETRANGE, 0,
        MAKELONG!(max_allowed_date.wYear, min_allowed_date.wYear) as LPARAM);
    SendMessageW((*infoPtr).hWndYearUpDown, UDM_SETBUDDY, (*infoPtr).hWndYearEdit as WPARAM, 0);
    SendMessageW((*infoPtr).hWndYearUpDown, UDM_SETPOS, 0, (*(*infoPtr).calendars.offset(calIdx as isize)).month.wYear as LPARAM);

    /* subclass edit box */
    let prevProc: LONG_PTR = SetWindowLongPtrW((*infoPtr).hWndYearEdit,
        GWLP_WNDPROC, ((EditWndProc as usize) as DWORD_PTR) as LONG_PTR);
    (*infoPtr).EditWndProc = core::ptr::read(core::ptr::addr_of!(prevProc) as *const WNDPROC);

    SetFocus((*infoPtr).hWndYearEdit);
}

unsafe extern "C" fn MONTHCAL_LButtonDown(infoPtr: *mut MONTHCAL_INFO, lParam: LPARAM) -> LRESULT {
    let mut ht: MCHITTESTINFO = core::mem::zeroed();
    let mut hit: DWORD = 0;

    /* Actually we don't need input focus for calendar, this is used to kill
       year updown and its buddy edit box */
    if IsWindow((*infoPtr).hWndYearUpDown) != 0 {
        SetFocus((*infoPtr).hwndSelf);
        return 0 as LRESULT;
    }

    SetCapture((*infoPtr).hwndSelf);

    ht.cbSize = core::mem::size_of::<MCHITTESTINFO>() as UINT;
    ht.pt.x = (LOWORD!(lParam as DWORD) as i16) as LONG;
    ht.pt.y = (HIWORD!(lParam as DWORD) as i16) as LONG;

    hit = MONTHCAL_HitTest(infoPtr, core::ptr::addr_of_mut!(ht)) as DWORD;

    match hit {
        MCHT_TITLEBTNNEXT => {
            MONTHCAL_GoToMonth(infoPtr, (DIRECTION_FORWARD as INT) as nav_direction);
            (*infoPtr).status = MC_NEXTPRESSED as INT;
            SetTimer(
                (*infoPtr).hwndSelf,
                MC_PREVNEXTMONTHTIMER as UINT_PTR,
                MC_PREVNEXTMONTHDELAY as UINT,
                None,
            );
            let _ = 0;
            InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), FALSE as BOOL);
            return 0 as LRESULT;
        }

        MCHT_TITLEBTNPREV => {
            MONTHCAL_GoToMonth(infoPtr, (DIRECTION_BACKWARD as INT) as nav_direction);
            (*infoPtr).status = MC_PREVPRESSED as INT;
            SetTimer(
                (*infoPtr).hwndSelf,
                MC_PREVNEXTMONTHTIMER as UINT_PTR,
                MC_PREVNEXTMONTHDELAY as UINT,
                None,
            );
            let _ = 0;
            InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), FALSE as BOOL);
            return 0 as LRESULT;
        }

        MCHT_TITLEMONTH => {
            let mut hMenu: HMENU = CreatePopupMenu();
            let mut buf: [WCHAR; 32 as usize] = core::mem::zeroed();
            let mut menupoint: POINT = core::mem::zeroed();
            let mut i: INT = 0;

            i = 0 as INT;
            while i < 12 as INT {
                GetLocaleInfoW(
                    LOCALE_USER_DEFAULT as DWORD,
                    (LOCALE_SMONTHNAME1 + (i) as u32) as LCTYPE,
                    core::ptr::addr_of_mut!(buf) as LPWSTR,
                    ARRAY_SIZE!(buf) as INT,
                );
                AppendMenuW(
                    hMenu,
                    (MF_STRING | MF_ENABLED) as UINT,
                    (i + 1) as UINT_PTR,
                    core::ptr::addr_of_mut!(buf) as LPCWSTR,
                );
                i += 1;
            }
            menupoint.x = ht.pt.x;
            menupoint.y = ht.pt.y;
            ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(menupoint));
            i = TrackPopupMenu(
                hMenu,
                (TPM_LEFTALIGN | TPM_NONOTIFY | TPM_RIGHTBUTTON | TPM_RETURNCMD) as UINT,
                menupoint.x,
                menupoint.y,
                0 as INT,
                (*infoPtr).hwndSelf,
                core::ptr::null::<RECT>(),
            ) as INT;

            if (i > 0 as INT)
                && (i < 13 as INT)
                && ((*(*infoPtr).calendars.offset(ht.iOffset as isize)).month.wMonth as INT != i)
            {
                let mut delta: INT = i - ((*(*infoPtr).calendars.offset(ht.iOffset as isize)).month.wMonth as INT);
                let mut st: SYSTEMTIME = core::mem::zeroed();

                /* check if change allowed by range set */
                st = if delta < 0 as INT {
                    (*(*infoPtr).calendars.offset(0 as isize)).month
                } else {
                    (*(*infoPtr).calendars.offset(
                        (MONTHCAL_GetCalCount(infoPtr) - 1 as INT) as isize,
                    ))
                    .month
                };
                MONTHCAL_GetMonth(core::ptr::addr_of_mut!(st), delta);

                if MONTHCAL_IsDateInValidRange(infoPtr, core::ptr::addr_of_mut!(st), FALSE as BOOL) != 0 {
                    MONTHCAL_Scroll(infoPtr, delta, FALSE as BOOL);
                    MONTHCAL_NotifyDayState(infoPtr);
                    MONTHCAL_NotifySelectionChange(infoPtr);
                    InvalidateRect(
                        (*infoPtr).hwndSelf,
                        core::ptr::null::<RECT>(),
                        FALSE as BOOL,
                    );
                }
            }
            return 0 as LRESULT;
        }

        MCHT_TITLEYEAR => {
            MONTHCAL_EditYear(infoPtr, ht.iOffset as INT);
            return 0 as LRESULT;
        }

        MCHT_TODAYLINK => {
            if ((*infoPtr).dwStyle & MCS_MULTISELECT as DWORD) != 0 {
                let mut range: [SYSTEMTIME; 2 as usize] = core::mem::zeroed();

                range[1 as usize] = (*infoPtr).todaysDate;
                range[0 as usize] = range[1 as usize];
                MONTHCAL_SetSelRange(infoPtr, core::ptr::addr_of_mut!(range) as *mut SYSTEMTIME);
            } else {
                MONTHCAL_SetCurSel(infoPtr, core::ptr::addr_of_mut!((*infoPtr).todaysDate));
            }

            MONTHCAL_NotifySelectionChange(infoPtr);
            MONTHCAL_NotifySelect(infoPtr);
            return 0 as LRESULT;
        }

        MCHT_CALENDARDATENEXT | MCHT_CALENDARDATEPREV | MCHT_CALENDARDATE => {
            let mut st: [SYSTEMTIME; 2 as usize] = core::mem::zeroed();

            MONTHCAL_CopyDate(
                core::ptr::addr_of_mut!(ht.st),
                core::ptr::addr_of_mut!((*infoPtr).firstSel),
            );

            st[1 as usize] = ht.st;
            st[0 as usize] = st[1 as usize];
            /* clear selection range */
            MONTHCAL_SetSelRange(infoPtr, core::ptr::addr_of_mut!(st) as *mut SYSTEMTIME);

            (*infoPtr).status = MC_SEL_LBUTDOWN as INT;
            MONTHCAL_SetDayFocus(infoPtr, core::ptr::addr_of_mut!(ht.st));
            return 0 as LRESULT;
        }

        _ => {}
    }

    return 1 as LRESULT;
}
unsafe extern "C" fn MONTHCAL_LButtonUp(
    mut infoPtr: *mut MONTHCAL_INFO,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut nmhdr: NMHDR = tagNMHDR {
        hwndFrom: ::core::ptr::null_mut::<HWND__>(),
        idFrom: 0,
        code: 0,
    };
    let mut ht: MCHITTESTINFO = MCHITTESTINFO {
        cbSize: 0,
        pt: tagPOINT { x: 0, y: 0 },
        uHit: 0,
        st: _SYSTEMTIME {
            wYear: 0,
            wMonth: 0,
            wDayOfWeek: 0,
            wDay: 0,
            wHour: 0,
            wMinute: 0,
            wSecond: 0,
            wMilliseconds: 0,
        },
        rc: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        iOffset: 0,
        iRow: 0,
        iCol: 0,
    };
    let mut hit: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_LButtonUp\0" as *const u8 as *const ::core::ffi::c_char,
            2280 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if (*infoPtr).status & (MC_PREVPRESSED | MC_NEXTPRESSED) != 0 {
        let mut r: *mut RECT = ::core::ptr::null_mut::<RECT>();
        KillTimer((*infoPtr).hwndSelf, MC_PREVNEXTMONTHTIMER as UINT_PTR);
        r = if (*infoPtr).status & MC_PREVPRESSED != 0 {
            core::ptr::addr_of_mut!((*infoPtr).titlebtnprev)
        } else {
            core::ptr::addr_of_mut!((*infoPtr).titlebtnnext)
        };
        (*infoPtr).status &= !(MC_PREVPRESSED | MC_NEXTPRESSED);
        InvalidateRect(
            (*infoPtr).hwndSelf,
            r as LPCRECT,
            FALSE as BOOL,
        );
    }
    ReleaseCapture();
    nmhdr.hwndFrom = (*infoPtr).hwndSelf;
    nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
    nmhdr.code = NM_RELEASEDCAPTURE as UINT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_LButtonUp\0" as *const u8 as *const ::core::ffi::c_char,
            2298 as ::core::ffi::c_int,
            b"Sent notification from %p to %p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            (*infoPtr).hwndNotify,
        );
    }
    SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nmhdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmhdr) as LPARAM,
    );
    if (*infoPtr).status & MC_SEL_LBUTDOWN == 0 {
        return 0 as LRESULT;
    }
    ht.cbSize = core::mem::size_of::<MCHITTESTINFO>() as UINT;
    ht.pt.x = (LOWORD!(lParam as DWORD) as ::core::ffi::c_short) as LONG;
    ht.pt.y = (HIWORD!(lParam as DWORD) as ::core::ffi::c_short) as LONG;
    hit = MONTHCAL_HitTest(infoPtr, core::ptr::addr_of_mut!(ht)) as DWORD;
    (*infoPtr).status = MC_SEL_LBUTUP;
    MONTHCAL_SetDayFocus(infoPtr, ::core::ptr::null::<SYSTEMTIME>());
    if (hit & MCHT_CALENDARDATE as DWORD) == MCHT_CALENDARDATE as DWORD {
        let mut sel: SYSTEMTIME = (*infoPtr).minSel;
        MONTHCAL_SetCurSel(infoPtr, core::ptr::addr_of_mut!(ht.st));
        if MONTHCAL_IsDateEqual(
            core::ptr::addr_of_mut!(sel),
            core::ptr::addr_of_mut!(ht.st),
        ) == 0
        {
            MONTHCAL_NotifySelectionChange(infoPtr);
        }
        MONTHCAL_NotifySelect(infoPtr);
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn MONTHCAL_Timer(infoPtr: *mut MONTHCAL_INFO, id: WPARAM) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_Timer\0" as *const u8 as *const ::core::ffi::c_char,
            2333 as ::core::ffi::c_int,
            b"%ld\n\0" as *const u8 as *const ::core::ffi::c_char,
            id,
        );
    }
    if id == MC_PREVNEXTMONTHTIMER as WPARAM {
        if ((*infoPtr).status & MC_NEXTPRESSED) != 0 {
            MONTHCAL_GoToMonth(infoPtr, (DIRECTION_FORWARD) as nav_direction);
        }
        if ((*infoPtr).status & MC_PREVPRESSED) != 0 {
            MONTHCAL_GoToMonth(infoPtr, (DIRECTION_BACKWARD) as nav_direction);
        }
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::null_mut(),
            FALSE,
        );
    } else if id == MC_TODAYUPDATETIMER as WPARAM {
        let mut st: SYSTEMTIME = core::mem::zeroed();

        if (*infoPtr).todaySet != 0 {
            return 0 as LRESULT;
        }

        GetLocalTime(core::ptr::addr_of_mut!(st));
        MONTHCAL_UpdateToday(infoPtr, core::ptr::addr_of_mut!(st));

        /* notification sent anyway */
        MONTHCAL_NotifySelectionChange(infoPtr);

        return 0 as LRESULT;
    } else {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/monthcal.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MONTHCAL_Timer\0" as *const u8 as *const ::core::ffi::c_char,
                2356 as ::core::ffi::c_int,
                b"got unknown timer %ld\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                id,
            );
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn MONTHCAL_MouseMove(infoPtr: *mut MONTHCAL_INFO, lParam: LPARAM) -> LRESULT {
    let mut ht: MCHITTESTINFO = core::mem::zeroed();
    let mut st_ht: SYSTEMTIME = core::mem::zeroed();
    let mut hit: INT = 0;
    let mut r: RECT = core::mem::zeroed();

    if ((*infoPtr).status & MC_SEL_LBUTDOWN) == 0 {
        return 0 as LRESULT;
    }

    ht.cbSize = core::mem::size_of::<MCHITTESTINFO>() as UINT;
    ht.pt.x = (LOWORD!(lParam as DWORD) as ::core::ffi::c_short) as LONG;
    ht.pt.y = (HIWORD!(lParam as DWORD) as ::core::ffi::c_short) as LONG;
    ht.iOffset = -(1);

    hit = MONTHCAL_HitTest(infoPtr, core::ptr::addr_of_mut!(ht)) as INT;

    /* not on the calendar date numbers? bail out */

    if (hit & (MCHT_CALENDARDATE as i32)) != (MCHT_CALENDARDATE as i32) {
        MONTHCAL_SetDayFocus(infoPtr, core::ptr::null_mut());
        return 0 as LRESULT;
    }

    st_ht = ht.st;

    /* if pointer is over focused day still there's nothing to do */
    if MONTHCAL_SetDayFocus(infoPtr, core::ptr::addr_of_mut!(ht.st)) == 0 {
        return 0 as LRESULT;
    }

    MONTHCAL_GetDayRect(
        infoPtr,
        core::ptr::addr_of_mut!(ht.st),
        core::ptr::addr_of_mut!(r),
        ht.iOffset,
    );

    'done: {
        if ((*infoPtr).dwStyle & MCS_MULTISELECT) != 0 {
            let mut st: [SYSTEMTIME; 2] = core::mem::zeroed();

            MONTHCAL_GetSelRange(infoPtr, st.as_mut_ptr());

            /* If we're still at the first selected date and range is empty, return.
               If range isn't empty we should change range to a single firstSel */
            if MONTHCAL_IsDateEqual(
                core::ptr::addr_of_mut!((*infoPtr).firstSel),
                core::ptr::addr_of_mut!(st_ht),
            ) != 0
                && MONTHCAL_IsDateEqual(
                    core::ptr::addr_of_mut!(st[0]),
                    core::ptr::addr_of_mut!(st[1]),
                ) != 0
            {
                break 'done;
            }

            MONTHCAL_IsSelRangeValid(
                infoPtr,
                core::ptr::addr_of_mut!(st_ht),
                core::ptr::addr_of_mut!((*infoPtr).firstSel),
                core::ptr::addr_of_mut!(st_ht),
            );

            st[0] = (*infoPtr).firstSel;
            /* we should overwrite timestamp here */
            MONTHCAL_CopyDate(
                core::ptr::addr_of_mut!(st_ht),
                core::ptr::addr_of_mut!(st[1]),
            );

            /* bounds will be swapped here if needed */
            MONTHCAL_SetSelRange(infoPtr, st.as_mut_ptr());

            return 0 as LRESULT;
        }
    }

    /* FIXME: this should specify a rectangle containing only the days that changed
       using InvalidateRect */
    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null::<RECT>(),
        FALSE,
    );

    return 0 as LRESULT;
}

unsafe extern "C" fn MONTHCAL_Paint(infoPtr: *mut MONTHCAL_INFO, hdc_paint: HDC) -> LRESULT {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();

    if !hdc_paint.is_null() {
        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps.rcPaint));
        hdc = hdc_paint;
    } else {
        hdc = BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }

    MONTHCAL_Refresh(infoPtr, hdc, core::ptr::addr_of_mut!(ps));
    if hdc_paint.is_null() {
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }
    return 0;
}

unsafe extern "C" fn MONTHCAL_EraseBkgnd(infoPtr: *const MONTHCAL_INFO, hdc: HDC) -> LRESULT {
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if GetClipBox(hdc, core::ptr::addr_of_mut!(rc)) == 0 {
        return FALSE as LRESULT;
    }

    FillRect(
        hdc,
        core::ptr::addr_of_mut!(rc),
        (*infoPtr).brushes[BrushBackground as usize],
    );

    return TRUE as LRESULT;
}

unsafe extern "C" fn MONTHCAL_PrintClient(infoPtr: *mut MONTHCAL_INFO, hdc: HDC, options: DWORD) -> LRESULT {
    if (options & (PRF_CHECKVISIBLE as DWORD)) != 0 && IsWindowVisible((*infoPtr).hwndSelf) == 0 {
        return 0 as LRESULT;
    }

    if (options & (PRF_ERASEBKGND as DWORD)) != 0 {
        MONTHCAL_EraseBkgnd(infoPtr, hdc);
    }

    if (options & (PRF_CLIENT as DWORD)) != 0 {
        MONTHCAL_Paint(infoPtr, hdc);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn MONTHCAL_SetFocus(infoPtr: *const MONTHCAL_INFO) -> LRESULT {
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), FALSE);

    return 0;
}

unsafe extern "C" fn MONTHCAL_UpdateSize(infoPtr: *mut MONTHCAL_INFO) {
    static O0W: [WCHAR; 3] = [0x0030u16, 0x0030, 0x0000];
    let mut title: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).title);
    let mut prev: *mut RECT = core::ptr::addr_of_mut!((*infoPtr).titlebtnprev);
    let mut next: *mut RECT = core::ptr::addr_of_mut!((*infoPtr).titlebtnnext);
    let mut titlemonth: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).titlemonth);
    let mut titleyear: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).titleyear);
    let mut wdays: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).wdays);
    let mut weeknumrect: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).weeknums);
    let mut days: *mut RECT = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).days);
    let mut todayrect: *mut RECT = core::ptr::addr_of_mut!((*infoPtr).todayrect);

    let mut xdiv: INT = 0;
    let mut dx: INT = 0;
    let mut dy: INT = 0;
    let mut i: INT = 0;
    let mut j: INT = 0;
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut c_dx: INT = 0;
    let mut c_dy: INT = 0;
    let mut buff: [WCHAR; 80 as usize] = [0; 80 as usize];
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut day_width: INT = 0;
    let mut client: RECT = core::mem::zeroed();
    let mut font: HFONT = core::ptr::null_mut();
    let mut size: SIZE = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(client));

    hdc = GetDC((*infoPtr).hwndSelf);
    font = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    /* get the height and width of each day's text */
    GetTextMetricsW(hdc, core::ptr::addr_of_mut!(tm));
    (*infoPtr).textHeight = tm.tmHeight + tm.tmExternalLeading + tm.tmInternalLeading;

    /* find widest day name for current locale and font */
    day_width = 0;
    i = 0;
    while i < 7 {
        let mut sz: SIZE = core::mem::zeroed();

        if get_localized_dayname(infoPtr, i as core::ffi::c_uint, core::ptr::addr_of_mut!(buff) as *mut WCHAR, ARRAY_SIZE!(buff) as core::ffi::c_uint) != 0 {
            GetTextExtentPoint32W(hdc, core::ptr::addr_of_mut!(buff) as *mut WCHAR as LPCWSTR, lstrlenW(core::ptr::addr_of_mut!(buff) as *mut WCHAR as LPCWSTR), core::ptr::addr_of_mut!(sz));
            if sz.cx > day_width {
                day_width = sz.cx;
            }
            i += 1;
        } else {
            /* locale independent fallback on failure */
            static sunW: [WCHAR; 3] = [0x0053u16, 0x0075, 0x006E];
            GetTextExtentPoint32W(hdc, core::ptr::addr_of!(sunW) as LPCWSTR, ARRAY_SIZE!(sunW) as INT, core::ptr::addr_of_mut!(sz));
            day_width = sz.cx;
            break;
        }
    }

    day_width += 2;

    /* recalculate the height and width increments and offsets */
    size.cx = 0;
    GetTextExtentPoint32W(hdc, &raw const O0W as LPCWSTR, 2, core::ptr::addr_of_mut!(size));

    /* restore the originally selected font */
    SelectObject(hdc, font as HGDIOBJ);
    ReleaseDC((*infoPtr).hwndSelf, hdc);

    xdiv = if ((*infoPtr).dwStyle & MCS_WEEKNUMBERS) != 0 { 8 } else { 7 };

    (*infoPtr).width_increment = max(day_width, size.cx * 2 + 4);
    (*infoPtr).height_increment = (*infoPtr).textHeight;

    /* calculate title area */
    (*title).top = 0;
    (*title).bottom = 3 * (*infoPtr).height_increment / 2;
    (*title).left = 0;
    (*title).right = (*infoPtr).width_increment * xdiv;

    /* set the dimensions of the next and previous buttons and center */
    /* the month text vertically */
    (*next).top = (*title).top + 4;
    (*prev).top = (*next).top;
    (*next).bottom = (*title).bottom - 4;
    (*prev).bottom = (*next).bottom;
    (*prev).left = (*title).left + 4;
    (*prev).right = (*prev).left + ((*title).bottom - (*title).top);
    (*next).right = (*title).right - 4;
    (*next).left = (*next).right - ((*title).bottom - (*title).top);

    /* titlemonth->left and right change based upon the current month
       and are recalculated in refresh as the current month may change
       without the control being resized */
    (*titleyear).top = (*title).top + ((*infoPtr).height_increment) / 2;
    (*titlemonth).top = (*titleyear).top;
    (*titleyear).bottom = (*title).bottom - ((*infoPtr).height_increment) / 2;
    (*titlemonth).bottom = (*titleyear).bottom;

    /* week numbers */
    (*weeknumrect).left = 0;
    (*weeknumrect).right = if ((*infoPtr).dwStyle & MCS_WEEKNUMBERS) != 0 { (*prev).right } else { 0 };

    /* days abbreviated names */
    (*days).left = (*weeknumrect).right;
    (*wdays).left = (*days).left;
    (*days).right = (*wdays).left + 7 * (*infoPtr).width_increment;
    (*wdays).right = (*days).right;
    (*wdays).top = (*title).bottom;
    (*wdays).bottom = (*wdays).top + (*infoPtr).height_increment;

    (*days).top = (*wdays).bottom;
    (*weeknumrect).top = (*wdays).bottom;
    (*days).bottom = (*days).top + 6 * (*infoPtr).height_increment;
    (*weeknumrect).bottom = (*days).bottom;

    (*todayrect).left = 0;
    (*todayrect).right = (*title).right;
    (*todayrect).top = (*days).bottom;
    (*todayrect).bottom = (*days).bottom + (*infoPtr).height_increment;

    /* compute calendar count, update all calendars */
    x = (client.right + MC_CALENDAR_PADDING) / ((*title).right - (*title).left + MC_CALENDAR_PADDING);
    /* today label affects whole height */
    if ((*infoPtr).dwStyle & MCS_NOTODAY) != 0 {
        y = (client.bottom + MC_CALENDAR_PADDING) / ((*days).bottom - (*title).top + MC_CALENDAR_PADDING);
    } else {
        y = (client.bottom - (*todayrect).bottom + (*todayrect).top + MC_CALENDAR_PADDING)
            / ((*days).bottom - (*title).top + MC_CALENDAR_PADDING);
    }

    /* TODO: ensure that count is properly adjusted to fit 12 months constraint */
    if x == 0 {
        x = 1;
    }
    if y == 0 {
        y = 1;
    }

    if x * y != MONTHCAL_GetCalCount(infoPtr) {
        (*infoPtr).dim.cx = x;
        (*infoPtr).dim.cy = y;
        (*infoPtr).calendars = heap_realloc(
            (*infoPtr).calendars as *mut core::ffi::c_void,
            (MONTHCAL_GetCalCount(infoPtr) as SIZE_T) * (core::mem::size_of::<CALENDAR_INFO>() as SIZE_T),
        ) as *mut CALENDAR_INFO;

        (*infoPtr).monthdayState = heap_realloc(
            (*infoPtr).monthdayState as *mut core::ffi::c_void,
            (MONTHCAL_GetMonthRange(infoPtr, GMR_DAYSTATE, core::ptr::null_mut()) as SIZE_T)
                * (core::mem::size_of::<MONTHDAYSTATE>() as SIZE_T),
        ) as *mut MONTHDAYSTATE;
        MONTHCAL_NotifyDayState(infoPtr);

        /* update pointers that we'll need */
        title = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).title);
        wdays = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).wdays);
        days = core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(0 as isize)).days);
    }

    i = 1;
    while i < MONTHCAL_GetCalCount(infoPtr) {
        /* set months */
        *(*infoPtr).calendars.offset(i as isize) = *(*infoPtr).calendars.offset(0 as isize);
        MONTHCAL_GetMonth(core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset(i as isize)).month), i);
        i += 1;
    }

    /* offset all rectangles to center in client area */
    c_dx = (client.right - x * (*title).right - MC_CALENDAR_PADDING * (x - 1)) / 2;
    c_dy = (client.bottom - y * (*todayrect).bottom - MC_CALENDAR_PADDING * (y - 1)) / 2;

    /* if calendar doesn't fit client area show it at left/top bounds */
    if (*title).left + c_dx < 0 {
        c_dx = 0;
    }
    if (*title).top + c_dy < 0 {
        c_dy = 0;
    }

    i = 0;
    while i < y {
        j = 0;
        while j < x {
            dx = j * ((*title).right - (*title).left + MC_CALENDAR_PADDING) + c_dx;
            dy = i * ((*days).bottom - (*title).top + MC_CALENDAR_PADDING) + c_dy;

            OffsetRect(core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset((i * x + j) as isize)).title), dx, dy);
            OffsetRect(core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset((i * x + j) as isize)).titlemonth), dx, dy);
            OffsetRect(core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset((i * x + j) as isize)).titleyear), dx, dy);
            OffsetRect(core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset((i * x + j) as isize)).wdays), dx, dy);
            OffsetRect(core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset((i * x + j) as isize)).weeknums), dx, dy);
            OffsetRect(core::ptr::addr_of_mut!((*(*infoPtr).calendars.offset((i * x + j) as isize)).days), dx, dy);
            j += 1;
        }
        i += 1;
    }

    OffsetRect(prev as LPRECT, c_dx, c_dy);
    OffsetRect(
        next as LPRECT,
        (x - 1) * ((*title).right - (*title).left + MC_CALENDAR_PADDING) + c_dx,
        c_dy,
    );

    i = (*infoPtr).dim.cx * (*infoPtr).dim.cy - (*infoPtr).dim.cx;
    (*todayrect).left = (*(*infoPtr).calendars.offset(i as isize)).title.left;
    (*todayrect).right = (*(*infoPtr).calendars.offset(i as isize)).title.right;
    (*todayrect).top = (*(*infoPtr).calendars.offset(i as isize)).days.bottom;
    (*todayrect).bottom = (*(*infoPtr).calendars.offset(i as isize)).days.bottom + (*infoPtr).height_increment;

    if ((*__wine_dbch___default).flags as core::ffi::c_int & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/monthcal.c\0".as_ptr() as *const core::ffi::c_char,
            b"MONTHCAL_UpdateSize\0".as_ptr() as *const core::ffi::c_char,
            2667 as core::ffi::c_int,
            b"dx=%d dy=%d client[%s] title[%s] wdays[%s] days[%s] today[%s]\n\0".as_ptr() as *const core::ffi::c_char,
            (*infoPtr).width_increment,
            (*infoPtr).height_increment,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(client)),
            wine_dbgstr_rect(title),
            wine_dbgstr_rect(wdays),
            wine_dbgstr_rect(days),
            wine_dbgstr_rect(todayrect),
        );
    }
}

unsafe extern "C" fn MONTHCAL_Size(infoPtr: *mut MONTHCAL_INFO, Width: i32, Height: i32) -> LRESULT {
    MONTHCAL_UpdateSize(infoPtr);
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);

    return 0;
}

unsafe extern "C" fn MONTHCAL_GetFont(infoPtr: *const MONTHCAL_INFO) -> LRESULT {
    return (*infoPtr).hFont as LRESULT;
}

unsafe extern "C" fn MONTHCAL_SetFont(infoPtr: *mut MONTHCAL_INFO, hFont: HFONT, redraw: BOOL) -> LRESULT {
    let mut hOldFont: HFONT;
    let mut lf: LOGFONTW = tagLOGFONTW {
        lfHeight: 0,
        lfWidth: 0,
        lfEscapement: 0,
        lfOrientation: 0,
        lfWeight: 0,
        lfItalic: 0,
        lfUnderline: 0,
        lfStrikeOut: 0,
        lfCharSet: 0,
        lfOutPrecision: 0,
        lfClipPrecision: 0,
        lfQuality: 0,
        lfPitchAndFamily: 0,
        lfFaceName: [0; 32],
    };

    if hFont.is_null() {
        return 0 as LRESULT;
    }

    hOldFont = (*infoPtr).hFont;
    (*infoPtr).hFont = hFont;

    GetObjectW(
        (*infoPtr).hFont as HANDLE,
        ::core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
        &raw mut lf as LPVOID,
    );
    lf.lfWeight = FW_BOLD as LONG;
    (*infoPtr).hBoldFont = CreateFontIndirectW(&raw mut lf);

    MONTHCAL_UpdateSize(infoPtr);

    if redraw != 0 {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            ::core::ptr::null::<RECT>(),
            FALSE,
        );
    }

    return hOldFont as LRESULT;
}

unsafe extern "C" fn theme_changed(infoPtr: *const MONTHCAL_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    CloseThemeData(theme);
    OpenThemeData((*infoPtr).hwndSelf, core::ptr::addr_of!(themeClass) as LPCWSTR);
    return 0;
}

unsafe extern "C" fn MONTHCAL_StyleChanged(mut infoPtr: *mut MONTHCAL_INFO, mut wStyleType: WPARAM, mut lpss: *const STYLESTRUCT) -> INT {
    if wStyleType != GWL_STYLE as WPARAM {
        return 0 as INT;
    }
    (*infoPtr).dwStyle = (*lpss).styleNew;
    /* make room for week numbers */
    if (((*lpss).styleNew ^ (*lpss).styleOld) & ((MCS_WEEKNUMBERS | MCS_SHORTDAYSOFWEEK) as DWORD)) != 0 {
        MONTHCAL_UpdateSize(infoPtr);
    }
    return 0 as INT;
}

unsafe extern "C" fn MONTHCAL_StyleChanging(
    mut infoPtr: *mut MONTHCAL_INFO,
    mut wStyleType: WPARAM,
    mut lpss: *mut STYLESTRUCT,
) -> INT {
    /* block MCS_MULTISELECT change */
    if ((*lpss).styleNew ^ (*lpss).styleOld) & MCS_MULTISELECT as DWORD != 0 {
        if (*lpss).styleOld & MCS_MULTISELECT as DWORD != 0 {
            (*lpss).styleNew |= MCS_MULTISELECT as DWORD;
        } else {
            (*lpss).styleNew &= !MCS_MULTISELECT as DWORD;
        }
    }

    /* block MCS_DAYSTATE change */
    if ((*lpss).styleNew ^ (*lpss).styleOld) & MCS_DAYSTATE as DWORD != 0 {
        if (*lpss).styleOld & MCS_DAYSTATE as DWORD != 0 {
            (*lpss).styleNew |= MCS_DAYSTATE as DWORD;
        } else {
            (*lpss).styleNew &= !MCS_DAYSTATE as DWORD;
        }
    }

    return 0 as INT;
}

unsafe extern "C" fn MONTHCAL_Create(hwnd: HWND, lpcs: LPCREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut MONTHCAL_INFO = core::ptr::null_mut::<MONTHCAL_INFO>();

    /* allocate memory for info structure */
    infoPtr = heap_alloc_zero(core::mem::size_of::<MONTHCAL_INFO>() as SIZE_T) as *mut MONTHCAL_INFO;
    SetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int, infoPtr as DWORD_PTR as LONG_PTR);

    if infoPtr.is_null() {
        return 0 as LRESULT;
    }

    'fail: {
        (*infoPtr).hwndSelf = hwnd;
        (*infoPtr).hwndNotify = (*lpcs).hwndParent;
        (*infoPtr).dwStyle = GetWindowLongW(hwnd, GWL_STYLE as ::core::ffi::c_int) as DWORD;
        (*infoPtr).dim.cy = 1 as LONG;
        (*infoPtr).dim.cx = (*infoPtr).dim.cy;
        (*infoPtr).calendars = heap_alloc_zero(
            core::mem::size_of::<CALENDAR_INFO>() as SIZE_T,
        ) as *mut CALENDAR_INFO;
        if (*infoPtr).calendars.is_null() {
            break 'fail;
        }
        (*infoPtr).monthdayState = heap_alloc_zero(
            (3 as SIZE_T).wrapping_mul(core::mem::size_of::<MONTHDAYSTATE>() as SIZE_T),
        ) as *mut MONTHDAYSTATE;
        if (*infoPtr).monthdayState.is_null() {
            break 'fail;
        }

        /* initialize info structure */
        /* FIXME: calculate systemtime ->> localtime(subtract timezoneinfo) */

        GetLocalTime(&raw mut (*infoPtr).todaysDate);
        MONTHCAL_SetFirstDayOfWeek(infoPtr, -(1 as INT));

        (*infoPtr).maxSelCount = (if (*infoPtr).dwStyle & MCS_MULTISELECT as DWORD != 0 {
            7 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as INT;

        (*infoPtr).colors[MCSC_BACKGROUND as usize] = comctl32_color.clrWindow;
        (*infoPtr).colors[MCSC_TEXT as usize] = comctl32_color.clrWindowText;
        (*infoPtr).colors[MCSC_TITLEBK as usize] = comctl32_color.clrActiveCaption;
        (*infoPtr).colors[MCSC_TITLETEXT as usize] = comctl32_color.clrWindow;
        (*infoPtr).colors[MCSC_MONTHBK as usize] = comctl32_color.clrWindow;
        (*infoPtr).colors[MCSC_TRAILINGTEXT as usize] = comctl32_color.clrGrayText;

        (*infoPtr).brushes[BrushBackground as usize] =
            CreateSolidBrush((*infoPtr).colors[MCSC_BACKGROUND as usize]);
        (*infoPtr).brushes[BrushTitle as usize] =
            CreateSolidBrush((*infoPtr).colors[MCSC_TITLEBK as usize]);
        (*infoPtr).brushes[BrushMonth as usize] =
            CreateSolidBrush((*infoPtr).colors[MCSC_MONTHBK as usize]);

        (*infoPtr).pens[PenRed as usize] = CreatePen(
            PS_SOLID as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            RGB!(255, 0, 0),
        );
        (*infoPtr).pens[PenText as usize] = CreatePen(
            PS_SOLID as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            (*infoPtr).colors[MCSC_TEXT as usize],
        );

        (*infoPtr).minSel = (*infoPtr).todaysDate;
        (*infoPtr).maxSel = (*infoPtr).todaysDate;
        (*(*infoPtr).calendars.offset(0 as isize)).month = (*infoPtr).todaysDate;
        (*infoPtr).isUnicode = TRUE as BOOL;

        /* setup control layout and day state data */
        MONTHCAL_UpdateSize(infoPtr);

        /* today auto update timer, to be freed only on control destruction */
        SetTimer(
            (*infoPtr).hwndSelf,
            MC_TODAYUPDATETIMER as UINT_PTR,
            MC_TODAYUPDATEDELAY as UINT,
            None,
        );

        OpenThemeData((*infoPtr).hwndSelf, &raw const themeClass as LPCWSTR);

        return 0 as LRESULT;
    }

    /* fail: */
    heap_free((*infoPtr).monthdayState as *mut ::core::ffi::c_void);
    heap_free((*infoPtr).calendars as *mut ::core::ffi::c_void);
    heap_free(infoPtr as *mut ::core::ffi::c_void);
    return 0 as LRESULT;
}

unsafe extern "C" fn MONTHCAL_Destroy(infoPtr: *mut MONTHCAL_INFO) -> LRESULT {
    let mut i: INT;

    /* free month calendar info data */
    heap_free((*infoPtr).monthdayState as *mut core::ffi::c_void);
    heap_free((*infoPtr).calendars as *mut core::ffi::c_void);
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);

    CloseThemeData(GetWindowTheme((*infoPtr).hwndSelf));

    i = 0;
    while i < BrushLast as INT {
        DeleteObject((*infoPtr).brushes[i as usize] as HGDIOBJ);
        i += 1;
    }
    i = 0;
    while i < PenLast as INT {
        DeleteObject((*infoPtr).pens[i as usize] as HGDIOBJ);
        i += 1;
    }

    heap_free(infoPtr as *mut core::ffi::c_void);
    return 0;
}

unsafe extern "C" fn MONTHCAL_Notify(infoPtr: *mut MONTHCAL_INFO, hdr: *mut NMHDR) -> LRESULT {
    if (*hdr).code == (UDN_DELTAPOS) as u32 {
        let nmud: *mut NMUPDOWN = hdr as *mut NMUPDOWN;

        if (*hdr).hwndFrom == (*infoPtr).hWndYearUpDown && (*nmud).iDelta != 0 {
            MONTHCAL_Scroll(infoPtr, 12 * (*nmud).iDelta, FALSE);
            MONTHCAL_NotifyDayState(infoPtr);
            MONTHCAL_NotifySelectionChange(infoPtr);
        }
    }
    return 0;
}

unsafe extern "C" fn MONTHCAL_SetUnicodeFormat(
    infoPtr: *mut MONTHCAL_INFO,
    isUnicode: BOOL,
) -> BOOL {
    let mut prev: BOOL = (*infoPtr).isUnicode;
    (*infoPtr).isUnicode = isUnicode;
    return prev;
}

unsafe extern "C" fn MONTHCAL_GetUnicodeFormat(infoPtr: *const MONTHCAL_INFO) -> BOOL {
    (*infoPtr).isUnicode
}
unsafe extern "system" fn MONTHCAL_WindowProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: *mut MONTHCAL_INFO =
        GetWindowLongPtrW(hwnd, 0) as *mut MONTHCAL_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/monthcal.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MONTHCAL_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            2888 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lparam=%lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && (uMsg != WM_CREATE) {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    match uMsg {
        MCM_GETCURSEL => return MONTHCAL_GetCurSel(infoPtr, lParam as *mut SYSTEMTIME),
        MCM_SETCURSEL => return MONTHCAL_SetCurSel(infoPtr, lParam as *mut SYSTEMTIME),
        MCM_GETMAXSELCOUNT => return MONTHCAL_GetMaxSelCount(infoPtr),
        MCM_SETMAXSELCOUNT => return MONTHCAL_SetMaxSelCount(infoPtr, (wParam) as INT),
        MCM_GETSELRANGE => return MONTHCAL_GetSelRange(infoPtr, lParam as *mut SYSTEMTIME),
        MCM_SETSELRANGE => return MONTHCAL_SetSelRange(infoPtr, lParam as *mut SYSTEMTIME),
        MCM_GETMONTHRANGE => {
            return MONTHCAL_GetMonthRange(infoPtr, (wParam) as DWORD, lParam as *mut SYSTEMTIME);
        }
        MCM_SETDAYSTATE => {
            return MONTHCAL_SetDayState(infoPtr, wParam as INT, lParam as *mut MONTHDAYSTATE);
        }
        MCM_GETMINREQRECT => return MONTHCAL_GetMinReqRect(infoPtr, lParam as *mut RECT),
        MCM_GETCOLOR => return MONTHCAL_GetColor(infoPtr, (wParam) as UINT) as LRESULT,
        MCM_SETCOLOR => {
            return MONTHCAL_SetColor(infoPtr, (wParam) as UINT, lParam as COLORREF);
        }
        MCM_GETTODAY => return MONTHCAL_GetToday(infoPtr, lParam as *mut SYSTEMTIME),
        MCM_SETTODAY => return MONTHCAL_SetToday(infoPtr, lParam as *mut SYSTEMTIME),
        MCM_HITTEST => return MONTHCAL_HitTest(infoPtr, lParam as *mut MCHITTESTINFO),
        MCM_GETFIRSTDAYOFWEEK => return MONTHCAL_GetFirstDayOfWeek(infoPtr),
        MCM_SETFIRSTDAYOFWEEK => return MONTHCAL_SetFirstDayOfWeek(infoPtr, lParam as INT),
        MCM_GETRANGE => return MONTHCAL_GetRange(infoPtr, lParam as *mut SYSTEMTIME),
        MCM_SETRANGE => {
            return MONTHCAL_SetRange(infoPtr, wParam as SHORT, lParam as *mut SYSTEMTIME);
        }
        MCM_GETMONTHDELTA => return MONTHCAL_GetMonthDelta(infoPtr),
        MCM_SETMONTHDELTA => return MONTHCAL_SetMonthDelta(infoPtr, wParam as INT),
        MCM_GETMAXTODAYWIDTH => return MONTHCAL_GetMaxTodayWidth(infoPtr),
        MCM_SETUNICODEFORMAT => return MONTHCAL_SetUnicodeFormat(infoPtr, wParam as BOOL) as LRESULT,
        MCM_GETUNICODEFORMAT => return MONTHCAL_GetUnicodeFormat(infoPtr) as LRESULT,
        MCM_GETCALENDARCOUNT => return MONTHCAL_GetCalCount(infoPtr) as LRESULT,
        WM_GETDLGCODE => {
            return (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT;
        }
        WM_RBUTTONUP => return MONTHCAL_RButtonUp(infoPtr, lParam),
        WM_LBUTTONDOWN => return MONTHCAL_LButtonDown(infoPtr, lParam),
        WM_MOUSEMOVE => return MONTHCAL_MouseMove(infoPtr, lParam),
        WM_LBUTTONUP => return MONTHCAL_LButtonUp(infoPtr, lParam),
        WM_PAINT => return MONTHCAL_Paint(infoPtr, wParam as HDC),
        WM_PRINTCLIENT => {
            return MONTHCAL_PrintClient(infoPtr, wParam as HDC, lParam as DWORD);
        }
        WM_ERASEBKGND => return MONTHCAL_EraseBkgnd(infoPtr, wParam as HDC),
        WM_SETFOCUS => return MONTHCAL_SetFocus(infoPtr),
        WM_SIZE => {
            return MONTHCAL_Size(
                infoPtr,
                (LOWORD!(lParam) as SHORT) as ::core::ffi::c_int,
                (HIWORD!(lParam) as SHORT) as ::core::ffi::c_int,
            );
        }
        WM_NOTIFY => return MONTHCAL_Notify(infoPtr, lParam as *mut NMHDR),
        WM_CREATE => return MONTHCAL_Create(hwnd, lParam as LPCREATESTRUCTW),
        WM_SETFONT => {
            return MONTHCAL_SetFont(infoPtr, wParam as HFONT, lParam as BOOL);
        }
        WM_GETFONT => return MONTHCAL_GetFont(infoPtr),
        WM_TIMER => return MONTHCAL_Timer(infoPtr, wParam),
        WM_THEMECHANGED => return theme_changed(infoPtr),
        WM_DESTROY => return MONTHCAL_Destroy(infoPtr),
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            return 0 as LRESULT;
        }
        WM_STYLECHANGED => {
            return (MONTHCAL_StyleChanged(infoPtr, wParam, lParam as *mut STYLESTRUCT)) as LRESULT;
        }
        WM_STYLECHANGING => {
            return (MONTHCAL_StyleChanging(infoPtr, wParam, lParam as *mut STYLESTRUCT)) as LRESULT;
        }
        _ => {
            if (uMsg >= WM_USER)
                && (uMsg < WM_APP)
                && (COMCTL32_IsReflectedMessage(uMsg) == 0)
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
                        b"dll/win32/comctl32/monthcal.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"MONTHCAL_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        3029 as ::core::ffi::c_int,
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
}
#[no_mangle]
pub unsafe extern "C" fn MONTHCAL_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = CS_GLOBALCLASS as UINT;
    wndClass.lpfnWndProc =
        Some(MONTHCAL_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut MONTHCAL_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR,
    ) as HICON;
    wndClass.hbrBackground =
        (COLOR_WINDOW + 1) as HBRUSH;
    wndClass.lpszClassName = (MONTHCAL_CLASSW).as_ptr();
    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn MONTHCAL_Unregister() {
    UnregisterClassW((MONTHCAL_CLASSW).as_ptr() as LPCWSTR, core::ptr::null_mut());
}
