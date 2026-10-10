use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble ----
// WIN32_NO_STATUS: valueless feature flag, nothing to emit.
// wingdi.h / winuser.h macros the island does not carry:
const ENUM_CURRENT_SETTINGS: DWORD = 0xFFFF_FFFF; // ((DWORD)-1)
const CDS_FULLSCREEN: DWORD = 0x0000_0004;

extern "system" {
    fn EnumDisplaySettingsA(lpszDeviceName: LPCSTR, iModeNum: DWORD, lpDevMode: *mut DEVMODEA) -> BOOL;
    fn ChangeDisplaySettingsA(lpDevMode: *mut DEVMODEA, dwFlags: DWORD) -> LONG;
}

// ======== SHIM_NS Force8BitColor =============================================================
pub mod Force8BitColor {
    use super::*;
    setup_shim!(Force8BitColor);

    // SHIM_NUM_HOOKS 0; SHIM_NOTIFY_FN = Notify
    pub unsafe extern "system" fn Notify(fdwReason: DWORD, ptr: PVOID) -> BOOL {
        if fdwReason == SHIM_REASON_INIT {
            let mut dm: DEVMODEA = core::mem::zeroed();
            dm.dmSize = core::mem::size_of::<DEVMODEA>() as WORD;
            if EnumDisplaySettingsA(core::ptr::null(), ENUM_CURRENT_SETTINGS, &mut dm) != 0
                && dm.dmBitsPerPel != 8
            {
                dm.dmBitsPerPel = 8;
                dm.dmFields |= DM_BITSPERPEL;
                ChangeDisplaySettingsA(&mut dm, CDS_FULLSCREEN);
            }
        }
        return TRUE as BOOL;
    }

    implement_shim!(Force8BitColor, 0usize, Some(Notify), []);
}

// ======== SHIM_NS Force640x480 ===============================================================
pub mod Force640x480 {
    use super::*;
    setup_shim!(Force640x480);

    // SHIM_NUM_HOOKS 0; SHIM_NOTIFY_FN = Notify
    pub unsafe extern "system" fn Notify(fdwReason: DWORD, ptr: PVOID) -> BOOL {
        if fdwReason == SHIM_REASON_INIT {
            let mut dm: DEVMODEA = core::mem::zeroed();
            dm.dmSize = core::mem::size_of::<DEVMODEA>() as WORD;
            if EnumDisplaySettingsA(core::ptr::null(), ENUM_CURRENT_SETTINGS, &mut dm) != 0
                && (dm.dmPelsWidth != 640 || dm.dmPelsHeight != 480)
            {
                dm.dmPelsWidth = 640;
                dm.dmPelsHeight = 480;
                dm.dmFields |= DM_PELSWIDTH | DM_PELSHEIGHT;
                ChangeDisplaySettingsA(&mut dm, CDS_FULLSCREEN);
            }
        }
        return TRUE as BOOL;
    }

    implement_shim!(Force640x480, 0usize, Some(Notify), []);
}
