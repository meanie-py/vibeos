use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut AppRegSettings: [TCHAR; 65] = unsafe {
    ::core::mem::transmute::<
        [u8; 130],
        [TCHAR; 65],
    >(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0A\0p\0p\0l\0e\0t\0s\0\\\0V\0o\0l\0u\0m\0e\0 \0C\0o\0n\0t\0r\0o\0l\0\0\0",
    )
};

static mut AppOptionsKey: [TCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [TCHAR; 8]>(*b"O\0p\0t\0i\0o\0n\0s\0\0\0") };

static mut LineStatesValue: [TCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [TCHAR; 11]>(*b"L\0i\0n\0e\0S\0t\0a\0t\0e\0s\0\0\0")
};

static mut StyleValue: [TCHAR; 6] =
    unsafe { ::core::mem::transmute::<[u8; 12], [TCHAR; 6]>(*b"S\0t\0y\0l\0e\0\0\0") };

#[no_mangle]
pub static mut hAppSettingsKey: HKEY = ::core::ptr::null::<HKEY__>() as *mut HKEY__;

unsafe extern "C" fn LengthOfStrResource(mut hInst: HINSTANCE, mut uID: UINT) -> INT {
    let mut hrSrc: HRSRC = core::ptr::null_mut::<HRSRC__>();
    let mut hRes: HGLOBAL = core::ptr::null_mut::<core::ffi::c_void>();
    let mut lpName: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut lpStr: LPWSTR = core::ptr::null_mut::<WCHAR>();

    if hInst.is_null() {
        return -1;
    }

    /* There are always blocks of 16 strings */
    lpName = MAKEINTRESOURCE!(((uID >> 4) + 1) as usize) as LPWSTR;

    /* Find the string table block */
    hrSrc = FindResourceW(
        hInst as HMODULE,
        lpName as LPCWSTR,
        RT_STRING as LPCWSTR,
    );
    if !hrSrc.is_null()
        && {
            hRes = LoadResource(hInst, hrSrc);
            !hRes.is_null()
        }
        && {
            lpStr = LockResource(hRes) as LPWSTR;
            !lpStr.is_null()
        }
    {
        let mut x: UINT = 0;

        /* Find the string we're looking for */
        uID &= 0xF; /* position in the block, same as % 16 */
        x = 0;
        while x < uID {
            lpStr = lpStr.offset(((*lpStr) as isize) + 1);
            x = x.wrapping_add(1);
        }

        /* Found the string */
        return *lpStr as INT;
    }
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn AllocAndLoadString(lpTarget: *mut LPWSTR, hInst: HINSTANCE, uID: UINT) -> INT {
    let mut ln: INT = 0;

    ln = LengthOfStrResource(hInst, uID);
    let fresh31 = ln;
    ln = ln + 1;
    if fresh31 > 0 {
        *lpTarget = LocalAlloc(
            LMEM_FIXED,
            (ln as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !(*lpTarget).is_null() {
            let mut Ret: INT = LoadStringW(hInst, uID, *lpTarget, ln) as INT;
            if Ret == 0 {
                LocalFree(*lpTarget as HLOCAL);
            }
            return Ret;
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn LoadAndFormatString_v(
    mut hInstance: HINSTANCE,
    mut uID: UINT,
    mut lpTarget: *mut LPWSTR,
    ap_list: *mut ::core::ffi::c_void,
) -> DWORD {
    let mut Ret: DWORD = 0;
    let mut lpFormat: LPWSTR = core::ptr::null_mut();

    if AllocAndLoadString(core::ptr::addr_of_mut!(lpFormat),
                           hInstance,
                           uID) > 0
    {
        /* let's use FormatMessage to format it because it has the ability to
           allocate memory automatically */
        Ret = FormatMessageW((FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING) as DWORD,
                             lpFormat as LPCVOID,
                             0,
                             0,
                             lpTarget as LPWSTR,
                             0,
                             // the trampoline's `&ap` (rust_varargs_shim.c) is the va_list *
                             ap_list as *mut va_list);

        LocalFree(lpFormat as HLOCAL);
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "C" fn InitAppConfig() -> BOOL {
    return ((RegCreateKeyEx(
        HKEY_CURRENT_USER,
        core::ptr::addr_of!(AppRegSettings) as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hAppSettingsKey),
        core::ptr::null_mut(),
    )) as u32 == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CloseAppConfig() {
    if !hAppSettingsKey.is_null() {
        RegCloseKey(hAppSettingsKey);
        hAppSettingsKey = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn LoadXYCoordWnd(mut PrefContext: PPREFERENCES_CONTEXT) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lResult: LONG = 0;
    let mut DeviceMixerSettings: [TCHAR; 256] = [0; 256];
    let mut dwData: DWORD = 0;
    let mut cbData: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    /* Append the registry key path and device name key into one single string */
    StringCchPrintf!(
        core::ptr::addr_of_mut!(DeviceMixerSettings) as *mut TCHAR,
        _countof!(DeviceMixerSettings),
        (&[0x0025, 0x0073, 0x005c, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* TEXT "%s\\%s" */,
        core::ptr::addr_of!(AppRegSettings) as *const TCHAR,
        core::ptr::addr_of_mut!((*PrefContext).DeviceName) as *mut TCHAR,
    );

    lResult = RegOpenKeyEx(
        HKEY_CURRENT_USER,
        core::ptr::addr_of_mut!(DeviceMixerSettings) as *mut TCHAR as LPCWSTR,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    );
    if (lResult) as u32 != ERROR_SUCCESS {
        return (FALSE) as BOOL;
    }

    lResult = RegQueryValueEx(
        hKey,
        (&[0x0058, 0x0000u16]).as_ptr() /* TEXT "X" */,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwData) as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if (lResult) as u32 != ERROR_SUCCESS {
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }

    /* Cache the X coordinate point */
    (*(*PrefContext).MixerWindow).WndPosX = dwData as INT;

    lResult = RegQueryValueEx(
        hKey,
        (&[0x0059, 0x0000u16]).as_ptr() /* TEXT "Y" */,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwData) as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if (lResult) as u32 != ERROR_SUCCESS {
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }

    /* Cache the Y coordinate point */
    (*(*PrefContext).MixerWindow).WndPosY = dwData as INT;

    RegCloseKey(hKey);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SaveXYCoordWnd(hWnd: HWND, PrefContext: PPREFERENCES_CONTEXT) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lResult: LONG = 0;
    let mut DeviceMixerSettings: [TCHAR; 256] = [0 as TCHAR; 256];
    let mut wp: WINDOWPLACEMENT = core::mem::zeroed();

    /* Get the placement coordinate data from the window */
    wp.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
    GetWindowPlacement(hWnd, core::ptr::addr_of_mut!(wp));

    /* Append the registry key path and device name key into one single string */
    StringCchPrintf!(
        core::ptr::addr_of_mut!(DeviceMixerSettings) as *mut TCHAR,
        _countof!(DeviceMixerSettings),
        (['%' as u16, 's' as u16, '\\' as u16, '%' as u16, 's' as u16, 0u16].as_ptr() as *const TCHAR),
        core::ptr::addr_of!(AppRegSettings) as *const TCHAR,
        core::ptr::addr_of_mut!((*PrefContext).DeviceName) as *mut TCHAR,
    );

    lResult = RegOpenKeyEx(
        HKEY_CURRENT_USER,
        core::ptr::addr_of_mut!(DeviceMixerSettings) as *mut TCHAR as LPCWSTR,
        0 as DWORD,
        KEY_SET_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    if lResult != ERROR_SUCCESS as LONG {
        return (FALSE) as BOOL;
    }

    lResult = RegSetValueEx(
        hKey,
        (['X' as u16, 0u16].as_ptr() as *const TCHAR),
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(wp.rcNormalPosition.left) as LPBYTE as *const BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    ) as LONG;
    if lResult != ERROR_SUCCESS as LONG {
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }

    lResult = RegSetValueEx(
        hKey,
        (['Y' as u16, 0u16].as_ptr() as *const TCHAR),
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(wp.rcNormalPosition.top) as LPBYTE as *const BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    ) as LONG;
    if lResult != ERROR_SUCCESS as LONG {
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }

    RegCloseKey(hKey);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn WriteLineConfig(mut szDeviceName: LPTSTR, mut szLineName: LPTSTR, mut LineState: PSNDVOL_REG_LINESTATE, mut cbSize: DWORD) -> BOOL {
    let mut hLineKey: HKEY = core::ptr::null_mut();
    let mut szDevRegKey: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Ret: BOOL = (FALSE) as i32;

    static FMT_SUBKEY: [u16; 6] = ['%' as u16, 's' as u16, '\\' as u16, '%' as u16, 's' as u16, 0];

    _stprintf(
        core::ptr::addr_of_mut!(szDevRegKey) as *mut TCHAR,
        core::ptr::addr_of!(FMT_SUBKEY) as *const TCHAR,
        szDeviceName,
        szLineName,
    );

    if (RegCreateKeyEx(
        hAppSettingsKey,
        core::ptr::addr_of_mut!(szDevRegKey) as *mut TCHAR as LPCTSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        &raw mut hLineKey,
        core::ptr::null_mut(),
    )) as u32 == ERROR_SUCCESS
    {
        /* now update line states */
        RegSetValueEx(
            hLineKey,
            core::ptr::addr_of!(LineStatesValue) as LPCTSTR,
            0,
            REG_BINARY,
            LineState as *const BYTE,
            cbSize,
        );
        Ret = (TRUE) as i32;

        RegCloseKey(hLineKey);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn ReadLineConfig(szDeviceName: LPTSTR, szLineName: LPTSTR, szControlName: LPTSTR, Flags: *mut DWORD) -> BOOL {
    let mut hLineKey: HKEY = core::ptr::null_mut();
    let mut Type: DWORD = 0;
    let mut i: DWORD = 0;
    let mut Size: DWORD = 0 as DWORD;
    let mut LineStates: PSNDVOL_REG_LINESTATE = core::ptr::null_mut();
    let mut szDevRegKey: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Ret: BOOL = FALSE as BOOL;

    // (&[0x0025, 0x0073, 0x005c, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* TEXT "%s\\%s" */
    let FmtDevRegKey: [u16; 6] = ['%' as u16, 's' as u16, '\\' as u16, '%' as u16, 's' as u16, 0];

    _stprintf(
        szDevRegKey.as_mut_ptr(),
        FmtDevRegKey.as_ptr() as LPCTSTR,
        szDeviceName,
        szLineName,
    );

    if RegCreateKeyEx(
        hAppSettingsKey,
        szDevRegKey.as_mut_ptr(),
        0 as DWORD,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE as DWORD,
        (KEY_READ | KEY_WRITE) as REGSAM,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hLineKey),
        core::ptr::null_mut(),
    ) == ERROR_SUCCESS as LSTATUS
    {
        'ExitClose: {
            if RegQueryValueEx(
                hLineKey,
                core::ptr::addr_of!(LineStatesValue) as LPCTSTR,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Type),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Size),
            ) != ERROR_SUCCESS as LSTATUS
                || Type != REG_BINARY as DWORD
                || Size == 0 as DWORD
                || (Size as usize)
                    % core::mem::size_of::<SNDVOL_REG_LINESTATE>()
                    != 0 as usize
            {
                break 'ExitClose;
            }

            LineStates = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                Size as SIZE_T,
            ) as PSNDVOL_REG_LINESTATE;

            if !LineStates.is_null() {
                if RegQueryValueEx(
                    hLineKey,
                    core::ptr::addr_of!(LineStatesValue) as LPCTSTR,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(Type),
                    LineStates as LPBYTE,
                    core::ptr::addr_of_mut!(Size),
                ) != ERROR_SUCCESS as LSTATUS
                    || Type != REG_BINARY as DWORD
                    || Size == 0 as DWORD
                    || (Size as usize)
                        % core::mem::size_of::<SNDVOL_REG_LINESTATE>()
                        != 0 as usize
                {
                    break 'ExitClose;
                }

                /* try to find the control */
                i = 0 as DWORD;
                while (i as usize)
                    < (Size as usize) / core::mem::size_of::<SNDVOL_REG_LINESTATE>()
                {
                    if _tcscmp(
                        szControlName,
                        core::ptr::addr_of!((*LineStates.offset(i as isize)).LineName) as LPCTSTR,
                    ) == 0
                    {
                        *Flags = (*LineStates.offset(i as isize)).Flags;
                        Ret = TRUE as BOOL;
                        break;
                    }
                    i += 1;
                }
            }
        }

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            LineStates as PVOID,
        );
        RegCloseKey(hLineKey);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn GetStyleValue() -> DWORD {
    let mut hOptionsKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut dwStyle: DWORD = 0 as DWORD;
    let mut dwSize: DWORD = 0;
    if RegOpenKeyEx(
        hAppSettingsKey,
        core::ptr::addr_of!(AppOptionsKey) as LPCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hOptionsKey),
    ) == ERROR_SUCCESS as LSTATUS
    {
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        RegQueryValueEx(
            hOptionsKey,
            core::ptr::addr_of!(StyleValue) as LPCWSTR,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwStyle) as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        );

        RegCloseKey(hOptionsKey);
    }

    return dwStyle;
}
