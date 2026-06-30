use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static b64: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
#[no_mangle]
pub unsafe extern "C" fn encodeBase64(in_buf: PBYTE, in_len: i32, out_buf: *mut i8, max_len: i32, out_len: *mut i32) -> SECURITY_STATUS {
    let mut div: i32;
    let mut i: i32;
    let mut d: PBYTE = in_buf;
    let bytes: i32 = (in_len * 8 + 5) / 6;
    let pad_bytes: i32 = if bytes % 4 != 0 { 4 - (bytes % 4) } else { 0 };

    TRACE!(b"bytes is %d, pad bytes is %d\n\0".as_ptr() as *const i8, bytes, pad_bytes);
    *out_len = bytes + pad_bytes;

    if bytes + pad_bytes + 1 > max_len {
        return SEC_E_BUFFER_TOO_SMALL;
    }

    div = in_len / 3;

    i = 0;
    while div > 0 {
        *out_buf.offset((i + 0) as isize) = b64[((*d.offset(0) >> 2) & 0x3f) as usize] as i8;
        *out_buf.offset((i + 1) as isize) = b64[(((*d.offset(0) << 4) & 0x30) | (*d.offset(1) >> 4 & 0x0f)) as usize] as i8;
        *out_buf.offset((i + 2) as isize) = b64[(((*d.offset(1) << 2) & 0x3c) | (*d.offset(2) >> 6 & 0x03)) as usize] as i8;
        *out_buf.offset((i + 3) as isize) = b64[(*d.offset(2) & 0x3f) as usize] as i8;
        i += 4;
        d = d.offset(3);
        div -= 1;
    }

    match pad_bytes {
        1 => {
            *out_buf.offset((i + 0) as isize) = b64[((*d.offset(0) >> 2) & 0x3f) as usize] as i8;
            *out_buf.offset((i + 1) as isize) = b64[(((*d.offset(0) << 4) & 0x30) | (*d.offset(1) >> 4 & 0x0f)) as usize] as i8;
            *out_buf.offset((i + 2) as isize) = b64[(((*d.offset(1) << 2) & 0x3c)) as usize] as i8;
            *out_buf.offset((i + 3) as isize) = b'=' as i8;
            *out_buf.offset((i + 4) as isize) = 0;
        }
        2 => {
            *out_buf.offset((i + 0) as isize) = b64[((*d.offset(0) >> 2) & 0x3f) as usize] as i8;
            *out_buf.offset((i + 1) as isize) = b64[(((*d.offset(0) << 4) & 0x30)) as usize] as i8;
            *out_buf.offset((i + 2) as isize) = b'=' as i8;
            *out_buf.offset((i + 3) as isize) = b'=' as i8;
            *out_buf.offset((i + 4) as isize) = 0;
        }
        _ => {
            *out_buf.offset(i as isize) = 0;
        }
    }

    SEC_E_OK
}

unsafe fn decode(c: core::ffi::c_char) -> u8 {
    if c >= b'A' as core::ffi::c_char && c <= b'Z' as core::ffi::c_char {
        return (c - b'A' as core::ffi::c_char) as u8;
    }
    if c >= b'a' as core::ffi::c_char && c <= b'z' as core::ffi::c_char {
        return (c - b'a' as core::ffi::c_char + 26) as u8;
    }
    if c >= b'0' as core::ffi::c_char && c <= b'9' as core::ffi::c_char {
        return (c - b'0' as core::ffi::c_char + 52) as u8;
    }
    if c == b'+' as core::ffi::c_char {
        return 62;
    }
    if c == b'/' as core::ffi::c_char {
        return 63;
    } else {
        return 64;
    }
}
#[no_mangle]
pub unsafe extern "C" fn decodeBase64(in_buf: *mut core::ffi::c_char, in_len: core::ffi::c_int, out_buf: PBYTE, max_len: core::ffi::c_int, out_len: *mut core::ffi::c_int) -> SECURITY_STATUS {
    let mut len: core::ffi::c_int = in_len;
    let mut i: core::ffi::c_int;
    let mut d: *mut core::ffi::c_char = in_buf;
    let mut ip0: core::ffi::c_int;
    let mut ip1: core::ffi::c_int;
    let mut ip2: core::ffi::c_int;
    let mut ip3: core::ffi::c_int;

    TRACE!(b"in_len: %d\n\0".as_ptr() as *const core::ffi::c_char, in_len);

    if (in_len % 4) != 0 {
        return SEC_E_INVALID_TOKEN;
    }

    if in_len > max_len {
        return SEC_E_BUFFER_TOO_SMALL;
    }

    i = 0;
    while len > 4 {
        ip0 = decode(*d.offset(0)) as core::ffi::c_int;
        if ip0 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip1 = decode(*d.offset(1)) as core::ffi::c_int;
        if ip1 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip2 = decode(*d.offset(2)) as core::ffi::c_int;
        if ip2 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip3 = decode(*d.offset(3)) as core::ffi::c_int;
        if ip3 > 63 {
            return SEC_E_INVALID_TOKEN;
        }

        *out_buf.offset((i + 0) as isize) = ((ip0 << 2) | (ip1 >> 4)) as u8;
        *out_buf.offset((i + 1) as isize) = ((ip1 << 4) | (ip2 >> 2)) as u8;
        *out_buf.offset((i + 2) as isize) = ((ip2 << 6) | ip3) as u8;
        len -= 4;
        i += 3;
        d = d.offset(4);
    }

    if *d.offset(2) == b'=' as core::ffi::c_char {
        ip0 = decode(*d.offset(0)) as core::ffi::c_int;
        if ip0 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip1 = decode(*d.offset(1)) as core::ffi::c_int;
        if ip1 > 63 {
            return SEC_E_INVALID_TOKEN;
        }

        *out_buf.offset(i as isize) = ((ip0 << 2) | (ip1 >> 4)) as u8;
        i += 1;
    } else if *d.offset(3) == b'=' as core::ffi::c_char {
        ip0 = decode(*d.offset(0)) as core::ffi::c_int;
        if ip0 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip1 = decode(*d.offset(1)) as core::ffi::c_int;
        if ip1 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip2 = decode(*d.offset(2)) as core::ffi::c_int;
        if ip2 > 63 {
            return SEC_E_INVALID_TOKEN;
        }

        *out_buf.offset((i + 0) as isize) = ((ip0 << 2) | (ip1 >> 4)) as u8;
        *out_buf.offset((i + 1) as isize) = ((ip1 << 4) | (ip2 >> 2)) as u8;
        i += 2;
    } else {
        ip0 = decode(*d.offset(0)) as core::ffi::c_int;
        if ip0 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip1 = decode(*d.offset(1)) as core::ffi::c_int;
        if ip1 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip2 = decode(*d.offset(2)) as core::ffi::c_int;
        if ip2 > 63 {
            return SEC_E_INVALID_TOKEN;
        }
        ip3 = decode(*d.offset(3)) as core::ffi::c_int;
        if ip3 > 63 {
            return SEC_E_INVALID_TOKEN;
        }

        *out_buf.offset((i + 0) as isize) = ((ip0 << 2) | (ip1 >> 4)) as u8;
        *out_buf.offset((i + 1) as isize) = ((ip1 << 4) | (ip2 >> 2)) as u8;
        *out_buf.offset((i + 2) as isize) = ((ip2 << 6) | ip3) as u8;
        i += 3;
    }

    *out_len = i;
    SEC_E_OK
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * base64 encoder/decoder
//  *
//  * Copyright 2005 by Kai Blin
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2 of the License, or (at your option) any later version.
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
// #include "precomp.h"
//
// #include <wine/debug.h>
// WINE_DEFAULT_DEBUG_CHANNEL(ntlm);
//
// static const char b64[] =
// "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
//
// SECURITY_STATUS encodeBase64(PBYTE in_buf, int in_len, char* out_buf,
//         int max_len, int *out_len)
// {
//     int div, i;
//     PBYTE d = in_buf;
//     int bytes = (in_len*8 + 5)/6, pad_bytes = (bytes % 4) ? 4 - (bytes % 4) : 0;
//
//     TRACE("bytes is %d, pad bytes is %d\n", bytes, pad_bytes);
//     *out_len = bytes + pad_bytes;
//
//     if(bytes + pad_bytes + 1 > max_len)
//         return SEC_E_BUFFER_TOO_SMALL;
//
//     /* Three bytes of input give 4 chars of output */
//     div = in_len / 3;
//
//     i = 0;
//     while(div > 0)
//     {
//         /* first char is the first 6 bits of the first byte*/
//         out_buf[i + 0] = b64[ ( d[0] >> 2) & 0x3f ];
//         /* second char is the last 2 bits of the first byte and the first 4
//          * bits of the second byte */
//         out_buf[i + 1] = b64[ ((d[0] << 4) & 0x30) | (d[1] >> 4 & 0x0f)];
//         /* third char is the last 4 bits of the second byte and the first 2
//          * bits of the third byte */
//         out_buf[i + 2] = b64[ ((d[1] << 2) & 0x3c) | (d[2] >> 6 & 0x03)];
//         /* fourth char is the remaining 6 bits of the third byte */
//         out_buf[i + 3] = b64[   d[2]       & 0x3f];
//         i += 4;
//         d += 3;
//         div--;
//     }
//
//     switch(pad_bytes)
//     {
//         case 1:
//             /* first char is the first 6 bits of the first byte*/
//             out_buf[i + 0] = b64[ ( d[0] >> 2) & 0x3f ];
//             /* second char is the last 2 bits of the first byte and the first 4
//              * bits of the second byte */
//             out_buf[i + 1] = b64[ ((d[0] << 4) & 0x30) | (d[1] >> 4 & 0x0f)];
//             /* third char is the last 4 bits of the second byte padded with
//              * two zeroes */
//             out_buf[i + 2] = b64[ ((d[1] << 2) & 0x3c) ];
//             /* fourth char is a = to indicate one byte of padding */
//             out_buf[i + 3] = '=';
//             out_buf[i + 4] = 0;
//             break;
//         case 2:
//             /* first char is the first 6 bits of the first byte*/
//             out_buf[i + 0] = b64[ ( d[0] >> 2) & 0x3f ];
//             /* second char is the last 2 bits of the first byte padded with
//              * four zeroes*/
//             out_buf[i + 1] = b64[ ((d[0] << 4) & 0x30)];
//             /* third char is = to indicate padding */
//             out_buf[i + 2] = '=';
//             /* fourth char is = to indicate padding */
//             out_buf[i + 3] = '=';
//             out_buf[i + 4] = 0;
//             break;
//         default:
//             out_buf[i] = 0;
//     }
//
//     return SEC_E_OK;
// }
//
// static inline BYTE decode(char c)
// {
//     if( c >= 'A' && c <= 'Z')
//         return c - 'A';
//     if( c >= 'a' && c <= 'z')
//         return c - 'a' + 26;
//     if( c >= '0' && c <= '9')
//         return c - '0' + 52;
//     if( c == '+')
//         return 62;
//     if( c == '/')
//         return 63;
//     else
//         return 64;
// }
//
// SECURITY_STATUS decodeBase64(char *in_buf, int in_len, PBYTE out_buf,
//         int max_len, int *out_len)
// {
//     int len = in_len, i;
//     char *d = in_buf;
//     int  ip0, ip1, ip2, ip3;
//
//     TRACE("in_len: %d\n", in_len);
//
//     if((in_len % 4) != 0)
//         return SEC_E_INVALID_TOKEN;
//
//     if(in_len > max_len)
//         return SEC_E_BUFFER_TOO_SMALL;
//
//     i = 0;
//     while(len > 4)
//     {
//         if((ip0 = decode(d[0])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip1 = decode(d[1])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip2 = decode(d[2])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip3 = decode(d[3])) > 63)
//             return SEC_E_INVALID_TOKEN;
//
//         out_buf[i + 0] = (ip0 << 2) | (ip1 >> 4);
//         out_buf[i + 1] = (ip1 << 4) | (ip2 >> 2);
//         out_buf[i + 2] = (ip2 << 6) |  ip3;
//         len -= 4;
//         i += 3;
//         d += 4;
//     }
//
//     if(d[2] == '=')
//     {
//         if((ip0 = decode(d[0])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip1 = decode(d[1])) > 63)
//             return SEC_E_INVALID_TOKEN;
//
//         out_buf[i] = (ip0 << 2) | (ip1 >> 4);
//         i++;
//     }
//     else if(d[3] == '=')
//     {
//         if((ip0 = decode(d[0])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip1 = decode(d[1])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip2 = decode(d[2])) > 63)
//             return SEC_E_INVALID_TOKEN;
//
//         out_buf[i + 0] = (ip0 << 2) | (ip1 >> 4);
//         out_buf[i + 1] = (ip1 << 4) | (ip2 >> 2);
//         i += 2;
//     }
//     else
//     {
//         if((ip0 = decode(d[0])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip1 = decode(d[1])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip2 = decode(d[2])) > 63)
//             return SEC_E_INVALID_TOKEN;
//         if((ip3 = decode(d[3])) > 63)
//             return SEC_E_INVALID_TOKEN;
//
//
//         out_buf[i + 0] = (ip0 << 2) | (ip1 >> 4);
//         out_buf[i + 1] = (ip1 << 4) | (ip2 >> 2);
//         out_buf[i + 2] = (ip2 << 6) |  ip3;
//         i += 3;
//     }
//     *out_len = i;
//     return SEC_E_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
