// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_B9CB75831CC6: [u16; 4] = [0x002Cu16, 0x0009, 0x0022, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn PerformIO(
    hDevice: HANDLE,
    dwCtlCode: DWORD,
    lpBufferIn: LPVOID,
    dwBufferSizeIn: DWORD,
    lpBufferOut: LPVOID,
    dwBufferSizeOut: DWORD,
    lpNumberBytes: LPDWORD,
) -> DWORD {
    let mut overlapped: OVERLAPPED = core::mem::zeroed();
    let mut dwResult: DWORD;

    ZeroMemory!(
        core::ptr::addr_of_mut!(overlapped) as *mut core::ffi::c_void,
        core::mem::size_of::<OVERLAPPED>(),
    );
    overlapped.hEvent = CreateEvent(
        core::ptr::null_mut(),
        (TRUE as i32),
        (FALSE as i32),
        core::ptr::null(),
    );
    if overlapped.hEvent.is_null() {
        // failed to init event
        return GetLastError();
    }

    if DeviceIoControl(
        hDevice,
        dwCtlCode,
        lpBufferIn,
        dwBufferSizeIn,
        lpBufferOut,
        dwBufferSizeOut,
        lpNumberBytes,
        core::ptr::addr_of_mut!(overlapped),
    ) != 0
    {
        dwResult = ERROR_SUCCESS;
    } else if GetLastError() == ERROR_IO_PENDING {
        if GetOverlappedResult(
            hDevice,
            core::ptr::addr_of_mut!(overlapped),
            lpNumberBytes,
            (TRUE as i32),
        ) != 0
        {
            dwResult = ERROR_SUCCESS;
        } else {
            dwResult = GetLastError();
        }
    } else {
        dwResult = GetLastError();
    }
    CloseHandle(overlapped.hEvent);
    return dwResult;
}
#[no_mangle]
pub unsafe extern "C" fn InstallSoftwareDeviceInterface(
    mut DeviceId: LPGUID,
    mut InterfaceId: LPGUID,
    mut ReferenceString: LPWSTR,
) -> DWORD {
    let mut hDevInfo: HDEVINFO = core::ptr::null_mut();
    let mut DeviceInterfaceData: SP_DEVICE_INTERFACE_DATA = core::mem::zeroed();
    let mut SWBusGuid: GUID = GUID {
        Data1: 0x4747b320,
        Data2: 0x62ce,
        Data3: 0x11cf,
        Data4: [0xa5, 0xd6, 0x28, 0xdb, 0x04, 0xc1, 0x00, 0x00],
    };
    let mut DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W = core::ptr::null_mut();
    let mut hDevice: HANDLE = core::ptr::null_mut();
    let mut InstallInterface: PSWENUM_INSTALL_INTERFACE = core::ptr::null_mut();
    let mut dwResult: DWORD = 0;

    hDevInfo = SetupDiGetClassDevsW(
        core::ptr::addr_of_mut!(SWBusGuid),
        core::ptr::null(),
        core::ptr::null_mut(),
        DIGCF_DEVICEINTERFACE | DIGCF_PRESENT,
    );
    if hDevInfo.is_null() {
        // failed
        return GetLastError();
    }

    DeviceInterfaceData.cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;
    if SetupDiEnumDeviceInterfaces(
        hDevInfo,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(SWBusGuid),
        0,
        core::ptr::addr_of_mut!(DeviceInterfaceData),
    ) == 0
    {
        // failed
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return GetLastError();
    }

    DeviceInterfaceDetailData = HeapAlloc(
        GetProcessHeap(),
        0,
        ((MAX_PATH as usize) * core::mem::size_of::<WCHAR>()
            + core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>()) as SIZE_T,
    ) as PSP_DEVICE_INTERFACE_DETAIL_DATA_W;
    if DeviceInterfaceDetailData.is_null() {
        // failed
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return GetLastError();
    }

    (*DeviceInterfaceDetailData).cbSize =
        core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD;
    if SetupDiGetDeviceInterfaceDetailW(
        hDevInfo,
        core::ptr::addr_of_mut!(DeviceInterfaceData),
        DeviceInterfaceDetailData,
        ((MAX_PATH as usize) * core::mem::size_of::<WCHAR>()
            + core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>()) as DWORD,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        // failed
        HeapFree(
            GetProcessHeap(),
            0,
            DeviceInterfaceDetailData as PVOID,
        );
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return GetLastError();
    }

    hDevice = CreateFileW(
        core::ptr::addr_of_mut!((*DeviceInterfaceDetailData).DevicePath) as *mut WCHAR as LPCWSTR,
        GENERIC_READ | GENERIC_WRITE,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_OVERLAPPED | FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if hDevice == INVALID_HANDLE_VALUE {
        // failed
        HeapFree(
            GetProcessHeap(),
            0,
            DeviceInterfaceDetailData as PVOID,
        );
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return GetLastError();
    }

    InstallInterface = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<SWENUM_INSTALL_INTERFACE>() as SIZE_T).wrapping_add(
            (wcslen(ReferenceString as *const WCHAR) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ),
    ) as PSWENUM_INSTALL_INTERFACE;
    if InstallInterface.is_null() {
        // failed
        CloseHandle(hDevice);
        HeapFree(
            GetProcessHeap(),
            0,
            DeviceInterfaceDetailData as PVOID,
        );
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return GetLastError();
    }

    // init install interface param
    (*InstallInterface).DeviceId = *DeviceId;
    (*InstallInterface).InterfaceId = *InterfaceId;
    wcscpy(
        core::ptr::addr_of_mut!((*InstallInterface).ReferenceString) as *mut WCHAR,
        ReferenceString as *const WCHAR,
    );

    PerformIO(
        hDevice,
        (IOCTL_SWENUM_INSTALL_INTERFACE) as DWORD,
        InstallInterface as LPVOID,
        (core::mem::size_of::<SWENUM_INSTALL_INTERFACE>() as usize).wrapping_add(
            (wcslen(ReferenceString as *const WCHAR) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
        ) as DWORD,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    dwResult = HeapFree(
        GetProcessHeap(),
        0,
        InstallInterface as PVOID,
    ) as DWORD;

    CloseHandle(hDevice);
    HeapFree(
        GetProcessHeap(),
        0,
        DeviceInterfaceDetailData as PVOID,
    );
    SetupDiDestroyDeviceInfoList(hDevInfo);
    return dwResult;
}
#[no_mangle]
pub unsafe extern "C" fn InstallSoftwareDeviceInterfaceInf(
    mut InfName: LPWSTR,
    mut SectionName: LPWSTR,
) -> DWORD {
    let mut hDevInfo: HDEVINFO = core::ptr::null_mut();
    let mut hInf: HINF = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut DeviceInterfaceData: SP_DEVICE_INTERFACE_DATA = core::mem::zeroed();
    let mut SWBusGuid: GUID = GUID {
        Data1: 0x4747b320,
        Data2: 0x62ce,
        Data3: 0x11cf,
        Data4: [0xa5, 0xd6, 0x28, 0xdb, 0x04, 0xc1, 0, 0],
    };

    hDevInfo = SetupDiGetClassDevsW(
        core::ptr::addr_of_mut!(SWBusGuid),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    );
    if hDevInfo.is_null() {
        // failed
        return GetLastError();
    }

    DeviceInterfaceData.cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;
    if SetupDiEnumDeviceInterfaces(
        hDevInfo,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(SWBusGuid),
        0,
        core::ptr::addr_of_mut!(DeviceInterfaceData),
    ) == 0
    {
        // failed
        return GetLastError();
    }

    hInf = SetupOpenInfFileW(
        InfName,
        core::ptr::null_mut(),
        INF_STYLE_WIN4,
        core::ptr::null_mut(),
    );
    if hInf == INVALID_HANDLE_VALUE {
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return GetLastError();
    }

    //
    // FIXME check if interface is already installed
    //

    hKey = SetupDiCreateDeviceInterfaceRegKeyW(
        hDevInfo,
        core::ptr::addr_of_mut!(DeviceInterfaceData),
        0,
        KEY_ALL_ACCESS,
        hInf,
        SectionName,
    );

    SetupCloseInfFile(hInf);
    SetupDiDestroyDeviceInfoList(hDevInfo);
    if hKey != (INVALID_HANDLE_VALUE).cast() {
        RegCloseKey(hKey);
    }
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn StreamingDeviceSetupW(
    hwnd: HWND,
    hinst: HINSTANCE,
    lpszCmdLine: LPWSTR,
    nCmdShow: i32,
) {
    let mut Length: DWORD = 0;
    let mut dwResult: DWORD = 0;
    let mut pCmdLine: LPWSTR = core::ptr::null_mut();
    let mut pStr: LPWSTR = core::ptr::null_mut();
    let mut Guids: [GUID; 2] = core::mem::zeroed();
    let mut DevicePath: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
    let mut hResult: HRESULT = 0;
    let mut Index: DWORD = 0;

    Length = ((wcslen(lpszCmdLine) as DWORD) + (1 as DWORD))
        * (core::mem::size_of::<WCHAR>() as DWORD);

    pCmdLine = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        Length as SIZE_T,
    ) as LPWSTR;
    if pCmdLine.is_null() {
        // no memory
        return;
    }

    hResult = StringCbCopyExW(
        pCmdLine,
        Length as usize,
        lpszCmdLine,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        STRSAFE_NULL_ON_FAILURE as DWORD,
    );
    if hResult != S_OK as HRESULT {
        // failed
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pCmdLine as LPVOID,
        );
        return;
    }

    pStr = wcstok(pCmdLine as *mut u16, RUSTCONV_WSTR_B9CB75831CC6.as_ptr() as *const u16)
        as LPWSTR;
    Index = 0 as DWORD;
    loop {
        if pStr.is_null() {
            // invalid parameter
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pCmdLine as LPVOID,
            );
            return;
        }

        hResult = IIDFromString(
            pStr as *const u16,
            (core::ptr::addr_of_mut!(Guids) as *mut GUID).offset(Index as isize) as *mut GUID,
        );
        if hResult != S_OK as HRESULT {
            // invalid parameter
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pCmdLine as LPVOID,
            );
            return;
        }

        Index = Index.wrapping_add(1);
        pStr = wcstok(core::ptr::null_mut(), RUSTCONV_WSTR_B9CB75831CC6.as_ptr() as *const u16)
            as LPWSTR;

        if !(Index < 2 as DWORD) {
            break;
        }
    }

    dwResult = InstallSoftwareDeviceInterface(
        (core::ptr::addr_of_mut!(Guids) as *mut GUID).offset(0 as isize)
            as LPGUID,
        (core::ptr::addr_of_mut!(Guids) as *mut GUID).offset(1 as isize)
            as LPGUID,
        pStr,
    );
    if dwResult == ERROR_SUCCESS as DWORD {
        pStr = wcstok(core::ptr::null_mut(), RUSTCONV_WSTR_B9CB75831CC6.as_ptr() as *const u16)
            as LPWSTR;
        if !pStr.is_null() {
            wcscpy(
                core::ptr::addr_of_mut!(DevicePath) as *mut u16,
                pStr as *const u16,
            );
            pStr = wcstok(core::ptr::null_mut(), RUSTCONV_WSTR_B9CB75831CC6.as_ptr() as *const u16)
                as LPWSTR;
            if !pStr.is_null() {
                dwResult = InstallSoftwareDeviceInterfaceInf(
                    core::ptr::addr_of_mut!(DevicePath) as *mut u16,
                    pStr,
                );
            }
        }
    }

    let _ = (hwnd, hinst, nCmdShow, dwResult);
}
