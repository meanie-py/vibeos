//! rshell.cpp → Rust: rshell.dll loader with shell32/shdocvw ordinal
//! fallbacks. Integration deltas vs raw output: FARPROC (Option) handled via
//! if-let + transmute, wide literals via w!, real CLSIDs from libuuid (raw
//! output stubbed CLSID_BandSiteMenu as zeros — silent bug).

use core::ffi::c_void;
use explorer_sys::com::{
    CLSCTX_INPROC_SERVER, CLSID_BandSiteMenu, CLSID_RebarBandSite, CLSID_StartMenu,
    CoCreateInstance, IShellDesktopTray, IUnknown, REFIID,
};
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

pub static mut ghRShell: HINSTANCE = core::ptr::null_mut();

type FnStartMenuCreateInstance =
    unsafe extern "system" fn(riid: *const IID, ppv: *mut *mut c_void) -> HRESULT;
type FnSHCreateDesktop =
    unsafe extern "system" fn(ShellDesk: *mut IShellDesktopTray) -> HANDLE;
type FnSHDesktopMessageLoop = unsafe extern "system" fn(hDesktop: HANDLE) -> BOOL;
type FnWinListInit = unsafe extern "system" fn() -> BOOL;
type FnShellDDEInit = unsafe extern "system" fn(bInit: BOOL);
type FnBandSiteMenuCreateInstance =
    unsafe extern "system" fn(riid: *const IID, ppv: *mut *mut c_void) -> HRESULT;
type FnBandSiteCreateInstance = unsafe extern "system" fn(
    pUnkOuter: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT;

#[no_mangle]
pub unsafe extern "C" fn InitRSHELL() {
    ghRShell = LoadLibraryW(w!("rshell.dll").as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn _CStartMenu_CreateInstance(
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if !ghRShell.is_null() {
        if let Some(f) = GetProcAddress(ghRShell, b"CStartMenu_CreateInstance\0".as_ptr() as LPCSTR) {
            let f: FnStartMenuCreateInstance = core::mem::transmute(f);
            return f(riid, ppv);
        }
    }

    CoCreateInstance(
        &CLSID_StartMenu,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        riid,
        ppv,
    )
}

#[no_mangle]
pub unsafe extern "system" fn _SHCreateDesktop(ShellDesk: *mut IShellDesktopTray) -> HANDLE {
    if !ghRShell.is_null() {
        if let Some(f) = GetProcAddress(ghRShell, b"SHCreateDesktop\0".as_ptr() as LPCSTR) {
            let f: FnSHCreateDesktop = core::mem::transmute(f);
            return f(ShellDesk);
        }
    }

    let hFallback = GetModuleHandleW(w!("shell32.dll").as_ptr());
    if !hFallback.is_null() {
        if let Some(f) = GetProcAddress(hFallback, 200 as LPCSTR) {
            let f: FnSHCreateDesktop = core::mem::transmute(f);
            return f(ShellDesk);
        }
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn _SHDesktopMessageLoop(hDesktop: HANDLE) -> BOOL {
    if !ghRShell.is_null() {
        if let Some(f) = GetProcAddress(ghRShell, b"SHDesktopMessageLoop\0".as_ptr() as LPCSTR) {
            let f: FnSHDesktopMessageLoop = core::mem::transmute(f);
            return f(hDesktop);
        }
    }

    let hFallback = GetModuleHandleW(w!("shell32.dll").as_ptr());
    if !hFallback.is_null() {
        if let Some(f) = GetProcAddress(hFallback, 201 as LPCSTR) {
            let f: FnSHDesktopMessageLoop = core::mem::transmute(f);
            return f(hDesktop);
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn _WinList_Init() -> BOOL {
    if !ghRShell.is_null() {
        if let Some(f) = GetProcAddress(ghRShell, b"WinList_Init\0".as_ptr() as LPCSTR) {
            let f: FnWinListInit = core::mem::transmute(f);
            return f();
        }
    }

    let hFallback = LoadLibraryW(w!("shdocvw.dll").as_ptr());
    if !hFallback.is_null() {
        if let Some(f) = GetProcAddress(hFallback, 110 as LPCSTR) {
            let f: FnWinListInit = core::mem::transmute(f);
            return f();
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn _ShellDDEInit(bInit: BOOL) {
    if !ghRShell.is_null() {
        if let Some(f) = GetProcAddress(ghRShell, b"ShellDDEInit\0".as_ptr() as LPCSTR) {
            let f: FnShellDDEInit = core::mem::transmute(f);
            f(bInit);
            return;
        }
    }

    let hFallback = GetModuleHandleW(w!("shell32.dll").as_ptr());
    if !hFallback.is_null() {
        if let Some(f) = GetProcAddress(hFallback, 188 as LPCSTR) {
            let f: FnShellDDEInit = core::mem::transmute(f);
            f(bInit);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn _CBandSiteMenu_CreateInstance(
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if !ghRShell.is_null() {
        if let Some(f) = GetProcAddress(ghRShell, b"CBandSiteMenu_CreateInstance\0".as_ptr() as LPCSTR) {
            let f: FnBandSiteMenuCreateInstance = core::mem::transmute(f);
            return f(riid, ppv);
        }
    }

    CoCreateInstance(
        &CLSID_BandSiteMenu,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        riid,
        ppv,
    )
}

#[no_mangle]
pub unsafe extern "system" fn _CBandSite_CreateInstance(
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if !ghRShell.is_null() {
        if let Some(f) = GetProcAddress(ghRShell, b"CBandSite_CreateInstance\0".as_ptr() as LPCSTR) {
            let f: FnBandSiteCreateInstance = core::mem::transmute(f);
            return f(pUnkOuter, riid, ppv);
        }
    }

    CoCreateInstance(
        &CLSID_RebarBandSite,
        pUnkOuter,
        CLSCTX_INPROC_SERVER,
        riid,
        ppv,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Explorer
//  *
//  * Copyright 2014 Giannis Adamopoulos
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
//
// static HINSTANCE ghRShell = NULL;
//
// typedef HRESULT(WINAPI * PSTARTMENU_CREATEINSTANCE)(REFIID riid, void **ppv);
//
// VOID InitRSHELL(VOID)
// {
//     ghRShell = LoadLibraryW(L"rshell.dll");
// }
//
// HRESULT WINAPI _CStartMenu_CreateInstance(REFIID riid, void **ppv)
// {
//     if (ghRShell)
//     {
//         PSTARTMENU_CREATEINSTANCE func = (PSTARTMENU_CREATEINSTANCE)GetProcAddress(ghRShell, "CStartMenu_CreateInstance");
//         if (func)
//         {
//             return func(riid, ppv);
//         }
//     }
//
//     return CoCreateInstance(CLSID_StartMenu,
//                             NULL,
//                             CLSCTX_INPROC_SERVER,
//                             riid,
//                             ppv);
// }
//
// typedef HANDLE(WINAPI * PSHCREATEDESKTOP)(IShellDesktopTray *ShellDesk);
//
// HANDLE WINAPI _SHCreateDesktop(IShellDesktopTray *ShellDesk)
// {
//     HINSTANCE hFallback;
//
//     if (ghRShell)
//     {
//         PSHCREATEDESKTOP func = (PSHCREATEDESKTOP)GetProcAddress(ghRShell, "SHCreateDesktop");
//         if (func)
//         {
//             return func(ShellDesk);
//         }
//     }
//
//     hFallback = GetModuleHandleW(L"shell32.dll");
//
//     if (hFallback)
//     {
//         PSHCREATEDESKTOP func = (PSHCREATEDESKTOP) GetProcAddress(hFallback, (LPCSTR) 200);
//         if (func)
//         {
//             return func(ShellDesk);
//         }
//     }
//
//     return 0;
// }
//
// typedef BOOL(WINAPI *PSHDESKTOPMESSAGELOOP)(HANDLE hDesktop);
//
// BOOL WINAPI _SHDesktopMessageLoop(HANDLE hDesktop)
// {
//     HINSTANCE hFallback;
//
//     if (ghRShell)
//     {
//         PSHDESKTOPMESSAGELOOP func = (PSHDESKTOPMESSAGELOOP)GetProcAddress(ghRShell, "SHDesktopMessageLoop");
//         if (func)
//         {
//             return func(hDesktop);
//         }
//     }
//
//     hFallback = GetModuleHandleW(L"shell32.dll");
//
//     if (hFallback)
//     {
//         PSHDESKTOPMESSAGELOOP func = (PSHDESKTOPMESSAGELOOP) GetProcAddress(hFallback, (LPCSTR) 201);
//         if (func)
//         {
//             return func(hDesktop);
//         }
//     }
//
//     return FALSE;
// }
//
// typedef BOOL (WINAPI *PWINLIST_INIT)(void);
//
// BOOL WINAPI _WinList_Init(void)
// {
//     HINSTANCE hFallback;
//
//     if (ghRShell)
//     {
//         PWINLIST_INIT func = (PWINLIST_INIT)GetProcAddress(ghRShell, "WinList_Init");
//         if (func)
//         {
//             return func();
//         }
//     }
//
//     hFallback = LoadLibraryW(L"shdocvw.dll");
//
//     if (hFallback)
//     {
//         PWINLIST_INIT func = (PWINLIST_INIT) GetProcAddress(hFallback, (LPCSTR) 110);
//         if (func)
//         {
//             return func();
//         }
//     }
//
//     return FALSE;
// }
//
// typedef void (WINAPI *PSHELLDDEINIT)(BOOL bInit);
//
// void WINAPI _ShellDDEInit(BOOL bInit)
// {
//     HINSTANCE hFallback;
//
//     if (ghRShell)
//     {
//         PSHELLDDEINIT func = (PSHELLDDEINIT)GetProcAddress(ghRShell, "ShellDDEInit");
//         if (func)
//         {
//             func(bInit);
//             return;
//         }
//     }
//
//     hFallback = GetModuleHandleW(L"shell32.dll");
//
//     if (hFallback)
//     {
//         PSHELLDDEINIT func = (PSHELLDDEINIT) GetProcAddress(hFallback, (LPCSTR) 188);
//         if (func)
//         {
//             func(bInit);
//             return;
//         }
//     }
// }
//
// typedef HRESULT (WINAPI *CBANDSITEMENU_CREATEINSTANCE)(REFIID riid, void **ppv);
// HRESULT WINAPI _CBandSiteMenu_CreateInstance(REFIID riid, void **ppv)
// {
//     if (ghRShell)
//     {
//         CBANDSITEMENU_CREATEINSTANCE func = (CBANDSITEMENU_CREATEINSTANCE)GetProcAddress(ghRShell, "CBandSiteMenu_CreateInstance");
//         if (func)
//         {
//             return func(riid, ppv);
//         }
//     }
//
//     return CoCreateInstance(CLSID_BandSiteMenu,
//                             NULL,
//                             CLSCTX_INPROC_SERVER,
//                             riid,
//                             ppv);
// }
//
// typedef HRESULT (WINAPI *CBANDSITE_CREATEINSTANCE)(LPUNKNOWN pUnkOuter, REFIID riid, void **ppv);
// HRESULT WINAPI _CBandSite_CreateInstance(LPUNKNOWN pUnkOuter, REFIID riid, void **ppv)
// {
//     if (ghRShell)
//     {
//         CBANDSITE_CREATEINSTANCE func = (CBANDSITE_CREATEINSTANCE)GetProcAddress(ghRShell, "CBandSite_CreateInstance");
//         if (func)
//         {
//             return func(pUnkOuter, riid, ppv);
//         }
//     }
//
//     return CoCreateInstance(CLSID_RebarBandSite,
//                             pUnkOuter,
//                             CLSCTX_INPROC_SERVER,
//                             riid,
//                             ppv);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
