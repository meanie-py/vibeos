use crate::shlwapi::ordinal::SHStringFromGUIDW;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_47EEE22AAF35: [u16; 58] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072,
    0x005C, 0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_6E342AB1A0F1: [u16; 7] = [0x0043u16, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn RegQueryInfoKeyA( hKey: HKEY, lpClass: LPSTR, lpcchClass: LPDWORD, lpReserved: LPDWORD, lpcSubKeys: LPDWORD, lpcbMaxSubKeyLen: LPDWORD, lpcbMaxClassLen: LPDWORD, lpcValues: LPDWORD, lpcbMaxValueNameLen: LPDWORD, lpcbMaxValueLen: LPDWORD, lpcbSecurityDescriptor: LPDWORD, lpftLastWriteTime: *mut FILETIME, ) -> LONG;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut lpszContentTypeA: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"Content Type\0") };

static mut lpszContentTypeW: [WCHAR; 13] = [
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut szMimeDbContentA: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(
        *b"MIME\\Database\\Content Type\\\0",
    )
};

static mut szMimeDbContentW: [WCHAR; 28] = [
    'M' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut dwLenMimeDbContent: DWORD = 27 as DWORD;

static mut szExtensionA: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"Extension\0") };

static mut szExtensionW: [WCHAR; 10] = [
    'E' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];


#[repr(C)]
pub struct SHUSKEY {
    pub HKCUstart: HKEY,
    pub HKCUkey: HKEY,
    pub HKLMstart: HKEY,
    pub HKLMkey: HKEY,
    pub lpszPath: [WCHAR; (MAX_PATH) as _],
}
pub type LPSHUSKEY = *mut SHUSKEY;

pub const REG_HKCU: i32 = 1;
pub const REG_HKLM: i32 = 0;

unsafe extern "C" fn REG_GetHKEYFromHUSKEY(mut hUSKey: HUSKEY, mut which: BOOL) -> HKEY {
    let mut test: HKEY = hUSKey as HKEY;
    let mut mihk: LPSHUSKEY = hUSKey as LPSHUSKEY;
    if test == HKEY_CLASSES_ROOT
        || test == HKEY_CURRENT_CONFIG
        || test == HKEY_CURRENT_USER
        || test == HKEY_DYN_DATA
        || test == HKEY_LOCAL_MACHINE
        || test == HKEY_PERFORMANCE_DATA
        || test == HKEY_USERS
    {
        return test;
    }
    if which == REG_HKCU {
        return (*mihk).HKCUkey;
    }
    return (*mihk).HKLMkey;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegOpenUSKeyA(Path: LPCSTR, AccessType: REGSAM, hRelativeUSKey: HUSKEY, phNewUSKey: PHUSKEY, fIgnoreHKCU: BOOL) -> LONG {
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    if !Path.is_null() {
        MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, Path, -(1 as ::core::ffi::c_int), core::ptr::addr_of_mut!(szPath) as LPWSTR, MAX_PATH as ::core::ffi::c_int);
    }
    return SHRegOpenUSKeyW(
        (if !Path.is_null() {
            core::ptr::addr_of_mut!(szPath) as *mut WCHAR
        } else {
            core::ptr::null_mut::<WCHAR>()
        }) as LPCWSTR,
        AccessType,
        hRelativeUSKey,
        phNewUSKey,
        fIgnoreHKCU,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHRegOpenUSKeyW(
    mut Path: LPCWSTR,
    mut AccessType: REGSAM,
    mut hRelativeUSKey: HUSKEY,
    mut phNewUSKey: PHUSKEY,
    mut fIgnoreHKCU: BOOL,
) -> LONG {
    let mut ret2: LONG = 0;
    let mut ret1: LONG = (!ERROR_SUCCESS) as LONG;
    let mut hKey: LPSHUSKEY = core::ptr::null_mut::<SHUSKEY>();

    WINE_TRACE!("(%s,0x%x,%p,%p,%d)\n", debugstr_w(Path as *const WCHAR), AccessType as LONG, hRelativeUSKey, phNewUSKey, fIgnoreHKCU);

    if !phNewUSKey.is_null() {
        *phNewUSKey = core::ptr::null_mut() as HUSKEY;
    }

    /* Create internal HUSKEY */
    hKey = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<SHUSKEY>() as SIZE_T,
    ) as LPSHUSKEY;
    lstrcpynW(
        &raw mut (*hKey).lpszPath as LPWSTR,
        Path,
        (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
    );

    if !hRelativeUSKey.is_null() {
        (*hKey).HKCUstart = SHRegDuplicateHKey(REG_GetHKEYFromHUSKEY(
            hRelativeUSKey,
            REG_HKCU as BOOL,
        ));
        (*hKey).HKLMstart = SHRegDuplicateHKey(REG_GetHKEYFromHUSKEY(
            hRelativeUSKey,
            REG_HKLM as BOOL,
        ));
        /* FIXME: if either of these keys is NULL, create the start key from
         *        the relative keys start+path
         */
    } else {
        (*hKey).HKCUstart = HKEY_CURRENT_USER as HKEY;
        (*hKey).HKLMstart = HKEY_LOCAL_MACHINE as HKEY;
    }

    if fIgnoreHKCU == 0 {
        ret1 = RegOpenKeyExW(
            (*hKey).HKCUstart,
            &raw mut (*hKey).lpszPath as *mut WCHAR as LPCWSTR,
            0 as DWORD,
            AccessType,
            &raw mut (*hKey).HKCUkey,
        ) as LONG;
        if ret1 != 0 {
            (*hKey).HKCUkey = 0 as HKEY;
        }
    }

    ret2 = RegOpenKeyExW(
        (*hKey).HKLMstart,
        &raw mut (*hKey).lpszPath as *mut WCHAR as LPCWSTR,
        0 as DWORD,
        AccessType,
        &raw mut (*hKey).HKLMkey,
    ) as LONG;
    if ret2 != 0 {
        (*hKey).HKLMkey = 0 as HKEY;
    }

    if ret1 != 0 || ret2 != 0 {
        WINE_TRACE!("one or more opens failed: HKCU=%d HKLM=%d\n", ret1, ret2);
    }

    if ret1 != 0 && ret2 != 0 {
        /* Neither open succeeded: fail */
        SHRegCloseUSKey(hKey as HUSKEY);
        return ret2;
    }

    WINE_TRACE!("HUSKEY=%p\n", hKey);

    if !phNewUSKey.is_null() {
        *phNewUSKey = hKey as HUSKEY;
    }
    return ERROR_SUCCESS as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegCloseUSKey(mut hUSKey: HUSKEY) -> LONG {
    let mut hKey: LPSHUSKEY = hUSKey as LPSHUSKEY;
    let mut ret: LONG = ERROR_SUCCESS as LONG;
    if hKey.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    if !(*hKey).HKCUkey.is_null() {
        ret = RegCloseKey((*hKey).HKCUkey) as LONG;
    }
    if !(*hKey).HKCUstart.is_null() && (*hKey).HKCUstart != HKEY_CURRENT_USER {
        ret = RegCloseKey((*hKey).HKCUstart) as LONG;
    }
    if !(*hKey).HKLMkey.is_null() {
        ret = RegCloseKey((*hKey).HKLMkey) as LONG;
    }
    if !(*hKey).HKLMstart.is_null() && (*hKey).HKLMstart != HKEY_LOCAL_MACHINE {
        ret = RegCloseKey((*hKey).HKLMstart) as LONG;
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, hKey as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegCreateUSKeyA(
    path: LPCSTR,
    samDesired: REGSAM,
    relative_key: HUSKEY,
    new_uskey: PHUSKEY,
    flags: DWORD,
) -> LONG {
    let mut pathW: *mut WCHAR;
    let mut ret: LONG;

    if !path.is_null() {
        let len: INT = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, path, -1, ::core::ptr::null_mut(), 0 as INT) as INT;
        pathW = HeapAlloc(GetProcessHeap(), 0 as DWORD, (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as *mut WCHAR;
        if pathW.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
        MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, path, -1, pathW as LPWSTR, len as INT);
    } else {
        pathW = ::core::ptr::null_mut();
    }

    ret = SHRegCreateUSKeyW(pathW as LPCWSTR, samDesired, relative_key, new_uskey, flags);
    HeapFree(GetProcessHeap(), 0 as DWORD, pathW as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegCreateUSKeyW(
    mut path: LPCWSTR,
    mut samDesired: REGSAM,
    mut relative_key: HUSKEY,
    mut new_uskey: PHUSKEY,
    mut flags: DWORD,
) -> LONG {
    let mut ret: LONG = ERROR_CALL_NOT_IMPLEMENTED as LONG;
    let mut ret_key: *mut SHUSKEY = core::ptr::null_mut::<SHUSKEY>();
    WINE_TRACE!("(%s, 0x%08x, %p, %p, 0x%08x)\n", debugstr_w(path as *const WCHAR), samDesired, relative_key, new_uskey, flags);
    if new_uskey.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }
    *new_uskey = core::ptr::null_mut();
    if (flags & !(SHREGSET_FORCE_HKCU as DWORD)) != 0 {
        WINE_FIXME!("unsupported flags 0x%08x\n", flags);
        return ERROR_SUCCESS as LONG;
    }
    ret_key = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<SHUSKEY>() as SIZE_T,
    ) as *mut SHUSKEY;
    lstrcpynW(
        core::ptr::addr_of_mut!((*ret_key).lpszPath) as LPWSTR,
        path,
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as i32,
    );
    if !relative_key.is_null() {
        (*ret_key).HKCUstart = SHRegDuplicateHKey(REG_GetHKEYFromHUSKEY(
            relative_key,
            REG_HKCU,
        ));
        (*ret_key).HKLMstart = SHRegDuplicateHKey(REG_GetHKEYFromHUSKEY(
            relative_key,
            REG_HKLM,
        ));
    } else {
        (*ret_key).HKCUstart = HKEY_CURRENT_USER;
        (*ret_key).HKLMstart = HKEY_LOCAL_MACHINE;
    }
    if (flags & (SHREGSET_FORCE_HKCU as DWORD)) != 0 {
        ret = RegCreateKeyExW(
            (*ret_key).HKCUstart,
            path,
            0 as DWORD,
            core::ptr::null_mut::<WCHAR>(),
            0 as DWORD,
            samDesired,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            core::ptr::addr_of_mut!((*ret_key).HKCUkey),
            core::ptr::null_mut::<DWORD>(),
        ) as LONG;
        if ret == ERROR_SUCCESS as LONG {
            *new_uskey = ret_key as HUSKEY;
        } else {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                ret_key as PVOID,
            );
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegDeleteEmptyUSKeyA(
    hUSKey: HUSKEY,
    pszValue: LPCSTR,
    delRegFlags: SHREGDEL_FLAGS,
) -> LONG {
    // FIXME("(%p, %s, 0x%08x) stub\n", hUSKey, debugstr_a(pszValue), delRegFlags);
    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SHRegDeleteEmptyUSKeyW(
    hUSKey: HUSKEY,
    pszValue: LPCWSTR,
    delRegFlags: SHREGDEL_FLAGS,
) -> LONG {
    // FIXME("(%p, %s, 0x%08x) stub\n", hUSKey, debugstr_w(pszValue), delRegFlags);
    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SHRegDeleteUSValueA(
    hUSKey: HUSKEY,
    pszValue: LPCSTR,
    delRegFlags: SHREGDEL_FLAGS,
) -> LONG {
    // FIXME("(%p, %s, 0x%08x) stub\n", hUSKey, debugstr_a(pszValue), delRegFlags);
    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SHRegDeleteUSValueW(
    hUSKey: HUSKEY,
    pszValue: LPCWSTR,
    delRegFlags: SHREGDEL_FLAGS,
) -> LONG {
    // FIXME("(%p, %s, 0x%08x) stub\n", hUSKey, debugstr_w(pszValue), delRegFlags);
    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SHRegEnumUSValueA(
    hUSKey: HUSKEY,
    dwIndex: DWORD,
    pszValueName: LPSTR,
    pcchValueNameLen: LPDWORD,
    pdwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
    enumRegFlags: SHREGENUM_FLAGS,
) -> LONG {
    let mut dokey: HKEY;

    if (enumRegFlags == (SHREGENUM_HKCU as SHREGENUM_FLAGS) || enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))
        && {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKCU);
            !dokey.is_null()
        }
    {
        return RegEnumValueA(
            dokey,
            dwIndex,
            pszValueName,
            pcchValueNameLen,
            core::ptr::null_mut(),
            pdwType,
            pvData as LPBYTE,
            pcbData,
        ) as LONG;
    }

    if (enumRegFlags == (SHREGENUM_HKLM as SHREGENUM_FLAGS) || enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))
        && {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKLM);
            !dokey.is_null()
        }
    {
        return RegEnumValueA(
            dokey,
            dwIndex,
            pszValueName,
            pcchValueNameLen,
            core::ptr::null_mut(),
            pdwType,
            pvData as LPBYTE,
            pcbData,
        ) as LONG;
    }

    return ERROR_INVALID_FUNCTION as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegEnumUSValueW(
    hUSKey: HUSKEY,
    dwIndex: DWORD,
    pszValueName: LPWSTR,
    pcchValueNameLen: LPDWORD,
    pdwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
    enumRegFlags: SHREGENUM_FLAGS,
) -> LONG {
    let mut dokey: HKEY = core::ptr::null_mut();

    if ((enumRegFlags == (SHREGENUM_HKCU as SHREGENUM_FLAGS)) ||
        (enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))) &&
        { dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKCU); !dokey.is_null() }
    {
        return RegEnumValueW(dokey, dwIndex, pszValueName, pcchValueNameLen,
            core::ptr::null_mut(), pdwType, pvData as LPBYTE, pcbData);
    }

    if ((enumRegFlags == (SHREGENUM_HKLM as SHREGENUM_FLAGS)) ||
        (enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))) &&
        { dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKLM); !dokey.is_null() }
    {
        return RegEnumValueW(dokey, dwIndex, pszValueName, pcchValueNameLen,
            core::ptr::null_mut(), pdwType, pvData as LPBYTE, pcbData);
    }

    return (ERROR_INVALID_FUNCTION) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegQueryUSValueA(
    hUSKey: HUSKEY,
    pszValue: LPCSTR,
    pdwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
    fIgnoreHKCU: BOOL,
    pvDefaultData: LPVOID,
    dwDefaultDataSize: DWORD,
) -> LONG {
    let mut ret: LONG = !ERROR_SUCCESS as LONG;
    let mut i: LONG = 0;
    let mut maxmove: LONG = 0;
    let mut dokey: HKEY = core::ptr::null_mut();
    let mut src: *mut CHAR = core::ptr::null_mut();
    let mut dst: *mut CHAR = core::ptr::null_mut();

    /* if user wants HKCU, and it exists, then try it */
    if fIgnoreHKCU == 0 && {
        dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKCU);
        !dokey.is_null()
    } {
        ret = RegQueryValueExA(
            dokey,
            pszValue,
            core::ptr::null_mut(),
            pdwType,
            pvData as LPBYTE,
            pcbData,
        ) as LONG;
        WINE_TRACE!("HKCU RegQueryValue returned %08x\n", ret);
    }

    /* if HKCU did not work and HKLM exists, then try it */
    if (ret != ERROR_SUCCESS as LONG) && {
        dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKLM);
        !dokey.is_null()
    } {
        ret = RegQueryValueExA(
            dokey,
            pszValue,
            core::ptr::null_mut(),
            pdwType,
            pvData as LPBYTE,
            pcbData,
        ) as LONG;
        WINE_TRACE!("HKLM RegQueryValue returned %08x\n", ret);
    }

    /* if neither worked, and default data exists, then use it */
    if ret != ERROR_SUCCESS as LONG {
        if !pvDefaultData.is_null() && dwDefaultDataSize != 0 as DWORD {
            maxmove = (if dwDefaultDataSize >= *pcbData {
                *pcbData
            } else {
                dwDefaultDataSize
            }) as LONG;
            src = pvDefaultData as *mut CHAR;
            dst = pvData as *mut CHAR;
            i = 0 as LONG;
            while i < maxmove {
                let fresh16 = src;
                src = src.offset(1);
                let fresh17 = dst;
                dst = dst.offset(1);
                *fresh17 = *fresh16;
                i += 1;
            }
            *pcbData = maxmove as DWORD;
            WINE_TRACE!("setting default data\n");
            ret = ERROR_SUCCESS as LONG;
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegQueryUSValueW(
    hUSKey: HUSKEY,
    pszValue: LPCWSTR,
    pdwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
    fIgnoreHKCU: BOOL,
    pvDefaultData: LPVOID,
    dwDefaultDataSize: DWORD,
) -> LONG {
    let mut ret: LONG = !ERROR_SUCCESS as LONG;
    let mut i: LONG = 0;
    let mut maxmove: LONG = 0;
    let mut dokey: HKEY = core::ptr::null_mut();
    let mut src: *mut CHAR = core::ptr::null_mut();
    let mut dst: *mut CHAR = core::ptr::null_mut();

    /* if user wants HKCU, and it exists, then try it */
    if fIgnoreHKCU == 0 && {
        dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKCU);
        !dokey.is_null()
    } {
        ret = RegQueryValueExW(
            dokey,
            pszValue,
            core::ptr::null_mut(),
            pdwType,
            pvData as LPBYTE,
            pcbData,
        ) as LONG;
        WINE_TRACE!("HKCU RegQueryValue returned %08x\n", ret);
    }

    /* if HKCU did not work and HKLM exists, then try it */
    if (ret != ERROR_SUCCESS as LONG) && {
        dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKLM);
        !dokey.is_null()
    } {
        ret = RegQueryValueExW(
            dokey,
            pszValue,
            core::ptr::null_mut(),
            pdwType,
            pvData as LPBYTE,
            pcbData,
        ) as LONG;
        WINE_TRACE!("HKLM RegQueryValue returned %08x\n", ret);
    }

    /* if neither worked, and default data exists, then use it */
    if ret != ERROR_SUCCESS as LONG {
        if !pvDefaultData.is_null() && dwDefaultDataSize != 0 as DWORD {
            maxmove = (if dwDefaultDataSize >= *pcbData {
                *pcbData
            } else {
                dwDefaultDataSize
            }) as LONG;
            src = pvDefaultData as *mut CHAR;
            dst = pvData as *mut CHAR;
            i = 0 as LONG;
            while i < maxmove {
                let fresh18 = src;
                src = src.offset(1);
                let fresh19 = dst;
                dst = dst.offset(1);
                *fresh19 = *fresh18;
                i += 1;
            }
            *pcbData = maxmove as DWORD;
            WINE_TRACE!("setting default data\n");
            ret = ERROR_SUCCESS as LONG;
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetUSValueA(
    pSubKey: LPCSTR,
    pValue: LPCSTR,
    pwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
    flagIgnoreHKCU: BOOL,
    pDefaultData: LPVOID,
    wDefaultDataSize: DWORD,
) -> LONG {
    let mut myhuskey: HUSKEY = core::ptr::null_mut();
    let mut ret: LONG = 0;

    if pvData.is_null() || pcbData.is_null() {
        return (ERROR_INVALID_FUNCTION) as LONG;
    }

    ret = SHRegOpenUSKeyA(
        pSubKey,
        0x1,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(myhuskey),
        flagIgnoreHKCU,
    );
    if ret == (ERROR_SUCCESS as LONG) {
        ret = SHRegQueryUSValueA(
            myhuskey,
            pValue,
            pwType,
            pvData,
            pcbData,
            flagIgnoreHKCU,
            pDefaultData,
            wDefaultDataSize,
        );
        SHRegCloseUSKey(myhuskey);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetUSValueW(
    mut pSubKey: LPCWSTR,
    mut pValue: LPCWSTR,
    mut pwType: LPDWORD,
    mut pvData: LPVOID,
    mut pcbData: LPDWORD,
    mut flagIgnoreHKCU: BOOL,
    mut pDefaultData: LPVOID,
    mut wDefaultDataSize: DWORD,
) -> LONG {
    let mut myhuskey: HUSKEY = core::ptr::null_mut();
    let mut ret: LONG = 0;
    if pvData.is_null() || pcbData.is_null() {
        return (ERROR_INVALID_FUNCTION) as LONG;
    }
    WINE_TRACE!("key '%s', value '%s', datalen %d,  %s\n", debugstr_w(pSubKey as *const WCHAR), debugstr_w(pValue as *const WCHAR), *pcbData, if flagIgnoreHKCU != 0 { b"Ignoring HKCU\0" as *const u8 as *const ::core::ffi::c_char } else { b"Tries HKCU then HKLM\0" as *const u8 as *const ::core::ffi::c_char });
    ret = SHRegOpenUSKeyW(
        pSubKey,
        0x1,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(myhuskey),
        flagIgnoreHKCU,
    );
    if ret == (ERROR_SUCCESS as LONG) {
        ret = SHRegQueryUSValueW(
            myhuskey,
            pValue,
            pwType,
            pvData,
            pcbData,
            flagIgnoreHKCU,
            pDefaultData,
            wDefaultDataSize,
        );
        SHRegCloseUSKey(myhuskey);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegSetUSValueA(
    pszSubKey: LPCSTR,
    pszValue: LPCSTR,
    dwType: DWORD,
    pvData: LPVOID,
    cbData: DWORD,
    dwFlags: DWORD,
) -> LONG {
    let mut ignoreHKCU: BOOL = TRUE as BOOL;
    let mut hkey: HUSKEY = core::ptr::null_mut();
    let mut ret: LONG = 0;

    WINE_TRACE!("(%s,%s,%d,%p,%d,0x%08x\n", debugstr_a(pszSubKey as *const i8), debugstr_a(pszValue as *const i8), dwType, pvData, cbData, dwFlags);

    if pvData.is_null() {
        return ERROR_INVALID_FUNCTION as LONG;
    }

    if (dwFlags & SHREGSET_HKCU as DWORD) != 0
        || (dwFlags & SHREGSET_FORCE_HKCU as DWORD) != 0
    {
        ignoreHKCU = FALSE as BOOL;
    }

    ret = SHRegOpenUSKeyA(
        pszSubKey,
        KEY_ALL_ACCESS as REGSAM,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hkey),
        ignoreHKCU,
    );
    if ret == ERROR_SUCCESS as LONG {
        ret = SHRegWriteUSValueA(hkey, pszValue, dwType, pvData, cbData, dwFlags);
        SHRegCloseUSKey(hkey);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegSetUSValueW(
    pszSubKey: LPCWSTR,
    pszValue: LPCWSTR,
    dwType: DWORD,
    pvData: LPVOID,
    cbData: DWORD,
    dwFlags: DWORD,
) -> LONG {
    let mut ignoreHKCU: BOOL = TRUE as BOOL;
    let mut hkey: HUSKEY = core::ptr::null_mut();
    let mut ret: LONG = 0;

    WINE_TRACE!("(%s,%s,%d,%p,%d,0x%08x\n", debugstr_w(pszSubKey as *const WCHAR), debugstr_w(pszValue as *const WCHAR), dwType, pvData, cbData, dwFlags);

    if pvData.is_null() {
        return ERROR_INVALID_FUNCTION as LONG;
    }

    if (dwFlags & (SHREGSET_HKCU as DWORD)) != 0
        || (dwFlags & (SHREGSET_FORCE_HKCU as DWORD)) != 0
    {
        ignoreHKCU = FALSE as BOOL;
    }

    ret = SHRegOpenUSKeyW(
        pszSubKey,
        KEY_ALL_ACCESS as REGSAM,
        0 as HUSKEY,
        core::ptr::addr_of_mut!(hkey),
        ignoreHKCU,
    );
    if ret == ERROR_SUCCESS as LONG {
        ret = SHRegWriteUSValueW(hkey, pszValue, dwType, pvData, cbData, dwFlags);
        SHRegCloseUSKey(hkey);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetBoolUSValueA(
    pszSubKey: LPCSTR,
    pszValue: LPCSTR,
    fIgnoreHKCU: BOOL,
    fDefault: BOOL,
) -> BOOL {
    let mut type_: DWORD = 0;
    let mut datalen: DWORD = 0;
    let mut work: DWORD = 0;
    let mut ret: BOOL = fDefault;
    let mut data: [CHAR; 10] = [0; 10];

    datalen = (core::mem::size_of::<[CHAR; 10]>() as DWORD) - 1;
    if SHRegGetUSValueA(
        pszSubKey,
        pszValue,
        &raw mut type_,
        data.as_mut_ptr() as LPVOID,
        &raw mut datalen,
        fIgnoreHKCU,
        0 as LPVOID,
        0,
    ) == 0
    {
        let mut fallthrough: bool = false;
        if type_ == REG_SZ {
            data[9 as usize] = 0 as CHAR;
            if lstrcmpiA(data.as_mut_ptr() as LPCSTR, b"YES\0".as_ptr() as LPCSTR) == 0 {
                ret = (TRUE as i32);
            }
            if lstrcmpiA(data.as_mut_ptr() as LPCSTR, b"TRUE\0".as_ptr() as LPCSTR) == 0 {
                ret = (TRUE as i32);
            }
            if lstrcmpiA(data.as_mut_ptr() as LPCSTR, b"NO\0".as_ptr() as LPCSTR) == 0 {
                ret = (FALSE as i32);
            }
            if lstrcmpiA(data.as_mut_ptr() as LPCSTR, b"FALSE\0".as_ptr() as LPCSTR) == 0 {
                ret = (FALSE as i32);
            }
        } else if type_ == REG_DWORD {
            // `data` is a CHAR buffer (align 1): the C's `*(LPDWORD)data` is an unaligned read.
            work = (data.as_ptr() as *const DWORD).read_unaligned();
            ret = (work != 0) as BOOL;
        } else if type_ == REG_BINARY {
            if datalen == 1 {
                ret = (data[0 as usize] != 0 as CHAR) as BOOL;
            } else {
                fallthrough = true;
            }
        } else {
            fallthrough = true;
        }

        if fallthrough {
            FIXME!("Unsupported registry data type %d\n", type_);
            ret = (FALSE as i32);
        }
    } else {
        ret = fDefault;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetBoolUSValueW(
    pszSubKey: LPCWSTR,
    pszValue: LPCWSTR,
    fIgnoreHKCU: BOOL,
    fDefault: BOOL,
) -> BOOL {
    static wYES: [WCHAR; 4] = [0x0059u16, 0x0045, 0x0053, 0x0000];
    static wTRUE: [WCHAR; 5] = [0x0054u16, 0x0052, 0x0055, 0x0045, 0x0000];
    static wNO: [WCHAR; 3] = [0x004Eu16, 0x004F, 0x0000];
    static wFALSE: [WCHAR; 6] = [0x0046u16, 0x0041, 0x004C, 0x0053, 0x0045, 0x0000];
    let mut type_0: DWORD = 0;
    let mut datalen: DWORD = 0;
    let mut work: DWORD = 0;
    let mut ret: BOOL = fDefault;
    let mut data: [WCHAR; 10] = [0; 10];

    datalen = ((core::mem::size_of::<[WCHAR; 10]>() as DWORD)
        - 1)
        * (core::mem::size_of::<WCHAR>() as DWORD);
    if SHRegGetUSValueW(
        pszSubKey,
        pszValue,
        core::ptr::addr_of_mut!(type_0),
        core::ptr::addr_of_mut!(data) as *mut WCHAR as LPVOID,
        core::ptr::addr_of_mut!(datalen),
        fIgnoreHKCU,
        core::ptr::null_mut(),
        0,
    ) == 0
    {
        /* process returned data via type into bool */
        let mut to_default: bool = false;
        match type_0 {
            REG_SZ => {
                data[9] = '\0' as WCHAR; /* set end of string */
                if lstrcmpiW(
                    core::ptr::addr_of_mut!(data) as *mut WCHAR as LPCWSTR,
                    core::ptr::addr_of!(wYES) as LPCWSTR,
                ) == 0
                    || lstrcmpiW(
                        core::ptr::addr_of_mut!(data) as *mut WCHAR as LPCWSTR,
                        core::ptr::addr_of!(wTRUE) as LPCWSTR,
                    ) == 0
                {
                    ret = (TRUE as i32);
                } else if lstrcmpiW(
                    core::ptr::addr_of_mut!(data) as *mut WCHAR as LPCWSTR,
                    core::ptr::addr_of!(wNO) as LPCWSTR,
                ) == 0
                    || lstrcmpiW(
                        core::ptr::addr_of_mut!(data) as *mut WCHAR as LPCWSTR,
                        core::ptr::addr_of!(wFALSE) as LPCWSTR,
                    ) == 0
                {
                    ret = (FALSE as i32);
                }
            }
            REG_DWORD => {
                // `data` is a WCHAR buffer (align 2): an unaligned DWORD read, as in the C.
                work = (core::ptr::addr_of!(data) as *const DWORD).read_unaligned();
                ret = (work != 0) as core::ffi::c_int as BOOL;
            }
            REG_BINARY => {
                if datalen == 1 {
                    ret = (data[0] != '\0' as WCHAR) as core::ffi::c_int
                        as BOOL;
                } else {
                    to_default = true;
                }
            }
            _ => {
                to_default = true;
            }
        }
        if to_default {
            FIXME!("Unsupported registry data type %d\n", type_0);
            ret = (FALSE as i32);
        }
    } else {
        ret = fDefault;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegQueryInfoUSKeyA(
    hUSKey: HUSKEY,
    pcSubKeys: LPDWORD,
    pcchMaxSubKeyLen: LPDWORD,
    pcValues: LPDWORD,
    pcchMaxValueNameLen: LPDWORD,
    enumRegFlags: SHREGENUM_FLAGS,
) -> LONG {
    let mut dokey: HKEY = core::ptr::null_mut();
    let mut ret: LONG = 0;

    /* if user wants HKCU, and it exists, then try it */
    if ((enumRegFlags == (SHREGENUM_HKCU as SHREGENUM_FLAGS)) ||
        (enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))) &&
        {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKCU);
            !dokey.is_null()
        }
    {
        ret = RegQueryInfoKeyA(
            dokey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pcSubKeys,
            pcchMaxSubKeyLen,
            core::ptr::null_mut(),
            pcValues,
            pcchMaxValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if (ret == (ERROR_SUCCESS as LONG)) ||
            (enumRegFlags == (SHREGENUM_HKCU as SHREGENUM_FLAGS))
        {
            return ret;
        }
    }
    if ((enumRegFlags == (SHREGENUM_HKLM as SHREGENUM_FLAGS)) ||
        (enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))) &&
        {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKLM);
            !dokey.is_null()
        }
    {
        return RegQueryInfoKeyA(
            dokey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pcSubKeys,
            pcchMaxSubKeyLen,
            core::ptr::null_mut(),
            pcValues,
            pcchMaxValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    return (ERROR_INVALID_FUNCTION) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegQueryInfoUSKeyW(
    hUSKey: HUSKEY,
    pcSubKeys: LPDWORD,
    pcchMaxSubKeyLen: LPDWORD,
    pcValues: LPDWORD,
    pcchMaxValueNameLen: LPDWORD,
    enumRegFlags: SHREGENUM_FLAGS,
) -> LONG {
    let mut dokey: HKEY = core::ptr::null_mut();
    let mut ret: LONG;

    WINE_TRACE!("(%p,%p,%p,%p,%p,%d)\n", hUSKey, pcSubKeys, pcchMaxSubKeyLen, pcValues, pcchMaxValueNameLen, enumRegFlags as ::core::ffi::c_uint);

    if (enumRegFlags as ::core::ffi::c_uint == SHREGENUM_HKCU as ::core::ffi::c_uint
        || enumRegFlags as ::core::ffi::c_uint == SHREGENUM_DEFAULT as ::core::ffi::c_uint)
        && {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKCU);
            !dokey.is_null()
        }
    {
        ret = RegQueryInfoKeyW(
            dokey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pcSubKeys,
            pcchMaxSubKeyLen,
            core::ptr::null_mut(),
            pcValues,
            pcchMaxValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
        if ret == ERROR_SUCCESS as LONG
            || enumRegFlags as ::core::ffi::c_uint == SHREGENUM_HKCU as ::core::ffi::c_uint
        {
            return ret;
        }
    }

    if (enumRegFlags as ::core::ffi::c_uint == SHREGENUM_HKLM as ::core::ffi::c_uint
        || enumRegFlags as ::core::ffi::c_uint == SHREGENUM_DEFAULT as ::core::ffi::c_uint)
        && {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKLM);
            !dokey.is_null()
        }
    {
        return RegQueryInfoKeyW(
            dokey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pcSubKeys,
            pcchMaxSubKeyLen,
            core::ptr::null_mut(),
            pcValues,
            pcchMaxValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
    }

    return ERROR_INVALID_FUNCTION as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegEnumUSKeyA(
    hUSKey: HUSKEY,
    dwIndex: DWORD,
    pszName: LPSTR,
    pcchValueNameLen: LPDWORD,
    enumRegFlags: SHREGENUM_FLAGS,
) -> LONG {
    let mut dokey: HKEY;

    DPRINT1!(
        "(%p,%d,%p,%p(%d),%d)\n",
        hUSKey,
        dwIndex,
        pszName,
        pcchValueNameLen,
        *pcchValueNameLen,
        enumRegFlags as u32
    );

    if (enumRegFlags == (SHREGENUM_HKCU as SHREGENUM_FLAGS) || enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))
        && {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKCU);
            !dokey.is_null()
        }
    {
        return RegEnumKeyExA(
            dokey,
            dwIndex,
            pszName,
            pcchValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    if (enumRegFlags == (SHREGENUM_HKLM as SHREGENUM_FLAGS) || enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))
        && {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKLM);
            !dokey.is_null()
        }
    {
        return RegEnumKeyExA(
            dokey,
            dwIndex,
            pszName,
            pcchValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    DPRINT1!("no support for SHREGENUM_BOTH\n");

    return (ERROR_INVALID_FUNCTION) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegEnumUSKeyW(
    hUSKey: HUSKEY,
    dwIndex: DWORD,
    pszName: LPWSTR,
    pcchValueNameLen: LPDWORD,
    enumRegFlags: SHREGENUM_FLAGS,
) -> LONG {
    let mut dokey: HKEY = core::ptr::null_mut();

    if (enumRegFlags == (SHREGENUM_HKCU as SHREGENUM_FLAGS) || enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))
        && {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKCU);
            !dokey.is_null()
        }
    {
        return RegEnumKeyExW(
            dokey,
            dwIndex,
            pszName,
            pcchValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
    }

    if (enumRegFlags == (SHREGENUM_HKLM as SHREGENUM_FLAGS) || enumRegFlags == (SHREGENUM_DEFAULT as SHREGENUM_FLAGS))
        && {
            dokey = REG_GetHKEYFromHUSKEY(hUSKey, REG_HKLM);
            !dokey.is_null()
        }
    {
        return RegEnumKeyExW(
            dokey,
            dwIndex,
            pszName,
            pcchValueNameLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
    }

    return ERROR_INVALID_FUNCTION as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegWriteUSValueA(
    hUSKey: HUSKEY,
    pszValue: LPCSTR,
    dwType: DWORD,
    pvData: LPVOID,
    cbData: DWORD,
    dwFlags: DWORD,
) -> LONG {
    let mut szValue: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if !pszValue.is_null() {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pszValue,
            -1,
            &raw mut szValue as LPWSTR,
            (MAX_PATH as i32),
        );
    }

    return SHRegWriteUSValueW(
        hUSKey,
        (if !pszValue.is_null() {
            &raw mut szValue as *mut WCHAR
        } else {
            core::ptr::null_mut::<WCHAR>()
        }) as LPCWSTR,
        dwType,
        pvData,
        cbData,
        dwFlags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHRegWriteUSValueW(
    mut hUSKey: HUSKEY,
    mut pszValue: LPCWSTR,
    mut dwType: DWORD,
    mut pvData: LPVOID,
    mut cbData: DWORD,
    mut dwFlags: DWORD,
) -> LONG {
    let mut dummy: DWORD = 0;
    let mut hKey: LPSHUSKEY = hUSKey as LPSHUSKEY;
    let mut ret: LONG = ERROR_SUCCESS as LONG;

    TRACE!(
        "(%p,%s,%d,%p,%d,%d)\n",
        hUSKey,
        debugstr_w(pszValue as *const WCHAR),
        dwType,
        pvData,
        cbData,
        dwFlags
    );

    if hUSKey.is_null()
        || IsBadWritePtr(hUSKey as PVOID, core::mem::size_of::<SHUSKEY>() as UINT_PTR) != 0
        || dwFlags & (SHREGSET_FORCE_HKCU | SHREGSET_FORCE_HKLM) as DWORD == 0
    {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if dwFlags & (SHREGSET_FORCE_HKCU | SHREGSET_HKCU) as DWORD != 0 {
        if (*hKey).HKCUkey.is_null() {
            /* Create the key */
            ret = RegCreateKeyW(
                (*hKey).HKCUstart,
                &raw mut (*hKey).lpszPath as *mut WCHAR as LPCWSTR,
                &raw mut (*hKey).HKCUkey,
            ) as LONG;
            WARN!("Creating HKCU key, ret = %d\n", ret);
            if ret != 0 && dwFlags & (SHREGSET_FORCE_HKCU) as DWORD != 0 {
                (*hKey).HKCUkey = 0 as HKEY;
                return ret;
            }
        }

        if ret == 0 {
            if dwFlags & SHREGSET_FORCE_HKCU as DWORD != 0
                || RegQueryValueExW(
                    (*hKey).HKCUkey,
                    pszValue,
                    ::core::ptr::null_mut::<DWORD>(),
                    ::core::ptr::null_mut::<DWORD>(),
                    ::core::ptr::null_mut::<BYTE>(),
                    &raw mut dummy,
                ) != 0
            {
                /* Doesn't exist or we are forcing: Write value */
                ret = RegSetValueExW(
                    (*hKey).HKCUkey,
                    pszValue,
                    0 as DWORD,
                    dwType,
                    pvData as *const BYTE,
                    cbData,
                ) as LONG;
                WARN!("Writing HKCU value, ret = %d\n", ret);
            }
        }
    }

    if dwFlags & (SHREGSET_FORCE_HKLM | SHREGSET_HKLM) as DWORD != 0 {
        if (*hKey).HKLMkey.is_null() {
            /* Create the key */
            ret = RegCreateKeyW(
                (*hKey).HKLMstart,
                &raw mut (*hKey).lpszPath as *mut WCHAR as LPCWSTR,
                &raw mut (*hKey).HKLMkey,
            ) as LONG;
            WARN!("Creating HKLM key, ret = %d\n", ret);
            if ret != 0 && dwFlags & (SHREGSET_FORCE_HKLM) as DWORD != 0 {
                (*hKey).HKLMkey = 0 as HKEY;
                return ret;
            }
        }

        if ret == 0 {
            if dwFlags & SHREGSET_FORCE_HKLM as DWORD != 0
                || RegQueryValueExW(
                    (*hKey).HKLMkey,
                    pszValue,
                    ::core::ptr::null_mut::<DWORD>(),
                    ::core::ptr::null_mut::<DWORD>(),
                    ::core::ptr::null_mut::<BYTE>(),
                    &raw mut dummy,
                ) != 0
            {
                /* Doesn't exist or we are forcing: Write value */
                ret = RegSetValueExW(
                    (*hKey).HKLMkey,
                    pszValue,
                    0 as DWORD,
                    dwType,
                    pvData as *const BYTE,
                    cbData,
                ) as LONG;
                WARN!("Writing HKLM value, ret = %d\n", ret);
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetPathA(
    hKey: HKEY,
    lpszSubKey: LPCSTR,
    lpszValue: LPCSTR,
    lpszPath: LPSTR,
    dwFlags: DWORD,
) -> DWORD {
    let mut dwSize: DWORD = MAX_PATH;
    return SHGetValueA(hKey, lpszSubKey, lpszValue, core::ptr::null_mut(), lpszPath as LPVOID, core::ptr::addr_of_mut!(dwSize));
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetPathW(
    hKey: HKEY,
    lpszSubKey: LPCWSTR,
    lpszValue: LPCWSTR,
    lpszPath: LPWSTR,
    dwFlags: DWORD,
) -> DWORD {
    let mut dwSize: DWORD = MAX_PATH;
    WINE_TRACE!("(hkey=%p,%s,%s,%p,%d)\n", hKey, debugstr_w(lpszSubKey as *const WCHAR), debugstr_w(lpszValue as *const WCHAR), lpszPath, dwFlags);
    return SHGetValueW(
        hKey,
        lpszSubKey,
        lpszValue,
        core::ptr::null_mut(),
        lpszPath as LPVOID,
        core::ptr::addr_of_mut!(dwSize),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHRegSetPathA(
    hKey: HKEY,
    lpszSubKey: LPCSTR,
    lpszValue: LPCSTR,
    lpszPath: LPCSTR,
    dwFlags: DWORD,
) -> DWORD {
    let mut szBuff: [core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    lstrcpyA(core::ptr::addr_of_mut!(szBuff) as LPSTR, lpszPath);

    /* FIXME: PathUnExpandEnvStringsA(szBuff); */

    return SHSetValueA(
        hKey,
        lpszSubKey,
        lpszValue,
        REG_SZ,
        core::ptr::addr_of_mut!(szBuff) as LPCVOID,
        lstrlenA(core::ptr::addr_of_mut!(szBuff) as LPCSTR) as DWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHRegSetPathW(
    hKey: HKEY,
    lpszSubKey: LPCWSTR,
    lpszValue: LPCWSTR,
    lpszPath: LPCWSTR,
    dwFlags: DWORD,
) -> DWORD {
    let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    WINE_FIXME!("(hkey=%p,%s,%s,%p,%d) - semi-stub\n", hKey, debugstr_w(lpszSubKey as *const WCHAR), debugstr_w(lpszValue as *const WCHAR), lpszPath, dwFlags);

    lstrcpyW(core::ptr::addr_of_mut!(szBuff) as LPWSTR, lpszPath);

    /* FIXME: PathUnExpandEnvStringsW(szBuff); */

    return SHSetValueW(
        hKey,
        lpszSubKey,
        lpszValue,
        REG_SZ as DWORD,
        core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCVOID,
        lstrlenW(core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR) as DWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHGetValueA(
    hKey: HKEY,
    lpszSubKey: LPCSTR,
    lpszValue: LPCSTR,
    pwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
) -> DWORD {
    let mut dwRet: DWORD = 0;
    let mut hSubKey: HKEY = core::ptr::null_mut();

    /*   lpszSubKey can be 0. In this case the value is taken from the
     *   current key.
     */
    if !lpszSubKey.is_null() {
        dwRet = RegOpenKeyExA(hKey, lpszSubKey, 0, KEY_QUERY_VALUE as REGSAM, &raw mut hSubKey) as DWORD;
    }

    if dwRet == 0 {
        /* SHQueryValueEx expands Environment strings */
        dwRet = SHQueryValueExA(
            if !hSubKey.is_null() { hSubKey } else { hKey },
            lpszValue,
            core::ptr::null_mut(),
            pwType,
            pvData,
            pcbData,
        );
        if !hSubKey.is_null() {
            RegCloseKey(hSubKey);
        }
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetValueW(
    hKey: HKEY,
    lpszSubKey: LPCWSTR,
    lpszValue: LPCWSTR,
    pwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
) -> DWORD {
    let mut dwRet: DWORD = 0;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    WINE_TRACE!("(hkey=%p,%s,%s,%p,%p,%p)\n", hKey, debugstr_w(lpszSubKey as *const WCHAR), debugstr_w(lpszValue as *const WCHAR), pwType, pvData, pcbData);
    if !lpszSubKey.is_null() {
        dwRet = RegOpenKeyExW(
            hKey,
            lpszSubKey,
            0,
            KEY_QUERY_VALUE,
            &raw mut hSubKey,
        ) as DWORD;
    }
    if dwRet == 0 {
        dwRet = SHQueryValueExW(
            if !hSubKey.is_null() { hSubKey } else { hKey },
            lpszValue,
            core::ptr::null_mut(),
            pwType,
            pvData,
            pcbData,
        );
        if !hSubKey.is_null() {
            RegCloseKey(hSubKey);
        }
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHSetValueA(
    hKey: HKEY,
    lpszSubKey: LPCSTR,
    lpszValue: LPCSTR,
    dwType: DWORD,
    pvData: LPCVOID,
    cbData: DWORD,
) -> DWORD {
    let mut dwRet: DWORD = ERROR_SUCCESS as DWORD;
    let mut dwDummy: DWORD = 0;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    WINE_TRACE!("(hkey=%p,%s,%s,%d,%p,%d)\n", hKey, debugstr_a(lpszSubKey as *const ::core::ffi::c_char), debugstr_a(lpszValue as *const ::core::ffi::c_char), dwType, pvData, cbData);
    if !lpszSubKey.is_null() && *lpszSubKey as ::core::ffi::c_int != 0 {
        dwRet = RegCreateKeyExA(
            hKey,
            lpszSubKey,
            0 as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
            KEY_SET_VALUE as REGSAM,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hSubKey),
            core::ptr::addr_of_mut!(dwDummy),
        ) as DWORD;
    } else {
        hSubKey = hKey;
    }
    if dwRet == 0 {
        dwRet = RegSetValueExA(
            hSubKey,
            lpszValue,
            0 as DWORD,
            dwType,
            pvData as *const BYTE,
            cbData,
        ) as DWORD;
        if hSubKey != hKey {
            RegCloseKey(hSubKey);
        }
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHSetValueW(
    hKey: HKEY,
    lpszSubKey: LPCWSTR,
    lpszValue: LPCWSTR,
    dwType: DWORD,
    pvData: LPCVOID,
    cbData: DWORD,
) -> DWORD {
    let mut dwRet: DWORD = ERROR_SUCCESS as DWORD;
    let mut dwDummy: DWORD = 0;
    let mut hSubKey: HKEY = core::ptr::null_mut();

    if !lpszSubKey.is_null() && *lpszSubKey != 0 {
        dwRet = RegCreateKeyExW(
            hKey,
            lpszSubKey,
            0 as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
            KEY_SET_VALUE as REGSAM,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hSubKey),
            core::ptr::addr_of_mut!(dwDummy),
        ) as DWORD;
    } else {
        hSubKey = hKey;
    }
    if dwRet == 0 {
        dwRet = RegSetValueExW(
            hSubKey,
            lpszValue,
            0 as DWORD,
            dwType,
            pvData as *const BYTE,
            cbData,
        ) as DWORD;
        if hSubKey != hKey {
            RegCloseKey(hSubKey);
        }
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryInfoKeyA(
    hKey: HKEY,
    pwSubKeys: LPDWORD,
    pwSubKeyMax: LPDWORD,
    pwValues: LPDWORD,
    pwValueMax: LPDWORD,
) -> LONG {
    return RegQueryInfoKeyA(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pwSubKeys,
        pwSubKeyMax,
        core::ptr::null_mut(),
        pwValues,
        pwValueMax,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryInfoKeyW(hKey: HKEY, pwSubKeys: LPDWORD, pwSubKeyMax: LPDWORD, pwValues: LPDWORD, pwValueMax: LPDWORD) -> LONG {
    return RegQueryInfoKeyW(hKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), pwSubKeys, pwSubKeyMax, core::ptr::null_mut(), pwValues, pwValueMax, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryValueExA(
    hKey: HKEY,
    lpszValue: LPCSTR,
    lpReserved: LPDWORD,
    pwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
) -> DWORD {
    let mut dwRet: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwUnExpDataLen: DWORD = 0;
    let mut dwExpDataLen: DWORD = 0;

    if !pcbData.is_null() {
        dwUnExpDataLen = *pcbData;
    }

    dwRet = (RegQueryValueExA(
        hKey,
        lpszValue,
        lpReserved,
        core::ptr::addr_of_mut!(dwType),
        (pvData).cast(),
        core::ptr::addr_of_mut!(dwUnExpDataLen),
    )) as u32;

    if !pcbData.is_null() && dwType == REG_EXPAND_SZ {
        let mut nBytesToAlloc: DWORD = 0;
        let mut szData: LPSTR = core::ptr::null_mut();

        if pvData.is_null() || dwRet == ERROR_MORE_DATA {
            let mut cNull: i8 = 0;
            nBytesToAlloc = dwUnExpDataLen;

            szData = LocalAlloc(
                LMEM_ZEROINIT as UINT,
                nBytesToAlloc as SIZE_T,
            ) as LPSTR;
            RegQueryValueExA(
                hKey,
                lpszValue,
                lpReserved,
                core::ptr::null_mut(),
                szData as LPBYTE,
                core::ptr::addr_of_mut!(nBytesToAlloc),
            );
            dwExpDataLen = ExpandEnvironmentStringsA(
                szData as LPCSTR,
                core::ptr::addr_of_mut!(cNull),
                1 as DWORD,
            );
            dwUnExpDataLen = max!(nBytesToAlloc, dwExpDataLen);
            LocalFree(szData as HLOCAL);
        } else {
            nBytesToAlloc = ((lstrlenA(pvData as LPCSTR) + 1)
                as usize)
                .wrapping_mul(core::mem::size_of::<CHAR>() as usize) as DWORD;
            szData = LocalAlloc(
                LMEM_ZEROINIT as UINT,
                nBytesToAlloc as SIZE_T,
            ) as LPSTR;
            lstrcpyA(szData, pvData as LPCSTR);
            dwExpDataLen = ExpandEnvironmentStringsA(
                szData as LPCSTR,
                pvData as LPSTR,
                (*pcbData as usize).wrapping_div(core::mem::size_of::<CHAR>() as usize) as DWORD,
            );
            if dwExpDataLen > *pcbData {
                dwRet = ERROR_MORE_DATA;
            }
            dwUnExpDataLen = max!(nBytesToAlloc, dwExpDataLen);
            LocalFree(szData as HLOCAL);
        }
    }

    if dwType == REG_EXPAND_SZ {
        dwType = REG_SZ;
    }
    if !pwType.is_null() {
        *pwType = dwType;
    }
    if !pcbData.is_null() {
        *pcbData = dwUnExpDataLen;
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryValueExW(
    hKey: HKEY,
    lpszValue: LPCWSTR,
    lpReserved: LPDWORD,
    pwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
) -> DWORD {
    let mut dwRet: DWORD;
    let mut dwType: DWORD = core::mem::zeroed();
    let mut dwUnExpDataLen: DWORD = 0;
    let mut dwExpDataLen: DWORD;

    if !pcbData.is_null() {
        dwUnExpDataLen = *pcbData;
    }

    dwRet = RegQueryValueExW(
        hKey,
        lpszValue,
        lpReserved,
        core::ptr::addr_of_mut!(dwType),
        (pvData).cast(),
        core::ptr::addr_of_mut!(dwUnExpDataLen),
    ) as DWORD;
    if dwRet != ERROR_SUCCESS && dwRet != ERROR_MORE_DATA {
        return dwRet;
    }

    if !pcbData.is_null() && dwType == REG_EXPAND_SZ {
        let mut nBytesToAlloc: DWORD;

        /* Expand type REG_EXPAND_SZ into REG_SZ */
        let mut szData: LPWSTR;

        /* If the caller didn't supply a buffer or the buffer is too small we have
         * to allocate our own
         */
        if pvData.is_null() || dwRet == ERROR_MORE_DATA {
            let mut cNull: WCHAR = '\0' as u16;
            nBytesToAlloc = dwUnExpDataLen;

            szData = LocalAlloc(LMEM_ZEROINIT, (nBytesToAlloc) as SIZE_T) as LPWSTR;
            RegQueryValueExW(
                hKey,
                lpszValue,
                lpReserved,
                core::ptr::null_mut::<DWORD>(),
                szData as LPBYTE,
                core::ptr::addr_of_mut!(nBytesToAlloc),
            );
            dwExpDataLen = ExpandEnvironmentStringsW(
                szData,
                core::ptr::addr_of_mut!(cNull),
                1,
            );
            dwUnExpDataLen = max!(nBytesToAlloc, dwExpDataLen);
            LocalFree(szData as HLOCAL);
        } else {
            nBytesToAlloc = (((lstrlenW(pvData as LPCWSTR) + 1)
                as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)) as DWORD;
            szData = LocalAlloc(LMEM_ZEROINIT, (nBytesToAlloc) as SIZE_T) as LPWSTR;
            lstrcpyW(szData, pvData as LPCWSTR);
            dwExpDataLen = ExpandEnvironmentStringsW(
                szData as LPCWSTR,
                pvData as LPWSTR,
                *pcbData / (core::mem::size_of::<WCHAR>() as DWORD),
            );
            if dwExpDataLen > *pcbData {
                dwRet = ERROR_MORE_DATA;
            }
            dwUnExpDataLen = max!(nBytesToAlloc, dwExpDataLen);
            LocalFree(szData as HLOCAL);
        }
    }

    /* Update the type and data size if the caller wanted them */
    if dwType == REG_EXPAND_SZ {
        dwType = REG_SZ;
    }
    if !pwType.is_null() {
        *pwType = dwType;
    }
    if !pcbData.is_null() {
        *pcbData = dwUnExpDataLen;
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteKeyA(hKey: HKEY, lpszSubKey: LPCSTR) -> DWORD {
    let mut subkeyW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        lpszSubKey,
        -(1 as i32),
        core::ptr::addr_of_mut!(subkeyW) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
            / core::mem::size_of::<WCHAR>()) as i32,
    );
    return SHDeleteKeyW(hKey, core::ptr::addr_of_mut!(subkeyW) as *mut WCHAR as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteKeyW(mut hKey: HKEY, mut lpszSubKey: LPCWSTR) -> DWORD {
    let mut dwRet: DWORD = 0;
    let mut dwMaxSubkeyLen: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut szNameBuf: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszName: *mut WCHAR = core::ptr::addr_of_mut!(szNameBuf) as *mut WCHAR;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    WINE_TRACE!("(hkey=%p,%s)\n", hKey, debugstr_w(lpszSubKey as *const WCHAR));
    dwRet = RegOpenKeyExW(
        hKey,
        lpszSubKey,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hSubKey),
    ) as DWORD;
    if dwRet == 0 {
        dwRet = RegQueryInfoKeyW(
            hSubKey,
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwMaxSubkeyLen),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<_FILETIME>(),
        ) as DWORD;
        if dwRet == 0 {
            dwMaxSubkeyLen = dwMaxSubkeyLen.wrapping_add(1);
            if dwMaxSubkeyLen as usize
                > (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as usize)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
            {
                lpszName = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    (dwMaxSubkeyLen as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
            }
            if lpszName.is_null() {
                dwRet = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            } else {
                while dwRet == ERROR_SUCCESS as DWORD {
                    dwSize = dwMaxSubkeyLen;
                    dwRet = RegEnumKeyExW(
                        hSubKey,
                        0 as DWORD,
                        lpszName as LPWSTR,
                        core::ptr::addr_of_mut!(dwSize),
                        core::ptr::null_mut::<DWORD>(),
                        core::ptr::null_mut::<WCHAR>(),
                        core::ptr::null_mut::<DWORD>(),
                        core::ptr::null_mut::<_FILETIME>(),
                    ) as DWORD;
                    if dwRet == ERROR_SUCCESS as DWORD
                        || dwRet == ERROR_MORE_DATA as DWORD
                    {
                        dwRet = SHDeleteKeyW(hSubKey, lpszName as LPCWSTR);
                    }
                }
                if dwRet == ERROR_NO_MORE_ITEMS as DWORD {
                    dwRet = ERROR_SUCCESS as DWORD;
                }
                if lpszName != core::ptr::addr_of_mut!(szNameBuf) as *mut WCHAR {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        lpszName as PVOID,
                    );
                }
            }
        }
        RegCloseKey(hSubKey);
        if dwRet == 0 {
            dwRet = RegDeleteKeyW(hKey, lpszSubKey) as DWORD;
        }
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteEmptyKeyA(mut hKey: HKEY, mut lpszSubKey: LPCSTR) -> DWORD {
    let mut dwRet: DWORD;
    let mut dwKeyCount: DWORD = 0 as DWORD;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    TRACE!(
        "(hkey=%p,%s)\n",
        hKey,
        debugstr_a(lpszSubKey as *const i8)
    );
    dwRet = RegOpenKeyExA(
        hKey,
        lpszSubKey,
        0 as REGSAM,
        KEY_READ as DWORD,
        core::ptr::addr_of_mut!(hSubKey),
    ) as DWORD;
    if dwRet == 0 {
        dwRet = RegQueryInfoKeyA(
            hSubKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwKeyCount),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;
        RegCloseKey(hSubKey);
        if dwRet == 0 {
            if dwKeyCount == 0 {
                dwRet = RegDeleteKeyA(hKey, lpszSubKey) as DWORD;
            } else {
                dwRet = ERROR_KEY_HAS_CHILDREN as DWORD;
            }
        }
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteEmptyKeyW(hKey: HKEY, lpszSubKey: LPCWSTR) -> DWORD {
    let mut dwRet: DWORD;
    let mut dwKeyCount: DWORD = 0;
    let mut hSubKey: HKEY = core::ptr::null_mut();

    dwRet = RegOpenKeyExW(hKey, lpszSubKey, 0, KEY_READ, core::ptr::addr_of_mut!(hSubKey)) as DWORD;
    if dwRet == 0 {
        dwRet = RegQueryInfoKeyW(
            hSubKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwKeyCount),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;
        RegCloseKey(hSubKey);
        if dwRet == 0 {
            if dwKeyCount == 0 {
                dwRet = RegDeleteKeyW(hKey, lpszSubKey) as DWORD;
            } else {
                dwRet = ERROR_KEY_HAS_CHILDREN;
            }
        }
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteOrphanKeyA(hKey: HKEY, lpszSubKey: LPCSTR) -> DWORD {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut dwKeyCount: DWORD = 0;
    let mut dwValueCount: DWORD = 0;
    let mut dwRet: DWORD;

    dwRet = RegOpenKeyExA(
        hKey,
        lpszSubKey,
        0,
        KEY_READ,
        &mut hSubKey,
    ) as DWORD;

    if dwRet == 0 {
        /* Get subkey and value count */
        dwRet = RegQueryInfoKeyA(
            hSubKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwKeyCount,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwValueCount,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;

        if dwRet == 0 && dwKeyCount == 0 && dwValueCount == 0 {
            dwRet = RegDeleteKeyA(hKey, lpszSubKey) as DWORD;
        }
        RegCloseKey(hSubKey);
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteOrphanKeyW(hKey: HKEY, lpszSubKey: LPCWSTR) -> DWORD {
    let mut hSubKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut dwKeyCount: DWORD = 0;
    let mut dwValueCount: DWORD = 0;
    let mut dwRet: DWORD;
    dwRet = RegOpenKeyExW(
        hKey,
        lpszSubKey,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hSubKey),
    ) as DWORD;
    if dwRet == 0 {
        dwRet = RegQueryInfoKeyW(
            hSubKey,
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwKeyCount),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwValueCount),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<_FILETIME>(),
        ) as DWORD;
        if dwRet == 0 && dwKeyCount == 0 && dwValueCount == 0 {
            dwRet = RegDeleteKeyW(hKey, lpszSubKey) as DWORD;
        }
        RegCloseKey(hSubKey);
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteValueA(hKey: HKEY, lpszSubKey: LPCSTR, lpszValue: LPCSTR) -> DWORD {
    let mut dwRet: DWORD;
    let mut hSubKey: HKEY = core::ptr::null_mut();

    dwRet = RegOpenKeyExA(hKey, lpszSubKey, 0 as DWORD, KEY_SET_VALUE as REGSAM, core::ptr::addr_of_mut!(hSubKey)) as DWORD;
    if dwRet == 0 {
        dwRet = RegDeleteValueA(hSubKey, lpszValue) as DWORD;
        RegCloseKey(hSubKey);
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteValueW(hKey: HKEY, lpszSubKey: LPCWSTR, lpszValue: LPCWSTR) -> DWORD {
    let mut dwRet: DWORD = 0;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    WINE_TRACE!("(hkey=%p,%s,%s)\n", hKey, debugstr_w(lpszSubKey as *const WCHAR), debugstr_w(lpszValue as *const WCHAR));
    dwRet = RegOpenKeyExW(
        hKey,
        lpszSubKey,
        0,
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(hSubKey),
    ) as DWORD;
    if dwRet == 0 {
        dwRet = RegDeleteValueW(hSubKey, lpszValue) as DWORD;
        RegCloseKey(hSubKey);
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumKeyExA(
    hKey: HKEY,
    dwIndex: DWORD,
    lpszSubKey: LPSTR,
    pwLen: LPDWORD,
) -> LONG {
    return RegEnumKeyExA(
        hKey,
        dwIndex,
        lpszSubKey,
        pwLen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumKeyExW(
    hKey: HKEY,
    dwIndex: DWORD,
    lpszSubKey: LPWSTR,
    pwLen: LPDWORD,
) -> LONG {
    WINE_TRACE!("(hkey=%p,%d,%s,%p)\n", hKey, dwIndex, debugstr_w(lpszSubKey as *const WCHAR), pwLen);
    return RegEnumKeyExW(
        hKey,
        dwIndex,
        lpszSubKey,
        pwLen,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<WCHAR>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<FILETIME>(),
    ) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumValueA(
    hKey: HKEY,
    dwIndex: DWORD,
    lpszValue: LPSTR,
    pwLen: LPDWORD,
    pwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
) -> LONG {
    return RegEnumValueA(
        hKey,
        dwIndex,
        lpszValue,
        pwLen,
        ::core::ptr::null_mut::<DWORD>(),
        pwType,
        pvData as LPBYTE,
        pcbData,
    ) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumValueW(
    hKey: HKEY,
    dwIndex: DWORD,
    lpszValue: LPWSTR,
    pwLen: LPDWORD,
    pwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
) -> LONG {
    return RegEnumValueW(
        hKey,
        dwIndex,
        lpszValue,
        pwLen,
        core::ptr::null_mut(),
        pwType,
        pvData as LPBYTE,
        pcbData,
    ) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetValueGoodBootA(
    hkey: HKEY,
    pSubKey: LPCSTR,
    pValue: LPCSTR,
    pwType: LPDWORD,
    pvData: LPVOID,
    pbData: LPDWORD,
) -> DWORD {
    if GetSystemMetrics((SM_CLEANBOOT as i32)) != 0 {
        return ERROR_INVALID_FUNCTION as DWORD;
    }
    return SHGetValueA(hkey, pSubKey, pValue, pwType, pvData, pbData);
}

#[no_mangle]
pub unsafe extern "system" fn SHGetValueGoodBootW(hkey: HKEY, pSubKey: LPCWSTR, pValue: LPCWSTR, pwType: LPDWORD, pvData: LPVOID, pbData: LPDWORD) -> DWORD {
    if GetSystemMetrics((SM_CLEANBOOT as i32)) != 0 {
        return ERROR_INVALID_FUNCTION as DWORD;
    }
    return SHGetValueW(hkey, pSubKey, pValue, pwType, pvData, pbData);
}

#[no_mangle]
pub unsafe extern "system" fn RegisterMIMETypeForExtensionA(lpszSubKey: LPCSTR, lpszValue: LPCSTR) -> BOOL {
    if lpszValue.is_null() {
        WARN!("Invalid lpszValue would crash under Win32!\n");
        return (FALSE) as BOOL;
    }

    return (SHSetValueA(
        HKEY_CLASSES_ROOT,
        lpszSubKey,
        core::ptr::addr_of!(lpszContentTypeA) as LPCSTR,
        REG_SZ,
        lpszValue as LPCVOID,
        strlen(lpszValue as *const i8) as DWORD,
    ) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterMIMETypeForExtensionW(lpszSubKey: LPCWSTR, lpszValue: LPCWSTR) -> BOOL {
    if lpszValue.is_null() {
        WARN!("Invalid lpszValue would crash under Win32!\n");
        return (FALSE) as BOOL;
    }

    return (SHSetValueW(HKEY_CLASSES_ROOT, lpszSubKey, core::ptr::addr_of!(lpszContentTypeW) as LPCWSTR,
                        REG_SZ, lpszValue as LPCVOID, (strlenW(lpszValue)) as DWORD) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterMIMETypeForExtensionA(lpszSubKey: LPCSTR) -> BOOL {
    return (SHDeleteValueA(HKEY_CLASSES_ROOT, lpszSubKey, core::ptr::addr_of!(lpszContentTypeA) as LPCSTR) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterMIMETypeForExtensionW(lpszSubKey: LPCWSTR) -> BOOL {
    return (SHDeleteValueW(
        HKEY_CLASSES_ROOT,
        lpszSubKey,
        core::ptr::addr_of!(lpszContentTypeW) as LPCWSTR,
    ) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetMIMETypeSubKeyA(
    lpszType: LPCSTR,
    lpszBuffer: LPSTR,
    dwLen: DWORD,
) -> BOOL {
    if dwLen > dwLenMimeDbContent && !lpszType.is_null() && !lpszBuffer.is_null() {
        let mut dwStrLen: size_t = strlen(lpszType as *const ::core::ffi::c_char);
        if dwStrLen < dwLen.wrapping_sub(dwLenMimeDbContent) as size_t {
            memcpy(
                lpszBuffer as *mut ::core::ffi::c_void,
                core::ptr::addr_of!(szMimeDbContentA) as *const ::core::ffi::c_void,
                dwLenMimeDbContent as size_t,
            );
            memcpy(
                lpszBuffer.offset(dwLenMimeDbContent as isize) as *mut ::core::ffi::c_void,
                lpszType as *const ::core::ffi::c_void,
                dwStrLen + 1 as size_t,
            );
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetMIMETypeSubKeyW(lpszType: LPCWSTR, lpszBuffer: LPWSTR, dwLen: DWORD) -> BOOL {
    if dwLen > dwLenMimeDbContent && !lpszType.is_null() && !lpszBuffer.is_null() {
        let mut dwStrLen: DWORD = strlenW(lpszType) as DWORD;

        if dwStrLen < dwLen - dwLenMimeDbContent {
            memcpy(
                lpszBuffer as *mut core::ffi::c_void,
                core::ptr::addr_of!(szMimeDbContentW) as *const WCHAR as *const core::ffi::c_void,
                (dwLenMimeDbContent as usize) * core::mem::size_of::<WCHAR>(),
            );
            memcpy(
                lpszBuffer.offset(dwLenMimeDbContent as isize) as *mut core::ffi::c_void,
                lpszType as *const core::ffi::c_void,
                ((dwStrLen + 1) as usize) * core::mem::size_of::<WCHAR>(),
            );
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn MIME_GetExtensionA(
    mut lpszType: LPCSTR,
    mut lpExt: LPSTR,
    mut iLen: INT,
) -> BOOL {
    let mut szSubKey: [c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwlen: DWORD = (iLen - 1) as DWORD;
    let mut dwType: DWORD = 0;
    let mut bRet: BOOL = (FALSE as i32);
    if iLen > 0 && !lpExt.is_null() {
        *lpExt = 0 as CHAR;
    }
    if !lpszType.is_null()
        && !lpExt.is_null()
        && iLen > 2
        && GetMIMETypeSubKeyA(
            lpszType,
            &raw mut szSubKey as LPSTR,
            MAX_PATH as DWORD,
        ) != 0
        && SHGetValueA(
            HKEY_CLASSES_ROOT as HKEY,
            &raw mut szSubKey as *mut c_char as LPCSTR,
            b"Extension\0".as_ptr() as LPCSTR,
            &raw mut dwType,
            lpExt.offset(1 as isize) as LPVOID,
            &raw mut dwlen,
        ) == 0
        && *lpExt.offset(1 as isize) as c_int != 0
    {
        if *lpExt.offset(1 as isize) as c_int == b'.' as c_int {
            memmove(
                lpExt as *mut c_void,
                lpExt.offset(1 as isize) as *const c_void,
                strlen(
                    lpExt.offset(1 as isize) as *const c_char,
                )
                .wrapping_add(1 as usize),
            );
        } else {
            *lpExt = b'.' as CHAR; /* Supply a '.' */
        }
        bRet = TRUE as BOOL;
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn MIME_GetExtensionW(lpszType: LPCWSTR, lpExt: LPWSTR, iLen: INT) -> BOOL {
    let mut szSubKey: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwlen: DWORD = (iLen - 1) as DWORD;
    let mut dwType: DWORD = 0;
    let mut bRet: BOOL = (FALSE as i32);

    if iLen > 0 && !lpExt.is_null() {
        *lpExt = '\0' as WCHAR;
    }

    if !lpszType.is_null()
        && !lpExt.is_null()
        && iLen > 2
        && GetMIMETypeSubKeyW(
            lpszType,
            core::ptr::addr_of_mut!(szSubKey) as *mut WCHAR as LPWSTR,
            MAX_PATH as DWORD,
        ) != 0
        && SHGetValueW(
            HKEY_CLASSES_ROOT,
            core::ptr::addr_of_mut!(szSubKey) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of!(szExtensionW) as LPCWSTR,
            core::ptr::addr_of_mut!(dwType),
            lpExt.offset(1 as isize) as LPVOID,
            core::ptr::addr_of_mut!(dwlen),
        ) == 0
        && *lpExt.offset(1 as isize) != 0
    {
        if *lpExt.offset(1 as isize) == '.' as WCHAR {
            memmove(
                lpExt as *mut core::ffi::c_void,
                lpExt.offset(1 as isize) as *const core::ffi::c_void,
                ((strlenW(lpExt.offset(1 as isize)) as usize)
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>(),
            );
        } else {
            *lpExt = '.' as WCHAR; /* Supply a '.' */
        }
        bRet = (TRUE as i32);
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterExtensionForMIMETypeA(lpszExt: LPCSTR, lpszType: LPCSTR) -> BOOL {
    let mut dwLen: DWORD = 0;
    let mut szKey: [c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    WINE_TRACE!("(%s,%s)\n", debugstr_a(lpszExt as *const c_char), debugstr_a(lpszType as *const c_char));

    if GetMIMETypeSubKeyA(lpszType, &raw mut szKey as LPSTR, MAX_PATH as DWORD) == 0 {
        return (FALSE) as BOOL;
    }

    dwLen = strlen(lpszExt as *const c_char).wrapping_add(1 as usize) as DWORD;

    if SHSetValueA(
        HKEY_CLASSES_ROOT,
        &raw mut szKey as *mut c_char as LPCSTR,
        &raw const szExtensionA as LPCSTR,
        REG_SZ as DWORD,
        lpszExt as LPCVOID,
        dwLen,
    ) != 0
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterExtensionForMIMETypeW(
    mut lpszExt: LPCWSTR,
    mut lpszType: LPCWSTR,
) -> BOOL {
    let mut dwLen: DWORD = 0;
    let mut szKey: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    WINE_TRACE!("(%s,%s)\n", debugstr_w(lpszExt as *const WCHAR), debugstr_w(lpszType as *const WCHAR));
    if GetMIMETypeSubKeyW(lpszType, &raw mut szKey as LPWSTR, MAX_PATH as DWORD) == 0 {
        return (FALSE) as BOOL;
    }
    dwLen = ((lstrlenW(lpszExt) + 1 as ::core::ffi::c_int) as usize)
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
    if SHSetValueW(
        HKEY_CLASSES_ROOT,
        &raw mut szKey as *mut WCHAR as LPCWSTR,
        &raw const szExtensionW as LPCWSTR,
        REG_SZ as DWORD,
        lpszExt as LPCVOID,
        dwLen,
    ) != 0
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterExtensionForMIMETypeA(lpszType: LPCSTR) -> BOOL {
    let mut szKey: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    TRACE!("(%s)\n", debugstr_a(lpszType as *const ::core::ffi::c_char));

    if GetMIMETypeSubKeyA(lpszType, core::ptr::addr_of_mut!(szKey) as LPSTR, MAX_PATH) == 0 {
        return (FALSE) as BOOL;
    }

    if SHDeleteValueA(
        HKEY_CLASSES_ROOT,
        core::ptr::addr_of_mut!(szKey) as LPCSTR,
        core::ptr::addr_of!(szExtensionA) as LPCSTR,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    if SHDeleteOrphanKeyA(
        HKEY_CLASSES_ROOT,
        core::ptr::addr_of_mut!(szKey) as LPCSTR,
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterExtensionForMIMETypeW(lpszType: LPCWSTR) -> BOOL {
    let mut szKey: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if GetMIMETypeSubKeyW(lpszType, core::ptr::addr_of_mut!(szKey) as *mut WCHAR as LPWSTR, MAX_PATH) == 0 {
        return (FALSE) as BOOL;
    }

    if SHDeleteValueW(HKEY_CLASSES_ROOT, core::ptr::addr_of_mut!(szKey) as *mut WCHAR as LPCWSTR, core::ptr::addr_of!(szExtensionW) as LPCWSTR) == 0 {
        return (FALSE) as BOOL;
    }

    if SHDeleteOrphanKeyW(HKEY_CLASSES_ROOT, core::ptr::addr_of_mut!(szKey) as *mut WCHAR as LPCWSTR) == 0 {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegDuplicateHKey(hKey: HKEY) -> HKEY {
    let mut newKey: HKEY = core::ptr::null_mut();

    RegOpenKeyExA(hKey, core::ptr::null(), 0, MAXIMUM_ALLOWED, core::ptr::addr_of_mut!(newKey));

    return newKey;
}

#[no_mangle]
pub unsafe extern "system" fn SHCopyKeyA(
    hKeySrc: HKEY,
    lpszSrcSubKey: LPCSTR,
    hKeyDst: HKEY,
    dwReserved: DWORD,
) -> DWORD {
    let mut szSubKeyW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    WINE_TRACE!("(hkey=%p,%s,%p08x,%d)\n", hKeySrc, debugstr_a(lpszSrcSubKey as *const ::core::ffi::c_char), hKeyDst, dwReserved);

    if !lpszSrcSubKey.is_null() {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszSrcSubKey,
            -(1 as ::core::ffi::c_int),
            core::ptr::addr_of_mut!(szSubKeyW) as LPWSTR,
            MAX_PATH as ::core::ffi::c_int,
        );
    }

    return SHCopyKeyW(
        hKeySrc,
        (if !lpszSrcSubKey.is_null() {
            core::ptr::addr_of_mut!(szSubKeyW) as *mut WCHAR
        } else {
            core::ptr::null_mut::<WCHAR>()
        }) as LPCWSTR,
        hKeyDst,
        dwReserved,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHCopyKeyW(
    mut hKeySrc: HKEY,
    mut lpszSrcSubKey: LPCWSTR,
    mut hKeyDst: HKEY,
    mut dwReserved: DWORD,
) -> DWORD {
    let mut dwKeyCount: DWORD = 0 as DWORD;
    let mut dwValueCount: DWORD = 0 as DWORD;
    let mut dwMaxKeyLen: DWORD = 0 as DWORD;
    let mut dwMaxValueLen: DWORD = 0 as DWORD;
    let mut dwMaxDataLen: DWORD = 0 as DWORD;
    let mut i: DWORD = 0;
    let mut buff: [BYTE; 1024] = [0; 1024];
    let mut lpBuff: LPVOID = core::ptr::addr_of_mut!(buff) as *mut BYTE as LPVOID;
    let mut szName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszName: *mut WCHAR = core::ptr::addr_of_mut!(szName) as *mut WCHAR;
    let mut dwRet: DWORD = S_OK as DWORD;

    WINE_TRACE!("hkey=%p,%s,%p08x,%d)\n", hKeySrc, debugstr_w(lpszSrcSubKey as *const WCHAR), hKeyDst, dwReserved);

    if hKeyDst.is_null() || hKeySrc.is_null() {
        dwRet = ERROR_INVALID_PARAMETER as DWORD;
    } else {
        /* Open source key */
        if !lpszSrcSubKey.is_null() {
            dwRet = RegOpenKeyExW(
                hKeySrc,
                lpszSrcSubKey,
                0 as DWORD,
                KEY_ALL_ACCESS as REGSAM,
                core::ptr::addr_of_mut!(hKeySrc),
            ) as DWORD;
        }

        if dwRet != 0 {
            hKeyDst = core::ptr::null_mut(); /* Don't close this key since we didn't open it */
        } else {
            /* Get details about sub keys and values */
            dwRet = RegQueryInfoKeyW(
                hKeySrc,
                core::ptr::null_mut::<WCHAR>(),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(dwKeyCount),
                core::ptr::addr_of_mut!(dwMaxKeyLen),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(dwValueCount),
                core::ptr::addr_of_mut!(dwMaxValueLen),
                core::ptr::addr_of_mut!(dwMaxDataLen),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<_FILETIME>(),
            ) as DWORD;
            if dwRet == 0 {
                if dwMaxValueLen > dwMaxKeyLen {
                    dwMaxKeyLen = dwMaxValueLen; /* Get max size for key/value names */
                }

                let fresh15 = dwMaxKeyLen;
                dwMaxKeyLen = dwMaxKeyLen.wrapping_add(1);
                if fresh15 > (MAX_PATH - 1) as DWORD {
                    lpszName = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        (dwMaxKeyLen as SIZE_T)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                    ) as *mut WCHAR;
                }

                if dwMaxDataLen as usize > core::mem::size_of::<[BYTE; 1024]>() as usize {
                    lpBuff = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        dwMaxDataLen as SIZE_T,
                    ) as LPVOID;
                }

                if lpszName.is_null() || lpBuff.is_null() {
                    dwRet = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                }
            }
        }
    }

    /* Copy all the sub keys */
    i = 0 as DWORD;
    while i < dwKeyCount && dwRet == 0 {
        let mut hSubKeySrc: HKEY = core::ptr::null_mut();
        let mut hSubKeyDst: HKEY = core::ptr::null_mut();
        let mut dwSize: DWORD = dwMaxKeyLen;

        dwRet = RegEnumKeyExW(
            hKeySrc,
            i,
            lpszName as LPWSTR,
            core::ptr::addr_of_mut!(dwSize),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<_FILETIME>(),
        ) as DWORD;

        if dwRet == 0 {
            /* Open source sub key */
            dwRet = RegOpenKeyExW(
                hKeySrc,
                lpszName as LPCWSTR,
                0 as DWORD,
                KEY_READ as REGSAM,
                core::ptr::addr_of_mut!(hSubKeySrc),
            ) as DWORD;

            if dwRet == 0 {
                /* Create destination sub key */
                dwRet = RegCreateKeyW(
                    hKeyDst,
                    lpszName as LPCWSTR,
                    core::ptr::addr_of_mut!(hSubKeyDst),
                ) as DWORD;

                if dwRet == 0 {
                    /* Recursively copy keys and values from the sub key */
                    dwRet = SHCopyKeyW(
                        hSubKeySrc,
                        core::ptr::null::<WCHAR>(),
                        hSubKeyDst,
                        0 as DWORD,
                    );
                    RegCloseKey(hSubKeyDst);
                }
            }
            RegCloseKey(hSubKeySrc);
        }
        i = i.wrapping_add(1);
    }

    /* Copy all the values in this key */
    i = 0 as DWORD;
    while i < dwValueCount && dwRet == 0 {
        let mut dwNameSize: DWORD = dwMaxKeyLen;
        let mut dwType: DWORD = 0;
        let mut dwLen: DWORD = dwMaxDataLen;

        dwRet = RegEnumValueW(
            hKeySrc,
            i,
            lpszName as LPWSTR,
            core::ptr::addr_of_mut!(dwNameSize),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwType),
            lpBuff as LPBYTE,
            core::ptr::addr_of_mut!(dwLen),
        ) as DWORD;

        if dwRet == 0 {
            dwRet = SHSetValueW(
                hKeyDst,
                core::ptr::null::<WCHAR>(),
                lpszName as LPCWSTR,
                dwType,
                lpBuff as LPCVOID,
                dwLen,
            );
        }
        i = i.wrapping_add(1);
    }

    /* Free buffers if allocated */
    if lpszName != core::ptr::addr_of_mut!(szName) as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpszName as PVOID,
        );
    }
    if lpBuff != core::ptr::addr_of_mut!(buff) as *mut BYTE as LPVOID {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpBuff as PVOID,
        );
    }

    if !lpszSrcSubKey.is_null() && !hKeyDst.is_null() {
        RegCloseKey(hKeyDst);
    }
    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetIntW(hKey: HKEY, lpszValue: LPCWSTR, iDefault: i32) -> i32 {
    TRACE!("(%p,%s,%d)\n", hKey, debugstr_w(lpszValue), iDefault);

    if !hKey.is_null() {
        let mut szBuff: [WCHAR; 32] = [0; 32];
        let mut dwSize: DWORD = core::mem::size_of::<[WCHAR; 32]>() as DWORD;
        szBuff[0] = '\0' as WCHAR;
        SHQueryValueExW(
            hKey,
            lpszValue,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            szBuff.as_mut_ptr() as LPVOID,
            core::ptr::addr_of_mut!(dwSize),
        );

        if *(szBuff.as_ptr()) as i32 >= '0' as i32 && *(szBuff.as_ptr()) as i32 <= '9' as i32 {
            return StrToIntW(szBuff.as_mut_ptr() as LPCWSTR);
        }
    }
    return iDefault;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetCLSIDKeyA(guid: *const GUID, lpszValue: LPCSTR, bUseHKCU: BOOL, bCreate: BOOL, phKey: PHKEY) -> HRESULT {
    let mut szValue: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if !lpszValue.is_null() {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszValue,
            -(1 as i32),
            core::ptr::addr_of_mut!(szValue) as LPWSTR,
            (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as i32,
        );
    }

    return SHRegGetCLSIDKeyW(
        guid,
        (if !lpszValue.is_null() {
            core::ptr::addr_of_mut!(szValue) as *mut WCHAR
        } else {
            core::ptr::null_mut::<WCHAR>()
        }) as LPCWSTR,
        bUseHKCU,
        bCreate,
        phKey,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetCLSIDKeyW(
    guid: *const GUID,
    lpszValue: LPCWSTR,
    bUseHKCU: BOOL,
    bCreate: BOOL,
    phKey: PHKEY,
) -> HRESULT {
    const REACTOS: bool = true;
    static szClassIdKey: [WCHAR; 57] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072,
    0x005C, 0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C,
];
    const szClassIdKeyLen: usize =
        core::mem::size_of::<[WCHAR; 57]>() / core::mem::size_of::<WCHAR>();
    let mut szKey: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwRet: DWORD = 0;
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut ptr: *mut WCHAR = core::ptr::null_mut();

    /* Create the key string */
    {
        wcscpy(
            core::ptr::addr_of_mut!(szKey) as *mut WCHAR,
            if bUseHKCU != 0 {
                RUSTCONV_WSTR_47EEE22AAF35.as_ptr()
            } else {
                RUSTCONV_WSTR_6E342AB1A0F1.as_ptr()
            },
        );
        ptr = (core::ptr::addr_of_mut!(szKey) as *mut WCHAR).add(
            wcslen(core::ptr::addr_of_mut!(szKey) as *mut WCHAR),
        );
        SHStringFromGUIDW(guid, ptr, 39); /* Append guid */
        if !lpszValue.is_null() {
            ptr = (core::ptr::addr_of_mut!(szKey) as *mut WCHAR).add(
                wcslen(core::ptr::addr_of_mut!(szKey) as *mut WCHAR),
            );
            wcscat(ptr, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
            ptr = ptr.add(1);
            wcscat(ptr, lpszValue);
        }
    }

    hkey = if bUseHKCU != 0 {
        HKEY_CURRENT_USER
    } else {
        HKEY_CLASSES_ROOT
    };

    if bCreate != 0 {
        dwRet = RegCreateKeyW(
            hkey,
            core::ptr::addr_of_mut!(szKey) as *mut WCHAR as LPCWSTR,
            phKey,
        ) as DWORD;
    } else {
        dwRet = RegOpenKeyExW(
            hkey,
            core::ptr::addr_of_mut!(szKey) as *mut WCHAR as LPCWSTR,
            0,
            KEY_READ,
            phKey,
        ) as DWORD;
    }

    return if dwRet != 0 {
        HRESULT_FROM_WIN32!(dwRet)
    } else {
        S_OK
    };
}

#[no_mangle]
pub unsafe extern "system" fn SHRegisterValidateTemplate(
    filename: LPCWSTR,
    unknown: BOOL,
) -> HRESULT {
    // FIXME("stub: %s, %08x\n", debugstr_w(filename), unknown);
    S_OK
}
