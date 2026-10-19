use crate::prelude::*;
use core::ffi::c_void;













#[no_mangle]
pub unsafe extern "C" fn IsEqual__0(scid: *const SHCOLUMNID, guid: REFGUID, pid: UINT) -> bool {
    return (*scid).pid == pid && IsEqualGUID!(core::ptr::addr_of!((*scid).fmtid), guid) != false;
}
#[no_mangle]
pub unsafe extern "C" fn IsEqual__1(scid: &SHCOLUMNID, col: &FolderViewColumn) -> bool {
    return !col.pkg.is_null() && IsEqual__0(scid, &*col.pkg, col.pki as UINT);
}

impl FolderViewColumn {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CZipFolder {
    pub unsafe fn GetDefaultSearchGUID(&mut self, pguid: *mut GUID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn EnumSearches(&mut self, ppenum: *mut *mut IEnumExtraSearch,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn GetDefaultColumn(&mut self, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
            if !pSort.is_null() {
                *pSort = (COL_NAME as u32);
            }
            if !pDisplay.is_null() {
                *pDisplay = (COL_NAME as u32);
            }
            return S_OK;
        }
    pub unsafe fn EnumObjects(&mut self, hwndOwner: HWND, dwFlags: DWORD, ppEnumIDList: *mut LPENUMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        // IID_IEnumIDList = {000214F2-0000-0000-C000-000000000046}
        let IID_IEnumIDList_local: IID = core::mem::transmute::<[u8; 16], IID>([
            0xF2, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
        ]);
        return _CEnumZipContents_CreateInstance(
            core::ptr::addr_of_mut!((*this_).__base8),
            dwFlags,
            (*this_).m_ZipDir.GetString(),
            &IID_IEnumIDList_local,
            (ppEnumIDList as *mut core::ffi::c_void).cast(),
        );
    }
    pub unsafe fn BindToStorage(&mut self, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        UNIMPLEMENTED!();
        return E_NOTIMPL;
    }
    pub unsafe fn SetNameOf(&mut self, hwndOwner: HWND,
        pidl: PCUITEMID_CHILD,
        lpName: LPCOLESTR,
        dwFlags: DWORD,
        pPidlOut: *mut PITEMID_CHILD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn GetCurFolder(&mut self, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        *pidl = ILClone((*this_).m_CurDir.as_ptr());
        return S_OK;
    }
    pub unsafe fn GetClassID(&mut self, lpClassId: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        *lpClassId = CLSID_ZipFolderStorageHandler;
        return S_OK;
    }
}
