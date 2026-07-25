//! Rust-owned activation shim for the Start menu coclass.
//!
//! `CStartMenuDummy` has no object or vtable of its own. Its ATL creator only
//! enforces the standard out-pointer/aggregation contract before delegating to
//! the shared shellmenu implementation in `RSHELL_CStartMenu_CreateInstance`.

use crate::shellcom::REFIID;
use crate::types::{HRESULT, LPVOID, CLASS_E_NOAGGREGATION, E_POINTER};

extern "system" {
    fn RSHELL_CStartMenu_CreateInstance(riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn CStartMenuDummy_CreateInstance(
    outer: LPVOID,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    RSHELL_CStartMenu_CreateInstance(riid, ppv)
}
