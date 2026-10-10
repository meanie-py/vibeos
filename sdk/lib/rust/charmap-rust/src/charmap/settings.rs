use crate::charmap::charmap::ChangeMapFont;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_szGeneralRegKey: [TCHAR; 27] = unsafe {
    ::core::mem::transmute::<[u8; 54], [TCHAR; 27]>(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0C\0h\0a\0r\0M\0a\0p\0\0\0",
    )
};

#[no_mangle]
pub static mut hWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;
#[no_mangle]
pub unsafe extern "C" fn QueryStringValue(
    mut hKey: HKEY,
    mut lpSubKey: LPCTSTR,
    mut lpValueName: LPCTSTR,
    mut pszBuffer: LPTSTR,
    mut dwBufferLen: DWORD,
) -> LONG {
    let mut lResult: LONG;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut cbData: DWORD;
    let mut dwType: DWORD = core::mem::zeroed();

    'done: {
        if !lpSubKey.is_null() {
            lResult = RegOpenKey(hKey, lpSubKey, core::ptr::addr_of_mut!(hSubKey)) as LONG;
            if lResult != ERROR_SUCCESS as LONG {
                break 'done;
            }
            hKey = hSubKey;
        }

        cbData = dwBufferLen
            .wrapping_sub(1 as DWORD)
            .wrapping_mul(core::mem::size_of::<TCHAR>() as DWORD);
        lResult = RegQueryValueEx(
            hKey,
            lpValueName,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            pszBuffer as LPBYTE,
            core::ptr::addr_of_mut!(cbData),
        ) as LONG;
        if lResult != ERROR_SUCCESS as LONG {
            break 'done;
        }
        if dwType != REG_SZ as DWORD {
            lResult = -(1 as LONG);
            break 'done;
        }

        *pszBuffer.offset((cbData as usize / core::mem::size_of::<TCHAR>()) as isize) =
            '\0' as i32 as TCHAR;
    }

    if lResult != ERROR_SUCCESS as LONG {
        *pszBuffer.offset(0 as isize) = '\0' as i32 as TCHAR;
    }
    if !hSubKey.is_null() {
        RegCloseKey(hSubKey);
    }
    return lResult;
}
#[no_mangle]
pub unsafe extern "C" fn LoadSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut iItemIndex: i32 = -1;

    if (RegOpenKeyEx(HKEY_CURRENT_USER, core::ptr::addr_of!(g_szGeneralRegKey) as LPCTSTR, 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as u32 == ERROR_SUCCESS
    {
        let mut szBuffer: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut dwAdvancedChecked: DWORD = 0;
        let mut type_0: DWORD = 0;
        let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        let mut lstatus: LSTATUS = 0;

        let FontStr: [u16; 5] = ['F' as u16, 'o' as u16, 'n' as u16, 't' as u16, 0];
        let CodePageStr: [u16; 9] = ['C' as u16, 'o' as u16, 'd' as u16, 'e' as u16, 'P' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 0];
        let AdvancedStr: [u16; 9] = ['A' as u16, 'd' as u16, 'v' as u16, 'a' as u16, 'n' as u16, 'c' as u16, 'e' as u16, 'd' as u16, 0];

        /* Restore last selected font */
        if (QueryStringValue(HKEY_CURRENT_USER, core::ptr::addr_of!(g_szGeneralRegKey) as LPCTSTR, FontStr.as_ptr() as LPCTSTR, core::ptr::addr_of_mut!(szBuffer) as LPTSTR, (core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>() / core::mem::size_of_val(&szBuffer[0])) as DWORD)) as u32 == ERROR_SUCCESS
        {
            //Get combobox handle
            hWnd = GetDlgItem(hCharmapDlg, (IDC_FONTCOMBO as i32));

            //Search for match and return index if match found
            iItemIndex = ComboBox_FindStringExact!(hWnd, -1, core::ptr::addr_of!(szBuffer) as LPCTSTR);
            if iItemIndex != CB_ERR
            {
                ComboBox_SetCurSel!(hWnd, iItemIndex);
                ChangeMapFont(hCharmapDlg);
            }
        }

        /* Restore last selected character set */
        if (QueryStringValue(HKEY_CURRENT_USER, core::ptr::addr_of!(g_szGeneralRegKey) as LPCTSTR, CodePageStr.as_ptr() as LPCTSTR, core::ptr::addr_of_mut!(szBuffer) as LPTSTR, (core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>() / core::mem::size_of_val(&szBuffer[0])) as DWORD)) as u32 == ERROR_SUCCESS
        {
            //Get combobox handle
            hWnd = GetDlgItem(hCharmapDlg, (IDC_COMBO_CHARSET as i32));

            iItemIndex = ComboBox_FindStringExact!(hWnd, -1, core::ptr::addr_of!(szBuffer) as LPCTSTR);
            if iItemIndex != CB_ERR
            {
                ComboBox_SetCurSel!(hWnd, iItemIndex);
            }
        }

        lstatus = RegQueryValueEx(hKey, AdvancedStr.as_ptr() as LPCTSTR, core::ptr::null_mut(), core::ptr::addr_of_mut!(type_0), core::ptr::addr_of_mut!(dwAdvancedChecked) as LPBYTE, core::ptr::addr_of_mut!(size));
        if (lstatus) as u32 == ERROR_SUCCESS && type_0 == REG_DWORD && dwAdvancedChecked != FALSE
        {
            SendDlgItemMessage(hCharmapDlg, (IDC_CHECK_ADVANCED as i32), BM_CLICK, 0, 0);
        }

    RegCloseKey(hKey);
    }
    else
    {
        /* Default font seems to be Arial */
        let ArialStr: [u16; 6] = ['A' as u16, 'r' as u16, 'i' as u16, 'a' as u16, 'l' as u16, 0];

        hWnd = GetDlgItem(hCharmapDlg, (IDC_FONTCOMBO as i32));

        iItemIndex = ComboBox_FindStringExact!(hWnd, -1, ArialStr.as_ptr() as LPCTSTR);
        if iItemIndex != CB_ERR
        {
            ComboBox_SetCurSel!(hWnd, iItemIndex);
            ChangeMapFont(hCharmapDlg);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SaveSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();

    if (RegCreateKeyEx(HKEY_CURRENT_USER, core::ptr::addr_of!(g_szGeneralRegKey) as LPCTSTR, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKey), core::ptr::null_mut())) as u32 == ERROR_SUCCESS
    {
        let mut szBuffer: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let FontName: [u16; 5] = ['F' as u16, 'o' as u16, 'n' as u16, 't' as u16, 0];
        let CodePageName: [u16; 9] = ['C' as u16, 'o' as u16, 'd' as u16, 'e' as u16, 'P' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 0];
        let AdvancedName: [u16; 9] = ['A' as u16, 'd' as u16, 'v' as u16, 'a' as u16, 'n' as u16, 'c' as u16, 'e' as u16, 'd' as u16, 0];

        hWnd = GetDlgItem(hCharmapDlg, (IDC_FONTCOMBO as i32));
        ComboBox_GetText!(hWnd, core::ptr::addr_of_mut!(szBuffer) as LPTSTR, MAX_PATH);

        if *(core::ptr::addr_of_mut!(szBuffer) as *mut TCHAR) != 0 {
            RegSetValueEx(hKey, core::ptr::addr_of!(FontName) as LPCTSTR, 0, REG_SZ, core::ptr::addr_of_mut!(szBuffer) as LPBYTE, MAX_PATH as DWORD);
        }

        hWnd = GetDlgItem(hCharmapDlg, (IDC_COMBO_CHARSET as i32));
        ComboBox_GetText!(hWnd, core::ptr::addr_of_mut!(szBuffer) as LPTSTR, MAX_PATH);

        if *(core::ptr::addr_of_mut!(szBuffer) as *mut TCHAR) != 0 {
            RegSetValueEx(hKey, core::ptr::addr_of!(CodePageName) as LPCTSTR, 0, REG_SZ, core::ptr::addr_of_mut!(szBuffer) as LPBYTE, MAX_PATH as DWORD);
        }

        RegSetValueEx(hKey, core::ptr::addr_of!(AdvancedName) as LPCTSTR, 0, REG_DWORD, core::ptr::addr_of_mut!(Settings.IsAdvancedView) as LPBYTE, core::mem::size_of::<DWORD>() as DWORD);

        RegCloseKey(hKey);
    }
}
