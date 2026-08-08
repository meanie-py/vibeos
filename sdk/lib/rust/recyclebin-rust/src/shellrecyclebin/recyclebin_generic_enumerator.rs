// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct RecycleBinGenericEnum {
    pub lpVtbl: *const core::ffi::c_void,
    pub m_ref: LONG,
    pub m_current: *mut IRecycleBinEnumList,
    pub m_dwLogicalDrives: DWORD,
    pub m_skip: SIZE_T,
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBinGenericEnum_QueryInterface(
    this: *mut RecycleBinGenericEnum,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }
    if IsEqualIID(riid, IID_IUnknown) != 0 || IsEqualIID(riid, IID_IRecycleBinEnumList) != 0 {
        *ppvObject = this as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }
    AddRef();
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGenericEnum_AddRef(this: *mut RecycleBinGenericEnum) -> u32 {
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*this).m_ref));
    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGenericEnum_dtor(this: *mut RecycleBinGenericEnum) {
    if !(*this).m_current.is_null() {
        IRecycleBinEnumList_Release((*this).m_current);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGenericEnum_Release(this: *mut RecycleBinGenericEnum) -> ULONG {
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*this).m_ref));
    if refCount == 0 {
        drop(Box::from_raw(this));
    }
    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBinGenericEnum_Next(
    this: *mut RecycleBinGenericEnum,
    celt: DWORD,
    rgelt: *mut *mut IRecycleBinFile,
    pceltFetched: *mut DWORD,
) -> HRESULT {
    if rgelt.is_null() {
        return E_POINTER;
    }
    if pceltFetched.is_null() && celt > 1 {
        return E_INVALIDARG;
    }

    let mut hr: HRESULT;
    let mut fetched: DWORD = 0;
    loop {
        if (*this).m_current.is_null() && (*this).m_dwLogicalDrives != 0 {
            let bounds = ('Z' as u16 - 'A' as u16) as DWORD + 1;
            let mut i: DWORD = 0;
            while i < bounds {
                if ((*this).m_dwLogicalDrives & (1 << i)) != 0 {
                    let mut szVolumeName: [u16; 4 as usize] =
                        core::mem::MaybeUninit::zeroed().assume_init();
                    szVolumeName[0] = ('A' as u16) + i as u16;
                    szVolumeName[1] = ':' as u16;
                    szVolumeName[2] = '\\' as u16;
                    szVolumeName[3] = UNICODE_NULL;
                    if GetDriveTypeW(szVolumeName.as_ptr()) != DRIVE_FIXED {
                        (*this).m_dwLogicalDrives &= !(1 << i);
                        i += 1;
                        continue;
                    }
                    let mut prb: *mut IRecycleBin = core::ptr::null_mut();
                    hr = GetDefaultRecycleBin(szVolumeName.as_ptr(), &mut prb as *mut *mut IRecycleBin);
                    if !(hr >= 0) {
                        return hr;
                    }
                    hr = (*(*prb).lpVtbl).EnumObjects(prb as *mut core::ffi::c_void, &mut (*this).m_current);
                    (*(*prb).lpVtbl).Release(prb as *mut core::ffi::c_void);
                    if !(hr >= 0) {
                        return hr;
                    }
                    (*this).m_dwLogicalDrives &= !(1 << i);
                    break;
                }
                i += 1;
            }
        }

        if (*this).m_current.is_null() {
            if !pceltFetched.is_null() {
                *pceltFetched = fetched;
            }
            return S_FALSE;
        }

        while (*this).m_skip > 0 {
            let mut rbf: *mut IRecycleBinFile = core::ptr::null_mut();
            hr = (*(*(*this).m_current).lpVtbl).Next(
                (*this).m_current,
                1,
                &mut rbf,
                core::ptr::null_mut(),
            );
            if hr == S_OK {
                (*(*rbf).lpVtbl).Release(rbf as *mut core::ffi::c_void);
            } else if hr == S_FALSE {
                break;
            } else if !(hr >= 0) {
                return hr;
            }
        }

        if (*this).m_skip > 0 {
            continue;
        }

        let mut newFetched: DWORD;
        hr = (*(*(*this).m_current).lpVtbl).Next(
            (*this).m_current,
            celt - fetched,
            rgelt.add(fetched as usize),
            &mut newFetched,
        );
        if hr >= 0 {
            fetched += newFetched;
        }

        if hr == S_FALSE || newFetched == 0 {
            (*(*(*this).m_current).lpVtbl).Release((*this).m_current as *mut core::ffi::c_void);
            (*this).m_current = core::ptr::null_mut();
        } else if !(hr >= 0) {
            return hr;
        }

        if fetched == celt {
            if !pceltFetched.is_null() {
                *pceltFetched = fetched;
            }
            return S_OK;
        }
    }
    return UNREACHABLE;
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBinGenericEnum_Skip(
    this: *mut RecycleBinGenericEnum,
    celt: u32,
) -> i32 {
    (*this).m_skip += celt as usize;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBinGenericEnum_Reset(this: *mut RecycleBinGenericEnum) -> HRESULT {
    if !((*this).m_current).is_null() {
        IRecycleBinEnumList_Release((*this).m_current);
        (*this).m_current = core::ptr::null_mut();
    }
    (*this).m_skip = 0;
    (*this).m_dwLogicalDrives = GetLogicalDrives();
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGenericEnum_ctor(this: *mut RecycleBinGenericEnum) {
    (*this).m_ref = 1;
    (*this).m_current = core::ptr::null_mut();
    (*this).m_dwLogicalDrives = 0;
    (*this).m_skip = 0;
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGenericEnum_Constructor(
    pprbel: *mut *mut IRecycleBinEnumList,
) -> HRESULT {
    let pThis = RecycleBinGenericEnum();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }
    *pprbel = pThis as *mut IRecycleBinEnumList;
    IRecycleBinEnumList_Reset(*pprbel)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Recycle bin management
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Enumerates contents of all recycle bins
//  * COPYRIGHT:   Copyright 2007 Hervé Poussineau (hpoussin@reactos.org)
//  *              Copyright 2024 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "recyclebin_private.h"
//
// class RecycleBinGenericEnum : public IRecycleBinEnumList
// {
// public:
//     RecycleBinGenericEnum();
//     virtual ~RecycleBinGenericEnum();
//
//     /* IUnknown methods */
//     HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, void **ppvObject) override;
//     STDMETHODIMP_(ULONG) AddRef() override;
//     STDMETHODIMP_(ULONG) Release() override;
//
//     /* IRecycleBinEnumList methods */
//     HRESULT STDMETHODCALLTYPE Next(DWORD celt, IRecycleBinFile **rgelt, DWORD *pceltFetched) override;
//     HRESULT STDMETHODCALLTYPE Skip(DWORD celt) override;
//     HRESULT STDMETHODCALLTYPE Reset() override;
//
// protected:
//     LONG m_ref;
//     IRecycleBinEnumList *m_current;
//     DWORD m_dwLogicalDrives;
//     SIZE_T m_skip;
// };
//
// HRESULT STDMETHODCALLTYPE
// RecycleBinGenericEnum::QueryInterface(REFIID riid, void **ppvObject)
// {
//     TRACE("(%p, %s, %p)\n", this, debugstr_guid(&riid), ppvObject);
//
//     if (!ppvObject)
//         return E_POINTER;
//
//     if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_IRecycleBinEnumList))
//         *ppvObject = static_cast<IRecycleBinEnumList *>(this);
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
// STDMETHODIMP_(ULONG)
// RecycleBinGenericEnum::AddRef()
// {
//     ULONG refCount = InterlockedIncrement(&m_ref);
//     TRACE("(%p)\n", this);
//     return refCount;
// }
//
// RecycleBinGenericEnum::~RecycleBinGenericEnum()
// {
//     TRACE("(%p)\n", this);
//
//     if (m_current)
//         m_current->Release();
// }
//
// STDMETHODIMP_(ULONG)
// RecycleBinGenericEnum::Release()
// {
//     TRACE("(%p)\n", this);
//
//     ULONG refCount = InterlockedDecrement(&m_ref);
//     if (refCount == 0)
//         delete this;
//     return refCount;
// }
//
// HRESULT STDMETHODCALLTYPE
// RecycleBinGenericEnum::Next(DWORD celt, IRecycleBinFile **rgelt, DWORD *pceltFetched)
// {
//     TRACE("(%p, %u, %p, %p)\n", this, celt, rgelt, pceltFetched);
//
//     if (!rgelt)
//         return E_POINTER;
//     if (!pceltFetched && celt > 1)
//         return E_INVALIDARG;
//
//     HRESULT hr;
//     DWORD fetched = 0;
//     while (TRUE)
//     {
//         /* Get enumerator implementation */
//         if (!m_current && m_dwLogicalDrives)
//         {
//             for (DWORD i = 0; i < L'Z' - L'A' + 1; ++i)
//             {
//                 if (m_dwLogicalDrives & (1 << i))
//                 {
//                     WCHAR szVolumeName[4];
//                     szVolumeName[0] = (WCHAR)(L'A' + i);
//                     szVolumeName[1] = L':';
//                     szVolumeName[2] = L'\\';
//                     szVolumeName[3] = UNICODE_NULL;
//                     if (GetDriveTypeW(szVolumeName) != DRIVE_FIXED)
//                     {
//                         m_dwLogicalDrives &= ~(1 << i);
//                         continue;
//                     }
//
//                     IRecycleBin *prb;
//                     hr = GetDefaultRecycleBin(szVolumeName, &prb);
//                     if (!SUCCEEDED(hr))
//                         return hr;
//                     hr = prb->EnumObjects(&m_current);
//                     prb->Release();
//
//                     if (!SUCCEEDED(hr))
//                         return hr;
//
//                     m_dwLogicalDrives &= ~(1 << i);
//                     break;
//                 }
//             }
//         }
//
//         if (!m_current)
//         {
//             /* Nothing more to enumerate */
//             if (pceltFetched)
//                 *pceltFetched = fetched;
//             return S_FALSE;
//         }
//
//         /* Skip some elements */
//         while (m_skip > 0)
//         {
//             IRecycleBinFile *rbf;
//             hr = m_current->Next(1, &rbf, NULL);
//             if (hr == S_OK)
//                 rbf->Release();
//             else if (hr == S_FALSE)
//                 break;
//             else if (!SUCCEEDED(hr))
//                 return hr;
//         }
//
//         if (m_skip > 0)
//             continue;
//
//         /* Fill area */
//         DWORD newFetched;
//         hr = m_current->Next(celt - fetched, &rgelt[fetched], &newFetched);
//         if (SUCCEEDED(hr))
//             fetched += newFetched;
//
//         if (hr == S_FALSE || newFetched == 0)
//         {
//             m_current->Release();
//             m_current = NULL;
//         }
//         else if (!SUCCEEDED(hr))
//             return hr;
//
//         if (fetched == celt)
//         {
//             if (pceltFetched)
//                 *pceltFetched = fetched;
//             return S_OK;
//         }
//     }
//
//     /* Never go here */
//     UNREACHABLE;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBinGenericEnum::Skip(DWORD celt)
// {
//     TRACE("(%p, %u)\n", this, celt);
//     m_skip += celt;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBinGenericEnum::Reset()
// {
//     TRACE("(%p)\n", this);
//
//     if (m_current)
//     {
//         m_current->Release();
//         m_current = NULL;
//     }
//     m_skip = 0;
//     m_dwLogicalDrives = ::GetLogicalDrives();
//     return S_OK;
// }
//
// RecycleBinGenericEnum::RecycleBinGenericEnum()
//     : m_ref(1)
//     , m_current(NULL)
//     , m_dwLogicalDrives(0)
//     , m_skip(0)
// {
// }
//
// extern "C"
// HRESULT
// RecycleBinGenericEnum_Constructor(
//      IRecycleBinEnumList **pprbel)
// {
//     RecycleBinGenericEnum *pThis = new RecycleBinGenericEnum();
//     if (!pThis)
//         return E_OUTOFMEMORY;
//
//     *pprbel = static_cast<IRecycleBinEnumList *>(pThis);
//     return (*pprbel)->Reset();
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
