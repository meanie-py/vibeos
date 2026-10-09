use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PREADERMODEINFO = PVOID;

#[no_mangle]
pub unsafe extern "system" fn CreatePage(dwInitParam: DWORD, hWndParent: HWND) -> HWND {
    // UNIMPLEMENTED;
    (NULL).cast()
}

#[no_mangle]
pub unsafe extern "system" fn CreateProxyPage(dwUnknown1: DWORD, dwUnknown2: DWORD) -> HWND {
    // UNIMPLEMENTED;
    (NULL).cast()
}

#[no_mangle]
pub unsafe extern "system" fn DoReaderMode(prmi: PREADERMODEINFO) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetProcessDword(dwUnknown1: DWORD, dwUnknown2: DWORD) {
    // UNIMPLEMENTED;
}
