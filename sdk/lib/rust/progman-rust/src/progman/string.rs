use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut szTitle: [WCHAR; 256] = [0; 256];
#[no_mangle]
pub unsafe extern "C" fn STRING_LoadStrings() {
    LoadStringW(Globals.hInstance, IDS_PROGRAM_MANAGER as UINT, core::ptr::addr_of_mut!(szTitle) as LPWSTR, ARRAYSIZE!(szTitle) as INT);
}
#[no_mangle]
pub unsafe extern "C" fn STRING_LoadMenus() {
    let mut hMainMenu: HMENU = core::ptr::null_mut();

    /* Create the menu */
    hMainMenu = LoadMenuW(Globals.hInstance, MAKEINTRESOURCEW!(MAIN_MENU) as LPCWSTR);
    Globals.hFileMenu = GetSubMenu(hMainMenu, 0 as ::core::ffi::c_int);
    Globals.hOptionMenu = GetSubMenu(hMainMenu, 1 as ::core::ffi::c_int);
    Globals.hWindowsMenu = GetSubMenu(hMainMenu, 2 as ::core::ffi::c_int);
    Globals.hLanguageMenu = GetSubMenu(hMainMenu, 3 as ::core::ffi::c_int);

    if !Globals.hMDIWnd.is_null() {
        SendMessageW(
            Globals.hMDIWnd,
            WM_MDISETMENU as UINT,
            hMainMenu as WPARAM,
            Globals.hWindowsMenu as LPARAM,
        );
    } else {
        SetMenu(Globals.hMainWnd, hMainMenu);
    }

    /* Destroy the old menu */
    if !Globals.hMainMenu.is_null() {
        DestroyMenu(Globals.hMainMenu);
    }
    Globals.hMainMenu = hMainMenu;
}
