use crate::input::input_list::GetSystemLibraryPath;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2A3A3E69588F: [u16; 8] = [0x0073u16, 0x0068, 0x006C, 0x0077, 0x0061, 0x0070, 0x0069, 0x0000];
pub static RUSTCONV_WSTR_464EEE2E93B7: [u16; 10] = [0x004Cu16, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0049, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_65DF0D412656: [u16; 12] = [
    0x004Cu16, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_6F4B49BAF187: [u16; 12] = [
    0x004Cu16, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0054, 0x0065, 0x0078,
    0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_8AF36D3C73B6: [u16; 9] = [0x0049u16, 0x004D, 0x0045, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_D7727E89129A: [u16; 50] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x004B, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072,
    0x0064, 0x0020, 0x004C, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_FCC5F82B6F58: [u16; 20] = [
    0x004Cu16, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0044, 0x0069, 0x0073,
    0x0070, 0x006C, 0x0061, 0x0079, 0x0020, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut _LayoutList: *mut LAYOUT_LIST_NODE =
    ::core::ptr::null::<LAYOUT_LIST_NODE>() as *mut LAYOUT_LIST_NODE;

pub type FN_SHLoadRegUIStringW =
    unsafe extern "system" fn(HKEY, LPCWSTR, LPWSTR, DWORD) -> HRESULT;

unsafe extern "C" fn LayoutList_AppendNode(
    mut dwKLID: DWORD,
    mut wSpecialId: WORD,
    mut pszFile: LPCWSTR,
    mut pszName: LPCWSTR,
    mut pszImeFile: LPCWSTR,
) -> *mut LAYOUT_LIST_NODE {
    let mut pCurrent: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();
    let mut pNew: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();

    if pszName.is_null() {
        return core::ptr::null_mut();
    }

    pCurrent = _LayoutList;

    pNew = malloc(core::mem::size_of::<LAYOUT_LIST_NODE>() as usize) as *mut LAYOUT_LIST_NODE;
    if pNew.is_null() {
        return core::ptr::null_mut();
    }

    ZeroMemory!(
        pNew as *mut ::core::ffi::c_void,
        core::mem::size_of::<LAYOUT_LIST_NODE>() as usize,
    );

    (*pNew).dwKLID = dwKLID;
    (*pNew).wSpecialId = wSpecialId;

    (*pNew).pszName = _wcsdup(pszName as *const u16) as LPWSTR;
    (*pNew).pszFile = _wcsdup(pszFile as *const u16) as LPWSTR;
    (*pNew).pszImeFile = _wcsdup(pszImeFile as *const u16) as LPWSTR;
    if (*pNew).pszName.is_null()
        || (*pNew).pszFile.is_null()
        || !pszImeFile.is_null() && (*pNew).pszImeFile.is_null()
    {
        free((*pNew).pszName as *mut ::core::ffi::c_void);
        free((*pNew).pszFile as *mut ::core::ffi::c_void);
        free((*pNew).pszImeFile as *mut ::core::ffi::c_void);
        free(pNew as *mut ::core::ffi::c_void);
        return core::ptr::null_mut();
    }

    if pCurrent.is_null() {
        _LayoutList = pNew;
    } else {
        while !(*pCurrent).pNext.is_null() {
            pCurrent = (*pCurrent).pNext as *mut LAYOUT_LIST_NODE;
        }

        (*pNew).pPrev = pCurrent as *mut _LAYOUT_LIST_NODE;
        (*pCurrent).pNext = pNew as *mut _LAYOUT_LIST_NODE;
    }

    return pNew;
}
#[no_mangle]
pub unsafe extern "C" fn LayoutList_Destroy() {
    let mut pCurrent: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();
    let mut pNext: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();
    if _LayoutList.is_null() {
        return;
    }
    pCurrent = _LayoutList;
    while !pCurrent.is_null() {
        pNext = (*pCurrent).pNext as *mut LAYOUT_LIST_NODE;
        free((*pCurrent).pszName as *mut core::ffi::c_void);
        free((*pCurrent).pszFile as *mut core::ffi::c_void);
        free((*pCurrent).pszImeFile as *mut core::ffi::c_void);
        free(pCurrent as *mut core::ffi::c_void);
        pCurrent = pNext;
    }
    _LayoutList = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn FakeSHLoadRegUIStringW(
    mut hkey: HKEY,
    mut value: LPCWSTR,
    mut buf: LPWSTR,
    mut size: DWORD,
) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    let mut hSHLWAPI: HINSTANCE = LoadLibraryW(win32macros_rust::wide_bytes!(b"s\0h\0l\0w\0a\0p\0i\0\0\0"));
    // C: `fn = (FN_SHLoadRegUIStringW)GetProcAddress(..); if (fn) hr = fn(..);`. The island
    // spells FN_SHLoadRegUIStringW as a bare fn pointer, so the NULL test is FARPROC's.
    if let Some(p) = GetProcAddress(hSHLWAPI, (439 as INT_PTR) as LPCSTR) {
        let r#fn: FN_SHLoadRegUIStringW = ::core::mem::transmute(p);
        hr = r#fn(hkey, value, buf, size);
    }
    FreeLibrary(hSHLWAPI as HMODULE);
    return hr;
}

unsafe extern "C" fn LayoutList_ReadLayout(hLayoutKey: HKEY, szKLID: LPCWSTR, szSystemDirectory: LPCWSTR) -> BOOL {
    let mut szFile: [WCHAR; 80] = [0; 80];
    let mut szImeFile: [WCHAR; 80] = [0; 80];
    let mut szBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFilePath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwSize: DWORD = 0;
    let mut dwKLID: DWORD = DWORDfromString(szKLID as *const WCHAR);
    let mut wSpecialId: WORD = 0;
    let mut pszImeFile: LPWSTR = core::ptr::null_mut::<WCHAR>();

    dwSize = core::mem::size_of::<[WCHAR; 80]>() as DWORD;
    if (RegQueryValueExW(
        hLayoutKey,
        RUSTCONV_WSTR_65DF0D412656.as_ptr(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        &raw mut szFile as *mut WCHAR as LPBYTE,
        &raw mut dwSize,
    )) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL; /* No "Layout File" value */
    }

    if IS_IME_KLID!(dwKLID) != 0 {
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        dwSize = core::mem::size_of::<[WCHAR; 80]>() as DWORD;
        if (RegQueryValueExW(
            hLayoutKey,
            RUSTCONV_WSTR_8AF36D3C73B6.as_ptr(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            &raw mut szImeFile as *mut WCHAR as LPBYTE,
            &raw mut dwSize,
        )) as u32 != ERROR_SUCCESS
        {
            return (FALSE) as BOOL; /* No "IME File" value */
        }

        if !wcschr(&raw mut szImeFile as *mut WCHAR, 0x005Cu16).is_null() {
            return (FALSE) as BOOL; /* Invalid character */
        }

        GetSystemLibraryPath(
            &raw mut szPath as *mut WCHAR,
            (ARRAYSIZE!(szPath)) as INT,
            &raw mut szImeFile as *mut WCHAR,
        );
        if GetFileAttributesW(&raw mut szPath as *mut WCHAR) == INVALID_FILE_ATTRIBUTES {
            return (FALSE) as BOOL; /* Does not exist */
        }

        pszImeFile = &raw mut szImeFile as *mut WCHAR as LPWSTR;
    }

    /* Build the "Layout File" full path and check existence */
    StringCchPrintfW!(
        &raw mut szFilePath as *mut WCHAR,
        ARRAYSIZE!(szFilePath),
        RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
        szSystemDirectory,
        &raw mut szFile as *mut WCHAR,
    );
    if GetFileAttributesW(&raw mut szFilePath as *mut WCHAR) == INVALID_FILE_ATTRIBUTES {
        return (FALSE) as BOOL; /* No layout file found */
    }

    /* Get the special ID */
    dwSize = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    if (RegQueryValueExW(
        hLayoutKey,
        RUSTCONV_WSTR_464EEE2E93B7.as_ptr(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        &raw mut szBuffer as *mut WCHAR as LPBYTE,
        &raw mut dwSize,
    )) as u32 == ERROR_SUCCESS
    {
        wSpecialId = LOWORD!(DWORDfromString(&raw mut szBuffer as *mut WCHAR));
    }

    /* If there is a valid "Layout Display Name", then use it as the entry name */
    if FakeSHLoadRegUIStringW(
        hLayoutKey,
        RUSTCONV_WSTR_FCC5F82B6F58.as_ptr(),
        &raw mut szBuffer as *mut WCHAR,
        ARRAYSIZE!(szBuffer),
    ) == S_OK
    {
        LayoutList_AppendNode(
            dwKLID,
            wSpecialId,
            &raw mut szFile as *mut WCHAR as LPCWSTR,
            &raw mut szBuffer as *mut WCHAR as LPCWSTR,
            pszImeFile as LPCWSTR,
        );
        return (TRUE) as BOOL;
    }

    /* Otherwise, use "Layout Text" value as the entry name */
    dwSize = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    if (RegQueryValueExW(
        hLayoutKey,
        RUSTCONV_WSTR_6F4B49BAF187.as_ptr(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        &raw mut szBuffer as *mut WCHAR as LPBYTE,
        &raw mut dwSize,
    )) as u32 == ERROR_SUCCESS
    {
        LayoutList_AppendNode(
            dwKLID,
            wSpecialId,
            &raw mut szFile as *mut WCHAR as LPCWSTR,
            &raw mut szBuffer as *mut WCHAR as LPCWSTR,
            pszImeFile as LPCWSTR,
        );
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LayoutList_Create() {
    let mut szSystemDirectory: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szKLID: [WCHAR; (KL_NAMELENGTH) as _] = [0; (KL_NAMELENGTH) as _];
    let mut dwSize: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hLayoutKey: HKEY = core::ptr::null_mut();

    if GetSystemDirectoryW(
        core::ptr::addr_of_mut!(szSystemDirectory) as LPWSTR,
        ARRAYSIZE!(szSystemDirectory) as UINT,
    ) == 0
    {
        return;
    }

    if (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_D7727E89129A.as_ptr() as LPCWSTR,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    )) as u32 != ERROR_SUCCESS
    {
        return;
    }

    dwIndex = 0;
    loop {
        dwSize = ARRAYSIZE!(szKLID) as DWORD;
        if (RegEnumKeyExW(
            hKey,
            dwIndex,
            core::ptr::addr_of_mut!(szKLID) as LPWSTR,
            core::ptr::addr_of_mut!(dwSize),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32 != ERROR_SUCCESS
        {
            break;
        }

        if (RegOpenKeyExW(
            hKey,
            core::ptr::addr_of_mut!(szKLID) as LPCWSTR,
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hLayoutKey),
        )) as u32 == ERROR_SUCCESS
        {
            LayoutList_ReadLayout(
                hLayoutKey,
                core::ptr::addr_of_mut!(szKLID) as LPCWSTR,
                core::ptr::addr_of_mut!(szSystemDirectory) as LPCWSTR,
            );
            RegCloseKey(hLayoutKey);
        }

        dwIndex = dwIndex.wrapping_add(1);
    }

    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn LayoutList_GetByHkl(mut hkl: HKL) -> *mut LAYOUT_LIST_NODE {
    let mut pCurrent: *mut LAYOUT_LIST_NODE = core::ptr::null_mut();

    if IS_SPECIAL_HKL!(hkl) != 0 {
        let wSpecialId: WORD = SPECIALIDFROMHKL!(hkl);

        pCurrent = _LayoutList as *mut LAYOUT_LIST_NODE;
        while !pCurrent.is_null() {
            if wSpecialId == (*pCurrent).wSpecialId {
                return pCurrent;
            }
            pCurrent = (*pCurrent).pNext as *mut LAYOUT_LIST_NODE;
        }
    } else if IS_IME_HKL!(hkl) != 0 {
        pCurrent = _LayoutList as *mut LAYOUT_LIST_NODE;
        while !pCurrent.is_null() {
            if hkl == UlongToHandle!((*pCurrent).dwKLID) as HKL {
                return pCurrent;
            }
            pCurrent = (*pCurrent).pNext as *mut LAYOUT_LIST_NODE;
        }
    } else {
        pCurrent = _LayoutList as *mut LAYOUT_LIST_NODE;
        while !pCurrent.is_null() {
            if HIWORD!(hkl as DWORD) == LOWORD!((*pCurrent).dwKLID) {
                return pCurrent;
            }
            pCurrent = (*pCurrent).pNext as *mut LAYOUT_LIST_NODE;
        }
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn LayoutList_GetFirst() -> *mut LAYOUT_LIST_NODE {
    return _LayoutList;
}
