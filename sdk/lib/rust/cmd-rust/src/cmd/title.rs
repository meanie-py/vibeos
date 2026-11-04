use crate::cmd::console::ConOutResPaging;
use crate::cmd::console::ConSetTitle;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The C declares this `extern`; one definition lives in cmd.rs (scan_extern_copy).
use crate::cmd::cmd::bTitleSet;
#[no_mangle]
pub unsafe extern "C" fn cmd_title(mut param: LPTSTR) -> INT {
    /* Do nothing if no args */
    if *param == '\0' as u16 {
        return 0;
    }

    /* Asking help? */
    let title_help_lit: [u16; 3] = ['/' as u16, '?' as u16, 0];
    if !(_tcsncmp(param, title_help_lit.as_ptr(), 2) != 0) {
        ConOutResPaging((TRUE) as BOOL, STRING_TITLE_HELP);
        return 0;
    }

    /* Set the new console title, and tell CMD to not reset it */
    bTitleSet = (FALSE) as i32;
    return ConSetTitle(param);
}
