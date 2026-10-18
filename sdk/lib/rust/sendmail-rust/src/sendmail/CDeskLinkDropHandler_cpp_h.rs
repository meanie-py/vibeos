use crate::prelude::*;
use crate::sendmail::sendmail_cpp_h::CreateShellLink;
use crate::sendmail::sendmail_cpp_h::g_ModuleRefCnt;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_02D2E571821E: [WCHAR; 5] = [0x002E, 0x007A, 0x0069, 0x0070, 0x0000];
pub static CPP_COM_WSTR_C186F67FBF2C: [WCHAR; 5] = [0x002E, 0x0075, 0x0072, 0x006C, 0x0000];
pub static CPP_COM_WSTR_FE706C396A0C: [WCHAR; 5] = [0x002E, 0x006C, 0x006E, 0x006B, 0x0000];

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

extern "system" {
    fn CoTaskMemFree(pv: *mut core::ffi::c_void);
}

// ---- file-scope preamble (translated) ----
pub static mut __wine_dbch_sendmail: __wine_debug_channel = __wine_debug_channel {
    flags: 0xff,
    name: [115, 101, 110, 100, 109, 97, 105, 108, 0, 0, 0, 0, 0, 0, 0], // "sendmail\0" zero-padded to 15
};

pub static mut __wine_dbch___default: *const __wine_debug_channel =
    core::ptr::addr_of!(__wine_dbch_sendmail);



















impl CDeskLinkDropHandler {
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
        *pdwEffect &= DROPEFFECT_LINK;

        return S_OK;
    }
    pub unsafe fn DragOver(&mut self, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        *pdwEffect &= DROPEFFECT_LINK;

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
            return E_POINTER;
        }

        let mut szDir = [0u16; (MAX_PATH) as _];
        let mut szDest = [0u16; (MAX_PATH) as _];
        let mut szSrc = [0u16; (MAX_PATH) as _];
        SHGetSpecialFolderPathW(core::ptr::null_mut(), szDir.as_mut_ptr(), (CSIDL_DESKTOPDIRECTORY as i32), (FALSE) as BOOL);

        let mut pDesktop: *mut IShellFolder = core::ptr::null_mut();
        let mut hr: HRESULT = 0;
        let result: HRESULT = 'quit: {
            hr = SHGetDesktopFolder(core::ptr::addr_of_mut!(pDesktop));
            if FAILED_UNEXPECTEDLY!(hr) != false {
                break 'quit hr;
            }

            let mut pida = CDataObjectHIDA::new(pDataObject);
            if FAILED_UNEXPECTEDLY!(pida.hr()) != false {
                break 'quit pida.hr();
            }

            let pidlParent: LPCITEMIDLIST = HIDA_GetPIDLFolder(pida.as_ptr());
            let mut i: UINT = 0;
            while i < (*pida.as_ptr()).cidl {
                let pidlChild: LPCITEMIDLIST = HIDA_GetPIDLItem(pida.as_ptr(), i);

                let mut pidl: *mut ITEMIDLIST = ILCombine(pidlParent, pidlChild);
                if pidl.is_null() {
                    break;
                }

                StringCbCopyW(szDest.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), szDir.as_mut_ptr());
                if SHGetPathFromIDListW(pidl, szSrc.as_mut_ptr()) != 0 {
                    let mut pszSourceExt: LPCWSTR = core::ptr::null();
                    let mut bIsLink: BOOL = 0;

                    pszSourceExt = PathFindExtensionW(szSrc.as_mut_ptr()) as LPCWSTR;
                    bIsLink = ((_wcsicmp(pszSourceExt, CPP_COM_WSTR_FE706C396A0C.as_ptr()) == 0) ||
                               (_wcsicmp(pszSourceExt, CPP_COM_WSTR_C186F67FBF2C.as_ptr()) == 0)) as BOOL;

                    if bIsLink != 0 {
                        PathAppendW(szDest.as_mut_ptr(), PathFindFileNameW(szSrc.as_mut_ptr()));
                    } else {
                        let mut strTitle: CStringW = CStringW::new();
                        atl_format_id!(strTitle, IDS_SHORTCUT, PathFindFileNameW(szSrc.as_mut_ptr()));

                        PathAppendW(szDest.as_mut_ptr(), strTitle.as_ptr());
                        PathRemoveExtensionW(szDest.as_mut_ptr());
                        StringCbCatW(szDest.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), CPP_COM_WSTR_FE706C396A0C.as_ptr());
                    }

                    if PathFileExistsW(szDest.as_mut_ptr()) != 0 {
                        let mut strName: CStringW = CStringW::from_ptr(PathFindFileNameW(szDest.as_mut_ptr()));
                        PathYetAnotherMakeUniqueName(szDest.as_mut_ptr(), szDir.as_mut_ptr(), core::ptr::null_mut(), strName.as_ptr());
                    }

                    if bIsLink != 0 {
                        hr = if CopyFileW(szSrc.as_mut_ptr(), szDest.as_mut_ptr(), (TRUE) as BOOL) != 0 {
                            S_OK
                        } else {
                            E_FAIL
                        };
                    } else if PathIsDirectoryW(szSrc.as_mut_ptr()) != 0 || (_wcsicmp(pszSourceExt, CPP_COM_WSTR_02D2E571821E.as_ptr()) == 0) {
                        hr = CreateShellLink(szDest.as_mut_ptr(), szSrc.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), -1, core::ptr::null_mut());
                    } else {
                        /* Set default working directory for the shortcut */
                        let mut strWorkingDir: CStringW = CStringW::from_ptr(szSrc.as_mut_ptr());
                        PathRemoveFileSpecW(strWorkingDir.GetBuffer(0));

                        hr = CreateShellLink(szDest.as_mut_ptr(), szSrc.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), strWorkingDir.as_ptr(), core::ptr::null_mut(), -1, core::ptr::null_mut());
                    }
                } else {
                    let mut strret: STRRET = core::mem::zeroed();
                    hr = (((*(*pDesktop).lpVtbl).GetDisplayNameOf).unwrap_unchecked())(pDesktop, pidl, SHGDN_INFOLDER, core::ptr::addr_of_mut!(strret));
                    if FAILED_UNEXPECTEDLY!(hr) != false {
                        CoTaskMemFree(pidl as *mut core::ffi::c_void);
                        break;
                    }

                    hr = StrRetToBufW(core::ptr::addr_of_mut!(strret), pidl, szSrc.as_mut_ptr(), _countof!(szSrc));
                    if FAILED_UNEXPECTEDLY!(hr) != false {
                        CoTaskMemFree(pidl as *mut core::ffi::c_void);
                        break;
                    }

                    let mut strTitle: CStringW = CStringW::new();
                    atl_format_id!(strTitle, IDS_SHORTCUT, szSrc.as_mut_ptr());

                    PathAppendW(szDest.as_mut_ptr(), strTitle.as_ptr());
                    PathRemoveExtensionW(szDest.as_mut_ptr());
                    StringCbCatW(szDest.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), CPP_COM_WSTR_FE706C396A0C.as_ptr());

                    hr = CreateShellLink(szDest.as_mut_ptr(), core::ptr::null_mut(), pidl, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), -1, core::ptr::null_mut());
                }

                if FAILED_UNEXPECTEDLY!(hr) != false {
                    CoTaskMemFree(pidl as *mut core::ffi::c_void);
                    break;
                }
                CoTaskMemFree(pidl as *mut core::ffi::c_void);
                i += 1;
            }

            hr
        };

        if !pDesktop.is_null() {
            (((*(*pDesktop).lpVtbl).Release).unwrap_unchecked())(pDesktop);
        }
        result
    }
    pub unsafe fn GetClassID(&mut self, lpClassId: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        if lpClassId.is_null() {
            return E_POINTER;
        }

        *lpClassId = CLSID_DeskLinkDropHandler;

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
