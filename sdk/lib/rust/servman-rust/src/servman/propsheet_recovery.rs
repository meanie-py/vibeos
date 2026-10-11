use crate::servman::misc::AllocAndLoadString;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_3AAC72956FE8: [u16; 10] = [0x002Fu16, 0x0066, 0x0061, 0x0069, 0x006C, 0x003D, 0x0025, 0x0031, 0x0025, 0x0000];
pub static RUSTCONV_WSTR_4F8320D91E97: [u16; 2] = [0x0030u16, 0x0000];
pub static RUSTCONV_WSTR_60864AAE2645: [u16; 2] = [0x0031u16, 0x0000];
pub static RUSTCONV_WSTR_63FBF55E5299: [u16; 68] = [
    0x0045u16, 0x0078, 0x0065, 0x0063, 0x0075, 0x0074, 0x0061, 0x0062, 0x006C, 0x0065,
    0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0020, 0x0028, 0x002A, 0x002E,
    0x0065, 0x0078, 0x0065, 0x003B, 0x002A, 0x002E, 0x0063, 0x006F, 0x006D, 0x003B,
    0x002A, 0x002E, 0x0063, 0x006D, 0x0064, 0x003B, 0x002A, 0x002E, 0x0062, 0x0061,
    0x0074, 0x0029, 0x0000, 0x002A, 0x002E, 0x0065, 0x0078, 0x0065, 0x003B, 0x002A,
    0x002E, 0x0063, 0x006F, 0x006D, 0x003B, 0x002A, 0x002E, 0x0063, 0x006D, 0x0064,
    0x003B, 0x002A, 0x002E, 0x0062, 0x0061, 0x0074, 0x0000, 0x0000,
];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _RECOVERYDATA {
    pub pService: *mut ENUM_SERVICE_STATUS_PROCESS,
    pub pServiceFailure: LPSERVICE_FAILURE_ACTIONS,
    pub bChanged: BOOL,
}

pub type RECOVERYDATA = _RECOVERYDATA;
pub type PRECOVERYDATA = *mut _RECOVERYDATA;

unsafe extern "C" fn InitRecoveryPage(mut hwndDlg: HWND) {
    let mut lpAction: LPWSTR = core::ptr::null_mut();
    let mut id: INT = 0;
    id = IDS_NO_ACTION as INT;
    while id <= IDS_RESTART_COMPUTER as INT {
        if AllocAndLoadString(core::ptr::addr_of_mut!(lpAction), hInstance, id as UINT) != 0 {
            SendDlgItemMessageW(
                hwndDlg,
                IDC_FIRST_FAILURE as INT,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                lpAction as LPARAM,
            );
            SendDlgItemMessageW(
                hwndDlg,
                IDC_SECOND_FAILURE as INT,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                lpAction as LPARAM,
            );
            SendDlgItemMessageW(
                hwndDlg,
                IDC_SUBSEQUENT_FAILURES as INT,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                lpAction as LPARAM,
            );
            LocalFree(lpAction as HLOCAL);
        }
        id += 1;
    }

    SendDlgItemMessageW(
        hwndDlg,
        IDC_FIRST_FAILURE as INT,
        CB_SETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_SECOND_FAILURE as INT,
        CB_SETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_SUBSEQUENT_FAILURES as INT,
        CB_SETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_RESET_TIME as INT,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        RUSTCONV_WSTR_4F8320D91E97.as_ptr() as LPARAM,
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_RESTART_TIME as INT,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        RUSTCONV_WSTR_60864AAE2645.as_ptr() as LPARAM,
    );

    id = IDC_RESTART_TEXT1 as INT;
    while id <= IDC_RESTART_OPTIONS as INT {
        EnableWindow(
            GetDlgItem(hwndDlg, id),
            FALSE as INT,
        );
        id += 1;
    }
}

unsafe extern "C" fn GetServiceFailure(pRecoveryData: PRECOVERYDATA) -> BOOL {
    let mut pServiceFailure: LPSERVICE_FAILURE_ACTIONS = core::ptr::null_mut();
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut cbBytesNeeded: DWORD = 0;

    'done: {
        hManager = OpenSCManager(
            core::ptr::null(),
            core::ptr::null(),
            SC_MANAGER_CONNECT,
        );
        if hManager.is_null() {
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(
            hManager,
            (*(*pRecoveryData).pService).lpServiceName as LPCWSTR,
            SERVICE_QUERY_CONFIG,
        );
        if hService.is_null() {
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceConfig2(
            hService,
            SERVICE_CONFIG_FAILURE_ACTIONS,
            core::ptr::null_mut(),
            0,
            &raw mut cbBytesNeeded,
        ) == 0
        {
            if cbBytesNeeded == 0 {
                bResult = (FALSE) as i32;
                break 'done;
            }
        }

        pServiceFailure = HeapAlloc(
            GetProcessHeap(),
            0,
            cbBytesNeeded as SIZE_T,
        ) as LPSERVICE_FAILURE_ACTIONS;
        if pServiceFailure.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceConfig2(
            hService,
            SERVICE_CONFIG_FAILURE_ACTIONS,
            pServiceFailure as LPBYTE,
            cbBytesNeeded,
            &raw mut cbBytesNeeded,
        ) == 0
        {
            bResult = (FALSE) as i32;
            break 'done;
        }

        (*pRecoveryData).pServiceFailure = pServiceFailure;
    }

    // done:
    if (bResult) as u32 == FALSE && !pServiceFailure.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pServiceFailure as PVOID,
        );
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    return bResult;
}

unsafe extern "C" fn ShowFailureActions(mut hwndDlg: HWND, mut pRecoveryData: PRECOVERYDATA) {
    let mut szBuffer: [WCHAR; 256] = [0; 256];
    let mut startPtr: PWSTR = core::ptr::null_mut();
    let mut endPtr: PWSTR = core::ptr::null_mut();
    let mut index: INT = 0;
    let mut id: INT = 0;
    let mut length: INT = 0;
    let mut i: DWORD = 0;
    i = 0;
    while i < min!((*(*pRecoveryData).pServiceFailure).cActions, 3) {
        index = -(1);
        match (*(*(*pRecoveryData).pServiceFailure).lpsaActions.offset(i as isize)).Type {
            SC_ACTION_NONE => {
                index = 0;
            }
            SC_ACTION_RESTART => {
                index = 1;

                wsprintf(
                    szBuffer.as_mut_ptr(),
                    RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
                    (*(*(*pRecoveryData).pServiceFailure).lpsaActions.offset(i as isize)).Delay / 60000,
                );
                SendDlgItemMessageW(
                    hwndDlg,
                    (IDC_RESTART_TIME as i32),
                    WM_SETTEXT,
                    0,
                    szBuffer.as_mut_ptr() as LPARAM,
                );

                id = (IDC_RESTART_TEXT1 as i32);
                while id <= (IDC_RESTART_TEXT2 as i32) {
                    EnableWindow(GetDlgItem(hwndDlg, id), (TRUE) as BOOL);
                    id += 1;
                }
            }
            SC_ACTION_REBOOT => {
                index = 3;

                EnableWindow(GetDlgItem(hwndDlg, (IDC_RESTART_OPTIONS as i32)), (TRUE) as BOOL);
            }
            SC_ACTION_RUN_COMMAND => {
                index = 2;

                id = (IDC_RUN_GROUPBOX as i32);
                while id <= (IDC_ADD_FAILCOUNT as i32) {
                    EnableWindow(GetDlgItem(hwndDlg, id), (TRUE) as BOOL);
                    id += 1;
                }
            }
            _ => {}
        }

        if index != -(1) {
            SendDlgItemMessageW(
                hwndDlg,
                (IDC_FIRST_FAILURE as DWORD).wrapping_add(i) as INT,
                CB_SETCURSEL,
                index as WPARAM,
                0,
            );
        }
        i = i.wrapping_add(1);
    }

    wsprintf(
        szBuffer.as_mut_ptr(),
        RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
        (*(*pRecoveryData).pServiceFailure).dwResetPeriod / 86400,
    );
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_RESET_TIME as i32),
        WM_SETTEXT,
        0,
        szBuffer.as_mut_ptr() as LPARAM,
    );

    if !(*(*pRecoveryData).pServiceFailure).lpCommand.is_null() {
        ZeroMemory!(
            szBuffer.as_mut_ptr() as *mut core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 256]>(),
        );

        startPtr = (*(*pRecoveryData).pServiceFailure).lpCommand;
        if *startPtr == 0x0022u16 {
            startPtr = startPtr.offset(1);
        }

        endPtr = wcschr(startPtr, 0x0022u16);
        if !endPtr.is_null() {
            length = ((endPtr as LONG_PTR) - (startPtr as LONG_PTR)) as INT;
            CopyMemory!(
                szBuffer.as_mut_ptr() as *mut core::ffi::c_void,
                startPtr as *const core::ffi::c_void,
                length as usize,
            );
        } else {
            wcscpy(szBuffer.as_mut_ptr(), startPtr);
        }

        SendDlgItemMessageW(
            hwndDlg,
            (IDC_PROGRAM as i32),
            WM_SETTEXT,
            0,
            szBuffer.as_mut_ptr() as LPARAM,
        );

        ZeroMemory!(
            szBuffer.as_mut_ptr() as *mut core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 256]>(),
        );

        if !endPtr.is_null() {
            startPtr = endPtr.offset(1 as isize);
            while iswspace(*startPtr) != 0 {
                startPtr = startPtr.offset(1);
            }

            endPtr = wcsstr((*(*pRecoveryData).pServiceFailure).lpCommand, RUSTCONV_WSTR_3AAC72956FE8.as_ptr());
            if !endPtr.is_null() {
                while iswspace(*endPtr.offset(-(1 as isize))) != 0 {
                    endPtr = endPtr.offset(-1);
                }

                length = ((endPtr as LONG_PTR) - (startPtr as LONG_PTR)) as INT;
                CopyMemory!(
                    szBuffer.as_mut_ptr() as *mut core::ffi::c_void,
                    startPtr as *const core::ffi::c_void,
                    length as usize,
                );
            } else {
                wcscpy(szBuffer.as_mut_ptr(), startPtr);
            }

            SendDlgItemMessageW(
                hwndDlg,
                (IDC_PARAMETERS as i32),
                WM_SETTEXT,
                0,
                szBuffer.as_mut_ptr() as LPARAM,
            );

            endPtr = wcsstr((*(*pRecoveryData).pServiceFailure).lpCommand, RUSTCONV_WSTR_3AAC72956FE8.as_ptr());
            if !endPtr.is_null() {
                SendDlgItemMessageW(
                    hwndDlg,
                    (IDC_ADD_FAILCOUNT as i32),
                    BM_SETCHECK,
                    (BST_CHECKED) as WPARAM,
                    0,
                );
            }
        }
    }
}

unsafe extern "C" fn UpdateFailureActions(mut hwndDlg: HWND, mut pRecoveryData: PRECOVERYDATA) {
    let mut id: INT = 0;
    let mut index: INT = 0;
    let mut bRestartService: BOOL = (FALSE) as i32;
    let mut bRunProgram: BOOL = (FALSE) as i32;
    let mut bRebootComputer: BOOL = (FALSE) as i32;
    id = (IDC_FIRST_FAILURE as i32);
    while id <= (IDC_SUBSEQUENT_FAILURES as i32) {
        index = SendDlgItemMessageW(
            hwndDlg,
            id,
            CB_GETCURSEL,
            0,
            0,
        ) as INT;
        match index {
            1 => {
                bRestartService = (TRUE) as i32;
            }
            2 => {
                bRunProgram = (TRUE) as i32;
            }
            3 => {
                bRebootComputer = (TRUE) as i32;
            }
            _ => {}
        }
        id += 1;
    }
    id = (IDC_RESTART_TEXT1 as i32);
    while id <= (IDC_RESTART_TEXT2 as i32) {
        EnableWindow(GetDlgItem(hwndDlg, id), bRestartService);
        id += 1;
    }
    id = (IDC_RUN_GROUPBOX as i32);
    while id <= (IDC_ADD_FAILCOUNT as i32) {
        EnableWindow(GetDlgItem(hwndDlg, id), bRunProgram);
        id += 1;
    }
    EnableWindow(GetDlgItem(hwndDlg, (IDC_RESTART_OPTIONS as i32)), bRebootComputer);
}

unsafe extern "C" fn BrowseFile(hwndDlg: HWND) {
    let mut szFile: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut pszFilter: PWCHAR = RUSTCONV_WSTR_63FBF55E5299.as_ptr() as PWCHAR;
    let mut ofn: OPENFILENAME = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(ofn) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAME>(),
    );

    ofn.lStructSize = core::mem::size_of::<OPENFILENAME>() as DWORD;
    ofn.Flags = (OFN_EXPLORER | OFN_HIDEREADONLY | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_ENABLESIZING) as DWORD;
    ofn.hwndOwner = hwndDlg;
    ofn.lpstrFilter = pszFilter as LPCWSTR;
    ofn.lpstrFile = (core::ptr::addr_of_mut!(szFile) as *mut WCHAR) as LPWSTR;
    ofn.nMaxFile = MAX_PATH as DWORD;

    if GetOpenFileName(core::ptr::addr_of_mut!(ofn)) != 0 {
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_PROGRAM as i32),
            WM_SETTEXT,
            0,
            (core::ptr::addr_of_mut!(szFile) as *mut WCHAR) as LPARAM,
        );
    }
}

unsafe extern "C" fn SetFailureActions(hwndDlg: HWND) {
    let mut FailureActions: SERVICE_FAILURE_ACTIONS = core::mem::zeroed();
    let mut bRestartService: BOOL = (FALSE) as i32;
    let mut bRunProgram: BOOL = (FALSE) as i32;
    let mut bRebootComputer: BOOL = (FALSE) as i32;
    let mut id: INT;
    let mut index: INT;

    ZeroMemory!(
        core::ptr::addr_of_mut!(FailureActions) as *mut core::ffi::c_void,
        core::mem::size_of::<SERVICE_FAILURE_ACTIONS>(),
    );

    /* Count the number of valid failure actions */
    id = (IDC_FIRST_FAILURE as i32);
    while id <= (IDC_SUBSEQUENT_FAILURES as i32) {
        index = SendDlgItemMessageW(
            hwndDlg,
            id,
            CB_GETCURSEL,
            0,
            0,
        ) as INT;
        match index {
            1 => {
                /* Restart Service */
                bRestartService = (TRUE) as i32;
                FailureActions.cActions = FailureActions.cActions.wrapping_add(1);
            }
            2 => {
                /* Run Program */
                bRunProgram = (TRUE) as i32;
                FailureActions.cActions = FailureActions.cActions.wrapping_add(1);
            }
            3 => {
                /* Reboot Computer */
                bRebootComputer = (TRUE) as i32;
                FailureActions.cActions = FailureActions.cActions.wrapping_add(1);
            }
            _ => {}
        }
        id += 1;
    }

    if bRestartService != 0 {
        // IDC_RESTART_TIME
    }

    if bRunProgram != 0 {
        // IDC_RESTART_TIME
    }

    if bRebootComputer != 0 {
        // IDC_RESTART_TIME
    }
}

#[no_mangle]
pub unsafe extern "system" fn RecoveryPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pRecoveryData: PRECOVERYDATA;

    /* Get the window context */
    pRecoveryData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PRECOVERYDATA;
    if pRecoveryData.is_null() && uMsg != WM_INITDIALOG {
        return FALSE as INT_PTR;
    }

    match uMsg {
        WM_INITDIALOG => {
            pRecoveryData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                ::core::mem::size_of::<RECOVERYDATA>() as SIZE_T,
            ) as PRECOVERYDATA;
            if !pRecoveryData.is_null() {
                SetWindowLongPtr(
                    hwndDlg,
                    GWLP_USERDATA,
                    pRecoveryData as LONG_PTR,
                );

                (*pRecoveryData).pService =
                    (*((*(lParam as LPPROPSHEETPAGE)).lParam as PSERVICEPROPSHEET)).pService;

                InitRecoveryPage(hwndDlg);

                if GetServiceFailure(pRecoveryData) != 0 {
                    ShowFailureActions(hwndDlg, pRecoveryData);
                }
            }
        }

        WM_DESTROY => {
            if !pRecoveryData.is_null() {
                if !(*pRecoveryData).pServiceFailure.is_null() {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        (*pRecoveryData).pServiceFailure as PVOID,
                    );
                }

                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    pRecoveryData as PVOID,
                );
            }
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_FIRST_FAILURE | IDC_SECOND_FAILURE | IDC_SUBSEQUENT_FAILURES => {
                    if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                        UpdateFailureActions(hwndDlg, pRecoveryData);
                        (*pRecoveryData).bChanged = TRUE as BOOL;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_RESET_TIME | IDC_RESTART_TIME | IDC_PROGRAM | IDC_PARAMETERS => {
                    if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                        (*pRecoveryData).bChanged = TRUE as BOOL;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_ADD_FAILCOUNT => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        (*pRecoveryData).bChanged = TRUE as BOOL;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_BROWSE_PROGRAM => {
                    BrowseFile(hwndDlg);
                }

                IDC_RESTART_OPTIONS => {}

                _ => {}
            }
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_APPLY => {
                    if (*pRecoveryData).bChanged != 0 {
                        SetFailureActions(hwndDlg);
                        (*pRecoveryData).bChanged = FALSE as BOOL;
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
