use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut Sec: [UINT; 16] = [
    60 as ::core::ffi::c_int as UINT,
    120 as ::core::ffi::c_int as UINT,
    180 as ::core::ffi::c_int as UINT,
    300 as ::core::ffi::c_int as UINT,
    600 as ::core::ffi::c_int as UINT,
    900 as ::core::ffi::c_int as UINT,
    1200 as ::core::ffi::c_int as UINT,
    1500 as ::core::ffi::c_int as UINT,
    1800 as ::core::ffi::c_int as UINT,
    2700 as ::core::ffi::c_int as UINT,
    3600 as ::core::ffi::c_int as UINT,
    7200 as ::core::ffi::c_int as UINT,
    10800 as ::core::ffi::c_int as UINT,
    14400 as ::core::ffi::c_int as UINT,
    18000 as ::core::ffi::c_int as UINT,
    0 as ::core::ffi::c_int as UINT,
];

#[repr(C)]
pub struct _POWER_SCHEME {
    pub ListEntry: _LIST_ENTRY,
    pub uId: u32,
    pub pszName: LPTSTR,
    pub pszDescription: LPTSTR,
    pub PowerPolicy: POWER_POLICY,
}
pub type POWER_SCHEME = _POWER_SCHEME;
pub type PPOWER_SCHEME = *mut _POWER_SCHEME;

#[repr(C)]
pub struct _POWER_SCHEMES_PAGE_DATA {
    pub PowerSchemesList: _LIST_ENTRY,
    pub pActivePowerScheme: PPOWER_SCHEME,
    pub pSelectedPowerScheme: PPOWER_SCHEME,
}
pub type POWER_SCHEMES_PAGE_DATA = _POWER_SCHEMES_PAGE_DATA;
pub type PPOWER_SCHEMES_PAGE_DATA = *mut _POWER_SCHEMES_PAGE_DATA;

#[repr(C)]
pub struct _SAVE_POWER_SCHEME_DATA {
    pub pPageData: PPOWER_SCHEMES_PAGE_DATA,
    pub pNewScheme: PPOWER_SCHEME,
    pub hwndPage: HWND,
}
pub type SAVE_POWER_SCHEME_DATA = _SAVE_POWER_SCHEME_DATA;
pub type PSAVE_POWER_SCHEME_DATA = *mut _SAVE_POWER_SCHEME_DATA;

unsafe extern "C" fn AddPowerScheme(
    pPageData: PPOWER_SCHEMES_PAGE_DATA,
    uId: UINT,
    dwName: DWORD,
    pszName: LPTSTR,
    dwDescription: DWORD,
    pszDescription: LPWSTR,
    pp: PPOWER_POLICY,
) -> PPOWER_SCHEME {
    let mut pScheme: PPOWER_SCHEME = core::ptr::null_mut();
    let mut bResult: BOOL = (FALSE) as i32;

    pScheme = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<POWER_SCHEME>() as SIZE_T,
    ) as PPOWER_SCHEME;
    if pScheme.is_null() {
        return core::ptr::null_mut();
    }

    (*pScheme).uId = uId;
    CopyMemory!(
        core::ptr::addr_of_mut!((*pScheme).PowerPolicy) as *mut core::ffi::c_void,
        pp as *const core::ffi::c_void,
        core::mem::size_of::<POWER_POLICY>() as SIZE_T,
    );

    'done: {
        if dwName != 0 {
            (*pScheme).pszName = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                dwName as SIZE_T,
            ) as LPTSTR;
            if (*pScheme).pszName.is_null() {
                break 'done;
            }

            _tcscpy(((*pScheme).pszName).cast(), pszName.cast());
        }

        if dwDescription != 0 {
            (*pScheme).pszDescription = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                dwDescription as SIZE_T,
            ) as LPTSTR;
            if (*pScheme).pszDescription.is_null() {
                break 'done;
            }

            _tcscpy(((*pScheme).pszDescription).cast(), pszDescription);
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*pPageData).PowerSchemesList),
            core::ptr::addr_of_mut!((*pScheme).ListEntry),
        );
        bResult = (TRUE) as i32;
    }

    // done:
    if (bResult) as u32 == FALSE {
        if !(*pScheme).pszName.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*pScheme).pszName as PVOID,
            );
        }

        if !(*pScheme).pszDescription.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*pScheme).pszDescription as PVOID,
            );
        }

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pScheme as PVOID,
        );
        pScheme = core::ptr::null_mut();
    }

    return pScheme;
}

unsafe extern "C" fn DeletePowerScheme(pScheme: PPOWER_SCHEME) {
    RemoveEntryList(core::ptr::addr_of_mut!((*pScheme).ListEntry));

    if !(*pScheme).pszName.is_null() {
        HeapFree(GetProcessHeap(), 0, (*pScheme).pszName as PVOID);
    }

    if !(*pScheme).pszDescription.is_null() {
        HeapFree(GetProcessHeap(), 0, (*pScheme).pszDescription as PVOID);
    }

    HeapFree(GetProcessHeap(), 0, pScheme as PVOID);
}

unsafe extern "system" fn EnumPowerSchemeCallback(
    uiIndex: UINT,
    dwName: DWORD,
    pszName: LPTSTR,
    dwDesc: DWORD,
    pszDesc: LPWSTR,
    pp: PPOWER_POLICY,
    lParam: LPARAM,
) -> BOOLEAN {
    if ValidatePowerPolicies(core::ptr::null_mut(), pp) != 0 {
        AddPowerScheme(
            lParam as PPOWER_SCHEMES_PAGE_DATA,
            uiIndex,
            dwName,
            pszName,
            dwDesc,
            pszDesc,
            pp,
        );
    }

    return (TRUE) as BOOLEAN;
}

unsafe extern "C" fn BuildSchemesList(pPageData: PPOWER_SCHEMES_PAGE_DATA) {
    InitializeListHead(core::ptr::addr_of_mut!((*pPageData).PowerSchemesList));

    unsafe extern "system" fn EnumPowerSchemeCallbackShim(
        a: u32,
        b: u32,
        name: *mut u16,
        c: u32,
        d: *mut u16,
        e: *mut _POWER_POLICY,
        f: LPARAM,
    ) -> u8 {
        unsafe { EnumPowerSchemeCallback(a, b, name, c, d, e, f) }
    }

    EnumPwrSchemes(
        Some(
            EnumPowerSchemeCallbackShim,
        ),
        pPageData as LPARAM,
    );
}

unsafe extern "C" fn DestroySchemesList(pPageData: PPOWER_SCHEMES_PAGE_DATA) {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pScheme: PPOWER_SCHEME = core::ptr::null_mut();

    loop {
        ListEntry = (*pPageData).PowerSchemesList.Flink;
        if ListEntry == core::ptr::addr_of_mut!((*pPageData).PowerSchemesList) {
            break;
        }

        pScheme = CONTAINING_RECORD!(ListEntry, POWER_SCHEME, ListEntry) as PPOWER_SCHEME;
        DeletePowerScheme(pScheme);
    }

    (*pPageData).pActivePowerScheme = core::ptr::null_mut();
    (*pPageData).pSelectedPowerScheme = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn Pos_InitData(mut hwndDlg: HWND) -> BOOLEAN {
    let mut spc: SYSTEM_POWER_CAPABILITIES = core::mem::zeroed();

    if GetPwrCapabilities(core::ptr::addr_of_mut!(spc)) == 0 {
        return (FALSE) as BOOLEAN;
    }

    ShowWindow(
        GetDlgItem(hwndDlg, (IDC_STANDBY as i32)),
        if IS_PWR_SUSPEND_ALLOWED!(core::ptr::addr_of_mut!(spc)) != 0 {
            (SW_SHOW) as i32
        } else {
            (SW_HIDE) as i32
        },
    );
    ShowWindow(
        GetDlgItem(hwndDlg, (IDC_STANDBYACLIST as i32)),
        if IS_PWR_SUSPEND_ALLOWED!(core::ptr::addr_of_mut!(spc)) != 0 {
            (SW_SHOW) as i32
        } else {
            (SW_HIDE) as i32
        },
    );
    if spc.SystemBatteriesPresent != 0 {
        ShowWindow(
            GetDlgItem(hwndDlg, (IDC_STANDBYDCLIST as i32)),
            if IS_PWR_SUSPEND_ALLOWED!(core::ptr::addr_of_mut!(spc)) != 0 {
                (SW_SHOW) as i32
            } else {
                (SW_HIDE) as i32
            },
        );
    }

    ShowWindow(
        GetDlgItem(hwndDlg, (IDC_HIBERNATE as i32)),
        if IS_PWR_HIBERNATE_ALLOWED!(core::ptr::addr_of_mut!(spc)) != 0 {
            (SW_SHOW) as i32
        } else {
            (SW_HIDE) as i32
        },
    );
    ShowWindow(
        GetDlgItem(hwndDlg, (IDC_HIBERNATEACLIST as i32)),
        if IS_PWR_HIBERNATE_ALLOWED!(core::ptr::addr_of_mut!(spc)) != 0 {
            (SW_SHOW) as i32
        } else {
            (SW_HIDE) as i32
        },
    );
    if spc.SystemBatteriesPresent != 0 {
        ShowWindow(
            GetDlgItem(hwndDlg, (IDC_HIBERNATEDCLIST as i32)),
            if IS_PWR_HIBERNATE_ALLOWED!(core::ptr::addr_of_mut!(spc)) != 0 {
                (SW_SHOW) as i32
            } else {
                (SW_HIDE) as i32
            },
        );
    }

    return (TRUE) as BOOLEAN;
}

unsafe extern "C" fn LoadConfig(
    hwndDlg: HWND,
    pPageData: PPOWER_SCHEMES_PAGE_DATA,
    mut pScheme: PPOWER_SCHEME,
) {
    let mut i: INT = 0 as INT;
    let mut iCurSel: INT = 0 as INT;
    let mut szTemp: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szConfig: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pp: PPOWER_POLICY;
    let mut hwndCtrl: HWND;

    iCurSel = SendDlgItemMessage(
        hwndDlg,
        IDC_ENERGYLIST as i32,
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    if iCurSel == CB_ERR as INT {
        return;
    }

    EnableWindow(
        GetDlgItem(hwndDlg, IDC_DELETE_BTN as i32),
        (iCurSel > 0 as INT) as i32,
    );

    if pScheme.is_null() {
        pScheme = SendDlgItemMessage(
            hwndDlg,
            IDC_ENERGYLIST as i32,
            CB_GETITEMDATA as UINT,
            iCurSel as WPARAM,
            0 as LPARAM,
        ) as PPOWER_SCHEME;
        if pScheme == CB_ERR as PPOWER_SCHEME {
            return;
        }
    }

    (*pPageData).pSelectedPowerScheme = pScheme;

    if LoadString(
        hApplet,
        IDS_CONFIG1 as UINT,
        (szTemp.as_mut_ptr()).cast(),
        _countof!(szTemp) as INT,
    ) != 0
    {
        _stprintf((szConfig.as_mut_ptr()).cast(), (szTemp.as_mut_ptr()).cast(), (*pScheme).pszName);
        SetWindowText(
            GetDlgItem(hwndDlg, IDC_GRPDETAIL as i32),
            (szConfig.as_mut_ptr()).cast(),
        );
    }

    pp = core::ptr::addr_of_mut!((*pScheme).PowerPolicy);

    i = 0 as INT;
    while i < 16 as INT {
        hwndCtrl = GetDlgItem(hwndDlg, IDC_MONITORACLIST as i32);
        if !hwndCtrl.is_null() && Sec[i as usize] as ULONG == (*pp).user.VideoTimeoutAc {
            SendMessage(hwndCtrl, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
        }

        hwndCtrl = GetDlgItem(hwndDlg, IDC_MONITORDCLIST as i32);
        if !hwndCtrl.is_null() && Sec[i as usize] as ULONG == (*pp).user.VideoTimeoutDc {
            SendMessage(hwndCtrl, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
        }

        hwndCtrl = GetDlgItem(hwndDlg, IDC_DISKACLIST as i32);
        if !hwndCtrl.is_null() && Sec[i as usize] as ULONG == (*pp).user.SpindownTimeoutAc {
            SendMessage(
                hwndCtrl,
                CB_SETCURSEL as UINT,
                (i - 2 as INT) as WPARAM,
                0 as LPARAM,
            );
        }

        hwndCtrl = GetDlgItem(hwndDlg, IDC_DISKDCLIST as i32);
        if !hwndCtrl.is_null() && Sec[i as usize] as ULONG == (*pp).user.SpindownTimeoutDc {
            SendMessage(
                hwndCtrl,
                CB_SETCURSEL as UINT,
                (i - 2 as INT) as WPARAM,
                0 as LPARAM,
            );
        }

        hwndCtrl = GetDlgItem(hwndDlg, IDC_STANDBYACLIST as i32);
        if !hwndCtrl.is_null() && Sec[i as usize] as ULONG == (*pp).user.IdleTimeoutAc {
            SendMessage(hwndCtrl, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
        }

        hwndCtrl = GetDlgItem(hwndDlg, IDC_STANDBYDCLIST as i32);
        if !hwndCtrl.is_null() && Sec[i as usize] as ULONG == (*pp).user.IdleTimeoutDc {
            SendMessage(hwndCtrl, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
        }

        hwndCtrl = GetDlgItem(hwndDlg, IDC_HIBERNATEACLIST as i32);
        if !hwndCtrl.is_null() && Sec[i as usize] as ULONG == (*pp).mach.DozeS4TimeoutAc {
            SendMessage(hwndCtrl, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
        }

        hwndCtrl = GetDlgItem(hwndDlg, IDC_HIBERNATEDCLIST as i32);
        if !hwndCtrl.is_null() && Sec[i as usize] as ULONG == (*pp).mach.DozeS4TimeoutDc {
            SendMessage(hwndCtrl, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
        }

        i += 1;
    }
}

unsafe extern "C" fn Pos_InitPage(hwndDlg: HWND) {
    let mut ifrom: c_int = 0;
    let mut i: c_int = 0;
    let mut imin: c_int = 0;
    let mut hwnd: HWND = core::ptr::null_mut();
    let mut szName: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut index: LRESULT;

    i = 1;
    while i < 9 {
        'body: {
            match i {
                1 => {
                    hwnd = GetDlgItem(hwndDlg, (IDC_MONITORACLIST as i32));
                    imin = (IDS_TIMEOUT1 as i32);
                }
                2 => {
                    hwnd = GetDlgItem(hwndDlg, (IDC_STANDBYACLIST as i32));
                    imin = (IDS_TIMEOUT1 as i32);
                }
                3 => {
                    hwnd = GetDlgItem(hwndDlg, (IDC_DISKACLIST as i32));
                    imin = (IDS_TIMEOUT3 as i32);
                }
                4 => {
                    hwnd = GetDlgItem(hwndDlg, (IDC_HIBERNATEACLIST as i32));
                    imin = (IDS_TIMEOUT3 as i32);
                }
                5 => {
                    hwnd = GetDlgItem(hwndDlg, (IDC_MONITORDCLIST as i32));
                    imin = (IDS_TIMEOUT1 as i32);
                }
                6 => {
                    hwnd = GetDlgItem(hwndDlg, (IDC_STANDBYDCLIST as i32));
                    imin = (IDS_TIMEOUT1 as i32);
                }
                7 => {
                    hwnd = GetDlgItem(hwndDlg, (IDC_DISKDCLIST as i32));
                    imin = (IDS_TIMEOUT3 as i32);
                }
                8 => {
                    hwnd = GetDlgItem(hwndDlg, (IDC_HIBERNATEDCLIST as i32));
                    imin = (IDS_TIMEOUT3 as i32);
                }
                _ => {
                    hwnd = core::ptr::null_mut();
                    return;
                }
            }

            if hwnd.is_null() {
                break 'body;
            }

            ifrom = imin;
            while ((ifrom) as u32) < IDS_TIMEOUT15 + 1 {
                if LoadString(
                    hApplet,
                    ifrom as UINT,
                    core::ptr::addr_of_mut!(szName) as LPWSTR,
                    _countof!(szName) as c_int,
                ) != 0
                {
                    index = SendMessage(
                        hwnd,
                        CB_ADDSTRING,
                        0,
                        core::ptr::addr_of_mut!(szName) as *mut TCHAR as LPARAM,
                    );
                    if index == (CB_ERR) as _ {
                        return;
                    }

                    SendMessage(
                        hwnd,
                        CB_SETITEMDATA,
                        index as WPARAM,
                        Sec[(ifrom - (IDS_TIMEOUT16 as i32)) as usize] as LPARAM,
                    );
                }
                ifrom += 1;
            }

            if LoadString(
                hApplet,
                IDS_TIMEOUT16 as UINT,
                core::ptr::addr_of_mut!(szName) as LPWSTR,
                _countof!(szName) as c_int,
            ) != 0
            {
                index = SendMessage(
                    hwnd,
                    CB_ADDSTRING,
                    0,
                    core::ptr::addr_of_mut!(szName) as *mut TCHAR as LPARAM,
                );
                if index == (CB_ERR) as _ {
                    return;
                }

                SendMessage(
                    hwnd,
                    CB_SETITEMDATA,
                    index as WPARAM,
                    Sec[0 as usize] as LPARAM,
                );
            }
        }
        i += 1;
    }
}

unsafe extern "C" fn Pos_SaveData(hwndDlg: HWND, pPageData: PPOWER_SCHEMES_PAGE_DATA) {
    let mut pScheme: PPOWER_SCHEME;
    let mut hwndCtrl: HWND;
    let mut tmp: INT;

    pScheme = (*pPageData).pSelectedPowerScheme;

    hwndCtrl = GetDlgItem(hwndDlg, (IDC_MONITORACLIST as i32));
    if hwndCtrl != core::ptr::null_mut() {
        tmp = SendMessage(hwndCtrl, CB_GETCURSEL, 0, 0) as INT;
        if tmp > 0 && tmp < 16 {
            (*pScheme).PowerPolicy.user.VideoTimeoutAc = Sec[tmp as usize] as ULONG;
        }
    }

    hwndCtrl = GetDlgItem(hwndDlg, (IDC_MONITORDCLIST as i32));
    if hwndCtrl != core::ptr::null_mut() {
        tmp = SendMessage(hwndCtrl, CB_GETCURSEL, 0, 0) as INT;
        if tmp > 0 && tmp < 16 {
            (*pScheme).PowerPolicy.user.VideoTimeoutDc = Sec[tmp as usize] as ULONG;
        }
    }

    hwndCtrl = GetDlgItem(hwndDlg, (IDC_DISKACLIST as i32));
    if hwndCtrl != core::ptr::null_mut() {
        tmp = SendMessage(hwndCtrl, CB_GETCURSEL, 0, 0) as INT;
        if tmp > 0 && tmp < 16 {
            (*pScheme).PowerPolicy.user.SpindownTimeoutAc = Sec[(tmp + 2) as usize] as ULONG;
        }
    }

    hwndCtrl = GetDlgItem(hwndDlg, (IDC_DISKDCLIST as i32));
    if hwndCtrl != core::ptr::null_mut() {
        tmp = SendMessage(hwndCtrl, CB_GETCURSEL, 0, 0) as INT;
        if tmp > 0 && tmp < 16 {
            (*pScheme).PowerPolicy.user.SpindownTimeoutDc = Sec[(tmp + 2) as usize] as ULONG;
        }
    }

    hwndCtrl = GetDlgItem(hwndDlg, (IDC_STANDBYACLIST as i32));
    if hwndCtrl != core::ptr::null_mut() {
        tmp = SendMessage(hwndCtrl, CB_GETCURSEL, 0, 0) as INT;
        if tmp > 0 && tmp < 16 {
            (*pScheme).PowerPolicy.user.IdleTimeoutAc = Sec[tmp as usize] as ULONG;
        }
    }

    hwndCtrl = GetDlgItem(hwndDlg, (IDC_STANDBYDCLIST as i32));
    if hwndCtrl != core::ptr::null_mut() {
        tmp = SendMessage(hwndCtrl, CB_GETCURSEL, 0, 0) as INT;
        if tmp > 0 && tmp < 16 {
            (*pScheme).PowerPolicy.user.IdleTimeoutDc = Sec[tmp as usize] as ULONG;
        }
    }

    hwndCtrl = GetDlgItem(hwndDlg, (IDC_HIBERNATEACLIST as i32));
    if hwndCtrl != core::ptr::null_mut() {
        tmp = SendMessage(hwndCtrl, CB_GETCURSEL, 0, 0) as INT;
        if tmp > 0 && tmp < 16 {
            (*pScheme).PowerPolicy.mach.DozeS4TimeoutAc = Sec[tmp as usize] as ULONG;
        }
    }

    hwndCtrl = GetDlgItem(hwndDlg, (IDC_HIBERNATEDCLIST as i32));
    if hwndCtrl != core::ptr::null_mut() {
        tmp = SendMessage(hwndCtrl, CB_GETCURSEL, 0, 0) as INT;
        if tmp > 0 && tmp < 16 {
            (*pScheme).PowerPolicy.mach.DozeS4TimeoutDc = Sec[tmp as usize] as ULONG;
        }
    }

    if SetActivePwrScheme((*pScheme).uId, core::ptr::null_mut(), core::ptr::addr_of_mut!((*pScheme).PowerPolicy)) != 0 {
        (*pPageData).pActivePowerScheme = pScheme;
    }
}

unsafe extern "C" fn DelScheme(hwnd: HWND, pPageData: PPOWER_SCHEMES_PAGE_DATA) -> BOOL {
    let mut szTitleBuffer: [WCHAR; 256] = [0; 256];
    let mut szRawBuffer: [WCHAR; 256] = [0; 256];
    let mut szCookedBuffer: [WCHAR; 512] = [0; 512];
    let mut iCurSel: INT = 0;
    let mut hList: HWND = core::ptr::null_mut();
    let mut pScheme: PPOWER_SCHEME = core::ptr::null_mut();
    let mut szErrorText: [WCHAR; 512] = [0; 512];

    hList = GetDlgItem(hwnd, (IDC_ENERGYLIST as i32));

    iCurSel = SendMessage(hList, CB_GETCURSEL, 0, 0) as INT;
    if iCurSel == CB_ERR {
        return (FALSE) as BOOL;
    }

    SendMessage(hList, CB_SETCURSEL, iCurSel as WPARAM, 0);

    pScheme = SendMessage(hList, CB_GETITEMDATA, iCurSel as WPARAM, 0) as PPOWER_SCHEME;
    if pScheme == CB_ERR as PPOWER_SCHEME {
        return (FALSE) as BOOL;
    }

    LoadStringW(hApplet, IDS_DEL_SCHEME_TITLE, core::ptr::addr_of_mut!(szTitleBuffer) as LPWSTR, _countof!(szTitleBuffer) as INT);
    LoadStringW(hApplet, IDS_DEL_SCHEME, core::ptr::addr_of_mut!(szRawBuffer) as LPWSTR, _countof!(szRawBuffer) as INT);
    StringCchPrintfW!(core::ptr::addr_of_mut!(szCookedBuffer) as LPWSTR, _countof!(szCookedBuffer) as size_t, core::ptr::addr_of_mut!(szRawBuffer) as *mut WCHAR as LPCWSTR, (*pScheme).pszName);

    if (MessageBoxW(hwnd, core::ptr::addr_of_mut!(szCookedBuffer) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitleBuffer) as *mut WCHAR as LPCWSTR, (MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2) as UINT)) as u32 == IDYES {
        if DeletePwrScheme((*pScheme).uId) == 0 {
            LoadStringW(hApplet, IDS_DEL_SCHEME_ERROR, core::ptr::addr_of_mut!(szErrorText) as LPWSTR, (core::mem::size_of::<[WCHAR; 512]>() / core::mem::size_of::<WCHAR>()) as INT);
            MessageBoxW(core::ptr::null_mut(), core::ptr::addr_of_mut!(szErrorText) as *mut WCHAR as LPCWSTR, core::ptr::null(), (MB_OK | MB_ICONERROR) as UINT);
            return (FALSE) as BOOL;
        }

        iCurSel = SendMessage(hList, CB_FINDSTRING, (-(1 as INT)) as WPARAM, (*pScheme).pszName as LPARAM) as INT;
        if iCurSel != CB_ERR {
            SendMessage(hList, CB_DELETESTRING, iCurSel as WPARAM, 0);
        }

        DeletePowerScheme(pScheme);

        iCurSel = SendMessage(hList, CB_FINDSTRING, (-(1 as INT)) as WPARAM, (*(*pPageData).pActivePowerScheme).pszName as LPARAM) as INT;
        if iCurSel != CB_ERR {
            SendMessage(hList, CB_SETCURSEL, iCurSel as WPARAM, 0);
        } else {
            SendMessage(hList, CB_SETCURSEL, 0 as WPARAM, 0);
        }

        LoadConfig(hwnd, pPageData, core::ptr::null_mut());
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn SavePowerScheme(
    hwndDlg: HWND,
    pSaveSchemeData: PSAVE_POWER_SCHEME_DATA,
) -> BOOL {
    let mut pPageData: PPOWER_SCHEMES_PAGE_DATA = core::ptr::null_mut();
    let mut pScheme: PPOWER_SCHEME = core::ptr::null_mut();
    let mut szSchemeName: [TCHAR; 512] = [0; 512];
    let mut bRet: BOOL = (FALSE) as i32;

    pPageData = (*pSaveSchemeData).pPageData;

    GetDlgItemText(
        hwndDlg,
        (IDC_SCHEMENAME as i32),
        (core::ptr::addr_of_mut!(szSchemeName) as LPTSTR).cast(),
        (_countof!(szSchemeName)) as i32,
    );

    pScheme = AddPowerScheme(
        pPageData,
        -(1 as ::core::ffi::c_int) as UINT,
        ((_tcslen((core::ptr::addr_of_mut!(szSchemeName) as *const TCHAR).cast()) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<TCHAR>() as usize)) as DWORD,
        core::ptr::addr_of_mut!(szSchemeName) as LPTSTR,
        core::mem::size_of::<TCHAR>() as DWORD,
        win32macros_rust::wide_bytes!(b"\0\0") as _, // TEXT("")
        core::ptr::addr_of_mut!((*(*pPageData).pSelectedPowerScheme).PowerPolicy),
    );
    if !pScheme.is_null() {
        if WritePwrScheme(
            core::ptr::addr_of_mut!((*pScheme).uId),
            ((*pScheme).pszName).cast(),
            ((*pScheme).pszDescription).cast(),
            core::ptr::addr_of_mut!((*pScheme).PowerPolicy),
        ) != 0
        {
            (*pSaveSchemeData).pNewScheme = pScheme;
            bRet = (TRUE) as i32;
        } else {
            DeletePowerScheme(pScheme);
        }
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "system" fn SaveSchemeDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pSaveSchemeData: PSAVE_POWER_SCHEME_DATA =
        GetWindowLongPtr(hwndDlg, DWLP_USER) as PSAVE_POWER_SCHEME_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pSaveSchemeData = lParam as PSAVE_POWER_SCHEME_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pSaveSchemeData as LONG_PTR);

            SetDlgItemText(
                hwndDlg,
                (IDC_SCHEMENAME as i32),
                (*(*(*pSaveSchemeData).pPageData).pSelectedPowerScheme).pszName as LPCWSTR,
            );
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    EndDialog(
                        hwndDlg,
                        SavePowerScheme(hwndDlg, pSaveSchemeData) as INT_PTR,
                    );
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, FALSE as INT_PTR);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "C" fn SaveScheme(hwndDlg: HWND, pPageData: PPOWER_SCHEMES_PAGE_DATA) {
    let mut SaveSchemeData: SAVE_POWER_SCHEME_DATA = core::mem::zeroed();
    let mut BackupPowerPolicy: POWER_POLICY = core::mem::zeroed();
    let mut hwndList: HWND = core::ptr::null_mut();
    let mut index: INT = 0;

    SaveSchemeData.pPageData = pPageData;
    SaveSchemeData.pNewScheme = core::ptr::null_mut();
    SaveSchemeData.hwndPage = hwndDlg;

    CopyMemory!(
        core::ptr::addr_of_mut!(BackupPowerPolicy) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*(*pPageData).pSelectedPowerScheme).PowerPolicy) as *const core::ffi::c_void,
        core::mem::size_of::<POWER_POLICY>() as usize,
    );

    Pos_SaveData(hwndDlg, pPageData);

    if DialogBoxParam(
        hApplet,
        MAKEINTRESOURCE!(IDD_SAVEPOWERSCHEME) as LPCWSTR,
        hwndDlg,
        Some(SaveSchemeDlgProc),
        core::ptr::addr_of_mut!(SaveSchemeData) as LPARAM,
    ) != 0
    {
        if !SaveSchemeData.pNewScheme.is_null() {
            hwndList = GetDlgItem(hwndDlg, (IDC_ENERGYLIST as i32));

            index = SendDlgItemMessage(
                hwndDlg,
                (IDC_ENERGYLIST as i32),
                CB_FINDSTRING as UINT,
                -(1 as INT) as WPARAM,
                (*SaveSchemeData.pNewScheme).pszName as LPARAM,
            ) as INT;

            if index == CB_ERR as INT {
                index = SendMessage(
                    hwndList,
                    CB_ADDSTRING as UINT,
                    0 as WPARAM,
                    (*SaveSchemeData.pNewScheme).pszName as LPARAM,
                ) as INT;
                if index != CB_ERR as INT {
                    SendMessage(
                        hwndList,
                        CB_SETITEMDATA as UINT,
                        index as WPARAM,
                        SaveSchemeData.pNewScheme as LPARAM,
                    );

                    SendMessage(
                        hwndList,
                        CB_SETCURSEL as UINT,
                        index as WPARAM,
                        0 as LPARAM,
                    );
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_DELETE_BTN as i32)),
                        (TRUE) as BOOL,
                    );
                }
            }
            LoadConfig(hwndDlg, pPageData, SaveSchemeData.pNewScheme);
        }
    }

    CopyMemory!(
        core::ptr::addr_of_mut!((*(*pPageData).pSelectedPowerScheme).PowerPolicy) as *mut core::ffi::c_void,
        core::ptr::addr_of!(BackupPowerPolicy) as *const core::ffi::c_void,
        core::mem::size_of::<POWER_POLICY>() as usize,
    );
}

unsafe extern "C" fn CreateEnergyList(
    mut hwndDlg: HWND,
    mut pPageData: PPOWER_SCHEMES_PAGE_DATA,
) -> BOOL {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut pScheme: PPOWER_SCHEME = core::ptr::null_mut();
    let mut index: INT = 0;
    let mut pp: POWER_POLICY = core::mem::zeroed();
    let mut spc: SYSTEM_POWER_CAPABILITIES = core::mem::zeroed();
    let mut hwndList: HWND = core::ptr::null_mut();
    let mut aps: UINT = 0;

    hwndList = GetDlgItem(hwndDlg, (IDC_ENERGYLIST as i32));

    if GetActivePwrScheme(core::ptr::addr_of_mut!(aps)) == 0 {
        return (FALSE) as BOOL;
    }

    if ReadGlobalPwrPolicy(core::ptr::addr_of_mut!(gGPP)) == 0 {
        return (FALSE) as BOOL;
    }

    if ReadPwrScheme(aps, core::ptr::addr_of_mut!(pp)) == 0 {
        return (FALSE) as BOOL;
    }

    if ValidatePowerPolicies(core::ptr::addr_of_mut!(gGPP), core::ptr::null_mut()) == 0 {
        return (FALSE) as BOOL;
    }

    if GetPwrCapabilities(core::ptr::addr_of_mut!(spc)) == 0 {
        return (FALSE) as BOOL;
    }

    if CanUserWritePwrScheme() != 0 {
        // TODO:
        // Enable write / delete powerscheme button
    }

    Pos_InitPage(hwndDlg);

    if GetActivePwrScheme(core::ptr::addr_of_mut!(aps)) == 0 {
        return (FALSE) as BOOL;
    }

    ListEntry = (*pPageData).PowerSchemesList.Flink as PLIST_ENTRY;
    while ListEntry != core::ptr::addr_of_mut!((*pPageData).PowerSchemesList) {
        pScheme = CONTAINING_RECORD!(ListEntry, POWER_SCHEME, ListEntry);

        index = SendMessage(
            hwndList,
            CB_ADDSTRING,
            0,
            (*pScheme).pszName as LPARAM,
        ) as INT;
        if index == CB_ERR {
            break;
        }

        SendMessage(
            hwndList,
            CB_SETITEMDATA,
            index as WPARAM,
            pScheme as LPARAM,
        );

        if aps == (*pScheme).uId {
            SendMessage(
                hwndList,
                CB_SELECTSTRING,
                TRUE as WPARAM,
                (*pScheme).pszName as LPARAM,
            );

            (*pPageData).pActivePowerScheme = pScheme;
            LoadConfig(hwndDlg, pPageData, pScheme);
        }

        ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
    }

    if SendMessage(
        hwndList,
        CB_GETCOUNT,
        0,
        0,
    ) > 0
    {
        EnableWindow(
            GetDlgItem(hwndDlg, (IDC_SAVEAS_BTN as i32)),
            (TRUE) as BOOL,
        );
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PowerSchemesDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pPageData: PPOWER_SCHEMES_PAGE_DATA = core::ptr::null_mut();

    pPageData = GetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int) as PPOWER_SCHEMES_PAGE_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pPageData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                ::core::mem::size_of::<POWER_SCHEMES_PAGE_DATA>() as SIZE_T,
            ) as PPOWER_SCHEMES_PAGE_DATA;
            SetWindowLongPtr(
                hwndDlg,
                DWLP_USER as ::core::ffi::c_int,
                pPageData as LONG_PTR,
            );

            BuildSchemesList(pPageData);

            if Pos_InitData(hwndDlg) == 0 {
                // TODO:
                // Initialization failed
                // Handle error
                let msg: [u16; 21] = [
                    'P' as u16, 'o' as u16, 's' as u16, '_' as u16, 'I' as u16, 'n' as u16,
                    'i' as u16, 't' as u16, 'D' as u16, 'a' as u16, 't' as u16, 'a' as u16,
                    ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16,
                    'd' as u16, '\n' as u16, 0,
                ];
                MessageBox(
                    hwndDlg,
                    msg.as_ptr(),
                    core::ptr::null_mut(),
                    MB_OK as UINT,
                );
            }

            if CreateEnergyList(hwndDlg, pPageData) == 0 {
                // TODO:
                // Initialization failed
                // Handle error
                let msg: [u16; 25] = [
                    'C' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16,
                    'E' as u16, 'n' as u16, 'e' as u16, 'r' as u16, 'g' as u16, 'y' as u16,
                    'L' as u16, 'i' as u16, 's' as u16, 't' as u16, ' ' as u16, 'f' as u16,
                    'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'd' as u16, '\n' as u16,
                    0,
                ];
                MessageBox(
                    hwndDlg,
                    msg.as_ptr(),
                    core::ptr::null_mut(),
                    MB_OK as UINT,
                );
            }
            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            if !pPageData.is_null() {
                DestroySchemesList(pPageData);
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    pPageData as PVOID,
                );
                SetWindowLongPtr(
                    hwndDlg,
                    DWLP_USER as ::core::ffi::c_int,
                    core::ptr::null_mut::<::core::ffi::c_void>() as LONG_PTR,
                );
            }
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_ENERGYLIST => {
                    if HIWORD!(wParam) as ::core::ffi::c_int == CBN_SELCHANGE as ::core::ffi::c_int {
                        LoadConfig(hwndDlg, pPageData, core::ptr::null_mut());
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_DELETE_BTN => {
                    DelScheme(hwndDlg, pPageData);
                }

                IDC_SAVEAS_BTN => {
                    SaveScheme(hwndDlg, pPageData);
                }

                IDC_MONITORACLIST
                | IDC_MONITORDCLIST
                | IDC_DISKACLIST
                | IDC_DISKDCLIST
                | IDC_STANDBYACLIST
                | IDC_STANDBYDCLIST
                | IDC_HIBERNATEACLIST
                | IDC_HIBERNATEDCLIST => {
                    if HIWORD!(wParam) as ::core::ffi::c_int == CBN_SELCHANGE as ::core::ffi::c_int {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_APPLY => {
                    Pos_SaveData(hwndDlg, pPageData);
                    return TRUE as INT_PTR;
                }

                PSN_SETACTIVE => {
                    Pos_InitData(hwndDlg);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
