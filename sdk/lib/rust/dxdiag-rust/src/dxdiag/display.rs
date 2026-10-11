use crate::dxdiag::d3dtest::D3DTests;
use crate::dxdiag::ddtest::DDTests;
use crate::dxdiag::dxdiag::EnableDialogTheme;
use crate::dxdiag::network::GetFileVersion;
use crate::dxdiag::system::GetRegValue;
use crate::dxdiag::dxdiag::InsertTabCtrlItem;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_37AD5F74A84E: [u16; 6] = [0x0025u16, 0x0073, 0x0020, 0x0025, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_5D7ED91F3A7A: [u16; 29] = [
    0x0048u16, 0x0061, 0x0072, 0x0064, 0x0077, 0x0061, 0x0072, 0x0065, 0x0049, 0x006E,
    0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x002E,
    0x0043, 0x0068, 0x0069, 0x0070, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_6098CEA026EF: [u16; 10] = [0x005Cu16, 0x0044, 0x0052, 0x0049, 0x0056, 0x0045, 0x0052, 0x0053, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_A05585F1D2A3: [u16; 28] = [
    0x0048u16, 0x0061, 0x0072, 0x0064, 0x0077, 0x0061, 0x0072, 0x0065, 0x0049, 0x006E,
    0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x002E,
    0x0044, 0x0061, 0x0063, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_EF3AC45CCF76: [u16; 31] = [
    0x0048u16, 0x0061, 0x0072, 0x0064, 0x0077, 0x0061, 0x0072, 0x0065, 0x0049, 0x006E,
    0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x002E,
    0x004D, 0x0065, 0x006D, 0x006F, 0x0072, 0x0079, 0x0053, 0x0069, 0x007A, 0x0065,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_DEVCLASS_DISPLAY: GUID = _GUID {
    Data1: 0x4d36e968 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe325 as ::core::ffi::c_ushort,
    Data3: 0x11ce as ::core::ffi::c_ushort,
    Data4: [
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut DISPLAY_GUID: GUID = _GUID {
    Data1: 0x67685559 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x3106 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};
#[no_mangle]
pub unsafe extern "C" fn GetFileModifyTime(pFullPath: LPCWSTR, szTime: *mut WCHAR, szTimeSize: core::ffi::c_int) -> BOOL {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut AccessTime: FILETIME = core::mem::zeroed();
    let mut SysTime: SYSTEMTIME = core::mem::zeroed();
    let mut LocalTime: SYSTEMTIME = core::mem::zeroed();
    let mut Length: UINT = 0;

    hFile = CreateFileW(pFullPath, GENERIC_READ, 0, core::ptr::null_mut(), OPEN_EXISTING, 0, core::ptr::null_mut());
    if hFile.is_null() {
        return (FALSE) as BOOL;
    }

    if GetFileTime(hFile, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(AccessTime)) == 0 {
        CloseHandle(hFile);
        return (FALSE) as BOOL;
    }
    CloseHandle(hFile);

    if FileTimeToSystemTime(core::ptr::addr_of_mut!(AccessTime), core::ptr::addr_of_mut!(SysTime)) == 0 {
        return (FALSE) as BOOL;
    }

    if SystemTimeToTzSpecificLocalTime(core::ptr::null_mut(), core::ptr::addr_of_mut!(SysTime), core::ptr::addr_of_mut!(LocalTime)) == 0 {
        return (FALSE) as BOOL;
    }

    Length = GetDateFormatW(LOCALE_USER_DEFAULT, DATE_SHORTDATE, core::ptr::addr_of_mut!(LocalTime), core::ptr::null_mut(), szTime, szTimeSize) as UINT;
    *szTime.offset(Length.wrapping_sub(1 as UINT) as isize) = 0x0020u16;
    return GetTimeFormatW(LOCALE_USER_DEFAULT, TIME_FORCE24HOURFORMAT, core::ptr::addr_of_mut!(LocalTime), core::ptr::null_mut(), szTime.offset(Length as isize), (szTimeSize as UINT).wrapping_sub(Length) as core::ffi::c_int);
}
unsafe extern "system" fn DriverFilesCallback(
    Context: PVOID,
    Notification: UINT,
    Param1: UINT_PTR,
    Param2: UINT_PTR,
) -> UINT {
    let mut pFile: LPCWSTR = core::ptr::null::<WCHAR>();
    let mut pBuffer: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let pFullPath: LPCWSTR = Param1 as LPCWSTR;
    let mut szVer: [WCHAR; 60] = [0; 60];
    let mut Length: LRESULT = 0;
    let mut fLength: LRESULT = 0;
    let hDlgCtrls: *mut HWND = Context as *mut HWND;

    if !wcsstr(pFullPath, RUSTCONV_WSTR_6098CEA026EF.as_ptr()).is_null() {
        /* exclude files from drivers dir to have failsafe file version/date information */
        return NO_ERROR as UINT;
    }

    pFile = wcsrchr(pFullPath, 0x005Cu16) as LPCWSTR;
    if pFile.is_null() {
        return NO_ERROR as UINT;
    }

    pFile = pFile.offset(1);
    fLength = (wcslen(pFile) as LRESULT) + (1 as LRESULT);

    Length = (SendMessageW(
        *hDlgCtrls.offset(0 as isize),
        WM_GETTEXTLENGTH as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as LRESULT) + (1 as LRESULT);

    pBuffer = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((Length + fLength) as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if pBuffer.is_null() {
        return ERROR_OUTOFMEMORY as UINT;
    }

    Length = SendMessageW(
        *hDlgCtrls.offset(0 as isize),
        WM_GETTEXT as UINT,
        Length as WPARAM,
        pBuffer as LPARAM,
    );
    if Length != 0 {
        let fresh32 = Length;
        Length += 1;
        *pBuffer.offset(fresh32 as isize) = 0x002Cu16;
    } else {
        /* set file version */
        if GetFileVersion(
            pFullPath,
            core::ptr::addr_of_mut!(szVer) as *mut WCHAR,
            (core::mem::size_of::<[WCHAR; 60]>()
                / core::mem::size_of::<WCHAR>()) as DWORD,
        ) != 0
        {
            SendMessageW(
                *hDlgCtrls.offset(1 as isize),
                WM_SETTEXT as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(szVer) as *mut WCHAR as LPARAM,
            );
        }
        /* set file time */
        if GetFileModifyTime(
            pFullPath,
            core::ptr::addr_of_mut!(szVer) as *mut WCHAR,
            (core::mem::size_of::<[WCHAR; 60]>()
                / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
        ) != 0
        {
            SendMessageW(
                *hDlgCtrls.offset(2 as isize),
                WM_SETTEXT as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(szVer) as *mut WCHAR as LPARAM,
            );
        }
    }

    wcscpy(pBuffer.offset(Length as isize), pFile);
    SendMessageW(
        *hDlgCtrls.offset(0 as isize),
        WM_SETTEXT as UINT,
        0 as WPARAM,
        pBuffer as LPARAM,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pBuffer as PVOID,
    );
    return NO_ERROR as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn EnumerateDrivers(mut Context: PVOID, mut hList: HDEVINFO, mut pInfoData: PSP_DEVINFO_DATA) {
    let mut hQueue: HSPFILEQ = core::ptr::null_mut();
    let mut DeviceInstallParams: SP_DEVINSTALL_PARAMS = core::mem::zeroed();
    let mut DriverInfoData: SP_DRVINFO_DATA = core::mem::zeroed();
    let mut Result: DWORD = 0;

    DeviceInstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS>() as DWORD;
    if SetupDiGetDeviceInstallParamsW(hList, pInfoData, core::ptr::addr_of_mut!(DeviceInstallParams)) == 0 {
        return;
    }

    DeviceInstallParams.FlagsEx |= (DI_FLAGSEX_INSTALLEDDRIVER | (DI_FLAGSEX_ALLOWEXCLUDEDDRVS as i32)) as DWORD;
    if SetupDiSetDeviceInstallParams(hList, pInfoData, core::ptr::addr_of_mut!(DeviceInstallParams)) == 0 {
        return;
    }

    if SetupDiBuildDriverInfoList(hList, pInfoData, SPDIT_CLASSDRIVER as DWORD) == 0 {
        return;
    }

    DriverInfoData.cbSize = core::mem::size_of::<SP_DRVINFO_DATA>() as DWORD;
    if SetupDiEnumDriverInfoW(
        hList,
        pInfoData,
        SPDIT_CLASSDRIVER as DWORD,
        0 as DWORD,
        core::ptr::addr_of_mut!(DriverInfoData),
    ) == 0
    {
        return;
    }

    DriverInfoData.cbSize = core::mem::size_of::<SP_DRVINFO_DATA>() as DWORD;
    if SetupDiSetSelectedDriverW(hList, pInfoData, core::ptr::addr_of_mut!(DriverInfoData)) == 0 {
        return;
    }

    hQueue = SetupOpenFileQueue();
    if hQueue == (INVALID_HANDLE_VALUE as HSPFILEQ) {
        return;
    }

    DeviceInstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS>() as DWORD;
    if SetupDiGetDeviceInstallParamsW(hList, pInfoData, core::ptr::addr_of_mut!(DeviceInstallParams)) == 0 {
        SetupCloseFileQueue(hQueue);
        return;
    }

    DeviceInstallParams.FileQueue = hQueue;
    DeviceInstallParams.Flags |= DI_NOVCP as DWORD;

    if SetupDiSetDeviceInstallParamsW(hList, pInfoData, core::ptr::addr_of_mut!(DeviceInstallParams)) == 0 {
        SetupCloseFileQueue(hQueue);
        return;
    }

    if SetupDiCallClassInstaller(DIF_INSTALLDEVICEFILES as DI_FUNCTION, hList, pInfoData) == 0 {
        SetupCloseFileQueue(hQueue);
        return;
    }

    /* enumerate the driver files */
    SetupScanFileQueueW(
        hQueue,
        SPQ_SCAN_USE_CALLBACK as DWORD,
        core::ptr::null_mut(),
        Some(DriverFilesCallback),
        Context,
        core::ptr::addr_of_mut!(Result),
    );
    SetupCloseFileQueue(hQueue);
}

unsafe extern "C" fn DisplayPageSetDeviceDetails(hDlgCtrls: *mut HWND, classGUID: LPCGUID, deviceGUID: *mut LPGUID) {
    let mut hInfo: HDEVINFO = core::ptr::null_mut();
    let mut dwIndex: DWORD = 0;
    let mut InfoData: SP_DEVINFO_DATA = core::mem::zeroed();
    let mut szText: [WCHAR; 100] = [0; 100];

    /* create the setup list */
    hInfo = SetupDiGetClassDevsW(
        classGUID as *const GUID,
        core::ptr::null(),
        core::ptr::null_mut(),
        (DIGCF_PRESENT | DIGCF_PROFILE) as DWORD,
    );
    if hInfo == INVALID_HANDLE_VALUE {
        return;
    }

    loop {
        ZeroMemory!(
            core::ptr::addr_of_mut!(InfoData) as PVOID,
            core::mem::size_of::<SP_DEVINFO_DATA>() as SIZE_T,
        );
        InfoData.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;

        if SetupDiEnumDeviceInfo(hInfo, dwIndex, core::ptr::addr_of_mut!(InfoData)) != 0 {
            /* set device name */
            if SetupDiGetDeviceRegistryPropertyW(
                hInfo,
                core::ptr::addr_of_mut!(InfoData),
                SPDRP_DEVICEDESC as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szText) as *mut WCHAR as PBYTE,
                core::mem::size_of::<[WCHAR; 100]>() as DWORD,
                core::ptr::null_mut(),
            ) != 0
            {
                SendMessageW(
                    *hDlgCtrls.offset(0 as isize),
                    WM_SETTEXT as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
                );
            }

            /* set the manufacturer name */
            if SetupDiGetDeviceRegistryPropertyW(
                hInfo,
                core::ptr::addr_of_mut!(InfoData),
                SPDRP_MFG as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szText) as *mut WCHAR as PBYTE,
                core::mem::size_of::<[WCHAR; 100]>() as DWORD,
                core::ptr::null_mut(),
            ) != 0
            {
                SendMessageW(
                    *hDlgCtrls.offset(1 as isize),
                    WM_SETTEXT as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
                );
            }

            /* FIXME
             * we currently enumerate only the first adapter
             */
            EnumerateDrivers(
                hDlgCtrls.offset(2 as isize) as PVOID,
                hInfo,
                core::ptr::addr_of_mut!(InfoData),
            );
            break;
        }

        if GetLastError() == ERROR_NO_MORE_ITEMS as DWORD {
            break;
        }

        dwIndex = dwIndex.wrapping_add(1);

        if !(TRUE != 0) {
            break;
        }
    }

    /* destroy the setup list */
    SetupDiDestroyDeviceInfoList(hInfo);
}

unsafe extern "C" fn InitializeDialog(hwndDlg: HWND, pDispDevice: PDISPLAY_DEVICEW) -> BOOL {
    let mut szText: [WCHAR; 100] = [0; 100];
    let mut szFormat: [WCHAR; 30] = [0; 30];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hDlgCtrls: [HWND; 5] = [core::ptr::null_mut(); 5];
    let mut dwMemory: DWORD = 0;
    let mut DevMode: DEVMODEW = core::mem::zeroed();
    let mut ppObj: *mut IDirect3D9 = core::ptr::null_mut();
    let mut Identifier: D3DADAPTER_IDENTIFIER9 = core::mem::zeroed();
    let mut hResult: HRESULT = 0;

    szText[0 as usize] = 0x0000u16;

    /* fix wine */
    //ppObj = Direct3DCreate9(D3D_SDK_VERSION);
    if !ppObj.is_null() {
        hResult = IDirect3D9_GetAdapterIdentifier(
            ppObj,
            D3DADAPTER_DEFAULT as UINT,
            2 as DWORD,
            core::ptr::addr_of_mut!(Identifier),
        );
        if hResult == D3D_OK {
            if Identifier.WHQLLevel != 0 {
                /* adapter is WHQL certified */
                LoadStringW(
                    hInst,
                    IDS_OPTION_YES as UINT,
                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                    (core::mem::size_of::<[WCHAR; 100]>() / core::mem::size_of::<WCHAR>())
                        as ::core::ffi::c_int,
                );
            } else {
                LoadStringW(
                    hInst,
                    IDS_NOT_APPLICABLE as UINT,
                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                    (core::mem::size_of::<[WCHAR; 100]>() / core::mem::size_of::<WCHAR>())
                        as ::core::ffi::c_int,
                );
            }
        }
        IDirect3D9_Release(ppObj);
    } else {
        LoadStringW(
            hInst,
            IDS_NOT_APPLICABLE as UINT,
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 100]>() / core::mem::size_of::<WCHAR>())
                as ::core::ffi::c_int,
        );
    }
    szText[((core::mem::size_of::<[WCHAR; 100]>() / core::mem::size_of::<WCHAR>())
        - (1 as usize))] = 0x0000u16;
    SendDlgItemMessageW(
        hwndDlg,
        IDC_STATIC_ADAPTER_LOGO as ::core::ffi::c_int,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
    );

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        (core::ptr::addr_of_mut!((*pDispDevice).DeviceKey) as *mut WCHAR)
            .offset(18 as isize) as *mut WCHAR as LPCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return (FALSE) as BOOL;
    }

    if GetRegValue(
        hKey,
        core::ptr::null_mut::<WCHAR>(),
        RUSTCONV_WSTR_5D7ED91F3A7A.as_ptr() as LPWSTR,
        REG_BINARY as DWORD,
        core::ptr::addr_of_mut!(szText) as LPWSTR,
        core::mem::size_of::<[WCHAR; 100]>() as DWORD,
    ) != 0
    {
        /* set chip type */
        SendDlgItemMessageW(
            hwndDlg,
            IDC_STATIC_ADAPTER_CHIP as ::core::ffi::c_int,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
        );
    }

    if GetRegValue(
        hKey,
        core::ptr::null_mut::<WCHAR>(),
        RUSTCONV_WSTR_A05585F1D2A3.as_ptr() as LPWSTR,
        REG_BINARY as DWORD,
        core::ptr::addr_of_mut!(szText) as LPWSTR,
        core::mem::size_of::<[WCHAR; 100]>() as DWORD,
    ) != 0
    {
        /* set DAC type */
        SendDlgItemMessageW(
            hwndDlg,
            IDC_STATIC_ADAPTER_DAC as ::core::ffi::c_int,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
        );
    }

    if GetRegValue(
        hKey,
        core::ptr::null_mut::<WCHAR>(),
        RUSTCONV_WSTR_EF3AC45CCF76.as_ptr() as LPWSTR,
        REG_BINARY as DWORD,
        core::ptr::addr_of_mut!(dwMemory) as LPWSTR,
        core::mem::size_of::<DWORD>() as DWORD,
    ) != 0
    {
        /* set chip memory size */
        if dwMemory > 1048576 as DWORD {
            /* buggy ATI driver requires that */
            dwMemory /= 1048576 as DWORD;
        }
        szFormat[0 as usize] = 0x0000u16;
        if LoadStringW(
            hInst,
            IDS_FORMAT_ADAPTER_MEM as UINT,
            core::ptr::addr_of_mut!(szFormat) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 30]>() / core::mem::size_of::<WCHAR>())
                as ::core::ffi::c_int,
        ) != 0
        {
            szFormat[((core::mem::size_of::<[WCHAR; 30]>() / core::mem::size_of::<WCHAR>())
                - (1 as usize))] = 0x0000u16;
        }
        wsprintfW(
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            core::ptr::addr_of_mut!(szFormat) as *mut WCHAR as LPCWSTR,
            dwMemory,
        );
        SendDlgItemMessageW(
            hwndDlg,
            IDC_STATIC_ADAPTER_MEM as ::core::ffi::c_int,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
        );
    }

    /* retrieve current display mode */
    DevMode.dmSize = core::mem::size_of::<DEVMODEW>() as WORD;
    if EnumDisplaySettingsW(
        (*pDispDevice).DeviceName.as_ptr() as LPCWSTR,
        ENUM_CURRENT_SETTINGS as DWORD,
        core::ptr::addr_of_mut!(DevMode),
    ) != 0
    {
        szFormat[0 as usize] = 0x0000u16;
        if LoadStringW(
            hInst,
            IDS_FORMAT_ADAPTER_MODE as UINT,
            core::ptr::addr_of_mut!(szFormat) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 30]>() / core::mem::size_of::<WCHAR>())
                as ::core::ffi::c_int,
        ) != 0
        {
            szFormat[((core::mem::size_of::<[WCHAR; 30]>() / core::mem::size_of::<WCHAR>())
                - (1 as usize))] = 0x0000u16;
        }
        wsprintfW(
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            core::ptr::addr_of_mut!(szFormat) as *mut WCHAR as LPCWSTR,
            DevMode.dmPelsWidth,
            DevMode.dmPelsHeight,
            DevMode.dmBitsPerPel,
            DevMode.dmDisplayFrequency,
        );
        SendDlgItemMessageW(
            hwndDlg,
            IDC_STATIC_ADAPTER_MODE as ::core::ffi::c_int,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
        );
    }

    /* query attached monitor */
    wcscpy(
        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
        (*pDispDevice).DeviceName.as_mut_ptr(),
    );
    ZeroMemory!(
        pDispDevice as *mut ::core::ffi::c_void,
        core::mem::size_of::<DISPLAY_DEVICEW>() as usize,
    );
    (*pDispDevice).cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
    if EnumDisplayDevicesW(
        core::ptr::addr_of_mut!(szText) as LPCWSTR,
        0 as DWORD,
        pDispDevice,
        0 as DWORD,
    ) != 0
    {
        /* set monitor name */
        SendDlgItemMessageW(
            hwndDlg,
            IDC_STATIC_ADAPTER_MONITOR as ::core::ffi::c_int,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!((*pDispDevice).DeviceString) as *mut WCHAR as LPARAM,
        );
    }

    hDlgCtrls[0 as usize] = GetDlgItem(hwndDlg, (IDC_STATIC_ADAPTER_ID as i32));
    hDlgCtrls[1 as usize] = GetDlgItem(hwndDlg, (IDC_STATIC_ADAPTER_VENDOR as i32));
    hDlgCtrls[2 as usize] = GetDlgItem(hwndDlg, (IDC_STATIC_ADAPTER_DRIVER as i32));
    hDlgCtrls[3 as usize] = GetDlgItem(hwndDlg, (IDC_STATIC_ADAPTER_VERSION as i32));
    hDlgCtrls[4 as usize] = GetDlgItem(hwndDlg, (IDC_STATIC_ADAPTER_DATE as i32));

    DisplayPageSetDeviceDetails(
        core::ptr::addr_of_mut!(hDlgCtrls) as *mut HWND,
        core::ptr::addr_of!(GUID_DEVCLASS_DISPLAY),
        core::ptr::null_mut(),
    );
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeDisplayAdapters(pContext: PDXDIAG_CONTEXT) {
    let mut DispDevice: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut pDisplayAdapters: *mut PDXDIAG_DISPLAY = core::ptr::null_mut();
    let mut pDisplayAdapter: PDXDIAG_DISPLAY = core::ptr::null_mut();
    let mut hwndDlg: HWND = core::ptr::null_mut();
    let mut szDisplay: [WCHAR; 20] = [0; 20];
    let mut szText: [WCHAR; 30] = [0; 30];
    let mut dwOffset: DWORD = 0;

    loop {
        ZeroMemory!(
            core::ptr::addr_of_mut!(DispDevice) as *mut core::ffi::c_void,
            core::mem::size_of::<DISPLAY_DEVICEW>(),
        );
        DispDevice.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
        if EnumDisplayDevicesW(
            core::ptr::null_mut(),
            ((*pContext).NumDisplayAdapter as DWORD).wrapping_add(dwOffset),
            core::ptr::addr_of_mut!(DispDevice),
            0,
        ) == 0
        {
            return;
        }

        /* skip devices not attached to the desktop and mirror drivers */
        if (DispDevice.StateFlags & (DISPLAY_DEVICE_ATTACHED_TO_DESKTOP as DWORD)) == 0
            || (DispDevice.StateFlags & (DISPLAY_DEVICE_MIRRORING_DRIVER as DWORD)) != 0
        {
            dwOffset = dwOffset.wrapping_add(1);
            continue;
        }
        if (*pContext).NumDisplayAdapter != 0 {
            pDisplayAdapters = HeapReAlloc(
                GetProcessHeap(),
                0 as DWORD,
                (*pContext).DisplayAdapters as PVOID,
                (((*pContext).NumDisplayAdapter as SIZE_T)
                    .wrapping_add(1 as SIZE_T))
                    .wrapping_mul(core::mem::size_of::<PDXDIAG_DISPLAY>() as SIZE_T),
            ) as *mut PDXDIAG_DISPLAY;
        } else {
            pDisplayAdapters = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                core::mem::size_of::<PDXDIAG_DISPLAY>() as SIZE_T,
            ) as *mut PDXDIAG_DISPLAY;
        }

        if pDisplayAdapters.is_null() {
            break;
        }

        pDisplayAdapter = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            core::mem::size_of::<DXDIAG_DISPLAY>() as SIZE_T,
        ) as PDXDIAG_DISPLAY;
        if pDisplayAdapter.is_null() {
            break;
        }

        (*pContext).DisplayAdapters = pDisplayAdapters;
        hwndDlg = CreateDialogParamW(
            hInst,
            MAKEINTRESOURCEW!(IDD_DISPLAY_DIALOG),
            (*pContext).hMainDialog,
            Some(DisplayPageWndProc),
            pDisplayAdapter as LPARAM,
        );
        EnableDialogTheme(hwndDlg);
        if hwndDlg.is_null() {
            break;
        }

        /* initialize the dialog */
        InitializeDialog(hwndDlg, core::ptr::addr_of_mut!(DispDevice));

        szDisplay[0 as usize] = 0x0000u16;
        LoadStringW(
            hInst,
            IDS_DISPLAY_DIALOG as UINT,
            core::ptr::addr_of_mut!(szDisplay) as *mut WCHAR,
            (core::mem::size_of::<[WCHAR; 20]>()
                / core::mem::size_of::<WCHAR>()) as i32,
        );
        szDisplay[(core::mem::size_of::<[WCHAR; 20]>()
            / core::mem::size_of::<WCHAR>())
            - (1 as usize)] = 0x0000u16;

        wsprintfW(
            core::ptr::addr_of_mut!(szText) as *mut WCHAR,
            RUSTCONV_WSTR_37AD5F74A84E.as_ptr(),
            core::ptr::addr_of_mut!(szDisplay) as *mut WCHAR,
            ((*pContext).NumDisplayAdapter as DWORD).wrapping_add(1 as DWORD),
        );
        InsertTabCtrlItem(
            GetDlgItem((*pContext).hMainDialog, IDC_TAB_CONTROL as i32),
            ((*pContext).NumDisplayAdapter as i32).wrapping_add(1 as i32),
            core::ptr::addr_of_mut!(szText) as *mut WCHAR,
        );

        (*pDisplayAdapter).guid = DISPLAY_GUID;
        (*pDisplayAdapter).guid.Data1 = (*pDisplayAdapter)
            .guid
            .Data1
            .wrapping_add(((*pContext).NumDisplayAdapter as DWORD).wrapping_add(dwOffset));
        (*pDisplayAdapter).hDisplayWnd = hwndDlg;
        *pDisplayAdapters.offset((*pContext).NumDisplayAdapter as isize) = pDisplayAdapter;
        (*pContext).NumDisplayAdapter = (*pContext).NumDisplayAdapter.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "system" fn DisplayPageWndProc(
    hDlg: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut rect: RECT = core::mem::zeroed();
    let mut hMainDialog: HWND = core::ptr::null_mut();
    let mut pDisplay: PDXDIAG_DISPLAY =
        GetWindowLongPtr(hDlg, DWLP_USER) as PDXDIAG_DISPLAY;
    match (message as i64) as u32 {
        WM_INITDIALOG => {
            pDisplay = lParam as PDXDIAG_DISPLAY;
            SetWindowLongPtr(hDlg, DWLP_USER, pDisplay as LONG_PTR);
            SetWindowPos(
                hDlg,
                core::ptr::null_mut(),
                10,
                32,
                0,
                0,
                (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT,
            );
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_BUTTON_TESTDD | IDC_BUTTON_TEST3D => {
                    hMainDialog = GetWindow(hDlg, GW_OWNER);
                    GetWindowRect(hMainDialog, core::ptr::addr_of_mut!(rect));
                    /* FIXME log result errors */
                    if IDC_BUTTON_TESTDD == (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                        DDTests(core::ptr::addr_of_mut!((*pDisplay).guid));
                    } else if IDC_BUTTON_TEST3D == (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                        D3DTests(core::ptr::addr_of_mut!((*pDisplay).guid));
                    }
                    SetWindowPos(
                        hMainDialog,
                        core::ptr::null_mut(),
                        rect.left as ::core::ffi::c_int,
                        rect.top as ::core::ffi::c_int,
                        rect.right as ::core::ffi::c_int,
                        rect.bottom as ::core::ffi::c_int,
                        (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT,
                    );
                }
                _ => {}
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
