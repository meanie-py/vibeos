use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope constants, types, or tables in this preamble. The only declaration is `cmd_cls`, which is a function and is translated separately.
#[no_mangle]
pub unsafe extern "C" fn cmd_cls(param: LPTSTR) -> INT {
    static SLASH_QUESTION: [u16; 3] = ['/' as u16, '?' as u16, 0];
    if _tcsncmp(param, SLASH_QUESTION.as_ptr(), 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_CLS_HELP);
        return 0;
    }

    ConClearScreen(core::ptr::addr_of_mut!(StdOutScreen));
    return 0;
}
