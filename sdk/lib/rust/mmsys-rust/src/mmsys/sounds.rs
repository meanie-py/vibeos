// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2680D837A730: [u16; 23] = [
    0x0041u16, 0x0070, 0x0070, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x0073, 0x005C,
    0x0053, 0x0063, 0x0068, 0x0065, 0x006D, 0x0065, 0x0073, 0x005C, 0x0041, 0x0070,
    0x0070, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_28EE6E1C27D0: [u16; 13] = [
    0x0044u16, 0x0069, 0x0073, 0x0070, 0x0046, 0x0069, 0x006C, 0x0065, 0x004E, 0x0061,
    0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_33BCD0384968: [u16; 8] = [0x005Cu16, 0x006D, 0x0065, 0x0064, 0x0069, 0x0061, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_340EF3D1C2FD: [u16; 9] = [0x002Eu16, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_38FF7EF192D3: [u16; 9] = [0x005Cu16, 0x006D, 0x0065, 0x0064, 0x0069, 0x0061, 0x005C, 0x002A, 0x0000];
pub static RUSTCONV_WSTR_4903533D150E: [u16; 5] = [0x0041u16, 0x0070, 0x0070, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_A9DCCBC70D07: [u16; 15] = [
    0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x002E, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_AA9585F5E66B: [u16; 18] = [
    0x0041u16, 0x0070, 0x0070, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x0073, 0x005C,
    0x0053, 0x0063, 0x0068, 0x0065, 0x006D, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_ABAF39E8ABA9: [u16; 6] = [0x004Eu16, 0x0061, 0x006D, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_B2B592D2A268: [u16; 22] = [
    0x0041u16, 0x0070, 0x0070, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x0073, 0x005C,
    0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x004C, 0x0061, 0x0062, 0x0065, 0x006C,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_EA39AA8DA3EB: [u16; 19] = [
    0x0025u16, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F,
    0x0074, 0x0025, 0x005C, 0x004D, 0x0065, 0x0064, 0x0069, 0x0061, 0x0000,
];
pub static RUSTCONV_WSTR_F8608C2A8ECB: [u16; 10] = [0x0077u16, 0x0061, 0x0076, 0x0065, 0x0061, 0x0075, 0x0064, 0x0069, 0x006F, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LABEL_MAP {
    pub szName: PWSTR,
    pub szDesc: PWSTR,
    pub szIcon: PWSTR,
    pub AppMap: *mut _APP_MAP,
    pub Next: *mut _LABEL_MAP,
}
pub type LABEL_MAP = _LABEL_MAP;
pub type PLABEL_MAP = *mut _LABEL_MAP;

#[repr(C)]
pub struct _APP_MAP {
    pub szName: [WCHAR; (MAX_PATH) as _],
    pub szDesc: [WCHAR; (MAX_PATH) as _],
    pub szIcon: [WCHAR; (MAX_PATH) as _],
    pub Next: *mut _APP_MAP,
    pub LabelMap: PLABEL_MAP,
}
pub type APP_MAP = _APP_MAP;
pub type PAPP_MAP = *mut _APP_MAP;

#[repr(C)]
pub struct _LABEL_CONTEXT {
    pub LabelMap: PLABEL_MAP,
    pub AppMap: PAPP_MAP,
    pub szValue: [WCHAR; (MAX_PATH) as _],
    pub Next: *mut _LABEL_CONTEXT,
}
pub type LABEL_CONTEXT = _LABEL_CONTEXT;
pub type PLABEL_CONTEXT = *mut _LABEL_CONTEXT;

#[repr(C)]
pub struct _SOUND_SCHEME_CONTEXT {
    pub szName: [WCHAR; (MAX_PATH) as _],
    pub szDesc: [WCHAR; (MAX_PATH) as _],
    pub LabelContext: PLABEL_CONTEXT,
}
pub type SOUND_SCHEME_CONTEXT = _SOUND_SCHEME_CONTEXT;
pub type PSOUND_SCHEME_CONTEXT = *mut _SOUND_SCHEME_CONTEXT;

#[repr(C)]
pub struct _GLOBAL_DATA {
    pub szDefault: [WCHAR; (MAX_PATH) as _],
    pub hSoundsImageList: HIMAGELIST,
    pub pLabelMap: PLABEL_MAP,
    pub pAppMap: PAPP_MAP,
    pub NumWavOut: UINT,
    pub hPlayIcon: HICON,
    pub hStopIcon: HICON,
}
pub type GLOBAL_DATA = _GLOBAL_DATA;
pub type PGLOBAL_DATA = *mut _GLOBAL_DATA;

pub const ID_SOUND_TEST_TIMER: u32 = 1;
#[no_mangle]
pub unsafe extern "C" fn MakeFilter(psz: LPWSTR) -> LPWSTR {
    let mut pch: *mut u16 = core::ptr::null_mut();
    pch = psz;
    while *pch != UNICODE_NULL {
        /* replace vertical bar with NULL */
        if *pch == 0x007Cu16 {
            *pch = UNICODE_NULL;
        }
        pch = pch.add(1);
    }
    return psz;
}
#[no_mangle]
pub unsafe extern "C" fn FindLabel(pGlobalData: PGLOBAL_DATA, pAppMap: PAPP_MAP, szName: PCWSTR) -> PLABEL_MAP {
    let mut pMap: PLABEL_MAP = (*pGlobalData).pLabelMap;

    while !pMap.is_null() {
        if wcscmp((*pMap).szName as *const u16, szName as *const u16) == 0 {
            return pMap;
        }

        pMap = (*pMap).Next;
    }

    pMap = (*pAppMap).LabelMap;

    while !pMap.is_null() {
        if wcscmp((*pMap).szName as *const u16, szName as *const u16) == 0 {
            return pMap;
        }

        pMap = (*pMap).Next;
    }

    pMap = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<LABEL_MAP>() as SIZE_T,
    ) as PLABEL_MAP;
    if pMap.is_null() {
        return core::ptr::null_mut();
    }

    (*pMap).szDesc = _wcsdup(szName as *const u16) as PWSTR;
    (*pMap).szName = (*pMap).szDesc;
    if (*pMap).szName.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pMap as PVOID,
        );
        return core::ptr::null_mut();
    }

    (*pMap).AppMap = pAppMap;
    (*pMap).Next = (*pGlobalData).pLabelMap;
    (*pGlobalData).pLabelMap = pMap;

    return pMap;
}
#[no_mangle]
pub unsafe extern "C" fn RemoveLabel(pGlobalData: PGLOBAL_DATA, pMap: PLABEL_MAP) {
    let mut pCurMap: PLABEL_MAP = (*pGlobalData).pLabelMap;

    if pCurMap == pMap {
        (*pGlobalData).pLabelMap = (*(*pGlobalData).pLabelMap).Next;
        return;
    }

    while !pCurMap.is_null() {
        if (*pCurMap).Next == pMap {
            (*pCurMap).Next = (*(*pCurMap).Next).Next;
            return;
        }
        pCurMap = (*pCurMap).Next;
    }
}

unsafe extern "C" fn FreeLabelMap(mut pGlobalData: PGLOBAL_DATA) {
    let mut pCurMap: PLABEL_MAP;
    while !(*pGlobalData).pLabelMap.is_null() {
        pCurMap = (*(*pGlobalData).pLabelMap).Next;

        /* Prevent double freeing (for "FindLabel") */
        if (*(*pGlobalData).pLabelMap).szName != (*(*pGlobalData).pLabelMap).szDesc {
            free((*(*pGlobalData).pLabelMap).szName as *mut core::ffi::c_void);
        }

        free((*(*pGlobalData).pLabelMap).szDesc as *mut core::ffi::c_void);
        free((*(*pGlobalData).pLabelMap).szIcon as *mut core::ffi::c_void);

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*pGlobalData).pLabelMap as PVOID,
        );
        (*pGlobalData).pLabelMap = pCurMap;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FindApp(mut pGlobalData: PGLOBAL_DATA, mut szName: PCWSTR) -> PAPP_MAP {
    let mut pMap: PAPP_MAP = (*pGlobalData).pAppMap;
    while !pMap.is_null() {
        if wcscmp(
            core::ptr::addr_of_mut!((*pMap).szName) as *mut WCHAR,
            szName as *const WCHAR,
        ) != 0
        {
            // not equal: keep searching
        } else {
            return pMap;
        }
        pMap = (*pMap).Next;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn FreeAppMap(mut pGlobalData: PGLOBAL_DATA) {
    let mut pCurMap: PAPP_MAP = core::ptr::null_mut::<_APP_MAP>();
    while !(*pGlobalData).pAppMap.is_null() {
        pCurMap = (*(*pGlobalData).pAppMap).Next as PAPP_MAP;
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*pGlobalData).pAppMap as PVOID,
        );
        (*pGlobalData).pAppMap = pCurMap;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FindLabelContext(
    mut pGlobalData: PGLOBAL_DATA,
    mut pSoundScheme: PSOUND_SCHEME_CONTEXT,
    mut AppName: PCWSTR,
    mut LabelName: PCWSTR,
) -> PLABEL_CONTEXT {
    let mut pLabelContext: PLABEL_CONTEXT = core::ptr::null_mut();
    pLabelContext = (*pSoundScheme).LabelContext;
    while !pLabelContext.is_null() {
        if (*pLabelContext).AppMap.is_null() {
            RtlAssert(
                b"pLabelContext->AppMap\0".as_ptr() as *const i8 as PVOID,
                b"dll/cpl/mmsys/sounds.c\0".as_ptr() as *const i8 as PVOID,
                212 as ULONG,
                b"\0".as_ptr() as *const i8 as PCHAR,
            );
        }
        if (*pLabelContext).LabelMap.is_null() {
            RtlAssert(
                b"pLabelContext->LabelMap\0".as_ptr() as *const i8 as PVOID,
                b"dll/cpl/mmsys/sounds.c\0".as_ptr() as *const i8 as PVOID,
                213 as ULONG,
                b"\0".as_ptr() as *const i8 as PCHAR,
            );
        }
        if _wcsicmp(
            &raw mut (*(*pLabelContext).AppMap).szName as *mut WCHAR,
            AppName as *const wchar_t,
        ) == 0
            && _wcsicmp(
                (*(*pLabelContext).LabelMap).szName as *const wchar_t,
                LabelName as *const wchar_t,
            ) == 0
        {
            return pLabelContext;
        }
        pLabelContext = (*pLabelContext).Next as PLABEL_CONTEXT;
    }
    pLabelContext = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<LABEL_CONTEXT>() as SIZE_T,
    ) as PLABEL_CONTEXT;
    if pLabelContext.is_null() {
        return core::ptr::null_mut();
    }
    (*pLabelContext).AppMap = FindApp(pGlobalData, AppName);
    (*pLabelContext).LabelMap = FindLabel(pGlobalData, (*pLabelContext).AppMap, LabelName);
    if (*pLabelContext).AppMap.is_null() {
        RtlAssert(
            b"pLabelContext->AppMap\0".as_ptr() as *const i8 as PVOID,
            b"dll/cpl/mmsys/sounds.c\0".as_ptr() as *const i8 as PVOID,
            228 as ULONG,
            b"\0".as_ptr() as *const i8 as PCHAR,
        );
    }
    if (*pLabelContext).LabelMap.is_null() {
        RtlAssert(
            b"pLabelContext->LabelMap\0".as_ptr() as *const i8 as PVOID,
            b"dll/cpl/mmsys/sounds.c\0".as_ptr() as *const i8 as PVOID,
            229 as ULONG,
            b"\0".as_ptr() as *const i8 as PCHAR,
        );
    }
    (*pLabelContext).szValue[0 as usize] = UNICODE_NULL;
    (*pLabelContext).Next = (*pSoundScheme).LabelContext;
    (*pSoundScheme).LabelContext = pLabelContext;
    return pLabelContext;
}
#[no_mangle]
pub unsafe extern "C" fn LoadEventLabel(pGlobalData: PGLOBAL_DATA, hKey: HKEY, szSubKey: PCWSTR) -> BOOL {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut cbValue: DWORD = 0;
    let mut szDesc: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szData: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pMap: PLABEL_MAP = core::ptr::null_mut();

    if (RegOpenKeyExW(
        hKey,
        szSubKey,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hSubKey),
    )) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL;
    }

    cbValue = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    if (RegQueryValueExW(
        hSubKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szDesc) as LPBYTE,
        core::ptr::addr_of_mut!(cbValue),
    )) as u32 != ERROR_SUCCESS
    {
        RegCloseKey(hSubKey);
        return (FALSE) as BOOL;
    }

    cbValue = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    if (RegQueryValueExW(
        hSubKey,
        RUSTCONV_WSTR_28EE6E1C27D0.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szData) as LPBYTE,
        core::ptr::addr_of_mut!(cbValue),
    )) as u32 != ERROR_SUCCESS
    {
        RegCloseKey(hSubKey);
        return (FALSE) as BOOL;
    }

    pMap = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<LABEL_MAP>() as SIZE_T,
    ) as PLABEL_MAP;
    if pMap.is_null() {
        return (FALSE) as BOOL;
    }

    (*pMap).szName = _wcsdup(szSubKey as *const WCHAR) as PWSTR;
    (*pMap).szDesc = _wcsdup(core::ptr::addr_of_mut!(szDesc) as *mut WCHAR) as PWSTR;
    (*pMap).szIcon = _wcsdup(core::ptr::addr_of_mut!(szData) as *mut WCHAR) as PWSTR;

    if !(*pGlobalData).pLabelMap.is_null() {
        (*pMap).Next = (*pGlobalData).pLabelMap as *mut _LABEL_MAP;
        (*pGlobalData).pLabelMap = pMap;
    } else {
        (*pGlobalData).pLabelMap = pMap;
        (*(*pGlobalData).pLabelMap).Next = core::ptr::null_mut();
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LoadEventLabels(pGlobalData: PGLOBAL_DATA) -> BOOL {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut dwCurKey: DWORD = 0;
    let mut szName: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut dwName: DWORD = 0;
    let mut dwResult: DWORD = 0;
    let mut dwCount: DWORD = 0;
    if (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_B2B592D2A268.as_ptr(),
        0,
        KEY_READ,
        &raw mut hSubKey,
    )) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL;
    }

    dwCurKey = 0;
    dwCount = 0;
    loop {
        dwName = _countof!(szName);
        dwResult = (RegEnumKeyExW(
            hSubKey,
            dwCurKey,
            &raw mut szName as LPWSTR,
            &raw mut dwName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;

        if dwResult == ERROR_SUCCESS {
            if LoadEventLabel(pGlobalData, hSubKey, &raw mut szName as *mut WCHAR as PCWSTR) != 0 {
                dwCount = dwCount.wrapping_add(1);
            }
        }
        dwCurKey = dwCurKey.wrapping_add(1);

        if !(dwResult == ERROR_SUCCESS) {
            break;
        }
    }
    RegCloseKey(hSubKey);
    return (dwCount != 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn AddSoundProfile(hwndDlg: HWND, hKey: HKEY, szSubKey: PCWSTR, SetDefault: BOOL) -> BOOL {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut szValue: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut cbValue: DWORD = 0;
    let mut dwResult: DWORD = 0;
    let mut lResult: LRESULT = 0;
    let mut pScheme: PSOUND_SCHEME_CONTEXT = core::ptr::null_mut();

    if RegOpenKeyExW(
        hKey,
        szSubKey,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hSubKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return (FALSE) as BOOL;
    }

    cbValue = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    dwResult = RegQueryValueExW(
        hSubKey,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szValue) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbValue),
    ) as DWORD;
    RegCloseKey(hSubKey);

    if dwResult != ERROR_SUCCESS as DWORD {
        return (FALSE) as BOOL;
    }

    /* Try to add the new profile */
    lResult = ComboBox_AddString!(
        GetDlgItem(hwndDlg, IDC_SOUND_SCHEME as ::core::ffi::c_int),
        core::ptr::addr_of_mut!(szValue) as *mut WCHAR as LPARAM,
    ) as LRESULT;
    if lResult == CB_ERR as LRESULT {
        return (FALSE) as BOOL;
    }

    /* Allocate the profile scheme buffer */
    pScheme = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<SOUND_SCHEME_CONTEXT>() as SIZE_T,
    ) as PSOUND_SCHEME_CONTEXT;
    if pScheme.is_null() {
        /* We failed to allocate the buffer, no need to keep a dangling string in the combobox */
        ComboBox_DeleteString!(
            GetDlgItem(hwndDlg, IDC_SOUND_SCHEME as ::core::ffi::c_int),
            lResult as WPARAM,
        );
        return (FALSE) as BOOL;
    }

    StringCchCopyW(
        core::ptr::addr_of_mut!((*pScheme).szDesc) as *mut WCHAR,
        (_countof!((*pScheme).szDesc)) as _,
        core::ptr::addr_of_mut!(szValue) as *mut WCHAR,
    );
    StringCchCopyW(
        core::ptr::addr_of_mut!((*pScheme).szName) as *mut WCHAR,
        (_countof!((*pScheme).szName)) as _,
        szSubKey,
    );

    /* Associate the value with the item in the combobox */
    ComboBox_SetItemData!(
        GetDlgItem(hwndDlg, IDC_SOUND_SCHEME as ::core::ffi::c_int),
        lResult as WPARAM,
        pScheme as LPARAM,
    );

    /* Optionally, select the profile */
    if SetDefault != 0 {
        ComboBox_SetCurSel!(
            GetDlgItem(hwndDlg, IDC_SOUND_SCHEME as ::core::ffi::c_int),
            lResult as WPARAM,
        );
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EnumerateSoundProfiles(pGlobalData: PGLOBAL_DATA, hwndDlg: HWND, hKey: HKEY) -> DWORD {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut dwName: DWORD = 0;
    let mut dwCurKey: DWORD = 0;
    let mut dwResult: DWORD = 0;
    let mut dwNumSchemes: DWORD = 0;
    let mut cbDefault: DWORD = 0;
    let mut szName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    cbDefault = core::mem::size_of_val(&(*pGlobalData).szDefault) as DWORD;
    if RegQueryValueExW(
        hKey,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        addr_of_mut!((*pGlobalData).szDefault) as *mut WCHAR as LPBYTE,
        addr_of_mut!(cbDefault),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return FALSE as DWORD;
    }

    if RegOpenKeyExW(
        hKey,
        RUSTCONV_WSTR_ABAF39E8ABA9.as_ptr(),
        0 as DWORD,
        KEY_READ as REGSAM,
        addr_of_mut!(hSubKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return FALSE as DWORD;
    }

    dwNumSchemes = 0 as DWORD;
    dwCurKey = 0 as DWORD;
    loop {
        dwName = _countof!(szName) as DWORD;
        dwResult = RegEnumKeyExW(
            hSubKey,
            dwCurKey,
            addr_of_mut!(szName) as *mut WCHAR as LPWSTR,
            addr_of_mut!(dwName),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;

        if dwResult == ERROR_SUCCESS as DWORD {
            if AddSoundProfile(
                hwndDlg,
                hSubKey,
                addr_of_mut!(szName) as *mut WCHAR as PCWSTR,
                (_wcsicmp(
                    addr_of_mut!(szName) as *mut WCHAR,
                    addr_of_mut!((*pGlobalData).szDefault) as *mut WCHAR,
                ) == 0) as ::core::ffi::c_int,
            ) != 0
            {
                dwNumSchemes = dwNumSchemes.wrapping_add(1);
            }
        }

        dwCurKey = dwCurKey.wrapping_add(1);
        if !(dwResult == ERROR_SUCCESS as DWORD) {
            break;
        }
    }

    RegCloseKey(hSubKey);
    return dwNumSchemes;
}
#[no_mangle]
pub unsafe extern "C" fn FindSoundProfile(hwndDlg: HWND, szName: PCWSTR) -> PSOUND_SCHEME_CONTEXT {
    let mut lCount: c_int;
    let mut lIndex: c_int;
    let mut lResult: LRESULT;
    let mut pScheme: PSOUND_SCHEME_CONTEXT;
    let mut hwndComboBox: HWND;

    hwndComboBox = GetDlgItem(hwndDlg, (IDC_SOUND_SCHEME as i32));
    lCount = ComboBox_GetCount!(hwndComboBox);
    if lCount == (CB_ERR) as _ {
        return core::ptr::null_mut();
    }

    lIndex = 0;
    while lIndex < lCount {
        lResult = ComboBox_GetItemData!(hwndComboBox, lIndex);
        if lResult != (CB_ERR) as _ {
            pScheme = lResult as PSOUND_SCHEME_CONTEXT;
            if _wcsicmp(core::ptr::addr_of_mut!((*pScheme).szName) as *mut WCHAR, szName) == 0 {
                return pScheme;
            }
        }
        lIndex += 1;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn FreeSoundProfiles(hwndDlg: HWND) {
    let mut lCount: LRESULT = 0;
    let mut lIndex: LRESULT = 0;
    let mut lResult: LRESULT = 0;
    let mut pScheme: PSOUND_SCHEME_CONTEXT = core::ptr::null_mut();
    let mut pLabelContext: PLABEL_CONTEXT = core::ptr::null_mut();
    let mut hwndComboBox: HWND = core::ptr::null_mut();

    hwndComboBox = GetDlgItem(hwndDlg, IDC_SOUND_SCHEME as ::core::ffi::c_int);
    lCount = ComboBox_GetCount!(hwndComboBox) as LRESULT;
    if lCount == CB_ERR as LRESULT {
        return;
    }

    lIndex = 0 as LRESULT;
    while lIndex < lCount {
        'body: {
            lResult = ComboBox_GetItemData!(hwndComboBox, lIndex);
            if lResult == CB_ERR as LRESULT {
                break 'body;
            }

            pScheme = lResult as PSOUND_SCHEME_CONTEXT;

            while !(*pScheme).LabelContext.is_null() {
                pLabelContext = (*(*pScheme).LabelContext).Next as PLABEL_CONTEXT;
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    (*pScheme).LabelContext as PVOID,
                );
                (*pScheme).LabelContext = pLabelContext;
            }

            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pScheme as PVOID,
            );
        }
        lIndex += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ImportSoundLabel(pGlobalData: PGLOBAL_DATA, hwndDlg: HWND, hKey: HKEY, szProfile: PCWSTR, szLabelName: PCWSTR, szAppName: PCWSTR, AppMap: PAPP_MAP, LabelMap: PLABEL_MAP) -> BOOL {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut szValue: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut cbValue: DWORD;
    let mut cchLength: DWORD;
    let mut pScheme: PSOUND_SCHEME_CONTEXT;
    let mut pLabelContext: PLABEL_CONTEXT;
    let mut bCurrentProfile: BOOL;
    let mut bActiveProfile: BOOL;

    bCurrentProfile = (_wcsicmp(szProfile, RUSTCONV_WSTR_340EF3D1C2FD.as_ptr()) == 0) as BOOL;
    bActiveProfile = (_wcsicmp(szProfile, core::ptr::addr_of_mut!((*pGlobalData).szDefault) as *mut WCHAR) == 0) as BOOL;

    if (RegOpenKeyExW(hKey,
                      szProfile as LPCWSTR,
                      0,
                      KEY_READ,
                      core::ptr::addr_of_mut!(hSubKey))) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL;
    }

    cbValue = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    if (RegQueryValueExW(hSubKey,
                         core::ptr::null(),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         core::ptr::addr_of_mut!(szValue) as *mut WCHAR as LPBYTE,
                         core::ptr::addr_of_mut!(cbValue))) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL;
    }

    if bCurrentProfile != 0 {
        pScheme = FindSoundProfile(hwndDlg, core::ptr::addr_of_mut!((*pGlobalData).szDefault) as *mut WCHAR as PCWSTR);
    } else {
        pScheme = FindSoundProfile(hwndDlg, szProfile);
    }

    if pScheme.is_null() {
        return (FALSE) as BOOL;
    }
    pLabelContext = FindLabelContext(pGlobalData, pScheme, core::ptr::addr_of_mut!((*AppMap).szName) as *mut WCHAR as PCWSTR, (*LabelMap).szName as PCWSTR);

    cchLength = ExpandEnvironmentStringsW(core::ptr::addr_of_mut!(szValue) as *mut WCHAR as LPCWSTR,
                                       core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPWSTR,
                                       (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as DWORD);
    if cchLength == 0
        || cchLength as usize > (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>())
    {
        return (FALSE) as BOOL;
    }

    if bCurrentProfile != 0 {
        StringCchCopyW(core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR as STRSAFE_LPWSTR,
                               (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as size_t,
                               core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as STRSAFE_LPCWSTR);
    } else if bActiveProfile == 0 {
        StringCchCopyW(core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR as STRSAFE_LPWSTR,
                               (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as size_t,
                               core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as STRSAFE_LPCWSTR);
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ImportSoundEntry(
    mut pGlobalData: PGLOBAL_DATA,
    mut hwndDlg: HWND,
    mut hKey: HKEY,
    mut szLabelName: PCWSTR,
    mut szAppName: PCWSTR,
    mut pAppMap: PAPP_MAP,
) -> DWORD {
    let mut hSubKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut dwNumProfiles: DWORD = 0;
    let mut dwCurKey: DWORD = 0;
    let mut dwResult: DWORD = 0;
    let mut dwProfile: DWORD = 0;
    let mut szProfile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pLabel: PLABEL_MAP = ::core::ptr::null_mut::<_LABEL_MAP>();
    if RegOpenKeyExW(
        hKey,
        szLabelName as LPCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        &raw mut hSubKey,
    ) != ERROR_SUCCESS as LSTATUS
    {
        return FALSE as DWORD;
    }
    pLabel = FindLabel(pGlobalData, pAppMap, szLabelName);
    if pLabel.is_null() {
        RtlAssert(
            b"pLabel\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"dll/cpl/mmsys/sounds.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            617 as ULONG,
            b"\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        );
    }
    RemoveLabel(pGlobalData, pLabel);
    (*pLabel).AppMap = pAppMap as *mut _APP_MAP;
    (*pLabel).Next = (*pAppMap).LabelMap as *mut _LABEL_MAP;
    (*pAppMap).LabelMap = pLabel;
    dwNumProfiles = 0 as DWORD;
    dwCurKey = 0 as DWORD;
    loop {
        dwProfile = _countof!(szProfile) as DWORD;
        dwResult = RegEnumKeyExW(
            hSubKey,
            dwCurKey,
            &raw mut szProfile as LPWSTR,
            &raw mut dwProfile,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<WCHAR>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<_FILETIME>(),
        ) as DWORD;
        if dwResult == ERROR_SUCCESS as DWORD {
            if ImportSoundLabel(
                pGlobalData,
                hwndDlg,
                hSubKey,
                &raw mut szProfile as *mut WCHAR as PCWSTR,
                szLabelName,
                szAppName,
                pAppMap,
                pLabel,
            ) != 0
            {
                dwNumProfiles = dwNumProfiles.wrapping_add(1);
            }
        }
        dwCurKey = dwCurKey.wrapping_add(1);
        if !(dwResult == ERROR_SUCCESS as DWORD) {
            break;
        }
    }
    RegCloseKey(hSubKey);
    return dwNumProfiles;
}
#[no_mangle]
pub unsafe extern "C" fn ImportAppProfile(
    pGlobalData: PGLOBAL_DATA,
    hwndDlg: HWND,
    hKey: HKEY,
    szAppName: PCWSTR,
) -> DWORD {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut szDefault: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut cbValue: DWORD = 0;
    let mut dwCurKey: DWORD = 0;
    let mut dwResult: DWORD = 0;
    let mut dwNumEntry: DWORD = 0;
    let mut dwName: DWORD = 0;
    let mut szName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szIcon: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut AppMap: PAPP_MAP = core::ptr::null_mut();

    AppMap = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        ::core::mem::size_of::<APP_MAP>() as SIZE_T,
    ) as PAPP_MAP;
    if AppMap.is_null() {
        return 0 as DWORD;
    }

    if RegOpenKeyExW(
        hKey,
        szAppName as LPCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hSubKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        HeapFree(GetProcessHeap(), 0 as DWORD, AppMap as PVOID);
        return 0 as DWORD;
    }

    cbValue = ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    if RegQueryValueExW(
        hSubKey,
        core::ptr::null::<WCHAR>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(szDefault) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbValue),
    ) != ERROR_SUCCESS as LSTATUS
    {
        RegCloseKey(hSubKey);
        HeapFree(GetProcessHeap(), 0 as DWORD, AppMap as PVOID);
        return 0 as DWORD;
    }

    cbValue = ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
    if RegQueryValueExW(
        hSubKey,
        RUSTCONV_WSTR_28EE6E1C27D0.as_ptr(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(szIcon) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbValue),
    ) != ERROR_SUCCESS as LSTATUS
    {
        szIcon[0 as usize] = UNICODE_NULL as WCHAR;
    }

    /* initialize app map */
    StringCchCopyW(
        core::ptr::addr_of_mut!((*AppMap).szName) as *mut WCHAR,
        _countof!((*AppMap).szName) as size_t,
        szAppName,
    );
    StringCchCopyW(
        core::ptr::addr_of_mut!((*AppMap).szDesc) as *mut WCHAR,
        _countof!((*AppMap).szDesc) as size_t,
        core::ptr::addr_of_mut!(szDefault) as *mut WCHAR,
    );
    StringCchCopyW(
        core::ptr::addr_of_mut!((*AppMap).szIcon) as *mut WCHAR,
        _countof!((*AppMap).szIcon) as size_t,
        core::ptr::addr_of_mut!(szIcon) as *mut WCHAR,
    );

    (*AppMap).Next = (*pGlobalData).pAppMap;
    (*pGlobalData).pAppMap = AppMap;

    dwCurKey = 0 as DWORD;
    dwNumEntry = 0 as DWORD;
    loop {
        dwName = _countof!(szName) as DWORD;
        dwResult = RegEnumKeyExW(
            hSubKey,
            dwCurKey,
            core::ptr::addr_of_mut!(szName) as *mut WCHAR,
            core::ptr::addr_of_mut!(dwName),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<_FILETIME>(),
        ) as DWORD;
        if dwResult == ERROR_SUCCESS as DWORD {
            if ImportSoundEntry(
                pGlobalData,
                hwndDlg,
                hSubKey,
                core::ptr::addr_of_mut!(szName) as *mut WCHAR as PCWSTR,
                szAppName,
                AppMap,
            ) != 0
            {
                dwNumEntry = dwNumEntry.wrapping_add(1);
            }
        }
        dwCurKey = dwCurKey.wrapping_add(1);
        if !(dwResult == ERROR_SUCCESS as DWORD) {
            break;
        }
    }

    RegCloseKey(hSubKey);
    return dwNumEntry;
}
#[no_mangle]
pub unsafe extern "C" fn ImportSoundProfiles(pGlobalData: PGLOBAL_DATA, hwndDlg: HWND, hKey: HKEY) -> BOOL {
    let mut dwCurKey: DWORD;
    let mut dwResult: DWORD;
    let mut dwNumApps: DWORD;
    let mut szName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hSubKey: HKEY = core::mem::zeroed();

    if RegOpenKeyExW(
        hKey,
        RUSTCONV_WSTR_4903533D150E.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hSubKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return (FALSE) as BOOL;
    }

    dwNumApps = 0;
    dwCurKey = 0;
    loop {
        dwResult = RegEnumKeyW(
            hSubKey,
            dwCurKey,
            szName.as_mut_ptr(),
            _countof!(szName),
        ) as DWORD;

        if dwResult == ERROR_SUCCESS as DWORD {
            if ImportAppProfile(
                pGlobalData,
                hwndDlg,
                hSubKey,
                szName.as_mut_ptr() as PCWSTR,
            ) != 0
            {
                dwNumApps = dwNumApps.wrapping_add(1);
            }
        }
        dwCurKey = dwCurKey.wrapping_add(1);
        if !(dwResult == ERROR_SUCCESS as DWORD) {
            break;
        }
    }
    RegCloseKey(hSubKey);

    return (dwNumApps != 0 as DWORD) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LoadSoundProfiles(pGlobalData: PGLOBAL_DATA, hwndDlg: HWND) -> BOOL {
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut dwNumSchemes: DWORD;

    if (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_AA9585F5E66B.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hSubKey),
    )) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL;
    }

    dwNumSchemes = EnumerateSoundProfiles(pGlobalData, hwndDlg, hSubKey);

    if dwNumSchemes != 0 {
        ImportSoundProfiles(pGlobalData, hwndDlg, hSubKey);
    }

    RegCloseKey(hSubKey);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LoadSoundFiles(hwndDlg: HWND) -> BOOL {
    let mut szList: [WCHAR; 256] = [0; 256];
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ptr: PWSTR = core::ptr::null_mut();
    let mut FileData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut lResult: LRESULT = 0;
    let mut length: UINT = 0;

    /* Add no sound listview item */
    if LoadStringW(
        hApplet,
        IDS_NO_SOUND as UINT,
        szList.as_mut_ptr(),
        _countof!(szList) as ::core::ffi::c_int,
    ) != 0
    {
        szList[(_countof!(szList) - 1) as usize] = UNICODE_NULL as WCHAR;
        ComboBox_AddString!(
            GetDlgItem(hwndDlg, IDC_SOUND_LIST as ::core::ffi::c_int),
            szList.as_mut_ptr(),
        );
    }

    /* Load sound files */
    length = GetWindowsDirectoryW(szPath.as_mut_ptr(), _countof!(szPath) as UINT);
    if length == 0 as UINT
        || length as usize >= (_countof!(szPath) - CONST_STR_LEN!(RUSTCONV_WSTR_38FF7EF192D3)) as usize
    {
        return FALSE as BOOL;
    }

    //PathCchAppend(szPath, _countof(szPath), L"media\\*");
    StringCchCatW(szPath.as_mut_ptr(), _countof!(szPath) as usize, RUSTCONV_WSTR_38FF7EF192D3.as_ptr());

    hFile = FindFirstFileW(szPath.as_mut_ptr(), core::ptr::addr_of_mut!(FileData));
    if hFile == INVALID_HANDLE_VALUE as HANDLE {
        return FALSE as BOOL;
    }

    loop {
        'body: {
            if (FileData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY as DWORD) != 0 {
                break 'body;
            }

            ptr = wcsrchr(
                core::ptr::addr_of_mut!(FileData.cFileName) as *mut WCHAR,
                0x005Cu16,
            ) as PWSTR;
            if !ptr.is_null() {
                ptr = ptr.offset(1);
            } else {
                ptr = core::ptr::addr_of_mut!(FileData.cFileName) as *mut WCHAR as PWSTR;
            }
            lResult = SendDlgItemMessageW(
                hwndDlg,
                IDC_SOUND_LIST as ::core::ffi::c_int,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                ptr as LPARAM,
            );
            if lResult != CB_ERR as LRESULT {
                StringCchCopyW(
                    szPath.as_mut_ptr().offset((length + CONST_STR_LEN!(RUSTCONV_WSTR_33BCD0384968)) as isize),
                    (_countof!(szPath) - (length + CONST_STR_LEN!(RUSTCONV_WSTR_33BCD0384968))) as usize,
                    core::ptr::addr_of_mut!(FileData.cFileName) as *mut WCHAR,
                );
                SendDlgItemMessageW(
                    hwndDlg,
                    IDC_SOUND_LIST as ::core::ffi::c_int,
                    CB_SETITEMDATA as UINT,
                    lResult as WPARAM,
                    _wcsdup(szPath.as_mut_ptr()) as LPARAM,
                );
            }
        }
        if !(FindNextFileW(hFile, core::ptr::addr_of_mut!(FileData)) != 0 as BOOL) {
            break;
        }
    }

    FindClose(hFile);
    return TRUE as BOOL;
}

unsafe extern "C" fn FreeSoundFiles(hwndDlg: HWND) {
    let mut lCount: c_int;
    let mut lIndex: c_int;
    let mut lResult: LRESULT;
    let mut pSoundPath: PWSTR;
    let mut hwndComboBox: HWND;

    hwndComboBox = GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32));
    lCount = ComboBox_GetCount!(hwndComboBox);
    if lCount == (CB_ERR) as _ {
        return;
    }

    lIndex = 0;
    while lIndex < lCount {
        'body: {
            lResult = ComboBox_GetItemData!(hwndComboBox, lIndex);
            if lResult == (CB_ERR) as _ {
                break 'body;
            }

            pSoundPath = lResult as PWSTR;
            free(pSoundPath as *mut ::core::ffi::c_void);
        }
        lIndex += 1;
    }
}

unsafe extern "C" fn FindSoundFileInList(mut hwndDlg: HWND, mut pSoundPath: LPCWSTR) -> LRESULT {
    let mut lCount: LRESULT = 0;
    let mut lIndex: LRESULT = 0;
    let mut lResult: LRESULT = 0;
    let mut pszPath: LPWSTR = core::ptr::null_mut();
    let mut hwndComboBox: HWND = core::ptr::null_mut();

    hwndComboBox = GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32));
    lCount = ComboBox_GetCount!(hwndComboBox) as LRESULT;
    if lCount == (CB_ERR) as _ {
        return (CB_ERR) as LRESULT;
    }

    lIndex = 0;
    while lIndex < lCount {
        'body: {
            lResult = ComboBox_GetItemData!(hwndComboBox, lIndex);
            if lResult == (CB_ERR) as _ || lResult == 0 {
                break 'body;
            }

            pszPath = lResult as LPWSTR;
            if _wcsicmp(pszPath, pSoundPath) == 0 {
                return lIndex;
            }
        }
        lIndex += 1;
    }

    return (CB_ERR) as LRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn ShowSoundScheme(pGlobalData: PGLOBAL_DATA, hwndDlg: HWND) -> BOOL {
    let mut lIndex: LRESULT = 0;
    let mut pScheme: PSOUND_SCHEME_CONTEXT = core::ptr::null_mut();
    let mut pAppMap: PAPP_MAP = core::ptr::null_mut();
    let mut pLabelMap: PLABEL_MAP = core::ptr::null_mut();
    let mut pLabelContext: PLABEL_CONTEXT = core::ptr::null_mut();
    let mut hDlgCtrl: HWND = core::ptr::null_mut();
    let mut hList: HWND = core::ptr::null_mut();
    let mut tvItem: TVINSERTSTRUCTW = core::mem::zeroed();
    let mut hTreeItem: HTREEITEM = core::ptr::null_mut();

    hDlgCtrl = GetDlgItem(hwndDlg, (IDC_SOUND_SCHEME as i32));
    hList = GetDlgItem(hwndDlg, (IDC_SCHEME_LIST as i32));

    if !(*pGlobalData).hSoundsImageList.is_null() {
        TreeView_SetImageList!(hList, (*pGlobalData).hSoundsImageList, TVSIL_NORMAL);
    }

    lIndex = SendMessageW(hDlgCtrl, CB_GETCURSEL, 0, 0);
    if lIndex == (CB_ERR) as _ {
        return (FALSE) as BOOL;
    }

    lIndex = SendMessageW(hDlgCtrl, CB_GETITEMDATA, lIndex as WPARAM, 0);
    if lIndex == (CB_ERR) as _ {
        return (FALSE) as BOOL;
    }
    pScheme = lIndex as PSOUND_SCHEME_CONTEXT;

    StringCchCopyW(
        core::ptr::addr_of_mut!((*pGlobalData).szDefault) as *mut WCHAR as PWSTR,
        _countof!(*core::ptr::addr_of_mut!((*pGlobalData).szDefault)) as usize,
        core::ptr::addr_of!((*pScheme).szName) as *const WCHAR as PCWSTR,
    );

    pAppMap = (*pGlobalData).pAppMap;
    while !pAppMap.is_null() {
        ZeroMemory!(
            core::ptr::addr_of_mut!(tvItem) as *mut core::ffi::c_void,
            core::mem::size_of::<TVINSERTSTRUCTW>(),
        );
        tvItem.hParent = TVI_ROOT;
        tvItem.hInsertAfter = TVI_FIRST;

        tvItem.__bindgen_anon_1.item.mask = TVIF_STATE | TVIF_TEXT | TVIF_PARAM | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
        tvItem.__bindgen_anon_1.item.state = TVIS_EXPANDED;
        tvItem.__bindgen_anon_1.item.stateMask = TVIS_EXPANDED;
        tvItem.__bindgen_anon_1.item.pszText = core::ptr::addr_of_mut!((*pAppMap).szDesc) as *mut WCHAR as LPWSTR;
        tvItem.__bindgen_anon_1.item.iImage = (IMAGE_SOUND_SECTION as i32);
        tvItem.__bindgen_anon_1.item.iSelectedImage = (IMAGE_SOUND_SECTION as i32);
        tvItem.__bindgen_anon_1.item.lParam = core::ptr::null_mut::<core::ffi::c_void>() as LPARAM;

        hTreeItem = TreeView_InsertItem!(hList, core::ptr::addr_of_mut!(tvItem));

        pLabelMap = (*pAppMap).LabelMap;
        while !pLabelMap.is_null() {
            pLabelContext = FindLabelContext(
                pGlobalData,
                pScheme,
                core::ptr::addr_of!((*pAppMap).szName) as *const WCHAR as PCWSTR,
                ((*pLabelMap).szName) as *const WCHAR as PCWSTR,
            );

            ZeroMemory!(
                core::ptr::addr_of_mut!(tvItem) as *mut core::ffi::c_void,
                core::mem::size_of::<TVINSERTSTRUCTW>(),
            );
            tvItem.hParent = hTreeItem;
            tvItem.hInsertAfter = TVI_SORT;

            tvItem.__bindgen_anon_1.item.mask = TVIF_STATE | TVIF_TEXT | TVIF_PARAM | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
            tvItem.__bindgen_anon_1.item.state = TVIS_EXPANDED;
            tvItem.__bindgen_anon_1.item.stateMask = TVIS_EXPANDED;
            tvItem.__bindgen_anon_1.item.pszText = ((*pLabelMap).szDesc) as *mut WCHAR as LPWSTR;
            if !pLabelContext.is_null()
                && wcslen(core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR as PCWSTR) > 0 as usize
            {
                tvItem.__bindgen_anon_1.item.iImage = (IMAGE_SOUND_ASSIGNED as i32);
                tvItem.__bindgen_anon_1.item.iSelectedImage = (IMAGE_SOUND_ASSIGNED as i32);
            } else {
                tvItem.__bindgen_anon_1.item.iImage = (IMAGE_SOUND_NONE as i32);
                tvItem.__bindgen_anon_1.item.iSelectedImage = (IMAGE_SOUND_NONE as i32);
            }
            tvItem.__bindgen_anon_1.item.lParam = pLabelContext as LPARAM;

            TreeView_InsertItem!(hList, core::ptr::addr_of_mut!(tvItem));

            pLabelMap = (*pLabelMap).Next;
        }
        pAppMap = (*pAppMap).Next;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ApplyChanges(hwndDlg: HWND) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut lIndex: LRESULT = 0;
    let mut pScheme: PSOUND_SCHEME_CONTEXT = core::ptr::null_mut();
    let mut hDlgCtrl: HWND = core::ptr::null_mut();
    let mut pLabelContext: PLABEL_CONTEXT = core::ptr::null_mut();
    let mut Buffer: [WCHAR; 100] = [0; 100];

    hDlgCtrl = GetDlgItem(hwndDlg, (IDC_SOUND_SCHEME as i32));

    lIndex = SendMessageW(hDlgCtrl, CB_GETCURSEL, 0, 0);
    if lIndex == (CB_ERR) as _ {
        return (FALSE) as BOOL;
    }

    lIndex = SendMessageW(hDlgCtrl, CB_GETITEMDATA, lIndex as WPARAM, 0);
    if lIndex == (CB_ERR) as _ {
        return (FALSE) as BOOL;
    }
    pScheme = lIndex as PSOUND_SCHEME_CONTEXT;

    if (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_AA9585F5E66B.as_ptr(),
        0,
        KEY_WRITE,
        &mut hKey,
    )) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL;
    }

    RegSetValueExW(
        hKey,
        core::ptr::null(),
        0,
        REG_SZ,
        core::ptr::addr_of_mut!((*pScheme).szName) as *mut WCHAR as LPBYTE,
        (((wcslen((*pScheme).szName.as_mut_ptr()) as usize) + 1)
            * core::mem::size_of::<WCHAR>()) as DWORD,
    );
    RegCloseKey(hKey);

    if (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_2680D837A730.as_ptr(),
        0,
        KEY_WRITE,
        &mut hKey,
    )) as u32 != ERROR_SUCCESS
    {
        return (FALSE) as BOOL;
    }

    pLabelContext = (*pScheme).LabelContext;

    while !pLabelContext.is_null() {
        StringCchPrintfW!(
            Buffer.as_mut_ptr(),
            _countof!(Buffer),
            RUSTCONV_WSTR_A9DCCBC70D07.as_ptr(),
            (*(*pLabelContext).AppMap).szName.as_mut_ptr(),
            (*(*pLabelContext).LabelMap).szName,
        );

        if (RegOpenKeyExW(hKey, Buffer.as_ptr(), 0, KEY_WRITE, &mut hSubKey)) as u32 == ERROR_SUCCESS
        {
            dwType = (if !wcschr((*pLabelContext).szValue.as_mut_ptr(), 0x0025u16).is_null() {
                REG_EXPAND_SZ
            } else {
                REG_SZ
            }) as DWORD;
            RegSetValueExW(
                hSubKey,
                core::ptr::null(),
                0,
                dwType,
                core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR as LPBYTE,
                (((wcslen((*pLabelContext).szValue.as_mut_ptr()) as usize) + 1)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );
            RegCloseKey(hSubKey);
        }

        pLabelContext = (*pLabelContext).Next;
    }
    RegCloseKey(hKey);

    SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR as LONG_PTR);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitImageList(
    StartResource: UINT,
    EndResource: UINT,
    Width: UINT,
    Height: UINT,
    type_: ULONG,
) -> HIMAGELIST {
    let mut hImage: HANDLE;
    let mut himl: HIMAGELIST;
    let mut i: UINT;
    let mut ret: INT;

    /* Create the toolbar icon image list */
    himl = ImageList_Create(
        (Width) as i32,
        (Height) as i32,
        ILC_MASK | ILC_COLOR32,
        (EndResource - StartResource) as i32,
        0,
    );
    if himl.is_null() {
        return core::ptr::null_mut();
    }

    ret = 0;
    i = StartResource;
    while i <= EndResource && ret != -1 {
        hImage = LoadImageW(
            hApplet,
            MAKEINTRESOURCEW!(i),
            type_ as UINT,
            (Width) as i32,
            (Height) as i32,
            LR_LOADTRANSPARENT,
        );
        if hImage.is_null() {
            ret = -1;
            break;
        }

        if type_ == IMAGE_BITMAP {
            ret = ImageList_AddMasked(
                himl,
                hImage as HBITMAP,
                RGB!(255, 0, 128),
            );
        } else if type_ == IMAGE_ICON {
            ret = ImageList_AddIcon!(himl, hImage as HICON);
        }

        DeleteObject(hImage as HGDIOBJ);
        i += 1;
    }

    if ret == -1 {
        ImageList_Destroy(himl);
        himl = core::ptr::null_mut();
    }

    return himl;
}

unsafe extern "C" fn GetSoundDuration(pFileName: LPCWSTR) -> DWORD {
    let mut openParms: MCI_OPEN_PARMSW = core::mem::zeroed();
    let mut closeParms: MCI_GENERIC_PARMS = core::mem::zeroed();
    let mut setParms: MCI_SET_PARMS = core::mem::zeroed();
    let mut statusParms: MCI_STATUS_PARMS = core::mem::zeroed();
    let mut mciError: MCIERROR = 0;

    ZeroMemory!(
        &raw mut openParms as *mut core::ffi::c_void,
        core::mem::size_of::<MCI_OPEN_PARMSW>(),
    );
    openParms.lpstrDeviceType = RUSTCONV_WSTR_F8608C2A8ECB.as_ptr();
    openParms.lpstrElementName = pFileName;

    mciError = mciSendCommandW(
        0,
        MCI_OPEN,
        (MCI_OPEN_TYPE | MCI_OPEN_ELEMENT | MCI_WAIT) as DWORD_PTR,
        &raw mut openParms as DWORD_PTR,
    );
    if mciError != 0 {
        return 0;
    }

    ZeroMemory!(
        &raw mut setParms as *mut core::ffi::c_void,
        core::mem::size_of::<MCI_SET_PARMS>(),
    );
    setParms.dwTimeFormat = MCI_FORMAT_MILLISECONDS;

    mciError = mciSendCommandW(
        openParms.wDeviceID,
        MCI_SET,
        (MCI_SET_TIME_FORMAT | MCI_WAIT) as DWORD_PTR,
        &raw mut setParms as DWORD_PTR,
    );
    if mciError == 0 {
        ZeroMemory!(
            &raw mut statusParms as *mut core::ffi::c_void,
            core::mem::size_of::<MCI_STATUS_PARMS>(),
        );
        statusParms.dwItem = MCI_STATUS_LENGTH;

        mciError = mciSendCommandW(
            openParms.wDeviceID,
            MCI_STATUS,
            (MCI_STATUS_ITEM | MCI_WAIT) as DWORD_PTR,
            &raw mut statusParms as DWORD_PTR,
        );
    }

    closeParms.dwCallback = 0;
    mciSendCommandW(
        openParms.wDeviceID,
        MCI_CLOSE,
        (MCI_WAIT) as DWORD_PTR,
        &raw mut closeParms as DWORD_PTR,
    );

    if mciError != 0 {
        return 0;
    }

    return (statusParms.dwReturn) as DWORD;
}

unsafe extern "C" fn StartSoundTest(hwndDlg: HWND, pszSound: LPCWSTR) -> BOOL {
    let mut dwDuration: DWORD;

    dwDuration = GetSoundDuration(pszSound);
    if dwDuration == 0 {
        return (FALSE) as BOOL;
    }

    if PlaySoundW(
        pszSound,
        core::ptr::null_mut(),
        SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
    ) != 0
    {
        SetTimer(
            hwndDlg,
            (ID_SOUND_TEST_TIMER) as UINT_PTR,
            dwDuration,
            None,
        );
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn StopSoundTest(hwndDlg: HWND) -> BOOL {
    if KillTimer(hwndDlg, ID_SOUND_TEST_TIMER as UINT_PTR) != 0 {
        PlaySoundW(
            core::ptr::null::<WCHAR>(),
            (core::ptr::null_mut::<HINSTANCE__>()).cast(),
            SND_ASYNC as DWORD,
        );
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn SoundsDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();

    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    let mut filename: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFilter: [WCHAR; 256] = [0; 256];
    let mut szTitle: [WCHAR; 256] = [0; 256];
    let mut pFileName: LPWSTR = core::ptr::null_mut();
    let mut lResult: LRESULT = 0;

    pGlobalData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    'sw_msg: {
        match uMsg {
            WM_INITDIALOG => {
                pGlobalData = HeapAlloc(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    core::mem::size_of::<GLOBAL_DATA>() as SIZE_T,
                ) as PGLOBAL_DATA;
                SetWindowLongPtrW(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

                (*pGlobalData).NumWavOut = waveOutGetNumDevs();

                (*pGlobalData).hPlayIcon = LoadImageW(
                    hApplet,
                    MAKEINTRESOURCEW!(IDI_PLAY_ICON) as LPCWSTR,
                    IMAGE_ICON,
                    32,
                    32,
                    LR_DEFAULTCOLOR,
                ) as HICON;
                (*pGlobalData).hStopIcon = LoadImageW(
                    hApplet,
                    MAKEINTRESOURCEW!(IDI_STOP_ICON) as LPCWSTR,
                    IMAGE_ICON,
                    32,
                    32,
                    LR_DEFAULTCOLOR,
                ) as HICON;
                SendDlgItemMessageW(
                    hwndDlg,
                    (IDC_PLAY_SOUND as i32),
                    BM_SETIMAGE,
                    (IMAGE_ICON) as WPARAM,
                    (*pGlobalData).hPlayIcon as LPARAM,
                );

                (*pGlobalData).hSoundsImageList = InitImageList(
                    IDI_SOUND_SECTION,
                    IDI_SOUND_ASSIGNED,
                    GetSystemMetrics((SM_CXSMICON) as i32) as UINT,
                    GetSystemMetrics((SM_CXSMICON) as i32) as UINT,
                    IMAGE_ICON,
                );

                LoadEventLabels(pGlobalData);
                LoadSoundProfiles(pGlobalData, hwndDlg);
                LoadSoundFiles(hwndDlg);
                ShowSoundScheme(pGlobalData, hwndDlg);

                if wParam == GetDlgItem(hwndDlg, (IDC_SOUND_SCHEME as i32)) as WPARAM {
                    return TRUE as INT_PTR;
                }
                SetFocus(GetDlgItem(hwndDlg, (IDC_SOUND_SCHEME as i32)));
                return FALSE as INT_PTR;
            }
            WM_COMMAND => {
                'sw_cmd: {
                    match (LOWORD!(wParam)) as u32 {
                        IDC_BROWSE_SOUND => {
                            ZeroMemory!(
                                core::ptr::addr_of_mut!(ofn) as *mut core::ffi::c_void,
                                core::mem::size_of::<OPENFILENAMEW>(),
                            );
                            ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
                            ofn.hwndOwner = hwndDlg;
                            ofn.lpstrFile = core::ptr::addr_of_mut!(filename) as *mut WCHAR as LPWSTR;
                            *ofn.lpstrFile.offset(0 as isize) = UNICODE_NULL as WCHAR;
                            ofn.nMaxFile = (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
                                / core::mem::size_of::<WCHAR>()) as DWORD;
                            LoadStringW(
                                hApplet,
                                IDS_WAVE_FILES_FILTER,
                                core::ptr::addr_of_mut!(szFilter) as *mut WCHAR as LPWSTR,
                                (core::mem::size_of::<[WCHAR; 256]>()
                                    / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
                            );
                            ofn.lpstrFilter = MakeFilter(
                                core::ptr::addr_of_mut!(szFilter) as *mut WCHAR as LPWSTR,
                            ) as LPCWSTR;
                            ofn.nFilterIndex = 0;
                            LoadStringW(
                                hApplet,
                                IDS_BROWSE_FOR_SOUND,
                                core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPWSTR,
                                (core::mem::size_of::<[WCHAR; 256]>()
                                    / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
                            );
                            ofn.lpstrTitle = core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR;
                            ofn.lpstrInitialDir = RUSTCONV_WSTR_EA39AA8DA3EB.as_ptr();
                            ofn.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY;

                            if GetOpenFileNameW(core::ptr::addr_of_mut!(ofn)) != 0 {
                                // search if list already contains that sound
                                lResult = FindSoundFileInList(
                                    hwndDlg,
                                    core::ptr::addr_of_mut!(filename) as *mut WCHAR as LPCWSTR,
                                );
                                if lResult != (CB_ERR) as _ {
                                    SendDlgItemMessageW(
                                        hwndDlg,
                                        (IDC_SOUND_LIST as i32),
                                        CB_SETCURSEL,
                                        lResult as WPARAM,
                                        0,
                                    );
                                    break 'sw_cmd;
                                }

                                // extract file name
                                pFileName = wcsrchr(
                                    core::ptr::addr_of_mut!(filename) as *mut WCHAR,
                                    0x005Cu16,
                                ) as LPWSTR;

                                pFileName = pFileName.offset(1);

                                // add to list
                                lResult = SendDlgItemMessageW(
                                    hwndDlg,
                                    (IDC_SOUND_LIST as i32),
                                    CB_ADDSTRING,
                                    0,
                                    pFileName as LPARAM,
                                );
                                if lResult != (CB_ERR) as _ {
                                    // add path and select item
                                    SendDlgItemMessageW(
                                        hwndDlg,
                                        (IDC_SOUND_LIST as i32),
                                        CB_SETITEMDATA,
                                        lResult as WPARAM,
                                        _wcsdup(
                                            core::ptr::addr_of_mut!(filename) as *mut WCHAR,
                                        ) as LPARAM,
                                    );
                                    SendDlgItemMessageW(
                                        hwndDlg,
                                        (IDC_SOUND_LIST as i32),
                                        CB_SETCURSEL,
                                        lResult as WPARAM,
                                        0,
                                    );
                                }
                            }
                            break 'sw_cmd;
                        }
                        IDC_PLAY_SOUND => {
                            let mut lIndex: LRESULT = 0;

                            if StopSoundTest(hwndDlg) != 0 {
                                SendDlgItemMessageW(
                                    hwndDlg,
                                    (IDC_PLAY_SOUND as i32),
                                    BM_SETIMAGE,
                                    (IMAGE_ICON) as WPARAM,
                                    (*pGlobalData).hPlayIcon as LPARAM,
                                );
                                break 'sw_cmd;
                            }

                            lIndex = ComboBox_GetCurSel!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32))) as LRESULT;
                            if lIndex == (CB_ERR) as _ {
                                break 'sw_cmd;
                            }

                            lIndex = ComboBox_GetItemData!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), lIndex);
                            if lIndex != (CB_ERR) as _ {
                                if StartSoundTest(hwndDlg, lIndex as LPWSTR) != 0 {
                                    SendDlgItemMessageW(
                                        hwndDlg,
                                        (IDC_PLAY_SOUND as i32),
                                        BM_SETIMAGE,
                                        (IMAGE_ICON) as WPARAM,
                                        (*pGlobalData).hStopIcon as LPARAM,
                                    );
                                }
                            }
                            break 'sw_cmd;
                        }
                        IDC_SOUND_SCHEME => {
                            if (HIWORD!(wParam)) as u32 == CBN_SELENDOK {
                                TreeView_DeleteAllItems!(GetDlgItem(hwndDlg, (IDC_SCHEME_LIST as i32)));
                                ShowSoundScheme(pGlobalData, hwndDlg);
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), (FALSE) as BOOL);
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_TEXT_SOUND as i32)), (FALSE) as BOOL);
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_PLAY_SOUND as i32)), (FALSE) as BOOL);
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_BROWSE_SOUND as i32)), (FALSE) as BOOL);
                                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                            }
                            break 'sw_cmd;
                        }
                        IDC_SOUND_LIST => {
                            if (HIWORD!(wParam)) as u32 == CBN_SELENDOK {
                                let mut pLabelContext: PLABEL_CONTEXT = core::ptr::null_mut();
                                let mut hItem: HTREEITEM = core::ptr::null_mut();
                                let mut item: TVITEMW = core::mem::zeroed();
                                let mut lIndex: LRESULT = 0;

                                hItem = TreeView_GetSelection!(GetDlgItem(hwndDlg, (IDC_SCHEME_LIST as i32)));
                                if hItem.is_null() {
                                    break 'sw_cmd;
                                }

                                lIndex = ComboBox_GetCurSel!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32))) as LRESULT;
                                if lIndex == (CB_ERR) as _ {
                                    break 'sw_cmd;
                                }

                                ZeroMemory!(
                                    core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
                                    core::mem::size_of::<TVITEMW>(),
                                );
                                item.mask = TVIF_PARAM;
                                item.hItem = hItem;
                                if TreeView_GetItem!(GetDlgItem(hwndDlg, (IDC_SCHEME_LIST as i32)), core::ptr::addr_of_mut!(item)) != 0 {
                                    let mut lResult: LRESULT = 0;
                                    pLabelContext = item.lParam as PLABEL_CONTEXT;

                                    lResult = ComboBox_GetItemData!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), lIndex);
                                    if lResult == (CB_ERR) as _ || lResult == 0 {
                                        if lIndex != (*pLabelContext).szValue[0 as usize] as LRESULT {
                                            /* Update the tree view item image */
                                            item.mask = TVIF_IMAGE | TVIF_SELECTEDIMAGE;
                                            item.iImage = (IMAGE_SOUND_NONE as i32);
                                            item.iSelectedImage = (IMAGE_SOUND_NONE as i32);
                                            TreeView_SetItem!(GetDlgItem(hwndDlg, (IDC_SCHEME_LIST as i32)), core::ptr::addr_of_mut!(item));

                                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);

                                            EnableWindow(GetDlgItem(hwndDlg, (IDC_PLAY_SOUND as i32)), (FALSE) as BOOL);
                                        }

                                        (*pLabelContext).szValue[0 as usize] = UNICODE_NULL as WCHAR;

                                        break 'sw_cmd;
                                    }

                                    if !pLabelContext.is_null()
                                        && (_wcsicmp(
                                            core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR,
                                            lResult as PWSTR,
                                        ) != 0
                                            || (lIndex != (*pLabelContext).szValue[0 as usize] as LRESULT))
                                    {
                                        /* Update the tree view item image */
                                        item.mask = TVIF_IMAGE | TVIF_SELECTEDIMAGE;
                                        item.iImage = (IMAGE_SOUND_ASSIGNED as i32);
                                        item.iSelectedImage = (IMAGE_SOUND_ASSIGNED as i32);
                                        TreeView_SetItem!(GetDlgItem(hwndDlg, (IDC_SCHEME_LIST as i32)), core::ptr::addr_of_mut!(item));

                                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);

                                        ///
                                        /// Should store in current member
                                        ///
                                        StringCchCopyW(
                                            core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR,
                                            (core::mem::size_of::<[WCHAR; MAX_PATH as usize]>()
                                                / core::mem::size_of::<WCHAR>()),
                                            lResult as PWSTR,
                                        );
                                    }

                                    if wcslen(lResult as PWSTR) != 0
                                        && lIndex != 0
                                        && (*pGlobalData).NumWavOut != 0
                                    {
                                        EnableWindow(GetDlgItem(hwndDlg, (IDC_PLAY_SOUND as i32)), (TRUE) as BOOL);
                                    } else {
                                        EnableWindow(GetDlgItem(hwndDlg, (IDC_PLAY_SOUND as i32)), (FALSE) as BOOL);
                                    }
                                }
                            }
                            break 'sw_cmd;
                        }
                        _ => {}
                    }
                }
                break 'sw_msg;
            }
            WM_DESTROY => {
                StopSoundTest(hwndDlg);
                FreeSoundFiles(hwndDlg);
                FreeSoundProfiles(hwndDlg);
                FreeAppMap(pGlobalData);
                FreeLabelMap(pGlobalData);
                if !(*pGlobalData).hSoundsImageList.is_null() {
                    ImageList_Destroy((*pGlobalData).hSoundsImageList);
                }

                if !(*pGlobalData).hStopIcon.is_null() {
                    DestroyIcon((*pGlobalData).hStopIcon);
                }

                if !(*pGlobalData).hPlayIcon.is_null() {
                    DestroyIcon((*pGlobalData).hPlayIcon);
                }

                HeapFree(GetProcessHeap(), 0, pGlobalData as PVOID);
                break 'sw_msg;
            }
            WM_NOTIFY => {
                let mut pLabelContext: PLABEL_CONTEXT = core::ptr::null_mut();
                let mut ptr: PWSTR = core::ptr::null_mut();

                let mut lpnm: LPNMHDR = lParam as LPNMHDR;

                'sw_code: {
                    match ((*lpnm).code) as i32 {
                        PSN_APPLY => {
                            ApplyChanges(hwndDlg);
                            break 'sw_code;
                        }
                        PSN_KILLACTIVE => {
                            if StopSoundTest(hwndDlg) != 0 {
                                SendDlgItemMessageW(
                                    hwndDlg,
                                    (IDC_PLAY_SOUND as i32),
                                    BM_SETIMAGE,
                                    (IMAGE_ICON) as WPARAM,
                                    (*pGlobalData).hPlayIcon as LPARAM,
                                );
                            }

                            break 'sw_code;
                        }
                        __c if __c == TVN_SELCHANGED as i32 => {
                            let mut nm: LPNMTREEVIEWW = lParam as LPNMTREEVIEWW;
                            let mut lCount: LRESULT = 0;
                            let mut lIndex: LRESULT = 0;
                            let mut lResult: LRESULT = 0;

                            if StopSoundTest(hwndDlg) != 0 {
                                SendDlgItemMessageW(
                                    hwndDlg,
                                    (IDC_PLAY_SOUND as i32),
                                    BM_SETIMAGE,
                                    (IMAGE_ICON) as WPARAM,
                                    (*pGlobalData).hPlayIcon as LPARAM,
                                );
                            }

                            pLabelContext = (*nm).itemNew.lParam as PLABEL_CONTEXT;
                            if pLabelContext.is_null() {
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), (FALSE) as BOOL);
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_TEXT_SOUND as i32)), (FALSE) as BOOL);
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_BROWSE_SOUND as i32)), (FALSE) as BOOL);
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_PLAY_SOUND as i32)), (FALSE) as BOOL);
                                return FALSE as INT_PTR;
                            }

                            EnableWindow(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), (TRUE) as BOOL);
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_TEXT_SOUND as i32)), (TRUE) as BOOL);
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_BROWSE_SOUND as i32)), (TRUE) as BOOL);

                            if wcslen(core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR) == 0 {
                                lIndex = ComboBox_SetCurSel!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), 0) as LRESULT;
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_PLAY_SOUND as i32)), (FALSE) as BOOL);
                                break 'sw_code;
                            }

                            if (*pGlobalData).NumWavOut != 0 {
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_PLAY_SOUND as i32)), (TRUE) as BOOL);
                            }

                            lCount = ComboBox_GetCount!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32))) as LRESULT;
                            lIndex = 0;
                            while lIndex < lCount {
                                'body: {
                                    lResult = ComboBox_GetItemData!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), lIndex);
                                    if lResult == (CB_ERR) as _ || lResult == 0 {
                                        break 'body;
                                    }

                                    if wcscmp(
                                        lResult as PWSTR,
                                        core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR,
                                    ) == 0
                                    {
                                        ComboBox_SetCurSel!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), lIndex);
                                        return FALSE as INT_PTR;
                                    }
                                }
                                lIndex += 1;
                            }

                            ptr = wcsrchr(
                                core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR,
                                0x005Cu16,
                            ) as PWSTR;
                            if !ptr.is_null() {
                                ptr = ptr.offset(1);
                            } else {
                                ptr = core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR as PWSTR;
                            }

                            lIndex = ComboBox_AddString!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), ptr as LPCWSTR) as LRESULT;
                            if lIndex != (CB_ERR) as _ {
                                ComboBox_SetItemData!(
                                    GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)),
                                    lIndex,
                                    _wcsdup(core::ptr::addr_of_mut!((*pLabelContext).szValue) as *mut WCHAR),
                                );
                                ComboBox_SetCurSel!(GetDlgItem(hwndDlg, (IDC_SOUND_LIST as i32)), lIndex);
                            }
                            break 'sw_code;
                        }
                        _ => {}
                    }
                }
                break 'sw_msg;
            }
            WM_TIMER => {
                if wParam == ID_SOUND_TEST_TIMER as WPARAM {
                    KillTimer(hwndDlg, (ID_SOUND_TEST_TIMER) as UINT_PTR);
                    SendDlgItemMessageW(
                        hwndDlg,
                        (IDC_PLAY_SOUND as i32),
                        BM_SETIMAGE,
                        (IMAGE_ICON) as WPARAM,
                        (*pGlobalData).hPlayIcon as LPARAM,
                    );
                }

                break 'sw_msg;
            }
            _ => {}
        }
    }

    return FALSE as INT_PTR;
}
