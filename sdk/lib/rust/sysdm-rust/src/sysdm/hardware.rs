// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PDEVMGREXEC = Option<unsafe extern "system" fn(hWndParent: HWND, hInst: HMODULE, Unknown: PVOID, nCmdShow: i32) -> i32>;
unsafe extern "C" fn LaunchDeviceManager(hWndParent: HWND) -> BOOL {
    #[cfg(__REACTOS__)]
    {
        return ((ShellExecuteW(core::ptr::null_mut(), SW_SHOWNORMAL.as_ptr(), 32.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), FALSE) as INT_PTR) > FALSE) as BOOL;
    }
    #[cfg(not(__REACTOS__))]
    {
        let hDll: HMODULE;
        let DevMgrExec: PDEVMGREXEC;
        let Ret: BOOL;

        hDll = LoadLibrary(_TEXT(SW_SHOW.as_ptr()));
        if hDll.is_null() {
            return FALSE;
        }

        DevMgrExec = GetProcAddress(hDll, b"DeviceManager_ExecuteW\0".as_ptr() as *const i8) as PDEVMGREXEC;
        if DevMgrExec.is_none() {
            FreeLibrary(hDll);
            return FALSE;
        }

        Ret = DevMgrExec(hWndParent, hApplet, core::ptr::null_mut(), SW_SHOWNORMAL);
        FreeLibrary(hDll);
        return Ret;
    }
}
unsafe extern "C" fn LaunchHardwareWizard(hWndParent: HWND) {
    let mut shInputDll: SHELLEXECUTEINFO = core::mem::zeroed();

    memset(
        core::ptr::addr_of_mut!(shInputDll) as *mut core::ffi::c_void,
        0x0,
        core::mem::size_of::<SHELLEXECUTEINFO>(),
    );

    shInputDll.cbSize = core::mem::size_of::<SHELLEXECUTEINFO>() as u32;
    shInputDll.hwnd = hWndParent;
    shInputDll.lpVerb = _T((b"open\0".as_ptr() as *const i8).cast());
    shInputDll.lpFile = _T((b"rundll32.exe\0".as_ptr() as *const i8).cast());
    shInputDll.lpParameters = _T((b"shell32.dll,Control_RunDLL hdwwiz.cpl\0".as_ptr() as *const i8).cast());

    if ShellExecuteEx(core::ptr::addr_of_mut!(shInputDll)) == 0 {
        MessageBox(
            core::ptr::null_mut(),
            _T((b"Can't start hdwwiz.cpl\0".as_ptr() as *const i8).cast()),
            core::ptr::null_mut(),
            MB_OK | MB_ICONERROR,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn HardwarePageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {}

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_HARDWARE_DEVICE_MANAGER => {
                    if LaunchDeviceManager(hwndDlg) == 0 {
                        /* FIXME */
                    }
                    return TRUE;
                }

                IDC_HARDWARE_WIZARD => {
                    LaunchHardwareWizard(hwndDlg);
                    return TRUE;
                }

                IDC_HARDWARE_PROFILE => {
                    DialogBox!(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_HARDWAREPROFILES),
                        hwndDlg,
                        Some(HardProfDlgProc),
                    );
                    return TRUE;
                }

                _ => {}
            }
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
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/hardware.c
//  * PURPOSE:     Hardware devices
//  * COPYRIGHT:   Copyright Thomas Weidenmueller <w3seek@reactos.org>
//  *              Copyright 2006 Ged Murphy <gedmurphy@gmail.com>
//  *
//  */
//
// #include "precomp.h"
//
// typedef BOOL (WINAPI *PDEVMGREXEC)(HWND hWndParent, HINSTANCE hInst, PVOID Unknown, int nCmdShow);
//
// static BOOL
// LaunchDeviceManager(HWND hWndParent)
// {
// /* Hack for ROS to start our devmgmt until we have MMC */
// #ifdef __REACTOS__
//     return ((INT_PTR)ShellExecuteW(NULL, L"open", L"devmgmt.exe", NULL, NULL, SW_SHOWNORMAL) > 32);
// #else
//     HMODULE hDll;
//     PDEVMGREXEC DevMgrExec;
//     BOOL Ret;
//
//     hDll = LoadLibrary(_TEXT("devmgr.dll"));
//     if(!hDll)
//         return FALSE;
//
//     DevMgrExec = (PDEVMGREXEC)GetProcAddress(hDll, "DeviceManager_ExecuteW");
//     if(!DevMgrExec)
//     {
//         FreeLibrary(hDll);
//         return FALSE;
//     }
//
//     /* Run the Device Manager */
//     Ret = DevMgrExec(hWndParent, hApplet, NULL /* ??? */, SW_SHOW);
//     FreeLibrary(hDll);
//     return Ret;
// #endif /* __REACTOS__ */
// }
//
// static VOID
// LaunchHardwareWizard(HWND hWndParent)
// {
//     SHELLEXECUTEINFO shInputDll;
//
//     memset(&shInputDll, 0x0, sizeof(SHELLEXECUTEINFO));
//
//     shInputDll.cbSize = sizeof(shInputDll);
//     shInputDll.hwnd = hWndParent;
//     shInputDll.lpVerb = _T("open");
//     shInputDll.lpFile = _T("rundll32.exe");
//     shInputDll.lpParameters = _T("shell32.dll,Control_RunDLL hdwwiz.cpl");
//
//     if (ShellExecuteEx(&shInputDll) == 0)
//     {
//         MessageBox(NULL,
//                    _T("Can't start hdwwiz.cpl"),
//                    NULL,
//                    MB_OK | MB_ICONERROR);
//     }
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// HardwarePageProc(HWND hwndDlg,
//                  UINT uMsg,
//                  WPARAM wParam,
//                  LPARAM lParam)
// {
//     UNREFERENCED_PARAMETER(lParam);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_HARDWARE_DEVICE_MANAGER:
//                     if (!LaunchDeviceManager(hwndDlg))
//                     {
//                         /* FIXME */
//                     }
//                     return TRUE;
//
//                 case IDC_HARDWARE_WIZARD:
//                     LaunchHardwareWizard(hwndDlg);
//                     return TRUE;
//
//                 case IDC_HARDWARE_PROFILE:
//                     DialogBox(hApplet,
//                               MAKEINTRESOURCE(IDD_HARDWAREPROFILES),
//                               hwndDlg,
//                               HardProfDlgProc);
//                     return TRUE;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
