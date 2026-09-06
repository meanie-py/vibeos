use crate::prelude::*;
use varargs_rust::VaArgs;
use core::ffi::c_void;
// ber.c is the add_pch SKIP source: it compiles against <winldap.h>, not
// "winldap_private.h", and the two declare BerElement/berval INCOMPATIBLY (§9.108).
// So the ber types come from THIS TU's own island and must SHADOW the prelude's glob --
// a named import wins over a glob, which is the only reason both can coexist in scope.
// lib.rs already documents this contract; the translation just never honoured it, and
// `BerElement` resolved to nothing at all.
use crate::types_1::{berval, BerElement, BERVAL, PBERVAL};

// ---- file-scope preamble (translated) ----
pub const LBER_ERROR: u32 = !0u32;
// Stable Rust cannot DEFINE a C-variadic fn nor do va_arg (`c_variadic`,
// rust-lang/rust#44930). The exported `WLDAP32_ber_printf`/`_scanf` are therefore
// 3-line va_start trampolines in dll/win32/wldap32/rust_varargs_shim.c that hand the
// va_list straight back here as `*mut c_void`; the entire format walk stays in Rust.
// Nightly is NOT an option -- `#![feature]`/RUSTC_BOOTSTRAP are hard-gated.
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_printf_v(
    berelement: *mut BerElement,
    fmt: PCHAR,
    ap: *mut c_void,
) -> INT {

#[cfg(feature = "have_ldap")]
    {
        let mut list = VaArgs::from_ptr(ap);
        let mut ret: c_int = 0;
        let mut new_fmt: [c_char; 2] = [0; 2];
        let mut fmt_ptr = fmt;

        new_fmt[1] = 0;

        while *fmt_ptr != 0 {
            new_fmt[0] = *fmt_ptr;
            fmt_ptr = fmt_ptr.add(1);

            match new_fmt[0] as u8 {
                b'b' | b'e' | b'i' => {
                    let i: c_int = list.arg::<c_int>();
                    ret = ber_printf(berelement, new_fmt.as_ptr(), i);
                }
                b'o' | b's' => {
                    let str: *mut c_char = list.arg::<*mut c_char>();
                    ret = ber_printf(berelement, new_fmt.as_ptr(), str);
                }
                b't' => {
                    let tag: c_uint = list.arg::<c_uint>();
                    ret = ber_printf(berelement, new_fmt.as_ptr(), tag);
                }
                b'v' => {
                    let array: *mut *mut c_char = list.arg::<*mut *mut c_char>();
                    ret = ber_printf(berelement, new_fmt.as_ptr(), array);
                }
                b'V' => {
                    let array: *mut *mut berval = list.arg::<*mut *mut berval>();
                    ret = ber_printf(berelement, new_fmt.as_ptr(), array);
                }
                b'X' => {
                    let str: *mut c_char = list.arg::<*mut c_char>();
                    let len: c_int = list.arg::<c_int>();
                    new_fmt[0] = b'B' as c_char;
                    ret = ber_printf(berelement, new_fmt.as_ptr(), str, len);
                }
                b'n' | b'{' | b'}' | b'[' | b']' => {
                    ret = ber_printf(berelement, new_fmt.as_ptr());
                }
                _ => {
                    FIXME!("Unknown format '%c'\n", new_fmt[0]);
                    ret = -1;
                }
            }

            if ret == -1 {
                break;
            }
        }

        ret
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        (LBER_ERROR) as i32
    }
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_bvecfree(berval: *mut PBERVAL) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_bvecfree(berval: *mut PBERVAL) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_bvecfree(berval: *mut PBERVAL) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_bvecfree(berval: *mut PBERVAL) {
    #[cfg(feature = "have_ldap")]
    {
        ber_bvecfree(berval);
    }
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_bvfree(berval: *mut BERVAL) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_bvfree(berval: *mut BERVAL) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_bvfree(berval: *mut BERVAL) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_bvfree(berval: *mut BERVAL) {
    #[cfg(feature = "have_ldap")]
    {
        ber_bvfree(berval);
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_first_element(
    berelement: *mut BerElement,
    len: *mut ULONG,
    opaque: *mut *mut CHAR,
) -> ULONG {
    #[cfg(feature = "have_ldap")]
    {
        return ber_first_element(berelement, len, opaque);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return LBER_ERROR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_flatten(berelement: *mut BerElement, berval: *mut PBERVAL) -> INT {
    #[cfg(feature = "have_ldap")]
    {
        return ber_flatten(berelement, berval);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return (LBER_ERROR) as i32;
    }
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_free(berelement: *mut BerElement, buf: INT) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_free(berelement: *mut BerElement, buf: INT) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ber_free(berelement: *mut BerElement, buf: INT) {}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_free(berelement: *mut BerElement, buf: INT) {
    #[cfg(feature = "have_ldap")]
    {
        ber_free(berelement, buf);
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_next_element(
    berelement: *mut BerElement,
    len: *mut ULONG,
    opaque: *mut CHAR,
) -> ULONG {
    #[cfg(feature = "have_ldap")]
    {
        return ber_next_element(berelement, len, opaque);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return LBER_ERROR;
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_peek_tag(berelement: *mut BerElement, len: *mut ULONG) -> ULONG {
    #[cfg(feature = "have_ldap")]
    {
        return ber_peek_tag(berelement, len);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return LBER_ERROR;
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_skip_tag(berelement: *mut BerElement, len: *mut ULONG) -> ULONG {
    #[cfg(feature = "have_ldap")]
    {
        return ber_skip_tag(berelement, len);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return LBER_ERROR;
    }
}

// Stable Rust cannot DEFINE a C-variadic fn nor do va_arg (`c_variadic`,
// rust-lang/rust#44930). The exported `WLDAP32_ber_printf`/`_scanf` are therefore
// 3-line va_start trampolines in dll/win32/wldap32/rust_varargs_shim.c that hand the
// va_list straight back here as `*mut c_void`; the entire format walk stays in Rust.
// Nightly is NOT an option -- `#![feature]`/RUSTC_BOOTSTRAP are hard-gated.
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_scanf_v(berelement: *mut BerElement, fmt: PCHAR, ap: *mut c_void) -> INT {
    #[cfg(feature = "have_ldap")]
    {
        let mut list = VaArgs::from_ptr(ap);
        let mut ret: c_int = 0;
        let mut new_fmt: [c_char; 2] = [0; 2];

        new_fmt[1] = 0;
        let mut fmt_ptr = fmt;
        while *fmt_ptr != 0 {
            new_fmt[0] = *fmt_ptr;
            fmt_ptr = fmt_ptr.offset(1);
            match new_fmt[0] as u8 {
                b'a' => {
                    let ptr: *mut *mut c_char = list.arg::<*mut *mut c_char>();
                    ret = ber_scanf(berelement, new_fmt.as_ptr(), ptr);
                }
                b'b' | b'e' | b'i' => {
                    let i: *mut c_int = list.arg::<*mut c_int>();
                    ret = ber_scanf(berelement, new_fmt.as_ptr(), i);
                }
                b't' => {
                    let tag: *mut c_uint = list.arg::<*mut c_uint>();
                    ret = ber_scanf(berelement, new_fmt.as_ptr(), tag);
                }
                b'v' => {
                    let array: *mut *mut *mut c_char = list.arg::<*mut *mut *mut c_char>();
                    ret = ber_scanf(berelement, new_fmt.as_ptr(), array);
                }
                b'B' => {
                    let str_: *mut *mut c_char = list.arg::<*mut *mut c_char>();
                    let len: *mut c_int = list.arg::<*mut c_int>();
                    ret = ber_scanf(berelement, new_fmt.as_ptr(), str_, len);
                }
                b'O' => {
                    let ptr: *mut *mut berval = list.arg::<*mut *mut berval>();
                    ret = ber_scanf(berelement, new_fmt.as_ptr(), ptr);
                }
                b'V' => {
                    let array: *mut *mut *mut berval = list.arg::<*mut *mut *mut berval>();
                    ret = ber_scanf(berelement, new_fmt.as_ptr(), array);
                }
                b'n' | b'x' | b'{' | b'}' | b'[' | b']' => {
                    ret = ber_scanf(berelement, new_fmt.as_ptr());
                }
                _ => {
                    FIXME!("Unknown format '%c'\n", new_fmt[0]);
                    ret = -1;
                }
            }
            if ret == -1 {
                break;
            }
        }
        ret
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        LBER_ERROR as INT
    }
}


#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_alloc_t(options: INT) -> *mut BerElement {
    #[cfg(feature = "have_ldap")]
    {
        return ber_alloc_t(options);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_bvdup(berval: *mut BERVAL) -> *mut BERVAL {
    #[cfg(feature = "have_ldap")]
    {
        return ber_bvdup(berval);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ber_init(berval: *mut BERVAL) -> *mut BerElement {
    #[cfg(feature = "have_ldap")]
    {
        return ber_init(berval);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return core::ptr::null_mut();
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * WLDAP32 - LDAP support for Wine
//  *
//  * Copyright 2005 Hans Leidekker
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
// #include "config.h"
//
// #include <stdarg.h>
// #include "windef.h"
// #include "winbase.h"
// #include "winldap.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(wldap32);
//
// #ifndef LBER_ERROR
// # define LBER_ERROR (~0U)
// #endif
//
// /***********************************************************************
//  *      ber_alloc_t     (WLDAP32.@)
//  *
//  * Allocate a berelement structure.
//  *
//  * PARAMS
//  *  options [I] Must be LBER_USE_DER.
//  *
//  * RETURNS
//  *  Success: Pointer to an allocated berelement structure.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  Free the berelement structure with ber_free.
//  */
// BerElement * CDECL WLDAP32_ber_alloc_t( INT options )
// {
// #ifdef HAVE_LDAP
//     return ber_alloc_t( options );
// #else
//     return NULL;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_bvdup     (WLDAP32.@)
//  *
//  * Copy a berval structure.
//  *
//  * PARAMS
//  *  berval [I] Pointer to the berval structure to be copied.
//  *
//  * RETURNS
//  *  Success: Pointer to a copy of the berval structure.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  Free the copy with ber_bvfree.
//  */
// BERVAL * CDECL WLDAP32_ber_bvdup( BERVAL *berval )
// {
// #ifdef HAVE_LDAP
//     return ber_bvdup( berval );
// #else
//     return NULL;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_bvecfree     (WLDAP32.@)
//  *
//  * Free an array of berval structures.
//  *
//  * PARAMS
//  *  berval [I] Pointer to an array of berval structures.
//  *
//  * RETURNS
//  *  Nothing.
//  *
//  * NOTES
//  *  Use this function only to free an array of berval structures
//  *  returned by a call to ber_scanf with a 'V' in the format string.
//  */
// void CDECL WLDAP32_ber_bvecfree( PBERVAL *berval )
// {
// #ifdef HAVE_LDAP
//     ber_bvecfree( berval );
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_bvfree     (WLDAP32.@)
//  *
//  * Free a berval structure.
//  *
//  * PARAMS
//  *  berval [I] Pointer to a berval structure.
//  *
//  * RETURNS
//  *  Nothing.
//  *
//  * NOTES
//  *  Use this function only to free berval structures allocated by
//  *  an LDAP API.
//  */
// void CDECL WLDAP32_ber_bvfree( BERVAL *berval )
// {
// #ifdef HAVE_LDAP
//     ber_bvfree( berval );
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_first_element     (WLDAP32.@)
//  *
//  * Return the tag of the first element in a set or sequence.
//  *
//  * PARAMS
//  *  berelement [I] Pointer to a berelement structure.
//  *  len        [O] Receives the length of the first element.
//  *  opaque     [O] Receives a pointer to a cookie.
//  *
//  * RETURNS
//  *  Success: Tag of the first element.
//  *  Failure: LBER_DEFAULT (no more data).
//  *
//  * NOTES
//  *  len and cookie should be passed to ber_next_element.
//  */
// ULONG CDECL WLDAP32_ber_first_element( BerElement *berelement, ULONG *len, CHAR **opaque )
// {
// #ifdef HAVE_LDAP
//     return ber_first_element( berelement, len, opaque );
// #else
//     return LBER_ERROR;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_flatten     (WLDAP32.@)
//  *
//  * Flatten a berelement structure into a berval structure.
//  *
//  * PARAMS
//  *  berelement [I] Pointer to a berelement structure.
//  *  berval    [O] Pointer to a berval structure.
//  *
//  * RETURNS
//  *  Success: 0
//  *  Failure: LBER_ERROR
//  *
//  * NOTES
//  *  Free the berval structure with ber_bvfree.
//  */
// INT CDECL WLDAP32_ber_flatten( BerElement *berelement, PBERVAL *berval )
// {
// #ifdef HAVE_LDAP
//     return ber_flatten( berelement, berval );
// #else
//     return LBER_ERROR;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_free     (WLDAP32.@)
//  *
//  * Free a berelement structure.
//  *
//  * PARAMS
//  *  berelement [I] Pointer to the berelement structure to be freed.
//  *  buf       [I] Flag.
//  *
//  * RETURNS
//  *  Nothing.
//  *
//  * NOTES
//  *  Set buf to 0 if the berelement was allocated with ldap_first_attribute
//  *  or ldap_next_attribute, otherwise set it to 1.
//  */
// void CDECL WLDAP32_ber_free( BerElement *berelement, INT buf )
// {
// #ifdef HAVE_LDAP
//     ber_free( berelement, buf );
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_init     (WLDAP32.@)
//  *
//  * Initialise a berelement structure from a berval structure.
//  *
//  * PARAMS
//  *  berval [I] Pointer to a berval structure.
//  *
//  * RETURNS
//  *  Success: Pointer to a berelement structure.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  Call ber_free to free the returned berelement structure.
//  */
// BerElement * CDECL WLDAP32_ber_init( BERVAL *berval )
// {
// #ifdef HAVE_LDAP
//     return ber_init( berval );
// #else
//     return NULL;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_next_element     (WLDAP32.@)
//  *
//  * Return the tag of the next element in a set or sequence.
//  *
//  * PARAMS
//  *  berelement [I]   Pointer to a berelement structure.
//  *  len        [I/O] Receives the length of the next element.
//  *  opaque     [I/O] Pointer to a cookie.
//  *
//  * RETURNS
//  *  Success: Tag of the next element.
//  *  Failure: LBER_DEFAULT (no more data).
//  *
//  * NOTES
//  *  len and cookie are initialized by ber_first_element and should
//  *  be passed on in subsequent calls to ber_next_element.
//  */
// ULONG CDECL WLDAP32_ber_next_element( BerElement *berelement, ULONG *len, CHAR *opaque )
// {
// #ifdef HAVE_LDAP
//     return ber_next_element( berelement, len, opaque );
// #else
//     return LBER_ERROR;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_peek_tag     (WLDAP32.@)
//  *
//  * Return the tag of the next element.
//  *
//  * PARAMS
//  *  berelement [I] Pointer to a berelement structure.
//  *  len        [O] Receives the length of the next element.
//  *
//  * RETURNS
//  *  Success: Tag of the next element.
//  *  Failure: LBER_DEFAULT (no more data).
//  */
// ULONG CDECL WLDAP32_ber_peek_tag( BerElement *berelement, ULONG *len )
// {
// #ifdef HAVE_LDAP
//     return ber_peek_tag( berelement, len );
// #else
//     return LBER_ERROR;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_skip_tag     (WLDAP32.@)
//  *
//  * Skip the current tag and return the tag of the next element.
//  *
//  * PARAMS
//  *  berelement [I] Pointer to a berelement structure.
//  *  len        [O] Receives the length of the skipped element.
//  *
//  * RETURNS
//  *  Success: Tag of the next element.
//  *  Failure: LBER_DEFAULT (no more data).
//  */
// ULONG CDECL WLDAP32_ber_skip_tag( BerElement *berelement, ULONG *len )
// {
// #ifdef HAVE_LDAP
//     return ber_skip_tag( berelement, len );
// #else
//     return LBER_ERROR;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_printf     (WLDAP32.@)
//  *
//  * Encode a berelement structure.
//  *
//  * PARAMS
//  *  berelement [I/O] Pointer to a berelement structure.
//  *  fmt        [I]   Format string.
//  *  ...        [I]   Values to encode.
//  *
//  * RETURNS
//  *  Success: Non-negative number.
//  *  Failure: LBER_ERROR
//  *
//  * NOTES
//  *  berelement must have been allocated with ber_alloc_t. This function
//  *  can be called multiple times to append data.
//  */
// INT __cdecl WLDAP32_ber_printf( BerElement *berelement, PCHAR fmt, ... )
// {
// #ifdef HAVE_LDAP
//     __ms_va_list list;
//     int ret = 0;
//     char new_fmt[2];
//
//     new_fmt[1] = 0;
////     while (*fmt)
//     {
//         new_fmt[0] = *fmt++;
//         switch(new_fmt[0])
//         {
//         case 'b':
//         case 'e':
//         case 'i':
//             {
//                 int i = va_arg( list, int );
//                 ret = ber_printf( berelement, new_fmt, i );
//                 break;
//             }
//         case 'o':
//         case 's':
//             {
//                 char *str = va_arg( list, char * );
//                 ret = ber_printf( berelement, new_fmt, str );
//                 break;
//             }
//         case 't':
//             {
//                 unsigned int tag = va_arg( list, unsigned int );
//                 ret = ber_printf( berelement, new_fmt, tag );
//                 break;
//             }
//         case 'v':
//             {
//                 char **array = va_arg( list, char ** );
//                 ret = ber_printf( berelement, new_fmt, array );
//                 break;
//             }
//         case 'V':
//             {
//                 struct berval **array = va_arg( list, struct berval ** );
//                 ret = ber_printf( berelement, new_fmt, array );
//                 break;
//             }
//         case 'X':
//             {
//                 char *str = va_arg( list, char * );
//                 int len = va_arg( list, int );
//                 new_fmt[0] = 'B';  /* 'X' is deprecated */
//                 ret = ber_printf( berelement, new_fmt, str, len );
//                 break;
//             }
//         case 'n':
//         case '{':
//         case '}':
//         case '[':
//         case ']':
//             ret = ber_printf( berelement, new_fmt );
//             break;
//         default:
//             FIXME( "Unknown format '%c'\n", new_fmt[0] );
//             ret = -1;
//             break;
//         }
//         if (ret == -1) break;
//     }
////     return ret;
// #else
//     return LBER_ERROR;
// #endif
// }
//
//
// /***********************************************************************
//  *      ber_scanf     (WLDAP32.@)
//  *
//  * Decode a berelement structure.
//  *
//  * PARAMS
//  *  berelement [I/O] Pointer to a berelement structure.
//  *  fmt        [I]   Format string.
//  *  ...        [I]   Pointers to values to be decoded.
//  *
//  * RETURNS
//  *  Success: Non-negative number.
//  *  Failure: LBER_ERROR
//  *
//  * NOTES
//  *  berelement must have been allocated with ber_init. This function
//  *  can be called multiple times to decode data.
//  */
// INT __cdecl WLDAP32_ber_scanf( BerElement *berelement, PCHAR fmt, ... )
// {
// #ifdef HAVE_LDAP
//     __ms_va_list list;
//     int ret = 0;
//     char new_fmt[2];
//
//     new_fmt[1] = 0;
////     while (*fmt)
//     {
//         new_fmt[0] = *fmt++;
//         switch(new_fmt[0])
//         {
//         case 'a':
//             {
//                 char **ptr = va_arg( list, char ** );
//                 ret = ber_scanf( berelement, new_fmt, ptr );
//                 break;
//             }
//         case 'b':
//         case 'e':
//         case 'i':
//             {
//                 int *i = va_arg( list, int * );
//                 ret = ber_scanf( berelement, new_fmt, i );
//                 break;
//             }
//         case 't':
//             {
//                 unsigned int *tag = va_arg( list, unsigned int * );
//                 ret = ber_scanf( berelement, new_fmt, tag );
//                 break;
//             }
//         case 'v':
//             {
//                 char ***array = va_arg( list, char *** );
//                 ret = ber_scanf( berelement, new_fmt, array );
//                 break;
//             }
//         case 'B':
//             {
//                 char **str = va_arg( list, char ** );
//                 int *len = va_arg( list, int * );
//                 ret = ber_scanf( berelement, new_fmt, str, len );
//                 break;
//             }
//         case 'O':
//             {
//                 struct berval **ptr = va_arg( list, struct berval ** );
//                 ret = ber_scanf( berelement, new_fmt, ptr );
//                 break;
//             }
//         case 'V':
//             {
//                 struct berval ***array = va_arg( list, struct berval *** );
//                 ret = ber_scanf( berelement, new_fmt, array );
//                 break;
//             }
//         case 'n':
//         case 'x':
//         case '{':
//         case '}':
//         case '[':
//         case ']':
//             ret = ber_scanf( berelement, new_fmt );
//             break;
//         default:
//             FIXME( "Unknown format '%c'\n", new_fmt[0] );
//             ret = -1;
//             break;
//         }
//         if (ret == -1) break;
//     }
////     return ret;
// #else
//     return LBER_ERROR;
// #endif
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
