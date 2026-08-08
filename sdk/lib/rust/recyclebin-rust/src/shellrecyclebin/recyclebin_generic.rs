// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct RecycleBinGeneric {
    pub lpVtbl: *const RecycleBinGenericVtbl,
    pub m_ref: LONG,
}

#[repr(C)]
pub struct RecycleBinGenericVtbl {
    pub QueryInterface: Option<unsafe extern "system" fn(this: *mut RecycleBinGeneric, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT>,
    pub AddRef: Option<unsafe extern "system" fn(this: *mut RecycleBinGeneric) -> ULONG>,
    pub Release: Option<unsafe extern "system" fn(this: *mut RecycleBinGeneric) -> ULONG>,
    pub DeleteFile: Option<unsafe extern "system" fn(this: *mut RecycleBinGeneric, szFileName: LPCWSTR) -> HRESULT>,
    pub EmptyRecycleBin: Option<unsafe extern "system" fn(this: *mut RecycleBinGeneric) -> HRESULT>,
    pub EnumObjects: Option<unsafe extern "system" fn(this: *mut RecycleBinGeneric, ppEnumList: *mut *mut IRecycleBinEnumList) -> HRESULT>,
    pub GetDirectory: Option<unsafe extern "system" fn(this: *mut RecycleBinGeneric, szPath: LPWSTR) -> HRESULT>,
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBinGeneric_GetDirectory(
    this: *mut RecycleBinGeneric,
    szPath: LPWSTR,
) -> HRESULT {
    E_UNEXPECTED
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_QueryInterface(
    this: *mut RecycleBinGeneric,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }
    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IRecycleBin) {
        *ppvObject = this as *mut core::ffi::c_void;
        AddRef(this);
        S_OK
    } else {
        *ppvObject = core::ptr::null_mut();
        E_NOINTERFACE
    }
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_AddRef(this: *mut RecycleBinGeneric) -> u32 {
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*this).m_ref));
    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_dtor(this: *mut RecycleBinGeneric) {
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_Release(this: *mut RecycleBinGeneric) -> ULONG {
    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*this).m_ref));
    if ref_count == 0 {
        RecycleBinGeneric_destroy(this);
    }
    (ref_count) as u32
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBinGeneric_DeleteFile(this: *mut RecycleBinGeneric, szFileName: LPCWSTR) -> HRESULT {
    let mut szFullName: LPWSTR = core::ptr::null_mut();
    let mut dwBufferLength: u32 = 0;
    loop {
        let len = GetFullPathNameW(szFileName, dwBufferLength, szFullName, core::ptr::null_mut());
        if len == 0 {
            if !szFullName.is_null() {
                CoTaskMemFree((szFullName).cast());
            }
            return HRESULT_FROM_WIN32((GetLastError()) as LONG);
        } else if len < dwBufferLength {
            break;
        }
        if !szFullName.is_null() {
            CoTaskMemFree((szFullName).cast());
        }
        dwBufferLength = len;
        szFullName = CoTaskMemAlloc((dwBufferLength * (core::mem::size_of::<u16>()) as u32) as SIZE_T) as LPWSTR;
        if szFullName.is_null() {
            return HRESULT_FROM_WIN32((ERROR_NOT_ENOUGH_MEMORY) as LONG);
        }
    }

    let mut szVolume = [0u16; MAX_PATH as usize];
    swprintf(
        szVolume.as_mut_ptr(),
        &[0x25u16, 0x63u16, 0x3Au16, 0x5Cu16, 0x0u16] as *const u16 as LPCWSTR,
        *szFullName.add(0 as usize) as u32,
    );

    if szVolume[0 as usize] == 0x5Cu16
        && szVolume[1 as usize] == 0x5Cu16
        && (szVolume[2 as usize] == 0x2Eu16 || szVolume[2 as usize] == 0x3Fu16)
        && szVolume[3 as usize] == 0x5Cu16
    {
        MoveMemory!(
            szVolume.as_mut_ptr() as *mut core::ffi::c_void,
            szVolume.as_ptr().add(4 as usize) as *const core::ffi::c_void,
            (_countof!(szVolume) - 4) * (core::mem::size_of::<u16>() as u32) as usize,
        );
    }

    let mut prb: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr = GetDefaultRecycleBin(szVolume.as_mut_ptr(), (&mut prb as *mut *mut core::ffi::c_void).cast());
    if !((hr) >= 0) {
        CoTaskMemFree((szFullName).cast());
        return hr;
    }

    let vtbl = *(prb as *const *const extern "system" fn(*mut core::ffi::c_void, LPCWSTR) -> HRESULT);
    let delete_file = *vtbl.add(3);
    hr = delete_file(prb, szFullName);
    CoTaskMemFree((szFullName).cast());

    let vtbl_release = *(prb as *const *const extern "system" fn(*mut core::ffi::c_void) -> u32);
    let release = *vtbl_release.add(2);
    release(prb);

    hr
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBinGeneric_EmptyRecycleBin(this: *mut RecycleBinGeneric) -> HRESULT {
    let dwLogicalDrives = GetLogicalDrives();
    if dwLogicalDrives == 0 {
        return HRESULT_FROM_WIN32((GetLastError()) as LONG);
    }
    let end = b'Z' - b'A' + 1;
    for i in (0 as u32)..(end as u32) {
        if (dwLogicalDrives & (1 << i)) == 0 {
            continue;
        }
        let mut szVolumeName: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
        let format_str: &[u16] = &[b'%' as u16, b'c' as u16, b':' as u16, b'\\' as u16, 0u16];
        swprintf(
            szVolumeName.as_mut_ptr(),
            format_str.as_ptr(),
            (b'A' as u16 + i as u16) as u32,
        );
        if GetDriveTypeW(szVolumeName.as_ptr()) != DRIVE_FIXED {
            continue;
        }
        let mut prb: *mut IRecycleBin = core::ptr::null_mut();
        let hr = GetDefaultRecycleBin(szVolumeName.as_ptr(), &mut prb as *mut *mut IRecycleBin);
        if !((hr) >= 0) {
            return hr;
        }
        let hr = EmptyRecycleBin(prb);
        IRecycleBin_Release(prb);
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBinGeneric_EnumObjects(
    this: *mut RecycleBinGeneric,
    ppEnumList: *mut *mut IRecycleBinEnumList,
) -> HRESULT {
    RecycleBinGenericEnum_Constructor(ppEnumList)
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_ctor(this: *mut RecycleBinGeneric) {
    (*this).m_ref = 1;
}
#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_Constructor(ppUnknown: *mut *mut core::ffi::c_void) -> i32 {
    let pThis: *mut core::ffi::c_void = RecycleBinGeneric();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }
    *ppUnknown = pThis;
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_IsEqualFileIdentity(
    p1: *const RECYCLEBINFILEIDENTITY,
    p2: *const RECYCLEBINFILEIDENTITY,
) -> i32 {
    ((*p1).DeletionTime.dwLowDateTime == (*p2).DeletionTime.dwLowDateTime
        && (*p1).DeletionTime.dwHighDateTime == (*p2).DeletionTime.dwHighDateTime
        && _wcsicmp((*p1).RecycledFullPath, (*p2).RecycledFullPath) == 0) as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Recycle bin management
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Deals with a system-wide recycle bin
//  * COPYRIGHT:   Copyright 2007 Hervé Poussineau (hpoussin@reactos.org)
//  *              Copyright 2024 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "recyclebin_private.h"
//
// class RecycleBinGeneric : public IRecycleBin
// {
// public:
//     RecycleBinGeneric();
//     virtual ~RecycleBinGeneric();
//
//     /* IUnknown methods */
//     HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, void **ppvObject) override;
//     STDMETHODIMP_(ULONG) AddRef() override;
//     STDMETHODIMP_(ULONG) Release() override;
//
//     /* IRecycleBin methods */
//     HRESULT STDMETHODCALLTYPE DeleteFile(LPCWSTR szFileName) override;
//     HRESULT STDMETHODCALLTYPE EmptyRecycleBin() override;
//     HRESULT STDMETHODCALLTYPE EnumObjects(IRecycleBinEnumList **ppEnumList) override;
//     HRESULT STDMETHODCALLTYPE GetDirectory(LPWSTR szPath) override
//     {
//         return E_UNEXPECTED;
//     }
//
// protected:
//     LONG m_ref;
// };
//
// HRESULT STDMETHODCALLTYPE RecycleBinGeneric::QueryInterface(REFIID riid, void **ppvObject)
// {
//     TRACE("(%p, %s, %p)\n", this, debugstr_guid(&riid), ppvObject);
//
//     if (!ppvObject)
//         return E_POINTER;
//
//     if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_IRecycleBin))
//         *ppvObject = static_cast<IRecycleBin *>(this);
//     else
//     {
//         *ppvObject = NULL;
//         return E_NOINTERFACE;
//     }
//
//     AddRef();
//     return S_OK;
// }
//
// STDMETHODIMP_(ULONG) RecycleBinGeneric::AddRef()
// {
//     ULONG refCount = InterlockedIncrement(&m_ref);
//     TRACE("(%p)\n", this);
//     return refCount;
// }
//
// RecycleBinGeneric::~RecycleBinGeneric()
// {
//     TRACE("(%p)\n", this);
// }
//
// STDMETHODIMP_(ULONG) RecycleBinGeneric::Release()
// {
//     TRACE("(%p)\n", this);
//
//     ULONG refCount = InterlockedDecrement(&m_ref);
//     if (refCount == 0)
//         delete this;
//     return refCount;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBinGeneric::DeleteFile(LPCWSTR szFileName)
// {
//     TRACE("(%p, %s)\n", this, debugstr_w(szFileName));
//
//     /* Get full file name */
//     LPWSTR szFullName = NULL;
//     DWORD dwBufferLength = 0;
//     while (TRUE)
//     {
//         DWORD len = GetFullPathNameW(szFileName, dwBufferLength, szFullName, NULL);
//         if (len == 0)
//         {
//             if (szFullName)
//                 CoTaskMemFree(szFullName);
//             return HRESULT_FROM_WIN32(GetLastError());
//         }
//         else if (len < dwBufferLength)
//             break;
//         if (szFullName)
//             CoTaskMemFree(szFullName);
//         dwBufferLength = len;
//         szFullName = (LPWSTR)CoTaskMemAlloc(dwBufferLength * sizeof(WCHAR));
//         if (!szFullName)
//             return HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY);
//     }
//
//     /* Get associated volume path */
//     WCHAR szVolume[MAX_PATH];
//     swprintf(szVolume, L"%c:\\", szFullName[0]);
//
//     /* Skip namespace (if any): "\\.\" or "\\?\" */
//     if (szVolume[0] == '\\' &&
//         szVolume[1] == '\\' &&
//         (szVolume[2] == '.' || szVolume[2] == '?') &&
//         szVolume[3] == '\\')
//     {
//         MoveMemory(szVolume, &szVolume[4], (_countof(szVolume) - 4) * sizeof(WCHAR));
//     }
//
//     IRecycleBin *prb;
//     HRESULT hr = GetDefaultRecycleBin(szVolume, &prb);
//     if (!SUCCEEDED(hr))
//     {
//         CoTaskMemFree(szFullName);
//         return hr;
//     }
//
//     hr = prb->DeleteFile(szFullName);
//     CoTaskMemFree(szFullName);
//     prb->Release();
//     return hr;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBinGeneric::EmptyRecycleBin()
// {
//     TRACE("(%p)\n", this);
//
//     DWORD dwLogicalDrives = GetLogicalDrives();
//     if (dwLogicalDrives == 0)
//         return HRESULT_FROM_WIN32(GetLastError());
//
//     for (DWORD i = 0; i < 'Z' - 'A' + 1; i++)
//     {
//         if (!(dwLogicalDrives & (1 << i)))
//             continue;
//
//         WCHAR szVolumeName[MAX_PATH];
//         swprintf(szVolumeName, L"%c:\\", L'A' + i);
//         if (GetDriveTypeW(szVolumeName) != DRIVE_FIXED)
//             continue;
//
//         IRecycleBin *prb;
//         HRESULT hr = GetDefaultRecycleBin(szVolumeName, &prb);
//         if (!SUCCEEDED(hr))
//             return hr;
//
//         hr = prb->EmptyRecycleBin();
//         prb->Release();
//     }
//
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBinGeneric::EnumObjects(IRecycleBinEnumList **ppEnumList)
// {
//     TRACE("(%p, %p)\n", this, ppEnumList);
//     return RecycleBinGenericEnum_Constructor(ppEnumList);
// }
//
// RecycleBinGeneric::RecycleBinGeneric()
//     : m_ref(1)
// {
// }
//
// extern "C"
// HRESULT RecycleBinGeneric_Constructor( IUnknown **ppUnknown)
// {
//     /* This RecycleBin implementation was introduced to be able to manage all
//      * drives at once, and instanciate the 'real' implementations when needed */
//     RecycleBinGeneric *pThis = new RecycleBinGeneric();
//     if (!pThis)
//         return E_OUTOFMEMORY;
//
//     *ppUnknown = static_cast<IRecycleBin *>(pThis);
//     return S_OK;
// }
//
// extern "C"
// BOOL RecycleBinGeneric_IsEqualFileIdentity(const RECYCLEBINFILEIDENTITY *p1, const RECYCLEBINFILEIDENTITY *p2)
// {
//     return p1->DeletionTime.dwLowDateTime == p2->DeletionTime.dwLowDateTime &&
//            p1->DeletionTime.dwHighDateTime == p2->DeletionTime.dwHighDateTime &&
//            _wcsicmp(p1->RecycledFullPath, p2->RecycledFullPath) == 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
