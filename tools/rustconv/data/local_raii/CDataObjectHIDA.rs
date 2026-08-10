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
