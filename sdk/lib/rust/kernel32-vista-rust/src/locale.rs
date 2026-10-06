//! Locale APIs — LocaleNameToLCID.c, LCIDToLocaleName.c,
//! GetUserDefaultLocaleName.c, GetSystemDefaultLocaleName.c, IsValidLocaleName.c,
//! GetLocaleInfoEx.c.

use crate::common::*;
use crate::localenametolcid_table::neutral_lcid_remap;
use core::ffi::c_int;

/// LocaleNameToLCID.c
#[no_mangle]
pub unsafe extern "system" fn LocaleNameToLCID(lpName: LPCWSTR, dwFlags: DWORD) -> LCID {
    /* Validate flags */
    if dwFlags & !LOCALE_ALLOW_NEUTRAL_NAMES != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* LOCALE_NAME_USER_DEFAULT (NULL) */
    if lpName.is_null() {
        return GetUserDefaultLCID();
    }

    /* LOCALE_NAME_INVARIANT (L"") */
    if *lpName == 0 {
        return MAKELCID(
            MAKELANGID(LANG_INVARIANT as u16, SUBLANG_NEUTRAL as u16),
            SORT_DEFAULT as u16,
        );
    }

    /* Call the RTL function (include neutral names) */
    let mut Lcid: LCID = 0;
    let Status = RtlLocaleNameToLcid(lpName, &mut Lcid, RTL_LOCALE_ALLOW_NEUTRAL_NAMES);
    if !NT_SUCCESS(Status) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Are neutral locales allowed? */
    if (dwFlags & LOCALE_ALLOW_NEUTRAL_NAMES) == 0 {
        let LangId = LANGIDFROMLCID(Lcid);
        let SubLangId = SUBLANGID(LangId);
        if SubLangId == SUBLANG_NEUTRAL as u16 {
            /* Adjust it to be the default locale */
            Lcid = MAKELCID(
                MAKELANGID(PRIMARYLANGID(LangId), SUBLANG_DEFAULT as u16),
                SORT_DEFAULT as u16,
            );
        } else if SubLangId >= 0x18 {
            /* Handle special neutral LCIDs (generated table) */
            match neutral_lcid_remap(Lcid) {
                Some(mapped) => return mapped,
                None => {
                    /* C: DPRINT1 + ASSERT(FALSE) (no-op under NDEBUG) -> return 0 */
                    return 0;
                }
            }
        }
    }

    Lcid
}

/// LCIDToLocaleName.c
#[no_mangle]
pub unsafe extern "system" fn LCIDToLocaleName(
    Locale: LCID,
    lpName: LPWSTR,
    cchName: c_int,
    dwFlags: DWORD,
) -> c_int {
    let mut Buffer: [WCHAR; LOCALE_NAME_MAX_LENGTH as usize] = [0; LOCALE_NAME_MAX_LENGTH as usize];
    let mut LocaleNameString: UNICODE_STRING = core::mem::zeroed();
    let mut RtlFlags: DWORD = 0;
    let mut cchName = cchName;

    if cchName < 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if dwFlags & LOCALE_ALLOW_NEUTRAL_NAMES != 0 {
        RtlFlags |= RTL_LOCALE_ALLOW_NEUTRAL_NAMES;
    }

    if !lpName.is_null() {
        cchName = core::cmp::min(cchName, LOCALE_NAME_MAX_LENGTH as c_int);
        LocaleNameString.Buffer = lpName;
        LocaleNameString.Length = 0;
        LocaleNameString.MaximumLength = (cchName * core::mem::size_of::<WCHAR>() as c_int) as USHORT;
    } else {
        LocaleNameString.Buffer = Buffer.as_mut_ptr();
        LocaleNameString.Length = 0;
        LocaleNameString.MaximumLength =
            core::mem::size_of::<[WCHAR; LOCALE_NAME_MAX_LENGTH as usize]>() as USHORT;
    }

    /* Call the RTL function */
    let Status = RtlLcidToLocaleName(Locale, &mut LocaleNameString, RtlFlags, FALSE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    /* Return the length including the terminating null */
    (LocaleNameString.Length as c_int / core::mem::size_of::<WCHAR>() as c_int) + 1
}

/// GetSystemDefaultLocaleName.c (upstream ab1aefbabb3, for dwrite).
#[no_mangle]
pub unsafe extern "system" fn GetSystemDefaultLocaleName(localename: LPWSTR, len: INT) -> INT {
    let lcid: LCID = GetSystemDefaultLCID();
    LCIDToLocaleName(lcid, localename, len, 0)
}

/// GetUserDefaultLocaleName.c
#[no_mangle]
pub unsafe extern "system" fn GetUserDefaultLocaleName(
    lpLocaleName: LPWSTR,
    cchLocaleName: INT,
) -> INT {
    let mut cchLocaleName = cchLocaleName;

    if lpLocaleName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if cchLocaleName <= 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    cchLocaleName = core::cmp::min(cchLocaleName, LOCALE_NAME_MAX_LENGTH as INT);
    let mut LocaleNameString: UNICODE_STRING = core::mem::zeroed();
    LocaleNameString.Buffer = lpLocaleName;
    LocaleNameString.Length = 0;
    LocaleNameString.MaximumLength = (cchLocaleName * core::mem::size_of::<WCHAR>() as INT) as USHORT;

    let Status = RtlLcidToLocaleName(LOCALE_USER_DEFAULT, &mut LocaleNameString, 0, FALSE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    LocaleNameString.Length as INT / core::mem::size_of::<WCHAR>() as INT
}

/// IsValidLocaleName.c
#[no_mangle]
pub unsafe extern "system" fn IsValidLocaleName(lpLocaleName: LPCWSTR) -> BOOL {
    let lcid = LocaleNameToLCID(lpLocaleName, LOCALE_ALLOW_NEUTRAL_NAMES);
    (lcid != 0) as BOOL
}

/// GetLocaleInfoEx.c (from wine-locale.c)
#[no_mangle]
pub unsafe extern "system" fn GetLocaleInfoEx(
    locale: LPCWSTR,
    info: LCTYPE,
    buffer: LPWSTR,
    len: INT,
) -> INT {
    let lcid = LocaleNameToLCID(locale, 0);

    if lcid == 0 {
        return 0;
    }

    /* special handling for neutral locale names */
    if !locale.is_null() && wcslen(locale) == 2 {
        match info {
            LOCALE_SNAME => {
                if len != 0 && len < 3 {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                    return 0;
                }
                if len != 0 {
                    wcscpy(buffer, locale);
                }
                return 3;
            }
            LOCALE_SPARENT => {
                if len != 0 {
                    *buffer = 0;
                }
                return 1;
            }
            _ => {}
        }
    }

    GetLocaleInfoW(lcid, info, buffer, len)
}
