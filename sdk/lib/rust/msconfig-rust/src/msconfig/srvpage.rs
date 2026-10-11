use crate::msconfig::msconfig::DisableAllExcept;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hServicesPage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hServicesListCtrl: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hServicesDialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;
#[no_mangle]
pub unsafe extern "system" fn ServicesPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut column: LVCOLUMNW = core::mem::zeroed();
    let mut szTemp: [TCHAR; 256] = [0; 256];
    let mut dwStyle: DWORD = 0;

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);

    match message {
        WM_INITDIALOG => {
            hServicesListCtrl = GetDlgItem(hDlg, (IDC_SERVICES_LIST as i32));
            hServicesDialog = hDlg;

            dwStyle = SendMessage(hServicesListCtrl, LVM_GETEXTENDEDLISTVIEWSTYLE, 0, 0) as DWORD;
            dwStyle = dwStyle | LVS_EX_FULLROWSELECT | LVS_EX_CHECKBOXES;
            SendMessage(hServicesListCtrl, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, dwStyle as LPARAM);

            SetWindowPos(hDlg, core::ptr::null_mut(), 10, 32, 0, 0, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);

            // Initialize the application page's controls
            column.mask = LVCF_TEXT | LVCF_WIDTH;

            LoadString(hInst, IDS_SERVICES_COLUMN_SERVICE, szTemp.as_mut_ptr(), 256);
            column.pszText = szTemp.as_mut_ptr();
            column.cx = 200;
            ListView_InsertColumn!(hServicesListCtrl, 0, core::ptr::addr_of_mut!(column));

            column.mask = LVCF_TEXT | LVCF_WIDTH;
            LoadString(hInst, IDS_SERVICES_COLUMN_REQ, szTemp.as_mut_ptr(), 256);
            column.pszText = szTemp.as_mut_ptr();
            column.cx = 70;
            ListView_InsertColumn!(hServicesListCtrl, 1, core::ptr::addr_of_mut!(column));

            column.mask = LVCF_TEXT | LVCF_WIDTH;
            LoadString(hInst, IDS_SERVICES_COLUMN_VENDOR, szTemp.as_mut_ptr(), 256);
            column.pszText = szTemp.as_mut_ptr();
            column.cx = 200;
            ListView_InsertColumn!(hServicesListCtrl, 2, core::ptr::addr_of_mut!(column));

            column.mask = LVCF_TEXT | LVCF_WIDTH;
            LoadString(hInst, IDS_SERVICES_COLUMN_STATUS, szTemp.as_mut_ptr(), 256);
            column.pszText = szTemp.as_mut_ptr();
            column.cx = 70;
            ListView_InsertColumn!(hServicesListCtrl, 3, core::ptr::addr_of_mut!(column));

            GetServices();
            DisableAllExcept(hDlg, IDC_SERVICES_LIST); // FIXME: Implement saving
            return TRUE as INT_PTR;
        }
        WM_NOTIFY => {
            if wParam == (IDC_SERVICES_LIST as WPARAM) {
                let pnmlv: *mut NMLISTVIEW = lParam as *mut NMLISTVIEW;
                let toggled: UINT = ((*pnmlv).uOldState ^ (*pnmlv).uNewState) & LVIS_STATEIMAGEMASK;
                if (*pnmlv).hdr.code == (LVN_ITEMCHANGING) as u32 && ((*pnmlv).uChanged & LVIF_STATE) != 0 && toggled != 0 {
                    // Only allow checkbox changes during WM_INITDIALOG
                    if IsWindowEnabled(GetDlgItem(hDlg, (IDC_BTN_SERVICES_ACTIVATE as i32))) == 0 {
                        MessageBeep((1 as u32).wrapping_neg() as UINT);
                        SetWindowLongPtr(hDlg, DWLP_MSGRESULT, (TRUE) as _);
                        return TRUE as INT_PTR;
                    }
                }
            }
        }
        _ => {}
    }

    return 0 as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn GetServices() {
    let mut item: LVITEMW = core::mem::zeroed();
    let mut wCodePage: WORD = 0;
    let mut wLangID: WORD = 0;
    let mut ScHandle: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut BytesNeeded: DWORD = 0;
    let mut ResumeHandle: DWORD = 0;
    let mut NumServices: DWORD = 0;
    let mut dwHandle: DWORD = 0;
    let mut dwLen: DWORD = 0;
    let mut Index: usize = 0;
    let mut BufLen: UINT = 0;
    let mut szStatus: [TCHAR; 128] = [0; 128];
    let mut lpData: *mut TCHAR = core::ptr::null_mut();
    let mut lpBuffer: *mut TCHAR = core::ptr::null_mut();
    let mut szStrFileInfo: [TCHAR; 80] = [0; 80];
    let mut FileName: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pvData: LPVOID = core::ptr::null_mut();

    let mut pServiceFailureActions: LPSERVICE_FAILURE_ACTIONS = core::ptr::null_mut();
    let mut pServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
    let mut pServiceStatus: *mut ENUM_SERVICE_STATUS_PROCESS = core::ptr::null_mut();

    // (&[0x0020, 0x0000u16]).as_ptr() /* _T " " */ and (&[0x0022, 0x0000u16]).as_ptr() /* _T "\"" */ and the other wide literals as explicit u16 arrays
    let space: [u16; 2] = [' ' as u16, 0];
    let quote: [u16; 2] = ['"' as u16, 0];
    let translation: [u16; 25] = [
        '\\' as u16, 'V' as u16, 'a' as u16, 'r' as u16, 'F' as u16, 'i' as u16,
        'l' as u16, 'e' as u16, 'I' as u16, 'n' as u16, 'f' as u16, 'o' as u16,
        '\\' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'n' as u16, 's' as u16,
        'l' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0,
    ];
    let fileinfo_fmt: [u16; 36] = [
        'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16,
        'F' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'I' as u16, 'n' as u16,
        'f' as u16, 'o' as u16, '\\' as u16, '%' as u16, '0' as u16, '4' as u16,
        'X' as u16, '%' as u16, '0' as u16, '4' as u16, 'X' as u16, '\\' as u16,
        'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'a' as u16, 'n' as u16,
        'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 0,
    ];

    ScHandle = OpenSCManager(core::ptr::null_mut(), core::ptr::null_mut(), SC_MANAGER_ENUMERATE_SERVICE);
    if !ScHandle.is_null() {
        if EnumServicesStatusEx(ScHandle, SC_ENUM_PROCESS_INFO, SERVICE_WIN32, SERVICE_STATE_ALL, pServiceStatus as LPBYTE, 0, core::ptr::addr_of_mut!(BytesNeeded), core::ptr::addr_of_mut!(NumServices), core::ptr::addr_of_mut!(ResumeHandle), core::ptr::null()) == 0 {
            /* Call function again if required size was returned */
            if GetLastError() == ERROR_MORE_DATA {
                /* reserve memory for service info array */
                pServiceStatus = HeapAlloc(GetProcessHeap(), 0, BytesNeeded as SIZE_T) as *mut ENUM_SERVICE_STATUS_PROCESS;
                if pServiceStatus.is_null() {
                    CloseServiceHandle(ScHandle);
                    return;
                }

                /* fill array with service info */
                if EnumServicesStatusEx(ScHandle, SC_ENUM_PROCESS_INFO, SERVICE_WIN32, SERVICE_STATE_ALL, pServiceStatus as LPBYTE, BytesNeeded, core::ptr::addr_of_mut!(BytesNeeded), core::ptr::addr_of_mut!(NumServices), core::ptr::addr_of_mut!(ResumeHandle), core::ptr::null()) == 0 {
                    HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                    CloseServiceHandle(ScHandle);
                    return;
                }
            } else {
                /* exit on failure */
                CloseServiceHandle(ScHandle);
                return;
            }
        }

        if NumServices != 0 {
            if pServiceStatus.is_null() {
                CloseServiceHandle(ScHandle);
                return;
            }

            Index = 0 as usize;
            while Index < NumServices as usize {
                memset(core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void, 0, core::mem::size_of::<LV_ITEM>());
                item.mask = LVIF_TEXT;
                item.iImage = 0;
                item.pszText = (*pServiceStatus.add(Index)).lpDisplayName;
                item.iItem = ListView_GetItemCount!(hServicesListCtrl);
                item.lParam = 0;
                item.iItem = ListView_InsertItem!(hServicesListCtrl, core::ptr::addr_of_mut!(item));

                if (*pServiceStatus.add(Index)).ServiceStatusProcess.dwCurrentState == SERVICE_RUNNING {
                    ListView_SetCheckState!(hServicesListCtrl, item.iItem, TRUE);
                }

                BytesNeeded = 0;
                hService = OpenService(ScHandle, (*pServiceStatus.add(Index)).lpServiceName, SC_MANAGER_CONNECT);
                if !hService.is_null() {
                    /* check if service is required by the system*/
                    if QueryServiceConfig2(hService, SERVICE_CONFIG_FAILURE_ACTIONS, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(BytesNeeded)) == 0 {
                        if GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                            pServiceFailureActions = HeapAlloc(GetProcessHeap(), 0, BytesNeeded as SIZE_T) as LPSERVICE_FAILURE_ACTIONS;
                            if pServiceFailureActions.is_null() {
                                HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                                CloseServiceHandle(hService);
                                CloseServiceHandle(ScHandle);
                                return;
                            }

                            if QueryServiceConfig2(hService, SERVICE_CONFIG_FAILURE_ACTIONS, pServiceFailureActions as LPBYTE, BytesNeeded, core::ptr::addr_of_mut!(BytesNeeded)) == 0 {
                                HeapFree(GetProcessHeap(), 0, pServiceFailureActions as PVOID);
                                HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                                CloseServiceHandle(hService);
                                CloseServiceHandle(ScHandle);
                                return;
                            }
                        } else {
                            /* exit on failure */
                            HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                            CloseServiceHandle(hService);
                            CloseServiceHandle(ScHandle);
                            return;
                        }
                    }

                    if !pServiceFailureActions.is_null() {
                        if (*pServiceFailureActions).cActions != 0
                            && (*(*pServiceFailureActions).lpsaActions.add(0 as usize)).Type == SC_ACTION_REBOOT
                        {
                            LoadString(hInst, IDS_SERVICES_YES, szStatus.as_mut_ptr(), 128);
                            item.pszText = szStatus.as_mut_ptr();
                            item.iSubItem = 1;
                            SendMessage(hServicesListCtrl, (LVM_SETITEMTEXT as u32), item.iItem as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM);
                        }
                        HeapFree(GetProcessHeap(), 0, pServiceFailureActions as PVOID);
                        pServiceFailureActions = core::ptr::null_mut();
                    }

                    /* get vendor of service binary */
                    BytesNeeded = 0;
                    if QueryServiceConfig(hService, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(BytesNeeded)) == 0 {
                        if GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                            pServiceConfig = HeapAlloc(GetProcessHeap(), 0, BytesNeeded as SIZE_T) as LPQUERY_SERVICE_CONFIG;
                            if pServiceConfig.is_null() {
                                HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                                CloseServiceHandle(hService);
                                CloseServiceHandle(ScHandle);
                                return;
                            }
                            if QueryServiceConfig(hService, pServiceConfig, BytesNeeded, core::ptr::addr_of_mut!(BytesNeeded)) == 0 {
                                HeapFree(GetProcessHeap(), 0, pServiceConfig as PVOID);
                                HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                                CloseServiceHandle(hService);
                                CloseServiceHandle(ScHandle);
                                return;
                            }
                        } else {
                            /* exit on failure */
                            HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                            CloseServiceHandle(hService);
                            CloseServiceHandle(ScHandle);
                            return;
                        }
                    }

                    if *(*pServiceConfig).lpBinaryPathName.add(0 as usize) != ('"' as u16) {
                        /* Assume everything before the first space is the binary path */
                        /* FIXME: This is a reasonable heuristic but some
                         *        services use unquoted paths with spaces */
                        StringCchCopyN(FileName.as_mut_ptr(),
                                       (_countof!(FileName)) as _,
                                       (*pServiceConfig).lpBinaryPathName,
                                       _tcscspn((*pServiceConfig).lpBinaryPathName, space.as_ptr()));
                    } else {
                        /* Binary path is inside the quotes */
                        StringCchCopyN(FileName.as_mut_ptr(),
                                       (_countof!(FileName)) as _,
                                       (*pServiceConfig).lpBinaryPathName.add(1 as usize),
                                       _tcscspn((*pServiceConfig).lpBinaryPathName.add(1 as usize), quote.as_ptr()));
                    }

                    HeapFree(GetProcessHeap(), 0, pServiceConfig as PVOID);
                    pServiceConfig = core::ptr::null_mut();

                    dwLen = GetFileVersionInfoSize(FileName.as_mut_ptr(), core::ptr::addr_of_mut!(dwHandle));
                    if dwLen != 0 {
                        lpData = HeapAlloc(GetProcessHeap(), 0, dwLen as SIZE_T) as *mut TCHAR;
                        if lpData.is_null() {
                            HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                            CloseServiceHandle(hService);
                            CloseServiceHandle(ScHandle);
                            return;
                        }
                        if GetFileVersionInfo(FileName.as_mut_ptr(), dwHandle, dwLen, lpData as LPVOID) == 0 {
                            HeapFree(GetProcessHeap(), 0, lpData as PVOID);
                            HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
                            CloseServiceHandle(hService);
                            CloseServiceHandle(ScHandle);
                            return;
                        }

                        if VerQueryValue(lpData as LPVOID, translation.as_ptr(), core::ptr::addr_of_mut!(pvData), core::ptr::addr_of_mut!(BufLen) as PUINT) != 0 {
                            wCodePage = LOWORD!(*(pvData as *mut DWORD)) as WORD;
                            wLangID = HIWORD!(*(pvData as *mut DWORD)) as WORD;
                            wsprintf(szStrFileInfo.as_mut_ptr(), fileinfo_fmt.as_ptr(), wCodePage as core::ffi::c_int, wLangID as core::ffi::c_int);
                        }

                        if VerQueryValue(lpData as LPVOID, szStrFileInfo.as_mut_ptr(), core::ptr::addr_of_mut!(lpBuffer) as *mut LPVOID, core::ptr::addr_of_mut!(BufLen) as PUINT) != 0 {
                            item.pszText = lpBuffer;
                            item.iSubItem = 2;
                            SendMessage(hServicesListCtrl, (LVM_SETITEMTEXT as u32), item.iItem as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM);
                        }
                        HeapFree(GetProcessHeap(), 0, lpData as PVOID);
                    } else {
                        LoadString(hInst, IDS_SERVICES_UNKNOWN, szStatus.as_mut_ptr(), 128);
                        item.pszText = szStatus.as_mut_ptr();
                        item.iSubItem = 2;
                        SendMessage(hServicesListCtrl, (LVM_SETITEMTEXT as u32), item.iItem as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM);
                    }
                    CloseServiceHandle(hService);
                }

                LoadString(hInst, (if (*pServiceStatus.add(Index)).ServiceStatusProcess.dwCurrentState == SERVICE_STOPPED { IDS_SERVICES_STATUS_STOPPED } else { IDS_SERVICES_STATUS_RUNNING }) as UINT, szStatus.as_mut_ptr(), 128);
                item.pszText = szStatus.as_mut_ptr();
                item.iSubItem = 3;
                SendMessage(hServicesListCtrl, (LVM_SETITEMTEXT as u32), item.iItem as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM);

                Index += 1;
            }
        }

        HeapFree(GetProcessHeap(), 0, pServiceStatus as PVOID);
        CloseServiceHandle(ScHandle);
    }
}
