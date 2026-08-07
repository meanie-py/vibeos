// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const CCH_LAYOUT_ID: usize = 8;
pub const CCH_ULONG_DEC: usize = 10;
unsafe extern "C" fn IsLayoutExists(szLayoutID: PWSTR, szLangID: PWSTR) -> BOOLEAN {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    const SIZE_PRELOAD: usize = (CCH_LAYOUT_ID + 1) as usize;
    const SIZE_LAYOUTNUM: usize = (3 + 1) as usize;
    const SIZE_TMP: usize = (CCH_LAYOUT_ID + 1) as usize;
    const SIZE_OLDLANGID: usize = (CCH_LAYOUT_ID + 1) as usize;
    let mut szPreload: [u16; SIZE_PRELOAD] = [0u16; SIZE_PRELOAD];
    let mut szLayoutNum: [u16; SIZE_LAYOUTNUM] = [0u16; SIZE_LAYOUTNUM];
    let mut szTmp: [u16; SIZE_TMP] = [0u16; SIZE_TMP];
    let mut szOldLangID: [u16; SIZE_OLDLANGID] = [0u16; SIZE_OLDLANGID];
    let mut dwIndex: u32 = 0;
    let mut dwType: u32;
    let mut dwSize: u32;
    let mut IsLangExists: BOOLEAN = FALSE;
    let mut langid: LANGID;

    let layout_key_path: &[u8] = b"K\0e\0y\0b\0o\0a\0r\0d\0 \0L\0a\0y\0o\0u\0t\0\\\0P\0r\0e\0l\0o\0a\0d\0\0\0";
    let format_str: &[u8] = b"0\00\00\00\00\0%\0s\0\0\0";
    let substitutes_key_path: &[u8] = b"K\0e\0y\0b\0o\0a\0r\0d\0 \0L\0a\0y\0o\0u\0t\0\\\0S\0u\0b\0s\0t\0i\0t\0u\0t\0e\0s\0\0\0";

    if RegOpenKeyExW(HKEY_CURRENT_USER, layout_key_path.as_ptr() as PWSTR,
        0, KEY_QUERY_VALUE, &mut hKey as *mut HKEY) == ERROR_SUCCESS {
        dwSize = (szLayoutNum.len() * core::mem::size_of::<u16>()) as u32;
        while RegEnumValueW(hKey, dwIndex, szLayoutNum.as_mut_ptr(), &mut dwSize as *mut u32,
            core::ptr::null_mut(), &mut dwType as *mut u32, core::ptr::null_mut(), core::ptr::null_mut()) == ERROR_SUCCESS {
            dwSize = (szPreload.len() * core::mem::size_of::<u16>()) as u32;
            if RegQueryValueExW(hKey, szLayoutNum.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(),
                szPreload.as_mut_ptr() as LPBYTE, &mut dwSize as *mut u32) != ERROR_SUCCESS {
                RegCloseKey(hKey);
                return FALSE;
            }
            langid = wcstoul(szPreload.as_ptr(), core::ptr::null_mut(), 16) as LANGID;
            GetLocaleInfoW(langid, LOCALE_ILANGUAGE, szTmp.as_mut_ptr(), (szTmp.len()) as i32);
            wsprintf(szOldLangID.as_mut_ptr(), format_str.as_ptr() as PWSTR, szTmp.as_ptr());
            if wcscmp(szOldLangID.as_ptr(), szLangID) == 0 {
                IsLangExists = TRUE;
            } else {
                IsLangExists = FALSE;
            }
            if szPreload[0] == 0x64u16 {
                if RegOpenKeyExW(HKEY_CURRENT_USER, substitutes_key_path.as_ptr() as PWSTR,
                    0, KEY_QUERY_VALUE, &mut hSubKey as *mut HKEY) == ERROR_SUCCESS {
                    dwSize = (szTmp.len() * core::mem::size_of::<u16>()) as u32;
                    RegQueryValueExW(hSubKey, szPreload.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(),
                        szTmp.as_mut_ptr() as LPBYTE, &mut dwSize as *mut u32);
                    if (wcscmp(szTmp.as_ptr(), szLayoutID) == 0) && (IsLangExists != 0) {
                        RegCloseKey(hSubKey);
                        RegCloseKey(hKey);
                        return TRUE;
                    }
                }
            } else {
                if (wcscmp(szPreload.as_ptr(), szLayoutID) == 0) && (IsLangExists != 0) {
                    RegCloseKey(hKey);
                    return TRUE;
                }
            }
            IsLangExists = FALSE;
            dwSize = (szLayoutNum.len() * core::mem::size_of::<u16>()) as u32;
            dwIndex += 1;
        }
        RegCloseKey(hKey);
    }
    return FALSE;
}
unsafe extern "C" fn GetLayoutCount(szLang: PWSTR) -> i32 {
    let mut hKey: HKEY = core::ptr::null_mut();
    const KEYBOARD_PATH: &[u16] = &[
        0x004b, 0x0065, 0x0079, 0x0062, 0x006f, 0x0061, 0x0072, 0x0064, 0x0020,
        0x004c, 0x0061, 0x0079, 0x006f, 0x0075, 0x0074, 0x005c, 0x0050, 0x0072,
        0x0065, 0x006c, 0x006f, 0x0061, 0x0064, 0x0000,
    ];
    let mut szLayoutID: [u16; (3 + 1) as usize] =
        [0; (3 + 1) as usize];
    let mut szPreload: [u16; (CCH_LAYOUT_ID + 1) as usize] =
        [0; (CCH_LAYOUT_ID + 1) as usize];
    let mut szLOLang: [u16; MAX_PATH as usize] =
        [0; MAX_PATH as usize];
    let mut dwIndex: u32 = 0;
    let mut dwType: u32 = 0;
    let mut dwSize: u32;
    let mut Count: u32 = 0;
    let mut i: u32;
    let mut j: u32;

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        KEYBOARD_PATH.as_ptr() as PWSTR,
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    ) == ERROR_SUCCESS
    {
        dwSize = core::mem::size_of::<[u16; (3 + 1) as usize]>() as u32;
        while RegEnumValue(
            hKey,
            dwIndex,
            szLayoutID.as_mut_ptr(),
            &mut dwSize,
            core::ptr::null_mut(),
            &mut dwType,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == ERROR_SUCCESS
        {
            dwSize = core::mem::size_of::<[u16; (CCH_LAYOUT_ID + 1) as usize]>() as u32;
            RegQueryValueExW(
                hKey,
                szLayoutID.as_ptr() as PWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                szPreload.as_mut_ptr() as LPBYTE,
                &mut dwSize,
            );

            i = 4;
            j = 0;
            while i < (wcslen(szPreload.as_ptr()) as u32 + 1) {
                szLOLang[j as usize] = szPreload[i as usize];
                i += 1;
                j += 1;
            }

            if wcscmp(szLOLang.as_ptr(), szLang) == 0 {
                Count += 1;
            }

            dwSize = core::mem::size_of::<[u16; (3 + 1) as usize]>() as u32;
            dwIndex += 1;
        }

        RegCloseKey(hKey);
    }

    Count as i32
}
unsafe extern "C" fn AddNewLayout(szLayoutID: PWSTR, szLangID: PWSTR) -> BOOLEAN {
    let mut NewLayout: [u16; (CCH_ULONG_DEC as usize) + (1 as usize)] = [0u16; (CCH_ULONG_DEC as usize) + (1 as usize)];
    let mut Lang: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut LangID: [u16; (CCH_LAYOUT_ID as usize) + (1 as usize)] = [0u16; (CCH_LAYOUT_ID as usize) + (1 as usize)];
    let mut SubPath: [u16; (CCH_LAYOUT_ID as usize) + (1 as usize)] = [0u16; (CCH_LAYOUT_ID as usize) + (1 as usize)];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut cValues: u32 = 0;
    let mut lcid: LCID = 0;

    if RegOpenKeyExW(HKEY_CURRENT_USER, L"Keyboard Layout\\Preload", 0, KEY_ALL_ACCESS, &mut hKey) == ERROR_SUCCESS {
        if RegQueryInfoKey(hKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), &mut cValues, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) == ERROR_SUCCESS {
            _ultow(cValues.wrapping_add(1), NewLayout.as_mut_ptr(), 10);

            lcid = wcstoul(szLangID, core::ptr::null_mut(), 16);

            GetLocaleInfoW(MAKELCID(lcid, SORT_DEFAULT), LOCALE_ILANGUAGE, Lang.as_mut_ptr(), (core::mem::size_of_val(&Lang) / core::mem::size_of::<u16>()) as i32);
            wsprintf(LangID.as_mut_ptr(), L"0000%s", Lang.as_ptr());

            if IsLayoutExists(szLayoutID, LangID.as_ptr()) != 0u8 {
                RegCloseKey(hKey);
                return FALSE;
            }

            if GetLayoutCount(Lang.as_ptr()) >= 1 {
                wsprintf(SubPath.as_mut_ptr(), L"d%03d%s", GetLayoutCount(Lang.as_ptr()), Lang.as_ptr());
            } else if (wcscmp(LangID.as_ptr(), szLayoutID) != 0) && (GetLayoutCount(Lang.as_ptr()) == 0) {
                wsprintf(SubPath.as_mut_ptr(), L"d%03d%s", 0, Lang.as_ptr());
            } else {
                SubPath[0 as usize] = 0u16;
            }

            if wcslen(SubPath.as_ptr()) != 0 {
                if RegCreateKeyExW(HKEY_CURRENT_USER, L"Keyboard Layout\\Substitutes", 0, core::ptr::null_mut(),
                                    REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS,
                                    core::ptr::null_mut(), &mut hSubKey, core::ptr::null_mut()) == ERROR_SUCCESS {
                    if RegSetValueExW(hSubKey, SubPath.as_ptr(), 0, REG_SZ, szLayoutID as LPBYTE,
                                       (((CCH_LAYOUT_ID as usize) + (1 as usize)) * core::mem::size_of::<u16>()) as u32) != ERROR_SUCCESS {
                        RegCloseKey(hSubKey);
                        RegCloseKey(hKey);
                        return FALSE;
                    }
                    RegCloseKey(hSubKey);
                }
                lstrcpy(szLayoutID, SubPath.as_ptr());
            }

            RegSetValueExW(hKey,
                           NewLayout.as_ptr(),
                           0,
                           REG_SZ,
                           szLayoutID as LPBYTE,
                           (((CCH_LAYOUT_ID as usize) + (1 as usize)) * core::mem::size_of::<u16>()) as u32);
        }
        RegCloseKey(hKey);
    }

    TRUE
}
#[no_mangle]
pub unsafe extern "C" fn AddNewKbLayoutsByLcid(Lcid: LCID) {
    let wide_0000s: &[u16] = &[0x30u16, 0x30u16, 0x30u16, 0x30u16, 0x25u16, 0x73u16, 0x00u16];
    let wide_intl_inf: &[u16] = &[0x69u16, 0x6eu16, 0x74u16, 0x6cu16, 0x2eu16, 0x69u16, 0x6eu16, 0x66u16, 0x00u16];
    let wide_locales: &[u16] = &[0x4cu16, 0x6fu16, 0x63u16, 0x61u16, 0x6cu16, 0x65u16, 0x73u16, 0x00u16];

    let mut hIntlInf: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut szLang: [u16; (CCH_LAYOUT_ID + 1) as usize] = [0u16; (CCH_LAYOUT_ID + 1) as usize];
    let mut szLangID: [u16; (CCH_LAYOUT_ID + 1) as usize] = [0u16; (CCH_LAYOUT_ID + 1) as usize];
    let mut szLangStr: [u16; MAX_STR_SIZE as usize] = [0u16; MAX_STR_SIZE as usize];
    let mut szLayoutStr: [u16; MAX_STR_SIZE as usize] = [0u16; MAX_STR_SIZE as usize];
    let mut szStr: [u16; MAX_STR_SIZE as usize] = [0u16; MAX_STR_SIZE as usize];
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut Count: i32;
    let mut FieldCount: u32;

    GetLocaleInfoW(
        MAKELCID(Lcid, SORT_DEFAULT),
        LOCALE_ILANGUAGE,
        szLang.as_mut_ptr(),
        szLang.len() as u32,
    );
    wsprintf(szLangID.as_mut_ptr(), wide_0000s.as_ptr(), szLang.as_ptr());

    hIntlInf = SetupOpenInfFileW(wide_intl_inf.as_ptr(), core::ptr::null_mut(), INF_STYLE_WIN4, core::ptr::null_mut());
    if hIntlInf as isize == INVALID_HANDLE_VALUE {
        return;
    }

    if !SetupOpenAppendInfFile(core::ptr::null_mut(), hIntlInf, core::ptr::null_mut()) != 0 {
        SetupCloseInfFile(hIntlInf);
        hIntlInf = core::ptr::null_mut();
        return;
    }

    Count = SetupGetLineCount(hIntlInf, wide_locales.as_ptr());
    if Count <= 0 {
        return;
    }

    if SetupFindFirstLine(hIntlInf, wide_locales.as_ptr(), szLangID.as_mut_ptr(), &mut InfContext) != 0 {
        FieldCount = SetupGetFieldCount(&mut InfContext);
        if FieldCount != 0 {
            let mut Index = 5;
            while Index <= FieldCount {
                if SetupGetStringField(&mut InfContext, Index, szStr.as_mut_ptr(), MAX_STR_SIZE, core::ptr::null_mut()) != 0 {
                    if wcslen(szStr.as_ptr()) != 13 {
                        Index += 1;
                        continue;
                    }

                    wsprintf(szLangStr.as_mut_ptr(), wide_0000s.as_ptr(), szStr.as_ptr());
                    szLangStr[8 as usize] = 0u16;

                    let len = wcslen(szStr.as_ptr()) as usize;
                    let mut i: usize = 5 as usize;
                    let mut j: usize = 0 as usize;
                    while i <= len {
                        szLayoutStr[j] = szStr[i];
                        i += 1;
                        j += 1;
                    }

                    AddNewLayout(szLayoutStr.as_mut_ptr(), szLangStr.as_mut_ptr());
                }
                Index += 1;
            }
        }
    }

    SetupCloseInfFile(hIntlInf);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS International Control Panel
//  * FILE:            dll/cpl/intl/kblayouts.c
//  * PURPOSE:         Functions for manipulation with keyboard layouts
//  * PROGRAMMER:      Dmitry Chapyshev (dmitry@reactos.org)
//  */
//
// #include "intl.h"
//
// /* Character Count of a layout ID like "00000409" */
// #define CCH_LAYOUT_ID    8
//
// /* Maximum Character Count of a ULONG in decimal */
// #define CCH_ULONG_DEC    10
//
//
// /* szLayoutID like 00000409, szLangID like 00000409 */
// static BOOL
// IsLayoutExists(PWSTR szLayoutID, PWSTR szLangID)
// {
//     HKEY hKey, hSubKey;
//     WCHAR szPreload[CCH_LAYOUT_ID + 1], szLayoutNum[3 + 1],
//           szTmp[CCH_LAYOUT_ID + 1], szOldLangID[CCH_LAYOUT_ID + 1];
//     DWORD dwIndex = 0, dwType, dwSize;
//     BOOL IsLangExists = FALSE;
//     LANGID langid;
//
//     if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Keyboard Layout\\Preload",
//         0, KEY_QUERY_VALUE, &hKey) == ERROR_SUCCESS)
//     {
//         dwSize = sizeof(szLayoutNum);
//
//         while (RegEnumValueW(hKey, dwIndex, szLayoutNum, &dwSize, NULL, &dwType, NULL, NULL) == ERROR_SUCCESS)
//         {
//             dwSize = sizeof(szPreload);
//             if (RegQueryValueExW(hKey, szLayoutNum, NULL, NULL, (LPBYTE)szPreload, &dwSize) != ERROR_SUCCESS)
//             {
//                 RegCloseKey(hKey);
//                 return FALSE;
//             }
//
//             langid = (LANGID)wcstoul(szPreload, NULL, 16);
//             GetLocaleInfoW(langid, LOCALE_ILANGUAGE, szTmp, sizeof(szTmp) / sizeof(WCHAR));
//             wsprintf(szOldLangID, L"0000%s", szTmp);
//
//             if (wcscmp(szOldLangID, szLangID) == 0)
//                 IsLangExists = TRUE;
//             else
//                 IsLangExists = FALSE;
//
//             if (szPreload[0] == 'd')
//             {
//                 if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Keyboard Layout\\Substitutes",
//                                  0, KEY_QUERY_VALUE, &hSubKey) == ERROR_SUCCESS)
//                 {
//                     dwSize = sizeof(szTmp);
//                     RegQueryValueExW(hSubKey, szPreload, NULL, NULL, (LPBYTE)szTmp, &dwSize);
//
//                     if ((wcscmp(szTmp, szLayoutID) == 0)&&(IsLangExists))
//                     {
//                         RegCloseKey(hSubKey);
//                         RegCloseKey(hKey);
//                         return TRUE;
//                     }
//                 }
//             }
//             else
//             {
//                 if ((wcscmp(szPreload, szLayoutID) == 0) && (IsLangExists))
//                 {
//                     RegCloseKey(hKey);
//                     return TRUE;
//                 }
//             }
//
//             IsLangExists = FALSE;
//             dwSize = sizeof(szLayoutNum);
//             dwIndex++;
//         }
//
//         RegCloseKey(hKey);
//     }
//
//     return FALSE;
// }
//
// static INT
// GetLayoutCount(PWSTR szLang)
// {
//     HKEY hKey;
//     WCHAR szLayoutID[3 + 1], szPreload[CCH_LAYOUT_ID + 1], szLOLang[MAX_PATH];
//     DWORD dwIndex = 0, dwType, dwSize;
//     UINT Count = 0, i, j;
//
//     if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Keyboard Layout\\Preload",
//         0, KEY_QUERY_VALUE, &hKey) == ERROR_SUCCESS)
//     {
//         dwSize = sizeof(szLayoutID);
//
//         while (RegEnumValue(hKey, dwIndex, szLayoutID, &dwSize, NULL, &dwType, NULL, NULL) == ERROR_SUCCESS)
//         {
//             dwSize = sizeof(szPreload);
//             RegQueryValueExW(hKey, szLayoutID, NULL, NULL, (LPBYTE)szPreload, &dwSize);
//
//             for (i = 4, j = 0; i < wcslen(szPreload)+1; i++, j++)
//                 szLOLang[j] = szPreload[i];
//
//             if (wcscmp(szLOLang, szLang) == 0) Count += 1;
//
//             dwSize = sizeof(szLayoutID);
//             dwIndex++;
//         }
//
//         RegCloseKey(hKey);
//     }
//
//     return Count;
// }
//
// /* szLayoutID like 00000409, szLangID like 00000409 */
// static BOOL
// AddNewLayout(PWSTR szLayoutID, PWSTR szLangID)
// {
//     WCHAR NewLayout[CCH_ULONG_DEC + 1], Lang[MAX_PATH],
//           LangID[CCH_LAYOUT_ID + 1], SubPath[CCH_LAYOUT_ID + 1];
//     HKEY hKey, hSubKey;
//     DWORD cValues;
//     LCID lcid;
//
//     if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Keyboard Layout\\Preload", 0, KEY_ALL_ACCESS, &hKey) == ERROR_SUCCESS)
//     {
//         if (RegQueryInfoKey(hKey, NULL, NULL, NULL, NULL, NULL, NULL, &cValues, NULL, NULL, NULL, NULL) == ERROR_SUCCESS)
//         {
//             _ultow(cValues + 1, NewLayout, 10);
//
//             lcid = wcstoul(szLangID, NULL, 16);
//
//             GetLocaleInfoW(MAKELCID(lcid, SORT_DEFAULT), LOCALE_ILANGUAGE, Lang, sizeof(Lang) / sizeof(WCHAR));
//             wsprintf(LangID, L"0000%s", Lang);
//
//             if (IsLayoutExists(szLayoutID, LangID))
//             {
//                 RegCloseKey(hKey);
//                 return FALSE;
//             }
//
//             if (GetLayoutCount(Lang) >= 1)
//             {
//                 wsprintf(SubPath, L"d%03d%s", GetLayoutCount(Lang), Lang);
//             }
//             else if ((wcscmp(LangID, szLayoutID) != 0) && (GetLayoutCount(Lang) == 0))
//             {
//                 wsprintf(SubPath, L"d%03d%s", 0, Lang);
//             }
//             else SubPath[0] = L'\0';
//
//             if (wcslen(SubPath) != 0)
//             {
//                 if (RegCreateKeyExW(HKEY_CURRENT_USER, L"Keyboard Layout\\Substitutes", 0, NULL,
//                                     REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS,
//                                     NULL, &hSubKey, NULL) == ERROR_SUCCESS)
//                 {
//                     if (RegSetValueExW(hSubKey, SubPath, 0, REG_SZ, (LPBYTE)szLayoutID,
//                                        (DWORD)((CCH_LAYOUT_ID + 1) * sizeof(WCHAR))) != ERROR_SUCCESS)
//                     {
//                         RegCloseKey(hSubKey);
//                         RegCloseKey(hKey);
//                         return FALSE;
//                     }
//                     RegCloseKey(hSubKey);
//                 }
//                 lstrcpy(szLayoutID, SubPath);
//             }
//
//             RegSetValueExW(hKey,
//                            NewLayout,
//                            0,
//                            REG_SZ,
//                            (LPBYTE)szLayoutID,
//                            (DWORD)((CCH_LAYOUT_ID + 1) * sizeof(WCHAR)));
//         }
//         RegCloseKey(hKey);
//     }
//
//     return TRUE;
// }
//
// VOID
// AddNewKbLayoutsByLcid(LCID Lcid)
// {
//     HINF hIntlInf;
//     WCHAR szLang[CCH_LAYOUT_ID + 1], szLangID[CCH_LAYOUT_ID + 1];
//     WCHAR szLangStr[MAX_STR_SIZE], szLayoutStr[MAX_STR_SIZE], szStr[MAX_STR_SIZE];
//     INFCONTEXT InfContext;
//     LONG Count;
//     DWORD FieldCount, Index;
//
//     GetLocaleInfoW(MAKELCID(Lcid, SORT_DEFAULT), LOCALE_ILANGUAGE, szLang, sizeof(szLang) / sizeof(WCHAR));
//     wsprintf(szLangID, L"0000%s", szLang);
//
//     hIntlInf = SetupOpenInfFileW(L"intl.inf", NULL, INF_STYLE_WIN4, NULL);
//
//     if (hIntlInf == INVALID_HANDLE_VALUE)
//         return;
//
//     if (!SetupOpenAppendInfFile(NULL, hIntlInf, NULL))
//     {
//         SetupCloseInfFile(hIntlInf);
//         hIntlInf = NULL;
//         return;
//     }
//
//     Count = SetupGetLineCount(hIntlInf, L"Locales");
//     if (Count <= 0) return;
//
//     if (SetupFindFirstLine(hIntlInf, L"Locales", szLangID, &InfContext))
//     {
//         FieldCount = SetupGetFieldCount(&InfContext);
//
//         if (FieldCount != 0)
//         {
//             for (Index = 5; Index <= FieldCount; Index++)
//             {
//                 if (SetupGetStringField(&InfContext, Index, szStr, MAX_STR_SIZE, NULL))
//                 {
//                     INT i, j;
//
//                     if (wcslen(szStr) != 13) continue;
//
//                     wsprintf(szLangStr, L"0000%s", szStr);
//                     szLangStr[8] = L'\0';
//
//                     for (i = 5, j = 0; i <= wcslen(szStr); i++, j++)
//                         szLayoutStr[j] = szStr[i];
//
//                     AddNewLayout(szLayoutStr, szLangStr);
//                 }
//             }
//         }
//     }
//     SetupCloseInfFile(hIntlInf);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
