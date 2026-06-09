use ntdll_sys::nt::*;
use core::ffi::{c_void, c_char};
use crate::types::*;

// ---- file-scope type preamble (translated) ----
pub type PSESSION = *mut core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DeleteExtractedFiles(
    ps: *mut core::ffi::c_void,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn GetDllVersion() -> LPCSTR {
    /* This is what it returns on Windows XP SP3 */
    return b"5.1.2600.5512\0".as_ptr() as *const i8;
}

