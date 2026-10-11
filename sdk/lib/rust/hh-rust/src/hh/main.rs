use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The only file-scope declaration is a function-pointer typedef. Since it is a typedef of a function type (not a struct), it maps to a type alias for an `extern "system"` function pointer, using the available `HMODULE` and `LPSTR` imports.

pub type DOWINMAIN = unsafe extern "system" fn(hMod: HMODULE, cmdline: LPSTR) -> i32;

#[no_mangle]
pub unsafe extern "system" fn WinMain(
    hInst: HINSTANCE,
    hPrevInst: HINSTANCE,
    cmdline: LPSTR,
    cmdshow: i32,
) -> i32 {
    let mut hModule: HMODULE;
    // main.c: `DOWINMAIN` is a function TYPE, so `DOWINMAIN *` is the function pointer; the
    // Rust `DOWINMAIN` already is a fn pointer, so the local is an Option of it, not a pointer
    // to one (the translation called the first bytes of doWinMain's code as an address).
    let mut doWinMain: Option<DOWINMAIN>;
    let mut ret: i32 = -1;
    hModule = LoadLibraryA(b"hhctrl.ocx\0".as_ptr() as *const i8);
    if !hModule.is_null() {
        doWinMain = core::mem::transmute::<FARPROC, Option<DOWINMAIN>>(
            GetProcAddress(hModule, b"doWinMain\0".as_ptr() as *const i8),
        );
        if let Some(f) = doWinMain {
            ret = f(hInst, cmdline);
        }

        FreeLibrary(hModule);
    }

    return ret;
}
