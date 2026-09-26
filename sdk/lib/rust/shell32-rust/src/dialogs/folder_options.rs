//! `dll/win32/shell32/dialogs/folder_options.cpp`, translated and FLIPPED — the C is commented
//! out of the CMake source list and these definitions are what shell32 links.
//!
//! One piece did NOT come across: `CMSGlobalFolderOptionsStub`, an ATL `CWindowImpl<>` whose
//! per-instance thunk and registered window class are generated from C++ templates. It lives in
//! `dll/win32/shell32/rust_folder_options_stub.cpp` behind a create/destroy bridge, the same
//! shape utils.cpp uses for `CStubWindow32`. SUPERSEDED 2026-09-24: the class is Rust now
//! (crate::atl_stub_windows), A/B-checked against the C++ build by tools/payloads/shelltail.
//!
//! This module REPLACES an earlier parked translation of the same file that was written against
//! the retired hand-written substrate (`crate::types` / `crate::win32`) and never compiled. Both
//! paid runs are intact under `tools/rustconv/.responses/`; this is integration, not deletion.
//!
//! A PAID artifact — do not delete it to tidy up.

use crate::atl_stub_windows::{CMSGlobalFolderOptionsStub_Create_RustBridge, CMSGlobalFolderOptionsStub_Destroy_RustBridge};
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

// folder_options.cpp:33 `LPCWSTR g_pszShell32 = L"%SystemRoot%\\system32\\shell32.dll";`
//
// THE `\\` IS AN ESCAPE, NOT TWO CHARACTERS. Both paid runs transcribed the source TEXT and
// emitted 0x005C twice for the first separator — while emitting it once for the second, so the
// output was not even self-consistent — which is a different path. Both arrays below are
// generated from the string VALUE. Worth knowing generally: a file-scope static still goes
// through the model even when call-site literals are sealed, because the `CPP_COM_WSTR_*`
// machinery covers expressions inside function bodies and not the preamble.
static G_PSZ_SHELL32_BUF: [WCHAR; 34] = [
    0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, 0x0074,
    0x0025, 0x005C, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C,
    0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C,
    0x0000,
];
// the space characters — folder_options.cpp:36 `L" \t\n\r\f\v"`
static G_PSZ_SPACE_BUF: [WCHAR; 7] = [0x0020, 0x0009, 0x000A, 0x000D, 0x000C, 0x000B, 0x0000];

// `folder_options.h` publishes both to the rest of shell32 and `dialogs/filetypes.cpp` (still
// C++) reads them, so the Rust definitions must carry the C names. They are `static mut`
// because that is what the C is — a mutable pointer VARIABLE, which the consumer loads rather
// than taking the buffer's address — and because a `static` of raw-pointer type is not `Sync`.
// Same treatment as iconcache's `ShellSmallIconSize` family.
#[no_mangle]
pub static mut g_pszShell32: LPCWSTR = G_PSZ_SHELL32_BUF.as_ptr();
#[no_mangle]
pub static mut g_pszSpace: LPCWSTR = G_PSZ_SPACE_BUF.as_ptr();

/// `folder_options.cpp:92` `#define GlobalFolderOptionsClassName _T("MSGlobalFolderOptionsStub")`.
/// Must match the name `crate::atl_stub_windows` registers, or the already-open lookup in
/// `ShowFolderOptionsDialog` stops finding the window and Folder Options opens a second copy.
static GLOBAL_FOLDER_OPTIONS_CLASS_NAME: [WCHAR; 26] = [
    0x004D, 0x0053, 0x0047, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x0046, 0x006F, 0x006C,
    0x0064, 0x0065, 0x0072, 0x004F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0053,
    0x0074, 0x0075, 0x0062, 0x0000,
];

// folder_options.cpp:124-128 `enum { PAGE_GENERAL, PAGE_VIEW, PAGE_FILETYPES };`
pub const PAGE_GENERAL: UINT = 0;
pub const PAGE_VIEW: UINT = 1;
#[allow(dead_code)]
pub const PAGE_FILETYPES: UINT = 2;

extern "system" {
    /// The three property-page dialog procs, all `EXTERN_C INT_PTR CALLBACK`
    /// (`dialogs/folder_options.h:22-45`) — C linkage, stdcall convention. General and View are
    /// Rust (`dialogs/general.rs`, `dialogs/view.rs`); FileTypes is still C++ in
    /// `dialogs/filetypes.cpp`. Declared rather than imported so all three arrive with one
    /// signature, typed against the island.
    fn FolderOptionsGeneralDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR;
    fn FolderOptionsViewDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR;
    fn FolderOptionsFileTypesDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR;

    /// Re-declared here, shadowing the island's, so the thread procedure can be spelled
    /// `extern "system"` once rather than `#[cfg]`-duplicated per arch — the island types this
    /// `LPTHREAD_START_ROUTINE`, which is `ntdll-sys`'s and so carries bindgen's literal
    /// convention. Same reasoning as `prelude::DLGPROC` and `dialogs/drive.rs`.
    fn SHCreateThread(
        pfnThreadProc: Option<unsafe extern "system" fn(pData: *mut c_void) -> DWORD>,
        pData: *mut c_void,
        flags: DWORD,
        pfnCallback: Option<unsafe extern "system" fn(pData: *mut c_void) -> DWORD>,
    ) -> BOOL;
}

/////////////////////////////////////////////////////////////////////////////
// utility functions

/// `folder_options.cpp:41-56`. `EXTERN_C` in the header now (plain `__cdecl`, no WINAPI)
/// because its sibling below is called from `dialogs/filetypes.cpp` and Rust cannot emit a
/// C++-mangled name.
#[no_mangle]
pub unsafe extern "C" fn Create24BppBitmap(hDC: HDC, cx: INT, cy: INT) -> HBITMAP {
    let mut bi: BITMAPINFO = core::mem::zeroed();
    let mut pvBits: LPVOID = core::ptr::null_mut();

    ZeroMemory(core::ptr::addr_of_mut!(bi), core::mem::size_of::<BITMAPINFO>());
    bi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    bi.bmiHeader.biWidth = cx as LONG;
    bi.bmiHeader.biHeight = cy as LONG;
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 24;
    bi.bmiHeader.biCompression = BI_RGB;

    CreateDIBSection(
        hDC,
        core::ptr::addr_of!(bi),
        DIB_RGB_COLORS,
        core::ptr::addr_of_mut!(pvBits),
        core::ptr::null_mut(),
        0,
    )
}

/// `folder_options.cpp:58-81`. Called from `dialogs/filetypes.cpp:1857`.
#[no_mangle]
pub unsafe extern "C" fn BitmapFromIcon(hIcon: HICON, cx: INT, cy: INT) -> HBITMAP {
    let hDC: HDC = CreateCompatibleDC(core::ptr::null_mut());
    if hDC.is_null() {
        return core::ptr::null_mut();
    }

    let hbm: HBITMAP = Create24BppBitmap(hDC, cx, cy);
    if hbm.is_null() {
        DeleteDC(hDC);
        return core::ptr::null_mut();
    }

    let hbmOld: HGDIOBJ = SelectObject(hDC, hbm as HGDIOBJ);
    {
        let rc = RECT { left: 0, top: 0, right: cx as LONG, bottom: cy as LONG };
        // C: `HBRUSH(COLOR_3DFACE + 1)` — a C++ functional cast putting a SYSTEM COLOUR INDEX
        // in the brush slot, which is the form FillRect special-cases. Not a real brush handle,
        // so it must not be created or deleted.
        FillRect(hDC, core::ptr::addr_of!(rc), (COLOR_3DFACE + 1) as usize as HBRUSH);
        if !hIcon.is_null() {
            DrawIconEx(hDC, 0, 0, hIcon, cx, cy, 0, core::ptr::null_mut(), DI_NORMAL);
        }
    }
    SelectObject(hDC, hbmOld);
    DeleteDC(hDC);

    hbm
}

/////////////////////////////////////////////////////////////////////////////

/// `folder_options.cpp:107-122`.
///
/// NOTE: This callback is needed to set large icon correctly.
unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, _lParam: LPARAM) -> INT {
    let hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_FOLDER_OPTIONS));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    0
}

/// `folder_options.cpp:130-176`.
unsafe extern "system" fn ShowFolderOptionsDialogThreadProc(param: *mut c_void) -> DWORD {
    // Required when started from rundll32 (IRegTreeOptions, SHAutoComplete (in PickIconDlg)).
    // A REAL RAII value: both paid runs wrote `let _com = CCoInit;`, which names the type and
    // constructs nothing, so neither CoInitialize nor CoUninitialize would have run — the same
    // silent defect `COleInit` had in brfolder.
    let _com = CCoInit::new();

    let mut pinfo: PROPSHEETHEADERW = core::mem::zeroed();
    let mut hppages: [HPROPSHEETPAGE; 3] = [core::ptr::null_mut(); 3];
    let mut hpage: HPROPSHEETPAGE;
    let mut num_pages: UINT = 0;

    hpage = SH_CreatePropertySheetPage(
        IDD_FOLDER_OPTIONS_GENERAL as WORD,
        Some(FolderOptionsGeneralDlg),
        0,
        core::ptr::null(),
    );
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    hpage = SH_CreatePropertySheetPage(
        IDD_FOLDER_OPTIONS_VIEW as WORD,
        Some(FolderOptionsViewDlg),
        0,
        core::ptr::null(),
    );
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    hpage = SH_CreatePropertySheetPage(
        IDD_FOLDER_OPTIONS_FILETYPES as WORD,
        Some(FolderOptionsFileTypesDlg),
        0,
        core::ptr::null(),
    );
    if !hpage.is_null() {
        hppages[num_pages as usize] = hpage;
        num_pages += 1;
    }

    // the stub window to hide taskbar button
    let style: DWORD = WS_DISABLED | WS_CLIPSIBLINGS | WS_CAPTION;
    let exstyle: DWORD = WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW;
    let mut pStub: *mut c_void = core::ptr::null_mut();
    let hStub: HWND = CMSGlobalFolderOptionsStub_Create_RustBridge(
        style,
        exstyle,
        core::ptr::addr_of_mut!(pStub),
    );
    if hStub.is_null() {
        // ERR("stub.Create failed\n");
        return 0;
    }

    // C memsets `pinfo` here; the `zeroed()` above already did it.
    pinfo.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    pinfo.dwFlags = PSH_NOCONTEXTHELP | PSH_USEICONID | PSH_USECALLBACK;
    pinfo.hwndParent = hStub;
    pinfo.nPages = num_pages;
    // PROPSHEETHEADERW carries three anonymous unions, and bindgen renders them two DIFFERENT
    // ways: `__bindgen_anon_3` (ppsp/phpage) is a real `union`, so the field is written
    // directly, while `_1` (hIcon/pszIcon) and `_2` (nStartPage/pStartPage) are
    // `__BindgenUnionField` wrappers that have to go through `.as_mut()`. Picking the wrong
    // anon index is the hazard here — `_2` is the START PAGE, not the page array.
    pinfo.__bindgen_anon_3.phpage = hppages.as_mut_ptr();
    pinfo.hInstance = shell32_hInstance;
    *pinfo.__bindgen_anon_1.pszIcon.as_mut() =
        MAKEINTRESOURCEW(IDI_SHELL_FOLDER_OPTIONS) as LPCWSTR;
    pinfo.pszCaption = MAKEINTRESOURCEW(IDS_FOLDER_OPTIONS);
    pinfo.pfnCallback = Some(PropSheetProc);
    // C: `PtrToUlong(param)` — the page index travelled here in a pointer slot.
    *pinfo.__bindgen_anon_2.nStartPage.as_mut() = param as usize as UINT;

    PropertySheetW(core::ptr::addr_of_mut!(pinfo));

    CMSGlobalFolderOptionsStub_Destroy_RustBridge(pStub);
    0
}

/// `folder_options.cpp:178-200`
/// `VOID WINAPI ShowFolderOptionsDialog(UINT Page, BOOL Async = FALSE)`.
///
/// The C++ DEFAULT ARGUMENT is written out at the two call sites below. Both paid runs made
/// `Async` a required parameter and then called this with ONE argument, which does not compile;
/// "fixing" that by passing TRUE would have sent both rundll32 entry points down the async
/// path, which the comment in the C says is exactly wrong.
///
/// IT DOES HAVE AN OUTSIDE CALLER, and finding it was a link failure: `CDesktopBrowser.cpp:527`
/// declares it BARE AT FILE SCOPE — not in any header — and calls it from `OnShowOptionsDlg`.
/// A header-only search does not see that, and neither does a `grep | head`. The C++-mangled,
/// stdcall-decorated name it wanted (`__Z23ShowFolderOptionsDialogji@8`) is not something Rust
/// can emit, so that declaration is `EXTERN_C` now and this is a plain `extern "system"` export.
/// Same shape as `Shell_GetCachedImageIndexW` in the iconcache flip.
#[no_mangle]
pub unsafe extern "system" fn ShowFolderOptionsDialog(Page: UINT, Async: BOOL) {
    let hWnd: HWND = FindWindowW(GLOBAL_FOLDER_OPTIONS_CLASS_NAME.as_ptr(), core::ptr::null());
    if !hWnd.is_null() {
        let hPop: HWND = GetLastActivePopup(hWnd);
        if hWnd == GetParent(hPop) {
            PostMessage(hPop, PSM_SETCURSEL, Page as WPARAM, 0);
        }
        SetForegroundWindow(hPop);
        return;
    }

    let param: *mut c_void = Page as usize as *mut c_void;
    if Async != 0 {
        SHCreateThread(Some(ShowFolderOptionsDialogThreadProc), param, 0, None);
    } else {
        // Rundll32 caller cannot be async!
        ShowFolderOptionsDialogThreadProc(param);
    }
}

/// `folder_options.cpp:202-227`.
unsafe fn Options_RunDLLCommon(_hWnd: HWND, _hInst: HINSTANCE, fOptions: INT, _nCmdShow: DWORD) {
    match fOptions {
        0 => {
            ShowFolderOptionsDialog(PAGE_GENERAL, FALSE);
        }
        // 1 = Taskbar settings. 3/4/6 (Start menu / tray icon / taskbar toolbars) sit behind
        // `#if (NTDDI_VERSION >= NTDDI_VISTA)` in the C, and shell32 builds
        // -DNTDDI_VERSION=0x06000000 (CMakeLists.txt), so all four arms are live.
        1 | 3 | 4 | 6 => {
            PostMessage(
                GetShellWindow(),
                WM_PROGMAN_OPENSHELLSETTINGS,
                fOptions as WPARAM,
                0,
            );
        }
        // Windows 8, 10
        7 => {
            ShowFolderOptionsDialog(PAGE_VIEW, FALSE);
        }
        _ => {
            // FIXME("unrecognized options id %d\n", fOptions);
        }
    }
}

/*************************************************************************
 *              Options_RunDLL (SHELL32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn Options_RunDLL(
    hWnd: HWND,
    hInst: HINSTANCE,
    cmd: LPCSTR,
    nCmdShow: DWORD,
) {
    Options_RunDLLCommon(hWnd, hInst, StrToIntA(cmd), nCmdShow);
}

/*************************************************************************
 *              Options_RunDLLA (SHELL32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn Options_RunDLLA(
    hWnd: HWND,
    hInst: HINSTANCE,
    cmd: LPCSTR,
    nCmdShow: DWORD,
) {
    Options_RunDLLCommon(hWnd, hInst, StrToIntA(cmd), nCmdShow);
}

/*************************************************************************
 *              Options_RunDLLW (SHELL32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn Options_RunDLLW(
    hWnd: HWND,
    hInst: HINSTANCE,
    cmd: LPCWSTR,
    nCmdShow: DWORD,
) {
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
