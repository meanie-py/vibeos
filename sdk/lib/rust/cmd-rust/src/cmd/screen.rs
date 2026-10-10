use crate::cmd::console::ConOutResPaging;
use crate::cmd::console::GetScreenSize;
use crate::cmd::console::SetCursorXY;
use crate::cmd::error::error_req_param_missing;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations: the preamble contains only the `CommandScreen` function (inside `#ifdef INCLUDE_CMD_SCREEN`), and function definitions are excluded from this pass.
#[no_mangle]
pub unsafe extern "C" fn CommandScreen(mut param: LPTSTR) -> INT {
    let mut x: SHORT = 0;
    let mut y: SHORT = 0;
    let mut maxx: SHORT = 0;
    let mut maxy: SHORT = 0;
    let mut bSkipText: BOOL = FALSE;

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_SCREEN_HELP);
        return 0;
    }

    nErrorLevel = 0;

    /* Retrieve the screen dimensions */
    GetScreenSize(core::ptr::addr_of_mut!(maxx), core::ptr::addr_of_mut!(maxy));

    /* Get row */
    while _istspace(*param) != 0 {
        param = param.offset(1);
    }

    if *param == 0 {
        error_req_param_missing();
        return 1;
    }

    y = _ttoi(param) as SHORT;
    if y < 0 || y > (maxy - 1) {
        ConOutResPuts(STRING_SCREEN_ROW);
        return 1;
    }

    /* Get column */
    param = _tcschr(param, _T(' '));
    if param.is_null() {
        error_req_param_missing();
        return 1;
    }

    while _istspace(*param) != 0 {
        param = param.offset(1);
    }

    if *param == 0 {
        error_req_param_missing();
        return 1;
    }

    x = _ttoi(param) as SHORT;
    if x < 0 || x > (maxx - 1) {
        ConErrResPuts(STRING_SCREEN_COL);
        return 1;
    }

    /* Get text */
    param = _tcschr(param, _T(' '));
    if param.is_null() {
        bSkipText = TRUE;
    } else {
        while _istspace(*param) != 0 {
            param = param.offset(1);
        }

        if *param == 0 {
            bSkipText = TRUE;
        }
    }

    bIgnoreEcho = TRUE;

    if bSkipText != 0 {
        x = 0;
    }

    SetCursorXY(x, y);

    if bSkipText == 0 {
        ConOutPuts(param);
    }

    return 0;
}
