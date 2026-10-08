use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut Applets: [APPLET; 1] = unsafe {
    [APPLET {
        idIcon: (IDI_CPLSYSTEM as i32),
        idName: (IDS_CPLSYSTEMNAME as i32),
        idDescription: (IDS_CPLSYSTEMDESCRIPTION as i32),
        AppletProc: Some(SystemApplet),
    }]
};

pub const NUM_APPLETS: usize = 1;
#[no_mangle]
pub unsafe extern "C" fn AddColumns(hList: HWND) {
    let mut dummy: LVCOLUMNW = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut Size: i32;
    let mut szBuf: [TCHAR; 256] = [0; 256];

    GetClientRect(hList, core::ptr::addr_of_mut!(rect));
    Size = rect.right - rect.left - 250;

    LoadString(
        hApplet,
        IDS_STATUS as u32,
        (szBuf.as_mut_ptr()).cast(),
        (core::mem::size_of_val(&szBuf) / core::mem::size_of::<TCHAR>()) as i32,
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(dummy) as *mut core::ffi::c_void,
        core::mem::size_of::<LVCOLUMNA>(),
    );
    dummy.mask = (LVCF_TEXT | LVCF_WIDTH | LVCF_SUBITEM) as u32;
    dummy.iSubItem = 0 as i32;
    dummy.pszText = (szBuf.as_mut_ptr() as LPTSTR).cast();
    dummy.cx = Size;
    let _ = ListView_InsertColumn!(hList, 0 as usize, core::ptr::addr_of_mut!(dummy));

    GetClientRect(hList, core::ptr::addr_of_mut!(rect));

    LoadString(
        hApplet,
        IDS_CONTROLLER as u32,
        (szBuf.as_mut_ptr()).cast(),
        (core::mem::size_of_val(&szBuf) / core::mem::size_of::<TCHAR>()) as i32,
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(dummy) as *mut core::ffi::c_void,
        core::mem::size_of::<LVCOLUMNA>(),
    );
    dummy.mask = (LVCF_TEXT | LVCF_WIDTH | LVCF_SUBITEM) as u32;
    dummy.iSubItem = 0 as i32;
    dummy.pszText = (szBuf.as_mut_ptr() as LPTSTR).cast();
    dummy.cx = rect.right - rect.left - Size;
    let _ = ListView_InsertColumn!(hList, 0 as usize, core::ptr::addr_of_mut!(dummy));
}

#[no_mangle]
pub unsafe extern "system" fn AdvancedPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            let mut szBuf: [WCHAR; 256] = [0; 256];
            let mut hComboHwnd: HWND = GetDlgItem(hwndDlg, (IDC_PREFERRED_DEV_COMBO as i32));

            LoadStringW(hApplet, IDS_NONE as UINT, core::ptr::addr_of_mut!(szBuf) as LPWSTR, _countof!(szBuf) as ::core::ffi::c_int);
            SendMessageW(hComboHwnd, CB_ADDSTRING as UINT, 0 as WPARAM, core::ptr::addr_of_mut!(szBuf) as *mut WCHAR as LPARAM);
            SendMessageW(hComboHwnd, CB_SETCURSEL as UINT, 0 as WPARAM, core::ptr::null_mut::<core::ffi::c_void>() as LPARAM);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    EndDialog(hwndDlg, LOWORD!(wParam) as INT_PTR);
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD!(wParam) as INT_PTR);
                }

                _ => {}
            }
        }

        WM_NOTIFY => {}

        WM_SYSCOMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                SC_CONTEXTHELP => {
                    // Not implemented yet
                }

                _ => {}
            }
        }

        _ => {}
    }

    return 0 as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn CustomPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            let mut szBuf: [WCHAR; 2] = [0; 2];
            let mut hComboHwnd: HWND;
            szBuf[1] = UNICODE_NULL;

            CheckDlgButton(hwndDlg, (IDC_JOYSTICK_RADIO as i32), BST_CHECKED);

            hComboHwnd = GetDlgItem(hwndDlg, (IDC_AXES_COMBO as i32));
            szBuf[0] = 0x0032u16;
            SendMessageW(hComboHwnd, CB_ADDSTRING, 0, szBuf.as_mut_ptr() as LPARAM);
            szBuf[0] = 0x0033u16;
            SendMessageW(hComboHwnd, CB_ADDSTRING, 0, szBuf.as_mut_ptr() as LPARAM);
            szBuf[0] = 0x0034u16;
            SendMessageW(hComboHwnd, CB_ADDSTRING, 0, szBuf.as_mut_ptr() as LPARAM);
            SendMessageW(hComboHwnd, CB_SETCURSEL, 0, 0);

            hComboHwnd = GetDlgItem(hwndDlg, (IDC_BUTTONS_COMBO as i32));
            szBuf[0] = 0x0030u16;
            SendMessageW(hComboHwnd, CB_ADDSTRING, 0, szBuf.as_mut_ptr() as LPARAM);
            szBuf[0] = 0x0031u16;
            SendMessageW(hComboHwnd, CB_ADDSTRING, 0, szBuf.as_mut_ptr() as LPARAM);
            szBuf[0] = 0x0032u16;
            SendMessageW(hComboHwnd, CB_ADDSTRING, 0, szBuf.as_mut_ptr() as LPARAM);
            szBuf[0] = 0x0033u16;
            SendMessageW(hComboHwnd, CB_ADDSTRING, 0, szBuf.as_mut_ptr() as LPARAM);
            szBuf[0] = 0x0034u16;
            SendMessageW(hComboHwnd, CB_ADDSTRING, 0, szBuf.as_mut_ptr() as LPARAM);
            SendMessageW(hComboHwnd, CB_SETCURSEL, 4, 0);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK => {
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }

                _ => {}
            }
        }

        WM_NOTIFY => {}

        WM_SYSCOMMAND => {
            match (LOWORD!(wParam)) as u32 {
                SC_CONTEXTHELP => {
                    // Not implemented yet
                }

                _ => {}
            }
        }

        _ => {}
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn AddPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    if uMsg == WM_INITDIALOG {
        // break
    } else if uMsg == WM_COMMAND {
        if LOWORD!(wParam) == (IDC_CUSTOM_BUTTON as u16) {
            DialogBox!(
                hApplet,
                MAKEINTRESOURCE!(IDD_CUSTOM),
                hwndDlg,
                Some(CustomPageProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
            );
        } else if (LOWORD!(wParam)) as u32 == IDOK {
            EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
        } else if (LOWORD!(wParam)) as u32 == IDCANCEL {
            EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
        }
    } else if uMsg == WM_NOTIFY {
        // break
    } else if uMsg == WM_SYSCOMMAND {
        if (LOWORD!(wParam)) as u32 == SC_CONTEXTHELP {
            // Not implemented yet
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn MainPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static mut s_hIcon: HICON = core::ptr::null_mut();
    static mut s_hIconSm: HICON = core::ptr::null_mut();
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            AddColumns(GetDlgItem(hwndDlg, (IDC_CONTROLLER_LIST as i32)));
            s_hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDI_CPLSYSTEM)) as HICON;
            s_hIconSm = LoadImageW(
                hApplet,
                MAKEINTRESOURCEW!(IDI_CPLSYSTEM),
                IMAGE_ICON,
                GetSystemMetrics((SM_CXSMICON) as i32),
                GetSystemMetrics((SM_CYSMICON) as i32),
                0,
            ) as HICON;
            SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as WPARAM, s_hIcon as LPARAM);
            SendMessageW(hwndDlg, WM_SETICON, (ICON_SMALL) as WPARAM, s_hIconSm as LPARAM);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_ADD_BUTTON => {
                    DialogBox!(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_ADD),
                        hwndDlg,
                        Some(AddPageProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
                    );
                }

                IDC_ADVANCED_BUTTON => {
                    DialogBox!(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_ADVANCED),
                        hwndDlg,
                        Some(AdvancedPageProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
                    );
                }

                IDOK | IDCANCEL => {
                    DestroyIcon(s_hIcon);
                    DestroyIcon(s_hIconSm);
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }
                _ => {}
            }
        }

        WM_NOTIFY => {}

        WM_SYSCOMMAND => {
            match (LOWORD!(wParam)) as u32 {
                SC_CLOSE => {
                    DestroyIcon(s_hIcon);
                    DestroyIcon(s_hIconSm);
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }

                SC_CONTEXTHELP => {
                    // Not implemented yet
                }
                _ => {}
            }
        }

        _ => {}
    }
    return 0 as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn SystemApplet(hwnd: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    UNREFERENCED_PARAMETER!(uMsg);
    UNREFERENCED_PARAMETER!(lParam1);
    UNREFERENCED_PARAMETER!(lParam2);

    DialogBox!(
        hApplet,
        MAKEINTRESOURCE!(IDD_PROPPAGEMAIN),
        hwnd,
        Some(MainPageProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
    );

    return (TRUE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(hwndCPl: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    let mut CPlInfo: *mut CPLINFO = core::ptr::null_mut();
    let i: UINT = lParam1 as UINT;
    UNREFERENCED_PARAMETER!(hwndCPl);

    match uMsg {
        CPL_INIT => {
            return (TRUE) as LONG;
        }
        CPL_GETCOUNT => {
            return (NUM_APPLETS) as LONG;
        }
        CPL_INQUIRE => {
            if i < (NUM_APPLETS as u32) {
                CPlInfo = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0;
                (*CPlInfo).idIcon = Applets[i as usize].idIcon;
                (*CPlInfo).idName = Applets[i as usize].idName;
                (*CPlInfo).idInfo = Applets[i as usize].idDescription;
            } else {
                return (TRUE) as LONG;
            }
        }
        CPL_DBLCLK => {
            if i < (NUM_APPLETS as u32) {
                Applets[i as usize]
                    .AppletProc
                    .unwrap_unchecked()(hwndCPl, uMsg, lParam1, lParam2);
            } else {
                return (TRUE) as LONG;
            }
        }
        _ => {}
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH | DLL_THREAD_ATTACH => {
            CoInitialize(core::ptr::null_mut());
            hApplet = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
