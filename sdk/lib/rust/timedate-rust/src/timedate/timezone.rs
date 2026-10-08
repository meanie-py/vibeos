use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hBitmap: HBITMAP = ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__;

static mut cxSource: ::core::ffi::c_int = 0;

static mut cySource: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut TimeZoneListHead: PTIMEZONE_ENTRY =
    ::core::ptr::null::<_TIMEZONE_ENTRY>() as *mut _TIMEZONE_ENTRY;

#[no_mangle]
pub static mut TimeZoneListTail: PTIMEZONE_ENTRY =
    ::core::ptr::null::<_TIMEZONE_ENTRY>() as *mut _TIMEZONE_ENTRY;

#[repr(C)]
pub struct _TIMEZONE_ENTRY {
    pub Prev: *mut _TIMEZONE_ENTRY,
    pub Next: *mut _TIMEZONE_ENTRY,
    pub Description: [WCHAR; 128],
    pub StandardName: [WCHAR; 33],
    pub DaylightName: [WCHAR; 33],
    pub TimezoneInfo: REG_TZI_FORMAT,
}
pub type TIMEZONE_ENTRY = _TIMEZONE_ENTRY;
pub type PTIMEZONE_ENTRY = *mut _TIMEZONE_ENTRY;

static RETRIEVE_TIMEZONE_UNUSED_CONTEXT_MARKER: () = ();

unsafe extern "C" fn GetSelectedTimeZoneEntry(hwndCombo: HWND) -> PTIMEZONE_ENTRY {
    let mut Entry: PTIMEZONE_ENTRY = core::ptr::null_mut();
    let mut dwIndex: DWORD = 0;
    let mut i: DWORD = 0;
    dwIndex = SendMessageW(hwndCombo, CB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM) as DWORD;
    if dwIndex == CB_ERR as DWORD {
        return core::ptr::null_mut();
    }
    Entry = TimeZoneListHead;
    i = 0 as DWORD;
    while i < dwIndex {
        if Entry.is_null() {
            return core::ptr::null_mut();
        }
        i = i.wrapping_add(1);
        Entry = (*Entry).Next;
    }
    return Entry;
}

unsafe extern "C" fn HasDaylightSaving(Entry: PTIMEZONE_ENTRY) -> BOOL {
    /* If StandardDate.wMonth is 0, DST does not apply to this timezone */
    return (!Entry.is_null()
        && ((*Entry).TimezoneInfo.StandardDate.wMonth as ::core::ffi::c_int)
            != 0) as BOOL;
}

unsafe extern "C" fn UpdateDstCheckbox(mut hwndDlg: HWND) {
    let mut hwndCombo: HWND = GetDlgItem(hwndDlg, (IDC_TIMEZONELIST as i32));
    let mut hwndCheckbox: HWND = GetDlgItem(hwndDlg, (IDC_AUTODAYLIGHT as i32));
    let mut Entry: PTIMEZONE_ENTRY = core::ptr::null_mut();
    let mut bHasDst: BOOL = 0;

    Entry = GetSelectedTimeZoneEntry(hwndCombo);
    bHasDst = HasDaylightSaving(Entry);

    /* Enable or disable the checkbox based on DST support, and respect user preference when DST is supported */
    if bHasDst != 0 {
        let mut bAutoDaylight: BOOL = GetAutoDaylight();

        EnableWindow(hwndCheckbox, (TRUE) as BOOL);
        SendMessageW(
            hwndCheckbox,
            BM_SETCHECK as UINT,
            (if bAutoDaylight != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as WPARAM,
            0 as LPARAM,
        );
    } else {
        EnableWindow(hwndCheckbox, (FALSE) as BOOL);
        SendMessageW(
            hwndCheckbox,
            BM_SETCHECK as UINT,
            BST_UNCHECKED as WPARAM,
            0 as LPARAM,
        );
    }
}

unsafe extern "C" fn GetLargerTimeZoneEntry(
    mut Bias: LONG,
    mut lpDescription: LPWSTR,
) -> PTIMEZONE_ENTRY {
    let mut Entry: PTIMEZONE_ENTRY = core::ptr::null_mut::<_TIMEZONE_ENTRY>();
    Entry = TimeZoneListHead;
    while !Entry.is_null() {
        if (*Entry).TimezoneInfo.Bias < Bias {
            return Entry;
        }
        if (*Entry).TimezoneInfo.Bias == Bias {
            if _wcsicmp(
                core::ptr::addr_of_mut!((*Entry).Description) as *mut WCHAR,
                lpDescription as *const WCHAR,
            ) > 0
            {
                return Entry;
            }
        }
        Entry = (*Entry).Next as PTIMEZONE_ENTRY;
    }
    return core::ptr::null_mut::<_TIMEZONE_ENTRY>();
}

unsafe extern "C" fn RetrieveTimeZone(mut hZoneKey: HKEY, mut Context: PVOID) -> LONG {
    let mut lError: LONG = 0;
    let mut Entry: PTIMEZONE_ENTRY = core::ptr::null_mut::<_TIMEZONE_ENTRY>();
    let mut Current: PTIMEZONE_ENTRY = core::ptr::null_mut::<_TIMEZONE_ENTRY>();
    let mut DescriptionSize: ULONG = 0;
    let mut StandardNameSize: ULONG = 0;
    let mut DaylightNameSize: ULONG = 0;

    Entry = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<TIMEZONE_ENTRY>() as SIZE_T,
    ) as PTIMEZONE_ENTRY;
    if Entry.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    DescriptionSize = core::mem::size_of::<[WCHAR; 128]>() as ULONG;
    StandardNameSize = core::mem::size_of::<[WCHAR; 33]>() as ULONG;
    DaylightNameSize = core::mem::size_of::<[WCHAR; 33]>() as ULONG;

    lError = QueryTimeZoneData(
        hZoneKey,
        core::ptr::null_mut::<ULONG>(),
        &raw mut (*Entry).TimezoneInfo,
        &raw mut (*Entry).Description as PWCHAR,
        &raw mut DescriptionSize,
        &raw mut (*Entry).StandardName as PWCHAR,
        &raw mut StandardNameSize,
        &raw mut (*Entry).DaylightName as PWCHAR,
        &raw mut DaylightNameSize,
    );
    if lError != ERROR_SUCCESS as LONG {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            Entry as PVOID,
        );
        return lError;
    }

    if TimeZoneListHead.is_null() && TimeZoneListTail.is_null() {
        (*Entry).Prev = core::ptr::null_mut::<_TIMEZONE_ENTRY>();
        (*Entry).Next = core::ptr::null_mut::<_TIMEZONE_ENTRY>();
        TimeZoneListHead = Entry;
        TimeZoneListTail = Entry;
    } else {
        Current = GetLargerTimeZoneEntry(
            (*Entry).TimezoneInfo.Bias,
            &raw mut (*Entry).Description as LPWSTR,
        );
        if !Current.is_null() {
            if Current == TimeZoneListHead {
                (*Entry).Prev = core::ptr::null_mut::<_TIMEZONE_ENTRY>();
                (*Entry).Next = TimeZoneListHead as *mut _TIMEZONE_ENTRY;
                (*TimeZoneListHead).Prev = Entry as *mut _TIMEZONE_ENTRY;
                TimeZoneListHead = Entry;
            } else {
                (*Entry).Prev = (*Current).Prev;
                (*Entry).Next = Current as *mut _TIMEZONE_ENTRY;
                (*(*Current).Prev).Next = Entry as *mut _TIMEZONE_ENTRY;
                (*Current).Prev = Entry as *mut _TIMEZONE_ENTRY;
            }
        } else {
            (*Entry).Prev = TimeZoneListTail as *mut _TIMEZONE_ENTRY;
            (*Entry).Next = core::ptr::null_mut::<_TIMEZONE_ENTRY>();
            (*TimeZoneListTail).Next = Entry as *mut _TIMEZONE_ENTRY;
            TimeZoneListTail = Entry;
        }
    }

    return ERROR_SUCCESS as LONG;
}

unsafe extern "C" fn CreateTimeZoneList() {
    EnumerateTimeZoneList(
        Some(RetrieveTimeZone as unsafe extern "C" fn(HKEY, PVOID) -> LONG),
        core::ptr::null_mut(),
    );
}

unsafe extern "C" fn DestroyTimeZoneList() {
    let mut Entry: PTIMEZONE_ENTRY = core::ptr::null_mut();
    while !TimeZoneListHead.is_null() {
        Entry = TimeZoneListHead;
        TimeZoneListHead = (*Entry).Next;
        if !TimeZoneListHead.is_null() {
            (*TimeZoneListHead).Prev = core::ptr::null_mut();
        }
        HeapFree(GetProcessHeap(), 0 as DWORD, Entry as PVOID);
    }
    TimeZoneListTail = core::ptr::null_mut();
}

unsafe extern "C" fn ShowTimeZoneList(hwnd: HWND) {
    let mut TimeZoneInfo: TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut Entry: PTIMEZONE_ENTRY = core::ptr::null_mut();
    let mut dwCount: DWORD = 0;
    let mut dwIndex: DWORD = 0 as DWORD;
    let mut bFound: BOOL = FALSE as BOOL;

    if GetTimeZoneInformation(core::ptr::addr_of_mut!(TimeZoneInfo)) == TIME_ZONE_ID_INVALID as DWORD {
        /* Failed to retrieve current time-zone info, reset it */
        ZeroMemory!(
            core::ptr::addr_of_mut!(TimeZoneInfo) as *mut core::ffi::c_void,
            core::mem::size_of::<TIME_ZONE_INFORMATION>(),
        );
    }

    Entry = TimeZoneListHead;
    while !Entry.is_null() {
        'body: {
            dwCount = SendMessageW(
                hwnd,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!((*Entry).Description) as *mut WCHAR as LPARAM,
            ) as DWORD;
            if dwCount == CB_ERR as DWORD || dwCount == CB_ERRSPACE as DWORD {
                break 'body;
            }

            /* If the time-zone was found in the list, skip the tests */
            if bFound != 0 {
                break 'body;
            }

            if TimeZoneInfo.StandardName[0] != 0 && (*Entry).StandardName[0] != 0 {
                /* Compare by name */
                if wcscmp(
                    core::ptr::addr_of_mut!((*Entry).StandardName) as *mut WCHAR,
                    core::ptr::addr_of_mut!(TimeZoneInfo.StandardName) as *mut WCHAR,
                ) == 0 as i32
                {
                    dwIndex = dwCount;
                    bFound = TRUE as BOOL;
                }
            } else {
                /* Compare by date and bias */
                if (*Entry).TimezoneInfo.Bias == TimeZoneInfo.Bias
                    && (*Entry).TimezoneInfo.StandardBias == TimeZoneInfo.StandardBias
                    && (*Entry).TimezoneInfo.DaylightBias == TimeZoneInfo.DaylightBias
                    && memcmp(
                        core::ptr::addr_of_mut!((*Entry).TimezoneInfo.StandardDate) as *const core::ffi::c_void,
                        core::ptr::addr_of_mut!(TimeZoneInfo.StandardDate) as *const core::ffi::c_void,
                        core::mem::size_of::<SYSTEMTIME>(),
                    ) == 0 as i32
                    && memcmp(
                        core::ptr::addr_of_mut!((*Entry).TimezoneInfo.DaylightDate) as *const core::ffi::c_void,
                        core::ptr::addr_of_mut!(TimeZoneInfo.DaylightDate) as *const core::ffi::c_void,
                        core::mem::size_of::<SYSTEMTIME>(),
                    ) == 0 as i32
                {
                    dwIndex = dwCount;
                    bFound = TRUE as BOOL;
                }
            }
        }
        Entry = (*Entry).Next;
    }

    SendMessageW(
        hwnd,
        CB_SETCURSEL as UINT,
        dwIndex as WPARAM,
        0 as LPARAM,
    );
}

unsafe extern "C" fn SetLocalTimeZone(hwnd: HWND) {
    let mut TimeZoneInformation: TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut Entry: PTIMEZONE_ENTRY = core::ptr::null_mut();

    Entry = GetSelectedTimeZoneEntry(hwnd);
    if Entry.is_null() {
        return;
    }

    wcscpy(
        core::ptr::addr_of_mut!(TimeZoneInformation.StandardName) as *mut u16,
        core::ptr::addr_of_mut!((*Entry).StandardName) as *const u16,
    );
    wcscpy(
        core::ptr::addr_of_mut!(TimeZoneInformation.DaylightName) as *mut u16,
        core::ptr::addr_of_mut!((*Entry).DaylightName) as *const u16,
    );

    TimeZoneInformation.Bias = (*Entry).TimezoneInfo.Bias;
    TimeZoneInformation.StandardBias = (*Entry).TimezoneInfo.StandardBias;
    TimeZoneInformation.DaylightBias = (*Entry).TimezoneInfo.DaylightBias;

    memcpy(
        core::ptr::addr_of_mut!(TimeZoneInformation.StandardDate) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*Entry).TimezoneInfo.StandardDate) as *const core::ffi::c_void,
        core::mem::size_of::<SYSTEMTIME>() as usize,
    );
    memcpy(
        core::ptr::addr_of_mut!(TimeZoneInformation.DaylightDate) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*Entry).TimezoneInfo.DaylightDate) as *const core::ffi::c_void,
        core::mem::size_of::<SYSTEMTIME>() as usize,
    );

    /* Set time zone information */
    SetTimeZoneInformation(core::ptr::addr_of_mut!(TimeZoneInformation));
}
#[no_mangle]
pub unsafe extern "system" fn TimeZonePageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut bitmap: BITMAP = core::mem::zeroed();

    match uMsg {
        WM_INITDIALOG => {
            CreateTimeZoneList();
            ShowTimeZoneList(GetDlgItem(hwndDlg, (IDC_TIMEZONELIST as i32)));

            /* Update DST checkbox based on the selected timezone's DST support */
            UpdateDstCheckbox(hwndDlg);

            hBitmap = LoadImageW(
                hApplet,
                MAKEINTRESOURCEW!(IDC_WORLD) as LPCWSTR,
                IMAGE_BITMAP as UINT,
                0 as core::ffi::c_int,
                0 as core::ffi::c_int,
                LR_DEFAULTCOLOR as UINT,
            ) as HBITMAP;
            if !hBitmap.is_null() {
                GetObjectW(
                    hBitmap as HANDLE,
                    core::mem::size_of::<BITMAP>() as core::ffi::c_int,
                    core::ptr::addr_of_mut!(bitmap) as LPVOID,
                );

                cxSource = bitmap.bmWidth;
                cySource = bitmap.bmHeight;
            }
        }

        WM_DRAWITEM => {
            let mut lpDrawItem: LPDRAWITEMSTRUCT = core::ptr::null_mut();
            lpDrawItem = lParam as LPDRAWITEMSTRUCT;
            if (*lpDrawItem).CtlID == IDC_WORLD_BACKGROUND {
                let mut hdcMem: HDC = core::ptr::null_mut();
                hdcMem = CreateCompatibleDC((*lpDrawItem).hDC);
                if !hdcMem.is_null() {
                    SelectObject(hdcMem, hBitmap as HGDIOBJ);
                    StretchBlt(
                        (*lpDrawItem).hDC,
                        (*lpDrawItem).rcItem.left,
                        (*lpDrawItem).rcItem.top,
                        (*lpDrawItem).rcItem.right - (*lpDrawItem).rcItem.left,
                        (*lpDrawItem).rcItem.bottom - (*lpDrawItem).rcItem.top,
                        hdcMem,
                        0,
                        0,
                        cxSource,
                        cySource,
                        SRCCOPY as DWORD,
                    );
                    DeleteDC(hdcMem);
                }
            }
        }

        WM_COMMAND => {
            if LOWORD!(wParam) as UINT == IDC_TIMEZONELIST as UINT
                && HIWORD!(wParam) as UINT == CBN_SELCHANGE as UINT
            {
                /* Update DST checkbox based on the selected timezone's DST support */
                UpdateDstCheckbox(hwndDlg);
                /* Enable the 'Apply' button */
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) as UINT == IDC_AUTODAYLIGHT as UINT
                && HIWORD!(wParam) as UINT == BN_CLICKED as UINT
            {
                /* Enable the 'Apply' button */
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            }
        }

        WM_DESTROY => {
            DestroyTimeZoneList();
            DeleteObject(hBitmap as HGDIOBJ);
        }

        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_APPLY => {
                    SetAutoDaylight(
                        (SendDlgItemMessage(
                            hwndDlg,
                            (IDC_AUTODAYLIGHT as i32),
                            BM_GETCHECK,
                            0 as WPARAM,
                            0 as LPARAM,
                        ) != BST_UNCHECKED as LRESULT) as core::ffi::c_int,
                    );
                    SetLocalTimeZone(GetDlgItem(hwndDlg, (IDC_TIMEZONELIST as i32)));
                    SetWindowLongPtr(
                        hwndDlg,
                        DWLP_MSGRESULT,
                        PSNRET_NOERROR as LONG_PTR,
                    );
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
