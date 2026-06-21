use ntdll_sys::nt::*;
use core::ffi::c_void;
// Canonical JAPANESE_ERA / PJAPANESE_ERA / PCJAPANESE_ERA + wcschr/wcstol/
// RtlStringCbCopyW substrate.
use crate::string::nls_subst::*;
// Internal NLS registry helpers (defined in locale.rs, the nls.c translation).
use crate::string::locale::{NLS_RegOpenKey, NLS_RegEnumValue};
// lstrcmpiW — WINAPI string compare (lstring.c translation).
use crate::string::lstring::lstrcmpiW;
// RtlZeroMemory inline (memset 0).
use crate::k32_externs::RtlZeroMemory;

// The C uses L"..." wide-string literals for the registry paths and value name.
// DeepSeek emitted `b"...".as_ptr() as LPCWSTR` which reinterprets 8-bit bytes as
// UTF-16 (garbage). These paths are pure ASCII, so build a real null-terminated
// UTF-16 array at compile time and hand back a 'static pointer to it.
macro_rules! wstr_ptr {
    ($s:literal) => {{
        const BYTES: &[u8] = $s.as_bytes();
        const N: usize = BYTES.len() + 1;
        const W: [WCHAR; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < BYTES.len() {
                a[i] = BYTES[i] as u16;
                i += 1;
            }
            a
        };
        W.as_ptr() as LPCWSTR
    }};
}

// bindgen defines TRUE/FALSE as u32 but BOOL as c_int (i32). The C uses them as
// BOOL values; shadow with BOOL-typed consts so the casts vanish.
const FALSE: BOOL = 0;
// winnt.h: #define UNICODE_NULL ((WCHAR)0)
const UNICODE_NULL: WCHAR = 0;

// ---- Win32 / CRT externs japanese.c calls (real exports) ----
extern "system" {
    fn GetLocalTime(lpSystemTime: LPSYSTEMTIME);
    fn SystemTimeToFileTime(lpSystemTime: *const SYSTEMTIME, lpFileTime: LPFILETIME) -> BOOL;
    fn CompareFileTime(lpFileTime1: *const FILETIME, lpFileTime2: *const FILETIME) -> LONG;
}
extern "C" {
    // msvcrt CRT: int _wtoi(const wchar_t *str)
    fn _wtoi(str: *const WCHAR) -> INT;
    // msvcrt CRT: qsort with a C-ABI comparator.
    fn qsort(
        base: *mut c_void,
        nmemb: usize,
        size: usize,
        compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> INT>,
    );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut s_bIsGannenCached: BOOL = FALSE;

static mut s_JapaneseEraCount: DWORD = 0 as DWORD;

static mut s_JapaneseEraTable: [JAPANESE_ERA; 16] = [JAPANESE_ERA {
    wYear: 0,
    wMonth: 0,
    wDay: 0,
    szEraName: [0; 16],
    szEraAbbrev: [0; 5],
    szEnglishEraName: [0; 24],
    szEnglishEraAbbrev: [0; 5],
}; 16];

pub const NDEBUG: () = ();

pub const JAPANESE_ERA_MAX: u32 = 16;

#[no_mangle]
pub unsafe extern "C" fn JapaneseEra_ClearCache() {
    s_bIsGannenCached = 0;
    s_JapaneseEraCount = 0;
}
unsafe extern "C" fn JapaneseEra_Compare(
    e1: *const core::ffi::c_void,
    e2: *const core::ffi::c_void,
) -> INT {
    let pEra1: PCJAPANESE_ERA = e1 as PCJAPANESE_ERA;
    let pEra2: PCJAPANESE_ERA = e2 as PCJAPANESE_ERA;
    if ((*pEra1).wYear as INT) < ((*pEra2).wYear as INT) {
        return -1 as INT;
    }
    if ((*pEra1).wYear as INT) > ((*pEra2).wYear as INT) {
        return 1 as INT;
    }
    if ((*pEra1).wMonth as INT) < ((*pEra2).wMonth as INT) {
        return -1 as INT;
    }
    if ((*pEra1).wMonth as INT) > ((*pEra2).wMonth as INT) {
        return 1 as INT;
    }
    if ((*pEra1).wDay as INT) < ((*pEra2).wDay as INT) {
        return -1 as INT;
    }
    if ((*pEra1).wDay as INT) > ((*pEra2).wDay as INT) {
        return 1 as INT;
    }
    return 0 as INT;
}
#[no_mangle]
pub unsafe extern "C" fn JapaneseEra_IsFirstYearGannen() -> BOOL {
    let mut KeyHandle: HANDLE;
    let mut dwIndex: DWORD;
    let mut szName: [WCHAR; 32] = [0; 32];
    let mut szValue: [WCHAR; 32] = [0; 32];
    static mut s_bFirstIsGannen: BOOL = 1;

    if s_bIsGannenCached != 0 {
        return s_bFirstIsGannen;
    }

    KeyHandle = NLS_RegOpenKey(
        core::ptr::null_mut(),
        wstr_ptr!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Nls\\Calendars\\Japanese"),
    );
    if KeyHandle.is_null() {
        return 1;
    }

    s_bFirstIsGannen = 1;
    dwIndex = 0;
    while dwIndex < 16 {
        if NLS_RegEnumValue(
            KeyHandle,
            dwIndex,
            szName.as_mut_ptr() as LPWSTR,
            core::mem::size_of::<[WCHAR; 32]>() as ULONG,
            szValue.as_mut_ptr() as LPWSTR,
            core::mem::size_of::<[WCHAR; 32]>() as ULONG,
        ) == 0
        {
            break;
        }

        if lstrcmpiW(
            szName.as_ptr() as LPCWSTR,
            wstr_ptr!("InitialEraYear"),
        ) == 0
        {
            s_bFirstIsGannen = (szValue[0] as i32 == 0x5143) as BOOL;
            break;
        }

        dwIndex = dwIndex.wrapping_add(1);
    }

    NtClose(KeyHandle);

    s_bIsGannenCached = 1;

    return s_bFirstIsGannen;
}
unsafe fn JapaneseEra_Load(pdwCount: *mut DWORD) -> PCJAPANESE_ERA {
    let mut KeyHandle: HANDLE;
    let mut dwIndex: DWORD;
    let mut szName: [WCHAR; 128] = [0; 128];
    let mut szValue: [WCHAR; 128] = [0; 128];
    let mut pEntry: *mut JAPANESE_ERA;
    let mut pch1: LPWSTR;
    let mut pch2: LPWSTR;
    let mut pch3: LPWSTR;
    let mut pch4: LPWSTR;

    /* return cache if any */
    if s_JapaneseEraCount != 0 {
        *pdwCount = s_JapaneseEraCount;
        return &raw mut s_JapaneseEraTable as *mut JAPANESE_ERA as PCJAPANESE_ERA;
    }

    /* init */
    *pdwCount = 0;
    RtlZeroMemory(&raw mut s_JapaneseEraTable as *mut ::core::ffi::c_void, core::mem::size_of::<[JAPANESE_ERA; 16]>() as SIZE_T);

    /* open registry key */
    KeyHandle = NLS_RegOpenKey(
        core::ptr::null_mut(),
        wstr_ptr!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Nls\\Calendars\\Japanese\\Eras"),
    );
    if KeyHandle.is_null() {
        return core::ptr::null();
    }

    /* for all values */
    dwIndex = 0;
    while dwIndex < JAPANESE_ERA_MAX as DWORD {
        pEntry = (&raw mut s_JapaneseEraTable as *mut JAPANESE_ERA).offset(dwIndex as isize);

        /* get name and data */
        if NLS_RegEnumValue(
            KeyHandle,
            dwIndex,
            &raw mut szName as LPWSTR,
            core::mem::size_of::<[WCHAR; 128]>() as ULONG,
            &raw mut szValue as LPWSTR,
            core::mem::size_of::<[WCHAR; 128]>() as ULONG,
        ) == 0
        {
            break;
        }

        /* split fields */
        pch1 = &raw mut szName as LPWSTR;
        pch2 = wcschr(pch1 as *const u16, ' ' as WCHAR) as LPWSTR;
        if pch2.is_null() {
            break;
        }
        *pch2 = UNICODE_NULL;
        pch2 = pch2.offset(1);

        pch3 = wcschr(pch2 as *const u16, ' ' as WCHAR) as LPWSTR;
        if pch3.is_null() {
            break;
        }
        *pch3 = UNICODE_NULL;
        pch3 = pch3.offset(1);

        (*pEntry).wYear = _wtoi(pch1 as *const u16) as WORD;
        (*pEntry).wMonth = _wtoi(pch2 as *const u16) as WORD;
        (*pEntry).wDay = _wtoi(pch3 as *const u16) as WORD;
        if (*pEntry).wYear == 0 || (*pEntry).wMonth == 0 || (*pEntry).wDay == 0 {
            break;
        }

        /* split fields */
        pch1 = &raw mut szValue as LPWSTR;
        pch2 = wcschr(pch1 as *const u16, '_' as WCHAR) as LPWSTR;
        if pch2.is_null() {
            break;
        }
        *pch2 = UNICODE_NULL;
        pch2 = pch2.offset(1);

        pch3 = wcschr(pch2 as *const u16, '_' as WCHAR) as LPWSTR;
        if pch3.is_null() {
            break;
        }
        *pch3 = UNICODE_NULL;
        pch3 = pch3.offset(1);

        pch4 = wcschr(pch3 as *const u16, '_' as WCHAR) as LPWSTR;
        if pch4.is_null() {
            break;
        }
        *pch4 = UNICODE_NULL;
        pch4 = pch4.offset(1);

        /* store */
        RtlStringCbCopyW(
            &raw mut (*pEntry).szEraName as LPWSTR,
            core::mem::size_of::<[WCHAR; 16]>() as usize,
            pch1 as *const u16,
        );
        RtlStringCbCopyW(
            &raw mut (*pEntry).szEraAbbrev as LPWSTR,
            core::mem::size_of::<[WCHAR; 5]>() as usize,
            pch2 as *const u16,
        );
        RtlStringCbCopyW(
            &raw mut (*pEntry).szEnglishEraName as LPWSTR,
            core::mem::size_of::<[WCHAR; 24]>() as usize,
            pch3 as *const u16,
        );
        RtlStringCbCopyW(
            &raw mut (*pEntry).szEnglishEraAbbrev as LPWSTR,
            core::mem::size_of::<[WCHAR; 5]>() as usize,
            pch4 as *const u16,
        );

        dwIndex = dwIndex.wrapping_add(1);
    }

    /* close key */
    NtClose(KeyHandle);

    /* sort */
    qsort(
        &raw mut s_JapaneseEraTable as *mut ::core::ffi::c_void,
        s_JapaneseEraCount as usize,
        core::mem::size_of::<JAPANESE_ERA>() as usize,
        Some(JapaneseEra_Compare as unsafe extern "C" fn(*const ::core::ffi::c_void, *const ::core::ffi::c_void) -> INT),
    );

    /* make cache */
    s_JapaneseEraCount = dwIndex;

    *pdwCount = s_JapaneseEraCount;

    return &raw mut s_JapaneseEraTable as *mut JAPANESE_ERA as PCJAPANESE_ERA;
}
unsafe fn JapaneseEra_ToSystemTime(
    pEra: PCJAPANESE_ERA,
    pst: LPSYSTEMTIME,
) -> BOOL {
    // ASSERT(pEra != NULL);
    if pEra.is_null() {
        RtlAssert(
            b"pEra != NULL\0".as_ptr() as *const i8 as PVOID,
            b"dll/win32/kernel32/winnls/string/japanese.c\0"
                .as_ptr() as *const i8 as PVOID,
            226u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    // ASSERT(pst != NULL);
    if pst.is_null() {
        RtlAssert(
            b"pst != NULL\0".as_ptr() as *const i8 as PVOID,
            b"dll/win32/kernel32/winnls/string/japanese.c\0"
                .as_ptr() as *const i8 as PVOID,
            227u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    // ZeroMemory(pst, sizeof(*pst));
    core::ptr::write_bytes(pst as *mut u8, 0, core::mem::size_of::<SYSTEMTIME>());
    // pst->wYear = pEra->wYear;
    (*pst).wYear = (*pEra).wYear;
    // pst->wMonth = pEra->wMonth;
    (*pst).wMonth = (*pEra).wMonth;
    // pst->wDay = pEra->wDay;
    (*pst).wDay = (*pEra).wDay;
    // return TRUE;
    return 1i32;
}
#[no_mangle]
pub unsafe extern "C" fn JapaneseEra_Find(mut pst: *const SYSTEMTIME) -> PCJAPANESE_ERA {
    let mut dwIndex: DWORD = 0;
    let mut dwCount: DWORD = 0;
    let mut pTable: PCJAPANESE_ERA = core::ptr::null();
    let mut pEntry: PCJAPANESE_ERA = core::ptr::null();
    let mut pPrevEntry: PCJAPANESE_ERA = core::ptr::null();
    let mut st1: SYSTEMTIME = core::mem::zeroed();
    let mut st2: SYSTEMTIME = core::mem::zeroed();
    let mut ft1: FILETIME = core::mem::zeroed();
    let mut ft2: FILETIME = core::mem::zeroed();
    let mut nCompare: LONG = 0;

    /* pst --> ft1 */
    if pst.is_null() {
        GetLocalTime(&mut st1);
        pst = &raw const st1;
    }
    SystemTimeToFileTime(pst, &mut ft1);

    /* load era table */
    pTable = JapaneseEra_Load(&mut dwCount);
    if pTable.is_null() || dwCount == 0 || dwCount > JAPANESE_ERA_MAX as DWORD {
        return core::ptr::null();
    }

    /* for all eras */
    dwIndex = 0;
    while dwIndex < dwCount {
        pEntry = &*pTable.add(dwIndex as usize) as PCJAPANESE_ERA;

        /* pEntry --> st2 --> ft2 */
        JapaneseEra_ToSystemTime(pEntry, &mut st2);
        SystemTimeToFileTime(&st2, &mut ft2);

        /* ft1 <=> ft2 */
        nCompare = CompareFileTime(&ft1, &ft2);
        if nCompare == 0 {
            return pEntry;
        }
        if nCompare < 0 {
            return pPrevEntry;
        }
        pPrevEntry = pEntry;
        dwIndex = dwIndex.wrapping_add(1);
    }

    return pPrevEntry;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS system libraries
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Japanese era support
//  * COPYRIGHT:   Copyright 2019 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
// #include "japanese.h"
//
// #define JAPANESE_ERA_MAX 16
//
// /* #define DONT_USE_REGISTRY */
//
// static BOOL         s_bIsGannenCached = FALSE;
// static DWORD        s_JapaneseEraCount = 0;
// static JAPANESE_ERA s_JapaneseEraTable[JAPANESE_ERA_MAX]
// #ifdef DONT_USE_REGISTRY
// =
// {
//     {1868, 1, 1, {0x660E, 0x6CBB}, {0x660E, 0}, L"Meiji", L"M"},
//     {1912, 7, 30, {0x5927, 0x6B63}, {0x5927, 0}, L"Taisho", L"T"},
//     {1926, 12, 25, {0x662D, 0x548C}, {0x662D, 0}, L"Showa", L"S"},
//     {1989, 1, 8, {0x5E73, 0x6210}, {0x5E73, 0}, L"Heisei", L"H"},
//     {2019, 5, 1, {0x4EE4, 0x548C}, {0x4EE4, 0}, L"Reiwa", L"R"},
// }
// #endif
// ;
//
// HANDLE NLS_RegOpenKey(HANDLE hRootKey, LPCWSTR szKeyName);
//
// BOOL NLS_RegEnumValue(HANDLE hKey, UINT ulIndex,
//                       LPWSTR szValueName, ULONG valueNameSize,
//                       LPWSTR szValueData, ULONG valueDataSize);
//
// void JapaneseEra_ClearCache(void)
// {
//     s_bIsGannenCached = FALSE;
//     s_JapaneseEraCount = 0;
// }
//
// static INT JapaneseEra_Compare(const void *e1, const void *e2)
// {
//     PCJAPANESE_ERA pEra1 = (PCJAPANESE_ERA)e1;
//     PCJAPANESE_ERA pEra2 = (PCJAPANESE_ERA)e2;
//     if (pEra1->wYear < pEra2->wYear)
//         return -1;
//     if (pEra1->wYear > pEra2->wYear)
//         return 1;
//     if (pEra1->wMonth < pEra2->wMonth)
//         return -1;
//     if (pEra1->wMonth > pEra2->wMonth)
//         return 1;
//     if (pEra1->wDay < pEra2->wDay)
//         return -1;
//     if (pEra1->wDay > pEra2->wDay)
//         return 1;
//     return 0;
// }
//
// BOOL JapaneseEra_IsFirstYearGannen(void)
// {
// #ifdef DONT_USE_REGISTRY
//     return TRUE;
// #else
//     HANDLE KeyHandle;
//     DWORD dwIndex;
//     WCHAR szName[32], szValue[32];
//     static BOOL s_bFirstIsGannen = TRUE;
//
//     if (s_bIsGannenCached)
//         return s_bFirstIsGannen;
//
//     KeyHandle = NLS_RegOpenKey(NULL, L"\\Registry\\Machine\\System\\"
//         L"CurrentControlSet\\Control\\Nls\\Calendars\\Japanese");
//     if (!KeyHandle)
//         return TRUE;
//
//     s_bFirstIsGannen = TRUE;
//     for (dwIndex = 0; dwIndex < 16; ++dwIndex)
//     {
//         if (!NLS_RegEnumValue(KeyHandle, dwIndex, szName, sizeof(szName),
//                               szValue, sizeof(szValue)))
//         {
//             break;
//         }
//
//         if (lstrcmpiW(szName, L"InitialEraYear") == 0)
//         {
//             s_bFirstIsGannen = (szValue[0] == 0x5143);
//             break;
//         }
//     }
//
//     NtClose(KeyHandle);
//
//     s_bIsGannenCached = TRUE;
//
//     return s_bFirstIsGannen;
// #endif
// }
//
// /*
//  * SEE ALSO:
//  * https://en.wikipedia.org/wiki/Japanese_era_name
//  * https://learn.microsoft.com/en-us/windows/win32/intl/era-handling-for-the-japanese-calendar
//  */
// static PCJAPANESE_ERA JapaneseEra_Load(DWORD *pdwCount)
// {
// #ifndef DONT_USE_REGISTRY
//     HANDLE KeyHandle;
//     DWORD dwIndex;
//     WCHAR szName[128], szValue[128];
//     JAPANESE_ERA *pEntry;
//     LPWSTR pch1, pch2, pch3, pch4;
// #endif
//
//     ASSERT(pdwCount != NULL);
//
//     /* return cache if any */
//     if (s_JapaneseEraCount != 0)
//     {
//         *pdwCount = s_JapaneseEraCount;
//         return s_JapaneseEraTable;
//     }
//
// #ifdef DONT_USE_REGISTRY
//     s_JapaneseEraCount = ARRAYSIZE(s_JapaneseEraTable);
// #else
//     /* init */
//     *pdwCount = 0;
//     RtlZeroMemory(&s_JapaneseEraTable, sizeof(s_JapaneseEraTable));
//
//     /* open registry key */
//     KeyHandle = NLS_RegOpenKey(NULL, L"\\Registry\\Machine\\System\\"
//         L"CurrentControlSet\\Control\\Nls\\Calendars\\Japanese\\Eras");
//     if (!KeyHandle)
//         return NULL;
//
//     /* for all values */
//     for (dwIndex = 0; dwIndex < JAPANESE_ERA_MAX; ++dwIndex)
//     {
//         pEntry = &s_JapaneseEraTable[dwIndex];
//
//         /* get name and data */
//         if (!NLS_RegEnumValue(KeyHandle, dwIndex, szName, sizeof(szName),
//                               szValue, sizeof(szValue)))
//         {
//             break;
//         }
//
//         /* split fields */
//         pch1 = szName;
//         pch2 = wcschr(pch1, L' ');
//         if (pch2 == NULL)
//         {
//             break;
//         }
//         *pch2++ = UNICODE_NULL;
//
//         pch3 = wcschr(pch2, L' ');
//         if (pch3 == NULL)
//         {
//             break;
//         }
//         *pch3++ = UNICODE_NULL;
//
//         pEntry->wYear = _wtoi(pch1);
//         pEntry->wMonth = _wtoi(pch2);
//         pEntry->wDay = _wtoi(pch3);
//         if (pEntry->wYear == 0 || pEntry->wMonth == 0 || pEntry->wDay == 0)
//         {
//             break;
//         }
//
//         /* split fields */
//         pch1 = szValue;
//         pch2 = wcschr(pch1, L'_');
//         if (pch2 == NULL)
//         {
//             break;
//         }
//         *pch2++ = UNICODE_NULL;
//
//         pch3 = wcschr(pch2, L'_');
//         if (pch3 == NULL)
//         {
//             break;
//         }
//         *pch3++ = UNICODE_NULL;
//
//         pch4 = wcschr(pch3, L'_');
//         if (pch4 == NULL)
//         {
//             break;
//         }
//         *pch4++ = UNICODE_NULL;
//
//         /* store */
//         RtlStringCbCopyW(pEntry->szEraName, sizeof(pEntry->szEraName), pch1);
//         RtlStringCbCopyW(pEntry->szEraAbbrev, sizeof(pEntry->szEraAbbrev), pch2);
//         RtlStringCbCopyW(pEntry->szEnglishEraName, sizeof(pEntry->szEnglishEraName), pch3);
//         RtlStringCbCopyW(pEntry->szEnglishEraAbbrev, sizeof(pEntry->szEnglishEraAbbrev), pch4);
//     }
//
//     /* close key */
//     NtClose(KeyHandle);
//
//     /* sort */
//     qsort(s_JapaneseEraTable, s_JapaneseEraCount, sizeof(JAPANESE_ERA),
//           JapaneseEra_Compare);
//
//     /* make cache */
//     s_JapaneseEraCount = dwIndex;
// #endif
//
//     *pdwCount = s_JapaneseEraCount;
//
//     return s_JapaneseEraTable;
// }
//
// static BOOL JapaneseEra_ToSystemTime(PCJAPANESE_ERA pEra, LPSYSTEMTIME pst)
// {
//     ASSERT(pEra != NULL);
//     ASSERT(pst != NULL);
//
//     ZeroMemory(pst, sizeof(*pst));
//     pst->wYear = pEra->wYear;
//     pst->wMonth = pEra->wMonth;
//     pst->wDay = pEra->wDay;
//     return TRUE;
// }
//
// PCJAPANESE_ERA JapaneseEra_Find(const SYSTEMTIME *pst OPTIONAL)
// {
//     DWORD dwIndex, dwCount = 0;
//     PCJAPANESE_ERA pTable, pEntry, pPrevEntry = NULL;
//     SYSTEMTIME st1, st2;
//     FILETIME ft1, ft2;
//     LONG nCompare;
//
//     /* pst --> ft1 */
//     if (pst == NULL)
//     {
//         GetLocalTime(&st1);
//         pst = &st1;
//     }
//     SystemTimeToFileTime(pst, &ft1);
//
//     /* load era table */
//     pTable = JapaneseEra_Load(&dwCount);
//     if (pTable == NULL || dwCount == 0 || dwCount > JAPANESE_ERA_MAX)
//     {
//         return NULL;
//     }
//
//     /* for all eras */
//     for (dwIndex = 0; dwIndex < dwCount; dwIndex++)
//     {
//         pEntry = &pTable[dwIndex];
//
//         /* pEntry --> st2 --> ft2 */
//         JapaneseEra_ToSystemTime(pEntry, &st2);
//         SystemTimeToFileTime(&st2, &ft2);
//
//         /* ft1 <=> ft2 */
//         nCompare = CompareFileTime(&ft1, &ft2);
//         if (nCompare == 0)
//             return pEntry;
//         if (nCompare < 0)
//             return pPrevEntry;
//         pPrevEntry = pEntry;
//     }
//
//     return pPrevEntry;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
