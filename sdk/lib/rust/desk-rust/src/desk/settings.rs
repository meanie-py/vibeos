// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _SETTINGS_DATA {
    pub DisplayDeviceList: PDISPLAY_DEVICE_ENTRY,
    pub CurrentDisplayDevice: PDISPLAY_DEVICE_ENTRY,
    pub hSpectrumBitmaps: [HBITMAP; (NUM_SPECTRUM_BITMAPS) as usize],
    pub cxSource: [i32; (NUM_SPECTRUM_BITMAPS) as usize],
    pub cySource: [i32; (NUM_SPECTRUM_BITMAPS) as usize],
}
pub type SETTINGS_DATA = _SETTINGS_DATA;
pub type PSETTINGS_DATA = *mut _SETTINGS_DATA;

#[repr(C)]
pub struct _TIMEOUTDATA {
    pub szRawBuffer: [TCHAR; 256],
    pub szCookedBuffer: [TCHAR; 256],
    pub nTimeout: INT,
}
pub type TIMEOUTDATA = _TIMEOUTDATA;
pub type PTIMEOUTDATA = *mut _TIMEOUTDATA;
unsafe extern "C" fn UpdateDisplay(hwndDlg: HWND, pData: PSETTINGS_DATA, bUpdateThumb: BOOL) {
    let mut Buffer: [TCHAR; 64] = core::mem::zeroed();
    let mut Pixel: [TCHAR; 64] = core::mem::zeroed();
    let mut index: DWORD;
    let hwndMonSel: HWND;
    let mut info: MONSL_MONINFO = core::mem::zeroed();

    LoadString(hApplet, IDS_PIXEL, Pixel.as_mut_ptr(), (_countof!(Pixel)) as i32);
    _stprintf(Buffer.as_mut_ptr(), Pixel.as_ptr(), (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsWidth, (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsHeight, Pixel.as_ptr());
    SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_RESOLUTION_TEXT) as i32, WM_SETTEXT, 0, Buffer.as_ptr() as LPARAM);

    index = 0;
    while index < (*(*pData).CurrentDisplayDevice).ResolutionsCount {
        if (*(*(*pData).CurrentDisplayDevice).Resolutions.offset(index as isize)).dmPelsWidth == (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsWidth &&
           (*(*(*pData).CurrentDisplayDevice).Resolutions.offset(index as isize)).dmPelsHeight == (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsHeight
        {
            if bUpdateThumb != 0 {
                (SendDlgItemMessage) as i32(hwndDlg, (IDC_SETTINGS_RESOLUTION) as i32, TBM_SETPOS, (TRUE) as u64, (index) as i64);
            }
            break;
        }
        index = index + 1;
    }

    if LoadString(hApplet, (2900 + (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmBitsPerPel), Buffer.as_mut_ptr(), (_countof!(Buffer)) as i32) != 0 {
        SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_BPP) as i32, CB_SELECTSTRING, (-1 as i32) as WPARAM, Buffer.as_ptr() as LPARAM);
    }

    hwndMonSel = GetDlgItem(hwndDlg, (IDC_SETTINGS_MONSEL) as i32);
    index = SendMessage(hwndMonSel, MSLM_GETCURSEL, 0, 0) as INT as DWORD;
    if index != (-1 as i32) as DWORD && SendMessage(hwndMonSel, MSLM_GETMONITORINFO, index, core::ptr::addr_of_mut!(info) as LPARAM) != 0 {
        info.Size.cx = ((*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsWidth) as i32;
        info.Size.cy = ((*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsHeight) as i32;
        SendMessage(hwndMonSel, MSLM_SETMONITORINFO, index, core::ptr::addr_of_mut!(info) as LPARAM);
    }
}
unsafe extern "C" fn CompareSettings(
    Entry: PSETTINGS_ENTRY,
    dmPelsWidth: DWORD,
    dmPelsHeight: DWORD,
    dmBitsPerPel: DWORD,
    dmDisplayFrequency: DWORD,
) -> i32 {
    if (*Entry).dmPelsWidth == dmPelsWidth
        && (*Entry).dmPelsHeight == dmPelsHeight
        && (*Entry).dmBitsPerPel == dmBitsPerPel
        && (*Entry).dmDisplayFrequency == dmDisplayFrequency
    {
        return 0;
    } else if ((*Entry).dmPelsWidth < dmPelsWidth)
        || ((*Entry).dmPelsWidth == dmPelsWidth && (*Entry).dmPelsHeight < dmPelsHeight)
        || ((*Entry).dmPelsWidth == dmPelsWidth
            && (*Entry).dmPelsHeight == dmPelsHeight
            && (*Entry).dmBitsPerPel < dmBitsPerPel)
    {
        return 1;
    }
    return -1;
}
unsafe extern "C" fn GetPossibleSettings(
    DeviceName: LPCTSTR,
    pSettingsCount: *mut DWORD,
    CurrentSettings: *mut PSETTINGS_ENTRY,
) -> PSETTINGS_ENTRY {
    let mut devmode: DEVMODE = core::mem::zeroed();
    let mut NbSettings: DWORD = 0;
    let mut iMode: DWORD = 0;
    let dwFlags: DWORD = 0;
    let mut Settings: PSETTINGS_ENTRY = core::ptr::null_mut();
    let hDC: HDC;
    let mut Current: PSETTINGS_ENTRY;
    let bpp: DWORD;
    let xres: DWORD;
    let yres: DWORD;
    let curDispFreq: DWORD;

    *CurrentSettings = core::ptr::null_mut();
    hDC = CreateIC(core::ptr::null_mut(), DeviceName, core::ptr::null_mut(), core::ptr::null_mut());
    bpp = (GetDeviceCaps(hDC, (PLANES) as i32)
        * GetDeviceCaps(hDC, (BITSPIXEL) as i32)) as u32;
    xres = (GetDeviceCaps(hDC, (HORZRES) as i32)) as u32;
    yres = (GetDeviceCaps(hDC, (VERTRES) as i32)) as u32;
    DeleteDC(hDC);

    devmode.dmSize = core::mem::size_of::<DEVMODE>() as WORD;
    devmode.dmDriverExtra = 0;

    if EnumDisplaySettingsEx(DeviceName, ENUM_CURRENT_SETTINGS, &mut devmode, dwFlags) == 0 {
        return core::ptr::null_mut();
    }

    curDispFreq = devmode.dmDisplayFrequency;

    while EnumDisplaySettingsEx(DeviceName, iMode, &mut devmode, dwFlags) != 0 {
        iMode += 1;

        if devmode.dmPelsWidth < 640
            || devmode.dmPelsHeight < 480
            || devmode.dmDisplayFrequency != curDispFreq
            || (devmode.dmBitsPerPel != 4
                && devmode.dmBitsPerPel != 8
                && devmode.dmBitsPerPel != 16
                && devmode.dmBitsPerPel != 24
                && devmode.dmBitsPerPel != 32)
        {
            continue;
        }

        Current = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<SETTINGS_ENTRY>()) as SIZE_T,
        ) as PSETTINGS_ENTRY;
        if !Current.is_null() {
            let mut Previous: PSETTINGS_ENTRY = core::ptr::null_mut();
            let mut Next: PSETTINGS_ENTRY = Settings;
            (*Current).dmPelsWidth = devmode.dmPelsWidth;
            (*Current).dmPelsHeight = devmode.dmPelsHeight;
            (*Current).dmBitsPerPel = devmode.dmBitsPerPel;
            (*Current).dmDisplayFrequency = devmode.dmDisplayFrequency;
            while !Next.is_null()
                && CompareSettings(
                    Next,
                    devmode.dmPelsWidth,
                    devmode.dmPelsHeight,
                    devmode.dmBitsPerPel,
                    devmode.dmDisplayFrequency,
                ) > 0
            {
                Previous = Next;
                Next = (*Next).Flink;
            }
            (*Current).Blink = Previous;
            (*Current).Flink = Next;
            if Previous.is_null() {
                Settings = Current;
            } else {
                (*Previous).Flink = Current;
            }
            if !Next.is_null() {
                (*Next).Blink = Current;
            }
            if devmode.dmPelsWidth == xres && devmode.dmPelsHeight == yres && devmode.dmBitsPerPel == bpp {
                *CurrentSettings = Current;
            }
            NbSettings += 1;
        }
    }

    *pSettingsCount = NbSettings;
    Settings
}
unsafe extern "C" fn AddDisplayDevice(
    pData: PSETTINGS_DATA,
    DisplayDevice: *const DISPLAY_DEVICE,
) -> BOOL {
    let mut newEntry: PDISPLAY_DEVICE_ENTRY = core::ptr::null_mut();
    let mut description: LPTSTR = core::ptr::null_mut();
    let mut name: LPTSTR = core::ptr::null_mut();
    let mut key: LPTSTR = core::ptr::null_mut();
    let mut devid: LPTSTR = core::ptr::null_mut();
    let mut descriptionSize: SIZE_T;
    let mut nameSize: SIZE_T;
    let mut keySize: SIZE_T;
    let mut devidSize: SIZE_T;
    let mut Current: PSETTINGS_ENTRY;
    let mut ResolutionsCount: DWORD = 1;
    let mut i: DWORD;

    'ByeBye: loop {
        newEntry = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<DISPLAY_DEVICE_ENTRY>()) as SIZE_T) as PDISPLAY_DEVICE_ENTRY;
        if newEntry.is_null() {
            break 'ByeBye;
        }

        (*newEntry).Settings = GetPossibleSettings(
            (*DisplayDevice).DeviceName.as_ptr(),
            &mut (*newEntry).SettingsCount,
            &mut (*newEntry).CurrentSettings,
        );
        if (*newEntry).Settings.is_null() {
            break 'ByeBye;
        }

        (*newEntry).InitialSettings.dmPelsWidth = (*(*newEntry).CurrentSettings).dmPelsWidth;
        (*newEntry).InitialSettings.dmPelsHeight = (*(*newEntry).CurrentSettings).dmPelsHeight;
        (*newEntry).InitialSettings.dmBitsPerPel = (*(*newEntry).CurrentSettings).dmBitsPerPel;
        (*newEntry).InitialSettings.dmDisplayFrequency = (*(*newEntry).CurrentSettings).dmDisplayFrequency;

        Current = (*newEntry).Settings;
        while !Current.is_null() {
            if !(*Current).Flink.is_null()
                && (((*Current).dmPelsWidth != (*(*Current).Flink).dmPelsWidth)
                    || ((*Current).dmPelsHeight != (*(*Current).Flink).dmPelsHeight))
            {
                ResolutionsCount = ResolutionsCount.wrapping_add(1);
            }
            Current = (*Current).Flink;
        }

        (*newEntry).Resolutions = HeapAlloc(
            GetProcessHeap(),
            0,
            ((ResolutionsCount as usize) * core::mem::size_of::<RESOLUTION_INFO>()) as SIZE_T,
        ) as *mut RESOLUTION_INFO;
        if (*newEntry).Resolutions.is_null() {
            break 'ByeBye;
        }

        (*newEntry).ResolutionsCount = ResolutionsCount;

        Current = (*newEntry).Settings;
        i = 0;
        while !Current.is_null() {
            if (*Current).Flink.is_null()
                || (!(*Current).Flink.is_null()
                    && (((*Current).dmPelsWidth != (*(*Current).Flink).dmPelsWidth)
                        || ((*Current).dmPelsHeight != (*(*Current).Flink).dmPelsHeight)))
            {
                (*(*newEntry).Resolutions.offset(i as isize)).dmPelsWidth = (*Current).dmPelsWidth;
                (*(*newEntry).Resolutions.offset(i as isize)).dmPelsHeight = (*Current).dmPelsHeight;
                i = i.wrapping_add(1);
            }
            Current = (*Current).Flink;
        }

        descriptionSize = ((_tcslen((*DisplayDevice).DeviceString.as_ptr()) + 1)
            * core::mem::size_of::<TCHAR>()) as u32;
        description = HeapAlloc(GetProcessHeap(), 0, descriptionSize) as LPTSTR;
        if description.is_null() {
            break 'ByeBye;
        }

        nameSize = ((_tcslen((*DisplayDevice).DeviceName.as_ptr()) + 1)
            * core::mem::size_of::<TCHAR>()) as u32;
        name = HeapAlloc(GetProcessHeap(), 0, nameSize) as LPTSTR;
        if name.is_null() {
            break 'ByeBye;
        }

        keySize = ((_tcslen((*DisplayDevice).DeviceKey.as_ptr()) + 1)
            * core::mem::size_of::<TCHAR>()) as u32;
        key = HeapAlloc(GetProcessHeap(), 0, keySize) as LPTSTR;
        if key.is_null() {
            break 'ByeBye;
        }

        devidSize = ((_tcslen((*DisplayDevice).DeviceID.as_ptr()) + 1)
            * core::mem::size_of::<TCHAR>()) as u32;
        devid = HeapAlloc(GetProcessHeap(), 0, devidSize) as LPTSTR;
        if devid.is_null() {
            break 'ByeBye;
        }

        memcpy(description as *mut core::ffi::c_void, (*DisplayDevice).DeviceString.as_ptr() as *const core::ffi::c_void, (descriptionSize) as usize);
        memcpy(name as *mut core::ffi::c_void, (*DisplayDevice).DeviceName.as_ptr() as *const core::ffi::c_void, (nameSize) as usize);
        memcpy(key as *mut core::ffi::c_void, (*DisplayDevice).DeviceKey.as_ptr() as *const core::ffi::c_void, (keySize) as usize);
        memcpy(devid as *mut core::ffi::c_void, (*DisplayDevice).DeviceID.as_ptr() as *const core::ffi::c_void, (devidSize) as usize);
        (*newEntry).DeviceDescription = (description).cast();
        (*newEntry).DeviceName = (name).cast();
        (*newEntry).DeviceKey = (key).cast();
        (*newEntry).DeviceID = (devid).cast();
        (*newEntry).DeviceStateFlags = (*DisplayDevice).StateFlags;
        (*newEntry).Flink = (*pData).DisplayDeviceList;
        (*pData).DisplayDeviceList = newEntry;
        return (TRUE) as i32;
    }

    if !newEntry.is_null() {
        if !(*newEntry).Settings.is_null() {
            Current = (*newEntry).Settings;
            while !Current.is_null() {
                let Next: PSETTINGS_ENTRY = (*Current).Flink;
                HeapFree(GetProcessHeap(), 0, Current as *mut core::ffi::c_void);
                Current = Next;
            }
        }
        if !(*newEntry).Resolutions.is_null() {
            HeapFree(GetProcessHeap(), 0, (*newEntry).Resolutions as *mut core::ffi::c_void);
        }
        HeapFree(GetProcessHeap(), 0, newEntry as *mut core::ffi::c_void);
    }
    if !description.is_null() {
        HeapFree(GetProcessHeap(), 0, description as *mut core::ffi::c_void);
    }
    if !name.is_null() {
        HeapFree(GetProcessHeap(), 0, name as *mut core::ffi::c_void);
    }
    if !key.is_null() {
        HeapFree(GetProcessHeap(), 0, key as *mut core::ffi::c_void);
    }
    (FALSE) as i32
}
unsafe extern "C" fn OnDisplayDeviceChanged(hwndDlg: HWND, pData: PSETTINGS_DATA, pDeviceEntry: PDISPLAY_DEVICE_ENTRY) {
    let mut Current: PSETTINGS_ENTRY;
    let mut index: u32;

    (*pData).CurrentDisplayDevice = pDeviceEntry;

    SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_BPP) as i32, CB_RESETCONTENT, 0, 0);
    Current = (*pDeviceEntry).Settings;
    while !Current.is_null() {
        let mut Buffer: [u16; 64] = [0; 64];
        if LoadString(hApplet, (2900 + (*Current).dmBitsPerPel), Buffer.as_mut_ptr(), (_countof!(Buffer)) as i32) != 0 {
            index = SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_BPP) as i32, CB_FINDSTRINGEXACT, (-1 as i32) as WPARAM, Buffer.as_ptr() as LPARAM) as u32;
            if index == CB_ERR as u32 {
                index = SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_BPP) as i32, CB_ADDSTRING, 0, Buffer.as_ptr() as LPARAM) as u32;
                SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_BPP) as i32, CB_SETITEMDATA, index as WPARAM, (*Current).dmBitsPerPel as LPARAM);
            }
        }
        Current = (*Current).Flink;
    }

    SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_DEVICE) as i32, WM_SETTEXT, 0, (*pDeviceEntry).DeviceDescription as LPARAM);

    SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_RESOLUTION) as i32, TBM_CLEARTICS, (TRUE) as u32, 0);
    SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_RESOLUTION) as i32, TBM_SETRANGE, (TRUE) as u32, MAKELONG!(0, (*pDeviceEntry).ResolutionsCount - 1) as LPARAM);

    UpdateDisplay(hwndDlg, pData, (TRUE) as BOOL);
}
unsafe extern "C" fn SettingsOnInitDialog(hwndDlg: HWND) {
    let mut bitmap: BITMAP = core::mem::zeroed();
    let mut Result: u32 = 0;
    let mut iDevNum: u32 = 0;
    let mut i: u32;
    let mut displayDevice: DISPLAY_DEVICE = core::mem::zeroed();
    let pData: PSETTINGS_DATA;

    pData = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<SETTINGS_DATA>()) as SIZE_T) as PSETTINGS_DATA;
    if pData.is_null() {
        return;
    }

    SetWindowLongPtr(hwndDlg, DWLP_USER, (pData as isize) as i32);

    (*pData).DisplayDeviceList = core::ptr::null_mut();
    displayDevice.cb = core::mem::size_of::<DISPLAY_DEVICE>() as u32;
    while EnumDisplayDevices(core::ptr::null_mut(), iDevNum, &mut displayDevice, EDD_GET_DEVICE_INTERFACE_NAME) != 0 {
        if (displayDevice.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) != 0 {
            if AddDisplayDevice(pData, &displayDevice) != 0 {
                Result += 1;
            }
        }
        iDevNum += 1;
    }

    if Result == 0 {
        EnableWindow(GetDlgItem(hwndDlg, (IDC_SETTINGS_BPP) as i32), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_SETTINGS_RESOLUTION) as i32), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_SETTINGS_RESOLUTION_TEXT) as i32), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_SETTINGS_ADVANCED) as i32), (FALSE) as BOOL);
        ShowWindow(GetDlgItem(hwndDlg, (IDC_SETTINGS_SPECTRUM) as i32), (SW_HIDE) as i32);

        memset(
            (*pData).hSpectrumBitmaps.as_mut_ptr() as *mut core::ffi::c_void,
            0,
            core::mem::size_of_val(&(*pData).hSpectrumBitmaps),
        );
        return;
    } else if Result == 1 {
        let mut monitors: MONSL_MONINFO = core::mem::zeroed();

        OnDisplayDeviceChanged(hwndDlg, pData, (*pData).DisplayDeviceList);

        ShowWindow(GetDlgItem(hwndDlg, (IDC_SETTINGS_MONTEXT) as i32), (SW_HIDE) as i32);
        ShowWindow(GetDlgItem(hwndDlg, (IDC_SETTINGS_MONSEL) as i32), (SW_HIDE) as i32);

        monitors.Position.x = 0;
        monitors.Position.y = 0;
        monitors.Size.cx = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsWidth as i32;
        monitors.Size.cy = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsHeight as i32;
        monitors.Flags = 0;
        SendDlgItemMessage(
            hwndDlg,
            (IDC_SETTINGS_MONSEL) as i32,
            MSLM_SETMONITORSINFO,
            1,
            &monitors as *const _ as LPARAM,
        );
    } else {
        let pMonitors: PMONSL_MONINFO;

        OnDisplayDeviceChanged(hwndDlg, pData, (*pData).DisplayDeviceList);

        ShowWindow(GetDlgItem(hwndDlg, (IDC_RESOLUTION_PREVIEW) as i32), (SW_HIDE) as i32);

        pMonitors = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<MONSL_MONINFO>() * (Result) as usize) as SIZE_T,
        ) as PMONSL_MONINFO;
        if !pMonitors.is_null() {
            let hack: u32 = 1280;
            i = 0;
            while i < Result {
                (*pMonitors.offset(i as isize)).Position.x = (hack * i) as i32;
                (*pMonitors.offset(i as isize)).Position.y = 0;
                (*pMonitors.offset(i as isize)).Size.cx = (*(*(*pData).DisplayDeviceList).CurrentSettings).dmPelsWidth as i32;
                (*pMonitors.offset(i as isize)).Size.cy = (*(*(*pData).DisplayDeviceList).CurrentSettings).dmPelsHeight as i32;
                (*pMonitors.offset(i as isize)).Flags = 0;
                i += 1;
            }

            SendDlgItemMessage(
                hwndDlg,
                (IDC_SETTINGS_MONSEL) as i32,
                MSLM_SETMONITORSINFO,
                Result as WPARAM,
                pMonitors as LPARAM,
            );

            HeapFree(GetProcessHeap(), 0, pMonitors as *mut core::ffi::c_void);
        }
    }

    i = 0;
    while i < NUM_SPECTRUM_BITMAPS {
        (*pData).hSpectrumBitmaps[i as usize] = LoadImageW(
            hApplet,
            MAKEINTRESOURCEW!(IDB_SPECTRUM_4 + i),
            IMAGE_BITMAP,
            0,
            0,
            LR_DEFAULTCOLOR,
        ) as HBITMAP;

        if !(*pData).hSpectrumBitmaps[i as usize].is_null() {
            if GetObjectW(
                (*pData).hSpectrumBitmaps[i as usize] as HGDIOBJ,
                core::mem::size_of::<BITMAP>() as i32,
                &mut bitmap as *mut _ as *mut core::ffi::c_void,
            ) != 0
            {
                (*pData).cxSource[i as usize] = bitmap.bmWidth;
                (*pData).cySource[i as usize] = bitmap.bmHeight;
            } else {
                (*pData).cxSource[i as usize] = 0;
                (*pData).cySource[i as usize] = 0;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn ShowResolutionPreview(draw: LPDRAWITEMSTRUCT, pData: PSETTINGS_DATA) {
    let mut hBrush: HBRUSH;
    let hDC: HDC;
    let hOldObj: HGDIOBJ;
    let mut rcItem: RECT = RECT {
        left: (MONITOR_LEFT) as i32,
        top: (MONITOR_TOP) as i32,
        right: (MONITOR_RIGHT) as i32,
        bottom: (MONITOR_BOTTOM) as i32,
    };

    hDC = CreateCompatibleDC((*draw).hDC);
    hOldObj = SelectObject(hDC, (g_GlobalData.hMonitorBitmap).cast());

    hBrush = CreateSolidBrush(g_GlobalData.desktop_color);
    FillRect(hDC, &mut rcItem, hBrush);
    DeleteObject((hBrush).cast());

    GdiTransparentBlt(
        (*draw).hDC,
        (*draw).rcItem.left,
        (*draw).rcItem.top,
        (*draw).rcItem.right - (*draw).rcItem.left + 1,
        (*draw).rcItem.bottom - (*draw).rcItem.top + 1,
        hDC,
        0,
        0,
        g_GlobalData.bmMonWidth,
        g_GlobalData.bmMonHeight,
        MONITOR_ALPHA,
    );

    SelectObject(hDC, hOldObj);
    DeleteDC(hDC);
}
unsafe extern "C" fn ShowColorSpectrum(
    hDC: HDC,
    client: LPRECT,
    BitsPerPel: u32,
    pData: *mut SETTINGS_DATA,
) {
    let hdcMem: HDC;
    let iBitmap: i32;

    hdcMem = CreateCompatibleDC(hDC);

    if hdcMem.is_null() {
        return;
    }

    match BitsPerPel {
        4 => {
            iBitmap = 0;
        }
        8 => {
            iBitmap = 1;
        }
        _ => {
            iBitmap = 2;
        }
    }

    if SelectObject(
        hdcMem,
        ((*pData).hSpectrumBitmaps[iBitmap as usize]).cast(),
    ) != (0) as *mut c_void
    {
        StretchBlt(
            hDC,
            (*client).left,
            (*client).top,
            (*client).right - (*client).left,
            (*client).bottom - (*client).top,
            hdcMem,
            0,
            0,
            (*pData).cxSource[iBitmap as usize],
            (*pData).cySource[iBitmap as usize],
            SRCCOPY,
        );
    }

    DeleteDC(hdcMem);
}
unsafe extern "C" fn OnBPPChanged(hwndDlg: HWND, pData: PSETTINGS_DATA) {
    let mut Current: PSETTINGS_ENTRY;
    let dmNewBitsPerPel: u32;
    let index: u32;
    let hSpectrumDC: HDC;
    let hSpectrumControl: HWND;
    let mut client: RECT = core::mem::zeroed();

    index = SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_BPP) as i32, CB_GETCURSEL, 0, 0) as u32;
    dmNewBitsPerPel = SendDlgItemMessage(hwndDlg, (IDC_SETTINGS_BPP) as i32, CB_GETITEMDATA, index, 0) as u32;

    hSpectrumControl = GetDlgItem(hwndDlg, (IDC_SETTINGS_SPECTRUM) as i32);
    hSpectrumDC = GetDC(hSpectrumControl);
    if hSpectrumDC == core::ptr::null_mut() {
        return;
    }

    GetClientRect(hSpectrumControl, &mut client);
    ShowColorSpectrum(hSpectrumDC, &mut client, dmNewBitsPerPel, pData);
    ReleaseDC(hSpectrumControl, hSpectrumDC);

    Current = (*(*pData).CurrentDisplayDevice).CurrentSettings;
    if dmNewBitsPerPel == (*Current).dmBitsPerPel {
        return;
    }

    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);

    if dmNewBitsPerPel < (*Current).dmBitsPerPel {
        Current = (*Current).Blink;
        while Current != core::ptr::null_mut() {
            if (*Current).dmBitsPerPel == dmNewBitsPerPel
                && (*Current).dmPelsHeight == (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsHeight
                && (*Current).dmPelsWidth == (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsWidth
            {
                (*(*pData).CurrentDisplayDevice).CurrentSettings = Current;
                UpdateDisplay(hwndDlg, pData, (TRUE) as BOOL);
                return;
            }
            Current = (*Current).Blink;
        }
    } else {
        Current = (*Current).Flink;
        while Current != core::ptr::null_mut() {
            if (*Current).dmBitsPerPel == dmNewBitsPerPel
                && (*Current).dmPelsHeight == (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsHeight
                && (*Current).dmPelsWidth == (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsWidth
            {
                (*(*pData).CurrentDisplayDevice).CurrentSettings = Current;
                UpdateDisplay(hwndDlg, pData, (TRUE) as BOOL);
                return;
            }
            Current = (*Current).Flink;
        }
    }

    Current = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).Blink;
    while Current != core::ptr::null_mut() {
        if (*Current).dmBitsPerPel == dmNewBitsPerPel {
            (*(*pData).CurrentDisplayDevice).CurrentSettings = Current;
            UpdateDisplay(hwndDlg, pData, (TRUE) as BOOL);
            return;
        }
        Current = (*Current).Blink;
    }

    Current = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).Flink;
    while Current != core::ptr::null_mut() {
        if (*Current).dmBitsPerPel == dmNewBitsPerPel {
            (*(*pData).CurrentDisplayDevice).CurrentSettings = Current;
            UpdateDisplay(hwndDlg, pData, (TRUE) as BOOL);
            return;
        }
        Current = (*Current).Flink;
    }
}
unsafe extern "C" fn OnResolutionChanged(
    hwndDlg: HWND,
    pData: PSETTINGS_DATA,
    NewPosition: DWORD,
    bUpdateThumb: BOOL,
) {
    let mut Current: PSETTINGS_ENTRY;
    let dmNewPelsHeight = (*(*(*pData).CurrentDisplayDevice).Resolutions.offset(NewPosition as isize)).dmPelsHeight;
    let dmNewPelsWidth = (*(*(*pData).CurrentDisplayDevice).Resolutions.offset(NewPosition as isize)).dmPelsWidth;
    let dmBitsPerPel: DWORD;
    let dmDisplayFrequency: DWORD;

    Current = (*(*pData).CurrentDisplayDevice).CurrentSettings;
    if dmNewPelsHeight == (*Current).dmPelsHeight && dmNewPelsWidth == (*Current).dmPelsWidth {
        return;
    }

    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
    InvalidateRect(GetDlgItem(hwndDlg, (IDC_RESOLUTION_PREVIEW) as i32), core::ptr::null_mut(), (TRUE) as BOOL);

    dmBitsPerPel = (*Current).dmBitsPerPel;
    dmDisplayFrequency = (*Current).dmDisplayFrequency;

    if CompareSettings(Current, dmNewPelsWidth, dmNewPelsHeight, dmBitsPerPel, dmDisplayFrequency) < 0 {
        Current = (*Current).Blink;
        while !Current.is_null() {
            if (*Current).dmPelsHeight == dmNewPelsHeight
                && (*Current).dmPelsWidth == dmNewPelsWidth
                && (*Current).dmBitsPerPel == dmBitsPerPel
            {
                (*(*pData).CurrentDisplayDevice).CurrentSettings = Current;
                UpdateDisplay(hwndDlg, pData, bUpdateThumb);
                return;
            }
            Current = (*Current).Blink;
        }
    } else {
        Current = (*Current).Flink;
        while !Current.is_null() {
            if (*Current).dmPelsHeight == dmNewPelsHeight
                && (*Current).dmPelsWidth == dmNewPelsWidth
                && (*Current).dmBitsPerPel == dmBitsPerPel
            {
                (*(*pData).CurrentDisplayDevice).CurrentSettings = Current;
                UpdateDisplay(hwndDlg, pData, bUpdateThumb);
                return;
            }
            Current = (*Current).Flink;
        }
    }

    Current = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).Flink;
    while !Current.is_null() {
        if dmNewPelsHeight == (*Current).dmPelsHeight && dmNewPelsWidth == (*Current).dmPelsWidth {
            (*(*pData).CurrentDisplayDevice).CurrentSettings = Current;
            UpdateDisplay(hwndDlg, pData, bUpdateThumb);
            return;
        }
        Current = (*Current).Flink;
    }

    Current = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).Blink;
    while !Current.is_null() {
        if dmNewPelsHeight == (*Current).dmPelsHeight && dmNewPelsWidth == (*Current).dmPelsWidth {
            (*(*pData).CurrentDisplayDevice).CurrentSettings = Current;
            UpdateDisplay(hwndDlg, pData, bUpdateThumb);
            return;
        }
        Current = (*Current).Blink;
    }
}
#[no_mangle]
pub unsafe extern "system" fn SettingsPageCallbackProc(
    hwnd: HWND,
    uMsg: u32,
    ppsp: *mut u32,
) -> u32 {
    let mut Ret: u32 = 0;

    match uMsg {
        PSPCB_CREATE => {
            Ret = (RegisterMonitorSelectionControl(hApplet)) as u32;
        }
        PSPCB_RELEASE => {
            UnregisterMonitorSelectionControl(hApplet);
        }
        _ => {}
    }

    return Ret;
}
unsafe extern "system" fn ConfirmDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pData: PTIMEOUTDATA;

    pData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PTIMEOUTDATA;

    match uMsg {
        WM_INITDIALOG => {
            pData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<TIMEOUTDATA>()) as SIZE_T,
            ) as PTIMEOUTDATA;
            if pData.is_null() {
                return FALSE;
            }

            SetWindowLongPtr(hwndDlg, DWLP_USER, pData as LONG_PTR);

            (*pData).nTimeout = 15;

            LoadString(
                hApplet,
                IDS_TIMEOUTTEXT,
                (*pData).szRawBuffer.as_mut_ptr(),
                (_countof!((*pData).szRawBuffer)) as i32,
            );

            _stprintf(
                (*pData).szCookedBuffer.as_mut_ptr(),
                (*pData).szRawBuffer.as_ptr(),
                (*pData).nTimeout,
            );
            SetDlgItemText(
                hwndDlg,
                (IDC_TIMEOUTTEXT) as i32,
                (*pData).szCookedBuffer.as_ptr(),
            );

            SetTimer(
                hwndDlg,
                1,
                1000,
                core::ptr::null_mut(),
            );
        }

        WM_TIMER => {
            (*pData).nTimeout -= 1;

            _stprintf(
                (*pData).szCookedBuffer.as_mut_ptr(),
                (*pData).szRawBuffer.as_ptr(),
                (*pData).nTimeout,
            );
            SetDlgItemText(
                hwndDlg,
                (IDC_TIMEOUTTEXT) as i32,
                (*pData).szCookedBuffer.as_ptr(),
            );

            if (*pData).nTimeout == 0 {
                KillTimer(hwndDlg, 1);
                EndDialog(hwndDlg, (IDNO) as INT_PTR);
            }
        }

        WM_COMMAND => {
            KillTimer(hwndDlg, 1);
            EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
        }

        WM_DESTROY => {
            HeapFree(GetProcessHeap(), 0, pData as *mut _);
        }

        _ => {}
    }

    FALSE
}
#[no_mangle]
pub unsafe extern "C" fn SwitchDisplayMode(
    hwndDlg: HWND,
    DeviceName: PWSTR,
    seInit: *mut SETTINGS_ENTRY,
    seNew: *mut SETTINGS_ENTRY,
    rc: *mut i32,
) -> u8 {
    let mut Message: [TCHAR; 1024] = [0; 1024];
    let mut Title: [TCHAR; 256] = [0; 256];
    let mut devmode: DEVMODE = core::mem::zeroed();

    RtlZeroMemory!(
        &mut devmode as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<DEVMODE>(),
    );
    devmode.dmSize = core::mem::size_of::<DEVMODE>() as u16;
    devmode.dmPelsWidth = (*seNew).dmPelsWidth;
    devmode.dmPelsHeight = (*seNew).dmPelsHeight;
    devmode.dmBitsPerPel = (*seNew).dmBitsPerPel;
    devmode.dmDisplayFrequency = (*seNew).dmDisplayFrequency;
    devmode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_BITSPERPEL | DM_DISPLAYFREQUENCY;

    *rc = ChangeDisplaySettingsEx(
        DeviceName,
        &mut devmode,
        core::ptr::null_mut(),
        CDS_UPDATEREGISTRY,
        core::ptr::null_mut(),
    );

    match (((*rc) as u32) as i32) as u32 {
        DISP_CHANGE_SUCCESSFUL => {}
        DISP_CHANGE_RESTART => {
            LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title.as_mut_ptr(), (_countof!(Title)) as i32);
            LoadString(hApplet, IDS_APPLY_NEEDS_RESTART, Message.as_mut_ptr(), (_countof!(Message)) as i32);
            MessageBox(hwndDlg, Message.as_ptr(), Title.as_ptr(), MB_OK | MB_ICONINFORMATION);
            return (FALSE) as u8;
        }
        DISP_CHANGE_FAILED => {
            LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title.as_mut_ptr(), (_countof!(Title)) as i32);
            LoadString(hApplet, IDS_APPLY_FAILED, Message.as_mut_ptr(), (_countof!(Message)) as i32);
            MessageBox(hwndDlg, Message.as_ptr(), Title.as_ptr(), MB_OK | MB_ICONSTOP);
            return (FALSE) as u8;
        }
        _ => {
            LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title.as_mut_ptr(), (_countof!(Title)) as i32);
            LoadString(hApplet, IDS_APPLY_FAILED, Message.as_mut_ptr(), (_countof!(Message)) as i32);
            MessageBox(hwndDlg, Message.as_ptr(), Title.as_ptr(), MB_OK | MB_ICONSTOP);
            return (FALSE) as u8;
        }
    }

    if DialogBox!(hApplet, MAKEINTRESOURCE!(IDD_CONFIRMSETTINGS), hwndDlg, ConfirmDlgProc) == (IDYES) as i32 {
        return (TRUE) as u8;
    } else {
        devmode.dmPelsWidth = (*seInit).dmPelsWidth;
        devmode.dmPelsHeight = (*seInit).dmPelsHeight;
        devmode.dmBitsPerPel = (*seInit).dmBitsPerPel;
        devmode.dmDisplayFrequency = (*seInit).dmDisplayFrequency;

        *rc = ChangeDisplaySettingsEx(
            DeviceName,
            &mut devmode,
            core::ptr::null_mut(),
            CDS_UPDATEREGISTRY,
            core::ptr::null_mut(),
        );

        match (((*rc) as u32) as i32) as u32 {
            DISP_CHANGE_SUCCESSFUL => {
                return (FALSE) as u8;
            }
            DISP_CHANGE_RESTART => {
                LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title.as_mut_ptr(), (_countof!(Title)) as i32);
                LoadString(hApplet, IDS_APPLY_NEEDS_RESTART, Message.as_mut_ptr(), (_countof!(Message)) as i32);
                MessageBox(hwndDlg, Message.as_ptr(), Title.as_ptr(), MB_OK | MB_ICONINFORMATION);
                return (FALSE) as u8;
            }
            DISP_CHANGE_FAILED => {
                LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title.as_mut_ptr(), (_countof!(Title)) as i32);
                LoadString(hApplet, IDS_APPLY_FAILED, Message.as_mut_ptr(), (_countof!(Message)) as i32);
                MessageBox(hwndDlg, Message.as_ptr(), Title.as_ptr(), MB_OK | MB_ICONSTOP);
                return (FALSE) as u8;
            }
            _ => {
                LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title.as_mut_ptr(), (_countof!(Title)) as i32);
                LoadString(hApplet, IDS_APPLY_FAILED, Message.as_mut_ptr(), (_countof!(Message)) as i32);
                MessageBox(hwndDlg, Message.as_ptr(), Title.as_ptr(), MB_OK | MB_ICONSTOP);
                return (FALSE) as u8;
            }
        }
    }
}
unsafe extern "C" fn FindBestElement(
    pDevice: PDISPLAY_DEVICE_ENTRY
) -> PSETTINGS_ENTRY {
    let Request = core::ptr::addr_of_mut!((*pDevice).InitialSettings);
    let mut BestEntry: PSETTINGS_ENTRY = core::ptr::null_mut();
    let mut Current: PSETTINGS_ENTRY;
    let mut Distance: LONG;
    let mut NearestDistance: LONG = (MAXLONG) as i32;

    Current = (*pDevice).Settings;
    while !Current.is_null() {
        Distance = 0x100000 * labs((*Current).dmBitsPerPel as i32 - (*Request).dmBitsPerPel as i32) +
                   0x100 * labs((*Current).dmPelsWidth as i32 - (*Request).dmPelsWidth as i32) +
                   0x100 * labs((*Current).dmPelsHeight as i32 - (*Request).dmPelsHeight as i32) +
                   labs((*Current).dmDisplayFrequency as i32 - (*Request).dmDisplayFrequency as i32);
        if Distance == 0 {
            return Current;
        }

        if Distance < NearestDistance {
            BestEntry = Current;
            NearestDistance = Distance;
        }

        Current = (*Current).Flink;
    }

    BestEntry
}
unsafe extern "C" fn ApplyDisplaySettings(hwndDlg: HWND, pData: PSETTINGS_DATA) {
    let mut rc: i32 = 0;

    let Ret: u8 = SwitchDisplayMode(
        hwndDlg,
        ((*(*pData).CurrentDisplayDevice).DeviceName).cast(),
        core::ptr::addr_of_mut!((*(*pData).CurrentDisplayDevice).InitialSettings),
        (*(*pData).CurrentDisplayDevice).CurrentSettings,
        core::ptr::addr_of_mut!(rc),
    );

    if rc != (DISP_CHANGE_SUCCESSFUL) as i32 {
        return;
    }

    if Ret != 0 {
        (*(*pData).CurrentDisplayDevice).InitialSettings.dmPelsWidth = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsWidth;
        (*(*pData).CurrentDisplayDevice).InitialSettings.dmPelsHeight = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsHeight;
        (*(*pData).CurrentDisplayDevice).InitialSettings.dmBitsPerPel = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmBitsPerPel;
        (*(*pData).CurrentDisplayDevice).InitialSettings.dmDisplayFrequency = (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmDisplayFrequency;
    } else {
        (*(*pData).CurrentDisplayDevice).CurrentSettings = FindBestElement((*pData).CurrentDisplayDevice);
        UpdateDisplay(hwndDlg, pData, (TRUE) as BOOL);
    }
}
#[no_mangle]
pub unsafe extern "system" fn SettingsPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pData: PSETTINGS_DATA = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETTINGS_DATA;

    match uMsg {
        WM_INITDIALOG => {
            SettingsOnInitDialog(hwndDlg);
        }
        WM_DRAWITEM => {
            let lpDrawItem: LPDRAWITEMSTRUCT = lParam as LPDRAWITEMSTRUCT;

            match (*lpDrawItem).CtlID {
                IDC_RESOLUTION_PREVIEW => {
                    ShowResolutionPreview(lpDrawItem, pData);
                }
                IDC_SETTINGS_SPECTRUM => {
                    ShowColorSpectrum(
                        (*lpDrawItem).hDC,
                        core::ptr::addr_of_mut!((*lpDrawItem).rcItem),
                        (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmBitsPerPel,
                        pData,
                    );
                }
                _ => {}
            }
        }
        WM_COMMAND => {
            let controlId: DWORD = LOWORD!(wParam);
            let command: DWORD = HIWORD!(wParam);

            if controlId == IDC_SETTINGS_ADVANCED && command == BN_CLICKED {
                if DisplayAdvancedSettings(hwndDlg, (*pData).CurrentDisplayDevice) != 0 {
                    let mut devmode: DEVMODE = core::mem::zeroed();
                    ZeroMemory!(
                        core::ptr::addr_of_mut!(devmode) as *mut core::ffi::c_void,
                        core::mem::size_of::<DEVMODE>(),
                    );
                    devmode.dmSize = core::mem::size_of::<DEVMODE>() as WORD;
                    if EnumDisplaySettingsExW(
                        ((*(*pData).CurrentDisplayDevice).DeviceName) as *const u16,
                        ENUM_CURRENT_SETTINGS,
                        core::ptr::addr_of_mut!(devmode),
                        0,
                    ) != 0
                    {
                        let pInitialSettings: PSETTINGS_ENTRY =
                            core::ptr::addr_of_mut!((*(*pData).CurrentDisplayDevice).InitialSettings);
                        (*pInitialSettings).dmPelsWidth = devmode.dmPelsWidth;
                        (*pInitialSettings).dmPelsHeight = devmode.dmPelsHeight;
                        (*pInitialSettings).dmBitsPerPel = devmode.dmBitsPerPel;
                        (*pInitialSettings).dmDisplayFrequency = devmode.dmDisplayFrequency;
                        (*(*pData).CurrentDisplayDevice).CurrentSettings =
                            FindBestElement((*pData).CurrentDisplayDevice);
                        UpdateDisplay(hwndDlg, pData, (TRUE) as BOOL);
                    }
                }
            } else if controlId == IDC_SETTINGS_BPP && command == CBN_SELCHANGE {
                OnBPPChanged(hwndDlg, pData);
            }
        }
        WM_HSCROLL => {
            match (LOWORD!(wParam)) as u32 {
                TB_LINEUP
                | TB_LINEDOWN
                | TB_PAGEUP
                | TB_PAGEDOWN
                | TB_TOP
                | TB_BOTTOM
                | TB_ENDTRACK => {
                    let newPosition: DWORD = SendDlgItemMessage(
                        hwndDlg,
                        (IDC_SETTINGS_RESOLUTION) as i32,
                        TBM_GETPOS,
                        0,
                        0,
                    ) as DWORD;
                    OnResolutionChanged(hwndDlg, pData, newPosition, (TRUE) as BOOL);
                }
                TB_THUMBTRACK => {
                    OnResolutionChanged(hwndDlg, pData, (HIWORD!(wParam)) as DWORD, FALSE);
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;
            if (*lpnm).code == PSN_APPLY as UINT {
                if (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsWidth
                    != (*(*pData).CurrentDisplayDevice).InitialSettings.dmPelsWidth
                    || (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmPelsHeight
                        != (*(*pData).CurrentDisplayDevice).InitialSettings.dmPelsHeight
                    || (*(*(*pData).CurrentDisplayDevice).CurrentSettings).dmBitsPerPel
                        != (*(*pData).CurrentDisplayDevice).InitialSettings.dmBitsPerPel
                {
                    ApplyDisplaySettings(hwndDlg, pData);
                }
            } else if (*lpnm).code == MSLN_MONITORCHANGED {
                let lpnmi: PMONSL_MONNMMONITORCHANGING = lParam as PMONSL_MONNMMONITORCHANGING;
                let mut Current: PDISPLAY_DEVICE_ENTRY = (*pData).DisplayDeviceList;
                for _i in 0..(*lpnmi).hdr.Index {
                    Current = (*Current).Flink;
                }
                OnDisplayDeviceChanged(hwndDlg, pData, Current);
            }
        }
        WM_CONTEXTMENU => {
            let mut pt: POINT = core::mem::zeroed();
            let mut ptClient: POINT;

            pt.x = LOWORD!(lParam) as i16 as i32;
            pt.y = HIWORD!(lParam) as i16 as i32;

            let hwndMonSel: HWND = GetDlgItem(hwndDlg, (IDC_SETTINGS_MONSEL) as i32);
            if wParam as HWND == hwndMonSel {
                let Index: INT;
                if pt.x == -(1 as i32) && pt.y == -(1 as i32) {
                    let mut rcMon: RECT = core::mem::zeroed();

                    Index = SendMessage(
                        hwndMonSel,
                        MSLM_GETCURSEL,
                        0,
                        0,
                    ) as INT;

                    if Index >= 0 as INT
                        && SendMessage(
                            hwndMonSel,
                            MSLM_GETMONITORRECT,
                            Index as WPARAM,
                            core::ptr::addr_of_mut!(rcMon) as LPARAM,
                        ) as INT
                            > 0 as INT
                    {
                        pt.x = rcMon.left + ((rcMon.right - rcMon.left) / 2 as i32);
                        pt.y = rcMon.top + ((rcMon.bottom - rcMon.top) / 2 as i32);
                    } else {
                        pt.x = 0 as i32;
                        pt.y = 0 as i32;
                    }

                    MapWindowPoints(hwndMonSel, core::ptr::null_mut(), core::ptr::addr_of_mut!(pt), 1);
                } else {
                    ptClient = pt;
                    MapWindowPoints(
                        core::ptr::null_mut(),
                        hwndMonSel,
                        core::ptr::addr_of_mut!(ptClient),
                        1,
                    );

                    Index = SendMessage(
                        hwndMonSel,
                        MSLM_HITTEST,
                        core::ptr::addr_of_mut!(ptClient) as WPARAM,
                        0,
                    ) as INT;
                }

                if Index >= 0 as INT {
                    let hPopup: HMENU = LoadPopupMenu(hApplet, MAKEINTRESOURCE!(IDM_MONITOR_MENU));
                    if !hPopup.is_null() {
                        EnableMenuItem(
                            hPopup,
                            ID_MENU_ATTACHED,
                            MF_BYCOMMAND | MF_DISABLED | MF_GRAYED,
                        );
                        EnableMenuItem(
                            hPopup,
                            ID_MENU_PRIMARY,
                            MF_BYCOMMAND | MF_DISABLED | MF_GRAYED,
                        );
                        EnableMenuItem(
                            hPopup,
                            ID_MENU_IDENTIFY,
                            MF_BYCOMMAND | MF_DISABLED | MF_GRAYED,
                        );
                        EnableMenuItem(
                            hPopup,
                            ID_MENU_PROPERTIES,
                            MF_BYCOMMAND | MF_DISABLED | MF_GRAYED,
                        );

                        let uiCmd: UINT = TrackPopupMenu(
                            hPopup,
                            TPM_RETURNCMD | TPM_RIGHTBUTTON,
                            pt.x,
                            pt.y,
                            0,
                            hwndDlg,
                            core::ptr::null_mut(),
                        ) as UINT;

                        match uiCmd {
                            ID_MENU_ATTACHED
                            | ID_MENU_PRIMARY
                            | ID_MENU_IDENTIFY
                            | ID_MENU_PROPERTIES => {}
                            _ => {}
                        }

                        DestroyMenu(hPopup);
                    }
                }
            }
        }
        WM_DESTROY => {
            let mut Current: PDISPLAY_DEVICE_ENTRY = (*pData).DisplayDeviceList;

            while !Current.is_null() {
                let Next: PDISPLAY_DEVICE_ENTRY = (*Current).Flink;
                let mut CurrentSettings: PSETTINGS_ENTRY = (*Current).Settings;
                while !CurrentSettings.is_null() {
                    let NextSettings: PSETTINGS_ENTRY = (*CurrentSettings).Flink;
                    HeapFree(GetProcessHeap(), 0, CurrentSettings as *mut core::ffi::c_void);
                    CurrentSettings = NextSettings;
                }
                HeapFree(GetProcessHeap(), 0, Current as *mut core::ffi::c_void);
                Current = Next;
            }

            for i in 0..NUM_SPECTRUM_BITMAPS {
                if !(*pData).hSpectrumBitmaps[i as usize].is_null() {
                    DeleteObject((*pData).hSpectrumBitmaps[i as usize] as HGDIOBJ);
                }
            }

            HeapFree(GetProcessHeap(), 0, pData as *mut core::ffi::c_void);
        }
        _ => {}
    }
    FALSE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * PURPOSE:         Settings property page
//  * PROGRAMMERS:     Trevor McCort (lycan359@gmail.com)
//  *                  Hervé Poussineau (hpoussin@reactos.org)
//  */
//
// #include "desk.h"
//
// typedef struct _SETTINGS_DATA
// {
//     PDISPLAY_DEVICE_ENTRY DisplayDeviceList;
//     PDISPLAY_DEVICE_ENTRY CurrentDisplayDevice;
//     HBITMAP hSpectrumBitmaps[NUM_SPECTRUM_BITMAPS];
//     int cxSource[NUM_SPECTRUM_BITMAPS];
//     int cySource[NUM_SPECTRUM_BITMAPS];
// } SETTINGS_DATA, *PSETTINGS_DATA;
//
// typedef struct _TIMEOUTDATA
// {
//     TCHAR szRawBuffer[256];
//     TCHAR szCookedBuffer[256];
//     INT nTimeout;
// } TIMEOUTDATA, *PTIMEOUTDATA;
//
// static VOID
// UpdateDisplay( HWND hwndDlg, PSETTINGS_DATA pData,  BOOL bUpdateThumb)
// {
//     TCHAR Buffer[64];
//     TCHAR Pixel[64];
//     DWORD index;
//     HWND hwndMonSel;
//     MONSL_MONINFO info;
//
//     LoadString(hApplet, IDS_PIXEL, Pixel, _countof(Pixel));
//     _stprintf(Buffer, Pixel, pData->CurrentDisplayDevice->CurrentSettings->dmPelsWidth, pData->CurrentDisplayDevice->CurrentSettings->dmPelsHeight, Pixel);
//     SendDlgItemMessage(hwndDlg, IDC_SETTINGS_RESOLUTION_TEXT, WM_SETTEXT, 0, (LPARAM)Buffer);
//
//     for (index = 0; index < pData->CurrentDisplayDevice->ResolutionsCount; index++)
//     {
//         if (pData->CurrentDisplayDevice->Resolutions[index].dmPelsWidth == pData->CurrentDisplayDevice->CurrentSettings->dmPelsWidth &&
//             pData->CurrentDisplayDevice->Resolutions[index].dmPelsHeight == pData->CurrentDisplayDevice->CurrentSettings->dmPelsHeight)
//         {
//             if (bUpdateThumb)
//                 SendDlgItemMessage(hwndDlg, IDC_SETTINGS_RESOLUTION, TBM_SETPOS, TRUE, index);
//             break;
//         }
//     }
//     if (LoadString(hApplet, (2900 + pData->CurrentDisplayDevice->CurrentSettings->dmBitsPerPel), Buffer, _countof(Buffer)))
//         SendDlgItemMessage(hwndDlg, IDC_SETTINGS_BPP, CB_SELECTSTRING, (WPARAM)-1, (LPARAM)Buffer);
//
//     hwndMonSel = GetDlgItem(hwndDlg, IDC_SETTINGS_MONSEL);
//     index = (INT)SendMessage(hwndMonSel, MSLM_GETCURSEL, 0, 0);
//     if (index != (DWORD)-1 && SendMessage(hwndMonSel, MSLM_GETMONITORINFO, index, (LPARAM)&info))
//     {
//         info.Size.cx = pData->CurrentDisplayDevice->CurrentSettings->dmPelsWidth;
//         info.Size.cy = pData->CurrentDisplayDevice->CurrentSettings->dmPelsHeight;
//         SendMessage(hwndMonSel, MSLM_SETMONITORINFO, index, (LPARAM)&info);
//     }
// }
//
// static int
// CompareSettings(PSETTINGS_ENTRY Entry, DWORD dmPelsWidth, DWORD dmPelsHeight,
//                 DWORD dmBitsPerPel, DWORD dmDisplayFrequency)
// {
//     if (Entry->dmPelsWidth  == dmPelsWidth  &&
//         Entry->dmPelsHeight == dmPelsHeight &&
//         Entry->dmBitsPerPel == dmBitsPerPel &&
//         Entry->dmDisplayFrequency == dmDisplayFrequency)
//     {
//         return 0;
//     }
//     else
//     if ((Entry->dmPelsWidth  < dmPelsWidth) ||
//         (Entry->dmPelsWidth == dmPelsWidth && Entry->dmPelsHeight < dmPelsHeight) ||
//         (Entry->dmPelsWidth == dmPelsWidth && Entry->dmPelsHeight == dmPelsHeight &&
//          Entry->dmBitsPerPel < dmBitsPerPel))
//     {
//         return 1;
//     }
//     return -1;
// }
//
// static PSETTINGS_ENTRY
// GetPossibleSettings( LPCTSTR DeviceName,  DWORD* pSettingsCount,  PSETTINGS_ENTRY* CurrentSettings)
// {
//     DEVMODE devmode;
//     DWORD NbSettings = 0;
//     DWORD iMode = 0;
//     DWORD dwFlags = 0;
//     PSETTINGS_ENTRY Settings = NULL;
//     HDC hDC;
//     PSETTINGS_ENTRY Current;
//     DWORD bpp, xres, yres;
//     DWORD curDispFreq;
//
//     /* Get current settings */
//     *CurrentSettings = NULL;
//     hDC = CreateIC(NULL, DeviceName, NULL, NULL);
//     bpp = GetDeviceCaps(hDC, PLANES);
//     bpp *= GetDeviceCaps(hDC, BITSPIXEL);
//     xres = GetDeviceCaps(hDC, HORZRES);
//     yres = GetDeviceCaps(hDC, VERTRES);
//     DeleteDC(hDC);
//
//     /* List all settings */
//     devmode.dmSize = (WORD)sizeof(devmode);
//     devmode.dmDriverExtra = 0;
//
//     if (!EnumDisplaySettingsEx(DeviceName, ENUM_CURRENT_SETTINGS, &devmode, dwFlags))
//         return NULL;
//
//     curDispFreq = devmode.dmDisplayFrequency;
//
//     while (EnumDisplaySettingsEx(DeviceName, iMode, &devmode, dwFlags))
//     {
//         iMode++;
//
//         if (devmode.dmPelsWidth < 640 ||
//             devmode.dmPelsHeight < 480 ||
//             devmode.dmDisplayFrequency != curDispFreq ||
//             (devmode.dmBitsPerPel != 4 &&
//              devmode.dmBitsPerPel != 8 &&
//              devmode.dmBitsPerPel != 16 &&
//              devmode.dmBitsPerPel != 24 &&
//              devmode.dmBitsPerPel != 32))
//         {
//             continue;
//         }
//
//         Current = HeapAlloc(GetProcessHeap(), 0, sizeof(SETTINGS_ENTRY));
//         if (Current != NULL)
//         {
//             /* Sort resolutions by increasing height, and BPP */
//             PSETTINGS_ENTRY Previous = NULL;
//             PSETTINGS_ENTRY Next = Settings;
//             Current->dmPelsWidth = devmode.dmPelsWidth;
//             Current->dmPelsHeight = devmode.dmPelsHeight;
//             Current->dmBitsPerPel = devmode.dmBitsPerPel;
//             Current->dmDisplayFrequency = devmode.dmDisplayFrequency;
//             while (Next != NULL &&
//                    CompareSettings(Next, devmode.dmPelsWidth,
//                                    devmode.dmPelsHeight, devmode.dmBitsPerPel,
//                                    devmode.dmDisplayFrequency) > 0)
//             {
//                 Previous = Next;
//                 Next = Next->Flink;
//             }
//             Current->Blink = Previous;
//             Current->Flink = Next;
//             if (Previous == NULL)
//                 Settings = Current;
//             else
//                 Previous->Flink = Current;
//             if (Next != NULL)
//                 Next->Blink = Current;
//             if (devmode.dmPelsWidth == xres && devmode.dmPelsHeight == yres && devmode.dmBitsPerPel == bpp)
//             {
//                 *CurrentSettings = Current;
//             }
//             NbSettings++;
//         }
//     }
//
//     *pSettingsCount = NbSettings;
//     return Settings;
// }
//
// static BOOL
// AddDisplayDevice( PSETTINGS_DATA pData,  const DISPLAY_DEVICE *DisplayDevice)
// {
//     PDISPLAY_DEVICE_ENTRY newEntry = NULL;
//     LPTSTR description = NULL;
//     LPTSTR name = NULL;
//     LPTSTR key = NULL;
//     LPTSTR devid = NULL;
//     SIZE_T descriptionSize, nameSize, keySize, devidSize;
//     PSETTINGS_ENTRY Current;
//     DWORD ResolutionsCount = 1;
//     DWORD i;
//
//     newEntry = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(DISPLAY_DEVICE_ENTRY));
//     if (!newEntry) goto ByeBye;
//
//     newEntry->Settings = GetPossibleSettings(DisplayDevice->DeviceName, &newEntry->SettingsCount, &newEntry->CurrentSettings);
//     if (!newEntry->Settings) goto ByeBye;
//
//     newEntry->InitialSettings.dmPelsWidth = newEntry->CurrentSettings->dmPelsWidth;
//     newEntry->InitialSettings.dmPelsHeight = newEntry->CurrentSettings->dmPelsHeight;
//     newEntry->InitialSettings.dmBitsPerPel = newEntry->CurrentSettings->dmBitsPerPel;
//     newEntry->InitialSettings.dmDisplayFrequency = newEntry->CurrentSettings->dmDisplayFrequency;
//
//     /* Count different resolutions */
//     for (Current = newEntry->Settings; Current != NULL; Current = Current->Flink)
//     {
//         if (Current->Flink != NULL &&
//             ((Current->dmPelsWidth != Current->Flink->dmPelsWidth) ||
//              (Current->dmPelsHeight != Current->Flink->dmPelsHeight)))
//         {
//             ResolutionsCount++;
//         }
//     }
//
//     newEntry->Resolutions = HeapAlloc(GetProcessHeap(), 0, ResolutionsCount * sizeof(RESOLUTION_INFO));
//     if (!newEntry->Resolutions) goto ByeBye;
//
//     newEntry->ResolutionsCount = ResolutionsCount;
//
//     /* Fill resolutions infos */
//     for (Current = newEntry->Settings, i = 0; Current != NULL; Current = Current->Flink)
//     {
//         if (Current->Flink == NULL ||
//             (Current->Flink != NULL &&
//             ((Current->dmPelsWidth != Current->Flink->dmPelsWidth) ||
//              (Current->dmPelsHeight != Current->Flink->dmPelsHeight))))
//         {
//             newEntry->Resolutions[i].dmPelsWidth = Current->dmPelsWidth;
//             newEntry->Resolutions[i].dmPelsHeight = Current->dmPelsHeight;
//             i++;
//         }
//     }
//     descriptionSize = (_tcslen(DisplayDevice->DeviceString) + 1) * sizeof(TCHAR);
//     description = HeapAlloc(GetProcessHeap(), 0, descriptionSize);
//     if (!description) goto ByeBye;
//
//     nameSize = (_tcslen(DisplayDevice->DeviceName) + 1) * sizeof(TCHAR);
//     name = HeapAlloc(GetProcessHeap(), 0, nameSize);
//     if (!name) goto ByeBye;
//
//     keySize = (_tcslen(DisplayDevice->DeviceKey) + 1) * sizeof(TCHAR);
//     key = HeapAlloc(GetProcessHeap(), 0, keySize);
//     if (!key) goto ByeBye;
//
//     devidSize = (_tcslen(DisplayDevice->DeviceID) + 1) * sizeof(TCHAR);
//     devid = HeapAlloc(GetProcessHeap(), 0, devidSize);
//     if (!devid) goto ByeBye;
//
//     memcpy(description, DisplayDevice->DeviceString, descriptionSize);
//     memcpy(name, DisplayDevice->DeviceName, nameSize);
//     memcpy(key, DisplayDevice->DeviceKey, keySize);
//     memcpy(devid, DisplayDevice->DeviceID, devidSize);
//     newEntry->DeviceDescription = description;
//     newEntry->DeviceName = name;
//     newEntry->DeviceKey = key;
//     newEntry->DeviceID = devid;
//     newEntry->DeviceStateFlags = DisplayDevice->StateFlags;
//     newEntry->Flink = pData->DisplayDeviceList;
//     pData->DisplayDeviceList = newEntry;
//     return TRUE;
//
// ByeBye:
//     if (newEntry != NULL)
//     {
//         if (newEntry->Settings != NULL)
//         {
//             Current = newEntry->Settings;
//             while (Current != NULL)
//             {
//                 PSETTINGS_ENTRY Next = Current->Flink;
//                 HeapFree(GetProcessHeap(), 0, Current);
//                 Current = Next;
//             }
//         }
//         if (newEntry->Resolutions != NULL)
//             HeapFree(GetProcessHeap(), 0, newEntry->Resolutions);
//         HeapFree(GetProcessHeap(), 0, newEntry);
//     }
//     if (description != NULL)
//         HeapFree(GetProcessHeap(), 0, description);
//     if (name != NULL)
//         HeapFree(GetProcessHeap(), 0, name);
//     if (key != NULL)
//         HeapFree(GetProcessHeap(), 0, key);
//     return FALSE;
// }
//
// static VOID
// OnDisplayDeviceChanged( HWND hwndDlg,  PSETTINGS_DATA pData,  PDISPLAY_DEVICE_ENTRY pDeviceEntry)
// {
//     PSETTINGS_ENTRY Current;
//     DWORD index;
//
//     pData->CurrentDisplayDevice = pDeviceEntry; /* Update variable */
//
//     /* Fill color depths combo box */
//     SendDlgItemMessage(hwndDlg, IDC_SETTINGS_BPP, CB_RESETCONTENT, 0, 0);
//     for (Current = pDeviceEntry->Settings; Current != NULL; Current = Current->Flink)
//     {
//         TCHAR Buffer[64];
//         if (LoadString(hApplet, (2900 + Current->dmBitsPerPel), Buffer, _countof(Buffer)))
//         {
//             index = (DWORD)SendDlgItemMessage(hwndDlg, IDC_SETTINGS_BPP, CB_FINDSTRINGEXACT, (WPARAM)-1, (LPARAM)Buffer);
//             if (index == (DWORD)CB_ERR)
//             {
//                 index = (DWORD)SendDlgItemMessage(hwndDlg, IDC_SETTINGS_BPP, CB_ADDSTRING, 0, (LPARAM)Buffer);
//                 SendDlgItemMessage(hwndDlg, IDC_SETTINGS_BPP, CB_SETITEMDATA, index, Current->dmBitsPerPel);
//             }
//         }
//     }
//
//     /* Fill device description */
//     SendDlgItemMessage(hwndDlg, IDC_SETTINGS_DEVICE, WM_SETTEXT, 0, (LPARAM)pDeviceEntry->DeviceDescription);
//
//     /* Fill resolutions slider */
//     SendDlgItemMessage(hwndDlg, IDC_SETTINGS_RESOLUTION, TBM_CLEARTICS, TRUE, 0);
//     SendDlgItemMessage(hwndDlg, IDC_SETTINGS_RESOLUTION, TBM_SETRANGE, TRUE, MAKELONG(0, pDeviceEntry->ResolutionsCount - 1));
//
//     UpdateDisplay(hwndDlg, pData, TRUE);
// }
//
// static VOID
// SettingsOnInitDialog( HWND hwndDlg)
// {
//     BITMAP bitmap;
//     DWORD Result = 0;
//     DWORD iDevNum = 0;
//     DWORD i;
//     DISPLAY_DEVICE displayDevice;
//     PSETTINGS_DATA pData;
//
//     pData = HeapAlloc(GetProcessHeap(), 0, sizeof(SETTINGS_DATA));
//     if (pData == NULL)
//         return;
//
//     SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pData);
//
//     /* Get video cards list */
//     pData->DisplayDeviceList = NULL;
//     displayDevice.cb = sizeof(displayDevice);
//     while (EnumDisplayDevices(NULL, iDevNum, &displayDevice, EDD_GET_DEVICE_INTERFACE_NAME))
//     {
//         if ((displayDevice.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) != 0)
//         {
//             if (AddDisplayDevice(pData, &displayDevice))
//                 Result++;
//         }
//         iDevNum++;
//     }
//
//     if (Result == 0)
//     {
//         /* No adapter found */
//         EnableWindow(GetDlgItem(hwndDlg, IDC_SETTINGS_BPP), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_SETTINGS_RESOLUTION), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_SETTINGS_RESOLUTION_TEXT), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_SETTINGS_ADVANCED), FALSE);
//         ShowWindow(GetDlgItem(hwndDlg, IDC_SETTINGS_SPECTRUM), SW_HIDE);
//
//         /* Do not initialize the color spectrum bitmaps */
//         memset(pData->hSpectrumBitmaps, 0, sizeof(pData->hSpectrumBitmaps));
//         return;
//     }
//     else if (Result == 1)
//     {
//         MONSL_MONINFO monitors;
//
//         /* Single video adapter */
//         OnDisplayDeviceChanged(hwndDlg, pData, pData->DisplayDeviceList);
//
//         ShowWindow(GetDlgItem(hwndDlg, IDC_SETTINGS_MONTEXT), SW_HIDE);
//         ShowWindow(GetDlgItem(hwndDlg, IDC_SETTINGS_MONSEL), SW_HIDE);
//
//         monitors.Position.x = monitors.Position.y = 0;
//         monitors.Size.cx = pData->CurrentDisplayDevice->CurrentSettings->dmPelsWidth;
//         monitors.Size.cy = pData->CurrentDisplayDevice->CurrentSettings->dmPelsHeight;
//         monitors.Flags = 0;
//         SendDlgItemMessage(hwndDlg,
//                    IDC_SETTINGS_MONSEL,
//                    MSLM_SETMONITORSINFO,
//                    1,
//                    (LPARAM)&monitors);
//     }
//     else /* FIXME: Incomplete! */
//     {
//         PMONSL_MONINFO pMonitors;
//         DWORD i;
//
//         OnDisplayDeviceChanged(hwndDlg, pData, pData->DisplayDeviceList);
//
//         ShowWindow(GetDlgItem(hwndDlg, IDC_RESOLUTION_PREVIEW), SW_HIDE);
//
//         pMonitors = (PMONSL_MONINFO)HeapAlloc(GetProcessHeap(), 0, sizeof(MONSL_MONINFO) * Result);
//         if (pMonitors)
//         {
//             DWORD hack = 1280;
//             for (i = 0; i < Result; i++)
//             {
//                 pMonitors[i].Position.x = hack * i;
//                 pMonitors[i].Position.y = 0;
//                 pMonitors[i].Size.cx = pData->DisplayDeviceList->CurrentSettings->dmPelsWidth;
//                 pMonitors[i].Size.cy = pData->DisplayDeviceList->CurrentSettings->dmPelsHeight;
//                 pMonitors[i].Flags = 0;
//             }
//
//             SendDlgItemMessage(hwndDlg,
//                        IDC_SETTINGS_MONSEL,
//                        MSLM_SETMONITORSINFO,
//                        Result,
//                        (LPARAM)pMonitors);
//
//             HeapFree(GetProcessHeap(), 0, pMonitors);
//         }
//     }
//
//     /* Initialize the color spectrum bitmaps */
//     for (i = 0; i < NUM_SPECTRUM_BITMAPS; i++)
//     {
//         pData->hSpectrumBitmaps[i] = LoadImageW(hApplet, MAKEINTRESOURCEW(IDB_SPECTRUM_4 + i), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR);
//
//         if (pData->hSpectrumBitmaps[i] != NULL)
//         {
//             if (GetObjectW(pData->hSpectrumBitmaps[i], sizeof(BITMAP), &bitmap) != 0)
//             {
//                 pData->cxSource[i] = bitmap.bmWidth;
//                 pData->cySource[i] = bitmap.bmHeight;
//             }
//             else
//             {
//                 pData->cxSource[i] = 0;
//                 pData->cySource[i] = 0;
//             }
//         }
//     }
// }
//
// static VOID
// ShowResolutionPreview( LPDRAWITEMSTRUCT draw,  PSETTINGS_DATA pData)
// {
//     HBRUSH hBrush;
//     HDC hDC;
//     HGDIOBJ hOldObj;
//     RECT rcItem = {
//         MONITOR_LEFT,
//         MONITOR_TOP,
//         MONITOR_RIGHT,
//         MONITOR_BOTTOM
//     };
//
//     hDC = CreateCompatibleDC(draw->hDC);
//     hOldObj = SelectObject(hDC, g_GlobalData.hMonitorBitmap);
//
//     /* FIXME: Draw resolution preview bitmap inside monitor. */
//     hBrush = CreateSolidBrush(g_GlobalData.desktop_color);
//     FillRect(hDC, &rcItem, hBrush);
//     DeleteObject(hBrush);
//
//     GdiTransparentBlt(draw->hDC,
//                       draw->rcItem.left, draw->rcItem.top,
//                       draw->rcItem.right - draw->rcItem.left + 1,
//                       draw->rcItem.bottom - draw->rcItem.top + 1,
//                       hDC,
//                       0, 0,
//                       g_GlobalData.bmMonWidth, g_GlobalData.bmMonHeight,
//                       MONITOR_ALPHA);
//
//     SelectObject(hDC, hOldObj);
//     DeleteDC(hDC);
// }
//
// /* Get the ID for SETTINGS_DATA::hSpectrumBitmaps */
// static VOID
// ShowColorSpectrum( HDC hDC,  LPRECT client,  DWORD BitsPerPel,  PSETTINGS_DATA pData)
// {
//     HDC hdcMem;
//     INT iBitmap;
//
//     hdcMem = CreateCompatibleDC(hDC);
//
//     if (!hdcMem)
//         return;
//
//     switch(BitsPerPel)
//     {
//         case 4:  iBitmap = 0; break;
//         case 8:  iBitmap = 1; break;
//         default: iBitmap = 2;
//     }
//
//     if (SelectObject(hdcMem, pData->hSpectrumBitmaps[iBitmap]))
//     {
//         StretchBlt(hDC,
//                client->left, client->top,
//                client->right - client->left,
//                client->bottom - client->top,
//                hdcMem, 0, 0,
//                pData->cxSource[iBitmap],
//                pData->cySource[iBitmap], SRCCOPY);
//     }
//
//     DeleteDC(hdcMem);
// }
//
// static VOID
// OnBPPChanged( HWND hwndDlg,  PSETTINGS_DATA pData)
// {
//     /* If new BPP is not compatible with resolution:
//      * 1) try to find the nearest smaller matching resolution
//      * 2) otherwise, get the nearest bigger resolution
//      */
//     PSETTINGS_ENTRY Current;
//     DWORD dmNewBitsPerPel;
//     DWORD index;
//     HDC hSpectrumDC;
//     HWND hSpectrumControl;
//     RECT client;
//
//     index = (DWORD)SendDlgItemMessage(hwndDlg, IDC_SETTINGS_BPP, CB_GETCURSEL, 0, 0);
//     dmNewBitsPerPel = (DWORD)SendDlgItemMessage(hwndDlg, IDC_SETTINGS_BPP, CB_GETITEMDATA, index, 0);
//
//     /* Show a new spectrum bitmap */
//     hSpectrumControl = GetDlgItem(hwndDlg, IDC_SETTINGS_SPECTRUM);
//     hSpectrumDC = GetDC(hSpectrumControl);
//     if (hSpectrumDC == NULL)
//         return;
//
//     GetClientRect(hSpectrumControl, &client);
//     ShowColorSpectrum(hSpectrumDC, &client, dmNewBitsPerPel, pData);
//     ReleaseDC(hSpectrumControl, hSpectrumDC);
//
//     /* Find if new parameters are valid */
//     Current = pData->CurrentDisplayDevice->CurrentSettings;
//     if (dmNewBitsPerPel == Current->dmBitsPerPel)
//     {
//         /* No change */
//         return;
//     }
//
//     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//
//     if (dmNewBitsPerPel < Current->dmBitsPerPel)
//     {
//         Current = Current->Blink;
//         while (Current != NULL)
//         {
//             if (Current->dmBitsPerPel == dmNewBitsPerPel
//              && Current->dmPelsHeight == pData->CurrentDisplayDevice->CurrentSettings->dmPelsHeight
//              && Current->dmPelsWidth == pData->CurrentDisplayDevice->CurrentSettings->dmPelsWidth)
//             {
//                 pData->CurrentDisplayDevice->CurrentSettings = Current;
//                 UpdateDisplay(hwndDlg, pData, TRUE);
//                 return;
//             }
//             Current = Current->Blink;
//         }
//     }
//     else
//     {
//         Current = Current->Flink;
//         while (Current != NULL)
//         {
//             if (Current->dmBitsPerPel == dmNewBitsPerPel
//              && Current->dmPelsHeight == pData->CurrentDisplayDevice->CurrentSettings->dmPelsHeight
//              && Current->dmPelsWidth == pData->CurrentDisplayDevice->CurrentSettings->dmPelsWidth)
//             {
//                 pData->CurrentDisplayDevice->CurrentSettings = Current;
//                 UpdateDisplay(hwndDlg, pData, TRUE);
//                 return;
//             }
//             Current = Current->Flink;
//         }
//     }
//
//     /* Search smaller resolution compatible with current color depth */
//     Current = pData->CurrentDisplayDevice->CurrentSettings->Blink;
//     while (Current != NULL)
//     {
//         if (Current->dmBitsPerPel == dmNewBitsPerPel)
//         {
//             pData->CurrentDisplayDevice->CurrentSettings = Current;
//             UpdateDisplay(hwndDlg, pData, TRUE);
//             return;
//         }
//         Current = Current->Blink;
//     }
//
//     /* Search bigger resolution compatible with current color depth */
//     Current = pData->CurrentDisplayDevice->CurrentSettings->Flink;
//     while (Current != NULL)
//     {
//         if (Current->dmBitsPerPel == dmNewBitsPerPel)
//         {
//             pData->CurrentDisplayDevice->CurrentSettings = Current;
//             UpdateDisplay(hwndDlg, pData, TRUE);
//             return;
//         }
//         Current = Current->Flink;
//     }
//
//     /* We shouldn't go there */
// }
//
// static VOID
// OnResolutionChanged( HWND hwndDlg,  PSETTINGS_DATA pData,  DWORD NewPosition,
//                      BOOL bUpdateThumb)
// {
//     /* If new resolution is not compatible with color depth:
//      * 1) try to find the nearest bigger matching color depth
//      * 2) otherwise, get the nearest smaller color depth
//      */
//     PSETTINGS_ENTRY Current;
//     DWORD dmNewPelsHeight = pData->CurrentDisplayDevice->Resolutions[NewPosition].dmPelsHeight;
//     DWORD dmNewPelsWidth = pData->CurrentDisplayDevice->Resolutions[NewPosition].dmPelsWidth;
//     DWORD dmBitsPerPel;
//     DWORD dmDisplayFrequency;
//
//     /* Find if new parameters are valid */
//     Current = pData->CurrentDisplayDevice->CurrentSettings;
//     if (dmNewPelsHeight == Current->dmPelsHeight && dmNewPelsWidth == Current->dmPelsWidth)
//     {
//         /* No change */
//         return;
//     }
//
//     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//     InvalidateRect(GetDlgItem(hwndDlg, IDC_RESOLUTION_PREVIEW), NULL, TRUE);
//
//     dmBitsPerPel = Current->dmBitsPerPel;
//     dmDisplayFrequency = Current->dmDisplayFrequency;
//
//     if (CompareSettings(Current, dmNewPelsWidth,
//                         dmNewPelsHeight, dmBitsPerPel,
//                         dmDisplayFrequency) < 0)
//     {
//         Current = Current->Blink;
//         while (Current != NULL)
//         {
//             if (Current->dmPelsHeight == dmNewPelsHeight
//              && Current->dmPelsWidth  == dmNewPelsWidth
//              && Current->dmBitsPerPel == dmBitsPerPel)
//             {
//                 pData->CurrentDisplayDevice->CurrentSettings = Current;
//                 UpdateDisplay(hwndDlg, pData, bUpdateThumb);
//                 return;
//             }
//             Current = Current->Blink;
//         }
//     }
//     else
//     {
//         Current = Current->Flink;
//         while (Current != NULL)
//         {
//             if (Current->dmPelsHeight == dmNewPelsHeight
//              && Current->dmPelsWidth  == dmNewPelsWidth
//              && Current->dmBitsPerPel == dmBitsPerPel)
//             {
//                 pData->CurrentDisplayDevice->CurrentSettings = Current;
//                 UpdateDisplay(hwndDlg, pData, bUpdateThumb);
//                 return;
//             }
//             Current = Current->Flink;
//         }
//     }
//
//     /* Search bigger color depth compatible with current resolution */
//     Current = pData->CurrentDisplayDevice->CurrentSettings->Flink;
//     while (Current != NULL)
//     {
//         if (dmNewPelsHeight == Current->dmPelsHeight && dmNewPelsWidth == Current->dmPelsWidth)
//         {
//             pData->CurrentDisplayDevice->CurrentSettings = Current;
//             UpdateDisplay(hwndDlg, pData, bUpdateThumb);
//             return;
//         }
//         Current = Current->Flink;
//     }
//
//     /* Search smaller color depth compatible with current resolution */
//     Current = pData->CurrentDisplayDevice->CurrentSettings->Blink;
//     while (Current != NULL)
//     {
//         if (dmNewPelsHeight == Current->dmPelsHeight && dmNewPelsWidth == Current->dmPelsWidth)
//         {
//             pData->CurrentDisplayDevice->CurrentSettings = Current;
//             UpdateDisplay(hwndDlg, pData, bUpdateThumb);
//             return;
//         }
//         Current = Current->Blink;
//     }
//
//     /* We shouldn't go there */
// }
//
// /* Property sheet page callback */
// UINT __stdcall
// SettingsPageCallbackProc(HWND hwnd, UINT uMsg, LPPROPSHEETPAGE ppsp)
// {
//     UINT Ret = 0;
//
//     switch (uMsg)
//     {
//         case PSPCB_CREATE:
//             Ret = RegisterMonitorSelectionControl(hApplet);
//             break;
//
//         case PSPCB_RELEASE:
//             UnregisterMonitorSelectionControl(hApplet);
//             break;
//     }
//
//     return Ret;
// }
//
// static INT_PTR __stdcall
// ConfirmDlgProc( HWND hwndDlg,  UINT uMsg,  WPARAM wParam,  LPARAM lParam)
// {
//     PTIMEOUTDATA pData;
//
//     pData = (PTIMEOUTDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             /* Allocate the local dialog data */
//             pData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(TIMEOUTDATA));
//             if (pData == NULL)
//                 return FALSE;
//
//             /* Link the dialog data to the dialog */
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pData);
//
//             /* Timeout in seconds */
//             pData->nTimeout = 15;
//
//             /* Load the raw timeout string */
//             LoadString(hApplet, IDS_TIMEOUTTEXT, pData->szRawBuffer, _countof(pData->szRawBuffer));
//
//             /* Cook the timeout string and show it */
//             _stprintf(pData->szCookedBuffer, pData->szRawBuffer, pData->nTimeout);
//             SetDlgItemText(hwndDlg, IDC_TIMEOUTTEXT, pData->szCookedBuffer);
//
//             /* Start the timer (ticks every second)*/
//             SetTimer(hwndDlg, 1, 1000, NULL);
//             break;
//
//         case WM_TIMER:
//             /* Update the timepout value */
//             pData->nTimeout--;
//
//             /* Update the timeout text */
//             _stprintf(pData->szCookedBuffer, pData->szRawBuffer, pData->nTimeout);
//             SetDlgItemText(hwndDlg, IDC_TIMEOUTTEXT, pData->szCookedBuffer);
//
//             /* Kill the timer and return a 'No', if we ran out of time */
//             if (pData->nTimeout == 0)
//             {
//                 KillTimer(hwndDlg, 1);
//                 EndDialog(hwndDlg, IDNO);
//             }
//             break;
//
//         case WM_COMMAND:
//             /* Kill the timer and return the clicked button id */
//             KillTimer(hwndDlg, 1);
//             EndDialog(hwndDlg, LOWORD(wParam));
//             break;
//
//         case WM_DESTROY:
//             /* Free the local dialog data */
//             HeapFree(GetProcessHeap(), 0, pData);
//             break;
//     }
//
//     return FALSE;
// }
//
// BOOL
// SwitchDisplayMode(HWND hwndDlg, PWSTR DeviceName, PSETTINGS_ENTRY seInit, PSETTINGS_ENTRY seNew,  PLONG rc)
// {
//     TCHAR Message[1024], Title[256];
//     DEVMODE devmode;
//
//     RtlZeroMemory(&devmode, sizeof(devmode));
//     devmode.dmSize = (WORD)sizeof(devmode);
//     devmode.dmPelsWidth = seNew->dmPelsWidth;
//     devmode.dmPelsHeight = seNew->dmPelsHeight;
//     devmode.dmBitsPerPel = seNew->dmBitsPerPel;
//     devmode.dmDisplayFrequency = seNew->dmDisplayFrequency;
//     devmode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_BITSPERPEL | DM_DISPLAYFREQUENCY;
//
//     *rc = ChangeDisplaySettingsEx(DeviceName,
//                                   &devmode,
//                                   NULL,
//                                   CDS_UPDATEREGISTRY,
//                                   NULL);
//     switch (*rc)
//     {
//         case DISP_CHANGE_SUCCESSFUL:
//             break;
//
//         case DISP_CHANGE_RESTART:
//             LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title, _countof(Title));
//             LoadString(hApplet, IDS_APPLY_NEEDS_RESTART, Message, _countof(Message));
//             MessageBox(hwndDlg, Message, Title, MB_OK | MB_ICONINFORMATION);
//             return FALSE;
//
//         case DISP_CHANGE_FAILED:
//         default:
//             LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title, _countof(Title));
//             LoadString(hApplet, IDS_APPLY_FAILED, Message, _countof(Message));
//             MessageBox(hwndDlg, Message, Title, MB_OK | MB_ICONSTOP);
//             return FALSE;
//     }
//
//     if (DialogBox(hApplet, MAKEINTRESOURCE(IDD_CONFIRMSETTINGS), hwndDlg, ConfirmDlgProc) == IDYES)
//     {
//         return TRUE;
//     }
//     else
//     {
//         devmode.dmPelsWidth = seInit->dmPelsWidth;
//         devmode.dmPelsHeight = seInit->dmPelsHeight;
//         devmode.dmBitsPerPel = seInit->dmBitsPerPel;
//         devmode.dmDisplayFrequency = seInit->dmDisplayFrequency;
//
//         *rc = ChangeDisplaySettingsEx(DeviceName,
//                                       &devmode,
//                                       NULL,
//                                       CDS_UPDATEREGISTRY,
//                                       NULL);
//         switch (*rc)
//         {
//             case DISP_CHANGE_SUCCESSFUL:
//                 return FALSE;
//
//             case DISP_CHANGE_RESTART:
//                 LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title, _countof(Title));
//                 LoadString(hApplet, IDS_APPLY_NEEDS_RESTART, Message, _countof(Message));
//                 MessageBox(hwndDlg, Message, Title, MB_OK | MB_ICONINFORMATION);
//                 return FALSE;
//
//             case DISP_CHANGE_FAILED:
//             default:
//                 LoadString(hApplet, IDS_DISPLAY_SETTINGS, Title, _countof(Title));
//                 LoadString(hApplet, IDS_APPLY_FAILED, Message, _countof(Message));
//                 MessageBox(hwndDlg, Message, Title, MB_OK | MB_ICONSTOP);
//                 return FALSE;
//         }
//     }
// }
//
// static
// PSETTINGS_ENTRY
// FindBestElement(
//      PDISPLAY_DEVICE_ENTRY pDevice)
// {
//     PSETTINGS_ENTRY Request = &pDevice->InitialSettings, BestEntry = NULL, Current;
//     LONG Distance, NearestDistance = MAXLONG;
//
//     /* Find the best entry in the list */
//     for (Current = pDevice->Settings; Current; Current = Current->Flink)
//     {
//         Distance = 0x100000 * labs(Current->dmBitsPerPel       - Request->dmBitsPerPel      ) +
//                       0x100 * labs(Current->dmPelsWidth        - Request->dmPelsWidth       ) +
//                       0x100 * labs(Current->dmPelsHeight       - Request->dmPelsHeight      ) +
//                               labs(Current->dmDisplayFrequency - Request->dmDisplayFrequency);
//         if (Distance == 0)
//             return Current;
//
//         if (Distance < NearestDistance)
//         {
//             BestEntry = Current;
//             NearestDistance = Distance;
//         }
//     }
//
//     return BestEntry;
// }
//
// static VOID
// ApplyDisplaySettings(HWND hwndDlg, PSETTINGS_DATA pData)
// {
//     BOOL Ret;
//     LONG rc;
//
//     Ret = SwitchDisplayMode(hwndDlg,
//                             pData->CurrentDisplayDevice->DeviceName,
//                             &pData->CurrentDisplayDevice->InitialSettings,
//                             pData->CurrentDisplayDevice->CurrentSettings,
//                             &rc);
//
//     if (rc != DISP_CHANGE_SUCCESSFUL)
//         return;
//
//     if (Ret)
//     {
//         pData->CurrentDisplayDevice->InitialSettings.dmPelsWidth = pData->CurrentDisplayDevice->CurrentSettings->dmPelsWidth;
//         pData->CurrentDisplayDevice->InitialSettings.dmPelsHeight = pData->CurrentDisplayDevice->CurrentSettings->dmPelsHeight;
//         pData->CurrentDisplayDevice->InitialSettings.dmBitsPerPel = pData->CurrentDisplayDevice->CurrentSettings->dmBitsPerPel;
//         pData->CurrentDisplayDevice->InitialSettings.dmDisplayFrequency = pData->CurrentDisplayDevice->CurrentSettings->dmDisplayFrequency;
//     }
//     else
//     {
//         pData->CurrentDisplayDevice->CurrentSettings = FindBestElement(pData->CurrentDisplayDevice);
//         UpdateDisplay(hwndDlg, pData, TRUE);
//     }
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// SettingsPageProc( HWND hwndDlg,  UINT uMsg,  WPARAM wParam,  LPARAM lParam)
// {
//     PSETTINGS_DATA pData;
//
//     pData = (PSETTINGS_DATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             SettingsOnInitDialog(hwndDlg);
//             break;
//         }
//         case WM_DRAWITEM:
//         {
//             LPDRAWITEMSTRUCT lpDrawItem;
//             lpDrawItem = (LPDRAWITEMSTRUCT)lParam;
//
//             switch (lpDrawItem->CtlID)
//             {
//                 case IDC_RESOLUTION_PREVIEW:
//                 {
//                     ShowResolutionPreview(lpDrawItem, pData);
//                     break;
//                 }
//                 case IDC_SETTINGS_SPECTRUM:
//                 {
//                     ShowColorSpectrum(lpDrawItem->hDC, &lpDrawItem->rcItem, pData->CurrentDisplayDevice->CurrentSettings->dmBitsPerPel, pData);
//                     break;
//                 }
//             }
//             break;
//         }
//         case WM_COMMAND:
//         {
//             DWORD controlId = LOWORD(wParam);
//             DWORD command   = HIWORD(wParam);
//
//             if (controlId == IDC_SETTINGS_ADVANCED && command == BN_CLICKED)
//             {
//                 if (DisplayAdvancedSettings(hwndDlg, pData->CurrentDisplayDevice))
//                 {
//                     DEVMODE devmode;
//                     ZeroMemory(&devmode, sizeof(devmode));
//                     devmode.dmSize = (WORD)sizeof(devmode);
//                     if (EnumDisplaySettingsExW(pData->CurrentDisplayDevice->DeviceName,
//                                                ENUM_CURRENT_SETTINGS, &devmode, 0))
//                     {
//                         PSETTINGS_ENTRY pInitialSettings = &pData->CurrentDisplayDevice->InitialSettings;
//                         pInitialSettings->dmPelsWidth = devmode.dmPelsWidth;
//                         pInitialSettings->dmPelsHeight = devmode.dmPelsHeight;
//                         pInitialSettings->dmBitsPerPel = devmode.dmBitsPerPel;
//                         pInitialSettings->dmDisplayFrequency = devmode.dmDisplayFrequency;
//                         pData->CurrentDisplayDevice->CurrentSettings =
//                             FindBestElement(pData->CurrentDisplayDevice);
//                         UpdateDisplay(hwndDlg, pData, TRUE);
//                     }
//                 }
//             }
//             else if (controlId == IDC_SETTINGS_BPP && command == CBN_SELCHANGE)
//                 OnBPPChanged(hwndDlg, pData);
//             break;
//         }
//         case WM_HSCROLL:
//         {
//             switch (LOWORD(wParam))
//             {
//                 case TB_LINEUP:
//                 case TB_LINEDOWN:
//                 case TB_PAGEUP:
//                 case TB_PAGEDOWN:
//                 case TB_TOP:
//                 case TB_BOTTOM:
//                 case TB_ENDTRACK:
//                 {
//                     DWORD newPosition = (DWORD)SendDlgItemMessage(hwndDlg, IDC_SETTINGS_RESOLUTION, TBM_GETPOS, 0, 0);
//                     OnResolutionChanged(hwndDlg, pData, newPosition, TRUE);
//                     break;
//                 }
//
//                 case TB_THUMBTRACK:
//                     OnResolutionChanged(hwndDlg, pData, HIWORD(wParam), FALSE);
//                     break;
//             }
//             break;
//         }
//         case WM_NOTIFY:
//         {
//             LPNMHDR lpnm = (LPNMHDR)lParam;
//             if (lpnm->code == (UINT)PSN_APPLY)
//             {
//                 if (pData->CurrentDisplayDevice->CurrentSettings->dmPelsWidth != pData->CurrentDisplayDevice->InitialSettings.dmPelsWidth
//                  || pData->CurrentDisplayDevice->CurrentSettings->dmPelsHeight != pData->CurrentDisplayDevice->InitialSettings.dmPelsHeight
//                  || pData->CurrentDisplayDevice->CurrentSettings->dmBitsPerPel != pData->CurrentDisplayDevice->InitialSettings.dmBitsPerPel)
//                 {
//                     /* Apply new settings */
//                     ApplyDisplaySettings(hwndDlg, pData);
//                 }
//             }
//             else if (lpnm->code == MSLN_MONITORCHANGED)
//             {
//                 PMONSL_MONNMMONITORCHANGING lpnmi = (PMONSL_MONNMMONITORCHANGING)lParam;
//                 PDISPLAY_DEVICE_ENTRY Current = pData->DisplayDeviceList;
//                 ULONG i;
//                 for (i = 0; i < lpnmi->hdr.Index; i++)
//                     Current = Current->Flink;
//                 OnDisplayDeviceChanged(hwndDlg, pData, Current);
//             }
//             break;
//         }
//
//         case WM_CONTEXTMENU:
//         {
//             HWND hwndMonSel;
//             HMENU hPopup;
//             UINT uiCmd;
//             POINT pt, ptClient;
//             INT Index;
//
//             pt.x = (SHORT)LOWORD(lParam);
//             pt.y = (SHORT)HIWORD(lParam);
//
//             hwndMonSel = GetDlgItem(hwndDlg, IDC_SETTINGS_MONSEL);
//             if ((HWND)wParam == hwndMonSel)
//             {
//                 if (pt.x == -1 && pt.y == -1)
//                 {
//                     RECT rcMon;
//
//                     Index = (INT)SendMessage(hwndMonSel,
//                                              MSLM_GETCURSEL,
//                                              0,
//                                              0);
//
//                     if (Index >= 0 &&
//                         (INT)SendMessage(hwndMonSel,
//                                          MSLM_GETMONITORRECT,
//                                          Index,
//                                          (LPARAM)&rcMon) > 0)
//                     {
//                         pt.x = rcMon.left + ((rcMon.right - rcMon.left) / 2);
//                         pt.y = rcMon.top + ((rcMon.bottom - rcMon.top) / 2);
//                     }
//                     else
//                         pt.x = pt.y = 0;
//
//                     MapWindowPoints(hwndMonSel, NULL, &pt, 1);
//                 }
//                 else
//                 {
//                     ptClient = pt;
//                     MapWindowPoints(NULL, hwndMonSel, &ptClient, 1);
//
//                     Index = (INT)SendMessage(hwndMonSel,
//                                              MSLM_HITTEST,
//                                              (WPARAM)&ptClient,
//                                              0);
//                 }
//
//                 if (Index >= 0)
//                 {
//                     hPopup = LoadPopupMenu(hApplet,
//                                            MAKEINTRESOURCE(IDM_MONITOR_MENU));
//                     if (hPopup != NULL)
//                     {
//                         /* FIXME: Enable/Disable menu items */
//                         EnableMenuItem(hPopup,
//                                        ID_MENU_ATTACHED,
//                                        MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
//                         EnableMenuItem(hPopup,
//                                        ID_MENU_PRIMARY,
//                                        MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
//                         EnableMenuItem(hPopup,
//                                        ID_MENU_IDENTIFY,
//                                        MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
//                         EnableMenuItem(hPopup,
//                                        ID_MENU_PROPERTIES,
//                                        MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
//
//                         uiCmd = (UINT)TrackPopupMenu(hPopup,
//                                                      TPM_RETURNCMD | TPM_RIGHTBUTTON,
//                                                      pt.x,
//                                                      pt.y,
//                                                      0,
//                                                      hwndDlg,
//                                                      NULL);
//
//                         switch (uiCmd)
//                         {
//                             case ID_MENU_ATTACHED:
//                             case ID_MENU_PRIMARY:
//                             case ID_MENU_IDENTIFY:
//                             case ID_MENU_PROPERTIES:
//                                 /* FIXME: Implement */
//                                 break;
//                         }
//
//                         DestroyMenu(hPopup);
//                     }
//                 }
//             }
//             break;
//         }
//
//         case WM_DESTROY:
//         {
//             DWORD i;
//             PDISPLAY_DEVICE_ENTRY Current = pData->DisplayDeviceList;
//
//             while (Current != NULL)
//             {
//                 PDISPLAY_DEVICE_ENTRY Next = Current->Flink;
//                 PSETTINGS_ENTRY CurrentSettings = Current->Settings;
//                 while (CurrentSettings != NULL)
//                 {
//                     PSETTINGS_ENTRY NextSettings = CurrentSettings->Flink;
//                     HeapFree(GetProcessHeap(), 0, CurrentSettings);
//                     CurrentSettings = NextSettings;
//                 }
//                 HeapFree(GetProcessHeap(), 0, Current);
//                 Current = Next;
//             }
//
//             for (i = 0; i < NUM_SPECTRUM_BITMAPS; i++)
//             {
//                 if (pData->hSpectrumBitmaps[i])
//                     DeleteObject(pData->hSpectrumBitmaps[i]);
//             }
//
//             HeapFree(GetProcessHeap(), 0, pData);
//         }
//     }
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
