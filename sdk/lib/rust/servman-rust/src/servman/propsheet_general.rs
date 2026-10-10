use crate::servman::misc::AllocAndLoadString;
use crate::servman::listview::ChangeListViewText;
use crate::servman::query::GetServiceConfig;
use crate::servman::query::GetServiceDescription;
use crate::servman::progress::RunActionWithProgress;
use crate::servman::mainwnd::SetMenuAndButtonStates;
use crate::servman::query::SetServiceConfig;
use crate::servman::query::SetServiceDescription;
use crate::servman::query::UpdateServiceStatus;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct PAGEDATA {
    pub dlgInfo: PSERVICEPROPSHEET,
    pub bDisplayNameChanged: BOOL,
    pub bDescriptionChanged: BOOL,
    pub bBinaryPathChanged: BOOL,
    pub bStartTypeChanged: BOOL,
}

pub type PPAGEDATA = *mut PAGEDATA;

unsafe extern "C" fn SetButtonStates(dlgInfo: PSERVICEPROPSHEET, hwndDlg: HWND) {
    let mut hButton: HWND = core::ptr::null_mut();
    let mut lpServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
    let mut Flags: DWORD = 0;
    let mut State: DWORD = 0;
    let mut i: UINT = 0;

    Flags = (*(*dlgInfo).pService).ServiceStatusProcess.dwControlsAccepted;
    State = (*(*dlgInfo).pService).ServiceStatusProcess.dwCurrentState;

    i = IDC_START as UINT;
    while i <= IDC_RESUME as UINT {
        hButton = GetDlgItem(hwndDlg, i as ::core::ffi::c_int);
        EnableWindow(hButton, FALSE as BOOL);
        i = i.wrapping_add(1);
    }

    lpServiceConfig = GetServiceConfig((*(*dlgInfo).pService).lpServiceName);
    if State == SERVICE_STOPPED as DWORD
        && !lpServiceConfig.is_null()
        && (*lpServiceConfig).dwStartType != SERVICE_DISABLED as DWORD
    {
        hButton = GetDlgItem(hwndDlg, IDC_START as ::core::ffi::c_int);
        EnableWindow(hButton, TRUE as BOOL);
    } else if (Flags & SERVICE_ACCEPT_STOP as DWORD) != 0
        && State == SERVICE_RUNNING as DWORD
    {
        hButton = GetDlgItem(hwndDlg, IDC_STOP as ::core::ffi::c_int);
        EnableWindow(hButton, TRUE as BOOL);
    } else if (Flags & SERVICE_ACCEPT_PAUSE_CONTINUE as DWORD) != 0
        && State == SERVICE_RUNNING as DWORD
    {
        hButton = GetDlgItem(hwndDlg, IDC_PAUSE as ::core::ffi::c_int);
        EnableWindow(hButton, TRUE as BOOL);
    }

    hButton = GetDlgItem(hwndDlg, IDC_START_PARAM as ::core::ffi::c_int);
    EnableWindow(
        hButton,
        (State == SERVICE_STOPPED as DWORD
            && !lpServiceConfig.is_null()
            && (*lpServiceConfig).dwStartType != SERVICE_DISABLED as DWORD) as BOOL,
    );

    if !lpServiceConfig.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpServiceConfig as PVOID,
        );
    }

    /* set the main toolbar */
    SetMenuAndButtonStates((*dlgInfo).Info);
}

unsafe extern "C" fn SetServiceStatusText(dlgInfo: PSERVICEPROPSHEET, hwndDlg: HWND) {
    let mut lpStatus: LPWSTR = core::ptr::null_mut();
    let mut id: UINT;

    if (*(*dlgInfo).pService).ServiceStatusProcess.dwCurrentState == SERVICE_RUNNING as DWORD
    {
        id = IDS_SERVICES_STARTED as UINT;
    }
    else
    {
        id = IDS_SERVICES_STOPPED as UINT;
    }

    if AllocAndLoadString(core::ptr::addr_of_mut!(lpStatus),
                           hInstance,
                           id) != 0
    {
        SendDlgItemMessageW(hwndDlg,
                            (IDC_SERV_STATUS as UINT) as i32,
                            WM_SETTEXT as UINT,
                            0 as WPARAM,
                            lpStatus as LPARAM);
        LocalFree(lpStatus as HLOCAL);
    }
}

unsafe extern "C" fn SetStartupType(mut lpServiceName: LPWSTR, mut hwndDlg: HWND) {
    let mut hList: HWND = core::ptr::null_mut();
    let mut pServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
    let mut lpBuf: LPWSTR = core::ptr::null_mut();
    let mut StartUp: DWORD = 0 as DWORD;
    let mut i: UINT = 0;
    hList = GetDlgItem(hwndDlg, IDC_START_TYPE as i32);
    i = IDS_SERVICES_AUTO as UINT;
    while i <= IDS_SERVICES_DIS as UINT {
        if AllocAndLoadString(core::ptr::addr_of_mut!(lpBuf), hInstance, i) != 0 {
            SendMessageW(
                hList,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                lpBuf as LPARAM,
            );
            LocalFree(lpBuf as HLOCAL);
        }
        i = i.wrapping_add(1);
    }
    pServiceConfig = GetServiceConfig(lpServiceName);
    if !pServiceConfig.is_null() {
        let sc: DWORD = (*pServiceConfig).dwStartType;
        if sc == SERVICE_AUTO_START as DWORD {
            StartUp = 0 as DWORD;
        } else if sc == SERVICE_DEMAND_START as DWORD {
            StartUp = 1 as DWORD;
        } else if sc == SERVICE_DISABLED as DWORD {
            StartUp = 2 as DWORD;
        }
        SendMessageW(
            hList,
            CB_SETCURSEL as UINT,
            StartUp as WPARAM,
            0 as LPARAM,
        );
        HeapFree(
            ProcessHeap,
            0 as DWORD,
            pServiceConfig as PVOID,
        );
    }
}

unsafe extern "C" fn InitGeneralPage(mut dlgInfo: PSERVICEPROPSHEET, mut hwndDlg: HWND) {
    let mut pServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut::<QUERY_SERVICE_CONFIGW>();
    let mut lpDescription: LPWSTR = core::ptr::null_mut::<WCHAR>();
    /* set the service name */
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_SERV_NAME as i32),
        WM_SETTEXT as UINT,
        0 as WPARAM,
        (*(*dlgInfo).pService).lpServiceName as LPARAM,
    );
    /* set the display name */
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_DISP_NAME as i32),
        WM_SETTEXT as UINT,
        0 as WPARAM,
        (*(*dlgInfo).pService).lpDisplayName as LPARAM,
    );
    /* set the description */
    lpDescription = GetServiceDescription((*(*dlgInfo).pService).lpServiceName);
    if !lpDescription.is_null() {
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_DESCRIPTION as i32),
            WM_SETTEXT as UINT,
            0 as WPARAM,
            lpDescription as LPARAM,
        );
        HeapFree(
            ProcessHeap,
            0 as DWORD,
            lpDescription as PVOID,
        );
    }
    pServiceConfig = GetServiceConfig((*(*dlgInfo).pService).lpServiceName);
    if !pServiceConfig.is_null() {
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_EXEPATH as i32),
            WM_SETTEXT as UINT,
            0 as WPARAM,
            (*pServiceConfig).lpBinaryPathName as LPARAM,
        );
        HeapFree(
            ProcessHeap,
            0 as DWORD,
            pServiceConfig as PVOID,
        );
    }

    /* set startup type */
    SetStartupType((*(*dlgInfo).pService).lpServiceName, hwndDlg);

    SetServiceStatusText(dlgInfo, hwndDlg);

    if (*(*dlgInfo).Info).bIsUserAnAdmin != 0 {
        EnableWindow(
            GetDlgItem(hwndDlg, (IDC_EDIT as i32)),
            (TRUE) as BOOL,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn SaveDlgInfo(pPageData: PPAGEDATA, hwndDlg: HWND) {
    let mut pServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
    let mut pDisplayName: PWSTR = core::ptr::null_mut();
    let mut pDescription: PWSTR;
    let mut nLength: INT;
    let mut StartUp: DWORD;

    pServiceConfig = HeapAlloc(ProcessHeap,
                               HEAP_ZERO_MEMORY as DWORD,
                               core::mem::size_of::<QUERY_SERVICE_CONFIGW>() as SIZE_T) as LPQUERY_SERVICE_CONFIG;
    if !pServiceConfig.is_null() {
        (*pServiceConfig).dwServiceType = SERVICE_NO_CHANGE as DWORD;
        (*pServiceConfig).dwErrorControl = SERVICE_NO_CHANGE as DWORD;
        (*pServiceConfig).dwStartType = SERVICE_NO_CHANGE as DWORD;

        if (*pPageData).bStartTypeChanged != 0 {
            StartUp = SendDlgItemMessageW(hwndDlg, IDC_START_TYPE as INT, CB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM) as DWORD;
            match StartUp {
                0 => {
                    (*pServiceConfig).dwStartType = SERVICE_AUTO_START as DWORD;
                }

                1 => {
                    (*pServiceConfig).dwStartType = SERVICE_DEMAND_START as DWORD;
                }
                2 => {
                    (*pServiceConfig).dwStartType = SERVICE_DISABLED as DWORD;
                }
                _ => {}
            }
        }

        if (*pPageData).bBinaryPathChanged != 0 {
            nLength = SendDlgItemMessageW(hwndDlg, IDC_EXEPATH as INT, WM_GETTEXTLENGTH as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
            (*pServiceConfig).lpBinaryPathName = HeapAlloc(ProcessHeap,
                                                         HEAP_ZERO_MEMORY as DWORD,
                                                         ((nLength as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as SIZE_T)
                                                             .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
            if (*pServiceConfig).lpBinaryPathName != core::ptr::null_mut() {
                SendDlgItemMessageW(hwndDlg, IDC_EXEPATH as INT, WM_GETTEXT as UINT, (nLength as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as WPARAM, (*pServiceConfig).lpBinaryPathName as LPARAM);
            }
        }

        if (*pPageData).bDisplayNameChanged != 0 {
            nLength = SendDlgItemMessageW(hwndDlg, IDC_DISP_NAME as INT, WM_GETTEXTLENGTH as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
            pDisplayName = HeapAlloc(ProcessHeap,
                                     HEAP_ZERO_MEMORY as DWORD,
                                     ((nLength as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as SIZE_T)
                                         .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as PWSTR;
            if !pDisplayName.is_null() {
                SendDlgItemMessageW(hwndDlg, IDC_DISP_NAME as INT, WM_GETTEXT as UINT, (nLength as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as WPARAM, pDisplayName as LPARAM);

                if !(*(*(*pPageData).dlgInfo).pService).lpDisplayName.is_null() {
                    HeapFree(ProcessHeap, 0 as DWORD, (*(*(*pPageData).dlgInfo).pService).lpDisplayName as PVOID);
                }

                (*(*(*pPageData).dlgInfo).pService).lpDisplayName = pDisplayName;
                (*pServiceConfig).lpDisplayName = pDisplayName;
            }
        }

        if SetServiceConfig(pServiceConfig,
                             (*(*(*pPageData).dlgInfo).pService).lpServiceName,
                             core::ptr::null_mut()) != 0
        {
            if (*pPageData).bDisplayNameChanged != 0 {
                ChangeListViewText((*(*pPageData).dlgInfo).Info,
                                   (*(*pPageData).dlgInfo).pService,
                                   LVNAME as UINT);
            }

            if (*pPageData).bStartTypeChanged != 0 {
                ChangeListViewText((*(*pPageData).dlgInfo).Info,
                                   (*(*pPageData).dlgInfo).pService,
                                   LVSTARTUP as UINT);
            }
        }

        if !(*pServiceConfig).lpBinaryPathName.is_null() {
            HeapFree(ProcessHeap, 0 as DWORD, (*pServiceConfig).lpBinaryPathName as PVOID);
        }

        HeapFree(ProcessHeap, 0 as DWORD, pServiceConfig as PVOID);
    }

    if (*pPageData).bDescriptionChanged != 0 {
        nLength = SendDlgItemMessageW(hwndDlg, IDC_DESCRIPTION as INT, WM_GETTEXTLENGTH as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
        pDescription = HeapAlloc(ProcessHeap, HEAP_ZERO_MEMORY as DWORD,
                                     ((nLength as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as SIZE_T)
                                         .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as PWSTR;
        if !pDescription.is_null() {
            SendDlgItemMessageW(hwndDlg, IDC_DESCRIPTION as INT, WM_GETTEXT as UINT, (nLength as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as WPARAM, pDescription as LPARAM);

            if SetServiceDescription((*(*(*pPageData).dlgInfo).pService).lpServiceName,
                                      pDescription as LPWSTR) != 0
            {
                // The translation passed two NULLs here; the C names the dialog's Info and service.
                ChangeListViewText((*(*pPageData).dlgInfo).Info, (*(*pPageData).dlgInfo).pService, LVDESC as UINT);
            }

            HeapFree(ProcessHeap, 0 as DWORD, pDescription as PVOID);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn GeneralPageProc(mut hwndDlg: HWND, mut uMsg: UINT, mut wParam: WPARAM, mut lParam: LPARAM) -> INT_PTR {
    let mut pPageData: PPAGEDATA = core::ptr::null_mut::<PAGEDATA>();

    /* Get the window context */
    pPageData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PPAGEDATA;
    if pPageData.is_null() && uMsg != WM_INITDIALOG as UINT {
        return FALSE as INT_PTR;
    }

    match uMsg {
        WM_INITDIALOG => {
            pPageData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                core::mem::size_of::<PAGEDATA>() as SIZE_T,
            ) as PPAGEDATA;
            if !pPageData.is_null() {
                SetWindowLongPtr(
                    hwndDlg,
                    GWLP_USERDATA,
                    pPageData as LONG_PTR,
                );

                (*pPageData).dlgInfo = (*(lParam as LPPROPSHEETPAGE)).lParam as PSERVICEPROPSHEET;
                if !(*pPageData).dlgInfo.is_null() {
                    InitGeneralPage((*pPageData).dlgInfo, hwndDlg);
                    SetButtonStates((*pPageData).dlgInfo, hwndDlg);
                }
            }
        }

        WM_DESTROY => {
            HeapFree(GetProcessHeap(), 0 as DWORD, pPageData as PVOID);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_START_TYPE => {
                    if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                        (*pPageData).bStartTypeChanged = TRUE as BOOL;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_DISP_NAME => {
                    if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                        (*pPageData).bDisplayNameChanged = TRUE as BOOL;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_DESCRIPTION => {
                    if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                        (*pPageData).bDescriptionChanged = TRUE as BOOL;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_EXEPATH => {
                    if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                        (*pPageData).bBinaryPathChanged = TRUE as BOOL;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_START => {
                    let mut szStartParams: [WCHAR; 256] =
                        [0 as WCHAR; 256];
                    let mut lpStartParams: LPWSTR = core::ptr::null_mut::<WCHAR>();

                    if GetDlgItemText(
                        hwndDlg,
                        (IDC_START_PARAM as i32),
                        core::ptr::addr_of_mut!(szStartParams) as LPWSTR,
                        256,
                    ) > 0 as UINT
                    {
                        lpStartParams = core::ptr::addr_of_mut!(szStartParams) as *mut WCHAR as LPWSTR;
                    }

                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_START_PARAM as i32)),
                        FALSE as BOOL,
                    );

                    RunActionWithProgress(
                        hwndDlg,
                        (*(*(*pPageData).dlgInfo).pService).lpServiceName,
                        (*(*(*pPageData).dlgInfo).pService).lpDisplayName,
                        ACTION_START as UINT,
                        lpStartParams as PVOID,
                    );

                    UpdateServiceStatus((*(*pPageData).dlgInfo).pService);
                    ChangeListViewText(
                        (*(*pPageData).dlgInfo).Info,
                        (*(*pPageData).dlgInfo).pService,
                        LVSTATUS as UINT,
                    );
                    SetButtonStates((*pPageData).dlgInfo, hwndDlg);
                    SetServiceStatusText((*pPageData).dlgInfo, hwndDlg);
                }

                IDC_STOP => {
                    RunActionWithProgress(
                        hwndDlg,
                        (*(*(*pPageData).dlgInfo).pService).lpServiceName,
                        (*(*(*pPageData).dlgInfo).pService).lpDisplayName,
                        ACTION_STOP as UINT,
                        NULL,
                    );

                    UpdateServiceStatus((*(*pPageData).dlgInfo).pService);
                    ChangeListViewText(
                        (*(*pPageData).dlgInfo).Info,
                        (*(*pPageData).dlgInfo).pService,
                        LVSTATUS as UINT,
                    );
                    SetButtonStates((*pPageData).dlgInfo, hwndDlg);
                    SetServiceStatusText((*pPageData).dlgInfo, hwndDlg);
                }

                IDC_PAUSE => {
                    RunActionWithProgress(
                        hwndDlg,
                        (*(*(*pPageData).dlgInfo).pService).lpServiceName,
                        (*(*(*pPageData).dlgInfo).pService).lpDisplayName,
                        ACTION_PAUSE as UINT,
                        NULL,
                    );

                    UpdateServiceStatus((*(*pPageData).dlgInfo).pService);
                    ChangeListViewText(
                        (*(*pPageData).dlgInfo).Info,
                        (*(*pPageData).dlgInfo).pService,
                        LVSTATUS as UINT,
                    );
                    SetButtonStates((*pPageData).dlgInfo, hwndDlg);
                    SetServiceStatusText((*pPageData).dlgInfo, hwndDlg);
                }

                IDC_RESUME => {
                    RunActionWithProgress(
                        hwndDlg,
                        (*(*(*pPageData).dlgInfo).pService).lpServiceName,
                        (*(*(*pPageData).dlgInfo).pService).lpDisplayName,
                        ACTION_RESUME as UINT,
                        NULL,
                    );

                    UpdateServiceStatus((*(*pPageData).dlgInfo).pService);
                    ChangeListViewText(
                        (*(*pPageData).dlgInfo).Info,
                        (*(*pPageData).dlgInfo).pService,
                        LVSTATUS as UINT,
                    );
                    SetButtonStates((*pPageData).dlgInfo, hwndDlg);
                    SetServiceStatusText((*pPageData).dlgInfo, hwndDlg);
                }

                IDC_EDIT => {
                    SendDlgItemMessage(
                        hwndDlg,
                        (IDC_DISP_NAME as i32),
                        EM_SETREADONLY as UINT,
                        FALSE as WPARAM,
                        0 as LPARAM,
                    );
                    SendDlgItemMessage(
                        hwndDlg,
                        (IDC_DESCRIPTION as i32),
                        EM_SETREADONLY as UINT,
                        FALSE as WPARAM,
                        0 as LPARAM,
                    );
                    SendDlgItemMessage(
                        hwndDlg,
                        (IDC_EXEPATH as i32),
                        EM_SETREADONLY as UINT,
                        FALSE as WPARAM,
                        0 as LPARAM,
                    );
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_EDIT as i32)),
                        FALSE as BOOL,
                    );
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_APPLY => {
                    if (*pPageData).bDisplayNameChanged != 0
                        || (*pPageData).bDescriptionChanged != 0
                        || (*pPageData).bBinaryPathChanged != 0
                        || (*pPageData).bStartTypeChanged != 0
                    {
                        SaveDlgInfo(pPageData, hwndDlg);
                        SetButtonStates((*pPageData).dlgInfo, hwndDlg);
                        (*pPageData).bDisplayNameChanged = FALSE as BOOL;
                        (*pPageData).bDescriptionChanged = FALSE as BOOL;
                        (*pPageData).bBinaryPathChanged = FALSE as BOOL;
                        (*pPageData).bStartTypeChanged = FALSE as BOOL;
                    }
                }
                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
