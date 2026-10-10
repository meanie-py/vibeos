use crate::cmd::console::ConOutChar;
use crate::cmd::console::ConOutResPaging;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn OnOffCommand(param: LPTSTR, flag: LPBOOL, message: INT) -> BOOL {
    let mut p2: *mut TCHAR;
    if _tcsnicmp(param, D_OFF, (4usize - 1 as usize)) == 0 {
        p2 = param.add(4usize - 1 as usize);
        while _istspace!(*p2) != 0 {
            p2 = p2.add(1);
        }
        if *p2 == ('\0' as u16) {
            *flag = (FALSE) as i32;
            return (TRUE) as BOOL;
        }
    } else if _tcsnicmp(param, D_ON, (3usize - 1 as usize)) == 0 {
        p2 = param.add(3usize - 1 as usize);
        while _istspace!(*p2) != 0 {
            p2 = p2.add(1);
        }
        if *p2 == ('\0' as u16) {
            *flag = (TRUE) as i32;
            return (TRUE) as BOOL;
        }
    } else if *param == ('\0' as u16) {
        ConOutResPrintf!((message) as u32, if *flag != 0 { D_ON } else { D_OFF });
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CommandEcho(mut param: LPTSTR) -> INT {
    let mut p1: LPTSTR;
    let slash_q: [u16; 3] = ['/' as u16, '?' as u16, 0];

    /* skip all spaces for the check of '/?', 'ON' and 'OFF' */
    p1 = param;
    while _istspace!(*p1 as INT) != 0 {
        p1 = p1.offset(1);
    }

    if _tcsncmp(p1, slash_q.as_ptr(), 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_ECHO_HELP4);
        return 0;
    }

    if OnOffCommand(p1, core::ptr::addr_of_mut!(bEcho), (STRING_ECHO_HELP5) as INT) == 0 {
        /* skip the first character */
        ConOutPuts!(param.offset(1 as isize));
        ConOutChar('\n' as TCHAR);
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn CommandEchos(param: LPTSTR) -> INT {
    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPuts!(STRING_ECHO_HELP1);
        return 0;
    }

    ConOutPrintf!((&[0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T "%s" */, param);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn CommandEchoerr(param: LPTSTR) -> INT {
    if _tcsncmp(param, (['/' as u16, '?' as u16, 0u16].as_ptr()), 2) == 0
    {
        ConOutResPuts!(STRING_ECHO_HELP2);
        return 0;
    }

    ConErrPuts!(param);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn CommandEchoserr(mut param: LPTSTR) -> INT {
    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPuts!(STRING_ECHO_HELP3);
        return 0;
    }

    ConErrPrintf!((&[0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T "%s" */, param);
    return 0;
}
