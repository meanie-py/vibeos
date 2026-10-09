// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_80EED9AFE258: [u16; 13] = [
    0x002Bu16, 0x006F, 0x0070, 0x0065, 0x006E, 0x0067, 0x006C, 0x002C, 0x002B, 0x0077,
    0x0067, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_8621E0B1C1CF: [u16; 13] = [
    0x0044u16, 0x0045, 0x0042, 0x0055, 0x0047, 0x0043, 0x0048, 0x0041, 0x004E, 0x004E,
    0x0045, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const KEY_RENDERER: [::core::ffi::c_ushort; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_ushort; 24]>(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0R\0e\0a\0c\0t\0O\0S\0\\\0O\0p\0e\0n\0G\0L\0\0\0",
    )
};

pub const KEY_DRIVERS: [::core::ffi::c_ushort; 59] = unsafe {
    ::core::mem::transmute::<
        [u8; 118],
        [::core::ffi::c_ushort; 59],
    >(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0O\0p\0e\0n\0G\0L\0D\0r\0i\0v\0e\0r\0s\0\0\0",
    )
};

pub const KEY_DEBUG_CHANNEL: [::core::ffi::c_ushort; 61] = unsafe {
    ::core::mem::transmute::<
        [u8; 122],
        [::core::ffi::c_ushort; 61],
    >(
        *b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0S\0e\0s\0s\0i\0o\0n\0 \0M\0a\0n\0a\0g\0e\0r\0\\\0E\0n\0v\0i\0r\0o\0n\0m\0e\0n\0t\0\0\0",
    )
};

static mut pOglDrivers: *mut PWCHAR = ::core::ptr::null::<PWCHAR>() as *mut PWCHAR;

static mut dwNumDrivers: DWORD = 0 as DWORD;

unsafe extern "C" fn InitSettings(hWndDlg: HWND) {
    let mut hKeyRenderer: HKEY = core::ptr::null_mut();
    let mut hKeyDrivers: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = MAX_KEY_LENGTH;
    let mut szBuffer: [WCHAR; (MAX_KEY_LENGTH) as _] = [0; (MAX_KEY_LENGTH) as _];
    let mut szBultin: [WCHAR; (MAX_KEY_LENGTH) as _] = [0; (MAX_KEY_LENGTH) as _];
    let mut szDriver: [WCHAR; (MAX_KEY_LENGTH) as _] = [0; (MAX_KEY_LENGTH) as _];

    LoadString(hApplet, (IDS_DEBUG_DNM as u32), szBultin.as_mut_ptr() as LPTSTR, 127);
    SendDlgItemMessageW(hWndDlg, (IDC_DEBUG_OUTPUT as i32), CB_ADDSTRING, 0, szBultin.as_mut_ptr() as LPARAM);

    LoadString(hApplet, (IDS_DEBUG_SET as u32), szBultin.as_mut_ptr() as LPTSTR, 127);
    SendDlgItemMessageW(hWndDlg, (IDC_DEBUG_OUTPUT as i32), CB_ADDSTRING, 0, szBultin.as_mut_ptr() as LPARAM);

    LoadString(hApplet, (IDS_DEBUG_CLEAR as u32), szBultin.as_mut_ptr() as LPTSTR, 127);
    SendDlgItemMessageW(hWndDlg, (IDC_DEBUG_OUTPUT as i32), CB_ADDSTRING, 0, szBultin.as_mut_ptr() as LPARAM);

    SendDlgItemMessageW(hWndDlg, (IDC_DEBUG_OUTPUT as i32), CB_SETCURSEL, 0, 0);

    LoadString(hApplet, (IDS_RENDERER_DEFAULT as u32), szBultin.as_mut_ptr() as LPTSTR, 127);
    SendDlgItemMessageW(hWndDlg, (IDC_RENDERER as i32), CB_ADDSTRING, 0, szBultin.as_mut_ptr() as LPARAM);

    LoadString(hApplet, (IDS_RENDERER_RSWR as u32), szBultin.as_mut_ptr() as LPTSTR, 127);
    SendDlgItemMessageW(hWndDlg, (IDC_RENDERER as i32), CB_ADDSTRING, 0, szBultin.as_mut_ptr() as LPARAM);

    if (RegCreateKeyExW(HKEY_CURRENT_USER, (KEY_RENDERER).as_ptr(), 0, core::ptr::null_mut(), 0, MAXIMUM_ALLOWED, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKeyRenderer), core::ptr::null_mut())) as u32 != ERROR_SUCCESS {
        return;
    }

    if (RegQueryValueExW(hKeyRenderer, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwType), szDriver.as_mut_ptr() as LPBYTE, core::ptr::addr_of_mut!(dwSize))) as u32 != ERROR_SUCCESS || dwSize == core::mem::size_of::<WCHAR>() as DWORD {
        SendDlgItemMessageW(hWndDlg, (IDC_RENDERER as i32), CB_SETCURSEL, (RENDERER_DEFAULT) as WPARAM, 0);
    }

    RegCloseKey(hKeyRenderer);

    if dwType == REG_SZ {
        let mut ret: DWORD;
        let mut iKey: DWORD;

        if wcsncmp(szBultin.as_mut_ptr(), szDriver.as_mut_ptr(), (MAX_KEY_LENGTH) as _) == 0 {
            SendDlgItemMessageW(hWndDlg, (IDC_RENDERER as i32), CB_SETCURSEL, (RENDERER_RSWR) as WPARAM, 0);
        }

        if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, (KEY_DRIVERS).as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKeyDrivers))) as u32 != ERROR_SUCCESS {
            return;
        }

        ret = (RegQueryInfoKeyW(hKeyDrivers, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwNumDrivers), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut())) as u32;

        if ret != ERROR_SUCCESS || dwNumDrivers == 0 {
            RegCloseKey(hKeyDrivers);
            return;
        }

        pOglDrivers = HeapAlloc(GetProcessHeap(), 0, (dwNumDrivers as SIZE_T).wrapping_mul(core::mem::size_of::<PWCHAR>() as SIZE_T)) as *mut PWCHAR;

        if pOglDrivers.is_null() {
            dwNumDrivers = 0;
        }

        iKey = 0;
        while iKey < dwNumDrivers {
            dwSize = MAX_KEY_LENGTH;
            ret = (RegEnumKeyEx(hKeyDrivers, iKey, szBuffer.as_mut_ptr(), core::ptr::addr_of_mut!(dwSize), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut())) as u32;

            if ret != ERROR_SUCCESS {
                break;
            }

            /* Mind the null terminator */
            dwSize = dwSize.wrapping_add(1);

            *pOglDrivers.offset(iKey as isize) = HeapAlloc(GetProcessHeap(), 0, (dwSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as PWCHAR;

            if (*pOglDrivers.offset(iKey as isize)).is_null() {
                break;
            }

            SendDlgItemMessageW(hWndDlg, (IDC_RENDERER as i32), CB_ADDSTRING, 0, szBuffer.as_mut_ptr() as LPARAM);

            StringCchCopy(*pOglDrivers.offset(iKey as isize), dwSize as size_t, szBuffer.as_mut_ptr());

            if wcsncmp(szBuffer.as_mut_ptr(), szDriver.as_mut_ptr(), (MAX_KEY_LENGTH) as _) == 0 {
                SendDlgItemMessageW(hWndDlg, (IDC_RENDERER as i32), CB_SETCURSEL, (iKey + 2) as WPARAM, 0);
            }

            iKey = iKey.wrapping_add(1);
        }

        RegCloseKey(hKeyDrivers);
    }

    return;
}

unsafe extern "C" fn SaveSettings(hWndDlg: HWND) {
    let mut hKeyRenderer: HKEY = core::ptr::null_mut();
    let mut hKeyDebug: HKEY = core::ptr::null_mut();
    let mut iSel: INT = 0;

    if (RegOpenKeyExW(HKEY_CURRENT_USER, (KEY_RENDERER).as_ptr(), 0, KEY_WRITE, core::ptr::addr_of_mut!(hKeyRenderer))) as u32 != ERROR_SUCCESS {
        return;
    }

    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, (KEY_DEBUG_CHANNEL).as_ptr(), 0, KEY_WRITE, core::ptr::addr_of_mut!(hKeyDebug))) as u32 == ERROR_SUCCESS
    {
        iSel = SendDlgItemMessageW(hWndDlg, (IDC_DEBUG_OUTPUT as i32), CB_GETCURSEL, 0, 0) as INT;

        match (iSel) as u32 {
            DEBUG_SET => {
                RegSetValueExW(hKeyDebug, RUSTCONV_WSTR_8621E0B1C1CF.as_ptr(), 0, REG_SZ, RUSTCONV_WSTR_80EED9AFE258.as_ptr() as PBYTE, (13 as usize * core::mem::size_of::<WCHAR>()) as DWORD);
            }
            DEBUG_CLEAR => {
                RegSetValueExW(hKeyDebug, RUSTCONV_WSTR_8621E0B1C1CF.as_ptr(), 0, REG_SZ, RUSTCONV_WSTR_96A296D224F2.as_ptr() as PBYTE, core::mem::size_of::<WCHAR>() as DWORD);
            }
            _ => {}
        }
        RegCloseKey(hKeyDebug);
    }

    iSel = SendDlgItemMessageW(hWndDlg, (IDC_RENDERER as i32), CB_GETCURSEL, 0, 0) as INT;

    match (iSel) as u32 {
        _ if (CB_ERR as u32) == (iSel as u32) => {
        }
        RENDERER_DEFAULT => {
            RegSetValueExW(hKeyRenderer, RUSTCONV_WSTR_96A296D224F2.as_ptr(), 0, REG_SZ, RUSTCONV_WSTR_96A296D224F2.as_ptr() as PBYTE, core::mem::size_of::<WCHAR>() as DWORD);
        }
        RENDERER_RSWR => {
            let mut szBuffer: [WCHAR; (MAX_KEY_LENGTH) as _] = [0; (MAX_KEY_LENGTH) as _];
            LoadString(hApplet, (IDS_RENDERER_RSWR as u32), szBuffer.as_mut_ptr() as LPTSTR, 127);
            RegSetValueExW(hKeyRenderer, RUSTCONV_WSTR_96A296D224F2.as_ptr(), 0, REG_SZ, szBuffer.as_mut_ptr() as PBYTE, ((wcslen(szBuffer.as_mut_ptr() as *const WCHAR) as usize + 1 as usize) * core::mem::size_of::<WCHAR>()) as DWORD);
        }
        _ => {
            /* Adjustment for DEFAULT and RSWR renderers */
            iSel -= 2;

            if iSel >= 0 && (iSel as DWORD) < dwNumDrivers {
                RegSetValueExW(hKeyRenderer, RUSTCONV_WSTR_96A296D224F2.as_ptr(), 0, REG_SZ, (*pOglDrivers.offset(iSel as isize)) as PBYTE, ((wcslen((*pOglDrivers.offset(iSel as isize))) as usize + 1 as usize) * core::mem::size_of::<WCHAR>()) as DWORD);
            }
        }
    }

    RegCloseKey(hKeyRenderer);
}
#[no_mangle]
pub unsafe extern "system" fn GeneralPageProc(hWndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut lppsn: LPPSHNOTIFY = core::ptr::null_mut();

    match uMsg {
        WM_INITDIALOG => {
            InitSettings(hWndDlg);
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_RENDERER as u16) ||
                LOWORD!(wParam) == (IDC_DEBUG_OUTPUT as u16)
            {
                if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE
                {
                    PropSheet_Changed!(GetParent(hWndDlg), hWndDlg);
                }
            }
            // break;
        }

        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == PSN_APPLY
            {
                SaveSettings(hWndDlg);
                return (TRUE) as INT_PTR;
            }
            // break;
        }

        WM_DESTROY => {
            if !pOglDrivers.is_null()
            {
                let mut iKey: DWORD = 0;

                iKey = 0 as DWORD;
                while iKey < dwNumDrivers {
                    HeapFree(GetProcessHeap(), 0, (*pOglDrivers.offset(iKey as isize)).cast());
                    iKey = iKey.wrapping_add(1);
                }

                HeapFree(GetProcessHeap(), 0, (pOglDrivers).cast());
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}
