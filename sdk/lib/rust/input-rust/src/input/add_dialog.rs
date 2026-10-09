use crate::input::input_list::InputList_Add;
use crate::input::locale_list::LocaleList_GetFirst;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_30C9A2BFFBC3: [u16; 8] = [0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_4D70F4A881C7: [u16; 2] = [0x003Au16, 0x0000];
pub static RUSTCONV_WSTR_4F11850FAC45: [u16; 5] = [0x0025u16, 0x0030, 0x0038, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_76A0D4CD5699: [u16; 9] = [0x0069u16, 0x006E, 0x0074, 0x006C, 0x002E, 0x0069, 0x006E, 0x0066, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn GetDefaultLayoutForLocale(mut dwLocaleId: DWORD) -> DWORD {
    let mut dwResult: DWORD = 0;
    let mut hIntlInf: HINF;

    hIntlInf = SetupOpenInfFileW(
        RUSTCONV_WSTR_76A0D4CD5699.as_ptr(),
        core::ptr::null::<WCHAR>(),
        INF_STYLE_WIN4,
        core::ptr::null_mut::<core::ffi::c_uint>(),
    );
    if hIntlInf != INVALID_HANDLE_VALUE {
        let mut szLangID: [WCHAR; (MAX_STR_LEN) as _] = [0; (MAX_STR_LEN) as _];
        let mut InfContext: INFCONTEXT = core::mem::zeroed();

        StringCchPrintfW!(
            &raw mut szLangID as *mut WCHAR,
            ARRAYSIZE!(szLangID),
            RUSTCONV_WSTR_4F11850FAC45.as_ptr(),
            dwLocaleId,
        );

        if SetupFindFirstLineW(
            hIntlInf,
            RUSTCONV_WSTR_30C9A2BFFBC3.as_ptr(),
            &raw mut szLangID as *mut WCHAR,
            &raw mut InfContext,
        ) != 0
        {
            if SetupGetFieldCount(&raw mut InfContext) >= 5 {
                let mut szField: [WCHAR; (MAX_STR_LEN) as _] = [0; (MAX_STR_LEN) as _];

                if SetupGetStringFieldW(
                    &raw mut InfContext,
                    5,
                    &raw mut szField as *mut WCHAR,
                    ARRAYSIZE!(szField),
                    core::ptr::null_mut::<DWORD>(),
                ) != 0
                {
                    if wcslen(&raw mut szField as *mut WCHAR) == 13 {
                        let mut pszSeparator: *mut WCHAR = RUSTCONV_WSTR_4D70F4A881C7.as_ptr() as *mut WCHAR;
                        let mut pszToken: *mut WCHAR;

                        pszToken = wcstok(&raw mut szField as *mut WCHAR, pszSeparator);
                        if !pszToken.is_null() {
                            pszToken = wcstok(core::ptr::null_mut::<WCHAR>(), pszSeparator);
                        }

                        if !pszToken.is_null() {
                            dwResult = DWORDfromString(pszToken);
                        }
                    }
                }
            }
        }

        SetupCloseInfFile(hIntlInf);
    }

    return dwResult;
}

unsafe extern "C" fn OnInitAddDialog(hwndDlg: HWND) {
    let hwndLocaleCombo: HWND = GetDlgItem(hwndDlg, (IDC_INPUT_LANG_COMBO as i32));
    let hwndLayoutCombo: HWND = GetDlgItem(hwndDlg, (IDC_KEYBOARD_LO_COMBO as i32));
    let mut pCurrentLocale: *mut LOCALE_LIST_NODE;
    let mut pCurrentLayout: *mut LAYOUT_LIST_NODE;
    let mut dwDefaultLocaleId: DWORD;
    let mut dwDefaultLayoutId: DWORD;
    let mut iItemIndex: INT;

    dwDefaultLocaleId = GetSystemDefaultLCID();

    pCurrentLocale = LocaleList_GetFirst();
    while !pCurrentLocale.is_null() {
        iItemIndex = ComboBox_AddString!(hwndLocaleCombo, (*pCurrentLocale).pszName);
        ComboBox_SetItemData!(hwndLocaleCombo, iItemIndex, pCurrentLocale);

        if (*pCurrentLocale).dwId == dwDefaultLocaleId {
            ComboBox_SetCurSel!(hwndLocaleCombo, iItemIndex);
        }
        pCurrentLocale = (*pCurrentLocale).pNext as *mut LOCALE_LIST_NODE;
    }

    dwDefaultLayoutId = GetDefaultLayoutForLocale(dwDefaultLocaleId);

    pCurrentLayout = LayoutList_GetFirst();
    while !pCurrentLayout.is_null() {
        iItemIndex = ComboBox_AddString!(hwndLayoutCombo, (*pCurrentLayout).pszName);
        ComboBox_SetItemData!(hwndLayoutCombo, iItemIndex, pCurrentLayout);

        if (*pCurrentLayout).dwKLID == dwDefaultLayoutId {
            ComboBox_SetCurSel!(hwndLayoutCombo, iItemIndex);
        }
        pCurrentLayout = (*pCurrentLayout).pNext as *mut LAYOUT_LIST_NODE;
    }

    if dwDefaultLayoutId == 0 {
        ComboBox_SetCurSel!(hwndLayoutCombo, 0);
    }
}

unsafe extern "C" fn OnCommandAddDialog(hwndDlg: HWND, wParam: WPARAM) {
    match (LOWORD!(wParam)) as u32 {
        IDOK => {
            let hwndLocaleCombo: HWND = GetDlgItem(hwndDlg, (IDC_INPUT_LANG_COMBO as i32));
            let hwndLayoutCombo: HWND = GetDlgItem(hwndDlg, (IDC_KEYBOARD_LO_COMBO as i32));
            let mut pCurrentLocale: *mut LOCALE_LIST_NODE = core::ptr::null_mut();
            let mut pCurrentLayout: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();

            pCurrentLocale = ComboBox_GetItemData!(
                hwndLocaleCombo,
                ComboBox_GetCurSel!(hwndLocaleCombo),
            ) as usize as *mut LOCALE_LIST_NODE;
            pCurrentLayout = ComboBox_GetItemData!(
                hwndLayoutCombo,
                ComboBox_GetCurSel!(hwndLayoutCombo),
            ) as usize as *mut LAYOUT_LIST_NODE;

            InputList_Add(pCurrentLocale, pCurrentLayout);

            EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
        }

        IDCANCEL => {
            EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
        }

        IDC_INPUT_LANG_COMBO => {
            if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                let hwndLocaleCombo: HWND = GetDlgItem(hwndDlg, (IDC_INPUT_LANG_COMBO as i32));
                let hwndLayoutCombo: HWND = GetDlgItem(hwndDlg, (IDC_KEYBOARD_LO_COMBO as i32));
                let mut pCurrentLocale: *mut LOCALE_LIST_NODE = core::ptr::null_mut();

                pCurrentLocale = ComboBox_GetItemData!(
                    hwndLocaleCombo,
                    ComboBox_GetCurSel!(hwndLocaleCombo),
                ) as usize as *mut LOCALE_LIST_NODE;
                if !pCurrentLocale.is_null() {
                    let mut dwLayoutId: DWORD;
                    let mut iIndex: INT;
                    let mut iCount: INT;

                    dwLayoutId = GetDefaultLayoutForLocale((*pCurrentLocale).dwId);

                    iCount = ComboBox_GetCount!(hwndLayoutCombo);

                    iIndex = 0;
                    while iIndex < iCount {
                        let mut pCurrentLayout: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();

                        pCurrentLayout = ComboBox_GetItemData!(hwndLayoutCombo, iIndex) as usize
                            as *mut LAYOUT_LIST_NODE;

                        if !pCurrentLayout.is_null() && (*pCurrentLayout).dwKLID == dwLayoutId {
                            ComboBox_SetCurSel!(hwndLayoutCombo, iIndex);
                            break;
                        }
                        iIndex += 1;
                    }
                }
            }
        }

        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "system" fn AddDialogProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            OnInitAddDialog(hwndDlg);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            OnCommandAddDialog(hwndDlg, wParam);
        }
        WM_DESTROY => {}
        _ => {}
    }

    return FALSE as INT_PTR;
}
