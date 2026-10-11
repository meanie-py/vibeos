use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The only file-scope declaration here is a function prototype, which the rules exclude (no `fn` definitions, no extern blocks), and it contains no types, constants, or tables to emit. Nothing to output.

#[no_mangle]
pub unsafe extern "system" fn WinMain(
    hInst: HINSTANCE,
    hPrevInst: HINSTANCE,
    cmdline: LPSTR,
    cmdshow: i32,
) -> i32 {
    (RunHTMLApplication(hInst, hPrevInst, cmdline, cmdshow)) as i32
}
