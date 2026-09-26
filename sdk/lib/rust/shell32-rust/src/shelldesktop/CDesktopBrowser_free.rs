use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn EnumDisplaySettingsW(device: LPCWSTR, mode: DWORD, devmode: *mut DEVMODEW) -> BOOL;
}

// ---- file-scope preamble (translated) ----
pub static szProgmanClassName: [u16; 8] = [
    0x0050, 0x0072, 0x006F, 0x0067, 0x006D, 0x0061, 0x006E, 0x0000,
];

pub static szProgmanWindowName: [u16; 16] = [
    0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020,
    0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000,
];

// Substrate the islands do not carry. `SHIsTempDisplayMode` is the only caller in this TU.
// Added to the PREAMBLE unit rather than the assembled file so it survives reassembly, and
// here rather than in `_CPP_SUPPORT_EXTERNS` because that table feeds the `--cpp-com` path
// only; this TU's file-scope half is a `--free-functions` run.

/// winuser.h:180 `#define ENUM_REGISTRY_SETTINGS ((DWORD)-2)`.
pub const ENUM_REGISTRY_SETTINGS: DWORD = 0xFFFF_FFFE;

#[no_mangle]
pub unsafe extern "system" fn SHIsTempDisplayMode() -> i32 {
    if // `SM_*` are u32 in the island; GetSystemMetrics takes an INT index.
    GetSystemMetrics(SM_REMOTESESSION as INT) != 0
        || GetSystemMetrics(SM_REMOTECONTROL as INT) != 0 {
        return FALSE;
    }

    let mut DevMode: DEVMODEW = core::mem::zeroed();
    ZeroMemory(&mut DevMode, core::mem::size_of::<DEVMODEW>());
    DevMode.dmSize = core::mem::size_of::<DEVMODEW>() as u16;

    if EnumDisplaySettingsW(core::ptr::null_mut(), ENUM_REGISTRY_SETTINGS, &mut DevMode) == 0 {
        return FALSE;
    }

    if DevMode.dmPelsWidth == 0 || DevMode.dmPelsHeight == 0 {
        return FALSE;
    }

    let hDC = GetDC(core::ptr::null_mut());
    let cxWidth = GetDeviceCaps(hDC, HORZRES as INT) as u32;
    let cyHeight = GetDeviceCaps(hDC, VERTRES as INT) as u32;
    ReleaseDC(core::ptr::null_mut(), hDC);

    (cxWidth != DevMode.dmPelsWidth || cyHeight != DevMode.dmPelsHeight) as i32
}
