use crate::servman::misc::AllocAndLoadString;
use crate::servman::misc::DisplayString;
use crate::servman::query::SetServiceDescription;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut bHelpOpen: BOOL = (FALSE) as i32;

#[repr(C)]
pub struct CREATE_DATA {
    pub hSelf: HWND,
    pub ServiceName: LPWSTR,
    pub DisplayName: LPWSTR,
    pub BinPath: LPWSTR,
    pub Description: LPWSTR,
    pub Options: LPWSTR,
}

pub type PCREATE_DATA = *mut CREATE_DATA;

unsafe extern "C" fn DoCreate(Data: PCREATE_DATA) -> BOOL {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hSc: SC_HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = FALSE as BOOL;

    /* open handle to the SCM */
    hSCManager = OpenSCManagerW(
        core::ptr::null(),
        core::ptr::null(),
        SC_MANAGER_ALL_ACCESS as DWORD,
    );
    if !hSCManager.is_null() {
        hSc = CreateServiceW(
            hSCManager,
            (*Data).ServiceName as LPCWSTR,
            (*Data).DisplayName as LPCWSTR,
            SERVICE_ALL_ACCESS as DWORD,
            SERVICE_WIN32_OWN_PROCESS as DWORD,
            SERVICE_DEMAND_START as DWORD,
            SERVICE_ERROR_NORMAL as DWORD,
            (*Data).BinPath as LPCWSTR,
            core::ptr::null(),
            core::ptr::null_mut(),
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
        );

        if !hSc.is_null() {
            let mut lpSuccess: LPWSTR = core::ptr::null_mut();

            /* Set the service description as CreateService
               does not do this for us */
            SetServiceDescription((*Data).ServiceName, (*Data).Description);

            /* report success to user */
            if AllocAndLoadString(
                core::ptr::addr_of_mut!(lpSuccess),
                hInstance,
                IDS_CREATE_SUCCESS as UINT,
            ) != 0
            {
                DisplayString(lpSuccess as PWCHAR);

                LocalFree(lpSuccess as HLOCAL);
            }

            CloseServiceHandle(hSc);
            bRet = TRUE as BOOL;
        }

        CloseServiceHandle(hSCManager);
    }

    return bRet;
}

unsafe extern "C" fn GetStringFromDialog(Data: PCREATE_DATA, id: UINT) -> LPWSTR {
    let mut hwnd: HWND = core::ptr::null_mut();
    let mut lpString: LPWSTR = core::ptr::null_mut();
    let mut iLen: INT = 0;
    hwnd = GetDlgItem((*Data).hSelf, id as core::ffi::c_int);
    if !hwnd.is_null() {
        iLen = GetWindowTextLengthW(hwnd) as INT;
        if iLen != 0 {
            lpString = HeapAlloc(
                ProcessHeap,
                0 as DWORD,
                ((iLen + 1) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            if !lpString.is_null() {
                GetWindowTextW(
                    hwnd,
                    lpString,
                    iLen + 1,
                );
            }
        }
    }
    return lpString;
}

unsafe extern "C" fn GetDataFromDialog(mut Data: PCREATE_DATA) -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;

    if {
        (*Data).ServiceName = GetStringFromDialog(Data, IDC_CREATE_SERVNAME);
        !(*Data).ServiceName.is_null()
    } {
        if {
            (*Data).DisplayName = GetStringFromDialog(Data, IDC_CREATE_DISPNAME);
            !(*Data).DisplayName.is_null()
        } {
            if {
                (*Data).BinPath = GetStringFromDialog(Data, IDC_CREATE_PATH);
                !(*Data).BinPath.is_null()
            } {
                (*Data).Description = GetStringFromDialog(Data, IDC_CREATE_DESC);
                (*Data).Options = GetStringFromDialog(Data, IDC_CREATE_OPTIONS);

                bRet = (TRUE) as i32;
            }
        }
    }

    return bRet;
}

unsafe extern "C" fn FreeMemory(Data: PCREATE_DATA) {
    if !(*Data).ServiceName.is_null() {
        HeapFree(ProcessHeap, 0, (*Data).ServiceName as PVOID);
    }
    if !(*Data).DisplayName.is_null() {
        HeapFree(ProcessHeap, 0, (*Data).DisplayName as PVOID);
    }
    if !(*Data).BinPath.is_null() {
        HeapFree(ProcessHeap, 0, (*Data).BinPath as PVOID);
    }
    if !(*Data).Description.is_null() {
        HeapFree(ProcessHeap, 0, (*Data).Description as PVOID);
    }
    if !(*Data).Options.is_null() {
        HeapFree(ProcessHeap, 0, (*Data).Options as PVOID);
    }
    HeapFree(ProcessHeap, 0, Data as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn CreateHelpDialogProc(
    mut hDlg: HWND,
    mut message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut hHelp: HWND = core::ptr::null_mut();
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut Buf: [WCHAR; 1000] = [0; 1000];

    match message {
        WM_INITDIALOG => {
            hIcon = LoadImageW(
                hInstance,
                MAKEINTRESOURCE!(IDI_SM_ICON) as LPCWSTR,
                IMAGE_ICON as UINT,
                16 as ::core::ffi::c_int,
                16 as ::core::ffi::c_int,
                0 as UINT,
            ) as HICON;

            SendMessageW(
                hDlg,
                WM_SETICON as UINT,
                ICON_SMALL as WPARAM,
                hIcon as LPARAM,
            );

            hHelp = GetDlgItem(
                hDlg,
                (IDC_CREATE_HELP as i32),
            );

            LoadStringW(
                hInstance,
                IDS_HELP_OPTIONS as UINT,
                &raw mut Buf as LPWSTR,
                (core::mem::size_of::<[WCHAR; 1000]>() as usize)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                    as ::core::ffi::c_int,
            );

            SetWindowTextW(hHelp, &raw mut Buf as *mut WCHAR as LPCWSTR);

            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            if ((LOWORD!(wParam) as ::core::ffi::c_int) as u32 == IDOK)
                || ((LOWORD!(wParam) as ::core::ffi::c_int) as u32 == IDCANCEL)
            {
                bHelpOpen = FALSE as BOOL;
                DestroyIcon(hIcon);
                DestroyWindow(hDlg);
                return TRUE as INT_PTR;
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn CreateDialogProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut hIcon: HICON = core::ptr::null_mut();

    'outer: {
        match message {
            WM_INITDIALOG => {
                hIcon = LoadImage(
                    hInstance,
                    MAKEINTRESOURCE!(IDI_SM_ICON),
                    IMAGE_ICON,
                    16,
                    16,
                    0,
                ) as HICON;
                if !hIcon.is_null() {
                    SendMessage(
                        hDlg,
                        WM_SETICON,
                        (ICON_SMALL) as _,
                        hIcon as LPARAM,
                    );
                    DestroyIcon(hIcon);
                }

                return TRUE as INT_PTR;
            }

            WM_COMMAND => {
                'inner: {
                    match (LOWORD!(wParam)) as u32 {
                        IDOK => {
                            let mut Data: PCREATE_DATA;

                            Data = HeapAlloc(
                                ProcessHeap,
                                HEAP_ZERO_MEMORY,
                                core::mem::size_of::<CREATE_DATA>() as SIZE_T,
                            ) as PCREATE_DATA;
                            if !Data.is_null() {
                                (*Data).hSelf = hDlg;

                                if GetDataFromDialog(Data) != 0 {
                                    DoCreate(Data);
                                } else {
                                    /* Something went wrong, leave the dialog
                                     * open so they can try again */
                                    FreeMemory(Data);
                                    break 'inner;
                                }

                                FreeMemory(Data);
                            }

                            EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                            return TRUE as INT_PTR;
                        }

                        IDCANCEL => {
                            EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                            return TRUE as INT_PTR;
                        }

                        ID_CREATE_HELP => {
                            let mut hHelp: HWND;

                            if bHelpOpen == 0 {
                                hHelp = CreateDialog!(
                                    hInstance,
                                    MAKEINTRESOURCE!(IDD_DLG_HELP_OPTIONS),
                                    hDlg,
                                    Some(CreateHelpDialogProc),
                                );
                                if !hHelp.is_null() {
                                    bHelpOpen = (TRUE) as i32;
                                }
                            }
                        }

                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }

    return FALSE as INT_PTR;
}
