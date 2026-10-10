use crate::cmd::console::ConOutResPaging;
use crate::cmd::error::error_invalid_switch;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cmd_shift(param: *mut TCHAR) -> ::core::ffi::c_int {
    let mut i: INT = 0;

    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_SHIFT_HELP);
        return 0;
    }

    nErrorLevel = 0;

    if bc.is_null() {
        /* not in batch - error!! */
        nErrorLevel = 1;
        return 1;
    }

    if _tcsicmp(param, (&[0x0064, 0x006f, 0x0077, 0x006e, 0x0000u16]).as_ptr() /* _T "down" */) == 0 {
        if (*bc).shiftlevel[0 as usize] != 0 {
            while i <= 9 {
                (*bc).shiftlevel[i as usize] -= 1;
                i += 1;
            }
        }
    } else {
        /* shift up */
        if *param == ('/' as u16) {
            if _istdigit!(*param.offset(1 as isize)) == 0 {
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
