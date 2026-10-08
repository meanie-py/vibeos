use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut pOldWndProc: WNDPROC = None;
#[no_mangle]
pub unsafe extern "C" fn SystemSetLocalTime(lpSystemTime: LPSYSTEMTIME) -> BOOL {
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut PrevSize: DWORD = 0;
    let mut priv_: TOKEN_PRIVILEGES = core::mem::zeroed();
    let mut previouspriv: TOKEN_PRIVILEGES = core::mem::zeroed();
    let mut Ret: BOOL = (FALSE) as i32;

    /*
     * Enable the SeSystemtimePrivilege privilege
     */

    if OpenProcessToken(
        GetCurrentProcess(),
        (TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY) as DWORD,
        core::ptr::addr_of_mut!(hToken),
    ) != 0
    {
        priv_.PrivilegeCount = 1 as DWORD;
        priv_.Privileges[0 as usize].Attributes = SE_PRIVILEGE_ENABLED as DWORD;

        if LookupPrivilegeValueW(
            core::ptr::null(),
            win32macros_rust::wide_bytes!(b"S\0e\0S\0y\0s\0t\0e\0m\0t\0i\0m\0e\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0"), // SE_SYSTEMTIME_NAME
            core::ptr::addr_of_mut!(priv_.Privileges[0 as usize].Luid),
        ) != 0
        {
            if AdjustTokenPrivileges(
                hToken,
                (FALSE) as BOOL,
                core::ptr::addr_of_mut!(priv_),
                core::mem::size_of::<TOKEN_PRIVILEGES>() as DWORD,
                core::ptr::addr_of_mut!(previouspriv),
                core::ptr::addr_of_mut!(PrevSize),
            ) != 0
                && GetLastError() == ERROR_SUCCESS as DWORD
            {
                /*
                 * We successfully enabled it, we're permitted to change the time.
                 * Call SetLocalTime twice to ensure correct results.
                 */
                Ret = (SetLocalTime(lpSystemTime) != 0
                    && SetLocalTime(lpSystemTime) != 0) as i32;

                /*
                 * For the sake of security, restore the previous status again
                 */
                if previouspriv.PrivilegeCount > 0 as DWORD {
                    AdjustTokenPrivileges(
                        hToken,
                        (FALSE) as BOOL,
                        core::ptr::addr_of_mut!(previouspriv),
                        0 as DWORD,
                        core::ptr::null_mut(),
                        0 as *mut DWORD,
                    );
                }
            }
        }
        CloseHandle(hToken);
    }

    return Ret;
}

unsafe extern "C" fn SetLocalSystemTime(hwnd: HWND) {
    let mut Time: SYSTEMTIME = core::mem::zeroed();

    if DateTime_GetSystemtime!(GetDlgItem(hwnd,
                                          (IDC_TIMEPICKER as i32)),
                               core::ptr::addr_of_mut!(Time)) == GDT_VALID &&
        SendMessageW(GetDlgItem(hwnd,
                                (IDC_MONTHCALENDAR as i32)),
                     MCCM_GETDATE,
                     core::ptr::addr_of_mut!(Time) as WPARAM,
                     0) != 0
    {
        SystemSetLocalTime(core::ptr::addr_of_mut!(Time));

        SetWindowLongPtrW(hwnd,
                          DWLP_MSGRESULT,
                          (PSNRET_NOERROR) as LONG_PTR);

        SendMessageW(GetDlgItem(hwnd,
                                (IDC_MONTHCALENDAR as i32)),
                     MCCM_RESET,
                     core::ptr::addr_of_mut!(Time) as WPARAM,
                     0);

        /* Broadcast the time change message */
        SendMessageW(HWND_BROADCAST,
                     WM_TIMECHANGE,
                     0,
                     0);
    }
}

unsafe extern "C" fn SetTimeZoneName(hwnd: HWND) {
    let mut TimeZoneInfo: TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut TimeZoneString: [WCHAR; 128] = [0u16; 128];
    let mut TimeZoneText: [WCHAR; 128] = [0u16; 128];
    let mut TimeZoneName: [WCHAR; 128] = [0u16; 128];
    let mut TimeZoneId: DWORD;

    TimeZoneId = GetTimeZoneInformation(core::ptr::addr_of_mut!(TimeZoneInfo));

    LoadStringW(
        hApplet,
        IDS_TIMEZONETEXT as UINT,
        core::ptr::addr_of_mut!(TimeZoneText) as LPWSTR,
        128 as ::core::ffi::c_int,
    );

    match TimeZoneId {
        v if v == TIME_ZONE_ID_STANDARD || v == TIME_ZONE_ID_UNKNOWN => {
            wcscpy(
                core::ptr::addr_of_mut!(TimeZoneName) as *mut WCHAR,
                core::ptr::addr_of_mut!(TimeZoneInfo.StandardName) as *mut WCHAR,
            );
        }
        v if v == TIME_ZONE_ID_DAYLIGHT => {
            wcscpy(
                core::ptr::addr_of_mut!(TimeZoneName) as *mut WCHAR,
                core::ptr::addr_of_mut!(TimeZoneInfo.DaylightName) as *mut WCHAR,
            );
        }
        _ => {
            let _ = TIME_ZONE_ID_INVALID;
            LoadStringW(
                hApplet,
                IDS_TIMEZONEINVALID as UINT,
                core::ptr::addr_of_mut!(TimeZoneName) as LPWSTR,
                128 as ::core::ffi::c_int,
            );
        }
    }

    wsprintfW(
        core::ptr::addr_of_mut!(TimeZoneString) as LPWSTR,
        core::ptr::addr_of_mut!(TimeZoneText) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(TimeZoneName) as *mut WCHAR,
    );

    SendDlgItemMessageW(
        hwnd,
        (IDC_TIMEZONE as i32),
        WM_SETTEXT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(TimeZoneString) as *mut WCHAR as LPARAM,
    );
}

unsafe extern "C" fn FillMonthsComboBox(hCombo: HWND) {
    let mut LocalDate: SYSTEMTIME = core::mem::zeroed();
    let mut szBuf: [WCHAR; 64] = [0; 64];
    let mut i: INT = 0;
    let mut Month: UINT = 0;

    GetLocalTime(core::ptr::addr_of_mut!(LocalDate));

    SendMessageW(
        hCombo,
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    Month = 1 as UINT;
    while Month <= 13 as UINT {
        i = GetLocaleInfoW(
            LOCALE_USER_DEFAULT as LCID,
            (if Month < 13 as UINT {
                (LOCALE_SMONTHNAME1 as UINT)
                    .wrapping_add(Month)
                    .wrapping_sub(1 as UINT)
            } else {
                LOCALE_SMONTHNAME13 as UINT
            }) as LCTYPE,
            core::ptr::addr_of_mut!(szBuf) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 64]>() / core::mem::size_of::<WCHAR>())
                as ::core::ffi::c_int,
        ) as INT;
        if i > 1 as INT {
            i = SendMessageW(
                hCombo,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(szBuf) as *mut WCHAR as LPARAM,
            ) as INT;
            if i != CB_ERR as INT {
                SendMessageW(
                    hCombo,
                    CB_SETITEMDATA as UINT,
                    i as WPARAM,
                    Month as LPARAM,
                );

                if Month == LocalDate.wMonth as UINT {
                    SendMessageW(
                        hCombo,
                        CB_SETCURSEL as UINT,
                        i as WPARAM,
                        0 as LPARAM,
                    );
                }
            }
        }
        Month = Month.wrapping_add(1);
    }
}

unsafe extern "C" fn GetCBSelectedMonth(hCombo: HWND) -> WORD {
    let mut i: INT = 0;
    let mut Ret: WORD = (-(1 as ::core::ffi::c_int)) as WORD;
    i = SendMessageW(hCombo, CB_GETCURSEL as u32, 0 as WPARAM, 0 as LPARAM) as INT;
    if i != CB_ERR as INT {
        i = SendMessageW(hCombo, CB_GETITEMDATA as u32, i as WPARAM, 0 as LPARAM) as INT;

        if i >= 1 as INT && i <= 13 as INT {
            Ret = i as WORD;
        }
    }

    return Ret;
}

unsafe extern "C" fn ChangeMonthCalDate(hMonthCal: HWND, Day: WORD, Month: WORD, Year: WORD) {
    SendMessageW(
        hMonthCal,
        MCCM_SETDATE,
        MAKEWPARAM!(Day, Month),
        MAKELPARAM!(Year, 0),
    );
}

unsafe extern "C" fn AutoUpdateMonthCal(hwndDlg: HWND, lpAutoUpdate: PNMMCCAUTOUPDATE) {
    UNREFERENCED_PARAMETER!(lpAutoUpdate);

    /* Update the controls */
    FillMonthsComboBox(GetDlgItem(hwndDlg, (IDC_MONTHCB as i32)));
}

unsafe extern "system" fn DTPProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_KEYDOWN => {
            /* Stop the timer when the user is about to change the time */
            if ((wParam != (VK_LEFT as WPARAM)) as i32 & (wParam != (VK_RIGHT as WPARAM)) as i32) != 0 {
                KillTimer(GetParent(hwnd), ID_TIMER as UINT_PTR);
            }
        }
        _ => {}
    }

    return CallWindowProcW(pOldWndProc, hwnd, uMsg, wParam, lParam) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn DateTimePageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut st: SYSTEMTIME = core::mem::zeroed();
    GetLocalTime(core::ptr::addr_of_mut!(st));

    match uMsg {
        WM_INITDIALOG => {
            FillMonthsComboBox(GetDlgItem(hwndDlg, (IDC_MONTHCB as i32)));

            /* Set range and current year */
            SendMessageW(
                GetDlgItem(hwndDlg, (IDC_YEAR as i32)),
                UDM_SETRANGE as UINT,
                0 as WPARAM,
                MAKELONG!(
                    9999 as ::core::ffi::c_short,
                    1900 as ::core::ffi::c_short,
                ) as LPARAM,
            );
            SendMessageW(
                GetDlgItem(hwndDlg, (IDC_YEAR as i32)),
                UDM_SETPOS as UINT,
                0 as WPARAM,
                MAKELONG!(
                    st.wYear as ::core::ffi::c_short,
                    0 as ::core::ffi::c_short,
                ) as LPARAM,
            );

            pOldWndProc = core::mem::transmute::<LONG_PTR, WNDPROC>(SetWindowLongPtrW(
                GetDlgItem(hwndDlg, (IDC_TIMEPICKER as i32)),
                GWLP_WNDPROC as ::core::ffi::c_int,
                (DTPProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR)
                    as LONG_PTR,
            ));

            SetTimer(
                hwndDlg,
                ID_TIMER as UINT_PTR,
                (1000 - st.wMilliseconds as ::core::ffi::c_int) as UINT,
                None,
            );
        }

        WM_TIMER => {
            SendMessageW(
                GetDlgItem(hwndDlg, (IDC_TIMEPICKER as i32)),
                DTM_SETSYSTEMTIME as UINT,
                GDT_VALID as WPARAM,
                core::ptr::addr_of_mut!(st) as LPARAM,
            );

            // Reset timeout.
            SetTimer(
                hwndDlg,
                ID_TIMER as UINT_PTR,
                (1000 - st.wMilliseconds as ::core::ffi::c_int) as UINT,
                None,
            );
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_MONTHCB => {
                    if (HIWORD!(wParam) as ::core::ffi::c_int) as u32 == CBN_SELCHANGE {
                        ChangeMonthCalDate(
                            GetDlgItem(hwndDlg, (IDC_MONTHCALENDAR as i32)),
                            (-(1 as ::core::ffi::c_int)) as WORD,
                            GetCBSelectedMonth(lParam as HWND),
                            (-(1 as ::core::ffi::c_int)) as WORD,
                        );
                    }
                }
                _ => {}
            }
        }

        WM_CTLCOLORSTATIC => {
            if (lParam as HWND) == GetDlgItem(hwndDlg, (IDC_YEARTEXT as i32)) {
                return GetSysColorBrush((COLOR_WINDOW) as i32) as INT_PTR;
            }
        }

        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match (*lpnm).idFrom {
                x if x == IDC_YEAR as UINT_PTR => match ((*lpnm).code) as i32 {
                    UDN_DELTAPOS => {
                        let mut wYear: SHORT = 0;
                        let mut updown: LPNMUPDOWN = lpnm as LPNMUPDOWN;
                        wYear = SendMessageW(
                            GetDlgItem(hwndDlg, (IDC_YEAR as i32)),
                            UDM_GETPOS as UINT,
                            0 as WPARAM,
                            0 as LPARAM,
                        ) as SHORT;
                        /* Enable the 'Apply' button */
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        ChangeMonthCalDate(
                            GetDlgItem(hwndDlg, (IDC_MONTHCALENDAR as i32)),
                            (-(1 as ::core::ffi::c_int)) as WORD,
                            (-(1 as ::core::ffi::c_int)) as WORD,
                            (wYear as ::core::ffi::c_int + (*updown).iDelta) as WORD,
                        );
                    }
                    _ => {}
                },

                x if x == IDC_TIMEPICKER as UINT_PTR => match ((*lpnm).code) as i32 {
                    DTN_DATETIMECHANGE => {
                        /* Stop the timer */
                        KillTimer(hwndDlg, ID_TIMER as UINT_PTR);

                        /* Tell the clock to stop ticking */
                        SendDlgItemMessageW(
                            hwndDlg,
                            (IDC_CLOCKWND as i32),
                            CLM_STOPCLOCK as UINT,
                            0 as WPARAM,
                            0 as LPARAM,
                        );

                        // TODO: Set the clock to the input time.

                        /* Enable the 'Apply' button */
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    _ => {}
                },

                x if x == IDC_MONTHCALENDAR as UINT_PTR => match (*lpnm).code {
                    MCCN_SELCHANGE => {
                        /* Enable the 'Apply' button */
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    MCCN_AUTOUPDATE => {
                        AutoUpdateMonthCal(hwndDlg, lpnm as PNMMCCAUTOUPDATE);
                    }
                    _ => {}
                },

                _ => match ((*lpnm).code) as i32 {
                    PSN_SETACTIVE => {
                        SetTimeZoneName(hwndDlg);
                    }
                    PSN_APPLY => {
                        SetLocalSystemTime(hwndDlg);

                        /* Tell the clock to start ticking */
                        SendDlgItemMessageW(
                            hwndDlg,
                            (IDC_CLOCKWND as i32),
                            CLM_STARTCLOCK as UINT,
                            0 as WPARAM,
                            0 as LPARAM,
                        );

                        SetTimer(
                            hwndDlg,
                            ID_TIMER as UINT_PTR,
                            (1000 - st.wMilliseconds as ::core::ffi::c_int)
                                as UINT,
                            None,
                        );
                        return TRUE as INT_PTR;
                    }
                    _ => {}
                },
            }
        }

        WM_TIMECHANGE => {
            /* FIXME: We don't get this message as we're not a top-level window... */
            SendMessageW(
                GetDlgItem(hwndDlg, (IDC_MONTHCALENDAR as i32)),
                MCCM_RESET as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
        }

        WM_DESTROY => {
            KillTimer(hwndDlg, ID_TIMER as UINT_PTR);
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
