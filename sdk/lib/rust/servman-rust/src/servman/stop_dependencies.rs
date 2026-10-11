use crate::servman::misc::AllocAndLoadString;
use crate::servman::query::GetServiceConfig;
use crate::servman::dependencies_tv2::TV2_GetDependants;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _STOP_DATA {
    pub ServiceName: LPWSTR,
    pub DisplayName: LPWSTR,
    pub ServiceList: LPWSTR,
}

pub type STOP_DATA = _STOP_DATA;
pub type PSTOP_DATA = *mut _STOP_DATA;

unsafe extern "C" fn AddServiceToList(lpServiceList: *mut LPWSTR, lpServiceToAdd: LPWSTR) -> LPWSTR {
    let mut lpNewList: LPWSTR = core::ptr::null_mut();
    let mut ptr: LPWSTR = core::ptr::null_mut();
    let mut dwToAddSize: DWORD = 0;
    let mut dwCurSize: DWORD = 0;

    dwToAddSize = (wcslen(lpServiceToAdd as *const WCHAR) as DWORD).wrapping_add(1);

    /* Is this is the first in the list? */
    if (*lpServiceList).is_null() {
        /* Add another char for double null */
        dwToAddSize = dwToAddSize.wrapping_add(1);

        lpNewList = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwToAddSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !lpNewList.is_null() {
            /* Copy the service name */
            StringCchCopy(
                lpNewList,
                dwToAddSize as usize,
                lpServiceToAdd,
            );

            /* Add the double null char */
            *lpNewList.offset(dwToAddSize.wrapping_sub(1) as isize) = 0x0000u16;
        }
    } else {
        ptr = *lpServiceList;
        dwCurSize = 0;

        /* Get the list size */
        loop {
            /* Break when we hit the double null */
            if *ptr == 0x0000u16 && *ptr.offset(1 as isize) == 0x0000u16 {
                break;
            }

            ptr = ptr.offset(1);
            dwCurSize = dwCurSize.wrapping_add(1);
        }
        dwCurSize = dwCurSize.wrapping_add(1);

        /* Add another char for double null */
        dwCurSize = dwCurSize.wrapping_add(1);

        /* Extend the list size */
        lpNewList = HeapReAlloc(
            GetProcessHeap(),
            0,
            *lpServiceList as *mut core::ffi::c_void,
            (dwCurSize.wrapping_add(dwToAddSize) as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !lpNewList.is_null() {
            /* Copy the service name */
            StringCchCopy(
                lpNewList.offset(dwCurSize.wrapping_sub(1) as isize),
                dwToAddSize as usize,
                lpServiceToAdd,
            );

            /* Add the double null char */
            *lpNewList.offset(dwCurSize.wrapping_add(dwToAddSize).wrapping_sub(1) as isize) = 0x0000u16;
        }
    }

    return lpNewList;
}

unsafe extern "C" fn BuildListOfServicesToStop(
    lpServiceList: *mut LPWSTR,
    lpServiceName: LPWSTR,
) -> BOOL {
    let mut lpServiceStatus: LPENUM_SERVICE_STATUS =
        core::ptr::null_mut::<ENUM_SERVICE_STATUSW>();
    let mut dwCount: DWORD = 0;
    let mut i: DWORD = 0;
    let mut bRet: BOOL = (FALSE) as i32;

    /* Get a list of service dependents */
    lpServiceStatus = TV2_GetDependants(lpServiceName, core::ptr::addr_of_mut!(dwCount));
    if !lpServiceStatus.is_null() {
        i = 0;
        while i < dwCount {
            /* Does this service need stopping? */
            if (*lpServiceStatus.offset(i as isize)).ServiceStatus.dwCurrentState
                != SERVICE_STOPPED
                && (*lpServiceStatus.offset(i as isize)).ServiceStatus.dwCurrentState
                    != SERVICE_STOP_PENDING
            {
                /* Add the service to the list */
                *lpServiceList = AddServiceToList(
                    lpServiceList,
                    (*lpServiceStatus.offset(i as isize)).lpServiceName,
                );

                /* We've got one */
                bRet = (TRUE) as i32;
            }
            i = i.wrapping_add(1);
        }

        HeapFree(
            GetProcessHeap(),
            0,
            lpServiceStatus as PVOID,
        );
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn GetListOfServicesToStop(lpServiceName: LPWSTR) -> LPWSTR {
    let mut lpServiceList: LPWSTR = core::ptr::null_mut();
    if BuildListOfServicesToStop(core::ptr::addr_of_mut!(lpServiceList), lpServiceName) != 0 {
        return lpServiceList;
    } else {
        return core::ptr::null_mut();
    }
}

unsafe extern "C" fn AddServiceNamesToStop(hServiceListBox: HWND, lpServiceList: LPWSTR) {
    let mut lpServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
    let mut lpStr: LPWSTR;

    lpStr = lpServiceList;

    /* Loop through all the services in the list */
    loop {
        /* Break when we hit the double null */
        if *lpStr == 0x0000u16 && *lpStr.offset(1 as isize) == 0x0000u16 {
            break;
        }

        /* If this isn't our first time in the loop we'll
           have been left on a null char */
        if *lpStr == 0x0000u16 {
            lpStr = lpStr.add(1);
        }

        /* Get the service's display name */
        lpServiceConfig = GetServiceConfig(lpStr);
        if !lpServiceConfig.is_null() {
            /* Add the service to the listbox */
            SendMessageW(
                hServiceListBox,
                LB_ADDSTRING as UINT,
                0 as WPARAM,
                (*lpServiceConfig).lpDisplayName as LPARAM,
            );

            HeapFree(GetProcessHeap(), 0 as DWORD, lpServiceConfig as LPVOID);
        }

        /* Move onto the next string */
        while *lpStr != 0x0000u16 {
            lpStr = lpStr.add(1);
        }
    }
}

unsafe extern "C" fn InitDialog(
    hDlg: HWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let mut StopData: PSTOP_DATA;
    let mut hServiceListBox: HWND;
    let mut lpPartialStr: LPWSTR = core::mem::zeroed();
    let mut lpStr: LPWSTR;
    let mut fullLen: DWORD;
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut bRet: BOOL = (FALSE) as i32;

    StopData = lParam as PSTOP_DATA;

    /* Load the icon for the window */
    hIcon = LoadImageW(
        hInstance,
        MAKEINTRESOURCE!(IDI_SM_ICON),
        IMAGE_ICON,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CXSMICON) as i32),
        0,
    ) as HICON;
    if !hIcon.is_null() {
        /* Set it */
        SendMessageW(
            hDlg,
            WM_SETICON,
            (ICON_SMALL) as WPARAM,
            hIcon as LPARAM,
        );
        DestroyIcon(hIcon);
    }

    /* Load the stop depends note */
    if AllocAndLoadString(&raw mut lpPartialStr, hInstance, IDS_STOP_DEPENDS) != 0 {
        /* Get the length required */
        fullLen = wcslen(lpPartialStr as *const wchar_t)
            .wrapping_add(wcslen((*StopData).DisplayName as *const wchar_t))
            .wrapping_add(1) as DWORD;

        lpStr = HeapAlloc(
            ProcessHeap,
            0,
            (fullLen as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !lpStr.is_null() {
            /* Add the service name to the depends note */
            _snwprintf(
                lpStr as *mut wchar_t,
                fullLen as size_t,
                lpPartialStr as *const wchar_t,
                (*StopData).DisplayName,
            );

            /* Add the string to the dialog */
            SendDlgItemMessageW(
                hDlg,
                (IDC_STOP_DEPENDS as i32),
                WM_SETTEXT,
                0,
                lpStr as LPARAM,
            );

            HeapFree(ProcessHeap, 0, lpStr as PVOID);

            bRet = (TRUE) as i32;
        }

        LocalFree(lpPartialStr as HLOCAL);
    }

    /* Display the list of services which need stopping */
    hServiceListBox = GetDlgItem(hDlg, (IDC_STOP_DEPENDS_LB as i32));
    if !hServiceListBox.is_null() {
        AddServiceNamesToStop(hServiceListBox, (*StopData).ServiceList as LPWSTR);
    }

    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn StopDependsDialogProc(
    hDlg: HWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match Message {
        WM_INITDIALOG => {
            return InitDialog(hDlg, Message, wParam, lParam) as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK | IDCANCEL => {
                    EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn CreateStopDependsDialog(hParent: HWND, ServiceName: LPWSTR, DisplayName: LPWSTR, ServiceList: LPWSTR) -> BOOL {
    let mut StopData: STOP_DATA = STOP_DATA {
        ServiceName: core::ptr::null_mut(),
        DisplayName: core::ptr::null_mut(),
        ServiceList: core::ptr::null_mut(),
    };
    let mut Result: INT_PTR;

    StopData.ServiceName = ServiceName;
    StopData.DisplayName = DisplayName;
    StopData.ServiceList = ServiceList;

    Result = DialogBoxParamW(
        hInstance,
        MAKEINTRESOURCEW!(IDD_DLG_DEPEND_STOP) as LPCWSTR,
        hParent,
        Some(StopDependsDialogProc),
        core::ptr::addr_of_mut!(StopData) as LPARAM,
    );
    if Result == IDOK as INT_PTR {
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
