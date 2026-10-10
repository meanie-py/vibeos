use crate::imm32::ime::ImmGetImeInfoEx;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_060243EB88A5: [u16; 5] = [0x004Cu16, 0x005A, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_0C671556E060: [u16; 16] = [
    0x0046u16, 0x0069, 0x006C, 0x0065, 0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069,
    0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_27D4CABE8F51: [u16; 26] = [
    0x005Cu16, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0025, 0x0030, 0x0034, 0x0058,
    0x0025, 0x0030, 0x0034, 0x0058, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_41B968879495: [u16; 11] = [
    0x006Bu16, 0x0062, 0x0064, 0x006A, 0x0070, 0x006E, 0x002E, 0x0064, 0x006C, 0x006C,
    0x0000,
];
pub static RUSTCONV_WSTR_46DDC7E3208A: [u16; 10] = [0x006Bu16, 0x0062, 0x0064, 0x0075, 0x0073, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_4F11850FAC45: [u16; 5] = [0x0025u16, 0x0030, 0x0038, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_65DF0D412656: [u16; 12] = [
    0x004Cu16, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_6AC3419A3E2E: [u16; 25] = [
    0x005Cu16, 0x0056, 0x0061, 0x0072, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E,
    0x0066, 0x006F, 0x005C, 0x0054, 0x0072, 0x0061, 0x006E, 0x0073, 0x006C, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_6F4B49BAF187: [u16; 12] = [
    0x004Cu16, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0054, 0x0065, 0x0078,
    0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_A1F295F00EBC: [u16; 24] = [
    0x004Bu16, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0020, 0x004C,
    0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x005C, 0x0050, 0x0072, 0x0065, 0x006C,
    0x006F, 0x0061, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_B53C7B64F5D8: [u16; 11] = [
    0x006Bu16, 0x0062, 0x0064, 0x006B, 0x006F, 0x0072, 0x002E, 0x0064, 0x006C, 0x006C,
    0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_D00C3BD73B38: [u16; 9] = [0x0049u16, 0x006D, 0x0065, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_D11304080ED2: [u16; 26] = [
    0x005Cu16, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0025, 0x0030, 0x0034, 0x0058,
    0x0030, 0x0034, 0x0042, 0x0030, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_D41D2DB41CC2: [u16; 12] = [
    0x0076u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_D9C0B63D2C56: [u16; 4] = [0x003Au16, 0x005C, 0x002F, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const REGKEY_KEYBOARD_LAYOUTS: [::core::ffi::c_ushort; 50] = unsafe {
    ::core::mem::transmute::<
        [u8; 100],
        [::core::ffi::c_ushort; 50],
    >(
        *b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0K\0e\0y\0b\0o\0a\0r\0d\0 \0L\0a\0y\0o\0u\0t\0s\0\0\0",
    )
};

static mut s_fnGetFileVersionInfoW: FN_GetFileVersionInfoW = None;

static mut s_fnGetFileVersionInfoSizeW: FN_GetFileVersionInfoSizeW = None;

static mut s_fnVerQueryValueW: FN_VerQueryValueW = None;

#[repr(C)]
pub struct tagREG_IME_LAYOUT {
    pub hKL: HKL,
    pub szImeKey: [WCHAR; 20],
    pub szFileName: [WCHAR; 80],
}
pub type REG_IME_LAYOUT = tagREG_IME_LAYOUT;
pub type PREG_IME_LAYOUT = *mut tagREG_IME_LAYOUT;

pub type FN_GetFileVersionInfoW =
    Option<unsafe extern "system" fn(LPCWSTR, DWORD, DWORD, LPVOID) -> BOOL>;
pub type FN_GetFileVersionInfoSizeW =
    Option<unsafe extern "system" fn(LPCWSTR, LPDWORD) -> DWORD>;
pub type FN_VerQueryValueW =
    Option<unsafe extern "system" fn(LPCVOID, LPCWSTR, *mut LPVOID, PUINT) -> BOOL>;

pub type FN_LZOpenFileW =
    Option<unsafe extern "system" fn(LPWSTR, LPOFSTRUCT, WORD) -> INT>;
pub type FN_LZCopy =
    Option<unsafe extern "system" fn(INT, INT) -> LONG>;
pub type FN_LZClose =
    Option<unsafe extern "system" fn(INT)>;

unsafe extern "C" fn Imm32GetVerInfoValue(
    mut pVerInfo: LPCVOID,
    mut cbVerInfo: DWORD,
    mut pszKey: PWSTR,
    mut cchKey: DWORD,
    mut pszName: PCWSTR,
) -> LPWSTR {
    let mut cchExtra: size_t = 0;
    let mut pszValue: LPWSTR = core::ptr::null_mut();
    let mut cbValue: UINT = 0;

    StringCchLengthW(
        pszKey as STRSAFE_LPCWSTR,
        cchKey as size_t,
        core::ptr::addr_of_mut!(cchExtra),
    );

    StringCchCatW(
        pszKey as STRSAFE_LPWSTR,
        cchKey as size_t,
        pszName as STRSAFE_LPCWSTR,
    );
    s_fnVerQueryValueW.unwrap_unchecked()(
        pVerInfo,
        pszKey as LPCWSTR,
        core::ptr::addr_of_mut!(pszValue) as *mut LPVOID,
        core::ptr::addr_of_mut!(cbValue),
    );
    *pszKey.offset(cchExtra as isize) = UNICODE_NULL;

    if !(cbValue as DWORD <= cbVerInfo) {
        RtlAssert(
            b"cbValue <= cbVerInfo\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"win32ss/user/imm32/install.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            53 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    return if cbValue != 0 {
        pszValue
    } else {
        core::ptr::null_mut()
    };
}

unsafe extern "C" fn Imm32LoadImeLangAndDesc(
    mut pInfoEx: PIMEINFOEX,
    mut pVerInfo: LPCVOID,
    mut cbVerInfo: DWORD,
) -> BOOL {
    /* Getting the version info. See VerQueryValue */
    let mut pw: LPWORD = core::ptr::null_mut();
    let mut cbData: UINT = 0;
    let ret: BOOL = (s_fnVerQueryValueW.unwrap_unchecked())(
        pVerInfo,
        RUSTCONV_WSTR_6AC3419A3E2E.as_ptr(),
        &raw mut pw as *mut LPVOID,
        &raw mut cbData,
    );
    if ret == 0 || cbData == 0 {
        return (FALSE) as BOOL;
    }

    if (*pInfoEx).hkl.is_null() {
        (*pInfoEx).hkl = UlongToHandle!(*pw) as HKL;
    }

    /* Try the current language and the Unicode codepage (0x04B0) */
    let LangID: LANGID = LANGIDFROMLCID!(GetThreadLocale());
    let mut szKey: [WCHAR; 80] = [0; 80];
    StringCchPrintfW!(
        &raw mut szKey as PWSTR,
        _countof!(szKey),
        RUSTCONV_WSTR_D11304080ED2.as_ptr(),
        LangID as core::ffi::c_uint,
    );
    let mut pszDesc: LPWSTR = Imm32GetVerInfoValue(
        pVerInfo,
        cbVerInfo,
        &raw mut szKey as PWSTR,
        _countof!(szKey),
        RUSTCONV_WSTR_0C671556E060.as_ptr(),
    );
    if pszDesc.is_null() {
        /* Retry the language and codepage of the IME module */
        StringCchPrintfW!(
            &raw mut szKey as PWSTR,
            _countof!(szKey),
            RUSTCONV_WSTR_27D4CABE8F51.as_ptr(),
            *pw.add(0 as usize) as core::ffi::c_uint,
            *pw.add(1 as usize) as core::ffi::c_uint,
        );
        pszDesc = Imm32GetVerInfoValue(
            pVerInfo,
            cbVerInfo,
            &raw mut szKey as PWSTR,
            _countof!(szKey),
            RUSTCONV_WSTR_0C671556E060.as_ptr(),
        );
    }

    /* The description */
    if !pszDesc.is_null() {
        StringCchCopyW(
            &raw mut (*pInfoEx).wszImeDescription as LPWSTR,
            (_countof!((*pInfoEx).wszImeDescription)) as _,
            pszDesc,
        );
    } else {
        (*pInfoEx).wszImeDescription[0 as usize] = UNICODE_NULL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32LoadImeFixedInfo(
    mut pInfoEx: PIMEINFOEX,
    mut pVerInfo: LPCVOID,
    mut cbVerInfo: DWORD,
) -> BOOL {
    let mut cbFixed: UINT = 0;
    let mut pFixed: *mut VS_FIXEDFILEINFO = core::ptr::null_mut::<VS_FIXEDFILEINFO>();
    if (s_fnVerQueryValueW.unwrap_unchecked())(
        pVerInfo,
        RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
        &raw mut pFixed as *mut LPVOID,
        &raw mut cbFixed,
    ) == 0
        || cbFixed == 0
    {
        return (FALSE) as BOOL;
    }

    /* NOTE: The IME module must contain a version info of input method driver. */
    if (*pFixed).dwFileType != VFT_DRV || (*pFixed).dwFileSubtype != VFT2_DRV_INPUTMETHOD
    {
        return (FALSE) as BOOL;
    }

    (*pInfoEx).dwProdVersion = (*pFixed).dwProductVersionMS;
    (*pInfoEx).dwImeWinVersion = IMEVER_0400;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32AssignNewLayout(mut cKLs: UINT, mut pLayouts: *const REG_IME_LAYOUT, mut wLangID: LANGID) -> HKL {
    let mut iKL: UINT = 0;
    let mut nID: UINT = 0;
    let mut wLow: UINT = MAX_USER_IMM_IME_ID as UINT;
    let mut wHigh: UINT = (MIN_USER_IMM_IME_ID as UINT).wrapping_sub(1 as UINT);
    iKL = 0 as UINT;
    while iKL < cKLs {
        wHigh = max(wHigh, (HIWORD!((*pLayouts.offset(iKL as isize)).hKL)) as _);
        wLow = min(wLow, (HIWORD!((*pLayouts.offset(iKL as isize)).hKL)) as _);
        iKL = iKL.wrapping_add(1);
    }
    if cKLs == 0 || wHigh.wrapping_add(1 as UINT) <= MAX_SYSTEM_IMM_IME_ID as UINT {
        nID = wHigh.wrapping_add(1 as UINT);
    } else if wLow.wrapping_sub(1 as UINT) > MIN_SYSTEM_IMM_IME_ID as UINT {
        nID = wLow.wrapping_sub(1 as UINT);
    } else {
        nID = MIN_USER_IMM_IME_ID as UINT;
        while nID <= MAX_USER_IMM_IME_ID as UINT {
            let mut hKL: HKL = LongToHandle!(MAKELONG!(wLangID, nID)) as HKL;
            iKL = 0 as UINT;
            while iKL < cKLs {
                if (*pLayouts.offset(iKL as isize)).hKL == hKL {
                    break;
                }
                iKL = iKL.wrapping_add(1);
            }
            if iKL >= cKLs {
                break;
            }
            nID = nID.wrapping_add(1);
        }
        if nID > MAX_USER_IMM_IME_ID as UINT {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/user/imm32/install.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32AssignNewLayout\0" as *const u8 as *const ::core::ffi::c_char,
                    173 as ::core::ffi::c_int,
                    b"No empty room for new IME HKL\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return core::ptr::null_mut();
        }
    }
    return UlongToHandle!(MAKELONG!(wLangID, nID)) as HKL;
}

unsafe extern "C" fn Imm32GetImeLayoutList(
    mut pLayouts: PREG_IME_LAYOUT,
    mut cLayouts: UINT,
) -> UINT {
    let mut hkeyLayouts: HKEY = core::ptr::null_mut();
    let mut lError: LSTATUS = RegOpenKeyW(
        HKEY_LOCAL_MACHINE,
        (REGKEY_KEYBOARD_LAYOUTS).as_ptr(),
        core::ptr::addr_of_mut!(hkeyLayouts),
    );
    if IS_ERROR_UNEXPECTEDLY!(lError) != false {
        return 0;
    }

    let mut iKey: UINT = 0;
    let mut nCount: UINT = 0;
    let mut szImeFileName: [WCHAR; 80] = [0; 80];
    let mut szImeKey: [WCHAR; 20] = [0; 20];
    let mut cbData: DWORD = 0;
    let mut Value: DWORD = 0;
    let mut hKL: HKL = core::ptr::null_mut();
    nCount = 0;
    iKey = nCount;
    'key_loop: loop {
        'body: {
            /* Get the key name */
            lError = RegEnumKeyW(
                hkeyLayouts,
                iKey as DWORD,
                core::ptr::addr_of_mut!(szImeKey) as LPWSTR,
                _countof!(szImeKey) as DWORD,
            );
            if (lError) as u32 != ERROR_SUCCESS {
                break 'key_loop;
            }

            if szImeKey[0] != 0x0045u16
                && szImeKey[0] != 0x0065u16
            {
                break 'body; /* Not an IME layout */
            }

            if pLayouts.is_null() {
                /* for counting only */
                nCount = nCount.wrapping_add(1);
                break 'body;
            }

            if cLayouts <= nCount {
                break 'key_loop;
            }

            let mut hkeyIME: HKEY = core::ptr::null_mut();
            lError = RegOpenKeyW(
                hkeyLayouts,
                core::ptr::addr_of_mut!(szImeKey) as LPCWSTR,
                core::ptr::addr_of_mut!(hkeyIME),
            ); /* Open the IME key */
            if IS_ERROR_UNEXPECTEDLY!(lError) != false {
                break 'body;
            }

            /* Load the "Ime File" value */
            szImeFileName[0] = UNICODE_NULL;
            cbData = core::mem::size_of::<[WCHAR; 80]>() as DWORD;
            RegQueryValueExW(
                hkeyIME,
                RUSTCONV_WSTR_D00C3BD73B38.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szImeFileName) as LPBYTE,
                core::ptr::addr_of_mut!(cbData),
            );
            szImeFileName[(_countof!(szImeFileName) - 1) as usize] = UNICODE_NULL; /* Avoid buffer overrun */

            RegCloseKey(hkeyIME);

            /* We don't allow the invalid "IME File" values for security reason */
            if szImeFileName[0] == 0
                || wcscspn(
                    core::ptr::addr_of_mut!(szImeFileName) as *mut WCHAR,
                    RUSTCONV_WSTR_D9C0B63D2C56.as_ptr(),
                ) != wcslen(core::ptr::addr_of_mut!(szImeFileName) as *mut WCHAR)
            {
                break 'body;
            }

            Imm32StrToUInt(
                core::ptr::addr_of_mut!(szImeKey) as PCWSTR,
                core::ptr::addr_of_mut!(Value),
                16 as ULONG,
            );
            hKL = UlongToHandle!(Value) as HKL;
            if IS_IME_HKL!(hKL) == false {
                /* Not an IMM IME */
                break 'body;
            }

            /* Store the IME key and the IME filename */
            (*pLayouts.offset(nCount as isize)).hKL = hKL;
            StringCchCopyW(
                core::ptr::addr_of_mut!((*pLayouts.offset(nCount as isize)).szImeKey) as LPWSTR,
                (_countof!((*pLayouts.offset(nCount as isize)).szImeKey)) as _,
                core::ptr::addr_of_mut!(szImeKey) as LPCWSTR,
            );
            StringCchCopyW(
                core::ptr::addr_of_mut!((*pLayouts.offset(nCount as isize)).szFileName) as LPWSTR,
                (_countof!((*pLayouts.offset(nCount as isize)).szFileName)) as _,
                core::ptr::addr_of_mut!(szImeFileName) as LPCWSTR,
            );
            nCount = nCount.wrapping_add(1);
        }
        iKey = iKey.wrapping_add(1);
    }

    RegCloseKey(hkeyLayouts);
    return nCount;
}

unsafe extern "C" fn Imm32WriteImeLayout(
    hKL: HKL,
    pchFileTitle: PCWSTR,
    pszLayoutText: PCWSTR,
) -> BOOL {
    let mut iPreload: UINT = 0;
    let mut hkeyLayouts: HKEY = core::ptr::null_mut();
    let mut hkeyIME: HKEY = core::ptr::null_mut();
    let mut hkeyPreload: HKEY = core::ptr::null_mut();
    let mut szImeKey: [WCHAR; 20] = [0; 20];
    let mut szPreloadNumber: [WCHAR; 20] = [0; 20];
    let mut szPreloadKey: [WCHAR; 20] = [0; 20];
    let mut cbData: DWORD = 0;
    let mut LangID: LANGID = 0;
    let mut lError: LONG = 0;
    let mut pszLayoutFile: LPCWSTR = core::ptr::null();

    /* Open the registry keyboard layouts */
    lError = RegOpenKeyW(
        HKEY_LOCAL_MACHINE,
        (REGKEY_KEYBOARD_LAYOUTS).as_ptr(),
        core::ptr::addr_of_mut!(hkeyLayouts),
    ) as LONG;
    if IS_ERROR_UNEXPECTEDLY!(lError) != false {
        return (FALSE) as BOOL;
    }

    /* Get the IME key from hKL */
    StringCchPrintfW!(
        szImeKey.as_mut_ptr(),
        _countof!(szImeKey),
        RUSTCONV_WSTR_4F11850FAC45.as_ptr(),
        HandleToUlong!(hKL),
    );

    /* Create a registry IME key */
    'failure: {
        lError = RegCreateKeyW(
            hkeyLayouts,
            szImeKey.as_ptr(),
            core::ptr::addr_of_mut!(hkeyIME),
        ) as LONG;
        if IS_ERROR_UNEXPECTEDLY!(lError) != false {
            break 'failure;
        }

        /* Write "Ime File" */
        cbData = ((wcslen(pchFileTitle) + 1)
            * core::mem::size_of::<WCHAR>()) as DWORD;
        lError = RegSetValueExW(
            hkeyIME,
            RUSTCONV_WSTR_D00C3BD73B38.as_ptr(),
            0,
            REG_SZ,
            pchFileTitle as *const u8,
            cbData,
        ) as LONG;
        if IS_ERROR_UNEXPECTEDLY!(lError) != false {
            break 'failure;
        }

        /* Write "Layout Text" */
        cbData = ((wcslen(pszLayoutText) + 1)
            * core::mem::size_of::<WCHAR>()) as DWORD;
        lError = RegSetValueExW(
            hkeyIME,
            RUSTCONV_WSTR_6F4B49BAF187.as_ptr(),
            0,
            REG_SZ,
            pszLayoutText as *const u8,
            cbData,
        ) as LONG;
        if IS_ERROR_UNEXPECTEDLY!(lError) != false {
            break 'failure;
        }

        /* Choose "Layout File" from hKL */
        LangID = LOWORD!(hKL);
        let sw = LOBYTE!(LangID);
        if (sw) as u32 == LANG_JAPANESE {
            pszLayoutFile = RUSTCONV_WSTR_41B968879495.as_ptr();
        } else if (sw) as u32 == LANG_KOREAN {
            pszLayoutFile = RUSTCONV_WSTR_B53C7B64F5D8.as_ptr();
        } else {
            pszLayoutFile = RUSTCONV_WSTR_46DDC7E3208A.as_ptr();
        }

        /* Write "Layout File" */
        cbData = ((wcslen(pszLayoutFile) + 1)
            * core::mem::size_of::<WCHAR>()) as DWORD;
        lError = RegSetValueExW(
            hkeyIME,
            RUSTCONV_WSTR_65DF0D412656.as_ptr(),
            0,
            REG_SZ,
            pszLayoutFile as *const u8,
            cbData,
        ) as LONG;
        if IS_ERROR_UNEXPECTEDLY!(lError) != false {
            break 'failure;
        }

        RegCloseKey(hkeyIME);
        RegCloseKey(hkeyLayouts);

        /* Create "Preload" key */
        RegCreateKeyW(
            HKEY_CURRENT_USER,
            RUSTCONV_WSTR_A1F295F00EBC.as_ptr(),
            core::ptr::addr_of_mut!(hkeyPreload),
        );

        // #define MAX_PRELOAD 0x400
        iPreload = 1;
        while iPreload < MAX_PRELOAD {
            Imm32UIntToStr(
                iPreload,
                10,
                szPreloadNumber.as_mut_ptr(),
                (_countof!(szPreloadNumber)) as USHORT,
            );

            /* Load the key of the preload number */
            cbData = core::mem::size_of::<[WCHAR; 20]>() as DWORD;
            lError = RegQueryValueExW(
                hkeyPreload,
                szPreloadNumber.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                szPreloadKey.as_mut_ptr() as *mut u8,
                core::ptr::addr_of_mut!(cbData),
            ) as LONG;
            szPreloadKey[(_countof!(szPreloadKey) - 1) as usize] =
                UNICODE_NULL; /* Avoid buffer overrun */

            if (lError) as u32 != ERROR_SUCCESS
                || _wcsicmp(szImeKey.as_ptr(), szPreloadKey.as_ptr())
                    == 0
            {
                break; /* Found an empty room or the same key */
            }
            iPreload = iPreload.wrapping_add(1);
        }

        if iPreload >= MAX_PRELOAD {
            /* Not found */
            RegCloseKey(hkeyPreload);
            return (FALSE) as BOOL;
        }
        // #undef MAX_PRELOAD

        /* Write the IME key to the preload number */
        cbData = ((wcslen(szImeKey.as_ptr()) + 1)
            * core::mem::size_of::<WCHAR>()) as DWORD;
        lError = RegSetValueExW(
            hkeyPreload,
            szPreloadNumber.as_ptr(),
            0,
            REG_SZ,
            szImeKey.as_mut_ptr() as *mut u8,
            cbData,
        ) as LONG;
        RegCloseKey(hkeyPreload);
        return ((lError) as u32 == ERROR_SUCCESS) as BOOL;
    }

    /* Failure: */
    RegCloseKey(hkeyIME);
    RegDeleteKeyW(hkeyLayouts, szImeKey.as_ptr());
    RegCloseKey(hkeyLayouts);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn Imm32CopyImeFile(pszOldFile: PCWSTR, pszNewFile: PCWSTR) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut bLoaded: BOOL = (FALSE) as i32;

    /* Load LZ32.dll for copying/decompressing file */
    let mut szLZ32Path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    Imm32GetSystemLibraryPath(
        core::ptr::addr_of_mut!(szLZ32Path) as LPWSTR,
        _countof!(szLZ32Path) as DWORD,
        RUSTCONV_WSTR_060243EB88A5.as_ptr(),
    );
    let mut hinstLZ32: HMODULE = GetModuleHandleW(core::ptr::addr_of_mut!(szLZ32Path) as LPCWSTR);
    if hinstLZ32.is_null() {
        hinstLZ32 = LoadLibraryW(core::ptr::addr_of_mut!(szLZ32Path) as LPCWSTR) as HMODULE;
        if IS_NULL_UNEXPECTEDLY!(hinstLZ32) != false {
            return (FALSE) as BOOL;
        }
        bLoaded = (TRUE) as i32;
    }

    'quit: {
        let mut fnLZOpenFileW: FN_LZOpenFileW = None;
        let mut fnLZCopy: FN_LZCopy = None;
        let mut fnLZClose: FN_LZClose = None;

        /* GET_FN(LZOpenFileW) expanded: */
        fnLZOpenFileW = ::core::mem::transmute::<FARPROC, FN_LZOpenFileW>(GetProcAddress(
            hinstLZ32 as HINSTANCE,
            b"LZOpenFileW\0".as_ptr() as LPCSTR,
        ));
        if fnLZOpenFileW.is_none() {
            break 'quit;
        }
        /* GET_FN(LZCopy) expanded: */
        fnLZCopy = ::core::mem::transmute::<FARPROC, FN_LZCopy>(GetProcAddress(
            hinstLZ32 as HINSTANCE,
            b"LZCopy\0".as_ptr() as LPCSTR,
        ));
        if fnLZCopy.is_none() {
            break 'quit;
        }
        /* GET_FN(LZClose) expanded: */
        fnLZClose = ::core::mem::transmute::<FARPROC, FN_LZClose>(GetProcAddress(
            hinstLZ32 as HINSTANCE,
            b"LZClose\0".as_ptr() as LPCSTR,
        ));
        if fnLZClose.is_none() {
            break 'quit;
        }

        let mut szDestA: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut hfDest: HFILE = 0;
        let mut hfSrc: HFILE = 0;
        let mut OFStruct: OFSTRUCT = core::mem::zeroed();

        if WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            pszNewFile as LPCWSTR,
            -1,
            core::ptr::addr_of_mut!(szDestA) as LPSTR,
            _countof!(szDestA) as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) == 0
        {
            break 'quit;
        }
        szDestA[(_countof!(szDestA) - 1) as usize] = ANSI_NULL as CHAR; /* Avoid buffer overrun */

        hfSrc = fnLZOpenFileW.unwrap_unchecked()(
            pszOldFile as PWSTR,
            core::ptr::addr_of_mut!(OFStruct),
            OF_READ as WORD,
        ) as HFILE;
        if hfSrc < 0 as HFILE {
            break 'quit;
        }

        hfDest = OpenFile(
            core::ptr::addr_of_mut!(szDestA) as LPCSTR,
            core::ptr::addr_of_mut!(OFStruct),
            OF_CREATE as UINT,
        );
        if hfDest != HFILE_ERROR as HFILE {
            ret = (fnLZCopy.unwrap_unchecked()(
                hfSrc as INT,
                hfDest as INT,
            ) >= 0 as LONG) as BOOL;
            _lclose(hfDest);
        }

        fnLZClose.unwrap_unchecked()(hfSrc as INT);
    }

    if bLoaded != 0 {
        FreeLibrary(hinstLZ32);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32LoadImeVerInfo(pImeInfoEx: PIMEINFOEX) -> BOOL {
    let mut hinstVersion: HINSTANCE = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE) as i32;
    let mut bLoaded: BOOL = (FALSE) as i32;
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pVerInfo: LPVOID = core::ptr::null_mut();
    let mut cbVerInfo: DWORD = 0;
    let mut dwHandle: DWORD = 0;

    /* Load version.dll to use the version info API */
    Imm32GetSystemLibraryPath(
        szPath.as_mut_ptr() as LPWSTR,
        _countof!(szPath) as DWORD,
        RUSTCONV_WSTR_D41D2DB41CC2.as_ptr() as LPCWSTR,
    );
    hinstVersion = GetModuleHandleW(szPath.as_mut_ptr() as LPCWSTR) as HINSTANCE;
    if hinstVersion.is_null() {
        hinstVersion = LoadLibraryW(szPath.as_mut_ptr() as LPCWSTR);
        if IS_NULL_UNEXPECTEDLY!(hinstVersion) != false {
            return (FALSE) as BOOL;
        }

        bLoaded = (TRUE) as i32;
    }

    'quit: {
        /* GET_FN(GetFileVersionInfoW) */
        s_fnGetFileVersionInfoW = ::core::mem::transmute::<FARPROC, FN_GetFileVersionInfoW>(
            GetProcAddress(hinstVersion, b"GetFileVersionInfoW\0".as_ptr() as LPCSTR),
        );
        if s_fnGetFileVersionInfoW.is_none() {
            break 'quit;
        }

        /* GET_FN(GetFileVersionInfoSizeW) */
        s_fnGetFileVersionInfoSizeW =
            ::core::mem::transmute::<FARPROC, FN_GetFileVersionInfoSizeW>(GetProcAddress(
                hinstVersion,
                b"GetFileVersionInfoSizeW\0".as_ptr() as LPCSTR,
            ));
        if s_fnGetFileVersionInfoSizeW.is_none() {
            break 'quit;
        }

        /* GET_FN(VerQueryValueW) */
        s_fnVerQueryValueW = ::core::mem::transmute::<FARPROC, FN_VerQueryValueW>(
            GetProcAddress(hinstVersion, b"VerQueryValueW\0".as_ptr() as LPCSTR),
        );
        if s_fnVerQueryValueW.is_none() {
            break 'quit;
        }

        /* The path of the IME module */
        Imm32GetSystemLibraryPath(
            szPath.as_mut_ptr() as LPWSTR,
            _countof!(szPath) as DWORD,
            (*pImeInfoEx).wszImeFile.as_mut_ptr() as LPCWSTR,
        );

        cbVerInfo = (s_fnGetFileVersionInfoSizeW.unwrap_unchecked())(
            szPath.as_mut_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!(dwHandle),
        );
        if IS_ZERO_UNEXPECTEDLY!(cbVerInfo) != false {
            break 'quit;
        }

        pVerInfo = ImmLocalAlloc(0, cbVerInfo);
        if IS_NULL_UNEXPECTEDLY!(pVerInfo) != false {
            break 'quit;
        }

        /* Load the version info of the IME module */
        if (s_fnGetFileVersionInfoW.unwrap_unchecked())(
            szPath.as_mut_ptr() as LPCWSTR,
            dwHandle,
            cbVerInfo,
            pVerInfo,
        ) != 0
            && Imm32LoadImeFixedInfo(pImeInfoEx, pVerInfo as LPCVOID, cbVerInfo) != 0
        {
            ret = Imm32LoadImeLangAndDesc(pImeInfoEx, pVerInfo as LPCVOID, cbVerInfo);
        }

        ImmLocalFree!(pVerInfo);
    }

    if bLoaded != 0 {
        FreeLibrary(hinstVersion as HMODULE);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmInstallIMEA(
    lpszIMEFileName: LPCSTR,
    lpszLayoutText: LPCSTR,
) -> HKL {
    let mut hKL: HKL = core::ptr::null_mut();
    let mut pszFileNameW: LPWSTR = core::ptr::null_mut();
    let mut pszLayoutTextW: LPWSTR = core::ptr::null_mut();

    'Quit: {
        pszFileNameW = Imm32WideFromAnsi(CP_ACP, lpszIMEFileName);
        if IS_NULL_UNEXPECTEDLY!(pszFileNameW) != false {
            break 'Quit;
        }

        pszLayoutTextW = Imm32WideFromAnsi(CP_ACP, lpszLayoutText);
        if IS_NULL_UNEXPECTEDLY!(pszLayoutTextW) != false {
            break 'Quit;
        }

        hKL = ImmInstallIMEW(pszFileNameW, pszLayoutTextW);
    }

    ImmLocalFree!((pszFileNameW).cast());
    ImmLocalFree!((pszLayoutTextW).cast());
    return hKL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmInstallIMEW(
    lpszIMEFileName: LPCWSTR,
    lpszLayoutText: LPCWSTR,
) -> HKL {
    /* Get full pathname and file title */
    let mut szImeFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pchFileTitle: PWSTR = core::ptr::null_mut();
    GetFullPathNameW(
        lpszIMEFileName,
        _countof!(szImeFileName),
        addr_of_mut!(szImeFileName) as *mut WCHAR,
        core::ptr::addr_of_mut!(pchFileTitle),
    );
    if IS_NULL_UNEXPECTEDLY!(pchFileTitle) != false {
        return core::ptr::null_mut();
    }

    /* Load the IME version info */
    let mut InfoEx: IMEINFOEX = core::mem::zeroed();
    let mut hNewKL: HKL = core::ptr::null_mut();
    hNewKL = core::ptr::null_mut();
    InfoEx.hkl = hNewKL;
    StringCchCopyW(
        addr_of_mut!(InfoEx.wszImeFile) as *mut WCHAR,
        (_countof!(InfoEx.wszImeFile)) as _,
        pchFileTitle,
    );
    if Imm32LoadImeVerInfo(core::ptr::addr_of_mut!(InfoEx)) == 0 || InfoEx.hkl.is_null() {
        return core::ptr::null_mut();
    }
    let mut wLangID: WORD = LOWORD!(InfoEx.hkl as DWORD_PTR);

    /* Get the IME layouts from registry */
    let mut pLayouts: PREG_IME_LAYOUT = core::ptr::null_mut();
    let mut iLayout: UINT = 0;
    let mut cLayouts: UINT = Imm32GetImeLayoutList(core::ptr::null_mut(), 0);
    if cLayouts != 0 {
        pLayouts = ImmLocalAlloc(
            0,
            (cLayouts as DWORD).wrapping_mul(core::mem::size_of::<REG_IME_LAYOUT>() as DWORD),
        ) as PREG_IME_LAYOUT;
        if IS_NULL_UNEXPECTEDLY!(pLayouts) != false {
            return core::ptr::null_mut();
        }

        if Imm32GetImeLayoutList(pLayouts, cLayouts) == 0 {
            ImmLocalFree!((pLayouts).cast());
            return core::ptr::null_mut();
        }

        iLayout = 0;
        while iLayout < cLayouts {
            if _wcsicmp(
                addr_of_mut!((*pLayouts.offset(iLayout as isize)).szFileName) as *mut WCHAR,
                pchFileTitle as *const WCHAR,
            ) == 0
            {
                if wLangID != LOWORD!((*pLayouts.offset(iLayout as isize)).hKL as DWORD_PTR) {
                    /* goto Quit: handled below by jumping past the main body */
                    return {
                        // Quit path: free layouts and return NULL-or-current handle
                        ImmLocalFree!((pLayouts).cast());
                        hNewKL
                    };
                }

                hNewKL = (*pLayouts.offset(iLayout as isize)).hKL; /* Found */
                break;
            }
            iLayout = iLayout.wrapping_add(1);
        }
    }

    /* If the IME for the specified filename is already loaded, then unload it now */
    if ImmGetImeInfoEx(
        core::ptr::addr_of_mut!(InfoEx),
        ImeInfoExImeFileName,
        (pchFileTitle).cast(),
    ) != 0
        && UnloadKeyboardLayout(InfoEx.hkl) == 0
    {
        hNewKL = core::ptr::null_mut();
        ImmLocalFree!((pLayouts).cast());
        return hNewKL;
    }

    let mut szImeDestPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szImeKey: [WCHAR; 20] = [0; 20];
    Imm32GetSystemLibraryPath(
        addr_of_mut!(szImeDestPath) as *mut WCHAR,
        _countof!(szImeDestPath),
        pchFileTitle as LPCWSTR,
    );

    /* If the source and the destination pathnames were different, then copy the IME file */
    if _wcsicmp(
        addr_of_mut!(szImeFileName) as *mut WCHAR,
        addr_of_mut!(szImeDestPath) as *mut WCHAR,
    ) != 0
        && Imm32CopyImeFile(
            addr_of_mut!(szImeFileName) as *mut WCHAR as PCWSTR,
            addr_of_mut!(szImeDestPath) as *mut WCHAR as PCWSTR,
        ) == 0
    {
        hNewKL = core::ptr::null_mut();
        ImmLocalFree!((pLayouts).cast());
        return hNewKL;
    }

    if hNewKL.is_null() {
        hNewKL = Imm32AssignNewLayout(cLayouts, pLayouts as *const REG_IME_LAYOUT, wLangID as LANGID);
    }

    if !hNewKL.is_null() {
        /* Write the IME layout to registry */
        if Imm32WriteImeLayout(
            hNewKL,
            pchFileTitle as PCWSTR,
            lpszLayoutText as PCWSTR,
        ) != 0
        {
            /* Replace the current keyboard layout */
            StringCchPrintfW!(
                addr_of_mut!(szImeKey) as *mut WCHAR,
                _countof!(szImeKey),
                RUSTCONV_WSTR_4F11850FAC45.as_ptr(),
                HandleToUlong!(hNewKL),
            );
            hNewKL = LoadKeyboardLayoutW(
                addr_of_mut!(szImeKey) as *mut WCHAR as LPCWSTR,
                KLF_REPLACELANG,
            );
        } else {
            hNewKL = core::ptr::null_mut();
        }
    }

    ImmLocalFree!((pLayouts).cast());
    return hNewKL;
}
