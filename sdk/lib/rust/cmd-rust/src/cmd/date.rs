use crate::cmd::console::ConInString;
use crate::cmd::console::ConOutResPaging;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut awMonths: [[WORD; 13]; 2] = [
    [
        0 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        28 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        30 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        30 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        30 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        30 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
    ],
    [
        0 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        29 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        30 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        30 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        30 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
        30 as ::core::ffi::c_int as WORD,
        31 as ::core::ffi::c_int as WORD,
    ],
];

unsafe extern "C" fn PromptDateString() {
    match nDateFormat {
        1 => {
            ConOutResPrintf!(STRING_DATE_HELP2, cDateSeparator as core::ffi::c_uint, cDateSeparator as core::ffi::c_uint);
        }
        2 => {
            ConOutResPrintf!(STRING_DATE_HELP3, cDateSeparator as core::ffi::c_uint, cDateSeparator as core::ffi::c_uint);
        }
        0 | _ => {
            ConOutResPrintf!(STRING_DATE_HELP1, cDateSeparator as core::ffi::c_uint, cDateSeparator as core::ffi::c_uint);
        }
    }
}

unsafe extern "C" fn ReadNumber(s: *mut LPTSTR, lpwValue: LPWORD) -> BOOL {
    if _istdigit!(**s) != 0 {
        while _istdigit!(**s) != 0 {
            *lpwValue = (*lpwValue as ::core::ffi::c_int * 10 as ::core::ffi::c_int
                + **s as ::core::ffi::c_int
                - ('0' as u16) as ::core::ffi::c_int) as WORD;
            *s = (*s).offset(1);
        }
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn ReadSeparator(s: *mut LPTSTR) -> BOOL {
    if **s == '/' as TCHAR || **s == '-' as TCHAR || **s == cDateSeparator {
        *s = (*s).add(1);
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn ParseDate(s: LPTSTR) -> BOOL {
    let mut d: SYSTEMTIME = core::mem::zeroed();
    let mut leap: u8;
    let mut p: LPTSTR = s;

    if *s == 0 {
        return (TRUE) as BOOL;
    }

    GetLocalTime(core::ptr::addr_of_mut!(d));

    d.wYear = 0;
    d.wDay = 0;
    d.wMonth = 0;

    match nDateFormat {
        1 => {
            /* ddmmyy */
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wDay)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wMonth)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wYear)) == 0 {
                return (FALSE) as BOOL;
            }
        }
        2 => {
            /* yymmdd */
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wYear)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wMonth)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wDay)) == 0 {
                return (FALSE) as BOOL;
            }
        }
        0 | _ => {
            /* mmddyy (default) */
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wMonth)) == 0 {
                return (FALSE) as BOOL;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return (FALSE) as i32;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wDay)) == 0 {
                return (FALSE) as i32;
            }
            if ReadSeparator(core::ptr::addr_of_mut!(p)) == 0 {
                return (FALSE) as i32;
            }
            if ReadNumber(core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(d.wYear)) == 0 {
                return (FALSE) as i32;
            }
        }
    }

    /* if only entered two digits: */
    /*   assume 2000's if value less than 80 */
    /*   assume 1900's if value greater or equal 80 */
    if d.wYear <= 99 {
        if d.wYear >= 80 {
            d.wYear = 1900 + d.wYear;
        } else {
            d.wYear = 2000 + d.wYear;
        }
    }

    leap = ((((d.wYear % 4) == 0)
        && ((d.wYear % 100) != 0))
        || ((d.wYear % 400) == 0)) as u8;

    if (d.wMonth >= 1 && d.wMonth <= 12)
        && (d.wDay >= 1 && d.wDay <= awMonths[leap as usize][d.wMonth as usize])
        && (d.wYear >= 1980 && d.wYear <= 2099)
    {
        SetLocalTime(core::ptr::addr_of_mut!(d));
        return (TRUE) as i32;
    }

    return (FALSE) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn cmd_date(param: LPTSTR) -> INT {
    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut argc: INT = 0;
    let mut i: INT = 0;
    let mut bPrompt: BOOL = (TRUE) as i32;
    let mut nDateString: INT = -1;
    let mut szDate: [TCHAR; 40] = [0; 40];

    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_DATE_HELP4);
        return 0;
    }

    nErrorLevel = 0;

    /* Build the parameter array */
    arg = split(param, &mut argc, (FALSE) as BOOL, (FALSE) as BOOL);

    /* Check for options */
    i = 0;
    while i < argc {
        if bEnableExtensions != 0
            && (_tcsicmp(*arg.offset(i as isize), (&[0x002f, 0x0054, 0x0000u16]).as_ptr() /* _T "/T" */) == 0)
        {
            bPrompt = (FALSE) as i32;
        }

        if (*(*arg.offset(i as isize)) != ('/' as u16)) && (nDateString == -1) {
            nDateString = i;
        }
        i += 1;
    }

    if nDateString == -1 {
        ConOutResPuts!(STRING_DATE_NOW);
        ConOutPrintf!((&[0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "%s\n" */, GetDateString());
    }

    if bPrompt == 0 {
        freep(arg);
        return 0;
    }

    loop {
        if nDateString == -1 {
            PromptDateString();
            ConInString(szDate.as_mut_ptr(), ARRAYSIZE!(szDate));

            while szDate[0] != 0
                && (szDate[(_tcslen(szDate.as_mut_ptr()) - 1) as usize] < (' ' as u16))
            {
                szDate[(_tcslen(szDate.as_mut_ptr()) - 1) as usize] = ('\0' as u16);
            }

            if ParseDate(szDate.as_mut_ptr()) != 0 {
                freep(arg);
                return 0;
            }
        } else {
            if ParseDate(*arg.offset(nDateString as isize)) != 0 {
                freep(arg);
                return 0;
            }

            /* Force input the next time around */
            nDateString = -1;
        }

        ConErrResPuts!(STRING_DATE_ERROR);
        nErrorLevel = 1;
    }

    freep(arg);
    return 0;
}
