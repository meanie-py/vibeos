use crate::prelude::*;
use core::ffi::c_void;





























































#[no_mangle]
pub unsafe extern "C" fn Range_Constructor(context: *mut ITfContext, anchorStart: DWORD, anchorEnd: DWORD, ppOut: *mut *mut ITfRange) -> HRESULT {
    let This: *mut CRange = CRange__new_in_0(cpp_alloc_CicNoThrow::<CRange>(), context, anchorStart as TfAnchor, anchorEnd as TfAnchor);
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    *ppOut = core::ptr::addr_of_mut!((*This).__base0) as *mut ITfRangeACP as *mut ITfRange;
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn TF_SELECTION_to_TS_SELECTION_ACP(tf: *const TF_SELECTION, tsAcp: *mut TS_SELECTION_ACP) -> HRESULT {
    return CRange::TF_SELECTION_to_TS_SELECTION_ACP(tf, tsAcp);
}

impl CRange {
    pub unsafe fn __ctor_0(&mut self, context: *mut ITfContext, anchorStart: TfAnchor, anchorEnd: TfAnchor) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_context = context;
        (*this_).m_anchorStart = anchorStart;
        (*this_).m_anchorEnd = anchorEnd;
        {
            if !context.is_null() {
                (*context).AddRef();
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_context.is_null() {
            let _ = (*(*this_).m_context).Release();
        }
    }
    pub unsafe fn TF_SELECTION_to_TS_SELECTION_ACP(tf: *const TF_SELECTION, tsAcp: *mut TS_SELECTION_ACP) -> HRESULT {
        if tf.is_null() || tsAcp.is_null() || (*tf).range.is_null() {
            return E_INVALIDARG;
        }
    
        let This: *mut CRange = (*tf).range as *mut CRange;
        (*tsAcp).acpStart = ((*This).m_anchorStart) as i32;
        (*tsAcp).acpEnd = ((*This).m_anchorEnd) as i32;
        (*tsAcp).style.ase = (*tf).style.ase as TsActiveSelEnd;
        (*tsAcp).style.fInterimChar = (*tf).style.fInterimChar;
        return S_OK;
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualGUID!(&(*riid), &(IID_PRIV_CRANGE))
        {
            *ppvObj = this_ as *mut core::ffi::c_void;
            return S_OK; // No AddRef
        }
    
        if IsEqualGUID!(&(*riid), &(IID_IUnknown)) || IsEqualGUID!(&(*riid), &(IID_ITfRange)) || IsEqualGUID!(&(*riid), &(IID_ITfRangeACP)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as *mut core::ffi::c_void;
        }
        else if IsEqualGUID!(&(*riid), &(IID_ITfRangeAnchor)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base1) as *mut core::ffi::c_void;
        }
        else if IsEqualGUID!(&(*riid), &(IID_ITfSource)) {
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base2) as *mut core::ffi::c_void;
        }
        else {
            *ppvObj = core::ptr::null_mut();
        }
    
        if !(*ppvObj).is_null()
        {
            (*this_).AddRef();
            return S_OK;
        }
    
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CRange__delete(this_);
        }
        return ret;
    }
    pub unsafe fn GetText(&mut self, ec: TfEditCookie,
        dwFlags: DWORD,
        pchText: *mut WCHAR,
        cchMax: ULONG,
        pcch: *mut ULONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn SetText(&mut self, ec: TfEditCookie,
        dwFlags: DWORD,
        pchText: *const WCHAR,
        cch: LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn GetFormattedText(&mut self, ec: TfEditCookie,
        ppDataObject: *mut *mut IDataObject,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn GetEmbedded(&mut self, ec: TfEditCookie, rguidService: &GUID, riid: &IID, ppunk: *mut *mut IUnknown) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, ec, rguidService, riid, ppunk);
        return E_NOTIMPL;
    }
    pub unsafe fn InsertEmbedded(&mut self, ec: TfEditCookie,
        dwFlags: DWORD,
        pDataObject: *mut IDataObject,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn ShiftStart(&mut self, ec: TfEditCookie,
        cchReq: LONG,
        pcch: *mut LONG,
        pHalt: *const TF_HALTCOND,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn ShiftEnd(&mut self, ec: TfEditCookie,
        cchReq: LONG,
        pcch: *mut LONG,
        pHalt: *const TF_HALTCOND,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn ShiftStartToRange(&mut self, ec: TfEditCookie,
        pRange: *mut ITfRange,
        aPos: TfAnchor,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn ShiftEndToRange(&mut self, ec: TfEditCookie,
        pRange: *mut ITfRange,
        aPos: TfAnchor,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn ShiftStartRegion(&mut self, ec: TfEditCookie,
        dir: TfShiftDir,
        pfNoRegion: *mut BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn ShiftEndRegion(&mut self, ec: TfEditCookie,
        dir: TfShiftDir,
        pfNoRegion: *mut BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn IsEmpty(&mut self, ec: TfEditCookie, pfEmpty: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_).IsEqualStart(
            ec,
            core::ptr::addr_of_mut!((*this_).__base0) as *mut ITfRange,
            TF_ANCHOR_END,
            pfEmpty,
        );
    }
    pub unsafe fn Collapse(&mut self, ec: TfEditCookie,
        aPos: TfAnchor,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn IsEqualStart(&mut self, ec: TfEditCookie, pWith: *mut ITfRange, aPos: TfAnchor, pfEqual: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_)._IsEqualX(ec, (FALSE) as BOOL, pWith, aPos, pfEqual);
    }
    pub unsafe fn IsEqualEnd(&mut self, ec: TfEditCookie, pWith: *mut ITfRange, aPos: TfAnchor, pfEqual: *mut BOOL) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_)._IsEqualX(ec, (TRUE) as BOOL, pWith, aPos, pfEqual);
    }
    pub unsafe fn CompareStart(&mut self, ec: TfEditCookie,
        pWith: *mut ITfRange,
        aPos: TfAnchor,
        plResult: *mut LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn CompareEnd(&mut self, ec: TfEditCookie,
        pWith: *mut ITfRange,
        aPos: TfAnchor,
        plResult: *mut LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn AdjustForInsert(&mut self, ec: TfEditCookie,
        cchInsert: ULONG,
        pfInsertOk: *mut BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn GetGravity(&mut self, pgStart: *mut TfGravity,
        pgEnd: *mut TfGravity,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn SetGravity(&mut self, ec: TfEditCookie,
        gStart: TfGravity,
        gEnd: TfGravity,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn Clone(&mut self, ppClone: *mut *mut ITfRange) -> HRESULT {
        let this_: *mut Self = self;
        if ppClone.is_null() {
            return E_INVALIDARG;
        }
    
        let pCloned: *mut CRange = (*this_)._Clone();
        if pCloned.is_null() {
            return E_OUTOFMEMORY;
        }
    
        *ppClone = core::ptr::addr_of_mut!((*pCloned).__base0) as *mut ITfRange;
        return S_OK;
    }
    pub unsafe fn GetContext(&mut self, ppContext: *mut *mut ITfContext,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("%p\n", ppContext);
        E_NOTIMPL
    }
    pub unsafe fn GetExtent__0(&mut self, pacpAnchor: *mut LONG,
        pcch: *mut LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn SetExtent__0(&mut self, acpAnchor: LONG,
        cch: LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn GetExtent__1(&mut self, ppStart: *mut *mut IAnchor,
        ppEnd: *mut *mut IAnchor,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn SetExtent__1(&mut self, pAnchorStart: *mut IAnchor,
        pAnchorEnd: *mut IAnchor,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn AdviseSink(&mut self, riid: &IID, punk: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn UnadviseSink(&mut self, dwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("\n");
        E_NOTIMPL
    }
    pub unsafe fn _Clone(&mut self) -> *mut CRange {
        let this_: *mut Self = self;
        let pRange: *mut CRange = CRange__new_in_0(cpp_alloc_CicNoThrow::<CRange>(), (*this_).m_context, (*this_).m_anchorStart, (*this_).m_anchorEnd);
        let _ = (*this_).m_anchorEnd;
        if pRange.is_null() {
            return core::ptr::null_mut();
        }
        return pRange;
    }
    pub unsafe fn _IsEqualX(&mut self, ec: TfEditCookie,
        bEnd: BOOL,
        pWith: *mut ITfRange,
        aPos: TfAnchor,
        pfEqual: *mut BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn _CompareX(&mut self, ec: TfEditCookie,
        bEnd: BOOL,
        pWidth: *mut ITfRange,
        aPos: TfAnchor,
        plResult: *mut LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}
