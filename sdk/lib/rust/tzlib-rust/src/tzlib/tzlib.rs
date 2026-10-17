// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_09A7C6FF8E65: [u16; 8] = [0x0044u16, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_0D644A93E8FC: [u16; 4] = [0x0054u16, 0x005A, 0x0049, 0x0000];
pub static RUSTCONV_WSTR_19D2AA1F9941: [u16; 4] = [0x0053u16, 0x0074, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_39CDE5D97C33: [u16; 8] = [0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_3D611D143C84: [u16; 6] = [0x0049u16, 0x006E, 0x0064, 0x0065, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_9A3585C3EC10: [u16; 27] = [
    0x0044u16, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0041, 0x0075, 0x0074,
    0x006F, 0x0044, 0x0061, 0x0079, 0x006C, 0x0069, 0x0067, 0x0068, 0x0074, 0x0054,
    0x0069, 0x006D, 0x0065, 0x0053, 0x0065, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_A55FEFE44CAF: [u16; 53] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0054, 0x0069, 0x006D, 0x0065, 0x005A, 0x006F, 0x006E,
    0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069,
    0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_B81589FB80C8: [u16; 4] = [0x0044u16, 0x006C, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_D30964B5BBF7: [u16; 56] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0054, 0x0069, 0x006D, 0x0065, 0x0020,
    0x005A, 0x006F, 0x006E, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_D6D237BF5118: [u16; 13] = [
    0x0049u16, 0x006E, 0x0064, 0x0065, 0x0078, 0x004D, 0x0061, 0x0070, 0x0070, 0x0069,
    0x006E, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_EAE8EEC66C20: [u16; 46] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x004E, 0x004C, 0x0053, 0x005C, 0x004C, 0x0061, 0x006E,
    0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn GetTimeZoneListIndex(pIndex: PULONG) -> BOOL {
    let mut lError: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwValueSize: DWORD = 0;
    let mut Length: DWORD = 0;
    let mut Buffer: LPWSTR = core::ptr::null_mut();
    let mut Ptr: LPWSTR = core::ptr::null_mut();
    let mut End: LPWSTR = core::ptr::null_mut();
    let mut bFound: BOOL = (FALSE as i32);
    let mut iLanguageID: ULONG = 0;
    let mut szLanguageIdString: [WCHAR; 9] = [0; 9];

    if *pIndex == (-(1 as i32)) as ULONG {
        *pIndex = 85; /* fallback to GMT time zone */

        lError = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE as HKEY,
            RUSTCONV_WSTR_EAE8EEC66C20.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hKey),
        ) as LONG;
        if lError != (ERROR_SUCCESS as i32) {
            return (FALSE) as BOOL;
        }

        dwValueSize = core::mem::size_of::<[WCHAR; 9]>() as DWORD;
        lError = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_39CDE5D97C33.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szLanguageIdString) as LPBYTE,
            core::ptr::addr_of_mut!(dwValueSize),
        ) as LONG;
        if lError != (ERROR_SUCCESS as i32) {
            RegCloseKey(hKey);
            return (FALSE) as BOOL;
        }

        iLanguageID = wcstoul(
            core::ptr::addr_of_mut!(szLanguageIdString) as *const WCHAR,
            core::ptr::null_mut(),
            16,
        );
        RegCloseKey(hKey);
    } else {
        iLanguageID = *pIndex;
    }

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_D30964B5BBF7.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    if lError != (ERROR_SUCCESS as i32) {
        return (FALSE) as BOOL;
    }

    dwValueSize = 0;
    lError = RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_D6D237BF5118.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwValueSize),
    ) as LONG;
    if (lError != (ERROR_SUCCESS as i32)) || (dwType != REG_MULTI_SZ) {
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }

    Buffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        dwValueSize as SIZE_T,
    ) as LPWSTR;
    if Buffer.is_null() {
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }

    lError = RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_D6D237BF5118.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        Buffer as LPBYTE,
        core::ptr::addr_of_mut!(dwValueSize),
    ) as LONG;

    RegCloseKey(hKey);

    if (lError != (ERROR_SUCCESS as i32)) || (dwType != REG_MULTI_SZ) {
        HeapFree(
            GetProcessHeap(),
            0,
            Buffer as PVOID,
        );
        return (FALSE) as BOOL;
    }

    Ptr = Buffer;
    while *Ptr != 0 {
        Length = wcslen(Ptr as *const WCHAR) as DWORD;
        if wcstoul(
            Ptr as *const WCHAR,
            core::ptr::null_mut(),
            16,
        ) == iLanguageID
        {
            bFound = (TRUE as i32);
        }

        Ptr = Ptr.add(Length as usize + 1 as usize);
        if *Ptr == 0 {
            break;
        }

        if bFound != 0 {
            *pIndex = wcstoul(
                Ptr as *const WCHAR,
                core::ptr::addr_of_mut!(End),
                10,
            ) as ULONG;
            HeapFree(
                GetProcessHeap(),
                0,
                Buffer as PVOID,
            );
            return (TRUE) as BOOL;
        }

        Length = wcslen(Ptr as *const WCHAR) as DWORD;
        Ptr = Ptr.add(Length as usize + 1 as usize);
    }

    HeapFree(
        GetProcessHeap(),
        0,
        Buffer as PVOID,
    );
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn QueryTimeZoneData(
    hZoneKey: HKEY,
    Index: PULONG,
    TimeZoneInfo: PREG_TZI_FORMAT,
    Description: PWCHAR,
    DescriptionSize: PULONG,
    StandardName: PWCHAR,
    StandardNameSize: PULONG,
    DaylightName: PWCHAR,
    DaylightNameSize: PULONG,
) -> LONG {
    let mut lError: LONG;
    let mut dwValueSize: DWORD;

    if !Index.is_null() {
        dwValueSize = ::core::mem::size_of::<ULONG>() as DWORD;
        lError = RegQueryValueExW(
            hZoneKey,
            RUSTCONV_WSTR_3D611D143C84.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Index as LPBYTE,
            core::ptr::addr_of_mut!(dwValueSize),
        );
        if lError != (ERROR_SUCCESS as i32) {
            *Index = 0;
        }
    }

    /* The time zone information structure is mandatory for a valid time zone */
    dwValueSize = ::core::mem::size_of::<_REG_TZI_FORMAT>() as DWORD;
    lError = RegQueryValueExW(
        hZoneKey,
        RUSTCONV_WSTR_0D644A93E8FC.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        TimeZoneInfo as LPBYTE,
        core::ptr::addr_of_mut!(dwValueSize),
    );
    if lError != (ERROR_SUCCESS as i32) {
        return lError;
    }

    if !Description.is_null() && !DescriptionSize.is_null() && *DescriptionSize > 0 {
        lError = RegQueryValueExW(
            hZoneKey,
            RUSTCONV_WSTR_09A7C6FF8E65.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Description as LPBYTE,
            DescriptionSize as LPDWORD,
        );
        if lError != (ERROR_SUCCESS as i32) {
            *Description = 0;
        }
    }

    if !StandardName.is_null() && !StandardNameSize.is_null() && *StandardNameSize > 0 {
        lError = RegQueryValueExW(
            hZoneKey,
            RUSTCONV_WSTR_19D2AA1F9941.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            StandardName as LPBYTE,
            StandardNameSize as LPDWORD,
        );
        if lError != (ERROR_SUCCESS as i32) {
            *StandardName = 0;
        }
    }

    if !DaylightName.is_null() && !DaylightNameSize.is_null() && *DaylightNameSize > 0 {
        lError = RegQueryValueExW(
            hZoneKey,
            RUSTCONV_WSTR_B81589FB80C8.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            DaylightName as LPBYTE,
            DaylightNameSize as LPDWORD,
        );
        if lError != (ERROR_SUCCESS as i32) {
            *DaylightName = 0;
        }
    }

    return (ERROR_SUCCESS) as LONG;
}
#[no_mangle]
pub unsafe extern "C" fn EnumerateTimeZoneList(
    Callback: PENUM_TIMEZONE_CALLBACK,
    Context: PVOID,
) {
    let mut lError: LONG;
    let mut hZonesKey: HKEY = core::ptr::null_mut();
    let mut hZoneKey: HKEY = core::ptr::null_mut();
    let mut dwIndex: DWORD;
    let mut dwNameSize: DWORD;
    let mut szKeyName: [WCHAR; 256] = [0; 256];

    /* Open the registry key containing the list of time zones */
    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_D30964B5BBF7.as_ptr(),
        0,
        KEY_ENUMERATE_SUB_KEYS,
        core::ptr::addr_of_mut!(hZonesKey),
    ) as LONG;
    if lError != (ERROR_SUCCESS as i32) {
        return;
    }

    /* Enumerate it */
    dwIndex = 0;
    'enumerate: loop {
        'body: {
            dwNameSize = core::mem::size_of::<[WCHAR; 256]>() as DWORD;
            lError = RegEnumKeyExW(
                hZonesKey,
                dwIndex,
                core::ptr::addr_of_mut!(szKeyName) as LPWSTR,
                core::ptr::addr_of_mut!(dwNameSize),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as LONG;
            // if (lError != ERROR_SUCCESS && lError != ERROR_MORE_DATA)
            if lError == (ERROR_NO_MORE_ITEMS as i32) {
                break 'enumerate;
            }

            /* Open the time zone sub-key */
            if RegOpenKeyExW(
                hZonesKey,
                core::ptr::addr_of_mut!(szKeyName) as LPCWSTR,
                0,
                KEY_QUERY_VALUE,
                core::ptr::addr_of_mut!(hZoneKey),
            ) != 0
            {
                /* We failed, continue with another sub-key */
                break 'body;
            }

            /* Call the user-provided callback */
            lError = (Callback.unwrap_unchecked())(hZoneKey, Context) as LONG;
            // lError = QueryTimeZoneData(hZoneKey, Context);

            RegCloseKey(hZoneKey);
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    RegCloseKey(hZonesKey);
}
#[no_mangle]
pub unsafe extern "C" fn GetAutoDaylight() -> BOOL {
    let mut lError: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwDisabled: DWORD = 0;
    let mut dwValueSize: DWORD = 0;

    lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE as HKEY,
                           RUSTCONV_WSTR_A55FEFE44CAF.as_ptr() as LPCWSTR,
                           0 as DWORD,
                           KEY_QUERY_VALUE as REGSAM,
                           core::ptr::addr_of_mut!(hKey)) as LONG;
    if lError != ERROR_SUCCESS as LONG {
        return FALSE as BOOL;
    }

    // NOTE: On Vista+: REG_DWORD "DynamicDaylightTimeDisabled"
    dwValueSize = core::mem::size_of::<DWORD>() as DWORD;
    lError = RegQueryValueExW(hKey,
                              RUSTCONV_WSTR_9A3585C3EC10.as_ptr() as LPCWSTR,
                              core::ptr::null_mut(),
                              core::ptr::addr_of_mut!(dwType),
                              core::ptr::addr_of_mut!(dwDisabled) as LPBYTE,
                              core::ptr::addr_of_mut!(dwValueSize)) as LONG;

    RegCloseKey(hKey);

    if (lError != ERROR_SUCCESS as LONG)
        || (dwType != REG_DWORD as DWORD)
        || ((dwValueSize as usize) != core::mem::size_of::<DWORD>())
    {
        /*
         * The call failed (non zero) because the registry value isn't available,
         * which means auto-daylight shouldn't be disabled.
         */
        dwDisabled = FALSE as DWORD;
    }

    return (dwDisabled == 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SetAutoDaylight(mut EnableAutoDaylightTime: BOOL) {
    let mut lError: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwDisabled: DWORD = TRUE;

    lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                           RUSTCONV_WSTR_A55FEFE44CAF.as_ptr(),
                           0,
                           KEY_SET_VALUE,
                           core::ptr::addr_of_mut!(hKey)) as LONG;
    if lError != ERROR_SUCCESS as LONG {
        return;
    }

    if EnableAutoDaylightTime == 0 {
        /* Auto-Daylight disabled: set the value to TRUE */
        // NOTE: On Vista+: REG_DWORD "DynamicDaylightTimeDisabled"
        RegSetValueExW(hKey,
                       RUSTCONV_WSTR_9A3585C3EC10.as_ptr(),
                       0,
                       REG_DWORD,
                       core::ptr::addr_of_mut!(dwDisabled) as LPBYTE,
                       core::mem::size_of::<DWORD>() as DWORD);
    } else {
        /* Auto-Daylight enabled: just delete the value */
        RegDeleteValueW(hKey, RUSTCONV_WSTR_9A3585C3EC10.as_ptr());
    }

    RegCloseKey(hKey);
}
