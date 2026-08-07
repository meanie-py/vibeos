// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn load_string(hModule: HINSTANCE, mut resId: u32, pwszBuffer: LPWSTR, cMaxChars: i32) -> i32 {
    let hMemory: HGLOBAL;
    let hResource: HRSRC;
    let mut pString: *mut u16;
    let mut idxString: i32;

    if HIWORD!(resId) == 0xffff {
        resId = (-(resId as i32)) as u32;
    }

    hResource = FindResourceW(hModule, MAKEINTRESOURCEW!(LOWORD!(resId >> 4) + 1), RT_STRING as LPWSTR);
    if hResource.is_null() {
        return 0;
    }
    hMemory = LoadResource(hModule, hResource);
    if hMemory.is_null() {
        return 0;
    }
    pString = LockResource(hMemory) as *mut u16;

    idxString = (resId & 0xf) as i32;
    while idxString > 0 {
        idxString -= 1;
        pString = pString.offset((*pString as isize) + 1);
    }

    if pwszBuffer.is_null() {
        return *pString as i32;
    }

    let mut cMaxChars = cMaxChars;
    cMaxChars = if (*pString as i32) < cMaxChars { *pString as i32 } else { cMaxChars - 1 };
    if cMaxChars >= 0 {
        memcpy(pwszBuffer as *mut core::ffi::c_void, pString.offset(1) as *const core::ffi::c_void, (cMaxChars as usize) * core::mem::size_of::<u16>());
        *pwszBuffer.offset(cMaxChars as isize) = 0u16;
    }

    cMaxChars
}
#[no_mangle]
pub unsafe extern "C" fn RegLoadMUIStringW(
    hKey: HKEY,
    pszValue: LPCWSTR,
    pszOutBuf: LPWSTR,
    cbOutBuf: DWORD,
    pcbData: LPDWORD,
    Flags: DWORD,
    pszDirectory: LPCWSTR,
) -> i32 {
    let mut dwValueType: DWORD = 0;
    let mut cbData: DWORD = 0;
    let mut pwszTempBuffer: LPWSTR = core::ptr::null_mut();
    let mut pwszExpandedBuffer: LPWSTR = core::ptr::null_mut();
    let mut result: i32;

    if hKey.is_null() || pszOutBuf.is_null() {
        return (ERROR_INVALID_PARAMETER) as i32;
    }

    if !pszDirectory.is_null() && *pszDirectory != 0 {
        return (ERROR_INVALID_PARAMETER) as i32;
    }

    result = RegQueryValueExW(
        hKey,
        pszValue,
        core::ptr::null_mut(),
        &mut dwValueType,
        core::ptr::null_mut(),
        &mut cbData,
    );
    if result != (ERROR_SUCCESS) as i32 {
        HeapFree(GetProcessHeap(), 0, pwszTempBuffer as HGLOBAL);
        HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as HGLOBAL);
        return result;
    }
    if !(dwValueType == REG_SZ || dwValueType == REG_EXPAND_SZ) || cbData == 0 {
        result = (ERROR_FILE_NOT_FOUND) as i32;
        HeapFree(GetProcessHeap(), 0, pwszTempBuffer as HGLOBAL);
        HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as HGLOBAL);
        return result;
    }
    pwszTempBuffer = HeapAlloc(GetProcessHeap(), 0, (cbData) as SIZE_T) as LPWSTR;
    if pwszTempBuffer.is_null() {
        result = (ERROR_NOT_ENOUGH_MEMORY) as i32;
        HeapFree(GetProcessHeap(), 0, pwszTempBuffer as HGLOBAL);
        HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as HGLOBAL);
        return result;
    }
    result = RegQueryValueExW(
        hKey,
        pszValue,
        core::ptr::null_mut(),
        &mut dwValueType,
        pwszTempBuffer as LPBYTE,
        &mut cbData,
    );
    if result != (ERROR_SUCCESS) as i32 {
        HeapFree(GetProcessHeap(), 0, pwszTempBuffer as HGLOBAL);
        HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as HGLOBAL);
        return result;
    }

    if dwValueType == REG_EXPAND_SZ {
        cbData = ExpandEnvironmentStringsW(pwszTempBuffer, core::ptr::null_mut(), 0) * core::mem::size_of::<u16>() as u32;
        if cbData == 0 {
            HeapFree(GetProcessHeap(), 0, pwszTempBuffer as HGLOBAL);
            HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as HGLOBAL);
            return result;
        }
        pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, (cbData) as SIZE_T) as LPWSTR;
        if pwszExpandedBuffer.is_null() {
            result = (ERROR_NOT_ENOUGH_MEMORY) as i32;
            HeapFree(GetProcessHeap(), 0, pwszTempBuffer as HGLOBAL);
            HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as HGLOBAL);
            return result;
        }
        ExpandEnvironmentStringsW(pwszTempBuffer, pwszExpandedBuffer, cbData);
    } else {
        pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, (cbData) as SIZE_T) as LPWSTR;
        memcpy(pwszExpandedBuffer as *mut core::ffi::c_void, pwszTempBuffer as *const core::ffi::c_void, cbData as usize);
    }

    result = (ERROR_SUCCESS) as i32;
    if *pwszExpandedBuffer != '@' as u16 {
        lstrcpynW(pszOutBuf, pwszExpandedBuffer, (cbOutBuf / core::mem::size_of::<u16>() as u32) as i32);
    } else {
        let pComma: *mut u16 = wcsrchr(pwszExpandedBuffer, ',' as u16);
        let mut uiStringId: u32;
        let hModule: HMODULE;

        if pComma.is_null() || *pComma.offset(1 as isize) != '-' as u16 {
            result = (ERROR_BADKEY) as i32;
            HeapFree(GetProcessHeap(), 0, pwszTempBuffer as HGLOBAL);
            HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as HGLOBAL);
            return result;
        }

        uiStringId = _wtoi(pComma.offset(2 as isize)) as u32;
        *pComma = '\0' as u16;

        hModule = LoadLibraryExW(
            pwszExpandedBuffer.offset(1 as isize),
            core::ptr::null_mut(),
            LOAD_LIBRARY_AS_DATAFILE,
        );
        if hModule.is_null() || load_string(hModule, uiStringId, pszOutBuf, (cbOutBuf / core::mem::size_of::<u16>() as u32) as i32) == 0 {
            result = (ERROR_BADKEY) as i32;
        }
        FreeLibrary(hModule);
    }

    HeapFree(GetProcessHeap(), 0, pwszTempBuffer as HGLOBAL);
    HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as HGLOBAL);
    result
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "desk.h"
//
// /******************************************************************************
//  * load_string [Internal]
//  *
//  * This is basically a copy of user32/resource.c's LoadStringW. Necessary to
//  * avoid importing user32, which is higher level than advapi32. Helper for
//  * RegLoadMUIString.
//  */
// static int load_string(HINSTANCE hModule, UINT resId, LPWSTR pwszBuffer, INT cMaxChars)
// {
//     HGLOBAL hMemory;
//     HRSRC hResource;
//     WCHAR *pString;
//     int idxString;
//
//     /* Negative values have to be inverted. */
//     if (HIWORD(resId) == 0xffff)
//         resId = (UINT)(-((INT)resId));
//
//     /* Load the resource into memory and get a pointer to it. */
//     hResource = FindResourceW(hModule, MAKEINTRESOURCEW(LOWORD(resId >> 4) + 1), (LPWSTR)RT_STRING);
//     if (!hResource) return 0;
//     hMemory = LoadResource(hModule, hResource);
//     if (!hMemory) return 0;
//     pString = LockResource(hMemory);
//
//     /* Strings are length-prefixed. Lowest nibble of resId is an index. */
//     idxString = resId & 0xf;
//     while (idxString--) pString += *pString + 1;
//
//     /* If no buffer is given, return length of the string. */
//     if (!pwszBuffer) return *pString;
//
//     /* Else copy over the string, respecting the buffer size. */
//     cMaxChars = (*pString < cMaxChars) ? *pString : (cMaxChars - 1);
//     if (cMaxChars >= 0)
//     {
//         memcpy(pwszBuffer, pString+1, cMaxChars * sizeof(WCHAR));
//         pwszBuffer[cMaxChars] = L'\0';
//     }
//
//     return cMaxChars;
// }
//
//
// /************************************************************************
//  *  RegLoadMUIStringW
//  *
//  * @implemented
//  */
// LONG
// RegLoadMUIStringW( HKEY hKey,
//                    LPCWSTR pszValue  ,
//                    LPWSTR pszOutBuf,
//                    DWORD cbOutBuf,
//                    LPDWORD pcbData ,
//                    DWORD Flags,
//                    LPCWSTR pszDirectory  )
// {
//     DWORD dwValueType, cbData;
//     LPWSTR pwszTempBuffer = NULL, pwszExpandedBuffer = NULL;
//     LONG result;
//
//     /* Parameter sanity checks. */
//     if (!hKey || !pszOutBuf)
//         return ERROR_INVALID_PARAMETER;
//
//     if (pszDirectory && *pszDirectory)
//     {
//         //FIXME("BaseDir parameter not yet supported!\n");
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Check for value existence and correctness of it's type, allocate a buffer and load it. */
//     result = RegQueryValueExW(hKey, pszValue, NULL, &dwValueType, NULL, &cbData);
//     if (result != ERROR_SUCCESS) goto cleanup;
//     if (!(dwValueType == REG_SZ || dwValueType == REG_EXPAND_SZ) || !cbData)
//     {
//         result = ERROR_FILE_NOT_FOUND;
//         goto cleanup;
//     }
//     pwszTempBuffer = HeapAlloc(GetProcessHeap(), 0, cbData);
//     if (!pwszTempBuffer)
//     {
//         result = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//     result = RegQueryValueExW(hKey, pszValue, NULL, &dwValueType, (LPBYTE)pwszTempBuffer, &cbData);
//     if (result != ERROR_SUCCESS) goto cleanup;
//
//     /* Expand environment variables, if appropriate, or copy the original string over. */
//     if (dwValueType == REG_EXPAND_SZ)
//     {
//         cbData = ExpandEnvironmentStringsW(pwszTempBuffer, NULL, 0) * sizeof(WCHAR);
//         if (!cbData) goto cleanup;
//         pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, cbData);
//         if (!pwszExpandedBuffer)
//         {
//             result = ERROR_NOT_ENOUGH_MEMORY;
//             goto cleanup;
//         }
//         ExpandEnvironmentStringsW(pwszTempBuffer, pwszExpandedBuffer, cbData);
//     }
//     else
//     {
//         pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, cbData);
//         memcpy(pwszExpandedBuffer, pwszTempBuffer, cbData);
//     }
//
//     /* If the value references a resource based string, parse the value and load the string.
//      * Else just copy over the original value. */
//     result = ERROR_SUCCESS;
//     if (*pwszExpandedBuffer != L'@') /* '@' is the prefix for resource based string entries. */
//     {
//         lstrcpynW(pszOutBuf, pwszExpandedBuffer, cbOutBuf / sizeof(WCHAR));
//     }
//     else
//     {
//         WCHAR *pComma = wcsrchr(pwszExpandedBuffer, L',');
//         UINT uiStringId;
//         HMODULE hModule;
//
//         /* Format of the expanded value is 'path_to_dll,-resId' */
//         if (!pComma || pComma[1] != L'-')
//         {
//             result = ERROR_BADKEY;
//             goto cleanup;
//         }
//
//         uiStringId = _wtoi(pComma+2);
//         *pComma = L'\0';
//
//         hModule = LoadLibraryExW(pwszExpandedBuffer + 1, NULL, LOAD_LIBRARY_AS_DATAFILE);
//         if (!hModule || !load_string(hModule, uiStringId, pszOutBuf, cbOutBuf / sizeof(WCHAR)))
//             result = ERROR_BADKEY;
//         FreeLibrary(hModule);
//     }
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, pwszTempBuffer);
//     HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer);
//     return result;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
