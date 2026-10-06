use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn AppendStringToMultiSZ(
    hKey: HKEY,
    ValueName: PCWSTR,
    ValueToAppend: PCWSTR,
) -> i32 {
    let mut Buffer: PWSTR = core::ptr::null_mut();
    let mut dwRegType: u32 = 0;
    let mut dwRequired: u32 = 0;
    let mut dwLength: u32;
    let mut dwTmp: u32;
    let mut rc: i32;

    rc = RegQueryValueExW(
        hKey,
        ValueName,
        core::ptr::null_mut(),
        &mut dwRegType,
        core::ptr::null_mut(),
        &mut dwRequired,
    );
    if rc != (ERROR_FILE_NOT_FOUND) as i32 {
        if rc != (ERROR_SUCCESS) as i32 {
            '__cleanup: {
                HeapFree(GetProcessHeap(), 0, Buffer as *mut _);
                return rc;
            }
        }
        if dwRegType != (REG_MULTI_SZ) as u32 {
            rc = (ERROR_GEN_FAILURE) as i32;
            '__cleanup: {
                HeapFree(GetProcessHeap(), 0, Buffer as *mut _);
                return rc;
            }
        }

        dwTmp = dwRequired + (wcslen(ValueToAppend) * (core::mem::size_of::<u16>() as u32) as usize) as u32 + core::mem::size_of_val(&UNICODE_NULL) as u32;
        dwLength = dwTmp;
        Buffer = HeapAlloc(GetProcessHeap(), 0, (dwLength) as SIZE_T) as PWSTR;
        if Buffer.is_null() {
            rc = (ERROR_NOT_ENOUGH_MEMORY) as i32;
            '__cleanup: {
                HeapFree(GetProcessHeap(), 0, Buffer as *mut _);
                return rc;
            }
        }

        rc = RegQueryValueExW(
            hKey,
            ValueName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Buffer as *mut u8,
            &mut dwTmp,
        );
        if rc != (ERROR_SUCCESS) as i32 {
            '__cleanup: {
                HeapFree(GetProcessHeap(), 0, Buffer as *mut _);
                return rc;
            }
        }
    } else {
        dwRequired = core::mem::size_of::<u16>() as u32;
        dwLength = (wcslen(ValueToAppend) * (core::mem::size_of::<u16>() as u32) as usize + (2 * core::mem::size_of_val(&UNICODE_NULL) as u32) as usize) as u32;
        Buffer = HeapAlloc(GetProcessHeap(), 0, (dwLength) as SIZE_T) as PWSTR;
        if Buffer.is_null() {
            rc = (ERROR_NOT_ENOUGH_MEMORY) as i32;
            '__cleanup: {
                HeapFree(GetProcessHeap(), 0, Buffer as *mut _);
                return rc;
            }
        }
    }

    wcscpy(
        Buffer.offset((dwRequired / core::mem::size_of::<u16>() as u32 - 1) as isize),
        ValueToAppend,
    );
    *Buffer.offset((dwLength / core::mem::size_of::<u16>() as u32 - 1) as isize) = UNICODE_NULL;

    rc = RegSetValueExW(
        hKey,
        ValueName,
        0,
        (REG_MULTI_SZ) as u32,
        Buffer as *const u8,
        dwLength,
    );

    HeapFree(GetProcessHeap(), 0, Buffer as *mut _);
    return rc;
}
unsafe extern "C" fn GetUniqueConnectionName(
    hNetworkKey: HKEY,
    ppszNameBuffer: *mut PWSTR,
) -> u32 {
    let mut Length: i32 = 0;
    let mut pszDefaultName: PWSTR = core::ptr::null_mut();
    let mut pszNameBuffer: PWSTR;
    let mut dwSubKeys: u32 = 0;
    let dwError: u32;

    Length = LoadStringW(
        netcfgx_hInstance,
        (IDS_NET_CONNECT) as u32,
        &pszDefaultName as *const PWSTR as LPWSTR,
        0,
    );
    if Length == 0 {
        pszDefaultName = w!("Network Connection") as PWSTR;
        Length = (wcslen(pszDefaultName)) as i32;
    }

    dwError = (RegQueryInfoKeyW(
        hNetworkKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwSubKeys,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    pszNameBuffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,((Length + if dwSubKeys != 0 {
            6
        } else {
            1
        }) as usize * core::mem::size_of::<u16>()) as SIZE_T,
    ) as PWSTR;
    if pszNameBuffer.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    if dwSubKeys != 0 {
        StringCchPrintfW!(
            pszNameBuffer,
            (Length + 6) as usize,
            w!("%.*s %lu") as LPCWSTR,
            Length,
            pszDefaultName,
            dwSubKeys + 1,
        );
    } else {
        StringCchPrintfW!(
            pszNameBuffer,
            (Length + 1) as usize,
            w!("%.*s") as LPCWSTR,
            Length,
            pszDefaultName,
        );
    }

    *ppszNameBuffer = pszNameBuffer;

    return ERROR_SUCCESS;
}
unsafe extern "C" fn InstallNetDevice(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    UuidString: LPCWSTR,
    Characteristics: u32,
    BusType: LPCWSTR,
) -> u32 {
    let mut DeviceInstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
    let mut InstanceId: LPWSTR = core::ptr::null_mut();
    let mut ComponentId: LPWSTR = core::ptr::null_mut();
    let mut DeviceName: LPWSTR = core::ptr::null_mut();
    let mut ExportName: LPWSTR = core::ptr::null_mut();
    let mut rc: i32;
    let mut hInterfacesKey: HKEY = core::ptr::null_mut();
    let mut hInterfaceKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hNetworkKey: HKEY = core::ptr::null_mut();
    let mut hLinkageKey: HKEY = core::ptr::null_mut();
    let mut hConnectionKey: HKEY = core::ptr::null_mut();
    let mut dwShowIcon: u32;
    let mut dwLength: u32 = 0;
    let mut dwValue: u32;
    let mut pszNameBuffer: PWSTR = core::ptr::null_mut();
    let mut dwRegType: u32 = 0;

    'cleanup: loop {
        DeviceInstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as u32;
        if SetupDiGetDeviceInstallParamsW(DeviceInfoSet, DeviceInfoData, &mut DeviceInstallParams) == 0 {
            rc = GetLastError() as i32;
            break 'cleanup;
        }

        DeviceInstallParams.Flags |= (DI_DONOTCALLCONFIGMG) as u32;

        if SetupDiSetDeviceInstallParamsW(DeviceInfoSet, DeviceInfoData, &mut DeviceInstallParams) == 0 {
            rc = GetLastError() as i32;
            break 'cleanup;
        }

        if SetupDiInstallDevice(DeviceInfoSet, DeviceInfoData) == 0 {
            rc = GetLastError() as i32;
            break 'cleanup;
        }

        if SetupDiGetDeviceInstanceIdW(DeviceInfoSet, DeviceInfoData, core::ptr::null_mut(), 0, &mut dwLength) != 0 {
            rc = (ERROR_GEN_FAILURE) as i32;
            break 'cleanup;
        }

        InstanceId = HeapAlloc(GetProcessHeap(), 0,(dwLength as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if InstanceId.is_null() {
            rc = (ERROR_NOT_ENOUGH_MEMORY) as i32;
            break 'cleanup;
        }

        if SetupDiGetDeviceInstanceIdW(DeviceInfoSet, DeviceInfoData, InstanceId, dwLength, core::ptr::null_mut()) == 0 {
            rc = GetLastError() as i32;
            break 'cleanup;
        }

        DeviceName = HeapAlloc(GetProcessHeap(), 0,((wcslen(w!("\\Device\\") as LPCWSTR) + wcslen(UuidString)) * core::mem::size_of::<u16>() + core::mem::size_of_val(&UNICODE_NULL)) as SIZE_T) as LPWSTR;
        if DeviceName.is_null() {
            rc = (ERROR_NOT_ENOUGH_MEMORY) as i32;
            break 'cleanup;
        }
        wcscpy(DeviceName, w!("\\Device\\") as LPCWSTR);
        wcscat(DeviceName, UuidString);

        ExportName = HeapAlloc(GetProcessHeap(), 0,((wcslen(w!("\\Device\\Tcpip_") as LPCWSTR) + wcslen(UuidString)) * core::mem::size_of::<u16>() + core::mem::size_of_val(&UNICODE_NULL)) as SIZE_T) as LPWSTR;
        if ExportName.is_null() {
            rc = (ERROR_NOT_ENOUGH_MEMORY) as i32;
            break 'cleanup;
        }
        wcscpy(ExportName, w!("\\Device\\Tcpip_") as LPCWSTR);
        wcscat(ExportName, UuidString);

        rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces") as LPCWSTR, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_WRITE, core::ptr::null_mut(), &mut hInterfacesKey, core::ptr::null_mut());
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegCreateKeyExW(hInterfacesKey, UuidString, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_WRITE, core::ptr::null_mut(), &mut hInterfaceKey, core::ptr::null_mut());
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hInterfaceKey, w!("DefaultGateway") as LPCWSTR, 0, (REG_SZ) as u32, w!("0.0.0.0") as *const BYTE, ((wcslen(w!("0.0.0.0") as LPCWSTR) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hInterfaceKey, w!("IPAddress") as LPCWSTR, 0, (REG_SZ) as u32, w!("0.0.0.0") as *const BYTE, ((wcslen(w!("0.0.0.0") as LPCWSTR) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hInterfaceKey, w!("SubnetMask") as LPCWSTR, 0, (REG_SZ) as u32, w!("0.0.0.0") as *const BYTE, ((wcslen(w!("0.0.0.0") as LPCWSTR) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        dwValue = 1;
        rc = RegSetValueExW(hInterfaceKey, w!("EnableDHCP") as LPCWSTR, 0, (REG_DWORD) as u32, &dwValue as *const _ as *const u8, core::mem::size_of::<u32>() as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        hKey = SetupDiOpenDevRegKey(DeviceInfoSet, DeviceInfoData, (DICS_FLAG_GLOBAL) as u32, 0, (DIREG_DRV) as u32, KEY_READ | KEY_WRITE);
        if hKey == (INVALID_HANDLE_VALUE).cast() && GetLastError() == ERROR_FILE_NOT_FOUND {
            hKey = SetupDiCreateDevRegKeyW(DeviceInfoSet, DeviceInfoData, (DICS_FLAG_GLOBAL) as u32, 0, (DIREG_DRV) as u32, core::ptr::null_mut(), core::ptr::null_mut());
        }
        if hKey == (INVALID_HANDLE_VALUE).cast() {
            hKey = core::ptr::null_mut();
            rc = GetLastError() as i32;
            break 'cleanup;
        }

        rc = RegSetValueExW(hKey, w!("NetCfgInstanceId") as LPCWSTR, 0, (REG_SZ) as u32, UuidString as *const u8, ((wcslen(UuidString) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hKey, w!("Characteristics") as LPCWSTR, 0, (REG_DWORD) as u32, &Characteristics as *const _ as *const u8, core::mem::size_of::<u32>() as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        /* Get ComponentId (from MatchingDeviceId value) */
        rc = RegQueryValueExW(hKey, w!("MatchingDeviceId") as LPCWSTR, core::ptr::null_mut(), &mut dwRegType, core::ptr::null_mut(), &mut dwLength);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }
        if dwRegType != (REG_SZ) as u32 {
            rc = (ERROR_GEN_FAILURE) as i32;
            break 'cleanup;
        }
        ComponentId = HeapAlloc(GetProcessHeap(), 0, dwLength as SIZE_T) as LPWSTR;
        if ComponentId.is_null() {
            rc = (ERROR_NOT_ENOUGH_MEMORY) as i32;
            break 'cleanup;
        }
        rc = RegQueryValueExW(hKey, w!("MatchingDeviceId") as LPCWSTR, core::ptr::null_mut(), &mut dwRegType, ComponentId as *mut BYTE, &mut dwLength);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hKey, w!("ComponentId") as LPCWSTR, 0, (REG_SZ) as u32, ComponentId as *const u8, dwLength);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        if !BusType.is_null() {
            rc = RegSetValueExW(hKey, w!("BusType") as LPCWSTR, 0, (REG_SZ) as u32, BusType as *const u8, ((wcslen(BusType) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
            if rc != (ERROR_SUCCESS) as i32 {
                break 'cleanup;
            }
        }

        rc = RegCreateKeyExW(hKey, w!("Linkage") as LPCWSTR, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, core::ptr::null_mut(), &mut hLinkageKey, core::ptr::null_mut());
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hLinkageKey, w!("Export") as LPCWSTR, 0, (REG_SZ) as u32, DeviceName as *const u8, ((wcslen(DeviceName) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hLinkageKey, w!("RootDevice") as LPCWSTR, 0, (REG_SZ) as u32, UuidString as *const u8, ((wcslen(UuidString) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hLinkageKey, w!("UpperBind") as LPCWSTR, 0, (REG_SZ) as u32, w!("Tcpip") as *const BYTE, ((wcslen(w!("Tcpip") as LPCWSTR) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }
        RegCloseKey(hKey);
        hKey = core::ptr::null_mut();

        rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}") as LPCWSTR, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_CREATE_SUB_KEY | KEY_QUERY_VALUE, core::ptr::null_mut(), &mut hNetworkKey, core::ptr::null_mut());
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = (GetUniqueConnectionName(hNetworkKey, &mut pszNameBuffer)) as i32;
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegCreateKeyExW(hNetworkKey, UuidString, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_CREATE_SUB_KEY, core::ptr::null_mut(), &mut hKey, core::ptr::null_mut());
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegCreateKeyExW(hKey, w!("Connection") as LPCWSTR, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, core::ptr::null_mut(), &mut hConnectionKey, core::ptr::null_mut());
        RegCloseKey(hKey);
        hKey = core::ptr::null_mut();
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hConnectionKey, w!("Name") as LPCWSTR, 0, (REG_SZ) as u32, pszNameBuffer as *const u8, ((wcslen(pszNameBuffer) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegSetValueExW(hConnectionKey, w!("PnpInstanceID") as LPCWSTR, 0, (REG_SZ) as u32, InstanceId as *const u8, ((wcslen(InstanceId) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        dwShowIcon = 1;
        rc = RegSetValueExW(hConnectionKey, w!("ShowIcon") as LPCWSTR, 0, (REG_DWORD) as u32, &dwShowIcon as *const _ as *const u8, core::mem::size_of_val(&dwShowIcon) as u32);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Linkage") as LPCWSTR, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_QUERY_VALUE | KEY_SET_VALUE, core::ptr::null_mut(), &mut hKey, core::ptr::null_mut());
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }
        rc = AppendStringToMultiSZ(hKey, w!("Bind") as PCWSTR, DeviceName);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }
        rc = AppendStringToMultiSZ(hKey, w!("Export") as PCWSTR, ExportName);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }
        rc = AppendStringToMultiSZ(hKey, w!("Route") as PCWSTR, UuidString);
        if rc != (ERROR_SUCCESS) as i32 {
            break 'cleanup;
        }

        if SetupDiRestartDevices(DeviceInfoSet, DeviceInfoData) == 0 {
            rc = GetLastError() as i32;
            break 'cleanup;
        }

        rc = (ERROR_SUCCESS) as i32;
        break;
    }

    HeapFree(GetProcessHeap(), 0, pszNameBuffer as *mut _);
    HeapFree(GetProcessHeap(), 0, InstanceId as *mut _);
    HeapFree(GetProcessHeap(), 0, ComponentId as *mut _);
    HeapFree(GetProcessHeap(), 0, DeviceName as *mut _);
    HeapFree(GetProcessHeap(), 0, ExportName as *mut _);
    if !hInterfaceKey.is_null() {
        RegCloseKey(hInterfaceKey);
    }
    if !hInterfacesKey.is_null() {
        RegCloseKey(hInterfacesKey);
    }
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    if !hNetworkKey.is_null() {
        RegCloseKey(hNetworkKey);
    }
    if !hLinkageKey.is_null() {
        RegCloseKey(hLinkageKey);
    }
    if !hConnectionKey.is_null() {
        RegCloseKey(hConnectionKey);
    }

    rc as u32
}

unsafe extern "C" fn InstallNetClient() -> DWORD {
    // FIXME("Installation of network clients is not yet supported\n");
    ERROR_GEN_FAILURE
}

unsafe extern "C" fn InstallNetService() -> DWORD {
    // FIXME("Installation of network services is not yet supported\n");
    ERROR_GEN_FAILURE
}

unsafe extern "C" fn InstallNetTransport() -> DWORD {
    // FIXME("Installation of network protocols is not yet supported\n");
    ERROR_GEN_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NetClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    let mut DriverInfoData: SP_DRVINFO_DATA_W = core::mem::zeroed();
    let mut DriverInfoDetail: SP_DRVINFO_DETAIL_DATA_W = core::mem::zeroed();
    let mut SectionName: [u16; (LINE_LEN) as usize] = [0; (LINE_LEN) as usize];
    let mut hInf: HINF = INVALID_HANDLE_VALUE;
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut ErrorLine: u32 = 0;
    let mut CharacteristicsInt: i32 = 0;
    let mut Characteristics: u32 = 0;
    let mut BusType: LPWSTR = core::ptr::null_mut();
    let mut RpcStatus: RPC_STATUS = 0;
    let mut Uuid: UUID = core::mem::zeroed();
    let mut UuidRpcString: LPWSTR = core::ptr::null_mut();
    let mut UuidString: LPWSTR = core::ptr::null_mut();
    let mut rc: i32 = 0;
    let mut dwLength: u32 = 0;

    if InstallFunction != (DIF_INSTALLDEVICE) as u32 {
        return (ERROR_DI_DO_DEFAULT) as u32;
    }

    DriverInfoData.cbSize = core::mem::size_of::<SP_DRVINFO_DATA_W>() as u32;
    if SetupDiGetSelectedDriverW(DeviceInfoSet, DeviceInfoData, &mut DriverInfoData) == 0 {
        rc = GetLastError() as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    DriverInfoDetail.cbSize = core::mem::size_of::<SP_DRVINFO_DETAIL_DATA_W>() as u32;
    if SetupDiGetDriverInfoDetailW(
        DeviceInfoSet,
        DeviceInfoData,
        &mut DriverInfoData,
        &mut DriverInfoDetail,
        core::mem::size_of_val(&DriverInfoDetail) as u32,
        core::ptr::null_mut(),
    ) == 0
        && GetLastError() != ERROR_INSUFFICIENT_BUFFER
    {
        rc = GetLastError() as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    hInf = SetupOpenInfFileW(
        core::ptr::addr_of!(DriverInfoDetail.InfFileName) as *const u16,
        core::ptr::null(),
        (INF_STYLE_WIN4) as u32,
        &mut ErrorLine,
    );
    if hInf == INVALID_HANDLE_VALUE {
        rc = GetLastError() as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    if SetupDiGetActualSectionToInstallW(
        hInf,
        core::ptr::addr_of!(DriverInfoDetail.SectionName) as *const u16,
        SectionName.as_mut_ptr(),
        (LINE_LEN) as u32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        rc = GetLastError() as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    if SetupFindFirstLineW(
        hInf,
        SectionName.as_ptr(),
        w!("Characteristics") as LPCWSTR,
        &mut InfContext,
    ) == 0
    {
        rc = GetLastError() as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    if SetupGetIntField(&mut InfContext, 1, &mut CharacteristicsInt) == 0 {
        rc = GetLastError() as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    Characteristics = CharacteristicsInt as u32;
    if IsEqualIID!(core::ptr::addr_of!((*DeviceInfoData).ClassGuid), &GUID_DEVCLASS_NET) {
        if SetupFindFirstLineW(
            hInf,
            SectionName.as_ptr(),
            w!("BusType") as LPCWSTR,
            &mut InfContext,
        ) != 0
        {
            if SetupGetStringFieldW(
                &mut InfContext,
                1,
                core::ptr::null_mut(),
                0,
                &mut dwLength,
            ) == 0
            {
                rc = GetLastError() as i32;
                return 'cleanup: loop {
                    if hInf != INVALID_HANDLE_VALUE {
                        SetupCloseInfFile(hInf);
                    }
                    if UuidRpcString != core::ptr::null_mut() {
                        RpcStringFreeW(&mut UuidRpcString);
                    }
                    HeapFree(GetProcessHeap(), 0, BusType as *mut _);
                    HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
                    break rc as u32;
                };
            }

            BusType = HeapAlloc(
                GetProcessHeap(),
                0,((dwLength as usize) * core::mem::size_of::<u16>()) as SIZE_T,
            ) as LPWSTR;
            if BusType.is_null() {
                rc = (ERROR_NOT_ENOUGH_MEMORY) as i32;
                return 'cleanup: loop {
                    if hInf != INVALID_HANDLE_VALUE {
                        SetupCloseInfFile(hInf);
                    }
                    if UuidRpcString != core::ptr::null_mut() {
                        RpcStringFreeW(&mut UuidRpcString);
                    }
                    HeapFree(GetProcessHeap(), 0, BusType as *mut _);
                    HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
                    break rc as u32;
                };
            }

            if SetupGetStringFieldW(
                &mut InfContext,
                1,
                BusType,
                dwLength,
                core::ptr::null_mut(),
            ) == 0
            {
                rc = GetLastError() as i32;
                return 'cleanup: loop {
                    if hInf != INVALID_HANDLE_VALUE {
                        SetupCloseInfFile(hInf);
                    }
                    if UuidRpcString != core::ptr::null_mut() {
                        RpcStringFreeW(&mut UuidRpcString);
                    }
                    HeapFree(GetProcessHeap(), 0, BusType as *mut _);
                    HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
                    break rc as u32;
                };
            }
        }
    }

    RpcStatus = UuidCreate(&mut Uuid);
    if RpcStatus != RPC_S_OK && RpcStatus != (RPC_S_UUID_LOCAL_ONLY) as i32 {
        rc = (ERROR_GEN_FAILURE) as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    RpcStatus = UuidToStringW(&mut Uuid, &mut UuidRpcString);
    if RpcStatus != RPC_S_OK {
        rc = (ERROR_GEN_FAILURE) as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    UuidString = HeapAlloc(
        GetProcessHeap(),
        0,((2 + wcslen(UuidRpcString)) * core::mem::size_of::<u16>()
            + core::mem::size_of_val(&UNICODE_NULL)) as SIZE_T,
    ) as LPWSTR;
    if UuidString.is_null() {
        rc = (ERROR_NOT_ENOUGH_MEMORY) as i32;
        return 'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if UuidRpcString != core::ptr::null_mut() {
                RpcStringFreeW(&mut UuidRpcString);
            }
            HeapFree(GetProcessHeap(), 0, BusType as *mut _);
            HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
            break rc as u32;
        };
    }

    wcscpy(UuidString, w!("{") as LPCWSTR);
    wcscat(UuidString, UuidRpcString);
    wcscat(UuidString, w!("}") as LPCWSTR);

    if IsEqualIID!(core::ptr::addr_of!((*DeviceInfoData).ClassGuid), &GUID_DEVCLASS_NET) {
        rc = InstallNetDevice(DeviceInfoSet, DeviceInfoData, UuidString, Characteristics, BusType) as i32;
    } else if IsEqualIID!(core::ptr::addr_of!((*DeviceInfoData).ClassGuid), &GUID_DEVCLASS_NETCLIENT) {
        rc = InstallNetClient() as i32;
    } else if IsEqualIID!(core::ptr::addr_of!((*DeviceInfoData).ClassGuid), &GUID_DEVCLASS_NETSERVICE) {
        rc = InstallNetService() as i32;
    } else if IsEqualIID!(core::ptr::addr_of!((*DeviceInfoData).ClassGuid), &GUID_DEVCLASS_NETTRANS) {
        rc = InstallNetTransport() as i32;
    } else {
        rc = (ERROR_GEN_FAILURE) as i32;
    }

    'cleanup: loop {
        if hInf != INVALID_HANDLE_VALUE {
            SetupCloseInfFile(hInf);
        }
        if UuidRpcString != core::ptr::null_mut() {
            RpcStringFreeW(&mut UuidRpcString);
        }
        HeapFree(GetProcessHeap(), 0, BusType as *mut _);
        HeapFree(GetProcessHeap(), 0, UuidString as *mut _);
        break rc as u32;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Configuration of network devices
//  * FILE:            dll/win32/netcfgx/installer.c
//  * PURPOSE:         Network devices installer
//  *
//  * PROGRAMMERS:     Hervé Poussineau (hpoussin@reactos.org)
//  */
//
// #include "precomp.h"
// #include <strsafe.h>
//
//
// /* Append a REG_SZ to an existing REG_MULTI_SZ string in the registry.
//  * If the value doesn't exist, create it.
//  * Returns ERROR_SUCCESS if success. Otherwise, returns an error code
//  */
// static
// LONG
// AppendStringToMultiSZ(
//      HKEY hKey,
//      PCWSTR ValueName,
//      PCWSTR ValueToAppend)
// {
//     PWSTR Buffer = NULL;
//     DWORD dwRegType;
//     DWORD dwRequired, dwLength;
//     DWORD dwTmp;
//     LONG rc;
//
//     rc = RegQueryValueExW(hKey,
//                           ValueName,
//                           NULL,
//                           &dwRegType,
//                           NULL,
//                           &dwRequired);
//     if (rc != ERROR_FILE_NOT_FOUND)
//     {
//         if (rc != ERROR_SUCCESS)
//             goto cleanup;
//         if (dwRegType != REG_MULTI_SZ)
//         {
//             rc = ERROR_GEN_FAILURE;
//             goto cleanup;
//         }
//
//         dwTmp = dwLength = dwRequired + wcslen(ValueToAppend) * sizeof(WCHAR) + sizeof(UNICODE_NULL);
//         Buffer = HeapAlloc(GetProcessHeap(), 0, dwLength);
//         if (!Buffer)
//         {
//             rc = ERROR_NOT_ENOUGH_MEMORY;
//             goto cleanup;
//         }
//
//         rc = RegQueryValueExW(hKey,
//                               ValueName,
//                               NULL,
//                               NULL,
//                               (BYTE*)Buffer,
//                               &dwTmp);
//         if (rc != ERROR_SUCCESS)
//             goto cleanup;
//     }
//     else
//     {
//         dwRequired = sizeof(WCHAR);
//         dwLength = wcslen(ValueToAppend) * sizeof(WCHAR) + 2 * sizeof(UNICODE_NULL);
//         Buffer = HeapAlloc(GetProcessHeap(), 0, dwLength);
//         if (!Buffer)
//         {
//             rc = ERROR_NOT_ENOUGH_MEMORY;
//             goto cleanup;
//         }
//     }
//
//     /* Append the value */
//     wcscpy(&Buffer[dwRequired / sizeof(WCHAR) - 1], ValueToAppend);
//     /* Terminate the REG_MULTI_SZ string */
//     Buffer[dwLength / sizeof(WCHAR) - 1] = UNICODE_NULL;
//
//     rc = RegSetValueExW(hKey,
//                         ValueName,
//                         0,
//                         REG_MULTI_SZ,
//                         (const BYTE*)Buffer,
//                         dwLength);
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, Buffer);
//     return rc;
// }
//
// static
// DWORD
// GetUniqueConnectionName(
//      HKEY hNetworkKey,
//      PWSTR *ppszNameBuffer)
// {
//     int Length = 0;
//     PWSTR pszDefaultName = NULL;
//     PWSTR pszNameBuffer = NULL;
//     DWORD dwSubKeys = 0;
//     DWORD dwError;
//
//     TRACE("GetNewConnectionName()\n");
//
//     Length = LoadStringW(netcfgx_hInstance, IDS_NET_CONNECT, (LPWSTR)&pszDefaultName, 0);
//     if (Length == 0)
//     {
//         pszDefaultName = L"Network Connection";
//         Length = wcslen(pszDefaultName);
//     }
//
//     TRACE("Length %d\n", Length);
//
//     dwError = RegQueryInfoKeyW(hNetworkKey,
//                                NULL, NULL, NULL, &dwSubKeys,
//                                NULL, NULL, NULL, NULL, NULL, NULL, NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         ERR("RegQueryInfoKeyW: Error %lu\n", dwError);
//         return dwError;
//     }
//
//     TRACE("Adapter Count: %lu\n", dwSubKeys);
//
//     pszNameBuffer = HeapAlloc(GetProcessHeap(),
//                               HEAP_ZERO_MEMORY,
//                               (Length + ((dwSubKeys != 0) ? 6 : 1)) * sizeof(WCHAR));
//     if (pszNameBuffer == NULL)
//     {
//         return ERROR_OUTOFMEMORY;
//     }
//
//     if (dwSubKeys != 0)
//         StringCchPrintfW(pszNameBuffer, Length + 6, L"%.*s %lu", Length, pszDefaultName, dwSubKeys + 1);
//     else
//         StringCchPrintfW(pszNameBuffer, Length + 1, L"%.*s", Length, pszDefaultName);
//
//     TRACE("Adapter Name: %S\n", pszNameBuffer);
//
//     *ppszNameBuffer = pszNameBuffer;
//
//     return ERROR_SUCCESS;
// }
//
// static
// DWORD
// InstallNetDevice(
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData,
//     LPCWSTR UuidString,
//     DWORD Characteristics,
//     LPCWSTR BusType)
// {
//     SP_DEVINSTALL_PARAMS_W DeviceInstallParams;
//     LPWSTR InstanceId = NULL;
//     LPWSTR ComponentId = NULL;
//     LPWSTR DeviceName = NULL;
//     LPWSTR ExportName = NULL;
//     LONG rc;
//     HKEY hInterfacesKey = NULL, hInterfaceKey = NULL;
//     HKEY hKey = NULL;
//     HKEY hNetworkKey = NULL;
//     HKEY hLinkageKey = NULL;
//     HKEY hConnectionKey = NULL;
//     DWORD dwShowIcon, dwLength, dwValue;
//     PWSTR pszNameBuffer = NULL;
//     PWSTR ptr;
//
//     DeviceInstallParams.cbSize = sizeof(DeviceInstallParams);
//     if (!SetupDiGetDeviceInstallParamsW(DeviceInfoSet,
//                                         DeviceInfoData,
//                                         &DeviceInstallParams))
//     {
//         rc = GetLastError();
//         ERR("SetupDiGetDeviceInstallParamsW() failed (Error %lu)\n", rc);
//         goto cleanup;
//     }
//
//     /* Do not start the adapter in the call to SetupDiInstallDevice */
//     DeviceInstallParams.Flags |= DI_DONOTCALLCONFIGMG;
//
//     if (!SetupDiSetDeviceInstallParamsW(DeviceInfoSet,
//                                         DeviceInfoData,
//                                         &DeviceInstallParams))
//     {
//         rc = GetLastError();
//         ERR("SetupDiSetDeviceInstallParamsW() failed (Error %lu)\n", rc);
//         goto cleanup;
//     }
//
//     /* Install the adapter */
//     if (!SetupDiInstallDevice(DeviceInfoSet, DeviceInfoData))
//     {
//         rc = GetLastError();
//         ERR("SetupDiInstallDevice() failed (Error %lu)\n", rc);
//         goto cleanup;
//     }
//
//     /* Get Instance ID */
//     if (SetupDiGetDeviceInstanceIdW(DeviceInfoSet, DeviceInfoData, NULL, 0, &dwLength))
//     {
//         ERR("SetupDiGetDeviceInstanceIdW() returned TRUE. FALSE expected\n");
//         rc = ERROR_GEN_FAILURE;
//         goto cleanup;
//     }
//
//     InstanceId = HeapAlloc(GetProcessHeap(), 0, dwLength * sizeof(WCHAR));
//     if (!InstanceId)
//     {
//         ERR("HeapAlloc() failed\n");
//         rc = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//
//     if (!SetupDiGetDeviceInstanceIdW(DeviceInfoSet, DeviceInfoData, InstanceId, dwLength, NULL))
//     {
//         rc = GetLastError();
//         ERR("SetupDiGetDeviceInstanceIdW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     ComponentId = HeapAlloc(GetProcessHeap(), 0, dwLength * sizeof(WCHAR));
//     if (!ComponentId)
//     {
//         ERR("HeapAlloc() failed\n");
//         rc = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//
//     wcscpy(ComponentId, InstanceId);
//     ptr = wcsrchr(ComponentId, L'\\');
//     if (ptr != NULL)
//         *ptr = UNICODE_NULL;
//
//     /* Create device name */
//     DeviceName = HeapAlloc(GetProcessHeap(), 0, (wcslen(L"\\Device\\") + wcslen(UuidString)) * sizeof(WCHAR) + sizeof(UNICODE_NULL));
//     if (!DeviceName)
//     {
//         ERR("HeapAlloc() failed\n");
//         rc = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//     wcscpy(DeviceName, L"\\Device\\");
//     wcscat(DeviceName, UuidString);
//
//     /* Create export name */
//     ExportName = HeapAlloc(GetProcessHeap(), 0, (wcslen(L"\\Device\\Tcpip_") + wcslen(UuidString)) * sizeof(WCHAR) + sizeof(UNICODE_NULL));
//     if (!ExportName)
//     {
//         ERR("HeapAlloc() failed\n");
//         rc = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//     wcscpy(ExportName, L"\\Device\\Tcpip_");
//     wcscat(ExportName, UuidString);
//
//     /* Write Tcpip parameters in new service Key */
//     rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces", 0, NULL, REG_OPTION_NON_VOLATILE, KEY_WRITE, NULL, &hInterfacesKey, NULL);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegCreateKeyExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegCreateKeyExW(hInterfacesKey, UuidString, 0, NULL, REG_OPTION_NON_VOLATILE, KEY_WRITE, NULL, &hInterfaceKey, NULL);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegCreateKeyExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hInterfaceKey, L"DefaultGateway", 0, REG_SZ, (const BYTE*)L"0.0.0.0", (wcslen(L"0.0.0.0") + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hInterfaceKey, L"IPAddress", 0, REG_SZ, (const BYTE*)L"0.0.0.0", (wcslen(L"0.0.0.0") + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hInterfaceKey, L"SubnetMask", 0, REG_SZ, (const BYTE*)L"0.0.0.0", (wcslen(L"0.0.0.0") + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     dwValue = 1;
//     rc = RegSetValueExW(hInterfaceKey, L"EnableDHCP", 0, REG_DWORD, (const BYTE*)&dwValue, sizeof(DWORD));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Write 'Linkage' key in hardware key */
//     hKey = SetupDiOpenDevRegKey(DeviceInfoSet, DeviceInfoData, DICS_FLAG_GLOBAL, 0, DIREG_DRV, KEY_READ | KEY_WRITE);
//     if (hKey == INVALID_HANDLE_VALUE && GetLastError() == ERROR_FILE_NOT_FOUND)
//         hKey = SetupDiCreateDevRegKeyW(DeviceInfoSet, DeviceInfoData, DICS_FLAG_GLOBAL, 0, DIREG_DRV, NULL, NULL);
//     if (hKey == INVALID_HANDLE_VALUE)
//     {
//         hKey = NULL;
//         rc = GetLastError();
//         ERR("SetupDiCreateDevRegKeyW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hKey, L"NetCfgInstanceId", 0, REG_SZ, (const BYTE*)UuidString, (wcslen(UuidString) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hKey, L"Characteristics", 0, REG_DWORD, (const BYTE*)&Characteristics, sizeof(DWORD));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hKey, L"ComponentId", 0, REG_SZ, (const BYTE*)ComponentId, (wcslen(ComponentId) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     if (BusType)
//     {
//         rc = RegSetValueExW(hKey, L"BusType", 0, REG_SZ, (const BYTE*)BusType, (wcslen(BusType) + 1) * sizeof(WCHAR));
//         if (rc != ERROR_SUCCESS)
//         {
//             ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//             goto cleanup;
//         }
//     }
//
//     rc = RegCreateKeyExW(hKey, L"Linkage", 0, NULL, REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, NULL, &hLinkageKey, NULL);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegCreateKeyExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hLinkageKey, L"Export", 0, REG_SZ, (const BYTE*)DeviceName, (wcslen(DeviceName) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hLinkageKey, L"RootDevice", 0, REG_SZ, (const BYTE*)UuidString, (wcslen(UuidString) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hLinkageKey, L"UpperBind", 0, REG_SZ, (const BYTE*)L"Tcpip", (wcslen(L"Tcpip") + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//     RegCloseKey(hKey);
//     hKey = NULL;
//
//     /* Write connection information in network subkey */
//     rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                          L"SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}",
//                          0,
//                          NULL,
//                          REG_OPTION_NON_VOLATILE,
//                          KEY_CREATE_SUB_KEY | KEY_QUERY_VALUE,
//                          NULL,
//                          &hNetworkKey,
//                          NULL);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegCreateKeyExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = GetUniqueConnectionName(hNetworkKey, &pszNameBuffer);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("GetUniqueConnectionName() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegCreateKeyExW(hNetworkKey, UuidString, 0, NULL, REG_OPTION_NON_VOLATILE, KEY_CREATE_SUB_KEY, NULL, &hKey, NULL);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegCreateKeyExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegCreateKeyExW(hKey, L"Connection", 0, NULL, REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, NULL, &hConnectionKey, NULL);
//     RegCloseKey(hKey);
//     hKey = NULL;
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegCreateKeyExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hConnectionKey, L"Name", 0, REG_SZ, (const BYTE*)pszNameBuffer, (wcslen(pszNameBuffer) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegSetValueExW(hConnectionKey, L"PnpInstanceID", 0, REG_SZ, (const BYTE*)InstanceId, (wcslen(InstanceId) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     dwShowIcon = 1;
//     rc = RegSetValueExW(hConnectionKey, L"ShowIcon", 0, REG_DWORD, (const BYTE*)&dwShowIcon, sizeof(dwShowIcon));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Write linkage information in Tcpip service */
//     rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Linkage", 0, NULL, REG_OPTION_NON_VOLATILE, KEY_QUERY_VALUE | KEY_SET_VALUE, NULL, &hKey, NULL);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegCreateKeyExW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//     rc = AppendStringToMultiSZ(hKey, L"Bind", DeviceName);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("AppendStringToMultiSZ() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//     rc = AppendStringToMultiSZ(hKey, L"Export", ExportName);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("AppendStringToMultiSZ() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//     rc = AppendStringToMultiSZ(hKey, L"Route", UuidString);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("AppendStringToMultiSZ() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Start the device */
//     if (!SetupDiRestartDevices(DeviceInfoSet, DeviceInfoData))
//     {
//         rc = GetLastError();
//         ERR("SetupDiRestartDevices() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = ERROR_SUCCESS;
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, pszNameBuffer);
//     HeapFree(GetProcessHeap(), 0, InstanceId);
//     HeapFree(GetProcessHeap(), 0, ComponentId);
//     HeapFree(GetProcessHeap(), 0, DeviceName);
//     HeapFree(GetProcessHeap(), 0, ExportName);
//     if (hInterfaceKey != NULL)
//         RegCloseKey(hInterfaceKey);
//     if (hInterfacesKey != NULL)
//         RegCloseKey(hInterfacesKey);
//     if (hKey != NULL)
//         RegCloseKey(hKey);
//     if (hNetworkKey != NULL)
//         RegCloseKey(hNetworkKey);
//     if (hLinkageKey != NULL)
//         RegCloseKey(hLinkageKey);
//     if (hConnectionKey != NULL)
//         RegCloseKey(hConnectionKey);
//
//     return rc;
// }
//
// static
// DWORD
// InstallNetClient(VOID)
// {
//     FIXME("Installation of network clients is not yet supported\n");
//     return ERROR_GEN_FAILURE;
// }
//
// static
// DWORD
// InstallNetService(VOID)
// {
//     FIXME("Installation of network services is not yet supported\n");
//     return ERROR_GEN_FAILURE;
// }
//
// static
// DWORD
// InstallNetTransport(VOID)
// {
//     FIXME("Installation of network protocols is not yet supported\n");
//     return ERROR_GEN_FAILURE;
// }
//
// DWORD
// WINAPI
// NetClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     SP_DRVINFO_DATA_W DriverInfoData;
//     SP_DRVINFO_DETAIL_DATA_W DriverInfoDetail;
//     WCHAR SectionName[LINE_LEN];
//     HINF hInf = INVALID_HANDLE_VALUE;
//     INFCONTEXT InfContext;
//     UINT ErrorLine;
//     INT CharacteristicsInt;
//     DWORD Characteristics;
//     LPWSTR BusType = NULL;
//     RPC_STATUS RpcStatus;
//     UUID Uuid;
//     LPWSTR UuidRpcString = NULL;
//     LPWSTR UuidString = NULL;
//     LONG rc;
//     DWORD dwLength;
//
//     if (InstallFunction != DIF_INSTALLDEVICE)
//         return ERROR_DI_DO_DEFAULT;
//
//     TRACE("%lu %p %p\n", InstallFunction, DeviceInfoSet, DeviceInfoData);
//
//     /* Get driver info details */
//     DriverInfoData.cbSize = sizeof(SP_DRVINFO_DATA_W);
//     if (!SetupDiGetSelectedDriverW(DeviceInfoSet, DeviceInfoData, &DriverInfoData))
//     {
//         rc = GetLastError();
//         ERR("SetupDiGetSelectedDriverW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     DriverInfoDetail.cbSize = sizeof(SP_DRVINFO_DETAIL_DATA_W);
//     if (!SetupDiGetDriverInfoDetailW(DeviceInfoSet, DeviceInfoData, &DriverInfoData, &DriverInfoDetail, sizeof(DriverInfoDetail), NULL)
//      && GetLastError() != ERROR_INSUFFICIENT_BUFFER)
//     {
//         rc = GetLastError();
//         ERR("SetupDiGetDriverInfoDetailW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     hInf = SetupOpenInfFileW(DriverInfoDetail.InfFileName, NULL, INF_STYLE_WIN4, &ErrorLine);
//     if (hInf == INVALID_HANDLE_VALUE)
//     {
//         rc = GetLastError();
//         ERR("SetupOpenInfFileW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     if (!SetupDiGetActualSectionToInstallW(hInf, DriverInfoDetail.SectionName, SectionName, LINE_LEN, NULL, NULL))
//     {
//         rc = GetLastError();
//         ERR("SetupDiGetActualSectionToInstallW() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Get Characteristics and BusType (optional) from .inf file */
//     if (!SetupFindFirstLineW(hInf, SectionName, L"Characteristics", &InfContext))
//     {
//         rc = GetLastError();
//         ERR("Unable to find key %S in section %S of file %S (error 0x%lx)\n",
//             L"Characteristics", SectionName, DriverInfoDetail.InfFileName, rc);
//         goto cleanup;
//     }
//
//     if (!SetupGetIntField(&InfContext, 1, &CharacteristicsInt))
//     {
//         rc = GetLastError();
//         ERR("SetupGetIntField() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     Characteristics = (DWORD)CharacteristicsInt;
//     if (IsEqualIID(&DeviceInfoData->ClassGuid, &GUID_DEVCLASS_NET))
//     {
//         if (SetupFindFirstLineW(hInf, SectionName, L"BusType", &InfContext))
//         {
//             if (!SetupGetStringFieldW(&InfContext, 1, NULL, 0, &dwLength))
//             {
//                 rc = GetLastError();
//                 ERR("SetupGetStringFieldW() failed with error 0x%lx\n", rc);
//                 goto cleanup;
//             }
//
//             BusType = HeapAlloc(GetProcessHeap(), 0, dwLength * sizeof(WCHAR));
//             if (!BusType)
//             {
//                 ERR("HeapAlloc() failed\n");
//                 rc = ERROR_NOT_ENOUGH_MEMORY;
//                 goto cleanup;
//             }
//
//             if (!SetupGetStringFieldW(&InfContext, 1, BusType, dwLength, NULL))
//             {
//                 rc = GetLastError();
//                 ERR("SetupGetStringFieldW() failed with error 0x%lx\n", rc);
//                 goto cleanup;
//             }
//         }
//     }
//
//     /* Create a new UUID */
//     RpcStatus = UuidCreate(&Uuid);
//     if (RpcStatus != RPC_S_OK && RpcStatus != RPC_S_UUID_LOCAL_ONLY)
//     {
//         ERR("UuidCreate() failed with RPC status 0x%lx\n", RpcStatus);
//         rc = ERROR_GEN_FAILURE;
//         goto cleanup;
//     }
//
//     RpcStatus = UuidToStringW(&Uuid, &UuidRpcString);
//     if (RpcStatus != RPC_S_OK)
//     {
//         ERR("UuidToStringW() failed with RPC status 0x%lx\n", RpcStatus);
//         rc = ERROR_GEN_FAILURE;
//         goto cleanup;
//     }
//
//     /* Add curly braces around Uuid */
//     UuidString = HeapAlloc(GetProcessHeap(), 0, (2 + wcslen(UuidRpcString)) * sizeof(WCHAR) + sizeof(UNICODE_NULL));
//     if (!UuidString)
//     {
//         ERR("HeapAlloc() failed\n");
//         rc = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//
//     wcscpy(UuidString, L"{");
//     wcscat(UuidString, UuidRpcString);
//     wcscat(UuidString, L"}");
//
//     if (IsEqualIID(&DeviceInfoData->ClassGuid, &GUID_DEVCLASS_NET))
//         rc = InstallNetDevice(DeviceInfoSet, DeviceInfoData, UuidString, Characteristics, BusType);
//     else if (IsEqualIID(&DeviceInfoData->ClassGuid, &GUID_DEVCLASS_NETCLIENT))
//         rc = InstallNetClient();
//     else if (IsEqualIID(&DeviceInfoData->ClassGuid, &GUID_DEVCLASS_NETSERVICE))
//         rc = InstallNetService();
//     else if (IsEqualIID(&DeviceInfoData->ClassGuid, &GUID_DEVCLASS_NETTRANS))
//         rc = InstallNetTransport();
//     else
//     {
//         ERR("Invalid class guid\n");
//         rc = ERROR_GEN_FAILURE;
//     }
//
// cleanup:
//     if (hInf != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(hInf);
//     if (UuidRpcString != NULL)
//         RpcStringFreeW(&UuidRpcString);
//     HeapFree(GetProcessHeap(), 0, BusType);
//     HeapFree(GetProcessHeap(), 0, UuidString);
//
//     TRACE("Returning 0x%lx\n", rc);
//     return rc;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
