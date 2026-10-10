use crate::servman::misc::GetError;
use crate::servman::query::GetServiceConfig;
use crate::servman::misc::ResourceMessageBox;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_7CEA81E6A9E5: [u16; 12] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
    0x006D, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const DEFAULT_PASSWORD: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b" \0 \0 \0 \0 \0 \0 \0 \0 \0 \0 \0 \0 \0 \0 \0\0\0",
    )
};

// NDEBUG (valueless object-like macro; no Rust const emitted)

#[repr(C)]
pub struct LOGONDATA {
    pub pService: *mut ENUM_SERVICE_STATUS_PROCESS,
    pub pServiceConfig: LPQUERY_SERVICE_CONFIG,
    pub szAccountName: [WCHAR; 64],
    pub szPassword1: [WCHAR; 64],
    pub szPassword2: [WCHAR; 64],
    pub nInteractive: INT,
    pub bInitialized: BOOL,
    pub bLocalSystem: BOOL,
    pub bAccountChanged: BOOL,
}

pub type PLOGONDATA = *mut LOGONDATA;

unsafe extern "C" fn SetControlStates(hwndDlg: HWND, pLogonData: PLOGONDATA, bLocalSystem: BOOL) {
    EnableWindow(GetDlgItem(hwndDlg, (IDC_LOGON_INTERACTIVE as i32)), bLocalSystem);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_LOGON_ACCOUNTNAME as i32)), (bLocalSystem == 0) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_LOGON_SEARCH as i32)), (FALSE) as BOOL);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_LOGON_PW1TEXT as i32)), (bLocalSystem == 0) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_LOGON_PASSWORD1 as i32)), (bLocalSystem == 0) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_LOGON_PW2TEXT as i32)), (bLocalSystem == 0) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_LOGON_PASSWORD2 as i32)), (bLocalSystem == 0) as i32);

    if bLocalSystem != 0 {
        SendDlgItemMessageW(hwndDlg, (IDC_LOGON_INTERACTIVE as i32), BM_SETCHECK, (*pLogonData).nInteractive as WPARAM, 0);

        if ((*pLogonData).bInitialized) as u32 == TRUE {
            GetDlgItemText(hwndDlg, (IDC_LOGON_ACCOUNTNAME as i32), core::ptr::addr_of_mut!((*pLogonData).szAccountName) as LPWSTR, 64);
            GetDlgItemText(hwndDlg, (IDC_LOGON_PASSWORD1 as i32), core::ptr::addr_of_mut!((*pLogonData).szPassword1) as LPWSTR, 64);
            GetDlgItemText(hwndDlg, (IDC_LOGON_PASSWORD2 as i32), core::ptr::addr_of_mut!((*pLogonData).szPassword2) as LPWSTR, 64);
        }

        SetDlgItemText(hwndDlg, (IDC_LOGON_ACCOUNTNAME as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
        SetDlgItemText(hwndDlg, (IDC_LOGON_PASSWORD1 as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
        SetDlgItemText(hwndDlg, (IDC_LOGON_PASSWORD2 as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    } else {
        if ((*pLogonData).bInitialized) as u32 == TRUE {
            (*pLogonData).nInteractive = SendDlgItemMessageW(hwndDlg, (IDC_LOGON_INTERACTIVE as i32), BM_GETCHECK, 0, 0) as INT;
        }
        SendDlgItemMessageW(hwndDlg, (IDC_LOGON_INTERACTIVE as i32), BM_SETCHECK, BST_UNCHECKED as WPARAM, 0);

        SetDlgItemText(hwndDlg, (IDC_LOGON_ACCOUNTNAME as i32), core::ptr::addr_of_mut!((*pLogonData).szAccountName) as *mut WCHAR as LPCWSTR);
        SetDlgItemText(hwndDlg, (IDC_LOGON_PASSWORD1 as i32), core::ptr::addr_of_mut!((*pLogonData).szPassword1) as *mut WCHAR as LPCWSTR);
        SetDlgItemText(hwndDlg, (IDC_LOGON_PASSWORD2 as i32), core::ptr::addr_of_mut!((*pLogonData).szPassword2) as *mut WCHAR as LPCWSTR);
    }

    (*pLogonData).bLocalSystem = bLocalSystem;
}

unsafe extern "C" fn SetServiceAccount(
    mut lpServiceName: LPWSTR,
    mut dwServiceType: DWORD,
    mut lpStartName: LPWSTR,
    mut lpPassword: LPWSTR,
) -> BOOL {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hSc: SC_HANDLE = core::ptr::null_mut();
    let mut scLock: SC_LOCK = core::ptr::null_mut();
    let mut bRet: BOOL = (FALSE) as i32;

    hSCManager = OpenSCManagerW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SC_MANAGER_LOCK,
    );
    if !hSCManager.is_null() {
        scLock = LockServiceDatabase(hSCManager);
        if !scLock.is_null() {
            hSc = OpenServiceW(
                hSCManager,
                lpServiceName,
                SERVICE_CHANGE_CONFIG,
            );
            if !hSc.is_null() {
                if ChangeServiceConfigW(
                    hSc,
                    dwServiceType,
                    SERVICE_NO_CHANGE,
                    SERVICE_NO_CHANGE,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    lpStartName,
                    lpPassword,
                    core::ptr::null_mut(),
                ) != 0
                {
                    bRet = (TRUE) as i32;
                }

                CloseServiceHandle(hSc);
            }

            UnlockServiceDatabase(scLock);
        }

        CloseServiceHandle(hSCManager);
    }

    if bRet == 0 {
        GetError();
    }

    return bRet;
}

unsafe extern "C" fn OnQueryInitialFocus(hwndDlg: HWND, pLogonData: PLOGONDATA) -> BOOL {
    let hwnd: HWND = GetDlgItem(hwndDlg, if (*pLogonData).bLocalSystem != 0 { (IDC_LOGON_SYSTEMACCOUNT as i32) } else { (IDC_LOGON_THISACCOUNT as i32) });

    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, hwnd as LPARAM);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn OnApply(hwndDlg: HWND, pLogonData: PLOGONDATA) -> BOOL {
    let mut szAccountName: [WCHAR; 64] = [0; 64];
    let mut szPassword1: [WCHAR; 64] = [0; 64];
    let mut szPassword2: [WCHAR; 64] = [0; 64];
    let mut dwServiceType: DWORD = SERVICE_NO_CHANGE;
    let mut bRet: BOOL = (TRUE) as i32;

    if (*pLogonData).bAccountChanged == 0 {
        return (TRUE) as BOOL;
    }

    if (SendDlgItemMessageW(hwndDlg, (IDC_LOGON_SYSTEMACCOUNT as i32), BM_GETCHECK, 0, 0)) as u32 == (BST_CHECKED) as _ {
        /* System account selected */
        wcscpy(core::ptr::addr_of_mut!(szAccountName) as *mut WCHAR, RUSTCONV_WSTR_7CEA81E6A9E5.as_ptr());
        wcscpy(core::ptr::addr_of_mut!(szPassword1) as *mut WCHAR, RUSTCONV_WSTR_96A296D224F2.as_ptr());
        wcscpy(core::ptr::addr_of_mut!(szPassword2) as *mut WCHAR, RUSTCONV_WSTR_96A296D224F2.as_ptr());

        /* Handle the interactive flag */
        dwServiceType = (*(*pLogonData).pServiceConfig).dwServiceType;
        if (SendDlgItemMessageW(hwndDlg, (IDC_LOGON_INTERACTIVE as i32), BM_GETCHECK, 0, 0)) as u32 == (BST_CHECKED) as _ {
            dwServiceType |= SERVICE_INTERACTIVE_PROCESS;
        } else {
            dwServiceType &= !SERVICE_INTERACTIVE_PROCESS;
        }
    } else {
        /* Other account selected */
        GetDlgItemText(hwndDlg, (IDC_LOGON_ACCOUNTNAME as i32), core::ptr::addr_of_mut!(szAccountName) as LPWSTR, 64);
        GetDlgItemText(hwndDlg, (IDC_LOGON_PASSWORD1 as i32), core::ptr::addr_of_mut!(szPassword1) as LPWSTR, 64);
        GetDlgItemText(hwndDlg, (IDC_LOGON_PASSWORD2 as i32), core::ptr::addr_of_mut!(szPassword2) as LPWSTR, 64);

        if wcscmp(core::ptr::addr_of_mut!(szPassword1) as *mut WCHAR, core::ptr::addr_of_mut!(szPassword2) as *mut WCHAR) != 0 {
            ResourceMessageBox(GetModuleHandle(core::ptr::null_mut()) as HINSTANCE, hwndDlg, MB_OK | MB_ICONWARNING, IDS_APPNAME, IDS_NOT_SAME_PASSWORD);
            return (FALSE) as BOOL;
        }

        if wcscmp(core::ptr::addr_of_mut!(szPassword1) as *mut WCHAR, (DEFAULT_PASSWORD).as_ptr()) == 0 {
            ResourceMessageBox(GetModuleHandle(core::ptr::null_mut()) as HINSTANCE, hwndDlg, MB_OK | MB_ICONWARNING, IDS_APPNAME, IDS_INVALID_PASSWORD);
            return (FALSE) as BOOL;
        }
    }

    bRet = SetServiceAccount(
        (*(*pLogonData).pService).lpServiceName,
        dwServiceType,
        core::ptr::addr_of_mut!(szAccountName) as LPWSTR,
        core::ptr::addr_of_mut!(szPassword1) as LPWSTR,
    );
    if (bRet) as u32 == FALSE {
    }

    if (bRet) as u32 == TRUE {
        (*pLogonData).bAccountChanged = (FALSE) as i32;
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "system" fn LogonPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pLogonData: PLOGONDATA;

    /* Get the window context */
    pLogonData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA as i32) as PLOGONDATA;
    if pLogonData.is_null() && uMsg != WM_INITDIALOG as UINT {
        return FALSE as INT_PTR;
    }

    'sw: {
        if uMsg == WM_INITDIALOG as UINT {
            pLogonData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                core::mem::size_of::<LOGONDATA>() as SIZE_T,
            ) as PLOGONDATA;
            if !pLogonData.is_null() {
                SetWindowLongPtr(
                    hwndDlg,
                    GWLP_USERDATA as i32,
                    pLogonData as LONG_PTR,
                );

                (*pLogonData).bInitialized = FALSE as BOOL;
                (*pLogonData).pService =
                    (*((*(lParam as LPPROPSHEETPAGE)).lParam as PSERVICEPROPSHEET)).pService;

                (*pLogonData).pServiceConfig =
                    GetServiceConfig((*(*pLogonData).pService).lpServiceName);
                if !(*pLogonData).pServiceConfig.is_null() {
                    wcscpy(
                        &raw mut (*pLogonData).szPassword1 as *mut WCHAR,
                        (DEFAULT_PASSWORD).as_ptr(),
                    );
                    wcscpy(
                        &raw mut (*pLogonData).szPassword2 as *mut WCHAR,
                        (DEFAULT_PASSWORD).as_ptr(),
                    );

                    if (*(*pLogonData).pServiceConfig).lpServiceStartName.is_null()
                        || _wcsicmp(
                            (*(*pLogonData).pServiceConfig).lpServiceStartName,
                            RUSTCONV_WSTR_7CEA81E6A9E5.as_ptr(),
                        ) == 0 as i32
                    {
                        SendDlgItemMessageW(
                            hwndDlg,
                            IDC_LOGON_SYSTEMACCOUNT as i32,
                            BM_SETCHECK as UINT,
                            BST_CHECKED as WPARAM,
                            0 as LPARAM,
                        );
                        if ((*(*pLogonData).pServiceConfig).dwServiceType
                            & SERVICE_INTERACTIVE_PROCESS as DWORD)
                            != 0
                        {
                            (*pLogonData).nInteractive = BST_CHECKED as INT;
                            SendDlgItemMessageW(
                                hwndDlg,
                                IDC_LOGON_INTERACTIVE as i32,
                                BM_SETCHECK as UINT,
                                BST_CHECKED as WPARAM,
                                0 as LPARAM,
                            );
                        }
                        SetControlStates(
                            hwndDlg,
                            pLogonData,
                            TRUE as BOOL,
                        );
                    } else {
                        wcscpy(
                            &raw mut (*pLogonData).szAccountName as *mut WCHAR,
                            (*(*pLogonData).pServiceConfig).lpServiceStartName,
                        );
                        SendDlgItemMessageW(
                            hwndDlg,
                            IDC_LOGON_THISACCOUNT as i32,
                            BM_SETCHECK as UINT,
                            BST_CHECKED as WPARAM,
                            0 as LPARAM,
                        );
                        SetControlStates(
                            hwndDlg,
                            pLogonData,
                            FALSE as BOOL,
                        );
                    }
                }

                (*pLogonData).bInitialized = TRUE as BOOL;
            }

            EnableWindow(
                GetDlgItem(hwndDlg, IDC_LOGON_HWPROFILE as i32),
                FALSE as BOOL,
            );
            break 'sw;
        }

        if uMsg == WM_DESTROY as UINT {
            if !(*pLogonData).pServiceConfig.is_null() {
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    (*pLogonData).pServiceConfig as PVOID,
                );
            }

            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pLogonData as PVOID,
            );
            break 'sw;
        }

        if uMsg == WM_COMMAND as UINT {
            'sw2: {
                if LOWORD!(wParam) as i32 == IDC_LOGON_SYSTEMACCOUNT as i32 {
                    if HIWORD!(wParam) as i32 == BN_CLICKED as i32 {
                        if (*pLogonData).bInitialized != 0 {
                            (*pLogonData).bAccountChanged = TRUE as BOOL;
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        }
                        SetControlStates(
                            hwndDlg,
                            pLogonData,
                            TRUE as BOOL,
                        );
                    }
                    break 'sw2;
                }

                if LOWORD!(wParam) as i32 == IDC_LOGON_THISACCOUNT as i32 {
                    if HIWORD!(wParam) as i32 == BN_CLICKED as i32 {
                        if (*pLogonData).bInitialized != 0 {
                            (*pLogonData).bAccountChanged = TRUE as BOOL;
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        }
                        SetControlStates(
                            hwndDlg,
                            pLogonData,
                            FALSE as BOOL,
                        );
                    }
                    break 'sw2;
                }

                if LOWORD!(wParam) as i32 == IDC_LOGON_INTERACTIVE as i32 {
                    if HIWORD!(wParam) as i32 == BN_CLICKED as i32 {
                        if (*pLogonData).bInitialized != 0 {
                            (*pLogonData).bAccountChanged = TRUE as BOOL;
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        }
                    }
                    break 'sw2;
                }

                if LOWORD!(wParam) as i32 == IDC_LOGON_ACCOUNTNAME as i32
                    || LOWORD!(wParam) as i32 == IDC_LOGON_PASSWORD1 as i32
                    || LOWORD!(wParam) as i32 == IDC_LOGON_PASSWORD2 as i32
                {
                    if HIWORD!(wParam) as i32 == EN_CHANGE as i32
                        && (*pLogonData).bInitialized != 0
                    {
                        (*pLogonData).bAccountChanged = TRUE as BOOL;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    break 'sw2;
                }
            }
            break 'sw;
        }

        if uMsg == WM_NOTIFY as UINT {
            'sw3: {
                if (*(lParam as LPNMHDR)).code == PSN_QUERYINITIALFOCUS as UINT {
                    return OnQueryInitialFocus(hwndDlg, pLogonData) as INT_PTR;
                }

                if (*(lParam as LPNMHDR)).code == PSN_APPLY as UINT {
                    return OnApply(hwndDlg, pLogonData) as INT_PTR;
                }
                break 'sw3;
            }
            break 'sw;
        }
    }

    return FALSE as INT_PTR;
}
