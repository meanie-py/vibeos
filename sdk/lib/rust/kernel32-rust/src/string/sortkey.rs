use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::string::nls_subst::*;
use crate::string::collation::collation_table;

// ---- file-scope preamble (translated) ----
pub type WCHAR = u16;

// NOTE: the C `#define get_char_typeW(x) iswctype((x) >> 8, (x) & 0xFF)` is
// already provided faithfully (over the Wine wctype table) by
// `nls_subst::get_char_typeW(ch: WCHAR) -> u16`, glob-imported above. We use
// that one instead of a local shadow so call sites pass WCHAR (u16) as in C.
#[no_mangle]
pub unsafe extern "C" fn wine_get_sortkey(
    flags: i32,
    src: *const u16,
    srclen: i32,
    dst: *mut i8,
    dstlen: i32,
) -> i32 {
    let mut dummy: [u16; 4] = [0; 4];
    let mut key_len: [i32; 4] = [0; 4];
    let mut key_ptr: [*mut i8; 4] = [core::ptr::null_mut(); 4];
    let src_save: *const u16 = src;
    let srclen_save: i32 = srclen;

    key_len[0] = 0;
    key_len[1] = 0;
    key_len[2] = 0;
    key_len[3] = 0;
    let mut srclen = srclen;
    let mut src = src;
    while srclen != 0 {
        let mut decomposed_len: u32 = 1;
        dummy[0] = *src;
        if decomposed_len != 0 {
            let mut i: u32 = 0;
            while i < decomposed_len {
                let wch: u16 = dummy[i as usize];
                let ce: u32;

                if (flags & NORM_IGNORESYMBOLS as i32) != 0 && (get_char_typeW(wch) & (C1_PUNCT | C1_SPACE)) != 0 {
                    i += 1;
                    continue;
                }

                let mut wch = wch;
                if (flags & 0x00000001) != 0 {
                    wch = tolowerW(wch);
                }

                ce = collation_table[(collation_table[(wch >> 8) as usize] as usize) + (wch & 0xff) as usize];
                if ce != u32::MAX {
                    if (ce >> 16) != 0 {
                        key_len[0] += 2;
                    }
                    if ((ce >> 8) & 0xff) != 0 {
                        key_len[1] += 1;
                    }
                    if ((ce >> 4) & 0x0f) != 0 {
                        key_len[2] += 1;
                    }
                    if (ce & 1) != 0 {
                        if (wch >> 8) != 0 {
                            key_len[3] += 1;
                        }
                        key_len[3] += 1;
                    }
                } else {
                    key_len[0] += 2;
                    if (wch >> 8) != 0 {
                        key_len[0] += 1;
                    }
                    if (wch & 0xff) != 0 {
                        key_len[0] += 1;
                    }
                }
                i += 1;
            }
        }
        srclen -= 1;
        src = src.offset(1);
    }

    if dstlen == 0 {
        return key_len[0] + key_len[1] + key_len[2] + key_len[3] + 4 + 1;
    }

    if dstlen < key_len[0] + key_len[1] + key_len[2] + key_len[3] + 4 + 1 {
        return 0;
    }

    src = src_save;
    srclen = srclen_save;

    key_ptr[0] = dst;
    key_ptr[1] = key_ptr[0].offset((key_len[0] + 1) as isize);
    key_ptr[2] = key_ptr[1].offset((key_len[1] + 1) as isize);
    key_ptr[3] = key_ptr[2].offset((key_len[2] + 1) as isize);

    while srclen != 0 {
        let mut decomposed_len: u32 = 1;
        dummy[0] = *src;
        if decomposed_len != 0 {
            let mut i: u32 = 0;
            while i < decomposed_len {
                let wch: u16 = dummy[i as usize];
                let ce: u32;

                if (flags & NORM_IGNORESYMBOLS as i32) != 0 && (get_char_typeW(wch) & (C1_PUNCT | C1_SPACE)) != 0 {
                    i += 1;
                    continue;
                }

                let mut wch = wch;
                if (flags & 0x00000001) != 0 {
                    wch = tolowerW(wch);
                }

                ce = collation_table[(collation_table[(wch >> 8) as usize] as usize) + (wch & 0xff) as usize];
                if ce != u32::MAX {
                    let mut key: u16;
                    key = (ce >> 16) as u16;
                    if key != 0 {
                        *key_ptr[0] = (key >> 8) as i8;
                        key_ptr[0] = key_ptr[0].offset(1);
                        *key_ptr[0] = (key & 0xff) as i8;
                        key_ptr[0] = key_ptr[0].offset(1);
                    }
                    key = ((ce >> 8) & 0xff) as u16;
                    if key != 0 {
                        *key_ptr[1] = (key + 1) as i8;
                        key_ptr[1] = key_ptr[1].offset(1);
                    }
                    key = ((ce >> 4) & 0x0f) as u16;
                    if key != 0 {
                        *key_ptr[2] = (key + 1) as i8;
                        key_ptr[2] = key_ptr[2].offset(1);
                    }
                    if (ce & 1) != 0 {
                        if (wch >> 8) != 0 {
                            *key_ptr[3] = (wch >> 8) as i8;
                            key_ptr[3] = key_ptr[3].offset(1);
                        }
                        if (wch & 0xff) != 0 {
                            *key_ptr[3] = (wch & 0xff) as i8;
                            key_ptr[3] = key_ptr[3].offset(1);
                        }
                    }
                } else {
                    *key_ptr[0] = 0xffu8 as i8;
                    key_ptr[0] = key_ptr[0].offset(1);
                    *key_ptr[0] = 0xfeu8 as i8;
                    key_ptr[0] = key_ptr[0].offset(1);
                    if (wch >> 8) != 0 {
                        *key_ptr[0] = (wch >> 8) as i8;
                        key_ptr[0] = key_ptr[0].offset(1);
                    }
                    if (wch & 0xff) != 0 {
                        *key_ptr[0] = (wch & 0xff) as i8;
                        key_ptr[0] = key_ptr[0].offset(1);
                    }
                }
                i += 1;
            }
        }
        srclen -= 1;
        src = src.offset(1);
    }

    *key_ptr[0] = 1i8;
    *key_ptr[1] = 1i8;
    *key_ptr[2] = 1i8;
    *key_ptr[3] = 1i8;
    key_ptr[3] = key_ptr[3].offset(1);
    *key_ptr[3] = 0i8;

    key_ptr[3].offset_from(dst) as i32
}
unsafe fn compare_unicode_weights(
    flags: i32,
    str1: *const WCHAR,
    len1: i32,
    str2: *const WCHAR,
    len2: i32,
) -> i32 {
    let mut ce1: u32;
    let mut ce2: u32;
    let mut ret: i32;
    let mut str1 = str1;
    let mut len1 = len1;
    let mut str2 = str2;
    let mut len2 = len2;

    while len1 > 0 && len2 > 0 {
        if flags & NORM_IGNORESYMBOLS as i32 != 0 {
            let mut skip = 0;
            if get_char_typeW(*str1) & (C1_PUNCT | C1_SPACE) != 0 {
                str1 = str1.offset(1);
                len1 -= 1;
                skip = 1;
            }
            if get_char_typeW(*str2) & (C1_PUNCT | C1_SPACE) != 0 {
                str2 = str2.offset(1);
                len2 -= 1;
                skip = 1;
            }
            if skip != 0 {
                continue;
            }
        }

        if flags & SORT_STRINGSORT as i32 == 0 {
            if *str1 as i32 == '-' as i32 || *str1 as i32 == '\'' as i32 {
                if *str2 as i32 != '-' as i32 && *str2 as i32 != '\'' as i32 {
                    str1 = str1.offset(1);
                    len1 -= 1;
                    continue;
                }
            } else if *str2 as i32 == '-' as i32 || *str2 as i32 == '\'' as i32 {
                str2 = str2.offset(1);
                len2 -= 1;
                continue;
            }
        }

        ce1 = collation_table[(collation_table[(*str1 as u32 >> 8) as usize] as usize) + (*str1 as u32 & 0xff) as usize];
        ce2 = collation_table[(collation_table[(*str2 as u32 >> 8) as usize] as usize) + (*str2 as u32 & 0xff) as usize];

        if ce1 != u32::MAX && ce2 != u32::MAX {
            ret = ((ce1 >> 16) as i32).wrapping_sub((ce2 >> 16) as i32);
        } else {
            ret = *str1 as i32 - *str2 as i32;
        }

        if ret != 0 {
            return ret;
        }

        str1 = str1.offset(1);
        str2 = str2.offset(1);
        len1 -= 1;
        len2 -= 1;
    }

    while len1 != 0 && *str1 == 0 {
        str1 = str1.offset(1);
        len1 -= 1;
    }
    while len2 != 0 && *str2 == 0 {
        str2 = str2.offset(1);
        len2 -= 1;
    }

    len1 - len2
}
unsafe fn compare_diacritic_weights(
    flags: i32,
    str1: *const WCHAR,
    len1: i32,
    str2: *const WCHAR,
    len2: i32,
) -> i32 {
    let mut ce1: u32;
    let mut ce2: u32;
    let mut ret: i32;
    let mut str1 = str1;
    let mut len1 = len1;
    let mut str2 = str2;
    let mut len2 = len2;

    while len1 > 0 && len2 > 0 {
        if flags & NORM_IGNORESYMBOLS as i32 != 0 {
            let mut skip = 0;
            if get_char_typeW(*str1) & (C1_PUNCT | C1_SPACE) != 0 {
                str1 = str1.offset(1);
                len1 -= 1;
                skip = 1;
            }
            if get_char_typeW(*str2) & (C1_PUNCT | C1_SPACE) != 0 {
                str2 = str2.offset(1);
                len2 -= 1;
                skip = 1;
            }
            if skip != 0 {
                continue;
            }
        }

        ce1 = collation_table[(collation_table[(*str1 as usize) >> 8] + ((*str1 as u32) & 0xff)) as usize];
        ce2 = collation_table[(collation_table[(*str2 as usize) >> 8] + ((*str2 as u32) & 0xff)) as usize];

        if ce1 != u32::MAX && ce2 != u32::MAX {
            ret = ((ce1 >> 8) & 0xff) as i32 - ((ce2 >> 8) & 0xff) as i32;
        } else {
            ret = *str1 as i32 - *str2 as i32;
        }

        if ret != 0 {
            return ret;
        }

        str1 = str1.offset(1);
        str2 = str2.offset(1);
        len1 -= 1;
        len2 -= 1;
    }

    while len1 != 0 && *str1 == 0 {
        str1 = str1.offset(1);
        len1 -= 1;
    }
    while len2 != 0 && *str2 == 0 {
        str2 = str2.offset(1);
        len2 -= 1;
    }

    len1 - len2
}
unsafe fn compare_case_weights(
    flags: i32,
    mut str1: *const WCHAR,
    mut len1: i32,
    mut str2: *const WCHAR,
    mut len2: i32,
) -> i32 {
    let mut ce1: u32 = 0;
    let mut ce2: u32 = 0;
    let mut ret: i32 = 0;

    while len1 > 0 && len2 > 0 {
        if flags & NORM_IGNORESYMBOLS as i32 != 0 {
            let mut skip: i32 = 0;
            if get_char_typeW(*str1) & (C1_PUNCT | C1_SPACE) != 0 {
                str1 = str1.offset(1);
                len1 -= 1;
                skip = 1;
            }
            if get_char_typeW(*str2) & (C1_PUNCT | C1_SPACE) != 0 {
                str2 = str2.offset(1);
                len2 -= 1;
                skip = 1;
            }
            if skip != 0 {
                continue;
            }
        }

        ce1 = collation_table[(collation_table[(*str1 >> 8) as usize] + (*str1 & 0xff) as u32) as usize];
        ce2 = collation_table[(collation_table[(*str2 >> 8) as usize] + (*str2 & 0xff) as u32) as usize];

        if ce1 != u32::MAX && ce2 != u32::MAX {
            ret = ((ce1 >> 4) & 0x0f) as i32 - ((ce2 >> 4) & 0x0f) as i32;
        } else {
            ret = *str1 as i32 - *str2 as i32;
        }

        if ret != 0 {
            return ret;
        }

        str1 = str1.offset(1);
        str2 = str2.offset(1);
        len1 -= 1;
        len2 -= 1;
    }

    while len1 != 0 && *str1 == 0 {
        str1 = str1.offset(1);
        len1 -= 1;
    }
    while len2 != 0 && *str2 == 0 {
        str2 = str2.offset(1);
        len2 -= 1;
    }

    len1 - len2
}
#[no_mangle]
pub unsafe extern "C" fn wine_compare_string(
    flags: i32,
    str1: *const WCHAR,
    len1: i32,
    str2: *const WCHAR,
    len2: i32,
) -> i32 {
    let mut ret: i32;

    ret = compare_unicode_weights(flags, str1, len1, str2, len2);
    if ret == 0 {
        if flags & NORM_IGNORENONSPACE as i32 == 0 {
            ret = compare_diacritic_weights(flags, str1, len1, str2, len2);
        }
        if ret == 0 && flags & NORM_IGNORECASE as i32 == 0 {
            ret = compare_case_weights(flags, str1, len1, str2, len2);
        }
    }
    return ret;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Unicode sort key generation
//  *
//  * Copyright 2003 Dmitry Timoshkov
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
// #include "wine/unicode.h"
//
// #ifdef __REACTOS__
// #define get_char_typeW(x) iswctype((x) >> 8, (x) & 0xFF)
// #endif
// extern unsigned int wine_decompose( WCHAR ch, WCHAR *dst, unsigned int dstlen );
// extern const unsigned int collation_table[];
//
// /*
//  * flags - normalization NORM_* flags
//  *
//  * FIXME: 'variable' flag not handled
//  */
// int wine_get_sortkey(int flags, const WCHAR *src, int srclen, char *dst, int dstlen)
// {
//     WCHAR dummy[4]; /* no decomposition is larger than 4 chars */
//     int key_len[4];
//     char *key_ptr[4];
//     const WCHAR *src_save = src;
//     int srclen_save = srclen;
//
//     key_len[0] = key_len[1] = key_len[2] = key_len[3] = 0;
//     for (; srclen; srclen--, src++)
//     {
//         unsigned int i, decomposed_len = 1;/*wine_decompose(*src, dummy, 4);*/
//         dummy[0] = *src;
//         if (decomposed_len)
//         {
//             for (i = 0; i < decomposed_len; i++)
//             {
//                 WCHAR wch = dummy[i];
//                 unsigned int ce;
//
//                 /* tests show that win2k just ignores NORM_IGNORENONSPACE,
//                  * and skips white space and punctuation characters for
//                  * NORM_IGNORESYMBOLS.
//                  */
//                 if ((flags & NORM_IGNORESYMBOLS) && (get_char_typeW(wch) & (C1_PUNCT | C1_SPACE)))
//                     continue;
//
//                 if (flags & NORM_IGNORECASE) wch = tolowerW(wch);
//
//                 ce = collation_table[collation_table[wch >> 8] + (wch & 0xff)];
//                 if (ce != (unsigned int)-1)
//                 {
//                     if (ce >> 16) key_len[0] += 2;
//                     if ((ce >> 8) & 0xff) key_len[1]++;
//                     if ((ce >> 4) & 0x0f) key_len[2]++;
//                     if (ce & 1)
//                     {
//                         if (wch >> 8) key_len[3]++;
//                         key_len[3]++;
//                     }
//                 }
//                 else
//                 {
//                     key_len[0] += 2;
//                     if (wch >> 8) key_len[0]++;
//                     if (wch & 0xff) key_len[0]++;
// 		}
//             }
//         }
//     }
//
//     if (!dstlen) /* compute length */
//         /* 4 * '\1' + 1 * '\0' + key length */
//         return key_len[0] + key_len[1] + key_len[2] + key_len[3] + 4 + 1;
//
//     if (dstlen < key_len[0] + key_len[1] + key_len[2] + key_len[3] + 4 + 1)
//         return 0; /* overflow */
//
//     src = src_save;
//     srclen = srclen_save;
//
//     key_ptr[0] = dst;
//     key_ptr[1] = key_ptr[0] + key_len[0] + 1;
//     key_ptr[2] = key_ptr[1] + key_len[1] + 1;
//     key_ptr[3] = key_ptr[2] + key_len[2] + 1;
//
//     for (; srclen; srclen--, src++)
//     {
//         unsigned int i, decomposed_len = 1;/*wine_decompose(*src, dummy, 4);*/
//         dummy[0] = *src;
//         if (decomposed_len)
//         {
//             for (i = 0; i < decomposed_len; i++)
//             {
//                 WCHAR wch = dummy[i];
//                 unsigned int ce;
//
//                 /* tests show that win2k just ignores NORM_IGNORENONSPACE,
//                  * and skips white space and punctuation characters for
//                  * NORM_IGNORESYMBOLS.
//                  */
//                 if ((flags & NORM_IGNORESYMBOLS) && (get_char_typeW(wch) & (C1_PUNCT | C1_SPACE)))
//                     continue;
//
//                 if (flags & NORM_IGNORECASE) wch = tolowerW(wch);
//
//                 ce = collation_table[collation_table[wch >> 8] + (wch & 0xff)];
//                 if (ce != (unsigned int)-1)
//                 {
//                     WCHAR key;
//                     if ((key = ce >> 16))
//                     {
//                         *key_ptr[0]++ = key >> 8;
//                         *key_ptr[0]++ = key & 0xff;
//                     }
//                     /* make key 1 start from 2 */
//                     if ((key = (ce >> 8) & 0xff)) *key_ptr[1]++ = key + 1;
//                     /* make key 2 start from 2 */
//                     if ((key = (ce >> 4) & 0x0f)) *key_ptr[2]++ = key + 1;
//                     /* key 3 is always a character code */
//                     if (ce & 1)
//                     {
//                         if (wch >> 8) *key_ptr[3]++ = wch >> 8;
//                         if (wch & 0xff) *key_ptr[3]++ = wch & 0xff;
//                     }
//                 }
//                 else
//                 {
//                     *key_ptr[0]++ = 0xff;
//                     *key_ptr[0]++ = 0xfe;
//                     if (wch >> 8) *key_ptr[0]++ = wch >> 8;
//                     if (wch & 0xff) *key_ptr[0]++ = wch & 0xff;
//                 }
//             }
//         }
//     }
//
//     *key_ptr[0] = '\1';
//     *key_ptr[1] = '\1';
//     *key_ptr[2] = '\1';
//     *key_ptr[3]++ = '\1';
//     *key_ptr[3] = 0;
//
//     return key_ptr[3] - dst;
// }
//
// static inline int compare_unicode_weights(int flags, const WCHAR *str1, int len1,
//                                           const WCHAR *str2, int len2)
// {
//     unsigned int ce1, ce2;
//     int ret;
//
//     /* 32-bit collation element table format:
//      * unicode weight - high 16 bit, diacritic weight - high 8 bit of low 16 bit,
//      * case weight - high 4 bit of low 8 bit.
//      */
//     while (len1 > 0 && len2 > 0)
//     {
//         if (flags & NORM_IGNORESYMBOLS)
//         {
//             int skip = 0;
//             /* FIXME: not tested */
//             if (get_char_typeW(*str1) & (C1_PUNCT | C1_SPACE))
//             {
//                 str1++;
//                 len1--;
//                 skip = 1;
//             }
//             if (get_char_typeW(*str2) & (C1_PUNCT | C1_SPACE))
//             {
//                 str2++;
//                 len2--;
//                 skip = 1;
//             }
//             if (skip) continue;
//         }
//
//        /* hyphen and apostrophe are treated differently depending on
//         * whether SORT_STRINGSORT specified or not
//         */
//         if (!(flags & SORT_STRINGSORT))
//         {
//             if (*str1 == '-' || *str1 == '\'')
//             {
//                 if (*str2 != '-' && *str2 != '\'')
//                 {
//                     str1++;
//                     len1--;
//                     continue;
//                 }
//             }
//             else if (*str2 == '-' || *str2 == '\'')
//             {
//                 str2++;
//                 len2--;
//                 continue;
//             }
//         }
//
//         ce1 = collation_table[collation_table[*str1 >> 8] + (*str1 & 0xff)];
//         ce2 = collation_table[collation_table[*str2 >> 8] + (*str2 & 0xff)];
//
//         if (ce1 != (unsigned int)-1 && ce2 != (unsigned int)-1)
//             ret = (ce1 >> 16) - (ce2 >> 16);
//         else
//             ret = *str1 - *str2;
//
//         if (ret) return ret;
//
//         str1++;
//         str2++;
//         len1--;
//         len2--;
//     }
//     while (len1 && !*str1)
//     {
//         str1++;
//         len1--;
//     }
//     while (len2 && !*str2)
//     {
//         str2++;
//         len2--;
//     }
//     return len1 - len2;
// }
//
// static inline int compare_diacritic_weights(int flags, const WCHAR *str1, int len1,
//                                             const WCHAR *str2, int len2)
// {
//     unsigned int ce1, ce2;
//     int ret;
//
//     /* 32-bit collation element table format:
//      * unicode weight - high 16 bit, diacritic weight - high 8 bit of low 16 bit,
//      * case weight - high 4 bit of low 8 bit.
//      */
//     while (len1 > 0 && len2 > 0)
//     {
//         if (flags & NORM_IGNORESYMBOLS)
//         {
//             int skip = 0;
//             /* FIXME: not tested */
//             if (get_char_typeW(*str1) & (C1_PUNCT | C1_SPACE))
//             {
//                 str1++;
//                 len1--;
//                 skip = 1;
//             }
//             if (get_char_typeW(*str2) & (C1_PUNCT | C1_SPACE))
//             {
//                 str2++;
//                 len2--;
//                 skip = 1;
//             }
//             if (skip) continue;
//         }
//
//         ce1 = collation_table[collation_table[*str1 >> 8] + (*str1 & 0xff)];
//         ce2 = collation_table[collation_table[*str2 >> 8] + (*str2 & 0xff)];
//
//         if (ce1 != (unsigned int)-1 && ce2 != (unsigned int)-1)
//             ret = ((ce1 >> 8) & 0xff) - ((ce2 >> 8) & 0xff);
//         else
//             ret = *str1 - *str2;
//
//         if (ret) return ret;
//
//         str1++;
//         str2++;
//         len1--;
//         len2--;
//     }
//     while (len1 && !*str1)
//     {
//         str1++;
//         len1--;
//     }
//     while (len2 && !*str2)
//     {
//         str2++;
//         len2--;
//     }
//     return len1 - len2;
// }
//
// static inline int compare_case_weights(int flags, const WCHAR *str1, int len1,
//                                        const WCHAR *str2, int len2)
// {
//     unsigned int ce1, ce2;
//     int ret;
//
//     /* 32-bit collation element table format:
//      * unicode weight - high 16 bit, diacritic weight - high 8 bit of low 16 bit,
//      * case weight - high 4 bit of low 8 bit.
//      */
//     while (len1 > 0 && len2 > 0)
//     {
//         if (flags & NORM_IGNORESYMBOLS)
//         {
//             int skip = 0;
//             /* FIXME: not tested */
//             if (get_char_typeW(*str1) & (C1_PUNCT | C1_SPACE))
//             {
//                 str1++;
//                 len1--;
//                 skip = 1;
//             }
//             if (get_char_typeW(*str2) & (C1_PUNCT | C1_SPACE))
//             {
//                 str2++;
//                 len2--;
//                 skip = 1;
//             }
//             if (skip) continue;
//         }
//
//         ce1 = collation_table[collation_table[*str1 >> 8] + (*str1 & 0xff)];
//         ce2 = collation_table[collation_table[*str2 >> 8] + (*str2 & 0xff)];
//
//         if (ce1 != (unsigned int)-1 && ce2 != (unsigned int)-1)
//             ret = ((ce1 >> 4) & 0x0f) - ((ce2 >> 4) & 0x0f);
//         else
//             ret = *str1 - *str2;
//
//         if (ret) return ret;
//
//         str1++;
//         str2++;
//         len1--;
//         len2--;
//     }
//     while (len1 && !*str1)
//     {
//         str1++;
//         len1--;
//     }
//     while (len2 && !*str2)
//     {
//         str2++;
//         len2--;
//     }
//     return len1 - len2;
// }
//
// int wine_compare_string(int flags, const WCHAR *str1, int len1,
//                         const WCHAR *str2, int len2)
// {
//     int ret;
//
//     ret = compare_unicode_weights(flags, str1, len1, str2, len2);
//     if (!ret)
//     {
//         if (!(flags & NORM_IGNORENONSPACE))
//             ret = compare_diacritic_weights(flags, str1, len1, str2, len2);
//         if (!ret && !(flags & NORM_IGNORECASE))
//             ret = compare_case_weights(flags, str1, len1, str2, len2);
//     }
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
