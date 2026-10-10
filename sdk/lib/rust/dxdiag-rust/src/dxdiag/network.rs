use crate::dxdiag::system::GetRegValue;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_033AA4E0A2D7: [u16; 48] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0044,
    0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x0050, 0x006C, 0x0061, 0x0079, 0x005C,
    0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0050, 0x0072,
    0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_332FB550977A: [u16; 13] = [
    0x0044u16, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0057, 0x0000,
];
pub static RUSTCONV_WSTR_3DCC8E3D3F50: [u16; 49] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0044,
    0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x0050, 0x006C, 0x0061, 0x0079, 0x0038,
    0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0050,
    0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_6C24BFD29528: [u16; 16] = [
    0x005Cu16, 0x0049, 0x006E, 0x0050, 0x0072, 0x006F, 0x0063, 0x0053, 0x0065, 0x0072,
    0x0076, 0x0065, 0x0072, 0x0033, 0x0032, 0x0000,
];
pub static RUSTCONV_WSTR_6E342AB1A0F1: [u16; 7] = [0x0043u16, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_9A56E6BBE687: [u16; 5] = [0x0050u16, 0x0061, 0x0074, 0x0068, 0x0000];
pub static RUSTCONV_WSTR_BA52125E4714: [u16; 14] = [
    0x0046u16, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C, 0x0079, 0x0020, 0x004E,
    0x0061, 0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_C0A6B47AC6CD: [u16; 5] = [0x0047u16, 0x0055, 0x0049, 0x0044, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut DirectPlay8SP: [DIRECTPLAY_GUID; 4] = unsafe {
    [
        DIRECTPLAY_GUID {
            Guid: ::core::mem::transmute::<
                [u8; 80],
                [WCHAR; 40],
            >(
                *b"{\x006\0D\x004\0A\x003\x006\x005\x000\0-\x006\x002\08\0D\0-\x001\x001\0D\x002\0-\0A\0E\x000\0F\0-\x000\x000\x006\x000\09\x007\0B\x000\x001\x004\x001\x001\0}\0\0\0\0\0",
            ),
            ResourceID: IDS_DIRECTPLAY8_MODEMSP as UINT,
        },
        DIRECTPLAY_GUID {
            Guid: ::core::mem::transmute::<
                [u8; 80],
                [WCHAR; 40],
            >(
                *b"{\x007\x004\x003\0B\x005\0D\x006\x000\0-\x006\x002\08\0D\0-\x001\x001\0D\x002\0-\0A\0E\x000\0F\0-\x000\x000\x006\x000\09\x007\0B\x000\x001\x004\x001\x001\0}\0\0\0\0\0",
            ),
            ResourceID: IDS_DIRECTPLAY8_SERIALSP as UINT,
        },
        DIRECTPLAY_GUID {
            Guid: ::core::mem::transmute::<
                [u8; 80],
                [WCHAR; 40],
            >(
                *b"{\x005\x003\09\x003\x004\x002\09\x000\0-\x006\x002\08\0D\0-\x001\x001\0D\x002\0-\0A\0E\x000\0F\0-\x000\x000\x006\x000\09\x007\0B\x000\x001\x004\x001\x001\0}\0\0\0\0\0",
            ),
            ResourceID: IDS_DIRECTPLAY8_IPXSP as UINT,
        },
        DIRECTPLAY_GUID {
            Guid: ::core::mem::transmute::<
                [u8; 80],
                [WCHAR; 40],
            >(
                *b"{\0E\0B\0F\0E\x007\0B\0A\x000\0-\x006\x002\08\0D\0-\x001\x001\0D\x002\0-\0A\0E\x000\0F\0-\x000\x000\x006\x000\09\x007\0B\x000\x001\x004\x001\x001\0}\0\0\0\0\0",
            ),
            ResourceID: IDS_DIRECTPLAY8_TCPSP as UINT,
        },
    ]
};

static mut DirectPlaySP: [DIRECTPLAY_GUID; 4] = unsafe {
    [
        DIRECTPLAY_GUID {
            Guid: ::core::mem::transmute::<
                [u8; 80],
                [WCHAR; 40],
            >(
                *b"{\x003\x006\0E\09\x005\0E\0E\x000\0-\08\x005\x007\x007\0-\x001\x001\0c\0f\0-\09\x006\x000\0C\0-\x000\x000\08\x000\0C\x007\x005\x003\x004\0E\08\x002\0}\0\0\0\0\0",
            ),
            ResourceID: IDS_DIRECTPLAY_TCPCONN as UINT,
        },
        DIRECTPLAY_GUID {
            Guid: ::core::mem::transmute::<
                [u8; 80],
                [WCHAR; 40],
            >(
                *b"6\08\x005\0B\0C\x004\x000\x000\0-\09\0D\x002\0C\0-\x001\x001\0c\0f\0-\0A\09\0C\0D\0-\x000\x000\0A\0A\x000\x000\x006\08\08\x006\0E\x003\0\0\0\0\0\0\0\0\0",
            ),
            ResourceID: IDS_DIRECTPLAY_IPXCONN as UINT,
        },
        DIRECTPLAY_GUID {
            Guid: ::core::mem::transmute::<
                [u8; 80],
                [WCHAR; 40],
            >(
                *b"{\x004\x004\0E\0A\0A\x007\x006\x000\0-\0C\0B\x006\08\0-\x001\x001\0c\0f\0-\09\0C\x004\0E\0-\x000\x000\0A\x000\0C\09\x000\x005\x004\x002\x005\0E\0}\0\0\0\0\0",
            ),
            ResourceID: IDS_DIRECTPLAY_MODEMCONN as UINT,
        },
        DIRECTPLAY_GUID {
            Guid: ::core::mem::transmute::<
                [u8; 80],
                [WCHAR; 40],
            >(
                *b"{\x000\0F\x001\0D\x006\08\x006\x000\0-\08\08\0D\09\0-\x001\x001\0c\0f\0-\09\0C\x004\0E\0-\x000\x000\0A\x000\0C\09\x000\x005\x004\x002\x005\0E\0}\0\0\0\0\0",
            ),
            ResourceID: IDS_DIRECTPLAY_SERIALCONN as UINT,
        },
    ]
};

#[repr(C)]
pub struct DIRECTPLAY_GUID {
    pub Guid: [WCHAR; 40],
    pub ResourceID: UINT,
}
pub type PDIRECTPLAY_GUID = *mut DIRECTPLAY_GUID;

#[repr(C)]
pub struct _LANGANDCODEPAGE_ {
    pub lang: WORD,
    pub code: WORD,
}
pub type LANGANDCODEPAGE = _LANGANDCODEPAGE_;
pub type LPLANGANDCODEPAGE = *mut _LANGANDCODEPAGE_;

unsafe extern "C" fn InitListViewColumns(hDlgCtrl: HWND) {
    let mut szText: [WCHAR; 256] = [0; 256];
    let mut lvcolumn: LVCOLUMNW = core::mem::zeroed();
    let mut Index: INT;

    ZeroMemory!(
        &raw mut lvcolumn as *mut core::ffi::c_void,
        core::mem::size_of::<LVCOLUMNW>(),
    );
    lvcolumn.pszText = &raw mut szText as *mut WCHAR;
    lvcolumn.mask = (LVCF_FMT | LVCF_TEXT | LVCF_SUBITEM | LVCF_WIDTH) as UINT;
    lvcolumn.fmt = LVCFMT_LEFT as INT;
    lvcolumn.cx = 200 as INT;

    Index = 0 as INT;
    while Index < 4 as INT {
        szText[0 as usize] = 0x0000u16;
        LoadStringW(
            hInst,
            (IDS_DIRECTPLAY_COL_NAME1 + (Index) as u32) as UINT,
            &raw mut szText as *mut WCHAR,
            (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
        );
        szText[((core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>())
            - 1 as usize)] = 0x0000u16;
        if Index != 0 {
            lvcolumn.cx = 98 as INT;
        }
        if SendMessageW(
            hDlgCtrl,
            LVM_INSERTCOLUMNW as UINT,
            Index as WPARAM,
            &raw mut lvcolumn as LPARAM,
        ) == -(1 as LRESULT)
        {
            return;
        }
        Index += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FindProviderIndex(szGuid: LPCWSTR, PreDefProviders: *mut DIRECTPLAY_GUID) -> UINT {
    let mut Index: UINT;
    Index = 0;
    while Index < 4 {
        if wcsncmp(core::ptr::addr_of_mut!((*PreDefProviders.add(Index as usize)).Guid) as *mut WCHAR, szGuid, 40) == 0 {
            return Index;
        }
        Index = Index.wrapping_add(1);
    }
    return UINT_MAX;
}
#[no_mangle]
pub unsafe extern "C" fn GetFileVersion(szAppName: LPCWSTR, szVer: *mut WCHAR, szVerSize: DWORD) -> BOOL {
    let mut VerSize: UINT = 0;
    let mut DummyHandle: DWORD = 0;
    let mut pBuf: LPVOID = core::ptr::null_mut();
    let mut lang: WORD = 0 as WORD;
    let mut code: WORD = 0 as WORD;
    let mut lplangcode: LPLANGANDCODEPAGE = core::ptr::null_mut();
    let mut szBuffer: [WCHAR; 100] = [0; 100];
    let mut pResult: *mut WCHAR = core::ptr::null_mut();
    let mut bResult: BOOL = FALSE as BOOL;
    let mut bVer: BOOL = 0;

    static wFormat: [WCHAR; 37] = [
    0x005Cu16, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0025, 0x0030, 0x0034, 0x0078,
    0x0025, 0x0030, 0x0034, 0x0078, 0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0056,
    0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000,
];
    static wTranslation: [WCHAR; 24] = [
    0x0056u16, 0x0061, 0x0072, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E, 0x0066,
    0x006F, 0x005C, 0x0054, 0x0072, 0x0061, 0x006E, 0x0073, 0x006C, 0x0061, 0x0074,
    0x0069, 0x006F, 0x006E, 0x0000,
];

    /* query version info size */
    VerSize = GetFileVersionInfoSizeW(szAppName, core::ptr::addr_of_mut!(DummyHandle)) as UINT;
    if VerSize == 0 {
        return FALSE as BOOL;
    }

    /* allocate buffer */
    pBuf = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        VerSize as SIZE_T,
    ) as LPVOID;
    if pBuf.is_null() {
        return FALSE as BOOL;
    }

    /* query version info */
    if GetFileVersionInfoW(
        szAppName,
        0 as DWORD,
        VerSize as DWORD,
        pBuf as PVOID,
    ) == 0
    {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pBuf as PVOID,
        );
        return FALSE as BOOL;
    }

    /* query lang code */
    if VerQueryValueW(
        pBuf as LPCVOID,
        core::ptr::addr_of!(wTranslation) as LPCWSTR,
        core::ptr::addr_of_mut!(lplangcode) as *mut LPVOID,
        core::ptr::addr_of_mut!(VerSize),
    ) != 0
    {
        /* FIXME find language from current locale / if not available,
         * default to english
         * for now default to first available language
         */
        lang = (*lplangcode).lang;
        code = (*lplangcode).code;
    }

    /* set up format */
    wsprintfW(
        core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
        core::ptr::addr_of!(wFormat) as LPCWSTR,
        lang as core::ffi::c_int,
        code as core::ffi::c_int,
    );

    /* query manufacturer */
    pResult = core::ptr::null_mut();
    bVer = VerQueryValueW(
        pBuf as LPCVOID,
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(pResult) as *mut LPVOID,
        core::ptr::addr_of_mut!(VerSize),
    );

    if (VerSize as DWORD) < szVerSize && bVer != 0 && !pResult.is_null() {
        wcscpy(szVer, pResult);
        pResult = wcschr(szVer as LPCWSTR, (0x0020u16 as core::ffi::c_int) as u16) as *mut WCHAR;
        if !pResult.is_null() {
            /* cut off build info */
            VerSize = pResult.offset_from(szVer) as core::ffi::c_longlong as UINT;
        }
        if GetLocaleInfoW(
            MAKELCID!(lang as DWORD, SORT_DEFAULT as DWORD),
            LOCALE_SLANGUAGE as LCTYPE,
            szVer.offset(VerSize as isize) as LPWSTR,
            szVerSize.wrapping_sub(VerSize as DWORD) as core::ffi::c_int,
        ) != 0
        {
            *szVer.offset(VerSize.wrapping_sub(1 as UINT) as isize) = 0x0020u16;
            *szVer.offset(szVerSize.wrapping_sub(1 as DWORD) as isize) = 0x0000u16;
        }
        bResult = TRUE as BOOL;
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pBuf as PVOID,
    );
    return bResult;
}

unsafe extern "C" fn EnumerateServiceProviders(hKey: HKEY, hDlgCtrl: HWND, PreDefProviders: *mut DIRECTPLAY_GUID) -> BOOL {
    let mut dwIndex: DWORD = 0 as DWORD;
    let mut result: LONG = 0;
    let mut szName: [WCHAR; 50] = [0; 50];
    let mut szGUID: [WCHAR; 40] = [0; 40];
    let mut szTemp: [WCHAR; 63] = [0; 63];
    let mut szResult: [WCHAR; (MAX_PATH + 20) as _] = [0; (MAX_PATH + 20) as _];
    let mut RegProviders: DWORD = 0 as DWORD;
    let mut ProviderIndex: DWORD = 0;
    let mut dwName: DWORD = 0;
    let mut Item: LVITEMW = core::mem::zeroed();
    let mut ItemCount: INT = 0;
    let mut lResult: LRESULT = 0;

    ItemCount = ListView_GetItemCount!(hDlgCtrl) as INT;
    ZeroMemory!(&raw mut Item as LPVOID, core::mem::size_of::<LVITEMW>() as usize);
    Item.mask = LVIF_TEXT as UINT;
    Item.pszText = &raw mut szResult as *mut WCHAR;
    Item.iItem = ItemCount;
    /* insert all predefined items first */
    dwIndex = 0 as DWORD;
    while dwIndex < 4 as DWORD {
        Item.iItem = (ItemCount as DWORD).wrapping_add(dwIndex) as INT;
        Item.iSubItem = 0 as INT;
        szResult[0 as usize] = 0x0000u16;
        LoadStringW(hInst, (*PreDefProviders.offset(dwIndex as isize)).ResourceID, &raw mut szResult as *mut WCHAR, (core::mem::size_of_val(&szResult) / core::mem::size_of::<WCHAR>()) as INT);
        szResult[((core::mem::size_of_val(&szResult) / core::mem::size_of::<WCHAR>()) - (1 as usize))] = 0x0000u16;
        Item.iItem = SendMessageW(hDlgCtrl, LVM_INSERTITEM as UINT, 0 as WPARAM, &raw mut Item as LPARAM) as INT;
        Item.iSubItem = 1 as INT;
        szResult[0 as usize] = 0x0000u16;
        LoadStringW(hInst, IDS_REG_FAIL as UINT, &raw mut szResult as *mut WCHAR, (core::mem::size_of_val(&szResult) / core::mem::size_of::<WCHAR>()) as INT);
        szResult[((core::mem::size_of_val(&szResult) / core::mem::size_of::<WCHAR>()) - (1 as usize))] = 0x0000u16;
        SendMessageW(hDlgCtrl, LVM_SETITEM as UINT, 0 as WPARAM, &raw mut Item as LPARAM);
        dwIndex += 1;
    }

    dwIndex = 0 as DWORD;
    loop {
        dwName = (core::mem::size_of_val(&szName) / core::mem::size_of::<WCHAR>()) as DWORD;
        result = RegEnumKeyEx(hKey, dwIndex, &raw mut szName as *mut WCHAR, &raw mut dwName, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) as LONG;
        if result == ERROR_SUCCESS as LONG {
            szName[((core::mem::size_of_val(&szName) / core::mem::size_of::<WCHAR>()) - (1 as usize))] = 0x0000u16;

            ProviderIndex = UINT_MAX as DWORD;
            if GetRegValue(hKey, &raw mut szName as *mut WCHAR, RUSTCONV_WSTR_C0A6B47AC6CD.as_ptr().cast_mut(), REG_SZ as DWORD, &raw mut szGUID as *mut WCHAR, core::mem::size_of_val(&szGUID) as DWORD) != 0 {
                ProviderIndex = FindProviderIndex(&raw const szGUID as *const WCHAR, PreDefProviders) as DWORD;
            }

            if ProviderIndex == UINT_MAX as DWORD {
                /* a custom service provider was found */
                Item.iItem = ListView_GetItemCount!(hDlgCtrl) as INT;

                /* FIXME
                 * on Windows Vista we need to use RegLoadMUIString which is not available for older systems
                 */
                if GetRegValue(hKey, &raw mut szName as *mut WCHAR, RUSTCONV_WSTR_BA52125E4714.as_ptr().cast_mut(), REG_SZ as DWORD, &raw mut szResult as *mut WCHAR, core::mem::size_of_val(&szResult) as DWORD) == 0 {
                    if GetRegValue(hKey, &raw mut szName as *mut WCHAR, RUSTCONV_WSTR_332FB550977A.as_ptr().cast_mut(), REG_SZ as DWORD, &raw mut szResult as *mut WCHAR, core::mem::size_of_val(&szResult) as DWORD) == 0 {
                        szResult[0 as usize] = 0x0000u16;
                    }
                }

                /* insert the new provider */
                Item.iSubItem = 0 as INT;
                lResult = SendMessageW(hDlgCtrl, LVM_INSERTITEM as UINT, 0 as WPARAM, &raw mut Item as LPARAM);
                /* adjust index */
                ProviderIndex = (lResult - ItemCount as LRESULT) as DWORD;
            }

            szResult[0 as usize] = 0x0000u16;
            /* check if the 'Path' key is available */
            if GetRegValue(hKey, &raw mut szName as *mut WCHAR, RUSTCONV_WSTR_9A56E6BBE687.as_ptr().cast_mut(), REG_SZ as DWORD, &raw mut szResult as *mut WCHAR, core::mem::size_of_val(&szResult) as DWORD) == 0 {
                /* retrieve the path by lookup the CLSID */
                wcscpy(&raw mut szTemp as *mut WCHAR, RUSTCONV_WSTR_6E342AB1A0F1.as_ptr());
                wcscpy((&raw mut szTemp as *mut WCHAR).offset(6 as isize), &raw mut szGUID as *mut WCHAR);
                wcscpy((&raw mut szTemp as *mut WCHAR).offset(44 as isize), RUSTCONV_WSTR_6C24BFD29528.as_ptr());
                if GetRegValue(HKEY_CLASSES_ROOT as usize as HKEY, &raw mut szTemp as *mut WCHAR, core::ptr::null_mut(), REG_SZ as DWORD, &raw mut szResult as *mut WCHAR, core::mem::size_of_val(&szResult) as DWORD) == 0 {
                    szResult[0 as usize] = 0x0000u16;
                    ProviderIndex = UINT_MAX as DWORD;
                }
            }
            if szResult[0 as usize] != 0 {
                /* insert path name */
                Item.iSubItem = 2 as INT;
                Item.iItem = ProviderIndex.wrapping_add(ItemCount as DWORD) as INT;
                SendMessageW(hDlgCtrl, LVM_SETITEM as UINT, 0 as WPARAM, &raw mut Item as LPARAM);
                /* retrieve file version */
                if GetFileVersion(&raw mut szResult as *mut WCHAR, &raw mut szTemp as *mut WCHAR, (core::mem::size_of_val(&szTemp) / core::mem::size_of::<WCHAR>()) as DWORD) == 0 {
                    szTemp[0 as usize] = 0x0000u16;
                    LoadStringW(hInst, IDS_VERSION_UNKNOWN as UINT, &raw mut szTemp as *mut WCHAR, (core::mem::size_of_val(&szTemp) / core::mem::size_of::<WCHAR>()) as INT);
                    szTemp[((core::mem::size_of_val(&szTemp) / core::mem::size_of::<WCHAR>()) - (1 as usize))] = 0x0000u16;
                }
                Item.iSubItem = 3 as INT;
                Item.pszText = &raw mut szTemp as *mut WCHAR;
                SendMessageW(hDlgCtrl, LVM_SETITEM as UINT, 0 as WPARAM, &raw mut Item as LPARAM);
                Item.pszText = &raw mut szResult as *mut WCHAR;
            }

            if ProviderIndex != UINT_MAX as DWORD {
                RegProviders |= (1 as DWORD).checked_shl(ProviderIndex as u32).unwrap_or(0); // C shifts by >= 32 (UB) when LVM_INSERTITEM fails: no bit
                szResult[0 as usize] = 0x0000u16;
                LoadStringW(hInst, IDS_REG_SUCCESS as UINT, &raw mut szResult as *mut WCHAR, (core::mem::size_of_val(&szResult) / core::mem::size_of::<WCHAR>()) as INT);
                Item.iSubItem = 1 as INT;

                Item.iItem = ProviderIndex.wrapping_add(ItemCount as DWORD) as INT;
                szResult[((core::mem::size_of_val(&szResult) / core::mem::size_of::<WCHAR>()) - (1 as usize))] = 0x0000u16;
                SendMessageW(hDlgCtrl, LVM_SETITEM as UINT, 0 as WPARAM, &raw mut Item as LPARAM);
            }
        }
        dwIndex = dwIndex.wrapping_add(1);
        if !(result != ERROR_NO_MORE_ITEMS as LONG) {
            break;
        }
    }

    /* check if all providers have been registered */
    //    if (RegProviders == 15)
    return TRUE as BOOL;
    return FALSE as BOOL;
}

unsafe extern "C" fn InitializeDirectPlayDialog(hwndDlg: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut result: LONG;
    let mut hDlgCtrl: HWND = core::ptr::null_mut();

    /* open DirectPlay8 key */
    result = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_3DCC8E3D3F50.as_ptr(),
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    if result != ERROR_SUCCESS as LONG {
        return;
    }

    hDlgCtrl = GetDlgItem(hwndDlg, (IDC_LIST_PROVIDER as i32));

    /* Highlights the entire row instead of just the selected item in the first column */
    SendMessage(
        hDlgCtrl,
        LVM_SETEXTENDEDLISTVIEWSTYLE as UINT,
        0 as WPARAM,
        LVS_EX_FULLROWSELECT as LPARAM,
    );

    /* initialize list ctrl */
    InitListViewColumns(hDlgCtrl);

    /* enumerate providers */
    result = EnumerateServiceProviders(
        hKey,
        hDlgCtrl,
        core::ptr::addr_of_mut!(DirectPlay8SP) as *mut DIRECTPLAY_GUID,
    ) as LONG;
    RegCloseKey(hKey);
    if result == 0 {
        return;
    }

    /* open DirectPlay key */
    result = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_033AA4E0A2D7.as_ptr(),
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    if result != ERROR_SUCCESS as LONG {
        return;
    }

    /* enumerate providers */
    EnumerateServiceProviders(
        hKey,
        hDlgCtrl,
        core::ptr::addr_of_mut!(DirectPlaySP) as *mut DIRECTPLAY_GUID,
    );
    RegCloseKey(hKey);
}

#[no_mangle]
pub unsafe extern "system" fn NetworkPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    match message {
        WM_INITDIALOG => {
            SetWindowPos(hDlg, core::ptr::null_mut(), 10, 32, 0, 0, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);
            InitializeDirectPlayDialog(hDlg);
            return (TRUE) as INT_PTR;
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}
