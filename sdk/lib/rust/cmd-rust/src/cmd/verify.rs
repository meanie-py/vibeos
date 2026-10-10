use crate::cmd::console::ConOutResPaging;
use crate::cmd::echo::OnOffCommand;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut bVerify: BOOL = (FALSE) as i32;

#[no_mangle]
pub unsafe extern "C" fn cmd_verify(param: LPTSTR) -> INT {
    if _tcsncmp(param, ['/' as u16, '?' as u16, 0u16].as_ptr(), 2) == 0
    {
        ConOutResPaging((TRUE) as BOOL, STRING_VERIFY_HELP1);
        return 0;
    }

    if OnOffCommand(param, core::ptr::addr_of_mut!(bVerify), (STRING_VERIFY_HELP2) as INT) == 0
    {
        ConErrResPuts!(STRING_VERIFY_HELP3);
        nErrorLevel = 1;
        return nErrorLevel;
    }

    nErrorLevel = 0;
    return nErrorLevel;
}
