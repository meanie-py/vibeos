use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::string::nls_subst::*;
// wine_digitmap / wine_compatmap are now Rust statics in the sibling digitmap module
// (DeepSeek emitted wrong-typed extern decls for them; import the real arrays instead).
use crate::string::digitmap::{wine_digitmap, wine_compatmap};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut wine_ligatures: [WCHAR; 35] = [
    0xc6 as ::core::ffi::c_int as WCHAR,
    0xde as ::core::ffi::c_int as WCHAR,
    0xdf as ::core::ffi::c_int as WCHAR,
    0xe6 as ::core::ffi::c_int as WCHAR,
    0xfe as ::core::ffi::c_int as WCHAR,
    0x132 as ::core::ffi::c_int as WCHAR,
    0x133 as ::core::ffi::c_int as WCHAR,
    0x152 as ::core::ffi::c_int as WCHAR,
    0x153 as ::core::ffi::c_int as WCHAR,
    0x1c4 as ::core::ffi::c_int as WCHAR,
    0x1c5 as ::core::ffi::c_int as WCHAR,
    0x1c6 as ::core::ffi::c_int as WCHAR,
    0x1c7 as ::core::ffi::c_int as WCHAR,
    0x1c8 as ::core::ffi::c_int as WCHAR,
    0x1c9 as ::core::ffi::c_int as WCHAR,
    0x1ca as ::core::ffi::c_int as WCHAR,
    0x1cb as ::core::ffi::c_int as WCHAR,
    0x1cc as ::core::ffi::c_int as WCHAR,
    0x1e2 as ::core::ffi::c_int as WCHAR,
    0x1e3 as ::core::ffi::c_int as WCHAR,
    0x1f1 as ::core::ffi::c_int as WCHAR,
    0x1f2 as ::core::ffi::c_int as WCHAR,
    0x1f3 as ::core::ffi::c_int as WCHAR,
    0x1fc as ::core::ffi::c_int as WCHAR,
    0x1fd as ::core::ffi::c_int as WCHAR,
    0x5f0 as ::core::ffi::c_int as WCHAR,
    0x5f1 as ::core::ffi::c_int as WCHAR,
    0x5f2 as ::core::ffi::c_int as WCHAR,
    0xfb00 as ::core::ffi::c_int as WCHAR,
    0xfb01 as ::core::ffi::c_int as WCHAR,
    0xfb02 as ::core::ffi::c_int as WCHAR,
    0xfb03 as ::core::ffi::c_int as WCHAR,
    0xfb04 as ::core::ffi::c_int as WCHAR,
    0xfb05 as ::core::ffi::c_int as WCHAR,
    0xfb06 as ::core::ffi::c_int as WCHAR,
];

static mut wine_expanded_ligatures: [[WCHAR; 4]; 35] = [
    [
        'A' as i32 as WCHAR,
        'E' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'T' as i32 as WCHAR,
        'H' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        's' as i32 as WCHAR,
        's' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'a' as i32 as WCHAR,
        'e' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        't' as i32 as WCHAR,
        'h' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'I' as i32 as WCHAR,
        'J' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'i' as i32 as WCHAR,
        'j' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'O' as i32 as WCHAR,
        'E' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'o' as i32 as WCHAR,
        'e' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'D' as i32 as WCHAR,
        0x17d as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'D' as i32 as WCHAR,
        0x17e as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'd' as i32 as WCHAR,
        0x17e as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'L' as i32 as WCHAR,
        'J' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'L' as i32 as WCHAR,
        'j' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'l' as i32 as WCHAR,
        'j' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'N' as i32 as WCHAR,
        'J' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'N' as i32 as WCHAR,
        'j' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'n' as i32 as WCHAR,
        'j' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x100 as ::core::ffi::c_int as WCHAR,
        0x112 as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x101 as ::core::ffi::c_int as WCHAR,
        0x113 as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'D' as i32 as WCHAR,
        'Z' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'D' as i32 as WCHAR,
        'z' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'd' as i32 as WCHAR,
        'z' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0xc1 as ::core::ffi::c_int as WCHAR,
        0xc9 as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0xe1 as ::core::ffi::c_int as WCHAR,
        0xe9 as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x5d5 as ::core::ffi::c_int as WCHAR,
        0x5d5 as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x5d5 as ::core::ffi::c_int as WCHAR,
        0x5d9 as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x5d9 as ::core::ffi::c_int as WCHAR,
        0x5d9 as ::core::ffi::c_int as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'f' as i32 as WCHAR,
        'f' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'f' as i32 as WCHAR,
        'i' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'f' as i32 as WCHAR,
        'l' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'f' as i32 as WCHAR,
        'f' as i32 as WCHAR,
        'i' as i32 as WCHAR,
        2 as ::core::ffi::c_int as WCHAR,
    ],
    [
        'f' as i32 as WCHAR,
        'f' as i32 as WCHAR,
        'l' as i32 as WCHAR,
        2 as ::core::ffi::c_int as WCHAR,
    ],
    [
        0x17f as ::core::ffi::c_int as WCHAR,
        't' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
    [
        's' as i32 as WCHAR,
        't' as i32 as WCHAR,
        '\0' as i32 as WCHAR,
        1 as ::core::ffi::c_int as WCHAR,
    ],
];

#[repr(C)]
pub struct wine_expanded_ligature_entry {
    pub chars: [WCHAR; 3],
    pub flags: WCHAR,
}
unsafe fn to_unicode_digit(ch: WCHAR) -> WCHAR {
    (ch as i32 + wine_digitmap[(wine_digitmap[(ch as i32 >> 8) as usize] as i32 + (ch as i32 & 0xff)) as usize] as i32) as WCHAR
}
unsafe fn to_unicode_native(ch: WCHAR) -> WCHAR {
    (ch as i32 + wine_compatmap[(wine_compatmap[(ch as i32 >> 8) as usize] as i32 + (ch as i32 & 0xff)) as usize] as i32) as WCHAR
}
unsafe fn get_ligature_len(mut wc: WCHAR) -> i32 {
    let mut low: i32 = 0;
    let mut high: i32 = (core::mem::size_of::<[WCHAR; 35]>() as usize)
        .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
        .wrapping_sub(1) as i32;
    while low <= high {
        let mut pos: i32 = (low + high) / 2;
        if (wine_ligatures[pos as usize] as i32) < wc as i32 {
            low = pos + 1;
        } else if (wine_ligatures[pos as usize] as i32) > wc as i32 {
            high = pos - 1;
        } else {
            return wine_expanded_ligatures[pos as usize][3] as i32;
        }
    }
    return 0;
}
unsafe fn get_ligature(mut wc: WCHAR) -> *const WCHAR {
    static mut empty_ligature: [WCHAR; 4] = [
        '\0' as WCHAR,
        '\0' as WCHAR,
        '\0' as WCHAR,
        0 as WCHAR,
    ];
    let mut low: i32 = 0;
    let mut high: i32 = (core::mem::size_of::<[WCHAR; 35]>() as usize)
        .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
        .wrapping_sub(1) as i32;
    while low <= high {
        let mut pos: i32 = (low + high) / 2;
        if (wine_ligatures[pos as usize] as i32) < wc as i32 {
            low = pos + 1;
        } else if wine_ligatures[pos as usize] as i32 > wc as i32 {
            high = pos - 1;
        } else {
            return core::ptr::addr_of!(wine_expanded_ligatures[pos as usize]) as *const WCHAR;
        }
    }
    return &raw const empty_ligature as *const WCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn wine_fold_string(
    mut flags: i32,
    mut src: *const WCHAR,
    mut srclen: i32,
    mut dst: *mut WCHAR,
    mut dstlen: i32,
) -> i32 {
    let mut dstbase: *mut WCHAR = dst;
    let mut expand: *const WCHAR = core::ptr::null();
    let mut i: i32 = 0;

    if srclen == -1 {
        srclen = (strlenW(src as *const u16) + 1) as i32;
    }

    if dstlen == 0 {
        dstlen = srclen;

        if flags & MAP_EXPAND_LIGATURES as i32 != 0 {
            loop {
                let fresh0 = srclen;
                srclen -= 1;
                if fresh0 == 0 {
                    break;
                }
                dstlen += get_ligature_len(*src);
                src = src.offset(1);
            }
        } else if flags & MAP_COMPOSITE as i32 != 0 {
            /* FIXME */
        } else if flags & MAP_PRECOMPOSED as i32 != 0 {
            /* FIXME */
        }
        return dstlen;
    }

    if srclen > dstlen {
        return 0;
    }

    dstlen -= srclen;

    i = 0;
    while i < srclen {
        let mut ch: WCHAR = *src;

        if flags & MAP_EXPAND_LIGATURES as i32 != 0 {
            expand = get_ligature(ch);
            if *expand.offset(0) != 0 {
                let fresh1 = dstlen;
                dstlen -= 1;
                if fresh1 == 0 {
                    return 0;
                }
                *dst.offset(0) = *expand.offset(0);
                if *expand.offset(2) != 0 {
                    let fresh2 = dstlen;
                    dstlen -= 1;
                    if fresh2 == 0 {
                        return 0;
                    }
                    dst = dst.offset(1);
                    *dst = *expand.offset(1);
                    ch = *expand.offset(2);
                } else {
                    ch = *expand.offset(1);
                }
                dst = dst.offset(1);
            }
        } else if flags & MAP_COMPOSITE as i32 != 0 {
            /* FIXME */
        } else if flags & MAP_PRECOMPOSED as i32 != 0 {
            /* FIXME */
        }

        if flags & MAP_FOLDDIGITS as i32 != 0 {
            ch = to_unicode_digit(ch);
        }
        if flags & MAP_FOLDCZONE as i32 != 0 {
            ch = to_unicode_native(ch);
        }

        let fresh3 = dst;
        dst = dst.offset(1);
        *fresh3 = ch;
        src = src.offset(1);
        i += 1;
    }

    return dst.offset_from(dstbase) as i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * String folding
//  *
//  * Copyright 2003 Jon Griffiths
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
//  */
//
// #include "wine/unicode.h"
//
// static inline WCHAR to_unicode_digit( WCHAR ch )
// {
//     extern const WCHAR wine_digitmap[] DECLSPEC_HIDDEN;
//     return ch + wine_digitmap[wine_digitmap[ch >> 8] + (ch & 0xff)];
// }
//
// static inline WCHAR to_unicode_native( WCHAR ch )
// {
//     extern const WCHAR wine_compatmap[] DECLSPEC_HIDDEN;
//     return ch + wine_compatmap[wine_compatmap[ch >> 8] + (ch & 0xff)];
// }
//
// static const WCHAR wine_ligatures[] =
// {
//     0x00c6, 0x00de, 0x00df, 0x00e6, 0x00fe, 0x0132, 0x0133, 0x0152,
//     0x0153, 0x01c4, 0x01c5, 0x01c6, 0x01c7, 0x01c8, 0x01c9, 0x01ca,
//     0x01cb, 0x01cc, 0x01e2, 0x01e3, 0x01f1, 0x01f2, 0x01f3, 0x01fc,
//     0x01fd, 0x05f0, 0x05f1, 0x05f2, 0xfb00, 0xfb01, 0xfb02, 0xfb03,
//     0xfb04, 0xfb05, 0xfb06
// };
//
// /* Unicode expanded ligatures */
// static const WCHAR wine_expanded_ligatures[][4] =
// {
//     { 'A','E','\0',1 },
//     { 'T','H','\0',1 },
//     { 's','s','\0',1 },
//     { 'a','e','\0',1 },
//     { 't','h','\0',1 },
//     { 'I','J','\0',1 },
//     { 'i','j','\0',1 },
//     { 'O','E','\0',1 },
//     { 'o','e','\0',1 },
//     { 'D',0x017d,'\0',1 },
//     { 'D',0x017e,'\0',1 },
//     { 'd',0x017e,'\0',1 },
//     { 'L','J','\0',1 },
//     { 'L','j','\0',1 },
//     { 'l','j','\0',1 },
//     { 'N','J','\0',1 },
//     { 'N','j','\0',1 },
//     { 'n','j','\0',1 },
//     { 0x0100,0x0112,'\0',1 },
//     { 0x0101,0x0113,'\0',1 },
//     { 'D','Z','\0',1 },
//     { 'D','z','\0',1 },
//     { 'd','z','\0',1 },
//     { 0x00c1,0x00c9,'\0',1 },
//     { 0x00e1,0x00e9,'\0',1 },
//     { 0x05d5,0x05d5,'\0',1 },
//     { 0x05d5,0x05d9,'\0',1 },
//     { 0x05d9,0x05d9,'\0',1 },
//     { 'f','f','\0',1 },
//     { 'f','i','\0',1 },
//     { 'f','l','\0',1 },
//     { 'f','f','i',2 },
//     { 'f','f','l',2 },
//     { 0x017f,'t','\0',1 },
//     { 's','t','\0',1 }
// };
//
// static inline int get_ligature_len( WCHAR wc )
// {
//     int low = 0, high = sizeof(wine_ligatures)/sizeof(WCHAR) -1;
//     while (low <= high)
//     {
//         int pos = (low + high) / 2;
//         if (wine_ligatures[pos] < wc)
//             low = pos + 1;
//         else if (wine_ligatures[pos] > wc)
//             high = pos - 1;
//         else
//             return wine_expanded_ligatures[pos][3];
//     }
//     return 0;
// }
//
// static inline const WCHAR* get_ligature( WCHAR wc )
// {
//     static const WCHAR empty_ligature[] = { '\0','\0','\0', 0 };
//     int low = 0, high = sizeof(wine_ligatures)/sizeof(WCHAR) -1;
//     while (low <= high)
//     {
//         int pos = (low + high) / 2;
//         if (wine_ligatures[pos] < wc)
//             low = pos + 1;
//         else if (wine_ligatures[pos] > wc)
//             high = pos - 1;
//         else
//             return wine_expanded_ligatures[pos];
//     }
//     return empty_ligature;
// }
//
// /* fold a unicode string */
// int wine_fold_string( int flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen )
// {
//     WCHAR *dstbase = dst;
//     const WCHAR *expand;
//     int i;
//
//     if (srclen == -1)
//         srclen = strlenW(src) + 1; /* Include terminating NUL in count */
//
//     if (!dstlen)
//     {
//         /* Calculate the required size for dst */
//         dstlen = srclen;
//
//         if (flags & MAP_EXPAND_LIGATURES)
//         {
//             while (srclen--)
//             {
//                 dstlen += get_ligature_len(*src);
//                 src++;
//             }
//         }
//         else if (flags & MAP_COMPOSITE)
//         {
//             /* FIXME */
//         }
//         else if (flags & MAP_PRECOMPOSED)
//         {
//             /* FIXME */
//         }
//         return dstlen;
//     }
//
//     if (srclen > dstlen)
//         return 0;
//
//     dstlen -= srclen;
//
//     /* Actually perform the mapping(s) specified */
//     for (i = 0; i < srclen; i++)
//     {
//         WCHAR ch = *src;
//
//         if (flags & MAP_EXPAND_LIGATURES)
//         {
//             expand = get_ligature(ch);
//             if (expand[0])
//             {
//                 if (!dstlen--)
//                     return 0;
//                 dst[0] = expand[0];
//                 if (expand[2])
//                 {
//                     if (!dstlen--)
//                         return 0;
//                     *++dst = expand[1];
//                     ch = expand[2];
//                 }
//                 else
//                     ch = expand[1];
//                 dst++;
//             }
//         }
//         else if (flags & MAP_COMPOSITE)
//         {
//             /* FIXME */
//         }
//         else if (flags & MAP_PRECOMPOSED)
//         {
//             /* FIXME */
//         }
//         if (flags & MAP_FOLDDIGITS)
//             ch = to_unicode_digit(ch);
//         if (flags & MAP_FOLDCZONE)
//             ch = to_unicode_native(ch);
//
//         *dst++ = ch;
//         src++;
//     }
//     return dst - dstbase;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
