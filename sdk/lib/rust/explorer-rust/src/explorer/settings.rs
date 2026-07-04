use core::ffi::c_void;
use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use explorer_sys::types::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

// ---- file-scope preamble (translated) ----
pub static mut g_TaskbarSettings: TaskbarSettings = TaskbarSettings {
    sr: ScreenResolution {
        cbSize: 0,
        iX: 0,
        iY: 0,
    },
    Shell: ShellSettings {
        cbSize: 0,
        fShowClock: 0,
        fShowSeconds: 0,
        fWeekdayTimeFormat: 0,
        Margins: TrayMargins {
            cxLeftWidth: 0,
            cxRightWidth: 0,
            cyTopHeight: 0,
            cyBottomHeight: 0,
        },
    },
    AutoHide: APPBARDATA {
        cbSize: 0,
        hWnd: core::ptr::null_mut(),
        uCallbackMessage: 0,
        uEdge: 0,
        rc: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        lParam: 0,
    },
    AlwaysOnTop: APPBARDATA {
        cbSize: 0,
        hWnd: core::ptr::null_mut(),
        uCallbackMessage: 0,
        uEdge: 0,
        rc: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        lParam: 0,
    },
    SmallIcons: 0,
    HideClock: 0,
    Locale: [0; 12],
};

#[no_mangle]
pub unsafe extern "C" fn TaskbarSettings_Save(this: *mut TaskbarSettings) -> i32 {
    SHSetValueW(
        hkExplorer,
        core::ptr::null_mut(),
        w!("EnableAutotray").as_ptr(),
        REG_DWORD,
        core::ptr::addr_of_mut!((*this).bHideInactiveIcons) as *const core::ffi::c_void,
        core::mem::size_of_val(&(*this).bHideInactiveIcons) as u32,
    );
    SHSetValueW(
        hkExplorer,
        w!("Advanced").as_ptr(),
        w!("PreferDateOverWeekday").as_ptr(),
        REG_DWORD,
        core::ptr::addr_of_mut!((*this).bPreferDate) as *const core::ffi::c_void,
        core::mem::size_of_val(&(*this).bPreferDate) as u32,
    );
    SHSetValueW(
        hkExplorer,
        w!("Advanced").as_ptr(),
        w!("ShowSeconds").as_ptr(),
        REG_DWORD,
        core::ptr::addr_of_mut!((*this).bShowSeconds) as *const core::ffi::c_void,
        core::mem::size_of_val(&(*this).bShowSeconds) as u32,
    );
    SHSetValueW(
        hkExplorer,
        w!("Advanced").as_ptr(),
        w!("TaskbarGlomming").as_ptr(),
        REG_DWORD,
        core::ptr::addr_of_mut!((*this).bGroupButtons) as *const core::ffi::c_void,
        core::mem::size_of_val(&(*this).bGroupButtons) as u32,
    );
    let mut bAllowSizeMove = if (*this).bLock != 0 { 0 } else { 1 };
    SHSetValueW(
        hkExplorer,
        w!("Advanced").as_ptr(),
        w!("TaskbarSizeMove").as_ptr(),
        REG_DWORD,
        core::ptr::addr_of_mut!(bAllowSizeMove) as *const core::ffi::c_void,
        core::mem::size_of_val(&bAllowSizeMove) as u32,
    );
    (*this).sr.cbSize = core::mem::size_of::<TW_STRUCKRECTS2>() as u32;
    SHSetValueW(
        hkExplorer,
        w!("Advanced").as_ptr(),
        w!("TaskbarSmallIcons").as_ptr(),
        REG_DWORD,
        core::ptr::addr_of_mut!((*this).bSmallIcons) as *const core::ffi::c_void,
        core::mem::size_of_val(&(*this).bSmallIcons) as u32,
    );
    SHSetValueW(
        hkExplorer,
        w!("Advanced").as_ptr(),
        w!("TaskbarSd").as_ptr(),
        REG_DWORD,
        core::ptr::addr_of_mut!((*this).bShowDesktopButton) as *const core::ffi::c_void,
        core::mem::size_of_val(&(*this).bShowDesktopButton) as u32,
    );
    SHSetValueW(
        hkExplorer,
        w!("StuckRects2").as_ptr(),
        w!("Settings").as_ptr(),
        REG_BINARY,
        core::ptr::addr_of_mut!((*this).sr) as *const core::ffi::c_void,
        core::mem::size_of_val(&(*this).sr) as u32,
    );
    1
}

#[no_mangle]
pub unsafe extern "C" fn TaskbarSettings_Load(this: *mut TaskbarSettings) -> BOOL {
    let mut dwRet: DWORD;
    let mut cbSize: DWORD;
    let mut dwValue: DWORD = 0;

    cbSize = core::mem::size_of::<DWORD>() as DWORD;
    dwRet = SHGetValueW(hkExplorer, w!("Advanced").as_ptr(), w!("TaskbarSizeMove").as_ptr(), core::ptr::null_mut(), &mut dwValue as *mut _ as *mut _, &mut cbSize);
    (*this).bLock = if dwRet == ERROR_SUCCESS { if dwValue == 0 { 1 } else { 0 } } else { 1 };

    dwRet = SHGetValueW(hkExplorer, w!("Advanced").as_ptr(), w!("PreferDateOverWeekday").as_ptr(), core::ptr::null_mut(), &mut dwValue as *mut _ as *mut _, &mut cbSize);
    (*this).bPreferDate = if dwRet == ERROR_SUCCESS { if dwValue != 0 { 1 } else { 0 } } else { 0 };

    dwRet = SHGetValueW(hkExplorer, w!("Advanced").as_ptr(), w!("ShowSeconds").as_ptr(), core::ptr::null_mut(), &mut dwValue as *mut _ as *mut _, &mut cbSize);
    (*this).bShowSeconds = if dwRet == ERROR_SUCCESS { if dwValue != 0 { 1 } else { 0 } } else { 0 };

    dwRet = SHGetValueW(hkExplorer, w!("Advanced").as_ptr(), w!("TaskbarGlomming").as_ptr(), core::ptr::null_mut(), &mut dwValue as *mut _ as *mut _, &mut cbSize);
    (*this).bGroupButtons = if dwRet == ERROR_SUCCESS { if dwValue != 0 { 1 } else { 0 } } else { 0 };

    dwRet = SHGetValueW(hkExplorer, core::ptr::null_mut(), w!("EnableAutotray").as_ptr(), core::ptr::null_mut(), &mut dwValue as *mut _ as *mut _, &mut cbSize);
    (*this).bHideInactiveIcons = if dwRet == ERROR_SUCCESS { if dwValue != 0 { 1 } else { 0 } } else { 0 };

    dwRet = SHGetValueW(hkExplorer, w!("Advanced").as_ptr(), w!("TaskbarSmallIcons").as_ptr(), core::ptr::null_mut(), &mut dwValue as *mut _ as *mut _, &mut cbSize);
    (*this).bSmallIcons = if dwRet == ERROR_SUCCESS { if dwValue != 0 { 1 } else { 0 } } else { 1 };

    dwRet = SHGetValueW(hkExplorer, w!("Advanced").as_ptr(), w!("CompactTrayIcons").as_ptr(), core::ptr::null_mut(), &mut dwValue as *mut _ as *mut _, &mut cbSize);
    if dwRet == ERROR_SUCCESS && dwValue <= TIM_Max {
        (*this).eCompactTrayIcons = dwValue as TrayIconsMode;
    } else {
        (*this).eCompactTrayIcons = TIM_Default;
    }

    dwRet = SHGetValueW(hkExplorer, w!("Advanced").as_ptr(), w!("TaskbarSd").as_ptr(), core::ptr::null_mut(), &mut dwValue as *mut _ as *mut _, &mut cbSize);
    (*this).bShowDesktopButton = if dwRet == ERROR_SUCCESS { if dwValue != 0 { 1 } else { 0 } } else { 1 };

    cbSize = core::mem::size_of::<TW_STRUCKRECTS2>() as DWORD;
    dwRet = SHGetValueW(hkExplorer, w!("StuckRects2").as_ptr(), w!("Settings").as_ptr(), core::ptr::null_mut(), &mut (*this).sr as *mut _ as *mut _, &mut cbSize);

    if dwRet != ERROR_SUCCESS || (*this).sr.cbSize != core::mem::size_of::<TW_STRUCKRECTS2>() as DWORD || cbSize != core::mem::size_of::<TW_STRUCKRECTS2>() as DWORD {
        (*this).sr.Position = ABE_BOTTOM;
        (*this).sr.AutoHide = 0;
        (*this).sr.AlwaysOnTop = 1;
        (*this).sr.SmSmallIcons = 0;
        (*this).sr.HideClock = 0;
        (*this).sr.Rect.left = 0;
        (*this).sr.Rect.top = 0;
        (*this).sr.Rect.bottom = 1;
        (*this).sr.Rect.right = 1;
        (*this).sr.Size.cx = 0;
        (*this).sr.Size.cy = 0;
    } else {
        if (*this).sr.Position > ABE_BOTTOM {
            (*this).sr.Position = ABE_BOTTOM;
        }
    }

    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Explorer
//  *
//  * Copyright 2013 - Edijs Kolesnikovics
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
//
// #include "precomp.h"
//
// TaskbarSettings g_TaskbarSettings;
//
// BOOL TaskbarSettings::Save()
// {
//     SHSetValueW(hkExplorer, NULL, L"EnableAutotray", REG_DWORD, &bHideInactiveIcons, sizeof(bHideInactiveIcons));
//     SHSetValueW(hkExplorer, L"Advanced", L"PreferDateOverWeekday", REG_DWORD, &bPreferDate, sizeof(bPreferDate));
//     SHSetValueW(hkExplorer, L"Advanced", L"ShowSeconds", REG_DWORD, &bShowSeconds, sizeof(bShowSeconds));
//     SHSetValueW(hkExplorer, L"Advanced", L"TaskbarGlomming", REG_DWORD, &bGroupButtons, sizeof(bGroupButtons));
//     BOOL bAllowSizeMove = !bLock;
//     SHSetValueW(hkExplorer, L"Advanced", L"TaskbarSizeMove", REG_DWORD, &bAllowSizeMove, sizeof(bAllowSizeMove));
//     sr.cbSize = sizeof(sr);
//     SHSetValueW(hkExplorer, L"Advanced", L"TaskbarSmallIcons", REG_DWORD, &bSmallIcons, sizeof(bSmallIcons));
//     SHSetValueW(hkExplorer, L"Advanced", L"TaskbarSd", REG_DWORD, &bShowDesktopButton, sizeof(bShowDesktopButton));
//     SHSetValueW(hkExplorer, L"StuckRects2", L"Settings", REG_BINARY, &sr, sizeof(sr));
//
//     /* TODO: AutoHide writes something to HKEY_CURRENT_USER\Software\Microsoft\Internet Explorer\Desktop\Components\0 figure out what and why */
//     return TRUE;
// }
//
// BOOL TaskbarSettings::Load()
// {
//     DWORD dwRet, cbSize, dwValue = NULL;
//
//     cbSize = sizeof(dwValue);
//     dwRet = SHGetValueW(hkExplorer, L"Advanced", L"TaskbarSizeMove", NULL, &dwValue, &cbSize);
//     bLock = (dwRet == ERROR_SUCCESS) ? (dwValue == 0) : TRUE;
//
//     dwRet = SHGetValueW(hkExplorer, L"Advanced", L"PreferDateOverWeekday", NULL, &dwValue, &cbSize);
//     bPreferDate = (dwRet == ERROR_SUCCESS) ? (dwValue != 0) : FALSE; /* This is opt-in setting */
//
//     dwRet = SHGetValueW(hkExplorer, L"Advanced", L"ShowSeconds", NULL, &dwValue, &cbSize);
//     bShowSeconds = (dwRet == ERROR_SUCCESS) ? (dwValue != 0) : FALSE;
//
//     dwRet = SHGetValueW(hkExplorer, L"Advanced", L"TaskbarGlomming", NULL, &dwValue, &cbSize);
//     bGroupButtons = (dwRet == ERROR_SUCCESS) ? (dwValue != 0) : FALSE;
//
//     dwRet = SHGetValueW(hkExplorer, NULL, L"EnableAutotray", NULL, &dwValue, &cbSize);
//     bHideInactiveIcons = (dwRet == ERROR_SUCCESS) ? (dwValue != 0) : FALSE;
//
//     dwRet = SHGetValueW(hkExplorer, L"Advanced", L"TaskbarSmallIcons", NULL, &dwValue, &cbSize);
//     bSmallIcons = (dwRet == ERROR_SUCCESS) ? (dwValue != 0) : TRUE;
//
//     dwRet = SHGetValueW(hkExplorer, L"Advanced", L"CompactTrayIcons", NULL, &dwValue, &cbSize);
//     if (dwRet == ERROR_SUCCESS && dwValue <= TIM_Max)
//         eCompactTrayIcons = static_cast<TrayIconsMode>(dwValue);
//     else
//         eCompactTrayIcons = TIM_Default;
//
//     dwRet = SHGetValueW(hkExplorer, L"Advanced", L"TaskbarSd", NULL, &dwValue, &cbSize);
//     bShowDesktopButton = (dwRet == ERROR_SUCCESS) ? (dwValue != 0) : TRUE;
//
//     cbSize = sizeof(sr);
//     dwRet = SHGetValueW(hkExplorer, L"StuckRects2", L"Settings", NULL, &sr, &cbSize);
//
//     /* Make sure we have correct values here */
//     if (dwRet != ERROR_SUCCESS || sr.cbSize != sizeof(sr) || cbSize != sizeof(sr))
//     {
//         sr.Position = ABE_BOTTOM;
//         sr.AutoHide = FALSE;
//         sr.AlwaysOnTop = TRUE;
//         sr.SmSmallIcons = FALSE;
//         sr.HideClock = FALSE;
//         sr.Rect.left = sr.Rect.top = 0;
//         sr.Rect.bottom = sr.Rect.right = 1;
//         sr.Size.cx = sr.Size.cy = 0;
//     }
//     else
//     {
//         if (sr.Position > ABE_BOTTOM)
//             sr.Position = ABE_BOTTOM;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
