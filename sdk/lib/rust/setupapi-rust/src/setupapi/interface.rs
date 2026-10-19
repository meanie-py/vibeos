// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_485CA664C128: [u16; 13] = [
    0x005Cu16, 0x005C, 0x003F, 0x005C, 0x0025, 0x0073, 0x0023, 0x0025, 0x0073, 0x005C,
    0x0025, 0x0073, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut AddInterface: [WCHAR; 13] = [
    'A' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut ClassGUID: [WCHAR; 10] = [
    'C' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut Control: [WCHAR; 8] = [
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut DeviceInstance: [WCHAR; 15] = [
    'D' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut DotInterfaces: [WCHAR; 12] = [
    '.' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut Linked: [WCHAR; 7] = [
    'L' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut SymbolicLink: [WCHAR; 13] = [
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

unsafe extern "C" fn CreateDeviceInterface(
    deviceInfo: *mut DeviceInfo,
    // The C's parameter is `SymbolicLink`, shadowing the file-static of that name; Rust
    // forbids a parameter shadowing a static, so it is renamed here only.
    SymbolicLink_local: LPCWSTR,
    pInterfaceGuid: LPCGUID,
    pDeviceInterface: *mut *mut DeviceInterface,
) -> BOOL {
    let mut deviceInterface: *mut DeviceInterface;

    *pDeviceInterface = core::ptr::null_mut();

    deviceInterface = HeapAlloc(
        GetProcessHeap(),
        0,
        (FIELD_OFFSET!(DeviceInterface, SymbolicLink) as SIZE_T)
            + ((strlenW(SymbolicLink_local) as SIZE_T) + 1)
                * (core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut DeviceInterface;
    if deviceInterface.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return (FALSE) as BOOL;
    }
    (*deviceInterface).DeviceInfo = deviceInfo;
    strcpyW(
        core::ptr::addr_of_mut!((*deviceInterface).SymbolicLink) as LPWSTR,
        SymbolicLink_local,
    );
    (*deviceInterface).Flags = 0; /* Flags will be updated later */
    memcpy(
        core::ptr::addr_of_mut!((*deviceInterface).InterfaceClassGuid) as *mut core::ffi::c_void,
        pInterfaceGuid as *const core::ffi::c_void,
        (core::mem::size_of::<GUID>() as SIZE_T) as _,
    );

    *pDeviceInterface = deviceInterface;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DestroyDeviceInterface(deviceInterface: *mut DeviceInterface) -> BOOL {
    return HeapFree(GetProcessHeap(), 0, deviceInterface as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn SETUP_CreateInterfaceList(
    list: *mut DeviceInfoSet,
    MachineName: PCWSTR,
    InterfaceGuid: *const GUID,
    DeviceInstanceW: PCWSTR,
    OnlyPresentInterfaces: BOOL,
) -> LONG {
    let mut hInterfaceKey: HKEY = core::ptr::null_mut();
    let mut hDeviceInstanceKey: HKEY = core::ptr::null_mut();
    let mut hReferenceKey: HKEY = core::ptr::null_mut();
    let mut hControlKey: HKEY = core::ptr::null_mut();
    let mut hEnumKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut rc: LONG = 0;
    // `WCHAR KeyBuffer[max(MAX_PATH, MAX_GUID_STRING_LEN) + 1]`: an array length must be a
    // const expression, and win32macros' `max` is a fn.
    let mut KeyBuffer: [WCHAR; (if MAX_PATH > MAX_GUID_STRING_LEN {
        MAX_PATH
    } else {
        MAX_GUID_STRING_LEN
    }) as usize
        + 1] = core::mem::zeroed();
    let mut pSymbolicLink: PWSTR = core::ptr::null_mut();
    let mut InstancePath: PWSTR = core::ptr::null_mut();
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut dwInstancePathLength: DWORD = 0;
    let mut dwRegType: DWORD = 0;
    let mut LinkedValue: DWORD = 0;
    let mut ClassGuid: GUID = core::mem::zeroed();
    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();

    'cleanup: {
        hInterfaceKey = INVALID_HANDLE_VALUE as HKEY;
        hDeviceInstanceKey = core::ptr::null_mut();
        hReferenceKey = core::ptr::null_mut();

        /* Open registry key related to this interface */
        hInterfaceKey = SetupDiOpenClassRegKeyExW(
            InterfaceGuid,
            KEY_ENUMERATE_SUB_KEYS as REGSAM,
            DIOCR_INTERFACE as DWORD,
            MachineName,
            core::ptr::null_mut(),
        );
        if hInterfaceKey == INVALID_HANDLE_VALUE as HKEY {
            /* Key doesn't exist. Let's keep it empty */
            rc = ERROR_SUCCESS as LONG;
            break 'cleanup;
        }

        /* Enumerate sub keys of hInterfaceKey */
        i = 0 as DWORD;
        loop {
            dwLength = (core::mem::size_of_val(&KeyBuffer)
                / core::mem::size_of_val(&KeyBuffer[0 as usize])) as DWORD;
            rc = RegEnumKeyExW(
                hInterfaceKey,
                i,
                &raw mut KeyBuffer as LPWSTR,
                &raw mut dwLength,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as LONG;
            if rc == ERROR_NO_MORE_ITEMS as LONG {
                break;
            }
            if rc != ERROR_SUCCESS as LONG {
                break 'cleanup;
            }
            i = i.wrapping_add(1);

            /* Open sub key */
            if !hDeviceInstanceKey.is_null() {
                RegCloseKey(hDeviceInstanceKey);
            }
            rc = RegOpenKeyExW(
                hInterfaceKey,
                &raw mut KeyBuffer as *mut WCHAR as LPCWSTR,
                0 as DWORD,
                (KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS) as REGSAM,
                &raw mut hDeviceInstanceKey,
            ) as LONG;
            if rc != ERROR_SUCCESS as LONG {
                break 'cleanup;
            }

            /* Read DeviceInstance */
            rc = RegQueryValueExW(
                hDeviceInstanceKey,
                &raw const DeviceInstance as LPCWSTR,
                core::ptr::null_mut(),
                &raw mut dwRegType,
                core::ptr::null_mut(),
                &raw mut dwInstancePathLength,
            ) as LONG;
            if rc != ERROR_SUCCESS as LONG {
                break 'cleanup;
            }
            if dwRegType != REG_SZ as DWORD {
                rc = ERROR_GEN_FAILURE as LONG;
                break 'cleanup;
            }
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                InstancePath as PVOID,
            );
            InstancePath = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                (dwInstancePathLength as SIZE_T)
                    .wrapping_add(::core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            if InstancePath.is_null() {
                rc = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'cleanup;
            }
            rc = RegQueryValueExW(
                hDeviceInstanceKey,
                &raw const DeviceInstance as LPCWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                InstancePath as LPBYTE,
                &raw mut dwInstancePathLength,
            ) as LONG;
            if rc != ERROR_SUCCESS as LONG {
                break 'cleanup;
            }
            *InstancePath.offset(
                ((dwInstancePathLength as usize) / ::core::mem::size_of::<WCHAR>()) as isize,
            ) = '\0' as WCHAR;

            if !DeviceInstanceW.is_null() {
                /* Check if device enumerator is not the right one */
                if strcmpW(DeviceInstanceW, InstancePath as PCWSTR) != 0 as i32 {
                    continue;
                }
            }

            /* Find class GUID associated to the device instance */
            rc = RegOpenKeyExW(
                (*list).HKLM,
                REGSTR_PATH_SYSTEMENUM,
                0 as DWORD, /* Options */
                READ_CONTROL as REGSAM,
                &raw mut hEnumKey,
            ) as LONG;
            if rc != ERROR_SUCCESS as LONG {
                break 'cleanup;
            }
            rc = RegOpenKeyExW(
                hEnumKey,
                InstancePath as LPCWSTR,
                0 as DWORD, /* Options */
                KEY_QUERY_VALUE as REGSAM,
                &raw mut hKey,
            ) as LONG;
            RegCloseKey(hEnumKey);
            if rc != ERROR_SUCCESS as LONG {
                break 'cleanup;
            }
            dwLength = (core::mem::size_of_val(&KeyBuffer) - ::core::mem::size_of::<WCHAR>()) as DWORD;
            rc = RegQueryValueExW(
                hKey,
                &raw const ClassGUID as LPCWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut KeyBuffer as *mut WCHAR as LPBYTE,
                &raw mut dwLength,
            ) as LONG;
            RegCloseKey(hKey);
            if rc != ERROR_SUCCESS as LONG {
                break 'cleanup;
            }
            KeyBuffer[(dwLength as usize) / ::core::mem::size_of::<WCHAR>()] = '\0' as WCHAR;
            KeyBuffer[37 as usize] = '\0' as WCHAR; /* Replace the } by a NULL character */
            if UuidFromStringW(
                (&raw mut KeyBuffer as *mut WCHAR).offset(1 as isize) as RPC_WSTR,
                &raw mut ClassGuid,
            ) != RPC_S_OK as RPC_STATUS
            {
                rc = ERROR_GEN_FAILURE as LONG;
                break 'cleanup;
            }

            /* If current device doesn't match the list GUID (if any), skip this entry */
            // guiddef.h: `#define IsEqualIID(riid1, riid2) IsEqualGUID(riid1, riid2)`.
            if !IsEqualGUID!(&raw const (*list).ClassGuid, &raw const GUID_NULL)
                && !IsEqualGUID!(&raw const (*list).ClassGuid, &raw const ClassGuid)
            {
                continue;
            }

            /* Enumerate subkeys of hDeviceInstanceKey (ie "#ReferenceString" in IoRegisterDeviceInterface). Skip entries that don't start with '#' */
            j = 0 as DWORD;
            loop {
                let mut interfaceInfo: *mut DeviceInterface = core::ptr::null_mut();

                dwLength = (core::mem::size_of_val(&KeyBuffer)
                    / core::mem::size_of_val(&KeyBuffer[0 as usize])) as DWORD;
                rc = RegEnumKeyExW(
                    hDeviceInstanceKey,
                    j,
                    &raw mut KeyBuffer as LPWSTR,
                    &raw mut dwLength,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as LONG;
                if rc == ERROR_NO_MORE_ITEMS as LONG {
                    break;
                }
                if rc != ERROR_SUCCESS as LONG {
                    break 'cleanup;
                }
                j = j.wrapping_add(1);
                if KeyBuffer[0 as usize] as i32 != '#' as i32 {
                    /* This entry doesn't represent an interesting entry */
                    continue;
                }

                /* Open sub key */
                if !hReferenceKey.is_null() {
                    RegCloseKey(hReferenceKey);
                }
                rc = RegOpenKeyExW(
                    hDeviceInstanceKey,
                    &raw mut KeyBuffer as *mut WCHAR as LPCWSTR,
                    0 as DWORD,
                    KEY_QUERY_VALUE as REGSAM,
                    &raw mut hReferenceKey,
                ) as LONG;
                if rc != ERROR_SUCCESS as LONG {
                    break 'cleanup;
                }

                /* Read SymbolicLink value */
                rc = RegQueryValueExW(
                    hReferenceKey,
                    &raw const SymbolicLink as LPCWSTR,
                    core::ptr::null_mut(),
                    &raw mut dwRegType,
                    core::ptr::null_mut(),
                    &raw mut dwLength,
                ) as LONG;
                if rc != ERROR_SUCCESS as LONG {
                    /* Skip device interface with invalid reference value (i.e. interface not actually available for this device) */
                    RegCloseKey(hReferenceKey);
                    // Divergence: the C leaves the closed handle in hReferenceKey, so the
                    // next entry's `if (hReferenceKey != NULL) RegCloseKey(...)` or the
                    // cleanup closes it a second time.
                    hReferenceKey = core::ptr::null_mut();
                    continue;
                }
                if dwRegType != REG_SZ as DWORD {
                    rc = ERROR_GEN_FAILURE as LONG;
                    break 'cleanup;
                }

                /* We have found a device */
                /* Step 1. Create a device info element */
                if CreateDeviceInfo(
                    list,
                    InstancePath as LPCWSTR,
                    &raw mut ClassGuid as LPCGUID,
                    &raw mut deviceInfo,
                ) == 0
                {
                    rc = GetLastError() as LONG;
                    break 'cleanup;
                }
                InsertTailList(
                    &raw mut (*list).ListHead,
                    &raw mut (*deviceInfo).ListEntry,
                );

                /* Step 2. Create an interface list for this element */
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    pSymbolicLink as PVOID,
                );
                pSymbolicLink = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    (dwLength as SIZE_T).wrapping_add(::core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as PWSTR;
                if pSymbolicLink.is_null() {
                    rc = ERROR_NOT_ENOUGH_MEMORY as LONG;
                    break 'cleanup;
                }
                rc = RegQueryValueExW(
                    hReferenceKey,
                    &raw const SymbolicLink as LPCWSTR,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    pSymbolicLink as LPBYTE,
                    &raw mut dwLength,
                ) as LONG;
                *pSymbolicLink.offset(
                    ((dwLength as usize) / ::core::mem::size_of::<WCHAR>()) as isize,
                ) = '\0' as WCHAR;
                if rc != ERROR_SUCCESS as LONG {
                    break 'cleanup;
                }
                if CreateDeviceInterface(
                    deviceInfo,
                    pSymbolicLink as LPCWSTR,
                    InterfaceGuid as LPCGUID,
                    &raw mut interfaceInfo,
                ) == 0
                {
                    rc = GetLastError() as LONG;
                    break 'cleanup;
                }

                /* Step 3. Update flags */
                if KeyBuffer[1 as usize] as i32 == '\0' as i32 {
                    (*interfaceInfo).Flags |= SPINT_DEFAULT as DWORD;
                }
                rc = RegOpenKeyExW(
                    hReferenceKey,
                    &raw const Control as LPCWSTR,
                    0 as DWORD,
                    KEY_QUERY_VALUE as REGSAM,
                    &raw mut hControlKey,
                ) as LONG;
                if rc != ERROR_SUCCESS as LONG {
                } else {
                    dwLength = ::core::mem::size_of::<DWORD>() as DWORD;
                    if RegQueryValueExW(
                        hControlKey,
                        &raw const Linked as LPCWSTR,
                        core::ptr::null_mut(),
                        &raw mut dwRegType,
                        &raw mut LinkedValue as LPBYTE,
                        &raw mut dwLength,
                    ) == ERROR_SUCCESS as LSTATUS
                        && dwRegType == REG_DWORD as DWORD
                        && LinkedValue != 0
                    {
                        (*interfaceInfo).Flags |= SPINT_ACTIVE as DWORD;
                    }
                    RegCloseKey(hControlKey);
                }

                InsertTailList(
                    &raw mut (*deviceInfo).InterfaceListHead,
                    &raw mut (*interfaceInfo).ListEntry,
                );
            }
        }
        rc = ERROR_SUCCESS as LONG;
    }

    /* cleanup: */
    if !hReferenceKey.is_null() {
        RegCloseKey(hReferenceKey);
    }
    if !hDeviceInstanceKey.is_null() {
        RegCloseKey(hDeviceInstanceKey);
    }
    if hInterfaceKey != INVALID_HANDLE_VALUE as HKEY {
        RegCloseKey(hInterfaceKey);
    }
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        InstancePath as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pSymbolicLink as PVOID,
    );
    return rc;
}

unsafe extern "C" fn CreateSymbolicLink(
    InterfaceGuid: LPGUID,
    ReferenceString: LPCWSTR,
    devInfo: *mut DeviceInfo,
) -> LPWSTR {
    let mut ActualLength: SIZE_T = 0;
    let mut Length: SIZE_T = 0;
    let mut GuidString: [WCHAR; (MAX_GUID_STRING_LEN as _)] = [0; (MAX_GUID_STRING_LEN as _)];
    let mut SymbolicLink_local: LPWSTR = core::ptr::null_mut();

    Length = (4 as SIZE_T)
        + wcslen((*devInfo).instanceId) as SIZE_T
        + (1 as SIZE_T)
        + ((MAX_GUID_STRING_LEN - 1) as SIZE_T)
        + (1 as SIZE_T)
        + wcslen(ReferenceString) as SIZE_T
        + (1 as SIZE_T);

    SymbolicLink_local = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        Length * (core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if SymbolicLink_local.is_null() {
        return core::ptr::null_mut();
    }

    pSetupStringFromGuid(
        InterfaceGuid,
        core::ptr::addr_of_mut!(GuidString) as PWSTR,
        ARRAYSIZE!(GuidString) as DWORD,
    );

    ActualLength = swprintf(
        SymbolicLink_local as *mut WCHAR,
        Length as size_t,
        RUSTCONV_WSTR_485CA664C128.as_ptr(),
        (*devInfo).instanceId,
        core::ptr::addr_of_mut!(GuidString) as *mut WCHAR,
        ReferenceString,
    ) as SIZE_T;
    ASSERT!(ActualLength == Length - (1 as SIZE_T));

    return SymbolicLink_local;
}

unsafe extern "C" fn InstallOneInterface(
    mut InterfaceGuid: LPGUID,
    mut ReferenceString: LPCWSTR,
    mut InterfaceSection: LPCWSTR,
    mut InterfaceFlags: UINT,
    mut hInf: HINF,
    mut DeviceInfoSet: HDEVINFO,
    mut devInfo: *mut DeviceInfo,
) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Path: LPWSTR = core::ptr::null_mut();
    let mut DeviceInterfaceData: SP_DEVICE_INTERFACE_DATA = core::mem::zeroed();
    let mut DevItf: *mut DeviceInterface = core::ptr::null_mut();

    if InterfaceFlags != 0 as UINT {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    Path = CreateSymbolicLink(InterfaceGuid, ReferenceString, devInfo);
    if Path.is_null() {
        return (FALSE) as BOOL;
    }

    CreateDeviceInterface(
        devInfo,
        Path as LPCWSTR,
        InterfaceGuid as LPCGUID,
        core::ptr::addr_of_mut!(DevItf),
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        Path as PVOID,
    );
    if DevItf.is_null() {
        return (FALSE) as BOOL;
    }

    memcpy(
        core::ptr::addr_of_mut!(DeviceInterfaceData.InterfaceClassGuid) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*DevItf).InterfaceClassGuid) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as usize,
    );
    DeviceInterfaceData.cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;
    DeviceInterfaceData.Flags = (*DevItf).Flags;
    DeviceInterfaceData.Reserved = DevItf as ULONG_PTR;

    hKey = SetupDiCreateDeviceInterfaceRegKeyW(
        DeviceInfoSet,
        core::ptr::addr_of_mut!(DeviceInterfaceData),
        0 as DWORD,
        KEY_ALL_ACCESS as REGSAM,
        core::ptr::null_mut(),
        core::ptr::null(),
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        DevItf as PVOID,
    );
    if hKey == INVALID_HANDLE_VALUE as HKEY {
        return (FALSE) as BOOL;
    }

    // Divergence: the C returns straight from SetupInstallFromInfSectionW and never closes
    // the key SetupDiCreateDeviceInterfaceRegKeyW opened for it, leaking one per interface.
    let ret = SetupInstallFromInfSectionW(
        core::ptr::null_mut(),
        hInf,
        InterfaceSection as PCWSTR,
        SPINST_REGISTRY as UINT,
        hKey,
        core::ptr::null(),
        0 as UINT,
        None,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    RegCloseKey(hKey);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiInstallDeviceInterfaces(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if DeviceInfoSet == INVALID_HANDLE_VALUE as HDEVINFO {
        SetLastError(ERROR_INVALID_HANDLE);
    } else {
        list = DeviceInfoSet as *mut DeviceInfoSet;
        if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC {
            SetLastError(ERROR_INVALID_HANDLE);
        } else if DeviceInfoData.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
        } else if !DeviceInfoData.is_null() && (*DeviceInfoData).Reserved == 0 {
            SetLastError(ERROR_INVALID_USER_BUFFER);
        } else if !DeviceInfoData.is_null()
            && (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>()
        {
            SetLastError(ERROR_INVALID_USER_BUFFER);
        } else {
            let mut devInfo: *mut DeviceInfo = core::ptr::null_mut();
            let mut SelectedDriver: *mut DriverInfoElement = core::ptr::null_mut();
            let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
            let mut SectionName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut SectionNameLength: DWORD = 0;
            let mut ContextInterface: INFCONTEXT = core::mem::zeroed();
            let mut InterfaceGuidString: LPWSTR = core::ptr::null_mut();
            let mut ReferenceString: LPWSTR = core::ptr::null_mut();
            let mut InterfaceSection: LPWSTR = core::ptr::null_mut();
            let mut InterfaceFlags: INT = 0;
            let mut InterfaceGuid: GUID = core::mem::zeroed();
            let mut Result: BOOL = 0;

            let mut at: u8 = 0;
            loop {
                match at {
                    0 => {
                        devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;

                        InstallParams.cbSize =
                            core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
                        Result = SetupDiGetDeviceInstallParamsW(
                            DeviceInfoSet,
                            DeviceInfoData,
                            core::ptr::addr_of_mut!(InstallParams),
                        );
                        if Result == 0 {
                            at = 1;
                            continue;
                        }

                        SelectedDriver =
                            InstallParams.ClassInstallReserved as *mut DriverInfoElement;
                        if SelectedDriver.is_null() {
                            SetLastError(ERROR_NO_DRIVER_SELECTED);
                            ret = (FALSE as i32);
                            at = 1;
                            continue;
                        }

                        /* Get .Interfaces section name */
                        Result = SetupDiGetActualSectionToInstallW(
                            (*(*SelectedDriver).InfFileDetails).hInf,
                            core::ptr::addr_of!((*SelectedDriver).Details.SectionName) as PCWSTR,
                            core::ptr::addr_of_mut!(SectionName) as PWSTR,
                            MAX_PATH,
                            core::ptr::addr_of_mut!(SectionNameLength),
                            core::ptr::null_mut(),
                        );
                        if Result == 0
                            || SectionNameLength as usize
                                > (MAX_PATH as usize)
                                    - strlenW(core::ptr::addr_of!(DotInterfaces) as PCWSTR)
                                    - (1 as usize)
                        {
                            at = 1;
                            continue;
                        }
                        strcatW(
                            core::ptr::addr_of_mut!(SectionName) as PWSTR,
                            core::ptr::addr_of!(DotInterfaces) as PCWSTR,
                        );

                        ret = (TRUE as i32);
                        Result = SetupFindFirstLineW(
                            (*(*SelectedDriver).InfFileDetails).hInf,
                            core::ptr::addr_of!(SectionName) as PCWSTR,
                            core::ptr::addr_of!(AddInterface) as PCWSTR,
                            core::ptr::addr_of_mut!(ContextInterface),
                        );
                        at = 2;
                        continue;
                    }
                    1 => {
                        /* cleanup: */
                        MyFree(InterfaceGuidString as PVOID);
                        if !ReferenceString.is_null() {
                            MyFree(ReferenceString as PVOID);
                        }
                        MyFree(InterfaceSection as PVOID);
                        InterfaceGuidString = core::ptr::null_mut();
                        ReferenceString = InterfaceGuidString;
                        InterfaceSection = ReferenceString;
                        Result = SetupFindNextMatchLineW(
                            core::ptr::addr_of_mut!(ContextInterface),
                            core::ptr::addr_of!(AddInterface) as PCWSTR,
                            core::ptr::addr_of_mut!(ContextInterface),
                        );
                        at = 2;
                        continue;
                    }
                    2 => {
                        /* while (ret && Result) */
                        if !(ret != 0 && Result != 0) {
                            break;
                        }

                        ret = GetStringField(
                            core::ptr::addr_of_mut!(ContextInterface),
                            1,
                            core::ptr::addr_of_mut!(InterfaceGuidString),
                        );
                        if ret == 0 {
                            at = 1;
                            continue;
                        } else if strlenW(InterfaceGuidString as PCWSTR)
                            != (MAX_GUID_STRING_LEN - 1) as usize
                        {
                            SetLastError(ERROR_INVALID_PARAMETER);
                            ret = (FALSE as i32);
                            at = 1;
                            continue;
                        }

                        *InterfaceGuidString
                            .offset((MAX_GUID_STRING_LEN - 2) as isize) =
                            0 as WCHAR; /* Replace the } by a NULL character */
                        if UuidFromStringW(
                            InterfaceGuidString.offset(1 as isize),
                            core::ptr::addr_of_mut!(InterfaceGuid),
                        ) != RPC_S_OK
                        {
                            /* Bad GUID, skip the entry */
                            SetLastError(ERROR_INVALID_PARAMETER);
                            ret = (FALSE as i32);
                            at = 1;
                            continue;
                        }

                        ret = GetStringField(
                            core::ptr::addr_of_mut!(ContextInterface),
                            2,
                            core::ptr::addr_of_mut!(ReferenceString),
                        );
                        if ret == 0 {
                            at = 1;
                            continue;
                        }

                        ret = GetStringField(
                            core::ptr::addr_of_mut!(ContextInterface),
                            3,
                            core::ptr::addr_of_mut!(InterfaceSection),
                        );
                        if ret == 0 {
                            /* ReferenceString is optional */
                            InterfaceSection = ReferenceString;
                            ReferenceString = core::ptr::null_mut();
                        }

                        ret = SetupGetIntField(
                            core::ptr::addr_of_mut!(ContextInterface),
                            (if !ReferenceString.is_null() {
                                4
                            } else {
                                3
                            }) as DWORD, /* Field index */
                            core::ptr::addr_of_mut!(InterfaceFlags),
                        );
                        if ret == 0 {
                            if GetLastError() == ERROR_INVALID_PARAMETER {
                                /* The field may be empty. Ignore the error */
                                InterfaceFlags = 0;
                                ret = (TRUE as i32);
                            } else {
                                at = 1;
                                continue;
                            }
                        }

                        /* Install Interface */
                        ret = InstallOneInterface(
                            core::ptr::addr_of_mut!(InterfaceGuid),
                            ReferenceString as LPCWSTR,
                            InterfaceSection as LPCWSTR,
                            InterfaceFlags as UINT,
                            (*(*SelectedDriver).InfFileDetails).hInf,
                            DeviceInfoSet,
                            devInfo,
                        );
                        at = 1;
                        continue;
                    }
                    _ => break,
                }
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiOpenDeviceInterfaceRegKey(
    DeviceInfoSet: HDEVINFO,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
    Reserved: DWORD,
    samDesired: REGSAM,
) -> HKEY {
    let mut hKey: HKEY = INVALID_HANDLE_VALUE as HKEY;
    let mut hDevKey: HKEY = core::ptr::null_mut();
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if DeviceInfoSet == INVALID_HANDLE_VALUE as HDEVINFO {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if {
        list = DeviceInfoSet as *mut DeviceInfoSet;
        (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD
    } {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if DeviceInterfaceData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if !DeviceInterfaceData.is_null()
        && (*DeviceInterfaceData).Reserved == 0 as ULONG_PTR
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if !DeviceInterfaceData.is_null()
        && (*DeviceInterfaceData).cbSize as usize
            != core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>()
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        let mut DevItf: *mut DeviceInterface = core::ptr::null_mut();
        let mut Path: LPWSTR = core::ptr::null_mut();
        let mut Guid: LPWSTR = core::ptr::null_mut();
        let mut Slash: LPWSTR = core::ptr::null_mut();
        let mut Length: DWORD = 0;

        DevItf = (*DeviceInterfaceData).Reserved as *mut DeviceInterface;

        Length = wcslen(core::ptr::addr_of_mut!((*DevItf).SymbolicLink) as *mut WCHAR) as DWORD;

        Path = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            ((Length + 2 as DWORD) as SIZE_T)
                * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as LPWSTR;
        if Path.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return INVALID_HANDLE_VALUE as HKEY;
        }

        wcscpy(
            Path,
            core::ptr::addr_of_mut!((*DevItf).SymbolicLink) as *mut WCHAR,
        );

        Guid = wcsrchr(Path, '}' as WCHAR) as LPWSTR;
        Slash = wcsrchr(Path, '\\' as WCHAR) as LPWSTR;
        if Guid.is_null() || Slash.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                Path as PVOID,
            );
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return INVALID_HANDLE_VALUE as HKEY;
        }

        if (Slash as ULONG_PTR) > (Guid as ULONG_PTR) {
            /* Create an extra slash */
            memmove(
                Slash.offset(1 as isize) as *mut core::ffi::c_void,
                Slash as *const core::ffi::c_void,
                (((wcslen(Slash as *const WCHAR) as SIZE_T)
                    + 1 as SIZE_T)
                    * core::mem::size_of::<WCHAR>() as SIZE_T) as _,
            );
            *Slash.offset(1 as isize) = 0x0023u16;
        }

        Guid = Path;
        while (Guid as ULONG_PTR) < (Slash as ULONG_PTR) {
            if *Guid == 0x005Cu16 {
                *Guid = 0x0023u16;
            }

            Guid = Guid.offset(1);
        }

        hKey = SetupDiOpenClassRegKeyExW(
            core::ptr::addr_of_mut!((*DeviceInterfaceData).InterfaceClassGuid),
            samDesired,
            DIOCR_INTERFACE as DWORD,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if hKey != INVALID_HANDLE_VALUE as HKEY {
            if RegOpenKeyExW(
                hKey,
                Path as LPCWSTR,
                0 as DWORD,
                samDesired,
                core::ptr::addr_of_mut!(hDevKey),
            ) == ERROR_SUCCESS as LSTATUS
            {
                RegCloseKey(hKey);
                hKey = hDevKey;
            } else {
                RegCloseKey(hKey);
                hKey = INVALID_HANDLE_VALUE as HKEY;
            }
        }

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            Path as PVOID,
        );
    }

    return hKey;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiDeleteDeviceInterfaceData(
    DeviceInfoSet: HDEVINFO,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
) -> BOOL {
    // FIXME("SetupDiDeleteDeviceInterfaceData(%p %p) stub\n", DeviceInfoSet, DeviceInterfaceData);
    (TRUE as i32)
}
