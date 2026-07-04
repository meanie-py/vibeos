use winsrv_sys::nt::*;
use core::ffi::c_void;

extern "system" {
    fn SetMenuItemInfoW(hMenu: HMENU, item: u32, fByPosition: i32, lpmii: *const MENUITEMINFOW) -> i32;
    fn SHRegGetBoolUSValueW( pszSubKey: LPCWSTR, pszValueName: LPCWSTR, fIgnoreHKCU: i32, fDefault: i32, ) -> i32;
    fn SHRegSetUSValueW( pszSubKey: LPCWSTR, pszValueName: LPCWSTR, dwType: u32, pvData: LPVOID, cbData: u32, dwFlags: u32, ) -> i32;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LANGCODEPAGE {
    pub wLanguage: u16,
    pub wCodePage: u16,
}
pub type PLANGCODEPAGE = *mut _LANGCODEPAGE;

pub const REGKEY_ADVANCED: &[u16] = unsafe {
    &[
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16,
        b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16,
        b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16,
        b'w' as u16, b's' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
        b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16,
        b'n' as u16, b'\\' as u16, b'E' as u16, b'x' as u16, b'p' as u16, b'l' as u16, b'o' as u16, b'r' as u16,
        b'e' as u16, b'r' as u16, b'\\' as u16, b'A' as u16, b'd' as u16, b'v' as u16, b'a' as u16, b'n' as u16,
        b'c' as u16, b'e' as u16, b'd' as u16, 0u16,
    ]
};
#[no_mangle]
pub unsafe extern "C" fn IsSameObject(punk1: *mut IUnknown, punk2: *mut IUnknown) -> HRESULT {
    let mut punk1_local = punk1;
    let mut punk2_local = punk2;
    let mut hRet: HRESULT;

    hRet = (*punk1_local).QueryInterface(
        &IID_IUnknown,
        &mut punk1_local as *mut *mut IUnknown as *mut LPVOID
    );
    if SUCCEEDED(hRet) == 0 {
        return hRet;
    }

    hRet = (*punk2_local).QueryInterface(
        &IID_IUnknown,
        &mut punk2_local as *mut *mut IUnknown as *mut LPVOID
    );

    (*punk1_local).Release();

    if SUCCEEDED(hRet) == 0 {
        return hRet;
    }

    (*punk2_local).Release();

    if punk1_local == punk2_local { S_OK } else { S_FALSE }
}
#[no_mangle]
pub unsafe extern "C" fn LoadPopupMenu(
    hInstance: HINSTANCE,
    lpMenuName: LPCWSTR,
) -> HMENU {
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
pub unsafe extern "C" fn FindSubMenu(
    hMenu: HMENU,
    uItem: u32,
    fByPosition: i32,
) -> HMENU {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();

    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_SUBMENU;

    if GetMenuItemInfoW(hMenu, uItem, fByPosition, &mut mii) != 0 {
        return mii.hSubMenu;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn GetCurrentLoggedOnUserName(szBuffer: LPWSTR, dwBufferSize: u32) -> u8 {
    let mut dwType: u32 = 0;
    let mut dwSize: u32;

    dwSize = (dwBufferSize * core::mem::size_of::<u16>() as u32) - 1;
    if RegQueryValueExW(
        hkExplorer,
        b"L\0o\0g\0o\0n\0 \0U\0s\0e\0r\0 \0N\0a\0m\0e\0\0\0".as_ptr() as LPCWSTR,
        0,
        &mut dwType,
        szBuffer as LPBYTE,
        &mut dwSize,
    ) == ERROR_SUCCESS
        && (dwSize / core::mem::size_of::<u16>() as u32) > 1
        && *szBuffer != 0
    {
        *szBuffer.add((dwSize / core::mem::size_of::<u16>() as u32) as usize) = 0;
        return 1u8;
    }

    dwSize = dwBufferSize;
    if GetUserNameW(szBuffer, &mut dwSize) == 0 {
        *szBuffer = 0;
        return 0u8;
    }

    1u8
}
#[no_mangle]
pub unsafe extern "C" fn FormatMenuString(
    hMenu: HMENU,
    uPosition: u32,
    uFlags: u32,
    mut args: ...
) -> i32 {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut szBuf: [u16; 128] = [0; 128];
    let mut szBufFmt: [u16; 128] = [0; 128];

    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_STRING;
    mii.dwTypeData = szBufFmt.as_mut_ptr();
    mii.cch = 128;
    
    if GetMenuItemInfoW(hMenu, uPosition, uFlags, &mut mii) != 0 {
        libc::vswprintf(
            szBuf.as_mut_ptr(),
            127,
            szBufFmt.as_ptr(),
            args.as_va_list()
        );
        szBuf[127] = 0;

        mii.dwTypeData = szBuf.as_mut_ptr();
        if SetMenuItemInfoW(hMenu, uPosition, uFlags as i32, &mii) != 0 {
            return 1;
        }
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn GetRegBool(
    pszSubKey: LPCWSTR,
    pszValueName: LPCWSTR,
    bDefaultValue: i32,
) -> i32 {
    
    SHRegGetBoolUSValueW(pszSubKey, pszValueName, 0, bDefaultValue)
}
#[no_mangle]
pub unsafe extern "C" fn SetRegDword(pszSubKey: LPCWSTR, pszValueName: LPCWSTR, dwValue: u32) -> u8 {
    
    const SHREGSET_FORCE_HKCU: u32 = 0x00000001;
    
    (SHRegSetUSValueW(
        pszSubKey,
        pszValueName,
        REG_DWORD,
        &dwValue as *const u32 as LPVOID,
        core::mem::size_of::<u32>() as u32,
        SHREGSET_FORCE_HKCU,
    ) == ERROR_SUCCESS as i32) as u8
}
#[no_mangle]
pub unsafe extern "C" fn GetAdvancedBool(pszValueName: LPCWSTR, bDefaultValue: i32) -> i32 {
    GetRegBool(b"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced\0".as_ptr() as LPCWSTR, pszValueName, bDefaultValue)
}
#[no_mangle]
pub unsafe extern "C" fn SetAdvancedDword(pszValueName: LPCWSTR, dwValue: u32) -> u8 {
    SetRegDword(REGKEY_ADVANCED.as_ptr(), pszValueName, dwValue)
}
#[no_mangle]
pub unsafe extern "C" fn GetVersionInfoString(
    szFileName: LPCWSTR,
    szVersionInfo: LPCWSTR,
    szBuffer: LPWSTR,
    cbBufLen: u32,
) -> u8 {
    let mut lpData: LPVOID = core::ptr::null_mut();
    let mut szSubBlock: [u16; 128] = [0; 128];
    let mut lpszLocalBuf: *mut u16 = core::ptr::null_mut();
    let UserLangId: LANGID;
    let mut lpTranslate: *mut LANGCODEPAGE = core::ptr::null_mut();
    let dwLen: u32;
    let mut dwHandle: u32 = 0;
    let mut cbTranslate: u32 = 0;
    let mut cbLen: u32 = 0;
    let mut bRet: u8 = 0;
    let mut i: u32;

    dwLen = GetFileVersionInfoSizeW(szFileName, &mut dwHandle);

    if dwLen > 0 {
        lpData = HeapAlloc(hProcessHeap, 0, dwLen as usize);

        if !lpData.is_null() {
            if GetFileVersionInfoW(szFileName, 0, dwLen, lpData) != 0 {
                UserLangId = GetUserDefaultLangID();

                VerQueryValueW(
                    lpData,
                    b"\\VarFileInfo\\Translation\0".as_ptr() as LPCWSTR,
                    &mut lpTranslate as *mut *mut LANGCODEPAGE as *mut LPVOID,
                    &mut cbTranslate,
                );

                i = 0;
                while i < cbTranslate / core::mem::size_of::<LANGCODEPAGE>() as u32 {
                    if LOBYTE((*lpTranslate.offset(i as isize)).wLanguage) == LOBYTE(UserLangId) {
                        wnsprintf(
                            szSubBlock.as_mut_ptr(),
                            128,
                            b"\\StringFileInfo\\%04X%04X\\%s\0".as_ptr() as LPCWSTR,
                            (*lpTranslate.offset(i as isize)).wLanguage,
                            (*lpTranslate.offset(i as isize)).wCodePage,
                            szVersionInfo,
                        );

                        if VerQueryValueW(
                            lpData,
                            szSubBlock.as_ptr(),
                            &mut lpszLocalBuf as *mut *mut u16 as *mut LPVOID,
                            &mut cbLen,
                        ) != 0
                        {
                            wcsncpy(szBuffer, lpszLocalBuf, cbBufLen / 2);

                            bRet = 1;
                            break;
                        }
                    }
                    i += 1;
                }
            }

            HeapFree(hProcessHeap, 0, lpData);
            lpData = core::ptr::null_mut();
        }
    }

    bRet
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
// #include <winver.h>
//
// typedef struct _LANGCODEPAGE
// {
//     WORD wLanguage;
//     WORD wCodePage;
// } LANGCODEPAGE, *PLANGCODEPAGE;
//
// HRESULT
// IsSameObject(IN IUnknown *punk1, IN IUnknown *punk2)
// {
//     HRESULT hRet;
//
//     hRet = punk1->QueryInterface(IID_PPV_ARG(IUnknown, &punk1));
//     if (!SUCCEEDED(hRet))
//         return hRet;
//
//     hRet = punk2->QueryInterface(IID_PPV_ARG(IUnknown, &punk2));
//
//     punk1->Release();
//
//     if (!SUCCEEDED(hRet))
//         return hRet;
//
//     punk2->Release();
//
//     /* We're dealing with the same object if the IUnknown pointers are equal */
//     return (punk1 == punk2) ? S_OK : S_FALSE;
// }
//
// HMENU
// LoadPopupMenu(IN HINSTANCE hInstance,
//               IN LPCWSTR lpMenuName)
// {
//     HMENU hMenu, hSubMenu = NULL;
//
//     hMenu = LoadMenuW(hInstance, lpMenuName);
//     if (hMenu != NULL)
//     {
//         hSubMenu = GetSubMenu(hMenu, 0);
//         if ((hSubMenu != NULL) &&
//             !RemoveMenu(hMenu, 0, MF_BYPOSITION))
//         {
//             hSubMenu = NULL;
//         }
//
//         DestroyMenu(hMenu);
//     }
//
//     return hSubMenu;
// }
//
// HMENU
// FindSubMenu(IN HMENU hMenu,
//             IN UINT uItem,
//             IN BOOL fByPosition)
// {
//     MENUITEMINFOW mii;
//
//     mii.cbSize = sizeof(mii);
//     mii.fMask = MIIM_SUBMENU;
//
//     if (GetMenuItemInfoW(hMenu, uItem, fByPosition, &mii))
//     {
//         return mii.hSubMenu;
//     }
//
//     return NULL;
// }
//
// BOOL
// GetCurrentLoggedOnUserName(OUT LPWSTR szBuffer,
//                            IN DWORD dwBufferSize)
// {
//     DWORD dwType;
//     DWORD dwSize;
//
//     /* Query the user name from the registry */
//     dwSize = (dwBufferSize * sizeof(WCHAR)) - 1;
//     if (RegQueryValueExW(hkExplorer,
//                          L"Logon User Name",
//                          0,
//                          &dwType,
//                          (LPBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS &&
//         (dwSize / sizeof(WCHAR)) > 1 &&
//         szBuffer[0] != L'\0')
//     {
//         szBuffer[dwSize / sizeof(WCHAR)] = L'\0';
//         return TRUE;
//     }
//
//     /* Fall back to GetUserName() */
//     dwSize = dwBufferSize;
//     if (!GetUserNameW(szBuffer, &dwSize))
//     {
//         szBuffer[0] = L'\0';
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// BOOL
// FormatMenuString(IN HMENU hMenu,
//                  IN UINT uPosition,
//                  IN UINT uFlags,
//                  ...)
// {
//     va_list vl;
//     MENUITEMINFOW mii;
//     WCHAR szBuf[128];
//     WCHAR szBufFmt[128];
//
//     /* Find the menu item and read the formatting string */
//     mii.cbSize = sizeof(mii);
//     mii.fMask = MIIM_STRING;
//     mii.dwTypeData = szBufFmt;
//     mii.cch = _countof(szBufFmt);
//     if (GetMenuItemInfoW(hMenu, uPosition, uFlags, &mii))
//     {
//         /* Format the string */
//         va_start(vl, uFlags);
//         _vsntprintf(szBuf,
//                     _countof(szBuf) - 1,
//                     szBufFmt,
//                     vl);
//         va_end(vl);
//         szBuf[_countof(szBuf) - 1] = L'\0';
//
//         /* Update the menu item */
//         mii.dwTypeData = szBuf;
//         if (SetMenuItemInfo(hMenu, uPosition, uFlags, &mii))
//         {
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// BOOL GetRegBool(IN LPCWSTR pszSubKey, IN LPCWSTR pszValueName, IN BOOL bDefaultValue)
// {
//     return SHRegGetBoolUSValueW(pszSubKey, pszValueName, FALSE, bDefaultValue);
// }
//
// BOOL SetRegDword(IN LPCWSTR pszSubKey, IN LPCWSTR pszValueName, IN DWORD dwValue)
// {
//     return (SHRegSetUSValueW(pszSubKey, pszValueName, REG_DWORD, &dwValue,
//                              sizeof(dwValue), SHREGSET_FORCE_HKCU) == ERROR_SUCCESS);
// }
//
// #define REGKEY_ADVANCED L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced"
//
// BOOL GetAdvancedBool(IN LPCWSTR pszValueName, IN BOOL bDefaultValue)
// {
//     return GetRegBool(REGKEY_ADVANCED, pszValueName, bDefaultValue);
// }
//
// BOOL SetAdvancedDword(IN LPCWSTR pszValueName, IN DWORD dwValue)
// {
//     return SetRegDword(REGKEY_ADVANCED, pszValueName, dwValue);
// }
//
// BOOL
// GetVersionInfoString(IN LPCWSTR szFileName,
//                      IN LPCWSTR szVersionInfo,
//                      OUT LPWSTR szBuffer,
//                      IN UINT cbBufLen)
// {
//     LPVOID lpData = NULL;
//     WCHAR szSubBlock[128];
//     WCHAR *lpszLocalBuf = NULL;
//     LANGID UserLangId;
//     PLANGCODEPAGE lpTranslate = NULL;
//     DWORD dwLen;
//     DWORD dwHandle;
//     UINT cbTranslate;
//     UINT cbLen;
//     BOOL bRet = FALSE;
//     UINT i;
//
//     dwLen = GetFileVersionInfoSizeW(szFileName, &dwHandle);
//
//     if (dwLen > 0)
//     {
//         lpData = HeapAlloc(hProcessHeap, 0, dwLen);
//
//         if (lpData != NULL)
//         {
//             if (GetFileVersionInfoW(szFileName,
//                                     0,
//                                     dwLen,
//                                     lpData) != 0)
//             {
//                 UserLangId = GetUserDefaultLangID();
//
//                 VerQueryValueW(lpData,
//                                L"\\VarFileInfo\\Translation",
//                                (LPVOID*)&lpTranslate,
//                                &cbTranslate);
//
//                 for (i = 0; i < cbTranslate / sizeof(LANGCODEPAGE); i++)
//                 {
//                     /* If the bottom eight bits of the language id's
//                     match, use this version information (since this
//                     means that the version information and the users
//                     default language are the same). */
//                     if (LOBYTE(lpTranslate[i].wLanguage) == LOBYTE(UserLangId))
//                     {
//                         wnsprintf(szSubBlock,
//                                   _countof(szSubBlock),
//                                   L"\\StringFileInfo\\%04X%04X\\%s",
//                                   lpTranslate[i].wLanguage,
//                                   lpTranslate[i].wCodePage,
//                                   szVersionInfo);
//
//                         if (VerQueryValueW(lpData,
//                                            szSubBlock,
//                                            (LPVOID*)&lpszLocalBuf,
//                                            &cbLen) != 0)
//                         {
//                             wcsncpy(szBuffer, lpszLocalBuf, cbBufLen / sizeof(*szBuffer));
//
//                             bRet = TRUE;
//                             break;
//                         }
//                     }
//                 }
//             }
//
//             HeapFree(hProcessHeap, 0, lpData);
//             lpData = NULL;
//         }
//     }
//
//     return bRet;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
