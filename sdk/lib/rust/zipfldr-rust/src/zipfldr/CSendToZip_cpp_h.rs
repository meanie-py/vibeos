use crate::prelude::*;
use core::ffi::c_void;





impl CSendToZip {
    pub unsafe fn DragEnter(&mut self, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pDataObject.assign(pDataObj);

        let mut etc: FORMATETC = FORMATETC {
            cfFormat: (CF_HDROP as u16),
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        (*this_).m_fCanDragDrop = (((*pDataObj).QueryGetData(core::ptr::addr_of_mut!(etc))) >= 0) as BOOL;

        if (*this_).m_fCanDragDrop != 0 {
            *pdwEffect &= DROPEFFECT_COPY;
        } else {
            *pdwEffect = DROPEFFECT_NONE;
        }

        return S_OK;
    }
    pub unsafe fn DragOver(&mut self, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_fCanDragDrop != 0 {
            *pdwEffect &= DROPEFFECT_COPY;
        } else {
            *pdwEffect = DROPEFFECT_NONE;
        }

        return S_OK;
    }
    pub unsafe fn DragLeave(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_fCanDragDrop = (FALSE as i32);
        (*this_).m_pDataObject.Release();
        return S_OK;
    }
    pub unsafe fn Drop(&mut self, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pDataObject.assign(pDataObj);

        if pDataObj.is_null() || (*this_).m_fCanDragDrop == 0 {
            DPRINT1!("Drop failed: %d %d\n", (pDataObj.is_null()) as i32, ((*this_).m_fCanDragDrop == 0) as i32);
            *pdwEffect = 0;
            (*this_).DragLeave();
            return E_FAIL;
        }

        let mut stg: STGMEDIUM = core::mem::zeroed();
        let mut etc: FORMATETC = FORMATETC {
            cfFormat: (CF_HDROP as u16),
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        let mut hr: HRESULT = (*pDataObj).GetData(core::ptr::addr_of_mut!(etc), core::ptr::addr_of_mut!(stg));
        if FAILED_UNEXPECTEDLY!(hr) {
            *pdwEffect = 0;
            (*this_).DragLeave();
            return E_FAIL;
        }

        let hDrop: HDROP = stg.__bindgen_anon_1.hGlobal as HDROP;
        let cItems: UINT = DragQueryFileW(hDrop, (-(1 as i32)) as u32, core::ptr::null_mut(), 0);

        let mut pCreator: *mut CZipCreator = CZipCreator::DoCreate(core::ptr::null_mut(), core::ptr::null_mut());

        let mut iItem: UINT = 0;
        while iItem < cItems {
            let mut szPath: [WCHAR; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];
            DragQueryFileW(hDrop, iItem, szPath.as_mut_ptr(), _countof!(szPath));

            (*pCreator).DoAddItem(szPath.as_ptr());
            iItem += 1;
        }

        ReleaseStgMedium(core::ptr::addr_of_mut!(stg));

        CZipCreator::runThread(pCreator);

        (*this_).DragLeave();
        return hr;
    }
}
