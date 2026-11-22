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
        match neutral_default(Lcid) {
            Some(Default) => return Default,
            /* C: DPRINT1 + ASSERT(FALSE) (no-op under NDEBUG) -> return 0 */
            None if is_neutral(Lcid) => return 0,
            None => {}
        }
    }

    Lcid
}

/// A neutral LCID names a language without a region: sublanguage 0 ("en" 0x0009), or one of
/// the script/region neutrals at sublanguage 0x18 and up ("zh-Hant" 0x7c04). The LANG_NEUTRAL
/// defaults (0x0400, 0x0800, 0x0c00, ...) and LOCALE_INVARIANT (0x007f) are not.
fn is_neutral(Lcid: LCID) -> bool {
    let LangId = LANGIDFROMLCID(Lcid);
    PRIMARYLANGID(LangId) != 0
        && Lcid != 0x007F
        && (SUBLANGID(LangId) == SUBLANG_NEUTRAL as u16 || SUBLANGID(LangId) >= 0x18)
}

/// The specific locale Windows resolves a neutral LCID to when neutral names are not
/// allowed. Mostly SUBLANG_DEFAULT or the C's generated table; the exceptions are where
/// that rule picks a different locale than Windows does (measured by Wine's locale test).
fn neutral_default(Lcid: LCID) -> Option<LCID> {
    if !is_neutral(Lcid) {
        return None;
    }
    let Exception = match Lcid {
        0x000A => Some(0x0C0A), // "es" -> es-ES (modern sort); 0x040A is es-ES_tradnl
        0x003C => Some(0x083C), // "ga" -> ga-IE; 0x043C is gd-GB, another language
        0x0004 => Some(0x0804), // "zh-Hans" -> zh-CN; 0x0404 is zh-TW
        0x7C1A => Some(0x241A), // "sr" -> sr-Latn-RS; missing from the C's table
        _ => None,
    };
    if Exception.is_some() {
        return Exception;
    }
    let LangId = LANGIDFROMLCID(Lcid);
    if SUBLANGID(LangId) == SUBLANG_NEUTRAL as u16 {
        return Some(MAKELCID(
            MAKELANGID(PRIMARYLANGID(LangId), SUBLANG_DEFAULT as u16),
            SORT_DEFAULT as u16,
        ));
    }
    neutral_lcid_remap(Lcid)
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

    // Divergences from the C, which passed these straight to RtlLcidToLocaleName and so
    // failed them. As on Windows: LOCALE_NEUTRAL and the custom defaults are the user's
    // locale, and without LOCALE_ALLOW_NEUTRAL_NAMES a neutral LCID names its default
    // specific locale (0x0004 is "zh-CN", 0x7c04 "zh-HK") instead of failing.
    let mut Locale = Locale;
    match Locale {
        LOCALE_NEUTRAL | LOCALE_CUSTOM_DEFAULT | LOCALE_CUSTOM_UNSPECIFIED
        | LOCALE_CUSTOM_UI_DEFAULT => Locale = LOCALE_USER_DEFAULT,
        _ if RtlFlags & RTL_LOCALE_ALLOW_NEUTRAL_NAMES == 0 => {
            if let Some(Default) = neutral_default(Locale) {
                Locale = Default;
            }
        }
        _ => {}
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
    // NULL is LOCALE_NAME_USER_DEFAULT to LocaleNameToLCID, but not a valid name: Windows
    // answers FALSE (the C answered TRUE).
    if lpLocaleName.is_null() {
        return 0;
    }
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
    // LOCALE_NOUSEROVERRIDE does not change a neutral name's SNAME or (empty) SPARENT.
    if !locale.is_null() && wcslen(locale) == 2 {
        match info & !LOCALE_NOUSEROVERRIDE {
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
