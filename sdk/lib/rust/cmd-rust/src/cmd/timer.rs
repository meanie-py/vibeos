use crate::cmd::locale::GetTimeString;
use crate::cmd::misc::freep;
use crate::cmd::misc::split;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NCS_NOT_SPECIFIED: i32 = -1;
pub const NCS_ON: i32 = 1;
pub const NCS_OFF: i32 = 0;
// PT(format), cT, cS are function-like/inline macros over per-function statics; not emitted here.

unsafe extern "C" fn PrintElapsedTime(mut time: DWORD, mut format: INT) {
    let mut h: DWORD = 0;
    let mut m: DWORD = 0;
    let mut s: DWORD = 0;
    let mut ms: DWORD = 0;
    match format {
        0 => {
            ConOutResPrintf!(STRING_TIMER_HELP1, time);
        }
        1 => {
            ms = time % 1000;
            time /= 1000;
            s = time % 60;
            time /= 60;
            m = time % 60;
            h = time / 60;
            ConOutResPrintf!(
                STRING_TIMER_HELP2,
                h,
                cTimeSeparator as ::core::ffi::c_int,
                m,
                cTimeSeparator as ::core::ffi::c_int,
                s,
                cDecimalSeparator as ::core::ffi::c_int,
                ms / 10,
            );
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn CommandTimer(param: LPTSTR) -> INT {
    static mut clksT: [DWORD; 10] = [0; 10];
    static mut clksS: [BOOL; 10] = [
        FALSE as BOOL, FALSE as BOOL, FALSE as BOOL, FALSE as BOOL,
        FALSE as BOOL, FALSE as BOOL, FALSE as BOOL, FALSE as BOOL,
        FALSE as BOOL, FALSE as BOOL,
    ];

    let lit_help: [u16; 3] = ['/' as u16, '?' as u16, 0];
    let lit_on: [u16; 3] = ['o' as u16, 'n' as u16, 0];
    let lit_off: [u16; 4] = ['o' as u16, 'f' as u16, 'f' as u16, 0];
    let lit_ON: [u16; 3] = ['O' as u16, 'N' as u16, 0];
    let lit_OFF: [u16; 4] = ['O' as u16, 'F' as u16, 'F' as u16, 0];
    let lit_fmt: [u16; 4] = ['%' as u16, 's' as u16, '\n' as u16, 0];

    let mut bS: BOOL = FALSE as BOOL;
    let mut bCanNSet: BOOL = TRUE as BOOL;
    let mut NewClkStatus: INT = NCS_NOT_SPECIFIED as INT;
    let mut clk_n: INT = 1 as INT;
    let mut iFormat: INT = 1 as INT;
    let mut argc: INT = 0;
    let mut p: *mut LPTSTR = core::ptr::null_mut();
    let mut i: INT = 0;

    if _tcsncmp(param, core::ptr::addr_of!(lit_help) as LPTSTR, 2 as usize) == 0 as INT
    {
        ConOutResPrintf!(STRING_TIMER_HELP3, cTimeSeparator as core::ffi::c_uint, cTimeSeparator as core::ffi::c_uint, cDecimalSeparator as core::ffi::c_uint);
        return 0 as INT;
    }

    nErrorLevel = 0 as INT;

    p = split(param, core::ptr::addr_of_mut!(argc), FALSE as BOOL, FALSE as BOOL);

    i = 0 as INT;
    while i < argc {
        'body: {
            if (_tcsicmp((*p.offset(i as isize)).offset(0 as isize), core::ptr::addr_of!(lit_on) as LPTSTR) == 0)
                && NewClkStatus == NCS_NOT_SPECIFIED as INT
            {
                NewClkStatus = NCS_ON as INT;
                break 'body;
            }

            if (_tcsicmp((*p.offset(i as isize)).offset(0 as isize), core::ptr::addr_of!(lit_off) as LPTSTR) == 0)
                && NewClkStatus == NCS_NOT_SPECIFIED as INT
            {
                NewClkStatus = NCS_OFF as INT;
                break 'body;
            }

            if *(*p.offset(i as isize)).offset(0 as isize) as INT == '/' as u16 as INT
            {
                if _istdigit!(*(*p.offset(i as isize)).offset(1 as isize)) != 0 && bCanNSet != 0
                {
                    clk_n = (*(*p.offset(i as isize)).offset(1 as isize)) as INT - '0' as u16 as INT;
                    bCanNSet = FALSE as BOOL;
                    break 'body;
                }

                if _totupper(*(*p.offset(i as isize)).offset(1 as isize)) as INT == 'S' as u16 as INT
                {
                    bS = TRUE as BOOL;
                    break 'body;
                }

                if _totupper(*(*p.offset(i as isize)).offset(1 as isize)) as INT == 'F' as u16 as INT
                {
                    iFormat = (*(*p.offset(i as isize)).offset(2 as isize)) as INT - '0' as u16 as INT;
                    break 'body;
                }
            }
        }
        i += 1;
    }

    if NewClkStatus == NCS_ON as INT
    {
        clksT[clk_n as usize] = GetTickCount();
        clksS[clk_n as usize] = TRUE as BOOL;

        ConOutResPrintf!(STRING_TIMER_TIME, clk_n,
            if clksS[clk_n as usize] != 0 { core::ptr::addr_of!(lit_ON) as *const u16 } else { core::ptr::addr_of!(lit_OFF) as *const u16 });
        ConOutPuts!(GetTimeString());
        freep(p);
        return 0 as INT;
    }

    if bS != 0
    {
        if clksS[clk_n as usize] != 0
        {
            ConOutResPrintf!(STRING_TIMER_TIME, clk_n,
                if clksS[clk_n as usize] != 0 { core::ptr::addr_of!(lit_ON) as *const u16 } else { core::ptr::addr_of!(lit_OFF) as *const u16 });
            ConOutPrintf!(core::ptr::addr_of!(lit_fmt) as *const u16, GetTimeString());
            PrintElapsedTime(GetTickCount().wrapping_sub(clksT[clk_n as usize]), iFormat);
            freep(p);
            return 0 as INT;
        }

        clksT[clk_n as usize] = GetTickCount();
        clksS[clk_n as usize] = TRUE as BOOL;
        ConOutResPrintf!(STRING_TIMER_TIME, clk_n,
            if clksS[clk_n as usize] != 0 { core::ptr::addr_of!(lit_ON) as *const u16 } else { core::ptr::addr_of!(lit_OFF) as *const u16 });
        ConOutPuts!(GetTimeString());
        freep(p);
        return 0 as INT;
    }

    if NewClkStatus == NCS_NOT_SPECIFIED as INT
    {
        if clksS[clk_n as usize] != 0
        {
            clksS[clk_n as usize] = FALSE as BOOL;
            ConOutResPrintf!(STRING_TIMER_TIME, clk_n,
                if clksS[clk_n as usize] != 0 { core::ptr::addr_of!(lit_ON) as *const u16 } else { core::ptr::addr_of!(lit_OFF) as *const u16 });
            ConOutPrintf!(core::ptr::addr_of!(lit_fmt) as *const u16, GetTimeString());
            PrintElapsedTime(GetTickCount().wrapping_sub(clksT[clk_n as usize]), iFormat);
            freep(p);
            return 0 as INT;
        }

        clksT[clk_n as usize] = GetTickCount();
        clksS[clk_n as usize] = TRUE as BOOL;
        ConOutResPrintf!(STRING_TIMER_TIME, clk_n,
            if clksS[clk_n as usize] != 0 { core::ptr::addr_of!(lit_ON) as *const u16 } else { core::ptr::addr_of!(lit_OFF) as *const u16 });
        ConOutPuts!(GetTimeString());
        freep(p);
        return 0 as INT;
    }

    if NewClkStatus == NCS_OFF as INT
    {
        if clksS[clk_n as usize] != 0
        {
            clksS[clk_n as usize] = FALSE as BOOL;
            ConOutResPrintf!(STRING_TIMER_TIME, clk_n,
                if clksS[clk_n as usize] != 0 { core::ptr::addr_of!(lit_ON) as *const u16 } else { core::ptr::addr_of!(lit_OFF) as *const u16 });
            ConOutPrintf!(core::ptr::addr_of!(lit_fmt) as *const u16, GetTimeString());
            PrintElapsedTime(GetTickCount().wrapping_sub(clksT[clk_n as usize]), iFormat);
            freep(p);
            return 0 as INT;
        }
        ConOutResPrintf!(STRING_TIMER_TIME, clk_n,
            if clksS[clk_n as usize] != 0 { core::ptr::addr_of!(lit_ON) as *const u16 } else { core::ptr::addr_of!(lit_OFF) as *const u16 });
        ConOutPuts!(GetTimeString());
        freep(p);
        return 0 as INT;
    }

    freep(p);
    return 0 as INT;
}
