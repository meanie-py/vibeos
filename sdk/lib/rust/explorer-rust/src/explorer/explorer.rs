use core::ffi::c_void;
use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use explorer_sys::types::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

extern "system" {
    fn SHGetShellKey(flags: u32, path: LPCWSTR, create: i32) -> HKEY;
    fn SHGetValueW( hkey: HKEY, subkey: LPCWSTR, value: LPCWSTR, type_: *mut u32, data: *mut core::ffi::c_void, size: *mut u32, ) -> u32;
    fn SHSetValueW( hkey: HKEY, subkey: LPCWSTR, value: LPCWSTR, type_: u32, data: *const core::ffi::c_void, size: u32, ) -> u32;
    fn IsOS(os: u32) -> i32;
    fn IsUserAnAdmin() -> i32;
    fn IntToPtr(val: i32) -> *mut core::ffi::c_void;
    fn SystemParametersInfo( action: u32, param: u32, data: *mut core::ffi::c_void, flags: u32, ) -> i32;
    fn PathFindFileNameW(pszPath: LPCWSTR) -> LPWSTR;
    fn StrStrIW(pszFirst: LPCWSTR, pszSrch: LPCWSTR) -> LPWSTR;
    fn CreateTrayWindow(ppTray: *mut *mut core::ffi::c_void);
    fn DesktopCreateWindow(pTray: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn DesktopDestroyShellWindow(hShellDesktop: *mut core::ffi::c_void);
    fn DoFinishStartupItems();
    fn DoStartStartupItems(pTray: *mut core::ffi::c_void) -> i32;
    fn FileIconInit(bFullInit: i32) -> i32;
    fn InitCommonControls();
    fn MessageBox(hWnd: *mut core::ffi::c_void, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: u32) -> i32;
    fn OleInitialize(pvReserved: *mut core::ffi::c_void) -> i32;
    fn OleUninitialize();
    fn ProcessRunOnceItems();
    fn ProcessStartupItems();
    fn ReleaseStartupMutex();
    fn TrayMessageLoop(pTray: *mut core::ffi::c_void);
    fn _ShellDDEInit(bInit: i32);
    fn _WinList_Init();
    fn _countof(arr: *const core::ffi::c_void) -> usize;
}

// ---- file-scope preamble (translated) ----
pub static mut hExplorerInstance: HINSTANCE = core::ptr::null_mut();
pub static mut hProcessHeap: HANDLE = core::ptr::null_mut();
pub static mut hkExplorer: HKEY = core::ptr::null_mut();
pub static mut bExplorerIsShell: BOOL = 0;

unsafe fn InitializeAtlModule(hInstance: HINSTANCE, bInitialize: BOOL) {
    if bInitialize != 0 {
        gModule.Init(ObjectMap, hInstance, core::ptr::null_mut());
    } else {
        gModule.Term();
    }
}

unsafe fn InitializeServerAdminUI() {

    const SHKEY_Root_HKCU: u32 = 0x80000001;
    const SHKEY_Key_Explorer: u32 = 0x00000000;
    const OS_ANYSERVER: u32 = 29;

    let hKey = SHGetShellKey(
        SHKEY_Root_HKCU | SHKEY_Key_Explorer,
        b"A\0d\0v\0a\0n\0c\0e\0d\0\0\0".as_ptr() as LPCWSTR,
        1,
    );
    if hKey.is_null() {
        return;
    }

    let mut value: u32 = 0;
    let mut size: u32 = core::mem::size_of::<u32>() as u32;
    let mut type_: u32 = 0;
    let error = SHGetValueW(
        hKey,
        core::ptr::null(),
        b"S\0e\0r\0v\0e\0r\0A\0d\0m\0i\0n\0U\0I\0\0\0".as_ptr() as LPCWSTR,
        &mut type_,
        &mut value as *mut u32 as *mut core::ffi::c_void,
        &mut size,
    );
    if error != 0 || type_ != REG_DWORD || size != core::mem::size_of::<u32>() as u32 {
        value = (IsOS(OS_ANYSERVER) != 0 && IsUserAnAdmin() != 0) as u32;
        SHSetValueW(
            hKey,
            core::ptr::null(),
            b"S\0e\0r\0v\0e\0r\0A\0d\0m\0i\0n\0U\0I\0\0\0".as_ptr() as LPCWSTR,
            REG_DWORD,
            &value as *const u32 as *const core::ffi::c_void,
            core::mem::size_of::<u32>() as u32,
        );
        if value != 0 {
            SystemParametersInfo(
                SPI_SETKEYBOARDCUES,
                0,
                IntToPtr(1),
                SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
            );
        }
    }
    RegCloseKey(hKey);
}

unsafe fn SetShellReadyEvent(lpEventName: LPCWSTR) -> u8 {
    let hEvent = OpenEventW(EVENT_MODIFY_STATE, 0u8, lpEventName);
    if !hEvent.is_null() {
        SetEvent(hEvent);
        CloseHandle(hEvent);
        return 1u8;
    }
    0u8
}

unsafe fn HideMinimizedWindows(bHide: i32) {
    let mut mm: MINIMIZEDMETRICS = core::mem::zeroed();
    mm.cbSize = core::mem::size_of::<MINIMIZEDMETRICS>() as u32;
    if SystemParametersInfoW(SPI_GETMINIMIZEDMETRICS, core::mem::size_of::<MINIMIZEDMETRICS>() as u32, &mut mm as *mut _ as *mut core::ffi::c_void, 0) == 0 {
        ERR!(b"SystemParametersInfoW failed with %lu\n\0".as_ptr() as *const i8, GetLastError());
        return;
    }
    if bHide != 0 {
        mm.iArrange |= ARW_HIDE as i32;
    } else {
        mm.iArrange &= !(ARW_HIDE as i32);
    }
    if SystemParametersInfoW(SPI_SETMINIMIZEDMETRICS, core::mem::size_of::<MINIMIZEDMETRICS>() as u32, &mut mm as *mut _ as *mut core::ffi::c_void, 0) == 0 {
        ERR!(b"SystemParametersInfoW failed with %lu\n\0".as_ptr() as *const i8, GetLastError());
    }
}

unsafe fn IsExplorerSystemShell() -> u8 {
    let mut bIsSystemShell: u8 = 1;
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if GetModuleFileNameW(core::ptr::null_mut(), szPath.as_mut_ptr(), MAX_PATH) == 0 {
        return 0;
    }

    let szExplorer = PathFindFileNameW(szPath.as_ptr());

    let mut hKeyWinlogon: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        0x80000002u32 as HKEY,
        ['S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16, 'N' as u16, 'T' as u16, '\\' as u16, 'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'l' as u16, 'o' as u16, 'g' as u16, 'o' as u16, 'n' as u16, 0].as_ptr(),
        0,
        KEY_READ,
        &mut hKeyWinlogon,
    ) == ERROR_SUCCESS as i32
    {
        let mut dwType: u32 = 0;
        let mut szShell: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut cbShell: u32 = core::mem::size_of_val(&szShell) as u32;

        let Status = RegQueryValueExW(
            hKeyWinlogon,
            ['S' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 0].as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            szShell.as_mut_ptr() as LPBYTE,
            &mut cbShell,
        );
        if Status == ERROR_SUCCESS as i32 {
            if (dwType != REG_SZ && dwType != REG_EXPAND_SZ) || StrStrIW(szShell.as_ptr(), szExplorer).is_null() {
                bIsSystemShell = 0;
            }
        }

        RegCloseKey(hKeyWinlogon);
    }

    bIsSystemShell
}

unsafe fn StartWithCommandLine(hInstance: HINSTANCE) -> i32 {
    let mut b: BOOL = 0;
    let mut parseResults: EXPLORER_CMDLINE_PARSE_RESULTS = core::mem::zeroed();

    if SHExplorerParseCmdLine(&mut parseResults) != 0 {
        b = SHCreateFromDesktop(&mut parseResults);
    }

    if !parseResults.strPath.is_null() {
        SHFree(parseResults.strPath as *mut core::ffi::c_void);
    }

    if !parseResults.pidlPath.is_null() {
        ILFree(parseResults.pidlPath as *mut core::ffi::c_void);
    }

    if !parseResults.pidlRoot.is_null() {
        ILFree(parseResults.pidlRoot as *mut core::ffi::c_void);
    }

    b
}

unsafe fn StartWithDesktop(hInstance: HINSTANCE) -> i32 {

    const IDS_STARTUP_ERROR: u32 = 0;

    InitializeAtlModule(hInstance, 1u8);

    if RegOpenKeyW(
        0x80000001u32 as HKEY,
        b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0E\0x\0p\0l\0o\0r\0e\0r\0\0".as_ptr() as LPCWSTR,
        core::ptr::addr_of_mut!(hkExplorer),
    ) != ERROR_SUCCESS
    {
        let mut Message: [u16; 256] = [0; 256];
        LoadStringW(hInstance, IDS_STARTUP_ERROR, Message.as_mut_ptr(), 256);
        MessageBox(core::ptr::null_mut(), Message.as_ptr(), core::ptr::null(), MB_ICONERROR);
        return 1;
    }

    hExplorerInstance = hInstance;
    hProcessHeap = GetProcessHeap();

    g_TaskbarSettings.Load();

    InitCommonControls();
    OleInitialize(core::ptr::null_mut());

    _ShellDDEInit(1);

    FileIconInit(1);

    _WinList_Init();

    InitializeServerAdminUI();

    OutputDebugStringA(b"[EXPLORER] taskbar: creating tray window\n\0".as_ptr() as *const i8);
    let mut Tray: *mut core::ffi::c_void = core::ptr::null_mut();
    CreateTrayWindow(core::ptr::addr_of_mut!(Tray));

    HideMinimizedWindows(1);

    ProcessRunOnceItems();

    let mut hShellDesktop: *mut core::ffi::c_void = core::ptr::null_mut();
    if !Tray.is_null() {
        hShellDesktop = DesktopCreateWindow(Tray);
    }

    if SetShellReadyEvent(b"m\0s\0g\0i\0n\0a\0:\0 \0S\0h\0e\0l\0l\0R\0e\0a\0d\0y\0E\0v\0e\0n\0t\0\0".as_ptr() as LPCWSTR) == 0 {
        SetShellReadyEvent(b"G\0l\0o\0b\0a\0l\0\\\0m\0s\0g\0i\0n\0a\0:\0 \0S\0h\0e\0l\0l\0R\0e\0a\0d\0y\0E\0v\0e\0n\0t\0\0".as_ptr() as LPCWSTR);
    }

    if DoStartStartupItems(Tray) != 0 {
        ProcessStartupItems();
        DoFinishStartupItems();
    }
    ReleaseStartupMutex();

    if !Tray.is_null() {
        OutputDebugStringA(b"[EXPLORER] taskbar: entering message loop\n\0".as_ptr() as *const i8);
        TrayMessageLoop(Tray);
        HideMinimizedWindows(0);
    }

    if !hShellDesktop.is_null() {
        DesktopDestroyShellWindow(hShellDesktop);
    }

    OleUninitialize();

    RegCloseKey(hkExplorer);
    hkExplorer = core::ptr::null_mut();

    InitializeAtlModule(hInstance, 0u8);

    0
}

#[no_mangle]
pub unsafe extern "system" fn _tWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPTSTR,
    nCmdShow: i32,
) -> i32 {
    OutputDebugStringA(b"[EXPLORER] _tWinMain entered\n\0".as_ptr() as *const i8);

    SetProcessShutdownParameters(2, 0);

    InitRSHELL();

    TRACE!(b"Explorer starting... Command line: %S\n\0".as_ptr() as *const i8, lpCmdLine);

    bExplorerIsShell = (GetShellWindow() == core::ptr::null_mut()) && (IsExplorerSystemShell() != 0);

    if bExplorerIsShell == 0 {
        return StartWithCommandLine(hInstance);
    }

    return StartWithDesktop(hInstance);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Explorer
//  *
//  * Copyright 2006 - 2007 Thomas Weidenmueller <w3seek@reactos.org>
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
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
//
// #include "precomp.h"
// #include <browseui_undoc.h>
//
// HINSTANCE hExplorerInstance;
// HANDLE hProcessHeap;
// HKEY hkExplorer = NULL;
// BOOL bExplorerIsShell = FALSE;
//
// class CExplorerModule : public CComModule
// {
// public:
// };
//
// BEGIN_OBJECT_MAP(ObjectMap)
// END_OBJECT_MAP()
//
// CExplorerModule gModule;
// CAtlWinModule   gWinModule;
//
// static VOID InitializeAtlModule(HINSTANCE hInstance, BOOL bInitialize)
// {
//     if (bInitialize)
//     {
//         gModule.Init(ObjectMap, hInstance, NULL);
//     }
//     else
//     {
//         gModule.Term();
//     }
// }
//
// static void
// InitializeServerAdminUI()
// {
//     HKEY hKey = SHGetShellKey(SHKEY_Root_HKCU | SHKEY_Key_Explorer, L"Advanced", TRUE);
//     if (!hKey)
//         return;
//
//     DWORD value, size = sizeof(value), type;
//     DWORD error = SHGetValueW(hKey, NULL, L"ServerAdminUI", &type, &value, &size);
//     if (error || type != REG_DWORD || size != sizeof(value))
//     {
//         // The value doesn't exist or is invalid, calculate and apply a default value
//         value = IsOS(OS_ANYSERVER) && IsUserAnAdmin();
//         SHSetValueW(hKey, NULL, L"ServerAdminUI", REG_DWORD, &value, sizeof(value));
//         if (value)
//         {
//             // TODO: Apply registry tweaks with RegInstallW; RegServerAdmin in the REGINST resource in shell32.
//             SystemParametersInfo(SPI_SETKEYBOARDCUES, 0, IntToPtr(TRUE), SPIF_SENDCHANGE | SPIF_UPDATEINIFILE);
//         }
//     }
//     RegCloseKey(hKey);
// }
//
// static BOOL
// SetShellReadyEvent(IN LPCWSTR lpEventName)
// {
//     HANDLE hEvent;
//
//     hEvent = OpenEventW(EVENT_MODIFY_STATE, FALSE, lpEventName);
//     if (hEvent != NULL)
//     {
//         SetEvent(hEvent);
//
//         CloseHandle(hEvent);
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static VOID
// HideMinimizedWindows(IN BOOL bHide)
// {
//     MINIMIZEDMETRICS mm;
//
//     mm.cbSize = sizeof(mm);
//     if (!SystemParametersInfoW(SPI_GETMINIMIZEDMETRICS, sizeof(mm), &mm, 0))
//     {
//         ERR("SystemParametersInfoW failed with %lu\n", GetLastError());
//         return;
//     }
//     if (bHide)
//         mm.iArrange |= ARW_HIDE;
//     else
//         mm.iArrange &= ~ARW_HIDE;
//     if (!SystemParametersInfoW(SPI_SETMINIMIZEDMETRICS, sizeof(mm), &mm, 0))
//         ERR("SystemParametersInfoW failed with %lu\n", GetLastError());
// }
//
// static BOOL
// IsExplorerSystemShell()
// {
//     BOOL bIsSystemShell = TRUE; // Assume we are the system shell by default.
//     WCHAR szPath[MAX_PATH];
//
//     if (!GetModuleFileNameW(NULL, szPath, _countof(szPath)))
//         return FALSE;
//
//     LPWSTR szExplorer = PathFindFileNameW(szPath);
//
//     HKEY hKeyWinlogon;
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
//                       0, KEY_READ, &hKeyWinlogon) == ERROR_SUCCESS)
//     {
//         LSTATUS Status;
//         DWORD dwType;
//         WCHAR szShell[MAX_PATH];
//         DWORD cbShell = sizeof(szShell);
//
//         // TODO: Add support for paths longer than MAX_PATH
//         Status = RegQueryValueExW(hKeyWinlogon, L"Shell", 0, &dwType, (LPBYTE)szShell, &cbShell);
//         if (Status == ERROR_SUCCESS)
//         {
//             if ((dwType != REG_SZ && dwType != REG_EXPAND_SZ) || !StrStrIW(szShell, szExplorer))
//                 bIsSystemShell = FALSE;
//         }
//
//         RegCloseKey(hKeyWinlogon);
//     }
//
//     return bIsSystemShell;
// }
//
// static INT
// StartWithCommandLine(IN HINSTANCE hInstance)
// {
//     BOOL b = FALSE;
//     EXPLORER_CMDLINE_PARSE_RESULTS parseResults = { 0 };
//
//     if (SHExplorerParseCmdLine(&parseResults))
//         b = SHCreateFromDesktop(&parseResults);
//
//     if (parseResults.strPath)
//         SHFree(parseResults.strPath);
//
//     if (parseResults.pidlPath)
//         ILFree(parseResults.pidlPath);
//
//     if (parseResults.pidlRoot)
//         ILFree(parseResults.pidlRoot);
//
//     return b;
// }
//
// static INT
// StartWithDesktop(IN HINSTANCE hInstance)
// {
//     InitializeAtlModule(hInstance, TRUE);
//
//     if (RegOpenKeyW(HKEY_CURRENT_USER,
//         L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer",
//         &hkExplorer) != ERROR_SUCCESS)
//     {
//         WCHAR Message[256];
//         LoadStringW(hInstance, IDS_STARTUP_ERROR, Message, _countof(Message));
//         MessageBox(NULL, Message, NULL, MB_ICONERROR);
//         return 1;
//     }
//
//     hExplorerInstance = hInstance;
//     hProcessHeap = GetProcessHeap();
//
//     g_TaskbarSettings.Load();
//
//     InitCommonControls();
//     OleInitialize(NULL);
//
//     /* Initialize shell dde support */
//     _ShellDDEInit(TRUE);
//
//     /* Initialize shell icons */
//     FileIconInit(TRUE);
//
//     /* Initialize CLSID_ShellWindows class */
//     _WinList_Init();
//
//     InitializeServerAdminUI();
//
//     /* Boot/test marker — taskbar (tray window) creation milestone. */
//     OutputDebugStringA("[EXPLORER] taskbar: creating tray window\n");
//     CComPtr<ITrayWindow> Tray;
//     CreateTrayWindow(&Tray);
//
//     /* This not only hides the minimized window captions in the bottom
//     left screen corner, but is also needed in order to receive
//     HSHELL_* notification messages (which are required for taskbar
//     buttons to work right) */
//     HideMinimizedWindows(TRUE);
//
//     ProcessRunOnceItems(); // Must be executed before the desktop is created
//
//     HANDLE hShellDesktop = NULL;
//     if (Tray != NULL)
//         hShellDesktop = DesktopCreateWindow(Tray);
//
//     /* WinXP: Notify msgina to hide the welcome screen */
//     if (!SetShellReadyEvent(L"msgina: ShellReadyEvent"))
//         SetShellReadyEvent(L"Global\\msgina: ShellReadyEvent");
//
//     if (DoStartStartupItems(Tray))
//     {
//         ProcessStartupItems();
//         DoFinishStartupItems();
//     }
//     ReleaseStartupMutex(); // For ProcessRunOnceItems
//
//     if (Tray != NULL)
//     {
//         /* Boot/test marker — taskbar up, entering its message loop. */
//         OutputDebugStringA("[EXPLORER] taskbar: entering message loop\n");
//         TrayMessageLoop(Tray);
//         HideMinimizedWindows(FALSE);
//     }
//
//     if (hShellDesktop != NULL)
//         DesktopDestroyShellWindow(hShellDesktop);
//
//     OleUninitialize();
//
//     RegCloseKey(hkExplorer);
//     hkExplorer = NULL;
//
//     InitializeAtlModule(hInstance, FALSE);
//
//     return 0;
// }
//
// INT WINAPI
// _tWinMain(IN HINSTANCE hInstance,
//           IN HINSTANCE hPrevInstance,
//           IN LPTSTR lpCmdLine,
//           IN INT nCmdShow)
// {
//     /* Boot marker — proves EXPLORER user-mode code is running.  Smoke
//      * harness (tests/qemu/vm.py) keys on this string. */
//     OutputDebugStringA("[EXPLORER] _tWinMain entered\n");
//
//     /*
//     * Set our shutdown parameters: we want to shutdown the very last,
//     * but before any TaskMgr instance (which has a shutdown level of 1).
//     */
//     SetProcessShutdownParameters(2, 0);
//
//     InitRSHELL();
//
//     TRACE("Explorer starting... Command line: %S\n", lpCmdLine);
//
//     bExplorerIsShell = (GetShellWindow() == NULL) && IsExplorerSystemShell();
//
//     if (!bExplorerIsShell)
//     {
//         return StartWithCommandLine(hInstance);
//     }
//
//     return StartWithDesktop(hInstance);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
