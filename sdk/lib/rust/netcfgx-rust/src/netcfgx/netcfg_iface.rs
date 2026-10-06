use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// (The translation also re-declared KEY_READ/KEY_WRITE/REG_SZ/REG_DWORD/ERROR_SUCCESS/
// WAIT_OBJECT_0/WAIT_TIMEOUT as `pub const X: u32 = X;` — self-referential, and the
// island already carries all seven. Dropped.)
unsafe extern "C" fn impl_from_INetCfgLock(iface: *mut INetCfgLock) -> *mut INetCfgImpl {
    (iface as *mut u8).offset(-(FIELD_OFFSET!(INetCfgImpl, lpVtblLock) as isize)) as *mut INetCfgImpl
}
unsafe extern "C" fn impl_from_INetCfgPnpReconfigCallback(iface: *mut INetCfgPnpReconfigCallback) -> *mut INetCfgImpl {
    (iface as *mut u8).offset(-(FIELD_OFFSET!(INetCfgImpl, lpVtblPnpReconfigCallback) as isize)) as *mut INetCfgImpl
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgLock_fnQueryInterface(
    iface: *mut INetCfgLock,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_INetCfgLock(iface);
    INetCfg_QueryInterface(This as *mut INetCfg, iid, ppvObj)
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgLock_fnAddRef(
    iface: *mut INetCfgLock
) -> ULONG {
    let This = impl_from_INetCfgLock(iface);

    return INetCfg_AddRef(This as *mut INetCfg);
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgLock_fnRelease(
    iface: *mut INetCfgLock
) -> ULONG {
    let This = impl_from_INetCfgLock(iface);
    INetCfg_Release(This as *mut INetCfg)
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgLock_fnAcquireWriteLock(
    iface: *mut INetCfgLock,
    cmsTimeout: u32,
    pszwClientDescription: LPCWSTR,
    ppszwClientDescription: *mut LPWSTR,
) -> HRESULT {
    let mut dwResult: u32;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szValue: [u16; 100] = [0; 100];
    let This = impl_from_INetCfgLock(iface);

    if (*This).bInitialized != 0 {
        return (NETCFG_E_ALREADY_INITIALIZED) as i32;
    }

    dwResult = WaitForSingleObject((*This).hMutex, cmsTimeout);
    if dwResult == WAIT_TIMEOUT {
        if !ppszwClientDescription.is_null() {
            if RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                w!("SYSTEM\\CurrentControlSet\\Control\\Network\\NetCfgLockHolder") as LPCWSTR,
                0,
                KEY_READ,
                &mut hKey,
            ) == (ERROR_SUCCESS) as i32
            {
                dwResult = core::mem::size_of_val(&szValue) as u32;
                if RegQueryValueExW(
                    hKey,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    szValue.as_mut_ptr() as LPBYTE,
                    &mut dwResult,
                ) == (ERROR_SUCCESS) as i32
                {
                    szValue[(core::mem::size_of_val(&szValue) / core::mem::size_of::<u16>()) - 1] = 0;
                    *ppszwClientDescription = CoTaskMemAlloc(((wcslen(szValue.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T,
                    ) as LPWSTR;
                    if !(*ppszwClientDescription).is_null() {
                        wcscpy(*ppszwClientDescription, szValue.as_ptr());
                    }
                }
                RegCloseKey(hKey);
            }
        }
        return S_FALSE;
    } else if dwResult == WAIT_OBJECT_0 {
        if RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control\\Network\\NetCfgLockHolder") as LPCWSTR,
            0,
            core::ptr::null_mut(),
            0,
            KEY_WRITE,
            core::ptr::null_mut(),
            &mut hKey,
            core::ptr::null_mut(),
        ) == (ERROR_SUCCESS) as i32
        {
            RegSetValueExW(
                hKey,
                core::ptr::null(),
                0,
                REG_SZ as DWORD,
                pszwClientDescription as LPBYTE,
                ((wcslen(pszwClientDescription) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32,
            );
            RegCloseKey(hKey);
        }
        return S_OK;
    }

    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgLock_fnReleaseWriteLock(
    iface: *mut INetCfgLock
) -> HRESULT {
    let This = impl_from_INetCfgLock(iface);

    if (*This).bInitialized != 0 {
        return (NETCFG_E_ALREADY_INITIALIZED) as i32;
    }

    if ReleaseMutex((*This).hMutex) != 0 {
        return S_OK;
    } else {
        return S_FALSE;
    }
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgLock_fnIsWriteLocked(
    iface: *mut INetCfgLock,
    ppszwClientDescription: *mut LPWSTR,
) -> HRESULT {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szValue: [u16; 100] = [0; 100];
    let mut dwSize: u32 = 0;
    let mut dwType: u32 = 0;
    let mut hr: HRESULT;

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Control\\Network\\NetCfgLockHolder") as LPCWSTR, 0, KEY_READ, &mut hKey) != (ERROR_SUCCESS) as i32 {
        return S_FALSE;
    }

    dwSize = core::mem::size_of_val(&szValue) as u32;
    if RegQueryValueExW(hKey, core::ptr::null_mut(), core::ptr::null_mut(), &mut dwType, szValue.as_mut_ptr() as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 && dwType == REG_SZ as DWORD {
        hr = S_OK;
        szValue[(core::mem::size_of_val(&szValue) / core::mem::size_of::<u16>()) - 1] = 0;
        *ppszwClientDescription = CoTaskMemAlloc(((wcslen(szValue.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if !(*ppszwClientDescription).is_null() {
            wcscpy(*ppszwClientDescription, szValue.as_ptr());
        } else {
            hr = E_OUTOFMEMORY;
        }
    } else {
        hr = E_FAIL;
    }
    RegCloseKey(hKey);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgPnpReconfigCallback_fnQueryInterface(
    iface: *mut INetCfgPnpReconfigCallback,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_INetCfgPnpReconfigCallback(iface);
    INetCfg_QueryInterface(This as *mut INetCfg, iid, ppvObj)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgPnpReconfigCallback_fnAddRef(
    iface: *mut INetCfgPnpReconfigCallback,
) -> u32 {
    let This = impl_from_INetCfgPnpReconfigCallback(iface);
    INetCfg_AddRef(This as *mut INetCfg)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgPnpReconfigCallback_fnRelease(
    iface: *mut INetCfgPnpReconfigCallback
) -> ULONG {
    let This = impl_from_INetCfgPnpReconfigCallback(iface);
    INetCfg_Release(This as *mut INetCfg)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgPnpReconfigCallback_fnSendPnpReconfig(
    iface: *mut INetCfgPnpReconfigCallback,
    Layer: NCPNP_RECONFIG_LAYER,
    pszwUpper: LPCWSTR,
    pszwLower: LPCWSTR,
    pvData: PVOID,
    dwSizeOfData: DWORD,
) -> HRESULT {
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn ReadBindingString(
    Item: *mut NetCfgComponentItem
) -> HRESULT {
    let mut szBuffer: [u16; 200] = [0; 200];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: u32 = 0;
    let mut dwSize: u32 = 0;

    if Item.is_null() || (*Item).szBindName.is_null() {
        return S_OK;
    }

    wcscpy(szBuffer.as_mut_ptr(), w!("SYSTEM\\CurrentControlSet\\Services\\") as LPCWSTR);
    wcscat(szBuffer.as_mut_ptr(), (*Item).szBindName);
    wcscat(szBuffer.as_mut_ptr(), w!("\\Linkage") as LPCWSTR);

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuffer.as_ptr(), 0, KEY_READ, &mut hKey) == (ERROR_SUCCESS) as i32
    {
        dwSize = 0;
        RegQueryValueExW(hKey, w!("Bind") as LPCWSTR, core::ptr::null_mut(), &mut dwType, core::ptr::null_mut(), &mut dwSize);

        if dwSize != 0
        {
            (*Item).pszBinding = CoTaskMemAlloc((dwSize as usize) as SIZE_T) as LPWSTR;
            if (*Item).pszBinding.is_null() {
                return E_OUTOFMEMORY;
            }

            RegQueryValueExW(hKey, w!("Bind") as LPCWSTR, core::ptr::null_mut(), &mut dwType, (*Item).pszBinding as LPBYTE, &mut dwSize);
        }

        RegCloseKey(hKey);
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn EnumClientServiceProtocol(hKey: HKEY, pGuid: *const GUID, pHead: *mut *mut NetCfgComponentItem) -> HRESULT {
    let mut dwIndex: u32 = 0;
    let mut dwSize: u32 = 0;
    let mut dwType: u32 = 0;
    let mut szName: [u16; 100] = [0; 100];
    let mut szText: [u16; 100] = [0; 100];
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut hNDIKey: HKEY = core::ptr::null_mut();
    let mut pLast: *mut NetCfgComponentItem = core::ptr::null_mut();
    let mut pCurrent: *mut NetCfgComponentItem;

    *pHead = core::ptr::null_mut();
    loop {
        szText[0] = 0u16;

        dwSize = (core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) as u32;
        if RegEnumKeyExW(hKey, dwIndex, szName.as_mut_ptr(), &mut dwSize, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) == (ERROR_SUCCESS) as i32 {
            dwIndex += 1;
            pCurrent = CoTaskMemAlloc((core::mem::size_of::<NetCfgComponentItem>()) as SIZE_T) as *mut NetCfgComponentItem;
            if pCurrent.is_null() {
                return E_OUTOFMEMORY;
            }

            ZeroMemory!(pCurrent as LPVOID, core::mem::size_of::<NetCfgComponentItem>());
            CopyMemory!(core::ptr::addr_of_mut!((*pCurrent).ClassGUID) as LPVOID, pGuid as LPVOID, core::mem::size_of::<GUID>());

            if CLSIDFromString(szName.as_ptr(), core::ptr::addr_of_mut!((*pCurrent).InstanceId)) < 0 {
            }
            if RegOpenKeyExW(hKey, szName.as_ptr(), 0, KEY_READ, &mut hSubKey) == (ERROR_SUCCESS) as i32 {
                dwSize = core::mem::size_of::<u32>() as u32;

                RegQueryValueExW(hSubKey, w!("Characteristics") as LPCWSTR, core::ptr::null_mut(), &mut dwType, core::ptr::addr_of_mut!((*pCurrent).dwCharacteristics) as LPBYTE, &mut dwSize);
                if dwType != REG_DWORD as DWORD {
                    (*pCurrent).dwCharacteristics = 0;
                }

                dwSize = core::mem::size_of_val(&szText) as u32;
                if RegQueryValueExW(hSubKey, w!("ComponentId") as LPCWSTR, core::ptr::null_mut(), &mut dwType, szText.as_mut_ptr() as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 {
                    if dwType == REG_SZ as DWORD {
                        szText[(core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) - 1] = 0u16;
                        (*pCurrent).szId = CoTaskMemAlloc(((wcslen(szText.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                        if !(*pCurrent).szId.is_null() {
                            wcscpy((*pCurrent).szId, szText.as_ptr());
                        }
                    }
                }

                dwSize = core::mem::size_of_val(&szText) as u32;
                if RegQueryValueExW(hSubKey, w!("Description") as LPCWSTR, core::ptr::null_mut(), &mut dwType, szText.as_mut_ptr() as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 {
                    if dwType == REG_SZ as DWORD {
                        szText[(core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) - 1] = 0u16;
                        (*pCurrent).szDisplayName = CoTaskMemAlloc(((wcslen(szText.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                        if !(*pCurrent).szDisplayName.is_null() {
                            wcscpy((*pCurrent).szDisplayName, szText.as_ptr());
                        }
                    }
                }

                if RegOpenKeyExW(hSubKey, w!("NDI") as LPCWSTR, 0, KEY_READ, &mut hNDIKey) == (ERROR_SUCCESS) as i32 {
                    dwSize = core::mem::size_of_val(&szText) as u32;
                    if RegQueryValueExW(hNDIKey, w!("HelpText") as LPCWSTR, core::ptr::null_mut(), &mut dwType, szText.as_mut_ptr() as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 {
                        if dwType == REG_SZ as DWORD {
                            szText[(core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) - 1] = 0u16;
                            (*pCurrent).szHelpText = CoTaskMemAlloc(((wcslen(szText.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                            if !(*pCurrent).szHelpText.is_null() {
                                wcscpy((*pCurrent).szHelpText, szText.as_ptr());
                            }
                        }
                    }

                    dwSize = core::mem::size_of_val(&szText) as u32;
                    if RegQueryValueExW(hNDIKey, w!("Service") as LPCWSTR, core::ptr::null_mut(), &mut dwType, szText.as_mut_ptr() as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 {
                        if dwType == REG_SZ as DWORD {
                            szText[(core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) - 1] = 0u16;
                            (*pCurrent).szBindName = CoTaskMemAlloc(((wcslen(szText.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                            if !(*pCurrent).szBindName.is_null() {
                                wcscpy((*pCurrent).szBindName, szText.as_ptr());
                            }
                        }
                    }
                    RegCloseKey(hNDIKey);
                }
                RegCloseKey(hSubKey);

                ReadBindingString(pCurrent);

                if pLast.is_null() {
                    *pHead = pCurrent;
                } else {
                    (*pLast).pNext = pCurrent;
                }

                pLast = pCurrent;
            }
        } else {
            break;
        }

        if TRUE == 0 {
            break;
        }
    }
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn EnumerateNetworkComponent(
    pGuid: *const GUID,
    pHead: *mut *mut NetCfgComponentItem,
) -> HRESULT {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut pszGuid: LPOLESTR = core::ptr::null_mut();
    let mut szName: [u16; 150] = [0; 150];

    let mut hr: HRESULT = StringFromCLSID(pGuid, &mut pszGuid);
    if (hr) >= 0 {
        swprintf(
            szName.as_mut_ptr(),
            w!("SYSTEM\\CurrentControlSet\\Control\\Network\\%s"),
            pszGuid,
        );
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            szName.as_ptr(),
            0,
            KEY_READ,
            &mut hKey,
        ) == (ERROR_SUCCESS) as i32
        {
            hr = EnumClientServiceProtocol(hKey, pGuid, pHead);
            RegCloseKey(hKey);
        }
        CoTaskMemFree(pszGuid as LPVOID);
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn EnumerateNetworkAdapter(pHead: *mut *mut NetCfgComponentItem) -> HRESULT {
    let mut dwSize: u32 = 0;
    let mut dwIndex: u32;
    let hInfo: HDEVINFO;
    let mut DevInfo: SP_DEVINFO_DATA = core::mem::zeroed();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szNetCfg: [u16; 50] = [0; 50];
    let mut szAdapterNetCfg: [u16; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szDetail: [u16; 200] = [0; 200];
    let mut szName: [u16; 130] = [0; 130];
    let mut pLast: *mut NetCfgComponentItem = core::ptr::null_mut();
    let mut pCurrent: *mut NetCfgComponentItem;

    let temp_detail = wsz!("SYSTEM\\CurrentControlSet\\Control\\Class\\");
    core::ptr::copy_nonoverlapping(temp_detail.as_ptr(), szDetail.as_mut_ptr(), temp_detail.len());

    let temp_name = wsz!("SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\");
    core::ptr::copy_nonoverlapping(temp_name.as_ptr(), szName.as_mut_ptr(), temp_name.len());

    hInfo = SetupDiGetClassDevs(&GUID_DEVCLASS_NET, core::ptr::null_mut(), core::ptr::null_mut(), (DIGCF_PRESENT) as u32);
    if hInfo.is_null() {
        return E_FAIL;
    }

    dwIndex = 0;
    loop {
        ZeroMemory!(&mut DevInfo as *mut _ as LPVOID, core::mem::size_of::<SP_DEVINFO_DATA>());
        DevInfo.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as u32;

        if SetupDiEnumDeviceInfo(hInfo, dwIndex, &mut DevInfo) == 0 {
            dwIndex += 1;
            break;
        }
        dwIndex += 1;

        if SetupDiGetDeviceRegistryPropertyW(hInfo, &mut DevInfo, (SPDRP_DRIVER) as u32, core::ptr::null_mut(), szDetail.as_mut_ptr().offset(39 as isize) as LPBYTE, (core::mem::size_of_val(&szDetail) / core::mem::size_of::<u16>() - 40) as u32, &mut dwSize) == 0 {
            break;
        }

        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szDetail.as_ptr(), 0, KEY_READ, &mut hKey) != (ERROR_SUCCESS) as i32 {
            break;
        }

        dwSize = core::mem::size_of_val(&szNetCfg) as u32;
        if RegQueryValueExW(hKey, w!("NetCfgInstanceId") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), szNetCfg.as_mut_ptr() as LPBYTE, &mut dwSize) != (ERROR_SUCCESS) as i32 {
            RegCloseKey(hKey);
            break;
        }

        pCurrent = CoTaskMemAlloc((core::mem::size_of::<NetCfgComponentItem>()) as SIZE_T) as *mut NetCfgComponentItem;
        if pCurrent.is_null() {
            break;
        }

        ZeroMemory!(pCurrent as LPVOID, core::mem::size_of::<NetCfgComponentItem>());
        CopyMemory!(core::ptr::addr_of_mut!((*pCurrent).ClassGUID) as LPVOID, &GUID_DEVCLASS_NET as *const _ as *const core::ffi::c_void, core::mem::size_of::<GUID>());
        CLSIDFromString(szNetCfg.as_ptr(), core::ptr::addr_of_mut!((*pCurrent).InstanceId));

        (*pCurrent).szBindName = CoTaskMemAlloc(((wcslen(szNetCfg.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if !(*pCurrent).szBindName.is_null() {
            wcscpy((*pCurrent).szBindName, szNetCfg.as_ptr());
        }

        dwSize = core::mem::size_of_val(&szAdapterNetCfg) as u32;
        if RegQueryValueExW(hKey, w!("ComponentId") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), szAdapterNetCfg.as_mut_ptr() as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 {
            (*pCurrent).szId = CoTaskMemAlloc(((wcslen(szAdapterNetCfg.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if !(*pCurrent).szId.is_null() {
                wcscpy((*pCurrent).szId, szAdapterNetCfg.as_ptr());
            }
        }

        dwSize = core::mem::size_of_val(&szAdapterNetCfg) as u32;
        if RegQueryValueExW(hKey, w!("DriverDesc") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), szAdapterNetCfg.as_mut_ptr() as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 {
            (*pCurrent).szDisplayName = CoTaskMemAlloc(((wcslen(szAdapterNetCfg.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if !(*pCurrent).szDisplayName.is_null() {
                wcscpy((*pCurrent).szDisplayName, szAdapterNetCfg.as_ptr());
            }
        }

        RegCloseKey(hKey);

        wcscpy(szName.as_mut_ptr().offset(80 as isize), szNetCfg.as_ptr());
        wcscpy(szName.as_mut_ptr().offset(118 as isize), w!("\\Connection") as LPCWSTR);

        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szName.as_ptr(), 0, KEY_READ, &mut hKey) == (ERROR_SUCCESS) as i32 {
            dwSize = core::mem::size_of_val(&szAdapterNetCfg) as u32;
            if RegQueryValueExW(hKey, w!("PnpInstanceID") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), szAdapterNetCfg.as_mut_ptr() as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 {
                (*pCurrent).szNodeId = CoTaskMemAlloc(((wcslen(szAdapterNetCfg.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                if !(*pCurrent).szNodeId.is_null() {
                    wcscpy((*pCurrent).szNodeId, szAdapterNetCfg.as_ptr());
                }
            }
            RegCloseKey(hKey);
        }

        if SetupDiGetDeviceRegistryPropertyW(hInfo, &mut DevInfo, (SPDRP_DEVICEDESC) as u32, core::ptr::null_mut(), szNetCfg.as_mut_ptr() as PBYTE, (core::mem::size_of_val(&szNetCfg) / core::mem::size_of::<u16>()) as u32, &mut dwSize) != 0 {
            szNetCfg[(core::mem::size_of_val(&szNetCfg) / core::mem::size_of::<u16>()) - 1] = 0;
            (*pCurrent).szDisplayName = CoTaskMemAlloc(((wcslen(szNetCfg.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if !(*pCurrent).szDisplayName.is_null() {
                wcscpy((*pCurrent).szDisplayName, szNetCfg.as_ptr());
            }
        }

        if !pLast.is_null() {
            (*pLast).pNext = pCurrent;
        } else {
            *pHead = pCurrent;
        }

        pLast = pCurrent;

        if TRUE == 0 {
            break;
        }
    }

    SetupDiDestroyDeviceInfoList(hInfo);
    NOERROR
}
#[no_mangle]
pub unsafe extern "C" fn FindNetworkComponent(
    mut pHead: *mut NetCfgComponentItem,
    pszwComponentId: LPCWSTR,
    pComponent: *mut *mut INetCfgComponent,
    iface: *mut INetCfg,
) -> HRESULT {
    while !pHead.is_null() {
        if _wcsicmp((*pHead).szId, pszwComponentId) == 0 {
            return INetCfgComponent_Constructor(
                core::ptr::null_mut(),
                &IID_INetCfgComponent,
                pComponent as *mut LPVOID,
                pHead,
                iface,
            );
        }
        pHead = (*pHead).pNext;
    }
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnQueryInterface(
    iface: *mut INetCfg,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut INetCfgImpl;
    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(iid, &IID_IUnknown) ||
       IsEqualIID!(iid, &IID_INetCfg)
    {
        *ppvObj = This as LPVOID;
        INetCfg_AddRef(iface);
        return S_OK;
    }
    else if IsEqualIID!(iid, &IID_INetCfgLock)
    {
        if (*This).bInitialized != 0
        {
            return (NETCFG_E_ALREADY_INITIALIZED) as i32;
        }

        *ppvObj = core::ptr::addr_of_mut!((*This).lpVtblLock) as LPVOID;
        (*This).hMutex = CreateMutexW(
            core::ptr::null_mut(),
            (FALSE) as i32,
            w!("NetCfgLock") as LPCWSTR,
        );

        INetCfgLock_AddRef(iface);
        return S_OK;
    }
    else if IsEqualIID!(iid, &IID_INetCfgPnpReconfigCallback)
    {
        if (*This).bInitialized != 0
        {
            return (NETCFG_E_ALREADY_INITIALIZED) as i32;
        }

        *ppvObj = core::ptr::addr_of_mut!((*This).lpVtblPnpReconfigCallback) as LPVOID;
        INetCfgPnpReconfigCallback_AddRef(iface);
        return S_OK;
    }

    return E_NOINTERFACE;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnAddRef(
    iface: *mut INetCfg
) -> ULONG {
    let This = iface as *mut INetCfgImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    return (refCount) as u32;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnRelease(iface: *mut INetCfg) -> ULONG {
    let This = iface as *mut INetCfgImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if refCount == 0 {
        CoTaskMemFree(This as LPVOID);
    }
    return (refCount) as u32;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnInitialize(
    iface: *mut INetCfg,
    pReserved: PVOID,
) -> HRESULT {
    let mut hr: HRESULT;
    let This = iface as *mut INetCfgImpl;

    if (*This).bInitialized != 0 {
        return (NETCFG_E_ALREADY_INITIALIZED) as i32;
    }

    hr = EnumerateNetworkAdapter(core::ptr::addr_of_mut!((*This).pNet));
    if ((hr) < 0) {
        return hr;
    }

    hr = EnumerateNetworkComponent(&GUID_DEVCLASS_NETCLIENT, core::ptr::addr_of_mut!((*This).pClient));
    if hr < 0 {
        return hr;
    }

    hr = EnumerateNetworkComponent(&GUID_DEVCLASS_NETSERVICE, core::ptr::addr_of_mut!((*This).pService));
    if hr < 0 {
        return hr;
    }

    hr = EnumerateNetworkComponent(&GUID_DEVCLASS_NETTRANS, core::ptr::addr_of_mut!((*This).pProtocol));
    if hr < 0 {
        return hr;
    }

    (*This).bInitialized = (TRUE) as i32;
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn ApplyOrCancelChanges(
    mut pHead: *mut NetCfgComponentItem,
    lpClassGUID: *const CLSID,
    bApply: u8,
) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szName: [u16; 200] = [0; 200];
    let mut pszGuid: LPOLESTR = core::ptr::null_mut();

    while !pHead.is_null() {
        if (*pHead).bChanged != 0 {
            if IsEqualGUID!(lpClassGUID, &GUID_DEVCLASS_NET) {
                if bApply != 0 {
                    if StringFromCLSID(
                        core::ptr::addr_of_mut!((*pHead).InstanceId),
                        &mut pszGuid,
                    ) == NOERROR
                    {
                        swprintf(
                            szName.as_mut_ptr(),
                            w!("SYSTEM\\CurrentControlSet\\Control\\Network\\%s") as LPCWSTR,
                            pszGuid,
                        );
                        CoTaskMemFree(pszGuid as LPVOID);

                        if RegOpenKeyExW(
                            HKEY_LOCAL_MACHINE,
                            szName.as_ptr(),
                            0,
                            KEY_READ,
                            &mut hKey,
                        ) == (ERROR_SUCCESS) as i32
                        {
                            RegSetValueExW(
                                hKey,
                                core::ptr::null(),
                                0,
                                REG_SZ as DWORD,
                                (*pHead).szDisplayName as LPBYTE,
                                (wcslen((*pHead).szDisplayName)
                                    + 1) as u32
                                    * core::mem::size_of::<u16>() as u32,
                            );
                            RegCloseKey(hKey);
                        }
                    }
                }
            } else if !(*pHead).pNCCC.is_null() {
                if bApply != 0 {
                    INetCfgComponentControl_ApplyRegistryChanges((*pHead).pNCCC);
                } else {
                    INetCfgComponentControl_CancelChanges((*pHead).pNCCC);
                }
            }
        }
        pHead = (*pHead).pNext;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FreeComponentItem(pItem: *mut NetCfgComponentItem) {
    CoTaskMemFree(((*pItem).szDisplayName).cast());
    CoTaskMemFree(((*pItem).szHelpText).cast());
    CoTaskMemFree(((*pItem).szId).cast());
    CoTaskMemFree(((*pItem).szBindName).cast());
    CoTaskMemFree(((*pItem).szNodeId).cast());
    CoTaskMemFree(((*pItem).pszBinding).cast());
    CoTaskMemFree(pItem as LPVOID);
}
#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnUninitialize(
    iface: *mut INetCfg
) -> HRESULT {
    let This = iface as *mut INetCfgImpl;
    let mut pItem: *mut NetCfgComponentItem;

    if (*This).bInitialized == 0 {
        return (NETCFG_E_NOT_INITIALIZED) as i32;
    }

    while (*This).pService != core::ptr::null_mut() {
        pItem = (*This).pService;
        (*This).pService = (*pItem).pNext;
        FreeComponentItem(pItem);
    }

    while (*This).pClient != core::ptr::null_mut() {
        pItem = (*This).pClient;
        (*This).pClient = (*pItem).pNext;
        FreeComponentItem(pItem);
    }

    while (*This).pProtocol != core::ptr::null_mut() {
        pItem = (*This).pProtocol;
        (*This).pProtocol = (*pItem).pNext;
        FreeComponentItem(pItem);
    }

    while (*This).pNet != core::ptr::null_mut() {
        pItem = (*This).pNet;
        (*This).pNet = (*pItem).pNext;
        FreeComponentItem(pItem);
    }

    (*This).bInitialized = (FALSE) as i32;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnApply(
    iface: *mut INetCfg
) -> HRESULT {
    let This = iface as *mut INetCfgImpl;

    if (*This).bInitialized == (NETCFG_E_NOT_INITIALIZED) as i32 {
        return (NETCFG_E_NOT_INITIALIZED) as i32;
    }

    ApplyOrCancelChanges((*This).pNet, &GUID_DEVCLASS_NET, (TRUE) as u8);
    ApplyOrCancelChanges((*This).pClient, &GUID_DEVCLASS_NETCLIENT, (TRUE) as u8);
    ApplyOrCancelChanges((*This).pService, &GUID_DEVCLASS_NETSERVICE, (TRUE) as u8);
    ApplyOrCancelChanges((*This).pProtocol, &GUID_DEVCLASS_NETTRANS, (TRUE) as u8);

    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnCancel(
    iface: *mut INetCfg
) -> HRESULT {
    let This = iface as *mut INetCfgImpl;

    if (*This).bInitialized == 0 {
        return (NETCFG_E_NOT_INITIALIZED) as i32;
    }

    ApplyOrCancelChanges((*This).pClient, &GUID_DEVCLASS_NETCLIENT, (FALSE) as u8);
    ApplyOrCancelChanges((*This).pService, &GUID_DEVCLASS_NETSERVICE, (FALSE) as u8);
    ApplyOrCancelChanges((*This).pProtocol, &GUID_DEVCLASS_NETTRANS, (FALSE) as u8);

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnEnumComponents(
    iface: *mut INetCfg,
    pguidClass: *const GUID,
    ppenumComponent: *mut *mut IEnumNetCfgComponent,
) -> HRESULT {
    let This = iface as *mut INetCfgImpl;

    if (*This).bInitialized == 0 {
        return (NETCFG_E_NOT_INITIALIZED) as i32;
    }

    if IsEqualGUID!(&GUID_DEVCLASS_NET, pguidClass) {
        return IEnumNetCfgComponent_Constructor(
            core::ptr::null_mut(),
            &IID_IEnumNetCfgComponent,
            ppenumComponent as *mut LPVOID,
            (*This).pNet,
            iface,
        );
    } else if IsEqualGUID!(&GUID_DEVCLASS_NETCLIENT, pguidClass) {
        return IEnumNetCfgComponent_Constructor(
            core::ptr::null_mut(),
            &IID_IEnumNetCfgComponent,
            ppenumComponent as *mut LPVOID,
            (*This).pClient,
            iface,
        );
    } else if IsEqualGUID!(&GUID_DEVCLASS_NETSERVICE, pguidClass) {
        return IEnumNetCfgComponent_Constructor(
            core::ptr::null_mut(),
            &IID_IEnumNetCfgComponent,
            ppenumComponent as *mut LPVOID,
            (*This).pService,
            iface,
        );
    } else if IsEqualGUID!(&GUID_DEVCLASS_NETTRANS, pguidClass) {
        return IEnumNetCfgComponent_Constructor(
            core::ptr::null_mut(),
            &IID_IEnumNetCfgComponent,
            ppenumComponent as *mut LPVOID,
            (*This).pProtocol,
            iface,
        );
    } else {
        return E_NOINTERFACE;
    }
}
#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnFindComponent(
    iface: *mut INetCfg,
    pszwComponentId: LPCWSTR,
    pComponent: *mut *mut INetCfgComponent,
) -> HRESULT {
    let mut hr: HRESULT;
    let This = iface as *mut INetCfgImpl;

    if (*This).bInitialized == 0 {
        return (NETCFG_E_NOT_INITIALIZED) as i32;
    }

    hr = FindNetworkComponent((*This).pNet, pszwComponentId, pComponent, iface);
    if hr == S_OK {
        return hr;
    }

    hr = FindNetworkComponent((*This).pClient, pszwComponentId, pComponent, iface);
    if hr == S_OK {
        return hr;
    }

    hr = FindNetworkComponent((*This).pService, pszwComponentId, pComponent, iface);
    if hr == S_OK {
        return hr;
    }

    hr = FindNetworkComponent((*This).pProtocol, pszwComponentId, pComponent, iface);
    if hr == S_OK {
        return hr;
    }

    return S_FALSE;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfg_fnQueryNetCfgClass(
    iface: *mut INetCfg,
    pguidClass: *const GUID,
    riid: *const IID,
    // `void **`, i.e. `*mut LPVOID` — LPVOID is already `*mut c_void`, so the translated
    // `*mut *mut LPVOID` was `void ***`. It would have linked and written the caller's
    // pointer one level too shallow. Caught by verify_c_signatures, which is the only
    // check that reads a crate no compiler has read.
    ppvObject: *mut LPVOID,
) -> HRESULT {
    INetCfgClass_Constructor(iface as *mut IUnknown, riid, ppvObject, pguidClass, iface as *mut INetCfg as *mut _)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfg_Constructor(
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let This: *mut INetCfgImpl;

    if ppv.is_null() {
        return E_POINTER;
    }

    This = CoTaskMemAlloc((core::mem::size_of::<INetCfgImpl>()) as SIZE_T) as *mut INetCfgImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = &vt_NetCfg as *const INetCfgVtbl as *const INetCfg;
    (*This).lpVtblLock = &vt_NetCfgLock as *const INetCfgLockVtbl as *const INetCfgLock;
    (*This).lpVtblPnpReconfigCallback = &vt_NetCfgPnpReconfigCallback as *const INetCfgPnpReconfigCallbackVtbl as *const INetCfgPnpReconfigCallback;
    (*This).hMutex = core::ptr::null_mut();
    (*This).bInitialized = (FALSE) as i32;
    (*This).pNet = core::ptr::null_mut();
    (*This).pClient = core::ptr::null_mut();
    (*This).pService = core::ptr::null_mut();
    (*This).pProtocol = core::ptr::null_mut();

    if !((INetCfg_QueryInterface(This as *mut INetCfg, riid, ppv)) >= 0) {
        INetCfg_Release(This as *mut INetCfg);
        return E_NOINTERFACE;
    }

    INetCfg_Release(This as *mut INetCfg);
    return S_OK;
}

// COM vtable tables — generated from netcfg_iface.c by vtbl_gen.py
/// netcfg_iface.c:144 `static const INetCfgLockVtbl vt_NetCfgLock`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgLock: INetCfgLockVtbl = INetCfgLockVtbl {
    QueryInterface: Some(INetCfgLock_fnQueryInterface),
    AddRef: Some(INetCfgLock_fnAddRef),
    Release: Some(INetCfgLock_fnRelease),
    AcquireWriteLock: Some(INetCfgLock_fnAcquireWriteLock),
    ReleaseWriteLock: Some(INetCfgLock_fnReleaseWriteLock),
    IsWriteLocked: Some(INetCfgLock_fnIsWriteLocked),
};

/// netcfg_iface.c:202 `static const INetCfgPnpReconfigCallbackVtbl vt_NetCfgPnpReconfigCallback`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgPnpReconfigCallback: INetCfgPnpReconfigCallbackVtbl = INetCfgPnpReconfigCallbackVtbl {
    QueryInterface: Some(INetCfgPnpReconfigCallback_fnQueryInterface),
    AddRef: Some(INetCfgPnpReconfigCallback_fnAddRef),
    Release: Some(INetCfgPnpReconfigCallback_fnRelease),
    SendPnpReconfig: Some(INetCfgPnpReconfigCallback_fnSendPnpReconfig),
};

/// netcfg_iface.c:837 `static const INetCfgVtbl vt_NetCfg`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfg: INetCfgVtbl = INetCfgVtbl {
    QueryInterface: Some(INetCfg_fnQueryInterface),
    AddRef: Some(INetCfg_fnAddRef),
    Release: Some(INetCfg_fnRelease),
    Initialize: Some(INetCfg_fnInitialize),
    Uninitialize: Some(INetCfg_fnUninitialize),
    Apply: Some(INetCfg_fnApply),
    Cancel: Some(INetCfg_fnCancel),
    EnumComponents: Some(INetCfg_fnEnumComponents),
    FindComponent: Some(INetCfg_fnFindComponent),
    QueryNetCfgClass: Some(INetCfg_fnQueryNetCfgClass),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// static __inline INetCfgImpl* impl_from_INetCfgLock(INetCfgLock *iface)
// {
//     return (INetCfgImpl*)((char *)iface - FIELD_OFFSET(INetCfgImpl, lpVtblLock));
// }
//
// static __inline INetCfgImpl* impl_from_INetCfgPnpReconfigCallback(INetCfgPnpReconfigCallback *iface)
// {
//     return (INetCfgImpl*)((char *)iface - FIELD_OFFSET(INetCfgImpl, lpVtblPnpReconfigCallback));
// }
//
//
// HRESULT
// WINAPI
// INetCfgLock_fnQueryInterface(
//     INetCfgLock * iface,
//     REFIID iid,
//     LPVOID * ppvObj)
// {
//     INetCfgImpl * This = impl_from_INetCfgLock(iface);
//     return INetCfg_QueryInterface((INetCfg*)This, iid, ppvObj);
// }
//
//
// ULONG
// WINAPI
// INetCfgLock_fnAddRef(
//     INetCfgLock * iface)
// {
//     INetCfgImpl * This = impl_from_INetCfgLock(iface);
//
//     return INetCfg_AddRef((INetCfg*)This);
// }
//
// ULONG
// WINAPI
// INetCfgLock_fnRelease(
//     INetCfgLock * iface)
// {
//     INetCfgImpl * This = impl_from_INetCfgLock(iface);
//     return INetCfg_Release((INetCfg*)This);
// }
//
// HRESULT
// WINAPI
// INetCfgLock_fnAcquireWriteLock(
//     INetCfgLock * iface,
//     DWORD cmsTimeout,
//     LPCWSTR pszwClientDescription,
//     LPWSTR *ppszwClientDescription)
// {
//     DWORD dwResult;
//     HKEY hKey;
//     WCHAR szValue[100];
//     INetCfgImpl * This = impl_from_INetCfgLock(iface);
//
//     if (This->bInitialized)
//         return NETCFG_E_ALREADY_INITIALIZED;
//
//     dwResult = WaitForSingleObject(This->hMutex, cmsTimeout);
//     if (dwResult == WAIT_TIMEOUT)
//     {
//         if (ppszwClientDescription)
//         {
//             if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Control\\Network\\NetCfgLockHolder", 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//             {
//                 dwResult = sizeof(szValue);
//                 if (RegQueryValueExW(hKey, NULL, NULL, NULL, (LPBYTE)szValue, &dwResult) == ERROR_SUCCESS)
//                 {
//                      szValue[(sizeof(szValue)/sizeof(WCHAR))-1] = L'\0';
//                      *ppszwClientDescription = CoTaskMemAlloc((wcslen(szValue)+1) * sizeof(WCHAR));
//                      if (*ppszwClientDescription)
//                          wcscpy(*ppszwClientDescription, szValue);
//                 }
//                 RegCloseKey(hKey);
//             }
//         }
//         return S_FALSE;
//     }
//     else if (dwResult == WAIT_OBJECT_0)
//     {
//         if (RegCreateKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Control\\Network\\NetCfgLockHolder", 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS)
//         {
//             RegSetValueExW(hKey, NULL, 0, REG_SZ, (LPBYTE)pszwClientDescription, (wcslen(pszwClientDescription)+1) * sizeof(WCHAR));
//             RegCloseKey(hKey);
//         }
//         return S_OK;
//     }
//
//     return E_FAIL;
// }
//
// HRESULT
// WINAPI
// INetCfgLock_fnReleaseWriteLock(
//     INetCfgLock * iface)
// {
//     INetCfgImpl * This = impl_from_INetCfgLock(iface);
//
//     if (This->bInitialized)
//         return NETCFG_E_ALREADY_INITIALIZED;
//
//
//     if (ReleaseMutex(This->hMutex))
//         return S_OK;
//     else
//         return S_FALSE;
// }
//
// HRESULT
// WINAPI
// INetCfgLock_fnIsWriteLocked(
//     INetCfgLock * iface,
//     LPWSTR *ppszwClientDescription)
// {
//     HKEY hKey;
//     WCHAR szValue[100];
//     DWORD dwSize, dwType;
//     HRESULT hr;
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Control\\Network\\NetCfgLockHolder", 0, KEY_READ, &hKey) != ERROR_SUCCESS)
//         return S_FALSE;
//
//     dwSize = sizeof(szValue);
//     if (RegQueryValueExW(hKey, NULL, NULL, &dwType, (LPBYTE)szValue, &dwSize) == ERROR_SUCCESS && dwType == REG_SZ)
//     {
//         hr = S_OK;
//         szValue[(sizeof(szValue)/sizeof(WCHAR))-1] = L'\0';
//         *ppszwClientDescription = CoTaskMemAlloc((wcslen(szValue)+1) * sizeof(WCHAR));
//         if (*ppszwClientDescription)
//             wcscpy(*ppszwClientDescription, szValue);
//         else
//             hr = E_OUTOFMEMORY;
//     }
//     else
//     {
//         hr = E_FAIL;
//     }
//     RegCloseKey(hKey);
//     return hr;
// }
//
// static const INetCfgLockVtbl vt_NetCfgLock =
// {
//     INetCfgLock_fnQueryInterface,
//     INetCfgLock_fnAddRef,
//     INetCfgLock_fnRelease,
//     INetCfgLock_fnAcquireWriteLock,
//     INetCfgLock_fnReleaseWriteLock,
//     INetCfgLock_fnIsWriteLocked
// };
//
// /***************************************************************
//  * INetCfgPnpReconfigCallback
//  */
//
// HRESULT
// WINAPI
// INetCfgPnpReconfigCallback_fnQueryInterface(
//     INetCfgPnpReconfigCallback * iface,
//     REFIID iid,
//     LPVOID * ppvObj)
// {
//     INetCfgImpl * This = impl_from_INetCfgPnpReconfigCallback(iface);
//     return INetCfg_QueryInterface((INetCfg*)This, iid, ppvObj);
// }
//
// ULONG
// WINAPI
// INetCfgPnpReconfigCallback_fnAddRef(
//     INetCfgPnpReconfigCallback * iface)
// {
//     INetCfgImpl * This = impl_from_INetCfgPnpReconfigCallback(iface);
//
//     return INetCfg_AddRef((INetCfg*)This);
// }
//
// ULONG
// WINAPI
// INetCfgPnpReconfigCallback_fnRelease(
//     INetCfgPnpReconfigCallback * iface)
// {
//     INetCfgImpl * This = impl_from_INetCfgPnpReconfigCallback(iface);
//     return INetCfg_Release((INetCfg*)This);
// }
//
// HRESULT
// WINAPI
// INetCfgPnpReconfigCallback_fnSendPnpReconfig(
//     INetCfgPnpReconfigCallback * iface,
//     NCPNP_RECONFIG_LAYER Layer,
//     LPCWSTR pszwUpper,
//     LPCWSTR pszwLower,
//     PVOID pvData,
//     DWORD dwSizeOfData)
// {
//     /* FIXME */
//     return E_NOTIMPL;
// }
//
// static const INetCfgPnpReconfigCallbackVtbl vt_NetCfgPnpReconfigCallback =
// {
//     INetCfgPnpReconfigCallback_fnQueryInterface,
//     INetCfgPnpReconfigCallback_fnAddRef,
//     INetCfgPnpReconfigCallback_fnRelease,
//     INetCfgPnpReconfigCallback_fnSendPnpReconfig
// };
//
//
// /***************************************************************
//  * INetCfg
//  */
//
// HRESULT
// ReadBindingString(
//     NetCfgComponentItem *Item)
// {
//     WCHAR szBuffer[200];
//     HKEY hKey;
//     DWORD dwType, dwSize;
//
//     if (Item == NULL || Item->szBindName == NULL)
//         return S_OK;
//
//     wcscpy(szBuffer, L"SYSTEM\\CurrentControlSet\\Services\\");
//     wcscat(szBuffer, Item->szBindName);
//     wcscat(szBuffer, L"\\Linkage");
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuffer, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//     {
//         dwSize = 0;
//         RegQueryValueExW(hKey, L"Bind", NULL, &dwType, NULL, &dwSize);
//
//         if (dwSize != 0)
//         {
//             Item->pszBinding = CoTaskMemAlloc(dwSize);
//             if (Item->pszBinding == NULL)
//                 return E_OUTOFMEMORY;
//
//             RegQueryValueExW(hKey, L"Bind", NULL, &dwType, (LPBYTE)Item->pszBinding, &dwSize);
//         }
//
//         RegCloseKey(hKey);
//     }
//
//     return S_OK;
// }
//
// HRESULT
// EnumClientServiceProtocol(HKEY hKey, const GUID * pGuid, NetCfgComponentItem ** pHead)
// {
//     DWORD dwIndex = 0;
//     DWORD dwSize;
//     DWORD dwType;
//     WCHAR szName[100];
//     WCHAR szText[100];
//     HKEY hSubKey, hNDIKey;
//     NetCfgComponentItem * pLast = NULL, *pCurrent;
//
//     *pHead = NULL;
//     do
//     {
//         szText[0] = L'\0';
//
//         dwSize = sizeof(szName)/sizeof(WCHAR);
//         if (RegEnumKeyExW(hKey, dwIndex++, szName, &dwSize, NULL, NULL, NULL, NULL) == ERROR_SUCCESS)
//         {
//             pCurrent = CoTaskMemAlloc(sizeof(NetCfgComponentItem));
//             if (!pCurrent)
//                 return E_OUTOFMEMORY;
//
//             ZeroMemory(pCurrent, sizeof(NetCfgComponentItem));
//             CopyMemory(&pCurrent->ClassGUID, pGuid, sizeof(GUID));
//
//             if (FAILED(CLSIDFromString(szName, &pCurrent->InstanceId)))
//             {
//                 /// ReactOS tcpip guid is not yet generated
//                 //CoTaskMemFree(pCurrent);
//                 //return E_FAIL;
//             }
//             if (RegOpenKeyExW(hKey, szName, 0, KEY_READ, &hSubKey) == ERROR_SUCCESS)
//             {
//                 /* retrieve Characteristics */
//                 dwSize = sizeof(DWORD);
//
//                 RegQueryValueExW(hSubKey, L"Characteristics", NULL, &dwType, (LPBYTE)&pCurrent->dwCharacteristics, &dwSize);
//                 if (dwType != REG_DWORD)
//                     pCurrent->dwCharacteristics = 0;
//
//                 /* retrieve ComponentId */
//                 dwSize = sizeof(szText);
//                 if (RegQueryValueExW(hSubKey, L"ComponentId", NULL, &dwType, (LPBYTE)szText, &dwSize) == ERROR_SUCCESS)
//                 {
//                     if (dwType == REG_SZ)
//                     {
//                         szText[(sizeof(szText)/sizeof(WCHAR))-1] = L'\0';
//                         pCurrent->szId = CoTaskMemAlloc((wcslen(szText)+1)* sizeof(WCHAR));
//                         if (pCurrent->szId)
//                             wcscpy(pCurrent->szId, szText);
//                     }
//                 }
//
//                 /* retrieve Description */
//                 dwSize = sizeof(szText);
//                 if (RegQueryValueExW(hSubKey, L"Description", NULL, &dwType, (LPBYTE)szText, &dwSize) == ERROR_SUCCESS)
//                 {
//                     if (dwType == REG_SZ)
//                     {
//                         szText[(sizeof(szText)/sizeof(WCHAR))-1] = L'\0';
//                         pCurrent->szDisplayName = CoTaskMemAlloc((wcslen(szText)+1)* sizeof(WCHAR));
//                         if (pCurrent->szDisplayName)
//                             wcscpy(pCurrent->szDisplayName, szText);
//                     }
//                 }
//
//                 if (RegOpenKeyExW(hSubKey, L"NDI", 0, KEY_READ, &hNDIKey) == ERROR_SUCCESS)
//                 {
//                     /* retrieve HelpText */
//                     dwSize = sizeof(szText);
//                     if (RegQueryValueExW(hNDIKey, L"HelpText", NULL, &dwType, (LPBYTE)szText, &dwSize) == ERROR_SUCCESS)
//                     {
//                         if (dwType == REG_SZ)
//                         {
//                             szText[(sizeof(szText)/sizeof(WCHAR))-1] = L'\0';
//                             pCurrent->szHelpText = CoTaskMemAlloc((wcslen(szText)+1)* sizeof(WCHAR));
//                             if (pCurrent->szHelpText)
//                                 wcscpy(pCurrent->szHelpText, szText);
//                         }
//                     }
//
//                     /* retrieve Service */
//                     dwSize = sizeof(szText);
//                     if (RegQueryValueExW(hNDIKey, L"Service", NULL, &dwType, (LPBYTE)szText, &dwSize) == ERROR_SUCCESS)
//                     {
//                         if (dwType == REG_SZ)
//                         {
//                             szText[(sizeof(szText)/sizeof(WCHAR))-1] = L'\0';
//                             pCurrent->szBindName = CoTaskMemAlloc((wcslen(szText)+1)* sizeof(WCHAR));
//                             if (pCurrent->szBindName)
//                                 wcscpy(pCurrent->szBindName, szText);
//                         }
//                     }
//                     RegCloseKey(hNDIKey);
//                 }
//                 RegCloseKey(hSubKey);
//
//                 ReadBindingString(pCurrent);
//
//                 if (!pLast)
//                     *pHead = pCurrent;
//                 else
//                     pLast->pNext = pCurrent;
//
//                 pLast = pCurrent;
//             }
//         }
//         else
//            break;
//
//     }while(TRUE);
//     return S_OK;
// }
//
//
//
// HRESULT
// EnumerateNetworkComponent(
//     const GUID *pGuid, NetCfgComponentItem ** pHead)
// {
//     HKEY hKey;
//     LPOLESTR pszGuid;
//     HRESULT hr;
//     WCHAR szName[150];
//
//     hr = StringFromCLSID(pGuid, &pszGuid);
//     if (SUCCEEDED(hr))
//     {
//         swprintf(szName, L"SYSTEM\\CurrentControlSet\\Control\\Network\\%s", pszGuid);
//         if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szName, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//         {
//             hr = EnumClientServiceProtocol(hKey, pGuid, pHead);
//             RegCloseKey(hKey);
//         }
//         CoTaskMemFree(pszGuid);
//     }
//     return hr;
// }
//
// HRESULT
// EnumerateNetworkAdapter(NetCfgComponentItem ** pHead)
// {
//     DWORD dwSize, dwIndex;
//     HDEVINFO hInfo;
//     SP_DEVINFO_DATA DevInfo;
//     HKEY hKey;
//     WCHAR szNetCfg[50];
//     WCHAR szAdapterNetCfg[MAX_DEVICE_ID_LEN];
//     WCHAR szDetail[200] = L"SYSTEM\\CurrentControlSet\\Control\\Class\\";
//     WCHAR szName[130] = L"SYSTEM\\CurrentControlSet\\Control\\Network\\{4D36E972-E325-11CE-BFC1-08002BE10318}\\";
//     NetCfgComponentItem * pLast = NULL, *pCurrent;
//
//     hInfo = SetupDiGetClassDevs(&GUID_DEVCLASS_NET, NULL, NULL, DIGCF_PRESENT );
//     if (!hInfo)
//     {
//         return E_FAIL;
//     }
//
//     dwIndex = 0;
//     do
//     {
//         ZeroMemory(&DevInfo, sizeof(SP_DEVINFO_DATA));
//         DevInfo.cbSize = sizeof(DevInfo);
//
//         /* get device info */
//         if (!SetupDiEnumDeviceInfo(hInfo, dwIndex++, &DevInfo))
//             break;
//
//         /* get device software registry path */
//         if (!SetupDiGetDeviceRegistryPropertyW(hInfo, &DevInfo, SPDRP_DRIVER, NULL, (LPBYTE)&szDetail[39], sizeof(szDetail)/sizeof(WCHAR) - 40, &dwSize))
//             break;
//
//         /* open device registry key */
//         if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szDetail, 0, KEY_READ, &hKey) != ERROR_SUCCESS)
//             break;
//
//         /* query NetCfgInstanceId for current device */
//         dwSize = sizeof(szNetCfg);
//         if (RegQueryValueExW(hKey, L"NetCfgInstanceId", NULL, NULL, (LPBYTE)szNetCfg, &dwSize) != ERROR_SUCCESS)
//         {
//             RegCloseKey(hKey);
//             break;
//         }
//
//         /* allocate new INetConnectionItem */
//         pCurrent = CoTaskMemAlloc(sizeof(NetCfgComponentItem));
//         if (!pCurrent)
//             break;
//
//         ZeroMemory(pCurrent, sizeof(NetCfgComponentItem));
//         CopyMemory(&pCurrent->ClassGUID, &GUID_DEVCLASS_NET, sizeof(GUID));
//         CLSIDFromString(szNetCfg, &pCurrent->InstanceId); //FIXME
//
//         /* set bind name */
//         pCurrent->szBindName = CoTaskMemAlloc((wcslen(szNetCfg)+1) *sizeof(WCHAR));
//         if (pCurrent->szBindName)
//             wcscpy(pCurrent->szBindName, szNetCfg);
//
//         /* retrieve ComponentId */
//         dwSize = sizeof(szAdapterNetCfg);
//         if (RegQueryValueExW(hKey, L"ComponentId", NULL, NULL, (LPBYTE)szAdapterNetCfg, &dwSize) == ERROR_SUCCESS)
//         {
//             pCurrent->szId = CoTaskMemAlloc((wcslen(szAdapterNetCfg)+1) * sizeof(WCHAR));
//             if (pCurrent->szId)
//                 wcscpy(pCurrent->szId, szAdapterNetCfg);
//         }
//         /* set INetCfgComponent::GetDisplayName */
//         dwSize = sizeof(szAdapterNetCfg);
//         if (RegQueryValueExW(hKey, L"DriverDesc", NULL, NULL, (LPBYTE)szAdapterNetCfg, &dwSize) == ERROR_SUCCESS)
//         {
//             pCurrent->szDisplayName = CoTaskMemAlloc((wcslen(szAdapterNetCfg)+1) * sizeof(WCHAR));
//             if (pCurrent->szDisplayName)
//                 wcscpy(pCurrent->szDisplayName, szAdapterNetCfg);
//         }
//
//         RegCloseKey(hKey);
//         /*  open network connections details */
//         wcscpy(&szName[80], szNetCfg);
//         wcscpy(&szName[118], L"\\Connection");
//
//         if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szName, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//         {
//             /* retrieve pnp instance id */
//             dwSize = sizeof(szAdapterNetCfg);
//             if (RegQueryValueExW(hKey, L"PnpInstanceID", NULL, NULL, (LPBYTE)szAdapterNetCfg, &dwSize) == ERROR_SUCCESS)
//             {
//                 pCurrent->szNodeId = CoTaskMemAlloc((wcslen(szAdapterNetCfg)+1) * sizeof(WCHAR));
//                 if (pCurrent->szNodeId)
//                     wcscpy(pCurrent->szNodeId, szAdapterNetCfg);
//             }
//             RegCloseKey(hKey);
//         }
//
//         if (SetupDiGetDeviceRegistryPropertyW(hInfo, &DevInfo, SPDRP_DEVICEDESC, NULL, (PBYTE)szNetCfg, sizeof(szNetCfg)/sizeof(WCHAR), &dwSize))
//         {
//             szNetCfg[(sizeof(szNetCfg)/sizeof(WCHAR))-1] = L'\0';
//             pCurrent->szDisplayName = CoTaskMemAlloc((wcslen(szNetCfg)+1) * sizeof(WCHAR));
//             if (pCurrent->szDisplayName)
//                 wcscpy(pCurrent->szDisplayName, szNetCfg);
//         }
//
//         if (pLast)
//             pLast->pNext = pCurrent;
//         else
//             *pHead = pCurrent;
//
//         pLast = pCurrent;
//
//     }while(TRUE);
//
//     SetupDiDestroyDeviceInfoList(hInfo);
//     return NOERROR;
// }
//
//
// HRESULT
// FindNetworkComponent(
//     NetCfgComponentItem * pHead,
//     LPCWSTR pszwComponentId,
//     INetCfgComponent **pComponent,
//     INetCfg * iface)
// {
//     while(pHead)
//     {
//         if (!_wcsicmp(pHead->szId, pszwComponentId))
//         {
//             return INetCfgComponent_Constructor(NULL, &IID_INetCfgComponent, (LPVOID*)pComponent, pHead, iface);
//         }
//         pHead = pHead->pNext;
//     }
//     return S_FALSE;
// }
//
//
//
// HRESULT
// WINAPI
// INetCfg_fnQueryInterface(
//     INetCfg * iface,
//     REFIID iid,
//     LPVOID * ppvObj)
// {
//     INetCfgImpl * This = (INetCfgImpl*)iface;
//     *ppvObj = NULL;
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_INetCfg))
//     {
//         *ppvObj = This;
//         INetCfg_AddRef(iface);
//         return S_OK;
//     }
//     else if (IsEqualIID (iid, &IID_INetCfgLock))
//     {
//         if (This->bInitialized)
//             return NETCFG_E_ALREADY_INITIALIZED;
//
//         *ppvObj = (LPVOID)&This->lpVtblLock;
//         This->hMutex = CreateMutexW(NULL, FALSE, L"NetCfgLock");
//
//         INetCfgLock_AddRef(iface);
//         return S_OK;
//     }
//     else if (IsEqualIID (iid, &IID_INetCfgPnpReconfigCallback))
//     {
//         if (This->bInitialized)
//             return NETCFG_E_ALREADY_INITIALIZED;
//
//         *ppvObj = (LPVOID)&This->lpVtblPnpReconfigCallback;
//         INetCfgPnpReconfigCallback_AddRef(iface);
//         return S_OK;
//     }
//
//     return E_NOINTERFACE;
// }
//
// ULONG
// WINAPI
// INetCfg_fnAddRef(
//     INetCfg * iface)
// {
//     INetCfgImpl * This = (INetCfgImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// INetCfg_fnRelease(
//     INetCfg * iface)
// {
//     INetCfgImpl * This = (INetCfgImpl*)iface;
//     ULONG refCount = InterlockedDecrement(&This->ref);
//
//     if (!refCount)
//     {
//         CoTaskMemFree (This);
//     }
//     return refCount;
// }
//
// HRESULT
// WINAPI
// INetCfg_fnInitialize(
//     INetCfg * iface,
//     PVOID pReserved)
// {
//     HRESULT hr;
//     INetCfgImpl *This = (INetCfgImpl *)iface;
//
//     if (This->bInitialized)
//         return NETCFG_E_ALREADY_INITIALIZED;
//
//     hr = EnumerateNetworkAdapter(&This->pNet);
//     if (FAILED(hr))
//         return hr;
//
//
//     hr = EnumerateNetworkComponent(&GUID_DEVCLASS_NETCLIENT, &This->pClient);
//     if (FAILED(hr))
//         return hr;
//
//     hr = EnumerateNetworkComponent(&GUID_DEVCLASS_NETSERVICE, &This->pService);
//     if (FAILED(hr))
//         return hr;
//
//
//     hr = EnumerateNetworkComponent(&GUID_DEVCLASS_NETTRANS, &This->pProtocol);
//     if (FAILED(hr))
//         return hr;
//
//     This->bInitialized = TRUE;
//     return S_OK;
// }
//
// VOID
// ApplyOrCancelChanges(
//     NetCfgComponentItem *pHead,
//     const CLSID * lpClassGUID,
//     BOOL bApply)
// {
//     HKEY hKey;
//     WCHAR szName[200];
//     LPOLESTR pszGuid;
//
//     while(pHead)
//     {
//         if (pHead->bChanged)
//         {
//             if (IsEqualGUID(lpClassGUID, &GUID_DEVCLASS_NET))
//             {
//                 if (bApply)
//                 {
//                     if (StringFromCLSID(&pHead->InstanceId, &pszGuid) == NOERROR)
//                     {
//                         swprintf(szName, L"SYSTEM\\CurrentControlSet\\Control\\Network\\%s", pszGuid);
//                         CoTaskMemFree(pszGuid);
//
//                         if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szName, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//                         {
//                             RegSetValueExW(hKey, NULL, 0, REG_SZ, (LPBYTE)pHead->szDisplayName, (wcslen(pHead->szDisplayName)+1) * sizeof(WCHAR));
//                             RegCloseKey(hKey);
//                         }
//                     }
//                 }
//             }
//             else if (pHead->pNCCC)
//             {
//                 if (bApply)
//                 {
//                     INetCfgComponentControl_ApplyRegistryChanges(pHead->pNCCC);
//                     //FIXME
//                     // implement INetCfgPnpReconfigCallback and pass it to
//                     //INetCfgComponentControl_ApplyPnpChanges(pHead->pNCCC, NULL);
//                 }
//                 else
//                 {
//                     INetCfgComponentControl_CancelChanges(pHead->pNCCC);
//                 }
//             }
//         }
//         pHead = pHead->pNext;
//     }
// }
//
// VOID
// FreeComponentItem(NetCfgComponentItem *pItem)
// {
//     CoTaskMemFree(pItem->szDisplayName);
//     CoTaskMemFree(pItem->szHelpText);
//     CoTaskMemFree(pItem->szId);
//     CoTaskMemFree(pItem->szBindName);
//     CoTaskMemFree(pItem->szNodeId);
//     CoTaskMemFree(pItem->pszBinding);
//     CoTaskMemFree(pItem);
// }
//
// HRESULT
// WINAPI
// INetCfg_fnUninitialize(
//     INetCfg * iface)
// {
//     INetCfgImpl *This = (INetCfgImpl *)iface;
//     NetCfgComponentItem *pItem;
//
//     if (!This->bInitialized)
//         return NETCFG_E_NOT_INITIALIZED;
//
//     /* Free the services */
//     while (This->pService != NULL)
//     {
//         pItem = This->pService;
//         This->pService = pItem->pNext;
//         FreeComponentItem(pItem);
//     }
//
//     /* Free the clients */
//     while (This->pClient != NULL)
//     {
//         pItem = This->pClient;
//         This->pClient = pItem->pNext;
//         FreeComponentItem(pItem);
//     }
//
//     /* Free the protocols */
//     while (This->pProtocol != NULL)
//     {
//         pItem = This->pProtocol;
//         This->pProtocol = pItem->pNext;
//         FreeComponentItem(pItem);
//     }
//
//     /* Free the adapters */
//     while (This->pNet != NULL)
//     {
//         pItem = This->pNet;
//         This->pNet = pItem->pNext;
//         FreeComponentItem(pItem);
//     }
//
//     This->bInitialized = FALSE;
//
//     return S_OK;
// }
//
//
// HRESULT
// WINAPI
// INetCfg_fnApply(
//     INetCfg * iface)
// {
//     INetCfgImpl *This = (INetCfgImpl *)iface;
//
//     if (!This->bInitialized)
//         return NETCFG_E_NOT_INITIALIZED;
//
//     ApplyOrCancelChanges(This->pNet, &GUID_DEVCLASS_NET, TRUE);
//     ApplyOrCancelChanges(This->pClient, &GUID_DEVCLASS_NETCLIENT, TRUE);
//     ApplyOrCancelChanges(This->pService, &GUID_DEVCLASS_NETSERVICE, TRUE);
//     ApplyOrCancelChanges(This->pProtocol, &GUID_DEVCLASS_NETTRANS, TRUE);
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfg_fnCancel(
//     INetCfg * iface)
// {
//     INetCfgImpl *This = (INetCfgImpl *)iface;
//
//     if (!This->bInitialized)
//         return NETCFG_E_NOT_INITIALIZED;
//
//     ApplyOrCancelChanges(This->pClient, &GUID_DEVCLASS_NETCLIENT, FALSE);
//     ApplyOrCancelChanges(This->pService, &GUID_DEVCLASS_NETSERVICE, FALSE);
//     ApplyOrCancelChanges(This->pProtocol, &GUID_DEVCLASS_NETTRANS, FALSE);
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfg_fnEnumComponents(
//     INetCfg * iface,
//     const GUID *pguidClass,
//     IEnumNetCfgComponent **ppenumComponent)
// {
//     INetCfgImpl *This = (INetCfgImpl *)iface;
//
//     if (!This->bInitialized)
//         return NETCFG_E_NOT_INITIALIZED;
//
//     if (IsEqualGUID(&GUID_DEVCLASS_NET, pguidClass))
//         return IEnumNetCfgComponent_Constructor (NULL, &IID_IEnumNetCfgComponent, (LPVOID*)ppenumComponent, This->pNet, iface);
//     else if (IsEqualGUID(&GUID_DEVCLASS_NETCLIENT, pguidClass))
//         return IEnumNetCfgComponent_Constructor (NULL, &IID_IEnumNetCfgComponent, (LPVOID*)ppenumComponent, This->pClient, iface);
//     else if (IsEqualGUID(&GUID_DEVCLASS_NETSERVICE, pguidClass))
//         return IEnumNetCfgComponent_Constructor (NULL, &IID_IEnumNetCfgComponent, (LPVOID*)ppenumComponent, This->pService, iface);
//     else if (IsEqualGUID(&GUID_DEVCLASS_NETTRANS, pguidClass))
//         return IEnumNetCfgComponent_Constructor (NULL, &IID_IEnumNetCfgComponent, (LPVOID*)ppenumComponent, This->pProtocol, iface);
//     else
//        return E_NOINTERFACE;
// }
//
//
// HRESULT
// WINAPI
// INetCfg_fnFindComponent(
//     INetCfg * iface,
//     LPCWSTR pszwComponentId,
//     INetCfgComponent **pComponent)
// {
//     HRESULT hr;
//     INetCfgImpl *This = (INetCfgImpl *)iface;
//
//     if (!This->bInitialized)
//         return NETCFG_E_NOT_INITIALIZED;
//
//     hr = FindNetworkComponent(This->pClient, pszwComponentId, pComponent, iface);
//     if (hr == S_OK)
//         return hr;
//
//     hr = FindNetworkComponent(This->pService, pszwComponentId, pComponent, iface);
//     if (hr == S_OK)
//         return hr;
//
//     hr = FindNetworkComponent(This->pProtocol, pszwComponentId, pComponent, iface);
//     if (hr == S_OK)
//         return hr;
//
//     return S_FALSE;
// }
//
// HRESULT
// WINAPI
// INetCfg_fnQueryNetCfgClass(
//     INetCfg * iface,
//     const GUID *pguidClass,
//     REFIID riid,
//     void **ppvObject)
// {
//     return INetCfgClass_Constructor((IUnknown *)iface, riid, ppvObject, pguidClass, iface);
// }
//
// static const INetCfgVtbl vt_NetCfg =
// {
//     INetCfg_fnQueryInterface,
//     INetCfg_fnAddRef,
//     INetCfg_fnRelease,
//     INetCfg_fnInitialize,
//     INetCfg_fnUninitialize,
//     INetCfg_fnApply,
//     INetCfg_fnCancel,
//     INetCfg_fnEnumComponents,
//     INetCfg_fnFindComponent,
//     INetCfg_fnQueryNetCfgClass,
// };
//
// HRESULT WINAPI INetCfg_Constructor (IUnknown * pUnkOuter, REFIID riid, LPVOID * ppv)
// {
//     INetCfgImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (INetCfgImpl *) CoTaskMemAlloc(sizeof (INetCfgImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const INetCfg*)&vt_NetCfg;
//     This->lpVtblLock = (const INetCfgLock*)&vt_NetCfgLock;
//     This->lpVtblPnpReconfigCallback = (const INetCfgPnpReconfigCallback*)&vt_NetCfgPnpReconfigCallback;
//     This->hMutex = NULL;
//     This->bInitialized = FALSE;
//     This->pNet = NULL;
//     This->pClient = NULL;
//     This->pService = NULL;
//     This->pProtocol = NULL;
//
//     if (!SUCCEEDED (INetCfg_QueryInterface ((INetCfg*)This, riid, ppv)))
//     {
//         INetCfg_Release((INetCfg*)This);
//         return E_NOINTERFACE;
//     }
//
//     INetCfg_Release((INetCfg*)This);
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
