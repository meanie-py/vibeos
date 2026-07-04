//! util.cpp → Rust. Menu/registry/version-info helpers; first cohab swap.
//!
//! Integration deltas vs the raw translation (.responses/20260703-234423-util.cpp):
//! - BOOL returns are i32 (C `BOOL` is int; u8 would leave garbage in the
//!   caller-visible register bits on i386/amd64).
//! - Wide literals via `winsrv_sys::w!` (the raw output cast NARROW byte
//!   strings to LPCWSTR in GetAdvancedBool/GetVersionInfoString — silent
//!   encoding bug).
//! - COM calls go through explorer_sys::com::IUnknown (lpVtbl-first).
//! - FormatMenuString is C-variadic; stable Rust can't define those, so the
//!   `va_start` shell stays in C++ (util_vararg.cpp) and forwards to
//!   FormatMenuStringV here (same pattern as rtl-diag's DbgPrint).

use core::ffi::c_void;
use explorer_sys::com::{IUnknown, IID_IUnknown, S_FALSE, S_OK};
use explorer_sys::externs::*;
use explorer_sys::globals::{hkExplorer, hProcessHeap};
use explorer_sys::LOBYTE;
use winsrv_sys::consts::SUCCEEDED;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

#[repr(C)]
pub struct LANGCODEPAGE {
    pub wLanguage: u16,
    pub wCodePage: u16,
}

pub static REGKEY_ADVANCED: &[u16] =
    w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced");

#[no_mangle]
pub unsafe extern "C" fn IsSameObject(punk1: *mut IUnknown, punk2: *mut IUnknown) -> HRESULT {
    let mut punk1_local = punk1;
    let mut punk2_local = punk2;
    let mut hRet: HRESULT;

    hRet = (*punk1_local).QueryInterface(
        &IID_IUnknown,
        &mut punk1_local as *mut *mut IUnknown as *mut *mut c_void,
    );
    if !SUCCEEDED(hRet) {
        return hRet;
    }

    hRet = (*punk2_local).QueryInterface(
        &IID_IUnknown,
        &mut punk2_local as *mut *mut IUnknown as *mut *mut c_void,
    );

    (*punk1_local).Release();

    if !SUCCEEDED(hRet) {
        return hRet;
    }

    (*punk2_local).Release();

    /* We're dealing with the same object if the IUnknown pointers are equal */
    if punk1_local == punk2_local {
        S_OK
    } else {
        S_FALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn LoadPopupMenu(hInstance: HINSTANCE, lpMenuName: LPCWSTR) -> HMENU {
    let mut hSubMenu: HMENU = core::ptr::null_mut();

    let hMenu = LoadMenuW(hInstance, lpMenuName);
    if !hMenu.is_null() {
        hSubMenu = GetSubMenu(hMenu, 0);
        if !hSubMenu.is_null() && RemoveMenu(hMenu, 0, MF_BYPOSITION) == 0 {
            hSubMenu = core::ptr::null_mut();
        }

        DestroyMenu(hMenu);
    }

    hSubMenu
}

#[no_mangle]
pub unsafe extern "C" fn FindSubMenu(hMenu: HMENU, uItem: u32, fByPosition: BOOL) -> HMENU {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();

    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_SUBMENU;

    if GetMenuItemInfoW(hMenu, uItem, fByPosition, &mut mii) != 0 {
        return mii.hSubMenu;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn GetCurrentLoggedOnUserName(szBuffer: LPWSTR, dwBufferSize: u32) -> BOOL {
    let mut dwType: u32 = 0;
    let mut dwSize: u32;

    /* Query the user name from the registry */
    dwSize = (dwBufferSize * core::mem::size_of::<u16>() as u32) - 1;
    if RegQueryValueExW(
        hkExplorer,
        w!("Logon User Name").as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        szBuffer as *mut u8,
        &mut dwSize,
    ) == ERROR_SUCCESS as i32
        && (dwSize / core::mem::size_of::<u16>() as u32) > 1
        && *szBuffer != 0
    {
        *szBuffer.add((dwSize / core::mem::size_of::<u16>() as u32) as usize) = 0;
        return 1;
    }

    /* Fall back to GetUserName() */
    dwSize = dwBufferSize;
    if GetUserNameW(szBuffer, &mut dwSize) == 0 {
        *szBuffer = 0;
        return 0;
    }

    1
}

/// Rust worker for the C-variadic FormatMenuString (shell in util_vararg.cpp
/// does `va_start` and forwards; see module doc).
#[no_mangle]
pub unsafe extern "C" fn FormatMenuStringV(
    hMenu: HMENU,
    uPosition: u32,
    uFlags: u32,
    vl: va_list,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut szBuf: [u16; 128] = [0; 128];
    let mut szBufFmt: [u16; 128] = [0; 128];

    /* Find the menu item and read the formatting string */
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_STRING;
    mii.dwTypeData = szBufFmt.as_mut_ptr();
    mii.cch = 128;

    if GetMenuItemInfoW(hMenu, uPosition, uFlags as BOOL, &mut mii) != 0 {
        /* Format the string */
        _vsnwprintf(szBuf.as_mut_ptr(), 128 - 1, szBufFmt.as_ptr(), vl);
        szBuf[128 - 1] = 0;

        /* Update the menu item */
        mii.dwTypeData = szBuf.as_mut_ptr();
        if SetMenuItemInfoW(hMenu, uPosition, uFlags as BOOL, &mii) != 0 {
            return 1;
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn GetRegBool(
    pszSubKey: LPCWSTR,
    pszValueName: LPCWSTR,
    bDefaultValue: BOOL,
) -> BOOL {
    SHRegGetBoolUSValueW(pszSubKey, pszValueName, 0, bDefaultValue)
}

#[no_mangle]
pub unsafe extern "C" fn SetRegDword(
    pszSubKey: LPCWSTR,
    pszValueName: LPCWSTR,
    dwValue: u32,
) -> BOOL {
    (SHRegSetUSValueW(
        pszSubKey,
        pszValueName,
        REG_DWORD,
        &dwValue as *const u32 as *const c_void,
        core::mem::size_of::<u32>() as u32,
        SHREGSET_FORCE_HKCU,
    ) == ERROR_SUCCESS as i32) as BOOL
}

#[no_mangle]
pub unsafe extern "C" fn GetAdvancedBool(pszValueName: LPCWSTR, bDefaultValue: BOOL) -> BOOL {
    GetRegBool(REGKEY_ADVANCED.as_ptr(), pszValueName, bDefaultValue)
}

#[no_mangle]
pub unsafe extern "C" fn SetAdvancedDword(pszValueName: LPCWSTR, dwValue: u32) -> BOOL {
    SetRegDword(REGKEY_ADVANCED.as_ptr(), pszValueName, dwValue)
}

#[no_mangle]
pub unsafe extern "C" fn GetVersionInfoString(
    szFileName: LPCWSTR,
    szVersionInfo: LPCWSTR,
    szBuffer: LPWSTR,
    cbBufLen: u32,
) -> BOOL {
    let mut szSubBlock: [u16; 128] = [0; 128];
    let mut lpszLocalBuf: *mut u16 = core::ptr::null_mut();
    let mut lpTranslate: *mut LANGCODEPAGE = core::ptr::null_mut();
    let mut dwHandle: u32 = 0;
    let mut cbTranslate: u32 = 0;
    let mut cbLen: u32 = 0;
    let mut bRet: BOOL = 0;

    let dwLen = GetFileVersionInfoSizeW(szFileName, &mut dwHandle);

    if dwLen > 0 {
        let lpData = HeapAlloc(hProcessHeap, 0, dwLen as SIZE_T);

        if !lpData.is_null() {
            if GetFileVersionInfoW(szFileName, 0, dwLen, lpData) != 0 {
                let UserLangId: LANGID = GetUserDefaultLangID();

                VerQueryValueW(
                    lpData,
                    w!("\\VarFileInfo\\Translation").as_ptr(),
                    &mut lpTranslate as *mut *mut LANGCODEPAGE as *mut *mut c_void,
                    &mut cbTranslate,
                );

                let mut i: u32 = 0;
                while i < cbTranslate / core::mem::size_of::<LANGCODEPAGE>() as u32 {
                    /* If the bottom eight bits of the language ids match, use
                    this version information (version info and the user's
                    default language agree). */
                    if LOBYTE((*lpTranslate.add(i as usize)).wLanguage) == LOBYTE(UserLangId) {
                        wnsprintfW(
                            szSubBlock.as_mut_ptr(),
                            128,
                            w!("\\StringFileInfo\\%04X%04X\\%s").as_ptr(),
                            (*lpTranslate.add(i as usize)).wLanguage as u32,
                            (*lpTranslate.add(i as usize)).wCodePage as u32,
                            szVersionInfo,
                        );

                        if VerQueryValueW(
                            lpData,
                            szSubBlock.as_ptr(),
                            &mut lpszLocalBuf as *mut *mut u16 as *mut *mut c_void,
                            &mut cbLen,
                        ) != 0
                        {
                            wcsncpy(szBuffer, lpszLocalBuf, (cbBufLen / 2) as usize);

                            bRet = 1;
                            break;
                        }
                    }
                    i += 1;
                }
            }

            HeapFree(hProcessHeap, 0, lpData);
        }
    }

    bRet
}
