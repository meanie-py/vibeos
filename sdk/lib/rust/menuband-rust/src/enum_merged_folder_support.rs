//! Hand-written substrate for `CEnumMergedFolder` — the one thing a worker body cannot own.
//!
//! CMergedFolder.cpp:66 declares `static int CALLBACK s_DsaDeleteCallback(void *pItem, void
//! *pData);` — a C++ STATIC member used as a raw callback pointer by `DSA_DestroyCallback` and
//! `DSA_EnumCallback`. Two things make it un-expressible in a translated body:
//!
//!   * `PFNDSAENUMCALLBACK` needs a concrete ABI, `unsafe extern "system" fn(*mut c_void,
//!     *mut c_void) -> c_int`, and the worker audit refuses any ABI declaration in a body —
//!     the scaffold owns ABI artifacts, which is what stops a translated body inventing a
//!     calling convention.
//!   * the translated `CEnumMergedFolder_s_DsaDeleteCallback_body` has a different SHAPE:
//!     three parameters, because every worker body takes an explicit `this` receiver. The
//!     C++ static has two, and DSA passes exactly two.
//!
//! So this file is the adapter. `pData` is the object pointer the C++ handed to
//! `DSA_DestroyCallback(m_hDsa, s_DsaDeleteCallback, this)`, which is where the receiver comes
//! from — the translated body derives it from `pData` internally anyway, so passing it in both
//! positions is consistent rather than redundant.

use crate::enum_merged_folder::{CEnumMergedFolder, CEnumMergedFolder_s_DsaDeleteCallback_body};

/// CMergedFolder.cpp:66 `static int CALLBACK s_DsaDeleteCallback(void *pItem, void *pData)`.
///
/// `CALLBACK` is `__stdcall`, which is `extern "system"` on every arch this tree targets.
pub unsafe extern "system" fn s_DsaDeleteCallback(
    pItem: *mut core::ffi::c_void,
    pData: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    CEnumMergedFolder_s_DsaDeleteCallback_body(pData as *mut CEnumMergedFolder, pItem, pData)
}

// ---------------------------------------------------------------------------------------------
// The six `CEnumMergedFolder_*_RustBridge` entry points.
//
// `merged_folder.rs` has been calling these since CMergedFolder was translated, while
// CEnumMergedFolder was still C++ and CMergedFolder.cpp defined them. The class is Rust now, so
// the .cpp leaves CMake and these move here under the SAME NAMES and the SAME ABI -- the caller
// is untouched by the flip and still declares them `extern "system"` (`WINAPI`).
//
// Five are plain forwards to the translated body. The creator is not.
// ---------------------------------------------------------------------------------------------

use crate::enum_merged_folder::{
    CEnumMergedFolder_Begin_body, CEnumMergedFolder_CreateInstance,
    CEnumMergedFolder_FindByName_body, CEnumMergedFolder_FindPidlInList_body,
    CEnumMergedFolder_SetSources_body, IID_IEnumIDList, LocalPidlInfo,
};
use crate::prelude::*;

/// CMergedFolder.cpp:829 — hands back a RAW object at **refcount 0**.
///
/// This is the one bridge that is not a forward, and the reason is a refcount contract the two
/// creators do not share. The C++ was `new CComObject<CEnumMergedFolder>()`, and a freshly
/// `new`-ed CComObject starts at **zero**: `CMergedFolder_cpp_com_assign_m_EnumSource` is what
/// AddRefs it to 1, leaving the object owned solely by `m_EnumSource`. The generated
/// `CEnumMergedFolder_CreateInstance` follows the COM activation contract instead -- allocate at
/// 1, QueryInterface to 2, Release back to 1 -- so handing its result straight to the same
/// assign would leave the enumerator at 2 and leak it for the life of the process.
///
/// So: create, then drop the activation reference by decrementing the counter DIRECTLY. Calling
/// `Release` would be wrong in the other direction -- it would see the count reach 0 and run the
/// destructor on the object we are about to return.
///
/// The `iface as *mut CEnumMergedFolder` cast is the same offset-0 assumption the C++ bridge
/// checked at runtime, and here it is structural rather than assumed: the generated struct
/// declares `IEnumIDList_iface` as its first field.
#[no_mangle]
pub unsafe extern "system" fn CEnumMergedFolder_CreateInstance_RustBridge() -> *mut CEnumMergedFolder
{
    let mut iface: *mut core::ffi::c_void = core::ptr::null_mut();
    let hr = CEnumMergedFolder_CreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IEnumIDList),
        &mut iface,
    );
    if hr < 0 || iface.is_null() {
        return core::ptr::null_mut();
    }
    let obj = iface as *mut CEnumMergedFolder;
    (*obj)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
    obj
}

#[no_mangle]
pub unsafe extern "system" fn CEnumMergedFolder_SetSources_RustBridge(
    obj: *mut CEnumMergedFolder,
    userLocal: *mut IShellFolder,
    allUsers: *mut IShellFolder,
) -> HRESULT {
    CEnumMergedFolder_SetSources_body(obj, userLocal, allUsers)
}

#[no_mangle]
pub unsafe extern "system" fn CEnumMergedFolder_Begin_RustBridge(
    obj: *mut CEnumMergedFolder,
    hwndOwner: HWND,
    flags: SHCONTF,
) -> HRESULT {
    CEnumMergedFolder_Begin_body(obj, hwndOwner, flags)
}

#[no_mangle]
pub unsafe extern "system" fn CEnumMergedFolder_FindByName_RustBridge(
    obj: *mut CEnumMergedFolder,
    hwndOwner: HWND,
    strParsingName: LPCWSTR,
    pinfo: *mut LocalPidlInfo,
) -> HRESULT {
    CEnumMergedFolder_FindByName_body(obj, hwndOwner, strParsingName, pinfo)
}

#[no_mangle]
pub unsafe extern "system" fn CEnumMergedFolder_FindPidlInList_RustBridge(
    obj: *mut CEnumMergedFolder,
    hwndOwner: HWND,
    pcidl: LPCITEMIDLIST,
    pinfo: *mut LocalPidlInfo,
) -> HRESULT {
    CEnumMergedFolder_FindPidlInList_body(obj, hwndOwner, pcidl, pinfo)
}

#[no_mangle]
pub unsafe extern "system" fn CEnumMergedFolder_QueryInterface_RustBridge(
    obj: *mut CEnumMergedFolder,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // Through the VTABLE, not the scaffold's `CEnumMergedFolder_QueryInterface`: that one is
    // module-private by design (the scaffold owns it and publishes it only as a vtable slot),
    // and the C++ this replaces was `obj->QueryInterface(riid, ppv)` -- itself a virtual call
    // landing on that exact slot. So this is the same dispatch, not a workaround for it.
    let iface = core::ptr::addr_of_mut!((*obj).IEnumIDList_iface);
    ((*(*iface).lpVtbl).QueryInterface)(iface, riid, ppv)
}
