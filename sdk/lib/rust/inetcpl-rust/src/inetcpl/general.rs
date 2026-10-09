use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut about_blank: [WCHAR; 12] = [
    'a' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    't' as i32 as WCHAR,
    ':' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut start_page: [WCHAR; 11] = [
    'S' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    't' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut default_page: [WCHAR; 17] = [
    'D' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut reg_ie_main: [WCHAR; 42] = [
    'S' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut disabled_general_buttons: [DWORD; 3] = [
    IDC_HOME_CURRENT as DWORD,
    IDC_HISTORY_SETTINGS as DWORD,
    0 as ::core::ffi::c_int as DWORD,
];

static mut disabled_delhist_buttons: [DWORD; 3] = [
    IDC_DELETE_FORM_DATA as DWORD,
    IDC_DELETE_PASSWORDS as DWORD,
    0 as ::core::ffi::c_int as DWORD,
];

// WINE_DEFAULT_DEBUG_CHANNEL(inetcpl): debug-channel macro, no Rust declaration emitted.
// IDC_* values come from the resource header and are not in the pre-resolved import list.

unsafe extern "C" fn delhist_on_command(mut hdlg: HWND, mut wparam: WPARAM) -> INT_PTR {
    match wparam {
        w if w == MAKEWPARAM!(IDOK, BN_CLICKED) => {
            if IsDlgButtonChecked(hdlg, (IDC_DELETE_TEMP_FILES as i32)) != 0 {
                FreeUrlCacheSpaceW(core::ptr::null_mut(), 100, 0);
            }

            if IsDlgButtonChecked(hdlg, (IDC_DELETE_COOKIES as i32)) != 0 {
                let mut pathW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

                if SHGetSpecialFolderPathW(core::ptr::null_mut(), core::ptr::addr_of_mut!(pathW) as LPWSTR, (CSIDL_COOKIES as i32), (TRUE) as BOOL) != 0 {
                    FreeUrlCacheSpaceW(core::ptr::addr_of_mut!(pathW) as LPCWSTR, 100, 0);
                }
            }

            if IsDlgButtonChecked(hdlg, (IDC_DELETE_HISTORY as i32)) != 0 {
                let mut pathW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

                if SHGetSpecialFolderPathW(core::ptr::null_mut(), core::ptr::addr_of_mut!(pathW) as LPWSTR, (CSIDL_HISTORY as i32), (TRUE) as BOOL) != 0 {
                    FreeUrlCacheSpaceW(core::ptr::addr_of_mut!(pathW) as LPCWSTR, 100, 0);
                }
            }

            EndDialog(hdlg, IDOK as INT_PTR);
            return TRUE as INT_PTR;
        }
        w if w == MAKEWPARAM!(IDCANCEL, BN_CLICKED) => {
            EndDialog(hdlg, IDCANCEL as INT_PTR);
            return TRUE as INT_PTR;
        }
        w if w == MAKEWPARAM!(IDC_DELETE_TEMP_FILES, BN_CLICKED)
            || w == MAKEWPARAM!(IDC_DELETE_COOKIES, BN_CLICKED)
            || w == MAKEWPARAM!(IDC_DELETE_HISTORY, BN_CLICKED)
            || w == MAKEWPARAM!(IDC_DELETE_FORM_DATA, BN_CLICKED)
            || w == MAKEWPARAM!(IDC_DELETE_PASSWORDS, BN_CLICKED) =>
        {
            let mut any: BOOL = (IsDlgButtonChecked(hdlg, (IDC_DELETE_TEMP_FILES as i32)) != 0
                || IsDlgButtonChecked(hdlg, (IDC_DELETE_COOKIES as i32)) != 0
                || IsDlgButtonChecked(hdlg, (IDC_DELETE_HISTORY as i32)) != 0
                || IsDlgButtonChecked(hdlg, (IDC_DELETE_FORM_DATA as i32)) != 0
                || IsDlgButtonChecked(hdlg, (IDC_DELETE_PASSWORDS as i32)) != 0) as BOOL;
            EnableWindow(GetDlgItem(hdlg, (IDOK) as i32), any);
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "system" fn delhist_dlgproc(
    hdlg: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> INT_PTR {
    match msg {
        WM_COMMAND => {
            return delhist_on_command(hdlg, wparam);
        }
        WM_INITDIALOG => {
            let mut ptr: *mut DWORD = core::ptr::addr_of_mut!(disabled_delhist_buttons) as *mut DWORD;
            while *ptr != 0 {
                EnableWindow(
                    GetDlgItem(hdlg, *ptr as ::core::ffi::c_int),
                    (FALSE) as BOOL,
                );
                ptr = ptr.offset(1);
            }
            CheckDlgButton(hdlg, (IDC_DELETE_TEMP_FILES as i32), BST_CHECKED as UINT);
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "C" fn parse_url_from_outside(url: LPCWSTR, out: LPWSTR, maxlen: DWORD) -> DWORD {
    let mut maxlen = maxlen;
    let hdll: HMODULE = LoadLibraryA(b"shdocvw.dll\0".as_ptr() as LPCSTR) as HMODULE;
    let pParseURLFromOutsideSourceRaw: *mut core::ffi::c_void =
        core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
            GetProcAddress(hdll as HINSTANCE, 170 as LPSTR as LPCSTR),
        );

    if !pParseURLFromOutsideSourceRaw.is_null() {
        let pParseURLFromOutsideSourceW: unsafe extern "system" fn(
            LPCWSTR,
            LPWSTR,
            LPDWORD,
            LPDWORD,
        ) -> DWORD = core::mem::transmute(pParseURLFromOutsideSourceRaw);
        let res: DWORD = pParseURLFromOutsideSourceW(
            url,
            out,
            core::ptr::addr_of_mut!(maxlen),
            core::ptr::null_mut(),
        );
        FreeLibrary(hdll);
        return res;
    }

    FreeLibrary(hdll);
    return 0;
}

unsafe extern "C" fn general_on_command(mut hwnd: HWND, mut wparam: WPARAM) -> INT_PTR {
    let mut buffer: [WCHAR; (INTERNET_MAX_URL_LENGTH) as _] = [0; (INTERNET_MAX_URL_LENGTH) as _];
    let mut len: DWORD = 0;
    let mut type_0: DWORD = 0;
    let mut res: LONG = 0;
    if wparam == MAKEWPARAM!(IDC_HOME_EDIT, EN_CHANGE) as WPARAM {
        /* enable apply button */
        SendMessageW(
            GetParent(hwnd),
            PSM_CHANGED as UINT,
            hwnd as WPARAM,
            0 as LPARAM,
        );
    } else if wparam == MAKEWPARAM!(IDC_HOME_BLANK, BN_CLICKED) as WPARAM {
        SetDlgItemTextW(
            hwnd,
            (IDC_HOME_EDIT as i32),
            core::ptr::addr_of!(about_blank) as LPCWSTR,
        );
    } else if wparam == MAKEWPARAM!(IDC_HOME_DEFAULT, BN_CLICKED) as WPARAM {
        len = ::core::mem::size_of::<[WCHAR; (INTERNET_MAX_URL_LENGTH) as _]>() as DWORD;
        type_0 = REG_SZ as DWORD;
        res = SHRegGetUSValueW(
            core::ptr::addr_of!(reg_ie_main) as LPCWSTR,
            core::ptr::addr_of!(default_page) as LPCWSTR,
            core::ptr::addr_of_mut!(type_0),
            core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPVOID,
            core::ptr::addr_of_mut!(len),
            (FALSE) as BOOL,
            core::ptr::addr_of!(about_blank) as *const WCHAR as LPBYTE as LPVOID,
            ::core::mem::size_of::<[WCHAR; 12]>() as DWORD,
        );
        if res == 0 && type_0 == REG_SZ as DWORD {
            SetDlgItemTextW(
                hwnd,
                (IDC_HOME_EDIT as i32),
                core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPCWSTR,
            );
        }
    } else if wparam == MAKEWPARAM!(IDC_HISTORY_DELETE, BN_CLICKED) as WPARAM {
        DialogBoxW!(
            hcpl,
            MAKEINTRESOURCEW((IDD_DELETE_HISTORY) as _),
            hwnd,
            Some(delhist_dlgproc),
        );
    } else {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/inetcpl/general.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"general_on_command\0" as *const u8 as *const ::core::ffi::c_char,
                207 as ::core::ffi::c_int,
                b"not implemented for command: %d/%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (wparam >> 16 as ::core::ffi::c_int & 0xffff as WPARAM) as WORD
                    as ::core::ffi::c_int,
                (wparam & 0xffff as WPARAM) as WORD as ::core::ffi::c_int,
            );
        }
        return FALSE as INT_PTR;
    }
    return TRUE as INT_PTR;
}

unsafe extern "C" fn general_on_initdialog(mut hwnd: HWND) {
    let mut buffer: [WCHAR; (INTERNET_MAX_URL_LENGTH) as _] = [0; (INTERNET_MAX_URL_LENGTH) as _];
    let mut len: DWORD;
    let mut type_0: DWORD;
    let mut res: LONG;
    let mut ptr: *mut DWORD = &raw mut disabled_general_buttons as *mut DWORD;

    /* disable unimplemented buttons */
    while *ptr != 0 {
        EnableWindow(GetDlgItem(hwnd, *ptr as ::core::ffi::c_int), (FALSE) as BOOL);
        ptr = ptr.offset(1);
    }

    /* read current homepage from the registry. Try HCU first, then HKLM */
    *(&raw mut buffer as *mut WCHAR) = 0 as WCHAR;
    len = ::core::mem::size_of::<[WCHAR; (INTERNET_MAX_URL_LENGTH) as _]>() as DWORD;
    type_0 = REG_SZ as DWORD;
    res = SHRegGetUSValueW(
        &raw const reg_ie_main as LPCWSTR,
        &raw const start_page as LPCWSTR,
        &raw mut type_0,
        &raw mut buffer as *mut WCHAR as LPVOID,
        &raw mut len,
        (FALSE) as BOOL,
        &raw const about_blank as *const WCHAR as LPBYTE as LPVOID,
        ::core::mem::size_of::<[WCHAR; 12]>() as DWORD,
    );

    if res == 0 && (type_0 == REG_SZ as DWORD) {
        SetDlgItemTextW(hwnd, (IDC_HOME_EDIT as i32), &raw mut buffer as *mut WCHAR as LPCWSTR);
    }
}

unsafe extern "C" fn general_on_notify(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> INT_PTR {
    let psn: *mut PSHNOTIFY;
    let mut buffer: [WCHAR; (INTERNET_MAX_URL_LENGTH) as _] = [0; (INTERNET_MAX_URL_LENGTH) as _];
    let mut parsed: [WCHAR; (INTERNET_MAX_URL_LENGTH) as _] = [0; (INTERNET_MAX_URL_LENGTH) as _];
    let mut res: LONG;

    psn = lparam as *mut PSHNOTIFY;

    if (*psn).hdr.code == (PSN_APPLY) as u32 {
        buffer[0] = 0 as WCHAR;
        GetDlgItemTextW(
            hwnd,
            IDC_HOME_EDIT as ::core::ffi::c_int,
            buffer.as_mut_ptr(),
            ARRAY_SIZE!(buffer) as ::core::ffi::c_int,
        );

        res = parse_url_from_outside(
            buffer.as_ptr(),
            parsed.as_mut_ptr(),
            ARRAY_SIZE!(parsed) as DWORD,
        ) as LONG;

        if res != 0 {
            let mut hkey: HKEY = core::ptr::null_mut();

            /* update the dialog, when needed */
            if lstrcmpW(buffer.as_ptr(), parsed.as_ptr()) != 0 {
                SetDlgItemTextW(
                    hwnd,
                    IDC_HOME_EDIT as ::core::ffi::c_int,
                    parsed.as_ptr(),
                );
            }

            /* update the registry */
            res = RegOpenKeyW(
                HKEY_CURRENT_USER as HKEY,
                &raw const reg_ie_main as LPCWSTR,
                &raw mut hkey,
            ) as LONG;
            if res == 0 {
                res = RegSetValueExW(
                    hkey,
                    &raw const start_page as LPCWSTR,
                    0 as DWORD,
                    REG_SZ as DWORD,
                    parsed.as_ptr() as *const BYTE,
                    ((lstrlenW(parsed.as_ptr())
                        + 1 as ::core::ffi::c_int)
                        as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD,
                ) as LONG;
                RegCloseKey(hkey);
                return (res == 0) as INT_PTR;
            }
        }
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn general_dlgproc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> INT_PTR {
    match msg {
        WM_INITDIALOG => {
            general_on_initdialog(hwnd);
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            return general_on_command(hwnd, wparam);
        }
        WM_NOTIFY => {
            return general_on_notify(hwnd, wparam, lparam);
        }
        _ => {
            if (msg == WM_SETCURSOR) || (msg == WM_NCHITTEST) || (msg == WM_MOUSEMOVE) {
                return (FALSE) as INT_PTR;
            }
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/cpl/inetcpl/general.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"general_dlgproc\0" as *const u8 as *const ::core::ffi::c_char,
                    318 as ::core::ffi::c_int,
                    b"(%p, 0x%08x/%d, 0x%lx, 0x%lx)\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    hwnd,
                    msg,
                    msg,
                    wparam,
                    lparam,
                );
            }
        }
    }
    return (FALSE) as INT_PTR;
}
