use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations: beep.c's preamble contains only the `cmd_beep` function, which is translated separately.
#[no_mangle]
pub unsafe extern "C" fn cmd_beep(param: LPTSTR) -> INT {
    if _tcsncmp(param, _T("/?"), 2) == 0
    {
        ConOutResPaging(TRUE, STRING_BEEP_HELP);
        return 0;
    }

    ConRingBell(ConStreamGetOSHandle(StdOut));
    return 0;
}
