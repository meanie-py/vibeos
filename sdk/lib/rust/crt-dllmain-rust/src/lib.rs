//! Default DllMain (startup/dllmain.c), named `vibeos_default_DllMain`:
//! crt-startup-rust's crtdll object maps `DllMain` onto it with lld
//! `-alternatename`, so it is used only when the consumer defines no DllMain
//! (never a duplicate symbol). The default DllEntryPoint is in crt-startup-rust.
//! No panic handler / eh_personality here — this links only alongside
//! crt-startup-rust + the consumer's staticlib, which provide them.
#![no_std]
#![allow(non_snake_case)]

use core::ffi::c_void;

type HANDLE = *mut c_void;
type DWORD = u32;
type LPVOID = *mut c_void;
type WINBOOL = i32;

const TRUE: WINBOOL = 1;
const DLL_PROCESS_ATTACH: DWORD = 1;

#[link(name = "kernel32")]
extern "system" {
    fn DisableThreadLibraryCalls(hLibModule: HANDLE) -> WINBOOL;
}

// startup/dllmain.c
#[no_mangle]
pub unsafe extern "system" fn vibeos_default_DllMain(
    hDllHandle: HANDLE,
    dwReason: DWORD,
    _lpreserved: LPVOID,
) -> WINBOOL {
    if dwReason == DLL_PROCESS_ATTACH {
        DisableThreadLibraryCalls(hDllHandle);
    }
    TRUE
}

// startup/dllentry.c
// The default DllEntryPoint is in crt-startup-rust (crtdll.rs), next to its caller: here it
// dragged this object -- and so this default DllMain -- into every DLL, shadowing a DllMain that
// lives in an archive.

// Required to compile this staticlib standalone; the dup vs crt-startup-rust
// (and the consumer staticlib) is absorbed by --allow-multiple-definition.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
