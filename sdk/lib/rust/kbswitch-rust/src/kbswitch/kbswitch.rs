use crate::kbswitch::imemenu::CleanupImeMenus;
use crate::kbswitch::imemenu::CreateImeMenu;
use crate::kbswitch::imemenu::GetRealImeMenuID;
use crate::kbswitch::imemenu::MenuFromImeMenu;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_258F57B256D3: [u16; 10] = [0x0053u16, 0x0048, 0x0045, 0x004C, 0x004C, 0x0048, 0x004F, 0x004F, 0x004B, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut KbSwitchSetHooks: FN_KbSwitchSetHooks = None;

#[no_mangle]
pub static mut SetPenMenuData: FN_SetPenMenuData = None;

#[no_mangle]
pub static mut g_hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut g_hHookDLL: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut g_hTrayIcon: HICON = ::core::ptr::null::<HICON__>() as *mut HICON__;

#[no_mangle]
pub static mut g_hwndLastActive: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut g_iKL: UINT = 0 as UINT;

#[no_mangle]
pub static mut g_cKLs: UINT = 0 as UINT;

#[no_mangle]
pub static mut g_ahKLs: [HKL; 64] = [
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
    ::core::ptr::null::<HKL__>() as *mut HKL__,
];

#[no_mangle]
pub static mut g_aiSysPenIcons: [WORD; 64] = [
    0 as ::core::ffi::c_int as WORD,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

#[no_mangle]
pub static mut g_anToolTipAtoms: [WORD; 64] = [
    0 as ::core::ffi::c_int as WORD,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

#[no_mangle]
pub static mut g_ahSysPenIcons: [HICON; 64] = [
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
    ::core::ptr::null::<HICON__>() as *mut HICON__,
];

#[no_mangle]
pub static mut g_bSysPenNotifyAdded: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut g_anFlags: [BYTE; 64] = [
    0 as ::core::ffi::c_int as BYTE,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

#[no_mangle]
pub static mut g_uTaskbarRestartMsg: UINT = 0 as UINT;

#[no_mangle]
pub static mut g_uShellHookMessage: UINT = 0 as UINT;

#[no_mangle]
pub static mut g_hTrayWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut g_hTrayNotifyWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut g_SpecialIds: [SPECIAL_ID; 256] = [tagSPECIAL_ID {
    dwLayoutId: 0,
    hKL: ::core::ptr::null::<HKL__>() as *mut HKL__,
    szKLID: [0; 9],
}; 256];

#[no_mangle]
pub static mut g_cSpecialIds: INT = 0 as INT;


// WINE_DEFAULT_DEBUG_CHANNEL(internat); -- debug-channel macro, no declaration emitted

pub const WM_NOTIFYICONMSG: u32 = 0x8064;

pub const WM_PENICONMSG: u32 = 0x8065;

pub const NOTIFY_ICON_ID_LANGUAGE: u32 = 223;

pub const NOTIFY_ICON_ID_SYSTEM_PEN: u32 = 224;

pub const TIMER_ID_LANG_CHANGED_DELAYED: u32 = 0x10000;

pub const TIMER_LANG_CHANGED_DELAY: u32 = 200;

pub const IME_HKL_MASK: u32 = 0xF000FFFF;

// IS_KOREAN_IME_HKL(hKL) is function-like: inlined by the functions, not emitted.

pub const MAX_KLS: usize = 64;

pub type FN_KbSwitchSetHooks = Option<unsafe extern "system" fn(bDoHook: BOOL) -> BOOL>;
pub type FN_SetPenMenuData = Option<unsafe extern "system" fn(nID: UINT, dwItemData: DWORD_PTR)>;

pub const LAYOUTF_FAR_EAST: u32 = 0x1;

pub const LAYOUTF_IME_ICON: u32 = 0x2;

pub const LAYOUTF_TOOLTIP_ATOM: u32 = 0x4;

pub const LAYOUTF_REMOVE_LEFT_DEF_MENU: u32 = 0x8;

pub const LAYOUTF_REMOVE_RIGHT_DEF_MENU: u32 = 0x10;

// CCH_LAYOUT_ID is not defined in this preamble (comes from a header).
// Chosen so that CCH_LAYOUT_ID + 1 == KL_NAMELENGTH (9).

#[repr(C)]
#[derive(Clone, Copy)]
pub struct tagSPECIAL_ID {
    pub dwLayoutId: DWORD,
    pub hKL: HKL,
    pub szKLID: [TCHAR; (CCH_LAYOUT_ID + 1) as _],
}
pub type SPECIAL_ID = tagSPECIAL_ID;
pub type PSPECIAL_ID = *mut SPECIAL_ID;

pub const MAX_SPECIAL_IDS: usize = 256;

#[repr(C)]
pub struct tagLOAD_ICON {
    pub nIconIndex: INT,
    pub cxIcon: INT,
    pub cyIcon: INT,
    pub iIcon: INT,
    pub hIcon: HICON,
}
pub type LOAD_ICON = tagLOAD_ICON;
pub type PLOAD_ICON = *mut LOAD_ICON;

pub const IFN_KbSwitchSetHooks: u32 = 1;

pub const IFN_SetPenMenuData: u32 = 14;

unsafe extern "C" fn IsImeOpen(hwndIme: HWND) -> BOOL {
    return SendMessage(hwndIme, WM_IME_CONTROL as UINT, IMC_GETOPENSTATUS as WPARAM, 0 as LPARAM) as BOOL;
}

unsafe extern "C" fn GetImeConversionMode(hwndIme: HWND) -> DWORD {
    return SendMessage(hwndIme, WM_IME_CONTROL, (IMC_GETCONVERSIONMODE as _), 0) as DWORD;
}

unsafe extern "C" fn UpdateTrayInfo() {
    let shell_tray_wnd: [u16; 14] = ['S' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, '_' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'W' as u16, 'n' as u16, 'd' as u16, 0];
    g_hTrayWnd = FindWindow(shell_tray_wnd.as_ptr(), core::ptr::null_mut());
    let tray_notify_wnd: [u16; 14] = ['T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'N' as u16, 'o' as u16, 't' as u16, 'i' as u16, 'f' as u16, 'y' as u16, 'W' as u16, 'n' as u16, 'd' as u16, 0];
    g_hTrayNotifyWnd = FindWindowEx(g_hTrayWnd, core::ptr::null_mut(), tray_notify_wnd.as_ptr(), core::ptr::null_mut());
}

unsafe extern "C" fn LoadSpecialIds() {
    let mut szKLID: [TCHAR; (KL_NAMELENGTH as _)] = [0; (KL_NAMELENGTH as _)];
    let mut szLayoutId: [TCHAR; 16] = [0; 16];
    let mut dwSize: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hLayoutKey: HKEY = core::ptr::null_mut();

    g_cSpecialIds = 0;

    if RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        ['S' as u16, 'Y' as u16, 'S' as u16, 'T' as u16, 'E' as u16, 'M' as u16, '\\' as u16,
         'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16,
         'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
         'S' as u16, 'e' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'o' as u16, 'n' as u16,
         't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, '\\' as u16, 'K' as u16, 'e' as u16,
         'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16,
         'L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, 's' as u16,
         0u16]
            .as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) != (ERROR_SUCCESS as i32)
    {
        return;
    }

    dwIndex = 0;
    'scan: while dwIndex < 1000 {
        'body: {
            dwSize = _countof!(szKLID) as DWORD;
            if RegEnumKeyEx(
                hKey,
                dwIndex,
                addr_of_mut!(szKLID) as *mut TCHAR,
                addr_of_mut!(dwSize),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) != (ERROR_SUCCESS as i32)
            {
                break 'scan;
            }

            if RegOpenKeyEx(
                hKey,
                addr_of_mut!(szKLID) as *mut TCHAR,
                0,
                KEY_READ,
                core::ptr::addr_of_mut!(hLayoutKey),
            ) != (ERROR_SUCCESS as i32)
            {
                break 'body;
            }

            dwSize = core::mem::size_of::<[TCHAR; 16]>() as DWORD;
            if RegQueryValueEx(
                hLayoutKey,
                ['L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16,
                 ' ' as u16, 'I' as u16, 'd' as u16, 0u16]
                    .as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                addr_of_mut!(szLayoutId) as *mut TCHAR as LPBYTE,
                addr_of_mut!(dwSize),
            ) == (ERROR_SUCCESS as i32)
            {
                let dwKLID: DWORD = _tcstoul(
                    addr_of_mut!(szKLID) as *mut TCHAR,
                    core::ptr::null_mut(),
                    16,
                ) as DWORD;
                let wLangId: WORD = LOWORD!(dwKLID);
                let wLayoutId: WORD = LOWORD!(_tcstoul(
                    addr_of_mut!(szLayoutId) as *mut TCHAR,
                    core::ptr::null_mut(),
                    16,
                ));
                let hKL: HKL = ((SPECIAL_MASK as DWORD)
                    | (MAKELONG!(wLangId, wLayoutId) as DWORD)) as LONG_PTR as HKL;

                /* Add a special ID */
                (*core::ptr::addr_of_mut!(g_SpecialIds))[g_cSpecialIds as usize].dwLayoutId =
                    wLayoutId as DWORD;
                (*core::ptr::addr_of_mut!(g_SpecialIds))[g_cSpecialIds as usize].hKL = hKL;
                StringCchCopy(
                    addr_of_mut!((*core::ptr::addr_of_mut!(g_SpecialIds))[g_cSpecialIds as usize].szKLID)
                        as *mut TCHAR,
                    _countof!((*core::ptr::addr_of_mut!(g_SpecialIds))[g_cSpecialIds as usize].szKLID)
                        as usize,
                    addr_of_mut!(szKLID) as *mut TCHAR,
                );
                g_cSpecialIds += 1;
            }

            RegCloseKey(hLayoutKey);

            if g_cSpecialIds >= _countof!(g_SpecialIds) as INT {
                ERR!("g_SpecialIds is full!");
                break 'scan;
            }
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    RegCloseKey(hKey);
}

unsafe extern "C" fn GetKLIDFromHKL(mut hKL: HKL, mut szKLID: LPTSTR, mut KLIDLength: SIZE_T) {
    *szKLID.offset(0 as isize) = 0 as WCHAR;

    if IS_IME_HKL!(hKL) != 0 {
        let fmt: [u16; 6] = ['%' as u16, '0' as u16, '8' as u16, 'l' as u16, 'x' as u16, 0];
        StringCchPrintf!(
            szKLID as STRSAFE_LPWSTR,
            KLIDLength as size_t,
            fmt.as_ptr() as STRSAFE_LPCWSTR,
            hKL as DWORD_PTR as DWORD,
        );
        return;
    }

    if IS_SPECIAL_HKL!(hKL) != 0 {
        let mut i: INT = 0;
        i = 0 as INT;
        while i < g_cSpecialIds {
            if (*(core::ptr::addr_of_mut!(g_SpecialIds) as *mut SPECIAL_ID).offset(i as isize)).hKL == hKL {
                StringCchCopy(
                    szKLID as STRSAFE_LPWSTR,
                    KLIDLength as size_t,
                    core::ptr::addr_of_mut!(
                        (*(core::ptr::addr_of_mut!(g_SpecialIds) as *mut SPECIAL_ID).offset(i as isize)).szKLID
                    ) as *mut TCHAR as STRSAFE_LPCWSTR,
                );
                return;
            }
            i += 1;
        }
    } else {
        let fmt: [u16; 6] = ['%' as u16, '0' as u16, '8' as u16, 'l' as u16, 'x' as u16, 0];
        StringCchPrintf!(
            szKLID as STRSAFE_LPWSTR,
            KLIDLength as size_t,
            fmt.as_ptr() as STRSAFE_LPCWSTR,
            LOWORD!(hKL) as ::core::ffi::c_uint,
        );
    }
}

unsafe extern "C" fn GetTargetWindow(hwndFore: HWND) -> HWND {
    let mut hwndTarget: HWND = if !hwndFore.is_null() { hwndFore } else { GetForegroundWindow() };
    let mut ShellTrayWnd: [u16; 14] = ['S' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, '_' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'W' as u16, 'n' as u16, 'd' as u16, 0];
    if hwndTarget.is_null()
        || IsWndClassName(hwndTarget, INDICATOR_CLASS.as_ptr()) != 0
        || IsWndClassName(hwndTarget, ShellTrayWnd.as_mut_ptr() as PCTSTR) != 0
    {
        hwndTarget = g_hwndLastActive;
    }
    return hwndTarget;
}

unsafe extern "C" fn GetActiveKL() -> HKL {
    /* FIXME: Get correct console window's HKL when console window */
    let mut hwndTarget: HWND = GetTargetWindow(core::ptr::null_mut());

    let mut dwTID: DWORD = GetWindowThreadProcessId(hwndTarget, core::ptr::null_mut());
    return GetKeyboardLayout(dwTID);
}

unsafe extern "C" fn DeletePenIcon(hwnd: HWND, iKL: UINT) {
    let _ = hwnd;

    if !g_ahSysPenIcons[iKL as usize].is_null() {
        DestroyIcon(g_ahSysPenIcons[iKL as usize]);
        g_ahSysPenIcons[iKL as usize] = core::ptr::null_mut();
    }
}

unsafe extern "C" fn DestroyPenIcons() {
    let mut iKL: INT = 0 as INT;
    iKL = 0 as INT;
    while (iKL as UINT) < g_cKLs {
        if !g_ahSysPenIcons[iKL as usize].is_null() {
            DestroyIcon(g_ahSysPenIcons[iKL as usize]);
            g_ahSysPenIcons[iKL as usize] = core::ptr::null_mut();
        }
        iKL += 1;
    }
}

unsafe extern "C" fn GetLayoutIndexFromHKL(mut hKL: HKL) -> UINT {
    let mut iKL: UINT = 0;
    while iKL < g_cKLs {
        if g_ahKLs[iKL as usize] == hKL {
            return iKL;
        }
        iKL = iKL.wrapping_add(1);
    }
    return 0;
}

unsafe extern "C" fn UpdateLayoutList(mut hKL: HKL) {
    if hKL.is_null() {
        hKL = GetActiveKL();
    }

    let mut ahSysPenIcons: [HICON; MAX_KLS] =
        [core::ptr::null_mut(); MAX_KLS];
    let mut aiSysPenIcons: [WORD; MAX_KLS] = [0; MAX_KLS];
    let mut anFlags: [BYTE; MAX_KLS] = [0; MAX_KLS];
    ZeroMemory!(
        ahSysPenIcons.as_mut_ptr() as *mut ::core::ffi::c_void,
        core::mem::size_of::<[HICON; MAX_KLS]>(),
    );
    FillMemory!(
        aiSysPenIcons.as_mut_ptr() as *mut ::core::ffi::c_void,
        core::mem::size_of::<[WORD; MAX_KLS]>(),
        0xFF as u8,
    );
    ZeroMemory!(
        anFlags.as_mut_ptr() as *mut ::core::ffi::c_void,
        core::mem::size_of::<[BYTE; MAX_KLS]>(),
    );

    let mut ahKLs: [HKL; MAX_KLS] = [core::ptr::null_mut(); MAX_KLS];
    let mut iKL: UINT = 0;
    let mut nKLs: UINT = GetKeyboardLayoutList(
        _countof!(ahKLs) as ::core::ffi::c_int,
        ahKLs.as_mut_ptr(),
    );

    /* Reuse old icons and flags */
    iKL = 0 as UINT;
    while iKL < nKLs {
        let mut wLangID: LANGID = LOWORD!(ahKLs[iKL as usize] as DWORD_PTR) as LANGID;
        match wLangID as ::core::ffi::c_int {
            LANGID_CHINESE_SIMPLIFIED
            | LANGID_CHINESE_TRADITIONAL
            | LANGID_JAPANESE
            | LANGID_KOREAN => {
                anFlags[iKL as usize] |= LAYOUTF_FAR_EAST as BYTE;
            }
            _ => {
                anFlags[iKL as usize] &= !(LAYOUTF_FAR_EAST as BYTE);
            }
        }

        let mut iKL2: UINT = 0;
        iKL2 = 0 as UINT;
        while iKL2 < g_cKLs {
            if ahKLs[iKL as usize] == g_ahKLs[iKL2 as usize] {
                ahSysPenIcons[iKL as usize] = g_ahSysPenIcons[iKL2 as usize];
                aiSysPenIcons[iKL as usize] = g_aiSysPenIcons[iKL2 as usize];
                anFlags[iKL as usize] = g_anFlags[iKL2 as usize];

                g_ahSysPenIcons[iKL2 as usize] = core::ptr::null_mut();
                break;
            }
            iKL2 += 1;
        }
        iKL += 1;
    }

    DestroyPenIcons();

    g_cKLs = nKLs;

    C_ASSERT!(core::mem::size_of::<[HKL; 64]>() == core::mem::size_of::<[HKL; MAX_KLS]>());
    CopyMemory!(
        core::ptr::addr_of_mut!(g_ahKLs) as *mut ::core::ffi::c_void,
        ahKLs.as_ptr() as *const ::core::ffi::c_void,
        core::mem::size_of::<[HKL; 64]>(),
    );

    C_ASSERT!(core::mem::size_of::<[HICON; 64]>() == core::mem::size_of::<[HICON; MAX_KLS]>());
    CopyMemory!(
        core::ptr::addr_of_mut!(g_ahSysPenIcons) as *mut ::core::ffi::c_void,
        ahSysPenIcons.as_ptr() as *const ::core::ffi::c_void,
        core::mem::size_of::<[HICON; 64]>(),
    );

    C_ASSERT!(core::mem::size_of::<[BYTE; 64]>() == core::mem::size_of::<[BYTE; MAX_KLS]>());
    CopyMemory!(
        core::ptr::addr_of_mut!(g_anFlags) as *mut ::core::ffi::c_void,
        anFlags.as_ptr() as *const ::core::ffi::c_void,
        core::mem::size_of::<[BYTE; 64]>(),
    );

    g_iKL = GetLayoutIndexFromHKL(hKL);
}

unsafe extern "C" fn GetHKLFromLayoutNum(iKL: UINT) -> HKL {
    return if iKL < g_cKLs {
        g_ahKLs[iKL as usize]
    } else {
        GetActiveKL()
    };
}

unsafe extern "C" fn GetKLIDFromLayoutNum(
    iKL: UINT,
    szKLID: LPTSTR,
    KLIDLength: SIZE_T,
) {
    GetKLIDFromHKL(GetHKLFromLayoutNum(iKL), szKLID, KLIDLength);
}

unsafe extern "C" fn GetSystemLibraryPath(szPath: LPTSTR, cchPath: SIZE_T, FileName: LPCTSTR) -> BOOL {
    if ((GetSystemDirectory(szPath, (cchPath) as u32)) == 0) {
        return (FALSE) as BOOL;
    }

    let backslash: [u16; 2] = ['\\' as u16, 0];
    StringCchCat(szPath, (cchPath) as _, backslash.as_ptr() as LPCTSTR);
    StringCchCat(szPath, (cchPath) as _, FileName);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn GetLayoutName(mut iKL: UINT, mut szName: LPTSTR, mut NameLength: SIZE_T) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut dwBufLen: DWORD;
    let mut szBuf: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szKLID: [TCHAR; (CCH_LAYOUT_ID + 1) as _] =
        [0; (CCH_LAYOUT_ID + 1) as _];

    GetKLIDFromLayoutNum(iKL, szKLID.as_mut_ptr(), _countof!(szKLID) as SIZE_T);

    let fmt_layout: [u16; 53] = [
        'S' as u16, 'Y' as u16, 'S' as u16, 'T' as u16, 'E' as u16, 'M' as u16, '\\' as u16,
        'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16,
        'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
        'S' as u16, 'e' as u16, 't' as u16, '\\' as u16,
        'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
        '\\' as u16,
        'K' as u16, 'e' as u16, 'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16,
        'd' as u16, ' ' as u16,
        'L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, 's' as u16,
        '\\' as u16,
        '%' as u16, 's' as u16, 0,
    ];

    StringCchPrintf!(
        szBuf.as_mut_ptr(),
        _countof!(szBuf) as SIZE_T,
        fmt_layout.as_ptr(),
        szKLID.as_mut_ptr(),
    );

    if RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        szBuf.as_mut_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) != (ERROR_SUCCESS as i32)
    {
        return (FALSE) as BOOL;
    }

    /* Use "Layout Display Name" value as an entry name if possible */
    let name_display: [u16; 20] = [
        'L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, ' ' as u16,
        'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16,
        ' ' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 0,
    ];
    hr = SHLoadRegUIString(hKey, name_display.as_ptr(), szName, NameLength as DWORD);
    if hr >= 0 {
        RegCloseKey(hKey);
        return (TRUE) as BOOL;
    }

    /* Otherwise, use "Layout Text" value as an entry name */
    dwBufLen = (NameLength as usize).wrapping_mul(core::mem::size_of::<TCHAR>() as usize) as DWORD;
    let name_text: [u16; 12] = [
        'L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, ' ' as u16,
        'T' as u16, 'e' as u16, 'x' as u16, 't' as u16, 0,
    ];
    if RegQueryValueEx(
        hKey,
        name_text.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        szName as LPBYTE,
        core::ptr::addr_of_mut!(dwBufLen),
    ) == (ERROR_SUCCESS as i32)
    {
        RegCloseKey(hKey);
        return (TRUE) as BOOL;
    }

    RegCloseKey(hKey);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn GetImeFile(mut szImeFile: LPTSTR, mut cchImeFile: SIZE_T, mut szKLID: LPCTSTR) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwBufLen: DWORD = 0;
    let mut szBuf: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    *szImeFile.offset(0 as isize) = UNICODE_NULL;

    if _tcslen(szKLID) != (CCH_LAYOUT_ID as _) {
        return (FALSE) as BOOL; /* Invalid LCID */
    }

    if *szKLID.offset(0 as isize) != ('E' as u16 as TCHAR)
        && *szKLID.offset(0 as isize) != ('e' as u16 as TCHAR)
    {
        return (FALSE) as BOOL; /* Not an IME HKL */
    }

    let fmt_klid: [u16; 53] = [
        'S' as u16, 'Y' as u16, 'S' as u16, 'T' as u16, 'E' as u16, 'M' as u16, '\\' as u16,
        'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16,
        'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
        'S' as u16, 'e' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'o' as u16, 'n' as u16,
        't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, '\\' as u16, 'K' as u16, 'e' as u16,
        'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16,
        'L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, 's' as u16,
        '\\' as u16, '%' as u16, 's' as u16, 0,
    ];

    StringCchPrintf!(
        szBuf.as_mut_ptr(),
        _countof!(szBuf),
        fmt_klid.as_ptr() as *const TCHAR,
        szKLID,
    );

    if RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        szBuf.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &raw mut hKey,
    ) != (ERROR_SUCCESS as i32)
    {
        return (FALSE) as BOOL;
    }

    dwBufLen = (cchImeFile as usize).wrapping_mul(core::mem::size_of::<TCHAR>()) as DWORD;

    let val_name: [u16; 9] = [
        'I' as u16, 'M' as u16, 'E' as u16, ' ' as u16, 'F' as u16, 'i' as u16, 'l' as u16,
        'e' as u16, 0,
    ];

    if RegQueryValueEx(
        hKey,
        val_name.as_ptr() as *const TCHAR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        szImeFile as LPBYTE,
        &raw mut dwBufLen,
    ) != (ERROR_SUCCESS as i32)
    {
        *szImeFile.offset(0 as isize) = UNICODE_NULL;
    }

    RegCloseKey(hKey);

    return (*szImeFile.offset(0 as isize) != UNICODE_NULL) as BOOL;
}

unsafe extern "system" fn EnumResNameProc(
    hModule: HMODULE,
    lpszType: LPCTSTR,
    lpszName: LPTSTR,
    lParam: LPARAM,
) -> BOOL {
    UNREFERENCED_PARAMETER!(lpszType);

    let pLoadIcon: PLOAD_ICON = lParam as PLOAD_ICON;
    if (*pLoadIcon).iIcon == (*pLoadIcon).nIconIndex {
        (*pLoadIcon).hIcon = LoadImage(
            hModule as HINSTANCE,
            lpszName as LPCWSTR,
            IMAGE_ICON as UINT,
            (*pLoadIcon).cxIcon as ::core::ffi::c_int,
            (*pLoadIcon).cyIcon as ::core::ffi::c_int,
            LR_DEFAULTCOLOR as UINT,
        ) as HICON;
        if !(*pLoadIcon).hIcon.is_null() {
            return (FALSE) as BOOL; /* Stop enumeration */
        }
    }

    (*pLoadIcon).iIcon += 1;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn FakeExtractIcon(pszImeFile: PCTSTR, nIconIndex: INT) -> HICON {
    let hImeDLL: HMODULE = LoadLibraryEx(
        pszImeFile as LPCWSTR,
        core::ptr::null_mut(),
        LOAD_LIBRARY_AS_DATAFILE as DWORD,
    ) as HMODULE;
    if hImeDLL.is_null() {
        return core::ptr::null_mut();
    }

    let mut LoadIcon: LOAD_ICON = LOAD_ICON {
        nIconIndex: nIconIndex,
        cxIcon: GetSystemMetrics(SM_CXSMICON as INT) as INT,
        cyIcon: GetSystemMetrics(SM_CXSMICON as INT) as INT,
        iIcon: 0,
        hIcon: core::ptr::null_mut(),
    };
    EnumResourceNames(
        hImeDLL,
        RT_GROUP_ICON as LPCWSTR,
        Some(EnumResNameProc),
        core::ptr::addr_of_mut!(LoadIcon) as LPARAM,
    );

    FreeLibrary(hImeDLL);

    return LoadIcon.hIcon;
}

unsafe extern "C" fn BitmapFromIcon(mut hIcon: HICON) -> HBITMAP {
    let mut hdcScreen: HDC = GetDC(::core::ptr::null_mut::<HWND__>());
    let mut hdc: HDC = CreateCompatibleDC(hdcScreen);
    let mut cxIcon: INT = GetSystemMetrics((SM_CXSMICON as i32)) as INT;
    let mut cyIcon: INT = GetSystemMetrics((SM_CYSMICON as i32)) as INT;
    let mut hbm: HBITMAP = CreateCompatibleBitmap(hdcScreen, cxIcon, cyIcon);
    let mut hbmOld: HGDIOBJ = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if !hbm.is_null() {
        hbmOld = SelectObject(hdc, hbm as HGDIOBJ);
        DrawIconEx(
            hdc,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            hIcon,
            cxIcon as ::core::ffi::c_int,
            cyIcon as ::core::ffi::c_int,
            0 as UINT,
            GetSysColorBrush((COLOR_MENU as i32)),
            DI_NORMAL as UINT,
        );
        SelectObject(hdc, hbmOld);
    }
    DeleteDC(hdc);
    ReleaseDC(::core::ptr::null_mut::<HWND__>(), hdcScreen);
    return hbm;
}

unsafe extern "C" fn GetImeStatus(hwndTarget: HWND) -> DWORD {
    let hwndIme: HWND = ImmGetDefaultIMEWnd(hwndTarget);
    if hwndIme.is_null() {
        return IME_STATUS_IME_CLOSED;
    }

    let hIMC: HIMC = SendMessage(
        hwndIme,
        WM_IME_SYSTEM as UINT,
        IMS_GETCONTEXT as WPARAM,
        hwndTarget as LPARAM,
    ) as HIMC;
    if hIMC.is_null() {
        return IME_STATUS_NO_IME;
    }

    let mut dwImeStatus: DWORD = if IsImeOpen(hwndIme) != 0 {
        IME_STATUS_IME_OPEN
    } else {
        IME_STATUS_IME_CLOSED
    };
    if GetACP() == 949 {
        let dwConversion: DWORD = GetImeConversionMode(hwndIme);
        if dwConversion & IME_CMODE_NATIVE != 0 {
            dwImeStatus |= IME_STATUS_IME_NATIVE;
        }

        if dwConversion & IME_CMODE_FULLSHAPE != 0 {
            dwImeStatus |= IME_STATUS_IME_FULLSHAPE;
        }
    }

    return dwImeStatus;
}

unsafe extern "C" fn LoadDefaultPenIcon(szImeFile: PCWSTR, hKL: HKL) -> HICON {
    let hwndTarget: HWND = if !g_hwndLastActive.is_null() {
        g_hwndLastActive
    } else {
        GetForegroundWindow()
    };
    let dwImeStatus: DWORD = GetImeStatus(hwndTarget);

    let mut nIconID: INT = -(1 as INT);
    if IS_KOREAN_IME_HKL!(hKL) {
        if dwImeStatus != (IME_STATUS_NO_IME as DWORD) {
            if (dwImeStatus & (IME_STATUS_IME_CLOSED as DWORD)) != 0 {
                nIconID = IDI_KOREAN_A_HALF as INT;
            } else {
                if (dwImeStatus & (IME_STATUS_IME_FULLSHAPE as DWORD)) != 0 {
                    if (dwImeStatus & (IME_STATUS_IME_NATIVE as DWORD)) != 0 {
                        nIconID = IDI_KOREAN_JR_FULL as INT;
                    } else {
                        nIconID = IDI_KOREAN_A_FULL as INT;
                    }
                } else {
                    if (dwImeStatus & (IME_STATUS_IME_NATIVE as DWORD)) != 0 {
                        nIconID = IDI_KOREAN_JR_HALF as INT;
                    } else {
                        nIconID = IDI_KOREAN_A_HALF as INT;
                    }
                }
            }
        }
    } else {
        if (dwImeStatus & (IME_STATUS_IME_CLOSED as DWORD)) != 0 {
            nIconID = IDI_IME_CLOSED as INT;
        } else if (dwImeStatus & (IME_STATUS_IME_OPEN as DWORD)) != 0 {
            nIconID = IDI_IME_OPEN as INT;
        } else {
            nIconID = IDI_IME_DISABLED as INT;
        }
    }

    if nIconID < (0 as INT) {
        return core::ptr::null_mut();
    }

    return LoadIcon(g_hHookDLL, MAKEINTRESOURCE!(nIconID));
}

unsafe extern "C" fn DeletePenNotifyIcon(hwnd: HWND) {
    if g_bSysPenNotifyAdded == 0 {
        return;
    }

    let mut nid: NOTIFYICONDATA = core::mem::zeroed();
    nid.cbSize = core::mem::size_of::<NOTIFYICONDATA>() as DWORD;
    nid.hWnd = hwnd;
    nid.uID = NOTIFY_ICON_ID_SYSTEM_PEN as UINT;
    if Shell_NotifyIcon(NIM_DELETE as DWORD, core::ptr::addr_of_mut!(nid)) == 0 {
        ERR!("Shell_NotifyIcon(NIM_DELETE) failed\n");
    } else {
        g_bSysPenNotifyAdded = FALSE as BOOL;
    }
}

unsafe extern "C" fn UpdatePenIcon(mut hwnd: HWND, mut iKL: UINT) {
    DeletePenIcon(hwnd, iKL);
    if g_anFlags[iKL as usize] as ::core::ffi::c_int & (LAYOUTF_FAR_EAST as i32) == 0 {
        DeletePenNotifyIcon(hwnd);
        return;
    }
    let mut szKLID: [TCHAR; (CCH_LAYOUT_ID + 1) as _] = [0; (CCH_LAYOUT_ID + 1) as _];
    let mut szImeFile: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    GetKLIDFromHKL(
        g_ahKLs[iKL as usize],
        &raw mut szKLID as LPTSTR,
        (_countof!(szKLID)) as SIZE_T,
    );
    if GetImeFile(
        &raw mut szImeFile as LPTSTR,
        (_countof!(szImeFile)) as SIZE_T,
        &raw mut szKLID as *mut TCHAR as LPCTSTR,
    ) == 0
    {
        DeletePenNotifyIcon(hwnd);
        return;
    }
    assert!(g_ahSysPenIcons[iKL as usize].is_null());
    if (g_anFlags[iKL as usize] as ::core::ffi::c_int & (LAYOUTF_IME_ICON as i32)) != 0 {
        g_ahSysPenIcons[iKL as usize] = FakeExtractIcon(
            &raw mut szImeFile as *mut TCHAR as PCTSTR,
            g_aiSysPenIcons[iKL as usize] as INT,
        );
    }
    if g_ahSysPenIcons[iKL as usize].is_null() {
        g_ahSysPenIcons[iKL as usize] = LoadDefaultPenIcon(
            &raw mut szImeFile as *mut TCHAR as PCWSTR,
            g_ahKLs[iKL as usize],
        );
    }
    if g_ahSysPenIcons[iKL as usize].is_null() {
        DeletePenNotifyIcon(hwnd);
        return;
    }
    let mut nid: NOTIFYICONDATA = _NOTIFYICONDATAW {
        cbSize: ::core::mem::size_of::<NOTIFYICONDATA>() as DWORD,
        hWnd: hwnd,
        uID: NOTIFY_ICON_ID_SYSTEM_PEN as UINT,
        uFlags: 0,
        uCallbackMessage: 0,
        hIcon: ::core::ptr::null_mut::<HICON__>(),
        szTip: [0; 128],
        dwState: 0,
        dwStateMask: 0,
        szInfo: [0; 256],
        __bindgen_anon_1: _NOTIFYICONDATAW__bindgen_ty_1 { uTimeout: 0 },
        szInfoTitle: [0; 64],
        dwInfoFlags: 0,
        guidItem: _GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
    };
    nid.uFlags = (NIF_ICON | NIF_MESSAGE | NIF_TIP) as UINT;
    nid.uCallbackMessage = WM_PENICONMSG as UINT;
    nid.hIcon = g_ahSysPenIcons[iKL as usize];
    if g_anToolTipAtoms[iKL as usize] != 0 {
        GlobalGetAtomName(
            g_anToolTipAtoms[iKL as usize],
            &raw mut nid.szTip as LPWSTR,
            _countof!({ nid.szTip }) as ::core::ffi::c_int,
        );
    } else {
        ImmGetDescription(
            g_ahKLs[iKL as usize],
            &raw mut nid.szTip as LPWSTR,
            _countof!({ nid.szTip }) as UINT,
        );
    }
    if Shell_NotifyIcon(
        (if g_bSysPenNotifyAdded != 0 {
            NIM_MODIFY
        } else {
            NIM_ADD
        }) as DWORD,
        &raw mut nid,
    ) == 0
    {
        ERR!("Shell_NotifyIcon failed\n");
    } else {
        g_bSysPenNotifyAdded = TRUE as BOOL;
    }
}

unsafe extern "C" fn CreateTrayIcon(szKLID: LPTSTR, szImeFile: LPCTSTR) -> HICON {
    let mut LangID: LANGID = 0;
    let mut szBuf: [TCHAR; 4] = [0; 4];
    let mut hdcScreen: HDC = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hbmColor: HBITMAP = core::ptr::null_mut();
    let mut hbmMono: HBITMAP = core::ptr::null_mut();
    let mut hBmpOld: HBITMAP = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hFontOld: HFONT = core::ptr::null_mut();
    let mut lf: LOGFONT = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut IconInfo: ICONINFO = core::mem::zeroed();
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut cxIcon: INT = GetSystemMetrics((SM_CXSMICON as i32)) as INT;
    let mut cyIcon: INT = GetSystemMetrics((SM_CYSMICON as i32)) as INT;
    let mut szPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if !szImeFile.is_null() && *szImeFile.offset(0 as isize) != 0 {
        if GetSystemLibraryPath(
            core::ptr::addr_of_mut!(szPath) as *mut TCHAR as LPTSTR,
            _countof!(szPath) as SIZE_T,
            szImeFile,
        ) != 0
        {
            return FakeExtractIcon(
                core::ptr::addr_of_mut!(szPath) as *mut TCHAR as PCTSTR,
                0 as INT,
            );
        }
    }

    /* Getting "EN", "FR", etc. from English, French, ... */
    LangID = LANGIDFROMLCID!(
        _tcstoul(szKLID, core::ptr::null_mut(), 16) as LCID,
    ) as LANGID;
    if GetLocaleInfo(
        LangID as LCID,
        (LOCALE_SABBREVLANGNAME | LOCALE_NOUSEROVERRIDE) as LCTYPE,
        core::ptr::addr_of_mut!(szBuf) as *mut TCHAR as LPTSTR,
        _countof!(szBuf) as INT,
    ) == 0 as INT
    {
        szBuf[0 as usize] = ('?' as u16);
        szBuf[1 as usize] = ('?' as u16);
    }
    szBuf[2 as usize] = 0 as TCHAR; /* Truncate the identifier to two characters: "ENG" --> "EN" etc. */

    /* Create hdc, hbmColor and hbmMono */
    hdcScreen = GetDC(core::ptr::null_mut());
    hdc = CreateCompatibleDC(hdcScreen);
    hbmColor = CreateCompatibleBitmap(hdcScreen, cxIcon, cyIcon);
    ReleaseDC(core::ptr::null_mut(), hdcScreen);
    hbmMono = CreateBitmap(
        cxIcon,
        cyIcon,
        1 as UINT,
        1 as UINT,
        core::ptr::null(),
    );

    /* Checking NULL */
    if hdc.is_null() || hbmColor.is_null() || hbmMono.is_null() {
        if !hbmMono.is_null() {
            DeleteObject(hbmMono as HGDIOBJ);
        }
        if !hbmColor.is_null() {
            DeleteObject(hbmColor as HGDIOBJ);
        }
        if !hdc.is_null() {
            DeleteDC(hdc);
        }
        return core::ptr::null_mut();
    }

    /* Create a font */
    hFont = core::ptr::null_mut();
    if SystemParametersInfo(
        SPI_GETICONTITLELOGFONT as UINT,
        core::mem::size_of::<LOGFONT>() as UINT,
        core::ptr::addr_of_mut!(lf) as PVOID,
        0 as UINT,
    ) != 0
    {
        /* Override the current size with something manageable */
        lf.lfHeight = -(11 as LONG);
        lf.lfWidth = 0 as LONG;
        hFont = CreateFontIndirect(core::ptr::addr_of_mut!(lf));
    }
    if hFont.is_null() {
        hFont = GetStockObject((DEFAULT_GUI_FONT as i32)) as HFONT;
    }

    SetRect(
        core::ptr::addr_of_mut!(rect),
        0 as INT,
        0 as INT,
        cxIcon,
        cyIcon,
    );

    /* Draw hbmColor */
    hBmpOld = SelectObject(hdc, hbmColor as HGDIOBJ) as HBITMAP;
    SetDCBrushColor(
        hdc,
        GetSysColor((COLOR_HIGHLIGHT as i32)) as COLORREF,
    );
    FillRect(
        hdc,
        core::ptr::addr_of_mut!(rect) as LPCRECT,
        GetStockObject((DC_BRUSH as i32)) as HBRUSH,
    );
    hFontOld = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
    SetTextColor(
        hdc,
        GetSysColor((COLOR_HIGHLIGHTTEXT as i32)) as COLORREF,
    );
    SetBkMode(hdc, TRANSPARENT as INT);
    DrawText(
        hdc,
        core::ptr::addr_of_mut!(szBuf) as *mut TCHAR as LPCTSTR,
        2 as INT,
        core::ptr::addr_of_mut!(rect),
        (DT_SINGLELINE | DT_CENTER | DT_VCENTER) as UINT,
    );
    SelectObject(hdc, hFontOld as HGDIOBJ);

    /* Fill hbmMono with black */
    SelectObject(hdc, hbmMono as HGDIOBJ);
    PatBlt(
        hdc,
        0 as INT,
        0 as INT,
        cxIcon,
        cyIcon,
        BLACKNESS as DWORD,
    );
    SelectObject(hdc, hBmpOld as HGDIOBJ);

    /* Create an icon from hbmColor and hbmMono */
    IconInfo.fIcon = TRUE as BOOL;
    IconInfo.yHotspot = 0 as DWORD;
    IconInfo.xHotspot = IconInfo.yHotspot;
    IconInfo.hbmColor = hbmColor;
    IconInfo.hbmMask = hbmMono;
    hIcon = CreateIconIndirect(core::ptr::addr_of_mut!(IconInfo));

    /* Clean up */
    DeleteObject(hFont as HGDIOBJ);
    DeleteObject(hbmMono as HGDIOBJ);
    DeleteObject(hbmColor as HGDIOBJ);
    DeleteDC(hdc);

    return hIcon;
}

unsafe extern "C" fn AddTrayIcon(hwnd: HWND) {
    let mut tnid: NOTIFYICONDATA = core::mem::zeroed();
    tnid.cbSize = core::mem::size_of::<NOTIFYICONDATA>() as DWORD;
    tnid.hWnd = hwnd;
    tnid.uID = NOTIFY_ICON_ID_LANGUAGE as UINT;
    tnid.uFlags = (NIF_ICON | NIF_MESSAGE | NIF_TIP) as UINT;
    let mut szKLID: [TCHAR; (CCH_LAYOUT_ID + 1) as _] = [0; (CCH_LAYOUT_ID + 1) as _];
    let mut szName: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szImeFile: [TCHAR; 80] = [0; 80];

    GetKLIDFromLayoutNum(
        g_iKL,
        core::ptr::addr_of_mut!(szKLID) as LPTSTR,
        (core::mem::size_of::<[TCHAR; (CCH_LAYOUT_ID + 1) as _]>() / core::mem::size_of::<TCHAR>()) as SIZE_T,
    );
    GetLayoutName(
        g_iKL,
        core::ptr::addr_of_mut!(szName) as LPTSTR,
        (core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<TCHAR>()) as SIZE_T,
    );
    GetImeFile(
        core::ptr::addr_of_mut!(szImeFile) as LPTSTR,
        (core::mem::size_of::<[TCHAR; 80]>() / core::mem::size_of::<TCHAR>()) as SIZE_T,
        core::ptr::addr_of_mut!(szKLID) as LPCTSTR,
    );

    tnid.uCallbackMessage = WM_NOTIFYICONMSG as UINT;
    tnid.hIcon = CreateTrayIcon(
        core::ptr::addr_of_mut!(szKLID) as LPTSTR,
        core::ptr::addr_of_mut!(szImeFile) as LPCTSTR,
    );
    StringCchCopy(
        core::ptr::addr_of_mut!(tnid.szTip) as LPTSTR,
        ((core::mem::size_of::<[TCHAR; 128]>() / core::mem::size_of::<TCHAR>()) as SIZE_T) as _,
        core::ptr::addr_of_mut!(szName) as LPCTSTR,
    );

    if Shell_NotifyIcon(NIM_ADD, core::ptr::addr_of_mut!(tnid)) == 0 {
        ERR!("Shell_NotifyIcon(NIM_ADD) failed\n");
    }

    if !g_hTrayIcon.is_null() {
        DestroyIcon(g_hTrayIcon);
    }
    g_hTrayIcon = tnid.hIcon;
}

unsafe extern "C" fn DeleteTrayIcon(hwnd: HWND) {
    let mut tnid: NOTIFYICONDATA = core::mem::zeroed();
    tnid.cbSize = core::mem::size_of::<NOTIFYICONDATA>() as DWORD;
    tnid.hWnd = hwnd;
    tnid.uID = NOTIFY_ICON_ID_LANGUAGE as UINT;
    if Shell_NotifyIcon(NIM_DELETE, core::ptr::addr_of_mut!(tnid)) == 0 {
        ERR!("Shell_NotifyIcon(NIM_DELETE) failed\n");
    }

    if !g_hTrayIcon.is_null() {
        DestroyIcon(g_hTrayIcon);
        g_hTrayIcon = core::ptr::null_mut();
    }
}

unsafe extern "C" fn UpdateTrayIcon(mut hwnd: HWND, mut szKLID: LPTSTR, mut szName: LPTSTR) {
    let mut tnid: NOTIFYICONDATA = _NOTIFYICONDATAW {
        cbSize: ::core::mem::size_of::<NOTIFYICONDATA>() as DWORD,
        hWnd: hwnd,
        uID: NOTIFY_ICON_ID_LANGUAGE as UINT,
        uFlags: (NIF_ICON | NIF_MESSAGE | NIF_TIP) as UINT,
        uCallbackMessage: 0,
        hIcon: ::core::ptr::null_mut::<HICON__>(),
        szTip: [0; 128],
        dwState: 0,
        dwStateMask: 0,
        szInfo: [0; 256],
        __bindgen_anon_1: _NOTIFYICONDATAW__bindgen_ty_1 { uTimeout: 0 },
        szInfoTitle: [0; 64],
        dwInfoFlags: 0,
        guidItem: _GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
    };
    let mut szImeFile: [TCHAR; 80] = [0; 80];
    GetImeFile(
        &raw mut szImeFile as LPTSTR,
        (::core::mem::size_of::<[TCHAR; 80]>() as SIZE_T)
            .wrapping_div(::core::mem::size_of::<TCHAR>() as SIZE_T),
        szKLID as LPCTSTR,
    );
    tnid.uCallbackMessage = WM_NOTIFYICONMSG as UINT;
    tnid.hIcon = CreateTrayIcon(szKLID, &raw mut szImeFile as *mut TCHAR as LPCTSTR);
    StringCchCopy(
        &raw mut tnid.szTip as STRSAFE_LPWSTR,
        (::core::mem::size_of::<[WCHAR; 128]>() as size_t)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as size_t),
        szName as STRSAFE_LPCWSTR,
    );
    Shell_NotifyIcon(NIM_MODIFY as DWORD, &raw mut tnid);
    if !g_hTrayIcon.is_null() {
        DestroyIcon(g_hTrayIcon);
    }
    g_hTrayIcon = tnid.hIcon;
}

unsafe extern "system" fn EnumWindowsProc(hwnd: HWND, lParam: LPARAM) -> BOOL {
    PostMessage(
        hwnd,
        WM_INPUTLANGCHANGEREQUEST as UINT,
        INPUTLANGCHANGE_SYSCHARSET as WPARAM,
        lParam,
    );
    return (TRUE) as BOOL;
}

unsafe extern "C" fn ActivateLayout(hwnd: HWND, iKL: UINT, hwndTarget: HWND, bNoActivate: BOOL) {
    let mut hKl: HKL = core::ptr::null_mut();
    let mut szKLID: [TCHAR; (CCH_LAYOUT_ID + 1) as _] = [0; (CCH_LAYOUT_ID + 1) as _];
    let mut szLangName: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut LangID: LANGID = 0;

    if iKL >= g_cKLs {
        return;
    }

    GetKLIDFromLayoutNum(
        iKL,
        core::ptr::addr_of_mut!(szKLID) as LPTSTR,
        _countof!(szKLID) as SIZE_T,
    );
    LangID = _tcstoul(
        core::ptr::addr_of_mut!(szKLID) as *mut TCHAR,
        core::ptr::null_mut(),
        16 as core::ffi::c_int,
    ) as LANGID;

    /* Switch to the new keyboard layout */
    GetLocaleInfo(
        LangID as LCID,
        LOCALE_SLANGUAGE as LCTYPE,
        core::ptr::addr_of_mut!(szLangName) as LPTSTR,
        _countof!(szLangName) as i32,
    );
    UpdateTrayIcon(
        hwnd,
        core::ptr::addr_of_mut!(szKLID) as LPTSTR,
        core::ptr::addr_of_mut!(szLangName) as LPTSTR,
    );

    if !hwndTarget.is_null() && bNoActivate == 0 {
        SetForegroundWindow(hwndTarget);
    }

    hKl = LoadKeyboardLayout(
        core::ptr::addr_of_mut!(szKLID) as *mut TCHAR as LPCWSTR,
        KLF_ACTIVATE as UINT,
    );
    if !hKl.is_null() {
        ActivateKeyboardLayout(hKl, KLF_SETFORPROCESS as UINT);
    }

    /* Post WM_INPUTLANGCHANGEREQUEST */
    if !hwndTarget.is_null() {
        PostMessage(
            hwndTarget,
            WM_INPUTLANGCHANGEREQUEST as UINT,
            INPUTLANGCHANGE_SYSCHARSET as WPARAM,
            hKl as LPARAM,
        );
    } else {
        EnumWindows(
            Some(EnumWindowsProc),
            hKl as LPARAM,
        );
    }

    g_iKL = iKL;
}

unsafe extern "C" fn BuildLeftPopupMenu() -> HMENU {
    let mut hMenu: HMENU = CreatePopupMenu();
    let mut szName: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szKLID: [TCHAR; (CCH_LAYOUT_ID + 1) as _] = [0; (CCH_LAYOUT_ID + 1) as _];
    let mut szImeFile: [TCHAR; 80] = [0; 80];
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut mii: MENUITEMINFO = core::mem::zeroed();
    mii.cbSize = core::mem::size_of::<MENUITEMINFO>() as UINT;
    let mut iKL: UINT = 0;

    iKL = 0 as UINT;
    while iKL < g_cKLs {
        GetKLIDFromHKL(
            g_ahKLs[iKL as usize],
            &raw mut szKLID as LPTSTR,
            (core::mem::size_of::<[TCHAR; (CCH_LAYOUT_ID + 1) as _]>() as SIZE_T)
                .wrapping_div(core::mem::size_of::<TCHAR>() as SIZE_T),
        );
        GetImeFile(
            &raw mut szImeFile as LPTSTR,
            (core::mem::size_of::<[TCHAR; 80]>() as SIZE_T)
                .wrapping_div(core::mem::size_of::<TCHAR>() as SIZE_T),
            &raw mut szKLID as *mut TCHAR as LPCTSTR,
        );

        if GetLayoutName(
            iKL,
            &raw mut szName as LPTSTR,
            (core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>() as SIZE_T)
                .wrapping_div(core::mem::size_of::<TCHAR>() as SIZE_T),
        ) == 0
        {
            iKL = iKL.wrapping_add(1);
            continue;
        }

        mii.fMask = (MIIM_ID | MIIM_STRING) as UINT;
        mii.wID = (ID_LANG_BASE as UINT).wrapping_add(iKL);
        mii.dwTypeData = &raw mut szName as *mut TCHAR as LPWSTR;

        hIcon = CreateTrayIcon(
            &raw mut szKLID as LPTSTR,
            &raw mut szImeFile as *mut TCHAR as LPCTSTR,
        );
        if !hIcon.is_null() {
            mii.hbmpItem = BitmapFromIcon(hIcon);
            if !mii.hbmpItem.is_null() {
                mii.fMask |= MIIM_BITMAP as UINT;
            }
        }

        InsertMenuItem(
            hMenu,
            (-(1 as ::core::ffi::c_int)) as UINT,
            (TRUE as UINT) as i32,
            &raw mut mii as LPCMENUITEMINFOW,
        );
        DestroyIcon(hIcon);

        iKL = iKL.wrapping_add(1);
    }

    CheckMenuItem(
        hMenu,
        (ID_LANG_BASE as UINT).wrapping_add(g_iKL),
        MF_CHECKED as UINT,
    );

    return hMenu;
}

unsafe extern "C" fn SetHooks() -> BOOL {
    let __name: [u16; 13] = ['i' as u16, 'n' as u16, 'd' as u16, 'i' as u16, 'c' as u16, 'd' as u16, 'l' as u16, 'l' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
    g_hHookDLL = LoadLibrary((__name.as_ptr()));
    if g_hHookDLL.is_null() {
        return (FALSE) as BOOL;
    }

    KbSwitchSetHooks = core::mem::transmute::<FARPROC, FN_KbSwitchSetHooks>(
        GetProcAddress(g_hHookDLL as HINSTANCE, MAKEINTRESOURCEA!(1 /* IFN_KbSwitchSetHooks */) as LPCSTR),
    );
    SetPenMenuData = core::mem::transmute::<FARPROC, FN_SetPenMenuData>(
        GetProcAddress(g_hHookDLL as HINSTANCE, MAKEINTRESOURCEA!(14 /* IFN_SetPenMenuData */) as LPCSTR),
    );

    if KbSwitchSetHooks.is_none()
        || SetPenMenuData.is_none()
        || (KbSwitchSetHooks.unwrap_unchecked())((TRUE as i32)) == 0
    {
        ERR!("SetHooks failed\n");
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn DeleteHooks() {
    if KbSwitchSetHooks.is_some() {
        (KbSwitchSetHooks.unwrap_unchecked())((FALSE as i32));
        KbSwitchSetHooks = None;
    }

    if !g_hHookDLL.is_null() {
        FreeLibrary(g_hHookDLL);
        g_hHookDLL = core::ptr::null_mut();
    }

    TRACE!("DeleteHooks OK\n");
}

unsafe extern "C" fn GetLayoutNum(hKL: HKL) -> UINT {
    let mut iKL: UINT;

    iKL = 0;
    while iKL < g_cKLs {
        if g_ahKLs[iKL as usize] == hKL {
            return iKL;
        }
        iKL = iKL.wrapping_add(1);
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateLanguageDisplay(mut hwnd: HWND, mut hKL: HKL) -> UINT {
    let mut szKLID: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szLangName: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut LangID: LANGID = 0;

    GetKLIDFromHKL(hKL, &raw mut szKLID as LPTSTR, (_countof!(szKLID)) as SIZE_T);
    LangID = _tcstoul(
        &raw mut szKLID as *mut TCHAR,
        core::ptr::null_mut(),
        16,
    ) as LANGID;
    GetLocaleInfo(
        LangID as LCID,
        LOCALE_SLANGUAGE as LCTYPE,
        &raw mut szLangName as LPWSTR,
        _countof!(szLangName) as core::ffi::c_int,
    );
    UpdateTrayIcon(
        hwnd,
        &raw mut szKLID as LPTSTR,
        &raw mut szLangName as LPTSTR,
    );
    g_iKL = GetLayoutNum(hKL);

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateLanguageDisplayCurrent(mut hwnd: HWND, mut hwndFore: HWND) -> UINT {
    UpdateLayoutList(core::ptr::null_mut());
    let mut dwThreadID: DWORD = GetWindowThreadProcessId(
        GetTargetWindow(hwndFore),
        core::ptr::null_mut(),
    );
    let mut hKL: HKL = GetKeyboardLayout(dwThreadID);
    UpdateLanguageDisplay(hwnd, hKL);
    UpdatePenIcon(hwnd, g_iKL);
    return 0;
}

unsafe extern "C" fn RememberLastActive(hwnd: HWND, mut hwndFore: HWND) -> BOOL {
    static SHELL_TRAY_WND: [u16; 14] = ['S' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, '_' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'W' as u16, 'n' as u16, 'd' as u16, 0];
    hwndFore = GetAncestor(hwndFore, GA_ROOT);

    if IsWindowVisible(hwndFore) == 0 {
        return (FALSE) as BOOL;
    }

    if IsWndClassName(hwndFore, INDICATOR_CLASS.as_ptr()) != 0
        || IsWndClassName(hwndFore, SHELL_TRAY_WND.as_ptr() as PCTSTR) != 0
    {
        return (FALSE) as BOOL; /* Special window */
    }

    g_hwndLastActive = hwndFore;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn KbSwitch_OnCreate(hwnd: HWND) -> INT {
    if SetHooks() == 0 {
        let text_failed: [u16; 17] = ['S' as u16, 'e' as u16, 't' as u16, 'H' as u16, 'o' as u16, 'o' as u16, 'k' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'd' as u16, '.' as u16, 0];
        MessageBox(
            core::ptr::null_mut(),
            text_failed.as_ptr(),
            core::ptr::null_mut(),
            MB_ICONERROR,
        );
        return -1;
    }

    LoadSpecialIds();
    UpdateTrayInfo();
    UpdateLayoutList(core::ptr::null_mut());
    AddTrayIcon(hwnd);
    UpdatePenIcon(hwnd, g_iKL);

    ActivateLayout(hwnd, g_iKL, core::ptr::null_mut(), (TRUE) as BOOL);
    let text_taskbar: [u16; 15] = ['T' as u16, 'a' as u16, 's' as u16, 'k' as u16, 'b' as u16, 'a' as u16, 'r' as u16, 'C' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'd' as u16, 0];
    g_uTaskbarRestartMsg = RegisterWindowMessage(text_taskbar.as_ptr());

    return 0;
}

unsafe extern "C" fn KbSwitch_OnDestroy(hwnd: HWND) {
    KillTimer(hwnd, (TIMER_ID_LANG_CHANGED_DELAYED) as UINT_PTR);
    DeleteHooks();
    DeleteTrayIcon(hwnd);
    DestroyPenIcons();
    DeletePenNotifyIcon(hwnd);
    PostQuitMessage(0);
}

unsafe extern "C" fn KbSwitch_OnTimer(hwnd: HWND, nTimerID: UINT_PTR) {
    if nTimerID == TIMER_ID_LANG_CHANGED_DELAYED as UINT_PTR {
        KillTimer(hwnd, nTimerID);
        let hKL: HKL = GetActiveKL();
        UpdateLayoutList(hKL);
        UpdateLanguageDisplay(hwnd, hKL);
        UpdatePenIcon(hwnd, g_iKL);
    }
}

unsafe extern "C" fn KbSwitch_OnNotifyIconMsg(hwnd: HWND, uMouseMsg: UINT) {
    if uMouseMsg != WM_LBUTTONUP as UINT && uMouseMsg != WM_RBUTTONUP as UINT {
        return;
    }

    UpdateLayoutList(core::ptr::null_mut());

    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(core::ptr::addr_of_mut!(pt));

    SetForegroundWindow(hwnd);

    let mut params: TPMPARAMS = core::mem::zeroed();
    params.cbSize = core::mem::size_of::<TPMPARAMS>() as UINT;
    GetWindowRect(g_hTrayNotifyWnd, core::ptr::addr_of_mut!(params.rcExclude));

    let mut nID: INT = 0;
    if uMouseMsg == WM_LBUTTONUP as UINT
    {
        /* Rebuild the left popup menu on every click to take care of keyboard layout changes */
        let hPopupMenu: HMENU = BuildLeftPopupMenu();
        let uFlags: UINT = (TPM_VERTICAL | TPM_RIGHTALIGN | TPM_RETURNCMD | TPM_LEFTBUTTON) as UINT;
        nID = TrackPopupMenuEx(hPopupMenu, uFlags, pt.x as core::ffi::c_int, pt.y as core::ffi::c_int, hwnd, core::ptr::addr_of_mut!(params)) as INT;
        DestroyMenu(hPopupMenu);
    }
    else /* WM_RBUTTONUP */
    {
        let hPopupMenu: HMENU = LoadMenu(g_hInst, MAKEINTRESOURCE!(IDR_POPUP));
        let hSubMenu: HMENU = GetSubMenu(hPopupMenu, 0);
        let uFlags: UINT = (TPM_VERTICAL | TPM_RIGHTALIGN | TPM_RETURNCMD | TPM_RIGHTBUTTON) as UINT;
        nID = TrackPopupMenuEx(hSubMenu, uFlags, pt.x as core::ffi::c_int, pt.y as core::ffi::c_int, hwnd, core::ptr::addr_of_mut!(params)) as INT;
        DestroyMenu(hPopupMenu);
    }

    PostMessage(hwnd, WM_NULL as UINT, 0 as WPARAM, 0 as LPARAM);

    if nID != 0 {
        PostMessage(hwnd, WM_COMMAND as UINT, nID as WPARAM, 0 as LPARAM);
    }
}

unsafe extern "C" fn IsRegImeToolbarShown() -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let TextPath: [u16; 27] = ['C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'P' as u16, 'a' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\\' as u16, 'I' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16, ' ' as u16, 'M' as u16, 'e' as u16, 't' as u16, 'h' as u16, 'o' as u16, 'd' as u16, 0];
    let mut error: LSTATUS = RegOpenKey(HKEY_CURRENT_USER, TextPath.as_ptr(), core::ptr::addr_of_mut!(hKey));
    if error != 0 {
        ERR!("Cannot open regkey: 0x%lX\n", error);
        return (TRUE) as BOOL;
    }

    let mut szText: [WCHAR; 8] = [0; 8];
    let mut cbValue: DWORD = core::mem::size_of::<[WCHAR; 8]>() as DWORD;
    let ShowStatus: [u16; 12] = ['S' as u16, 'h' as u16, 'o' as u16, 'w' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, 0];
    error = RegQueryValueEx(
        hKey,
        ShowStatus.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (core::ptr::addr_of_mut!(szText) as *mut WCHAR) as PBYTE,
        core::ptr::addr_of_mut!(cbValue),
    );
    if error != 0 {
        RegCloseKey(hKey);
        return (TRUE) as BOOL;
    }

    let mut ret: BOOL = (_wtoi(core::ptr::addr_of_mut!(szText) as *mut WCHAR) != 0) as BOOL;
    RegCloseKey(hKey);
    return ret;
}

unsafe extern "C" fn ShowImeToolbar(hwndTarget: HWND, bShowToolbar: BOOL) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: LSTATUS = RegOpenKey(HKEY_CURRENT_USER, (&[0x0043, 0x006f, 0x006e, 0x0074, 0x0072, 0x006f, 0x006c, 0x0020, 0x0050, 0x0061, 0x006e, 0x0065, 0x006c, 0x005c, 0x0049, 0x006e, 0x0070, 0x0075, 0x0074, 0x0020, 0x004d, 0x0065, 0x0074, 0x0068, 0x006f, 0x0064, 0x0000u16]).as_ptr() /* TEXT "Control Panel\\Input Method" */, core::ptr::addr_of_mut!(hKey));
    if error != 0 {
        ERR!("Cannot open regkey: 0x%lX\n", error);
        return;
    }

    let mut szText: [WCHAR; 8] = [0; 8];
    StringCchPrintf!(szText.as_mut_ptr(), _countof!(szText), (&[0x0025, 0x0064, 0x0000u16]).as_ptr() /* TEXT "%d" */, bShowToolbar);

    let mut cbValue: DWORD = ((lstrlen(szText.as_ptr() as LPCTSTR) + 1) as usize)
        .wrapping_mul(core::mem::size_of::<TCHAR>()) as DWORD;
    RegSetValueEx(hKey, (&[0x0053, 0x0068, 0x006f, 0x0077, 0x0020, 0x0053, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x0000u16]).as_ptr() /* TEXT "Show Status" */, 0, REG_SZ, szText.as_mut_ptr() as PBYTE, cbValue);
    RegCloseKey(hKey);

    let mut hwndIme: HWND = ImmGetDefaultIMEWnd(hwndTarget);
    PostMessage(hwndIme, WM_IME_SYSTEM, (IMS_NOTIFYIMESHOW as _), bShowToolbar as LPARAM);
}

unsafe extern "C" fn KbSwitch_OnPenIconMsg(hwnd: HWND, uMouseMsg: UINT) {
    if uMouseMsg != WM_LBUTTONUP as UINT && uMouseMsg != WM_RBUTTONUP as UINT {
        return;
    }

    if (g_anFlags[g_iKL as usize] as UINT & LAYOUTF_FAR_EAST as UINT) == 0 {
        return;
    }

    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(core::ptr::addr_of_mut!(pt));

    // Get target window
    let mut hwndTarget: HWND = GetTargetWindow(g_hwndLastActive);

    // Get default IME window
    let mut hwndIme: HWND = ImmGetDefaultIMEWnd(hwndTarget);
    if hwndIme.is_null() {
        WARN!("No default IME\n");
        return;
    }

    // Get IME context from another process
    let mut hIMC: HIMC = SendMessage(
        hwndIme,
        WM_IME_SYSTEM as UINT,
        IMS_GETCONTEXT as WPARAM,
        hwndTarget as LPARAM,
    ) as HIMC;
    if hIMC.is_null() {
        WARN!("No HIMC\n");
        return;
    }

    // Is IME open?
    let mut bImeOn: BOOL = IsImeOpen(hwndIme);

    // Workaround of TrackPopupMenu's bug.
    // NOTE: This might change IME status.
    SetForegroundWindow(hwnd);

    // Create IME menu
    let mut bRightButton: BOOL = (uMouseMsg == WM_RBUTTONUP as UINT) as BOOL;
    let mut pImeMenu: PIMEMENUNODE = CreateImeMenu(hIMC, core::ptr::null_mut(), bRightButton);
    let mut hMenu: HMENU = MenuFromImeMenu(pImeMenu as *const IMEMENUNODE);

    let mut hKL: HKL = g_ahKLs[g_iKL as usize];
    let mut bSoftOn: BOOL = FALSE as BOOL;
    let mut bShowToolbar: BOOL = FALSE as BOOL;
    let mut szText: [TCHAR; 128 as usize] = [0; 128 as usize];
    if bRightButton != 0 {
        if (g_anFlags[g_iKL as usize] as UINT & LAYOUTF_REMOVE_RIGHT_DEF_MENU as UINT) == 0 {
            // Add default menu items?
            if GetMenuItemCount(hMenu) != 0 {
                AppendMenu(
                    hMenu,
                    MF_SEPARATOR as UINT,
                    0 as UINT_PTR,
                    core::ptr::null(),
                ); // Separator
            }

            // "Input System (IME) configuration..."
            LoadString(
                g_hInst,
                IDS_INPUTSYSTEM as UINT,
                szText.as_mut_ptr(),
                _countof!(szText) as INT,
            );
            AppendMenu(
                hMenu,
                MF_STRING as UINT,
                ID_INPUTSYSTEM as UINT_PTR,
                szText.as_ptr(),
            );
        }
    } else {
        if (g_anFlags[g_iKL as usize] as UINT & LAYOUTF_REMOVE_LEFT_DEF_MENU as UINT) == 0 {
            // Add default menu items?
            if GetMenuItemCount(hMenu) != 0 {
                AppendMenu(
                    hMenu,
                    MF_SEPARATOR as UINT,
                    0 as UINT_PTR,
                    core::ptr::null(),
                ); // Separator
            }

            if !(IS_KOREAN_IME_HKL!(hKL) != false) {
                // Not Korean IME?
                // "IME ON / OFF"
                let mut nId: UINT = (if bImeOn != 0 {
                    IDS_IME_ON
                } else {
                    IDS_IME_OFF
                }) as UINT;
                LoadString(g_hInst, nId, szText.as_mut_ptr(), _countof!(szText) as INT);
                AppendMenu(
                    hMenu,
                    MF_STRING as UINT,
                    ID_IMEONOFF as UINT_PTR,
                    szText.as_ptr(),
                );
            }

            if (ImmGetProperty(hKL, IGP_CONVERSION as DWORD) as DWORD
                & IME_CMODE_SOFTKBD as DWORD)
                != 0
                && IsWindow(hwndIme) != 0
            // Is Soft Keyboard available?
            {
                // "Soft Keyboard ON / OFF"
                bSoftOn = (SendMessage(
                    hwndIme,
                    WM_IME_SYSTEM as UINT,
                    IMS_GETCONVSTATUS as WPARAM,
                    0 as LPARAM,
                ) & IME_CMODE_SOFTKBD as LRESULT) as BOOL;
                let mut nId_0: UINT = (if bSoftOn != 0 {
                    IDS_SOFTKBD_ON
                } else {
                    IDS_SOFTKBD_OFF
                }) as UINT;
                LoadString(g_hInst, nId_0, szText.as_mut_ptr(), _countof!(szText) as INT);
                AppendMenu(
                    hMenu,
                    MF_STRING as UINT,
                    ID_SOFTKBDONOFF as UINT_PTR,
                    szText.as_ptr(),
                );
            }

            if GetMenuItemCount(hMenu) != 0 {
                AppendMenu(
                    hMenu,
                    MF_SEPARATOR as UINT,
                    0 as UINT_PTR,
                    core::ptr::null(),
                ); // Separator
            }

            // "Show toolbar"
            LoadString(
                g_hInst,
                IDS_SHOWTOOLBAR as UINT,
                szText.as_mut_ptr(),
                _countof!(szText) as INT,
            );
            AppendMenu(
                hMenu,
                MF_STRING as UINT,
                ID_SHOWTOOLBAR as UINT_PTR,
                szText.as_ptr(),
            );
            bShowToolbar = IsRegImeToolbarShown();
            if bShowToolbar != 0 {
                CheckMenuItem(
                    hMenu,
                    ID_SHOWTOOLBAR as UINT,
                    MF_CHECKED as UINT,
                );
            }
        }
    }

    if GetMenuItemCount(hMenu) == 0 {
        // No items?
        // Clean up
        DestroyMenu(hMenu);
        CleanupImeMenus();

        SetForegroundWindow(hwndTarget);
        return;
    }

    // TrackPopupMenuEx flags
    let mut uFlags: UINT = (TPM_VERTICAL | TPM_RIGHTALIGN | TPM_RETURNCMD) as UINT;
    uFlags |= (if bRightButton != 0 {
        TPM_RIGHTBUTTON
    } else {
        TPM_LEFTBUTTON
    }) as UINT;

    // Exclude the notification area
    let mut params: TPMPARAMS = TPMPARAMS {
        cbSize: core::mem::size_of::<TPMPARAMS>() as UINT,
        rcExclude: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
    };
    GetWindowRect(g_hTrayNotifyWnd, core::ptr::addr_of_mut!(params.rcExclude));

    // Show the popup menu
    let mut nID: INT = TrackPopupMenuEx(
        hMenu,
        uFlags,
        pt.x,
        pt.y,
        hwnd,
        core::ptr::addr_of_mut!(params),
    ) as INT;

    // Workaround of TrackPopupMenu's bug
    PostMessage(
        hwnd,
        WM_NULL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    // Back to target window
    SetForegroundWindow(hwndTarget);

    if nID != 0 {
        // Action!
        if nID >= ID_STARTIMEMENU as INT {
            // IME internal menu ID?
            let mut mii: MENUITEMINFO = MENUITEMINFO {
                cbSize: core::mem::size_of::<MENUITEMINFO>() as UINT,
                fMask: MIIM_DATA as UINT,
                fType: 0,
                fState: 0,
                wID: 0,
                hSubMenu: core::ptr::null_mut(),
                hbmpChecked: core::ptr::null_mut(),
                hbmpUnchecked: core::ptr::null_mut(),
                dwItemData: 0,
                dwTypeData: core::ptr::null_mut(),
                cch: 0,
                hbmpItem: core::ptr::null_mut(),
            };
            GetMenuItemInfo(
                hMenu,
                nID as UINT,
                FALSE as BOOL,
                core::ptr::addr_of_mut!(mii),
            );

            if !pImeMenu.is_null() {
                nID = GetRealImeMenuID(pImeMenu as *const IMEMENUNODE, nID);
            }

            if SetPenMenuData.is_some() {
                SetPenMenuData.unwrap_unchecked()(
                    nID as UINT,
                    mii.dwItemData as DWORD_PTR,
                );
            }

            PostMessage(
                hwndIme,
                WM_IME_SYSTEM as UINT,
                IMS_IMEMENUITEMSELECTED as WPARAM,
                hwndTarget as LPARAM,
            );
        } else {
            // Otherwise action of IME menu item
            match (nID) as u32 {
                ID_INPUTSYSTEM => {
                    if IS_IME_HKL!(hKL) != 0 {
                        PostMessage(
                            hwndIme,
                            WM_IME_SYSTEM as UINT,
                            IMS_CONFIGURE as WPARAM,
                            hKL as LPARAM,
                        );
                    }
                }
                ID_IMEONOFF => {
                    PostMessage(
                        hwndIme,
                        WM_IME_SYSTEM as UINT,
                        IMS_SETOPENSTATUS as WPARAM,
                        (bImeOn == 0) as INT as LPARAM,
                    );
                }
                ID_SOFTKBDONOFF => {
                    PostMessage(
                        hwndIme,
                        WM_IME_SYSTEM as UINT,
                        IMS_SOFTKBDONOFF as WPARAM,
                        (bSoftOn == 0) as INT as LPARAM,
                    );
                }
                ID_SHOWTOOLBAR => {
                    ShowImeToolbar(hwndTarget, (bShowToolbar == 0) as INT);
                }
                _ => {
                    PostMessage(
                        hwnd,
                        WM_COMMAND as UINT,
                        nID as WPARAM,
                        0 as LPARAM,
                    );
                }
            }
        }
    }

    // Clean up
    DestroyMenu(hMenu);
    CleanupImeMenus();
}

unsafe extern "C" fn KbSwitch_OnCommand(hwnd: HWND, nID: UINT) {
    match nID {
        ID_EXIT => {
            PostMessage(hwnd, WM_CLOSE as UINT, 0 as WPARAM, 0 as LPARAM);
        }
        ID_PREFERENCES => {
            static CONTROL_EXE: [u16; 12] = ['c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, '.' as u16, 'e' as u16, 'x' as u16, 'e' as u16, 0];
            static INPUT_DLL: [u16; 10] = ['i' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
            static CANT_START: [u16; 22] = ['C' as u16, 'a' as u16, 'n' as u16, '\'' as u16, 't' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
            let mut ret: INT_PTR = ShellExecute(hwnd, core::ptr::null_mut(),
                                                (CONTROL_EXE.as_ptr()), (INPUT_DLL.as_ptr()),
                                                core::ptr::null_mut(), SW_SHOWNORMAL as INT) as INT_PTR;
            if ret <= 32 as INT_PTR {
                MessageBox(hwnd, (CANT_START.as_ptr()), core::ptr::null_mut(), MB_ICONERROR as UINT);
            }
        }
        _ => {
            if nID >= ID_LANG_BASE as UINT {
                if IsWindow(g_hwndLastActive) == 0 {
                    g_hwndLastActive = core::ptr::null_mut();
                }
                ActivateLayout(hwnd, nID.wrapping_sub(ID_LANG_BASE as UINT), g_hwndLastActive, FALSE as BOOL);
            }
        }
    }
}

unsafe extern "C" fn KbSwitch_OnLangChanged(
    mut hwnd: HWND,
    mut hwndTarget: HWND,
    mut hKL: HKL,
) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"base/applications/kbswitch/kbswitch.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"KbSwitch_OnLangChanged\0" as *const u8 as *const ::core::ffi::c_char,
            1300 as ::core::ffi::c_int,
            b"WM_LANG_CHANGED: hwndTarget:%p, hKL:%p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwndTarget,
            hKL,
        );
    }
    KillTimer(hwnd, TIMER_ID_LANG_CHANGED_DELAYED as UINT_PTR);
    SetTimer(
        hwnd,
        TIMER_ID_LANG_CHANGED_DELAYED as UINT_PTR,
        TIMER_LANG_CHANGED_DELAY as UINT,
        None,
    );
    return 0 as LRESULT;
}

unsafe extern "C" fn KbSwitch_OnWindowActivate(hwnd: HWND, hwndTarget: HWND, lParam: LPARAM) -> LRESULT {
    let mut hwndFore: HWND = if !hwndTarget.is_null() {
        hwndTarget
    } else {
        GetForegroundWindow()
    };
    if RememberLastActive(hwnd, hwndFore) != 0 {
        return UpdateLanguageDisplayCurrent(hwnd, hwndFore) as LRESULT;
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn KbSwitch_OnSettingChange(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) {
    if wParam == SPI_SETNONCLIENTMETRICS as WPARAM {
        PostMessage(hwnd, WM_WINDOW_ACTIVATE as UINT, 0 as WPARAM, 0 as LPARAM);
    }
}

unsafe extern "C" fn KbSwitch_OnDefault(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if uMsg == g_uTaskbarRestartMsg {
        UpdateTrayInfo();
        UpdateLayoutList(core::ptr::null_mut());
        AddTrayIcon(hwnd);
        UpdatePenIcon(hwnd, g_iKL);
        return 0 as LRESULT;
    }

    if uMsg == g_uShellHookMessage {
        if wParam == HSHELL_LANGUAGE as WPARAM {
            PostMessage(hwnd, WM_LANG_CHANGED as UINT, 0 as WPARAM, 0 as LPARAM);
        } else if wParam == HSHELL_WINDOWACTIVATED as WPARAM || wParam == HSHELL_RUDEAPPACTIVATED as WPARAM {
            PostMessage(hwnd, WM_WINDOW_ACTIVATE as UINT, 0 as WPARAM, 0 as LPARAM);
        }
        return 0 as LRESULT;
    }

    return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

unsafe extern "C" fn OnIndicatorMsg(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) {
    let hKL: HKL = lParam as HKL;
    let iKL: UINT = GetLayoutIndexFromHKL(hKL);
    if iKL >= g_cKLs {
        return;
    }

    g_iKL = iKL;

    match uMsg {
        INDICM_SETIMEICON => {
            if LOWORD!(wParam) == (MAXWORD as u16) {
                g_anFlags[iKL as usize] =
                    (g_anFlags[iKL as usize] as ::core::ffi::c_int & (!LAYOUTF_IME_ICON) as i32) as BYTE;
                g_aiSysPenIcons[iKL as usize] = MAXWORD as WORD;
            } else {
                g_anFlags[iKL as usize] =
                    (g_anFlags[iKL as usize] as ::core::ffi::c_int | (LAYOUTF_IME_ICON as i32)) as BYTE;
                g_aiSysPenIcons[iKL as usize] = wParam as WORD;
            }
            UpdatePenIcon(hwnd, iKL);
        }

        INDICM_SETIMETOOLTIPS => {
            if LOWORD!(wParam) == (MAXWORD as u16) {
                g_anFlags[iKL as usize] =
                    (g_anFlags[iKL as usize] as ::core::ffi::c_int & (!LAYOUTF_TOOLTIP_ATOM) as i32) as BYTE;
            } else {
                g_anFlags[iKL as usize] =
                    (g_anFlags[iKL as usize] as ::core::ffi::c_int | (LAYOUTF_TOOLTIP_ATOM as i32)) as BYTE;
                g_anToolTipAtoms[iKL as usize] = LOWORD!(wParam);
            }
            UpdatePenIcon(hwnd, iKL);
        }

        INDICM_REMOVEDEFAULTMENUITEMS => {
            if wParam != 0 {
                if (wParam & (RDMI_LEFT as WPARAM)) != 0 {
                    g_anFlags[iKL as usize] = (g_anFlags[iKL as usize] as ::core::ffi::c_int
                        | (LAYOUTF_REMOVE_LEFT_DEF_MENU as i32)) as BYTE;
                }
                if (wParam & (RDMI_RIGHT as WPARAM)) != 0 {
                    g_anFlags[iKL as usize] = (g_anFlags[iKL as usize] as ::core::ffi::c_int
                        | (LAYOUTF_REMOVE_RIGHT_DEF_MENU as i32)) as BYTE;
                }
            } else {
                g_anFlags[iKL as usize] = (g_anFlags[iKL as usize] as ::core::ffi::c_int
                    & (!(LAYOUTF_REMOVE_LEFT_DEF_MENU | LAYOUTF_REMOVE_RIGHT_DEF_MENU)) as i32) as BYTE;
            }
        }

        _ => {
            ERR!("uMsg: %u\n", uMsg);
            return;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn WndProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMsg {
        WM_CREATE => {
            return (KbSwitch_OnCreate(hwnd)) as LRESULT;
        }

        WM_TIMER => {
            KbSwitch_OnTimer(hwnd, wParam as UINT_PTR);
        }

        WM_LANG_CHANGED => {
            /* Comes from indicdll.dll and this module */
            return KbSwitch_OnLangChanged(hwnd, wParam as HWND, lParam as HKL);
        }

        WM_WINDOW_ACTIVATE => {
            /* Comes from indicdll.dll and this module */
            return KbSwitch_OnWindowActivate(hwnd, wParam as HWND, lParam);
        }

        WM_NOTIFYICONMSG => {
            KbSwitch_OnNotifyIconMsg(hwnd, lParam as UINT);
        }

        WM_PENICONMSG => {
            KbSwitch_OnPenIconMsg(hwnd, lParam as UINT);
        }

        WM_COMMAND => {
            KbSwitch_OnCommand(hwnd, LOWORD!(wParam) as UINT);
        }

        WM_SETTINGCHANGE => {
            KbSwitch_OnSettingChange(hwnd, wParam, lParam);
        }

        WM_DESTROY => {
            KbSwitch_OnDestroy(hwnd);
        }

        INDICM_SETIMEICON | INDICM_SETIMETOOLTIPS | INDICM_REMOVEDEFAULTMENUITEMS => {
            if !(InSendMessageEx(core::ptr::null_mut()) != 0) {
                /* Must be a PostMessage call for quick response, not SendMessage */
                OnIndicatorMsg(hwnd, uMsg, wParam, lParam);
            }
        }

        WM_INPUTLANGCHANGEREQUEST => {
            TRACE!("WM_INPUTLANGCHANGEREQUEST(%p, %p)\n", wParam, lParam);
            SetTimer(
                hwnd,
                TIMER_ID_LANG_CHANGED_DELAYED as UINT_PTR,
                TIMER_LANG_CHANGED_DELAY as UINT,
                None,
            );
        }

        _ => {
            return KbSwitch_OnDefault(hwnd, uMsg, wParam, lParam);
        }
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInst: HINSTANCE,
    lpCmdLine: LPTSTR,
    nCmdShow: INT,
) -> INT {
    let mut WndClass: WNDCLASS = core::mem::zeroed();
    let mut msg: MSG = core::mem::zeroed();
    let mut hMutex: HANDLE = core::ptr::null_mut();
    let mut hwnd: HWND = core::ptr::null_mut();

    let __sw = GetUserDefaultUILanguage();
    if __sw == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) {
        TRACE!("LAYOUT_RTL\n");
        SetProcessDefaultLayout(LAYOUT_RTL);
    }

    hMutex = CreateMutex(core::ptr::null_mut(), (FALSE as i32), INDICATOR_CLASS.as_ptr());
    if hMutex.is_null() {
        ERR!("!hMutex\n");
        return 1;
    }

    if GetLastError() == ERROR_ALREADY_EXISTS {
        ERR!("Another instance is already running\n");
        CloseHandle(hMutex);
        return 1;
    }

    g_hInst = hInstance;

    ZeroMemory!(
        core::ptr::addr_of_mut!(WndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASS>(),
    );
    WndClass.lpfnWndProc = Some(WndProc);
    WndClass.hInstance = hInstance;
    WndClass.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
    WndClass.lpszClassName = INDICATOR_CLASS.as_ptr();
    if RegisterClass(core::ptr::addr_of_mut!(WndClass)) == 0 {
        CloseHandle(hMutex);
        return 1;
    }

    hwnd = CreateWindow!(
        INDICATOR_CLASS.as_ptr(),
        core::ptr::null_mut(),
        0,
        0,
        0,
        1,
        1,
        HWND_DESKTOP,
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );
    g_uShellHookMessage = RegisterWindowMessage(RUSTCONV_WSTR_258F57B256D3.as_ptr());
    if RegisterShellHookWindow(hwnd) == 0 {
        ERR!("RegisterShellHookWindow failed\n");
        DestroyWindow(hwnd);
        CloseHandle(hMutex);
        return 1;
    }

    while GetMessage(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0,
    ) != 0
    {
        TranslateMessage(core::ptr::addr_of_mut!(msg));
        DispatchMessage(core::ptr::addr_of_mut!(msg));
    }

    CloseHandle(hMutex);
    return 0;
}
