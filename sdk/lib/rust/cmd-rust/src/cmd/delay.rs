use crate::cmd::console::ConOutResPaging;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope type, constant, enum, or table declarations in this preamble. The only top-level item is `CommandDelay`, which is a function and is excluded from this pass.
#[no_mangle]
pub unsafe extern "C" fn CommandDelay(mut param: LPTSTR) -> INT {
    static SLASH_QUESTION: [u16; 3] = ['/' as u16, '?' as u16, 0];
    static SLASH_M: [u16; 3] = ['/' as u16, 'm' as u16, 0];
    let mut val: DWORD = 0;
    let mut mul: DWORD = 1000;

    if _tcsncmp(param, SLASH_QUESTION.as_ptr(), 2) == 0
    {
        ConOutResPaging((TRUE) as BOOL, STRING_DELAY_HELP);
        return 0;
    }

    nErrorLevel = 0;

    if *param == 0
    {
        error_req_param_missing();
        return 1;
    }

    if _tcsnicmp(param, SLASH_M.as_ptr(), 2) == 0
    {
        mul = 1;
        param = param.add(2 as usize);
    }

    val = _ttoi(param) as DWORD;
    Sleep(val.wrapping_mul(mul));

    return 0;
}
