use crate::prelude::*;
use crate::shellrecyclebin::RecycleBin5Enum::RecycleBin5Enum_delete;
use crate::shellrecyclebin::RecycleBin5Enum::RecycleBin5Enum_new;
use crate::shellrecyclebin::RecycleBin5File::RecycleBin5File_delete;
use crate::shellrecyclebin::RecycleBin5File::RecycleBin5File_new;
// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct RecycleBin5File {
    pub lpVtbl: *const _IRecycleBinFileVtbl,
    pub m_ref: i32,
    pub m_recycleBin: *mut IRecycleBin5,
    pub m_deletedFile: DELETED_FILE_RECORD,
    pub m_FullName: *mut u16,
}

#[repr(C)]
pub struct RecycleBin5Enum {
    pub lpVtbl: *const _IRecycleBinEnumListVtbl,
    pub m_ref: i32,
    pub m_recycleBin: *mut IRecycleBin5,
    pub m_hInfo: HANDLE,
    pub m_pInfo: *mut INFO2_HEADER,
    pub m_dwCurrent: u32,
    pub m_pszPrefix: *mut u16,
}
unsafe extern "C" fn RecycleBin5File_Constructor(
    prb: *mut IRecycleBin5,
    Folder: LPCWSTR,
    pDeletedFile: *mut DELETED_FILE_RECORD,
    ppFile: *mut *mut IRecycleBinFile,
) -> HRESULT {
    if ppFile.is_null() {
        return E_POINTER;
    }
    *ppFile = core::ptr::null_mut();

    let pThis = RecycleBin5File_new();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr = RecycleBin5File_Init(pThis, prb, Folder, pDeletedFile);
    if ((hr) < 0) {
        RecycleBin5File_delete(pThis);
        return hr;
    }

    *ppFile = pThis as *mut IRecycleBinFile;
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5Enum_Constructor(
    prb: *mut IRecycleBin5,
    hInfo: HANDLE,
    hInfoMapped: HANDLE,
    szPrefix: LPCWSTR,
    ppUnknown: *mut *mut IUnknown,
) -> HRESULT {
    if ppUnknown.is_null() {
        return E_POINTER;
    }
    *ppUnknown = core::ptr::null_mut();

    let pThis: *mut RecycleBin5Enum = RecycleBin5Enum_new();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr: HRESULT = RecycleBin5Enum_Init(pThis, prb, hInfo, hInfoMapped, szPrefix);
    if hr < 0 {
        RecycleBin5Enum_delete(pThis);
        return hr;
    }

    *ppUnknown = pThis as *mut IUnknown;
    S_OK
}
