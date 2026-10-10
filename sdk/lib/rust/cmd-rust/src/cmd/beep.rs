use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations: beep.c's preamble contains only the `cmd_beep` function, which is translated separately.
#[no_mangle]
pub unsafe extern "C" fn cmd_beep(param: LPTSTR) -> INT {
    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0
    {
        ConOutResPaging((TRUE) as BOOL, STRING_BEEP_HELP);
        return 0;
    }

    ConRingBell(ConStreamGetOSHandle(StdOut));
    return 0;
}
