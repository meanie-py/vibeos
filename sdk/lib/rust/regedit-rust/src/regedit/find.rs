use crate::regedit::treeview::GetItemPath;
use crate::regedit::edit::GetKeyName;
use crate::regedit::listview::GetValueName;
use crate::regedit::treeview::SelectNode;
use crate::regedit::listview::SetValueName;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut s_szFindWhat: [WCHAR; 256] = [0; 256];

static mut s_szFindFlags: [WCHAR; 10] =
    unsafe { ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(*b"F\0i\0n\0d\0F\0l\0a\0g\0s\0\0\0") };

static mut s_szFindFlagsR: [WCHAR; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [WCHAR; 17]>(
        *b"F\0i\0n\0d\0F\0l\0a\0g\0s\0R\0e\0a\0c\0t\0O\0S\0\0\0",
    )
};

static mut s_hwndAbortDialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut s_bAbort: BOOL = 0;

static mut s_dwFlags: DWORD = 0;

static mut s_szName: [WCHAR; 260] = [0; 260];

static mut s_cchName: DWORD = 0;

static mut s_empty: [WCHAR; 1] = unsafe { ::core::mem::transmute::<[u8; 2], [WCHAR; 1]>(*b"\0\0") };

static mut s_backslash: [WCHAR; 2] =
    unsafe { ::core::mem::transmute::<[u8; 4], [WCHAR; 2]>(*b"\\\0\0\0") };

pub const RSF_WHOLESTRING: u32 = 0x00000001;

pub const RSF_LOOKATKEYS: u32 = 0x00000002;

pub const RSF_LOOKATVALUES: u32 = 0x00000004;

pub const RSF_LOOKATDATA: u32 = 0x00000008;

pub const RSF_MATCHCASE: u32 = 0x00010000;
#[no_mangle]
pub unsafe extern "C" fn DoEvents() -> BOOL {
    let mut msg: MSG = core::mem::zeroed();
    if PeekMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
        if msg.message == WM_QUIT {
            s_bAbort = (TRUE) as i32;
        }
        if IsDialogMessageW(s_hwndAbortDialog, core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }
    }
    return s_bAbort;
}

unsafe extern "C" fn lstrstri(psz1: LPCWSTR, psz2: LPCWSTR) -> LPWSTR {
    let mut i: INT;
    let mut cch1: INT;
    let mut cch2: INT;
    cch1 = wcslen(psz1) as INT;
    cch2 = wcslen(psz2) as INT;
    i = 0 as INT;
    while i <= cch1 - cch2 {
        if CompareStringW(
            LOCALE_SYSTEM_DEFAULT,
            NORM_IGNORECASE,
            psz1.offset(i as isize),
            cch2,
            psz2,
            cch2,
        ) == 2
        {
            return psz1.offset(i as isize) as LPWSTR;
        }
        i += 1;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn CompareName(pszName1: LPCWSTR, pszName2: LPCWSTR) -> BOOL {
    if (s_dwFlags & RSF_WHOLESTRING) != 0 {
        if (s_dwFlags & RSF_MATCHCASE) != 0 {
            return (wcscmp(pszName1, pszName2) == 0) as BOOL;
        } else {
            return (_wcsicmp(pszName1, pszName2) == 0) as BOOL;
        }
    } else {
        if (s_dwFlags & RSF_MATCHCASE) != 0 {
            return (!wcsstr(pszName1, pszName2).is_null()) as BOOL;
        } else {
            return (!lstrstri(pszName1, pszName2).is_null()) as BOOL;
        }
    }
}

unsafe extern "C" fn MatchString(pch1: LPCWCH, cch1: INT, pch2: LPCWCH, cch2: INT) -> BOOL {
    let mut i: INT;
    let dwNorm: DWORD = (if s_dwFlags & RSF_MATCHCASE as DWORD != 0 {
        0
    } else {
        NORM_IGNORECASE
    }) as DWORD;

    if s_dwFlags & RSF_WHOLESTRING as DWORD != 0 {
        return (2
            == CompareStringW(
                LOCALE_SYSTEM_DEFAULT,
                dwNorm,
                pch1 as LPCWSTR,
                cch1,
                pch2 as LPCWSTR,
                cch2,
            )) as BOOL;
    }

    if cch1 < cch2 {
        return (FALSE) as BOOL;
    }

    i = 0;
    while i <= cch1 - cch2 {
        if 2
            == CompareStringW(
                LOCALE_SYSTEM_DEFAULT,
                dwNorm,
                pch1.offset(i as isize) as LPCWSTR,
                cch2,
                pch2 as LPCWSTR,
                cch2,
            )
        {
            return (TRUE) as BOOL;
        }
        i += 1;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn MatchData(dwType: DWORD, pv1: LPCVOID, cb1: DWORD) -> BOOL {
    if dwType == REG_SZ || dwType == REG_EXPAND_SZ || dwType == REG_MULTI_SZ {
        return MatchString(
            pv1 as LPCWCH,
            (cb1 / (core::mem::size_of::<WCHAR>() as DWORD)) as INT,
            core::ptr::addr_of_mut!(s_szFindWhat) as *mut WCHAR as LPCWCH,
            lstrlenW(core::ptr::addr_of_mut!(s_szFindWhat) as *mut WCHAR as LPCWSTR) as INT,
        );
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn compare(x: *const ::core::ffi::c_void, y: *const ::core::ffi::c_void) -> i32 {
    let a = x as *const LPCWSTR;
    let b = y as *const LPCWSTR;
    return _wcsicmp(*a, *b);
}
#[no_mangle]
pub unsafe extern "C" fn RegFindRecurse(
    hKey: HKEY,
    pszSubKey: LPCWSTR,
    pszValueName: LPCWSTR,
    ppszFoundSubKey: *mut LPWSTR,
    ppszFoundValueName: *mut LPWSTR,
) -> BOOL {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut lResult: LONG = 0;
    let mut szSubKey: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut i: DWORD = 0;
    let mut c: DWORD = 0;
    let mut cb: DWORD = 0;
    let mut type_: DWORD = 0;
    let mut fPast: BOOL = 0;
    let mut ppszNames: *mut LPWSTR = core::ptr::null_mut();
    let mut pb: LPBYTE = core::ptr::null_mut();
    let mut success: bool = false;

    if DoEvents() != 0 {
        return (FALSE) as BOOL;
    }

    if wcslen(pszSubKey) >= _countof!(szSubKey) as size_t {
        return (FALSE) as BOOL;
    }

    StringCbCopyW(
        szSubKey.as_mut_ptr(),
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
        pszSubKey,
    );
    hSubKey = core::ptr::null_mut();

    lResult = RegOpenKeyExW(
        hKey,
        szSubKey.as_mut_ptr() as LPCWSTR,
        0,
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(hSubKey),
    ) as LONG;
    if lResult != ERROR_SUCCESS as LONG {
        return (FALSE) as BOOL;
    }

    lResult = RegQueryInfoKeyW(
        hSubKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(c),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as LONG;

    'body: {
        if lResult != ERROR_SUCCESS as LONG {
            break 'body;
        }
        ppszNames = malloc(
            (c as size_t).wrapping_mul(core::mem::size_of::<LPWSTR>() as size_t),
        ) as *mut LPWSTR;
        if ppszNames.is_null() {
            break 'body;
        }
        ZeroMemory!(
            ppszNames as *mut ::core::ffi::c_void,
            (c as size_t).wrapping_mul(core::mem::size_of::<LPWSTR>() as size_t),
        );

        /* Retrieve the value names associated with the current key */
        i = 0 as DWORD;
        while i < c {
            if DoEvents() != 0 {
                break 'body;
            }

            s_cchName = _countof!(s_szName);
            lResult = RegEnumValueW(
                hSubKey,
                i,
                core::ptr::addr_of_mut!(s_szName) as *mut WCHAR,
                core::ptr::addr_of_mut!(s_cchName),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(cb),
            ) as LONG;
            if lResult == ERROR_NO_MORE_ITEMS as LONG {
                c = i;
                break;
            }
            if lResult != ERROR_SUCCESS as LONG {
                break 'body;
            }
            if !(s_cchName >= _countof!(s_szName)) {
                *ppszNames.offset(i as isize) =
                    _wcsdup(core::ptr::addr_of_mut!(s_szName) as *mut WCHAR)
                        as LPWSTR;
            }
            i += 1;
        }

        qsort(
            ppszNames as *mut ::core::ffi::c_void,
            c as size_t,
            core::mem::size_of::<LPWSTR>() as size_t,
            Some(
                compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );

        /* If pszValueName is NULL, the function will search for all values within the key */
        fPast = (pszValueName.is_null()) as BOOL;

        /* Search within the values */
        i = 0 as DWORD;
        while i < c {
            if DoEvents() != 0 {
                break 'body;
            }

            if fPast == 0
                && _wcsicmp(*ppszNames.offset(i as isize) as LPCWSTR, pszValueName)
                    == 0
            {
                fPast = TRUE as BOOL;
            } else if fPast != 0 {
                if (s_dwFlags & RSF_LOOKATVALUES) != 0
                    && CompareName(
                        *ppszNames.offset(i as isize) as LPCWSTR,
                        core::ptr::addr_of_mut!(s_szFindWhat) as *mut WCHAR as LPCWSTR,
                    ) != 0
                {
                    *ppszFoundSubKey =
                        _wcsdup(szSubKey.as_mut_ptr() as *const WCHAR) as LPWSTR;
                    *ppszFoundValueName =
                        _wcsdup(*ppszNames.offset(i as isize) as *const WCHAR)
                            as LPWSTR;
                    success = true;
                    break 'body;
                }

                lResult = RegQueryValueExW(
                    hSubKey,
                    *ppszNames.offset(i as isize) as LPCWSTR,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(type_),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(cb),
                ) as LONG;
                if lResult != ERROR_SUCCESS as LONG {
                    break 'body;
                }
                pb = malloc(cb as size_t) as LPBYTE;
                if pb.is_null() {
                    break 'body;
                }
                lResult = RegQueryValueExW(
                    hSubKey,
                    *ppszNames.offset(i as isize) as LPCWSTR,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(type_),
                    pb,
                    core::ptr::addr_of_mut!(cb),
                ) as LONG;
                if lResult != ERROR_SUCCESS as LONG {
                    break 'body;
                }

                if (s_dwFlags & RSF_LOOKATDATA) != 0
                    && MatchData(type_, pb as LPCVOID, cb) != 0
                {
                    *ppszFoundSubKey =
                        _wcsdup(szSubKey.as_mut_ptr() as *const WCHAR) as LPWSTR;
                    *ppszFoundValueName =
                        _wcsdup(*ppszNames.offset(i as isize) as *const WCHAR)
                            as LPWSTR;
                    success = true;
                    break 'body;
                }
                free(pb as *mut ::core::ffi::c_void);
                pb = core::ptr::null_mut();
            }
            i += 1;
        }

        if !ppszNames.is_null() {
            i = 0 as DWORD;
            while i < c {
                free(*ppszNames.offset(i as isize) as *mut ::core::ffi::c_void);
                i += 1;
            }
            free(ppszNames as *mut ::core::ffi::c_void);
        }
        ppszNames = core::ptr::null_mut();

        /* Retrieve the number of sub-keys */
        lResult = RegQueryInfoKeyW(
            hSubKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(c),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
        if lResult != ERROR_SUCCESS as LONG {
            break 'body;
        }
        ppszNames = malloc(
            (c as size_t).wrapping_mul(core::mem::size_of::<LPWSTR>() as size_t),
        ) as *mut LPWSTR;
        if ppszNames.is_null() {
            break 'body;
        }
        ZeroMemory!(
            ppszNames as *mut ::core::ffi::c_void,
            (c as size_t).wrapping_mul(core::mem::size_of::<LPWSTR>() as size_t),
        );

        /* Retrieve the names of the sub-keys */
        i = 0 as DWORD;
        while i < c {
            if DoEvents() != 0 {
                break 'body;
            }

            s_cchName = _countof!(s_szName);
            lResult = RegEnumKeyExW(
                hSubKey,
                i,
                core::ptr::addr_of_mut!(s_szName) as *mut WCHAR,
                core::ptr::addr_of_mut!(s_cchName),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as LONG;
            if lResult == ERROR_NO_MORE_ITEMS as LONG {
                c = i;
                break;
            }
            if lResult != ERROR_SUCCESS as LONG {
                break 'body;
            }
            if !(s_cchName >= _countof!(s_szName)) {
                *ppszNames.offset(i as isize) =
                    _wcsdup(core::ptr::addr_of_mut!(s_szName) as *mut WCHAR)
                        as LPWSTR;
            }
            i += 1;
        }

        qsort(
            ppszNames as *mut ::core::ffi::c_void,
            c as size_t,
            core::mem::size_of::<LPWSTR>() as size_t,
            Some(
                compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );

        /* Search within the sub-keys */
        i = 0 as DWORD;
        while i < c {
            if DoEvents() != 0 {
                break 'body;
            }

            if (s_dwFlags & RSF_LOOKATKEYS) != 0
                && CompareName(
                    *ppszNames.offset(i as isize) as LPCWSTR,
                    core::ptr::addr_of_mut!(s_szFindWhat) as *mut WCHAR as LPCWSTR,
                ) != 0
            {
                *ppszFoundSubKey = malloc(
                    (wcslen(szSubKey.as_ptr())
                        + wcslen(*ppszNames.offset(i as isize) as *const WCHAR)
                            as size_t
                        + 2 as size_t)
                        * (core::mem::size_of::<WCHAR>() as size_t),
                ) as LPWSTR;
                if (*ppszFoundSubKey).is_null() {
                    break 'body;
                }
                if szSubKey[0 as usize] != 0 {
                    wcscpy(*ppszFoundSubKey, szSubKey.as_ptr());
                    wcscat(
                        *ppszFoundSubKey,
                        core::ptr::addr_of!(s_backslash) as *const WCHAR,
                    );
                } else {
                    **ppszFoundSubKey = 0 as WCHAR;
                }
                wcscat(*ppszFoundSubKey, *ppszNames.offset(i as isize) as LPCWSTR);
                *ppszFoundValueName = core::ptr::null_mut();
                success = true;
                break 'body;
            }

            /* Search within the value entries of the sub-key */
            if RegFindRecurse(
                hSubKey,
                *ppszNames.offset(i as isize) as LPCWSTR,
                core::ptr::null(),
                ppszFoundSubKey,
                ppszFoundValueName,
            ) != 0
            {
                let psz: LPWSTR = *ppszFoundSubKey;
                let cbFoundSubKey: SIZE_T = (wcslen(szSubKey.as_ptr()) as SIZE_T
                    + wcslen(psz as *const WCHAR) as SIZE_T
                    + 2 as SIZE_T)
                    * (core::mem::size_of::<WCHAR>() as SIZE_T);
                *ppszFoundSubKey = malloc(cbFoundSubKey as size_t) as LPWSTR;
                if (*ppszFoundSubKey).is_null() {
                    // The C leaked the recursion's two results here.
                    free(psz as *mut ::core::ffi::c_void);
                    free(*ppszFoundValueName as *mut ::core::ffi::c_void);
                    *ppszFoundValueName = core::ptr::null_mut();
                    break 'body;
                }
                if szSubKey[0 as usize] != 0 {
                    StringCbCopyW(
                        *ppszFoundSubKey,
                        cbFoundSubKey as size_t,
                        szSubKey.as_ptr(),
                    );
                    StringCbCatW(
                        *ppszFoundSubKey,
                        cbFoundSubKey as size_t,
                        core::ptr::addr_of!(s_backslash) as *const WCHAR,
                    );
                } else {
                    **ppszFoundSubKey = 0 as WCHAR;
                }
                wcscat(*ppszFoundSubKey, psz as LPCWSTR);
                free(psz as *mut ::core::ffi::c_void);
                success = true;
                break 'body;
            }
            i += 1;
        }
    }

    if success {
        /* success: */
        // The C kept `pb` on this path: a match found in a value's DATA leaked that value.
        free(pb as *mut ::core::ffi::c_void);
        if !ppszNames.is_null() {
            i = 0 as DWORD;
            while i < c {
                free(*ppszNames.offset(i as isize) as *mut ::core::ffi::c_void);
                i += 1;
            }
            free(ppszNames as *mut ::core::ffi::c_void);
        }
        RegCloseKey(hSubKey);
        return (TRUE) as BOOL;
    }

    /* err: */
    if !ppszNames.is_null() {
        i = 0 as DWORD;
        while i < c {
            free(*ppszNames.offset(i as isize) as *mut ::core::ffi::c_void);
            i += 1;
        }
        free(ppszNames as *mut ::core::ffi::c_void);
    }
    free(pb as *mut ::core::ffi::c_void);
    RegCloseKey(hSubKey);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn RegFindWalk(
    phKey: *mut HKEY,
    pszSubKey: LPCWSTR,
    pszValueName: LPCWSTR,
    ppszFoundSubKey: *mut LPWSTR,
    ppszFoundValueName: *mut LPWSTR,
) -> BOOL {
    let mut lResult: LONG = 0;
    let mut i: DWORD = 0;
    let mut c: DWORD = 0;
    let mut hBaseKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut szKeyName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szSubKey: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pch: LPWSTR = core::ptr::null_mut();
    let mut fPast: BOOL = 0;
    let mut fKeyMatched: BOOL = 0;
    let mut ppszNames: *mut LPWSTR = core::ptr::null_mut();
    let mut jump: u8 = 0;

    hBaseKey = *phKey;
    // Not NULL: an exit taken before the walk opens anything must not close a handle.
    hSubKey = hBaseKey;

    if (wcslen(pszSubKey) as usize) >= (_countof!(szSubKey) as usize) {
        return (FALSE) as BOOL;
    }

    if RegFindRecurse(hBaseKey, pszSubKey, pszValueName, ppszFoundSubKey, ppszFoundValueName) != 0 {
        return (TRUE) as BOOL;
    }

    StringCbCopyW(
        &raw mut szSubKey as *mut WCHAR,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
        pszSubKey,
    );

    'walk: {
        while szSubKey[0 as usize] != 0 {
            if DoEvents() != 0 {
                return (FALSE) as BOOL;
            }

            pch = wcsrchr(&raw mut szSubKey as *mut WCHAR, 0x005Cu16);
            if pch.is_null() {
                wcscpy(&raw mut szKeyName as *mut WCHAR, &raw mut szSubKey as *mut WCHAR);
                szSubKey[0 as usize] = 0 as WCHAR;
                hSubKey = hBaseKey;
            } else {
                lstrcpynW(
                    &raw mut szKeyName as LPWSTR,
                    pch.offset(1 as isize) as LPCWSTR,
                    (MAX_PATH) as i32,
                );
                *pch = 0 as WCHAR;
                lResult = RegOpenKeyExW(
                    hBaseKey,
                    &raw mut szSubKey as *mut WCHAR as LPCWSTR,
                    0,
                    KEY_ALL_ACCESS,
                    &raw mut hSubKey,
                );
                if (lResult) as u32 != ERROR_SUCCESS {
                    return (FALSE) as BOOL;
                }
            }

            lResult = RegQueryInfoKeyW(
                hSubKey,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut c,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if (lResult) as u32 != ERROR_SUCCESS {
                jump = 1;
                break 'walk;
            }

            ppszNames = malloc((c as usize).wrapping_mul(core::mem::size_of::<LPWSTR>() as usize)) as *mut LPWSTR;
            if ppszNames.is_null() {
                jump = 1;
                break 'walk;
            }
            ZeroMemory!(
                ppszNames as *mut core::ffi::c_void,
                (c as usize).wrapping_mul(core::mem::size_of::<LPWSTR>() as usize),
            );

            i = 0;
            while i < c {
                if DoEvents() != 0 {
                    jump = 1;
                    break 'walk;
                }

                s_cchName = _countof!(s_szName) as DWORD;
                lResult = RegEnumKeyExW(
                    hSubKey,
                    i,
                    &raw mut s_szName as LPWSTR,
                    &raw mut s_cchName,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if (lResult) as u32 == ERROR_NO_MORE_ITEMS {
                    c = i;
                    break;
                }
                if (lResult) as u32 != ERROR_SUCCESS {
                    break;
                }
                *ppszNames.offset(i as isize) = _wcsdup(&raw mut s_szName as *mut WCHAR) as LPWSTR;
                i += 1;
            }

            qsort(
                ppszNames as *mut core::ffi::c_void,
                c as usize,
                core::mem::size_of::<LPWSTR>() as usize,
                Some(compare as unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int),
            );

            fPast = (FALSE) as i32;
            i = 0;
            while i < c {
                if DoEvents() != 0 {
                    jump = 1;
                    break 'walk;
                }

                if fPast == 0
                    && _wcsicmp(
                        *ppszNames.offset(i as isize),
                        &raw mut szKeyName as *mut WCHAR,
                    ) == 0
                {
                    fPast = (TRUE) as i32;
                } else if fPast != 0 {
                    if (s_dwFlags & RSF_LOOKATKEYS) != 0
                        && CompareName(
                            *ppszNames.offset(i as isize) as LPCWSTR,
                            &raw mut s_szFindWhat as *mut WCHAR as LPCWSTR,
                        ) != 0
                    {
                        *ppszFoundSubKey = malloc(
                            (wcslen(&raw mut szSubKey as *mut WCHAR)
                                .wrapping_add(wcslen(*ppszNames.offset(i as isize) as *const WCHAR))
                                .wrapping_add(2 as usize))
                                .wrapping_mul(core::mem::size_of::<WCHAR>()),
                        ) as LPWSTR;
                        if (*ppszFoundSubKey).is_null() {
                            jump = 1;
                            break 'walk;
                        }
                        if szSubKey[0 as usize] != 0 {
                            wcscpy(*ppszFoundSubKey, &raw mut szSubKey as *mut WCHAR);
                            wcscat(*ppszFoundSubKey, &raw const s_backslash as *const WCHAR);
                        } else {
                            **ppszFoundSubKey = 0 as WCHAR;
                        }
                        wcscat(*ppszFoundSubKey, *ppszNames.offset(i as isize) as *const WCHAR);
                        *ppszFoundValueName = core::ptr::null_mut();
                        jump = 2;
                        break 'walk;
                    }

                    fKeyMatched = (_wcsicmp(
                        *ppszNames.offset(i as isize) as *const WCHAR,
                        &raw mut szKeyName as *mut WCHAR,
                    ) == 0) as BOOL;
                    if RegFindRecurse(
                        hSubKey,
                        *ppszNames.offset(i as isize) as LPCWSTR,
                        if fKeyMatched != 0 {
                            pszValueName
                        } else {
                            core::ptr::null()
                        },
                        ppszFoundSubKey,
                        ppszFoundValueName,
                    ) != 0
                    {
                        let mut psz: LPWSTR = *ppszFoundSubKey;
                        let mut cbFoundSubKey: usize = (wcslen(&raw mut szSubKey as *mut WCHAR) as usize)
                            .wrapping_add(wcslen(psz as *const WCHAR) as usize)
                            .wrapping_add(2 as usize)
                            .wrapping_mul(core::mem::size_of::<WCHAR>());
                        *ppszFoundSubKey = malloc(cbFoundSubKey) as LPWSTR;
                        if (*ppszFoundSubKey).is_null() {
                            // The C leaked the recursion's two results here.
                            free(psz as *mut core::ffi::c_void);
                            free(*ppszFoundValueName as *mut core::ffi::c_void);
                            *ppszFoundValueName = core::ptr::null_mut();
                            jump = 1;
                            break 'walk;
                        }
                        if szSubKey[0 as usize] != 0 {
                            StringCbCopyW(
                                *ppszFoundSubKey,
                                cbFoundSubKey,
                                &raw mut szSubKey as *mut WCHAR as LPCWSTR,
                            );
                            StringCbCatW(
                                *ppszFoundSubKey,
                                cbFoundSubKey,
                                &raw const s_backslash as LPCWSTR,
                            );
                        } else {
                            **ppszFoundSubKey = 0 as WCHAR;
                        }
                        wcscat(*ppszFoundSubKey, psz as *const WCHAR);
                        free(psz as *mut core::ffi::c_void);
                        jump = 2;
                        break 'walk;
                    }
                }
                i += 1;
            }

            if !ppszNames.is_null() {
                i = 0;
                while i < c {
                    free(*ppszNames.offset(i as isize) as *mut core::ffi::c_void);
                    i += 1;
                }
                free(ppszNames as *mut core::ffi::c_void);
            }
            ppszNames = core::ptr::null_mut();

            if hBaseKey != hSubKey {
                RegCloseKey(hSubKey);
            }
            // Divergence from the C: it left the closed handle in hSubKey, and the shared
            // success/err exits below closed it AGAIN once the walk moved on to the next root
            // (HKCU, HKLM, ...) -- by then possibly a handle another open had been given.
            hSubKey = hBaseKey;
        }

        if *phKey == HKEY_CLASSES_ROOT {
            *phKey = HKEY_CURRENT_USER;
            if RegFindRecurse(
                *phKey,
                &raw const s_empty as LPCWSTR,
                core::ptr::null(),
                ppszFoundSubKey,
                ppszFoundValueName,
            ) != 0
            {
                return (TRUE) as BOOL;
            }
        }

        if *phKey == HKEY_CURRENT_USER {
            *phKey = HKEY_LOCAL_MACHINE;
            if RegFindRecurse(
                *phKey,
                &raw const s_empty as LPCWSTR,
                core::ptr::null(),
                ppszFoundSubKey,
                ppszFoundValueName,
            ) != 0
            {
                jump = 2;
                break 'walk;
            }
        }

        if *phKey == HKEY_LOCAL_MACHINE {
            *phKey = HKEY_USERS;
            if RegFindRecurse(
                *phKey,
                &raw const s_empty as LPCWSTR,
                core::ptr::null(),
                ppszFoundSubKey,
                ppszFoundValueName,
            ) != 0
            {
                jump = 2;
                break 'walk;
            }
        }

        if *phKey == HKEY_USERS {
            *phKey = HKEY_CURRENT_CONFIG;
            if RegFindRecurse(
                *phKey,
                &raw const s_empty as LPCWSTR,
                core::ptr::null(),
                ppszFoundSubKey,
                ppszFoundValueName,
            ) != 0
            {
                jump = 2;
                break 'walk;
            }
        }

        jump = 1;
    }

    if jump == 2 {
        // success:
        if !ppszNames.is_null() {
            i = 0;
            while i < c {
                free(*ppszNames.offset(i as isize) as *mut core::ffi::c_void);
                i += 1;
            }
            free(ppszNames as *mut core::ffi::c_void);
        }
        if hBaseKey != hSubKey {
            RegCloseKey(hSubKey);
        }
        return (TRUE) as BOOL;
    }

    // err:
    if !ppszNames.is_null() {
        i = 0;
        while i < c {
            free(*ppszNames.offset(i as isize) as *mut core::ffi::c_void);
            i += 1;
        }
        free(ppszNames as *mut core::ffi::c_void);
    }
    if hBaseKey != hSubKey {
        RegCloseKey(hSubKey);
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn GetFindFlags() -> DWORD {
    let mut hKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut dwType: DWORD = 0;
    let mut dwValue: DWORD = 0;
    let mut cbData: DWORD = 0;
    let mut dwFlags: DWORD = RSF_LOOKATKEYS as DWORD | RSF_LOOKATVALUES as DWORD | RSF_LOOKATDATA as DWORD;

    if RegOpenKeyW(HKEY_CURRENT_USER as HKEY, &raw const g_szGeneralRegKey as LPCWSTR, &raw mut hKey) == ERROR_SUCCESS as LSTATUS
    {
        /* Retrieve flags from registry key */
        cbData = ::core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(hKey, &raw const s_szFindFlags as LPCWSTR, core::ptr::null_mut::<DWORD>(), &raw mut dwType, &raw mut dwValue as LPBYTE, &raw mut cbData) == ERROR_SUCCESS as LSTATUS
        {
            if dwType == REG_DWORD as DWORD {
                dwFlags = (dwFlags & !(0x0000FFFF as DWORD)) | ((dwValue & 0x0000FFFF as DWORD) << 0 as u32);
            }
        }

        /* Retrieve ReactOS Regedit specific flags from registry key */
        cbData = ::core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(hKey, &raw const s_szFindFlagsR as LPCWSTR, core::ptr::null_mut::<DWORD>(), &raw mut dwType, &raw mut dwValue as LPBYTE, &raw mut cbData) == ERROR_SUCCESS as LSTATUS
        {
            if dwType == REG_DWORD as DWORD {
                dwFlags = (dwFlags & !(0xFFFF0000 as DWORD)) | ((dwValue & 0x0000FFFF as DWORD) << 16 as u32);
            }
        }

        RegCloseKey(hKey);
    }
    return dwFlags;
}

unsafe extern "C" fn SetFindFlags(dwFlags: DWORD) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;
    let mut dwData: DWORD = 0;

    if (RegCreateKeyExW(
        HKEY_CURRENT_USER,
        addr_of!(g_szGeneralRegKey) as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        addr_of_mut!(hKey),
        addr_of_mut!(dwDisposition),
    )) as u32 == ERROR_SUCCESS
    {
        dwData = (dwFlags >> 0) & 0x0000FFFF;
        RegSetValueExW(
            hKey,
            addr_of!(s_szFindFlags) as LPCWSTR,
            0,
            REG_DWORD,
            addr_of!(dwData) as *const BYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );

        dwData = (dwFlags >> 16) & 0x0000FFFF;
        RegSetValueExW(
            hKey,
            addr_of!(s_szFindFlagsR) as LPCWSTR,
            0,
            REG_DWORD,
            addr_of!(dwData) as *const BYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );

        RegCloseKey(hKey);
    }
}

unsafe extern "system" fn AbortFindDialogProc(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(hDlg);

    if uMsg == WM_CLOSE {
        s_bAbort = (TRUE) as i32;
    } else if uMsg == WM_COMMAND {
        if (HIWORD!(wParam)) as u32 == BN_CLICKED {
            if (LOWORD!(wParam)) as u32 == IDCANCEL {
                s_bAbort = (TRUE) as i32;
            }
        }
    }
    return 0 as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn FindNext(hWnd: HWND) -> BOOL {
    let mut hKeyRoot: HKEY = core::ptr::null_mut();
    let mut pszKeyPath: LPCWSTR = core::ptr::null();
    let mut fSuccess: BOOL = 0;
    let mut szFullKey: [WCHAR; 512] = [0; 512];
    let mut pszValueName: LPCWSTR = core::ptr::null();
    let mut pszFoundSubKey: LPWSTR = core::ptr::null_mut();
    let mut pszFoundValueName: LPWSTR = core::ptr::null_mut();

    if wcslen(core::ptr::addr_of_mut!(s_szFindWhat) as *mut WCHAR) == 0 {
        FindDialog(hWnd);
        return (TRUE) as BOOL;
    }

    s_dwFlags = GetFindFlags();

    pszKeyPath = GetItemPath(
        (*g_pChildWnd).hTreeWnd,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKeyRoot),
    );
    if pszKeyPath.is_null() {
        hKeyRoot = HKEY_CLASSES_ROOT;
        pszKeyPath = core::ptr::addr_of!(s_empty) as *const WCHAR as LPCWSTR;
    }

    /* Create abort find dialog */
    s_hwndAbortDialog = CreateDialogW!(
        GetModuleHandle(core::ptr::null()) as HINSTANCE,
        MAKEINTRESOURCEW!(IDD_FINDING) as LPCWSTR,
        hWnd,
        Some(AbortFindDialogProc),
    );
    if !s_hwndAbortDialog.is_null() {
        ShowWindow(s_hwndAbortDialog, (SW_SHOW) as i32);
        UpdateWindow(s_hwndAbortDialog);
    }
    s_bAbort = (FALSE) as i32;

    pszValueName = GetValueName((*g_pChildWnd).hListWnd, -1) as LPCWSTR;

    EnableWindow(hFrameWnd, (FALSE) as BOOL);
    EnableWindow((*g_pChildWnd).hTreeWnd, (FALSE) as BOOL);
    EnableWindow((*g_pChildWnd).hListWnd, (FALSE) as BOOL);
    EnableWindow((*g_pChildWnd).hAddressBarWnd, (FALSE) as BOOL);

    fSuccess = RegFindWalk(
        core::ptr::addr_of_mut!(hKeyRoot),
        pszKeyPath,
        pszValueName,
        core::ptr::addr_of_mut!(pszFoundSubKey),
        core::ptr::addr_of_mut!(pszFoundValueName),
    );

    EnableWindow(hFrameWnd, (TRUE) as BOOL);
    EnableWindow((*g_pChildWnd).hTreeWnd, (TRUE) as BOOL);
    EnableWindow((*g_pChildWnd).hListWnd, (TRUE) as BOOL);
    EnableWindow((*g_pChildWnd).hAddressBarWnd, (TRUE) as BOOL);

    if !s_hwndAbortDialog.is_null() {
        DestroyWindow(s_hwndAbortDialog);
        s_hwndAbortDialog = core::ptr::null_mut();
    }

    if fSuccess != 0 {
        GetKeyName(
            core::ptr::addr_of_mut!(szFullKey) as LPWSTR,
            ARRAY_SIZE!(szFullKey),
            hKeyRoot,
            pszFoundSubKey as LPCWSTR,
        );
        SelectNode(
            (*g_pChildWnd).hTreeWnd,
            core::ptr::addr_of_mut!(szFullKey) as *mut WCHAR as LPCWSTR,
        );
        free(pszFoundSubKey as *mut core::ffi::c_void);

        if !pszFoundValueName.is_null() {
            SetValueName((*g_pChildWnd).hListWnd, pszFoundValueName as LPCWSTR);
            free(pszFoundValueName as *mut core::ffi::c_void);
            SetFocus((*g_pChildWnd).hListWnd);
        } else {
            SetFocus((*g_pChildWnd).hTreeWnd);
        }
    }
    return (fSuccess != 0 || s_bAbort != 0) as BOOL;
}

unsafe extern "system" fn FindDialogProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut iResult: INT_PTR = 0 as INT_PTR;
    let mut hControl: HWND = core::ptr::null_mut();
    let mut lStyle: LONG = 0;
    let mut dwFlags: DWORD = 0;
    static mut s_szSavedFindValue: [WCHAR; 256] = [0; 256];

    match uMsg {
        WM_INITDIALOG => {
            dwFlags = GetFindFlags();

            hControl = GetDlgItem(hDlg, (IDC_LOOKAT_KEYS as i32));
            if !hControl.is_null() {
                SendMessageW(hControl, BM_SETCHECK, if (dwFlags & RSF_LOOKATKEYS) != 0 { (TRUE) as _ } else { (FALSE) as _ }, 0);
            }

            hControl = GetDlgItem(hDlg, (IDC_LOOKAT_VALUES as i32));
            if !hControl.is_null() {
                SendMessageW(hControl, BM_SETCHECK, if (dwFlags & RSF_LOOKATVALUES) != 0 { (TRUE) as _ } else { (FALSE) as _ }, 0);
            }

            hControl = GetDlgItem(hDlg, (IDC_LOOKAT_DATA as i32));
            if !hControl.is_null() {
                SendMessageW(hControl, BM_SETCHECK, if (dwFlags & RSF_LOOKATDATA) != 0 { (TRUE) as _ } else { (FALSE) as _ }, 0);
            }

            /* Match whole string */
            hControl = GetDlgItem(hDlg, (IDC_MATCHSTRING as i32));
            if !hControl.is_null() {
                SendMessageW(hControl, BM_SETCHECK, if (dwFlags & RSF_WHOLESTRING) != 0 { (TRUE) as _ } else { (FALSE) as _ }, 0);
            }

            /* Case sensitivity */
            hControl = GetDlgItem(hDlg, (IDC_MATCHCASE as i32));
            if !hControl.is_null() {
                SendMessageW(hControl, BM_SETCHECK, if (dwFlags & RSF_MATCHCASE) != 0 { (TRUE) as _ } else { (FALSE) as _ }, 0);
            }

            hControl = GetDlgItem(hDlg, (IDC_FINDWHAT as i32));
            if !hControl.is_null() {
                SetWindowTextW(hControl, core::ptr::addr_of_mut!(s_szSavedFindValue) as *mut WCHAR);
                SetFocus(hControl);
                SendMessageW(hControl, EM_SETSEL, 0, -1);
            }
        }

        WM_CLOSE => {
            EndDialog(hDlg, 0);
        }

        WM_COMMAND => {
            match (HIWORD!(wParam)) as u32 {
                BN_CLICKED => {
                    match (LOWORD!(wParam)) as u32 {
                        IDOK => {
                            dwFlags = 0;

                            hControl = GetDlgItem(hDlg, (IDC_LOOKAT_KEYS as i32));
                            if !hControl.is_null() && ((SendMessageW(hControl, BM_GETCHECK, 0, 0)) as u32 == (BST_CHECKED) as _) {
                                dwFlags |= RSF_LOOKATKEYS;
                            }

                            hControl = GetDlgItem(hDlg, (IDC_LOOKAT_VALUES as i32));
                            if !hControl.is_null() && ((SendMessageW(hControl, BM_GETCHECK, 0, 0)) as u32 == (BST_CHECKED) as _) {
                                dwFlags |= RSF_LOOKATVALUES;
                            }

                            hControl = GetDlgItem(hDlg, (IDC_LOOKAT_DATA as i32));
                            if !hControl.is_null() && ((SendMessageW(hControl, BM_GETCHECK, 0, 0)) as u32 == (BST_CHECKED) as _) {
                                dwFlags |= RSF_LOOKATDATA;
                            }

                            hControl = GetDlgItem(hDlg, (IDC_MATCHSTRING as i32));
                            if !hControl.is_null() && ((SendMessageW(hControl, BM_GETCHECK, 0, 0)) as u32 == (BST_CHECKED) as _) {
                                dwFlags |= RSF_WHOLESTRING;
                            }

                            hControl = GetDlgItem(hDlg, (IDC_MATCHCASE as i32));
                            if !hControl.is_null() && ((SendMessageW(hControl, BM_GETCHECK, 0, 0)) as u32 == (BST_CHECKED) as _) {
                                dwFlags |= RSF_MATCHCASE;
                            }

                            SetFindFlags(dwFlags);

                            hControl = GetDlgItem(hDlg, (IDC_FINDWHAT as i32));
                            if !hControl.is_null() {
                                GetWindowTextW(hControl, core::ptr::addr_of_mut!(s_szFindWhat) as *mut WCHAR, (ARRAY_SIZE!(s_szFindWhat)) as i32);
                            }
                            EndDialog(hDlg, 1);
                        }

                        IDCANCEL => {
                            EndDialog(hDlg, 0);
                        }

                        _ => {}
                    }
                }

                EN_CHANGE => {
                    match (LOWORD!(wParam)) as u32 {
                        IDC_FINDWHAT => {
                            GetWindowTextW(lParam as HWND, core::ptr::addr_of_mut!(s_szSavedFindValue) as *mut WCHAR, (ARRAY_SIZE!(s_szSavedFindValue)) as i32);
                            hControl = GetDlgItem(hDlg, (IDOK) as i32);
                            if !hControl.is_null() {
                                lStyle = (GetWindowLongPtr(hControl, GWL_STYLE)) as i32;
                                if s_szSavedFindValue[0] != 0 {
                                    lStyle &= (!WS_DISABLED) as i32;
                                } else {
                                    lStyle |= (WS_DISABLED) as i32;
                                }
                                SetWindowLongPtr(hControl, GWL_STYLE, (lStyle) as _);
                                RedrawWindow(hControl, core::ptr::null_mut(), core::ptr::null_mut(), RDW_INVALIDATE);
                            }
                        }

                        _ => {}
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }
    return iResult;
}
#[no_mangle]
pub unsafe extern "C" fn FindNextMessageBox(hWnd: HWND) {
    if FindNext(hWnd) == 0 {
        let mut msg: [WCHAR; 128] = [0; 128];
        let mut caption: [WCHAR; 128] = [0; 128];

        LoadStringW(
            hInst,
            IDS_FINISHEDFIND as UINT,
            core::ptr::addr_of_mut!(msg) as LPWSTR,
            ARRAY_SIZE!(msg) as ::core::ffi::c_int,
        );
        LoadStringW(
            hInst,
            IDS_APP_TITLE as UINT,
            core::ptr::addr_of_mut!(caption) as LPWSTR,
            ARRAY_SIZE!(caption) as ::core::ffi::c_int,
        );
        MessageBoxW(
            hWnd,
            core::ptr::addr_of_mut!(msg) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(caption) as *mut WCHAR as LPCWSTR,
            MB_ICONINFORMATION as UINT,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn FindDialog(hWnd: HWND) {
    if DialogBoxParamW(GetModuleHandle(core::ptr::null_mut()) as HINSTANCE, MAKEINTRESOURCEW!(IDD_FIND) as LPCWSTR, hWnd, Some(FindDialogProc), 0) != 0 {
        FindNextMessageBox(hWnd);
    }
}
