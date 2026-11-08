use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szDontShowKey: [WCHAR; 77] = [
    'S' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];


#[repr(C)]
pub struct tagDLGDATAEX {
    pub dlgProc: DLGPROC,
    pub lParam: LPARAM,
    pub lpszId: LPCWSTR,
}
pub type DLGDATAEX = tagDLGDATAEX;

#[repr(C)]
pub struct tagDLGDATA {
    pub lpszTitle: LPCWSTR,
    pub lpszText: LPCWSTR,
    pub dwType: DWORD,
}
pub type DLGDATA = tagDLGDATA;

unsafe extern "system" fn SHDlgProcEx(
    hDlg: HWND,
    uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut d: *mut DLGDATAEX = GetWindowLongPtrW(hDlg, DWLP_USER) as *mut DLGDATAEX;

    if uMsg == WM_INITDIALOG {
        /* FIXME: Not sure where native stores its lParam */
        SetWindowLongPtrW(hDlg, DWLP_USER, lParam as LONG_PTR);
        d = lParam as *mut DLGDATAEX;

        if (*d).dlgProc.is_some() {
            return (*d).dlgProc.unwrap_unchecked()(hDlg, uMsg, wParam, (*d).lParam);
        }
        return (TRUE) as INT_PTR;
    }

    if uMsg == WM_COMMAND {
        // switch (LOWORD(wParam)) with fall-through, lowered as a dispatch loop.
        let mut at: u8 = if LOWORD!(wParam) == (IDYES as u16) {
            0
        } else if LOWORD!(wParam) == (IDNO as u16) {
            1
        } else if LOWORD!(wParam) == (IDOK as u16) || LOWORD!(wParam) == (IDCANCEL as u16) {
            2
        } else {
            3
        };
        'sw: loop {
            match at {
                0 => {
                    wParam = MAKELONG!(IDOK, HIWORD!(wParam)) as WPARAM;
                    /* Fall through ... */
                    at = 1;
                    continue;
                }
                1 => {
                    if LOWORD!(wParam) == (IDNO as u16) {
                        wParam = MAKELONG!(IDCANCEL, HIWORD!(wParam)) as WPARAM;
                    }
                    /* Fall through ... */
                    at = 2;
                    continue;
                }
                2 => {
                    TRACE!(
                        "WM_COMMAND: id=%s data=%p\n",
                        if LOWORD!(wParam) == (IDOK as u16) { "IDOK" } else { "IDCANCEL" },
                        d
                    );

                    if SendMessageW(
                        GetDlgItem(hDlg, (IDC_ERR_DONT_SHOW as i32)),
                        BM_GETCHECK,
                        0,
                        0,
                    ) != 0
                    {
                        let mut dwZero: DWORD = 0;

                        /* The user clicked 'don't show again', so set the key */
                        SHRegSetUSValueW(
                            core::ptr::addr_of!(szDontShowKey) as LPCWSTR,
                            (*d).lpszId,
                            REG_DWORD,
                            core::ptr::addr_of_mut!(dwZero) as LPVOID,
                            core::mem::size_of::<DWORD>() as DWORD,
                            SHREGSET_DEFAULT,
                        );
                    }
                    if (*d).dlgProc.is_none()
                        || (*d).dlgProc.unwrap_unchecked()(hDlg, uMsg, wParam, lParam) == 0
                    {
                        EndDialog(hDlg, (wParam) as INT_PTR);
                    }
                    return (TRUE) as INT_PTR;
                }
                _ => break 'sw,
            }
        }
    }

    if !d.is_null() && (*d).dlgProc.is_some() {
        return (*d).dlgProc.unwrap_unchecked()(hDlg, uMsg, wParam, lParam);
    }
    return (FALSE) as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn SHMessageBoxCheckExA(
    hWnd: HWND,
    hInst: HINSTANCE,
    lpszName: LPCSTR,
    dlgProc: DLGPROC,
    lParam: LPARAM,
    iRet: INT_PTR,
    lpszId: LPCSTR,
) -> INT_PTR {
    let mut szNameBuff: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szIdBuff: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szName: LPCWSTR = core::ptr::addr_of_mut!(szNameBuff) as *mut WCHAR as LPCWSTR;

    if IS_INTRESOURCE!(lpszName) {
        szName = lpszName as LPCWSTR; /* Resource Id or NULL */
    } else {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszName,
            -(1 as ::core::ffi::c_int),
            core::ptr::addr_of_mut!(szNameBuff) as LPWSTR,
            MAX_PATH as ::core::ffi::c_int,
        );
    }

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        lpszId,
        -(1 as ::core::ffi::c_int),
        core::ptr::addr_of_mut!(szIdBuff) as LPWSTR,
        MAX_PATH as ::core::ffi::c_int,
    );

    return SHMessageBoxCheckExW(
        hWnd,
        hInst,
        szName,
        dlgProc,
        lParam,
        iRet,
        core::ptr::addr_of_mut!(szIdBuff) as *mut WCHAR as LPCWSTR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHMessageBoxCheckExW(hWnd: HWND, hInst: HINSTANCE, lpszName: LPCWSTR, dlgProc: DLGPROC, lParam: LPARAM, iRet: INT_PTR, lpszId: LPCWSTR) -> INT_PTR {
    let mut d: DLGDATAEX = core::mem::zeroed();

    if SHRegGetBoolUSValueW(core::ptr::addr_of!(szDontShowKey) as LPCWSTR, lpszId, (FALSE) as BOOL, (TRUE) as BOOL) == 0 {
        return iRet;
    }

    d.dlgProc = dlgProc;
    d.lParam = lParam;
    d.lpszId = lpszId;
    return DialogBoxParamW(hInst, lpszName, hWnd, Some(SHDlgProcEx), core::ptr::addr_of_mut!(d) as LPARAM);
}

unsafe extern "system" fn SHDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    TRACE!("(%p,%u,%ld,%ld)\n", hDlg, uMsg, wParam, lParam);
    match uMsg {
        WM_INITDIALOG => {
            let d: *mut DLGDATA = lParam as *mut DLGDATA;
            TRACE!(
                "WM_INITDIALOG: %p, %s,%s,%d\n",
                hDlg,
                debugstr_w((*d).lpszTitle as *const WCHAR),
                debugstr_w((*d).lpszText as *const WCHAR),
                (*d).dwType
            );
            SetWindowTextW(hDlg, (*d).lpszTitle);
            SetWindowTextW(GetDlgItem(hDlg, (IDS_ERR_USER_MSG as i32)), (*d).lpszText);

            /* Set buttons according to dwType */
            match (*d).dwType {
                0 => {
                    ShowWindow(GetDlgItem(hDlg, (IDCANCEL as i32)), (SW_HIDE as i32));
                    /* FIXME: Move OK button to position of the Cancel button (cosmetic) */
                    ShowWindow(GetDlgItem(hDlg, (IDYES as i32)), (SW_HIDE as i32));
                    ShowWindow(GetDlgItem(hDlg, (IDNO as i32)), (SW_HIDE as i32));
                }
                1 => {
                    ShowWindow(GetDlgItem(hDlg, (IDYES as i32)), (SW_HIDE as i32));
                    ShowWindow(GetDlgItem(hDlg, (IDNO as i32)), (SW_HIDE as i32));
                }
                _ => {
                    ShowWindow(GetDlgItem(hDlg, (IDOK as i32)), (SW_HIDE as i32));
                    ShowWindow(GetDlgItem(hDlg, (IDCANCEL as i32)), (SW_HIDE as i32));
                }
            }
            return TRUE as INT_PTR;
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn SHMessageBoxCheckA(hWnd: HWND, lpszText: LPCSTR, lpszTitle: LPCSTR, dwType: DWORD, iRet: INT_PTR, lpszId: LPCSTR) -> INT_PTR {
    let mut szTitleBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szIdBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szTextBuff: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut iLen: core::ffi::c_int = 0;
    let mut iRetVal: INT_PTR = 0;

    if !lpszTitle.is_null() {
        MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, lpszTitle, -(1 as core::ffi::c_int), szTitleBuff.as_mut_ptr() as LPWSTR, MAX_PATH as core::ffi::c_int);
    }

    if !lpszText.is_null() {
        iLen = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, lpszText, -(1 as core::ffi::c_int), core::ptr::null_mut::<WCHAR>(), 0 as core::ffi::c_int);
        szTextBuff = HeapAlloc(GetProcessHeap(), 0 as DWORD, (iLen as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as *mut WCHAR;
        MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, lpszText, -(1 as core::ffi::c_int), szTextBuff as LPWSTR, iLen);
    }

    MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, lpszId, -(1 as core::ffi::c_int), szIdBuff.as_mut_ptr() as LPWSTR, MAX_PATH as core::ffi::c_int);

    iRetVal = SHMessageBoxCheckW(hWnd, szTextBuff as LPCWSTR, (if !lpszTitle.is_null() { szTitleBuff.as_mut_ptr() } else { core::ptr::null_mut::<WCHAR>() }) as LPCWSTR, dwType, iRet, szIdBuff.as_mut_ptr() as LPCWSTR);
    HeapFree(GetProcessHeap(), 0 as DWORD, szTextBuff as PVOID);
    return iRetVal;
}

#[no_mangle]
pub unsafe extern "system" fn SHMessageBoxCheckW(
    hWnd: HWND,
    lpszText: LPCWSTR,
    lpszTitle: LPCWSTR,
    dwType: DWORD,
    iRet: INT_PTR,
    lpszId: LPCWSTR,
) -> INT_PTR {
    let mut d: DLGDATA = DLGDATA {
        lpszTitle: core::ptr::null(),
        lpszText: core::ptr::null(),
        dwType: 0,
    };

    d.lpszTitle = lpszTitle;
    d.lpszText = lpszText;
    d.dwType = dwType;

    return SHMessageBoxCheckExW(
        hWnd,
        shlwapi_hInstance,
        IDD_ERR_DIALOG as LPCWSTR,
        Some(SHDlgProc),
        core::ptr::addr_of_mut!(d) as LPARAM,
        iRet,
        lpszId,
    );
}
