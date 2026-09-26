use crate::prelude::*;
use crate::shellrecyclebin::RecycleBinGeneric::RecycleBinGeneric_new;
// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct RecycleBinGeneric {
    pub lpVtbl: *const IRecycleBinVtbl,
    pub m_ref: i32,
}
#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_Constructor(ppUnknown: *mut *mut core::ffi::c_void) -> i32 {
    let pThis: *mut core::ffi::c_void = RecycleBinGeneric_new().cast();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }
    *ppUnknown = pThis;
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn RecycleBinGeneric_IsEqualFileIdentity(
    p1: *const RECYCLEBINFILEIDENTITY,
    p2: *const RECYCLEBINFILEIDENTITY,
) -> i32 {
    ((*p1).DeletionTime.dwLowDateTime == (*p2).DeletionTime.dwLowDateTime
        && (*p1).DeletionTime.dwHighDateTime == (*p2).DeletionTime.dwHighDateTime
        && _wcsicmp((*p1).RecycledFullPath, (*p2).RecycledFullPath) == 0) as i32
}
