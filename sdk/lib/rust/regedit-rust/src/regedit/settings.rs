use crate::regedit::treeview::GetItemPath;
use crate::regedit::edit::QueryStringValue;
use crate::regedit::childwnd::ResizeWnd;
use crate::regedit::treeview::SelectNode;
use crate::regedit::childwnd::get_root_key_name;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8E637B93C326: [u16; 5] = [0x0056u16, 0x0069, 0x0065, 0x0077, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_E905C42FBA6E: [u16; 8] = [0x004Cu16, 0x0061, 0x0073, 0x0074, 0x004B, 0x0065, 0x0079, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_szGeneralRegKey: [WCHAR; 58] = unsafe {
    ::core::mem::transmute::<
        [u8; 116],
        [WCHAR; 58],
    >(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0A\0p\0p\0l\0e\0t\0s\0\\\0R\0e\0g\0e\0d\0i\0t\0\0\0",
    )
};

#[repr(C)]
pub struct RegistryBinaryConfig {
    pub tPlacement: WINDOWPLACEMENT,
    pub TreeViewSize: i32,
    pub NameColumnSize: i32,
    pub TypeColumnSize: i32,
    pub DataColumnSize: i32,
    pub StatusBarVisible: BOOL,
}
#[no_mangle]
pub unsafe extern "C" fn LoadSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szBuffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if RegOpenKeyW(HKEY_CURRENT_USER, core::ptr::addr_of!(g_szGeneralRegKey) as LPCWSTR, core::ptr::addr_of_mut!(hKey)) == ERROR_SUCCESS as LONG
    {
        let mut tConfig: RegistryBinaryConfig = core::mem::zeroed();
        let mut iBufferSize: DWORD = core::mem::size_of::<RegistryBinaryConfig>() as DWORD;
        let mut bVisible: BOOL = FALSE as BOOL;

        if RegQueryValueExW(hKey, RUSTCONV_WSTR_8E637B93C326.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(tConfig) as LPBYTE, core::ptr::addr_of_mut!(iBufferSize)) == ERROR_SUCCESS as LONG
        {
            if iBufferSize == core::mem::size_of::<RegistryBinaryConfig>() as DWORD
            {
                let mut rcTemp: RECT = core::mem::zeroed();

                /* Update status bar settings */
                CheckMenuItem(GetSubMenu(hMenuFrame, (ID_VIEW_MENU as i32)), ID_VIEW_STATUSBAR as UINT, ((MF_BYCOMMAND | (if tConfig.StatusBarVisible != 0 { MF_CHECKED } else { MF_UNCHECKED })) as UINT));
                ShowWindow(hStatusBar, if tConfig.StatusBarVisible != 0 { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 });

                /* Update listview column width */
                let _ = ListView_SetColumnWidth!((*g_pChildWnd).hListWnd, 0, tConfig.NameColumnSize);
                let _ = ListView_SetColumnWidth!((*g_pChildWnd).hListWnd, 1, tConfig.TypeColumnSize);
                let _ = ListView_SetColumnWidth!((*g_pChildWnd).hListWnd, 2, tConfig.DataColumnSize);

                /* Update treeview (splitter) */
                GetClientRect(hFrameWnd, core::ptr::addr_of_mut!(rcTemp));
                (*g_pChildWnd).nSplitPos = tConfig.TreeViewSize;
                ResizeWnd(rcTemp.right, rcTemp.bottom);

                /* Apply program window settings */
                tConfig.tPlacement.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
                bVisible = SetWindowPlacement(hFrameWnd, core::ptr::addr_of_mut!(tConfig.tPlacement));
            }
        }

        /* In case we fail to restore the window, or open the key, show normal */
        if bVisible == 0
        {
            ShowWindow(hFrameWnd, (SW_SHOWNORMAL) as i32);
        }

        /* Restore key position */
        if QueryStringValue(HKEY_CURRENT_USER, core::ptr::addr_of!(g_szGeneralRegKey) as LPCWSTR, RUSTCONV_WSTR_E905C42FBA6E.as_ptr(), szBuffer.as_mut_ptr(), ARRAY_SIZE!(szBuffer) as DWORD) == ERROR_SUCCESS as LONG
        {
            SelectNode((*g_pChildWnd).hTreeWnd, szBuffer.as_mut_ptr());
        }

        RegCloseKey(hKey);
    }
    else
    {
        /* Failed to open key, show normal */
        ShowWindow(hFrameWnd, (SW_SHOWNORMAL) as i32);
    }
}

#[no_mangle]
pub unsafe extern "C" fn SaveSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();

    if (RegCreateKeyW(
        HKEY_CURRENT_USER,
        core::ptr::addr_of!(g_szGeneralRegKey) as LPCWSTR,
        core::ptr::addr_of_mut!(hKey),
    )) as u32 == ERROR_SUCCESS
    {
        let mut tConfig: RegistryBinaryConfig = core::mem::zeroed();
        let mut iBufferSize: DWORD = core::mem::size_of::<RegistryBinaryConfig>() as DWORD;
        let mut szBuffer: [WCHAR; MAX_PATH as usize] =
            [0 as WCHAR; MAX_PATH as usize];
        let mut keyPath: LPCWSTR = core::ptr::null();
        let mut rootName: LPCWSTR = core::ptr::null();
        let mut hRootKey: HKEY = core::ptr::null_mut();

        /* Save key position */
        keyPath = GetItemPath(
            (*g_pChildWnd).hTreeWnd,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hRootKey),
        );
        rootName = get_root_key_name(hRootKey);

        /* Load "My Computer" string and complete it */
        if LoadStringW(
            hInst,
            IDS_MY_COMPUTER,
            core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
            ARRAY_SIZE!(szBuffer) as i32,
        ) != 0
            && ((StringCbCatW(
                core::ptr::addr_of_mut!(szBuffer) as STRSAFE_LPWSTR,
                core::mem::size_of::<WCHAR>() * (MAX_PATH as usize),
                RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
            )) >= 0)
            && ((StringCbCatW(
                core::ptr::addr_of_mut!(szBuffer) as STRSAFE_LPWSTR,
                core::mem::size_of::<WCHAR>() * (MAX_PATH as usize),
                rootName as STRSAFE_LPCWSTR,
            )) >= 0)
            && ((StringCbCatW(
                core::ptr::addr_of_mut!(szBuffer) as STRSAFE_LPWSTR,
                core::mem::size_of::<WCHAR>() * (MAX_PATH as usize),
                RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
            )) >= 0)
        {
            let mut hr: HRESULT = S_OK;
            if !keyPath.is_null() {
                hr = StringCbCatW(
                    core::ptr::addr_of_mut!(szBuffer) as STRSAFE_LPWSTR,
                    core::mem::size_of::<WCHAR>() * (MAX_PATH as usize),
                    keyPath as STRSAFE_LPCWSTR,
                );
            }
            if hr >= 0 {
                RegSetValueExW(
                    hKey,
                    RUSTCONV_WSTR_E905C42FBA6E.as_ptr(),
                    0,
                    REG_SZ,
                    core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPBYTE,
                    (wcslen(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR)
                        as DWORD)
                        * (core::mem::size_of::<WCHAR>() as DWORD),
                );
            } else {
                DbgPrint(
                    b"err: (%s:%d): Buffer not big enough for '%S + %S'\n\0".as_ptr() as PCCH,
                    concat!(file!(), "\0").as_ptr(),
                    (line!() as ::core::ffi::c_int),
                    rootName,
                    keyPath,
                );
            }
        } else {
            DbgPrint(
                b"err: (%s:%d): Buffer not big enough for '%S'\n\0".as_ptr() as PCCH,
                concat!(file!(), "\0").as_ptr(),
                (line!() as ::core::ffi::c_int),
                rootName,
            );
        }

        /* Get statusbar settings */
        tConfig.StatusBarVisible = if (GetMenuState(
            GetSubMenu(hMenuFrame, (ID_VIEW_MENU as i32)),
            ID_VIEW_STATUSBAR,
            MF_BYCOMMAND,
        ) & MF_CHECKED)
            != 0
        {
            1
        } else {
            0
        };

        /* Get splitter position */
        tConfig.TreeViewSize = (*g_pChildWnd).nSplitPos;

        /* Get list view column width*/
        tConfig.NameColumnSize =
            ListView_GetColumnWidth!((*g_pChildWnd).hListWnd, 0);
        tConfig.TypeColumnSize =
            ListView_GetColumnWidth!((*g_pChildWnd).hListWnd, 1);
        tConfig.DataColumnSize =
            ListView_GetColumnWidth!((*g_pChildWnd).hListWnd, 2);

        /* Get program window settings */
        tConfig.tPlacement.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
        GetWindowPlacement(hFrameWnd, core::ptr::addr_of_mut!(tConfig.tPlacement));

        /* Save all the data */
        RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_8E637B93C326.as_ptr(),
            0,
            REG_BINARY,
            core::ptr::addr_of_mut!(tConfig) as LPBYTE,
            iBufferSize,
        );

        RegCloseKey(hKey);
    }
}
