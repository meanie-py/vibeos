// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1B08E6B1E928: [u16; 46] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x004E, 0x006C, 0x0073, 0x005C, 0x004C, 0x0061, 0x006E,
    0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut _LocaleList: *mut LOCALE_LIST_NODE =
    ::core::ptr::null::<LOCALE_LIST_NODE>() as *mut LOCALE_LIST_NODE;

unsafe extern "C" fn LocaleList_Append(mut dwId: DWORD, mut pszName: *const WCHAR) -> *mut LOCALE_LIST_NODE {
    let mut pCurrent: *mut LOCALE_LIST_NODE = core::ptr::null_mut();
    let mut pNew: *mut LOCALE_LIST_NODE = core::ptr::null_mut();

    if pszName.is_null() {
        return core::ptr::null_mut();
    }

    pCurrent = _LocaleList;

    pNew = malloc(core::mem::size_of::<LOCALE_LIST_NODE>()) as *mut LOCALE_LIST_NODE;
    if pNew.is_null() {
        return core::ptr::null_mut();
    }

    ZeroMemory!(pNew as *mut core::ffi::c_void, core::mem::size_of::<LOCALE_LIST_NODE>());

    (*pNew).pszName = _wcsdup(pszName) as *mut WCHAR;
    if (*pNew).pszName.is_null() {
        free(pNew as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*pNew).dwId = dwId;

    if pCurrent.is_null() {
        _LocaleList = pNew;
    } else {
        while !(*pCurrent).pNext.is_null() {
            pCurrent = (*pCurrent).pNext as *mut LOCALE_LIST_NODE;
        }

        (*pNew).pPrev = pCurrent as _;
        (*pCurrent).pNext = pNew as _;
    }

    return pNew;
}
#[no_mangle]
pub unsafe extern "C" fn LocaleList_Destroy() {
    let mut pCurrent: *mut LOCALE_LIST_NODE = core::ptr::null_mut();
    if _LocaleList.is_null() {
        return;
    }
    pCurrent = _LocaleList;
    while !pCurrent.is_null() {
        let pNext: *mut LOCALE_LIST_NODE = (*pCurrent).pNext;
        free((*pCurrent).pszName as *mut core::ffi::c_void);
        free(pCurrent as *mut core::ffi::c_void);
        pCurrent = pNext;
    }
    _LocaleList = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn LocaleList_Create() -> *mut LOCALE_LIST_NODE {
    let mut szValue: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwIndex: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    if (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_1B08E6B1E928.as_ptr(),
        0,
        KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE,
        &raw mut hKey,
    )) as u32 != ERROR_SUCCESS
    {
        return core::ptr::null_mut();
    }

    dwSize = ARRAYSIZE!(szValue) as DWORD;
    dwIndex = 0;

    while (RegEnumValueW(
        hKey,
        dwIndex,
        &raw mut szValue as LPWSTR,
        &raw mut dwSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32 == ERROR_SUCCESS
    {
        let mut szName: [WCHAR; (MAX_STR_LEN) as _] = [0; (MAX_STR_LEN) as _];
        let mut dwId: DWORD = 0;

        dwId = DWORDfromString(&raw mut szValue as *mut WCHAR);

        if GetLocaleInfoW(
            LOWORD!(dwId) as LCID,
            LOCALE_SLANGUAGE,
            &raw mut szName as LPWSTR,
            ARRAYSIZE!(szName) as core::ffi::c_int,
        ) != 0
        {
            LocaleList_Append(dwId, &raw mut szName as *mut WCHAR);
        }

        dwSize = ARRAYSIZE!(szValue) as DWORD;
        dwIndex = dwIndex.wrapping_add(1);
    }

    RegCloseKey(hKey);

    return _LocaleList;
}
#[no_mangle]
pub unsafe extern "C" fn LocaleList_GetByHkl(hkl: HKL) -> *mut LOCALE_LIST_NODE {
    let mut pCurrent: *mut LOCALE_LIST_NODE = core::ptr::null_mut();
    pCurrent = _LocaleList;
    while !pCurrent.is_null() {
        if LOWORD!((*pCurrent).dwId as DWORD) == LOWORD!(hkl as usize as DWORD) {
            return pCurrent;
        }
        pCurrent = (*pCurrent).pNext as *mut LOCALE_LIST_NODE;
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn LocaleList_GetFirst() -> *mut LOCALE_LIST_NODE {
    return _LocaleList;
}
