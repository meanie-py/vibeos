//! FLIPPED: this TU is wired into the crate and shipping; the C++ original is
//! commented out of dll/win32/shell32/CMakeLists.txt.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_1F8E5DD3F506: [WCHAR; 21] = [0x0044, 0x0061, 0x0074, 0x0061, 0x004F, 0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0041, 0x0074, 0x0074, 0x0072, 0x0069, 0x0062, 0x0075, 0x0074, 0x0065, 0x0073, 0x0000];

// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// Hand-verified Rust lowering of the C++ RAII struct `CDataObjectHIDA`
// (sdk/include/reactos/shellutils.h:738). Consumed by
// `rustconv translate --free-functions --local-raii CDataObjectHIDA=<this file>`:
// spliced verbatim into the assembled module, and its API sealed into each
// affected worker prompt so the model maps C++ usage onto this fixed surface
// instead of inventing the drop semantics.
//
// It borrows the HIDA out of an IDataObject WITHOUT copying it, so the medium
// must stay locked for exactly the lifetime of the guard — which is the whole
// reason it is RAII and the whole reason a hand lowering is required.
//
// C++ semantics preserved exactly:
//   ctor        — CreateCIDA(pDataObject, &m_cida, m_medium); m_hr = result
//   dtor        — DestroyCIDA(m_cida, m_medium)                → impl Drop
//   hr()        — the stored ctor HRESULT
//   operator bool()        → is_valid()      (m_cida != nullptr)
//   operator const CIDA*() → as_ptr()        (the implicit conversion at a call
//                                             site such as `f(x, cida)`)
//   operator->()           → (*cida.as_ptr()).field
//
// Two details that look like bugs and are not, so do not "fix" them:
//   * DestroyCIDA calls ReleaseStgMedium unconditionally. On the failure paths
//     CreateCIDA has already set tymed = TYMED_NULL, which makes that call a
//     documented no-op — including after CreateCIDA itself released the medium.
//   * C++ leaves m_medium uninitialized and only assigns pUnkForRelease. Every
//     path assigns tymed before anything reads it (GetData on success,
//     TYMED_NULL otherwise). Zeroing here is strictly safer and changes no path.
struct CDataObjectHIDA {
    m_medium: STGMEDIUM,
    m_cida: *const CIDA,
    m_hr: HRESULT,
}

impl CDataObjectHIDA {
    unsafe fn new(pDataObject: *mut IDataObject) -> CDataObjectHIDA {
        let mut medium: STGMEDIUM = core::mem::zeroed();
        let mut cida: *const CIDA = core::ptr::null();
        let hr = CDataObjectHIDA::CreateCIDA(pDataObject, &mut cida, &mut medium);
        CDataObjectHIDA { m_medium: medium, m_cida: cida, m_hr: hr }
    }

    unsafe fn CreateCIDA(
        pDataObject: *mut IDataObject,
        ppcida: *mut *const CIDA,
        medium: *mut STGMEDIUM,
    ) -> HRESULT {
        *ppcida = core::ptr::null();
        (*medium).pUnkForRelease = core::ptr::null_mut();
        // g_cfHIDA in C++ is a DECLSPEC_SELECTANY cache of the registered format.
        let mut format = FORMATETC {
            cfFormat: hida_clipformat(),
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        let hr = ((*(*pDataObject).lpVtbl).GetData)(
            pDataObject,
            (&mut format as *mut FORMATETC).cast(),
            medium.cast(),
        );
        if hr >= 0 {
            *ppcida = GlobalLock((*medium).__bindgen_anon_1.hGlobal) as *const CIDA;
            if !(*ppcida).is_null() {
                return S_OK;
            }
            ReleaseStgMedium(medium);
            (*medium).tymed = TYMED_NULL;
            return E_UNEXPECTED;
        }
        (*medium).tymed = TYMED_NULL;
        hr
    }

    unsafe fn hr(&self) -> HRESULT {
        self.m_hr
    }

    unsafe fn is_valid(&self) -> bool {
        !self.m_cida.is_null()
    }

    unsafe fn as_ptr(&self) -> *const CIDA {
        self.m_cida
    }
}

impl Drop for CDataObjectHIDA {
    fn drop(&mut self) {
        unsafe {
            if !self.m_cida.is_null() {
                GlobalUnlock(self.m_medium.__bindgen_anon_1.hGlobal);
            }
            ReleaseStgMedium(&mut self.m_medium);
        }
    }
}

use core::ffi::c_void;
use crate::prelude::*;
use crate::shell32::sdk_consts::*;

// ---- file-scope preamble (translated) ----
pub static mut g_DataObjectAttributes: CLIPFORMAT = 0;
pub static dwDefaultAttributeMask: DWORD = SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_STORAGE | SFGAO_CANRENAME |
    SFGAO_CANDELETE | SFGAO_READONLY | SFGAO_STREAM | SFGAO_FOLDER;
#[repr(C)]
pub struct DataObjectAttributes {
    pub dwMask: DWORD,
    pub dwAttributes: DWORD,
    pub cItems: UINT,
}
const _: () = assert!(core::mem::size_of::<DataObjectAttributes>() as SIZE_T == 0xc);
pub static wine_dbg_channel_shell: &[u8] = b"shell\0";
unsafe extern "C" fn _BindToObject(pidl: PCUIDLIST_ABSOLUTE, spFolder: &mut *mut IShellFolder) -> HRESULT {
    SHBindToObject(
        core::ptr::null_mut(),
        pidl,
        core::ptr::addr_of!(IID_IShellFolder), (spFolder as *mut *mut IShellFolder).cast(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn SHGetAttributesFromDataObject(
    pDataObject: *mut IDataObject,
    dwAttributeMask: DWORD,
    pdwAttributes: *mut DWORD,
    pcItems: *mut UINT,
) -> HRESULT {
    let mut dwAttributes: DWORD = 0;
    let mut cItems: UINT = 0;
    let mut hr: HRESULT = S_OK;

    if g_DataObjectAttributes == 0 {
        g_DataObjectAttributes = RegisterClipboardFormatW(CPP_COM_WSTR_1F8E5DD3F506.as_ptr()) as CLIPFORMAT;
    }

    if !pDataObject.is_null() {
        let mut data: DataObjectAttributes = core::mem::zeroed();
        if DataObject_GetData(pDataObject, g_DataObjectAttributes, &mut data as *mut _ as *mut c_void, core::mem::size_of::<DataObjectAttributes>() as SIZE_T) < 0 {
            memset(&mut data as *mut _ as *mut c_void, 0, core::mem::size_of::<DataObjectAttributes>());
        }

        let dwQueryAttributes = dwAttributeMask | dwDefaultAttributeMask;

        if (data.dwMask & dwQueryAttributes) != dwQueryAttributes {
            let hida = CDataObjectHIDA::new(pDataObject);
            let mut spFolder: *mut IShellFolder = core::ptr::null_mut();

            if !FAILED_UNEXPECTEDLY!({ hr = hida.hr(); hr }) &&
               !FAILED_UNEXPECTEDLY!({ hr = _BindToObject(HIDA_GetPIDLFolder(hida.as_ptr()), &mut spFolder); hr })
            {
                let mut apidl = CSimpleArray::<PCUIDLIST_RELATIVE>::new();
                let mut n: UINT = 0;
                while n < (*hida.as_ptr()).cidl {
                    apidl.Add(HIDA_GetPIDLItem(hida.as_ptr(), (n) as SIZE_T));
                    n += 1;
                }

                let mut rgfInOut = dwQueryAttributes;
                hr = ((*(*spFolder).lpVtbl).GetAttributesOf)(
                    spFolder,
                    apidl.GetSize() as u32,
                    apidl.GetData() as *const _,
                    &mut rgfInOut,
                );
                if !FAILED_UNEXPECTEDLY!(hr) {
                    data.dwMask = dwQueryAttributes;
                    data.dwAttributes = rgfInOut & dwQueryAttributes;
                    data.cItems = apidl.GetSize() as u32;

                    let hr2 = DataObject_SetData(pDataObject, g_DataObjectAttributes, &mut data as *mut _ as *mut c_void, core::mem::size_of::<DataObjectAttributes>() as SIZE_T);
                    // Report cache failure but don't fail the function: a free build's
                    // FAILED_UNEXPECTEDLY only traces.
                    let _ = hr2;
                }
            }

            if !spFolder.is_null() {
                ((*(*spFolder).lpVtbl).Release)(spFolder);
            }
        }

        dwAttributes = data.dwAttributes & dwAttributeMask;
        cItems = data.cItems;
    }

    if !pdwAttributes.is_null() {
        *pdwAttributes = dwAttributes;
    }
    if !pcItems.is_null() {
        *pcItems = cItems;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn DataObject_GetHIDACount(pdo: *mut IDataObject) -> HRESULT {
    if pdo.is_null() {
        return E_INVALIDARG;
    }
    let cida = CDataObjectHIDA::new(pdo);
    let hr = cida.hr();
    if hr >= 0 {
        (*(cida.as_ptr())).cidl as HRESULT
    } else {
        hr
    }
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_CIDA_ILCloneFull(pCIDA: *const CIDA, Index: u32) -> PIDLIST_ABSOLUTE {
    if Index < (*pCIDA).cidl {
        return ILCombine(HIDA_GetPIDLFolder(pCIDA), HIDA_GetPIDLItem(pCIDA, (Index) as SIZE_T));
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn SHELL_DataObject_ILCloneFullItem(
    pDO: *mut IDataObject,
    Index: u32,
) -> PIDLIST_ABSOLUTE {
    if pDO.is_null() {
        return core::ptr::null_mut();
    }
    let cida = CDataObjectHIDA::new(pDO);
    if (cida.hr()) >= 0 {
        SHELL_CIDA_ILCloneFull(cida.as_ptr(), Index)
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_DataObject_ILCloneFullItem_RustBridge(
    pDO: *mut IDataObject,
    Index: UINT,
) -> PIDLIST_ABSOLUTE {
    SHELL_DataObject_ILCloneFullItem(pDO, Index)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_CIDA_ILCloneFull_RustBridge(
    pCIDA: *const CIDA,
    Index: UINT,
) -> PIDLIST_ABSOLUTE {
    SHELL_CIDA_ILCloneFull(pCIDA, Index)
}

#[no_mangle]
pub unsafe extern "system" fn CDataObjectHIDA_CreateCIDA_RustBridge(
    pDataObject: *mut IDataObject,
    ppcida: *mut *const CIDA,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    CDataObjectHIDA::CreateCIDA(pDataObject, ppcida, pmedium)
}

#[no_mangle]
pub unsafe extern "system" fn CDataObjectHIDA_DestroyCIDA_RustBridge(
    pcida: *mut CIDA,
    pmedium: *mut STGMEDIUM,
) {
    DestroyCIDA(pcida, &mut *pmedium);
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_CloneDataObject(pDO: *mut IDataObject, ppDO: *mut *mut IDataObject) -> HRESULT {
    *ppDO = core::ptr::null_mut();
    let cida = CDataObjectHIDA::new(pDO);
    let mut hr = cida.hr();
    if hr >= 0 {
        let items = CCidaChildArrayHelper::new(cida.as_ptr());
        hr = items.hr();
        if hr < 0 {
            return hr;
        }
        hr = SHCreateFileDataObject(
            HIDA_GetPIDLFolder(cida.as_ptr()),
            (*cida.as_ptr()).cidl,
            items.GetItems() as *const *const _,
            core::ptr::null_mut(),
            ppDO.cast(),
        );
        if hr >= 0 {
            let mut pt: POINT = core::mem::zeroed();
            if DataObject_GetOffset(pDO, &mut pt) >= 0 {
                DataObject_SetOffset(*ppDO, &mut pt);
            }
        }
    }
    hr
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     SHGetAttributesFromDataObject implementation
//  * COPYRIGHT:   Copyright 2021 Mark Jansen <mark.jansen@reactos.org>
//  */
//
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
//
// static CLIPFORMAT g_DataObjectAttributes = 0;
// static const DWORD dwDefaultAttributeMask = SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_STORAGE | SFGAO_CANRENAME |
//                                             SFGAO_CANDELETE | SFGAO_READONLY | SFGAO_STREAM | SFGAO_FOLDER;
//
// struct DataObjectAttributes
// {
//     DWORD dwMask;
//     DWORD dwAttributes;
//     UINT cItems;
// };
//
// static_assert(sizeof(DataObjectAttributes) == 0xc, "Unexpected struct size!");
//
//
// static
// HRESULT _BindToObject(PCUIDLIST_ABSOLUTE pidl, CComPtr<IShellFolder>& spFolder)
// {
//     return SHBindToObject(NULL, pidl, IID_PPV_ARG(IShellFolder, &spFolder));
// }
//
// extern "C"
// HRESULT WINAPI SHGetAttributesFromDataObject(IDataObject* pDataObject, DWORD dwAttributeMask, DWORD* pdwAttributes, UINT* pcItems)
// {
//     DWORD dwAttributes = 0;
//     DWORD cItems = 0;
//     HRESULT hr = S_OK;
//
//     TRACE("(%p, 0x%x, %p, %p)\n", pDataObject, dwAttributeMask, pdwAttributes, pcItems);
//
//     if (!g_DataObjectAttributes)
//         g_DataObjectAttributes = (CLIPFORMAT)RegisterClipboardFormatW(L"DataObjectAttributes");
//
//     if (pDataObject)
//     {
//         DataObjectAttributes data = {};
//         if (FAILED(DataObject_GetData(pDataObject, g_DataObjectAttributes, &data, sizeof(data))))
//         {
//             TRACE("No attributes yet, creating new\n");
//             memset(&data, 0, sizeof(data));
//         }
//
//         DWORD dwQueryAttributes = dwAttributeMask | dwDefaultAttributeMask;
//
//         if ((data.dwMask & dwQueryAttributes) != dwQueryAttributes)
//         {
//             CDataObjectHIDA hida(pDataObject);
//             CComPtr<IShellFolder> spFolder;
//
//             if (!FAILED_UNEXPECTEDLY(hr = hida.hr()) &&
//                 !FAILED_UNEXPECTEDLY(hr = _BindToObject(HIDA_GetPIDLFolder(hida), spFolder)))
//             {
//                 CSimpleArray<PCUIDLIST_RELATIVE> apidl;
//                 for (UINT n = 0; n < hida->cidl; ++n)
//                 {
//                     apidl.Add(HIDA_GetPIDLItem(hida, n));
//                 }
//
//                 SFGAOF rgfInOut = dwQueryAttributes;
//                 hr = spFolder->GetAttributesOf(apidl.GetSize() as u32, apidl.GetData(), &rgfInOut);
//                 if (!FAILED_UNEXPECTEDLY(hr))
//                 {
//                     data.dwMask = dwQueryAttributes;
//                     // Only store what we asked for
//                     data.dwAttributes = rgfInOut & dwQueryAttributes;
//                     data.cItems = apidl.GetSize() as u32;
//
//                     HRESULT hr2;
//                     hr2 = DataObject_SetData(pDataObject, g_DataObjectAttributes, &data, sizeof(data));
//                     FAILED_UNEXPECTEDLY(hr2); // Report cache failure but don't fail the function
//                 }
//             }
//         }
//
//         // Only give the user what they asked for, not everything else we have!
//         dwAttributes = data.dwAttributes & dwAttributeMask;
//         cItems = data.cItems;
//     }
//
//     if (pdwAttributes)
//         *pdwAttributes = dwAttributes;
//
//     if (pcItems)
//         *pcItems = cItems;
//
//     return hr;
// }
//
// HRESULT DataObject_GetHIDACount(IDataObject *pdo)
// {
//     if (!pdo)
//         return E_INVALIDARG;
//     CDataObjectHIDA cida(pdo);
//     HRESULT hr = cida.hr();
//     return SUCCEEDED(hr) ? cida->cidl : hr;
// }
//
// PIDLIST_ABSOLUTE SHELL_CIDA_ILCloneFull( const CIDA *pCIDA,  UINT Index)
// {
//     if (Index < pCIDA->cidl)
//         return ILCombine(HIDA_GetPIDLFolder(pCIDA), HIDA_GetPIDLItem(pCIDA, Index));
//     return NULL;
// }
//
// PIDLIST_ABSOLUTE SHELL_DataObject_ILCloneFullItem( IDataObject *pDO,  UINT Index)
// {
//     if (!pDO)
//         return NULL;
//     CDataObjectHIDA cida(pDO);
//     return SUCCEEDED(cida.hr()) ? SHELL_CIDA_ILCloneFull(cida, Index) : NULL;
// }
//
// /* Rust bridge for the Rust-owned CDesktopFolder (shell32-rust desktop_folder). */
// extern "C" PIDLIST_ABSOLUTE WINAPI
// SHELL_DataObject_ILCloneFullItem_RustBridge(IDataObject *pDO, UINT Index)
// {
//     return SHELL_DataObject_ILCloneFullItem(pDO, Index);
// }
//
// /* Rust bridges for the Rust-owned CCopyToMenu/CMoveToMenu (shell32-rust
//  * copy_to_menu/move_to_menu). The Rust side rebuilds CDataObjectHIDA
//  * (shellutils.h) as a module-local RAII struct over the Create/Destroy pair. */
// extern "C" PIDLIST_ABSOLUTE WINAPI
// SHELL_CIDA_ILCloneFull_RustBridge(const CIDA *pCIDA, UINT Index)
// {
//     return SHELL_CIDA_ILCloneFull(pCIDA, Index);
// }
//
// extern "C" HRESULT WINAPI
// CDataObjectHIDA_CreateCIDA_RustBridge(IDataObject *pDataObject, CIDA **ppcida, STGMEDIUM *pmedium)
// {
//     return CDataObjectHIDA::CreateCIDA(pDataObject, ppcida, *pmedium);
// }
//
// extern "C" void WINAPI
// CDataObjectHIDA_DestroyCIDA_RustBridge(CIDA *pcida, STGMEDIUM *pmedium)
// {
//     CDataObjectHIDA::DestroyCIDA(pcida, *pmedium);
// }
//
// HRESULT SHELL_CloneDataObject( IDataObject *pDO,  IDataObject **ppDO)
// {
//     *ppDO = NULL;
//     CDataObjectHIDA cida(pDO);
//     HRESULT hr = cida.hr();
//     if (SUCCEEDED(hr))
//     {
//         CCidaChildArrayHelper items(cida);
//         if (FAILED(hr = items.hr()))
//             return hr;
//         hr = SHCreateFileDataObject(HIDA_GetPIDLFolder(cida), cida->cidl, items.GetItems(), NULL, ppDO);
//         if (SUCCEEDED(hr))
//         {
//             POINT pt;
//             if (SUCCEEDED(DataObject_GetOffset(pDO, &pt)))
//                 DataObject_SetOffset(*ppDO, &pt);
//         }
//     }
//     return hr;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
