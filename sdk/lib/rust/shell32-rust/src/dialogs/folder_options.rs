//! ORPHAN-BY-DESIGN: translated Wave-5 TU, parked pending substrate closure.
//! It still needs a few hundred SDK externs/constants/resource IDs resolved before
//! it compiles; wiring it in this state would break the build for every lane rather
//! than expose a regression. This is a PAID artifact — do not delete it to tidy up.
use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
use crate::win32::*;

// ---- file-scope preamble (translated) ----
pub static g_pszShell32: *const u16 = {
    const RAW: [u16; 32] = [
        0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006d,
        0x0052, 0x006f, 0x006f, 0x0074, 0x0025, 0x005c, 0x005c,
        0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006d, 0x0033,
        0x0032, 0x005c, 0x0073, 0x0068, 0x0065, 0x006c, 0x006c,
        0x0033, 0x0032, 0x002e, 0x0064,
        // actually L"%SystemRoot%\\system32\\shell32.dll" – need null terminator
        // Let's compute properly: the string is "%SystemRoot%\\system32\\shell32.dll\0"
        // I'll compute accurately.
        // Wait, I miscounted. Let's list characters:
        // '%' 0x0025, 'S' 0x0053, 'y' 0x0079, 's' 0x0073, 't' 0x0074, 'e' 0x0065, 'm' 0x006d,
        // 'R' 0x0052, 'o' 0x006f, 'o' 0x006f, 't' 0x0074, '%' 0x0025,
        // '\\' 0x005c, '\\' 0x005c,
        // 's' 0x0073, 'y' 0x0079, 's' 0x0073, 't' 0x0074, 'e' 0x0065, 'm' 0x006d,
        // '3' 0x0033, '2' 0x0032,
        // '\\' 0x005c,
        // 's' 0x0073, 'h' 0x0068, 'e' 0x0065, 'l' 0x006c, 'l' 0x006c,
        // '3' 0x0033, '2' 0x0032,
        // '.' 0x002e, 'd' 0x0064, 'l' 0x006c,
        // 'l' 0x006c, '\0' 0x0000.
        // Total 32 characters + null = 33? Actually string length: "%SystemRoot%\\system32\\shell32.dll" is 30? Let's count: %(1)S(2)y(3)s(4)t(5)e(6)m(7)R(8)o(9)o(10)t(11)%(12)\(13)\(14)s(15)y(16)s(17)t(18)e(19)m(20)3(21)2(22)\(23)s(24)h(25)e(26)l(27)l(28)3(29)2(30).(31)d(32)l(33)l(34). Wait.
        // Better to type the bytes manually. I'll do:
    ];
    // I'll just provide a corrected version below.
};

pub static g_pszShell32: *const u16 = {
    const RAW: &[u16] = &[
        0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006d,
        0x0052, 0x006f, 0x006f, 0x0074, 0x0025, 0x005c, 0x005c,
        0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006d, 0x0033,
        0x0032, 0x005c, 0x0073, 0x0068, 0x0065, 0x006c, 0x006c,
        0x0033, 0x0032, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000,
    ];
    RAW.as_ptr()
};

pub static g_pszSpace: *const u16 = {
    const RAW: &[u16] = &[
        0x0020, 0x0009, 0x000a, 0x000d, 0x000c, 0x000b, 0x0000,
    ];
    RAW.as_ptr()
};

pub const GlobalFolderOptionsClassName: *const u16 = {
    const RAW: &[u16] = &[
        0x004d, 0x0053, 0x0047, 0x006c, 0x006f, 0x0062, 0x0061,
        0x006c, 0x0046, 0x006f, 0x006c, 0x0064, 0x0065, 0x0072,
        0x004f, 0x0070, 0x0074, 0x0069, 0x006f, 0x006e, 0x0073,
        0x0053, 0x0074, 0x0075, 0x0062, 0x0000,
    ];
    RAW.as_ptr()
};

pub const PAGE_GENERAL: u32 = 0;
pub const PAGE_VIEW: u32 = 1;
pub const PAGE_FILETYPES: u32 = 2;

pub unsafe extern "system" fn SHCreatePropSheetExtArrayEx(
    hKey: HKEY,
    pszSubKey: LPCWSTR,
    max_iface: UINT,
    pDataObj: *mut IDataObject,
) -> HPSXA;
#[no_mangle]
pub unsafe extern "C" fn Create24BppBitmap(hDC: HDC, cx: i32, cy: i32) -> HBITMAP {
    let mut bi = core::mem::MaybeUninit::<BITMAPINFO>::uninit();
    let bi_ptr = bi.as_mut_ptr();
    ZeroMemory(bi_ptr as *mut core::ffi::c_void, core::mem::size_of::<BITMAPINFO>());
    (*bi_ptr).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    (*bi_ptr).bmiHeader.biWidth = cx;
    (*bi_ptr).bmiHeader.biHeight = cy;
    (*bi_ptr).bmiHeader.biPlanes = 1;
    (*bi_ptr).bmiHeader.biBitCount = 24;
    (*bi_ptr).bmiHeader.biCompression = BI_RGB;
    let mut pvBits: LPVOID = core::ptr::null_mut();
    let hbm = CreateDIBSection(
        hDC,
        bi_ptr as *const BITMAPINFO,
        DIB_RGB_COLORS,
        &mut pvBits as *mut LPVOID,
        core::ptr::null_mut(),
        0,
    );
    hbm
}
#[no_mangle]
pub unsafe extern "C" fn BitmapFromIcon(hIcon: HICON, cx: i32, cy: i32) -> HBITMAP {
    let hDC = CreateCompatibleDC(core::ptr::null_mut());
    if hDC.is_null() {
        return core::ptr::null_mut();
    }
    let hbm = Create24BppBitmap(hDC, cx, cy);
    if hbm.is_null() {
        DeleteDC(hDC);
        return core::ptr::null_mut();
    }
    let hbmOld = SelectObject(hDC, hbm);
    {
        let rc = RECT {
            left: 0,
            top: 0,
            right: cx,
            bottom: cy,
        };
        let brush = HBRUSH(COLOR_3DFACE + 1);
        FillRect(hDC, &rc, brush);
        if !hIcon.is_null() {
            DrawIconEx(
                hDC,
                0,
                0,
                hIcon,
                cx,
                cy,
                0,
                core::ptr::null_mut(),
                DI_NORMAL,
            );
        }
    }
    SelectObject(hDC, hbmOld);
    DeleteDC(hDC);
    hbm
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    match uMsg {
        PSCB_INITIALIZED => {
            let hIcon = LoadIconW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_FOLDER_OPTIONS));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, hIcon as LPARAM);
        }
        _ => {}
    }
    0
}

#[cfg(not(target_arch = "x86"))]
// CALLBACK is __stdcall on i386, which is what extern "system" means; cdecl here
// would corrupt the stack at every PropSheet callback.
unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    match uMsg {
        PSCB_INITIALIZED => {
            let hIcon = LoadIconW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_FOLDER_OPTIONS));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, hIcon as LPARAM);
        }
        _ => {}
    }
    0
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn ShowFolderOptionsDialogThreadProc(param: LPVOID) -> DWORD {
    let _com = CCoInit;
    let mut hppages: [HPROPSHEETPAGE; 3] = [core::ptr::null_mut(); 3];
    let mut num_pages: UINT = 0;

    let hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_GENERAL, FolderOptionsGeneralDlg, 0, core::ptr::null_mut());
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    let hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_VIEW, FolderOptionsViewDlg, 0, core::ptr::null_mut());
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    let hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_FILETYPES, FolderOptionsFileTypesDlg, 0, core::ptr::null_mut());
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    let style: DWORD = WS_DISABLED | WS_CLIPSIBLINGS | WS_CAPTION;
    let exstyle: DWORD = WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW;
    let mut stub = CMSGlobalFolderOptionsStub;
    if stub.Create(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), style, exstyle).is_null() {
        return 0;
    }

    let mut pinfo: PROPSHEETHEADERW = core::mem::zeroed();
    memset(core::ptr::addr_of_mut!(pinfo) as *mut core::ffi::c_void, 0x0, core::mem::size_of::<PROPSHEETHEADERW>() as usize);
    pinfo.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    pinfo.dwFlags = PSH_NOCONTEXTHELP | PSH_USEICONID | PSH_USECALLBACK;
    pinfo.hwndParent = stub;
    pinfo.nPages = num_pages;
    pinfo.phpage = hppages.as_mut_ptr();
    pinfo.hInstance = shell32_hInstance;
    pinfo.pszIcon = MAKEINTRESOURCEW(IDI_SHELL_FOLDER_OPTIONS);
    pinfo.pszCaption = MAKEINTRESOURCEW(IDS_FOLDER_OPTIONS);
    pinfo.pfnCallback = PropSheetProc;
    pinfo.nStartPage = PtrToUlong(param);

    PropertySheetW(core::ptr::addr_of_mut!(pinfo));

    stub.DestroyWindow();
    0
}

#[cfg(not(target_arch = "x86"))]
// LPTHREAD_START_ROUTINE is WINAPI (__stdcall on i386), not cdecl.
unsafe extern "system" fn ShowFolderOptionsDialogThreadProc(param: LPVOID) -> DWORD {
    let _com = CCoInit;
    let mut hppages: [HPROPSHEETPAGE; 3] = [core::ptr::null_mut(); 3];
    let mut num_pages: UINT = 0;

    let hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_GENERAL, FolderOptionsGeneralDlg, 0, core::ptr::null_mut());
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    let hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_VIEW, FolderOptionsViewDlg, 0, core::ptr::null_mut());
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    let hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_FILETYPES, FolderOptionsFileTypesDlg, 0, core::ptr::null_mut());
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    let style: DWORD = WS_DISABLED | WS_CLIPSIBLINGS | WS_CAPTION;
    let exstyle: DWORD = WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW;
    let mut stub = CMSGlobalFolderOptionsStub;
    if stub.Create(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), style, exstyle).is_null() {
        return 0;
    }

    let mut pinfo: PROPSHEETHEADERW = core::mem::zeroed();
    memset(core::ptr::addr_of_mut!(pinfo) as *mut core::ffi::c_void, 0x0, core::mem::size_of::<PROPSHEETHEADERW>() as usize);
    pinfo.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    pinfo.dwFlags = PSH_NOCONTEXTHELP | PSH_USEICONID | PSH_USECALLBACK;
    pinfo.hwndParent = stub;
    pinfo.nPages = num_pages;
    pinfo.phpage = hppages.as_mut_ptr();
    pinfo.hInstance = shell32_hInstance;
    pinfo.pszIcon = MAKEINTRESOURCEW(IDI_SHELL_FOLDER_OPTIONS);
    pinfo.pszCaption = MAKEINTRESOURCEW(IDS_FOLDER_OPTIONS);
    pinfo.pfnCallback = PropSheetProc;
    pinfo.nStartPage = PtrToUlong(param);

    PropertySheetW(core::ptr::addr_of_mut!(pinfo));

    stub.DestroyWindow();
    0
}

#[no_mangle]
pub unsafe extern "system" fn ShowFolderOptionsDialog(page: u32, async_: i32) {
    let _default_async: i32 = FALSE;

    let hwnd = FindWindow(GlobalFolderOptionsClassName, core::ptr::null_mut());
    if !hwnd.is_null() {
        let hpop = GetLastActivePopup(hwnd);
        if hwnd == GetParent(hpop) {
            PostMessage(hpop, PSM_SETCURSEL, page, 0);
        }
        SetForegroundWindow(hpop);
        return;
    }

    let param = UlongToPtr(page);
    if async_ != 0 {
        SHCreateThread(ShowFolderOptionsDialogThreadProc, param, 0, 0);
    } else {
        ShowFolderOptionsDialogThreadProc(param);
    }
}
unsafe extern "C" fn Options_RunDLLCommon(hWnd: HWND, hInst: HINSTANCE, fOptions: i32, nCmdShow: DWORD) {
    match fOptions {
        0 => {
            ShowFolderOptionsDialog(PAGE_GENERAL);
        },
        1 | 3 | 4 | 6 => {
            PostMessage(GetShellWindow(), WM_PROGMAN_OPENSHELLSETTINGS, fOptions, 0);
        },
        7 => {
            ShowFolderOptionsDialog(PAGE_VIEW);
        },
        _ => {
            FIXME!("unrecognized options id {}", fOptions);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Options_RunDLL(hWnd: HWND, hInst: HINSTANCE, cmd: LPCSTR, nCmdShow: DWORD) {
    Options_RunDLLCommon(hWnd, hInst, StrToIntA(cmd), nCmdShow);
}

#[no_mangle]
pub unsafe extern "system" fn Options_RunDLLA(hWnd: HWND, hInst: HINSTANCE, cmd: LPCSTR, nCmdShow: DWORD) {
    Options_RunDLLCommon(hWnd, hInst, StrToIntA(cmd), nCmdShow);
}

#[no_mangle]
pub unsafe extern "system" fn Options_RunDLLW(hWnd: HWND, hInst: HINSTANCE, cmd: LPCWSTR, nCmdShow: DWORD) {
    Options_RunDLLCommon(hWnd, hInst, StrToIntW(cmd), nCmdShow);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *    Folder Options
//  *
//  * Copyright 2007 Johannes Anderwald <johannes.anderwald@reactos.org>
//  * Copyright 2016-2018 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL (fprop);
//
// // Folder Options:
// // CLASSKEY = HKEY_CLASSES_ROOT\CLSID\{6DFD7C5C-2451-11d3-A299-00C04F8EF6AF}
//
// /////////////////////////////////////////////////////////////////////////////
// // strings
//
// // path to shell32
// LPCWSTR g_pszShell32 = L"%SystemRoot%\\system32\\shell32.dll";
//
// // the space characters
// LPCWSTR g_pszSpace = L" \t\n\r\f\v";
//
// /////////////////////////////////////////////////////////////////////////////
// // utility functions
//
// HBITMAP Create24BppBitmap(HDC hDC, INT cx, INT cy)
// {
//     BITMAPINFO bi;
//     LPVOID pvBits;
//
//     ZeroMemory(&bi, sizeof(bi));
//     bi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
//     bi.bmiHeader.biWidth = cx;
//     bi.bmiHeader.biHeight = cy;
//     bi.bmiHeader.biPlanes = 1;
//     bi.bmiHeader.biBitCount = 24;
//     bi.bmiHeader.biCompression = BI_RGB;
//
//     HBITMAP hbm = CreateDIBSection(hDC, &bi, DIB_RGB_COLORS, &pvBits, NULL, 0);
//     return hbm;
// }
//
// HBITMAP BitmapFromIcon(HICON hIcon, INT cx, INT cy)
// {
//     HDC hDC = CreateCompatibleDC(NULL);
//     if (!hDC)
//         return NULL;
//
//     HBITMAP hbm = Create24BppBitmap(hDC, cx, cy);
//     if (!hbm)
//     {
//         DeleteDC(hDC);
//         return NULL;
//     }
//
//     HGDIOBJ hbmOld = SelectObject(hDC, hbm);
//     {
//         RECT rc = { 0, 0, cx, cy };
//         FillRect(hDC, &rc, HBRUSH(COLOR_3DFACE + 1));
//         if (hIcon)
//         {
//             DrawIconEx(hDC, 0, 0, hIcon, cx, cy, 0, NULL, DI_NORMAL);
//         }
//     }
//     SelectObject(hDC, hbmOld);
//     DeleteDC(hDC);
//
//     return hbm;
// }
//
//
// /////////////////////////////////////////////////////////////////////////////
//
// // CMSGlobalFolderOptionsStub --- The owner window of Folder Options.
// // This window hides taskbar button of Folder Options.
//
// #define GlobalFolderOptionsClassName _T("MSGlobalFolderOptionsStub")
//
// class CMSGlobalFolderOptionsStub : public CWindowImpl<CMSGlobalFolderOptionsStub>
// {
// public:
//     DECLARE_WND_CLASS_EX(GlobalFolderOptionsClassName, 0, COLOR_WINDOWTEXT)
//
//     BEGIN_MSG_MAP(CMSGlobalFolderOptionsStub)
//     END_MSG_MAP()
// };
//
// /////////////////////////////////////////////////////////////////////////////
//
// extern "C" HPSXA WINAPI SHCreatePropSheetExtArrayEx(HKEY hKey, LPCWSTR pszSubKey, UINT max_iface, IDataObject *pDataObj);
//
// static int __stdcall
// PropSheetProc(HWND hwndDlg, UINT uMsg, LPARAM lParam)
// {
//     // NOTE: This callback is needed to set large icon correctly.
//     HICON hIcon;
//     switch (uMsg)
//     {
//         case PSCB_INITIALIZED:
//         {
//             hIcon = LoadIconW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_FOLDER_OPTIONS));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// enum {
//     PAGE_GENERAL,
//     PAGE_VIEW,
//     PAGE_FILETYPES
// };
//
// static DWORD __stdcall
// ShowFolderOptionsDialogThreadProc(LPVOID param)
// {
//     CCoInit com; // Required when started from rundll32 (IRegTreeOptions, SHAutoComplete (in PickIconDlg))
//     PROPSHEETHEADERW pinfo;
//     HPROPSHEETPAGE hppages[3];
//     HPROPSHEETPAGE hpage;
//     UINT num_pages = 0;
//
//     hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_GENERAL, FolderOptionsGeneralDlg, 0, NULL);
//     if (hpage)
//         hppages[num_pages++] = hpage;
//
//     hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_VIEW, FolderOptionsViewDlg, 0, NULL);
//     if (hpage)
//         hppages[num_pages++] = hpage;
//
//     hpage = SH_CreatePropertySheetPage(IDD_FOLDER_OPTIONS_FILETYPES, FolderOptionsFileTypesDlg, 0, NULL);
//     if (hpage)
//         hppages[num_pages++] = hpage;
//
//     // the stub window to hide taskbar button
//     DWORD style = WS_DISABLED | WS_CLIPSIBLINGS | WS_CAPTION;
//     DWORD exstyle = WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW;
//     CMSGlobalFolderOptionsStub stub;
//     if (!stub.Create(NULL, NULL, NULL, style, exstyle))
//     {
//         ERR("stub.Create failed\n");
//         return 0;
//     }
//
//     memset(&pinfo, 0x0, sizeof(PROPSHEETHEADERW));
//     pinfo.dwSize = sizeof(PROPSHEETHEADERW);
//     pinfo.dwFlags = PSH_NOCONTEXTHELP | PSH_USEICONID | PSH_USECALLBACK;
//     pinfo.hwndParent = stub;
//     pinfo.nPages = num_pages;
//     pinfo.phpage = hppages;
//     pinfo.hInstance = shell32_hInstance;
//     pinfo.pszIcon = MAKEINTRESOURCEW(IDI_SHELL_FOLDER_OPTIONS);
//     pinfo.pszCaption = MAKEINTRESOURCEW(IDS_FOLDER_OPTIONS);
//     pinfo.pfnCallback = PropSheetProc;
//     pinfo.nStartPage = PtrToUlong(param);
//
//     PropertySheetW(&pinfo);
//
//     stub.DestroyWindow();
//     return 0;
// }
//
// VOID WINAPI 
// ShowFolderOptionsDialog(UINT Page, BOOL Async = FALSE)
// {
//     HWND hWnd = FindWindow(GlobalFolderOptionsClassName, NULL);
//     if (hWnd)
//     {
//         HWND hPop = GetLastActivePopup(hWnd);
//         if (hWnd == GetParent(hPop))
//         {
//             PostMessage(hPop, PSM_SETCURSEL, Page, 0);
//         }
//         SetForegroundWindow(hPop);
//         return;
//     }
//     
//     LPVOID param = UlongToPtr(Page);
//     if (Async)
//         SHCreateThread(ShowFolderOptionsDialogThreadProc, param, 0, 0);
//     else
//         ShowFolderOptionsDialogThreadProc(param); // Rundll32 caller cannot be async!
// }
//
// static VOID
// Options_RunDLLCommon(HWND hWnd, HINSTANCE hInst, int fOptions, DWORD nCmdShow)
// {
//     switch(fOptions)
//     {
//         case 0:
//             ShowFolderOptionsDialog(PAGE_GENERAL);
//             break;
//
//         case 1: // Taskbar settings
//         case 3: // Start menu settings
//         case 4: // Tray icon settings
//         case 6: // Taskbar toolbars
//             PostMessage(GetShellWindow(), WM_PROGMAN_OPENSHELLSETTINGS, fOptions, 0);
//             break;
//
//         case 7: // Windows 8, 10
//             ShowFolderOptionsDialog(PAGE_VIEW);
//             break;
//
//         default:
//             FIXME("unrecognized options id %d\n", fOptions);
//     }
// }
//
// /*************************************************************************
//  *              Options_RunDLL (SHELL32.@)
//  */
// extern "C" VOID WINAPI
// Options_RunDLL(HWND hWnd, HINSTANCE hInst, LPCSTR cmd, DWORD nCmdShow)
// {
//     Options_RunDLLCommon(hWnd, hInst, StrToIntA(cmd), nCmdShow);
// }
//
// /*************************************************************************
//  *              Options_RunDLLA (SHELL32.@)
//  */
// extern "C" VOID WINAPI
// Options_RunDLLA(HWND hWnd, HINSTANCE hInst, LPCSTR cmd, DWORD nCmdShow)
// {
//     Options_RunDLLCommon(hWnd, hInst, StrToIntA(cmd), nCmdShow);
// }
//
// /*************************************************************************
//  *              Options_RunDLLW (SHELL32.@)
//  */
// extern "C" VOID WINAPI
// Options_RunDLLW(HWND hWnd, HINSTANCE hInst, LPCWSTR cmd, DWORD nCmdShow)
// {
//     Options_RunDLLCommon(hWnd, hInst, StrToIntW(cmd), nCmdShow);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
