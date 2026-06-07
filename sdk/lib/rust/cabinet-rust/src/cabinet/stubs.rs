use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
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


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include <windef.h>
// #include <reactos/debug.h>
//
// typedef PVOID PSESSION;
//
// VOID
// WINAPI
// DeleteExtractedFiles(
//     PSESSION ps)
// {
//     UNIMPLEMENTED;
// }
//
// LPCSTR
// WINAPI
// GetDllVersion(void)
// {
//     /* This is what it returns on Windows XP SP3 */
//     return "5.1.2600.5512";
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
