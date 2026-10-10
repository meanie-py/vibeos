use crate::servman::misc::AllocAndLoadString;
use crate::servman::misc::DisplayString;
use crate::servman::query::GetServiceDescription;
use crate::servman::mainwnd::UpdateServiceCount;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn DoDeleteService(Info: PMAIN_WND_INFO, hDlg: HWND) -> BOOL {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hSc: SC_HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = (FALSE) as i32;

    hSCManager = OpenSCManagerW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SC_MANAGER_ALL_ACCESS,
    );
    if !hSCManager.is_null() {
        hSc = OpenServiceW(
            hSCManager,
            (*(*Info).pCurrentService).lpServiceName as LPCWSTR,
            DELETE,
        );
        if !hSc.is_null() {
            if DeleteService(hSc) != 0 {
                let mut lpSuccess: LPWSTR = core::ptr::null_mut();

                /* report success to user */
                if AllocAndLoadString(
                    core::ptr::addr_of_mut!(lpSuccess),
                    hInstance,
                    IDS_DELETE_SUCCESS,
                ) != 0
                {
                    DisplayString(lpSuccess as PWCHAR);

                    LocalFree(lpSuccess as HLOCAL);
                }

                bRet = (TRUE) as i32;
            }

            CloseServiceHandle(hSc);
        }

        CloseServiceHandle(hSCManager);
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "system" fn DeleteDialogProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut Info: PMAIN_WND_INFO = core::ptr::null_mut();
    let mut hIcon: HICON = core::ptr::null_mut();

    /* Get the window context */
    Info = GetWindowLongPtr(hDlg, GWLP_USERDATA as _) as PMAIN_WND_INFO;
    if Info.is_null() && message != (WM_INITDIALOG as UINT) {
        return FALSE as INT_PTR;
    }

    if message == (WM_INITDIALOG as UINT) {
        let mut lpDescription: LPWSTR = core::ptr::null_mut();

        Info = lParam as PMAIN_WND_INFO;
        if !Info.is_null() {
            SetWindowLongPtrW(
                hDlg,
                GWLP_USERDATA as _,
                Info as LONG_PTR,
            );

            hIcon = LoadImageW(
                hInstance,
                MAKEINTRESOURCE!(IDI_SM_ICON) as LPCWSTR,
                IMAGE_ICON as _,
                16 as _,
                16 as _,
                0 as _,
            ) as HICON;
            if !hIcon.is_null() {
                SendMessageW(
                    hDlg,
                    WM_SETICON as _,
                    ICON_SMALL as _,
                    hIcon as LPARAM,
                );
                DestroyIcon(hIcon);
            }

            SendDlgItemMessageW(
                hDlg,
                IDC_DEL_NAME as _,
                WM_SETTEXT as _,
                0 as _,
                (*(*Info).pCurrentService).lpDisplayName as LPARAM,
            );

            lpDescription = GetServiceDescription((*(*Info).pCurrentService).lpServiceName);
            if !lpDescription.is_null() {
                SendDlgItemMessageW(
                    hDlg,
                    IDC_DEL_DESC as _,
                    WM_SETTEXT as _,
                    0 as _,
                    lpDescription as LPARAM,
                );
                HeapFree(
                    ProcessHeap,
                    0 as _,
                    (lpDescription).cast(),
                );
            }

            return TRUE as INT_PTR;
        }

        return FALSE as INT_PTR;
    } else if message == (WM_COMMAND as UINT) {
        if (LOWORD!(wParam) as UINT) == (IDOK as UINT) {
            if DoDeleteService(Info, hDlg) != 0 {
                let _ = ListView_DeleteItem!((*Info).hListView, (*Info).SelectedItem);
                UpdateServiceCount(Info);
            }
            EndDialog(hDlg, LOWORD!(wParam) as _);
            return TRUE as INT_PTR;
        } else if (LOWORD!(wParam) as UINT) == (IDCANCEL as UINT) {
            EndDialog(hDlg, LOWORD!(wParam) as _);
            return TRUE as INT_PTR;
        }
    }

    return FALSE as INT_PTR;
}
