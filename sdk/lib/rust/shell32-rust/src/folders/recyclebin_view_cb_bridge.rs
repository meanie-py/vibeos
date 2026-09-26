//! The two `_RustBridge` entry points `rust_recyclebin_bridges_stub.cpp` used to carry for
//! `CRecycleBinFolderViewCB`, now that the class itself is Rust
//! (`folders/CRecycleBinFolderViewCB.rs`).
//!
//! WHY THIS FILE EXISTS AT ALL, rather than `recycle_bin.rs` calling the two functions
//! directly: `recycle_bin.rs` is generated, and its call sites and `extern "system"`
//! declarations are banked translation output. Publishing the same two C-ABI symbols from
//! Rust makes the caller's side a no-op change — it keeps linking against the names it was
//! translated against — and leaves the bank untouched. The hop costs one un-inlinable call
//! per view creation, which happens once when the Recycle Bin folder is opened.
//!
//! WHY IT IS HAND-WRITTEN and not part of the generated module: a re-run of rustconv
//! overwrites `folders/CRecycleBinFolderViewCB.rs` wholesale, and these exports are not
//! translated from anything — the C++ they replace was glue this project wrote, not
//! ReactOS source.
//!
//! The C++ stub's `psf` parameter on the create path was `UNREFERENCED_PARAMETER`: ATL's
//! `ShellObjectCreator(sfviewcb)` takes no arguments beyond the out-pointer, and the folder
//! is handed over later by `Initialize`. That is preserved exactly — taking `psf` into
//! account here would be a behaviour change, not a tidy-up.

use crate::prelude::*;
// `IShellView` comes from the CLASS module, not the island: bindgen renders shobjidl's
// interfaces as C++ classes with a `vtable_` field, so the translated TU declares its own
// `lpVtbl`-shaped copy and `Initialize_body`'s signature is written against THAT one. Two
// aliases of the same layout are still two Rust types. It costs nothing to agree with the
// callee here, and the nominal split stops at the `extern` boundary anyway — `recycle_bin.rs`
// declares these two symbols against the island's spelling, and a pointer is a pointer once
// it is on the C ABI. That is the whole reason the bridge can exist.
use crate::recycle_bin_folder_view_cb::{
    CRecycleBinFolderViewCB, CRecycleBinFolderViewCB_CreateInstance,
    CRecycleBinFolderViewCB_Initialize_body, IShellFolderViewCB, IShellView,
    IID_IShellFolderViewCB,
};

/// `ShellObjectCreator(sfviewcb)` — create the callback and hand the caller the reference.
///
/// The C++ did `ShellObjectCreator` into a `CComPtr` and then `Detach()`, i.e. the caller
/// receives ONE reference and is responsible for releasing it. `CRecycleBinFolderViewCB_CreateInstance`
/// has the same net effect: it constructs at a refcount of 1, `QueryInterface` takes the
/// second, and its own `Release` drops back to the single reference now owned by `ppvcb`.
#[no_mangle]
pub unsafe extern "system" fn CRecycleBinFolderViewCB_CreateInstance_RustBridge(
    psf: *mut IShellFolder2,
    ppvcb: *mut *mut IShellFolderViewCB,
) -> HRESULT {
    let _ = psf;
    if ppvcb.is_null() {
        return E_POINTER;
    }
    CRecycleBinFolderViewCB_CreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IShellFolderViewCB),
        ppvcb as *mut *mut core::ffi::c_void,
    )
}

/// `pViewCB->Initialize(psf, psv, pidl)`.
///
/// The C++ reached the object with `static_cast<CRecycleBinFolderViewCB *>(pvcb)`, which is
/// the compiler applying the interface subobject's offset. Rust has no such cast, so the
/// offset is spelled out — and it must be spelled rather than assumed, even though
/// `IShellFolderViewCB_iface` is the first field today: the day another interface is added
/// ahead of it, an assumed zero silently reads the wrong fields instead of failing.
///
/// Returns `S_OK` unconditionally, as the C++ stub did — `Initialize` is `void`.
#[no_mangle]
pub unsafe extern "system" fn CRecycleBinFolderViewCB_Initialize_RustBridge(
    pvcb: *mut IShellFolderViewCB,
    psf: *mut IShellFolder2,
    psv: *mut IShellView,
    pidl: LPCITEMIDLIST,
) -> HRESULT {
    if pvcb.is_null() {
        return E_POINTER;
    }
    let this = (pvcb as *mut u8)
        .sub(core::mem::offset_of!(CRecycleBinFolderViewCB, IShellFolderViewCB_iface))
        as *mut CRecycleBinFolderViewCB;
    CRecycleBinFolderViewCB_Initialize_body(this, psf, psv, pidl);
    S_OK
}
