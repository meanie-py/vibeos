use crate::prelude::*;
use crate::shellrecyclebin::RecycleBin5Enum::RecycleBin5Enum;
use crate::shellrecyclebin::RecycleBin5Enum::RecycleBin5Enum_Init_body;
use crate::shellrecyclebin::RecycleBin5Enum::RecycleBin5Enum_delete;
use crate::shellrecyclebin::RecycleBin5Enum::RecycleBin5Enum_new;
use crate::shellrecyclebin::RecycleBin5File::RecycleBin5File_Init_body;
use crate::shellrecyclebin::RecycleBin5File::RecycleBin5File_delete;
use crate::shellrecyclebin::RecycleBin5File::RecycleBin5File_new;
// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

// NOTE: no local `RecycleBin5File`/`RecycleBin5Enum` preamble struct redeclarations here --
// they would be distinct nominal types from the real ones in RecycleBin5File.rs/RecycleBin5Enum.rs
// (which RecycleBin5File_new()/RecycleBin5Enum_new() below actually return), which is exactly
// what made `pThis` fail to typecheck against RecycleBin5Enum_delete originally.

// recyclebin_v5_enumerator.cpp:268-293 -- file-static constructor, not exported in the C++
// either (no EXTERN_C). Kept private; `Init` was renamed `..._Init_body` in the class-atomic
// split, so the call below is retargeted to that, not deleted/stubbed.
#[allow(dead_code)]
unsafe fn RecycleBin5File_Constructor(
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

    let hr = RecycleBin5File_Init_body(pThis, prb, Folder, pDeletedFile);
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

    let hr: HRESULT = RecycleBin5Enum_Init_body(pThis, prb, hInfo, hInfoMapped, szPrefix);
    if hr < 0 {
        RecycleBin5Enum_delete(pThis);
        return hr;
    }

    *ppUnknown = pThis as *mut IUnknown;
    S_OK
}
