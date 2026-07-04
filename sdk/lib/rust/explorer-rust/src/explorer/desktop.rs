use core::ffi::c_void;
use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use explorer_sys::types::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

extern "C" {
    fn CDesktopThread_new() -> *mut CDesktopThread;
    fn CDesktopThread_delete(this: *mut CDesktopThread);
    fn FAILED_UNEXPECTEDLY(hr: HRESULT) -> bool;
}
extern "system" {
    fn OleInitialize(pvReserved: LPVOID) -> i32;
    fn OleUninitialize();
    fn _SHCreateDesktop(pSdt: *mut IShellDesktopTray) -> *mut core::ffi::c_void;
    fn _SHDesktopMessageLoop(hDesktop: *mut core::ffi::c_void);
    fn SUCCEEDED(hr: i32) -> i32;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct CDesktopThread {
    pub m_Tray: *mut ITrayWindow,
    pub m_hInitEvent: HANDLE,
    pub m_hThread: HANDLE,
    pub m_ThreadId: DWORD,
}

pub type PCDesktopThread = *mut CDesktopThread;

#[no_mangle]
pub unsafe extern "C" fn CDesktopThread_ctor(this: *mut CDesktopThread) {
    (*this).m_Tray = core::ptr::null_mut();
    (*this).m_hInitEvent = core::ptr::null_mut();
    (*this).m_hThread = core::ptr::null_mut();
    (*this).m_ThreadId = 0;
}

#[no_mangle]
pub unsafe extern "C" fn CDesktopThread_dtor(this: *mut CDesktopThread) {
    CDesktopThread_Destroy(this);
}

#[no_mangle]
pub unsafe extern "C" fn CDesktopThread_Initialize(
    this: *mut CDesktopThread,
    pTray: *mut ITrayWindow,
) -> HRESULT {
    let mut Handles: [HANDLE; 2] = [core::ptr::null_mut(); 2];

    if pTray.is_null() || !(*this).m_Tray.is_null() {
        return E_FAIL;
    }

    (*this).m_hInitEvent = CreateEventW(
        core::ptr::null_mut(),
        0,
        0,
        core::ptr::null_mut(),
    );
    if (*this).m_hInitEvent.is_null() {
        return E_FAIL;
    }

    (*this).m_Tray = pTray;
    (*this).m_hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(CDesktopThread_s_DesktopThreadProc),
        this as LPVOID,
        0,
        core::ptr::addr_of_mut!((*this).m_ThreadId),
    );

    if (*this).m_hThread.is_null() {
        CloseHandle((*this).m_hInitEvent);
        (*this).m_hInitEvent = core::ptr::null_mut();

        (*this).m_Tray = core::ptr::null_mut();

        return E_FAIL;
    }

    Handles[0] = (*this).m_hThread;
    Handles[1] = (*this).m_hInitEvent;

    loop {
        let WaitResult = MsgWaitForMultipleObjects(
            2,
            Handles.as_ptr(),
            0,
            INFINITE,
            QS_ALLEVENTS,
        );

        if WaitResult == WAIT_OBJECT_0 + 2 {
            TrayProcessMessages((*this).m_Tray);
        } else if WaitResult != 0xFFFFFFFF && WaitResult != WAIT_OBJECT_0 {
            break;
        } else {
            CloseHandle((*this).m_hThread);
            (*this).m_hThread = core::ptr::null_mut();
            (*this).m_ThreadId = 0;

            CloseHandle((*this).m_hInitEvent);
            (*this).m_hInitEvent = core::ptr::null_mut();

            (*this).m_Tray = core::ptr::null_mut();

            return E_FAIL;
        }
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn CDesktopThread_Destroy(this: *mut CDesktopThread) {
    if !(*this).m_hThread.is_null() {
        let WaitResult = WaitForSingleObject((*this).m_hThread, 0);
        if WaitResult == WAIT_TIMEOUT {
            PostThreadMessageW((*this).m_ThreadId, WM_QUIT, 0, 0);
            WaitForSingleObject((*this).m_hThread, INFINITE);
        }

        CloseHandle((*this).m_hThread);
        (*this).m_hThread = core::ptr::null_mut();
        (*this).m_ThreadId = 0;
    }

    if !(*this).m_hInitEvent.is_null() {
        CloseHandle((*this).m_hInitEvent);
        (*this).m_hInitEvent = core::ptr::null_mut();
    }

    (*this).m_Tray = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn CDesktopThread_DesktopThreadProc(this: *mut CDesktopThread) -> u32 {
    let mut pSdt: *mut IShellDesktopTray = core::ptr::null_mut();
    let mut hDesktop: *mut core::ffi::c_void;
    let hRet: i32;
    let mut dwResult: u32 = 1;

    OleInitialize(core::ptr::null_mut());

    hRet = ((*(*this).m_Tray).lpVtbl).QueryInterface(
        (*this).m_Tray,
        &IID_IShellDesktopTray,
        &mut pSdt as *mut *mut IShellDesktopTray as *mut *mut core::ffi::c_void
    );
    if SUCCEEDED(hRet) == 0 {
        OleUninitialize();
        return dwResult;
    }

    hDesktop = _SHCreateDesktop(pSdt);
    if hDesktop.is_null() {
        if !pSdt.is_null() {
            ((*pSdt).lpVtbl).Release(pSdt);
        }
        OleUninitialize();
        return dwResult;
    }

    if SetEvent((*this).m_hInitEvent) == 0 {
        if !pSdt.is_null() {
            ((*pSdt).lpVtbl).Release(pSdt);
        }
        OleUninitialize();
        return dwResult;
    }

    _SHDesktopMessageLoop(hDesktop);
    dwResult = 0;

    if !pSdt.is_null() {
        ((*pSdt).lpVtbl).Release(pSdt);
    }
    OleUninitialize();
    dwResult
}

#[repr(C)]
struct IShellDesktopTray {
    lpVtbl: *const IShellDesktopTrayVtbl,
}

#[repr(C)]
struct IShellDesktopTrayVtbl {
    QueryInterface: unsafe extern "system" fn(*mut IShellDesktopTray, *const GUID, *mut *mut core::ffi::c_void) -> i32,
    AddRef: unsafe extern "system" fn(*mut IShellDesktopTray) -> u32,
    Release: unsafe extern "system" fn(*mut IShellDesktopTray) -> u32,
}

#[repr(C)]
struct GUID {
    Data1: u32,
    Data2: u16,
    Data3: u16,
    Data4: [u8; 8],
}

static IID_IShellDesktopTray: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

#[no_mangle]
pub unsafe extern "system" fn CDesktopThread_s_DesktopThreadProc(lpParameter: LPVOID) -> u32 {
    let pDesktopThread = lpParameter as *mut CDesktopThread;
    CDesktopThread_DesktopThreadProc(pDesktopThread)
}
#[no_mangle]
pub unsafe extern "C" fn DesktopCreateWindow(Tray: *mut ITrayWindow) -> *mut core::ffi::c_void {

    let pDesktopThread = CDesktopThread_new();

    let hres = CDesktopThread_Initialize(pDesktopThread, Tray);
    if FAILED_UNEXPECTEDLY(hres) {
        CDesktopThread_delete(pDesktopThread);
        return core::ptr::null_mut();
    }

    pDesktopThread as *mut core::ffi::c_void
}
#[no_mangle]
pub unsafe extern "C" fn DesktopDestroyShellWindow(hDesktop: HANDLE) {
    let pDesktopThread = hDesktop as *mut CDesktopThread;
    Box::from_raw(pDesktopThread);
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
//
// class CDesktopThread
// {
// private:
//     CComPtr<ITrayWindow> m_Tray;
//     HANDLE m_hInitEvent;
//     HANDLE m_hThread;
//     DWORD m_ThreadId;
//
//     DWORD DesktopThreadProc();
//     static DWORD WINAPI s_DesktopThreadProc(LPVOID lpParameter);
//
// public:
//     CDesktopThread();
//     ~CDesktopThread();
//
//     HRESULT Initialize(ITrayWindow* pTray);
//     void Destroy();
// };
//
// /*******************************************************************/
//
// CDesktopThread::CDesktopThread():
//     m_Tray(NULL),
//     m_hInitEvent(NULL),
//     m_hThread(NULL),
//     m_ThreadId(0)
// {
// }
//
// CDesktopThread::~CDesktopThread()
// {
//     Destroy();
// }
//
// HRESULT CDesktopThread::Initialize(ITrayWindow* pTray)
// {
//     HANDLE Handles[2];
//
//     if (!pTray || m_Tray)
//     {
//         return E_FAIL;
//     }
//
//     m_hInitEvent = CreateEventW(NULL, FALSE, FALSE, NULL);
//     if (!m_hInitEvent)
//     {
//         return E_FAIL;
//     }
//
//     m_Tray = pTray;
//     m_hThread = CreateThread(NULL, 0, s_DesktopThreadProc, (LPVOID)this, 0, &m_ThreadId);
//
//     if (!m_hThread)
//     {
//         CloseHandle(m_hInitEvent);
//         m_hInitEvent = NULL;
//
//         m_Tray = NULL;
//
//         return E_FAIL;
//     }
//
//     Handles[0] = m_hThread;
//     Handles[1] = m_hInitEvent;
//
//     for (;;)
//     {
//         DWORD WaitResult = MsgWaitForMultipleObjects(_countof(Handles), Handles, FALSE, INFINITE, QS_ALLEVENTS);
//
//         if (WaitResult == WAIT_OBJECT_0 + _countof(Handles))
//         {
//             TrayProcessMessages(m_Tray);
//         }
//         else if (WaitResult != WAIT_FAILED && WaitResult != WAIT_OBJECT_0)
//         {
//             break;
//         }
//         else
//         {
//             CloseHandle(m_hThread);
//             m_hThread = NULL;
//             m_ThreadId = 0;
//
//             CloseHandle(m_hInitEvent);
//             m_hInitEvent = NULL;
//
//             m_Tray = NULL;
//
//             return E_FAIL;
//         }
//     }
//     return S_OK;
// }
//
// void CDesktopThread::Destroy()
// {
//     if (m_hThread)
//     {
//         DWORD WaitResult = WaitForSingleObject(m_hThread, 0);
//         if (WaitResult == WAIT_TIMEOUT)
//         {
//             /* Send WM_QUIT message to the thread and wait for it to terminate */
//             PostThreadMessageW(m_ThreadId, WM_QUIT, 0, 0);
//             WaitForSingleObject(m_hThread, INFINITE);
//         }
//
//         CloseHandle(m_hThread);
//         m_hThread = NULL;
//         m_ThreadId = 0;
//     }
//
//     if (m_hInitEvent)
//     {
//         CloseHandle(m_hInitEvent);
//         m_hInitEvent = NULL;
//     }
//
//     m_Tray = NULL;
// }
//
// DWORD CDesktopThread::DesktopThreadProc()
// {
//     CComPtr<IShellDesktopTray> pSdt;
//     HANDLE hDesktop;
//     HRESULT hRet;
//     DWORD dwResult = 1;
//
//     OleInitialize(NULL);
//
//     hRet = m_Tray->QueryInterface(IID_PPV_ARG(IShellDesktopTray, &pSdt));
//     if (!SUCCEEDED(hRet))
//     {
//         goto Cleanup;
//     }
//
//     hDesktop = _SHCreateDesktop(pSdt);
//     if (!hDesktop)
//     {
//         goto Cleanup;
//     }
//
//     if (!SetEvent(m_hInitEvent))
//     {
//         /* Failed to notify that we initialized successfully, kill ourselves
//          * to make the main thread wake up! */
//         goto Cleanup;
//     }
//
//     _SHDesktopMessageLoop(hDesktop);
//     dwResult = 0;
//
// Cleanup:
//     OleUninitialize();
//     return dwResult;
// }
//
// DWORD WINAPI CDesktopThread::s_DesktopThreadProc(LPVOID lpParameter)
// {
//     CDesktopThread* pDesktopThread = static_cast<CDesktopThread*>(lpParameter);
//     return pDesktopThread->DesktopThreadProc();
// }
//
// /*******************************************************************/
//
// HANDLE
// DesktopCreateWindow(IN OUT ITrayWindow *Tray)
// {
//     CDesktopThread* pDesktopThread = new CDesktopThread();
//
//     HRESULT hres = pDesktopThread->Initialize(Tray);
//     if (FAILED_UNEXPECTEDLY(hres))
//     {
//         delete pDesktopThread;
//         return NULL;
//     }
//
//     return pDesktopThread;
// }
//
// VOID
// DesktopDestroyShellWindow(IN HANDLE hDesktop)
// {
//     CDesktopThread* pDesktopThread = reinterpret_cast<CDesktopThread*>(hDesktop);
//     delete pDesktopThread;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
