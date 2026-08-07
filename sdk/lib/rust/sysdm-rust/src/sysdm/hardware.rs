// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PDEVMGREXEC = Option<unsafe extern "system" fn(hWndParent: HWND, hInst: HMODULE, Unknown: PVOID, nCmdShow: i32) -> i32>;
// hardware.c:19 is `#ifdef __REACTOS__`, a BUILD-CONFIGURATION conditional, and we ARE
// ReactOS -- so only this arm exists and the `#else` (LoadLibrary devmgr.dll, which does not
// ship here) is not carried. The paid body emitted BOTH arms under `#[cfg(__REACTOS__)]` and
// `#[cfg(not(__REACTOS__))]`, a cfg nothing ever sets, so both compiled OUT and the function
// returned nothing at all; on top of that the surviving arm had L"open" as SW_SHOWNORMAL,
// L"devmgmt.exe" as 32, SW_SHOWNORMAL as FALSE, and the `> 32` as `> FALSE`.
unsafe extern "C" fn LaunchDeviceManager(_hWndParent: HWND) -> BOOL {
    ((ShellExecuteW(
        core::ptr::null_mut(),
        w!("open"),
        w!("devmgmt.exe"),
        core::ptr::null(),
        core::ptr::null(),
        (SW_SHOWNORMAL) as INT,
    ) as INT_PTR)
        > 32) as BOOL
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
    shInputDll.lpVerb = _T(w!("open"));
    shInputDll.lpFile = _T(w!("rundll32.exe"));
    shInputDll.lpParameters = _T(w!("shell32.dll,Control_RunDLL hdwwiz.cpl"));

    if ShellExecuteEx(core::ptr::addr_of_mut!(shInputDll)) == 0 {
        MessageBox(
            core::ptr::null_mut(),
            _T(w!("Can't start hdwwiz.cpl")),
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
                    return (TRUE) as INT_PTR;
                }

                IDC_HARDWARE_WIZARD => {
                    LaunchHardwareWizard(hwndDlg);
                    return (TRUE) as INT_PTR;
                }

                IDC_HARDWARE_PROFILE => {
                    DialogBox!(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_HARDWAREPROFILES),
                        hwndDlg,
                        Some(HardProfDlgProc),
                    );
                    return (TRUE) as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as INT_PTR
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
