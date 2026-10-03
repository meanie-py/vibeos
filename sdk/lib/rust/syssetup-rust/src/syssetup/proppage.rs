// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _MOUSE_INFO {
    pub hDeviceKey: HKEY,
    pub dwSampleRateIndex: DWORD,
    pub dwWheelDetection: DWORD,
    pub dwInputBufferLength: DWORD,
    pub dwInitPolled: DWORD,
}

pub type MOUSE_INFO = _MOUSE_INFO;
pub type PMOUSE_INFO = *mut _MOUSE_INFO;

pub static MouseSampleRates: [DWORD; 6] = [20, 40, 60, 80, 100, 200];

pub const DEFAULT_SAMPLERATEINDEX: u32 = 4;
pub const DEFAULT_WHEELDETECTION: u32 = 2;
pub const DEFAULT_INPUTBUFFERSIZE: u32 = 100;
pub const DEFAULT_MINBUFFERSIZE: u32 = 100;
pub const DEFAULT_MAXBUFFERSIZE: u32 = 300;

#[no_mangle]
pub unsafe extern "system" fn CdromPropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    // DPRINT!("CdromPropPageProvider(%p %p %lx)\n", lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn DiskPropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    // DPRINT!("DiskPropPageProvider(%p %p %lx)\n", lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn EisaUpHalPropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    // DPRINT!("EisaUpHalPropPageProvider(%p %p %lx)\n", lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn LegacyDriverPropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    // DPRINT1!("LegacyDriverPropPageProvider(%p %p %lx)\n", lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
    // UNIMPLEMENTED;
    (FALSE) as i32
}
unsafe extern "C" fn MouseGetSampleRateIndex(dwSampleRate: DWORD) -> DWORD {
    let mut i: DWORD;

    i = 0;
    while i < ARRAYSIZE!(MouseSampleRates) {
        if MouseSampleRates[i as usize] == dwSampleRate {
            return i;
        }
        i += 1;
    }

    return DEFAULT_SAMPLERATEINDEX;
}
unsafe extern "C" fn MouseOnDialogInit(
    hwndDlg: HWND,
    lParam: LPARAM,
) -> BOOL {
    let mut pMouseInfo: PMOUSE_INFO;
    let mut szBuffer: [u16; 64] = [0; 64];
    let mut i: u32;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwSampleRate: DWORD = 0;
    let mut lError: i32;

    pMouseInfo = (*(lParam as *mut _PROPSHEETPAGEW)).lParam as PMOUSE_INFO;
    if pMouseInfo.is_null() {
        return (FALSE) as i32;
    }

    SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pMouseInfo as DWORD_PTR) as LONG_PTR);

    i = 0;
    while i < ARRAYSIZE!(MouseSampleRates) {
        wsprintf(
            szBuffer.as_mut_ptr(),
            // proppage.c:137 is `wsprintf(szBuffer, L"%lu", ..)`. The translation embedded the C
            // SOURCE SPELLING — the L and the quotes — as a narrow byte string behind a wide
            // pointer, so the format string was the six characters L " % l u " rather than %lu.
            w!("%lu") as *const u16,
            MouseSampleRates[i as usize],
        );
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_PS2MOUSESAMPLERATE) as i32,
            (CB_ADDSTRING) as u32,
            0,
            szBuffer.as_ptr() as LPARAM,
        );
        i += 1;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    lError = RegQueryValueExW(
        (*pMouseInfo).hDeviceKey,
        w!("SampleRate") as *const u16,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(dwSampleRate) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );
    if lError == (ERROR_SUCCESS) as i32 && dwType == (REG_DWORD) as u32 && dwSize == core::mem::size_of::<DWORD>() as DWORD {
        (*pMouseInfo).dwSampleRateIndex = MouseGetSampleRateIndex(dwSampleRate);
    } else {
        (*pMouseInfo).dwSampleRateIndex = DEFAULT_SAMPLERATEINDEX;
    }

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_PS2MOUSESAMPLERATE) as i32,
        CB_SETCURSEL,
        ((*pMouseInfo).dwSampleRateIndex) as WPARAM,
        0,
    );

    i = (IDS_DETECTIONDISABLED) as u32;
    while i <= (IDS_ASSUMEPRESENT) as u32 {
        LoadStringW(hDllInstance, i, szBuffer.as_mut_ptr(), (ARRAYSIZE!(szBuffer)) as i32);
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_PS2MOUSEWHEEL) as i32,
            (CB_ADDSTRING) as u32,
            0,
            szBuffer.as_ptr() as LPARAM,
        );
        i += 1;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    lError = RegQueryValueExW(
        (*pMouseInfo).hDeviceKey,
        w!("EnableWheelDetection") as *const u16,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!((*pMouseInfo).dwWheelDetection) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );
    if lError != (ERROR_SUCCESS) as i32 || dwType != (REG_DWORD) as u32 || dwSize != core::mem::size_of::<DWORD>() as DWORD {
        (*pMouseInfo).dwWheelDetection = DEFAULT_WHEELDETECTION;
    }

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_PS2MOUSEWHEEL) as i32,
        CB_SETCURSEL,
        ((*pMouseInfo).dwWheelDetection) as WPARAM,
        0,
    );

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_PS2MOUSEINPUTUPDN) as i32,
        UDM_SETRANGE32,
        (DEFAULT_MINBUFFERSIZE) as WPARAM,
        (DEFAULT_MAXBUFFERSIZE) as LPARAM,
    );

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    lError = RegQueryValueExW(
        (*pMouseInfo).hDeviceKey,
        w!("MouseDataQueueSize") as *const u16,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!((*pMouseInfo).dwInputBufferLength) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );
    if lError != (ERROR_SUCCESS) as i32 || dwType != (REG_DWORD) as u32 || dwSize != core::mem::size_of::<DWORD>() as DWORD {
        (*pMouseInfo).dwInputBufferLength = DEFAULT_INPUTBUFFERSIZE;
    }

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_PS2MOUSEINPUTUPDN) as i32,
        UDM_SETPOS32,
        0,
        ((*pMouseInfo).dwInputBufferLength) as LPARAM,
    );

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    lError = RegQueryValueExW(
        (*pMouseInfo).hDeviceKey,
        w!("MouseInitializePolled") as *const u16,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!((*pMouseInfo).dwInitPolled) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );
    if lError != (ERROR_SUCCESS) as i32 || dwType != (REG_DWORD) as u32 || dwSize != core::mem::size_of::<DWORD>() as DWORD {
        (*pMouseInfo).dwInitPolled = 0;
    }

    SendDlgItemMessage(
        hwndDlg,
        (IDC_PS2MOUSEFASTINIT) as i32,
        (BM_SETCHECK) as u32,
        if (*pMouseInfo).dwInitPolled == 0 {
            BST_CHECKED as WPARAM
        } else {
            0
        },
        0,
    );

    (TRUE) as i32
}
unsafe extern "C" fn MouseOnCommand(
    hwndDlg: HWND,
    wParam: WPARAM,
    lParam: LPARAM) {
    match (LOWORD!(wParam)) as u32 {
        IDC_PS2MOUSESAMPLERATE |
        IDC_PS2MOUSEWHEEL |
        IDC_PS2MOUSEINPUTLEN |
        IDC_PS2MOUSEFASTINIT => {
            if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 ||
                HIWORD!(wParam) == (CBN_EDITCHANGE) as u16
            {
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            }
        }

        IDC_PS2MOUSEDEFAULTS => {
            if HIWORD!(wParam) == (BN_CLICKED) as u16
            {
                SendDlgItemMessageW(hwndDlg,
                                    (IDC_PS2MOUSESAMPLERATE) as i32,
                                    CB_SETCURSEL,
                                    (DEFAULT_SAMPLERATEINDEX) as WPARAM,
                                    0);

                SendDlgItemMessageW(hwndDlg,
                                    (IDC_PS2MOUSEWHEEL) as i32,
                                    CB_SETCURSEL,
                                    (DEFAULT_WHEELDETECTION) as WPARAM,
                                    0);

                SendDlgItemMessageW(hwndDlg,
                                    (IDC_PS2MOUSEINPUTUPDN) as i32,
                                    UDM_SETPOS32,
                                    0,
                                    (DEFAULT_INPUTBUFFERSIZE) as LPARAM);

                SendDlgItemMessage(hwndDlg,
                                   (IDC_PS2MOUSEFASTINIT) as i32,
                                   (BM_SETCHECK) as u32,
                                   BST_CHECKED as WPARAM,
                                   0 as LPARAM);

                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            }
        }

        _ => {}
    }
}
unsafe extern "C" fn MouseOnApply(hwndDlg: HWND) {
    let mut pMouseInfo: *mut MOUSE_INFO;
    let mut dwSampleRateIndex: DWORD = 0;
    let mut dwSampleRate: DWORD;
    let mut dwWheelDetection: DWORD;
    let mut dwInputBufferLength: DWORD;
    let mut dwInitPolled: DWORD;
    let mut uValue: u32;
    let mut bFailed: i32 = 0;
    let mut nIndex: i32;

    pMouseInfo = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut MOUSE_INFO;

    nIndex = SendDlgItemMessageW(
        hwndDlg,
        (IDC_PS2MOUSESAMPLERATE) as i32,
        (CB_GETCURSEL) as u32,
        0,
        0,
    ) as i32;
    if nIndex == CB_ERR {
        dwSampleRateIndex = DEFAULT_SAMPLERATEINDEX;
    } else {
        dwSampleRateIndex = nIndex as DWORD;
    }

    if dwSampleRateIndex != (*pMouseInfo).dwSampleRateIndex {
        dwSampleRate = MouseSampleRates[dwSampleRateIndex as usize];
        RegSetValueExW(
            (*pMouseInfo).hDeviceKey,
            [b'S' as u16, b'a' as u16, b'm' as u16, b'p' as u16, b'l' as u16, b'e' as u16, b'R' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0].as_ptr(),
            0,
            (REG_DWORD) as u32,
            core::ptr::addr_of_mut!(dwSampleRate) as LPBYTE,
            core::mem::size_of::<DWORD>() as u32,
        );
    }

    nIndex = SendDlgItemMessageW(
        hwndDlg,
        (IDC_PS2MOUSEWHEEL) as i32,
        (CB_GETCURSEL) as u32,
        0,
        0,
    ) as i32;
    if nIndex == CB_ERR {
        dwWheelDetection = DEFAULT_WHEELDETECTION;
    } else {
        dwWheelDetection = nIndex as DWORD;
    }

    if dwWheelDetection != (*pMouseInfo).dwWheelDetection {
        RegSetValueExW(
            (*pMouseInfo).hDeviceKey,
            [b'E' as u16, b'n' as u16, b'a' as u16, b'b' as u16, b'l' as u16, b'e' as u16, b'W' as u16, b'h' as u16, b'e' as u16, b'e' as u16, b'l' as u16, b'D' as u16, b'e' as u16, b't' as u16, b'e' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0].as_ptr(),
            0,
            (REG_DWORD) as u32,
            core::ptr::addr_of_mut!(dwWheelDetection) as LPBYTE,
            core::mem::size_of::<DWORD>() as u32,
        );
    }

    uValue = SendDlgItemMessageW(
        hwndDlg,
        (IDC_PS2MOUSEINPUTUPDN) as i32,
        (UDM_GETPOS32) as u32,
        0,
        core::ptr::addr_of_mut!(bFailed) as LPARAM,
    ) as u32;
    if bFailed != 0 {
        dwInputBufferLength = DEFAULT_INPUTBUFFERSIZE;
    } else {
        dwInputBufferLength = uValue as DWORD;
    }

    if dwInputBufferLength != (*pMouseInfo).dwInputBufferLength {
        RegSetValueExW(
            (*pMouseInfo).hDeviceKey,
            [b'M' as u16, b'o' as u16, b'u' as u16, b's' as u16, b'e' as u16, b'D' as u16, b'a' as u16, b't' as u16, b'a' as u16, b'Q' as u16, b'u' as u16, b'e' as u16, b'u' as u16, b'e' as u16, b'S' as u16, b'i' as u16, b'z' as u16, b'e' as u16, 0].as_ptr(),
            0,
            (REG_DWORD) as u32,
            core::ptr::addr_of_mut!(dwInputBufferLength) as LPBYTE,
            core::mem::size_of::<DWORD>() as u32,
        );
    }

    uValue = SendDlgItemMessage(
        hwndDlg,
        (IDC_PS2MOUSEFASTINIT) as i32,
        (BM_GETCHECK) as u32,
        0,
        0,
    ) as u32;
    dwInitPolled = if uValue == (BST_CHECKED) as u32 {
        0
    } else {
        1
    };

    if dwInitPolled != (*pMouseInfo).dwInitPolled {
        RegSetValueExW(
            (*pMouseInfo).hDeviceKey,
            [b'M' as u16, b'o' as u16, b'u' as u16, b's' as u16, b'e' as u16, b'I' as u16, b'n' as u16, b'i' as u16, b't' as u16, b'i' as u16, b'a' as u16, b'l' as u16, b'i' as u16, b'z' as u16, b'e' as u16, b'P' as u16, b'o' as u16, b'l' as u16, b'l' as u16, b'e' as u16, b'd' as u16, 0].as_ptr(),
            0,
            (REG_DWORD) as u32,
            core::ptr::addr_of_mut!(dwInitPolled) as LPBYTE,
            core::mem::size_of::<DWORD>() as u32,
        );
    }
}
unsafe extern "C" fn MouseOnNotify(
    hwndDlg: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
) -> i32 {
    match ((*(lParam as LPNMHDR)).code) as i32 {
        UDN_DELTAPOS => {
            if (*(lParam as LPNMHDR)).idFrom == IDC_PS2MOUSEINPUTUPDN as UINT_PTR {
                (*(lParam as LPNMUPDOWN)).iDelta *= 10;
                return (FALSE) as i32;
            }
        }
        PSN_APPLY => {
            MouseOnApply(hwndDlg);
            return (TRUE) as i32;
        }
        _ => {}
    }

    (FALSE) as i32
}
unsafe extern "C" fn MouseOnCtrlColorStatic(
    hwndDlg: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    if lParam as HWND != GetDlgItem(hwndDlg, (IDC_PS2MOUSEINPUTLEN) as i32) {
        return 0;
    }

    SetTextColor(wParam as HDC, GetSysColor((COLOR_WINDOWTEXT) as i32));
    SetBkColor(wParam as HDC, GetSysColor((COLOR_WINDOW) as i32));
    return GetSysColorBrush((COLOR_WINDOW) as i32) as INT_PTR;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn MouseDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    // DPRINT removed

    match uMsg {
        WM_INITDIALOG => {
            return MouseOnDialogInit(hwndDlg, lParam);
        }

        WM_COMMAND => {
            MouseOnCommand(hwndDlg, wParam, lParam);
        }

        WM_NOTIFY => {
            return MouseOnNotify(hwndDlg, wParam, lParam);
        }

        WM_CTLCOLORSTATIC => {
            return MouseOnCtrlColorStatic(hwndDlg, wParam, lParam);
        }

        _ => {}
    }

    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn MouseDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    // DPRINT removed

    match uMsg {
        WM_INITDIALOG => {
            return (MouseOnDialogInit(hwndDlg, lParam)) as i64;
        }

        WM_COMMAND => {
            MouseOnCommand(hwndDlg, wParam, lParam);
        }

        WM_NOTIFY => {
            return (MouseOnNotify(hwndDlg, wParam, lParam)) as i64;
        }

        WM_CTLCOLORSTATIC => {
            return MouseOnCtrlColorStatic(hwndDlg, wParam, lParam);
        }

        _ => {}
    }

    return (FALSE) as i64;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn MouseCallback(
    hWnd: HWND,
    uMsg: u32,
    ppsp: *mut PROPSHEETPAGEW,
) -> u32 {
    let pMouseInfo: *mut MOUSE_INFO;

    pMouseInfo = (*ppsp).lParam as *mut MOUSE_INFO;

    if uMsg == PSPCB_RELEASE {
        if (*pMouseInfo).hDeviceKey != core::ptr::null_mut() {
            RegCloseKey((*pMouseInfo).hDeviceKey);
        }
        HeapFree(GetProcessHeap(), 0, pMouseInfo as *mut _);
    }

    return 1;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn MouseCallback(
    hWnd: HWND,
    uMsg: u32,
    ppsp: *mut PROPSHEETPAGEW,
) -> u32 {
    let pMouseInfo: *mut MOUSE_INFO;

    pMouseInfo = (*ppsp).lParam as *mut MOUSE_INFO;

    if uMsg == (PSPCB_RELEASE) as u32 {
        if (*pMouseInfo).hDeviceKey != core::ptr::null_mut() {
            RegCloseKey((*pMouseInfo).hDeviceKey);
        }
        HeapFree(GetProcessHeap(), 0, pMouseInfo as *mut _);
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn PS2MousePropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    let mut PropSheetPage: PROPSHEETPAGEW = core::mem::zeroed();
    let mut hPropSheetPage: HPROPSHEETPAGE;
    let pMouseInfo: PMOUSE_INFO;

    // DPRINT removed

    if (*lpPropSheetPageRequest).PageRequested != (SPPSR_ENUM_ADV_DEVICE_PROPERTIES) as u32 {
        return (FALSE) as i32;
    }

    pMouseInfo = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<MOUSE_INFO>()) as SIZE_T,
    ) as PMOUSE_INFO;
    if pMouseInfo == core::ptr::null_mut() {
        return (FALSE) as i32;
    }

    (*pMouseInfo).hDeviceKey = SetupDiOpenDevRegKey(
        (*lpPropSheetPageRequest).DeviceInfoSet,
        (*lpPropSheetPageRequest).DeviceInfoData,
        (DICS_FLAG_GLOBAL) as u32,
        0,
        (DIREG_DEV) as u32,
        KEY_ALL_ACCESS,
    );
    if (*pMouseInfo).hDeviceKey == (INVALID_HANDLE_VALUE).cast() {
        // DPRINT removed
        HeapFree(
            GetProcessHeap(),
            0,
            pMouseInfo as LPVOID,
        );
        return (FALSE) as i32;
    }

    PropSheetPage.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    PropSheetPage.dwFlags = 0;
    PropSheetPage.hInstance = hDllInstance;
    PropSheetPage.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PS2MOUSEPROPERTIES);
    PropSheetPage.pfnDlgProc = Some(MouseDlgProc);
    PropSheetPage.lParam = pMouseInfo as LPARAM;
    PropSheetPage.pfnCallback = Some(MouseCallback);

    hPropSheetPage = CreatePropertySheetPageW(&mut PropSheetPage);
    if hPropSheetPage == core::ptr::null_mut() {
        // DPRINT removed
        HeapFree(
            GetProcessHeap(),
            0,
            pMouseInfo as LPVOID,
        );
        return (FALSE) as i32;
    }

    if lpfnAddPropSheetPageProc.map_or(0, |f| f(hPropSheetPage, lParam)) == 0 {
        // DPRINT removed
        DestroyPropertySheetPage(hPropSheetPage);
        HeapFree(
            GetProcessHeap(),
            0,
            pMouseInfo as LPVOID,
        );
        return (FALSE) as i32;
    }

    return (TRUE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn TapePropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    // DPRINT!("TapePropPageProvider(%p %p %lx)\n", lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
    (FALSE) as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS system libraries
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/syssetup/proppage.c
//  * PURPOSE:     Property page providers
//  * PROGRAMMERS: Copyright 2018 Eric Kohl <eric.kohl@reactos.org>
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _MOUSE_INFO
// {
//     HKEY hDeviceKey;
//     DWORD dwSampleRateIndex;
//     DWORD dwWheelDetection;
//     DWORD dwInputBufferLength;
//     DWORD dwInitPolled;
// } MOUSE_INFO, *PMOUSE_INFO;
//
// DWORD MouseSampleRates[] = {20, 40, 60, 80, 100, 200};
//
// #define DEFAULT_SAMPLERATEINDEX   4
// #define DEFAULT_WHEELDETECTION    2
// #define DEFAULT_INPUTBUFFERSIZE 100
// #define DEFAULT_MINBUFFERSIZE   100
// #define DEFAULT_MAXBUFFERSIZE   300
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CdromPropPageProvider(
//      PSP_PROPSHEETPAGE_REQUEST lpPropSheetPageRequest,
//      LPFNADDPROPSHEETPAGE lpfnAddPropSheetPageProc,
//      LPARAM lParam)
// {
//     DPRINT("CdromPropPageProvider(%p %p %lx)\n",
//            lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DiskPropPageProvider(
//      PSP_PROPSHEETPAGE_REQUEST lpPropSheetPageRequest,
//      LPFNADDPROPSHEETPAGE lpfnAddPropSheetPageProc,
//      LPARAM lParam)
// {
//     DPRINT("DiskPropPageProvider(%p %p %lx)\n",
//            lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EisaUpHalPropPageProvider(
//      PSP_PROPSHEETPAGE_REQUEST lpPropSheetPageRequest,
//      LPFNADDPROPSHEETPAGE lpfnAddPropSheetPageProc,
//      LPARAM lParam)
// {
//     DPRINT("EisaUpHalPropPageProvider(%p %p %lx)\n",
//            lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// LegacyDriverPropPageProvider(
//      PSP_PROPSHEETPAGE_REQUEST lpPropSheetPageRequest,
//      LPFNADDPROPSHEETPAGE lpfnAddPropSheetPageProc,
//      LPARAM lParam)
// {
//     DPRINT1("LegacyDriverPropPageProvider(%p %p %lx)\n",
//            lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// static
// DWORD
// MouseGetSampleRateIndex(
//     DWORD dwSampleRate)
// {
//     DWORD i;
//
//     for (i = 0; i < ARRAYSIZE!(MouseSampleRates); i++)
//     {
//         if (MouseSampleRates[i] == dwSampleRate)
//             return i;
//     }
//
//     return DEFAULT_SAMPLERATEINDEX;
// }
//
//
// static
// BOOL
// MouseOnDialogInit(
//     HWND hwndDlg,
//     LPARAM lParam)
// {
//     PMOUSE_INFO pMouseInfo;
//     WCHAR szBuffer[64];
//     UINT i;
//     DWORD dwType, dwSize;
//     DWORD dwSampleRate;
//     LONG lError;
//
//     /* Get the pointer to the mouse info struct */
//     pMouseInfo = (PMOUSE_INFO)((LPPROPSHEETPAGE)lParam)->lParam;
//     if (pMouseInfo == NULL)
//         return FALSE;
//
//     /* Keep the pointer to the mouse info struct */
//     SetWindowLongPtr(hwndDlg, GWLP_USERDATA, (DWORD_PTR)pMouseInfo);
//
//     /* Add the sample rates */
//     for (i = 0; i < ARRAYSIZE!(MouseSampleRates); i++)
//     {
//         wsprintf(szBuffer, L"%lu", MouseSampleRates[i]);
//         SendDlgItemMessageW(hwndDlg,
//                             IDC_PS2MOUSESAMPLERATE,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)szBuffer);
//     }
//
//     /* Read the SampleRate parameter */
//     dwSize = sizeof(DWORD);
//     lError = RegQueryValueExW(pMouseInfo->hDeviceKey,
//                               L"SampleRate",
//                               NULL,
//                               &dwType,
//                               (LPBYTE)&dwSampleRate,
//                               &dwSize);
//     if (lError == ERROR_SUCCESS && dwType == REG_DWORD && dwSize == sizeof(DWORD))
//     {
//         pMouseInfo->dwSampleRateIndex = MouseGetSampleRateIndex(dwSampleRate);
//     }
//     else
//     {
//         /* Set the default sample rate (100 samples per second) */
//         pMouseInfo->dwSampleRateIndex = DEFAULT_SAMPLERATEINDEX;
//     }
//
//     /* Set the sample rate */
//     SendDlgItemMessageW(hwndDlg,
//                         IDC_PS2MOUSESAMPLERATE,
//                         CB_SETCURSEL,
//                         pMouseInfo->dwSampleRateIndex,
//                         0);
//
//     /* Add the detection options */
//     for (i = IDS_DETECTIONDISABLED; i <= IDS_ASSUMEPRESENT; i++)
//     {
//         LoadStringW(hDllInstance, i, szBuffer, ARRAYSIZE!(szBuffer));
//         SendDlgItemMessageW(hwndDlg,
//                             IDC_PS2MOUSEWHEEL,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)szBuffer);
//     }
//
//     /* Read the EnableWheelDetection parameter */
//     dwSize = sizeof(DWORD);
//     lError = RegQueryValueExW(pMouseInfo->hDeviceKey,
//                               L"EnableWheelDetection",
//                               NULL,
//                               &dwType,
//                               (LPBYTE)&pMouseInfo->dwWheelDetection,
//                               &dwSize);
//     if (lError != ERROR_SUCCESS || dwType != REG_DWORD || dwSize != sizeof(DWORD))
//     {
//         /* Set the default wheel detection parameter */
//         pMouseInfo->dwWheelDetection = DEFAULT_WHEELDETECTION;
//     }
//
//     /* Set the wheel detection parameter */
//     SendDlgItemMessageW(hwndDlg,
//                         IDC_PS2MOUSEWHEEL,
//                         CB_SETCURSEL,
//                         pMouseInfo->dwWheelDetection,
//                         0);
//
//     /* Set the input buffer length range: 100-300 */
//     SendDlgItemMessageW(hwndDlg,
//                         IDC_PS2MOUSEINPUTUPDN,
//                         UDM_SETRANGE32,
//                         DEFAULT_MINBUFFERSIZE,
//                         DEFAULT_MAXBUFFERSIZE);
//
//     /* Read the MouseDataQueueSize parameter */
//     dwSize = sizeof(DWORD);
//     lError = RegQueryValueExW(pMouseInfo->hDeviceKey,
//                               L"MouseDataQueueSize",
//                               NULL,
//                               &dwType,
//                               (LPBYTE)&pMouseInfo->dwInputBufferLength,
//                               &dwSize);
//     if (lError != ERROR_SUCCESS || dwType != REG_DWORD || dwSize != sizeof(DWORD))
//     {
//         /* Set the default input buffer length (100 packets) */
//         pMouseInfo->dwInputBufferLength = DEFAULT_INPUTBUFFERSIZE;
//     }
//
//     /* Set the input buffer length */
//     SendDlgItemMessageW(hwndDlg,
//                         IDC_PS2MOUSEINPUTUPDN,
//                         UDM_SETPOS32,
//                         0,
//                         pMouseInfo->dwInputBufferLength);
//
//     /* Read the MouseInitializePolled parameter */
//     dwSize = sizeof(DWORD);
//     lError = RegQueryValueExW(pMouseInfo->hDeviceKey,
//                               L"MouseInitializePolled",
//                               NULL,
//                               &dwType,
//                               (LPBYTE)&pMouseInfo->dwInitPolled,
//                               &dwSize);
//     if (lError != ERROR_SUCCESS || dwType != REG_DWORD || dwSize != sizeof(DWORD))
//     {
//         /* Set the default init polled value (FALSE) */
//         pMouseInfo->dwInitPolled = 0;
//     }
//
//     /* Set the fast initialization option */
//     SendDlgItemMessage(hwndDlg,
//                        IDC_PS2MOUSEFASTINIT,
//                        BM_SETCHECK,
//                        (pMouseInfo->dwInitPolled == 0) ? BST_CHECKED : 0,
//                        0);
//
//     return TRUE;
// }
//
//
// static
// VOID
// MouseOnCommand(
//     HWND hwndDlg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     switch (LOWORD!(wParam))
//     {
//         case IDC_PS2MOUSESAMPLERATE:
//         case IDC_PS2MOUSEWHEEL:
//         case IDC_PS2MOUSEINPUTLEN:
//         case IDC_PS2MOUSEFASTINIT:
//             if (HIWORD!(wParam) == CBN_SELCHANGE ||
//                 HIWORD!(wParam) == CBN_EDITCHANGE)
//             {
//                 /* Enable the Apply button */
//                 PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
//             }
//             break;
//
//         case IDC_PS2MOUSEDEFAULTS:
//             if (HIWORD!(wParam) == BN_CLICKED)
//             {
//                 /* Sample rate: 100 */
//                 SendDlgItemMessageW(hwndDlg,
//                                     IDC_PS2MOUSESAMPLERATE,
//                                     CB_SETCURSEL,
//                                     DEFAULT_SAMPLERATEINDEX,
//                                     0);
//
//                 /* Wheel detection: Assume wheel present */
//                 SendDlgItemMessageW(hwndDlg,
//                                     IDC_PS2MOUSEWHEEL,
//                                     CB_SETCURSEL,
//                                     DEFAULT_WHEELDETECTION,
//                                     0);
//
//                 /* Input buffer length: 100 packets */
//                 SendDlgItemMessageW(hwndDlg,
//                                     IDC_PS2MOUSEINPUTUPDN,
//                                     UDM_SETPOS32,
//                                     0,
//                                     DEFAULT_INPUTBUFFERSIZE);
//
//                 /* Fast Initialization: Checked */
//                 SendDlgItemMessage(hwndDlg,
//                                    IDC_PS2MOUSEFASTINIT,
//                                    BM_SETCHECK,
//                                    (WPARAM)BST_CHECKED,
//                                    (LPARAM)0);
//
//                 /* Enable the Apply button */
//                 PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
//             }
//             break;
//
//         default:
//             break;
//     }
// }
//
//
// static
// VOID
// MouseOnApply(
//     HWND hwndDlg)
// {
//     PMOUSE_INFO pMouseInfo;
//     DWORD dwSampleRateIndex = 0;
//     DWORD dwSampleRate, dwWheelDetection;
//     DWORD dwInputBufferLength;
//     DWORD dwInitPolled;
//     UINT uValue;
//     BOOL bFailed;
//     INT nIndex;
//
//     pMouseInfo = (PMOUSE_INFO)GetWindowLongPtr(hwndDlg, GWLP_USERDATA);
//
//     /* Get the sample rate setting and store it if it was changed */
//     nIndex = SendDlgItemMessageW(hwndDlg, IDC_PS2MOUSESAMPLERATE,
//                                  CB_GETCURSEL, 0, 0);
//     if (nIndex == CB_ERR)
//         dwSampleRateIndex = DEFAULT_SAMPLERATEINDEX;
//     else
//         dwSampleRateIndex = (DWORD)nIndex;
//
//     if (dwSampleRateIndex != pMouseInfo->dwSampleRateIndex)
//     {
//         dwSampleRate = MouseSampleRates[dwSampleRateIndex];
//         RegSetValueExW(pMouseInfo->hDeviceKey,
//                        L"SampleRate",
//                        0,
//                        REG_DWORD,
//                        (LPBYTE)&dwSampleRate,
//                        sizeof(dwSampleRate));
//     }
//
//     /* Get the wheel detection setting and store it if it was changed */
//     nIndex = SendDlgItemMessageW(hwndDlg, IDC_PS2MOUSEWHEEL,
//                                  CB_GETCURSEL, 0, 0);
//     if (nIndex == CB_ERR)
//         dwWheelDetection = DEFAULT_WHEELDETECTION;
//     else
//         dwWheelDetection = (DWORD)nIndex;
//
//     if (dwWheelDetection != pMouseInfo->dwWheelDetection)
//     {
//         RegSetValueExW(pMouseInfo->hDeviceKey,
//                        L"EnableWheelDetection",
//                        0,
//                        REG_DWORD,
//                        (LPBYTE)&dwWheelDetection,
//                        sizeof(dwWheelDetection));
//     }
//
//     /* Get the input buffer length setting and store it if it was changed */
//     uValue = SendDlgItemMessageW(hwndDlg,
//                                  IDC_PS2MOUSEINPUTUPDN,
//                                  UDM_GETPOS32,
//                                  0,
//                                  (LPARAM)&bFailed);
//     if (bFailed)
//         dwInputBufferLength = DEFAULT_INPUTBUFFERSIZE;
//     else
//         dwInputBufferLength = (DWORD)uValue;
//
//     if (dwInputBufferLength != pMouseInfo->dwInputBufferLength)
//     {
//         RegSetValueExW(pMouseInfo->hDeviceKey,
//                        L"MouseDataQueueSize",
//                        0,
//                        REG_DWORD,
//                        (LPBYTE)&dwInputBufferLength,
//                        sizeof(dwInputBufferLength));
//     }
//
//     /* Get the fast initialization setting and store it if it was changed */
//     uValue = SendDlgItemMessage(hwndDlg,
//                                 IDC_PS2MOUSEFASTINIT,
//                                 BM_GETCHECK,
//                                 0,
//                                 0);
//     dwInitPolled = (uValue == BST_CHECKED) ? 0 : 1;
//
//     if (dwInitPolled != pMouseInfo->dwInitPolled)
//     {
//         RegSetValueExW(pMouseInfo->hDeviceKey,
//                        L"MouseInitializePolled",
//                        0,
//                        REG_DWORD,
//                        (LPBYTE)&dwInitPolled,
//                        sizeof(dwInitPolled));
//     }
// }
//
//
// static
// BOOL
// MouseOnNotify(
//     HWND hwndDlg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     switch (((LPNMHDR)lParam)->code)
//     {
//         case UDN_DELTAPOS:
//             if (((LPNMHDR)lParam)->idFrom == IDC_PS2MOUSEINPUTUPDN)
//             {
//                 ((LPNMUPDOWN)lParam)->iDelta *= 10;
//                 return FALSE;
//             }
//             break;
//
//         case PSN_APPLY:
//             MouseOnApply(hwndDlg);
//             return TRUE;
//     }
//
//     return FALSE;
// }
//
//
// static
// INT_PTR
// MouseOnCtrlColorStatic(
//     HWND hwndDlg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     if ((HWND)lParam != GetDlgItem(hwndDlg, IDC_PS2MOUSEINPUTLEN))
//         return 0;
//
//     SetTextColor((HDC)wParam, GetSysColor(COLOR_WINDOWTEXT));
//     SetBkColor((HDC)wParam, GetSysColor(COLOR_WINDOW));
//     return (INT_PTR)GetSysColorBrush(COLOR_WINDOW);
// }
//
//
// static
// INT_PTR
// __stdcall
// MouseDlgProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     DPRINT("MouseDlgProc\n");
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             return MouseOnDialogInit(hwndDlg, lParam);
//
//         case WM_COMMAND:
//             MouseOnCommand(hwndDlg, wParam, lParam);
//             break;
//
//         case WM_NOTIFY:
//             return MouseOnNotify(hwndDlg, wParam, lParam);
//
//         case WM_CTLCOLORSTATIC:
//             return MouseOnCtrlColorStatic(hwndDlg, wParam, lParam);
//     }
//
//     return FALSE;
// }
//
//
// static
// UINT
// __stdcall
// MouseCallback(
//     HWND hWnd,
//     UINT uMsg,
//     LPPROPSHEETPAGE ppsp)
// {
//     PMOUSE_INFO pMouseInfo;
//
//     pMouseInfo = (PMOUSE_INFO)ppsp->lParam;
//
//     if (uMsg == PSPCB_RELEASE)
//     {
//         if (pMouseInfo->hDeviceKey != NULL)
//             RegCloseKey(pMouseInfo->hDeviceKey);
//         HeapFree(GetProcessHeap(), 0, pMouseInfo);
//     }
//
//     return 1;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// PS2MousePropPageProvider(
//      PSP_PROPSHEETPAGE_REQUEST lpPropSheetPageRequest,
//      LPFNADDPROPSHEETPAGE lpfnAddPropSheetPageProc,
//      LPARAM lParam)
// {
//     PROPSHEETPAGEW PropSheetPage;
//     HPROPSHEETPAGE hPropSheetPage;
//     PMOUSE_INFO pMouseInfo;
//
//     DPRINT("PS2MousePropPageProvider(%p %p %lx)\n",
//            lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
//
//     if (lpPropSheetPageRequest->PageRequested != SPPSR_ENUM_ADV_DEVICE_PROPERTIES)
//         return FALSE;
//
//     pMouseInfo = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(MOUSE_INFO));
//     if (pMouseInfo == NULL)
//         return FALSE;
//
//     pMouseInfo->hDeviceKey = SetupDiOpenDevRegKey(lpPropSheetPageRequest->DeviceInfoSet,
//                                                   lpPropSheetPageRequest->DeviceInfoData,
//                                                   DICS_FLAG_GLOBAL,
//                                                   0,
//                                                   DIREG_DEV,
//                                                   KEY_ALL_ACCESS);
//     if (pMouseInfo->hDeviceKey == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("SetupDiOpenDevRegKey() failed (Error %lu)\n", GetLastError());
//         HeapFree(GetProcessHeap(), 0, pMouseInfo);
//         return FALSE;
//     }
//
//     PropSheetPage.dwSize = sizeof(PROPSHEETPAGEW);
//     PropSheetPage.dwFlags = 0;
//     PropSheetPage.hInstance = hDllInstance;
//     PropSheetPage.pszTemplate = MAKEINTRESOURCE!(IDD_PS2MOUSEPROPERTIES);
//     PropSheetPage.pfnDlgProc = MouseDlgProc;
//     PropSheetPage.lParam = (LPARAM)pMouseInfo;
//     PropSheetPage.pfnCallback = MouseCallback;
//
//     hPropSheetPage = CreatePropertySheetPageW(&PropSheetPage);
//     if (hPropSheetPage == NULL)
//     {
//         DPRINT1("CreatePropertySheetPageW() failed!\n");
//         HeapFree(GetProcessHeap(), 0, pMouseInfo);
//         return FALSE;
//     }
//
//     if (!(*lpfnAddPropSheetPageProc)(hPropSheetPage, lParam))
//     {
//         DPRINT1("lpfnAddPropSheetPageProc() failed!\n");
//         DestroyPropertySheetPage(hPropSheetPage);
//         HeapFree(GetProcessHeap(), 0, pMouseInfo);
//         return FALSE;
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
// TapePropPageProvider(
//      PSP_PROPSHEETPAGE_REQUEST lpPropSheetPageRequest,
//      LPFNADDPROPSHEETPAGE lpfnAddPropSheetPageProc,
//      LPARAM lParam)
// {
//     DPRINT("TapePropPageProvider(%p %p %lx)\n",
//            lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
