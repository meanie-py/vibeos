use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const INTERNET_MAX_URL_LENGTH: usize = (INTERNET_MAX_SCHEME_LENGTH as usize)
    .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
    .wrapping_add(INTERNET_MAX_PATH_LENGTH as usize);

static mut auto_config_url: [WCHAR; 14] = [
    'A' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut internet_settings: [WCHAR; 60] = [
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
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
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

static mut proxy_enable: [WCHAR; 12] = [
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut proxy_server: [WCHAR; 12] = [
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut connections: [WCHAR; 12] = [
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut default_connection_settings: [WCHAR; 26] = [
    'D' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
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

static mut initdialog_done: BOOL = 0;

// WINE_DEFAULT_DEBUG_CHANNEL(inetcpl): debug-channel declaration macro, no Rust item emitted here.

pub const CONNECTION_SETTINGS_VERSION: u32 = 0x46;

pub const CONNECTION_SETTINGS_MANUAL_PROXY: u32 = 0x2;

pub const CONNECTION_SETTINGS_PAC_SCRIPT: u32 = 0x4;

pub const CONNECTION_SETTINGS_WPAD: u32 = 0x8;

#[repr(C)]
pub struct connection_settings {
    pub version: DWORD,
    pub id: DWORD,
    pub flags: DWORD,
    pub data: [BYTE; 1],
    // DWORD proxy_server_len; UTF8 proxy_server[proxy_server_len];
    // DWORD bypass_list_len; UTF8 bypass_list[bypass_list_len];
    // DWORD configuration_script_len; UTF8 configuration_script[configuration_script_len];
    // DWORD unk[8]; set to 0
}

unsafe extern "C" fn create_connection_settings(manual_proxy: BOOL, proxy_server_arg: *const WCHAR, use_wpad: BOOL, use_pac_script: BOOL, pac_url: *const WCHAR, ret: *mut *mut connection_settings) -> DWORD {
    let mut size: DWORD = core::mem::offset_of!(connection_settings, data) as DWORD;
    let mut pos: DWORD;
    let mut proxy_server_len: DWORD;
    let mut pac_url_len: DWORD;

    size += core::mem::size_of::<DWORD>() as DWORD;
    if !proxy_server_arg.is_null() {
        proxy_server_len = WideCharToMultiByte(CP_UTF8, 0, proxy_server_arg, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as DWORD;
        if proxy_server_len == 0 {
            return 0;
        }
        proxy_server_len -= 1;
    } else {
        proxy_server_len = 0;
    }
    size += proxy_server_len;
    if !pac_url.is_null() {
        pac_url_len = WideCharToMultiByte(CP_UTF8, 0, pac_url, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as DWORD;
        if pac_url_len == 0 {
            return 0;
        }
        pac_url_len -= 1;
    } else {
        pac_url_len = 0;
    }
    // Diverges from the C, which never adds pac_url_len to the size and then writes that
    // many bytes into the block: any PAC URL over the 32 spare bytes overran the heap.
    size += pac_url_len;
    size += (core::mem::size_of::<DWORD>() as DWORD) * 10;

    *ret = heap_alloc_zero((size as SIZE_T) as _) as *mut connection_settings;
    if (*ret).is_null() {
        return 0;
    }

    (**ret).version = CONNECTION_SETTINGS_VERSION;
    (**ret).flags = 1;
    if manual_proxy != 0 {
        (**ret).flags |= CONNECTION_SETTINGS_MANUAL_PROXY;
    }
    if use_pac_script != 0 {
        (**ret).flags |= CONNECTION_SETTINGS_PAC_SCRIPT;
    }
    if use_wpad != 0 {
        (**ret).flags |= CONNECTION_SETTINGS_WPAD;
    }
    *(core::ptr::addr_of_mut!((**ret).data) as *mut BYTE as *mut DWORD).offset(0 as isize) = proxy_server_len;
    pos = core::mem::size_of::<DWORD>() as DWORD;
    if proxy_server_len != 0 {
        WideCharToMultiByte(CP_UTF8, 0, proxy_server_arg, -1, (core::ptr::addr_of_mut!((**ret).data) as *mut BYTE as *mut c_char).offset(pos as isize), proxy_server_len as c_int, core::ptr::null(), core::ptr::null_mut());
        pos += proxy_server_len;
    }
    pos += core::mem::size_of::<DWORD>() as DWORD; /* skip proxy bypass list */
    *((core::ptr::addr_of_mut!((**ret).data) as *mut BYTE).offset(pos as isize) as *mut DWORD).offset(0 as isize) = pac_url_len;
    pos += core::mem::size_of::<DWORD>() as DWORD;
    if pac_url_len != 0 {
        WideCharToMultiByte(CP_UTF8, 0, pac_url, -1, (core::ptr::addr_of_mut!((**ret).data) as *mut BYTE as *mut c_char).offset(pos as isize), pac_url_len as c_int, core::ptr::null(), core::ptr::null_mut());
        pos += pac_url_len;
    }
    return size;
}

unsafe extern "C" fn connections_on_initdialog(hwnd: HWND) {
    let mut type_0: DWORD = 0;
    let mut size: DWORD = 0;
    let mut enabled: DWORD = 0;
    let mut address: [WCHAR; INTERNET_MAX_URL_LENGTH] = [0; INTERNET_MAX_URL_LENGTH];
    let mut port: *mut WCHAR = core::ptr::null_mut();
    let mut pac_url: [WCHAR; INTERNET_MAX_URL_LENGTH] = [0; INTERNET_MAX_URL_LENGTH];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut con: HKEY = core::ptr::null_mut();
    let mut res: LONG = 0;

    SendMessageW(
        GetDlgItem(hwnd, IDC_EDIT_PAC_SCRIPT as ::core::ffi::c_int),
        EM_LIMITTEXT as UINT,
        INTERNET_MAX_URL_LENGTH as WPARAM,
        0 as LPARAM,
    );
    SendMessageW(
        GetDlgItem(hwnd, IDC_EDIT_PROXY_SERVER as ::core::ffi::c_int),
        EM_LIMITTEXT as UINT,
        (INTERNET_MAX_URL_LENGTH as WPARAM).wrapping_sub(10 as WPARAM),
        0 as LPARAM,
    );
    SendMessageW(
        GetDlgItem(hwnd, IDC_EDIT_PROXY_PORT as ::core::ffi::c_int),
        EM_LIMITTEXT as UINT,
        8 as WPARAM,
        0 as LPARAM,
    );

    res = RegOpenKeyW(
        HKEY_CURRENT_USER as HKEY,
        &raw const internet_settings as LPCWSTR,
        &raw mut hkey,
    ) as LONG;
    if res != 0 {
        return;
    }

    size = core::mem::size_of::<DWORD>() as DWORD;
    res = RegQueryValueExW(
        hkey,
        &raw const proxy_enable as LPCWSTR,
        core::ptr::null_mut(),
        &raw mut type_0,
        &raw mut enabled as LPBYTE,
        &raw mut size,
    ) as LONG;
    if res != 0 || type_0 != REG_DWORD as DWORD {
        enabled = 0 as DWORD;
    }

    size = core::mem::size_of::<[WCHAR; INTERNET_MAX_URL_LENGTH]>() as DWORD;
    res = RegQueryValueExW(
        hkey,
        &raw const proxy_server as LPCWSTR,
        core::ptr::null_mut(),
        &raw mut type_0,
        &raw mut address as *mut WCHAR as LPBYTE,
        &raw mut size,
    ) as LONG;
    if res != 0 || type_0 != REG_SZ as DWORD {
        address[0 as usize] = 0 as WCHAR;
    }

    size = core::mem::size_of::<[WCHAR; INTERNET_MAX_URL_LENGTH]>() as DWORD;
    res = RegQueryValueExW(
        hkey,
        &raw const auto_config_url as LPCWSTR,
        core::ptr::null_mut(),
        &raw mut type_0,
        &raw mut pac_url as *mut WCHAR as LPBYTE,
        &raw mut size,
    ) as LONG;
    if res != 0 || type_0 != REG_SZ as DWORD {
        pac_url[0 as usize] = 0 as WCHAR;
    }

    res = RegOpenKeyW(hkey, &raw const connections as LPCWSTR, &raw mut con) as LONG;
    RegCloseKey(hkey);
    if res == 0 {
        let mut settings: *mut connection_settings = core::ptr::null_mut::<connection_settings>();
        size = 0 as DWORD;

        loop {
            res = RegQueryValueExW(
                con,
                &raw const default_connection_settings as LPCWSTR,
                core::ptr::null_mut(),
                &raw mut type_0,
                settings as LPBYTE,
                &raw mut size,
            ) as LONG;
            if !(res == ERROR_MORE_DATA as LONG || settings.is_null()) {
                break;
            }
            let mut new_settings: *mut connection_settings = heap_realloc(
                settings as *mut ::core::ffi::c_void,
                (size as SIZE_T) as _,
            ) as *mut connection_settings;
            if new_settings.is_null() {
                RegCloseKey(con);
                heap_free(settings as *mut ::core::ffi::c_void);
                return;
            }
            settings = new_settings;
        }
        RegCloseKey(con);

        if res == 0 && type_0 == REG_BINARY as DWORD {
            if (*settings).version != CONNECTION_SETTINGS_VERSION as DWORD {
                FIXME!(
                    b"unexpected structure version (%x)\n\0".as_ptr() as *const i8,
                    (*settings).version,
                );
            } else if (*settings).flags & CONNECTION_SETTINGS_WPAD as DWORD != 0 {
                CheckDlgButton(
                    hwnd,
                    IDC_USE_WPAD as ::core::ffi::c_int,
                    BST_CHECKED as UINT,
                );
            }
        }
        heap_free(settings as *mut ::core::ffi::c_void);
    }

    if enabled != 0 {
        CheckDlgButton(
            hwnd,
            IDC_USE_PROXY_SERVER as ::core::ffi::c_int,
            BST_CHECKED as UINT,
        );
        EnableWindow(
            GetDlgItem(hwnd, IDC_EDIT_PROXY_SERVER as ::core::ffi::c_int),
            TRUE as BOOL,
        );
        EnableWindow(
            GetDlgItem(hwnd, IDC_EDIT_PROXY_PORT as ::core::ffi::c_int),
            TRUE as BOOL,
        );
    }

    port = wcschr(&raw mut address as *mut WCHAR, ':' as WCHAR) as *mut WCHAR;
    if !port.is_null() {
        *port = 0 as WCHAR;
        port = port.offset(1);
    }
    SetDlgItemTextW(
        hwnd,
        IDC_EDIT_PROXY_SERVER as ::core::ffi::c_int,
        &raw mut address as *mut WCHAR as LPCWSTR,
    );
    if !port.is_null() {
        SetDlgItemTextW(
            hwnd,
            IDC_EDIT_PROXY_PORT as ::core::ffi::c_int,
            port as LPCWSTR,
        );
    }

    if pac_url[0 as usize] != 0 {
        CheckDlgButton(
            hwnd,
            IDC_USE_PAC_SCRIPT as ::core::ffi::c_int,
            BST_CHECKED as UINT,
        );
        EnableWindow(
            GetDlgItem(hwnd, IDC_EDIT_PAC_SCRIPT as ::core::ffi::c_int),
            TRUE as BOOL,
        );
        SetDlgItemTextW(
            hwnd,
            IDC_EDIT_PAC_SCRIPT as ::core::ffi::c_int,
            &raw mut pac_url as *mut WCHAR as LPCWSTR,
        );
    }

    return;
}

unsafe extern "C" fn connections_on_command(mut hwnd: HWND, mut wparam: WPARAM) -> INT_PTR {
    let mut checked: BOOL;

    match wparam {
        x if x == (IDC_USE_PAC_SCRIPT as WPARAM) => {
            checked = IsDlgButtonChecked(hwnd, (IDC_USE_PAC_SCRIPT as i32)) as BOOL;
            EnableWindow(GetDlgItem(hwnd, (IDC_EDIT_PAC_SCRIPT as i32)), checked);
        }
        x if x == (IDC_USE_PROXY_SERVER as WPARAM) => {
            checked = IsDlgButtonChecked(hwnd, (IDC_USE_PROXY_SERVER as i32)) as BOOL;
            EnableWindow(GetDlgItem(hwnd, (IDC_EDIT_PROXY_SERVER as i32)), checked);
            EnableWindow(GetDlgItem(hwnd, (IDC_EDIT_PROXY_PORT as i32)), checked);
        }
        _ => {}
    }

    match wparam {
        x if x == (IDC_USE_WPAD as WPARAM)
            || x == (IDC_USE_PAC_SCRIPT as WPARAM)
            || x == (IDC_USE_PROXY_SERVER as WPARAM)
            || x == MAKEWPARAM!(IDC_EDIT_PAC_SCRIPT, EN_CHANGE)
            || x == MAKEWPARAM!(IDC_EDIT_PROXY_SERVER, EN_CHANGE)
            || x == MAKEWPARAM!(IDC_EDIT_PROXY_PORT, EN_CHANGE) =>
        {
            if initdialog_done != 0 {
                SendMessageW(
                    GetParent(hwnd),
                    PSM_CHANGED as UINT,
                    hwnd as WPARAM,
                    0 as LPARAM,
                );
            }
            return TRUE as INT_PTR;
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn connections_on_notify(mut hwnd: HWND, mut wparam: WPARAM, mut lparam: LPARAM) -> INT_PTR {
    let mut default_connection: *mut connection_settings = core::ptr::null_mut();
    let mut proxy: [WCHAR; INTERNET_MAX_URL_LENGTH] = [0; INTERNET_MAX_URL_LENGTH];
    let mut pac_script: [WCHAR; INTERNET_MAX_URL_LENGTH] = [0; INTERNET_MAX_URL_LENGTH];
    let mut psn: *mut PSHNOTIFY = lparam as *mut PSHNOTIFY;
    let mut proxy_len: DWORD = 0;
    let mut port_len: DWORD = 0;
    let mut pac_script_len: DWORD = 0;
    let mut use_proxy: DWORD = 0;
    let mut use_pac_script: DWORD = 0;
    let mut use_wpad: DWORD = 0;
    let mut size: DWORD = 0;
    let mut res: LRESULT = 0;
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut con: HKEY = core::ptr::null_mut();

    if (*psn).hdr.code != (PSN_APPLY) as u32 {
        return FALSE as INT_PTR;
    }

    res = RegOpenKeyW(
        HKEY_CURRENT_USER,
        core::ptr::addr_of!(internet_settings) as LPCWSTR,
        core::ptr::addr_of_mut!(hkey),
    ) as LRESULT;
    if res != 0 {
        return FALSE as INT_PTR;
    }

    use_proxy = IsDlgButtonChecked(hwnd, (IDC_USE_PROXY_SERVER as i32)) as DWORD;
    res = RegSetValueExW(
        hkey,
        core::ptr::addr_of!(proxy_enable) as LPCWSTR,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!(use_proxy) as *mut BYTE,
        core::mem::size_of::<DWORD>() as DWORD,
    ) as LRESULT;
    if res != 0 {
        RegCloseKey(hkey);
        return FALSE as INT_PTR;
    }

    // Diverges from the C, which reads the server into the whole buffer and then appends
    // ":80": a ProxyServer value of 2082+ characters (loaded from the registry at init, past
    // EM_LIMITTEXT) wrote past the stack array. Three slots stay free for ':', "80", NUL.
    proxy_len = GetDlgItemTextW(
        hwnd,
        (IDC_EDIT_PROXY_SERVER as i32),
        core::ptr::addr_of_mut!(proxy) as LPWSTR,
        (ARRAY_SIZE!(proxy) - 3) as ::core::ffi::c_int,
    ) as DWORD;
    if proxy_len != 0 {
        let fresh15 = proxy_len;
        proxy_len = proxy_len.wrapping_add(1);
        proxy[fresh15 as usize] = ':' as i32 as WCHAR;
        port_len = GetDlgItemTextW(
            hwnd,
            (IDC_EDIT_PROXY_PORT as i32),
            (core::ptr::addr_of_mut!(proxy) as *mut WCHAR).add(proxy_len as usize),
            (ARRAY_SIZE!(proxy) as DWORD).wrapping_sub(proxy_len) as ::core::ffi::c_int,
        ) as DWORD;
        if port_len == 0 {
            let fresh16 = proxy_len;
            proxy_len = proxy_len.wrapping_add(1);
            proxy[fresh16 as usize] = '8' as i32 as WCHAR;
            let fresh17 = proxy_len;
            proxy_len = proxy_len.wrapping_add(1);
            proxy[fresh17 as usize] = '0' as i32 as WCHAR;
            proxy[proxy_len as usize] = 0;
        }

        res = RegSetValueExW(
            hkey,
            core::ptr::addr_of!(proxy_server) as LPCWSTR,
            0,
            REG_SZ,
            core::ptr::addr_of_mut!(proxy) as *mut WCHAR as *mut BYTE,
            (proxy_len.wrapping_add(port_len) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ) as LRESULT;
    } else {
        res = RegDeleteValueW(hkey, core::ptr::addr_of!(proxy_server) as LPCWSTR) as LRESULT;
        if res == ERROR_FILE_NOT_FOUND as LRESULT {
            res = ERROR_SUCCESS as LRESULT;
        }
    }
    if res != 0 {
        RegCloseKey(hkey);
        return FALSE as INT_PTR;
    }

    use_pac_script = IsDlgButtonChecked(hwnd, (IDC_USE_PAC_SCRIPT as i32)) as DWORD;
    pac_script_len = GetDlgItemTextW(
        hwnd,
        (IDC_EDIT_PAC_SCRIPT as i32),
        core::ptr::addr_of_mut!(pac_script) as LPWSTR,
        ARRAY_SIZE!(pac_script) as ::core::ffi::c_int,
    ) as DWORD;
    if pac_script_len == 0 {
        use_pac_script = FALSE as DWORD;
    }
    if use_pac_script != 0 {
        res = RegSetValueExW(
            hkey,
            core::ptr::addr_of!(auto_config_url) as LPCWSTR,
            0,
            REG_SZ,
            core::ptr::addr_of_mut!(pac_script) as *mut WCHAR as *mut BYTE,
            (pac_script_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ) as LRESULT;
    } else {
        res = RegDeleteValueW(hkey, core::ptr::addr_of!(auto_config_url) as LPCWSTR) as LRESULT;
        if res == ERROR_FILE_NOT_FOUND as LRESULT {
            res = ERROR_SUCCESS as LRESULT;
        }
    }
    if res != 0 {
        RegCloseKey(hkey);
        return FALSE as INT_PTR;
    }

    use_wpad = IsDlgButtonChecked(hwnd, (IDC_USE_WPAD as i32)) as DWORD;

    res = RegCreateKeyExW(
        hkey,
        core::ptr::addr_of!(connections) as LPCWSTR,
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE as REGSAM,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(con),
        core::ptr::null_mut(),
    ) as LRESULT;
    RegCloseKey(hkey);
    if res != 0 {
        return FALSE as INT_PTR;
    }

    size = create_connection_settings(
        use_proxy as BOOL,
        core::ptr::addr_of_mut!(proxy) as *mut WCHAR,
        use_wpad as BOOL,
        use_pac_script as BOOL,
        core::ptr::addr_of_mut!(pac_script) as *mut WCHAR,
        core::ptr::addr_of_mut!(default_connection),
    );
    if size == 0 {
        RegCloseKey(con);
        return FALSE as INT_PTR;
    }

    res = RegSetValueExW(
        con,
        core::ptr::addr_of!(default_connection_settings) as LPCWSTR,
        0,
        REG_BINARY as DWORD,
        default_connection as *mut BYTE,
        size,
    ) as LRESULT;
    heap_free(default_connection as *mut core::ffi::c_void);
    RegCloseKey(con);
    return (res == 0) as ::core::ffi::c_int as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn connections_dlgproc(mut hwnd: HWND, mut msg: UINT, mut wparam: WPARAM, mut lparam: LPARAM) -> INT_PTR {
    match msg {
        WM_INITDIALOG => {
            connections_on_initdialog(hwnd);
            initdialog_done = TRUE as BOOL;
        }
        WM_COMMAND => return connections_on_command(hwnd, wparam),
        WM_NOTIFY => return connections_on_notify(hwnd, wparam, lparam),
        _ => {}
    }
    return FALSE as INT_PTR;
}
