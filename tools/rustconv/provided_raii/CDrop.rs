// Hand-verified Rust lowering of the file-local C++ RAII struct `CDrop`
// (dll/win32/shell32/dialogs/drvdefext.cpp:704). Consumed by
// `rustconv translate --cpp-com --local-raii CDrop=<this file>`: spliced verbatim
// into the assembled module, and its API sealed into each affected worker prompt
// so the model maps C++ usage onto this fixed surface instead of inventing the
// drop semantics.
//
// C++ semantics preserved exactly:
//   ctor      — IDataObject::GetData(CF_HDROP, TYMED_HGLOBAL); hr = result;
//               hDrop = stgm.hGlobal on success, else NULL. stgm is only
//               meaningful when hDrop != NULL.
//   dtor      — if (hDrop) ReleaseStgMedium(&stgm)         → impl Drop
//   GetCount  — DragQueryFileW(hDrop, 0xFFFFFFFF, NULL, 0)  (count query)
struct CDrop {
    hr: HRESULT,
    stgm: STGMEDIUM,
    hDrop: HDROP,
}

impl CDrop {
    unsafe fn new(pDO: *mut IDataObject) -> CDrop {
        let mut this = CDrop {
            hr: 0,
            stgm: core::mem::zeroed(),
            hDrop: core::ptr::null_mut(),
        };
        let mut format = FORMATETC {
            cfFormat: CF_HDROP as CLIPFORMAT,
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        this.hr = ((*(*pDO).lpVtbl).GetData)(pDO, &mut format, &mut this.stgm);
        if this.hr >= 0 {
            this.hDrop = this.stgm.u.hGlobal as HDROP;
        }
        this
    }

    unsafe fn GetCount(&mut self) -> UINT {
        DragQueryFileW(self.hDrop, 0xFFFFFFFF, core::ptr::null_mut(), 0)
    }
}

impl Drop for CDrop {
    fn drop(&mut self) {
        unsafe {
            if !self.hDrop.is_null() {
                ReleaseStgMedium(&mut self.stgm);
            }
        }
    }
}
