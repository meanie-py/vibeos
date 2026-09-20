use crate::prelude::*;
use crate::shellrecyclebin::RecycleBinGenericEnum::RecycleBinGenericEnum_new;
// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct RecycleBinGenericEnum {
    pub lpVtbl: *const IRecycleBinEnumListVtbl,
    pub m_ref: LONG,
    pub m_current: *mut IRecycleBinEnumList,
    pub m_dwLogicalDrives: DWORD,
    pub m_skip: SIZE_T,
}
pub type PRecycleBinGenericEnum = *mut RecycleBinGenericEnum;

// The C FORWARD DECLARATION of this function, which has no Rust equivalent: a prototype in
// the same TU as its definition is not an `extern`, it is nothing. Emitting it as a bodiless
// `fn` made the name defined twice AND produced "free function without a body".

#[no_mangle]
pub unsafe extern "C" fn RecycleBinGenericEnum_Constructor(
    pprbel: *mut *mut IRecycleBinEnumList,
) -> HRESULT {
    let pThis = RecycleBinGenericEnum_new();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }
    *pprbel = pThis as *mut IRecycleBinEnumList;
    IRecycleBinEnumList_Reset(*pprbel)
}
