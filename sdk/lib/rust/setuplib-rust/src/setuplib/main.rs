//! base/setup/lib/main.c -- setuplib.dll's entry point. Hand-ported (10 lines of C).
//!
//! It lives in the static library that setuplib.dll, setuplib_unittest and the modrun probe
//! all link; only the DLL references `DllMain`, so the executables drop it.
use crate::prelude::*;
use core::ffi::c_void;

const DLL_PROCESS_ATTACH: ULONG = 1;

extern "system" {
    // Declared in ndk/umfuncs.h; main.c declares it itself, as here.
    fn LdrDisableThreadCalloutsForDll(BaseAddress: *mut c_void) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hDll: HINSTANCE, dwReason: ULONG, _pReserved: PVOID) -> BOOL {
    if dwReason == DLL_PROCESS_ATTACH {
        LdrDisableThreadCalloutsForDll(hDll as *mut c_void);
        crate::setuplib::setuplib::ProcessHeap = RtlGetProcessHeap!();
    }
    TRUE as BOOL
}
