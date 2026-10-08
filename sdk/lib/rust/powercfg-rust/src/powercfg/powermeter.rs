// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_82294FD79A0D: [u16; 3] = [0x002Cu16, 0x0020, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_DEVCLASS_BATTERY: GUID = _GUID {
    Data1: 0x72631e54 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x78a4 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut SelectedBattery: UINT = 0 as UINT;

static mut hwndDlgDetail: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[repr(C)]
pub struct POWER_METER_INFO {
    pub Name: [WCHAR; 200],
    pub UniqueID: [WCHAR; 200],
    pub Manufacturer: [WCHAR; 200],
    pub Chem: [CHAR; 5],
    pub ACOnline: BOOL,
    pub Charging: BOOL,
    pub Critical: BOOL,
    pub BatteryLifePercent: BYTE,
    pub BatteryLifeTime: DWORD,
}

pub type PPOWER_METER_INFO = *mut POWER_METER_INFO;

unsafe extern "C" fn PowerMeterInfo_UpdateGlobalStats(ppmi: PPOWER_METER_INFO) {
    let mut sps: SYSTEM_POWER_STATUS = core::mem::zeroed();

    ZeroMemory!(ppmi as *mut core::ffi::c_void, core::mem::size_of::<POWER_METER_INFO>());

    if GetSystemPowerStatus(&mut sps as *mut SYSTEM_POWER_STATUS) != 0 {
        (*ppmi).ACOnline = ((sps.ACLineStatus as i32 != 0) as i32) as BOOL;
        (*ppmi).Charging = (((sps.BatteryFlag as u32 & BATTERY_FLAG_CHARGING) != 0) as i32) as BOOL;
        (*ppmi).Critical = (((sps.BatteryFlag as u32 & BATTERY_FLAG_CRITICAL) != 0) as i32) as BOOL;
        (*ppmi).BatteryLifePercent = sps.BatteryLifePercent;
        (*ppmi).BatteryLifeTime = sps.BatteryLifeTime;
    }
}

unsafe extern "C" fn PowerMeterInfo_UpdateBatteryStats(ppmi: PPOWER_METER_INFO, BatteryId: UINT) -> BOOL {
    let mut hDevInfo: HDEVINFO = core::ptr::null_mut();
    let mut InfoData: SP_DEVICE_INTERFACE_DATA = core::mem::zeroed();
    let mut dwSize: DWORD = 0;
    let mut InterfaceData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W = core::ptr::null_mut();
    let mut hDevice: HANDLE = core::ptr::null_mut();
    let mut dwWait: DWORD = 0;
    let mut dwReceived: DWORD = 0;
    let mut bqi: BATTERY_QUERY_INFORMATION = core::mem::zeroed();
    let mut bi: BATTERY_INFORMATION = core::mem::zeroed();
    let mut bws: BATTERY_WAIT_STATUS = core::mem::zeroed();
    let mut bs: BATTERY_STATUS = core::mem::zeroed();
    let file_name: *const ::core::ffi::c_char =
        b"dll/cpl/powercfg/powermeter.c\0".as_ptr() as *const ::core::ffi::c_char;

    ZeroMemory!(ppmi as *mut ::core::ffi::c_void, core::mem::size_of::<POWER_METER_INFO>());

    hDevInfo = SetupDiGetClassDevsW(
        core::ptr::addr_of_mut!(GUID_DEVCLASS_BATTERY) as *const _,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (DIGCF_PRESENT | DIGCF_INTERFACEDEVICE) as DWORD,
    );
    if hDevInfo == INVALID_HANDLE_VALUE {
        if DbgPrint(
            b"(%s:%d) SetupDiGetClassDevsW failed with %x\n\0".as_ptr() as PCSTR,
            file_name,
            70 as ::core::ffi::c_int,
            GetLastError(),
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                file_name,
                70 as ::core::ffi::c_int,
            );
        }
        return (FALSE) as BOOL;
    }

    InfoData.cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;
    if SetupDiEnumDeviceInterfaces(
        hDevInfo,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(GUID_DEVCLASS_BATTERY) as *const _,
        BatteryId as DWORD,
        core::ptr::addr_of_mut!(InfoData),
    ) == 0
    {
        if DbgPrint(
            b"(%s:%d) SetupDiEnumDeviceInterfaces failed with %x\n\0".as_ptr() as PCSTR,
            file_name,
            77 as ::core::ffi::c_int,
            GetLastError(),
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                file_name,
                77 as ::core::ffi::c_int,
            );
        }
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return (FALSE) as BOOL;
    }

    dwSize = 0;
    InterfaceData = core::ptr::null_mut();
    if SetupDiGetInterfaceDeviceDetailW(
        hDevInfo,
        core::ptr::addr_of_mut!(InfoData),
        InterfaceData,
        dwSize,
        core::ptr::addr_of_mut!(dwSize),
        core::ptr::null_mut(),
    ) == 0
    {
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            if DbgPrint(
                b"(%s:%d) SetupDiGetInterfaceDeviceDetailW failed with %x\n\0".as_ptr() as PCSTR,
                file_name,
                88 as ::core::ffi::c_int,
                GetLastError(),
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    file_name,
                    88 as ::core::ffi::c_int,
                );
            }
            SetupDiDestroyDeviceInfoList(hDevInfo);
            return (FALSE) as BOOL;
        }
    }
    InterfaceData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        dwSize as SIZE_T,
    ) as PSP_DEVICE_INTERFACE_DETAIL_DATA_W;
    if InterfaceData.is_null() {
        if DbgPrint(
            b"(%s:%d) HeapAlloc failed with %x\n\0".as_ptr() as PCSTR,
            file_name,
            96 as ::core::ffi::c_int,
            GetLastError(),
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                file_name,
                96 as ::core::ffi::c_int,
            );
        }
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return (FALSE) as BOOL;
    }
    (*InterfaceData).cbSize = core::mem::size_of::<_SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD;
    if SetupDiGetInterfaceDeviceDetailW(
        hDevInfo,
        core::ptr::addr_of_mut!(InfoData),
        InterfaceData,
        dwSize,
        core::ptr::addr_of_mut!(dwSize),
        core::ptr::null_mut(),
    ) == 0
    {
        if DbgPrint(
            b"(%s:%d) SetupDiGetInterfaceDeviceDetailW failed with %x\n\0".as_ptr() as PCSTR,
            file_name,
            103 as ::core::ffi::c_int,
            GetLastError(),
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                file_name,
                103 as ::core::ffi::c_int,
            );
        }
        HeapFree(
            GetProcessHeap(),
            0,
            InterfaceData as PVOID,
        );
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return (FALSE) as BOOL;
    }

    if DbgPrint(
        b"(%s:%d) Opening battery %S\n\0".as_ptr() as PCSTR,
        file_name,
        109 as ::core::ffi::c_int,
        core::ptr::addr_of_mut!((*InterfaceData).DevicePath) as *mut WCHAR,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            file_name,
            109 as ::core::ffi::c_int,
        );
    }
    hDevice = CreateFileW(
        core::ptr::addr_of_mut!((*InterfaceData).DevicePath) as *mut WCHAR as LPCWSTR,
        (GENERIC_READ | GENERIC_WRITE) as DWORD,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD,
        core::ptr::null_mut(),
        OPEN_EXISTING as DWORD,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        core::ptr::null_mut(),
    );
    if hDevice == INVALID_HANDLE_VALUE {
        if DbgPrint(
            b"(%s:%d) CreateFileW failed with %x\n\0".as_ptr() as PCSTR,
            file_name,
            115 as ::core::ffi::c_int,
            GetLastError(),
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                file_name,
                115 as ::core::ffi::c_int,
            );
        }
        HeapFree(
            GetProcessHeap(),
            0,
            InterfaceData as PVOID,
        );
        SetupDiDestroyDeviceInfoList(hDevInfo);
        return (FALSE) as BOOL;
    }

    dwWait = 0;
    if DeviceIoControl(
        hDevice,
        IOCTL_BATTERY_QUERY_TAG as DWORD,
        core::ptr::addr_of_mut!(dwWait) as LPVOID,
        core::mem::size_of::<DWORD>() as DWORD,
        core::ptr::addr_of_mut!(bqi.BatteryTag) as LPVOID,
        core::mem::size_of::<ULONG>() as DWORD,
        core::ptr::addr_of_mut!(dwReceived),
        core::ptr::null_mut(),
    ) != 0
    {
        bqi.InformationLevel = BatteryDeviceName;
        if DeviceIoControl(
            hDevice,
            IOCTL_BATTERY_QUERY_INFORMATION as DWORD,
            core::ptr::addr_of_mut!(bqi) as LPVOID,
            core::mem::size_of::<BATTERY_QUERY_INFORMATION>() as DWORD,
            core::ptr::addr_of_mut!((*ppmi).Name) as LPVOID,
            core::mem::size_of::<[WCHAR; 200]>() as DWORD,
            core::ptr::addr_of_mut!(dwReceived),
            core::ptr::null_mut(),
        ) != 0
        {
            // C writes `Name[dwReceived / sizeof(WCHAR)]`, ONE PAST the array when the driver fills it;
            // clamped to the last element (truncate, never corrupt the next field or panic).
            (*ppmi).Name[core::cmp::min((dwReceived as usize) / core::mem::size_of::<WCHAR>(), (*ppmi).Name.len() - 1)] =
                UNICODE_NULL;
        } else {
            (*ppmi).Name[0 as usize] = UNICODE_NULL;
        }

        bqi.InformationLevel = BatteryUniqueID;
        if DeviceIoControl(
            hDevice,
            IOCTL_BATTERY_QUERY_INFORMATION as DWORD,
            core::ptr::addr_of_mut!(bqi) as LPVOID,
            core::mem::size_of::<BATTERY_QUERY_INFORMATION>() as DWORD,
            core::ptr::addr_of_mut!((*ppmi).UniqueID) as LPVOID,
            core::mem::size_of::<[WCHAR; 200]>() as DWORD,
            core::ptr::addr_of_mut!(dwReceived),
            core::ptr::null_mut(),
        ) != 0
        {
            // C writes `UniqueID[dwReceived / sizeof(WCHAR)]`, ONE PAST the array when the driver fills it;
            // clamped to the last element (truncate, never corrupt the next field or panic).
            (*ppmi).UniqueID[core::cmp::min((dwReceived as usize) / core::mem::size_of::<WCHAR>(), (*ppmi).UniqueID.len() - 1)] =
                UNICODE_NULL;
        } else {
            (*ppmi).UniqueID[0 as usize] = UNICODE_NULL;
        }

        bqi.InformationLevel = BatteryInformation;
        if DeviceIoControl(
            hDevice,
            IOCTL_BATTERY_QUERY_INFORMATION as DWORD,
            core::ptr::addr_of_mut!(bqi) as LPVOID,
            core::mem::size_of::<BATTERY_QUERY_INFORMATION>() as DWORD,
            core::ptr::addr_of_mut!(bi) as LPVOID,
            core::mem::size_of::<BATTERY_INFORMATION>() as DWORD,
            core::ptr::addr_of_mut!(dwReceived),
            core::ptr::null_mut(),
        ) != 0
        {
            RtlCopyMemory!(
                core::ptr::addr_of_mut!((*ppmi).Chem) as *mut ::core::ffi::c_void,
                core::ptr::addr_of_mut!(bi.Chemistry) as *mut UCHAR as *const ::core::ffi::c_void,
                core::mem::size_of::<[UCHAR; 4]>() as size_t,
            );
            (*ppmi).Chem[4 as usize] = ANSI_NULL;
        } else {
            (*ppmi).Chem[0 as usize] = ANSI_NULL;
        }

        bws.BatteryTag = bqi.BatteryTag;
        if DeviceIoControl(
            hDevice,
            IOCTL_BATTERY_QUERY_STATUS as DWORD,
            core::ptr::addr_of_mut!(bws) as LPVOID,
            core::mem::size_of::<BATTERY_WAIT_STATUS>() as DWORD,
            core::ptr::addr_of_mut!(bs) as LPVOID,
            core::mem::size_of::<BATTERY_STATUS>() as DWORD,
            core::ptr::addr_of_mut!(dwReceived),
            core::ptr::null_mut(),
        ) != 0
        {
            (*ppmi).ACOnline = ((bs.PowerState & BATTERY_POWER_ON_LINE as ULONG) != 0) as ::core::ffi::c_int as BOOL;
            (*ppmi).Charging = (((bs.PowerState & BATTERY_CHARGING as ULONG) != 0)
                && ((bs.PowerState & BATTERY_DISCHARGING as ULONG) == 0))
                as ::core::ffi::c_int as BOOL;
            (*ppmi).Critical = ((bs.PowerState & BATTERY_CRITICAL as ULONG) != 0) as ::core::ffi::c_int as BOOL;
            // C divides by FullChargedCapacity unchecked; it is 0 when the information query
            // above failed (a fault on x86, 0 on ARM64). Reported as 0% instead of a panic.
            (*ppmi).BatteryLifePercent = if bi.FullChargedCapacity == 0 {
                0
            } else {
                (100 as ULONG).wrapping_mul(bs.Capacity) / bi.FullChargedCapacity
            } as BYTE;
            (*ppmi).BatteryLifeTime = BATTERY_LIFE_UNKNOWN as DWORD;
        }

        bqi.InformationLevel = BatteryEstimatedTime;
        if DeviceIoControl(
            hDevice,
            IOCTL_BATTERY_QUERY_INFORMATION as DWORD,
            core::ptr::addr_of_mut!(bqi) as LPVOID,
            core::mem::size_of::<BATTERY_QUERY_INFORMATION>() as DWORD,
            core::ptr::addr_of_mut!((*ppmi).BatteryLifeTime) as LPVOID,
            core::mem::size_of::<DWORD>() as DWORD,
            core::ptr::addr_of_mut!(dwReceived),
            core::ptr::null_mut(),
        ) == 0
        {
            (*ppmi).BatteryLifeTime = BATTERY_LIFE_UNKNOWN as DWORD; // == BATTERY_UNKNOWN_TIME;
        }

        bqi.InformationLevel = BatteryManufactureName;
        if DeviceIoControl(
            hDevice,
            IOCTL_BATTERY_QUERY_INFORMATION as DWORD,
            core::ptr::addr_of_mut!(bqi) as LPVOID,
            core::mem::size_of::<BATTERY_QUERY_INFORMATION>() as DWORD,
            core::ptr::addr_of_mut!((*ppmi).Manufacturer) as LPVOID,
            core::mem::size_of::<[WCHAR; 200]>() as DWORD,
            core::ptr::addr_of_mut!(dwReceived),
            core::ptr::null_mut(),
        ) != 0
        {
            // C writes `Manufacturer[dwReceived / sizeof(WCHAR)]`, ONE PAST the array when the driver fills it;
            // clamped to the last element (truncate, never corrupt the next field or panic).
            (*ppmi).Manufacturer[core::cmp::min((dwReceived as usize) / core::mem::size_of::<WCHAR>(), (*ppmi).Manufacturer.len() - 1)] =
                UNICODE_NULL;
        } else {
            (*ppmi).Manufacturer[0 as usize] = UNICODE_NULL;
        }
    }
    HeapFree(
        GetProcessHeap(),
        0,
        InterfaceData as PVOID,
    );
    SetupDiDestroyDeviceInfoList(hDevInfo);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn PowerMeterDetail_UpdateStats(hwndDlg: HWND) {
    let mut pmi: POWER_METER_INFO = core::mem::zeroed();
    let mut Status: [WCHAR; 200] = [0; 200];
    let mut Buffer: [WCHAR; 200] = [0; 200];

    PowerMeterInfo_UpdateBatteryStats(core::ptr::addr_of_mut!(pmi), SelectedBattery);

    SetDlgItemTextW(
        hwndDlg,
        (IDC_BATTERYNAME as i32),
        core::ptr::addr_of_mut!(pmi.Name) as *mut WCHAR as LPCWSTR,
    );
    SetDlgItemTextW(
        hwndDlg,
        (IDC_BATTERYUNIQUEID as i32),
        core::ptr::addr_of_mut!(pmi.UniqueID) as *mut WCHAR as LPCWSTR,
    );
    SetDlgItemTextA(
        hwndDlg,
        (IDC_BATTERYCHEMISTRY as i32),
        core::ptr::addr_of_mut!(pmi.Chem) as *mut CHAR as LPCSTR,
    );

    Status[0 as usize] = UNICODE_NULL;
    if pmi.ACOnline != 0 {
        if LoadString(
            hApplet,
            IDS_ONLINE as UINT,
            core::ptr::addr_of_mut!(Buffer) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 200]>()
                / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        ) != 0
        {
            wcscpy(
                core::ptr::addr_of_mut!(Status) as *mut WCHAR,
                core::ptr::addr_of_mut!(Buffer) as *mut WCHAR,
            );
        }
    }
    if pmi.Charging != 0 {
        if LoadString(
            hApplet,
            IDS_CHARGING as UINT,
            core::ptr::addr_of_mut!(Buffer) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 200]>()
                / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        ) != 0
        {
            if Status[0 as usize] != UNICODE_NULL {
                wcscat(
                    core::ptr::addr_of_mut!(Status) as *mut WCHAR,
                    RUSTCONV_WSTR_82294FD79A0D.as_ptr(),
                );
            }
            wcscat(
                core::ptr::addr_of_mut!(Status) as *mut WCHAR,
                core::ptr::addr_of_mut!(Buffer) as *mut WCHAR,
            );
        }
    } else {
        if LoadString(
            hApplet,
            IDS_DISCHARGING as UINT,
            core::ptr::addr_of_mut!(Buffer) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 200]>()
                / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        ) != 0
        {
            if Status[0 as usize] != UNICODE_NULL {
                wcscat(
                    core::ptr::addr_of_mut!(Status) as *mut WCHAR,
                    RUSTCONV_WSTR_82294FD79A0D.as_ptr(),
                );
            }
            wcscat(
                core::ptr::addr_of_mut!(Status) as *mut WCHAR,
                core::ptr::addr_of_mut!(Buffer) as *mut WCHAR,
            );
        }
    }
    // TODO BATTERY_CRITICAL
    SetDlgItemTextW(
        hwndDlg,
        (IDC_BATTERYPOWERSTATE as i32),
        core::ptr::addr_of_mut!(Status) as *mut WCHAR as LPCWSTR,
    );

    SetDlgItemTextW(
        hwndDlg,
        (IDC_BATTERYMANUFACTURER as i32),
        core::ptr::addr_of_mut!(pmi.Manufacturer) as *mut WCHAR as LPCWSTR,
    );
}

unsafe extern "C" fn PowerMeterDetail_InitDialog(hwndDlg: HWND) {
    let mut FormatBuffer: [WCHAR; 200] = [0; 200];
    let mut Buffer: [WCHAR; 200] = [0; 200];

    if LoadString(
        hApplet,
        IDS_DETAILEDBATTERY as UINT,
        core::ptr::addr_of_mut!(FormatBuffer) as *mut WCHAR as LPWSTR,
        _countof!(FormatBuffer) as core::ffi::c_int,
    ) != 0
    {
        StringCchPrintfW!(
            core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as STRSAFE_LPWSTR,
            _countof!(Buffer),
            core::ptr::addr_of_mut!(FormatBuffer) as *mut WCHAR as STRSAFE_LPCWSTR,
            SelectedBattery.wrapping_add(1 as UINT),
        );
        SetWindowTextW(
            hwndDlg,
            core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPCWSTR,
        );
    }
    PowerMeterDetail_UpdateStats(hwndDlg);
}

unsafe extern "C" fn CenterWindow(hWnd: HWND) {
    let mut hWndParent: HWND;
    let mut rcParent: RECT = core::mem::zeroed::<RECT>();
    let mut rcWindow: RECT = core::mem::zeroed::<RECT>();
    let mut size: SIZE = core::mem::zeroed::<SIZE>();
    let mut pos: POINT = core::mem::zeroed::<POINT>();
    let mut mi: MONITORINFO = core::mem::zeroed::<MONITORINFO>();
    let mut hMonitor: HMONITOR;

    hWndParent = GetParent(hWnd);
    if hWndParent.is_null() {
        hWndParent = GetDesktopWindow();
    }

    GetWindowRect(hWndParent, core::ptr::addr_of_mut!(rcParent));
    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rcWindow));
    size.cx = rcWindow.right - rcWindow.left;
    size.cy = rcWindow.bottom - rcWindow.top;

    mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    // hMonitor = MonitorFromRect(rcParent, MONITOR_DEFAULTTONEAREST);
    hMonitor = MonitorFromWindow(hWndParent, MONITOR_DEFAULTTONEAREST);
    GetMonitorInfoW(hMonitor, core::ptr::addr_of_mut!(mi));

    /* Center the window with respect to its parent. Formula:
     * with: pL = parent.left, pR = parent.right, w = window width,
     * X = pL + (pR - pL - w) / 2; and similar for Y, top, bottom, height. */
    pos.x = (rcParent.right + rcParent.left - size.cx) / 2;
    pos.y = (rcParent.bottom + rcParent.top - size.cy) / 2;

    /* Clip it to the monitor work area, so that top-left stays visible */
    pos.x = max(mi.rcWork.left, min(pos.x, mi.rcWork.right - size.cx));
    pos.y = max(mi.rcWork.top, min(pos.y, mi.rcWork.bottom - size.cy));

    SetWindowPos(hWnd, HWND_TOP, pos.x, pos.y, 0, 0, SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE);
}
#[no_mangle]
pub unsafe extern "system" fn PowerMeterDetailDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            PowerMeterDetail_InitDialog(hwndDlg);
            CenterWindow(hwndDlg);
            hwndDlgDetail = hwndDlg;
            return TRUE as INT_PTR;
        }
        WM_ACTIVATE => {
            /* Refresh the dialog on activation */
            if LOWORD!(wParam) as ::core::ffi::c_int != WA_INACTIVE as ::core::ffi::c_int {
                PowerMeterDetail_InitDialog(hwndDlg);
            }
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_REFRESH => {
                    PowerMeterDetail_InitDialog(hwndDlg);
                }
                IDOK | IDCANCEL => {
                    hwndDlgDetail = core::ptr::null_mut();
                    // DestroyWindow(hwndDlgDetail);
                    EndDialog(hwndDlg, 0 as INT_PTR);
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "system" fn PowerMeterDetailDlgThread(pParam: PVOID) -> DWORD {
    // HWND hwndDlg = (HWND)pParam;

    /* Use DialogBoxW() instead of CreateDialogW(), so that we can directly
     * use the default dialog message loop implementation furnished for modal
     * dialog boxes, instead of doing it ourselves for a modeless dialog.
     * But, set the parent to NULL so that we behave "like" a modeless dialog. */
    // hwndDlgDetail =
    DialogBoxW!(hApplet, MAKEINTRESOURCEW!(IDD_POWERMETERDETAILS), core::ptr::null_mut(), Some(PowerMeterDetailDlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR));
    return 0;
}

unsafe extern "C" fn GetBattIcon(mut hInst: HINSTANCE, mut ppmi: PPOWER_METER_INFO) -> HICON {
    let mut uIDIcon: UINT = IDI_BAT_UNKNOWN as UINT;

    // gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL].BatteryLevel
    if (*ppmi).Critical != 0
        || (0 <= (*ppmi).BatteryLifePercent as i32
            && ((*ppmi).BatteryLifePercent as i32) < 5)
    {
        // FIXME: Find which percentage is critical?
        uIDIcon = (if (*ppmi).Charging != 0 {
            IDI_BAT_CHARGE_CRIT
        } else {
            IDI_BAT_DISCHRG_CRIT
        }) as UINT;
    } else if 5 <= (*ppmi).BatteryLifePercent as i32
        && ((*ppmi).BatteryLifePercent as i32) < 33
    {
        uIDIcon = (if (*ppmi).Charging != 0 {
            IDI_BAT_CHARGE_LOW
        } else {
            IDI_BAT_DISCHRG_LOW
        }) as UINT;
    } else if 33 <= (*ppmi).BatteryLifePercent as i32
        && ((*ppmi).BatteryLifePercent as i32) < 66
    {
        uIDIcon = (if (*ppmi).Charging != 0 {
            IDI_BAT_CHARGE_HALF
        } else {
            IDI_BAT_DISCHRG_HALF
        }) as UINT;
    } else if 66 <= (*ppmi).BatteryLifePercent as i32
        && ((*ppmi).BatteryLifePercent as i32) < 100
    {
        uIDIcon = (if (*ppmi).Charging != 0 {
            IDI_BAT_CHARGE_HIGH
        } else {
            IDI_BAT_DISCHRG_HIGH
        }) as UINT;
    } else if (*ppmi).BatteryLifePercent as i32 >= 100 {
        uIDIcon = (if (*ppmi).Charging != 0 {
            IDI_BAT_CHARGE_FULL
        } else {
            IDI_BAT_DISCHRG_FULL
        }) as UINT;
    }

    return LoadIconW(hApplet, MAKEINTRESOURCEW!(uIDIcon));
}

unsafe extern "C" fn PowerMeter_InitDialog(hwndDlg: HWND) {
    let mut pmi: POWER_METER_INFO = core::mem::zeroed();
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut BatteryId: UINT = 0;
    let mut szPercent: [WCHAR; 5] = [0; 5];
    let mut Buffer: [WCHAR; 200] = [0; 200];

    *szPercent.as_mut_ptr() = UNICODE_NULL;
    LoadString(hApplet, IDS_PERCENT, szPercent.as_mut_ptr(), (_countof!(szPercent)) as i32);

    PowerMeterInfo_UpdateGlobalStats(core::ptr::addr_of_mut!(pmi));
    if pmi.ACOnline != 0 {
        hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDI_AC));
        SendDlgItemMessageW(hwndDlg, (IDC_IPOWERSOURCE as i32), STM_SETICON, hIcon as WPARAM, 0);

        if LoadString(hApplet, IDS_ONLINE, Buffer.as_mut_ptr(), (_countof!(Buffer)) as i32) != 0 {
            SetDlgItemTextW(hwndDlg, (IDC_POWERSOURCE as i32), Buffer.as_mut_ptr());
        }
    } else {
        hIcon = GetBattIcon(hApplet, core::ptr::addr_of_mut!(pmi));
        SendDlgItemMessageW(hwndDlg, (IDC_IPOWERSOURCE as i32), STM_SETICON, hIcon as WPARAM, 0);

        if LoadString(hApplet, IDS_OFFLINE, Buffer.as_mut_ptr(), (_countof!(Buffer)) as i32) != 0 {
            SetDlgItemTextW(hwndDlg, (IDC_POWERSOURCE as i32), Buffer.as_mut_ptr());
        }
    }
    StringCchPrintfW!(Buffer.as_mut_ptr(), _countof!(Buffer), szPercent.as_mut_ptr(), pmi.BatteryLifePercent as core::ffi::c_uint);
    SetDlgItemTextW(hwndDlg, (IDC_POWERSTATUS as i32), Buffer.as_mut_ptr());

    /* Check whether to show only the composite battery gauge, or the individual battery details */
    if IsDlgButtonChecked(hwndDlg, (IDC_SHOWDETAILS as i32)) != BST_CHECKED {
        /* Hide the individual battery controls */
        BatteryId = 0;
        while BatteryId < 8 {
            ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERY0 + BatteryId) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDI_BATTERYDETAIL0 + BatteryId) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERYPERCENT0 + BatteryId) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERYCHARGING0 + BatteryId) as i32), (SW_HIDE) as i32);
            BatteryId += 1;
        }
        ShowWindow(GetDlgItem(hwndDlg, (IDC_CLICKBATTINFO as i32)), (SW_HIDE) as i32);

        /* Show the composite battery gauge and set it up (0 to 100%) */
        ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTPROGRESS as i32)), (SW_SHOW) as i32);
        SendDlgItemMessageW(hwndDlg, (IDC_BATTPROGRESS as i32), PBM_SETRANGE, 0, MAKELPARAM!(0, 100));
        SendDlgItemMessageW(hwndDlg, (IDC_BATTPROGRESS as i32), PBM_SETPOS, min((pmi.BatteryLifePercent) as _, 100), 0);
        return;
    }

    /* Hide the composite battery gauge */
    ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTPROGRESS as i32)), (SW_HIDE) as i32);

    /* Show or hide the individual battery controls */
    ShowWindow(GetDlgItem(hwndDlg, (IDC_CLICKBATTINFO as i32)), (SW_SHOW) as i32);
    BatteryId = 0;
    while BatteryId < 8 {
        if PowerMeterInfo_UpdateBatteryStats(core::ptr::addr_of_mut!(pmi), BatteryId) == 0 {
            break;
        }

        ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERY0 + BatteryId) as i32), (SW_SHOW) as i32);
        ShowWindow(GetDlgItem(hwndDlg, (IDI_BATTERYDETAIL0 + BatteryId) as i32), (SW_SHOW) as i32);
        ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERYPERCENT0 + BatteryId) as i32), (SW_SHOW) as i32);
        ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERYCHARGING0 + BatteryId) as i32), (SW_SHOW) as i32);

        hIcon = GetBattIcon(hApplet, core::ptr::addr_of_mut!(pmi));
        SendDlgItemMessageW(hwndDlg, (IDI_BATTERYDETAIL0 + BatteryId) as i32, STM_SETICON, hIcon as WPARAM, 0);

        if pmi.Charging != 0 {
            if LoadString(hApplet, IDS_CHARGING, Buffer.as_mut_ptr(), (_countof!(Buffer)) as i32) != 0 {
                SetDlgItemTextW(hwndDlg, (IDC_BATTERYCHARGING0 + BatteryId) as i32, Buffer.as_mut_ptr());
            }
        } else {
            if LoadString(hApplet, IDS_DISCHARGING, Buffer.as_mut_ptr(), (_countof!(Buffer)) as i32) != 0 {
                SetDlgItemTextW(hwndDlg, (IDC_BATTERYCHARGING0 + BatteryId) as i32, Buffer.as_mut_ptr());
            }
        }
        StringCchPrintfW!(Buffer.as_mut_ptr(), _countof!(Buffer), szPercent.as_mut_ptr(), pmi.BatteryLifePercent as core::ffi::c_uint);
        SetDlgItemTextW(hwndDlg, (IDC_BATTERYPERCENT0 + BatteryId) as i32, Buffer.as_mut_ptr());
        BatteryId += 1;
    }
    while BatteryId < 8 {
        ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERY0 + BatteryId) as i32), (SW_HIDE) as i32);
        ShowWindow(GetDlgItem(hwndDlg, (IDI_BATTERYDETAIL0 + BatteryId) as i32), (SW_HIDE) as i32);
        ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERYPERCENT0 + BatteryId) as i32), (SW_HIDE) as i32);
        ShowWindow(GetDlgItem(hwndDlg, (IDC_BATTERYCHARGING0 + BatteryId) as i32), (SW_HIDE) as i32);
        BatteryId += 1;
    }
}
#[no_mangle]
pub unsafe extern "system" fn PowerMeterDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            PowerMeter_InitDialog(hwndDlg);
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            if (LOWORD!(wParam) as UINT) == IDC_SHOWDETAILS {
                PowerMeter_InitDialog(hwndDlg);
            } else if (LOWORD!(wParam) as UINT) >= IDI_BATTERYDETAIL0 && (LOWORD!(wParam) as UINT) <= IDI_BATTERYDETAIL7 {
                SelectedBattery = (LOWORD!(wParam) as UINT) - IDI_BATTERYDETAIL0;
                if IsWindow(hwndDlgDetail) == 0 {
                    let Thread: HANDLE = CreateThread(
                        core::ptr::null_mut(),
                        0,
                        Some(PowerMeterDetailDlgThread),
                        hwndDlg as PVOID,
                        0,
                        core::ptr::null_mut(),
                    );
                    if !Thread.is_null() {
                        CloseHandle(Thread);
                    }
                } else {
                    /* Activate the existing window */
                    ShowWindow(hwndDlgDetail, (SW_SHOWNA) as i32);
                    SwitchToThisWindow(hwndDlgDetail, (TRUE) as BOOL);
                }
            }
        }
        _ => {}
    }
    return (FALSE) as INT_PTR;
}
