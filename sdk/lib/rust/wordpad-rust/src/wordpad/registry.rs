use crate::wordpad::print::registry_read_pagemargins;
use crate::wordpad::print::registry_read_previewpages;
use crate::wordpad::print::registry_set_pagemargins;
use crate::wordpad::print::registry_set_previewpages;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut key_recentfiles: [WCHAR; 17] = [
    'R' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut key_options: [WCHAR; 8] = [
    'O' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut key_settings: [WCHAR; 9] = [
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut key_rtf: [WCHAR; 4] = [
    'R' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut key_text: [WCHAR; 5] = [
    'T' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut var_file: [WCHAR; 7] = [
    'F' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut var_framerect: [WCHAR; 10] = [
    'F' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut var_barstate0: [WCHAR; 10] = [
    'B' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '0' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut var_wrap: [WCHAR; 5] = [
    'W' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut var_maximized: [WCHAR; 10] = [
    'M' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'z' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

unsafe extern "C" fn registry_get_handle(hKey: *mut HKEY, action: LPDWORD, subKey: LPCWSTR) -> LRESULT {
    let mut ret: LONG;
    static wszProgramKey: [WCHAR; 58] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0041, 0x0070, 0x0070, 0x006C, 0x0065, 0x0074, 0x0073, 0x005C,
    0x0057, 0x006F, 0x0072, 0x0064, 0x0070, 0x0061, 0x0064, 0x0000,
];
    let mut key: LPWSTR = core::ptr::addr_of!(wszProgramKey) as *const WCHAR as LPWSTR;

    if !subKey.is_null() {
        let mut backslash: [WCHAR; 2] = [0x005Cu16, 0x0000];

        key = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((lstrlenW(core::ptr::addr_of!(wszProgramKey) as LPCWSTR)
                + lstrlenW(subKey)
                + lstrlenW(core::ptr::addr_of_mut!(backslash) as *mut WCHAR as LPCWSTR)
                + 1) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;

        if key.is_null() {
            return 1 as LRESULT;
        }

        lstrcpyW(key, core::ptr::addr_of!(wszProgramKey) as LPCWSTR);
        lstrcatW(key, core::ptr::addr_of_mut!(backslash) as *mut WCHAR as LPCWSTR);
        lstrcatW(key, subKey);
    }

    if !action.is_null() {
        ret = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            key as LPCWSTR,
            0,
            core::ptr::null_mut::<WCHAR>(),
            REG_OPTION_NON_VOLATILE,
            KEY_READ | KEY_WRITE,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            hKey as PHKEY,
            action as PDWORD,
        ) as LONG;
    } else {
        ret = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            key as LPCWSTR,
            0,
            KEY_READ | KEY_WRITE,
            hKey as PHKEY,
        ) as LONG;
    }

    if !subKey.is_null() {
        HeapFree(GetProcessHeap(), 0, key as PVOID);
    }

    return ret as LRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn registry_set_options(mut hMainWnd: HWND) {
    let mut hKey: HKEY = 0 as HKEY;
    let mut action: DWORD = 0;
    if registry_get_handle(
        &raw mut hKey,
        &raw mut action,
        &raw const key_options as LPCWSTR,
    ) == ERROR_SUCCESS as LRESULT
    {
        let mut wp: WINDOWPLACEMENT = core::mem::zeroed();
        let mut isMaximized: DWORD = 0;
        wp.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
        GetWindowPlacement(hMainWnd, &raw mut wp);
        isMaximized = (wp.showCmd == SW_SHOWMAXIMIZED as UINT) as ::core::ffi::c_int as DWORD;

        RegSetValueExW(
            hKey,
            &raw const var_framerect as LPCWSTR,
            0 as DWORD,
            REG_BINARY as DWORD,
            &raw mut wp.rcNormalPosition as LPBYTE as *const BYTE,
            core::mem::size_of::<RECT>() as DWORD,
        );
        RegSetValueExW(
            hKey,
            &raw const var_maximized as LPCWSTR,
            0 as DWORD,
            REG_DWORD as DWORD,
            &raw mut isMaximized as LPBYTE as *const BYTE,
            ::core::mem::size_of::<DWORD>() as DWORD,
        );

        registry_set_pagemargins(hKey);
        RegCloseKey(hKey);
    }

    if registry_get_handle(
        &raw mut hKey,
        &raw mut action,
        &raw const key_settings as LPCWSTR,
    ) == ERROR_SUCCESS as LRESULT
    {
        registry_set_previewpages(hKey);
        RegCloseKey(hKey);
    }
}
#[no_mangle]
pub unsafe extern "C" fn registry_read_winrect(rc: *mut RECT) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut size: DWORD = ::core::mem::size_of::<RECT>() as DWORD;

    if registry_get_handle(
        &raw mut hKey,
        core::ptr::null_mut(),
        &raw const key_options as LPCWSTR,
    ) != ERROR_SUCCESS as LRESULT
        || RegQueryValueExW(
            hKey,
            &raw const var_framerect as LPCWSTR,
            core::ptr::null_mut(),
            ::core::ptr::null_mut::<DWORD>(),
            rc as LPBYTE,
            &raw mut size,
        ) != ERROR_SUCCESS as LSTATUS
        || size as usize != ::core::mem::size_of::<RECT>() as usize
    {
        SetRect(
            rc as LPRECT,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            600 as ::core::ffi::c_int,
            300 as ::core::ffi::c_int,
        );
    }

    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn registry_read_maximized(bMaximized: *mut DWORD) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    if registry_get_handle(core::ptr::addr_of_mut!(hKey), core::ptr::null_mut(), core::ptr::addr_of!(key_options) as LPCWSTR) != ERROR_SUCCESS as LRESULT
        || RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(var_maximized) as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            bMaximized as LPBYTE,
            core::ptr::addr_of_mut!(size),
        ) != (ERROR_SUCCESS as i32)
        || size != core::mem::size_of::<DWORD>() as DWORD
    {
        *bMaximized = FALSE;
    }

    RegCloseKey(hKey);
}

unsafe extern "C" fn truncate_path(mut file: LPWSTR, mut out: LPWSTR, mut pos1: LPWSTR, mut pos2: LPWSTR) {
    static mut dots: [WCHAR; 4] = [0x002Eu16, 0x002E, 0x002E, 0x0000];
    pos1 = pos1.offset(1);
    *pos1 = 0;
    lstrcatW(out, file as LPCWSTR);
    lstrcatW(out, core::ptr::addr_of!(dots) as LPCWSTR);
    lstrcatW(out, pos2 as LPCWSTR);
}

unsafe extern "C" fn format_filelist_filename(file: LPWSTR, out: LPWSTR) {
    let mut pos_basename: LPWSTR = core::ptr::null_mut();
    let mut truncpos1: LPWSTR;
    let mut truncpos2: LPWSTR;
    let mut myDocs: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    SHGetFolderPathW(
        core::ptr::null_mut(),
        (CSIDL_PERSONAL as i32),
        core::ptr::null_mut(),
        SHGFP_TYPE_CURRENT as DWORD,
        core::ptr::addr_of_mut!(myDocs) as LPWSTR,
    );
    pos_basename = file_basename(file);
    truncpos1 = core::ptr::null_mut();
    truncpos2 = core::ptr::null_mut();

    // The C writes pos_basename[-1] unconditionally; for a recent-files entry with no
    // directory (or only a leading separator) file_basename returns `file` itself and that
    // store lands before the buffer. Such a name has nothing to truncate.
    if pos_basename == file {
        lstrcatW(out, file as LPCWSTR);
        return;
    }

    *pos_basename.offset(-(1 as isize)) = 0 as WCHAR;
    if lstrcmpiW(file as LPCWSTR, core::ptr::addr_of_mut!(myDocs) as LPCWSTR) == 0
        || lstrlenW(pos_basename as LPCWSTR) > (FILELIST_ENTRY_LENGTH as i32)
    {
        truncpos1 = pos_basename;
        *pos_basename.offset(-(1 as isize)) = '\\' as i32 as WCHAR;
    } else {
        let mut pos: LPWSTR;
        let mut morespace: BOOL = FALSE as BOOL;

        *pos_basename.offset(-(1 as isize)) = '\\' as i32 as WCHAR;

        pos = file;
        while pos < pos_basename {
            if *pos as i32 == '\\' as i32 || *pos as i32 == '/' as i32 {
                if !truncpos1.is_null() {
                    if (pos.offset_from(file) as i32
                        + lstrlenW(pos_basename as LPCWSTR))
                        > (FILELIST_ENTRY_LENGTH as i32)
                    {
                        break;
                    }

                    truncpos1 = pos;
                    morespace = TRUE as BOOL;
                    break;
                }

                if (pos.offset_from(file) as i32
                    + lstrlenW(pos_basename as LPCWSTR))
                    > (FILELIST_ENTRY_LENGTH as i32)
                {
                    break;
                }

                truncpos1 = pos;
            }
            pos = pos.offset(1);
        }

        if morespace != 0 {
            pos = pos_basename;
            while pos >= truncpos1 {
                if *pos as i32 == '\\' as i32 || *pos as i32 == '/' as i32 {
                    if (truncpos1.offset_from(file) as i32
                        + lstrlenW(pos_basename as LPCWSTR)
                        + pos_basename.offset_from(pos) as i32)
                        > (FILELIST_ENTRY_LENGTH as i32)
                    {
                        break;
                    }

                    truncpos2 = pos;
                }
                pos = pos.offset(-1);
            }
        }
    }

    if truncpos1 == pos_basename {
        lstrcatW(out, pos_basename as LPCWSTR);
    } else if truncpos1 == truncpos2 || truncpos2.is_null() {
        lstrcatW(out, file as LPCWSTR);
    } else {
        truncate_path(file, out, truncpos1, truncpos2);
    }
}
#[no_mangle]
pub unsafe extern "C" fn registry_read_filelist(hMainWnd: HWND) {
    let mut hFileKey: HKEY = core::ptr::null_mut();

    if registry_get_handle(&raw mut hFileKey, core::ptr::null_mut(), &raw const key_recentfiles as LPCWSTR) == ERROR_SUCCESS as LRESULT
    {
        let mut itemText: [WCHAR; (MAX_PATH + 3) as _] = [0; (MAX_PATH + 3) as _];
        let mut buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        /* The menu item name is not the same as the file name, so we need to store
        the file name here */
        static mut file1: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        static mut file2: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        static mut file3: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        static mut file4: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut numFormat: [WCHAR; 5] = [0x0026u16, 0x0025, 0x0064, 0x0020, 0x0000];
        let mut pFile: [LPWSTR; 4] = [
            &raw mut file1 as LPWSTR,
            &raw mut file2 as LPWSTR,
            &raw mut file3 as LPWSTR,
            &raw mut file4 as LPWSTR,
        ];
        let mut pathSize: DWORD = (MAX_PATH as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        let mut i: i32;
        let mut key: [WCHAR; 6] = [0; 6];
        let mut mi: MENUITEMINFOW = core::mem::zeroed();
        let mut hMenu: HMENU = GetMenu(hMainWnd);

        mi.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
        mi.fMask = (MIIM_ID | MIIM_DATA | MIIM_STRING | MIIM_FTYPE) as UINT;
        mi.fType = MFT_STRING as UINT;
        mi.dwTypeData = &raw mut itemText as *mut WCHAR as LPWSTR;
        mi.wID = ID_FILE_RECENT1 as UINT;

        RemoveMenu(hMenu, ID_FILE_RECENT_SEPARATOR, MF_BYCOMMAND);
        i = 0;
        while i < (FILELIST_ENTRIES as i32) {
            wsprintfW(&raw mut key as LPWSTR, &raw const var_file as LPCWSTR, i + 1);
            RemoveMenu(hMenu, ID_FILE_RECENT1 + i as UINT, MF_BYCOMMAND);
            if RegQueryValueExW(
                hFileKey,
                &raw mut key as *mut WCHAR as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                pFile[i as usize] as LPBYTE,
                &raw mut pathSize,
            ) != ERROR_SUCCESS as LSTATUS
            {
                break;
            }

            mi.dwItemData = pFile[i as usize] as ULONG_PTR;
            wsprintfW(&raw mut itemText as LPWSTR, &raw mut numFormat as *mut WCHAR as LPCWSTR, i + 1);

            lstrcpyW(&raw mut buffer as LPWSTR, pFile[i as usize] as LPCWSTR);

            format_filelist_filename(&raw mut buffer as LPWSTR, &raw mut itemText as LPWSTR);

            InsertMenuItemW(hMenu, ID_FILE_EXIT as UINT, (FALSE) as BOOL, &raw mut mi as LPCMENUITEMINFOW);
            mi.wID = mi.wID.wrapping_add(1);
            pathSize = (MAX_PATH as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
            i += 1;
        }
        mi.fType = MFT_SEPARATOR as UINT;
        mi.fMask = (MIIM_FTYPE | MIIM_ID) as UINT;
        InsertMenuItemW(hMenu, ID_FILE_EXIT as UINT, (FALSE) as BOOL, &raw mut mi as LPCMENUITEMINFOW);

        RegCloseKey(hFileKey);
    }
}
#[no_mangle]
pub unsafe extern "C" fn registry_set_filelist(mut newFile: LPCWSTR, mut hMainWnd: HWND) {
    let mut hKey: HKEY = ::core::ptr::null_mut();
    let mut action: DWORD = 0;
    if registry_get_handle(
        &raw mut hKey,
        &raw mut action,
        &raw const key_recentfiles as LPCWSTR,
    ) == ERROR_SUCCESS as LRESULT
    {
        let mut pFiles: [LPCWSTR; (FILELIST_ENTRIES as _)] =
            [::core::ptr::null::<WCHAR>(); (FILELIST_ENTRIES as _)];
        let mut i: ::core::ffi::c_int;
        let mut hMenu: HMENU = GetMenu(hMainWnd);
        let mut mi: MENUITEMINFOW = ::core::mem::zeroed::<MENUITEMINFOW>();
        let mut buffer: [WCHAR; 6] = [0; 6];

        mi.cbSize = ::core::mem::size_of::<MENUITEMINFOW>() as UINT;
        mi.fMask = MIIM_DATA as UINT;

        i = 0;
        while i < (FILELIST_ENTRIES as i32) {
            pFiles[i as usize] = ::core::ptr::null::<WCHAR>();
            i += 1;
        }

        i = 0;
        // The C stops only when GetMenuItemInfoW fails, and with four recent files the
        // separator's ID is ID_FILE_RECENT1+4, so it writes pFiles[4], one past the array
        // (benign on the stack there). A Rust index would panic: stop at FILELIST_ENTRIES.
        while i < (FILELIST_ENTRIES as i32)
            && GetMenuItemInfoW(
                hMenu,
                (ID_FILE_RECENT1 + (i) as u32) as UINT,
                FALSE as BOOL,
                &raw mut mi,
            ) != 0
        {
            pFiles[i as usize] = mi.dwItemData as LPWSTR as LPCWSTR;
            i += 1;
        }

        if lstrcmpiW(newFile, pFiles[0 as usize]) != 0 {
            i = 0;
            while i < (FILELIST_ENTRIES as i32) && !pFiles[i as usize].is_null() {
                if lstrcmpiW(pFiles[i as usize], newFile) == 0 {
                    let mut j: ::core::ffi::c_int;
                    j = 0;
                    while j < i {
                        pFiles[(i - j) as usize] =
                            pFiles[(i - j - 1) as usize];
                        j += 1;
                    }
                    pFiles[0 as usize] = ::core::ptr::null::<WCHAR>();
                    break;
                }
                i += 1;
            }

            if pFiles[0 as usize].is_null() {
                pFiles[0 as usize] = newFile;
            } else {
                i = 0;
                while ((i) as u32) < FILELIST_ENTRIES - 1 {
                    pFiles[(FILELIST_ENTRIES - 1 - (i) as u32) as usize] =
                        pFiles[(FILELIST_ENTRIES - 2 - (i) as u32) as usize];
                    i += 1;
                }
                pFiles[0 as usize] = newFile;
            }

            i = 0;
            while i < (FILELIST_ENTRIES as i32) && !pFiles[i as usize].is_null() {
                wsprintfW(
                    &raw mut buffer as LPWSTR,
                    &raw const var_file as LPCWSTR,
                    i + 1,
                );
                RegSetValueExW(
                    hKey,
                    &raw mut buffer as LPWSTR as LPCWSTR,
                    0 as DWORD,
                    REG_SZ as DWORD,
                    pFiles[i as usize] as *const BYTE,
                    ((lstrlenW(pFiles[i as usize]) + 1)
                        as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD,
                );
                i += 1;
            }
        }
        RegCloseKey(hKey);
    }
    registry_read_filelist(hMainWnd);
}
#[no_mangle]
pub unsafe extern "C" fn reg_formatindex(format: WPARAM) -> ::core::ffi::c_int {
    return if (format & SF_TEXT as WPARAM) != 0 {
        1 as i32
    } else {
        0 as i32
    };
}
#[no_mangle]
pub unsafe extern "C" fn registry_read_options() {
    let mut hKey: HKEY = core::ptr::null_mut();

    if registry_get_handle(
        &raw mut hKey,
        0 as usize as LPDWORD,
        core::ptr::addr_of!(key_options) as LPCWSTR,
    ) != ERROR_SUCCESS as LRESULT
    {
        registry_read_pagemargins(core::ptr::null_mut());
    } else {
        registry_read_pagemargins(hKey);
        RegCloseKey(hKey);
    }

    if registry_get_handle(
        &raw mut hKey,
        0 as usize as LPDWORD,
        core::ptr::addr_of!(key_settings) as LPCWSTR,
    ) != ERROR_SUCCESS as LRESULT
    {
        registry_read_previewpages(core::ptr::null_mut());
    } else {
        registry_read_previewpages(hKey);
        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn registry_read_formatopts(
    mut index: ::core::ffi::c_int,
    mut key: LPCWSTR,
    mut barState: *mut DWORD,
    mut wordWrap: *mut DWORD,
) {
    let mut hKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut action: DWORD = 0 as DWORD;
    let mut fetched: BOOL = FALSE as BOOL;
    *barState.offset(index as isize) = 0 as DWORD;
    *wordWrap.offset(index as isize) = 0 as DWORD;

    if registry_get_handle(&raw mut hKey, &raw mut action, key) != ERROR_SUCCESS as LRESULT {
        return;
    }

    if action == REG_OPENED_EXISTING_KEY as DWORD {
        let mut size: DWORD = ::core::mem::size_of::<DWORD>() as DWORD;

        if RegQueryValueExW(
            hKey,
            &raw const var_barstate0 as LPCWSTR,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            barState.offset(index as isize) as *mut DWORD as LPBYTE,
            &raw mut size,
        ) == ERROR_SUCCESS as LSTATUS
        {
            fetched = TRUE as BOOL;
        }
    }

    if fetched == 0 {
        *barState.offset(index as isize) = ((1 as DWORD) << BANDID_TOOLBAR)
            | ((1 as DWORD) << BANDID_FORMATBAR)
            | ((1 as DWORD) << BANDID_RULER)
            | ((1 as DWORD) << BANDID_STATUSBAR);
    }

    fetched = FALSE as BOOL;
    if action == REG_OPENED_EXISTING_KEY as DWORD {
        let mut size_0: DWORD = ::core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(
            hKey,
            &raw const var_wrap as LPCWSTR,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            wordWrap.offset(index as isize) as *mut DWORD as LPBYTE,
            &raw mut size_0,
        ) == ERROR_SUCCESS as LSTATUS
        {
            fetched = TRUE as BOOL;
        }
    }

    if fetched == 0 {
        if index == reg_formatindex(SF_RTF as WPARAM) {
            *wordWrap.offset(index as isize) = ID_WORDWRAP_WINDOW as DWORD;
        } else if index == reg_formatindex(SF_TEXT as WPARAM) {
            *wordWrap.offset(index as isize) = ID_WORDWRAP_NONE as DWORD;
        }
    }

    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn registry_read_formatopts_all(barState: *mut DWORD, wordWrap: *mut DWORD) {
    registry_read_formatopts(reg_formatindex((SF_RTF) as WPARAM), core::ptr::addr_of!(key_rtf) as LPCWSTR, barState, wordWrap);
    registry_read_formatopts(reg_formatindex((SF_TEXT) as WPARAM), core::ptr::addr_of!(key_text) as LPCWSTR, barState, wordWrap);
}

unsafe extern "C" fn registry_set_formatopts(
    mut index: ::core::ffi::c_int,
    mut key: LPCWSTR,
    mut barState: *mut DWORD,
    mut wordWrap: *mut DWORD,
) {
    let mut hKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut action: DWORD = 0 as DWORD;
    if registry_get_handle(&raw mut hKey, &raw mut action, key) == ERROR_SUCCESS as LRESULT {
        RegSetValueExW(
            hKey,
            &raw const var_barstate0 as LPCWSTR,
            0 as DWORD,
            REG_DWORD as DWORD,
            barState.offset(index as isize) as *mut DWORD as LPBYTE as *const BYTE,
            ::core::mem::size_of::<DWORD>() as DWORD,
        );
        RegSetValueExW(
            hKey,
            &raw const var_wrap as LPCWSTR,
            0 as DWORD,
            REG_DWORD as DWORD,
            wordWrap.offset(index as isize) as *mut DWORD as LPBYTE as *const BYTE,
            ::core::mem::size_of::<DWORD>() as DWORD,
        );
        RegCloseKey(hKey);
    }
}
#[no_mangle]
pub unsafe extern "C" fn registry_set_formatopts_all(barState: *mut DWORD, wordWrap: *mut DWORD) {
    registry_set_formatopts(
        reg_formatindex(SF_RTF as WPARAM),
        core::ptr::addr_of!(key_rtf) as LPCWSTR,
        barState,
        wordWrap,
    );
    registry_set_formatopts(
        reg_formatindex(SF_TEXT as WPARAM),
        core::ptr::addr_of!(key_text) as LPCWSTR,
        barState,
        wordWrap,
    );
}
