#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
//! `dll/win32/msvcrt/mbcs.c`, translated whole. Identifiers are kept EXACTLY as the C spells
//! them — that 1:1 correspondence is what makes the appended original-C block a usable diff
//! and what lets a reviewer check a body against its source without a rename table.

use crate::prelude::*;

// ---- file-scope preamble (translated) ----
pub static mut MSVCRT_mbctype: [u8; 257] = [0; 257];

#[repr(C)]
pub struct cp_extra_info_t {
    pub cp: i32,
    pub TrailBytes: [u8; 5],
}

pub static g_cpextrainfo: [cp_extra_info_t; 7] = [
    cp_extra_info_t { cp: 932, TrailBytes: [0x40, 0x7e, 0x80, 0xfc, 0] },
    cp_extra_info_t { cp: 936, TrailBytes: [0x40, 0xfe, 0, 0, 0] },
    cp_extra_info_t { cp: 949, TrailBytes: [0x41, 0xfe, 0, 0, 0] },
    cp_extra_info_t { cp: 950, TrailBytes: [0x40, 0x7e, 0xa1, 0xfe, 0] },
    cp_extra_info_t { cp: 1361, TrailBytes: [0x31, 0x7e, 0x81, 0xfe, 0] },
    cp_extra_info_t { cp: 20932, TrailBytes: [1, 255, 0, 0, 0] },
    cp_extra_info_t { cp: 0, TrailBytes: [1, 255, 0, 0, 0] },
];

pub static mbbtombc_932: [u8; 158] = [
    0x40,0x49,0x68,0x94,0x90,0x93,0x95,0x66,0x69,0x6a,0x96,0x7b,0x43,0x7c,0x44,0x5e,
    0x4f,0x50,0x51,0x52,0x53,0x54,0x55,0x56,0x57,0x58,0x46,0x47,0x83,0x81,0x84,0x48,
    0x97,0x60,0x61,0x62,0x63,0x64,0x65,0x66,0x67,0x68,0x69,0x6a,0x6b,0x6c,0x6d,0x6e,
    0x6f,0x70,0x71,0x72,0x73,0x74,0x75,0x76,0x77,0x78,0x79,0x6d,0x8f,0x6e,0x4f,0x76,
    0x77,0x78,0x79,0x6d,0x8f,0x6e,0x4f,0x51,0x65,0x81,0x82,0x83,0x84,0x85,0x86,0x87,
    0x88,0x89,0x8a,0x8b,0x8c,0x8d,0x8e,0x8f,0x90,0x91,0x92,0x93,0x94,0x95,0x50,0x42,
    0x75,0x76,0x41,0x45,0x92,0x40,0x42,0x44,0x46,0x48,0x83,0x85,0x87,0x62,0x5b,0x41,
    0x43,0x45,0x47,0x49,0x4a,0x4c,0x4e,0x50,0x52,0x54,0x56,0x58,0x5a,0x5c,0x5e,0x60,
    0x63,0x65,0x67,0x69,0x6a,0x6b,0x6c,0x6d,0x6e,0x71,0x74,0x77,0x7a,0x7d,0x7e,0x80,
    0x81,0x82,0x84,0x86,0x88,0x89,0x8a,0x8b,0x8c,0x8d,0x8f,0x93,0x4a,0x4b,
];

pub static mbctombb_932_punct: [u8; 88] = [
    0x20,0xa4,0xa1,0x2c,0x2e,0xa5,0x3a,0x3b,0x3f,0x21,0xde,0xdf,0x00,0x00,0x00,0x5e,
    0x7e,0x5f,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xb0,0x00,0x00,0x2f,0x00,
    0x00,0x00,0x7c,0x00,0x00,0x60,0x27,0x00,0x22,0x28,0x29,0x00,0x00,0x5b,0x5d,0x7b,
    0x7d,0x00,0x00,0x00,0x00,0xa2,0xa3,0x00,0x00,0x00,0x00,0x2b,0x2d,0x00,0x00,0x00,
    0x00,0x3d,0x00,0x3c,0x3e,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x5c,
    0x24,0x00,0x00,0x25,0x23,0x26,0x2a,0x40,
];

pub static mbctombb_932_kana: [u8; 86] = [
    0xa7,0xb1,0xa8,0xb2,0xa9,0xb3,0xaa,0xb4,0xab,0xb5,0xb6,0xb6,0xb7,0xb7,0xb8,0xb8,
    0xb9,0xb9,0xba,0xba,0xbb,0xbb,0xbc,0xbc,0xbd,0xbd,0xbe,0xbe,0xbf,0xbf,0xc0,0xc0,
    0xc1,0xc1,0xaf,0xc2,0xc2,0xc3,0xc3,0xc4,0xc4,0xc5,0xc6,0xc7,0xc8,0xc9,0xca,0xca,
    0xca,0xcb,0xcb,0xcb,0xcc,0xcc,0xcc,0xcd,0xcd,0xcd,0xce,0xce,0xce,0xcf,0xd0,0xd1,
    0xd2,0xd3,0xac,0xd4,0xad,0xd5,0xae,0xd6,0xd7,0xd8,0xd9,0xda,0xdb,0xdc,0xdc,0xb2,
    0xb4,0xa6,0xdd,0xb3,0xb6,0xb9,
];
unsafe extern "C" fn msvcrt_mbc_to_wc_l(ch: u32, locale: _locale_t) -> wchar_t {
    let mbcinfo: pthreadmbcinfo;
    let mut chW: wchar_t = 0;
    let mut mbch: [u8; 2] = [0; 2];
    let n_chars: i32;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if ch <= 0xff {
        mbch[0] = ch as u8;
        n_chars = 1;
    } else {
        mbch[0] = ((ch >> 8) & 0xff) as u8;
        mbch[1] = (ch & 0xff) as u8;
        n_chars = 2;
    }
    if MultiByteToWideChar(
        ((*mbcinfo).mbcodepage) as u32,
        0,
        mbch.as_ptr() as *const i8,
        n_chars,
        &mut chW,
        1,
    ) == 0
    {
        return 0;
    }
    chW
}
unsafe extern "C" fn u_strlen(str: *const u8) -> usize {
    strlen(str as *const i8)
}
unsafe extern "C" fn u_strncat(dst: *mut u8, src: *const u8, len: usize) -> *mut u8 {
    strncat(dst as *mut i8, src as *const i8, len) as *mut u8
}
unsafe extern "C" fn u_strcmp(s1: *const u8, s2: *const u8) -> i32 {
    strcmp(s1 as *const i8, s2 as *const i8)
}
unsafe extern "C" fn u_strcasecmp(s1: *const u8, s2: *const u8) -> i32 {
    _stricmp(s1 as *const i8, s2 as *const i8)
}
unsafe extern "C" fn u_strncmp(s1: *const u8, s2: *const u8, len: usize) -> i32 {
    strncmp(s1 as *const i8, s2 as *const i8, len)
}
unsafe extern "C" fn u_strncasecmp(s1: *const u8, s2: *const u8, len: usize) -> i32 {
    _strnicmp(s1 as *const i8, s2 as *const i8, len)
}
unsafe extern "C" fn u_strchr(s: *const u8, x: u8) -> *mut u8 {
    strchr(s as *const i8, x as i32) as *mut u8
}
unsafe extern "C" fn u_strpbrk(str: *const ::core::ffi::c_uchar, accept: *const ::core::ffi::c_uchar) -> *mut ::core::ffi::c_uchar {
    strpbrk(str as *const ::core::ffi::c_char, accept as *const ::core::ffi::c_char) as *mut ::core::ffi::c_uchar
}
unsafe extern "C" fn u_strrchr(s: *const u8, x: u8) -> *mut u8 {
    strrchr(s as *const i8, x as i32) as *mut u8
}
unsafe extern "C" fn u__strrev(str: *mut u8) -> *mut u8 {
    _strrev(str as *mut i8) as *mut u8
}
unsafe extern "C" fn u__strset(s: *mut u8, c: u8) -> *mut u8 {
    _strset(s as *mut i8, c as i32) as *mut u8
}
unsafe extern "C" fn u__strnset(s: *mut u8, c: u8, len: usize) -> *mut u8 {
    _strnset(s as *mut i8, (c) as i32, len) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn __p__mbctype() -> *mut u8 {
    (*get_mbcinfo()).mbctype.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn __p___mb_cur_max() -> *mut i32 {
    core::ptr::addr_of_mut!((*get_locinfo()).mb_cur_max)
}

#[no_mangle]
pub unsafe extern "C" fn ___mb_cur_max_func() -> i32 {
    (*get_locinfo()).mb_cur_max
}
#[no_mangle]
pub unsafe extern "C" fn create_mbcinfo(cp: i32, lcid: LCID, old_mbcinfo: pthreadmbcinfo) -> pthreadmbcinfo {
    let mbcinfo: pthreadmbcinfo;
    let mut newcp: i32;
    let mut cpi: CPINFO = core::mem::zeroed();
    let mut bytes: *const BYTE;
    let mut chartypes: [WORD; 256] = [0; 256];
    let mut bufA: [i8; 256] = [0; 256];
    let mut bufW: [wchar_t; 256] = [0; 256];
    let mut lowW: [wchar_t; 256] = [0; 256];
    let mut upW: [wchar_t; 256] = [0; 256];
    let mut charcount: i32;
    let maxchar: i32;
    let mut ret: i32;
    let mut i: i32;

    if !old_mbcinfo.is_null() && cp == (*old_mbcinfo).mbcodepage
        && (lcid == (-1i32) as LCID || lcid == ((*old_mbcinfo).mblcid) as u32) {
        InterlockedIncrement(core::ptr::addr_of_mut!((*old_mbcinfo).refcount));
        return old_mbcinfo;
    }

    mbcinfo = malloc(core::mem::size_of::<thread_data_t>()) as pthreadmbcinfo;
    if mbcinfo.is_null() {
        return core::ptr::null_mut();
    }
    (*mbcinfo).refcount = 1;

    match cp {
        _MB_CP_ANSI => {
            newcp = (GetACP()) as i32;
        }
        _MB_CP_OEM => {
            newcp = (GetOEMCP()) as i32;
        }
        _MB_CP_LOCALE => {
            newcp = ((*get_locinfo()).lc_codepage) as i32;
            if newcp != 0 {
            } else {
                newcp = 20127;
            }
        }
        _MB_CP_SBCS => {
            newcp = 20127;
        }
        _ => {
            newcp = cp;
        }
    }

    if lcid == (-1i32) as LCID {
        let mut wbuf: [wchar_t; (LOCALE_NAME_MAX_LENGTH) as usize] = [0; (LOCALE_NAME_MAX_LENGTH) as usize];
        sprintf(bufA.as_mut_ptr(), b".%d\0".as_ptr() as *const i8, newcp);
        (*mbcinfo).mblcid = if locale_to_sname(
            bufA.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            wbuf.as_mut_ptr()
        ) != 0 {
            (LocaleNameToLCID(wbuf.as_ptr(), LOCALE_ALLOW_NEUTRAL_NAMES)) as i32
        } else {
            -1
        };
    } else {
        (*mbcinfo).mblcid = (lcid) as i32;
    }

    if (*mbcinfo).mblcid == -1 {
        (*mbcinfo).mblcid = 0;
    }

    if GetCPInfo(newcp as u32, &mut cpi) == 0 {
        if newcp == 20127 {
            memset(
                &mut cpi as *mut CPINFO as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<CPINFO>()
            );
            cpi.MaxCharSize = 1;
            cpi.DefaultChar[0] = b'?' as u8;
        } else {
            free(mbcinfo as *mut core::ffi::c_void);
            return core::ptr::null_mut();
        }
    }

    memset(
        (*mbcinfo).mbctype.as_mut_ptr() as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[u8; 257]>()
    );
    memset(
        (*mbcinfo).mbcasemap.as_mut_ptr() as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[u8; 256]>()
    );

    bytes = cpi.LeadByte.as_mut_ptr();
    while *bytes.offset(0) != 0 || *bytes.offset(1) != 0 {
        i = *bytes.offset(0) as i32;
        while i <= *bytes.offset(1) as i32 {
            (*mbcinfo).mbctype[(i + 1) as usize] |= (_M1) as u8;
            i += 1;
        }
        bytes = bytes.offset(2);
    }

    if cpi.MaxCharSize == 2 {
        let mut cpextra: *const cp_extra_info_t = g_cpextrainfo.as_ptr();

        (*mbcinfo).ismbcodepage = 1;
        loop {
            if (*cpextra).cp == 0 || (*cpextra).cp == newcp {
                if (*cpextra).cp == 0 {
                }

                bytes = (*cpextra).TrailBytes.as_ptr();
                while *bytes.offset(0) != 0 || *bytes.offset(1) != 0 {
                    i = *bytes.offset(0) as i32;
                    while i <= *bytes.offset(1) as i32 {
                        (*mbcinfo).mbctype[(i + 1) as usize] |= (_M2) as u8;
                        i += 1;
                    }
                    bytes = bytes.offset(2);
                }
                break;
            }
            cpextra = cpextra.offset(1);
        }
    } else {
        (*mbcinfo).ismbcodepage = 0;
    }

    maxchar = if newcp == (CP_UTF8) as i32 { 128 } else { 256 };

    charcount = 0;
    i = 0;
    while i < maxchar {
        if ((*mbcinfo).mbctype[(i + 1) as usize] & (_M1) as u8) == 0 {
            bufA[charcount as usize] = i as i8;
            charcount += 1;
        }
        i += 1;
    }

    ret = MultiByteToWideChar(
        newcp as u32,
        0,
        bufA.as_ptr(),
        charcount,
        bufW.as_mut_ptr(),
        charcount
    );
    if ret != charcount {
    }

    GetStringTypeW(CT_CTYPE1, bufW.as_ptr(), charcount, chartypes.as_mut_ptr());
    LCMapStringW(((*mbcinfo).mblcid) as u32, LCMAP_LOWERCASE, bufW.as_ptr(), charcount, lowW.as_mut_ptr(), charcount);
    LCMapStringW(((*mbcinfo).mblcid) as u32, LCMAP_UPPERCASE, bufW.as_ptr(), charcount, upW.as_mut_ptr(), charcount);

    charcount = 0;
    i = 0;
    while i < maxchar {
        if ((*mbcinfo).mbctype[(i + 1) as usize] & (_M1) as u8) == 0 {
            if (chartypes[charcount as usize] & (C1_UPPER) as u16) != 0 {
                (*mbcinfo).mbctype[(i + 1) as usize] |= (_SBUP) as u8;
                bufW[charcount as usize] = lowW[charcount as usize];
            } else if (chartypes[charcount as usize] & (C1_LOWER) as u16) != 0 {
                (*mbcinfo).mbctype[(i + 1) as usize] |= (_SBLOW) as u8;
                bufW[charcount as usize] = upW[charcount as usize];
            }
            charcount += 1;
        }
        i += 1;
    }

    ret = WideCharToMultiByte(
        newcp as u32,
        0,
        bufW.as_ptr(),
        charcount,
        bufA.as_mut_ptr(),
        charcount,
        core::ptr::null_mut(),
        core::ptr::null_mut()
    );
    if ret != charcount {
    }

    charcount = 0;
    i = 0;
    while i < maxchar {
        if ((*mbcinfo).mbctype[(i + 1) as usize] & (_M1) as u8) == 0 {
            if ((*mbcinfo).mbctype[(i + 1) as usize] & ((_SBUP | _SBLOW)) as u8) != 0 {
                (*mbcinfo).mbcasemap[i as usize] = bufA[charcount as usize] as u8;
            }
            charcount += 1;
        }
        i += 1;
    }

    if newcp == 932 {
        i = 161;
        while i <= 165 {
            (*mbcinfo).mbctype[(i + 1) as usize] |= (_MP) as u8;
            i += 1;
        }
        i = 166;
        while i <= 223 {
            (*mbcinfo).mbctype[(i + 1) as usize] |= (_MS) as u8;
            i += 1;
        }
    }

    (*mbcinfo).mbcodepage = newcp;
    mbcinfo
}

#[no_mangle]
pub unsafe extern "C" fn _setmbcp(cp: i32) -> i32 {
    let data: *mut thread_data_t = msvcrt_get_thread_data();
    let mbcinfo: pthreadmbcinfo;

    mbcinfo = create_mbcinfo(cp, (-1i32) as LCID, get_mbcinfo());
    if mbcinfo.is_null() {
        *_errno() = (EINVAL) as i32;
        return -1;
    }

    if ((*data).locale_flags & (LOCALE_THREAD) as i32) != 0 {
        if ((*data).locale_flags & (LOCALE_FREE) as i32) != 0 {
            free_mbcinfo((*data).mbcinfo);
        }
        (*data).mbcinfo = mbcinfo;
    } else {
        _lock(_MB_CP_LOCK);
        free_mbcinfo((*MSVCRT_locale).mbcinfo);
        (*MSVCRT_locale).mbcinfo = mbcinfo;
        memcpy(
            core::ptr::addr_of_mut!(MSVCRT_mbctype) as *mut core::ffi::c_void,
            (*(*MSVCRT_locale).mbcinfo).mbctype.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of::<[u8; 257]>(),
        );
        _unlock(_MB_CP_LOCK);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _getmbcp() -> i32 {
    (*get_mbcinfo()).mbcodepage
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnextc_l(str: *const u8, locale: _locale_t) -> u32 {
    if _ismbblead_l(*str as u32, locale) != 0 {
        return ((*str as u32) << 8) | (*str.offset(1) as u32);
    }
    *str as u32
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnextc(str: *const u8) -> u32 {
    _mbsnextc_l(str, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbctolower_l(c: u32, locale: _locale_t) -> u32 {
    let mut str: [u8; 2] = [0; 2];
    let mut ret: [u8; 2] = [0; 2];
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if c > 0xff {
        if _ismbblead_l((c >> 8) & 0xff, locale) == 0 {
            return c;
        }

        str[0] = (c >> 8) as u8;
        str[1] = c as u8;
        match __crtLCMapStringA(
            ((*mbcinfo).mblcid) as u32,
            LCMAP_LOWERCASE,
            str.as_ptr() as *const i8,
            2,
            ret.as_mut_ptr() as *mut i8,
            2,
            ((*mbcinfo).mbcodepage) as u32,
            0
        ) {
            0 => {
                return c;
            }
            1 => {
                return ret[0] as u32;
            }
            _ => {
                return ret[1] as u32 + ((ret[0] as u32) << 8);
            }
        }
    }

    if (*(*mbcinfo).mbctype.as_ptr().add((c + 1) as usize) as u32 & _SBUP) != 0 {
        *(*mbcinfo).mbcasemap.as_ptr().add(c as usize) as u32
    } else {
        c
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbctolower(c: u32) -> u32 {
    _mbctolower_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbctoupper_l(c: u32, locale: _locale_t) -> u32 {
    let mut str: [u8; 2] = [0; 2];
    let mut ret: [u8; 2] = [0; 2];
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if c > 0xff {
        if _ismbblead_l((c >> 8) & 0xff, locale) == 0 {
            return c;
        }

        str[0] = (c >> 8) as u8;
        str[1] = c as u8;
        match __crtLCMapStringA(
            ((*mbcinfo).mblcid) as u32,
            LCMAP_UPPERCASE,
            str.as_ptr() as *const i8,
            2,
            ret.as_mut_ptr() as *mut i8,
            2,
            ((*mbcinfo).mbcodepage) as u32,
            0,
        ) {
            0 => {
                return c;
            }
            1 => {
                return ret[0] as u32;
            }
            _ => {
                return (ret[1] as u32) + ((ret[0] as u32) << 8);
            }
        }
    }

    if (*mbcinfo).mbctype[(c + 1) as usize] & (_SBLOW) as u8 != 0 {
        (*mbcinfo).mbcasemap[c as usize] as u32
    } else {
        c
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbctoupper(c: u32) -> u32 {
    _mbctoupper_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbctombb_l(c: core::ffi::c_uint, locale: _locale_t) -> core::ffi::c_uint {
    let mbcinfo: pthreadmbcinfo;
    let value: core::ffi::c_uint;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).mbcodepage == 932 {
        if c >= 0x829f && c <= 0x82f1 {
            return mbctombb_932_kana[(c - 0x829f) as usize] as core::ffi::c_uint;
        }
        if c >= 0x8340 && c <= 0x8396 && c != 0x837f {
            return mbctombb_932_kana[(c - 0x8340 - (if c >= 0x837f { 1 } else { 0 })) as usize] as core::ffi::c_uint;
        }
        if c >= 0x8140 && c <= 0x8197 {
            value = mbctombb_932_punct[(c - 0x8140) as usize] as core::ffi::c_uint;
            return if value != 0 { value } else { c };
        }
        if (c >= 0x824f && c <= 0x8258) ||
           (c >= 0x8260 && c <= 0x8279) {
            return c - 0x821f;
        }
        if c >= 0x8281 && c <= 0x829a {
            return c - 0x8220;
        }
    }
    return c;
}

#[no_mangle]
pub unsafe extern "C" fn _mbctombb(c: u32) -> u32 {
    _mbctombb_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbcjistojms_l(mut c: u32, locale: _locale_t) -> u32 {
    let mbcinfo: pthreadmbcinfo;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).mbcodepage == 932 {
        if HIBYTE!(c) >= 0x21 && HIBYTE!(c) <= 0x7e &&
           LOBYTE!(c) >= 0x21 && LOBYTE!(c) <= 0x7e {
            if HIBYTE!(c) % 2 != 0 {
                c = c.wrapping_add(0x1f);
            } else {
                c = c.wrapping_add(0x7d);
            }

            if LOBYTE!(c) >= 0x7F {
                c = c.wrapping_add(0x1);
            }

            c = ((HIBYTE!(c).wrapping_sub(0x21) / 2 + 0x81) << 8) | LOBYTE!(c);

            if HIBYTE!(c) > 0x9f {
                c = c.wrapping_add(0x4000);
            }
        } else {
            return 0;
        }
    }

    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbcjistojms(c: u32) -> u32 {
    _mbcjistojms_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbcjmstojis_l(c: u32, locale: _locale_t) -> u32 {
    let mbcinfo: pthreadmbcinfo;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).mbcodepage == 932 {
        if _ismbclegal_l(c, locale) != 0 && HIBYTE!(c) < 0xf0 {
            let mut c = c;
            if HIBYTE!(c) >= 0xe0 {
                c -= 0x4000;
            }

            c = (((HIBYTE!(c) - 0x81) * 2 + 0x21) << 8) | LOBYTE!(c);

            if LOBYTE!(c) > 0x7f {
                c -= 0x1;
            }

            if LOBYTE!(c) > 0x9d {
                c += 0x83;
            } else {
                c -= 0x1f;
            }

            return c;
        } else {
            return 0;
        }
    }

    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbcjmstojis(c: u32) -> u32 {
    _mbcjmstojis_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbclen_l(str: *const u8, locale: _locale_t) -> usize {
    if _ismbblead_l(*str as u32, locale) != 0 && *str.offset(1) != 0 {
        2
    } else {
        1
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbclen(str: *const u8) -> size_t {
    _mbclen_l(str, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsinc_l(str: *const u8, locale: _locale_t) -> *mut u8 {
    str.offset(_mbclen_l(str, locale) as isize) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn _mbsinc(str: *const u8) -> *mut u8 {
    _mbsinc_l(str, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsninc(str: *const u8, num: usize) -> *mut u8 {
    if str.is_null() {
        return core::ptr::null_mut();
    }

    let mut str = str;
    let mut num = num;

    while num > 0 && *str != 0 {
        if _ismbblead(*str as u32) != 0 {
            if *str.offset(1) == 0 {
                break;
            }
            str = str.offset(1);
        }
        str = str.offset(1);
        num -= 1;
    }

    str as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnlen_l(
    str: *const u8,
    maxsize: usize,
    locale: _locale_t) -> usize {
    let mbcinfo: pthreadmbcinfo;
    let mut i: usize = 0;
    let mut len: usize = 0;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if (*mbcinfo).ismbcodepage == 0 {
        return strnlen(str as *const i8, maxsize);
    }

    while i < maxsize && *str.add(i) != 0 {
        if _ismbblead_l(*str.add(i) as u32, locale) != 0 {
            i += 1;
            if *str.add(i) == 0 {
                break;
            }
        }
        i += 1;
        len += 1;
    }

    if i < maxsize {
        len
    } else {
        maxsize
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbslen(str: *const u8) -> usize {
    _mbsnlen_l(str, usize::MAX, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbslen_l(
    str: *const u8,
    locale: _locale_t) -> usize {
    _mbsnlen_l(str, usize::MAX, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnlen(str: *const u8, maxsize: size_t) -> size_t {
    _mbsnlen_l(str, maxsize, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbccpy_s_l(
    dest: *mut u8,
    maxsize: usize,
    copied: *mut i32,
    src: *const u8,
    locale: _locale_t) -> i32 {
    if !copied.is_null() {
        *copied = 0;
    }
    if MSVCRT_CHECK_PMT!(!dest.is_null() && maxsize >= 1) == 0 {
        return (EINVAL) as i32;
    }
    *dest.offset(0 as isize) = 0;
    if MSVCRT_CHECK_PMT!(!src.is_null()) == 0 {
        return (EINVAL) as i32;
    }

    if _ismbblead_l(*src as u32, locale) != 0 {
        if *src.offset(1 as isize) == 0 {
            if !copied.is_null() {
                *copied = 1;
            }
            *_errno() = (EILSEQ) as i32;
            return (EILSEQ) as i32;
        }

        if maxsize < 2 {
            MSVCRT_INVALID_PMT!(b"dst buffer is too small\0".as_ptr() as *const i8, (ERANGE) as i32);
            return (ERANGE) as i32;
        }

        let mut dest_ptr = dest;
        let mut src_ptr = src;
        *dest_ptr = *src_ptr;
        dest_ptr = dest_ptr.offset(1);
        src_ptr = src_ptr.offset(1);
        *dest_ptr = *src_ptr;
        if !copied.is_null() {
            *copied = 2;
        }
    } else {
        *dest = *src;
        if !copied.is_null() {
            *copied = 1;
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbccpy(dest: *mut u8, src: *const u8) {
    _mbccpy_s_l(dest, 2, core::ptr::null_mut(), src, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "C" fn _mbccpy_l(
    dest: *mut u8,
    src: *const u8,
    locale: _locale_t) {
    _mbccpy_s_l(dest, 2, core::ptr::null_mut(), src, locale);
}

#[no_mangle]
pub unsafe extern "C" fn _mbccpy_s(
    dest: *mut u8,
    maxsize: size_t,
    copied: *mut i32,
    src: *const u8,
) -> i32 {
    _mbccpy_s_l(dest, maxsize, copied, src, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncpy_l(
    dst: *mut u8,
    src: *const u8,
    mut n: usize,
    locale: _locale_t) -> *mut u8 {
    let ret: *mut u8 = dst;
    let mbcinfo: pthreadmbcinfo;

    if n == 0 {
        return dst;
    }
    if MSVCRT_CHECK_PMT!(!dst.is_null() && !src.is_null()) == 0 {
        return core::ptr::null_mut();
    }
    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    let mut dst_ptr = dst;
    let mut src_ptr = src;

    if (*mbcinfo).ismbcodepage != 0 {
        while *src_ptr != 0 && n != 0 {
            n = n.wrapping_sub(1);
            if _ismbblead_l(*src_ptr as u32, locale) != 0 {
                if *src_ptr.offset(1) == 0 {
                    *dst_ptr = 0;
                    dst_ptr = dst_ptr.offset(1);
                    *dst_ptr = 0;
                    dst_ptr = dst_ptr.offset(1);
                    break;
                }
                *dst_ptr = *src_ptr;
                dst_ptr = dst_ptr.offset(1);
                src_ptr = src_ptr.offset(1);
            }
            *dst_ptr = *src_ptr;
            dst_ptr = dst_ptr.offset(1);
            src_ptr = src_ptr.offset(1);
        }
    } else {
        while n != 0 {
            n = n.wrapping_sub(1);
            *dst_ptr = *src_ptr;
            let val = *src_ptr;
            dst_ptr = dst_ptr.offset(1);
            src_ptr = src_ptr.offset(1);
            if val == 0 {
                break;
            }
        }
    }
    while n != 0 {
        n = n.wrapping_sub(1);
        *dst_ptr = 0;
        dst_ptr = dst_ptr.offset(1);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncpy(dst: *mut u8, src: *const u8, n: size_t) -> *mut u8 {
    _mbsncpy_l(dst, src, n, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcpy_s_l(
    dst: *mut u8,
    size: usize,
    src: *const u8,
    n: usize,
    locale: _locale_t) -> i32 {
    let mut pos = 0;

    if dst.is_null() || size == 0 {
        return (EINVAL) as i32;
    }
    if src.is_null() {
        *dst.add(0) = b'\0';
        return (EINVAL) as i32;
    }
    if n == 0 {
        return 0;
    }

    let mbcinfo = if !locale.is_null() {
        (*locale).mbcinfo
    } else {
        get_mbcinfo()
    };

    if (*mbcinfo).ismbcodepage != 0 {
        let mut is_lead = FALSE;
        let mut src_ptr = src;
        let mut n_remaining = n;
        while *src_ptr != 0 && n_remaining != 0 {
            if pos == size {
                *dst.add(0) = b'\0';
                return (ERANGE) as i32;
            }
            is_lead = ((is_lead == 0) && _ismbblead(*src_ptr as u32) != 0) as u8;
            n_remaining -= 1;
            *dst.add(pos) = *src_ptr;
            pos += 1;
            src_ptr = src_ptr.add(1);
        }

        if is_lead != 0 {
            *dst.add(pos - 1) = 0;
        }
    } else {
        let mut n_remaining = n;
        let mut src_ptr = src;
        while n_remaining != 0 {
            n_remaining -= 1;
            if pos == size {
                *dst.add(0) = b'\0';
                return (ERANGE) as i32;
            }

            if *src_ptr == 0 {
                break;
            }
            *dst.add(pos) = *src_ptr;
            pos += 1;
            src_ptr = src_ptr.add(1);
        }
    }

    if pos < size {
        *dst.add(pos) = b'\0';
    } else {
        *dst.add(0) = b'\0';
        return (ERANGE) as i32;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcpy_s(
    dst: *mut u8,
    size: size_t,
    src: *const u8,
    n: size_t,
) -> i32 {
    _mbsnbcpy_s_l(dst, size, src, n, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbscpy_s_l(
    dst: *mut u8,
    size: usize,
    src: *const u8,
    locale: _locale_t) -> i32 {
    _mbsnbcpy_s_l(dst, size, src, usize::MAX, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbscpy_s(dst: *mut u8, size: size_t, src: *const u8) -> i32 {
    _mbscpy_s_l(dst, size, src, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcpy_l(dst: *mut u8, src: *const u8, n: usize, locale: _locale_t) -> *mut u8 {
    let ret: *mut u8 = dst;
    let mbcinfo: pthreadmbcinfo;

    if n == 0 {
        return dst;
    }
    if MSVCRT_CHECK_PMT!(!dst.is_null() && !src.is_null()) == 0 {
        return core::ptr::null_mut();
    }
    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }
    if (*mbcinfo).ismbcodepage != 0 {
        let mut is_lead: u8 = FALSE;
        let mut n_remaining = n;
        let mut dst_ptr = dst;
        let mut src_ptr = src;
        while *src_ptr != 0 && n_remaining != 0 {
            is_lead = (is_lead == 0 && _ismbblead_l(*src_ptr as u32, locale) != 0) as u8;
            n_remaining -= 1;
            *dst_ptr = *src_ptr;
            dst_ptr = dst_ptr.add(1);
            src_ptr = src_ptr.add(1);
        }

        if is_lead != 0 {
            *dst_ptr.sub(1) = 0;
        }
        let mut n_remaining = n_remaining;
        while n_remaining > 0 {
            n_remaining -= 1;
            *dst_ptr = 0;
            dst_ptr = dst_ptr.add(1);
        }
    } else {
        let mut n_remaining = n;
        let mut dst_ptr = dst;
        let mut src_ptr = src;
        while n_remaining != 0 {
            n_remaining -= 1;
            *dst_ptr = *src_ptr;
            let val = *src_ptr;
            dst_ptr = dst_ptr.add(1);
            src_ptr = src_ptr.add(1);
            if val == 0 {
                break;
            }
        }
        while n_remaining > 0 {
            n_remaining -= 1;
            *dst_ptr = 0;
            dst_ptr = dst_ptr.add(1);
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcpy(dst: *mut u8, src: *const u8, n: size_t) -> *mut u8 {
    _mbsnbcpy_l(dst, src, n, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbscmp_l(str: *const u8, cmp: *const u8, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if MSVCRT_CHECK_PMT!(str as usize != 0 && cmp as usize != 0) == 0 {
        return _NLSCMPERROR;
    }

    mbcinfo = if !locale.is_null() {
        (*locale).mbcinfo
    } else {
        get_mbcinfo()
    };

    if (*mbcinfo).ismbcodepage != 0 {
        let mut strc: u32;
        let mut cmpc: u32;
        let mut str_ptr = str;
        let mut cmp_ptr = cmp;
        loop {
            if *str_ptr == 0 {
                return if *cmp_ptr != 0 { -1 } else { 0 };
            }
            if *cmp_ptr == 0 {
                return 1;
            }
            strc = _mbsnextc_l(str_ptr, locale);
            cmpc = _mbsnextc_l(cmp_ptr, locale);
            if strc != cmpc {
                return if strc < cmpc { -1 } else { 1 };
            }
            str_ptr = str_ptr.offset(if strc > 255 { 2 } else { 1 });
            cmp_ptr = cmp_ptr.offset(if strc > 255 { 2 } else { 1 });
        }
    }
    u_strcmp(str, cmp)
}

#[no_mangle]
pub unsafe extern "C" fn _mbscmp(str: *const u8, cmp: *const u8) -> i32 {
    _mbscmp_l(str, cmp, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbicoll_l(
    str1: *const u8,
    str2: *const u8,
    len: usize,
    locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if (*mbcinfo).ismbcodepage == 0 {
        return _strnicoll_l(
            str1 as *const i8,
            str2 as *const i8,
            len,
            locale,
        );
    }
    CompareStringA(
        (*mbcinfo).mblcid as LCID,
        NORM_IGNORECASE,
        str1 as *const i8,
        strnlen(str1 as *const i8, len) as c_int,
        str2 as *const i8,
        strnlen(str2 as *const i8, len) as c_int,
    ) - (CSTR_EQUAL) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _mbsicoll_l(
    str1: *const u8,
    str2: *const u8,
    locale: _locale_t) -> i32 {
    _mbsnbicoll_l(str1, str2, (INT_MAX) as usize, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbicoll(str1: *const u8, str2: *const u8, len: size_t) -> i32 {
    _mbsnbicoll_l(str1, str2, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsicoll(str: *const u8, cmp: *const u8) -> i32 {
    _mbsnbicoll_l(str, cmp, (INT_MAX) as usize, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcoll_l(
    str1: *const u8,
    str2: *const u8,
    len: usize,
    locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if (*mbcinfo).ismbcodepage == 0 {
        return _strncoll_l(
            str1 as *const i8,
            str2 as *const i8,
            len,
            locale,
        );
    }
    CompareStringA(
        (*mbcinfo).mblcid as LCID,
        0,
        str1 as *const i8,
        strnlen(str1 as *const i8, len) as c_int,
        str2 as *const i8,
        strnlen(str2 as *const i8, len) as c_int,
    ) - (CSTR_EQUAL) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _mbscoll_l(
    str1: *const u8,
    str2: *const u8,
    locale: _locale_t) -> i32 {
    _mbsnbcoll_l(str1, str2, (INT_MAX) as usize, locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcoll(str1: *const u8, str2: *const u8, len: size_t) -> i32 {
    _mbsnbcoll_l(str1, str2, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbscoll(str: *const u8, cmp: *const u8) -> i32 {
    _mbsnbcoll_l(str, cmp, (INT_MAX) as usize, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsicmp_l(str: *const u8, cmp: *const u8, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if MSVCRT_CHECK_PMT!(str as usize != 0 && cmp as usize != 0) == 0 {
        return _NLSCMPERROR;
    }

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let mut strc: u32;
        let mut cmpc: u32;
        let mut str_ptr = str;
        let mut cmp_ptr = cmp;
        loop {
            if *str_ptr == 0 {
                return if *cmp_ptr != 0 { -1 } else { 0 };
            }
            if *cmp_ptr == 0 {
                return 1;
            }
            strc = _mbctolower_l(_mbsnextc_l(str_ptr, locale), locale);
            cmpc = _mbctolower_l(_mbsnextc_l(cmp_ptr, locale), locale);
            if strc != cmpc {
                return if strc < cmpc { -1 } else { 1 };
            }
            str_ptr = str_ptr.offset(if strc > 255 { 2 } else { 1 });
            cmp_ptr = cmp_ptr.offset(if strc > 255 { 2 } else { 1 });
        }
    }
    u_strcasecmp(str, cmp)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsicmp(str: *const u8, cmp: *const u8) -> i32 {
    _mbsicmp_l(str, cmp, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncmp_l(
    str: *const u8,
    cmp: *const u8,
    len: usize,
    locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;
    let mut strc: u32;
    let mut cmpc: u32;
    let mut str = str;
    let mut cmp = cmp;
    let mut len = len;

    if len == 0 {
        return 0;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage == 0 {
        return u_strncmp(str, cmp, len);
    }

    if MSVCRT_CHECK_PMT!(!str.is_null() && !cmp.is_null()) == 0 {
        return _NLSCMPERROR;
    }

    while len > 0 {
        len -= 1;
        let inc: i32;

        if *str == 0 {
            return if *cmp != 0 { -1 } else { 0 };
        }
        if *cmp == 0 {
            return 1;
        }
        strc = _mbsnextc_l(str, locale);
        cmpc = _mbsnextc_l(cmp, locale);
        if strc != cmpc {
            return if strc < cmpc { -1 } else { 1 };
        }
        inc = if strc > 255 { 2 } else { 1 };
        str = str.offset(inc as isize);
        cmp = cmp.offset(inc as isize);
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncmp(str: *const u8, cmp: *const u8, len: size_t) -> i32 {
    _mbsncmp_l(str, cmp, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcmp_l(str: *const u8, cmp: *const u8, len: usize, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return 0;
    }

    if MSVCRT_CHECK_PMT!(!str.is_null() && !cmp.is_null()) == 0 {
        return _NLSCMPERROR;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let mut strc: u32;
        let mut cmpc: u32;
        let mut len_remaining = len;
        let mut str_ptr = str;
        let mut cmp_ptr = cmp;

        while len_remaining != 0 {
            let clen: i32;
            if *str_ptr == 0 {
                return if *cmp_ptr != 0 { -1 } else { 0 };
            }
            if *cmp_ptr == 0 {
                return 1;
            }
            if _ismbblead_l(*str_ptr as u32, locale) != 0 {
                strc = if len_remaining >= 2 as usize { _mbsnextc_l(str_ptr, locale) } else { 0 };
                clen = 2;
            } else {
                strc = *str_ptr as u32;
                clen = 1;
            }
            if _ismbblead_l(*cmp_ptr as u32, locale) != 0 {
                cmpc = if len_remaining >= 2 as usize { _mbsnextc_l(cmp_ptr, locale) } else { 0 };
            } else {
                cmpc = *cmp_ptr as u32;
            }
            if strc != cmpc {
                return if strc < cmpc { -1 } else { 1 };
            }
            len_remaining -= clen as usize;
            str_ptr = str_ptr.add(clen as usize);
            cmp_ptr = cmp_ptr.add(clen as usize);
        }
        return 0;
    }
    u_strncmp(str, cmp, len)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcmp(str: *const u8, cmp: *const u8, len: size_t) -> i32 {
    _mbsnbcmp_l(str, cmp, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnicmp_l(
    str: *const u8,
    cmp: *const u8,
    len: usize,
    locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(!str.is_null() && !cmp.is_null()) == 0 {
        return _NLSCMPERROR;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let mut strc: u32;
        let mut cmpc: u32;
        let mut str_ptr = str;
        let mut cmp_ptr = cmp;
        let mut remaining = len;
        while remaining > 0 {
            remaining -= 1;
            if *str_ptr == 0 {
                return if *cmp_ptr != 0 { -1 } else { 0 };
            }
            if *cmp_ptr == 0 {
                return 1;
            }
            strc = _mbctolower_l(_mbsnextc_l(str_ptr, locale), locale);
            cmpc = _mbctolower_l(_mbsnextc_l(cmp_ptr, locale), locale);
            if strc != cmpc {
                return if strc < cmpc { -1 } else { 1 };
            }
            str_ptr = str_ptr.add(if strc > 255 { 2 } else { 1 });
            cmp_ptr = cmp_ptr.add(if strc > 255 { 2 } else { 1 });
        }
        return 0;
    }
    u_strncasecmp(str, cmp, len)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnicmp(str: *const u8, cmp: *const u8, len: size_t) -> i32 {
    _mbsnicmp_l(str, cmp, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbicmp_l(
    str: *const u8,
    cmp: *const u8,
    len: usize,
    locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(!str.is_null() && !cmp.is_null()) == 0 {
        return _NLSCMPERROR;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let mut strc: u32;
        let mut cmpc: u32;
        let mut len_remaining = len;
        let mut str_ptr = str;
        let mut cmp_ptr = cmp;
        while len_remaining != 0 {
            let clen: i32;
            if *str_ptr == 0 {
                return if *cmp_ptr != 0 { -1 } else { 0 };
            }
            if *cmp_ptr == 0 {
                return 1;
            }
            if _ismbblead_l(*str_ptr as u32, locale) != 0 {
                strc = if len_remaining >= 2 {
                    _mbsnextc_l(str_ptr, locale)
                } else {
                    0
                };
                clen = 2;
            } else {
                strc = *str_ptr as u32;
                clen = 1;
            }
            if _ismbblead_l(*cmp_ptr as u32, locale) != 0 {
                cmpc = if len_remaining >= 2 {
                    _mbsnextc_l(cmp_ptr, locale)
                } else {
                    0
                };
            } else {
                cmpc = *cmp_ptr as u32;
            }
            strc = _mbctolower_l(strc, locale);
            cmpc = _mbctolower_l(cmpc, locale);
            if strc != cmpc {
                return if strc < cmpc { -1 } else { 1 };
            }
            len_remaining -= clen as usize;
            str_ptr = str_ptr.add(clen as usize);
            cmp_ptr = cmp_ptr.add(clen as usize);
        }
        return 0;
    }
    u_strncasecmp(str, cmp, len)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbicmp(str: *const u8, cmp: *const u8, len: size_t) -> i32 {
    _mbsnbicmp_l(str, cmp, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbscat(
    dst: *mut u8,
    src: *const u8,
) -> *mut u8 {
    strcat(dst as *mut i8, src as *const i8);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn _mbscat_s_l(
    dst: *mut u8,
    size: usize,
    src: *const u8,
    locale: _locale_t) -> i32 {
    let mut i: usize;
    let mut j: usize;
    let mut ret = 0;

    if MSVCRT_CHECK_PMT!(dst != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(src != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }

    i = 0;
    while i < size {
        if *dst.add(i) == 0 {
            break;
        }
        i += 1;
    }
    if i == size {
        MSVCRT_INVALID_PMT!(
            b"dst is not NULL-terminated\0".as_ptr() as *const i8,
            (EINVAL) as i32,
        );
        if size != 0 {
            *dst.add(0) = 0;
        }
        return (EINVAL) as i32;
    }

    if i != 0 && _ismbblead_l(*dst.add(i - 1) as u32, locale) != 0 {
        ret = EILSEQ;
        i -= 1;
    }

    j = 0;
    while *src.add(j) != 0 && i + j < size {
        *dst.add(i + j) = *src.add(j);
        j += 1;
    }
    if i + j == size {
        MSVCRT_INVALID_PMT!(
            b"dst buffer is too small\0".as_ptr() as *const i8,
            (ERANGE) as i32,
        );
        *dst.add(0) = 0;
        return (ERANGE) as i32;
    }

    if j != 0 && _ismbblead_l(*src.add(j - 1) as u32, locale) != 0 {
        ret = EILSEQ;
        j -= 1;
    }

    *dst.add(i + j) = 0;
    (ret) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _mbscat_s(dst: *mut u8, size: size_t, src: *const u8) -> i32 {
    _mbscat_s_l(dst, size, src, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbscpy(
    dst: *mut u8,
    src: *const u8,
) -> *mut u8 {
    strcpy(dst as *mut i8, src as *const i8);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn _mbsstr(haystack: *const u8, needle: *const u8) -> *mut u8 {
    strstr(haystack as *const i8, needle as *const i8) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn _mbschr_l(mut s: *const u8, x: u32, locale: _locale_t) -> *mut u8 {
    let mbcinfo: pthreadmbcinfo;

    if MSVCRT_CHECK_PMT!(!s.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let mut c: u32;
        loop {
            c = _mbsnextc_l(s, locale);
            if c == x {
                return s as *mut u8;
            }
            if c == 0 {
                return core::ptr::null_mut();
            }
            s = s.offset(if c > 255 { 2 } else { 1 });
        }
    }
    return u_strchr(s, x as u8);
}

#[no_mangle]
pub unsafe extern "C" fn _mbschr(s: *const u8, x: u32) -> *mut u8 {
    _mbschr_l(s, x, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsrchr_l(s: *const u8, x: u32, locale: _locale_t) -> *mut u8 {
    let mbcinfo: pthreadmbcinfo;

    if MSVCRT_CHECK_PMT!(!s.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let mut match_ptr: *mut u8 = core::ptr::null_mut();
        let mut c: u32;
        let mut s_mut = s;

        while 1 != 0 {
            c = _mbsnextc_l(s_mut, locale);
            if c == x {
                match_ptr = s_mut as *mut u8;
            }
            if c == 0 {
                return match_ptr;
            }
            s_mut = s_mut.add(if c > 255 { 2 } else { 1 });
        }
    }
    u_strrchr(s, x as u8)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsrchr(s: *const u8, x: u32) -> *mut u8 {
    _mbsrchr_l(s, x, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbstok_s_l(
    str: *mut u8,
    delim: *const u8,
    ctx: *mut *mut u8,
    locale: _locale_t,
) -> *mut u8 {
    let mbcinfo: pthreadmbcinfo;
    let mut c: u32;

    if MSVCRT_CHECK_PMT!(delim != core::ptr::null()) == 0 {
        return core::ptr::null_mut();
    }
    if MSVCRT_CHECK_PMT!(ctx != core::ptr::null_mut()) == 0 {
        return core::ptr::null_mut();
    }
    if MSVCRT_CHECK_PMT!(!str.is_null() || !(*ctx).is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage == 0 {
        return strtok_s(
            str as *mut i8,
            delim as *const i8,
            ctx as *mut *mut i8,
        ) as *mut u8;
    }

    let mut str = if str.is_null() { *ctx } else { str };

    loop {
        c = _mbsnextc(str);
        if c == 0 || _mbschr(delim, c).is_null() {
            break;
        }
        str = str.add(if c > 255 {
            2
        } else {
            1
        });
    }

    if *str == 0 {
        *ctx = str;
        return core::ptr::null_mut();
    }

    *ctx = str.add(if c > 255 {
        2
    } else {
        1
    });

    loop {
        c = _mbsnextc(*ctx);
        if c == 0 || !_mbschr(delim, c).is_null() {
            break;
        }
        *ctx = (*ctx).add(if c > 255 {
            2
        } else {
            1
        });
    }

    if **ctx != 0 {
        **ctx = 0;
        *ctx = (*ctx).add(1);
        if c > 255 {
            **ctx = 0;
            *ctx = (*ctx).add(1);
        }
    }

    str
}

#[no_mangle]
pub unsafe extern "C" fn _mbstok_s(
    str: *mut u8,
    delim: *const u8,
    ctx: *mut *mut u8,
) -> *mut u8 {
    _mbstok_s_l(str, delim, ctx, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbstok_l(
    str: *mut u8,
    delim: *const u8,
    locale: _locale_t,
) -> *mut u8 {
    _mbstok_s_l(str, delim, core::ptr::addr_of_mut!((*msvcrt_get_thread_data()).mbstok_next), locale)
}

#[no_mangle]
pub unsafe extern "C" fn _mbstok(
    str: *mut u8,
    delim: *const u8,
) -> *mut u8 {
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    if str.is_null() && (*data).mbstok_next.is_null() {
        return core::ptr::null_mut();
    }

    _mbstok_s_l(str, delim, core::ptr::addr_of_mut!((*data).mbstok_next), core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbbtombc_l(c: core::ffi::c_uint, locale: _locale_t) -> core::ffi::c_uint {
    let mbcinfo: pthreadmbcinfo;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).mbcodepage == 932 {
        if c >= 0x20 && c <= 0x7e {
            if (c >= 0x41 && c <= 0x5a) || (c >= 0x61 && c <= 0x7a) || (c >= 0x30 && c <= 0x39) {
                return (mbbtombc_932[(c - 0x20) as usize] as core::ffi::c_uint) | 0x8200;
            } else {
                return (mbbtombc_932[(c - 0x20) as usize] as core::ffi::c_uint) | 0x8100;
            }
        } else if c >= 0xa1 && c <= 0xdf {
            if c >= 0xa6 && c <= 0xdd && c != 0xb0 {
                return (mbbtombc_932[(c - 0xa1 + 0x5f) as usize] as core::ffi::c_uint) | 0x8300;
            } else {
                return (mbbtombc_932[(c - 0xa1 + 0x5f) as usize] as core::ffi::c_uint) | 0x8100;
            }
        }
    }
    return c;
}

#[no_mangle]
pub unsafe extern "C" fn _mbbtombc(c: u32) -> u32 {
    _mbbtombc_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbbkana_l(c: core::ffi::c_uint, locale: _locale_t) -> core::ffi::c_int {
    let mbcinfo: pthreadmbcinfo;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).mbcodepage == 932 {
        return (c >= 0xa1 && c <= 0xdf) as core::ffi::c_int;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _ismbbkana(c: u32) -> i32 {
    _ismbbkana_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcdigit_l(ch: u32, locale: _locale_t) -> i32 {
    return _iswdigit_l(msvcrt_mbc_to_wc_l(ch, locale), locale);
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcdigit(ch: u32) -> i32 {
    _ismbcdigit_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcgraph_l(ch: u32, locale: _locale_t) -> i32 {
    return _iswgraph_l(msvcrt_mbc_to_wc_l(ch, locale), locale);
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcgraph(ch: u32) -> i32 {
    _ismbcgraph_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcalpha_l(ch: u32, locale: _locale_t) -> i32 {
    return _iswalpha_l(msvcrt_mbc_to_wc_l(ch, locale), locale);
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcalpha(ch: u32) -> i32 {
    _ismbcalpha_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbclower_l(ch: u32, locale: _locale_t) -> i32 {
    return _iswlower_l(msvcrt_mbc_to_wc_l(ch, locale), locale);
}

#[no_mangle]
pub unsafe extern "C" fn _ismbclower(ch: u32) -> i32 {
    _ismbclower_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcupper_l(ch: u32, locale: _locale_t) -> i32 {
    _iswupper_l(msvcrt_mbc_to_wc_l(ch, locale), locale)
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcupper(ch: u32) -> i32 {
    _ismbcupper_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcsymbol_l(ch: u32, locale: _locale_t) -> i32 {
    let wch: wchar_t = msvcrt_mbc_to_wc_l(ch, locale);
    let mut ctype: u16 = 0;
    if GetStringTypeW(CT_CTYPE3, &wch, 1, &mut ctype) == 0 {
        return 0;
    }
    ((ctype & (C3_SYMBOL) as u16) != 0) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcsymbol(ch: u32) -> i32 {
    _ismbcsymbol_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcalnum_l(ch: u32, locale: _locale_t) -> i32 {
    return _iswalnum_l(msvcrt_mbc_to_wc_l(ch, locale) as u16, locale);
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcalnum(ch: u32) -> i32 {
    _ismbcalnum_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcspace_l(ch: u32, locale: _locale_t) -> i32 {
    return _iswspace_l(msvcrt_mbc_to_wc_l(ch, locale), locale);
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcspace(ch: u32) -> i32 {
    _ismbcspace_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcprint_l(ch: u32, locale: _locale_t) -> i32 {
    return _iswprint_l(msvcrt_mbc_to_wc_l(ch, locale), locale);
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcprint(ch: u32) -> i32 {
    _ismbcprint_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcpunct_l(ch: u32, locale: _locale_t) -> i32 {
    return _iswpunct_l(msvcrt_mbc_to_wc_l(ch, locale), locale);
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcpunct(ch: u32) -> i32 {
    _ismbcpunct_l(ch, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbchira_l(c: core::ffi::c_uint, locale: _locale_t) -> core::ffi::c_int {
    let mbcinfo: pthreadmbcinfo;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).mbcodepage == 932 {
        return (c >= 0x829f && c <= 0x82f1) as core::ffi::c_int;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _ismbchira(c: u32) -> i32 {
    _ismbchira_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbckata_l(c: u32, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).mbcodepage == 932 {
        return (c >= 0x8340 && c <= 0x8396 && c != 0x837f) as i32;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _ismbckata(c: u32) -> i32 {
    _ismbckata_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbblead_l(c: u32, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    ((*mbcinfo).mbctype[((c & 0xff) + 1) as usize] & (_M1) as u8 != 0) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _ismbblead(c: u32) -> i32 {
    _ismbblead_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbbtrail_l(c: u32, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    ((*mbcinfo).mbctype[((c & 0xff) + 1) as usize] & (_M2) as u8 != 0) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _ismbbtrail(c: u32) -> i32 {
    _ismbbtrail_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbclegal_l(c: u32, locale: _locale_t) -> i32 {
    if _ismbblead_l(HIBYTE!(c), locale) != 0 && _ismbbtrail_l(LOBYTE!(c), locale) != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn _ismbclegal(c: u32) -> i32 {
    _ismbclegal_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbslead_l(start: *const u8, str: *const u8, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;
    let mut lead: i32 = 0;

    if MSVCRT_CHECK_PMT!(!start.is_null() && !str.is_null()) == 0 {
        return 0;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage == 0 {
        return 0;
    }

    let mut current = start;
    while current <= str {
        if *current == 0 {
            return 0;
        }
        lead = if lead == 0 && _ismbblead_l(*current as u32, locale) != 0 { 1 } else { 0 };
        current = current.add(1);
    }

    if lead != 0 { -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn _ismbslead(start: *const u8, str: *const u8) -> i32 {
    _ismbslead_l(start, str, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbstrail_l(start: *const u8, str: *const u8, locale: _locale_t) -> i32 {
    if MSVCRT_CHECK_PMT!(!start.is_null() && !str.is_null()) == 0 {
        return 0;
    }

    if (str > start) && _ismbslead_l(start, str.offset(-(1 as isize)), locale) != 0 {
        return -1;
    } else {
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn _ismbstrail(start: *const u8, str: *const u8) -> i32 {
    _ismbstrail_l(start, str, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsdec_l(
    start: *const u8,
    cur: *const u8,
    locale: _locale_t) -> *mut u8 {
    let mbcinfo: pthreadmbcinfo;

    if MSVCRT_CHECK_PMT!(start.is_null() as i32 == 0 && cur.is_null() as i32 == 0) == 0 {
        return core::ptr::null_mut();
    }
    if start >= cur {
        return core::ptr::null_mut();
    }

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if (*mbcinfo).ismbcodepage != 0 {
        return (if _ismbstrail_l(start, cur.offset(-(1 as isize)), locale) != 0 {
            cur.offset(-(2 as isize))
        } else {
            cur.offset(-(1 as isize))
        }) as *mut u8;
    }

    cur.offset(-(1 as isize)) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn _mbsdec(start: *const u8, cur: *const u8) -> *mut u8 {
    _mbsdec_l(start, cur, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbbtype_l(c: u8, r#type: i32, locale: _locale_t) -> i32 {
    if r#type == 1 {
        if _ismbbtrail_l(c as u32, locale) != 0 {
            (_MBC_TRAIL) as i32
        } else {
            _MBC_ILLEGAL
        }
    } else {
        if _ismbblead_l(c as u32, locale) != 0 {
            (_MBC_LEAD) as i32
        } else if _isprint_l(c as i32, locale) != 0 {
            (_MBC_SINGLE) as i32
        } else {
            _MBC_ILLEGAL
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbbtype(c: u8, r#type: i32) -> i32 {
    _mbbtype_l(c, r#type, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsbtype_l(str: *const u8, count: usize, locale: _locale_t) -> i32 {
    let mut lead = 0;
    let mbcinfo: pthreadmbcinfo;
    let end = str.add(count);

    if MSVCRT_CHECK_PMT!(!str.is_null()) == 0 {
        return _MBC_ILLEGAL;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    let mut str_ptr = str;
    while str_ptr < end {
        if *str_ptr == 0 {
            return _MBC_ILLEGAL;
        }
        lead = ((*mbcinfo).ismbcodepage != 0 && lead == 0 && _ismbblead_l(*str_ptr as u32, locale) != 0) as i32;
        str_ptr = str_ptr.add(1);
    }

    if lead != 0 {
        if _ismbbtrail_l(*str_ptr as u32, locale) != 0 {
            return (_MBC_TRAIL) as i32;
        } else {
            return _MBC_ILLEGAL;
        }
    } else {
        if _ismbblead_l(*str_ptr as u32, locale) != 0 {
            return (_MBC_LEAD) as i32;
        } else {
            return (_MBC_SINGLE) as i32;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbsbtype(str: *const u8, count: size_t) -> i32 {
    _mbsbtype_l(str, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsset_l(
    str: *mut u8,
    c: u32,
    locale: _locale_t) -> *mut u8 {
    let ret = str;
    let mbcinfo: pthreadmbcinfo;

    if MSVCRT_CHECK_PMT!(!str.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage == 0 || c < 256 {
        return u__strset(str, c as u8);
    }

    let c = c & 0xffff;

    let mut str = str;
    while *str.offset(0) != 0 && *str.offset(1) != 0 {
        *str = (c >> 8) as u8;
        str = str.offset(1);
        *str = (c & 0xff) as u8;
        str = str.offset(1);
    }
    if *str.offset(0) != 0 {
        *str.offset(0) = b'\0';
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mbsset(str: *mut u8, c: u32) -> *mut u8 {
    _mbsset_l(str, c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbset_l(str: *mut u8, c: u32, len: usize, locale: _locale_t) -> *mut u8 {
    let ret = str;
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return ret;
    }
    if MSVCRT_CHECK_PMT!(!str.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage == 0 || c < 256 {
        return u__strnset(str, c as u8, len);
    }

    let c = c & 0xffff;

    let mut str = str;
    let mut len = len;
    while *str.offset(0) != 0 && *str.offset(1) != 0 && len > 1 {
        *str = (c >> 8) as u8;
        str = str.offset(1);
        len -= 1;
        *str = (c & 0xff) as u8;
        str = str.offset(1);
        len -= 1;
    }
    if len != 0 && *str.offset(0) != 0 {
        *str.offset(0) = b' ';
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbset(str: *mut u8, c: u32, len: size_t) -> *mut u8 {
    _mbsnbset_l(str, c, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnset_l(
    str: *mut u8,
    c: u32,
    len: usize,
    locale: _locale_t) -> *mut u8 {
    let ret: *mut u8 = str;
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return ret;
    }
    if MSVCRT_CHECK_PMT!(!str.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage == 0 || c < 256 {
        return u__strnset(str, c as u8, len);
    }

    let c = c & 0xffff;

    let mut str_ptr = str;
    let mut len = len;
    // C: `while (str[0] && str[1] && len--)`
    //
    // The post-decrement is INSIDE the condition and is reached only after both byte
    // tests pass, so when len is 0 the loop exits having ALREADY wrapped it to
    // SIZE_MAX. The `if (len && str[0])` below then sees a non-zero len and truncates.
    // A `len > 0` guard with a separate `len -= 1` reads like the same loop and is not:
    // it leaves len at 0, the truncation never fires, and the string keeps its tail.
    loop {
        if *str_ptr.offset(0) == 0 || *str_ptr.offset(1) == 0 {
            break;
        }
        let before = len;
        len = len.wrapping_sub(1);
        if before == 0 {
            break;
        }
        *str_ptr = (c >> 8) as u8;
        str_ptr = str_ptr.offset(1);
        *str_ptr = (c & 0xff) as u8;
        str_ptr = str_ptr.offset(1);
    }
    if len != 0 && *str_ptr.offset(0) != 0 {
        *str_ptr.offset(0) = b'\0';
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnset(str: *mut u8, c: u32, len: size_t) -> *mut u8 {
    _mbsnset_l(str, c, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnccnt_l(str: *const u8, len: usize, locale: _locale_t) -> usize {
    let mut ret: usize;
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(!str.is_null()) == 0 {
        return 0;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        ret = 0;
        let mut str_ptr = str;
        let mut len_mut = len;
        while *str_ptr != 0 && len_mut > 0 {
            len_mut -= 1;
            if _ismbblead_l(*str_ptr as u32, locale) != 0 {
                if len_mut == 0 {
                    break;
                }
                len_mut -= 1;
                str_ptr = str_ptr.add(1);
            }
            str_ptr = str_ptr.add(1);
            ret += 1;
        }
        return ret;
    }
    ret = u_strlen(str);
    return min!(ret, len);
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnccnt(str: *const u8, len: size_t) -> size_t {
    _mbsnccnt_l(str, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcnt_l(str: *const u8, len: usize, locale: _locale_t) -> usize {
    let ret: usize;
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(!str.is_null()) == 0 {
        return 0;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }
    if (*mbcinfo).ismbcodepage != 0 {
        let mut xstr: *const u8 = str;
        let mut len_mut = len;
        while *xstr != 0 && len_mut > 0 {
            len_mut = len_mut.wrapping_sub(1);
            if _ismbblead_l(*xstr as u32, locale) != 0 {
                xstr = xstr.add(1);
                xstr = xstr.add(1);
            } else {
                xstr = xstr.add(1);
            }
        }
        return xstr.offset_from(str) as usize;
    }
    ret = u_strlen(str);
    return min!(ret, len);
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcnt(str: *const u8, len: size_t) -> size_t {
    _mbsnbcnt_l(str, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcat_l(dst: *mut u8, src: *const u8, len: usize, locale: _locale_t) -> *mut u8 {
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return dst;
    }

    if MSVCRT_CHECK_PMT!(dst as usize != 0 && src as usize != 0) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let res = dst;
        let mut dst_ptr = dst;

        while *dst_ptr != 0 {
            if _ismbblead_l(*dst_ptr as u32, locale) != 0 {
                dst_ptr = dst_ptr.add(1);
                if *dst_ptr != 0 {
                    dst_ptr = dst_ptr.add(1);
                } else {
                    dst_ptr = dst_ptr.sub(1);
                    break;
                }
            } else {
                dst_ptr = dst_ptr.add(1);
            }
        }

        let mut len_remaining = len;
        let mut src_ptr = src;
        while *src_ptr != 0 && len_remaining > 0 {
            *dst_ptr = *src_ptr;
            dst_ptr = dst_ptr.add(1);
            src_ptr = src_ptr.add(1);
            len_remaining -= 1;
        }
        *dst_ptr = 0;
        return res;
    }
    u_strncat(dst, src, len)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcat(dst: *mut u8, src: *const u8, len: size_t) -> *mut u8 {
    _mbsnbcat_l(dst, src, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcat_s_l(
    dst: *mut u8,
    size: usize,
    mut src: *const u8,
    len: usize,
    locale: _locale_t) -> i32 {
    let mut ptr: *mut u8 = dst;
    let mut i: usize;
    let mbcinfo: pthreadmbcinfo;
    let mut size = size;

    if dst.is_null() && size == 0 && len == 0 {
        return 0;
    }

    if MSVCRT_CHECK_PMT!(!dst.is_null() && size != 0 && !src.is_null()) == 0 {
        if !dst.is_null() && size != 0 {
            *dst = b'\0';
        }
        return (EINVAL) as i32;
    }

    while size != 0 && *ptr != 0 {
        size -= 1;
        ptr = ptr.add(1);
    }

    if size == 0 {
        *dst = b'\0';
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if ptr != dst && (*mbcinfo).ismbcodepage != 0 && _ismbblead_l(*ptr.sub(1) as u32, locale) != 0 {
        size += 1;
        ptr = ptr.sub(1);
    }

    i = 0;
    while *src != 0 && i < len {
        *ptr = *src;
        ptr = ptr.add(1);
        src = src.add(1);
        size -= 1;

        if size == 0 {
            *dst = b'\0';
            *_errno() = (ERANGE) as i32;
            return (ERANGE) as i32;
        }

        i += 1;
    }

    *ptr = b'\0';
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbsnbcat_s(
    dst: *mut u8,
    size: size_t,
    src: *const u8,
    len: size_t,
) -> i32 {
    _mbsnbcat_s_l(dst, size, src, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncat_l(dst: *mut u8, src: *const u8, len: usize, locale: _locale_t) -> *mut u8 {
    let mbcinfo: pthreadmbcinfo;

    if len == 0 {
        return dst;
    }

    if MSVCRT_CHECK_PMT!(!dst.is_null() && !src.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let res = dst;
        let mut dst_ptr = dst;
        while *dst_ptr != 0 {
            if _ismbblead_l(*dst_ptr as u32, locale) != 0 {
                dst_ptr = dst_ptr.add(1);
            }
            dst_ptr = dst_ptr.add(1);
        }
        let mut src_ptr = src;
        let mut remaining = len;
        while *src_ptr != 0 && remaining > 0 {
            *dst_ptr = *src_ptr;
            dst_ptr = dst_ptr.add(1);
            if _ismbblead_l(*src_ptr as u32, locale) != 0 {
                src_ptr = src_ptr.add(1);
                *dst_ptr = *src_ptr;
                dst_ptr = dst_ptr.add(1);
                src_ptr = src_ptr.add(1);
            } else {
                src_ptr = src_ptr.add(1);
            }
            remaining = remaining.wrapping_sub(1);
        }
        *dst_ptr = b'\0';
        return res;
    }
    u_strncat(dst, src, len)
}

#[no_mangle]
pub unsafe extern "C" fn _mbsncat(dst: *mut u8, src: *const u8, len: size_t) -> *mut u8 {
    _mbsncat_l(dst, src, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbslwr_l(s: *mut u8, locale: _locale_t) -> *mut u8 {
    let mbcinfo: pthreadmbcinfo;
    let ret: *mut u8 = s;

    if s.is_null() {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let mut c: u32;
        let mut ptr = s;

        while *ptr != 0 {
            c = _mbctolower_l(_mbsnextc_l(ptr, locale), locale);
            if c > 255 {
                *ptr = (c >> 8) as u8;
                ptr = ptr.add(1);
                c = c & 0xff;
            }
            *ptr = c as u8;
            ptr = ptr.add(1);
        }
    } else {
        let mut ptr = s;
        while *ptr != 0 {
            *ptr = _tolower_l(*ptr as i32, locale) as u8;
            ptr = ptr.add(1);
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mbslwr(s: *mut u8) -> *mut u8 {
    _mbslwr_l(s, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbslwr_s_l(s: *mut u8, len: usize, locale: _locale_t) -> i32 {
    let p: *mut u8 = s;
    let mbcinfo: pthreadmbcinfo;

    if s.is_null() && len == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(!s.is_null() && len != 0) == 0 {
        return (EINVAL) as i32;
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    // `s` and `len` are the C's own parameters, and the loops below MUTATE them — the
    // two statements after the loops read what the loops left behind. Walking a local
    // copy instead leaves the originals untouched, so `len` is never 0 (the buffer-too-
    // small EINVAL becomes unreachable) and `*s = 0` writes the terminator at the START
    // of the string rather than after the converted text.
    let mut s = s;
    let mut len = len;

    if (*mbcinfo).ismbcodepage != 0 {
        let mut c: u32;
        while *s != 0 && len > 0 {
            c = _mbctolower_l(_mbsnextc_l(s, locale), locale);
            /* Note that I assume that the size of the character is unchanged */
            if c > 255 {
                *s = (c >> 8) as u8;
                s = s.add(1);
                c = c & 0xff;
            }
            *s = c as u8;
            s = s.add(1);
            len -= 1;
        }
    } else {
        while *s != 0 && len > 0 {
            *s = _tolower_l(*s as i32, locale) as u8;
            s = s.add(1);
            len -= 1;
        }
    }

    if MSVCRT_CHECK_PMT!(len != 0) == 0 {
        *p = 0;
        return (EINVAL) as i32;
    }
    *s = 0;
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _mbslwr_s(str: *mut u8, len: size_t) -> i32 {
    _mbslwr_s_l(str, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsupr_l(s: *mut u8, locale: _locale_t) -> *mut u8 {
    let ret: *mut u8 = s;
    let mbcinfo: pthreadmbcinfo;

    if MSVCRT_CHECK_PMT!(!s.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage != 0 {
        let mut c: u32;
        let mut p: *mut u8 = s;
        while *p != 0 {
            c = _mbctoupper_l(_mbsnextc_l(p, locale), locale);
            if c > 255 {
                *p = (c >> 8) as u8;
                p = p.offset(1);
                c = c & 0xff;
            }
            *p = c as u8;
            p = p.offset(1);
        }
    } else {
        let mut p: *mut u8 = s;
        while *p != 0 {
            *p = _toupper_l(*p as i32, locale) as u8;
            p = p.offset(1);
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mbsupr(s: *mut u8) -> *mut u8 {
    _mbsupr_l(s, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsupr_s_l(s: *mut u8, len: usize, locale: _locale_t) -> i32 {
    let p: *mut u8 = s;

    if s.is_null() && len == 0 {
        return 0;
    }
    if MSVCRT_CHECK_PMT!(!s.is_null() && len != 0) == 0 {
        return (EINVAL) as i32;
    }

    // Mutate the PARAMETERS, as the C does — the two statements after the loops read
    // what the loops left behind. See the same note on `_mbslwr_s_l`.
    //
    // `get_mbcinfo()` rather than the `locale` argument is the C's own behaviour here,
    // and it disagrees with its `_mbslwr_s_l` twin. Upstream inconsistency, faithfully
    // reproduced — not ours to fix.
    let mut s = s;
    let mut len = len;

    if (*get_mbcinfo()).ismbcodepage != 0 {
        let mut c: u32;
        while *s != 0 && len > 0 {
            c = _mbctoupper_l(_mbsnextc_l(s, locale), locale);
            /* Note that I assume that the size of the character is unchanged */
            if c > 255 {
                *s = (c >> 8) as u8;
                s = s.offset(1);
                c = c & 0xff;
            }
            *s = c as u8;
            s = s.offset(1);
            len -= 1;
        }
    } else {
        while *s != 0 && len > 0 {
            *s = _toupper_l(*s as i32, locale) as u8;
            s = s.offset(1);
            len -= 1;
        }
    }

    if MSVCRT_CHECK_PMT!(len != 0) == 0 {
        *p = 0;
        return (EINVAL) as i32;
    }
    *s = 0;
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _mbsupr_s(s: *mut u8, len: size_t) -> i32 {
    _mbsupr_s_l(s, len, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsspn_l(
    string: *const u8,
    set: *const u8,
    locale: _locale_t) -> usize {
    let mut p: *const u8;
    let mut q: *const u8;

    if MSVCRT_CHECK_PMT!(!string.is_null() && !set.is_null()) == 0 {
        return 0;
    }

    p = string;
    while *p != 0 {
        q = set;
        while *q != 0 {
            if _ismbblead_l(*q as u32, locale) != 0 {
                if *q.offset(1) == 0 {
                    break;
                }

                if *p.offset(0) == *q.offset(0)
                    && *p.offset(1) == *q.offset(1)
                {
                    p = p.offset(1);
                    break;
                }
                q = q.offset(1);
            } else {
                if *p.offset(0) == *q.offset(0) {
                    break;
                }
            }
            q = q.offset(1);
        }
        if *q == 0 {
            break;
        }
        p = p.offset(1);
    }
    p.offset_from(string) as usize
}

#[no_mangle]
pub unsafe extern "C" fn _mbsspn(string: *const u8, set: *const u8) -> size_t {
    _mbsspn_l(string, set, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsspnp_l(
    string: *const u8,
    set: *const u8,
    locale: _locale_t) -> *mut u8 {
    if MSVCRT_CHECK_PMT!(!string.is_null() && !set.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    let string = string.add(_mbsspn_l(string, set, locale));
    if *string != 0 {
        string as *mut u8
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn _mbsspnp(string: *const u8, set: *const u8) -> *mut u8 {
    _mbsspnp_l(string, set, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbscspn_l(
    str: *const u8,
    cmp: *const u8,
    locale: _locale_t) -> usize {
    let mut p = str;
    while *p != 0 {
        let mut q = cmp;
        while *q != 0 {
            if _ismbblead_l(*q as u32, locale) != 0 {
                if *q.offset(1) == 0 {
                    return 0;
                }

                if *p.offset(0) == *q.offset(0)
                    && *p.offset(1) == *q.offset(1)
                {
                    return p.offset_from(str) as usize;
                }
                q = q.offset(1);
            } else if *p.offset(0) == *q.offset(0) {
                return p.offset_from(str) as usize;
            }
            q = q.offset(1);
        }
        p = p.offset(1);
    }
    p.offset_from(str) as usize
}

#[no_mangle]
pub unsafe extern "C" fn _mbscspn(str: *const u8, cmp: *const u8) -> size_t {
    _mbscspn_l(str, cmp, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbsrev_l(str: *mut u8, locale: _locale_t) -> *mut u8 {
    let mbcinfo: pthreadmbcinfo;
    let mut p: *mut u8;
    let mut tmp: u8;

    if MSVCRT_CHECK_PMT!(!str.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage == 0 {
        return u__strrev(str);
    }

    p = str;
    while *p != 0 {
        if _ismbblead_l(*p as u32, locale) != 0 {
            if *p.offset(1) != 0 {
                tmp = *p.offset(0);
                *p.offset(0) = *p.offset(1);
                *p.offset(1) = tmp;
                p = p.offset(1);
            } else {
                *p.offset(0) = 0;
            }
        }
        p = p.offset(1);
    }
    return u__strrev(str);
}

#[no_mangle]
pub unsafe extern "C" fn _mbsrev(str: *mut u8) -> *mut u8 {
    _mbsrev_l(str, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbspbrk_l(
    str: *const ::core::ffi::c_uchar,
    accept: *const ::core::ffi::c_uchar,
    locale: _locale_t) -> *mut ::core::ffi::c_uchar {
    let mut p: *const ::core::ffi::c_uchar;
    let mbcinfo: pthreadmbcinfo;

    if !locale.is_null() {
        mbcinfo = (*locale).mbcinfo;
    } else {
        mbcinfo = get_mbcinfo();
    }

    if (*mbcinfo).ismbcodepage == 0 {
        return u_strpbrk(str, accept);
    }

    if MSVCRT_CHECK_PMT!(!str.is_null() && !accept.is_null()) == 0 {
        return core::ptr::null_mut();
    }

    let mut str_ptr = str;
    while *str_ptr != 0 {
        p = accept;
        while *p != 0 {
            if *p == *str_ptr {
                if _ismbblead_l(*p as u32, locale) == 0
                    || *p.offset(1) == *str_ptr.offset(1)
                {
                    return str_ptr as *mut ::core::ffi::c_uchar;
                }
            }
            p = p.offset(if _ismbblead_l(*p as u32, locale) != 0 {
                2
            } else {
                1
            });
        }
        str_ptr = str_ptr.offset(if _ismbblead_l(*str_ptr as u32, locale) != 0 {
            2
        } else {
            1
        });
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn _mbspbrk(str: *const u8, accept: *const u8) -> *mut u8 {
    _mbspbrk_l(str, accept, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mblen_l(str: *const i8, size: usize, locale: _locale_t) -> i32 {
    let locinfo: pthreadlocinfo;

    if str.is_null() || *str == 0 || size == 0 {
        return 0;
    }

    if !locale.is_null() {
        locinfo = (*locale).locinfo;
    } else {
        locinfo = get_locinfo();
    }

    if (*locinfo).mb_cur_max == 1 {
        return 1;
    }
    if _isleadbyte_l((*str as u8) as i32, locale) == 0 {
        1
    } else {
        if size > 1 {
            2
        } else {
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mblen(str: *const i8, size: size_t) -> i32 {
    _mblen_l(str, size, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn mbrlen(str: *const i8, len: usize, state: *mut mbstate_t) -> usize {
    let mut s: mbstate_t = if !state.is_null() {
        *state
    } else {
        0
    };
    let ret: usize;

    if len == 0 || str.is_null() || *str == 0 {
        return 0;
    }

    if (*get_locinfo()).mb_cur_max == 1 {
        return 1;
    } else if s == 0 && isleadbyte((*str as u8) as i32) != 0 {
        if len == 1 {
            s = *str as u8 as mbstate_t;
            ret = 0usize.wrapping_sub(2);
        } else {
            ret = 2;
        }
    } else if s == 0 {
        ret = 1;
    } else {
        s = 0;
        ret = 2;
    }

    if !state.is_null() {
        *state = s;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _mbstrlen_l(str: *const i8, locale: _locale_t) -> usize {
    let locinfo: pthreadlocinfo;

    if locale.is_null() {
        locinfo = get_locinfo();
    } else {
        locinfo = (*locale).locinfo;
    }

    if (*locinfo).mb_cur_max > 1 {
        let len: usize;
        len = (MultiByteToWideChar(
            (*locinfo).lc_codepage,
            MB_ERR_INVALID_CHARS,
            str,
            -1,
            core::ptr::null_mut(),
            0,
        )) as usize;
        if len == 0 {
            *_errno() = (EILSEQ) as i32;
            return (1 as i32).wrapping_neg() as usize;
        }
        return len - 1;
    }

    strlen(str)
}

#[no_mangle]
pub unsafe extern "C" fn _mbstrlen(str: *const i8) -> size_t {
    _mbstrlen_l(str, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbtowc_l(
    dst: *mut wchar_t,
    str: *const i8,
    n: usize,
    locale: _locale_t,
) -> i32 {
    let locinfo: pthreadlocinfo;
    let mut tmpdst: wchar_t = 0;

    if locale.is_null() {
        locinfo = get_locinfo();
    } else {
        locinfo = (*locale).locinfo;
    }

    if n <= 0 || str.is_null() {
        return 0;
    }

    if *str == 0 {
        if !dst.is_null() {
            *dst = 0;
        }
        return 0;
    }

    if (*locinfo).lc_codepage == 0 {
        if !dst.is_null() {
            *dst = (*str as u8) as wchar_t;
        }
        return 1;
    }

    if n >= 2 && _isleadbyte_l((*str as u8) as i32, locale) != 0 {
        if MultiByteToWideChar(
            (*locinfo).lc_codepage,
            0,
            str,
            2,
            &mut tmpdst,
            1,
        ) == 0
        {
            return -1;
        }
        if !dst.is_null() {
            *dst = tmpdst;
        }
        return 2;
    }

    if MultiByteToWideChar(
        (*locinfo).lc_codepage,
        0,
        str,
        1,
        &mut tmpdst,
        1,
    ) == 0
    {
        return -1;
    }

    if !dst.is_null() {
        *dst = tmpdst;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn mbtowc(dst: *mut wchar_t, str: *const i8, n: size_t) -> i32 {
    _mbtowc_l(dst, str, n, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn btowc(c: i32) -> wint_t {
    let letter: u8 = c as u8;
    let mut ret: wchar_t = 0;

    if c == EOF {
        return WEOF;
    }
    if (*get_locinfo()).lc_codepage == 0 {
        return (c & 255) as wint_t;
    }
    if MultiByteToWideChar(
        (*get_locinfo()).lc_codepage,
        MB_ERR_INVALID_CHARS,
        core::ptr::addr_of!(letter) as LPCSTR,
        1,
        core::ptr::addr_of_mut!(ret),
        1,
    ) == 0
    {
        return WEOF;
    }

    ret as wint_t
}

#[no_mangle]
pub unsafe extern "C" fn mbrtowc(
    dst: *mut wchar_t,
    str: *const i8,
    n: usize,
    state: *mut mbstate_t,
) -> usize {
    let locinfo: pthreadlocinfo = get_locinfo();
    let mut s: mbstate_t = if !state.is_null() {
        *state
    } else {
        0
    };
    let mut tmpstr: [i8; 2] = [0; 2];
    let mut len: i32 = 0;

    if !dst.is_null() {
        *dst = 0;
    }

    if n == 0 || str.is_null() || *str == 0 {
        return 0;
    }

    if (*locinfo).mb_cur_max == 1 {
        tmpstr[len as usize] = *str;
        len += 1;
    } else if s == 0 && isleadbyte((*str as u8) as i32) != 0 {
        if n == 1 {
            s = *str as u8 as mbstate_t;
            len = -2;
        } else {
            tmpstr[0] = *str.offset(0);
            tmpstr[1] = *str.offset(1);
            len = 2;
        }
    } else if s == 0 {
        tmpstr[len as usize] = *str;
        len += 1;
    } else {
        tmpstr[0] = s as i8;
        tmpstr[1] = *str;
        len = 2;
        s = 0;
    }

    if len > 0 {
        if MultiByteToWideChar(
            (*locinfo).lc_codepage,
            0,
            tmpstr.as_ptr(),
            len,
            dst,
            if !dst.is_null() {
                1
            } else {
                0
            },
        ) == 0
        {
            len = -1;
        }
    }

    if !state.is_null() {
        *state = s;
    }
    len as usize
}
unsafe extern "C" fn get_utf8_char_len(ch: i8) -> i32 {
    // C promotes `char` to `int` before the mask, so a high-bit byte sign-extends and the
    // comparison still succeeds (0xF0 as a signed char is -16; -16 & 0xF8 == 0xF0 in int).
    if (ch as i32 & 0xf8) == 0xf0 {
        return 4;
    } else if (ch as i32 & 0xf0) == 0xe0 {
        return 3;
    } else if (ch as i32 & 0xe0) == 0xc0 {
        return 2;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn _mbstowcs_l(
    wcstr: *mut wchar_t,
    mbstr: *const i8,
    count: usize,
    locale: _locale_t,
) -> usize {
    let locinfo: pthreadlocinfo;
    let mut i: usize;
    let mut size: usize;

    if mbstr.is_null() {
        *_errno() = (EINVAL) as i32;
        return 0usize.wrapping_sub(1);
    }

    if locale.is_null() {
        locinfo = get_locinfo();
    } else {
        locinfo = (*locale).locinfo;
    }

    if (*locinfo).lc_codepage == 0 {
        if wcstr.is_null() {
            return strlen(mbstr);
        }

        i = 0;
        while i < count {
            *wcstr.add(i) = (*mbstr.add(i) as u8) as wchar_t;
            if *wcstr.add(i) == 0 {
                break;
            }
            i += 1;
        }
        return i;
    }

    if wcstr.is_null() {
        size = (MultiByteToWideChar(
            (*locinfo).lc_codepage,
            MB_ERR_INVALID_CHARS,
            mbstr,
            0i32.wrapping_sub(1),
            core::ptr::null_mut(),
            0,
        )) as usize;
        if size == 0 {
            *_errno() = (EILSEQ) as i32;
            return 0usize.wrapping_sub(1);
        }
        return size.wrapping_sub(1);
    }

    i = 0;
    size = 0;
    while i < count {
        if *mbstr.add(size) == b'\0' as i8 {
            break;
        }

        if (*locinfo).lc_codepage == CP_UTF8 {
            let chlen: i32 = get_utf8_char_len(*mbstr.add(size));
            let mut j: i32 = 1;
            while j < chlen {
                if *mbstr.add(size + j as usize) == 0 {
                    if count != 0 {
                        *wcstr.add(0) = b'\0' as wchar_t;
                    }
                    *_errno() = (EILSEQ) as i32;
                    return 0usize.wrapping_sub(1);
                }
                j += 1;
            }
            size += chlen as usize;
        } else {
            size += if _isleadbyte_l((*mbstr.add(size) as u8) as i32, locale) != 0 {
                2
            } else {
                1
            };
        }

        i += 1;
    }

    if size != 0 {
        size = (MultiByteToWideChar(
            (*locinfo).lc_codepage,
            MB_ERR_INVALID_CHARS,
            mbstr,
            size as i32,
            wcstr,
            count as i32,
        )) as usize;
        if size == 0 {
            if count != 0 {
                *wcstr.add(0) = b'\0' as wchar_t;
            }
            *_errno() = (EILSEQ) as i32;
            return 0usize.wrapping_sub(1);
        }
    }

    if size < count {
        *wcstr.add(size) = b'\0' as wchar_t;
    }

    size
}

#[no_mangle]
pub unsafe extern "C" fn mbstowcs(
    wcstr: *mut wchar_t,
    mbstr: *const i8,
    count: size_t,
) -> size_t {
    _mbstowcs_l(wcstr, mbstr, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbstowcs_s_l(
    ret: *mut usize,
    wcstr: *mut wchar_t,
    size: usize,
    mbstr: *const i8,
    count: usize,
    locale: _locale_t,
) -> i32 {
    let mut conv: usize;
    let mut err: i32 = 0;

    if wcstr.is_null() && size == 0 {
        conv = _mbstowcs_l(core::ptr::null_mut(), mbstr, 0, locale);
        if !ret.is_null() {
            *ret = conv + 1;
        }
        return 0;
    }

    if MSVCRT_CHECK_PMT!(!wcstr.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!mbstr.is_null()) == 0 {
        if size != 0 {
            *wcstr.add(0) = 0;
        }
        return (EINVAL) as i32;
    }

    if count == _TRUNCATE || size < count {
        conv = size;
    } else {
        conv = count;
    }

    conv = _mbstowcs_l(wcstr, mbstr, conv, locale);
    if conv < size {
        *wcstr.add(conv) = 0;
        conv += 1;
    } else if conv == size && count == _TRUNCATE && *wcstr.add(conv - 1) != 0 {
        *wcstr.add(conv - 1) = 0;
        err = (STRUNCATE) as i32;
    } else if conv == size && *wcstr.add(conv - 1) != 0 {
        MSVCRT_INVALID_PMT!(b"wcstr[size] is too small\0".as_ptr() as *const i8, (ERANGE) as i32);
        if size != 0 {
            *wcstr.add(0) = 0;
        }
        return (ERANGE) as i32;
    }

    if !ret.is_null() {
        *ret = conv;
    }
    err
}

#[no_mangle]
pub unsafe extern "C" fn _mbstowcs_s(
    ret: *mut size_t,
    wcstr: *mut wchar_t,
    size: size_t,
    mbstr: *const i8,
    count: size_t,
) -> i32 {
    _mbstowcs_s_l(ret, wcstr, size, mbstr, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn mbsrtowcs(
    wcstr: *mut wchar_t,
    pmbstr: *mut *const i8,
    count: usize,
    state: *mut mbstate_t,
) -> usize {
    let mut s = if !state.is_null() {
        *state
    } else {
        0
    };
    let mut tmpdst: wchar_t = 0;
    let mut ret: usize = 0;
    let mut p: *const i8;

    if MSVCRT_CHECK_PMT!(!pmbstr.is_null()) == 0 {
        return (1 as usize).wrapping_neg();
    }

    p = *pmbstr;
    while wcstr.is_null() || count > ret {
        let ch_len = mbrtowc(&mut tmpdst, p, 2, &mut s) as i32;
        if !wcstr.is_null() {
            *wcstr.add(ret) = tmpdst;
        }

        if ch_len < 0 {
            return (1 as usize).wrapping_neg();
        } else if ch_len == 0 {
            if !wcstr.is_null() {
                *pmbstr = core::ptr::null();
            }
            return ret;
        }

        p = p.add(ch_len as usize);
        ret += 1;
    }

    if !wcstr.is_null() {
        *pmbstr = p;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn mbsrtowcs_s(
    ret: *mut usize,
    wcstr: *mut wchar_t,
    len: usize,
    mbstr: *mut *const i8,
    count: usize,
    state: *mut mbstate_t,
) -> i32 {
    let mut tmp: usize = 0;
    let ret = if ret.is_null() { &mut tmp } else { &mut *ret };

    if MSVCRT_CHECK_PMT!(!wcstr.is_null() == (len != 0)) == 0 {
        *ret = 0usize.wrapping_sub(1);
        return (EINVAL) as i32;
    }

    *ret = mbsrtowcs(wcstr, mbstr, if count > len { len } else { count }, state);
    if *ret == 0usize.wrapping_sub(1) {
        if !wcstr.is_null() {
            *wcstr = 0;
        }
        return *_errno();
    }
    *ret = (*ret).wrapping_add(1);
    if *ret > len {
        if !wcstr.is_null() {
            *wcstr = 0;
        }
        return 0;
    }
    if !wcstr.is_null() {
        *wcstr.add((*ret).wrapping_sub(1)) = 0;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _mbctohira_l(c: u32, locale: _locale_t) -> u32 {
    if _ismbckata_l(c, locale) != 0 && c <= 0x8393 {
        return (c - 0x8340 - if c >= 0x837f { 1 } else { 0 }) + 0x829f;
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbctohira(c: u32) -> u32 {
    _mbctohira_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _mbctokata_l(c: core::ffi::c_uint, locale: _locale_t) -> core::ffi::c_uint {
    if _ismbchira_l(c, locale) != 0 {
        return (c - 0x829f) + 0x8340 + (if c >= 0x82de { 1 } else { 0 });
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn _mbctokata(c: u32) -> u32 {
    _mbctokata_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcl0_l(c: u32, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if (*mbcinfo).mbcodepage == 932 {
        return ((_ismbclegal_l(c, locale) != 0) && c >= 0x8140 && c <= 0x889e) as i32;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcl0(c: u32) -> i32 {
    _ismbcl0_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcl1_l(c: u32, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if (*mbcinfo).mbcodepage == 932 {
        return ((_ismbclegal_l(c, locale) != 0) && (c >= 0x889f) && (c <= 0x9872)) as i32;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcl1(c: u32) -> i32 {
    _ismbcl1_l(c, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcl2_l(c: u32, locale: _locale_t) -> i32 {
    let mbcinfo: pthreadmbcinfo;

    if locale.is_null() {
        mbcinfo = get_mbcinfo();
    } else {
        mbcinfo = (*locale).mbcinfo;
    }

    if (*mbcinfo).mbcodepage == 932 {
        return if _ismbclegal_l(c, locale) != 0 && c >= 0x989f && c <= 0xeaa4 { 1 } else { 0 };
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _ismbcl2(c: u32) -> i32 {
    _ismbcl2_l(c, core::ptr::null_mut())
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll mbcs functions
//  *
//  * Copyright 1999 Alexandre Julliard
//  * Copyright 2000 Jon Griffths
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  * FIXME
//  * Not currently binary compatible with win32. MSVCRT_mbctype must be
//  * populated correctly and the ismb* functions should reference it.
//  */
//
// #include <stdio.h>
// #include <limits.h>
// #include <mbctype.h>
// #include <mbstring.h>
//
// #include "msvcrt.h"
// #include "mtdll.h"
// #include "winnls.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// unsigned char MSVCRT_mbctype[257] = { 0 };
//
// /* It seems that the data about valid trail bytes is not available from kernel32
//  * so we have to store is here. The format is the same as for lead bytes in CPINFO */
// struct cp_extra_info_t
// {
//     int cp;
//     BYTE TrailBytes[MAX_LEADBYTES];
// };
//
// static struct cp_extra_info_t g_cpextrainfo[] =
// {
//     {932, {0x40, 0x7e, 0x80, 0xfc, 0, 0}},
//     {936, {0x40, 0xfe, 0, 0}},
//     {949, {0x41, 0xfe, 0, 0}},
//     {950, {0x40, 0x7e, 0xa1, 0xfe, 0, 0}},
//     {1361, {0x31, 0x7e, 0x81, 0xfe, 0, 0}},
//     {20932, {1, 255, 0, 0}},  /* seems to give different results on different systems */
//     {0, {1, 255, 0, 0}}       /* match all with FIXME */
// };
//
// /* Maps cp932 single byte character to multi byte character */
// static const unsigned char mbbtombc_932[] = {
//   0x40,0x49,0x68,0x94,0x90,0x93,0x95,0x66,0x69,0x6a,0x96,0x7b,0x43,0x7c,0x44,0x5e,
//   0x4f,0x50,0x51,0x52,0x53,0x54,0x55,0x56,0x57,0x58,0x46,0x47,0x83,0x81,0x84,0x48,
//   0x97,0x60,0x61,0x62,0x63,0x64,0x65,0x66,0x67,0x68,0x69,0x6a,0x6b,0x6c,0x6d,0x6e,
//   0x6f,0x70,0x71,0x72,0x73,0x74,0x75,0x76,0x77,0x78,0x79,0x6d,0x8f,0x6e,0x4f,0x76,
//   0x77,0x78,0x79,0x6d,0x8f,0x6e,0x4f,0x51,0x65,0x81,0x82,0x83,0x84,0x85,0x86,0x87,
//   0x88,0x89,0x8a,0x8b,0x8c,0x8d,0x8e,0x8f,0x90,0x91,0x92,0x93,0x94,0x95,0x50,
//        0x42,0x75,0x76,0x41,0x45,0x92,0x40,0x42,0x44,0x46,0x48,0x83,0x85,0x87,0x62,
//   0x5b,0x41,0x43,0x45,0x47,0x49,0x4a,0x4c,0x4e,0x50,0x52,0x54,0x56,0x58,0x5a,0x5c,
//   0x5e,0x60,0x63,0x65,0x67,0x69,0x6a,0x6b,0x6c,0x6d,0x6e,0x71,0x74,0x77,0x7a,0x7d,
//   0x7e,0x80,0x81,0x82,0x84,0x86,0x88,0x89,0x8a,0x8b,0x8c,0x8d,0x8f,0x93,0x4a,0x4b };
//
// /* Maps multibyte cp932 punctuation marks to single byte equivalents */
// static const unsigned char mbctombb_932_punct[] = {
//   0x20,0xa4,0xa1,0x2c,0x2e,0xa5,0x3a,0x3b,0x3f,0x21,0xde,0xdf,0x00,0x00,0x00,0x5e,
//   0x7e,0x5f,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xb0,0x00,0x00,0x2f,0x00,
//   0x00,0x00,0x7c,0x00,0x00,0x60,0x27,0x00,0x22,0x28,0x29,0x00,0x00,0x5b,0x5d,0x7b,
//   0x7d,0x00,0x00,0x00,0x00,0xa2,0xa3,0x00,0x00,0x00,0x00,0x2b,0x2d,0x00,0x00,0x00,
//   0x00,0x3d,0x00,0x3c,0x3e,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x5c,
//   0x24,0x00,0x00,0x25,0x23,0x26,0x2a,0x40};
//
// /* Maps multibyte cp932 hiragana/katakana to single-byte equivalents */
// static const unsigned char mbctombb_932_kana[] = {
//   0xa7,0xb1,0xa8,0xb2,0xa9,0xb3,0xaa,0xb4,0xab,0xb5,0xb6,0xb6,0xb7,0xb7,0xb8,0xb8,
//   0xb9,0xb9,0xba,0xba,0xbb,0xbb,0xbc,0xbc,0xbd,0xbd,0xbe,0xbe,0xbf,0xbf,0xc0,0xc0,
//   0xc1,0xc1,0xaf,0xc2,0xc2,0xc3,0xc3,0xc4,0xc4,0xc5,0xc6,0xc7,0xc8,0xc9,0xca,0xca,
//   0xca,0xcb,0xcb,0xcb,0xcc,0xcc,0xcc,0xcd,0xcd,0xcd,0xce,0xce,0xce,0xcf,0xd0,0xd1,
//   0xd2,0xd3,0xac,0xd4,0xad,0xd5,0xae,0xd6,0xd7,0xd8,0xd9,0xda,0xdb,0xdc,0xdc,0xb2,
//   0xb4,0xa6,0xdd,0xb3,0xb6,0xb9};
//
// static wchar_t msvcrt_mbc_to_wc_l(unsigned int ch, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//     wchar_t chW;
//     char mbch[2];
//     int n_chars;
//
//     if(locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (ch <= 0xff) {
//         mbch[0] = ch;
//         n_chars = 1;
//     } else {
//         mbch[0] = (ch >> 8) & 0xff;
//         mbch[1] = ch & 0xff;
//         n_chars = 2;
//     }
//     if (!MultiByteToWideChar(mbcinfo->mbcodepage, 0, mbch, n_chars, &chW, 1))
//     {
//         WARN("MultiByteToWideChar failed on %x\n", ch);
//         return 0;
//     }
//     return chW;
// }
//
// static inline size_t u_strlen( const unsigned char *str )
// {
//   return strlen( (const char*) str );
// }
//
// static inline unsigned char* u_strncat( unsigned char* dst, const unsigned char* src, size_t len )
// {
//   return (unsigned char*)strncat( (char*)dst, (const char*)src, len);
// }
//
// static inline int u_strcmp( const unsigned char *s1, const unsigned char *s2 )
// {
//   return strcmp( (const char*)s1, (const char*)s2 );
// }
//
// static inline int u_strcasecmp( const unsigned char *s1, const unsigned char *s2 )
// {
//   return _stricmp( (const char*)s1, (const char*)s2 );
// }
//
// static inline int u_strncmp( const unsigned char *s1, const unsigned char *s2, size_t len )
// {
//   return strncmp( (const char*)s1, (const char*)s2, len );
// }
//
// static inline int u_strncasecmp( const unsigned char *s1, const unsigned char *s2, size_t len )
// {
//   return _strnicmp( (const char*)s1, (const char*)s2, len );
// }
//
// static inline unsigned char *u_strchr( const unsigned char *s, unsigned char x )
// {
//   return (unsigned char*) strchr( (const char*)s, x );
// }
//
// static inline unsigned char* u_strpbrk(const unsigned char *str, const unsigned char *accept)
// {
//   return (unsigned char*)strpbrk((const char*)str, (const char*)accept);
// }
//
// static inline unsigned char *u_strrchr( const unsigned char *s, unsigned char x )
// {
//   return (unsigned char*) strrchr( (const char*)s, x );
// }
//
// static inline unsigned char* u__strrev(unsigned char *str)
// {
//   return (unsigned char*)_strrev((char *)str);
// }
//
// static inline unsigned char *u__strset( unsigned char *s, unsigned char c )
// {
//   return (unsigned char*) _strset( (char*)s, c);
// }
//
// static inline unsigned char *u__strnset( unsigned char *s, unsigned char c, size_t len )
// {
//   return (unsigned char*) _strnset( (char*)s, c, len );
// }
//
// /*********************************************************************
//  *		__p__mbctype (MSVCRT.@)
//  */
// unsigned char* CDECL __p__mbctype(void)
// {
//   return get_mbcinfo()->mbctype;
// }
//
// /*********************************************************************
//  *		__p___mb_cur_max(MSVCRT.@)
//  */
// int* CDECL __p___mb_cur_max(void)
// {
//   return &get_locinfo()->mb_cur_max;
// }
//
// /*********************************************************************
//  *		___mb_cur_max_func(MSVCRT.@)
//  */
// int CDECL ___mb_cur_max_func(void)
// {
//   return get_locinfo()->mb_cur_max;
// }
//
//
// threadmbcinfo* create_mbcinfo(int cp, LCID lcid, threadmbcinfo *old_mbcinfo)
// {
//   threadmbcinfo *mbcinfo;
//   int newcp;
//   CPINFO cpi;
//   BYTE *bytes;
//   WORD chartypes[256];
//   char bufA[256];
//   WCHAR bufW[256], lowW[256], upW[256];
//   int charcount, maxchar;
//   int ret;
//   int i;
//
//   if(old_mbcinfo && cp==old_mbcinfo->mbcodepage
//           && (lcid==-1 || lcid==old_mbcinfo->mblcid)) {
//     InterlockedIncrement(&old_mbcinfo->refcount);
//     return old_mbcinfo;
//   }
//
//   mbcinfo = malloc(sizeof(threadmbcinfo));
//   if(!mbcinfo)
//     return NULL;
//   mbcinfo->refcount = 1;
//
//   switch (cp)
//   {
//     case _MB_CP_ANSI:
//       newcp = GetACP();
//       break;
//     case _MB_CP_OEM:
//       newcp = GetOEMCP();
//       break;
//     case _MB_CP_LOCALE:
//       newcp = get_locinfo()->lc_codepage;
//       if(newcp)
//           break;
//       /* fall through (C locale) */
//     case _MB_CP_SBCS:
//       newcp = 20127;   /* ASCII */
//       break;
//     default:
//       newcp = cp;
//       break;
//   }
//
//   if(lcid == -1) {
//     WCHAR wbuf[LOCALE_NAME_MAX_LENGTH];
//     sprintf(bufA, ".%d", newcp);
//     mbcinfo->mblcid = locale_to_sname(bufA, NULL, NULL, wbuf) ? LocaleNameToLCID(wbuf, LOCALE_ALLOW_NEUTRAL_NAMES) : -1;
//   } else {
//     mbcinfo->mblcid = lcid;
//   }
//
//   if(mbcinfo->mblcid == -1)
//   {
//     WARN("Can't assign LCID to codepage (%d)\n", mbcinfo->mblcid);
//     mbcinfo->mblcid = 0;
//   }
//
//   if (!GetCPInfo(newcp, &cpi))
//   {
//     if (newcp == 20127) {
//       /* ASCII (code page 20127): single-byte, no lead bytes, default char '?' */
//       memset(&cpi, 0, sizeof(cpi));
//       cpi.MaxCharSize = 1;
//       cpi.DefaultChar[0] = '?';
//     } else {
//       WARN("Codepage %d not found\n", newcp);
//       free(mbcinfo);
//       return NULL;
//     }
//   }
//
//   /* setup the _mbctype */
//   memset(mbcinfo->mbctype, 0, sizeof(unsigned char[257]));
//   memset(mbcinfo->mbcasemap, 0, sizeof(unsigned char[256]));
//
//   bytes = cpi.LeadByte;
//   while (bytes[0] || bytes[1])
//   {
//     for (i = bytes[0]; i <= bytes[1]; i++)
//       mbcinfo->mbctype[i + 1] |= _M1;
//     bytes += 2;
//   }
//
//   if (cpi.MaxCharSize == 2)
//   {
//     /* trail bytes not available through kernel32 but stored in a structure in msvcrt */
//     struct cp_extra_info_t *cpextra = g_cpextrainfo;
//
//     mbcinfo->ismbcodepage = 1;
//     while (TRUE)
//     {
//       if (cpextra->cp == 0 || cpextra->cp == newcp)
//       {
//         if (cpextra->cp == 0)
//           FIXME("trail bytes data not available for DBCS codepage %d - assuming all bytes\n", newcp);
//
//         bytes = cpextra->TrailBytes;
//         while (bytes[0] || bytes[1])
//         {
//           for (i = bytes[0]; i <= bytes[1]; i++)
//             mbcinfo->mbctype[i + 1] |= _M2;
//           bytes += 2;
//         }
//         break;
//       }
//       cpextra++;
//     }
//   }
//   else
//     mbcinfo->ismbcodepage = 0;
//
//   maxchar = (newcp == CP_UTF8) ? 128 : 256;
//
//   /* we can't use GetStringTypeA directly because we don't have a locale - only a code page
//    */
//   charcount = 0;
//   for (i = 0; i < maxchar; i++)
//     if (!(mbcinfo->mbctype[i + 1] & _M1))
//       bufA[charcount++] = i;
//
//   ret = MultiByteToWideChar(newcp, 0, bufA, charcount, bufW, charcount);
//   if (ret != charcount)
//   {
//     ERR("MultiByteToWideChar of chars failed for cp %d, ret=%d (exp %d), error=%ld\n",
//             newcp, ret, charcount, GetLastError());
//   }
//
//   GetStringTypeW(CT_CTYPE1, bufW, charcount, chartypes);
//   LCMapStringW(mbcinfo->mblcid, LCMAP_LOWERCASE, bufW, charcount, lowW, charcount);
//   LCMapStringW(mbcinfo->mblcid, LCMAP_UPPERCASE, bufW, charcount, upW, charcount);
//
//   charcount = 0;
//   for (i = 0; i < maxchar; i++)
//     if (!(mbcinfo->mbctype[i + 1] & _M1))
//     {
//       if (chartypes[charcount] & C1_UPPER)
//       {
//         mbcinfo->mbctype[i + 1] |= _SBUP;
//         bufW[charcount] = lowW[charcount];
//       }
//       else if (chartypes[charcount] & C1_LOWER)
//       {
// 	mbcinfo->mbctype[i + 1] |= _SBLOW;
//         bufW[charcount] = upW[charcount];
//       }
//       charcount++;
//     }
//
//   ret = WideCharToMultiByte(newcp, 0, bufW, charcount, bufA, charcount, NULL, NULL);
//   if (ret != charcount)
//   {
//     ERR("WideCharToMultiByte failed for cp %d, ret=%d (exp %d), error=%ld\n",
//             newcp, ret, charcount, GetLastError());
//   }
//
//   charcount = 0;
//   for (i = 0; i < maxchar; i++)
//   {
//     if(!(mbcinfo->mbctype[i + 1] & _M1))
//     {
//       if(mbcinfo->mbctype[i + 1] & (_SBUP | _SBLOW))
//         mbcinfo->mbcasemap[i] = bufA[charcount];
//       charcount++;
//     }
//   }
//
//   if (newcp == 932)   /* CP932 only - set _MP and _MS */
//   {
//     /* On Windows it's possible to calculate the _MP and _MS from CT_CTYPE1
//      * and CT_CTYPE3. But as of Wine 0.9.43 we return wrong values what makes
//      * it hard. As this is set only for codepage 932 we hardcode it what gives
//      * also faster execution.
//      */
//     for (i = 161; i <= 165; i++)
//       mbcinfo->mbctype[i + 1] |= _MP;
//     for (i = 166; i <= 223; i++)
//       mbcinfo->mbctype[i + 1] |= _MS;
//   }
//
//   mbcinfo->mbcodepage = newcp;
//   return mbcinfo;
// }
//
// /*********************************************************************
//  *              _setmbcp (MSVCRT.@)
//  */
// int CDECL _setmbcp(int cp)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     threadmbcinfo *mbcinfo;
//
//     mbcinfo = create_mbcinfo(cp, -1, get_mbcinfo());
//     if(!mbcinfo)
//     {
//         *_errno() = EINVAL;
//         return -1;
//     }
//
//     if(data->locale_flags & LOCALE_THREAD)
//     {
//         if(data->locale_flags & LOCALE_FREE)
//             free_mbcinfo(data->mbcinfo);
//         data->mbcinfo = mbcinfo;
//     }
//     else
//     {
//         _lock(_MB_CP_LOCK);
//         free_mbcinfo(MSVCRT_locale->mbcinfo);
//         MSVCRT_locale->mbcinfo = mbcinfo;
//         memcpy(MSVCRT_mbctype, MSVCRT_locale->mbcinfo->mbctype, sizeof(MSVCRT_mbctype));
//         _unlock(_MB_CP_LOCK);
//     }
//     return 0;
// }
//
// /*********************************************************************
//  *		_getmbcp (MSVCRT.@)
//  */
// int CDECL _getmbcp(void)
// {
//   return get_mbcinfo()->mbcodepage;
// }
//
// /*********************************************************************
//  *		_mbsnextc_l(MSVCRT.@)
//  */
// unsigned int CDECL _mbsnextc_l(const unsigned char* str, _locale_t locale)
// {
//   if(_ismbblead_l(*str, locale))
//     return *str << 8 | str[1];
//   return *str;
// }
//
// /*********************************************************************
//  *		_mbsnextc(MSVCRT.@)
//  */
// unsigned int CDECL _mbsnextc(const unsigned char* str)
// {
//     return _mbsnextc_l(str, NULL);
// }
//
// /*********************************************************************
//  *		_mbctolower_l(MSVCRT.@)
//  */
// unsigned int CDECL _mbctolower_l(unsigned int c, _locale_t locale)
// {
//     unsigned char str[2], ret[2];
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if (c > 0xff)
//     {
//         if (!_ismbblead_l((c >> 8) & 0xff, locale))
//             return c;
//
//         str[0] = c >> 8;
//         str[1] = c;
//         switch(__crtLCMapStringA(mbcinfo->mblcid, LCMAP_LOWERCASE,
//                     (char*)str, 2, (char*)ret, 2, mbcinfo->mbcodepage, 0))
//         {
//         case 0:
//             return c;
//         case 1:
//             return ret[0];
//         default:
//             return ret[1] + (ret[0] << 8);
//         }
//     }
//
//     return mbcinfo->mbctype[c + 1] & _SBUP ? mbcinfo->mbcasemap[c] : c;
// }
//
// /*********************************************************************
//  *		_mbctolower(MSVCRT.@)
//  */
// unsigned int CDECL _mbctolower(unsigned int c)
// {
//     return _mbctolower_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_mbctoupper_l(MSVCRT.@)
//  */
// unsigned int CDECL _mbctoupper_l(unsigned int c, _locale_t locale)
// {
//     unsigned char str[2], ret[2];
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if (c > 0xff)
//     {
//         if (!_ismbblead_l((c >> 8) & 0xff, locale))
//             return c;
//
//         str[0] = c >> 8;
//         str[1] = c;
//         switch(__crtLCMapStringA(mbcinfo->mblcid, LCMAP_UPPERCASE,
//                     (char*)str, 2, (char*)ret, 2, mbcinfo->mbcodepage, 0))
//         {
//         case 0:
//             return c;
//         case 1:
//             return ret[0];
//         default:
//             return ret[1] + (ret[0] << 8);
//         }
//     }
//
//     return mbcinfo->mbctype[c + 1] & _SBLOW ? mbcinfo->mbcasemap[c] : c;
// }
//
// /*********************************************************************
//  *		_mbctoupper(MSVCRT.@)
//  */
// unsigned int CDECL _mbctoupper(unsigned int c)
// {
//     return _mbctoupper_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_mbctombb_l (MSVCRT.@)
//  */
// unsigned int CDECL _mbctombb_l(unsigned int c, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//     unsigned int value;
//
//     if(locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if(mbcinfo->mbcodepage == 932)
//     {
//         if(c >= 0x829f && c <= 0x82f1)    /* Hiragana */
//             return mbctombb_932_kana[c - 0x829f];
//         if(c >= 0x8340 && c <= 0x8396 && c != 0x837f)    /* Katakana */
//             return mbctombb_932_kana[c - 0x8340 - (c >= 0x837f ? 1 : 0)];
//         if(c >= 0x8140 && c <= 0x8197)    /* Punctuation */
//         {
//             value = mbctombb_932_punct[c - 0x8140];
//             return value ? value : c;
//         }
//         if((c >= 0x824f && c <= 0x8258) || /* Fullwidth digits */
//            (c >= 0x8260 && c <= 0x8279))   /* Fullwidth capitals letters */
//             return c - 0x821f;
//         if(c >= 0x8281 && c <= 0x829a)     /* Fullwidth small letters */
//             return c - 0x8220;
//         /* all other cases return c */
//     }
//     return c;
// }
//
// /*********************************************************************
//  *		_mbctombb (MSVCRT.@)
//  */
// unsigned int CDECL _mbctombb(unsigned int c)
// {
//     return _mbctombb_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_mbcjistojms_l(MSVCRT.@)
//  *
//  *		Converts a jis character to sjis.
//  *		Based on description from
//  *		http://www.slayers.ne.jp/~oouchi/code/jistosjis.html
//  */
// unsigned int CDECL _mbcjistojms_l(unsigned int c, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//
//   if(locale)
//       mbcinfo = locale->mbcinfo;
//   else
//       mbcinfo = get_mbcinfo();
//
//   /* Conversion takes place only when codepage is 932.
//      In all other cases, c is returned unchanged */
//   if(mbcinfo->mbcodepage == 932)
//   {
//     if(HIBYTE(c) >= 0x21 && HIBYTE(c) <= 0x7e &&
//        LOBYTE(c) >= 0x21 && LOBYTE(c) <= 0x7e)
//     {
//       if(HIBYTE(c) % 2)
//         c += 0x1f;
//       else
//         c += 0x7d;
//
//       if(LOBYTE(c) >= 0x7F)
//         c += 0x1;
//
//       c = (((HIBYTE(c) - 0x21)/2 + 0x81) << 8) | LOBYTE(c);
//
//       if(HIBYTE(c) > 0x9f)
//         c += 0x4000;
//     }
//     else
//       return 0; /* Codepage is 932, but c can't be converted */
//   }
//
//   return c;
// }
//
// /*********************************************************************
//  *		_mbcjistojms(MSVCRT.@)
//  */
// unsigned int CDECL _mbcjistojms(unsigned int c)
// {
//     return _mbcjistojms_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_mbcjmstojis_l(MSVCRT.@)
//  *
//  *		Converts a sjis character to jis.
//  */
// unsigned int CDECL _mbcjmstojis_l(unsigned int c, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//
//   if(locale)
//       mbcinfo = locale->mbcinfo;
//   else
//       mbcinfo = get_mbcinfo();
//
//   /* Conversion takes place only when codepage is 932.
//      In all other cases, c is returned unchanged */
//   if(mbcinfo->mbcodepage == 932)
//   {
//     if(_ismbclegal_l(c, locale) && HIBYTE(c) < 0xf0)
//     {
//       if(HIBYTE(c) >= 0xe0)
//         c -= 0x4000;
//
//       c = (((HIBYTE(c) - 0x81)*2 + 0x21) << 8) | LOBYTE(c);
//
//       if(LOBYTE(c) > 0x7f)
//         c -= 0x1;
//
//       if(LOBYTE(c) > 0x9d)
//         c += 0x83;
//       else
//         c -= 0x1f;
//     }
//     else
//       return 0; /* Codepage is 932, but c can't be converted */
//   }
//
//   return c;
// }
//
// /*********************************************************************
//  *		_mbcjmstojis(MSVCRT.@)
//  */
// unsigned int CDECL _mbcjmstojis(unsigned int c)
// {
//     return _mbcjmstojis_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_mbclen_l(MSVCRT.@)
//  */
// size_t CDECL _mbclen_l(const unsigned char* str, _locale_t locale)
// {
//     return _ismbblead_l(*str, locale) && str[1] ? 2 : 1;
// }
//
// /*********************************************************************
//  *		_mbclen(MSVCRT.@)
//  */
// size_t CDECL _mbclen(const unsigned char* str)
// {
//     return _mbclen_l(str, NULL);
// }
//
// /*********************************************************************
//  *		_mbsinc_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsinc_l(const unsigned char* str, _locale_t locale)
// {
//     return (unsigned char *)(str + _mbclen_l(str, locale));
// }
//
// /*********************************************************************
//  *		_mbsinc(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsinc(const unsigned char* str)
// {
//     return _mbsinc_l(str, NULL);
// }
//
// /*********************************************************************
//  *		_mbsninc(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsninc(const unsigned char* str, size_t num)
// {
//   if(!str)
//     return NULL;
//
//   while (num > 0 && *str)
//   {
//     if (_ismbblead(*str))
//     {
//       if (!*(str+1))
//          break;
//       str++;
//     }
//     str++;
//     num--;
//   }
//
//   return (unsigned char*)str;
// }
//
// /*********************************************************************
//  *              _mbsnlen_l(MSVCRT.@)
//  */
// size_t CDECL _mbsnlen_l(const unsigned char *str,
//         size_t maxsize, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//     size_t i = 0, len = 0;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if(!mbcinfo->ismbcodepage)
//         return strnlen((const char*)str, maxsize);
//
//     while(i<maxsize && str[i])
//     {
//         if (_ismbblead_l(str[i], locale))
//         {
//             i++;
//             if (!str[i])  /* count only full chars */
//                 break;
//         }
//         i++;
//         len++;
//     }
//     return i < maxsize ? len : maxsize;
// }
//
// /*********************************************************************
//  *		_mbslen(MSVCRT.@)
//  */
// size_t CDECL _mbslen(const unsigned char* str)
// {
//     return _mbsnlen_l(str, -1, NULL);
// }
//
// /*********************************************************************
//  *              _mbslen_l(MSVCRT.@)
//  */
// size_t CDECL _mbslen_l(const unsigned char* str, _locale_t locale)
// {
//     return _mbsnlen_l(str, -1, locale);
// }
//
// /*********************************************************************
//  *              _mbsnlen(MSVCRT.@)
//  */
// size_t CDECL _mbsnlen(const unsigned char* str, size_t maxsize)
// {
//     return _mbsnlen_l(str, maxsize, NULL);
// }
//
// /*********************************************************************
//  *              _mbccpy_s_l(MSVCRT.@)
//  */
// int CDECL _mbccpy_s_l(unsigned char* dest, size_t maxsize,
//         int *copied, const unsigned char* src, _locale_t locale)
// {
//     if(copied) *copied = 0;
//     if(!MSVCRT_CHECK_PMT(dest != NULL && maxsize >= 1)) return EINVAL;
//     dest[0] = 0;
//     if(!MSVCRT_CHECK_PMT(src != NULL)) return EINVAL;
//
//     if(_ismbblead_l(*src, locale)) {
//         if(!src[1]) {
//             if(copied) *copied = 1;
//             *_errno() = EILSEQ;
//             return EILSEQ;
//         }
//
//         if(maxsize < 2) {
//             MSVCRT_INVALID_PMT("dst buffer is too small", ERANGE);
//             return ERANGE;
//         }
//
//         *dest++ = *src++;
//         *dest = *src;
//         if(copied) *copied = 2;
//     }else {
//         *dest = *src;
//         if(copied) *copied = 1;
//     }
//
//     return 0;
// }
//
// /*********************************************************************
//  *		_mbccpy(MSVCRT.@)
//  */
// void CDECL _mbccpy(unsigned char* dest, const unsigned char* src)
// {
//     _mbccpy_s_l(dest, 2, NULL, src, NULL);
// }
//
// /*********************************************************************
//  *              _mbccpy_l(MSVCRT.@)
//  */
// void CDECL _mbccpy_l(unsigned char* dest, const unsigned char* src,
//         _locale_t locale)
// {
//     _mbccpy_s_l(dest, 2, NULL, src, locale);
// }
//
// /*********************************************************************
//  *              _mbccpy_s(MSVCRT.@)
//  */
// int CDECL _mbccpy_s(unsigned char* dest, size_t maxsize,
//         int *copied, const unsigned char* src)
// {
//     return _mbccpy_s_l(dest, maxsize, copied, src, NULL);
// }
//
//
// /*********************************************************************
//  *		_mbsncpy_l(MSVCRT.@)
//  * REMARKS
//  *  The parameter n is the number or characters to copy, not the size of
//  *  the buffer. Use _mbsnbcpy_l for a function analogical to strncpy
//  */
// unsigned char* CDECL _mbsncpy_l(unsigned char* dst, const unsigned char* src, size_t n, _locale_t locale)
// {
//     unsigned char* ret = dst;
//     pthreadmbcinfo mbcinfo;
//
//     if (!n)
//         return dst;
//     if (!MSVCRT_CHECK_PMT(dst && src))
//         return NULL;
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         while (*src && n)
//         {
//             n--;
//             if (_ismbblead_l(*src, locale))
//             {
//                 if (!*(src + 1))
//                 {
//                     *dst++ = 0;
//                     *dst++ = 0;
//                     break;
//                 }
//                 *dst++ = *src++;
//             }
//             *dst++ = *src++;
//         }
//     }
//     else
//     {
//         while (n)
//         {
//             n--;
//             if (!(*dst++ = *src++)) break;
//         }
//     }
//     while (n--) *dst++ = 0;
//     return ret;
// }
//
//
// /*********************************************************************
//  *		_mbsncpy(MSVCRT.@)
//  * REMARKS
//  *  The parameter n is the number or characters to copy, not the size of
//  *  the buffer. Use _mbsnbcpy for a function analogical to strncpy
//  */
// unsigned char* CDECL _mbsncpy(unsigned char* dst, const unsigned char* src, size_t n)
// {
//     return _mbsncpy_l(dst, src, n, NULL);
// }
//
// /*********************************************************************
//  *              _mbsnbcpy_s_l(MSVCRT.@)
//  * REMARKS
//  * Unlike _mbsnbcpy this function does not pad the rest of the dest
//  * string with 0
//  */
// int CDECL _mbsnbcpy_s_l(unsigned char* dst, size_t size,
//         const unsigned char* src, size_t n, _locale_t locale)
// {
//     size_t pos = 0;
//
//     if(!dst || size == 0)
//         return EINVAL;
//     if(!src)
//     {
//         dst[0] = '\0';
//         return EINVAL;
//     }
//     if(!n)
//         return 0;
//
//     if((locale ? locale->mbcinfo : get_mbcinfo())->ismbcodepage)
//     {
//         BOOL is_lead = FALSE;
//         while (*src && n)
//         {
//             if(pos == size)
//             {
//                 dst[0] = '\0';
//                 return ERANGE;
//             }
//             is_lead = (!is_lead && _ismbblead(*src));
//             n--;
//             dst[pos++] = *src++;
//         }
//
//         if (is_lead) /* if string ends with a lead, remove it */
//             dst[pos - 1] = 0;
//     }
//     else
//     {
//         while (n)
//         {
//             n--;
//             if(pos == size)
//             {
//                 dst[0] = '\0';
//                 return ERANGE;
//             }
//
//             if(!(*src)) break;
//             dst[pos++] = *src++;
//         }
//     }
//
//     if(pos < size)
//         dst[pos] = '\0';
//     else
//     {
//         dst[0] = '\0';
//         return ERANGE;
//     }
//
//     return 0;
// }
//
// /*********************************************************************
//  *              _mbsnbcpy_s(MSVCRT.@)
//  */
// int CDECL _mbsnbcpy_s(unsigned char* dst, size_t size, const unsigned char* src, size_t n)
// {
//     return _mbsnbcpy_s_l(dst, size, src, n, NULL);
// }
//
// /*********************************************************************
//  *              _mbscpy_s_l(MSVCRT.@)
//  */
// int CDECL _mbscpy_s_l(unsigned char *dst, size_t size,
//         const unsigned char *src, _locale_t locale)
// {
//     return _mbsnbcpy_s_l(dst, size, src, -1, locale);
// }
//
// /*********************************************************************
//  *              _mbscpy_s(MSVCRT.@)
//  */
// int CDECL _mbscpy_s(unsigned char *dst, size_t size, const unsigned char *src)
// {
//     return _mbscpy_s_l(dst, size, src, NULL);
// }
//
// /*********************************************************************
//  *              _mbsnbcpy_l(MSVCRT.@)
//  * REMARKS
//  *  Like strncpy this function doesn't enforce the string to be
//  *  NUL-terminated
//  */
// unsigned char* CDECL _mbsnbcpy_l(unsigned char* dst, const unsigned char* src, size_t n, _locale_t locale)
// {
//     unsigned char* ret = dst;
//     pthreadmbcinfo mbcinfo;
//
//     if (!n)
//         return dst;
//     if (!MSVCRT_CHECK_PMT(dst && src))
//         return NULL;
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//     if (mbcinfo->ismbcodepage)
//     {
//         BOOL is_lead = FALSE;
//         while (*src && n)
//         {
//             is_lead = (!is_lead && _ismbblead_l(*src, locale));
//             n--;
//             *dst++ = *src++;
//         }
//
//         if (is_lead) /* if string ends with a lead, remove it */
//             *(dst - 1) = 0;
//     }
//     else
//     {
//         while (n)
//         {
//             n--;
//             if (!(*dst++ = *src++)) break;
//         }
//     }
//     while (n--) *dst++ = 0;
//     return ret;
// }
//
// /*********************************************************************
//  *              _mbsnbcpy(MSVCRT.@)
//  * REMARKS
//  *  Like strncpy this function doesn't enforce the string to be
//  *  NUL-terminated
//  */
// unsigned char* CDECL _mbsnbcpy(unsigned char* dst, const unsigned char* src, size_t n)
// {
//     return _mbsnbcpy_l(dst, src, n, NULL);
// }
//
// /*********************************************************************
//  *		_mbscmp_l(MSVCRT.@)
//  */
// int CDECL _mbscmp_l(const unsigned char* str, const unsigned char* cmp, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//
//   if (!MSVCRT_CHECK_PMT(str && cmp))
//     return _NLSCMPERROR;
//
//   mbcinfo = locale ? locale->mbcinfo : get_mbcinfo();
//
//   if(mbcinfo->ismbcodepage)
//   {
//     unsigned int strc, cmpc;
//     do {
//       if(!*str)
//         return *cmp ? -1 : 0;
//       if(!*cmp)
//         return 1;
//       strc = _mbsnextc_l(str, locale);
//       cmpc = _mbsnextc_l(cmp, locale);
//       if(strc != cmpc)
//         return strc < cmpc ? -1 : 1;
//       str +=(strc > 255) ? 2 : 1;
//       cmp +=(strc > 255) ? 2 : 1; /* equal, use same increment */
//     } while(1);
//   }
//   return u_strcmp(str, cmp); /* ASCII CP */
// }
//
// /*********************************************************************
//  *		_mbscmp(MSVCRT.@)
//  */
// int CDECL _mbscmp(const unsigned char* str, const unsigned char* cmp)
// {
//     return _mbscmp_l(str, cmp, NULL);
// }
//
// /*********************************************************************
//  *              _mbsnbicoll_l(MSVCRT.@)
//  */
// int CDECL _mbsnbicoll_l(const unsigned char *str1, const unsigned char *str2, size_t len, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if(!mbcinfo->ismbcodepage)
//         return _strnicoll_l((const char*)str1, (const char*)str2, len, locale);
//     return CompareStringA(mbcinfo->mblcid, NORM_IGNORECASE,
//             (const char*)str1, strnlen((const char*)str1, len),
//             (const char*)str2, strnlen((const char*)str2, len)) - CSTR_EQUAL;
// }
//
// /*********************************************************************
//  *              _mbsicoll_l(MSVCRT.@)
//  */
// int CDECL _mbsicoll_l(const unsigned char *str1, const unsigned char *str2, _locale_t locale)
// {
//     return _mbsnbicoll_l(str1, str2, INT_MAX, locale);
// }
//
// /*********************************************************************
//  *              _mbsnbicoll(MSVCRT.@)
//  */
// int CDECL _mbsnbicoll(const unsigned char *str1, const unsigned char *str2, size_t len)
// {
//     return _mbsnbicoll_l(str1, str2, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbsicoll(MSVCRT.@)
//  */
// int CDECL _mbsicoll(const unsigned char* str, const unsigned char* cmp)
// {
//     return _mbsnbicoll_l(str, cmp, INT_MAX, NULL);
// }
//
// /*********************************************************************
//  *              _mbsnbcoll_l(MSVCRT.@)
//  */
// int CDECL _mbsnbcoll_l(const unsigned char *str1, const unsigned char *str2, size_t len, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if(!mbcinfo->ismbcodepage)
//         return _strncoll_l((const char*)str1, (const char*)str2, len, locale);
//     return CompareStringA(mbcinfo->mblcid, 0,
//             (const char*)str1, strnlen((const char*)str1, len),
//             (const char*)str2, strnlen((const char*)str2, len)) - CSTR_EQUAL;
// }
//
// /*********************************************************************
//  *              _mbscoll_l(MSVCRT.@)
//  */
// int CDECL _mbscoll_l(const unsigned char *str1, const unsigned char *str2, _locale_t locale)
// {
//     return _mbsnbcoll_l(str1, str2, INT_MAX, locale);
// }
//
// /*********************************************************************
//  *              _mbsnbcoll(MSVCRT.@)
//  */
// int CDECL _mbsnbcoll(const unsigned char *str1, const unsigned char *str2, size_t len)
// {
//     return _mbsnbcoll_l(str1, str2, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbscoll(MSVCRT.@)
//  */
// int CDECL _mbscoll(const unsigned char* str, const unsigned char* cmp)
// {
//     return _mbsnbcoll_l(str, cmp, INT_MAX, NULL);
// }
//
// /*********************************************************************
//  *		_mbsicmp_l(MSVCRT.@)
//  */
// int CDECL _mbsicmp_l(const unsigned char* str, const unsigned char* cmp, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//
//   if(!MSVCRT_CHECK_PMT(str && cmp))
//     return _NLSCMPERROR;
//
//   if(!locale)
//     mbcinfo = get_mbcinfo();
//   else
//     mbcinfo = locale->mbcinfo;
//   if(mbcinfo->ismbcodepage)
//   {
//     unsigned int strc, cmpc;
//     do {
//       if(!*str)
//         return *cmp ? -1 : 0;
//       if(!*cmp)
//         return 1;
//       strc = _mbctolower_l(_mbsnextc_l(str, locale), locale);
//       cmpc = _mbctolower_l(_mbsnextc_l(cmp, locale), locale);
//       if(strc != cmpc)
//         return strc < cmpc ? -1 : 1;
//       str +=(strc > 255) ? 2 : 1;
//       cmp +=(strc > 255) ? 2 : 1; /* equal, use same increment */
//     } while(1);
//   }
//   return u_strcasecmp(str, cmp); /* ASCII CP */
// }
//
// /*********************************************************************
//  *		_mbsicmp(MSVCRT.@)
//  */
// int CDECL _mbsicmp(const unsigned char* str, const unsigned char* cmp)
// {
//   return _mbsicmp_l(str, cmp, NULL);
// }
//
// /*********************************************************************
//  *		_mbsncmp_l(MSVCRT.@)
//  */
// int CDECL _mbsncmp_l(const unsigned char* str, const unsigned char* cmp,
//         size_t len, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//     unsigned int strc, cmpc;
//
//     if (!len)
//         return 0;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (!mbcinfo->ismbcodepage)
//         return u_strncmp(str, cmp, len); /* ASCII CP */
//
//     if (!MSVCRT_CHECK_PMT(str && cmp))
//         return _NLSCMPERROR;
//
//     while (len--)
//     {
//         int inc;
//
//         if (!*str)
//             return *cmp ? -1 : 0;
//         if (!*cmp)
//             return 1;
//         strc = _mbsnextc_l(str, locale);
//         cmpc = _mbsnextc_l(cmp, locale);
//         if (strc != cmpc)
//             return strc < cmpc ? -1 : 1;
//         inc = (strc > 255) ? 2 : 1; /* Equal, use same increment */
//         str += inc;
//         cmp += inc;
//     }
//     return 0; /* Matched len chars */
// }
//
// /*********************************************************************
//  *		_mbsncmp(MSVCRT.@)
//  */
// int CDECL _mbsncmp(const unsigned char* str, const unsigned char* cmp, size_t len)
// {
//     return _mbsncmp_l(str, cmp, len, NULL);
// }
//
// /*********************************************************************
//  *              _mbsnbcmp_l(MSVCRT.@)
//  */
// int CDECL _mbsnbcmp_l(const unsigned char* str, const unsigned char* cmp, size_t len, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if (!len)
//         return 0;
//
//     if (!MSVCRT_CHECK_PMT(str && cmp))
//         return _NLSCMPERROR;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         unsigned int strc, cmpc;
//         while (len)
//         {
//             int clen;
//             if (!*str)
//                 return *cmp ? -1 : 0;
//             if (!*cmp)
//                 return 1;
//             if (_ismbblead_l(*str, locale))
//             {
//                 strc = (len >= 2) ? _mbsnextc_l(str, locale) : 0;
//                 clen = 2;
//             }
//             else
//             {
//                 strc = *str;
//                 clen = 1;
//             }
//             if (_ismbblead_l(*cmp, locale))
//                 cmpc = (len >= 2) ? _mbsnextc_l(cmp, locale) : 0;
//             else
//                 cmpc = *cmp;
//             if(strc != cmpc)
//                 return strc < cmpc ? -1 : 1;
//             len -= clen;
//             str += clen;
//             cmp += clen;
//         }
//         return 0; /* Matched len chars */
//     }
//     return u_strncmp(str, cmp, len);
// }
//
// /*********************************************************************
//  *              _mbsnbcmp(MSVCRT.@)
//  */
// int CDECL _mbsnbcmp(const unsigned char* str, const unsigned char* cmp, size_t len)
// {
//     return _mbsnbcmp_l(str, cmp, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbsnicmp(MSVCRT.@)
//  *
//  * Compare two multibyte strings case insensitively to 'len' characters.
//  */
// int CDECL _mbsnicmp_l(const unsigned char* str, const unsigned char* cmp, size_t len, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if (!len)
//         return 0;
//     if (!MSVCRT_CHECK_PMT(str && cmp))
//         return _NLSCMPERROR;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//     /* FIXME: No tolower() for mb strings yet */
//     if (mbcinfo->ismbcodepage)
//     {
//         unsigned int strc, cmpc;
//         while (len--)
//         {
//             if (!*str)
//                 return *cmp ? -1 : 0;
//             if (!*cmp)
//                 return 1;
//             strc = _mbctolower_l(_mbsnextc_l(str, locale), locale);
//             cmpc = _mbctolower_l(_mbsnextc_l(cmp, locale), locale);
//             if (strc != cmpc)
//                 return strc < cmpc ? -1 : 1;
//             str += (strc > 255) ? 2 : 1;
//             cmp += (strc > 255) ? 2 : 1; /* Equal, use same increment */
//         }
//         return 0; /* Matched len chars */
//     }
//     return u_strncasecmp(str, cmp, len); /* ASCII CP */
// }
//
// /*********************************************************************
//  *		_mbsnicmp(MSVCRT.@)
//  *
//  * Compare two multibyte strings case insensitively to 'len' characters.
//  */
// int CDECL _mbsnicmp(const unsigned char* str, const unsigned char* cmp, size_t len)
// {
//   return _mbsnicmp_l(str, cmp, len,  NULL);
// }
//
// /*********************************************************************
//  *              _mbsnbicmp_l(MSVCRT.@)
//  */
// int CDECL _mbsnbicmp_l(const unsigned char* str, const unsigned char* cmp, size_t len, _locale_t locale)
// {
//
//     pthreadmbcinfo mbcinfo;
//
//     if (!len)
//         return 0;
//     if (!MSVCRT_CHECK_PMT(str && cmp))
//         return _NLSCMPERROR;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         unsigned int strc, cmpc;
//         while (len)
//         {
//             int clen;
//             if (!*str)
//                 return *cmp ? -1 : 0;
//             if (!*cmp)
//                 return 1;
//             if (_ismbblead_l(*str, locale))
//             {
//                 strc = (len >= 2) ? _mbsnextc_l(str, locale) : 0;
//                 clen = 2;
//             }
//             else
//             {
//                 strc = *str;
//                 clen = 1;
//             }
//             if (_ismbblead_l(*cmp, locale))
//                 cmpc = (len >= 2) ? _mbsnextc_l(cmp, locale) : 0;
//             else
//                 cmpc = *cmp;
//             strc = _mbctolower_l(strc, locale);
//             cmpc = _mbctolower_l(cmpc, locale);
//             if (strc != cmpc)
//                 return strc < cmpc ? -1 : 1;
//             len -= clen;
//             str += clen;
//             cmp += clen;
//         }
//         return 0; /* Matched len bytes */
//     }
//     return u_strncasecmp(str, cmp, len);
// }
//
// /*********************************************************************
//  *              _mbsnbicmp(MSVCRT.@)
//  */
// int CDECL _mbsnbicmp(const unsigned char* str, const unsigned char* cmp, size_t len)
// {
//     return _mbsnbicmp_l(str, cmp, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbscat (MSVCRT.@)
//  */
// unsigned char * CDECL _mbscat( unsigned char *dst, const unsigned char *src )
// {
//     strcat( (char *)dst, (const char *)src );
//     return dst;
// }
//
// /*********************************************************************
//  *		_mbscat_s_l (MSVCRT.@)
//  */
// int CDECL _mbscat_s_l( unsigned char *dst, size_t size,
//         const unsigned char *src, _locale_t locale )
// {
//     size_t i, j;
//     int ret = 0;
//
//     if(!MSVCRT_CHECK_PMT(dst != NULL)) return EINVAL;
//     if(!MSVCRT_CHECK_PMT(src != NULL)) return EINVAL;
//
//     for(i=0; i<size; i++)
//         if(!dst[i]) break;
//     if(i == size) {
//         MSVCRT_INVALID_PMT("dst is not NULL-terminated", EINVAL);
//         if(size) dst[0] = 0;
//         return EINVAL;
//     }
//
//     if(i && _ismbblead_l(dst[i-1], locale)) {
//         ret = EILSEQ;
//         i--;
//     }
//
//     for(j=0; src[j] && i+j<size; j++)
//         dst[i+j] = src[j];
//     if(i+j == size) {
//         MSVCRT_INVALID_PMT("dst buffer is too small", ERANGE);
//         dst[0] = 0;
//         return ERANGE;
//     }
//
//     if(j && _ismbblead_l(src[j-1], locale)) {
//         ret = EILSEQ;
//         j--;
//     }
//
//     dst[i+j] = 0;
//     return ret;
// }
//
// /*********************************************************************
//  *		_mbscat_s (MSVCRT.@)
//  */
// int CDECL _mbscat_s( unsigned char *dst, size_t size, const unsigned char *src )
// {
//     return _mbscat_s_l(dst, size, src, NULL);
// }
//
// /*********************************************************************
//  *		_mbscpy (MSVCRT.@)
//  */
// unsigned char* CDECL _mbscpy( unsigned char *dst, const unsigned char *src )
// {
//     strcpy( (char *)dst, (const char *)src );
//     return dst;
// }
//
// /*********************************************************************
//  *		_mbsstr (MSVCRT.@)
//  */
// unsigned char * CDECL _mbsstr(const unsigned char *haystack, const unsigned char *needle)
// {
//     return (unsigned char *)strstr( (const char *)haystack, (const char *)needle );
// }
//
// /*********************************************************************
//  *		_mbschr_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbschr_l(const unsigned char* s, unsigned int x, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//
//   if(!MSVCRT_CHECK_PMT(s))
//     return NULL;
//
//   if(locale)
//       mbcinfo = locale->mbcinfo;
//   else
//       mbcinfo = get_mbcinfo();
//   if(mbcinfo->ismbcodepage)
//   {
//     unsigned int c;
//     while (1)
//     {
//       c = _mbsnextc_l(s, locale);
//       if (c == x)
//         return (unsigned char*)s;
//       if (!c)
//         return NULL;
//       s += c > 255 ? 2 : 1;
//     }
//   }
//   return u_strchr(s, x); /* ASCII CP */
// }
//
// /*********************************************************************
//  *		_mbschr(MSVCRT.@)
//  */
// unsigned char* CDECL _mbschr(const unsigned char* s, unsigned int x)
// {
//   return _mbschr_l(s, x, NULL);
// }
//
// /*********************************************************************
//  *		_mbsrchr_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsrchr_l(const unsigned char *s, unsigned int x, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if (!MSVCRT_CHECK_PMT(s))
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         unsigned char *match = NULL;
//         unsigned int c;
//
//         while (1)
//         {
//             c = _mbsnextc_l(s, locale);
//             if (c == x)
//                 match = (unsigned char *)s;
//             if (!c)
//                 return match;
//             s += (c > 255) ? 2 : 1;
//         }
//     }
//     return u_strrchr(s, x);
// }
//
// /*********************************************************************
//  *		_mbsrchr(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsrchr(const unsigned char* s, unsigned int x)
// {
//     return _mbsrchr_l(s, x, NULL);
// }
//
// /*********************************************************************
//  *              _mbstok_s_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbstok_s_l(unsigned char *str, const unsigned char *delim,
//         unsigned char **ctx, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//     unsigned int c;
//
//     if(!MSVCRT_CHECK_PMT(delim != NULL)) return NULL;
//     if(!MSVCRT_CHECK_PMT(ctx != NULL)) return NULL;
//     if(!MSVCRT_CHECK_PMT(str || *ctx)) return NULL;
//
//     if(locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if(!mbcinfo->ismbcodepage)
//         return (unsigned char*)strtok_s((char*)str, (const char*)delim, (char**)ctx);
//
//     if(!str)
//         str = *ctx;
//
//     while((c=_mbsnextc(str)) && _mbschr(delim, c))
//         str += c>255 ? 2 : 1;
//     if(!*str)
//     {
//         *ctx = str;
//         return NULL;
//     }
//
//     *ctx = str + (c>255 ? 2 : 1);
//     while((c=_mbsnextc(*ctx)) && !_mbschr(delim, c))
//         *ctx += c>255 ? 2 : 1;
//     if (**ctx) {
//         *(*ctx)++ = 0;
//         if(c > 255)
//             *(*ctx)++ = 0;
//     }
//
//     return str;
// }
//
//
// /*********************************************************************
//  *              _mbstok_s(MSVCRT.@)
//  */
// unsigned char* CDECL _mbstok_s(unsigned char *str,
//         const unsigned char *delim, unsigned char **ctx)
// {
//     return _mbstok_s_l(str, delim, ctx, NULL);
// }
//
// /*********************************************************************
//  *              _mbstok_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbstok_l(unsigned char *str,
//         const unsigned char *delim, _locale_t locale)
// {
//     return _mbstok_s_l(str, delim, &msvcrt_get_thread_data()->mbstok_next, locale);
// }
//
// /*********************************************************************
//  *		_mbstok(MSVCRT.@)
//  */
// unsigned char* CDECL _mbstok(unsigned char *str, const unsigned char *delim)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     if(!str && !data->mbstok_next)
//         return NULL;
//
//     return _mbstok_s_l(str, delim, &data->mbstok_next, NULL);
// }
//
// /*********************************************************************
//  *		_mbbtombc_l(MSVCRT.@)
//  */
// unsigned int CDECL _mbbtombc_l(unsigned int c, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//
//   if(locale)
//       mbcinfo = locale->mbcinfo;
//   else
//       mbcinfo = get_mbcinfo();
//
//   if(mbcinfo->mbcodepage == 932)
//   {
//     if(c >= 0x20 && c <= 0x7e) {
//       if((c >= 0x41 && c <= 0x5a) || (c >= 0x61 && c <= 0x7a) || (c >= 0x30 && c <= 0x39))
//         return mbbtombc_932[c - 0x20] | 0x8200;
//       else
//         return mbbtombc_932[c - 0x20] | 0x8100;
//     }
//     else if(c >= 0xa1 && c <= 0xdf) {
//       if(c >= 0xa6 && c <= 0xdd && c != 0xb0)
//         return mbbtombc_932[c - 0xa1 + 0x5f] | 0x8300;
//       else
//         return mbbtombc_932[c - 0xa1 + 0x5f] | 0x8100;
//     }
//   }
//   return c;  /* not Japanese or no MB char */
// }
//
// /*********************************************************************
//  *		_mbbtombc(MSVCRT.@)
//  */
// unsigned int CDECL _mbbtombc(unsigned int c)
// {
//     return _mbbtombc_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_ismbbkana_l(MSVCRT.@)
//  */
// int CDECL _ismbbkana_l(unsigned int c, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if(locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if(mbcinfo->mbcodepage == 932)
//     {
//         /* Japanese/Katakana, CP 932 */
//         return (c >= 0xa1 && c <= 0xdf);
//     }
//     return 0;
// }
//
// /*********************************************************************
//  *              _ismbbkana(MSVCRT.@)
//  */
// int CDECL _ismbbkana(unsigned int c)
// {
//     return _ismbbkana_l( c, NULL );
// }
//
// /*********************************************************************
//  *              _ismbcdigit_l(MSVCRT.@)
//  */
// int CDECL _ismbcdigit_l(unsigned int ch, _locale_t locale)
// {
//     return _iswdigit_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbcdigit(MSVCRT.@)
//  */
// int CDECL _ismbcdigit(unsigned int ch)
// {
//     return _ismbcdigit_l( ch, NULL );
// }
//
// /*********************************************************************
//  *              _ismbcgraph_l(MSVCRT.@)
//  */
// int CDECL _ismbcgraph_l(unsigned int ch, _locale_t locale)
// {
//     return _iswgraph_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbcgraph(MSVCRT.@)
//  */
// int CDECL _ismbcgraph(unsigned int ch)
// {
//     return _ismbcgraph_l( ch, NULL );
// }
//
// /*********************************************************************
//  *              _ismbcalpha_l (MSVCRT.@)
//  */
// int CDECL _ismbcalpha_l(unsigned int ch, _locale_t locale)
// {
//     return _iswalpha_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbcalpha (MSVCRT.@)
//  */
// int CDECL _ismbcalpha(unsigned int ch)
// {
//     return _ismbcalpha_l( ch, NULL );
// }
//
// /*********************************************************************
//  *              _ismbclower_l (MSVCRT.@)
//  */
// int CDECL _ismbclower_l(unsigned int ch, _locale_t locale)
// {
//     return _iswlower_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbclower (MSVCRT.@)
//  */
// int CDECL _ismbclower(unsigned int ch)
// {
//     return _ismbclower_l( ch, NULL );
// }
//
// /*********************************************************************
//  *              _ismbcupper_l (MSVCRT.@)
//  */
// int CDECL _ismbcupper_l(unsigned int ch, _locale_t locale)
// {
//     return _iswupper_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbcupper (MSVCRT.@)
//  */
// int CDECL _ismbcupper(unsigned int ch)
// {
//     return _ismbcupper_l( ch, NULL );
// }
//
// /*********************************************************************
//  *              _ismbcsymbol_l(MSVCRT.@)
//  */
// int CDECL _ismbcsymbol_l(unsigned int ch, _locale_t locale)
// {
//     wchar_t wch = msvcrt_mbc_to_wc_l( ch, locale );
//     WORD ctype;
//     if (!GetStringTypeW(CT_CTYPE3, &wch, 1, &ctype))
//     {
//         WARN("GetStringTypeW failed on %x\n", ch);
//         return 0;
//     }
//     return ((ctype & C3_SYMBOL) != 0);
// }
//
// /*********************************************************************
//  *              _ismbcsymbol(MSVCRT.@)
//  */
// int CDECL _ismbcsymbol(unsigned int ch)
// {
//     return _ismbcsymbol_l(ch, NULL);
// }
//
// /*********************************************************************
//  *              _ismbcalnum_l (MSVCRT.@)
//  */
// int CDECL _ismbcalnum_l(unsigned int ch, _locale_t locale)
// {
//     return _iswalnum_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbcalnum (MSVCRT.@)
//  */
// int CDECL _ismbcalnum(unsigned int ch)
// {
//     return _ismbcalnum_l( ch, NULL );
// }
//
// /*********************************************************************
//  *              _ismbcspace_l (MSVCRT.@)
//  */
// int CDECL _ismbcspace_l(unsigned int ch, _locale_t locale)
// {
//     return _iswspace_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbcspace (MSVCRT.@)
//  */
// int CDECL _ismbcspace(unsigned int ch)
// {
//     return _ismbcspace_l( ch, NULL );
// }
//
// /*********************************************************************
//  *              _ismbcprint_l (MSVCRT.@)
//  */
// int CDECL _ismbcprint_l(unsigned int ch, _locale_t locale)
// {
//     return _iswprint_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbcprint (MSVCRT.@)
//  */
// int CDECL _ismbcprint(unsigned int ch)
// {
//     return _ismbcprint_l( ch, NULL );
// }
//
// /*********************************************************************
//  *              _ismbcpunct_l (MSVCRT.@)
//  */
// int CDECL _ismbcpunct_l(unsigned int ch, _locale_t locale)
// {
//     return _iswpunct_l( msvcrt_mbc_to_wc_l(ch, locale), locale );
// }
//
// /*********************************************************************
//  *              _ismbcpunct(MSVCRT.@)
//  */
// int CDECL _ismbcpunct(unsigned int ch)
// {
//     return _ismbcpunct_l( ch, NULL );
// }
//
// /*********************************************************************
//  *		_ismbchira_l(MSVCRT.@)
//  */
// int CDECL _ismbchira_l(unsigned int c, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//
//   if(locale)
//       mbcinfo = locale->mbcinfo;
//   else
//       mbcinfo = get_mbcinfo();
//
//   if(mbcinfo->mbcodepage == 932)
//   {
//     /* Japanese/Hiragana, CP 932 */
//     return (c >= 0x829f && c <= 0x82f1);
//   }
//   return 0;
// }
//
// /*********************************************************************
//  *		_ismbchira(MSVCRT.@)
//  */
// int CDECL _ismbchira(unsigned int c)
// {
//     return _ismbchira_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_ismbckata_l(MSVCRT.@)
//  */
// int CDECL _ismbckata_l(unsigned int c, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//
//   if(locale)
//       mbcinfo = locale->mbcinfo;
//   else
//       mbcinfo = get_mbcinfo();
//
//   if(mbcinfo->mbcodepage == 932)
//   {
//     /* Japanese/Katakana, CP 932 */
//     return (c >= 0x8340 && c <= 0x8396 && c != 0x837f);
//   }
//   return 0;
// }
//
// /*********************************************************************
//  *		_ismbckata(MSVCRT.@)
//  */
// int CDECL _ismbckata(unsigned int c)
// {
//     return _ismbckata_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_ismbblead_l(MSVCRT.@)
//  */
// int CDECL _ismbblead_l(unsigned int c, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     return (mbcinfo->mbctype[(c&0xff) + 1] & _M1) != 0;
// }
//
// /*********************************************************************
//  *		_ismbblead(MSVCRT.@)
//  */
// int CDECL _ismbblead(unsigned int c)
// {
//     return _ismbblead_l(c, NULL);
// }
//
// /*********************************************************************
//  *              _ismbbtrail_l(MSVCRT.@)
//  */
// int CDECL _ismbbtrail_l(unsigned int c, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     return (mbcinfo->mbctype[(c&0xff) + 1] & _M2) != 0;
// }
//
// /*********************************************************************
//  *		_ismbbtrail(MSVCRT.@)
//  */
// int CDECL _ismbbtrail(unsigned int c)
// {
//     return _ismbbtrail_l(c, NULL);
// }
//
// /*********************************************************************
//  *              _ismbclegal_l(MSVCRT.@)
//  */
// int CDECL _ismbclegal_l(unsigned int c, _locale_t locale)
// {
//     return _ismbblead_l(HIBYTE(c), locale) && _ismbbtrail_l(LOBYTE(c), locale);
// }
//
// /*********************************************************************
//  *              _ismbclegal(MSVCRT.@)
//  */
// int CDECL _ismbclegal(unsigned int c)
// {
//     return _ismbclegal_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_ismbslead_l(MSVCRT.@)
//  */
// int CDECL _ismbslead_l(const unsigned char* start, const unsigned char* str, _locale_t locale)
// {
//   pthreadmbcinfo mbcinfo;
//   int lead = 0;
//
//   if (!MSVCRT_CHECK_PMT(start && str))
//     return 0;
//
//   if(locale)
//       mbcinfo = locale->mbcinfo;
//   else
//       mbcinfo = get_mbcinfo();
//
//   if(!mbcinfo->ismbcodepage)
//     return 0;
//
//   /* Lead bytes can also be trail bytes so we need to analyse the string
//    */
//   while (start <= str)
//   {
//     if (!*start)
//       return 0;
//     lead = !lead && _ismbblead_l(*start, locale);
//     start++;
//   }
//
//   return lead ? -1 : 0;
// }
//
// /*********************************************************************
//  *		_ismbslead(MSVCRT.@)
//  */
// int CDECL _ismbslead(const unsigned char* start, const unsigned char* str)
// {
//   return _ismbslead_l(start, str, NULL);
// }
//
// /*********************************************************************
//  *		_ismbstrail_l(MSVCRT.@)
//  */
// int CDECL _ismbstrail_l(const unsigned char* start, const unsigned char* str, _locale_t locale)
// {
//   if (!MSVCRT_CHECK_PMT(start && str))
//     return 0;
//
//   /* Note: this function doesn't check _ismbbtrail */
//   if ((str > start) && _ismbslead_l(start, str-1, locale))
//     return -1;
//   else
//     return 0;
// }
//
// /*********************************************************************
//  *		_ismbstrail(MSVCRT.@)
//  */
// int CDECL _ismbstrail(const unsigned char* start, const unsigned char* str)
// {
//   return _ismbstrail_l(start, str, NULL);
// }
//
// /*********************************************************************
//  *		_mbsdec_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsdec_l(const unsigned char *start,
//         const unsigned char *cur, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if (!MSVCRT_CHECK_PMT(start && cur))
//         return NULL;
//     if (start >= cur)
//         return NULL;
//
//     if (!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if (mbcinfo->ismbcodepage)
//         return (unsigned char *)(_ismbstrail_l(start, cur - 1, locale) ? cur - 2 : cur - 1);
//
//     return (unsigned char *)cur - 1; /* ASCII CP or SB char */
// }
//
// /*********************************************************************
//  *		_mbsdec(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsdec(const unsigned char *start, const unsigned char *cur)
// {
//     return _mbsdec_l(start, cur, NULL);
// }
//
//
// /*********************************************************************
//  *		_mbbtype_l(MSVCRT.@)
//  */
// int CDECL _mbbtype_l(unsigned char c, int type, _locale_t locale)
// {
//     if (type == 1)
//         return _ismbbtrail_l(c, locale) ? _MBC_TRAIL : _MBC_ILLEGAL;
//     else
//         return _ismbblead_l(c, locale) ? _MBC_LEAD
//                 : _isprint_l(c, locale) ? _MBC_SINGLE : _MBC_ILLEGAL;
// }
//
// /*********************************************************************
//  *		_mbbtype(MSVCRT.@)
//  */
// int CDECL _mbbtype(unsigned char c, int type)
// {
//     return _mbbtype_l(c, type, NULL);
// }
//
// /*********************************************************************
//  *		_mbsbtype_l (MSVCRT.@)
//  */
// int CDECL _mbsbtype_l(const unsigned char *str, size_t count, _locale_t locale)
// {
//     int lead = 0;
//     pthreadmbcinfo mbcinfo;
//     const unsigned char *end = str + count;
//
//     if (!MSVCRT_CHECK_PMT(str))
//         return _MBC_ILLEGAL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     /* Lead bytes can also be trail bytes so we need to analyse the string.
//     * Also we must return _MBC_ILLEGAL for chars past the end of the string
//     */
//     while (str < end) /* Note: we skip the last byte - will check after the loop */
//     {
//         if (!*str)
//             return _MBC_ILLEGAL;
//         lead = mbcinfo->ismbcodepage && !lead && _ismbblead_l(*str, locale);
//         str++;
//     }
//
//     if (lead)
//     {
//         if (_ismbbtrail_l(*str, locale))
//             return _MBC_TRAIL;
//         else
//             return _MBC_ILLEGAL;
//     }
//     else
//     {
//         if (_ismbblead_l(*str, locale))
//             return _MBC_LEAD;
//         else
//             return _MBC_SINGLE;
//     }
// }
//
// /*********************************************************************
//  *		_mbsbtype (MSVCRT.@)
//  */
// int CDECL _mbsbtype(const unsigned char *str, size_t count)
// {
//     return _mbsbtype_l(str, count, NULL);
// }
//
//
// /*********************************************************************
//  *		_mbsset_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsset_l(unsigned char* str, unsigned int c, _locale_t locale)
// {
//     unsigned char* ret = str;
//     pthreadmbcinfo mbcinfo;
//
//     if (!MSVCRT_CHECK_PMT(str))
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (!mbcinfo->ismbcodepage || c < 256)
//         return u__strset(str, c); /* ASCII CP or SB char */
//
//     c &= 0xffff; /* Strip high bits */
//
//     while (str[0] && str[1])
//     {
//         *str++ = c >> 8;
//         *str++ = c & 0xff;
//     }
//     if (str[0])
//         str[0] = '\0'; /* FIXME: OK to shorten? */
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_mbsset(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsset(unsigned char* str, unsigned int c)
// {
//     return _mbsset_l(str, c, NULL);
// }
//
// /*********************************************************************
//  *		_mbsnbset_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsnbset_l(unsigned char *str, unsigned int c, size_t len, _locale_t locale)
// {
//     unsigned char *ret = str;
//     pthreadmbcinfo mbcinfo;
//
//     if (!len)
//         return ret;
//     if (!MSVCRT_CHECK_PMT(str))
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (!mbcinfo->ismbcodepage || c < 256)
//         return u__strnset(str, c, len); /* ASCII CP or SB char */
//
//     c &= 0xffff; /* Strip high bits */
//
//     while (str[0] && str[1] && (len > 1))
//     {
//         *str++ = c >> 8;
//         len--;
//         *str++ = c & 0xff;
//         len--;
//     }
//     if (len && str[0])
//     {
//         /* as per msdn pad with a blank character */
//         str[0] = ' ';
//     }
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_mbsnbset(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsnbset(unsigned char *str, unsigned int c, size_t len)
// {
//     return _mbsnbset_l(str, c, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbsnset(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsnset_l(unsigned char* str, unsigned int c, size_t len, _locale_t locale)
// {
//     unsigned char *ret = str;
//     pthreadmbcinfo mbcinfo;
//
//     if (!len)
//         return ret;
//     if (!MSVCRT_CHECK_PMT(str))
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (!mbcinfo->ismbcodepage || c < 256)
//         return u__strnset(str, c, len); /* ASCII CP or SB char */
//
//     c &= 0xffff; /* Strip high bits */
//
//     while (str[0] && str[1] && len--)
//     {
//         *str++ = c >> 8;
//         *str++ = c & 0xff;
//     }
//     if (len && str[0])
//         str[0] = '\0'; /* FIXME: OK to shorten? */
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_mbsnset(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsnset(unsigned char* str, unsigned int c, size_t len)
// {
//     return _mbsnset_l(str, c, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbsnccnt_l(MSVCRT.@)
//  * 'c' is for 'character'.
//  */
// size_t CDECL _mbsnccnt_l(const unsigned char* str, size_t len, _locale_t locale)
// {
//     size_t ret;
//     pthreadmbcinfo mbcinfo;
//
//     if (!len)
//         return 0;
//     if (!MSVCRT_CHECK_PMT(str))
//         return 0;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         ret = 0;
//         while (*str && len-- > 0)
//         {
//             if (_ismbblead_l(*str, locale))
//             {
//                 if (!len)
//                     break;
//                 len--;
//                 str++;
//             }
//             str++;
//             ret++;
//         }
//         return ret;
//     }
//     ret = u_strlen(str);
//     return min(ret, len); /* ASCII CP */
// }
//
// /*********************************************************************
//  *		_mbsnccnt(MSVCRT.@)
//  * 'c' is for 'character'.
//  */
// size_t CDECL _mbsnccnt(const unsigned char* str, size_t len)
// {
//     return _mbsnccnt_l(str, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbsnbcnt_l(MSVCRT.@)
//  * 'b' is for byte count.
//  */
// size_t CDECL _mbsnbcnt_l(const unsigned char* str, size_t len, _locale_t locale)
// {
//     size_t ret;
//     pthreadmbcinfo mbcinfo;
//
//     if (!len)
//         return 0;
//     if (!MSVCRT_CHECK_PMT(str))
//         return 0;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//     if (mbcinfo->ismbcodepage)
//     {
//         const unsigned char* xstr = str;
//         while (*xstr && len-- > 0)
//         {
//             if (_ismbblead_l(*xstr++, locale))
//                 xstr++;
//         }
//         return xstr - str;
//     }
//     ret = u_strlen(str);
//     return min(ret, len); /* ASCII CP */
// }
//
// /*********************************************************************
//  *		_mbsnbcnt(MSVCRT.@)
//  * 'b' is for byte count.
//  */
// size_t CDECL _mbsnbcnt(const unsigned char* str, size_t len)
// {
//     return _mbsnbcnt_l(str, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbsnbcat_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsnbcat_l(unsigned char *dst, const unsigned char *src, size_t len, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
// #ifdef __REACTOS__
//     if (!len)
//         return dst;
// #endif
//
//     if (!MSVCRT_CHECK_PMT(dst && src))
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         unsigned char *res = dst;
//
//         while (*dst)
//         {
//             if (_ismbblead_l(*dst++, locale))
//             {
//                 if (*dst)
//                 {
//                     dst++;
//                 }
//                 else
//                 {
//                     /* as per msdn overwrite the lead byte in front of '\0' */
//                     dst--;
//                     break;
//                 }
//             }
//         }
//         while (*src && len--) *dst++ = *src++;
//         *dst = '\0';
//         return res;
//     }
//     return u_strncat(dst, src, len); /* ASCII CP */
// }
//
// /*********************************************************************
//  *		_mbsnbcat(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsnbcat(unsigned char *dst, const unsigned char *src, size_t len)
// {
//     return _mbsnbcat_l(dst, src, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbsnbcat_s_l(MSVCRT.@)
//  */
// int CDECL _mbsnbcat_s_l(unsigned char *dst, size_t size, const unsigned char *src, size_t len, _locale_t locale)
// {
//     unsigned char *ptr = dst;
//     size_t i;
//     pthreadmbcinfo mbcinfo;
//
//     if (!dst && !size && !len)
//         return 0;
//
//     if (!MSVCRT_CHECK_PMT(dst && size && src))
//     {
//         if (dst && size)
//             *dst = '\0';
//         return EINVAL;
//     }
//
//     /* Find the null terminator of the destination buffer. */
//     while (size && *ptr)
//         size--, ptr++;
//
//     if (!size)
//     {
//         *dst = '\0';
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     /* If necessary, check that the character preceding the null terminator is
//      * a lead byte and move the pointer back by one for later overwrite. */
//     if (ptr != dst && mbcinfo->ismbcodepage && _ismbblead_l(*(ptr - 1), locale))
//         size++, ptr--;
//
//     for (i = 0; *src && i < len; i++)
//     {
//         *ptr++ = *src++;
//         size--;
//
//         if (!size)
//         {
//             *dst = '\0';
//             *_errno() = ERANGE;
//             return ERANGE;
//         }
//     }
//
//     *ptr = '\0';
//     return 0;
// }
//
// /*********************************************************************
//  *		_mbsnbcat_s(MSVCRT.@)
//  */
// int CDECL _mbsnbcat_s(unsigned char *dst, size_t size, const unsigned char *src, size_t len)
// {
//     return _mbsnbcat_s_l(dst, size, src, len, NULL);
// }
//
// /*********************************************************************
//  *		_mbsncat_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsncat_l(unsigned char* dst, const unsigned char* src, size_t len, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if (!len)
//         return dst;
//
//     if (!MSVCRT_CHECK_PMT(dst && src))
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         unsigned char *res = dst;
//         while (*dst)
//         {
//             if (_ismbblead_l(*dst++, locale))
//                 dst++;
//         }
//         while (*src && len--)
//         {
//             *dst++ = *src;
//             if (_ismbblead_l(*src++, locale))
//                 *dst++ = *src++;
//         }
//         *dst = '\0';
//         return res;
//     }
//     return u_strncat(dst, src, len); /* ASCII CP */
// }
//
// /*********************************************************************
//  *		_mbsncat(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsncat(unsigned char* dst, const unsigned char* src, size_t len)
// {
//     return _mbsncat_l(dst, src, len, NULL);
// }
//
// /*********************************************************************
//  *              _mbslwr_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbslwr_l(unsigned char *s, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//     unsigned char *ret = s;
//
//     if (!s)
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         unsigned int c;
//
//         while (*s)
//         {
//             c = _mbctolower_l(_mbsnextc_l(s, locale), locale);
//             /* Note that I assume that the size of the character is unchanged */
//             if (c > 255)
//             {
//                 *s++ = (c >> 8);
//                 c = c & 0xff;
//             }
//             *s++ = c;
//         }
//     }
//     else
//     {
//         for ( ; *s; s++) *s = _tolower_l(*s, locale);
//     }
//     return ret;
// }
//
// /*********************************************************************
//  *              _mbslwr(MSVCRT.@)
//  */
// unsigned char* CDECL _mbslwr(unsigned char *s)
// {
//     return _mbslwr_l(s, NULL);
// }
//
// /*********************************************************************
//  *              _mbslwr_s_l(MSVCRT.@)
//  */
// int CDECL _mbslwr_s_l(unsigned char* s, size_t len, _locale_t locale)
// {
//   unsigned char *p = s;
//   pthreadmbcinfo mbcinfo;
//
//   if (!s && !len)
//     return 0;
//   if (!MSVCRT_CHECK_PMT(s && len))
//     return EINVAL;
//
//   if (locale)
//     mbcinfo = locale->mbcinfo;
//   else
//     mbcinfo = get_mbcinfo();
//
//   if (mbcinfo->ismbcodepage)
//   {
//     unsigned int c;
//     for ( ; *s && len > 0; len--)
//     {
//       c = _mbctolower_l(_mbsnextc_l(s, locale), locale);
//       /* Note that I assume that the size of the character is unchanged */
//       if (c > 255)
//       {
//           *s++=(c>>8);
//           c=c & 0xff;
//       }
//       *s++=c;
//     }
//   }
//   else
//   {
//     for ( ; *s && len > 0; s++, len--)
//       *s = _tolower_l(*s, locale);
//   }
//
//   if (!MSVCRT_CHECK_PMT(len))
//   {
//     *p = 0;
//     return EINVAL;
//   }
//   *s = 0;
//   return 0;
// }
//
// /*********************************************************************
//  *              _mbslwr_s(MSVCRT.@)
//  */
// int CDECL _mbslwr_s(unsigned char* str, size_t len)
// {
//   return _mbslwr_s_l(str, len, NULL);
// }
//
// /*********************************************************************
//  *              _mbsupr_l(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsupr_l(unsigned char* s, _locale_t locale)
// {
//     unsigned char *ret = s;
//     pthreadmbcinfo mbcinfo;
//
//     if (!MSVCRT_CHECK_PMT(s))
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (mbcinfo->ismbcodepage)
//     {
//         unsigned int c;
//         while (*s)
//         {
//             c = _mbctoupper_l(_mbsnextc_l(s, locale), locale);
//             /* Note that I assume that the size of the character is unchanged */
//             if (c > 255)
//             {
//                 *s++ = (c >> 8);
//                 c = c & 0xff;
//             }
//             *s++ = c;
//         }
//     }
//     else
//         for ( ; *s; s++) *s = _toupper_l(*s, locale);
//     return ret;
// }
//
// /*********************************************************************
//  *              _mbsupr(MSVCRT.@)
//  */
// unsigned char* CDECL _mbsupr(unsigned char* s)
// {
//    return _mbsupr_l(s, NULL);
// }
//
// /*********************************************************************
//  *              _mbsupr_s_l(MSVCRT.@)
//  */
// int CDECL _mbsupr_s_l(unsigned char* s, size_t len, _locale_t locale)
// {
//   unsigned char *p = s;
//
//   if (!s && !len)
//     return 0;
//   if (!MSVCRT_CHECK_PMT(s && len))
//     return EINVAL;
//
//   if (get_mbcinfo()->ismbcodepage)
//   {
//     unsigned int c;
//     for ( ; *s && len > 0; len--)
//     {
//       c = _mbctoupper_l(_mbsnextc_l(s, locale), locale);
//       /* Note that I assume that the size of the character is unchanged */
//       if (c > 255)
//       {
//           *s++=(c>>8);
//           c=c & 0xff;
//       }
//       *s++=c;
//     }
//   }
//   else
//   {
//     for ( ; *s && len > 0; s++, len--)
//       *s = _toupper_l(*s, locale);
//   }
//
//   if (!MSVCRT_CHECK_PMT(len))
//   {
//     *p = 0;
//     return EINVAL;
//   }
//   *s = 0;
//   return 0;
// }
//
// /*********************************************************************
//  *              _mbsupr_s(MSVCRT.@)
//  */
// int CDECL _mbsupr_s(unsigned char* s, size_t len)
// {
//   return _mbsupr_s_l(s, len, NULL);
// }
//
// /*********************************************************************
//  *              _mbsspn_l (MSVCRT.@)
//  */
// size_t CDECL _mbsspn_l(const unsigned char* string,
//         const unsigned char* set, _locale_t locale)
// {
//     const unsigned char *p, *q;
//
//     if (!MSVCRT_CHECK_PMT(string && set))
//         return 0;
//
//     for (p = string; *p; p++)
//     {
//         for (q = set; *q; q++)
//         {
//             if (_ismbblead_l(*q, locale))
//             {
//                 /* duplicate a bug in native implementation */
//                 if (!q[1]) break;
//
//                 if (p[0] == q[0] && p[1] == q[1])
//                 {
//                     p++;
//                     break;
//                 }
//                 q++;
//             }
//             else
//             {
//                 if (p[0] == q[0]) break;
//             }
//         }
//         if (!*q) break;
//     }
//     return p - string;
// }
//
// /*********************************************************************
//  *              _mbsspn (MSVCRT.@)
//  */
// size_t CDECL _mbsspn(const unsigned char* string, const unsigned char* set)
// {
//     return _mbsspn_l(string, set, NULL);
// }
//
// /*********************************************************************
//  *              _mbsspnp_l (MSVCRT.@)
//  */
// unsigned char* CDECL _mbsspnp_l(const unsigned char* string, const unsigned char* set, _locale_t locale)
// {
//     if (!MSVCRT_CHECK_PMT(string && set))
//         return 0;
//
//     string += _mbsspn_l(string, set, locale);
//     return *string ? (unsigned char*)string : NULL;
// }
//
// /*********************************************************************
//  *              _mbsspnp (MSVCRT.@)
//  */
// unsigned char* CDECL _mbsspnp(const unsigned char* string, const unsigned char* set)
// {
//     return _mbsspnp_l(string, set, NULL);
// }
//
// /*********************************************************************
//  *		_mbscspn_l (MSVCRT.@)
//  */
// size_t CDECL _mbscspn_l(const unsigned char* str,
//         const unsigned char* cmp, _locale_t locale)
// {
//     const unsigned char *p, *q;
//
//     for (p = str; *p; p++)
//     {
//         for (q = cmp; *q; q++)
//         {
//             if (_ismbblead_l(*q, locale))
//             {
//                 /* duplicate a bug in native implementation */
//                 if (!q[1]) return 0;
//
//                 if (p[0] == q[0] && p[1] == q[1])
//                     return p - str;
//                 q++;
//             }
//             else if (p[0] == q[0])
//                 return p - str;
//         }
//     }
//     return p - str;
// }
//
// /*********************************************************************
//  *		_mbscspn (MSVCRT.@)
//  */
// size_t CDECL _mbscspn(const unsigned char* str, const unsigned char* cmp)
// {
//     return _mbscspn_l(str, cmp, NULL);
// }
//
// /*********************************************************************
//  *              _mbsrev_l (MSVCRT.@)
//  */
// unsigned char* CDECL _mbsrev_l(unsigned char* str, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//     unsigned char *p, tmp;
//
//     if (!MSVCRT_CHECK_PMT(str))
//         return NULL;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (!mbcinfo->ismbcodepage)
//         return u__strrev(str);
//
//     for (p = str; *p; p++)
//     {
//         if (_ismbblead_l(*p, locale))
//         {
//             if (p[1])
//             {
//                 tmp = p[0];
//                 p[0] = p[1];
//                 p[1] = tmp;
//                 p++;
//             }
//             else
//             {
//                 /* drop trailing lead char */
//                 p[0] = 0;
//             }
//         }
//     }
//     return u__strrev(str);
// }
//
// /*********************************************************************
//  *              _mbsrev (MSVCRT.@)
//  */
// unsigned char* CDECL _mbsrev(unsigned char* str)
// {
//     return _mbsrev_l(str, NULL);
// }
//
// /*********************************************************************
//  *		_mbspbrk_l (MSVCRT.@)
//  */
// unsigned char* CDECL _mbspbrk_l(const unsigned char *str,
//         const unsigned char *accept, _locale_t locale)
// {
//     const unsigned char* p;
//     pthreadmbcinfo mbcinfo;
//
//     if (locale)
//         mbcinfo = locale->mbcinfo;
//     else
//         mbcinfo = get_mbcinfo();
//
//     if (!mbcinfo->ismbcodepage)
//         return u_strpbrk(str, accept);
//
//     if (!MSVCRT_CHECK_PMT(str && accept))
//         return NULL;
//
//     while (*str)
//     {
//         for (p = accept; *p; p += (_ismbblead_l(*p, locale) ? 2 : 1))
//         {
//             if (*p == *str)
//                 if (!_ismbblead_l(*p, locale) || p[1] == str[1])
//                     return (unsigned char*)str;
//         }
//         str += (_ismbblead_l(*str, locale) ? 2 : 1);
//     }
//     return NULL;
// }
//
// /*********************************************************************
//  *		_mbspbrk (MSVCRT.@)
//  */
// unsigned char* CDECL _mbspbrk(const unsigned char *str, const unsigned char *accept)
// {
//     return _mbspbrk_l(str, accept, NULL);
// }
//
// /*
//  * Functions depending on locale codepage
//  */
//
// /*********************************************************************
//  *		_mblen_l(MSVCRT.@)
//  * REMARKS
//  *  Unlike most of the multibyte string functions this function uses
//  *  the locale codepage, not the codepage set by _setmbcp
//  */
// int CDECL _mblen_l(const char* str, size_t size, _locale_t locale)
// {
//     pthreadlocinfo locinfo;
//
//     if (!str || !*str || !size)
//         return 0;
//
//     if (locale)
//         locinfo = locale->locinfo;
//     else
//         locinfo = get_locinfo();
//
//     if (locinfo->mb_cur_max == 1)
//         return 1; /* ASCII CP */
//     return !_isleadbyte_l((unsigned char)*str, locale) ? 1 : (size > 1 ? 2 : -1);
// }
//
// /*********************************************************************
//  *		mblen(MSVCRT.@)
//  */
// int CDECL mblen(const char* str, size_t size)
// {
//     return _mblen_l(str, size, NULL);
// }
//
// /*********************************************************************
//  *              mbrlen(MSVCRT.@)
//  */
// size_t CDECL mbrlen(const char *str, size_t len, mbstate_t *state)
// {
//     mbstate_t s = (state ? *state : 0);
//     size_t ret;
//
//     if(!len || !str || !*str)
//         return 0;
//
//     if(get_locinfo()->mb_cur_max == 1) {
//         return 1;
//     }else if(!s && isleadbyte((unsigned char)*str)) {
//         if(len == 1) {
//             s = (unsigned char)*str;
//             ret = -2;
//         }else {
//             ret = 2;
//         }
//     }else if(!s) {
//         ret = 1;
//     }else {
//         s = 0;
//         ret = 2;
//     }
//
//     if(state)
//         *state = s;
//     return ret;
// }
//
// /*********************************************************************
//  *		_mbstrlen_l(MSVCRT.@)
//  */
// size_t CDECL _mbstrlen_l(const char* str, _locale_t locale)
// {
//     pthreadlocinfo locinfo;
//
//     if(!locale)
//         locinfo = get_locinfo();
//     else
//         locinfo = locale->locinfo;
//
//     if(locinfo->mb_cur_max > 1) {
//         size_t len;
//         len = MultiByteToWideChar(locinfo->lc_codepage, MB_ERR_INVALID_CHARS,
//                                   str, -1, NULL, 0);
//         if (!len) {
//             *_errno() = EILSEQ;
//             return -1;
//         }
//         return len - 1;
//     }
//
//     return strlen(str);
// }
//
// /*********************************************************************
//  *		_mbstrlen(MSVCRT.@)
//  */
// size_t CDECL _mbstrlen(const char* str)
// {
//     return _mbstrlen_l(str, NULL);
// }
//
// /*********************************************************************
//  *		_mbtowc_l(MSVCRT.@)
//  */
// int CDECL _mbtowc_l(wchar_t *dst, const char* str, size_t n, _locale_t locale)
// {
//     pthreadlocinfo locinfo;
//     wchar_t tmpdst;
//
//     if(!locale)
//         locinfo = get_locinfo();
//     else
//         locinfo = locale->locinfo;
//
//     if(n <= 0 || !str)
//         return 0;
//
//     if(!*str) {
//         if(dst) *dst = 0;
//         return 0;
//     }
//
//     if(!locinfo->lc_codepage) {
//         if(dst) *dst = (unsigned char)*str;
//         return 1;
//     }
//     if(n>=2 && _isleadbyte_l((unsigned char)*str, locale)) {
//         if(!MultiByteToWideChar(locinfo->lc_codepage, 0, str, 2, &tmpdst, 1))
//             return -1;
//         if(dst) *dst = tmpdst;
//         return 2;
//     }
//     if(!MultiByteToWideChar(locinfo->lc_codepage, 0, str, 1, &tmpdst, 1))
//         return -1;
//     if(dst) *dst = tmpdst;
//     return 1;
// }
//
// /*********************************************************************
//  *              mbtowc(MSVCRT.@)
//  */
// int CDECL mbtowc(wchar_t *dst, const char* str, size_t n)
// {
//     return _mbtowc_l(dst, str, n, NULL);
// }
//
// /*********************************************************************
//  *              btowc(MSVCRT.@)
//  */
// wint_t CDECL btowc(int c)
// {
//     unsigned char letter = c;
//     wchar_t ret;
//
//     if(c == EOF)
//         return WEOF;
//     if(!get_locinfo()->lc_codepage)
//         return c & 255;
//     if(!MultiByteToWideChar(get_locinfo()->lc_codepage,
//                 MB_ERR_INVALID_CHARS, (LPCSTR)&letter, 1, &ret, 1))
//         return WEOF;
//
//     return ret;
// }
//
// /*********************************************************************
//  *              mbrtowc(MSVCRT.@)
//  */
// size_t CDECL mbrtowc(wchar_t *dst, const char *str,
//         size_t n, mbstate_t *state)
// {
//     pthreadlocinfo locinfo = get_locinfo();
//     mbstate_t s = (state ? *state : 0);
//     char tmpstr[2];
//     int len = 0;
//
//     if(dst)
//         *dst = 0;
//
//     if(!n || !str || !*str)
//         return 0;
//
//     if(locinfo->mb_cur_max == 1) {
//         tmpstr[len++] = *str;
//     }else if(!s && isleadbyte((unsigned char)*str)) {
//         if(n == 1) {
//             s = (unsigned char)*str;
//             len = -2;
//         }else {
//             tmpstr[0] = str[0];
//             tmpstr[1] = str[1];
//             len = 2;
//         }
//     }else if(!s) {
//         tmpstr[len++] = *str;
//     }else {
//         tmpstr[0] = s;
//         tmpstr[1] = *str;
//         len = 2;
//         s = 0;
//     }
//
//     if(len > 0) {
//         if(!MultiByteToWideChar(locinfo->lc_codepage, 0, tmpstr, len, dst, dst ? 1 : 0))
//             len = -1;
//     }
//
//     if(state)
//         *state = s;
//     return len;
// }
//
// static inline int get_utf8_char_len(char ch)
// {
//     if((ch&0xf8) == 0xf0)
//         return 4;
//     else if((ch&0xf0) == 0xe0)
//         return 3;
//     else if((ch&0xe0) == 0xc0)
//         return 2;
//     return 1;
// }
//
// /*********************************************************************
//  *		_mbstowcs_l(MSVCRT.@)
//  */
// size_t CDECL _mbstowcs_l(wchar_t *wcstr, const char *mbstr,
//         size_t count, _locale_t locale)
// {
//     pthreadlocinfo locinfo;
//     size_t i, size;
//
//     if(!mbstr) {
//         *_errno() = EINVAL;
//         return -1;
//     }
//
//     if(!locale)
//         locinfo = get_locinfo();
//     else
//         locinfo = locale->locinfo;
//
//     if(!locinfo->lc_codepage) {
//         if(!wcstr)
//             return strlen(mbstr);
//
//         for(i=0; i<count; i++) {
//             wcstr[i] = (unsigned char)mbstr[i];
//             if(!wcstr[i]) break;
//         }
//         return i;
//     }
//
//     /* Ignore count parameter */
//     if(!wcstr) {
//         size = MultiByteToWideChar(locinfo->lc_codepage,
//                 MB_ERR_INVALID_CHARS, mbstr, -1, NULL, 0);
//         if(!size) {
//             *_errno() = EILSEQ;
//             return -1;
//         }
//         return size - 1;
//     }
//
//     for(i=0, size=0; i<count; i++) {
//         if(mbstr[size] == '\0')
//             break;
//
//         if(locinfo->lc_codepage == CP_UTF8) {
//             int j, chlen = get_utf8_char_len(mbstr[size]);
//
//             for(j = 1; j < chlen; j++)
//             {
//                 if(!mbstr[size + j])
//                 {
//                     if(count) wcstr[0] = '\0';
//                     *_errno() = EILSEQ;
//                     return -1;
//                 }
//             }
//             size += chlen;
//         }
//         else
//             size += (_isleadbyte_l((unsigned char)mbstr[size], locale) ? 2 : 1);
//     }
//
//     if(size) {
//         size = MultiByteToWideChar(locinfo->lc_codepage,
//                 MB_ERR_INVALID_CHARS, mbstr, size, wcstr, count);
//         if(!size) {
//             if(count) wcstr[0] = '\0';
//             *_errno() = EILSEQ;
//             return -1;
//         }
//     }
//
//     if(size<count)
//         wcstr[size] = '\0';
//
//     return size;
// }
//
// /*********************************************************************
//  *		mbstowcs(MSVCRT.@)
//  */
// size_t CDECL mbstowcs(wchar_t *wcstr,
//         const char *mbstr, size_t count)
// {
//     return _mbstowcs_l(wcstr, mbstr, count, NULL);
// }
//
// /*********************************************************************
//  *              _mbstowcs_s_l(MSVCRT.@)
//  */
// int CDECL _mbstowcs_s_l(size_t *ret, wchar_t *wcstr,
//         size_t size, const char *mbstr, size_t count, _locale_t locale)
// {
//     size_t conv;
//     int err = 0;
//
//     if(!wcstr && !size) {
//         conv = _mbstowcs_l(NULL, mbstr, 0, locale);
//         if(ret)
//             *ret = conv+1;
//         return 0;
//     }
//
//     if (!MSVCRT_CHECK_PMT(wcstr != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mbstr != NULL)) {
//         if(size) wcstr[0] = '\0';
//         return EINVAL;
//     }
//
//     if(count==_TRUNCATE || size<count)
//         conv = size;
//     else
//         conv = count;
//
//     conv = _mbstowcs_l(wcstr, mbstr, conv, locale);
//     if(conv<size)
//         wcstr[conv++] = '\0';
//     else if(conv==size && count==_TRUNCATE && wcstr[conv-1]!='\0') {
//         wcstr[conv-1] = '\0';
//         err = STRUNCATE;
//     }else if(conv==size && wcstr[conv-1]!='\0') {
//         MSVCRT_INVALID_PMT("wcstr[size] is too small", ERANGE);
//         if(size)
//             wcstr[0] = '\0';
//         return ERANGE;
//     }
//
//     if(ret)
//         *ret = conv;
//     return err;
// }
//
// /*********************************************************************
//  *              mbstowcs_s(MSVCRT.@)
//  */
// int CDECL _mbstowcs_s(size_t *ret, wchar_t *wcstr,
//         size_t size, const char *mbstr, size_t count)
// {
//     return _mbstowcs_s_l(ret, wcstr, size, mbstr, count, NULL);
// }
//
// /*********************************************************************
//  *              mbsrtowcs(MSVCRT.@)
//  */
// size_t CDECL mbsrtowcs(wchar_t *wcstr,
//         const char **pmbstr, size_t count, mbstate_t *state)
// {
//     mbstate_t s = (state ? *state : 0);
//     wchar_t tmpdst;
//     size_t ret = 0;
//     const char *p;
//
//     if(!MSVCRT_CHECK_PMT(pmbstr != NULL))
//         return -1;
//
//     p = *pmbstr;
//     while(!wcstr || count>ret) {
//         int ch_len = mbrtowc(&tmpdst, p, 2, &s);
//         if(wcstr)
//             wcstr[ret] = tmpdst;
//
//         if(ch_len < 0) {
//             return -1;
//         }else if(ch_len == 0) {
//             if(wcstr) *pmbstr = NULL;
//             return ret;
//         }
//
//         p += ch_len;
//         ret++;
//     }
//
//     if(wcstr) *pmbstr = p;
//     return ret;
// }
//
// /*********************************************************************
//  *              mbsrtowcs_s(MSVCRT.@)
//  */
// int CDECL mbsrtowcs_s(size_t *ret, wchar_t *wcstr, size_t len,
//         const char **mbstr, size_t count, mbstate_t *state)
// {
//     size_t tmp;
//
//     if(!ret) ret = &tmp;
//     if(!MSVCRT_CHECK_PMT(!!wcstr == !!len)) {
//         *ret = -1;
//         return EINVAL;
//     }
//
//     *ret = mbsrtowcs(wcstr, mbstr, count>len ? len : count, state);
//     if(*ret == -1) {
//         if(wcstr) *wcstr = 0;
//         return *_errno();
//     }
//     (*ret)++;
//     if(*ret > len) {
//         /* no place for terminating '\0' */
//         if(wcstr) *wcstr = 0;
//         return 0;
//     }
//     if(wcstr) wcstr[(*ret)-1] = 0;
//     return 0;
// }
//
// /*********************************************************************
//  *		_mbctohira_l (MSVCRT.@)
//  *
//  *              Converts a sjis katakana character to hiragana.
//  */
// unsigned int CDECL _mbctohira_l(unsigned int c, _locale_t locale)
// {
//     if(_ismbckata_l(c, locale) && c <= 0x8393)
//         return (c - 0x8340 - (c >= 0x837f ? 1 : 0)) + 0x829f;
//     return c;
// }
//
// /*********************************************************************
//  *		_mbctohira (MSVCRT.@)
//  */
// unsigned int CDECL _mbctohira(unsigned int c)
// {
//     return _mbctohira_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_mbctokata_l (MSVCRT.@)
//  *
//  *              Converts a sjis hiragana character to katakana.
//  */
// unsigned int CDECL _mbctokata_l(unsigned int c, _locale_t locale)
// {
//     if(_ismbchira_l(c, locale))
//         return (c - 0x829f) + 0x8340 + (c >= 0x82de ? 1 : 0);
//     return c;
// }
//
//
// /*********************************************************************
//  *		_mbctokata (MSVCRT.@)
//  */
// unsigned int CDECL _mbctokata(unsigned int c)
// {
//     return _mbctokata_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_ismbcl0_l (MSVCRT.@)
//  */
// int CDECL _ismbcl0_l(unsigned int c, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if(mbcinfo->mbcodepage == 932)
//     {
//         /* JIS non-Kanji */
//         return _ismbclegal_l(c, locale) && c >= 0x8140 && c <= 0x889e;
//     }
//
//     return 0;
// }
//
// /*********************************************************************
//  *		_ismbcl0 (MSVCRT.@)
//  */
// int CDECL _ismbcl0(unsigned int c)
// {
//     return _ismbcl0_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_ismbcl1_l (MSVCRT.@)
//  */
// int CDECL _ismbcl1_l(unsigned int c, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if(mbcinfo->mbcodepage == 932)
//     {
//         /* JIS level-1 */
//         return _ismbclegal_l(c, locale) && c >= 0x889f && c <= 0x9872;
//     }
//
//     return 0;
// }
//
// /*********************************************************************
//  *		_ismbcl1 (MSVCRT.@)
//  */
// int CDECL _ismbcl1(unsigned int c)
// {
//     return _ismbcl1_l(c, NULL);
// }
//
// /*********************************************************************
//  *		_ismbcl2_l (MSVCRT.@)
//  */
// int CDECL _ismbcl2_l(unsigned int c, _locale_t locale)
// {
//     pthreadmbcinfo mbcinfo;
//
//     if(!locale)
//         mbcinfo = get_mbcinfo();
//     else
//         mbcinfo = locale->mbcinfo;
//
//     if(mbcinfo->mbcodepage == 932)
//     {
//         /* JIS level-2 */
//         return _ismbclegal_l(c, locale) && c >= 0x989f && c <= 0xeaa4;
//     }
//
//     return 0;
// }
//
// /*********************************************************************
//  *		_ismbcl2 (MSVCRT.@)
//  */
// int CDECL _ismbcl2(unsigned int c)
// {
//     return _ismbcl2_l(c, NULL);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
