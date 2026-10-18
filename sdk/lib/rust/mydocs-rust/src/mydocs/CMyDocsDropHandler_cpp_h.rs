use crate::prelude::*;
use crate::mydocs::mydocs_cpp_h::g_ModuleRefCnt;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_99C4EAFCB663: [WCHAR; 7] = [0x0028, 0x006E, 0x0075, 0x006C, 0x006C, 0x0029, 0x0000];
pub static CPP_COM_WSTR_A03E9EF81E70: [WCHAR; 3] = [0x007C, 0x007C, 0x0000];

// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
//
// THE canonical copy. Pass it as
//   rustconv translate <tu> --free-functions --local-raii CDataObjectHIDA=tools/rustconv/data/local_raii/CDataObjectHIDA.rs
// Extracted from shell32/shldataobject.rs, which is one of the three already-flipped TUs that
// each had to carry an identical copy (the others: shell32/shlfolder.rs, dialogs/item_prop.rs).
// A fourth, COpenWithMenu.cpp, stalled unpaid for want of it — the refusal is `translate`
// declining to guess drop semantics, and the fix was always a file, never a retry.
// `hida_clipformat()` comes from the crate prelude (shellutils.rs:68); this file assumes it.
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
        let hr = (((*(*pDataObject).lpVtbl).GetData).unwrap_unchecked())(
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

    /// Mirrors the C++ public static. It is not only the destructor's body:
    /// dll/shellext/cabview/folder.cpp calls CreateCIDA/DestroyCIDA directly to
    /// hold a CIDA across a scope the guard cannot span, so it must stay callable.
    /// shell32-rust's prelude re-exports it and move_to_menu.rs reaches it through
    /// CDataObjectHIDA_DestroyCIDA_RustBridge — a lowering without it links short.
    unsafe fn DestroyCIDA(pcida: *const CIDA, medium: *mut STGMEDIUM) {
        if !pcida.is_null() {
            GlobalUnlock((*medium).__bindgen_anon_1.hGlobal);
        }
        ReleaseStgMedium(medium);
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
        unsafe { CDataObjectHIDA::DestroyCIDA(self.m_cida, &mut self.m_medium) }
    }
}

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut __wine_dbch_mydocs: __wine_debug_channel = __wine_debug_channel {
    flags: 0xff,
    name: [109, 121, 100, 111, 99, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    core::ptr::addr_of_mut!(__wine_dbch_mydocs);



















impl CMyDocsDropHandler {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            InterlockedIncrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        InterlockedDecrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
    }
    pub unsafe fn DragEnter(&mut self, pDataObject: *mut IDataObject, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        *pdwEffect &= DROPEFFECT_COPY; // Copy only

        return S_OK;
    }
    pub unsafe fn DragOver(&mut self, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        *pdwEffect &= DROPEFFECT_COPY; // Copy only

        return S_OK;
    }
    pub unsafe fn DragLeave(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        // TRACE("(%p)\n", this_);
        S_OK
    }
    pub unsafe fn Drop(&mut self, pDataObject: *mut IDataObject, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if pDataObject.is_null() {
            *pdwEffect = 0;
            (*this_).DragLeave();
            return E_POINTER;
        }

        let mut pDesktop: CComPtr<IShellFolder> = CComPtr::new();
        let mut hr: HRESULT = SHGetDesktopFolder(pDesktop.GetAddressOf());
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        // Retrieve an HIDA (handle of IDA)
        let mut pida = CDataObjectHIDA::new(pDataObject);
        if FAILED_UNEXPECTEDLY!(pida.hr()) {
            *pdwEffect = 0;
            (*this_).DragLeave();
            return pida.hr();
        }

        let mut iItem: UINT = 0;
        let mut cItems: UINT = (*pida.as_ptr()).cidl;

        // get the path of "My Documents"
        let mut szzDir: [WCHAR; (MAX_PATH + 1) as _] = core::mem::zeroed();
        SHGetSpecialFolderPathW(core::ptr::null_mut(), szzDir.as_mut_ptr(), (CSIDL_PERSONAL as i32), (FALSE) as BOOL);
        szzDir[(lstrlenW(szzDir.as_ptr()) + 1) as usize] = 0; // ends with double NULs

        // for all source items
        let mut strSrcList: CStringW = CStringW::new();
        let mut szSrc: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
        let pidlParent: PCIDLIST_ABSOLUTE = HIDA_GetPIDLFolder(pida.as_ptr());
        iItem = 0;
        while iItem < cItems {
            // query source pidl
            let pidlChild: PCITEMID_CHILD = HIDA_GetPIDLItem(pida.as_ptr(), iItem);
            let mut pidl: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::from_raw(ILCombine(pidlParent, pidlChild));

            // can get path?
            szSrc[0 as usize] = 0;
            if SHGetPathFromIDListW(pidl.as_ptr(), szSrc.as_mut_ptr()) == 0 {
                // try to retrieve path from desktop
                let mut strret: STRRET = core::mem::zeroed();
                let psf: *mut IShellFolder = *pDesktop;
                hr = (((*(*psf).lpVtbl).GetDisplayNameOf).unwrap_unchecked())(psf, pidl.as_ptr(), SHGDN_INFOLDER, core::ptr::addr_of_mut!(strret));
                if FAILED_UNEXPECTEDLY!(hr) {
                    break;
                }
                hr = StrRetToBufW(core::ptr::addr_of_mut!(strret), pidl.as_ptr(), szSrc.as_mut_ptr(), _countof!(szSrc));
                if FAILED_UNEXPECTEDLY!(hr) {
                    break;
                }
                if PathFileExistsW(szSrc.as_ptr()) == 0 {
                    break;
                }
            }

            if iItem > 0 {
                strSrcList += 0x007Cu16; // separator is '|'
            }
            strSrcList += szSrc.as_ptr();
            iItem += 1;
        }

        if iItem != cItems {
            // source not found
            let mut strText: CStringW = CStringW::new();
            atl_format_id!(strText, IDS_NOSRCFILEFOUND, if szSrc[0 as usize] != 0 { szSrc.as_ptr() } else { CPP_COM_WSTR_99C4EAFCB663.as_ptr() });
            MessageBoxW(core::ptr::null_mut(), strText.GetString(), core::ptr::null_mut(), MB_ICONERROR);

            *pdwEffect = 0;
            (*this_).DragLeave();
            return E_FAIL;
        }

        strSrcList += CPP_COM_WSTR_A03E9EF81E70.as_ptr(); // double separators

        // lock the buffer
        let pszzSrcList: LPWSTR = strSrcList.GetBuffer(0);

        // convert every separator to a NUL
        let cch: INT = strSrcList.GetLength();
        let mut i: INT = 0;
        while i < cch {
            if *pszzSrcList.add(i as usize) == 0x007Cu16 {
                *pszzSrcList.add(i as usize) = 0x0000u16;
            }
            i += 1;
        }

        // copy them
        let mut fileop: SHFILEOPSTRUCTW = core::mem::zeroed(); // { NULL }
        fileop.wFunc = FO_COPY;
        fileop.pFrom = pszzSrcList as _;
        fileop.pTo = szzDir.as_mut_ptr() as _;
        fileop.fFlags = (FOF_ALLOWUNDO | FOF_FILESONLY | FOF_MULTIDESTFILES | FOF_NOCONFIRMMKDIR) as u16;
        let res: i32 = SHFileOperationW(core::ptr::addr_of_mut!(fileop));
        if res != 0 {
            hr = E_FAIL;
        }

        // unlock buffer
        strSrcList.ReleaseBuffer();

        (*this_).DragLeave();
        return hr;
    }
    pub unsafe fn GetClassID(&mut self, lpClassId: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        if lpClassId.is_null() {
            return E_POINTER;
        }

        *lpClassId = CLSID_MyDocsDropHandler;

        return S_OK;
    }
    pub unsafe fn GetCurFile(&mut self, ppszFileName: *mut LPOLESTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn IsDirty(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn Load(&mut self, pszFileName: LPCOLESTR,
        dwMode: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn Save(&mut self, pszFileName: LPCOLESTR,
        fRemember: BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p)\n", this_);
        E_NOTIMPL
    }
    pub unsafe fn SaveCompleted(&mut self, pszFileName: LPCOLESTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("(%p)\n", this_);
        E_NOTIMPL
    }
}
