use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_Text: [TCHAR; 21] = unsafe {
    ::core::mem::transmute::<[u8; 42], [TCHAR; 21]>(
        *b"R\0e\0a\0c\0t\0O\0S\0 \0R\0o\0c\0k\0s\0!\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

// NOTE: MAX_TEXT_LENGTH's #define is not in this preamble; 100 is an assumed value.
#[no_mangle]
pub unsafe extern "C" fn LoadSettings() {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut len: DWORD = core::mem::size_of::<[TCHAR; 21]>() as DWORD;

    if RegCreateKeyEx(
        HKEY_CURRENT_USER,
        [
            'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16,
            '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
            'f' as u16, 't' as u16, '\\' as u16, 'S' as u16, 'c' as u16, 'r' as u16, 'e' as u16, 'e' as u16,
            'n' as u16, 'S' as u16, 'a' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 's' as u16, '\\' as u16,
            'T' as u16, 'e' as u16, 'x' as u16, 't' as u16, '3' as u16, 'D' as u16, 0,
        ]
        .as_ptr(),
        0,
        [0u16].as_ptr() as LPWSTR,
        0,
        KEY_READ,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hkey),
        core::ptr::null_mut(),
    ) == (ERROR_SUCCESS as i32)
    {
        RegQueryValueEx(
            hkey,
            [
                'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16,
                'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 0,
            ]
            .as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(g_Text) as *mut TCHAR as LPBYTE,
            core::ptr::addr_of_mut!(len),
        );
        RegCloseKey(hkey);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SaveSettings() {
    let mut hkey: HKEY = core::ptr::null_mut();

    if RegCreateKeyEx(
        HKEY_CURRENT_USER,
        (&[0x0053, 0x006f, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0053, 0x0063, 0x0072, 0x0065, 0x0065, 0x006e, 0x0053, 0x0061, 0x0076, 0x0065, 0x0072, 0x0073, 0x005c, 0x0054, 0x0065, 0x0078, 0x0074, 0x0033, 0x0044, 0x0000u16]).as_ptr() /* _T "Software\\Microsoft\\ScreenSavers\\Text3D" */,
        0,
        (&[0x0000u16]).as_ptr() as LPWSTR /* _T "" */,
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hkey),
        core::ptr::null_mut(),
    ) == (ERROR_SUCCESS as i32)
    {
        RegSetValueEx(
            hkey,
            (&[0x0044, 0x0069, 0x0073, 0x0070, 0x006c, 0x0061, 0x0079, 0x0053, 0x0074, 0x0072, 0x0069, 0x006e, 0x0067, 0x0000u16]).as_ptr() /* _T "DisplayString" */,
            0,
            REG_SZ,
            (core::ptr::addr_of_mut!(g_Text) as *mut TCHAR) as LPBYTE,
            ((_tcslen(core::ptr::addr_of_mut!(g_Text) as *mut TCHAR) as usize
                + 1 as usize)
                * core::mem::size_of::<TCHAR>()) as DWORD,
        );
        RegCloseKey(hkey);
    }
}
