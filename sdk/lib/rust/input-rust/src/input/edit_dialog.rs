use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn EditDialogProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            let mut pCurrentLayout: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();
            let mut pInput: *mut INPUT_LIST_NODE = core::ptr::null_mut();
            let mut hwndList: HWND = core::ptr::null_mut();

            pInput = lParam as *mut INPUT_LIST_NODE;

            if pInput.is_null() {
                return TRUE as INT_PTR;
            }

            SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, pInput as LONG_PTR);

            SetWindowTextW(
                GetDlgItem(hwndDlg, (IDC_INPUT_LANG_STR as i32)),
                (*(*pInput).pLocale).pszName as LPCWSTR,
            );

            hwndList = GetDlgItem(hwndDlg, (IDC_KB_LAYOUT_IME_COMBO as i32));

            pCurrentLayout = LayoutList_GetFirst();
            while !pCurrentLayout.is_null() {
                let mut iItemIndex: INT = 0;

                iItemIndex = ComboBox_AddString!(hwndList, (*pCurrentLayout).pszName as LPCWSTR);
                ComboBox_SetItemData!(hwndList, iItemIndex, pCurrentLayout);

                pCurrentLayout = (*pCurrentLayout).pNext as *mut LAYOUT_LIST_NODE;
            }

            ComboBox_SelectString!(
                hwndList,
                0,
                (*(*pInput).pLayout).pszName as LPCWSTR,
            );
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    let mut pInput: *mut INPUT_LIST_NODE = core::ptr::null_mut();
                    let mut hwndList: HWND = core::ptr::null_mut();

                    hwndList = GetDlgItem(hwndDlg, (IDC_KB_LAYOUT_IME_COMBO as i32));

                    pInput = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as *mut INPUT_LIST_NODE;

                    if !pInput.is_null() {
                        let mut pNewLayout: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();

                        pNewLayout = ComboBox_GetItemData!(
                            hwndList,
                            ComboBox_GetCurSel!(hwndList),
                        ) as *mut LAYOUT_LIST_NODE;
                        if !pNewLayout.is_null() {
                            (*pInput).pLayout = pNewLayout;
                            (*pInput).wFlags |= INPUT_LIST_NODE_FLAG_EDITED as WORD;
                        }
                    }

                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
