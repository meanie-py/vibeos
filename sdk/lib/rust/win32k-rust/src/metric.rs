//! Rust translation of `win32ss/user/ntuser/metric.c` — system metrics.
//! [ntuser wave 2: easy leaves]

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::ZwClose;
use ntkrnl_sys::base::NT_SUCCESS;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// aiSysMet lives in the anonymous PERUSERSERVERINFO tail of tagSERVERINFO,
// which bindgen made opaque (`__bindgen_padding_0`). Reach it via a cast at
// that field's offset (same opaque-base pattern as W32PROCESS in PROCESSINFO).
#[inline]
unsafe fn pusi(p: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    core::ptr::addr_of_mut!((*p).__bindgen_padding_0) as *mut _PERUSERSERVERINFO
}

// Registry helpers (still C, registry.c) — NTAPI (stdcall i386 → `_Reg*@N`).
extern "system" {
    fn RegOpenKey(pwsz_key_name: *const u16, phkey: *mut HKEY) -> NTSTATUS;
    fn RegReadDWORD(hkey: HKEY, pwsz_value: *const u16, pdw_data: *mut DWORD) -> BOOL;
    fn RegGetSectionDWORD(psz_section: *const u16, psz_value: *const u16, dw_default: DWORD) -> DWORD;
}

// NLS_MB_CODE_PAGE_TAG is `#define`d to the global BOOLEAN `NlsMbCodePageTag`,
// an `@ extern` DATA export of ntoskrnl — dead (FALSE) in the shared NLS model.
// Cross-DLL data: i386/arm64 PE need the `__imp_` thunk from raw-dylib
// `+verbatim`; amd64 auto-resolves through the import lib (and BUGCHECKs 0x6B
// if raw-dylib is added), so it uses the win32k_sys bindgen glob declaration.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut NlsMbCodePageTag: BOOLEAN;
}

static mut Setup: BOOL = 0;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserIsDBCSEnabled() -> BOOL {
    NlsMbCodePageTag as BOOL
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserIsDBCSEnabled() -> BOOL {
    NlsMbCodePageTag as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserIsIMMEnabled() -> BOOL {
    user_is_imm_enabled()
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserIsIMMEnabled() -> BOOL {
    user_is_imm_enabled()
}

unsafe fn user_is_imm_enabled() -> BOOL {
    if NlsMbCodePageTag != 0 {
        return TRUE as BOOL;
    }
    (RegGetSectionDWORD(
        w!("IMM").as_ptr() as *const u16,
        w!("LoadIMM").as_ptr() as *const u16,
        FALSE as DWORD,
    ) != 0) as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserIsCTFIMEEnabled() -> BOOL {
    user_is_ctfime_enabled()
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserIsCTFIMEEnabled() -> BOOL {
    user_is_ctfime_enabled()
}

unsafe fn user_is_ctfime_enabled() -> BOOL {
    if RegGetSectionDWORD(
        w!("IMM").as_ptr() as *const u16,
        w!("DontLoadCTFIME").as_ptr() as *const u16,
        FALSE as DWORD,
    ) != 0
    {
        return FALSE as BOOL;
    }
    UserIsIMMEnabled()
}

#[no_mangle]
pub unsafe extern "system" fn InitMetrics() -> BOOL {
    let pi_sys_met = (*pusi(gpsi)).aiSysMet.as_mut_ptr();
    let width: ULONG;
    let height: ULONG;

    let mut dw_value: DWORD = 0;
    let mut h_key: HKEY = core::ptr::null_mut();

    /* Clean boot (SM_CLEANBOOT fallback 0 = normal mode) */
    *pi_sys_met.offset(SM_CLEANBOOT as isize) = 0;
    if NT_SUCCESS(RegOpenKey(
        w!("\\REGISTRY\\MACHINE\\SYSTEM\\CurrentControlSet\\Control\\SafeBoot\\Option").as_ptr() as *const u16,
        &mut h_key,
    )) {
        if RegReadDWORD(h_key, w!("OptionValue").as_ptr() as *const u16, &mut dw_value) != 0 {
            *pi_sys_met.offset(SM_CLEANBOOT as isize) = dw_value as INT;
        }
        ZwClose(h_key as *mut c_void);
    }

    /* FIXME: HACK, due to missing MDEV on first init */
    if gpmdev.is_null() {
        width = 640;
        height = 480;
    } else {
        width = (*(*gpmdev).ppdevGlobal).gdiinfo.ulHorzRes;
        height = (*(*gpmdev).ppdevGlobal).gdiinfo.ulVertRes;
    }

    /* Screen sizes */
    *pi_sys_met.offset(SM_CXSCREEN as isize) = width as INT;
    *pi_sys_met.offset(SM_CYSCREEN as isize) = height as INT;
    *pi_sys_met.offset(SM_XVIRTUALSCREEN as isize) = 0;
    *pi_sys_met.offset(SM_YVIRTUALSCREEN as isize) = 0;
    *pi_sys_met.offset(SM_CXVIRTUALSCREEN as isize) = width as INT;
    *pi_sys_met.offset(SM_CYVIRTUALSCREEN as isize) = height as INT;

    /* NC area sizes */
    *pi_sys_met.offset(SM_CYCAPTION as isize) = gspv.ncm.iCaptionHeight + 1;
    *pi_sys_met.offset(SM_CYSMCAPTION as isize) = gspv.ncm.iSmCaptionHeight + 1;
    *pi_sys_met.offset(SM_CXSIZE as isize) = gspv.ncm.iCaptionHeight;
    *pi_sys_met.offset(SM_CYSIZE as isize) = gspv.ncm.iCaptionHeight;
    *pi_sys_met.offset(SM_CXSMSIZE as isize) = gspv.ncm.iSmCaptionWidth;
    *pi_sys_met.offset(SM_CYSMSIZE as isize) = gspv.ncm.iSmCaptionHeight;
    *pi_sys_met.offset(SM_CXBORDER as isize) = 1;
    *pi_sys_met.offset(SM_CYBORDER as isize) = 1;
    *pi_sys_met.offset(SM_CXFOCUSBORDER as isize) = 1;
    *pi_sys_met.offset(SM_CYFOCUSBORDER as isize) = 1;
    *pi_sys_met.offset(SM_CXDLGFRAME as isize) = 3;
    *pi_sys_met.offset(SM_CYDLGFRAME as isize) = 3;
    *pi_sys_met.offset(SM_CXEDGE as isize) = 2;
    *pi_sys_met.offset(SM_CYEDGE as isize) = 2;
    *pi_sys_met.offset(SM_CXFRAME as isize) = *pi_sys_met.offset(SM_CXDLGFRAME as isize) + gspv.ncm.iBorderWidth;
    *pi_sys_met.offset(SM_CYFRAME as isize) = *pi_sys_met.offset(SM_CYDLGFRAME as isize) + gspv.ncm.iBorderWidth;

    /* Window sizes */
    *pi_sys_met.offset(SM_CXMIN as isize) = 3 * core::cmp::max(gspv.ncm.iCaptionWidth, 8)
        + *pi_sys_met.offset(SM_CYSIZE as isize)
        + 4
        + 4 * gspv.tmCaptionFont.tmAveCharWidth as INT
        + 2 * *pi_sys_met.offset(SM_CXFRAME as isize);
    *pi_sys_met.offset(SM_CYMIN as isize) = *pi_sys_met.offset(SM_CYCAPTION as isize) + 2 * *pi_sys_met.offset(SM_CYFRAME as isize);
    *pi_sys_met.offset(SM_CXMAXIMIZED as isize) = *pi_sys_met.offset(SM_CXSCREEN as isize) + 2 * *pi_sys_met.offset(SM_CXFRAME as isize);
    *pi_sys_met.offset(SM_CYMAXIMIZED as isize) = *pi_sys_met.offset(SM_CYSCREEN as isize) - 20;
    *pi_sys_met.offset(SM_CXFULLSCREEN as isize) = *pi_sys_met.offset(SM_CXSCREEN as isize);
    *pi_sys_met.offset(SM_CYFULLSCREEN as isize) = *pi_sys_met.offset(SM_CYMAXIMIZED as isize) - *pi_sys_met.offset(SM_CYMIN as isize);
    *pi_sys_met.offset(SM_CYKANJIWINDOW as isize) = 0;
    *pi_sys_met.offset(SM_CXMINIMIZED as isize) = gspv.mm.iWidth + 6;
    *pi_sys_met.offset(SM_CYMINIMIZED as isize) = *pi_sys_met.offset(SM_CYCAPTION as isize) + 5;
    *pi_sys_met.offset(SM_CXMINSPACING as isize) = *pi_sys_met.offset(SM_CXMINIMIZED as isize) + gspv.mm.iHorzGap;
    *pi_sys_met.offset(SM_CYMINSPACING as isize) = *pi_sys_met.offset(SM_CYMINIMIZED as isize) + gspv.mm.iVertGap;
    *pi_sys_met.offset(SM_CXMAXTRACK as isize) = *pi_sys_met.offset(SM_CXVIRTUALSCREEN as isize) + 4 + 2 * *pi_sys_met.offset(SM_CXFRAME as isize);
    *pi_sys_met.offset(SM_CYMAXTRACK as isize) = *pi_sys_met.offset(SM_CYVIRTUALSCREEN as isize) + 4 + 2 * *pi_sys_met.offset(SM_CYFRAME as isize);

    /* Icon */
    *pi_sys_met.offset(SM_CXVSCROLL as isize) = gspv.ncm.iScrollWidth;
    *pi_sys_met.offset(SM_CYVTHUMB as isize) = gspv.ncm.iScrollHeight;
    *pi_sys_met.offset(SM_CYHSCROLL as isize) = gspv.ncm.iScrollWidth;
    *pi_sys_met.offset(SM_CXHTHUMB as isize) = gspv.ncm.iScrollHeight;
    *pi_sys_met.offset(SM_CYVSCROLL as isize) = gspv.ncm.iScrollHeight;
    *pi_sys_met.offset(SM_CXHSCROLL as isize) = gspv.ncm.iScrollHeight;
    *pi_sys_met.offset(SM_CXICON as isize) = 32;
    *pi_sys_met.offset(SM_CYICON as isize) = 32;
    *pi_sys_met.offset(SM_CXSMICON as isize) = 16;
    *pi_sys_met.offset(SM_CYSMICON as isize) = 16;
    *pi_sys_met.offset(SM_CXICONSPACING as isize) = gspv.im.iHorzSpacing;
    *pi_sys_met.offset(SM_CYICONSPACING as isize) = gspv.im.iVertSpacing;
    *pi_sys_met.offset(SM_CXCURSOR as isize) = 32;
    *pi_sys_met.offset(SM_CYCURSOR as isize) = 32;
    *pi_sys_met.offset(SM_CXMINTRACK as isize) = *pi_sys_met.offset(SM_CXMIN as isize);
    *pi_sys_met.offset(SM_CYMINTRACK as isize) = *pi_sys_met.offset(SM_CYMIN as isize);
    *pi_sys_met.offset(SM_CXDRAG as isize) = 4;
    *pi_sys_met.offset(SM_CYDRAG as isize) = 4;
    *pi_sys_met.offset(SM_ARRANGE as isize) = gspv.mm.iArrange;

    /* Menu */
    *pi_sys_met.offset(SM_CYMENU as isize) = gspv.ncm.iMenuHeight + 1;
    *pi_sys_met.offset(SM_MENUDROPALIGNMENT as isize) = gspv.bMenuDropAlign;
    *pi_sys_met.offset(SM_CXMENUCHECK as isize) =
        ((1 + gspv.tmMenuFont.tmHeight + gspv.tmMenuFont.tmExternalLeading) & !1) - 1;
    *pi_sys_met.offset(SM_CYMENUCHECK as isize) = *pi_sys_met.offset(SM_CXMENUCHECK as isize);
    *pi_sys_met.offset(SM_CXMENUSIZE as isize) = gspv.ncm.iMenuWidth;
    *pi_sys_met.offset(SM_CYMENUSIZE as isize) = gspv.ncm.iMenuHeight;

    /* Mouse */
    *pi_sys_met.offset(SM_MOUSEPRESENT as isize) = 1;
    *pi_sys_met.offset(SM_MOUSEWHEELPRESENT as isize) = 1;
    *pi_sys_met.offset(SM_CMOUSEBUTTONS as isize) = 2;
    *pi_sys_met.offset(SM_SWAPBUTTON as isize) = if gspv.bMouseBtnSwap != 0 { 1 } else { 0 };
    *pi_sys_met.offset(SM_CXDOUBLECLK as isize) = gspv.iDblClickWidth;
    *pi_sys_met.offset(SM_CYDOUBLECLK as isize) = gspv.iDblClickHeight;

    /* Version info */
    *pi_sys_met.offset(SM_TABLETPC as isize) = 0;
    *pi_sys_met.offset(SM_MEDIACENTER as isize) = 0;
    *pi_sys_met.offset(SM_STARTER as isize) = 0;
    *pi_sys_met.offset(SM_SERVERR2 as isize) = 0;
    *pi_sys_met.offset(SM_PENWINDOWS as isize) = 0;

    /* Other */
    *pi_sys_met.offset(SM_DEBUG as isize) = 0;
    *pi_sys_met.offset(SM_NETWORK as isize) = 3;
    *pi_sys_met.offset(SM_SLOWMACHINE as isize) = 0;
    *pi_sys_met.offset(SM_SECURE as isize) = 0;
    *pi_sys_met.offset(SM_DBCSENABLED as isize) = NlsMbCodePageTag as INT;
    *pi_sys_met.offset(SM_SHOWSOUNDS as isize) = gspv.bShowSounds;
    *pi_sys_met.offset(SM_MIDEASTENABLED as isize) = 0;
    *pi_sys_met.offset(SM_CMONITORS as isize) = 1;
    *pi_sys_met.offset(SM_SAMEDISPLAYFORMAT as isize) = 1;
    *pi_sys_met.offset(SM_IMMENABLED as isize) = NlsMbCodePageTag as INT;

    /* Reserved */
    *pi_sys_met.offset(SM_RESERVED1 as isize) = 0;
    *pi_sys_met.offset(SM_RESERVED2 as isize) = 0;
    *pi_sys_met.offset(SM_RESERVED3 as isize) = 0;
    *pi_sys_met.offset(SM_RESERVED4 as isize) = 0;
    *pi_sys_met.offset(64) = 0;
    *pi_sys_met.offset(65) = 0;
    *pi_sys_met.offset(66) = 0;

    if UserIsDBCSEnabled() != 0 {
        (*gpsi).dwSRVIFlags |= SRVINFO_DBCSENABLED;
    }
    if UserIsIMMEnabled() != 0 {
        (*gpsi).dwSRVIFlags |= SRVINFO_IMM32;
    }
    if UserIsCTFIMEEnabled() != 0 {
        (*gpsi).dwSRVIFlags |= SRVINFO_CTFIME_ENABLED;
    }

    Setup = TRUE as BOOL;
    TRUE as BOOL
}

// UserGetSystemMetrics — NTAPI (stdcall i386).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn UserGetSystemMetrics(index: ULONG) -> LONG {
    user_get_system_metrics(index)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetSystemMetrics(index: ULONG) -> LONG {
    user_get_system_metrics(index)
}

unsafe fn user_get_system_metrics(index: ULONG) -> LONG {
    if index == SM_DBCSENABLED {
        return (((*gpsi).dwSRVIFlags & SRVINFO_DBCSENABLED) != 0) as LONG;
    }
    if index == SM_IMMENABLED {
        return (((*gpsi).dwSRVIFlags & SRVINFO_IMM32) != 0) as LONG;
    }

    /* Get metrics from array */
    if index < SM_CMETRICS {
        return (*pusi(gpsi)).aiSysMet[index as usize];
    }

    /* Special values SM_REMOTESESSION/SM_SHUTTINGDOWN/SM_REMOTECONTROL are
     * all FIXME-0 in C, and any other (invalid) index also returns 0. */
    0
}
