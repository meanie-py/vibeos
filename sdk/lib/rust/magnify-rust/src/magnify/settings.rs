use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut uiZoom: UINT = 3 as UINT;

#[no_mangle]
pub static mut bShowWarning: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut bFollowMouse: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut bFollowFocus: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut bFollowCaret: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut bInvertColors: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut bStartMinimized: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut bShowMagnifier: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut AppBarConfig: _AppBarConfig_t = _AppBarConfig_t {
    cbSize: ::core::mem::size_of::<_AppBarConfig_t>() as DWORD,
    uEdge: -(2 as INT),
    value3: 0 as DWORD,
    value4: 1 as DWORD,
    appBarSizes: tagRECT {
        left: 101 as LONG,
        top: 101 as LONG,
        right: 101 as LONG,
        bottom: 101 as LONG,
    },
    rcFloating: tagRECT {
        left: 20 as LONG,
        top: 20 as LONG,
        right: 600 as LONG,
        bottom: 200 as LONG,
    },
};

// Field names assumed to match the `struct _AppBarConfig_t` definition
// (declared elsewhere in the crate); initializer order follows the C source.
#[no_mangle]
pub unsafe extern "C" fn LoadSettings() {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut value: LONG = 0;
    let mut len: ULONG = 0;
    let mut config_temp: _AppBarConfig_t = core::mem::zeroed();

    if (RegOpenKeyEx(HKEY_CURRENT_USER, b"Software\\Microsoft\\Magnify\0" as *const u8 as *const i8, 0, KEY_READ, core::ptr::addr_of_mut!(hkey))) as u32 != ERROR_SUCCESS {
        return;
    }

    len = core::mem::size_of::<_AppBarConfig_t>() as ULONG;
    if (RegQueryValueEx(hkey, b"AppBar\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(config_temp) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        if config_temp.cbSize == core::mem::size_of::<_AppBarConfig_t>() as ULONG {
            AppBarConfig = config_temp;
        }
    }

    len = core::mem::size_of::<LONG>() as ULONG;
    if (RegQueryValueEx(hkey, b"StationaryMagLevel\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        if value >= 0 && value <= 9 {
            uiZoom = value as UINT;
        }
    }

    len = core::mem::size_of::<LONG>() as ULONG;
    if (RegQueryValueEx(hkey, b"ShowWarning\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        bShowWarning = (value != 0) as BOOL;
    }

    len = core::mem::size_of::<LONG>() as ULONG;
    if (RegQueryValueEx(hkey, b"StationaryInvertColors\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        bInvertColors = (value != 0) as BOOL;
    }

    len = core::mem::size_of::<LONG>() as ULONG;
    if (RegQueryValueEx(hkey, b"StationaryStartMinimized\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        bStartMinimized = (value != 0) as BOOL;
    }

    len = core::mem::size_of::<LONG>() as ULONG;
    if (RegQueryValueEx(hkey, b"StationaryTrackCursor\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        bFollowMouse = (value != 0) as BOOL;
    }

    len = core::mem::size_of::<LONG>() as ULONG;
    if (RegQueryValueEx(hkey, b"StationaryTrackFocus\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        bFollowFocus = (value != 0) as BOOL;
    }

    len = core::mem::size_of::<LONG>() as ULONG;
    if (RegQueryValueEx(hkey, b"StationaryTrackSecondaryFocus\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        bFollowFocus = (value != 0) as BOOL;
    }

    len = core::mem::size_of::<LONG>() as ULONG;
    if (RegQueryValueEx(hkey, b"StationaryTrackText\0" as *const u8 as *const i8, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as *mut BYTE, core::ptr::addr_of_mut!(len))) as u32 == ERROR_SUCCESS {
        bFollowCaret = (value != 0) as BOOL;
    }

    RegCloseKey(hkey);
}
#[no_mangle]
pub unsafe extern "C" fn SaveSettings() {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut value: LONG = 0;
    if RegCreateKeyEx(
        HKEY_CURRENT_USER as HKEY,
        b"Software\\Microsoft\\Magnify\0".as_ptr() as LPCSTR,
        0 as DWORD,
        b"\0".as_ptr() as LPSTR,
        0 as DWORD,
        KEY_WRITE as REGSAM,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hkey),
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return;
    }

    RegSetValueEx(
        hkey,
        b"AppBar\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_BINARY as DWORD,
        core::ptr::addr_of_mut!(AppBarConfig) as *mut BYTE,
        core::mem::size_of::<_AppBarConfig_t>() as DWORD,
    );

    value = uiZoom as LONG;
    RegSetValueEx(
        hkey,
        b"StationaryMagLevel\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(value) as *mut BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );

    value = bShowWarning as LONG;
    RegSetValueEx(
        hkey,
        b"ShowWarning\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(value) as *mut BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );

    value = bInvertColors as LONG;
    RegSetValueEx(
        hkey,
        b"StationaryInvertColors\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(value) as *mut BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );

    value = bStartMinimized as LONG;
    RegSetValueEx(
        hkey,
        b"StationaryStartMinimized\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(value) as *mut BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );

    value = bFollowMouse as LONG;
    RegSetValueEx(
        hkey,
        b"StationaryTrackCursor\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(value) as *mut BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );

    value = bFollowFocus as LONG;
    RegSetValueEx(
        hkey,
        b"StationaryTrackFocus\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(value) as *mut BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );

    value = bFollowFocus as LONG;
    RegSetValueEx(
        hkey,
        b"StationaryTrackSecondaryFocus\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(value) as *mut BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );

    value = bFollowCaret as LONG;
    RegSetValueEx(
        hkey,
        b"StationaryTrackText\0".as_ptr() as LPCSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(value) as *mut BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );

    RegCloseKey(hkey);
}
