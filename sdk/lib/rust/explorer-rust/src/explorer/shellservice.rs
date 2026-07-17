use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;
unsafe extern "system" fn InitializeAllCallback(pItem: *mut core::ffi::c_void, pData: *mut core::ffi::c_void) -> i32 {
    let pOct = pItem as *mut IOleCommandTarget;
    let phr = pData as *mut HRESULT;
    TRACE!(b"Initializing SSO %p\n\0".as_ptr() as *const i8, pOct);
    *phr = (*pOct).Exec(
        &CGID_ShellServiceObject,
        OLECMDID_NEW,
        OLECMDEXECOPT_DODEFAULT,
        core::ptr::null_mut(),
        core::ptr::null_mut()
    );
    SUCCEEDED(*phr) as i32
}
unsafe extern "system" fn ShutdownAllCallback(pItem: *mut core::ffi::c_void, _pData: *mut core::ffi::c_void) -> i32 {
    let pOct = pItem as *mut IOleCommandTarget;
    TRACE!(b"Shutting down SSO %p\n\0".as_ptr() as PCHAR, pOct);
    (*pOct).Exec(
        &CGID_ShellServiceObject,
        OLECMDID_SAVE,
        OLECMDEXECOPT_DODEFAULT,
        core::ptr::null_mut(),
        core::ptr::null_mut()
    );
    1
}
unsafe extern "system" fn DeleteAllEnumCallback(pItem: *mut core::ffi::c_void, _pData: *mut core::ffi::c_void) -> i32 {
    let pOct = pItem as *mut IOleCommandTarget;
    TRACE!("Releasing SSO %p\n", pOct);
    (*pOct).Release();
    1
}
#[no_mangle]
pub unsafe extern "C" fn InitShellServices(phdpa: *mut HDPA) -> HRESULT {
    let mut pOct: *mut IOleCommandTarget = core::ptr::null_mut();
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut clsid: CLSID = core::mem::zeroed();
    let mut name: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut value: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut type_: u32 = 0;
    let mut ret: i32;
    let hdpa: HDPA;
    let mut hr: HRESULT = S_OK;
    let mut count: i32 = 0;

    *phdpa = core::ptr::null_mut();

    TRACE!(b"Enumerating Shell Service Object GUIDs...\n\0".as_ptr() as *const i8);

    if RegOpenKeyW(HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\ShellServiceObjectDelayLoad").as_ptr(),
        &mut hkey) != 0
    {
        ERR!(b"RegOpenKeyW failed.\n\0".as_ptr() as *const i8);
        return HRESULT_FROM_WIN32(GetLastError());
    }

    hdpa = DPA_Create(5);

    loop {
        let mut name_len: u32 = MAX_PATH as u32;
        let mut value_len: u32 = core::mem::size_of_val(&value) as u32;

        ret = RegEnumValueW(hkey, count as u32, name.as_mut_ptr(), &mut name_len, core::ptr::null_mut(), &mut type_, value.as_mut_ptr() as LPBYTE, &mut value_len);
        if ret != 0 {
            break;
        }

        if type_ != REG_SZ {
            WARN!(b"Value type was not REG_SZ.\n\0".as_ptr() as *const i8);
            count += 1;
            continue;
        }

        hr = CLSIDFromString(value.as_ptr(), &mut clsid);
        if FAILED_UNEXPECTEDLY(hr) != 0 {
            ERR!(b"CLSIDFromString failed %08x.\n\0".as_ptr() as *const i8, hr);
            RegCloseKey(hkey);
            *phdpa = core::ptr::null_mut();
            ShutdownShellServices(hdpa);
            return hr;
        }

        hr = CoCreateInstance(&clsid, core::ptr::null_mut(), CLSCTX_INPROC_SERVER, &IID_IOleCommandTarget, &mut pOct as *mut *mut IOleCommandTarget as *mut *mut core::ffi::c_void);
        if FAILED_UNEXPECTEDLY(hr) != 0 {
            ERR!(b"CoCreateInstance failed %08x.\n\0".as_ptr() as *const i8, hr);
            RegCloseKey(hkey);
            *phdpa = core::ptr::null_mut();
            ShutdownShellServices(hdpa);
            return hr;
        }

        DPA_AppendPtr(hdpa, pOct as *mut core::ffi::c_void);

        count += 1;
    }

    if ret != ERROR_NO_MORE_ITEMS as i32 {
        ERR!(b"RegEnumValueW failed %08x.\n\0".as_ptr() as *const i8, ret);
        hr = HRESULT_FROM_WIN32(GetLastError());
        RegCloseKey(hkey);
        *phdpa = core::ptr::null_mut();
        ShutdownShellServices(hdpa);
        return hr;
    }

    DPA_EnumCallback(hdpa, Some(InitializeAllCallback), &mut hr as *mut _ as *mut core::ffi::c_void);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        RegCloseKey(hkey);
        *phdpa = core::ptr::null_mut();
        ShutdownShellServices(hdpa);
        return hr;
    }

    RegCloseKey(hkey);

    *phdpa = hdpa;
    if count > 0 { S_OK } else { S_FALSE }
}
#[no_mangle]
pub unsafe extern "C" fn ShutdownShellServices(hdpa: HDPA) -> HRESULT {
    DPA_EnumCallback(hdpa, Some(ShutdownAllCallback), core::ptr::null_mut());
    DPA_EnumCallback(hdpa, Some(DeleteAllEnumCallback), core::ptr::null_mut());
    DPA_Destroy(hdpa);
    S_OK
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * ReactOS Explorer
// *
// * Copyright 2014 - David Quintana
// *
// * This library is free software; you can redistribute it and/or
// * modify it under the terms of the GNU Lesser General Public
// * License as published by the Free Software Foundation; either
// * version 2.1 of the License, or (at your option) any later version.
// *
// * This library is distributed in the hope that it will be useful,
// * but WITHOUT ANY WARRANTY; without even the implied warranty of
// * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// * Lesser General Public License for more details.
// *
// * You should have received a copy of the GNU Lesser General Public
// * License along with this library; if not, write to the Free Software
// * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
// */
//
// #include "precomp.h"
//
// static int __stdcall InitializeAllCallback(void* pItem, void* pData)
// {
//     IOleCommandTarget * pOct = reinterpret_cast<IOleCommandTarget *>(pItem);
//     HRESULT * phr = reinterpret_cast<HRESULT *>(pData);
//     TRACE("Initializing SSO %p\n", pOct);
//     *phr = pOct->Exec(&CGID_ShellServiceObject, OLECMDID_NEW, OLECMDEXECOPT_DODEFAULT, NULL, NULL);
//     return SUCCEEDED(*phr);
// }
//
// static int __stdcall ShutdownAllCallback(void* pItem, void* pData)
// {
//     IOleCommandTarget * pOct = reinterpret_cast<IOleCommandTarget *>(pItem);
//     TRACE("Shutting down SSO %p\n", pOct);
//     pOct->Exec(&CGID_ShellServiceObject, OLECMDID_SAVE, OLECMDEXECOPT_DODEFAULT, NULL, NULL);
//     return TRUE;
// }
//
// static int __stdcall DeleteAllEnumCallback(void* pItem, void* pData)
// {
//     IOleCommandTarget * pOct = reinterpret_cast<IOleCommandTarget *>(pItem);
//     TRACE("Releasing SSO %p\n", pOct);
//     pOct->Release();
//     return TRUE;
// }
//
// HRESULT InitShellServices(HDPA * phdpa)
// {
//     IOleCommandTarget * pOct;
//     HKEY    hkey;
//     CLSID   clsid;
//     WCHAR   name[MAX_PATH];
//     WCHAR   value[MAX_PATH];
//     DWORD   type;
//     LONG    ret;
//     HDPA    hdpa;
//     HRESULT hr = S_OK;
//     int     count = 0;
//
//     *phdpa = NULL;
//
//     TRACE("Enumerating Shell Service Object GUIDs...\n");
//
//     if (RegOpenKeyW(HKEY_LOCAL_MACHINE,
//         L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\ShellServiceObjectDelayLoad",
//         &hkey))
//     {
//         ERR("RegOpenKeyW failed.\n");
//         return HRESULT_FROM_WIN32(GetLastError());
//     }
//
//     hdpa = DPA_Create(5);
//
//     /* Enumerate */
//     do
//     {
//         DWORD   name_len = MAX_PATH;
//         DWORD   value_len = sizeof(value); /* byte count! */
//
//         ret = RegEnumValueW(hkey, count, name, &name_len, 0, &type, (LPBYTE) &value, &value_len);
//         if (ret)
//             break;
//
//         if (type != REG_SZ)
//         {
//             WARN("Value type was not REG_SZ.\n");
//             continue;
//         }
//
//         hr = CLSIDFromString(value, &clsid);
//         if (FAILED_UNEXPECTEDLY(hr))
//         {
//             ERR("CLSIDFromString failed %08x.\n", hr);
//             goto cleanup;
//         }
//
//         hr = CoCreateInstance(clsid, NULL, CLSCTX_INPROC_SERVER, IID_PPV_ARG(IOleCommandTarget, &pOct));
//         if (FAILED_UNEXPECTEDLY(hr))
//         {
//             ERR("CoCreateInstance failed %08x.\n", hr);
//             goto cleanup;
//         }
//
//         DPA_AppendPtr(hdpa, pOct);
//
//         count++;
//     }
//     while (1);
//
//     if (ret != ERROR_NO_MORE_ITEMS)
//     {
//         ERR("RegEnumValueW failed %08x.\n", ret);
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//
//     /* Initialize */
//     DPA_EnumCallback(hdpa, InitializeAllCallback, &hr);
//     if (FAILED_UNEXPECTEDLY(hr))
//         goto cleanup;
//
//     RegCloseKey(hkey);
//
//     *phdpa = hdpa;
//     return count > 0 ? S_OK : S_FALSE;
//
// cleanup:
//     RegCloseKey(hkey);
//     *phdpa = NULL;
//     ShutdownShellServices(hdpa);
//     return hr;
// }
//
// HRESULT ShutdownShellServices(HDPA hdpa)
// {
//     DPA_EnumCallback(hdpa, ShutdownAllCallback, NULL);
//     DPA_EnumCallback(hdpa, DeleteAllEnumCallback, NULL);
//     DPA_Destroy(hdpa);
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
