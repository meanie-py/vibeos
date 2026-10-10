use crate::cmd::console::ConOutResPaging;
use crate::cmd::error::error_invalid_switch;
use ntdll_sys::nt::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cmd_shift(param: LPTSTR) -> INT {
    let mut i: INT = 0;

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_SHIFT_HELP);
        return 0;
    }

    nErrorLevel = 0;

    if bc.is_null() {
        /* not in batch - error!! */
        nErrorLevel = 1;
        return 1;
    }

    if _tcsicmp(param, _T("down")) == 0 {
        if (*bc).shiftlevel[0 as usize] != 0 {
            while i <= 9 {
                (*bc).shiftlevel[i as usize] -= 1;
                i += 1;
            }
        }
    } else {
        /* shift up */
        if *param == _T('/') {
            if _istdigit(*param.offset(1 as isize)) == 0 {
                error_invalid_switch(*param.offset(1 as isize));
                return 1;
            }
            i = (*param.offset(1 as isize) as INT) - ('0' as INT);
        }

        while i < 9 {
            (*bc).shiftlevel[i as usize] =
                (*bc).shiftlevel[(i + 1) as usize];
            i += 1;
        }
        (*bc).shiftlevel[i as usize] += 1;
    }

    return 0;
}
