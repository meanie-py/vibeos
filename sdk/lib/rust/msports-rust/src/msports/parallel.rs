// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_08A57A1154F4: [u16; 53] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0050, 0x0061, 0x0072, 0x0070, 0x006F, 0x0072,
    0x0074, 0x005C, 0x0050, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065,
    0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_3A02218A5789: [u16; 7] = [0x004Cu16, 0x0050, 0x0054, 0x0025, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_6FB7FC12E6B0: [u16; 21] = [
    0x0046u16, 0x0069, 0x006C, 0x0074, 0x0065, 0x0072, 0x0052, 0x0065, 0x0073, 0x006F,
    0x0075, 0x0072, 0x0063, 0x0065, 0x004D, 0x0065, 0x0074, 0x0068, 0x006F, 0x0064,
    0x0000,
];
pub static RUSTCONV_WSTR_70A1DA9E4693: [u16; 16] = [
    0x005Cu16, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x004C, 0x0050, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_AF854C3D2274: [u16; 8] = [0x0025u16, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000];
pub static RUSTCONV_WSTR_BB3BD7223382: [u16; 34] = [
    0x0048u16, 0x0061, 0x0072, 0x0064, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0044,
    0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x004D, 0x0061, 0x0070, 0x005C, 0x0050,
    0x0041, 0x0052, 0x0041, 0x004C, 0x004C, 0x0045, 0x004C, 0x0020, 0x0050, 0x004F,
    0x0052, 0x0054, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_C1D34563AF12: [u16; 9] = [0x0050u16, 0x006F, 0x0072, 0x0074, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_D532F3AD5672: [u16; 19] = [
    0x0050u16, 0x0061, 0x0072, 0x0045, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x004C,
    0x0065, 0x0067, 0x0061, 0x0063, 0x0079, 0x005A, 0x0069, 0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_E76BB3C8AF6F: [u16; 19] = [
    0x0050u16, 0x0061, 0x0072, 0x0061, 0x006C, 0x006C, 0x0065, 0x006C, 0x0020, 0x0050,
    0x006F, 0x0072, 0x0074, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct PORT_DATA {
    pub DeviceInfoSet: HDEVINFO,
    pub DeviceInfoData: PSP_DEVINFO_DATA,
    pub szPortName: [WCHAR; 16],
    pub dwPortNumber: DWORD,
    pub dwFilterResourceMethod: DWORD,
    pub dwLegacy: DWORD,
}

pub type PPORT_DATA = *mut PORT_DATA;

unsafe extern "C" fn GetUsedPorts(pPortMap: PDWORD) {
    let mut szDeviceName: [WCHAR; 64] = [0 as WCHAR; 64];
    let mut szDosDeviceName: [WCHAR; 64] = [0 as WCHAR; 64];
    let mut dwIndex: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwPortNumber: DWORD = 0;
    let mut dwDeviceNameSize: DWORD = 0;
    let mut dwDosDeviceNameSize: DWORD = 0;
    let mut ptr: PWSTR = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD = 0;

    *pPortMap = 0 as DWORD;

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        RUSTCONV_WSTR_BB3BD7223382.as_ptr(),
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) as DWORD;
    if dwError == ERROR_SUCCESS as DWORD {
        dwIndex = 0 as DWORD;
        loop {
            dwDeviceNameSize = ARRAYSIZE!(szDeviceName) as DWORD;
            dwDosDeviceNameSize = core::mem::size_of::<[WCHAR; 64]>() as DWORD;
            dwError = RegEnumValueW(
                hKey,
                dwIndex,
                core::ptr::addr_of_mut!(szDeviceName) as *mut WCHAR,
                core::ptr::addr_of_mut!(dwDeviceNameSize),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(dwType),
                core::ptr::addr_of_mut!(szDosDeviceName) as *mut WCHAR as LPBYTE,
                core::ptr::addr_of_mut!(dwDosDeviceNameSize),
            ) as DWORD;
            if dwError != ERROR_SUCCESS as DWORD {
                break;
            }

            if dwType == REG_SZ as DWORD {
                if _wcsnicmp(
                    core::ptr::addr_of_mut!(szDosDeviceName) as *mut WCHAR,
                    RUSTCONV_WSTR_70A1DA9E4693.as_ptr(),
                    wcslen(RUSTCONV_WSTR_70A1DA9E4693.as_ptr()) as usize,
                ) == 0 as _
                {
                    ptr = (core::ptr::addr_of_mut!(szDosDeviceName) as *mut WCHAR)
                        .add(wcslen(RUSTCONV_WSTR_70A1DA9E4693.as_ptr()) as usize);
                    if wcschr(ptr, 0x002Eu16).is_null() {
                        dwPortNumber = wcstoul(
                            ptr,
                            core::ptr::null_mut(),
                            10 as _,
                        ) as DWORD;
                        if dwPortNumber != 0 as DWORD {
                            *pPortMap |= (1 << dwPortNumber);
                        }
                    }
                }
            }
            dwIndex = dwIndex.wrapping_add(1);
        }

        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn ReadPortSettings(pPortData: PPORT_DATA) {
    let mut dwSize: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD;

    (*pPortData).dwFilterResourceMethod = 1; /* Never use an interrupt */
    (*pPortData).dwLegacy = 0;               /* Disabled */
    (*pPortData).dwPortNumber = 0;           /* Unknown */

    hKey = SetupDiOpenDevRegKey((*pPortData).DeviceInfoSet,
                                (*pPortData).DeviceInfoData,
                                DICS_FLAG_GLOBAL,
                                0,
                                DIREG_DEV,
                                KEY_READ);
    if hKey != (INVALID_HANDLE_VALUE).cast() {
        dwSize = core::mem::size_of::<[WCHAR; 16]>() as DWORD;
        dwError = (RegQueryValueExW(hKey,
                                   RUSTCONV_WSTR_C1D34563AF12.as_ptr(),
                                   core::ptr::null_mut(),
                                   core::ptr::null_mut(),
                                   core::ptr::addr_of_mut!((*pPortData).szPortName) as PBYTE,
                                   core::ptr::addr_of_mut!(dwSize))) as u32;
        if dwError != ERROR_SUCCESS {
            ERR!("RegQueryValueExW failed (Error %lu)\n", dwError);
        }

        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        dwError = (RegQueryValueExW(hKey,
                                   RUSTCONV_WSTR_6FB7FC12E6B0.as_ptr(),
                                   core::ptr::null_mut(),
                                   core::ptr::null_mut(),
                                   core::ptr::addr_of_mut!((*pPortData).dwFilterResourceMethod) as PBYTE,
                                   core::ptr::addr_of_mut!(dwSize))) as u32;
        if dwError != ERROR_SUCCESS {
            ERR!("RegQueryValueExW failed (Error %lu)\n", dwError);
        }

        RegCloseKey(hKey);
    }

    dwError = (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                            RUSTCONV_WSTR_08A57A1154F4.as_ptr(),
                            0,
                            KEY_READ,
                            core::ptr::addr_of_mut!(hKey))) as u32;
    if dwError == ERROR_SUCCESS {
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        dwError = (RegQueryValueExW(hKey,
                                   RUSTCONV_WSTR_D532F3AD5672.as_ptr(),
                                   core::ptr::null_mut(),
                                   core::ptr::null_mut(),
                                   core::ptr::addr_of_mut!((*pPortData).dwLegacy) as PBYTE,
                                   core::ptr::addr_of_mut!(dwSize))) as u32;
        if dwError != ERROR_SUCCESS {
            ERR!("RegQueryValueExW failed (Error %lu)\n", dwError);
        }

        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn ChangePortNumber(
    pPortData: PPORT_DATA,
    dwNewPortNumber: DWORD,
) -> DWORD {
    let mut szDeviceDescription: [WCHAR; 256] = core::mem::zeroed();
    let mut szFriendlyName: [WCHAR; 256] = core::mem::zeroed();
    let mut szNewPortName: [WCHAR; 16] = core::mem::zeroed();
    let mut hDeviceKey: HKEY = INVALID_HANDLE_VALUE as HKEY;
    let mut dwError: DWORD = 0;

    /* We are done if old and new port number are the same */
    if (*pPortData).dwPortNumber == dwNewPortNumber {
        return ERROR_SUCCESS as DWORD;
    }

    /* Build the new port name */
    _swprintf(
        core::ptr::addr_of_mut!(szNewPortName) as *mut WCHAR,
        RUSTCONV_WSTR_3A02218A5789.as_ptr(),
        dwNewPortNumber,
    );

    /* Open the devices hardware key */
    hDeviceKey = SetupDiOpenDevRegKey(
        (*pPortData).DeviceInfoSet,
        (*pPortData).DeviceInfoData,
        DICS_FLAG_GLOBAL as DWORD,
        0 as DWORD,
        DIREG_DEV as DWORD,
        (KEY_READ | KEY_WRITE) as REGSAM,
    );
    if hDeviceKey == INVALID_HANDLE_VALUE as HKEY {
        return GetLastError();
    }

    /* Set the new 'PortName' value */
    dwError = RegSetValueExW(
        hDeviceKey,
        RUSTCONV_WSTR_C1D34563AF12.as_ptr(),
        0 as DWORD,
        REG_SZ as DWORD,
        core::ptr::addr_of_mut!(szNewPortName) as *mut WCHAR as LPBYTE,
        ((wcslen(core::ptr::addr_of_mut!(szNewPortName) as *mut WCHAR) as DWORD)
            + 1 as DWORD)
            * (core::mem::size_of::<WCHAR>() as DWORD),
    ) as DWORD;
    'done: {
        if dwError != ERROR_SUCCESS as DWORD {
            break 'done;
        }

        /* Save the new port name and number */
        wcscpy(
            core::ptr::addr_of_mut!((*pPortData).szPortName) as *mut WCHAR,
            core::ptr::addr_of_mut!(szNewPortName) as *mut WCHAR,
        );
        (*pPortData).dwPortNumber = dwNewPortNumber;

        /* Get the device description... */
        if SetupDiGetDeviceRegistryProperty(
            (*pPortData).DeviceInfoSet,
            (*pPortData).DeviceInfoData,
            SPDRP_DEVICEDESC as DWORD,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(szDeviceDescription) as *mut WCHAR as PBYTE,
            core::mem::size_of::<[WCHAR; 256]>() as DWORD,
            core::ptr::null_mut::<DWORD>(),
        ) != 0
        {
            /* ... and use it to build a new friendly name */
            _swprintf(
                core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR,
                RUSTCONV_WSTR_AF854C3D2274.as_ptr(),
                core::ptr::addr_of_mut!(szDeviceDescription) as *mut WCHAR,
                core::ptr::addr_of_mut!(szNewPortName) as *mut WCHAR,
            );
        } else {
            /* ... or build a generic friendly name */
            _swprintf(
                core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR,
                RUSTCONV_WSTR_E76BB3C8AF6F.as_ptr(),
                core::ptr::addr_of_mut!(szNewPortName) as *mut WCHAR,
            );
        }

        /* Set the friendly name for the device */
        SetupDiSetDeviceRegistryProperty(
            (*pPortData).DeviceInfoSet,
            (*pPortData).DeviceInfoData,
            SPDRP_FRIENDLYNAME as DWORD,
            core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR as PBYTE,
            ((wcslen(core::ptr::addr_of_mut!(szFriendlyName) as *mut WCHAR) as DWORD)
                + 1 as DWORD)
                * (core::mem::size_of::<WCHAR>() as DWORD),
        );
    }

    if hDeviceKey != INVALID_HANDLE_VALUE as HKEY {
        RegCloseKey(hDeviceKey);
    }

    return dwError;
}

unsafe extern "C" fn WritePortSettings(hwnd: HWND, pPortData: PPORT_DATA) {
    let mut PropChangeParams: SP_PROPCHANGE_PARAMS = core::mem::zeroed();
    let mut dwDisposition: DWORD = 0;
    let mut dwFilterResourceMethod: DWORD = 0;
    let mut dwLegacy: DWORD = 0;
    let mut dwPortNumber: DWORD = 0;
    let mut dwPortMap: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut bChanged: BOOL = (FALSE as i32);
    let mut dwError: DWORD = 0;

    dwFilterResourceMethod = 1;
    if Button_GetCheck!(GetDlgItem(hwnd, (IDC_TRY_INTERRUPT as i32))) as u32 == BST_CHECKED {
        dwFilterResourceMethod = 0;
    } else if Button_GetCheck!(GetDlgItem(hwnd, (IDC_NEVER_INTERRUPT as i32))) as u32 == BST_CHECKED {
        dwFilterResourceMethod = 1;
    } else if Button_GetCheck!(GetDlgItem(hwnd, (IDC_ANY_INTERRUPT as i32))) as u32 == BST_CHECKED {
        dwFilterResourceMethod = 2;
    }

    if dwFilterResourceMethod != (*pPortData).dwFilterResourceMethod {
        hKey = SetupDiOpenDevRegKey(
            (*pPortData).DeviceInfoSet,
            (*pPortData).DeviceInfoData,
            DICS_FLAG_GLOBAL,
            0,
            DIREG_DEV,
            KEY_WRITE,
        );
        if hKey != (INVALID_HANDLE_VALUE).cast() {
            dwError = RegSetValueExW(
                hKey,
                RUSTCONV_WSTR_6FB7FC12E6B0.as_ptr(),
                0,
                REG_DWORD,
                core::ptr::addr_of_mut!(dwFilterResourceMethod) as PBYTE as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            ) as DWORD;
            RegCloseKey(hKey);
            if dwError == ERROR_SUCCESS {
                (*pPortData).dwFilterResourceMethod = dwFilterResourceMethod;
                bChanged = (TRUE as i32);
            }
        }
    }

    dwLegacy = 0;
    if Button_GetCheck!(GetDlgItem(hwnd, (IDC_PARALLEL_LEGACY as i32))) as u32 == BST_CHECKED {
        dwLegacy = 1;
    }

    if dwLegacy != (*pPortData).dwLegacy {
        dwError = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_08A57A1154F4.as_ptr(),
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hKey),
            core::ptr::addr_of_mut!(dwDisposition),
        ) as DWORD;
        if dwError == ERROR_SUCCESS {
            dwError = RegSetValueExW(
                hKey,
                RUSTCONV_WSTR_D532F3AD5672.as_ptr(),
                0,
                REG_DWORD,
                core::ptr::addr_of_mut!(dwLegacy) as LPBYTE as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            ) as DWORD;
            RegCloseKey(hKey);

            if dwError == ERROR_SUCCESS {
                (*pPortData).dwLegacy = dwLegacy;
                bChanged = (TRUE as i32);
            }
        }
    }

    dwPortNumber = ComboBox_GetCurSel!(GetDlgItem(hwnd, (IDC_PARALLEL_NAME as i32))) as DWORD;
    if dwPortNumber != (LB_ERR) as u32 {
        dwPortNumber = dwPortNumber.wrapping_add(1);
        if dwPortNumber != (*pPortData).dwPortNumber {
            GetUsedPorts(core::ptr::addr_of_mut!(dwPortMap));

            if (dwPortMap & ((1 as DWORD) << dwPortNumber)) != 0 {
                ERR!("Port LPT%lu is already in use!\n", dwPortNumber);
            } else {
                ChangePortNumber(pPortData, dwPortNumber);
                bChanged = (TRUE as i32);
            }
        }
    }

    if bChanged != 0 {
        /* Notify the system */
        PostMessageW(
            HWND_BROADCAST,
            WM_WININICHANGE,
            0,
            core::ptr::addr_of_mut!((*pPortData).szPortName) as LPARAM,
        );

        /* Notify the installer (and device) */
        PropChangeParams.ClassInstallHeader.cbSize = core::mem::size_of::<SP_CLASSINSTALL_HEADER>() as DWORD;
        PropChangeParams.ClassInstallHeader.InstallFunction = DIF_PROPERTYCHANGE;
        PropChangeParams.Scope = DICS_FLAG_GLOBAL;
        PropChangeParams.StateChange = DICS_PROPCHANGE;

        SetupDiSetClassInstallParams(
            (*pPortData).DeviceInfoSet,
            (*pPortData).DeviceInfoData,
            core::ptr::addr_of_mut!(PropChangeParams) as PSP_CLASSINSTALL_HEADER,
            core::mem::size_of::<SP_PROPCHANGE_PARAMS>() as DWORD,
        );

        SetupDiCallClassInstaller(
            DIF_PROPERTYCHANGE,
            (*pPortData).DeviceInfoSet,
            (*pPortData).DeviceInfoData,
        );
    }
}

unsafe extern "C" fn OnInitDialog(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    let mut szBuffer: [WCHAR; 256] = [0; 256];
    let mut szPortInUse: [WCHAR; 64] = [0; 64];
    let mut pPortData: PPORT_DATA;
    let mut hwndControl: HWND;
    let mut dwPortMap: DWORD = 0;
    let mut i: UINT;

    pPortData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("pPortData is NULL\n");
        return (FALSE) as BOOL;
    }

    SetWindowLongPtr(hwnd, DWLP_USER, pPortData as LONG_PTR);

    /* Read and parse the port settings */
    ReadPortSettings(pPortData);

    /* Set the 'Filter Resource Method' radiobuttons */
    hwndControl = GetDlgItem(
        hwnd,
        (IDC_TRY_INTERRUPT + (*pPortData).dwFilterResourceMethod) as i32,
    );
    if !hwndControl.is_null() {
        Button_SetCheck!(hwndControl, BST_CHECKED);
    }

    /* Disable the 'Enable legacy PNP detection' checkbox */
    hwndControl = GetDlgItem(hwnd, (IDC_PARALLEL_LEGACY as i32));
    if !hwndControl.is_null() {
        Button_SetCheck!(
            GetDlgItem(hwnd, (IDC_PARALLEL_LEGACY as i32)),
            if (*pPortData).dwLegacy != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            },
        );
    }

    LoadStringW(
        hInstance,
        IDS_PORT_IN_USE,
        szPortInUse.as_mut_ptr(),
        (ARRAYSIZE!(szPortInUse)) as i32,
    );

    /* Fill the 'LPT Port Number' combobox */
    hwndControl = GetDlgItem(hwnd, (IDC_PARALLEL_NAME as i32));
    if !hwndControl.is_null() {
        GetUsedPorts(core::ptr::addr_of_mut!(dwPortMap));

        i = 1;
        while i < 4 {
            _swprintf(szBuffer.as_mut_ptr(), RUSTCONV_WSTR_3A02218A5789.as_ptr(), i);

            if (dwPortMap & (1 << i)) != 0 && ((*pPortData).dwPortNumber != i) {
                wcscat(szBuffer.as_mut_ptr(), szPortInUse.as_mut_ptr());
            }

            ComboBox_AddString!(hwndControl, szBuffer.as_mut_ptr());

            if _wcsicmp((*pPortData).szPortName.as_mut_ptr(), szBuffer.as_mut_ptr())
                == 0
            {
                (*pPortData).dwPortNumber = i;
            }
            i = i.wrapping_add(1);
        }

        if (*pPortData).dwPortNumber != 0 {
            ComboBox_SetCurSel!(
                hwndControl,
                (*pPortData).dwPortNumber.wrapping_sub(1) as i32,
            );
        }
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn OnNotify(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) {
    let mut pPortData: PPORT_DATA;

    pPortData = GetWindowLongPtr(hwnd, DWLP_USER) as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("pPortData is NULL\n");
        return;
    }

    if (*(lParam as LPNMHDR)).code == (PSN_APPLY as u32) {
        WritePortSettings(hwnd, pPortData);
    }
}

unsafe extern "C" fn OnDestroy(hwnd: HWND) {
    let mut pPortData: PPORT_DATA;

    pPortData = GetWindowLongPtr(hwnd, DWLP_USER) as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("pPortData is NULL\n");
        return;
    }

    HeapFree(GetProcessHeap(), 0, pPortData as PVOID);
    SetWindowLongPtr(hwnd, DWLP_USER, core::ptr::null_mut::<u8>() as LONG_PTR);
}

unsafe extern "system" fn ParallelSettingsDlgProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            return OnInitDialog(hwnd, wParam, lParam) as INT_PTR;
        }
        WM_NOTIFY => {
            OnNotify(hwnd, wParam, lParam);
        }
        WM_DESTROY => {
            OnDestroy(hwnd);
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn ParallelPortPropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    let mut PropSheetPage: PROPSHEETPAGEW = core::mem::zeroed();
    let mut hPropSheetPage: HPROPSHEETPAGE = core::ptr::null_mut();
    let mut pPortData: PPORT_DATA = core::ptr::null_mut();

    pPortData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<PORT_DATA>() as SIZE_T,
    ) as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("Port data allocation failed!\n");
        return (FALSE) as BOOL;
    }

    (*pPortData).DeviceInfoSet = (*lpPropSheetPageRequest).DeviceInfoSet;
    (*pPortData).DeviceInfoData = (*lpPropSheetPageRequest).DeviceInfoData;

    if (*lpPropSheetPageRequest).PageRequested == SPPSR_ENUM_ADV_DEVICE_PROPERTIES as DWORD {
        PropSheetPage.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
        PropSheetPage.dwFlags = 0 as DWORD;
        PropSheetPage.hInstance = hInstance;
        PropSheetPage.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PARALLELSETTINGS) as LPCWSTR;
        PropSheetPage.pfnDlgProc = Some(ParallelSettingsDlgProc);
        PropSheetPage.lParam = pPortData as LPARAM;
        PropSheetPage.pfnCallback = None;

        hPropSheetPage = CreatePropertySheetPageW(
            core::ptr::addr_of_mut!(PropSheetPage) as LPCPROPSHEETPAGEW,
        );
        if hPropSheetPage.is_null() {
            ERR!("CreatePropertySheetPageW() failed!\n");
            return (FALSE) as BOOL;
        }

        if (lpfnAddPropSheetPageProc.unwrap_unchecked())(hPropSheetPage, lParam) == 0 {
            ERR!("lpfnAddPropSheetPageProc() failed!\n");
            DestroyPropertySheetPage(hPropSheetPage);
            return (FALSE) as BOOL;
        }
    }

    return (TRUE) as BOOL;
}
