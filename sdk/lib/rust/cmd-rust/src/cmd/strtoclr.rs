use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_12271F972959: [u16; 4] = [0x0063u16, 0x0079, 0x0061, 0x0000];
pub static RUSTCONV_C2R_WSTR_18F5B062B23A: [u16; 3] = [0x0031u16, 0x0032, 0x0000];
pub static RUSTCONV_C2R_WSTR_1BA6083A05A2: [u16; 4] = [0x0079u16, 0x0065, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_250866022CDA: [u16; 3] = [0x0031u16, 0x0035, 0x0000];
pub static RUSTCONV_C2R_WSTR_371DC1DCF2F8: [u16; 4] = [0x0067u16, 0x0072, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_40D95A7C7F16: [u16; 2] = [0x0033u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_41C3A1B1872A: [u16; 4] = [0x0062u16, 0x006C, 0x0061, 0x0000];
pub static RUSTCONV_C2R_WSTR_4F8320D91E97: [u16; 2] = [0x0030u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_50CB49CB79DA: [u16; 4] = [0x0077u16, 0x0068, 0x0069, 0x0000];
pub static RUSTCONV_C2R_WSTR_60864AAE2645: [u16; 2] = [0x0031u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_672F89FB6759: [u16; 4] = [0x006Du16, 0x0061, 0x0067, 0x0000];
pub static RUSTCONV_C2R_WSTR_6855B5C2B40B: [u16; 2] = [0x0036u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_6FEA016A651B: [u16; 2] = [0x0038u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_70C8D1CE4DC3: [u16; 4] = [0x0072u16, 0x0065, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_73E7BE80B2D6: [u16; 4] = [0x0061u16, 0x0071, 0x0075, 0x0000];
pub static RUSTCONV_C2R_WSTR_77F906B94309: [u16; 2] = [0x0034u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_794CF95296C5: [u16; 4] = [0x0067u16, 0x0072, 0x0061, 0x0000];
pub static RUSTCONV_C2R_WSTR_7DCDBEF09744: [u16; 3] = [0x0031u16, 0x0033, 0x0000];
pub static RUSTCONV_C2R_WSTR_8432386E2C3B: [u16; 2] = [0x0035u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_85DD751867E3: [u16; 2] = [0x0032u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_C2R_WSTR_BE632811422A: [u16; 4] = [0x0062u16, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_C2R_WSTR_CBFF72B7063A: [u16; 3] = [0x0031u16, 0x0030, 0x0000];
pub static RUSTCONV_C2R_WSTR_CF6BA8A7C368: [u16; 3] = [0x0031u16, 0x0031, 0x0000];
pub static RUSTCONV_C2R_WSTR_D3605F0EA976: [u16; 4] = [0x0070u16, 0x0075, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_E2733B3DB9D9: [u16; 2] = [0x0037u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_E3F69F6C094F: [u16; 2] = [0x0039u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_FAE838146965: [u16; 3] = [0x0031u16, 0x0034, 0x0000];

#[no_mangle]
pub static mut clrtable: [CLRTABLE; 28] = unsafe {
    [
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_41C3A1B1872A.as_ptr() as LPTSTR,
            val: 0 as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_BE632811422A.as_ptr() as LPTSTR,
            val: _B as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_371DC1DCF2F8.as_ptr() as LPTSTR,
            val: _G as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_12271F972959.as_ptr() as LPTSTR,
            val: (_B | _G) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_70C8D1CE4DC3.as_ptr() as LPTSTR,
            val: _R as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_672F89FB6759.as_ptr() as LPTSTR,
            val: (_B | _R) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_1BA6083A05A2.as_ptr() as LPTSTR,
            val: (_R | _G) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_50CB49CB79DA.as_ptr() as LPTSTR,
            val: (_R | _G | _B) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_794CF95296C5.as_ptr() as LPTSTR,
            val: _I as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_4F8320D91E97.as_ptr() as LPTSTR,
            val: 0 as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_85DD751867E3.as_ptr() as LPTSTR,
            val: _G as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_40D95A7C7F16.as_ptr() as LPTSTR,
            val: (_B | _G) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_77F906B94309.as_ptr() as LPTSTR,
            val: _R as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_8432386E2C3B.as_ptr() as LPTSTR,
            val: (_B | _R) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_6855B5C2B40B.as_ptr() as LPTSTR,
            val: (_R | _G) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_E2733B3DB9D9.as_ptr() as LPTSTR,
            val: (_R | _G | _B) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_6FEA016A651B.as_ptr() as LPTSTR,
            val: _I as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_E3F69F6C094F.as_ptr() as LPTSTR,
            val: (_I | _B) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_CBFF72B7063A.as_ptr() as LPTSTR,
            val: (_I | _G) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_CF6BA8A7C368.as_ptr() as LPTSTR,
            val: (_I | _B | _G) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_18F5B062B23A.as_ptr() as LPTSTR,
            val: (_I | _R) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_7DCDBEF09744.as_ptr() as LPTSTR,
            val: (_I | _B | _R) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_FAE838146965.as_ptr() as LPTSTR,
            val: (_I | _R | _G) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_250866022CDA.as_ptr() as LPTSTR,
            val: (_I | _R | _G | _B) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_60864AAE2645.as_ptr() as LPTSTR,
            val: _B as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_73E7BE80B2D6.as_ptr() as LPTSTR,
            val: (_B | _G) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_D3605F0EA976.as_ptr() as LPTSTR,
            val: (_B | _R) as WORD,
        },
        _CLRTABLE {
            name: RUSTCONV_C2R_WSTR_96A296D224F2.as_ptr()
                as LPTSTR,
            val: 0 as WORD,
        },
    ]
};


pub const _B: WORD = (FOREGROUND_BLUE as u16);
pub const _G: WORD = (FOREGROUND_GREEN as u16);
pub const _R: WORD = (FOREGROUND_RED as u16);
pub const _I: WORD = (FOREGROUND_INTENSITY as u16);

pub const CP_OK: i32 = 0;
pub const CP_BLANK_NOT_FOUND: i32 = 1;
pub const CP_END_OF_STRING: i32 = 2;

pub const SC_HEX: i32 = 0x0100;
pub const SC_TXT: i32 = 0x0200;

#[repr(C)]
pub struct _CLRTABLE {
    pub name: LPTSTR,
    pub val: WORD,
}
pub type CLRTABLE = _CLRTABLE;
pub type PCLRTABLE = *mut _CLRTABLE;

pub static mut CLRSTR_0: [TCHAR; 4] = [b'b' as TCHAR, b'l' as TCHAR, b'a' as TCHAR, 0];
pub static mut CLRSTR_1: [TCHAR; 4] = [b'b' as TCHAR, b'l' as TCHAR, b'u' as TCHAR, 0];
pub static mut CLRSTR_2: [TCHAR; 4] = [b'g' as TCHAR, b'r' as TCHAR, b'e' as TCHAR, 0];
pub static mut CLRSTR_3: [TCHAR; 4] = [b'c' as TCHAR, b'y' as TCHAR, b'a' as TCHAR, 0];
pub static mut CLRSTR_4: [TCHAR; 4] = [b'r' as TCHAR, b'e' as TCHAR, b'd' as TCHAR, 0];
pub static mut CLRSTR_5: [TCHAR; 4] = [b'm' as TCHAR, b'a' as TCHAR, b'g' as TCHAR, 0];
pub static mut CLRSTR_6: [TCHAR; 4] = [b'y' as TCHAR, b'e' as TCHAR, b'l' as TCHAR, 0];
pub static mut CLRSTR_7: [TCHAR; 4] = [b'w' as TCHAR, b'h' as TCHAR, b'i' as TCHAR, 0];
pub static mut CLRSTR_8: [TCHAR; 4] = [b'g' as TCHAR, b'r' as TCHAR, b'a' as TCHAR, 0];
pub static mut CLRSTR_9: [TCHAR; 2] = [b'0' as TCHAR, 0];
pub static mut CLRSTR_10: [TCHAR; 2] = [b'2' as TCHAR, 0];
pub static mut CLRSTR_11: [TCHAR; 2] = [b'3' as TCHAR, 0];
pub static mut CLRSTR_12: [TCHAR; 2] = [b'4' as TCHAR, 0];
pub static mut CLRSTR_13: [TCHAR; 2] = [b'5' as TCHAR, 0];
pub static mut CLRSTR_14: [TCHAR; 2] = [b'6' as TCHAR, 0];
pub static mut CLRSTR_15: [TCHAR; 2] = [b'7' as TCHAR, 0];
pub static mut CLRSTR_16: [TCHAR; 2] = [b'8' as TCHAR, 0];
pub static mut CLRSTR_17: [TCHAR; 2] = [b'9' as TCHAR, 0];
pub static mut CLRSTR_18: [TCHAR; 3] = [b'1' as TCHAR, b'0' as TCHAR, 0];
pub static mut CLRSTR_19: [TCHAR; 3] = [b'1' as TCHAR, b'1' as TCHAR, 0];
pub static mut CLRSTR_20: [TCHAR; 3] = [b'1' as TCHAR, b'2' as TCHAR, 0];
pub static mut CLRSTR_21: [TCHAR; 3] = [b'1' as TCHAR, b'3' as TCHAR, 0];
pub static mut CLRSTR_22: [TCHAR; 3] = [b'1' as TCHAR, b'4' as TCHAR, 0];
pub static mut CLRSTR_23: [TCHAR; 3] = [b'1' as TCHAR, b'5' as TCHAR, 0];
pub static mut CLRSTR_24: [TCHAR; 2] = [b'1' as TCHAR, 0];
pub static mut CLRSTR_25: [TCHAR; 4] = [b'a' as TCHAR, b'q' as TCHAR, b'u' as TCHAR, 0];
pub static mut CLRSTR_26: [TCHAR; 4] = [b'p' as TCHAR, b'u' as TCHAR, b'r' as TCHAR, 0];
pub static mut CLRSTR_27: [TCHAR; 1] = [0];

unsafe extern "C" fn chop_blank(arg_str: *mut LPTSTR) -> INT {
    let mut str: LPTSTR = core::ptr::null_mut();
    str = _tcschr(*arg_str, (' ' as u16));
    if str.is_null() {
        str = _tcschr(*arg_str, ('\0' as u16));
        if !str.is_null() {
            *arg_str = str;
        }
        return CP_BLANK_NOT_FOUND;
    }

    while _istspace!(*str) != 0 {
        str = str.offset(1);
    }

    if *str == ('\0' as u16) {
        *arg_str = str;
        return CP_END_OF_STRING;
    }

    *arg_str = str;

    return CP_OK;
}

unsafe extern "C" fn hex_clr(str: LPTSTR) -> WORD {
    let mut ret: WORD = (-(1 as i32)) as WORD;
    let mut ch: TCHAR;

    ch = *str.offset(1 as isize);

    if _istdigit!(ch as i32) != 0 {
        ret = ((ch as i32) - (('0' as u16) as i32)) as WORD;
    } else {
        ch = _totupper((ch as i32) as u16) as TCHAR;

        if ch >= ('A' as u16) as TCHAR && ch <= ('F' as u16) as TCHAR {
            ret = ((ch as i32) - (('A' as u16) as i32) + 10) as WORD;
        } else {
            return (-(1 as i32)) as WORD;
        }
    }

    ch = *str.offset(0 as isize);

    if _istdigit!(ch as i32) != 0 {
        ret = (ret as i32
            | (((ch as i32) - (('0' as u16) as i32)) << 4)) as WORD;
    } else {
        ch = _totupper((ch as i32) as u16) as TCHAR;

        if ch >= ('A' as u16) as TCHAR && ch <= ('F' as u16) as TCHAR {
            ret = (ret as i32
                | (((ch as i32) - (('A' as u16) as i32) + 10)
                    << 4)) as WORD;
        } else {
            return (-(1 as i32)) as WORD;
        }
    }

    return ret;
}

unsafe extern "C" fn txt_clr(str: LPTSTR) -> WORD {
    let mut i: INT = 0;
    while *clrtable[i as usize].name != 0 {
        if _tcsnicmp(str, clrtable[i as usize].name, _tcslen(clrtable[i as usize].name)) == 0 {
            return clrtable[i as usize].val;
        }
        i += 1;
    }
    return (-(1 as INT)) as WORD;
}

unsafe extern "C" fn str_to_color(arg_str: *mut LPTSTR) -> WORD {
    let mut str: LPTSTR;
    let mut bBri: BOOL;

    let mut tmp_clr: WORD;
    let mut ret_clr: WORD;

    str = *arg_str;

    if !(*str == 0) {
    } else {
        return (-(1 as i32)) as WORD;
    }

    /* foreground */
    bBri = (FALSE) as i32;

    if _tcsnicmp(str, (&[0x0062, 0x0072, 0x0069, 0x0000u16]).as_ptr() /* _T "bri" */, 3) == 0 {
        bBri = (TRUE) as i32;

        if chop_blank(core::ptr::addr_of_mut!(str)) != 0 {
            return (-(1 as i32)) as WORD;
        }
    }

    tmp_clr = txt_clr(str) as WORD;
    if tmp_clr == (-(1 as i32)) as WORD {
        return (-(1 as i32)) as WORD;
    }

    /* skip spaces and "on" keyword */
    if chop_blank(core::ptr::addr_of_mut!(str)) != 0
        || chop_blank(core::ptr::addr_of_mut!(str)) != 0
    {
        return (-(1 as i32)) as WORD;
    }

    ret_clr = ((tmp_clr as i32) | (bBri << 3)) as WORD;

    /* background */
    bBri = (FALSE) as i32;

    if _tcsnicmp(str, (&[0x0062, 0x0072, 0x0069, 0x0000u16]).as_ptr() /* _T "bri" */, 3) == 0 {
        bBri = (TRUE) as i32;

        if chop_blank(core::ptr::addr_of_mut!(str)) != 0 {
            return (-(1 as i32)) as WORD;
        }
    }

    tmp_clr = txt_clr(str) as WORD;
    if tmp_clr == (-(1 as i32)) as WORD {
        return (-(1 as i32)) as WORD;
    }

    chop_blank(core::ptr::addr_of_mut!(str));

    *arg_str = str;

    /* NOTE: See the note on SC_HEX in the StringToColor()'s description */
    return ((ret_clr as i32)
        | ((tmp_clr as i32) << 4)
        | (bBri << 7)) as WORD;
}
#[no_mangle]
pub unsafe extern "C" fn StringToColor(lpColor: LPWORD, str: *mut LPTSTR) -> BOOL {
    let mut wRet: WORD = 0;

    wRet = str_to_color(str);
    if wRet == (-(1 as ::core::ffi::c_int)) as WORD {
        wRet = hex_clr(*str);
        chop_blank(str);
        if wRet == (-(1 as ::core::ffi::c_int)) as WORD {
            return (FALSE) as BOOL;
        }
    }

    *lpColor = wRet;

    return (TRUE) as BOOL;
}
