use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "system" fn AboutDialogProc(
    mut hDlg: HWND,
    mut message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    static mut hIcon: HICON = core::ptr::null_mut::<HICON__>() as HICON;

    match message {
        WM_INITDIALOG => {
            let mut hLicenseEditWnd: HWND = core::ptr::null_mut::<HWND__>();
            let mut strLicense: [WCHAR; 700] = [0; 700];

            hIcon = LoadImageW(
                hInstance,
                MAKEINTRESOURCEW!(IDI_ICON),
                IMAGE_ICON as UINT,
                16 as ::core::ffi::c_int,
                16 as ::core::ffi::c_int,
                0 as UINT,
            ) as HICON;
            if !hIcon.is_null() {
                SendMessageW(
                    hDlg,
                    WM_SETICON as UINT,
                    ICON_SMALL as WPARAM,
                    hIcon as LPARAM,
                );
            }

            hLicenseEditWnd = GetDlgItem(
                hDlg,
                (IDC_LICENSE_EDIT as i32),
            );

            if LoadStringW(
                hInstance,
                IDS_LICENSE as UINT,
                &raw mut strLicense as LPWSTR,
                (core::mem::size_of::<[WCHAR; 700]>() as usize)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                    as ::core::ffi::c_int,
            ) != 0
            {
                SetWindowTextW(
                    hLicenseEditWnd,
                    &raw mut strLicense as *mut WCHAR as LPCWSTR,
                );
            }

            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            if ((LOWORD!(wParam) as ::core::ffi::c_int)) as u32 == IDOK
                || ((LOWORD!(wParam) as ::core::ffi::c_int)) as u32 == IDCANCEL
            {
                DestroyIcon(hIcon);
                EndDialog(
                    hDlg,
                    LOWORD!(wParam) as INT_PTR,
                );
                return (TRUE) as INT_PTR;
            }

            // break;
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn ShowAboutDlg(hWndParent: HWND) {
    DialogBoxW!(
        hInstance,
        MAKEINTRESOURCEW!(IDD_ABOUTBOX),
        hWndParent,
        Some(AboutDialogProc),
    );
}
