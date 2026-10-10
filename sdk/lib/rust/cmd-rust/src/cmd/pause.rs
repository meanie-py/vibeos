use crate::cmd::console::ConOutChar;
use crate::cmd::console::ConOutResPaging;
use crate::cmd::misc::cgetchar;
use crate::cmd::error::msg_pause;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations: the preamble contains only the function `cmd_pause`, which is excluded from this pass. No typedefs, `#define` constants, enums, or tables are present.
#[no_mangle]
pub unsafe extern "C" fn cmd_pause(param: LPTSTR) -> INT {
    // TRACE("cmd_pause: '%s')\n", debugstr_aw(param)); omitted: debug-only macro with no callee token
    let slash: [u16; 3] = ['/' as u16, '?' as u16, 0];

    if _tcsncmp(param, _T(slash.as_ptr()), 2) == 0
    {
        ConOutResPaging(TRUE, STRING_PAUSE_HELP1);
        return 0;
    }

    if *param != 0 {
        ConOutPuts(param);
    } else {
        msg_pause();
    }

    cgetchar();

    ConOutChar(_T('\n' as u16));
    return 0;
}
