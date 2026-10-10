use crate::cmd::console::ConInString;
use crate::cmd::console::ConOutResPaging;
use crate::cmd::dir::FormatTime;
use crate::cmd::locale::GetTimeString;
use crate::cmd::misc::freep;
use crate::cmd::misc::split;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations in this preamble: no typedefs, `#define` constants, enums, or static tables. It contains only the functions `ParseTime` and `cmd_time`, which are out of scope for this pass.

unsafe extern "C" fn ParseTime(s: LPTSTR) -> BOOL {
    let mut t: SYSTEMTIME = core::mem::zeroed();
    let mut p: LPTSTR = s;

    if *s == 0 {
        return (TRUE) as BOOL;
    }

    GetLocalTime(core::ptr::addr_of_mut!(t));
    t.wHour = 0;
    t.wMinute = 0;
    t.wSecond = 0;
    t.wMilliseconds = 0;

    // first get hour
    if _istdigit!(*p) != 0 {
        while _istdigit!(*p) != 0 {
            t.wHour = (t.wHour as i32 * 10 + *p as i32 - ('0' as u16) as i32) as WORD;
            p = p.add(1);
        }
    } else {
        return (FALSE) as BOOL;
    }

    // get time separator
    if *p != cTimeSeparator {
        return (FALSE) as BOOL;
    }
    p = p.add(1);

    // now get minutes
    if _istdigit!(*p) != 0 {
        while _istdigit!(*p) != 0 {
            t.wMinute = (t.wMinute as i32 * 10 + *p as i32 - ('0' as u16) as i32) as WORD;
            p = p.add(1);
        }
    } else {
        return (FALSE) as BOOL;
    }

    // get time separator
    if *p != cTimeSeparator {
        return (FALSE) as BOOL;
    }
    p = p.add(1);

    // now get seconds
    if _istdigit!(*p) != 0 {
        while _istdigit!(*p) != 0 {
            t.wSecond = (t.wSecond as i32 * 10 + *p as i32 - ('0' as u16) as i32) as WORD;
            p = p.add(1);
        }
    } else {
        return (FALSE) as BOOL;
    }

    // get decimal separator
    if *p == cDecimalSeparator {
        p = p.add(1);

        // now get hundreths
        if _istdigit!(*p) != 0 {
            while _istdigit!(*p) != 0 {
                // t.wMilliseconds = t.wMilliseconds * 10 + *p - ('0' as u16);
                p = p.add(1);
            }
            // t.wMilliseconds *= 10;
        }
    }

    /* special case: 12 hour format */
    if nTimeFormat == 0 {
        if _totupper(*s) == ('P' as u16) {
            t.wHour += 12;
        }

        if _totupper(*s) == ('A' as u16) && t.wHour == 12 {
            t.wHour = 0;
        }
    }

    if t.wHour > 23
        || t.wMinute > 60
        || t.wSecond > 60
        || t.wMilliseconds > 999
    {
        return (FALSE) as BOOL;
    }

    SetLocalTime(core::ptr::addr_of_mut!(t));

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_time(param: LPTSTR) -> INT {
    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut argc: INT = core::mem::zeroed();
    let mut i: INT;
    let mut nTimeString: INT = -1;
    let mut szTime: [TCHAR; 40] = [0; 40];
    let szp: LPTSTR = core::ptr::addr_of_mut!(szTime) as LPTSTR;

    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_TIME_HELP1);
        return 0;
    }

    nErrorLevel = 0;

    /* Build the parameter array */
    arg = split(param, core::ptr::addr_of_mut!(argc), (FALSE) as BOOL, (FALSE) as BOOL);

    /* Check for options */
    i = 0;
    while i < argc {
        if bEnableExtensions != 0
            && (_tcsicmp(*arg.offset(i as isize), (&[0x002f, 0x0054, 0x0000u16]).as_ptr() /* _T "/T" */) == 0)
        {
            /* Display current time in short format */
            let mut st: SYSTEMTIME = core::mem::zeroed();
            GetLocalTime(core::ptr::addr_of_mut!(st));
            FormatTime(szp, core::ptr::addr_of_mut!(st));
            ConOutPrintf!((&[0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "%s\n" */, szp);
            freep(arg);
            return 0;
        }

        if (*(*arg.offset(i as isize)) != ('/' as u16)) && (nTimeString == -1) {
            nTimeString = i;
        }
        i += 1;
    }

    if nTimeString == -1 {
        ConOutResPuts!(STRING_TIME_NOW);
        ConOutPrintf!((&[0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "%s\n" */, GetTimeString());
    }

    loop {
        if nTimeString == -1 {
            ConOutResPuts!(STRING_TIME_HELP2);
            ConInString(szp, ARRAYSIZE!(szTime));

            while *szp != 0
                && (szTime[(_tcslen(szp) - 1) as usize] < (' ' as u16))
            {
                szTime[(_tcslen(szp) - 1) as usize] = ('\0' as u16);
            }

            if ParseTime(szp) != 0 {
                freep(arg);
                return 0;
            }
        } else {
            if ParseTime(*arg.offset(nTimeString as isize)) != 0 {
                freep(arg);
                return 0;
            }

            /* Force input the next time around */
            nTimeString = -1;
        }

        ConErrResPuts!(STRING_TIME_ERROR);
        nErrorLevel = 1;
    }

    freep(arg);
    return 0;
}
