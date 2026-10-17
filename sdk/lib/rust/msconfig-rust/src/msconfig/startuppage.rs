use crate::msconfig::msconfig::DisableAllExcept;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hStartupPage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hStartupListCtrl: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hStartupDialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

// Prototypes for GetAutostartEntriesFromRegistry and
// GetDisabledAutostartEntriesFromRegistry are not emitted here: they are
// provided by the translated function definitions.

#[no_mangle]
pub unsafe extern "system" fn StartupPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut column: LVCOLUMNW = core::mem::zeroed();
    let mut szTemp: [TCHAR; 256] = [0; 256];
    let mut dwStyle: DWORD;

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);

    match message {
        WM_INITDIALOG => {
            hStartupListCtrl = GetDlgItem(hDlg, (IDC_STARTUP_LIST as i32));
            hStartupDialog = hDlg;

            dwStyle = SendMessage(hStartupListCtrl, LVM_GETEXTENDEDLISTVIEWSTYLE, 0, 0) as DWORD;
            dwStyle = dwStyle | LVS_EX_FULLROWSELECT | LVS_EX_CHECKBOXES;
            SendMessage(hStartupListCtrl, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, (dwStyle) as _);

            SetWindowPos(hDlg, core::ptr::null_mut(), 10, 32, 0, 0, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);

            // Initialize the application page's controls
            column.mask = LVCF_TEXT | LVCF_WIDTH;

            LoadString(hInst, IDS_STARTUP_COLUMN_ELEMENT, core::ptr::addr_of_mut!(szTemp) as *mut TCHAR, 256);
            column.pszText = core::ptr::addr_of_mut!(szTemp) as *mut TCHAR;
            column.cx = 150;
            let _ = ListView_InsertColumn!(hStartupListCtrl, 0, core::ptr::addr_of_mut!(column));

            column.mask = LVCF_TEXT | LVCF_WIDTH;
            LoadString(hInst, IDS_STARTUP_COLUMN_CMD, core::ptr::addr_of_mut!(szTemp) as *mut TCHAR, 256);
            column.pszText = core::ptr::addr_of_mut!(szTemp) as *mut TCHAR;
            column.cx = 150;
            let _ = ListView_InsertColumn!(hStartupListCtrl, 1, core::ptr::addr_of_mut!(column));

            column.mask = LVCF_TEXT | LVCF_WIDTH;
            LoadString(hInst, IDS_STARTUP_COLUMN_PATH, core::ptr::addr_of_mut!(szTemp) as *mut TCHAR, 256);
            column.pszText = core::ptr::addr_of_mut!(szTemp) as *mut TCHAR;
            column.cx = 250;
            let _ = ListView_InsertColumn!(hStartupListCtrl, 2, core::ptr::addr_of_mut!(column));

            GetAutostartEntriesFromRegistry(HKEY_LOCAL_MACHINE, [
                'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
                '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
                'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16,
                'w' as u16, 's' as u16, '\\' as u16, 'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16,
                'n' as u16, 't' as u16, 'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16,
                'n' as u16, '\\' as u16, 'R' as u16, 'u' as u16, 'n' as u16, 0,
            ].as_ptr() as *mut TCHAR);
            GetAutostartEntriesFromRegistry(HKEY_CURRENT_USER, [
                'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
                '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
                'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16,
                'w' as u16, 's' as u16, '\\' as u16, 'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16,
                'n' as u16, 't' as u16, 'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16,
                'n' as u16, '\\' as u16, 'R' as u16, 'u' as u16, 'n' as u16, 0,
            ].as_ptr() as *mut TCHAR);
            GetDisabledAutostartEntriesFromRegistry([
                'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
                '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
                'f' as u16, 't' as u16, '\\' as u16, 'S' as u16, 'h' as u16, 'a' as u16, 'r' as u16, 'e' as u16,
                'd' as u16, ' ' as u16, 'T' as u16, 'o' as u16, 'o' as u16, 'l' as u16, 's' as u16, '\\' as u16,
                'M' as u16, 'S' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16,
                '\\' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, 'u' as u16, 'p' as u16,
                'r' as u16, 'e' as u16, 'g' as u16, 0,
            ].as_ptr() as *mut TCHAR);
            GetDisabledAutostartEntriesFromRegistry([
                'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
                '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
                'f' as u16, 't' as u16, '\\' as u16, 'S' as u16, 'h' as u16, 'a' as u16, 'r' as u16, 'e' as u16,
                'd' as u16, ' ' as u16, 'T' as u16, 'o' as u16, 'o' as u16, 'l' as u16, 's' as u16, '\\' as u16,
                'M' as u16, 'S' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16,
                '\\' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, 'u' as u16, 'p' as u16,
                'f' as u16, 'o' as u16, 'l' as u16, 'd' as u16, 'e' as u16, 'r' as u16, 0,
            ].as_ptr() as *mut TCHAR);

            // FIXME: What about HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Userinit
            // FIXME: Common Startup (startmenu)
            DisableAllExcept(hDlg, IDC_STARTUP_LIST); // FIXME: Implement saving
            return (TRUE) as INT_PTR;
        }

        WM_NOTIFY => {
            if wParam == (IDC_STARTUP_LIST as WPARAM) {
                let pnmlv: *mut NMLISTVIEW = lParam as *mut NMLISTVIEW;
                let toggled: UINT = ((*pnmlv).uOldState ^ (*pnmlv).uNewState) & LVIS_STATEIMAGEMASK as UINT;
                if (*pnmlv).hdr.code == (LVN_ITEMCHANGING) as u32 && ((*pnmlv).uChanged & LVIF_STATE as UINT) != 0 && toggled != 0 {
                    // Only allow checkbox changes during WM_INITDIALOG
                    if IsWindowEnabled(GetDlgItem(hDlg, (IDC_BTN_STARTUP_ACTIVATE as i32))) == 0 {
                        MessageBeep(-(1 as ::core::ffi::c_int) as UINT);
                        SetWindowLongPtr(hDlg, DWLP_MSGRESULT, (TRUE) as _);
                        return (TRUE) as INT_PTR;
                    }
                }
            }
        }

        _ => {}
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn GetDisabledAutostartEntriesFromRegistry(szBasePath: *mut TCHAR) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut Index: DWORD = 0;
    let mut SubIndex: DWORD = 0;
    let mut dwValues: DWORD = 0;
    let mut dwSubValues: DWORD = 0;
    let mut dwValueLength: DWORD = 0;
    let mut dwDataLength: DWORD = MAX_VALUE_NAME as DWORD;
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut Data: *mut TCHAR = core::ptr::null_mut();
    let mut szValueName: [TCHAR; MAX_KEY_LENGTH as usize] = [0; MAX_KEY_LENGTH as usize];
    let mut szSubValueName: [TCHAR; MAX_KEY_LENGTH as usize] = [0; MAX_KEY_LENGTH as usize];
    let mut szSubPath: [TCHAR; MAX_KEY_LENGTH as usize] = [0; MAX_KEY_LENGTH as usize];

    if RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        szBasePath,
        0 as DWORD,
        (KEY_READ | KEY_ENUMERATE_SUB_KEYS) as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) == ERROR_SUCCESS as LSTATUS
    {
        if RegQueryInfoKey(
            hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwValues),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == ERROR_SUCCESS as LSTATUS
        {
            Index = 0 as DWORD;
            while Index < dwValues {
                dwValueLength = MAX_KEY_LENGTH as DWORD;
                dwDataLength = MAX_VALUE_NAME as DWORD;
                Data = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    (MAX_VALUE_NAME as SIZE_T)
                        .wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
                ) as *mut TCHAR;
                if Data.is_null() {
                    break;
                }

                if RegEnumKeyEx(
                    hKey,
                    Index,
                    core::ptr::addr_of_mut!(szValueName) as *mut TCHAR,
                    core::ptr::addr_of_mut!(dwValueLength),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) != ERROR_SUCCESS as LSTATUS
                {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        Data as PVOID,
                    );
                    Index = Index.wrapping_add(1);
                    continue;
                }
                // The C's _stprintf into szSubPath (MAX_KEY_LENGTH) took the base path plus a
                // subkey name that is itself up to MAX_KEY_LENGTH: a long autostart entry
                // overran the stack buffer. Bounded; a truncated path just fails to open.
                let _ = StringCchPrintfW!(
                    core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR,
                    MAX_KEY_LENGTH as usize,
                    (&[0x0025, 0x0073, 0x005c, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T "%s\\%s" */,
                    szBasePath,
                    core::ptr::addr_of_mut!(szValueName) as *mut TCHAR
                );
                memset(
                    core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    core::mem::size_of::<LV_ITEM>() as size_t,
                );
                item.mask = LVIF_TEXT as UINT;
                item.iImage = 0 as ::core::ffi::c_int;
                item.pszText = core::ptr::addr_of_mut!(szValueName) as *mut TCHAR as LPWSTR;
                item.iItem = ListView_GetItemCount!(hStartupListCtrl);
                item.lParam = 0 as LPARAM;
                let _ = ListView_InsertItem!(hStartupListCtrl, core::ptr::addr_of_mut!(item));
                if RegOpenKeyEx(
                    HKEY_LOCAL_MACHINE,
                    core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR,
                    0 as DWORD,
                    (KEY_READ | KEY_ENUMERATE_SUB_KEYS) as REGSAM,
                    core::ptr::addr_of_mut!(hSubKey),
                ) == ERROR_SUCCESS as LSTATUS
                {
                    if RegQueryInfoKey(
                        hSubKey,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(dwSubValues),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) == ERROR_SUCCESS as LSTATUS
                    {
                        SubIndex = 0 as DWORD;
                        while SubIndex < dwSubValues {
                            dwValueLength = MAX_KEY_LENGTH as DWORD;
                            dwDataLength = MAX_VALUE_NAME as DWORD;
                            if RegEnumValue(
                                hSubKey,
                                SubIndex,
                                core::ptr::addr_of_mut!(szSubValueName) as *mut TCHAR,
                                core::ptr::addr_of_mut!(dwValueLength),
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                Data as LPBYTE,
                                core::ptr::addr_of_mut!(dwDataLength),
                            ) == ERROR_SUCCESS as LSTATUS
                            {
                                item.iSubItem = -(1 as ::core::ffi::c_int);
                                if _tcscmp(
                                    core::ptr::addr_of_mut!(szSubValueName) as *mut TCHAR,
                                    (&[0x0063, 0x006f, 0x006d, 0x006d, 0x0061, 0x006e, 0x0064, 0x0000u16]).as_ptr() /* _T "command" */,
                                ) == 0
                                {
                                    item.iSubItem = 1 as ::core::ffi::c_int;
                                } else if _tcscmp(
                                    core::ptr::addr_of_mut!(szSubValueName) as *mut TCHAR,
                                    (&[0x006b, 0x0065, 0x0079, 0x0000u16]).as_ptr() /* _T "key" */,
                                ) == 0
                                    || _tcscmp(
                                        core::ptr::addr_of_mut!(szSubValueName) as *mut TCHAR,
                                        (&[0x006c, 0x006f, 0x0063, 0x0061, 0x0074, 0x0069, 0x006f, 0x006e, 0x0000u16]).as_ptr() /* _T "location" */,
                                    ) == 0
                                {
                                    item.iSubItem = 2 as ::core::ffi::c_int;
                                } else if _tcscmp(
                                    core::ptr::addr_of_mut!(szSubValueName) as *mut TCHAR,
                                    (&[0x0069, 0x0074, 0x0065, 0x006d, 0x0000u16]).as_ptr() /* _T "item" */,
                                ) == 0
                                {
                                    item.iSubItem = 0 as ::core::ffi::c_int;
                                }
                                if item.iSubItem != -(1 as ::core::ffi::c_int) {
                                    GetLongPathName(
                                        Data as LPCWSTR,
                                        Data as LPWSTR,
                                        _tcsclen(Data as *const TCHAR) as DWORD,
                                    );
                                    item.pszText = Data as LPWSTR;
                                    SendMessage(
                                        hStartupListCtrl,
                                        LVM_SETITEMTEXT as UINT,
                                        item.iItem as WPARAM,
                                        core::ptr::addr_of_mut!(item) as LPARAM,
                                    );
                                }
                            }
                            SubIndex = SubIndex.wrapping_add(1);
                        }
                    }
                }
                RegCloseKey(hSubKey);
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    Data as PVOID,
                );
                Index = Index.wrapping_add(1);
            }
        }
        RegCloseKey(hKey);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetAutostartEntriesFromRegistry(mut hRootKey: HKEY, mut KeyName: *mut TCHAR) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Index: DWORD = 0;
    let mut dwValues: DWORD = 0;
    let mut retVal: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwValueLength: DWORD = 0;
    let mut dwDataLength: DWORD = MAX_VALUE_NAME;
    let mut Data: *mut TCHAR = core::ptr::null_mut();
    let mut lpValueName: [TCHAR; (MAX_KEY_LENGTH as usize)] = core::mem::zeroed();
    let mut Path: [TCHAR; (MAX_KEY_LENGTH + 5) as _] = core::mem::zeroed();
    let mut item: LV_ITEM = core::mem::zeroed();

    if (RegOpenKeyEx(hRootKey, KeyName, 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as u32 == ERROR_SUCCESS {
        if (RegQueryInfoKey(
            hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwValues),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32 == ERROR_SUCCESS
        {
            Index = 0;
            retVal = ERROR_SUCCESS;
            while Index < dwValues {
                dwValueLength = MAX_KEY_LENGTH;
                dwDataLength = MAX_VALUE_NAME;
                Data = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (16383 as SIZE_T) * (core::mem::size_of::<TCHAR>() as SIZE_T),
                ) as *mut TCHAR;
                if Data.is_null() {
                    break;
                }
                retVal = (RegEnumValue(
                    hKey,
                    Index,
                    core::ptr::addr_of_mut!(lpValueName) as *mut TCHAR,
                    core::ptr::addr_of_mut!(dwValueLength),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwType),
                    Data as LPBYTE,
                    core::ptr::addr_of_mut!(dwDataLength),
                )) as u32;
                if retVal == ERROR_SUCCESS {
                    memset(
                        core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
                        0,
                        core::mem::size_of::<LV_ITEM>(),
                    );
                    item.mask = LVIF_TEXT;
                    item.iImage = 0;
                    item.pszText = core::ptr::addr_of_mut!(lpValueName) as *mut TCHAR;
                    item.iItem = ListView_GetItemCount!(hStartupListCtrl);
                    item.lParam = 0;
                    let _ = ListView_InsertItem!(hStartupListCtrl, core::ptr::addr_of_mut!(item));
                    ListView_SetCheckState!(hStartupListCtrl, item.iItem, TRUE);

                    if (dwType == REG_SZ) || (dwType == REG_EXPAND_SZ) {
                        GetLongPathName(Data, Data, _tcsclen(Data) as DWORD);
                        item.pszText = Data;
                        item.iSubItem = 1;
                        SendMessage(
                            hStartupListCtrl,
                            (LVM_SETITEMTEXT as u32),
                            item.iItem as WPARAM,
                            core::ptr::addr_of_mut!(item) as LPARAM,
                        );
                    }

                    match hRootKey {
                        HKEY_LOCAL_MACHINE => {
                            _tcscpy(
                                core::ptr::addr_of_mut!(Path) as *mut TCHAR,
                                ['H' as TCHAR, 'K' as TCHAR, 'L' as TCHAR, 'M' as TCHAR, '\\' as TCHAR, 0 as TCHAR, 0 as TCHAR].as_ptr(),
                            );
                        }
                        HKEY_CURRENT_USER => {
                            _tcscpy(
                                core::ptr::addr_of_mut!(Path) as *mut TCHAR,
                                ['H' as TCHAR, 'K' as TCHAR, 'C' as TCHAR, 'U' as TCHAR, '\\' as TCHAR, 0 as TCHAR, 0 as TCHAR].as_ptr(),
                            );
                        }
                        _ => {
                            _tcscpy(
                                core::ptr::addr_of_mut!(Path) as *mut TCHAR,
                                [0 as TCHAR, 0 as TCHAR].as_ptr(),
                            );
                        }
                    }

                    _tcscat(core::ptr::addr_of_mut!(Path) as *mut TCHAR, KeyName);
                    item.pszText = core::ptr::addr_of_mut!(Path) as *mut TCHAR;
                    item.iSubItem = 2;
                    SendMessage(
                        hStartupListCtrl,
                        (LVM_SETITEMTEXT as u32),
                        item.iItem as WPARAM,
                        core::ptr::addr_of_mut!(item) as LPARAM,
                    );
                }
                HeapFree(GetProcessHeap(), 0, Data as LPVOID);
                Index += 1;
            }
        }
        RegCloseKey(hKey);
    }
}
