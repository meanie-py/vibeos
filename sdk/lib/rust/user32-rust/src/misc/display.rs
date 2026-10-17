use crate::ntwrapper::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
use win32k_sys::win32k::*;
use crate::internal::*;
use crate::winbase::*;
use crate::FIELD_OFFSET;
use core::ffi::c_void;

extern "system" {
    fn NtUserEnumDisplayDevices( Device: PUNICODE_STRING, iDevNum: u32, lpDisplayDevice: *mut core::ffi::c_void, dwFlags: u32, ) -> i32;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree( hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void, ) -> i32;
    fn NtUserEnumDisplayMonitors( hdc: HDC, lprcClip: LPCRECT, hMonitorList: *mut HMONITOR, pRectList: LPRECT, listSize: i32, ) -> i32;
    fn SetLastError(dwErrCode: u32);
    fn NtUserMonitorFromPoint(ptPoint: POINT, dwFlags: u32) -> HMONITOR;
    fn NtUserMonitorFromRect(lpcRect: LPCRECT, dwFlags: u32) -> HMONITOR;
    fn NtUserMonitorFromWindow(hWnd: HWND, dwFlags: u32) -> HMONITOR;
    fn GdiConvertToDevmodeW(dm: LPDEVMODEA) -> LPDEVMODEW;
    fn NtUserChangeDisplaySettings( pDeviceName: PUNICODE_STRING, pDevMode: LPDEVMODEW, dwflags: DWORD, lParam: LPVOID, ) -> LONG;
}

// ---- file-scope preamble (translated) ----
pub const SIZEOF_DEVMODEA_300: u32 = 124;
pub const SIZEOF_DEVMODEA_400: u32 = 148;
pub const SIZEOF_DEVMODEA_500: u32 = 156;
pub const SIZEOF_DEVMODEW_300: u32 = 188;
pub const SIZEOF_DEVMODEW_400: u32 = 212;
pub const SIZEOF_DEVMODEW_500: u32 = 220;

#[no_mangle]
pub unsafe extern "system" fn EnumDisplayDevicesA(
    lpDevice: LPCSTR,
    iDevNum: u32,
    lpDisplayDevice: PDISPLAY_DEVICEA,
    dwFlags: u32,
) -> i32 {
    let mut Device: _UNICODE_STRING = core::mem::zeroed();
    let mut DisplayDeviceW: DISPLAY_DEVICEW = core::mem::zeroed();

    if RtlCreateUnicodeStringFromAsciiz(&mut Device, lpDevice as PCSZ) == 0 {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }

    RtlZeroMemory(
        &mut DisplayDeviceW as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<DISPLAY_DEVICEW>(),
    );
    DisplayDeviceW.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as u32;

    let rc = NtUserEnumDisplayDevices(
        &mut Device,
        iDevNum,
        &mut DisplayDeviceW as *mut _ as *mut core::ffi::c_void,
        dwFlags,
    );

    if rc != 0 {
        (*lpDisplayDevice).StateFlags = DisplayDeviceW.StateFlags;
        WideCharToMultiByte(
            CP_ACP,
            0,
            DisplayDeviceW.DeviceName.as_ptr(),
            -1,
            (*lpDisplayDevice).DeviceName.as_mut_ptr(),
            (core::mem::size_of_val(&(*lpDisplayDevice).DeviceName)
                / core::mem::size_of_val(&(*lpDisplayDevice).DeviceName[0])) as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            DisplayDeviceW.DeviceString.as_ptr(),
            -1,
            (*lpDisplayDevice).DeviceString.as_mut_ptr(),
            (core::mem::size_of_val(&(*lpDisplayDevice).DeviceString)
                / core::mem::size_of_val(&(*lpDisplayDevice).DeviceString[0])) as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            DisplayDeviceW.DeviceID.as_ptr(),
            -1,
            (*lpDisplayDevice).DeviceID.as_mut_ptr(),
            (core::mem::size_of_val(&(*lpDisplayDevice).DeviceID)
                / core::mem::size_of_val(&(*lpDisplayDevice).DeviceID[0])) as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            DisplayDeviceW.DeviceKey.as_ptr(),
            -1,
            (*lpDisplayDevice).DeviceKey.as_mut_ptr(),
            (core::mem::size_of_val(&(*lpDisplayDevice).DeviceKey)
                / core::mem::size_of_val(&(*lpDisplayDevice).DeviceKey[0])) as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }

    RtlFreeUnicodeString(&mut Device);

    rc
}

#[no_mangle]
pub unsafe extern "system" fn EnumDisplayDevicesW(
    lpDevice: LPCWSTR,
    iDevNum: u32,
    lpDisplayDevice: *mut core::ffi::c_void,
    dwFlags: u32,
) -> i32 {
    let mut Device: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Device, lpDevice as PCWSTR);

    let rc = NtUserEnumDisplayDevices(&mut Device, iDevNum, lpDisplayDevice, dwFlags);

    rc
}

#[no_mangle]
pub unsafe extern "system" fn EnumDisplayMonitors(
    hdc: HDC,
    lprcClip: LPCRECT,
    lpfnEnum: MONITORENUMPROC,
    dwData: LPARAM,
) -> i32 {
    let mut iCount: i32;
    let mut hMonitorList: *mut HMONITOR;
    let mut pRectList: LPRECT;
    let hHeap: *mut core::ffi::c_void;
    let mut ret: i32 = 0;

    iCount = NtUserEnumDisplayMonitors(hdc, lprcClip, core::ptr::null_mut(), core::ptr::null_mut(), 0);
    if iCount < 0 {
        return 0;
    }
    if iCount == 0 {
        return 1;
    }

    hHeap = GetProcessHeap();
    hMonitorList = HeapAlloc(
        hHeap,
        0,
        (core::mem::size_of::<HMONITOR>() * (iCount) as usize) as SIZE_T,
    ) as *mut HMONITOR;
    if hMonitorList.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    pRectList = HeapAlloc(
        hHeap,
        0,
        (core::mem::size_of::<RECT>() * (iCount) as usize) as SIZE_T,
    ) as LPRECT;
    if pRectList.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        if !hMonitorList.is_null() {
            HeapFree(hHeap, 0, hMonitorList as *mut core::ffi::c_void);
        }
        if !pRectList.is_null() {
            HeapFree(hHeap, 0, pRectList as *mut core::ffi::c_void);
        }
        return ret;
    }

    'cleanup: {
        iCount = NtUserEnumDisplayMonitors(hdc, lprcClip, hMonitorList, pRectList, iCount);
        if iCount <= 0 {
            break 'cleanup;
        }

        let mut i: i32 = 0;
        while i < iCount {
            let hMonitor: HMONITOR = *hMonitorList.offset(i as isize);
            let pMonitorRect: LPRECT = pRectList.offset(i as isize);
            let mut hMonitorDC: HDC = core::ptr::null_mut();

            if !hdc.is_null() {
                hMonitorDC = hdc;
            }

            if lpfnEnum.unwrap()(hMonitor, hMonitorDC, pMonitorRect as *mut _, dwData) == 0 {
                break 'cleanup;
            }

            i += 1;
        }

        ret = 1;
    }

    if !hMonitorList.is_null() {
        HeapFree(hHeap, 0, hMonitorList as *mut core::ffi::c_void);
    }
    if !pRectList.is_null() {
        HeapFree(hHeap, 0, pRectList as *mut core::ffi::c_void);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumDisplaySettingsExA(
    lpszDeviceName: LPCSTR,
    iModeNum: DWORD,
    lpDevMode: LPDEVMODEA,
    dwFlags: DWORD,
) -> BOOL {
    let Status: NTSTATUS;
    let mut usDeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut pusDeviceName: PUNICODE_STRING = core::ptr::null_mut();
    let lpExtendedDevMode: LPDEVMODEW;
    let mut Result: BOOL = 0;

    if !lpszDeviceName.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(
            core::ptr::addr_of_mut!(usDeviceName),
            lpszDeviceName as PCSZ,
        ) == 0
        {
            SetLastError(ERROR_OUTOFMEMORY);
            return 0;
        }
        pusDeviceName = core::ptr::addr_of_mut!(usDeviceName);
    }

    lpExtendedDevMode = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<DEVMODEW>() + (*lpDevMode).dmDriverExtra as usize) as SIZE_T,
    ) as LPDEVMODEW;

    if lpExtendedDevMode.is_null() {
        if !pusDeviceName.is_null() {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(usDeviceName));
        }
        return 0;
    }

    (*lpExtendedDevMode).dmSize = core::mem::size_of::<DEVMODEW>() as WORD;
    (*lpExtendedDevMode).dmDriverExtra = (*lpDevMode).dmDriverExtra;

    Status = NtUserEnumDisplaySettings(pusDeviceName, iModeNum, lpExtendedDevMode, dwFlags);

    if !pusDeviceName.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(usDeviceName));
    }

    if NT_SUCCESS(Status) {
        let OldSize: WORD = (*lpDevMode).dmSize;

        WideCharToMultiByte(
            CP_THREAD_ACP,
            0,
            (*lpExtendedDevMode).dmDeviceName.as_ptr(),
            CCHDEVICENAME as i32,
            (*lpDevMode).dmDeviceName.as_mut_ptr() as LPSTR,
            CCHDEVICENAME as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        (*lpDevMode).dmSpecVersion = (*lpExtendedDevMode).dmSpecVersion;
        (*lpDevMode).dmDriverVersion = (*lpExtendedDevMode).dmDriverVersion;
        (*lpDevMode).dmFields = (*lpExtendedDevMode).dmFields;
        (*lpDevMode).__bindgen_anon_1.__bindgen_anon_2.dmPosition.x =
            (*lpExtendedDevMode).__bindgen_anon_1.__bindgen_anon_2.dmPosition.x;
        (*lpDevMode).__bindgen_anon_1.__bindgen_anon_2.dmPosition.y =
            (*lpExtendedDevMode).__bindgen_anon_1.__bindgen_anon_2.dmPosition.y;
        (*lpDevMode).__bindgen_anon_1.__bindgen_anon_1.dmScale = (*lpExtendedDevMode).__bindgen_anon_1.__bindgen_anon_1.dmScale;
        (*lpDevMode).__bindgen_anon_1.__bindgen_anon_1.dmCopies = (*lpExtendedDevMode).__bindgen_anon_1.__bindgen_anon_1.dmCopies;
        (*lpDevMode).__bindgen_anon_1.__bindgen_anon_1.dmDefaultSource = (*lpExtendedDevMode).__bindgen_anon_1.__bindgen_anon_1.dmDefaultSource;
        (*lpDevMode).__bindgen_anon_1.__bindgen_anon_1.dmPrintQuality = (*lpExtendedDevMode).__bindgen_anon_1.__bindgen_anon_1.dmPrintQuality;
        (*lpDevMode).dmColor = (*lpExtendedDevMode).dmColor;
        (*lpDevMode).dmDuplex = (*lpExtendedDevMode).dmDuplex;
        (*lpDevMode).dmYResolution = (*lpExtendedDevMode).dmYResolution;
        (*lpDevMode).dmTTOption = (*lpExtendedDevMode).dmTTOption;
        (*lpDevMode).dmCollate = (*lpExtendedDevMode).dmCollate;
        WideCharToMultiByte(
            CP_THREAD_ACP,
            0,
            (*lpExtendedDevMode).dmFormName.as_ptr(),
            CCHFORMNAME as i32,
            (*lpDevMode).dmFormName.as_mut_ptr() as LPSTR,
            CCHFORMNAME as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        (*lpDevMode).dmLogPixels = (*lpExtendedDevMode).dmLogPixels;
        (*lpDevMode).dmBitsPerPel = (*lpExtendedDevMode).dmBitsPerPel;
        (*lpDevMode).dmPelsWidth = (*lpExtendedDevMode).dmPelsWidth;
        (*lpDevMode).dmPelsHeight = (*lpExtendedDevMode).dmPelsHeight;
        (*lpDevMode).__bindgen_anon_2.dmDisplayFlags =
            (*lpExtendedDevMode).__bindgen_anon_2.dmDisplayFlags;
        (*lpDevMode).dmDisplayFrequency = (*lpExtendedDevMode).dmDisplayFrequency;

        if OldSize as i32 > SIZEOF_DEVMODEA_300 as i32 {
            (*lpDevMode).dmICMMethod = (*lpExtendedDevMode).dmICMMethod;
            (*lpDevMode).dmICMIntent = (*lpExtendedDevMode).dmICMIntent;
            (*lpDevMode).dmMediaType = (*lpExtendedDevMode).dmMediaType;
            (*lpDevMode).dmDitherType = (*lpExtendedDevMode).dmDitherType;
            (*lpDevMode).dmReserved1 = (*lpExtendedDevMode).dmReserved1;
            (*lpDevMode).dmReserved2 = (*lpExtendedDevMode).dmReserved2;

            if OldSize as i32 > SIZEOF_DEVMODEA_400 as i32 {
                (*lpDevMode).dmPanningWidth = (*lpExtendedDevMode).dmPanningWidth;
                (*lpDevMode).dmPanningHeight = (*lpExtendedDevMode).dmPanningHeight;
            }
        }

        (*lpDevMode).dmSize = OldSize;

        if (*lpDevMode).dmDriverExtra != 0 && (*lpExtendedDevMode).dmDriverExtra != 0 {
            if (*lpDevMode).dmDriverExtra > (*lpExtendedDevMode).dmDriverExtra {
                (*lpDevMode).dmDriverExtra = (*lpExtendedDevMode).dmDriverExtra;
            }

            core::ptr::copy_nonoverlapping(
                lpExtendedDevMode.add(1) as *const u8,
                (lpDevMode as PUCHAR).add(OldSize as usize),
                (*lpDevMode).dmDriverExtra as usize,
            );
        }

        if (OldSize as usize) < FIELD_OFFSET!(DEVMODEA, dmPanningHeight) {
            (*lpDevMode).dmFields &= !DM_PANNINGHEIGHT;
        }

        if (OldSize as usize) < FIELD_OFFSET!(DEVMODEA, dmPanningWidth) {
            (*lpDevMode).dmFields &= !DM_PANNINGWIDTH;
        }

        Result = 1;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, lpExtendedDevMode as PVOID);

    Result
}

#[no_mangle]
pub unsafe extern "system" fn EnumDisplaySettingsA(
    lpszDeviceName: LPCSTR,
    iModeNum: DWORD,
    lpDevMode: LPDEVMODEA,
) -> BOOL {
    (*lpDevMode).dmSize = FIELD_OFFSET!(DEVMODEA, dmICMMethod) as WORD;
    (*lpDevMode).dmDriverExtra = 0;

    EnumDisplaySettingsExA(lpszDeviceName, iModeNum, lpDevMode, 0)
}

#[no_mangle]
pub unsafe extern "system" fn EnumDisplaySettingsExW(
    lpszDeviceName: LPCWSTR,
    iModeNum: DWORD,
    lpDevMode: LPDEVMODEW,
    dwFlags: DWORD,
) -> BOOL {
    let Status: NTSTATUS;
    let mut usDeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut pusDeviceName: PUNICODE_STRING = core::ptr::null_mut();
    let lpExtendedDevMode: LPDEVMODEW;
    let mut Result: BOOL = 0;

    if !lpszDeviceName.is_null() {
        RtlInitUnicodeString(&mut usDeviceName, lpszDeviceName);
        pusDeviceName = &mut usDeviceName;
    }

    lpExtendedDevMode = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<DEVMODEW>() + (*lpDevMode).dmDriverExtra as usize) as SIZE_T,
    ) as LPDEVMODEW;
    if lpExtendedDevMode.is_null() {
        return 0;
    }

    (*lpExtendedDevMode).dmSize = core::mem::size_of::<DEVMODEW>() as WORD;
    (*lpExtendedDevMode).dmDriverExtra = (*lpDevMode).dmDriverExtra;

    Status = NtUserEnumDisplaySettings(pusDeviceName, iModeNum, lpExtendedDevMode, dwFlags);
    if NT_SUCCESS(Status) {
        let OldSize: WORD = (*lpDevMode).dmSize;
        let OldDriverExtra: WORD = (*lpDevMode).dmDriverExtra;

        RtlCopyMemory(
            lpDevMode as *mut core::ffi::c_void,
            lpExtendedDevMode as *const core::ffi::c_void,
            OldSize as usize,
        );

        (*lpDevMode).dmSize = OldSize;
        (*lpDevMode).dmDriverExtra = OldDriverExtra;

        if (*lpDevMode).dmDriverExtra != 0 && (*lpExtendedDevMode).dmDriverExtra != 0 {
            if (*lpDevMode).dmDriverExtra > (*lpExtendedDevMode).dmDriverExtra {
                (*lpDevMode).dmDriverExtra = (*lpExtendedDevMode).dmDriverExtra;
            }

            RtlCopyMemory(
                (lpDevMode as *mut u8).add(OldSize as usize) as *mut core::ffi::c_void,
                lpExtendedDevMode.add(1) as *const core::ffi::c_void,
                (*lpDevMode).dmDriverExtra as usize,
            );
        }

        if (OldSize as usize) < FIELD_OFFSET!(DEVMODEW, dmPanningHeight) {
            (*lpDevMode).dmFields &= !DM_PANNINGHEIGHT;
        }

        if (OldSize as usize) < FIELD_OFFSET!(DEVMODEW, dmPanningWidth) {
            (*lpDevMode).dmFields &= !DM_PANNINGWIDTH;
        }

        Result = 1;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, lpExtendedDevMode as *mut core::ffi::c_void);

    Result
}

#[no_mangle]
pub unsafe extern "system" fn EnumDisplaySettingsW(
    lpszDeviceName: LPCWSTR,
    iModeNum: DWORD,
    lpDevMode: LPDEVMODEW,
) -> BOOL {
    (*lpDevMode).dmSize = FIELD_OFFSET!(DEVMODEW, dmICMMethod) as WORD;
    (*lpDevMode).dmDriverExtra = 0;

    EnumDisplaySettingsExW(lpszDeviceName, iModeNum, lpDevMode, 0)
}

#[no_mangle]
pub unsafe extern "system" fn GetMonitorInfoA(
    hMonitor: HMONITOR,
    lpmi: LPMONITORINFO,
) -> BOOL {
    if (*lpmi).cbSize as usize == core::mem::size_of::<MONITORINFO>() {
        return NtUserGetMonitorInfo(hMonitor, lpmi);
    } else if (*lpmi).cbSize as usize != core::mem::size_of::<MONITORINFOEXA>() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    } else {
        let mut miExW: MONITORINFOEXW = core::mem::zeroed();
        let res: INT;

        miExW.cbSize = core::mem::size_of::<MONITORINFOEXW>() as _;
        if NtUserGetMonitorInfo(hMonitor, &mut miExW as *mut _ as LPMONITORINFO) == 0 {
            return FALSE as BOOL;
        }
        memcpy(
            lpmi as *mut _,
            &miExW as *const _ as *const _,
            core::mem::size_of::<MONITORINFO>(),
        );
        res = WideCharToMultiByte(
            CP_THREAD_ACP,
            0,
            miExW.szDevice.as_ptr(),
            -1,
            (*(lpmi as LPMONITORINFOEXA)).szDevice.as_mut_ptr(),
            CCHDEVICENAME as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if res == 0 {
            WARN!(b"WideCharToMultiByte() failed!\n\0".as_ptr() as LPCSTR);
            return FALSE as BOOL;
        }
    }

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetMonitorInfoW(
    hMonitor: HMONITOR,
    lpmi: LPMONITORINFO,
) -> i32 {
    NtUserGetMonitorInfo(hMonitor, lpmi)
}

#[no_mangle]
pub unsafe extern "system" fn MonitorFromPoint(ptPoint: POINT, dwFlags: u32) -> HMONITOR {
    NtUserMonitorFromPoint(ptPoint, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn MonitorFromRect(
    lpcRect: LPCRECT,
    dwFlags: u32,
) -> HMONITOR {
    NtUserMonitorFromRect(lpcRect, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn MonitorFromWindow(hWnd: HWND, dwFlags: u32) -> HMONITOR {
    NtUserMonitorFromWindow(hWnd, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn ChangeDisplaySettingsExA(
    lpszDeviceName: LPCSTR,
    lpDevMode: LPDEVMODEA,
    hwnd: HWND,
    dwflags: DWORD,
    lParam: LPVOID,
) -> LONG {
    let rc: LONG;
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut pDeviceName: PUNICODE_STRING = &mut DeviceName;

    if !lpszDeviceName.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(pDeviceName, lpszDeviceName as PCSZ) == 0 {
            SetLastError(ERROR_OUTOFMEMORY);
            return DISP_CHANGE_BADPARAM;
        }
    } else {
        pDeviceName = core::ptr::null_mut();
    }

    if !lpDevMode.is_null() {
        let pDevModeW = GdiConvertToDevmodeW(lpDevMode);
        if !pDevModeW.is_null() {
            rc = NtUserChangeDisplaySettings(pDeviceName, pDevModeW, dwflags, lParam);
            RtlFreeHeap(GetProcessHeap(), 0, pDevModeW as LPVOID);
        } else {
            rc = DISP_CHANGE_SUCCESSFUL as LONG;
        }
    } else {
        rc = NtUserChangeDisplaySettings(pDeviceName, core::ptr::null_mut(), dwflags, lParam);
    }

    if !lpszDeviceName.is_null() {
        RtlFreeUnicodeString(&mut DeviceName);
    }

    rc
}

#[no_mangle]
pub unsafe extern "system" fn ChangeDisplaySettingsA(
    lpDevMode: LPDEVMODEA,
    dwflags: DWORD,
) -> LONG {
    if !lpDevMode.is_null() {
        (*lpDevMode).dmDriverExtra = 0;
    }
    ChangeDisplaySettingsExA(
        core::ptr::null_mut(),
        lpDevMode,
        core::ptr::null_mut(),
        dwflags,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn ChangeDisplaySettingsExW(
    lpszDeviceName: LPCWSTR,
    lpDevMode: LPDEVMODEW,
    hwnd: HWND,
    dwflags: DWORD,
    lParam: LPVOID,
) -> LONG {
    let rc: LONG;
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut pDeviceName: PUNICODE_STRING = &mut DeviceName;

    if lpszDeviceName != core::ptr::null_mut() {
        RtlInitUnicodeString(pDeviceName, lpszDeviceName as PCWSTR);
    } else {
        pDeviceName = core::ptr::null_mut();
    }

    rc = NtUserChangeDisplaySettings(pDeviceName, lpDevMode, dwflags, lParam);

    rc
}

#[no_mangle]
pub unsafe extern "system" fn ChangeDisplaySettingsW(
    lpDevMode: LPDEVMODEW,
    dwflags: DWORD,
) -> LONG {
    if !lpDevMode.is_null() {
        (*lpDevMode).dmDriverExtra = 0;
    }
    ChangeDisplaySettingsExW(
        core::ptr::null_mut(),
        lpDevMode,
        core::ptr::null_mut(),
        dwflags,
        core::ptr::null_mut(),
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS user32.dll
//  * FILE:            win32ss/user/user32/misc/display.c
//  * PURPOSE:         DDE
//  * PROGRAMMER:      Casper S. Hornstrup (chorns@users.sourceforge.net)
//  */
//
// #include <user32.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(user32);
//
// #define SIZEOF_DEVMODEA_300 124
// #define SIZEOF_DEVMODEA_400 148
// #define SIZEOF_DEVMODEA_500 156
// #define SIZEOF_DEVMODEW_300 188
// #define SIZEOF_DEVMODEW_400 212
// #define SIZEOF_DEVMODEW_500 220
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// EnumDisplayDevicesA(
//     LPCSTR lpDevice,
//     DWORD iDevNum,
//     PDISPLAY_DEVICEA lpDisplayDevice,
//     DWORD dwFlags)
// {
//     BOOL rc;
//     UNICODE_STRING Device;
//     DISPLAY_DEVICEW DisplayDeviceW;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&Device, (PCSZ)lpDevice))
//     {
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//
//     RtlZeroMemory(&DisplayDeviceW, sizeof(DISPLAY_DEVICEW));
//     DisplayDeviceW.cb = sizeof(DISPLAY_DEVICEW);
//     rc = NtUserEnumDisplayDevices(&Device,
//                                   iDevNum,
//                                   &DisplayDeviceW,
//                                   dwFlags);
//     if (rc)
//     {
//         /* Copy result from DisplayDeviceW to lpDisplayDevice. Buffers have the same size so result is always NULL terminated. */
//         lpDisplayDevice->StateFlags = DisplayDeviceW.StateFlags;
//         WideCharToMultiByte(CP_ACP, 0, DisplayDeviceW.DeviceName, -1,
//                             lpDisplayDevice->DeviceName,
//                             sizeof(lpDisplayDevice->DeviceName) / sizeof(lpDisplayDevice->DeviceName[0]),
//                             NULL, NULL);
//         WideCharToMultiByte(CP_ACP, 0, DisplayDeviceW.DeviceString, -1,
//                             lpDisplayDevice->DeviceString,
//                             sizeof(lpDisplayDevice->DeviceString) / sizeof(lpDisplayDevice->DeviceString[0]),
//                             NULL, NULL);
//         WideCharToMultiByte(CP_ACP, 0, DisplayDeviceW.DeviceID, -1,
//                             lpDisplayDevice->DeviceID,
//                             sizeof(lpDisplayDevice->DeviceID) / sizeof(lpDisplayDevice->DeviceID[0]),
//                             NULL, NULL);
//         WideCharToMultiByte(CP_ACP, 0, DisplayDeviceW.DeviceKey, -1,
//                             lpDisplayDevice->DeviceKey,
//                             sizeof(lpDisplayDevice->DeviceKey) / sizeof(lpDisplayDevice->DeviceKey[0]),
//                             NULL, NULL);
//     }
//
//     RtlFreeUnicodeString(&Device);
//
//     return rc;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EnumDisplayDevicesW(
//     LPCWSTR lpDevice,
//     DWORD iDevNum,
//     PDISPLAY_DEVICEW lpDisplayDevice,
//     DWORD dwFlags)
// {
//     UNICODE_STRING Device;
//     BOOL rc;
//
//     RtlInitUnicodeString(&Device, lpDevice);
//
//     rc = NtUserEnumDisplayDevices(
//              &Device,
//              iDevNum,
//              lpDisplayDevice,
//              dwFlags);
//
//     return rc;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EnumDisplayMonitors(
//     HDC hdc,
//     LPCRECT lprcClip,
//     MONITORENUMPROC lpfnEnum,
//     LPARAM dwData)
// {
//     INT iCount, i;
//     HMONITOR *hMonitorList;
//     LPRECT pRectList;
//     HANDLE hHeap;
//     BOOL ret = FALSE;
//
//     /* get list of monitors/rects */
//     iCount = NtUserEnumDisplayMonitors(hdc, lprcClip, NULL, NULL, 0);
//     if (iCount < 0)
//     {
//         /* FIXME: SetLastError() */
//         return FALSE;
//     }
//     if (iCount == 0)
//     {
//         return TRUE;
//     }
//
//     hHeap = GetProcessHeap();
//     hMonitorList = HeapAlloc(hHeap, 0, sizeof (HMONITOR) * iCount);
//     if (hMonitorList == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//     pRectList = HeapAlloc(hHeap, 0, sizeof (RECT) * iCount);
//     if (pRectList == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto cleanup;
//     }
//
//     iCount = NtUserEnumDisplayMonitors(hdc, lprcClip, hMonitorList, pRectList, iCount);
//     if (iCount <= 0)
//     {
//         /* FIXME: SetLastError() */
//         goto cleanup;
//     }
//
//     /* enumerate list */
//     for (i = 0; i < iCount; i++)
//     {
//         HMONITOR hMonitor = hMonitorList[i];
//         LPRECT pMonitorRect = pRectList + i;
//         HDC hMonitorDC = NULL;
//
//         if (hdc != NULL)
//         {
//             /* make monitor DC */
//             hMonitorDC = hdc;
//         }
//
//         if (!lpfnEnum(hMonitor, hMonitorDC, pMonitorRect, dwData))
//             goto cleanup; /* return FALSE */
//     }
//     
//     ret = TRUE;
//     
// cleanup:
//     if(hMonitorList)
//         HeapFree(hHeap, 0, hMonitorList);
//     if(pRectList)
//         HeapFree(hHeap, 0, pRectList);
//     return ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EnumDisplaySettingsExA(
//     LPCSTR lpszDeviceName,
//     DWORD iModeNum,
//     LPDEVMODEA lpDevMode,
//     DWORD dwFlags)
// {
//     NTSTATUS Status;
//     UNICODE_STRING usDeviceName;
//     PUNICODE_STRING pusDeviceName = NULL;
//     LPDEVMODEW lpExtendedDevMode;
//     BOOL Result = FALSE;
//
//     if (lpszDeviceName)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&usDeviceName, (PCSZ)lpszDeviceName))
//         {
//             SetLastError(ERROR_OUTOFMEMORY);
//             return FALSE;
//         }
//         pusDeviceName = &usDeviceName;
//     }
//
//     /* Allocate memory for DEVMODEW and extra data */
//     lpExtendedDevMode = RtlAllocateHeap(RtlGetProcessHeap(),
//                                         HEAP_ZERO_MEMORY,
//                                         sizeof(DEVMODEW) + lpDevMode->dmDriverExtra);
//     if (!lpExtendedDevMode)
//     {
//         if (pusDeviceName)
//             RtlFreeUnicodeString(&usDeviceName);
//
//         return FALSE;
//     }
//
//     /* Initialize structure fields */
//     lpExtendedDevMode->dmSize = sizeof(DEVMODEW);
//     lpExtendedDevMode->dmDriverExtra = lpDevMode->dmDriverExtra;
//
//     Status = NtUserEnumDisplaySettings(pusDeviceName, iModeNum, lpExtendedDevMode, dwFlags);
//
//     if (pusDeviceName)
//         RtlFreeUnicodeString(&usDeviceName);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* Store old structure size */
//         WORD OldSize = lpDevMode->dmSize;
//
// #define COPYS(f,len) WideCharToMultiByte(CP_THREAD_ACP, 0, lpExtendedDevMode->f, len, (LPSTR)lpDevMode->f, len, NULL, NULL)
// #define COPYN(f) lpDevMode->f = lpExtendedDevMode->f
//
//         COPYS(dmDeviceName, CCHDEVICENAME);
//         COPYN(dmSpecVersion);
//         COPYN(dmDriverVersion);
//         COPYN(dmFields);
//         COPYN(dmPosition.x);
//         COPYN(dmPosition.y);
//         COPYN(dmScale);
//         COPYN(dmCopies);
//         COPYN(dmDefaultSource);
//         COPYN(dmPrintQuality);
//         COPYN(dmColor);
//         COPYN(dmDuplex);
//         COPYN(dmYResolution);
//         COPYN(dmTTOption);
//         COPYN(dmCollate);
//         COPYS(dmFormName,CCHFORMNAME);
//         COPYN(dmLogPixels);
//         COPYN(dmBitsPerPel);
//         COPYN(dmPelsWidth);
//         COPYN(dmPelsHeight);
//         COPYN(dmDisplayFlags); // aka dmNup
//         COPYN(dmDisplayFrequency);
//
//         /* we're done with 0x300 fields */
//         if (OldSize > SIZEOF_DEVMODEA_300)
//         {
//             COPYN(dmICMMethod);
//             COPYN(dmICMIntent);
//             COPYN(dmMediaType);
//             COPYN(dmDitherType);
//             COPYN(dmReserved1);
//             COPYN(dmReserved2);
//
//             /* we're done with 0x400 fields */
//             if (OldSize > SIZEOF_DEVMODEA_400)
//             {
//                 COPYN(dmPanningWidth);
//                 COPYN(dmPanningHeight);
//             }
//         }
//
//         /* Restore old size */
//         lpDevMode->dmSize = OldSize;
//
//         /* Extra data presented? */
//         if (lpDevMode->dmDriverExtra && lpExtendedDevMode->dmDriverExtra)
//         {
//             /* We choose the smallest size */
//             if (lpDevMode->dmDriverExtra > lpExtendedDevMode->dmDriverExtra)
//                 lpDevMode->dmDriverExtra = lpExtendedDevMode->dmDriverExtra;
//
//             /* Copy extra data */
//             RtlCopyMemory((PUCHAR)lpDevMode + OldSize,
//                           lpExtendedDevMode + 1,
//                           lpDevMode->dmDriverExtra);
//         }
//
//         /* If the size of source structure is less, than used, we clean unsupported flags */
//         if (OldSize < FIELD_OFFSET!(DEVMODEA, dmPanningHeight))
//             lpDevMode->dmFields &= ~DM_PANNINGHEIGHT;
//
//         if (OldSize < FIELD_OFFSET!(DEVMODEA, dmPanningWidth))
//             lpDevMode->dmFields &= ~DM_PANNINGWIDTH;
//
//         Result = TRUE;
//     }
//
//     /* Free memory */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, lpExtendedDevMode);
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EnumDisplaySettingsA(
//     LPCSTR lpszDeviceName,
//     DWORD iModeNum,
//     LPDEVMODEA lpDevMode)
// {
//     /* Fixup sizes */
//     lpDevMode->dmSize = FIELD_OFFSET!(DEVMODEA, dmICMMethod);
//     lpDevMode->dmDriverExtra = 0;
//
//     return EnumDisplaySettingsExA(lpszDeviceName, iModeNum, lpDevMode, 0);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EnumDisplaySettingsExW(
//     LPCWSTR lpszDeviceName,
//     DWORD iModeNum,
//     LPDEVMODEW lpDevMode,
//     DWORD dwFlags)
// {
//     NTSTATUS Status;
//     UNICODE_STRING usDeviceName;
//     PUNICODE_STRING pusDeviceName = NULL;
//     LPDEVMODEW lpExtendedDevMode;
//     BOOL Result = FALSE;
//
//     if (lpszDeviceName)
//     {
//         RtlInitUnicodeString(&usDeviceName, lpszDeviceName);
//         pusDeviceName = &usDeviceName;
//     }
//
//     /* Allocate memory for DEVMODEW and extra data */
//     lpExtendedDevMode = RtlAllocateHeap(RtlGetProcessHeap(),
//                                         HEAP_ZERO_MEMORY,
//                                         sizeof(DEVMODEW) + lpDevMode->dmDriverExtra);
//     if (!lpExtendedDevMode)
//         return FALSE;
//
//     /* Initialize structure fields */
//     lpExtendedDevMode->dmSize = sizeof(DEVMODEW);
//     lpExtendedDevMode->dmDriverExtra = lpDevMode->dmDriverExtra;
//
//     Status = NtUserEnumDisplaySettings(pusDeviceName, iModeNum, lpExtendedDevMode, dwFlags);
//     if (NT_SUCCESS(Status))
//     {
//         /* Store old structure sizes */
//         WORD OldSize = lpDevMode->dmSize;
//         WORD OldDriverExtra = lpDevMode->dmDriverExtra;
//
//         /* Copy general data */
//         RtlCopyMemory(lpDevMode, lpExtendedDevMode, OldSize);
//
//         /* Restore old sizes */
//         lpDevMode->dmSize = OldSize;
//         lpDevMode->dmDriverExtra = OldDriverExtra;
//
//         /* Extra data presented? */
//         if (lpDevMode->dmDriverExtra && lpExtendedDevMode->dmDriverExtra)
//         {
//             /* We choose the smallest size */
//             if (lpDevMode->dmDriverExtra > lpExtendedDevMode->dmDriverExtra)
//                 lpDevMode->dmDriverExtra = lpExtendedDevMode->dmDriverExtra;
//
//             /* Copy extra data */
//             RtlCopyMemory((PUCHAR)lpDevMode + OldSize,
//                           lpExtendedDevMode + 1,
//                           lpDevMode->dmDriverExtra);
//         }
//
//         /* If the size of source structure is less, than used, we clean unsupported flags */
//         if (OldSize < FIELD_OFFSET!(DEVMODEW, dmPanningHeight))
//             lpDevMode->dmFields &= ~DM_PANNINGHEIGHT;
//
//         if (OldSize < FIELD_OFFSET!(DEVMODEW, dmPanningWidth))
//             lpDevMode->dmFields &= ~DM_PANNINGWIDTH;
//
//         Result = TRUE;
//     }
//
//     /* Free memory */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, lpExtendedDevMode);
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EnumDisplaySettingsW(
//     LPCWSTR lpszDeviceName,
//     DWORD iModeNum,
//     LPDEVMODEW lpDevMode)
// {
//     /* Fixup sizes */
//     lpDevMode->dmSize = FIELD_OFFSET!(DEVMODEW, dmICMMethod);
//     lpDevMode->dmDriverExtra = 0;
//
//     return EnumDisplaySettingsExW(lpszDeviceName, iModeNum, lpDevMode, 0);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetMonitorInfoA(
//     HMONITOR hMonitor,
//     LPMONITORINFO lpmi)
// {
//     if (lpmi->cbSize == sizeof (MONITORINFO))
//     {
//         return NtUserGetMonitorInfo(hMonitor, lpmi);
//     }
//     else if (lpmi->cbSize != sizeof (MONITORINFOEXA))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//     else
//     {
//         MONITORINFOEXW miExW;
//         INT res;
//
//         miExW.cbSize = sizeof (MONITORINFOEXW);
//         if (!NtUserGetMonitorInfo(hMonitor, (LPMONITORINFO)&miExW))
//         {
//             return FALSE;
//         }
//         memcpy(lpmi, &miExW, sizeof (MONITORINFO));
//         res = WideCharToMultiByte(CP_THREAD_ACP, 0, miExW.szDevice, -1,
//                                   ((LPMONITORINFOEXA)lpmi)->szDevice, CCHDEVICENAME,
//                                   NULL, NULL);
//         if (res == 0)
//         {
//             WARN!("WideCharToMultiByte() failed!\n");
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetMonitorInfoW(
//     HMONITOR hMonitor,
//     LPMONITORINFO lpmi)
// {
//     return NtUserGetMonitorInfo(hMonitor, lpmi);
// }
//
//
// /*
//  * @implemented
//  */
// HMONITOR
// WINAPI
// MonitorFromPoint(
//     IN POINT ptPoint,
//     IN DWORD dwFlags )
// {
//     return NtUserMonitorFromPoint(ptPoint, dwFlags);
// }
//
//
// /*
//  * @implemented
//  */
// HMONITOR
// WINAPI
// MonitorFromRect(
//     IN LPCRECT lpcRect,
//     IN DWORD dwFlags )
// {
//     return NtUserMonitorFromRect(lpcRect, dwFlags);
// }
//
//
// /*
//  * @implemented
//  */
// HMONITOR
// WINAPI
// MonitorFromWindow(
//     IN HWND hWnd,
//     IN DWORD dwFlags )
// {
//     return NtUserMonitorFromWindow(hWnd, dwFlags);
// }
//
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// ChangeDisplaySettingsExA(
//     LPCSTR lpszDeviceName,
//     LPDEVMODEA lpDevMode,
//     HWND hwnd,
//     DWORD dwflags,
//     LPVOID lParam)
// {
//     LONG rc;
//     UNICODE_STRING DeviceName;
//     PUNICODE_STRING pDeviceName = &DeviceName;
//
//     if (lpszDeviceName != NULL)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(pDeviceName, (PCSZ)lpszDeviceName))
//         {
//             SetLastError(ERROR_OUTOFMEMORY);
//             return DISP_CHANGE_BADPARAM; /* FIXME what to return? */
//         }
//     }
//     else
//         pDeviceName = NULL;
//
//     if (lpDevMode != NULL)
//     {
//         LPDEVMODEW pDevModeW;
//         pDevModeW = GdiConvertToDevmodeW(lpDevMode);
//         if(pDevModeW)
//         {
//             rc = NtUserChangeDisplaySettings(pDeviceName, pDevModeW, dwflags, lParam);
//             RtlFreeHeap(GetProcessHeap(), 0, pDevModeW);
//         }
//         else
//             rc = DISP_CHANGE_SUCCESSFUL;
//     }
//     else
//         rc = NtUserChangeDisplaySettings(pDeviceName, NULL, dwflags, lParam);
//
//     if (lpszDeviceName != NULL)
//         RtlFreeUnicodeString(&DeviceName);
//
//     return rc;
// }
//
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// ChangeDisplaySettingsA(
//     LPDEVMODEA lpDevMode,
//     DWORD dwflags)
// {
//     if(lpDevMode)
//         lpDevMode->dmDriverExtra = 0;
//     return ChangeDisplaySettingsExA ( NULL, lpDevMode, NULL, dwflags, 0 );
// }
//
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// ChangeDisplaySettingsExW(
//     LPCWSTR lpszDeviceName,
//     LPDEVMODEW lpDevMode,
//     HWND hwnd,
//     DWORD dwflags,
//     LPVOID lParam)
// {
//     LONG rc;
//     UNICODE_STRING DeviceName;
//     PUNICODE_STRING pDeviceName = &DeviceName;
//
//     if (lpszDeviceName != NULL)
//         RtlInitUnicodeString(pDeviceName, lpszDeviceName);
//     else
//         pDeviceName = NULL;
//
//     rc = NtUserChangeDisplaySettings(pDeviceName, lpDevMode, dwflags, lParam);
//
//     return rc;
// }
//
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// ChangeDisplaySettingsW(
//     LPDEVMODEW lpDevMode,
//     DWORD dwflags)
// {
//     if(lpDevMode)
//         lpDevMode->dmDriverExtra = 0;
//     return ChangeDisplaySettingsExW(NULL, lpDevMode, NULL, dwflags, 0);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
