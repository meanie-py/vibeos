// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0310757CC873: [u16; 17] = [
    0x0053u16, 0x0065, 0x0072, 0x0069, 0x0061, 0x006C, 0x0020, 0x0050, 0x006F, 0x0072,
    0x0074, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000,
];
pub static RUSTCONV_WSTR_08A9E10E053A: [u16; 11] = [
    0x0042u16, 0x006F, 0x006F, 0x0074, 0x0043, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067,
    0x0000,
];
pub static RUSTCONV_WSTR_3CA71E797F46: [u16; 8] = [0x004Cu16, 0x006F, 0x0067, 0x0043, 0x006F, 0x006E, 0x0066, 0x0000];
pub static RUSTCONV_WSTR_516C3CC4ECB4: [u16; 13] = [
    0x0050u16, 0x006F, 0x0072, 0x0074, 0x0053, 0x0075, 0x0062, 0x0043, 0x006C, 0x0061,
    0x0073, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_5E7833CF4723: [u16; 5] = [0x0025u16, 0x0073, 0x0025, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_6C902C4E71A8: [u16; 5] = [0x004Cu16, 0x0050, 0x0054, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_6FB7FC12E6B0: [u16; 21] = [
    0x0046u16, 0x0069, 0x006C, 0x0074, 0x0065, 0x0072, 0x0052, 0x0065, 0x0073, 0x006F,
    0x0075, 0x0072, 0x0063, 0x0065, 0x004D, 0x0065, 0x0074, 0x0068, 0x006F, 0x0064,
    0x0000,
];
pub static RUSTCONV_WSTR_AA3F7978D9EA: [u16; 30] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0045, 0x006E, 0x0075, 0x006D, 0x0000,
];
pub static RUSTCONV_WSTR_AF854C3D2274: [u16; 8] = [0x0025u16, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000];
pub static RUSTCONV_WSTR_C1D34563AF12: [u16; 9] = [0x0050u16, 0x006F, 0x0072, 0x0074, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_E76BB3C8AF6F: [u16; 19] = [
    0x0050u16, 0x0061, 0x0072, 0x0061, 0x006C, 0x006C, 0x0065, 0x006C, 0x0020, 0x0050,
    0x006F, 0x0072, 0x0074, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_3DB566BC3DA4: [u16; 4] = [0x0043u16, 0x004F, 0x004D, 0x0000];
pub static RUSTCONV_C2R_WSTR_C372F2CCF22D: [u16; 4] = [0x004Cu16, 0x0050, 0x0054, 0x0000];

#[no_mangle]
pub static mut pszCom: LPWSTR = unsafe {
    RUSTCONV_C2R_WSTR_3DB566BC3DA4.as_ptr()
        as LPWSTR
};

#[no_mangle]
pub static mut pszLpt: LPWSTR = unsafe {
    RUSTCONV_C2R_WSTR_C372F2CCF22D.as_ptr()
        as LPWSTR
};

pub const UnknownPort: i32 = 0;
pub const ParallelPort: i32 = 1;
pub const SerialPort: i32 = 2;
pub type PORT_TYPE = i32;
#[no_mangle]
pub unsafe extern "C" fn GetBootResourceList(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ppResourceList: *mut PCM_RESOURCE_LIST,
) -> BOOL {
    let mut DeviceInstanceIdBuffer: [WCHAR; 128] = [0; 128];
    let mut hEnumKey: HKEY = core::ptr::null_mut();
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let mut hConfigKey: HKEY = core::ptr::null_mut();
    let mut lpBuffer: LPBYTE = core::ptr::null_mut();
    let mut dwDataSize: DWORD = 0;
    let mut lError: LONG = 0;
    let mut ret: BOOL = (FALSE as i32);

    *ppResourceList = core::ptr::null_mut();

    'done: {
        if SetupDiGetDeviceInstanceIdW(
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::addr_of_mut!(DeviceInstanceIdBuffer) as PWSTR,
            ARRAYSIZE!(DeviceInstanceIdBuffer),
            core::ptr::addr_of_mut!(dwDataSize),
        ) == 0
        {
            ERR!("SetupDiGetDeviceInstanceIdW() failed\n");
            return (FALSE) as BOOL;
        }

        lError = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_AA3F7978D9EA.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hEnumKey),
        );
        if lError != (ERROR_SUCCESS as i32) {
            ERR!("RegOpenKeyExW() failed (Error %lu)\n", lError);
            break 'done;
        }

        lError = RegOpenKeyExW(
            hEnumKey,
            core::ptr::addr_of_mut!(DeviceInstanceIdBuffer) as LPCWSTR,
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hDeviceKey),
        );
        if lError != (ERROR_SUCCESS as i32) {
            ERR!("RegOpenKeyExW() failed (Error %lu)\n", lError);
            break 'done;
        }

        lError = RegOpenKeyExW(
            hDeviceKey,
            RUSTCONV_WSTR_3CA71E797F46.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hConfigKey),
        );
        if lError != (ERROR_SUCCESS as i32) {
            ERR!("RegOpenKeyExW() failed (Error %lu)\n", lError);
            break 'done;
        }

        /* Get the configuration data size */
        lError = RegQueryValueExW(
            hConfigKey,
            RUSTCONV_WSTR_08A9E10E053A.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwDataSize),
        );
        if lError != (ERROR_SUCCESS as i32) {
            ERR!("RegQueryValueExW() failed (Error %lu)\n", lError);
            break 'done;
        }

        /* Allocate the buffer */
        lpBuffer = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwDataSize) as SIZE_T,
        ) as LPBYTE;
        if lpBuffer.is_null() {
            ERR!("Failed to allocate the resource list buffer\n");
            break 'done;
        }

        /* Retrieve the configuration data */
        lError = RegQueryValueExW(
            hConfigKey,
            RUSTCONV_WSTR_08A9E10E053A.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            lpBuffer,
            core::ptr::addr_of_mut!(dwDataSize),
        );
        if lError == (ERROR_SUCCESS as i32) {
            ERR!("RegQueryValueExW() failed (Error %lu)\n", lError);
            ret = (TRUE as i32);
        }
    }

    if ret == (FALSE as i32) && !lpBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, lpBuffer as PVOID);
    }

    if !hConfigKey.is_null() {
        RegCloseKey(hConfigKey);
    }

    if !hDeviceKey.is_null() {
        RegCloseKey(hDeviceKey);
    }

    if !hEnumKey.is_null() {
        RegCloseKey(hEnumKey);
    }

    if ret != (FALSE as i32) {
        *ppResourceList = lpBuffer as PCM_RESOURCE_LIST;
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn GetSerialPortNumber(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    let mut lpResourceList: PCM_RESOURCE_LIST = core::ptr::null_mut();
    let mut lpResDes: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut i: ULONG;
    let mut dwBaseAddress: DWORD = 0;
    let mut dwPortNumber: DWORD = 0;

    if GetBootResourceList(DeviceInfoSet,
                            DeviceInfoData,
                            &mut lpResourceList as *mut PCM_RESOURCE_LIST) != 0
    {
        if (*lpResourceList).Count > 0
        {
            i = 0;
            while i < (*lpResourceList).List[0].PartialResourceList.Count
            {
                lpResDes = (core::ptr::addr_of_mut!((*lpResourceList).List[0].PartialResourceList.PartialDescriptors)
                    as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                    .offset(i as isize)
                    as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

                match ((*lpResDes).Type as i32) as u32 {
                    CmResourceTypePort => {
                        TRACE!("Port: Start: %I64x  Length: %lu\n",
                              (*lpResDes).u.Port.Start.QuadPart,
                              (*lpResDes).u.Port.Length);
                        if ((*lpResDes).u.Port.Start.__bindgen_anon_1.HighPart == 0) && (dwBaseAddress == 0)
                        {
                            dwBaseAddress = (*lpResDes).u.Port.Start.__bindgen_anon_1.LowPart as DWORD;
                        }
                    }

                    CmResourceTypeInterrupt => {
                        TRACE!("Interrupt: Level: %lu  Vector: %lu\n",
                              (*lpResDes).u.Interrupt.Level,
                              (*lpResDes).u.Interrupt.Vector);
                    }

                    _ => {}
                }
                i += 1;
            }
        }

        HeapFree(GetProcessHeap(), 0, lpResourceList as PVOID);
    }

    match dwBaseAddress {
        0x3f8 => {
            dwPortNumber = 1;
        }

        0x2f8 => {
            dwPortNumber = 2;
        }

        0x3e8 => {
            dwPortNumber = 3;
        }

        0x2e8 => {
            dwPortNumber = 4;
        }

        _ => {}
    }

    return dwPortNumber;
}
#[no_mangle]
pub unsafe extern "C" fn GetParallelPortNumber(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    let mut lpResourceList: PCM_RESOURCE_LIST = core::ptr::null_mut();
    let mut lpResDes: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut i: ULONG;
    let mut dwBaseAddress: DWORD = 0;
    let mut dwPortNumber: DWORD = 0;

    if GetBootResourceList(
        DeviceInfoSet,
        DeviceInfoData,
        core::ptr::addr_of_mut!(lpResourceList),
    ) != 0
    {
        if (*lpResourceList).Count > 0 {
            i = 0;
            while i
                < (*lpResourceList).List[0 as usize]
                    .PartialResourceList
                    .Count
            {
                lpResDes = core::ptr::addr_of_mut!(
                    (*core::ptr::addr_of_mut!((*lpResourceList).List[0 as usize]
                        .PartialResourceList
                        .PartialDescriptors[0]).add(i as usize))
                );

                match ((*lpResDes).Type as i32) as u32 {
                    CmResourceTypePort => {
                        TRACE!(
                            "Port: Start: %I64x  Length: %lu\n",
                            (*lpResDes).u.Port.Start.QuadPart,
                            (*lpResDes).u.Port.Length
                        );
                        if (*lpResDes).u.Port.Start.__bindgen_anon_1.HighPart == 0
                            && dwBaseAddress == 0
                        {
                            dwBaseAddress = (*lpResDes).u.Port.Start.__bindgen_anon_1.LowPart as DWORD;
                        }
                    }
                    CmResourceTypeInterrupt => {
                        TRACE!(
                            "Interrupt: Level: %lu  Vector: %lu\n",
                            (*lpResDes).u.Interrupt.Level,
                            (*lpResDes).u.Interrupt.Vector
                        );
                    }
                    _ => {}
                }

                i = i.wrapping_add(1);
            }
        }

        HeapFree(
            GetProcessHeap(),
            0,
            lpResourceList as PVOID,
        );
    }

    match dwBaseAddress {
        0x378 => {
            dwPortNumber = 1;
        }
        0x278 => {
            dwPortNumber = 2;
        }
        _ => {}
    }

    return dwPortNumber;
}

unsafe extern "C" fn InstallSerialPort(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    let mut szDeviceDescription: [WCHAR; 256] = [0; 256];
    let mut szFriendlyName: [WCHAR; 256] = [0; 256];
    let mut szPortName: [WCHAR; 8] = [0; 8];
    let mut dwPortNumber: DWORD = 0 as DWORD;
    let mut dwSize: DWORD;
    let mut hComDB: HCOMDB = HCOMDB_INVALID_HANDLE_VALUE as HCOMDB;
    let mut hKey: HKEY;
    let mut lError: LONG;

    /* Open the com port database */
    ComDBOpen(core::ptr::addr_of_mut!(hComDB));

    /* Try to read the 'PortName' value and determine the port number */
    hKey = SetupDiCreateDevRegKeyW(
        DeviceInfoSet,
        DeviceInfoData,
        DICS_FLAG_GLOBAL as DWORD,
        0 as DWORD,
        DIREG_DEV as DWORD,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if hKey != INVALID_HANDLE_VALUE as HKEY {
        dwSize = core::mem::size_of::<[WCHAR; 8]>() as DWORD;
        lError = RegQueryValueEx(
            hKey,
            RUSTCONV_WSTR_C1D34563AF12.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            szPortName.as_mut_ptr() as PBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        if lError == ERROR_SUCCESS as LONG {
            if _wcsnicmp(
                szPortName.as_mut_ptr(),
                pszCom,
                wcslen(pszCom) as usize,
            ) == 0
            {
                dwPortNumber = _wtoi(
                    szPortName
                        .as_mut_ptr()
                        .offset(wcslen(pszCom) as isize),
                ) as DWORD;
            }
        }

        RegCloseKey(hKey);
    }

    /* Determine the port number from its resources ... */
    if dwPortNumber == 0 as DWORD {
        dwPortNumber = GetSerialPortNumber(DeviceInfoSet, DeviceInfoData);
    }

    if dwPortNumber != 0 as DWORD {
        /* ... and claim the port number in the database */
        ComDBClaimPort(
            hComDB,
            dwPortNumber,
            FALSE as BOOL,
            core::ptr::null_mut(),
        );
    } else {
        /* ... or claim the next free port number */
        ComDBClaimNextFreePort(hComDB, core::ptr::addr_of_mut!(dwPortNumber));
    }

    /* Build the name of the port device */
    _swprintf(
        szPortName.as_mut_ptr(),
        RUSTCONV_WSTR_5E7833CF4723.as_ptr(),
        pszCom,
        dwPortNumber,
    );

    /* Close the com port database */
    if hComDB != HCOMDB_INVALID_HANDLE_VALUE as HCOMDB {
        ComDBClose(hComDB);
    }

    /* Set the 'PortName' value */
    hKey = SetupDiCreateDevRegKeyW(
        DeviceInfoSet,
        DeviceInfoData,
        DICS_FLAG_GLOBAL as DWORD,
        0 as DWORD,
        DIREG_DEV as DWORD,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if hKey != INVALID_HANDLE_VALUE as HKEY {
        RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_C1D34563AF12.as_ptr(),
            0 as DWORD,
            REG_SZ as DWORD,
            szPortName.as_mut_ptr() as LPBYTE as *const BYTE,
            ((wcslen(szPortName.as_mut_ptr()) as usize
                + 1 as usize)
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );

        RegCloseKey(hKey);
    }

    /* Install the device */
    if SetupDiInstallDevice(DeviceInfoSet, DeviceInfoData) == 0 {
        return GetLastError();
    }

    /* Get the device description... */
    if SetupDiGetDeviceRegistryPropertyW(
        DeviceInfoSet,
        DeviceInfoData,
        SPDRP_DEVICEDESC as DWORD,
        core::ptr::null_mut(),
        szDeviceDescription.as_mut_ptr() as PBYTE,
        (256 as usize * core::mem::size_of::<WCHAR>()) as DWORD,
        core::ptr::null_mut(),
    ) != 0
    {
        /* ... and use it to build a new friendly name */
        _swprintf(
            szFriendlyName.as_mut_ptr(),
            RUSTCONV_WSTR_AF854C3D2274.as_ptr(),
            szDeviceDescription.as_mut_ptr(),
            szPortName.as_mut_ptr(),
        );
    } else {
        /* ... or build a generic friendly name */
        _swprintf(
            szFriendlyName.as_mut_ptr(),
            RUSTCONV_WSTR_0310757CC873.as_ptr(),
            szPortName.as_mut_ptr(),
        );
    }

    /* Set the friendly name for the device */
    SetupDiSetDeviceRegistryPropertyW(
        DeviceInfoSet,
        DeviceInfoData,
        SPDRP_FRIENDLYNAME as DWORD,
        szFriendlyName.as_mut_ptr() as LPBYTE,
        ((wcslen(szFriendlyName.as_mut_ptr()) as usize
            + 1 as usize)
            * core::mem::size_of::<WCHAR>()) as DWORD,
    );

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn InstallParallelPort(mut DeviceInfoSet: HDEVINFO, mut DeviceInfoData: PSP_DEVINFO_DATA) -> DWORD {
    let mut szDeviceDescription: [WCHAR; 256] = [0; 256];
    let mut szFriendlyName: [WCHAR; 256] = [0; 256];
    let mut szPortName: [WCHAR; 8] = [0; 8];
    let mut dwPortNumber: DWORD = 0 as DWORD;
    let mut dwSize: DWORD = 0;
    let mut dwValue: DWORD = 0;
    let mut lError: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    /* Try to read the 'PortName' value and determine the port number */
    hKey = SetupDiCreateDevRegKeyW(
        DeviceInfoSet,
        DeviceInfoData,
        DICS_FLAG_GLOBAL as DWORD,
        0 as DWORD,
        DIREG_DEV as DWORD,
        core::ptr::null_mut(),
        core::ptr::null::<WCHAR>(),
    );
    if hKey != INVALID_HANDLE_VALUE as HKEY {
        dwSize = core::mem::size_of_val(&szPortName) as DWORD;
        lError = RegQueryValueEx(
            hKey,
            RUSTCONV_WSTR_C1D34563AF12.as_ptr(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(szPortName) as *mut WCHAR as PBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        if lError == ERROR_SUCCESS as LONG {
            if _wcsnicmp(
                core::ptr::addr_of_mut!(szPortName) as *mut WCHAR,
                pszLpt,
                wcslen(pszLpt) as usize,
            ) == 0 as i32
            {
                dwPortNumber = _wtoi(
                    (core::ptr::addr_of_mut!(szPortName) as *mut WCHAR)
                        .offset(wcslen(pszLpt) as isize),
                ) as DWORD;
                if ((*__wine_dbch___default).flags as i32
                    & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        (__dbcl_0) as u32,
                        __dbch_0,
                        b"dll/win32/msports/classinst.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"InstallParallelPort\0" as *const u8 as *const ::core::ffi::c_char,
                        454 as ::core::ffi::c_int,
                        b"LPT port number found: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
                        dwPortNumber,
                    );
                }
            }
        }
        RegCloseKey(hKey);
    }

    /* ... try to determine the port number from its resources */
    if dwPortNumber == 0 as DWORD {
        dwPortNumber = GetParallelPortNumber(DeviceInfoSet, DeviceInfoData) as DWORD;
        if ((*__wine_dbch___default).flags as i32
            & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                (__dbcl_1) as u32,
                __dbch_1,
                b"dll/win32/msports/classinst.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"InstallParallelPort\0" as *const u8 as *const ::core::ffi::c_char,
                466 as ::core::ffi::c_int,
                b"GetParallelPortNumber() returned port number: %lu\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                dwPortNumber,
            );
        }
    }

    if dwPortNumber == 0 as DWORD {
        if ((*__wine_dbch___default).flags as i32
            & ((1i32) << (__WINE_DBCL_FIXME as i32))) != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                (__dbcl_2) as u32,
                __dbch_2,
                b"dll/win32/msports/classinst.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"InstallParallelPort\0" as *const u8 as *const ::core::ffi::c_char,
                472 as ::core::ffi::c_int,
                b"Got no valid port numer!\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }

    if dwPortNumber != 0 as DWORD {
        _swprintf(
            core::ptr::addr_of_mut!(szPortName) as *mut WCHAR,
            RUSTCONV_WSTR_5E7833CF4723.as_ptr(),
            pszLpt,
            dwPortNumber,
        );
    } else {
        wcscpy(
            core::ptr::addr_of_mut!(szPortName) as *mut WCHAR,
            RUSTCONV_WSTR_6C902C4E71A8.as_ptr(),
        );
    }

    if dwPortNumber != 0 as DWORD {
        /* Set the 'PortName' value */
        hKey = SetupDiCreateDevRegKeyW(
            DeviceInfoSet,
            DeviceInfoData,
            DICS_FLAG_GLOBAL as DWORD,
            0 as DWORD,
            DIREG_DEV as DWORD,
            core::ptr::null_mut(),
            core::ptr::null::<WCHAR>(),
        );
        if hKey != INVALID_HANDLE_VALUE as HKEY {
            RegSetValueExW(
                hKey,
                RUSTCONV_WSTR_C1D34563AF12.as_ptr(),
                0 as DWORD,
                REG_SZ as DWORD,
                core::ptr::addr_of_mut!(szPortName) as *mut WCHAR as LPBYTE as *const BYTE,
                ((wcslen(core::ptr::addr_of_mut!(szPortName) as *mut WCHAR) as usize
                    + 1 as usize)
                    * core::mem::size_of::<WCHAR>()) as DWORD,
            );

            /*
             * FIXME / HACK:
             * This is to get the w2k3 parport.sys to work until we have our own.
             * This setting makes the driver accept resources with an IRQ instead
             * of only resources without an IRQ.
             *
             * We should probably also fix IO manager to actually give devices a
             * chance to register without an IRQ. CORE-9645
             */
            dwValue = 0 as DWORD;
            RegSetValueExW(
                hKey,
                RUSTCONV_WSTR_6FB7FC12E6B0.as_ptr(),
                0 as DWORD,
                REG_DWORD as DWORD,
                core::ptr::addr_of_mut!(dwValue) as LPBYTE as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );

            RegCloseKey(hKey);
        }
    }

    /* Install the device */
    if SetupDiInstallDevice(DeviceInfoSet, DeviceInfoData) == 0 {
        return GetLastError();
    }

    /* Get the device description... */
    if SetupDiGetDeviceRegistryPropertyW(
        DeviceInfoSet,
        DeviceInfoData,
        SPDRP_DEVICEDESC as DWORD,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(szDeviceDescription) as *mut WCHAR as PBYTE,
        (256 as usize * core::mem::size_of::<WCHAR>()) as DWORD,
        core::ptr::null_mut::<DWORD>(),
    ) != 0
    {
        /* ... and use it to build a new friendly name */
        _swprintf(
            core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR,
            RUSTCONV_WSTR_AF854C3D2274.as_ptr(),
            core::ptr::addr_of_mut!(szDeviceDescription) as *mut WCHAR,
            core::ptr::addr_of_mut!(szPortName) as *mut WCHAR,
        );
    } else {
        /* ... or build a generic friendly name */
        _swprintf(
            core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR,
            RUSTCONV_WSTR_E76BB3C8AF6F.as_ptr(),
            core::ptr::addr_of_mut!(szPortName) as *mut WCHAR,
        );
    }

    if ((*__wine_dbch___default).flags as i32
        & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0
    {
        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl_3) as u32,
            __dbch_3,
            b"dll/win32/msports/classinst.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"InstallParallelPort\0" as *const u8 as *const ::core::ffi::c_char,
            554 as ::core::ffi::c_int,
            b"Friendly name: %S\n\0" as *const u8 as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR,
        );
    }

    /* Set the friendly name for the device */
    SetupDiSetDeviceRegistryPropertyW(
        DeviceInfoSet,
        DeviceInfoData,
        SPDRP_FRIENDLYNAME as DWORD,
        core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR as LPBYTE as *const BYTE,
        ((wcslen(core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR) as usize
            + 1 as usize)
            * core::mem::size_of::<WCHAR>()) as DWORD,
    );

    return ERROR_SUCCESS as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn InstallDeviceData(DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hInf: HINF = INVALID_HANDLE_VALUE;
    let mut DriverInfoData: SP_DRVINFO_DATA = core::mem::zeroed();
    let mut DriverInfoDetailData: PSP_DRVINFO_DETAIL_DATA = core::ptr::null_mut();
    let mut InfSectionWithExt: [WCHAR; 256] = [0; 256];
    let mut buffer: [BYTE; 2048] = [0; 2048];
    let mut dwRequired: DWORD = 0;

    'done: {
        hKey = SetupDiCreateDevRegKeyW(
            DeviceInfoSet,
            DeviceInfoData,
            DICS_FLAG_GLOBAL,
            0,
            DIREG_DRV,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if hKey.is_null() {
            break 'done;
        }

        DriverInfoData.cbSize = core::mem::size_of::<SP_DRVINFO_DATA>() as DWORD;
        if SetupDiGetSelectedDriverW(
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::addr_of_mut!(DriverInfoData),
        ) == 0
        {
            break 'done;
        }

        DriverInfoDetailData = buffer.as_mut_ptr() as PSP_DRVINFO_DETAIL_DATA;
        (*DriverInfoDetailData).cbSize =
            core::mem::size_of::<SP_DRVINFO_DETAIL_DATA>() as DWORD;
        if SetupDiGetDriverInfoDetailW(
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::addr_of_mut!(DriverInfoData),
            DriverInfoDetailData,
            2048,
            core::ptr::addr_of_mut!(dwRequired),
        ) == 0
        {
            if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                break 'done;
            }
        }

        hInf = SetupOpenInfFileW(
            core::ptr::addr_of_mut!((*DriverInfoDetailData).InfFileName) as *mut WCHAR,
            core::ptr::null_mut(),
            INF_STYLE_WIN4,
            core::ptr::null_mut(),
        );
        if hInf == INVALID_HANDLE_VALUE {
            break 'done;
        }

        if SetupDiGetActualSectionToInstallW(
            hInf,
            core::ptr::addr_of_mut!((*DriverInfoDetailData).SectionName) as *mut WCHAR,
            core::ptr::addr_of_mut!(InfSectionWithExt) as *mut WCHAR,
            256,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            break 'done;
        }

        SetupInstallFromInfSectionW(
            core::ptr::null_mut(),
            hInf,
            core::ptr::addr_of_mut!(InfSectionWithExt) as *mut WCHAR,
            SPINST_REGISTRY,
            hKey,
            core::ptr::null_mut(),
            0,
            None,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    if hInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hInf);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetPortType(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> PORT_TYPE {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwSize: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut bData: BYTE = 0;
    let mut PortType: PORT_TYPE = UnknownPort;
    let mut lError: LONG = 0;

    hKey = SetupDiCreateDevRegKeyW(
        DeviceInfoSet,
        DeviceInfoData,
        DICS_FLAG_GLOBAL,
        0,
        DIREG_DRV,
        core::ptr::null_mut(),
        core::ptr::null(),
    );
    'done: {
        if hKey.is_null() {
            break 'done;
        }

        dwSize = core::mem::size_of::<BYTE>() as DWORD;
        lError = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_516C3CC4ECB4.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(bData) as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        );

        if lError == (ERROR_SUCCESS as i32)
            && dwSize as usize == core::mem::size_of::<BYTE>()
            && dwType == REG_BINARY
        {
            if bData == 0 {
                PortType = ParallelPort;
            } else {
                PortType = SerialPort;
            }
        }
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    return PortType;
}

unsafe extern "C" fn InstallPort(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    let mut PortType: PORT_TYPE = UnknownPort;

    InstallDeviceData(DeviceInfoSet, DeviceInfoData);

    PortType = GetPortType(DeviceInfoSet, DeviceInfoData);
    match PortType as ::core::ffi::c_uint {
        1 => return InstallParallelPort(DeviceInfoSet, DeviceInfoData),
        2 => return InstallSerialPort(DeviceInfoSet, DeviceInfoData),
        _ => return ERROR_DI_DO_DEFAULT as DWORD,
    }
}

unsafe extern "C" fn RemovePort(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    let mut PortType: PORT_TYPE = UnknownPort;
    let mut hComDB: HCOMDB = HCOMDB_INVALID_HANDLE_VALUE;
    let mut hKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut lError: LONG = 0;
    let mut dwPortNumber: DWORD = 0;
    let mut dwPortNameSize: DWORD = 0;
    let mut szPortName: [WCHAR; 8] = [0; 8];
    PortType = GetPortType(DeviceInfoSet, DeviceInfoData);
    if PortType as u32 == SerialPort as u32 {
        if ComDBOpen(core::ptr::addr_of_mut!(hComDB)) == ERROR_SUCCESS as LONG {
            hKey = SetupDiOpenDevRegKey(
                DeviceInfoSet,
                DeviceInfoData,
                DICS_FLAG_GLOBAL as DWORD,
                0 as DWORD,
                DIREG_DEV as DWORD,
                KEY_READ as REGSAM,
            );
            if hKey != INVALID_HANDLE_VALUE as HKEY {
                dwPortNameSize = ::core::mem::size_of::<[WCHAR; 8]>() as DWORD;
                lError = RegQueryValueEx(
                    hKey,
                    RUSTCONV_WSTR_C1D34563AF12.as_ptr(),
                    core::ptr::null_mut::<DWORD>(),
                    core::ptr::null_mut::<DWORD>(),
                    core::ptr::addr_of_mut!(szPortName) as *mut WCHAR as LPBYTE,
                    core::ptr::addr_of_mut!(dwPortNameSize),
                ) as LONG;
                RegCloseKey(hKey);
                if lError == ERROR_SUCCESS as LONG {
                    dwPortNumber = _wtoi(
                        (core::ptr::addr_of_mut!(szPortName) as *mut WCHAR)
                            .offset(wcslen(pszCom as *const wchar_t) as isize),
                    ) as DWORD;
                    ComDBReleasePort(hComDB, dwPortNumber);
                }
            }
            ComDBClose(hComDB);
        }
    }
    if SetupDiRemoveDevice(DeviceInfoSet, DeviceInfoData) == 0 {
        return GetLastError();
    }
    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn PortsClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    match InstallFunction {
        DIF_INSTALLDEVICE => {
            return InstallPort(DeviceInfoSet, DeviceInfoData);
        }
        DIF_REMOVE => {
            return RemovePort(DeviceInfoSet, DeviceInfoData);
        }
        _ => {
            TRACE!("Install function %u ignored\n", InstallFunction);
            return ERROR_DI_DO_DEFAULT;
        }
    }
}
