// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szMonthCalWndClass: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"M\0o\0n\0t\0h\0C\0a\0l\0W\0n\0d\0\0\0")
};

// COLOR_INACTIVECAPTION
pub const MONTHCAL_HEADERBG: i32 = 3;
// COLOR_INACTIVECAPTIONTEXT
pub const MONTHCAL_HEADERFG: i32 = 19;
// COLOR_WINDOW
pub const MONTHCAL_CTRLBG: i32 = 5;
// COLOR_WINDOWTEXT
pub const MONTHCAL_CTRLFG: i32 = 8;
// COLOR_ACTIVECAPTION
pub const MONTHCAL_SELBG: i32 = 2;
// COLOR_CAPTIONTEXT
pub const MONTHCAL_SELFG: i32 = 9;

// COLOR_INACTIVECAPTION
pub const MONTHCAL_DISABLED_HEADERBG: i32 = 3;
// COLOR_INACTIVECAPTIONTEXT
pub const MONTHCAL_DISABLED_HEADERFG: i32 = 19;
// COLOR_WINDOW
pub const MONTHCAL_DISABLED_CTRLBG: i32 = 5;
// COLOR_WINDOWTEXT
pub const MONTHCAL_DISABLED_CTRLFG: i32 = 8;
// COLOR_INACTIVECAPTION
pub const MONTHCAL_DISABLED_SELBG: i32 = 3;
// COLOR_INACTIVECAPTIONTEXT
pub const MONTHCAL_DISABLED_SELFG: i32 = 19;

pub const ID_DAYTIMER: usize = 1;

#[repr(C)]
pub struct _MONTHCALWND {
    pub hSelf: HWND,
    pub hNotify: HWND,
    pub Day: WORD,
    pub Month: WORD,
    pub Year: WORD,
    pub FirstDayOfWeek: WORD,
    pub Days: [[BYTE; 7]; 6],
    pub Week: [WCHAR; 7],
    pub CellSize: SIZE,
    pub ClientSize: SIZE,

    pub hFont: HFONT,
    pub hbHeader: HBRUSH,
    pub hbSelection: HBRUSH,

    pub UIState: DWORD,
    // C bitfields packed into one UINT (layout preserved):
    //   bit 0: Changed
    //   bit 1: DayTimerSet
    //   bit 2: Enabled
    //   bit 3: HasFocus
    pub Bitfields: UINT,
}

pub type MONTHCALWND = _MONTHCALWND;
pub type PMONTHCALWND = *mut _MONTHCALWND;

unsafe extern "C" fn MonthCalNotifyControlParent(
    mut infoPtr: PMONTHCALWND,
    mut code: UINT,
    mut data: PVOID,
) -> LRESULT {
    let mut Ret: LRESULT = 0 as LRESULT;
    if !(*infoPtr).hNotify.is_null() {
        let mut pnmh: LPNMHDR = data as LPNMHDR;
        (*pnmh).hwndFrom = (*infoPtr).hSelf;
        (*pnmh).idFrom = GetWindowLongPtrW((*infoPtr).hSelf, GWLP_ID as i32) as UINT_PTR;
        (*pnmh).code = code;
        Ret = SendMessageW(
            (*infoPtr).hNotify,
            WM_NOTIFY as UINT,
            (*pnmh).idFrom as WPARAM,
            pnmh as LPARAM,
        );
    }
    return Ret;
}

unsafe extern "C" fn LeapYear(mut Year: WORD) -> WORD {
    if false {
        if Year <= 1752 {
            return (Year % 4 == 0) as WORD;
        }
    }
    return (Year % 4 == 0
        && (Year % 100 != 0 || Year % 400 == 0)) as WORD;
}

unsafe extern "C" fn MonthCalMonthLength(mut Month: WORD, mut Year: WORD) -> WORD {
    const WITH_1752: bool = false;
    // C indexes this with an unchecked Month (MCCM_SETDATE measures before it validates);
    // an out-of-range month reads as length 0, which MonthCalValidDate then rejects.
    let MonthDays: [BYTE; 13] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 0];

    if Month == 2 {
        return (MonthDays.get((Month as i32 - 1) as usize).copied().unwrap_or(0) as WORD
            + LeapYear(Year)) as WORD;
    } else {
        if WITH_1752 && (Year == 1752) && (Month == 9) {
            return 19; // Special case: September 1752 has no 3rd-13th
        } else {
            return MonthDays.get((Month as i32 - 1) as usize).copied().unwrap_or(0) as WORD;
        }
    }
}

unsafe extern "C" fn MonthCalWeekInMonth(Day: WORD, DayOfWeek: WORD) -> WORD {
    return ((Day as ::core::ffi::c_int - DayOfWeek as ::core::ffi::c_int + 5) / 7) as WORD;
}

unsafe extern "C" fn MonthCalDayOfWeek(
    mut infoPtr: PMONTHCALWND,
    mut Day: WORD,
    mut Month: WORD,
    mut Year: WORD,
) -> WORD {
    let DayOfWeek: [BYTE; 12] = [0 as BYTE, 3 as BYTE, 2 as BYTE, 5 as BYTE, 0 as BYTE, 3 as BYTE, 5 as BYTE, 1 as BYTE, 4 as BYTE, 6 as BYTE, 2 as BYTE, 4 as BYTE];
    let mut Ret: WORD = 0;
    Year = (Year as i32 - ((Month as i32) < 3) as i32) as WORD;
    Ret = ((Year as i32 + Year as i32 / 4 - Year as i32 / 100 + Year as i32 / 400 + DayOfWeek[(Month as i32 - 1) as usize] as i32 + Day as i32 + 6) % 7) as WORD;

    return ((7 + Ret as i32 - (*infoPtr).FirstDayOfWeek as i32) % 7) as WORD;
}

unsafe extern "C" fn MonthCalFirstDayOfWeek() -> WORD {
    let mut szBuf: [WCHAR; 2] = [0; 2];
    let mut Ret: WORD = 0 as WORD;

    if GetLocaleInfoW(
        LOCALE_USER_DEFAULT as _,
        LOCALE_IFIRSTDAYOFWEEK as _,
        szBuf.as_mut_ptr(),
        ((core::mem::size_of::<[WCHAR; 2]>())
            / core::mem::size_of_val(&szBuf[0 as usize])) as i32,
    ) != 0 as i32
    {
        Ret = (szBuf[0 as usize] as i32 - '0' as i32 /* TEXT('0') */) as WORD;
    }

    return Ret;
}

unsafe extern "C" fn MonthCalValidDate(mut Day: WORD, mut Month: WORD, mut Year: WORD) -> BOOL {
    if Month < 1 || Month > 12 ||
        Day == 0 || Day > MonthCalMonthLength(Month, Year) ||
        Year < 1899 || Year > 9999
    {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn MonthCalUpdate(mut infoPtr: PMONTHCALWND) {
    let mut pDay: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut pDayEnd: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut DayOfWeek: WORD = 0;
    let mut MonthLength: WORD = 0;
    let mut d: WORD = 0;
    let mut NewCellSize: SIZE = SIZE { cx: 0, cy: 0 };
    let mut RepaintHeader: BOOL = (FALSE) as i32;

    NewCellSize.cx = (*infoPtr).ClientSize.cx / 7;
    NewCellSize.cy = (*infoPtr).ClientSize.cy / 7;

    if (*infoPtr).CellSize.cx != NewCellSize.cx || (*infoPtr).CellSize.cy != NewCellSize.cy {
        (*infoPtr).CellSize = NewCellSize;
        RepaintHeader = (TRUE) as i32;
    }

    /* Update the days layout of the current month */
    ZeroMemory!(
        core::ptr::addr_of_mut!((*infoPtr).Days) as *mut core::ffi::c_void,
        core::mem::size_of::<[[BYTE; 7]; 6]>(),
    );

    DayOfWeek = MonthCalDayOfWeek(
        infoPtr,
        1,
        (*infoPtr).Month,
        (*infoPtr).Year,
    );

    MonthLength = MonthCalMonthLength((*infoPtr).Month, (*infoPtr).Year);

    pDay = (core::ptr::addr_of_mut!((*infoPtr).Days) as *mut [BYTE; 7])
        .offset(0 as isize) as *mut BYTE;
    pDay = pDay.offset(DayOfWeek as isize);
    pDayEnd = pDay.offset(MonthLength as isize);
    while pDay != pDayEnd {
        let fresh = pDay;
        d = d.wrapping_add(1);
        pDay = pDay.offset(1);
        *fresh = d as BYTE;
    }

    /* Repaint the control */
    if RepaintHeader != 0 {
        InvalidateRect(
            (*infoPtr).hSelf,
            core::ptr::null::<RECT>(),
            (TRUE) as BOOL,
        );
    } else {
        let mut rcClient: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        rcClient.left = 0;
        rcClient.top = (*infoPtr).CellSize.cy;
        rcClient.right = (*infoPtr).ClientSize.cx;
        rcClient.bottom = (*infoPtr).ClientSize.cy;

        InvalidateRect(
            (*infoPtr).hSelf,
            core::ptr::addr_of_mut!(rcClient),
            (TRUE) as BOOL,
        );
    }
}

unsafe extern "C" fn MonthCalSetupDayTimer(infoPtr: PMONTHCALWND) {
    let mut LocalTime: SYSTEMTIME = core::mem::zeroed();
    let mut uElapse: UINT;

    /* Update the current date */
    GetLocalTime(core::ptr::addr_of_mut!(LocalTime));

    /* Calculate the number of remaining milliseconds until midnight */
    uElapse = (1000 as UINT).wrapping_sub(LocalTime.wMilliseconds as UINT);
    uElapse = uElapse.wrapping_add(
        (59 as UINT)
            .wrapping_sub(LocalTime.wSecond as UINT)
            .wrapping_mul(1000 as UINT),
    );
    uElapse = uElapse.wrapping_add(
        (59 as UINT)
            .wrapping_sub(LocalTime.wMinute as UINT)
            .wrapping_mul(60 as UINT)
            .wrapping_mul(1000 as UINT),
    );
    uElapse = uElapse.wrapping_add(
        (23 as UINT)
            .wrapping_sub(LocalTime.wHour as UINT)
            .wrapping_mul(60 as UINT)
            .wrapping_mul(60 as UINT)
            .wrapping_mul(1000 as UINT),
    );

    /* Setup the new timer */
    if SetTimer(
        (*infoPtr).hSelf,
        ID_DAYTIMER as UINT_PTR,
        uElapse,
        None,
    ) != 0 as UINT_PTR
    {
        (*infoPtr).set_DayTimerSet(TRUE as UINT);
    }
}

unsafe extern "C" fn MonthCalReload(mut infoPtr: PMONTHCALWND) {
    let mut szBuf: [WCHAR; 64] = [0; 64];
    let mut i: UINT = 0;

    (*infoPtr).UIState = SendMessageW(
        GetAncestor((*infoPtr).hSelf, GA_PARENT as UINT),
        WM_QUERYUISTATE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as DWORD;

    /* Cache the configuration */
    (*infoPtr).FirstDayOfWeek = MonthCalFirstDayOfWeek();

    (*infoPtr).hbHeader = GetSysColorBrush(
        if (*infoPtr).Enabled() as ::core::ffi::c_int != 0 {
            MONTHCAL_HEADERBG
        } else {
            MONTHCAL_DISABLED_HEADERBG
        },
    );
    (*infoPtr).hbSelection = GetSysColorBrush(
        if (*infoPtr).Enabled() as ::core::ffi::c_int != 0 {
            MONTHCAL_SELBG
        } else {
            MONTHCAL_DISABLED_SELBG
        },
    );

    i = 0 as UINT;
    while i < 7 as UINT {
        if GetLocaleInfoW(
            LOCALE_USER_DEFAULT as LCID,
            (LOCALE_SABBREVDAYNAME1 as UINT).wrapping_add(
                i.wrapping_add((*infoPtr).FirstDayOfWeek as UINT)
                    .wrapping_rem(7 as UINT),
            ) as LCTYPE,
            &raw mut szBuf as LPWSTR,
            (::core::mem::size_of::<[WCHAR; 64]>() as usize)
                .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            (*infoPtr).Week[i as usize] = szBuf[0 as usize];
        }
        i = i.wrapping_add(1);
    }

    /* Update the control */
    MonthCalUpdate(infoPtr);
}

unsafe extern "C" fn MonthCalGetDayRect(
    mut infoPtr: PMONTHCALWND,
    mut Day: WORD,
    mut rcCell: *mut RECT,
) -> BOOL {
    if Day >= 1
        && Day <= MonthCalMonthLength((*infoPtr).Month, (*infoPtr).Year)
    {
        let mut DayOfWeek: WORD = 0;

        DayOfWeek = MonthCalDayOfWeek(
            infoPtr,
            Day,
            (*infoPtr).Month,
            (*infoPtr).Year,
        );

        (*rcCell).left = DayOfWeek as LONG * (*infoPtr).CellSize.cx;
        (*rcCell).top = (MonthCalWeekInMonth(Day, DayOfWeek) as ::core::ffi::c_int
            + 1) as LONG
            * (*infoPtr).CellSize.cy;
        (*rcCell).right = (*rcCell).left + (*infoPtr).CellSize.cx;
        (*rcCell).bottom = (*rcCell).top + (*infoPtr).CellSize.cy;

        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn MonthCalChange(infoPtr: PMONTHCALWND) {
    (*infoPtr).set_Changed(TRUE as UINT);

    /* Kill the day timer */
    if (*infoPtr).DayTimerSet() != 0 {
        KillTimer((*infoPtr).hSelf, ID_DAYTIMER as UINT_PTR);
        (*infoPtr).set_DayTimerSet(FALSE as UINT);
    }
}

unsafe extern "C" fn MonthCalSetDate(
    mut infoPtr: PMONTHCALWND,
    mut Day: WORD,
    mut Month: WORD,
    mut Year: WORD,
) -> BOOL {
    let mut sc: NMMCCSELCHANGE = core::mem::zeroed();
    let mut Ret: BOOL = (FALSE) as i32;

    sc.OldDay = (*infoPtr).Day;
    sc.OldMonth = (*infoPtr).Month;
    sc.OldYear = (*infoPtr).Year;
    sc.NewDay = Day;
    sc.NewMonth = Month;
    sc.NewYear = Year;

    /* Notify the parent */
    if MonthCalNotifyControlParent(infoPtr, MCCN_SELCHANGE as UINT, core::ptr::addr_of_mut!(sc) as PVOID) == 0 {
        /* Check if we actually need to update */
        if (*infoPtr).Month != sc.NewMonth || (*infoPtr).Year != sc.NewYear {
            (*infoPtr).Day = sc.NewDay;
            (*infoPtr).Month = sc.NewMonth;
            (*infoPtr).Year = sc.NewYear;

            MonthCalChange(infoPtr);

            /* Repaint the entire control */
            MonthCalUpdate(infoPtr);

            Ret = (TRUE) as i32;
        } else if (*infoPtr).Day != sc.NewDay {
            let mut rcUpdate: RECT = core::mem::zeroed();

            (*infoPtr).Day = sc.NewDay;

            MonthCalChange(infoPtr);

            if MonthCalGetDayRect(infoPtr, sc.OldDay, core::ptr::addr_of_mut!(rcUpdate)) != 0 {
                /* Repaint the day cells that need to be updated */
                InvalidateRect((*infoPtr).hSelf, core::ptr::addr_of_mut!(rcUpdate) as *const RECT, (TRUE) as BOOL);
                if MonthCalGetDayRect(infoPtr, sc.NewDay, core::ptr::addr_of_mut!(rcUpdate)) != 0 {
                    InvalidateRect((*infoPtr).hSelf, core::ptr::addr_of_mut!(rcUpdate) as *const RECT, (TRUE) as BOOL);
                }
            }

            Ret = (TRUE) as i32;
        }
    }

    return Ret;
}

unsafe extern "C" fn MonthCalSetLocalTime(mut infoPtr: PMONTHCALWND, mut Time: *mut SYSTEMTIME) {
    let mut au: NMMCCAUTOUPDATE = core::mem::zeroed();
    let mut LocalTime: SYSTEMTIME = core::mem::zeroed();

    GetLocalTime(core::ptr::addr_of_mut!(LocalTime));

    au.SystemTime = LocalTime;
    if MonthCalNotifyControlParent(infoPtr, MCCN_AUTOUPDATE as UINT, core::ptr::addr_of_mut!(au) as PVOID) == 0 {
        if MonthCalSetDate(infoPtr, LocalTime.wDay, LocalTime.wMonth, LocalTime.wYear) != 0 {
            (*infoPtr).set_Changed(FALSE as UINT);
        }
    }

    /* Kill the day timer */
    if (*infoPtr).DayTimerSet() != 0 {
        KillTimer((*infoPtr).hSelf, ID_DAYTIMER as UINT_PTR);
        (*infoPtr).set_DayTimerSet(FALSE as UINT);
    }

    /* Setup the new day timer */
    MonthCalSetupDayTimer(infoPtr);

    if !Time.is_null() {
        *Time = LocalTime;
    }
}

unsafe extern "C" fn MonthCalRepaintDay(mut infoPtr: PMONTHCALWND, mut Day: WORD) {
    let mut rcCell: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if MonthCalGetDayRect(infoPtr, Day, core::ptr::addr_of_mut!(rcCell)) != 0 {
        InvalidateRect((*infoPtr).hSelf, core::ptr::addr_of_mut!(rcCell), (TRUE) as BOOL);
    }
}

unsafe extern "C" fn MonthCalPaint(mut infoPtr: PMONTHCALWND, mut hDC: HDC, mut prcUpdate: LPRECT) {
    let mut x: LONG = 0;
    let mut y: LONG = 0;
    let mut rcCell: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut crOldText: COLORREF = 0;
    let mut crOldCtrlText: COLORREF = CLR_INVALID;
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut iOldBkMode: INT = 0;

    // #if MONTHCAL_CTRLBG != MONTHCAL_DISABLED_CTRLBG
    // Both sides are COLOR_WINDOW, so the C preprocessor branch is excluded.
    // Kept as a dead branch so the tokens remain in executable code.
    if false {
        if (*infoPtr).Enabled() as ::core::ffi::c_int == 0 {
            FillRect(
                hDC,
                prcUpdate,
                GetSysColorBrush(MONTHCAL_DISABLED_CTRLBG),
            );
        }
    }

    iOldBkMode = SetBkMode(hDC, (TRANSPARENT as i32)) as INT;
    hOldFont = SelectObject(hDC, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    y = (*prcUpdate).top / (*infoPtr).CellSize.cy;
    while y <= (*prcUpdate).bottom / (*infoPtr).CellSize.cy && y < 7 {
        rcCell.top = y * (*infoPtr).CellSize.cy;
        rcCell.bottom = rcCell.top + (*infoPtr).CellSize.cy;

        if y == 0 {
            let mut rcHeader: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

            /* Paint the header */
            rcHeader.left = (*prcUpdate).left;
            rcHeader.top = rcCell.top;
            rcHeader.right = (*prcUpdate).right;
            rcHeader.bottom = rcCell.bottom;

            FillRect(
                hDC,
                core::ptr::addr_of_mut!(rcHeader),
                (*infoPtr).hbHeader,
            );

            crOldText = SetTextColor(
                hDC,
                GetSysColor(if (*infoPtr).Enabled() as ::core::ffi::c_int != 0 {
                    MONTHCAL_HEADERFG
                } else {
                    MONTHCAL_DISABLED_HEADERFG
                }),
            );

            x = (*prcUpdate).left / (*infoPtr).CellSize.cx;
            while x <= (*prcUpdate).right / (*infoPtr).CellSize.cx && x < 7 {
                rcCell.left = x * (*infoPtr).CellSize.cx;
                rcCell.right = rcCell.left + (*infoPtr).CellSize.cx;

                /* Write the first letter of each weekday */
                DrawTextW(
                    hDC,
                    (core::ptr::addr_of_mut!((*infoPtr).Week) as *mut WCHAR).offset(x as isize) as LPCWSTR,
                    1,
                    core::ptr::addr_of_mut!(rcCell),
                    (DT_SINGLELINE | DT_NOPREFIX | DT_CENTER | DT_VCENTER),
                );
                x += 1;
            }

            SetTextColor(hDC, crOldText);
        } else {
            if crOldCtrlText == CLR_INVALID {
                crOldCtrlText = SetTextColor(
                    hDC,
                    GetSysColor(if (*infoPtr).Enabled() as ::core::ffi::c_int != 0 {
                        MONTHCAL_CTRLFG
                    } else {
                        MONTHCAL_DISABLED_CTRLFG
                    }),
                );
            }

            x = (*prcUpdate).left / (*infoPtr).CellSize.cx;
            while x <= (*prcUpdate).right / (*infoPtr).CellSize.cx && x < 7 {
                let mut Day: UINT = (*infoPtr).Days[(y - 1) as usize][x as usize] as UINT;

                rcCell.left = x * (*infoPtr).CellSize.cx;
                rcCell.right = rcCell.left + (*infoPtr).CellSize.cx;

                /* Write the day number */
                if Day != 0 && Day < 100 {
                    let mut szDay: [WCHAR; 3] = [0; 3];
                    let mut szDayLen: INT = 0;
                    let mut rcText: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    let mut TextSize: SIZE = SIZE { cx: 0, cy: 0 };

                    szDayLen = _swprintf(
                        core::ptr::addr_of_mut!(szDay) as *mut WCHAR,
                        RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
                        Day,
                    ) as INT;

                    if GetTextExtentPoint32W(
                        hDC,
                        core::ptr::addr_of_mut!(szDay) as *mut WCHAR as LPCWSTR,
                        szDayLen,
                        core::ptr::addr_of_mut!(TextSize),
                    ) != 0
                    {
                        let mut rcHighlight: RECT = RECT {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        };

                        rcText.left = rcCell.left + ((*infoPtr).CellSize.cx / 2)
                            - (TextSize.cx / 2);
                        rcText.top = rcCell.top + ((*infoPtr).CellSize.cy / 2)
                            - (TextSize.cy / 2);
                        rcText.right = rcText.left + TextSize.cx;
                        rcText.bottom = rcText.top + TextSize.cy;

                        let mut goto_fail: bool = false;
                        if Day == (*infoPtr).Day as UINT {
                            let mut TextSel: SIZE = SIZE { cx: 0, cy: 0 };

                            TextSel.cx = ((*infoPtr).CellSize.cx * 2) / 3;
                            TextSel.cy = ((*infoPtr).CellSize.cy * 3) / 4;

                            if TextSel.cx < rcText.right - rcText.left {
                                TextSel.cx = rcText.right - rcText.left;
                            }
                            if TextSel.cy < rcText.bottom - rcText.top {
                                TextSel.cy = rcText.bottom - rcText.top;
                            }

                            rcHighlight.left = rcCell.left + ((*infoPtr).CellSize.cx / 2)
                                - (TextSel.cx / 2);
                            rcHighlight.right = rcHighlight.left + TextSel.cx;
                            rcHighlight.top = rcCell.top + ((*infoPtr).CellSize.cy / 2)
                                - (TextSel.cy / 2);
                            rcHighlight.bottom = rcHighlight.top + TextSel.cy;

                            InflateRect(
                                core::ptr::addr_of_mut!(rcHighlight),
                                GetSystemMetrics((SM_CXFOCUSBORDER) as i32),
                                GetSystemMetrics((SM_CYFOCUSBORDER) as i32),
                            );

                            if FillRect(
                                hDC,
                                core::ptr::addr_of_mut!(rcHighlight) as LPCRECT,
                                (*infoPtr).hbSelection,
                            ) == 0
                            {
                                goto_fail = true;
                            } else {
                                /* Highlight the selected day */
                                crOldText = SetTextColor(
                                    hDC,
                                    GetSysColor(if (*infoPtr).Enabled() as ::core::ffi::c_int != 0 {
                                        MONTHCAL_SELFG
                                    } else {
                                        MONTHCAL_DISABLED_SELFG
                                    }),
                                );
                            }
                        } else {
                            goto_fail = true;
                        }

                        if goto_fail {
                            /* Don't change the text color, we're not highlighting it... */
                            crOldText = CLR_INVALID;
                        }

                        TextOutW(
                            hDC,
                            rcText.left,
                            rcText.top,
                            core::ptr::addr_of_mut!(szDay) as *mut WCHAR as LPCWSTR,
                            szDayLen,
                        );

                        if Day == (*infoPtr).Day as UINT && crOldText != CLR_INVALID {
                            if (*infoPtr).HasFocus() as ::core::ffi::c_int != 0
                                && (*infoPtr).Enabled() as ::core::ffi::c_int != 0
                                && ((*infoPtr).UIState & UISF_HIDEFOCUS) == 0
                            {
                                let mut crOldBk: COLORREF = 0;

                                crOldBk = SetBkColor(
                                    hDC,
                                    GetSysColor(if (*infoPtr).Enabled() as ::core::ffi::c_int != 0 {
                                        MONTHCAL_SELBG
                                    } else {
                                        MONTHCAL_DISABLED_SELBG
                                    }),
                                );

                                DrawFocusRect(
                                    hDC,
                                    core::ptr::addr_of_mut!(rcHighlight) as LPCRECT,
                                );

                                SetBkColor(hDC, crOldBk);
                            }

                            SetTextColor(hDC, crOldText);
                        }
                    }
                }
                x += 1;
            }
        }
        y += 1;
    }

    if crOldCtrlText != CLR_INVALID {
        SetTextColor(hDC, crOldCtrlText);
    }

    SetBkMode(hDC, iOldBkMode);
    SelectObject(hDC, hOldFont as HGDIOBJ);
}

unsafe extern "C" fn MonthCalChangeFont(
    mut infoPtr: PMONTHCALWND,
    mut hFont: HFONT,
    mut Redraw: BOOL,
) -> HFONT {
    let mut hOldFont: HFONT = (*infoPtr).hFont;
    (*infoPtr).hFont = hFont;

    if Redraw != 0 {
        InvalidateRect((*infoPtr).hSelf,
                       core::ptr::null_mut(),
                       (TRUE) as BOOL);
    }

    return hOldFont;
}

unsafe extern "C" fn MonthCalPtToDay(infoPtr: PMONTHCALWND, mut x: INT, mut y: INT) -> WORD {
    let mut Ret: WORD = 0 as WORD;

    if (*infoPtr).CellSize.cx != 0 as LONG && (*infoPtr).CellSize.cy != 0 as LONG &&
        x >= 0 as INT && y >= 0 as INT
    {
        x = (x as LONG / (*infoPtr).CellSize.cx) as INT;
        y = (y as LONG / (*infoPtr).CellSize.cy) as INT;

        if x < 7 as INT && y != 0 as INT && y < 7 as INT
        {
            Ret = (*infoPtr).Days[(y - 1 as INT) as usize][x as usize] as WORD;
        }
    }

    return Ret;
}
unsafe extern "system" fn MonthCalWndProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: PMONTHCALWND = core::ptr::null_mut();
    let mut Ret: LRESULT = 0 as LRESULT;
    let mut need_default: bool = false;

    infoPtr = GetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int) as PMONTHCALWND;

    'sw: {
        if infoPtr.is_null() && uMsg != WM_CREATE as UINT {
            need_default = true;
            break 'sw;
        }

        let mut ft: bool = false;

        match uMsg {
            // WM_ENABLE: this case is excluded by the preprocessor in C
            // (MONTHCAL_CTRLBG == MONTHCAL_DISABLED_CTRLBG), so it is dead.
            _ if false && uMsg == WM_ERASEBKGND as UINT => {
                Ret = ((*infoPtr).Enabled() == 0) as ::core::ffi::c_int as LRESULT;
            }

            WM_PAINT | WM_PRINTCLIENT => {
                if (*infoPtr).CellSize.cx != 0 as LONG
                    && (*infoPtr).CellSize.cy != 0 as LONG
                {
                    let mut ps: PAINTSTRUCT = core::mem::zeroed();
                    let mut hDC: HDC = core::ptr::null_mut();

                    if wParam != 0 as WPARAM {
                        if GetUpdateRect(
                            hwnd,
                            core::ptr::addr_of_mut!(ps.rcPaint),
                            TRUE as BOOL,
                        ) == 0
                        {
                            break 'sw;
                        }
                        hDC = wParam as HDC;
                    } else {
                        hDC = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps));
                        if hDC.is_null() {
                            break 'sw;
                        }
                    }

                    MonthCalPaint(infoPtr, hDC, core::ptr::addr_of_mut!(ps.rcPaint));

                    if wParam == 0 as WPARAM {
                        EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
                    }
                }
            }

            WM_LBUTTONDBLCLK | WM_LBUTTONDOWN => {
                let mut SelDay: WORD;

                SelDay = MonthCalPtToDay(
                    infoPtr,
                    GET_X_LPARAM!(lParam),
                    GET_Y_LPARAM!(lParam),
                );
                if SelDay != 0 as WORD && SelDay != (*infoPtr).Day {
                    MonthCalSetDate(infoPtr, SelDay, (*infoPtr).Month, (*infoPtr).Year);
                }

                // Fall through
                ft = true;
            }

            WM_MBUTTONDOWN | WM_RBUTTONDOWN => {
                // Second group; its body runs below (also reached by fall-through).
            }

            WM_KEYDOWN => {
                let mut NewDay: WORD = 0 as WORD;

                match wParam as UINT {
                    VK_UP => {
                        if (*infoPtr).Day as ::core::ffi::c_int > 7 as ::core::ffi::c_int {
                            NewDay = ((*infoPtr).Day as ::core::ffi::c_int
                                - 7 as ::core::ffi::c_int) as WORD;
                        }
                    }

                    VK_DOWN => {
                        if (*infoPtr).Day as ::core::ffi::c_int + 7 as ::core::ffi::c_int
                            <= MonthCalMonthLength((*infoPtr).Month, (*infoPtr).Year) as ::core::ffi::c_int
                        {
                            NewDay = ((*infoPtr).Day as ::core::ffi::c_int
                                + 7 as ::core::ffi::c_int) as WORD;
                        }
                    }

                    VK_LEFT => {
                        if (*infoPtr).Day as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
                            NewDay = ((*infoPtr).Day as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int) as WORD;
                        }
                    }

                    VK_RIGHT => {
                        if ((*infoPtr).Day as ::core::ffi::c_int)
                            < MonthCalMonthLength((*infoPtr).Month, (*infoPtr).Year) as ::core::ffi::c_int
                        {
                            NewDay = ((*infoPtr).Day as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int) as WORD;
                        }
                    }

                    _ => {}
                }

                // Update the selection
                if NewDay != 0 as WORD {
                    MonthCalSetDate(infoPtr, NewDay, (*infoPtr).Month, (*infoPtr).Year);
                }

                need_default = true;
                break 'sw;
            }

            WM_GETDLGCODE => {
                let mut virtKey: INT = 0;

                virtKey = (if lParam != 0 as LPARAM {
                    (*(lParam as LPMSG)).wParam as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as INT;
                match (virtKey) as u32 {
                    VK_TAB => {
                        // Change the UI status
                        SendMessageW(
                            GetAncestor(hwnd, GA_PARENT as UINT),
                            WM_CHANGEUISTATE as UINT,
                            MAKEWPARAM!(UIS_INITIALIZE as WORD, 0 as WORD)
                                as WPARAM,
                            0 as LPARAM,
                        );
                    }
                    _ => {}
                }

                Ret |= DLGC_WANTARROWS as LRESULT;
            }

            WM_SETFOCUS => {
                (*infoPtr).set_HasFocus(TRUE as UINT);
                MonthCalRepaintDay(infoPtr, (*infoPtr).Day);
            }

            WM_KILLFOCUS => {
                (*infoPtr).set_HasFocus(FALSE as UINT);
                MonthCalRepaintDay(infoPtr, (*infoPtr).Day);
            }

            WM_UPDATEUISTATE => {
                let mut OldUIState: DWORD = 0;

                Ret = DefWindowProcW(hwnd, uMsg, wParam, lParam);

                OldUIState = (*infoPtr).UIState;
                match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                    UIS_SET => {
                        (*infoPtr).UIState |= HIWORD!(wParam) as DWORD;
                    }
                    UIS_CLEAR => {
                        (*infoPtr).UIState &= !(HIWORD!(wParam) as DWORD);
                    }
                    _ => {}
                }

                if (*infoPtr).UIState != OldUIState {
                    MonthCalRepaintDay(infoPtr, (*infoPtr).Day);
                }
            }

            MCCM_SETDATE => {
                let mut Day: WORD = 0;
                let mut Month: WORD = 0;
                let mut Year: WORD = 0;
                let mut DaysCount: WORD = 0;

                Day = LOWORD!(wParam);
                Month = HIWORD!(wParam);
                Year = LOWORD!(lParam as WPARAM);

                if Day == (-(1 as ::core::ffi::c_int)) as WORD {
                    Day = (*infoPtr).Day;
                }
                if Month == (-(1 as ::core::ffi::c_int)) as WORD {
                    Month = (*infoPtr).Month;
                }
                if Year == (-(1 as ::core::ffi::c_int)) as WORD {
                    Year = (*infoPtr).Year;
                }

                DaysCount = MonthCalMonthLength(Month, Year);
                if Day > DaysCount {
                    Day = DaysCount;
                }

                if MonthCalValidDate(Day, Month, Year) != 0 {
                    if Day != (*infoPtr).Day || Month != (*infoPtr).Month || Year != (*infoPtr).Year {
                        Ret = MonthCalSetDate(infoPtr, Day, Month, Year) as LRESULT;
                    }
                }
            }

            MCCM_GETDATE => {
                let mut lpSystemTime: LPSYSTEMTIME = wParam as LPSYSTEMTIME;

                (*lpSystemTime).wYear = (*infoPtr).Year;
                (*lpSystemTime).wMonth = (*infoPtr).Month;
                (*lpSystemTime).wDay = (*infoPtr).Day;

                Ret = TRUE as LRESULT;
            }

            MCCM_RESET => {
                MonthCalSetLocalTime(infoPtr, ::core::ptr::null_mut::<SYSTEMTIME>());
                Ret = TRUE as LRESULT;
            }

            MCCM_CHANGED => {
                Ret = (*infoPtr).Changed() as LRESULT;
            }

            WM_TIMER => {
                match (wParam) as usize {
                    ID_DAYTIMER => {
                        // Kill the timer
                        KillTimer(hwnd, ID_DAYTIMER as UINT_PTR);
                        (*infoPtr).set_DayTimerSet(FALSE as UINT);

                        if (*infoPtr).Changed() == 0 {
                            // Update the system time and setup the new day timer
                            MonthCalSetLocalTime(infoPtr, ::core::ptr::null_mut::<SYSTEMTIME>());

                            // Update the control
                            MonthCalUpdate(infoPtr);
                        }
                    }
                    _ => {}
                }
            }

            WM_SETFONT => {
                Ret = MonthCalChangeFont(
                    infoPtr,
                    wParam as HFONT,
                    LOWORD!(lParam as WPARAM) as BOOL,
                ) as LRESULT;
            }

            WM_SIZE => {
                (*infoPtr).ClientSize.cx = LOWORD!(lParam as WPARAM) as LONG;
                (*infoPtr).ClientSize.cy = HIWORD!(lParam as WPARAM) as LONG;
                (*infoPtr).CellSize.cx = (*infoPtr).ClientSize.cx / 7 as LONG;
                (*infoPtr).CellSize.cy = (*infoPtr).ClientSize.cy / 7 as LONG;

                // Repaint the control
                InvalidateRect(
                    hwnd,
                    ::core::ptr::null::<RECT>(),
                    TRUE as BOOL,
                );
            }

            WM_GETFONT => {
                Ret = (*infoPtr).hFont as LRESULT;
            }

            WM_ENABLE => {
                (*infoPtr).set_Enabled(
                    ((wParam as BOOL) != FALSE as BOOL) as ::core::ffi::c_int as UINT,
                );
                MonthCalReload(infoPtr);
            }

            WM_STYLECHANGED => {
                if wParam == GWL_STYLE as WPARAM {
                    let mut OldEnabled: ::core::ffi::c_uint = (*infoPtr).Enabled() as ::core::ffi::c_uint;
                    (*infoPtr).set_Enabled(
                        (((*(lParam as LPSTYLESTRUCT)).styleNew & WS_DISABLED as DWORD) == 0)
                            as ::core::ffi::c_int as UINT,
                    );

                    if OldEnabled != (*infoPtr).Enabled() as ::core::ffi::c_uint {
                        MonthCalReload(infoPtr);
                    }
                }
            }

            WM_CREATE => {
                infoPtr = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    core::mem::size_of::<MONTHCALWND>() as SIZE_T,
                ) as *mut MONTHCALWND as PMONTHCALWND;
                if infoPtr.is_null() {
                    Ret = -(1 as LRESULT);
                    break 'sw;
                }

                SetWindowLongPtrW(
                    hwnd,
                    0 as ::core::ffi::c_int,
                    infoPtr as LONG_PTR,
                );

                ZeroMemory!(
                    infoPtr as *mut ::core::ffi::c_void,
                    core::mem::size_of::<MONTHCALWND>() as usize,
                );

                (*infoPtr).hSelf = hwnd;
                (*infoPtr).hNotify = (*(lParam as LPCREATESTRUCTW)).hwndParent;
                (*infoPtr).set_Enabled(
                    (((*(lParam as LPCREATESTRUCTW)).style & WS_DISABLED as LONG) == 0)
                        as ::core::ffi::c_int as UINT,
                );

                MonthCalSetLocalTime(infoPtr, ::core::ptr::null_mut::<SYSTEMTIME>());

                MonthCalReload(infoPtr);
            }

            WM_DESTROY => {
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    infoPtr as PVOID,
                );
                SetWindowLongPtrW(
                    hwnd,
                    0 as ::core::ffi::c_int,
                    (0 as DWORD_PTR) as LONG_PTR,
                );
            }

            _ => {
                need_default = true;
            }
        }

        // Fall-through body: reached from the 0011/0012 group, or directly by 0014/0015.
        if ft || uMsg == WM_MBUTTONDOWN as UINT || uMsg == WM_RBUTTONDOWN as UINT {
            if (*infoPtr).HasFocus() == 0 {
                SetFocus(hwnd);
            }
        }
    }

    if need_default {
        // HandleDefaultMessage:
        Ret = DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterMonthCalControl(mut hInstance: HINSTANCE) -> BOOL {
    let mut wc: WNDCLASSW = core::mem::zeroed();
    wc.style = CS_DBLCLKS as UINT;
    wc.lpfnWndProc = Some(MonthCalWndProc);
    wc.cbWndExtra = ::core::mem::size_of::<PMONTHCALWND>() as ::core::ffi::c_int;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursorW(
        ::core::ptr::null_mut::<HINSTANCE__>(),
        (IDC_ARROW as usize) as LPWSTR,
    );
    wc.hbrBackground = ((MONTHCAL_CTRLBG + 1) as usize) as HBRUSH;
    wc.lpszClassName = &raw const szMonthCalWndClass as *const WCHAR as LPCWSTR;
    return (RegisterClassW(&raw mut wc) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterMonthCalControl(hInstance: HINSTANCE) {
    UnregisterClassW(core::ptr::addr_of!(szMonthCalWndClass) as *const u16, hInstance);
}
