use ntdll_sys::nt::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::prelude::memmove;
use core::ffi::c_void;
use crate::string::nls_subst::*;
use crate::string::locale::CompareStringW;

extern "C" {
    fn strlen(s: LPCSTR) -> usize;
    fn wcslen(s: *const WCHAR) -> usize;
    // CRT, cdecl — on i386 this is `_wcscat` (undecorated), NOT the stdcall
    // `_wcscat@8` that `extern "system"` would emit. Must be extern "C".
    fn wcscat(dest: LPWSTR, src: LPCWSTR) -> LPWSTR;
}
extern "system" {
    fn CompareStringA( Locale: u32, dwCmpFlags: u32, lpString1: LPCSTR, cchCount1: i32, lpString2: LPCSTR, cchCount2: i32, ) -> i32;
    fn GetThreadLocale() -> u32;
}

#[no_mangle]
pub unsafe extern "system" fn lstrcmpA(
    lpString1: LPCSTR,
    lpString2: LPCSTR,
) -> i32 {
    let mut Result: i32;

    if lpString1 == lpString2 {
        return 0;
    }
    if lpString1.is_null() {
        return -1;
    }
    if lpString2.is_null() {
        return 1;
    }

    Result = CompareStringA(GetThreadLocale(), 0, lpString1, -1, lpString2, -1);
    if Result != 0 {
        Result -= 2;
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn lstrcmpiA(lpString1: LPCSTR, lpString2: LPCSTR) -> i32 {
    let mut Result: i32;
    if lpString1 == lpString2 {
        return 0i32;
    }
    if lpString1.is_null() {
        return -1i32;
    }
    if lpString2.is_null() {
        return 1i32;
    }
    Result = CompareStringA(GetThreadLocale(), NORM_IGNORECASE, lpString1, -1i32, lpString2, -1i32);
    if Result != 0i32 {
        Result -= 2i32;
    }
    return Result;
}
#[no_mangle]
pub unsafe extern "system" fn lstrcpynA(
    lpString1: LPSTR,
    lpString2: LPCSTR,
    iMaxLength: i32,
) -> LPSTR {
    let mut d: LPSTR = lpString1;
    let mut s: LPCSTR = lpString2;
    let mut count: u32 = iMaxLength as u32;
    let mut ret: LPSTR = core::ptr::null_mut();

    // _SEH2_TRY
    {
        while (count > 1) && (*s != 0) {
            count -= 1;
            *d = *s;
            d = d.offset(1);
            s = s.offset(1);
        }

        if count != 0 {
            *d = 0;
        }

        ret = lpString1;
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        // Exception handler body is empty
    }
    // _SEH2_END

    ret
}
#[no_mangle]
pub unsafe extern "system" fn lstrcpyA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR {
    let mut Ret: LPSTR = core::ptr::null_mut();

    // _SEH2_TRY
    {
        memmove(
            lpString1 as *mut core::ffi::c_void,
            lpString2 as *const core::ffi::c_void,
            strlen(lpString2) + 1,
        );
        Ret = lpString1;
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        // Exception handler body is empty
    }
    // _SEH2_END;

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn lstrcatA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR {
    let mut Ret: LPSTR = core::ptr::null_mut();

    // _SEH2_TRY
    {
        Ret = strcat(lpString1, lpString2);
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        // Exception handler body is empty
    }
    // _SEH2_END;

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn lstrlenA(lpString: LPCSTR) -> i32 {
    let mut Ret: i32 = 0;

    if lpString.is_null() {
        return 0;
    }

    // _SEH2_TRY
    {
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        let __exception_code: i32;
        // _SEH2_TRY expansion
        {
            // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            // We need to use __try/__except simulation
            // For now, we'll just call strlen directly without SEH
            // since Rust doesn't have SEH
            Ret = strlen(lpString) as i32;
        }
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) - empty handler
        // _SEH2_END
    }
    // _SEH2_END

    return Ret;
}
#[no_mangle]
pub unsafe extern "system" fn lstrcmpW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> i32 {
    let mut Result: i32;

    if lpString1 == lpString2 {
        return 0;
    }
    if lpString1.is_null() {
        return -1;
    }
    if lpString2.is_null() {
        return 1;
    }

    Result = CompareStringW(GetThreadLocale(), 0, lpString1, -1, lpString2, -1);
    if Result != 0 {
        Result -= 2;
    }

    return Result;
}
#[no_mangle]
pub unsafe extern "system" fn lstrcmpiW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> i32 {
    let mut Result: i32;

    if lpString1 == lpString2 {
        return 0;
    }
    if lpString1.is_null() {
        return -1;
    }
    if lpString2.is_null() {
        return 1;
    }

    Result = CompareStringW(GetThreadLocale(), NORM_IGNORECASE, lpString1, -1, lpString2, -1);
    if Result != 0 {
        Result -= 2;
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn lstrcpynW(
    lpString1: LPWSTR,
    lpString2: LPCWSTR,
    iMaxLength: i32,
) -> LPWSTR {
    let mut d: LPWSTR = lpString1;
    let mut s: LPCWSTR = lpString2;
    let mut count: u32 = iMaxLength as u32;
    let mut Ret: LPWSTR = core::ptr::null_mut();

    // _SEH2_TRY
    {
        while (count > 1) && (*s != 0) {
            count -= 1;
            *d = *s;
            d = d.offset(1);
            s = s.offset(1);
        }

        if count != 0 {
            *d = 0;
        }

        Ret = lpString1;
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        // Exception handler body is empty
    }
    // _SEH2_END;

    return Ret;
}
#[no_mangle]
pub unsafe extern "system" fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR {
    let mut Ret: LPWSTR = core::ptr::null_mut();

    // _SEH2_TRY
    {
        Ret = wcscpy(lpString1, lpString2);
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        // Exception handler body is empty
    }
    // _SEH2_END

    return Ret;
}
#[no_mangle]
pub unsafe extern "system" fn lstrcatW(
    lpString1: LPWSTR,
    lpString2: LPCWSTR,
) -> LPWSTR {
    let mut Ret: LPWSTR = core::ptr::null_mut();

    // _SEH2_TRY
    {
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            Ret = wcscat(lpString1, lpString2);
        }
    }
    // _SEH2_END

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn lstrlenW(lpString: LPCWSTR) -> i32 {
    let mut ret: i32 = 0;

    if lpString.is_null() {
        return 0;
    }

    // _SEH2_TRY
    {
        ret = wcslen(lpString) as i32;
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        // empty handler
    }
    // _SEH2_END

    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/winnls/string/lstring.c
//  * PURPOSE:         Local string functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// #include <k32.h>
//
// /*
//  * @implemented
//  */
// int
// WINAPI
// lstrcmpA(LPCSTR lpString1, LPCSTR lpString2)
// {
//     int Result;
//
//     if (lpString1 == lpString2)
//         return 0;
//     if (lpString1 == NULL)
//         return -1;
//     if (lpString2 == NULL)
//         return 1;
//
//     Result = CompareStringA(GetThreadLocale(), 0, lpString1, -1, lpString2, -1);
//     if (Result)
//         Result -= 2;
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// int
// WINAPI
// lstrcmpiA(LPCSTR lpString1, LPCSTR lpString2)
// {
//     int Result;
//
//     if (lpString1 == lpString2)
//         return 0;
//     if (lpString1 == NULL)
//         return -1;
//     if (lpString2 == NULL)
//         return 1;
//
//     Result = CompareStringA(GetThreadLocale(), NORM_IGNORECASE, lpString1, -1, lpString2, -1);
//     if (Result)
//         Result -= 2;
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// lstrcpynA(LPSTR lpString1, LPCSTR lpString2, int iMaxLength)
// {
//     LPSTR d = lpString1;
//     LPCSTR s = lpString2;
//     UINT count = iMaxLength;
//     LPSTR Ret = NULL;
//
//     _SEH2_TRY
//     {
//         while ((count > 1) && *s)
//         {
//             count--;
//             *d++ = *s++;
//         }
//
//         if (count)
//             *d = 0;
//
//         Ret = lpString1;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// lstrcpyA(LPSTR lpString1, LPCSTR lpString2)
// {
//     LPSTR Ret = NULL;
//
//     _SEH2_TRY
//     {
//         memmove(lpString1, lpString2, strlen(lpString2) + 1);
//         Ret = lpString1;
//      }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// LPSTR
// WINAPI
// lstrcatA(LPSTR lpString1, LPCSTR lpString2)
// {
//     LPSTR Ret = NULL;
//
//     _SEH2_TRY
//     {
//         Ret = strcat(lpString1, lpString2);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// int
// WINAPI
// lstrlenA(LPCSTR lpString)
// {
//     INT Ret = 0;
//
//     if (lpString == NULL)
//         return 0;
//
//     _SEH2_TRY
//     {
//         Ret = strlen(lpString);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// int
// WINAPI
// lstrcmpW(LPCWSTR lpString1, LPCWSTR lpString2)
// {
//     int Result;
//
//     if (lpString1 == lpString2)
//         return 0;
//     if (lpString1 == NULL)
//         return -1;
//     if (lpString2 == NULL)
//         return 1;
//
//     Result = CompareStringW(GetThreadLocale(), 0, lpString1, -1, lpString2, -1);
//     if (Result)
//         Result -= 2;
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// int
// WINAPI
// lstrcmpiW(LPCWSTR lpString1, LPCWSTR lpString2)
// {
//     int Result;
//
//     if (lpString1 == lpString2)
//         return 0;
//     if (lpString1 == NULL)
//         return -1;
//     if (lpString2 == NULL)
//         return 1;
//
//     Result = CompareStringW(GetThreadLocale(), NORM_IGNORECASE, lpString1, -1, lpString2, -1);
//     if (Result)
//         Result -= 2;
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// LPWSTR
// WINAPI
// lstrcpynW(LPWSTR lpString1, LPCWSTR lpString2, int iMaxLength)
// {
//     LPWSTR d = lpString1;
//     LPCWSTR s = lpString2;
//     UINT count = iMaxLength;
//     LPWSTR Ret = NULL;
//
//     _SEH2_TRY
//     {
//         while ((count > 1) && *s)
//         {
//             count--;
//             *d++ = *s++;
//         }
//
//         if (count)
//             *d = 0;
//
//         Ret = lpString1;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// LPWSTR
// WINAPI
// lstrcpyW(LPWSTR lpString1, LPCWSTR lpString2)
// {
//     LPWSTR Ret = NULL;
//
//     _SEH2_TRY
//     {
//         Ret = wcscpy(lpString1, lpString2);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// LPWSTR
// WINAPI
// lstrcatW(LPWSTR lpString1, LPCWSTR lpString2)
// {
//     LPWSTR Ret = NULL;
//
//     _SEH2_TRY
//     {
//         Ret = wcscat(lpString1, lpString2);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// int
// WINAPI
// lstrlenW(LPCWSTR lpString)
// {
//     INT Ret = 0;
//
//     if (lpString == NULL)
//         return 0;
//
//     _SEH2_TRY
//     {
//         Ret = wcslen(lpString);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
