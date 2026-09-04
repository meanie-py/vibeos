use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.
// This C file contains only function definitions, no types/constants/statics/tables.

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ldap_abandon(ld: *mut WLDAP32_LDAP, msgid: ULONG) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_abandon(ld: *mut WLDAP32_LDAP, msgid: ULONG) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if ld.is_null() {
        return !0u32;
    }
    ret = map_error(ldap_abandon_ext(ld, msgid, core::ptr::null_mut(), core::ptr::null_mut()));

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_check_filterA(ld: *mut WLDAP32_LDAP, filter: PCHAR) -> u32 {
    let mut ret: u32;
    let mut filterW: LPWSTR = core::ptr::null_mut();

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    if !filter.is_null() {
        filterW = strAtoW(filter);
        if filterW.is_null() {
            return WLDAP32_LDAP_NO_MEMORY;
        }
    }

    ret = ldap_check_filterW(ld, filterW);

    strfreeW(filterW);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_check_filterW(ld: *mut WLDAP32_LDAP, filter: LPWSTR) -> u32 {
    debugstr_w!(filter);
    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }
    return WLDAP32_LDAP_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn ldap_cleanup(instance: HANDLE) -> ULONG {
    // TRACE!( "(%p)\n", instance );
    WLDAP32_LDAP_SUCCESS
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ldap_count_entries(
    ld: *mut WLDAP32_LDAP,
    res: *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_count_entries(
    ld: *mut WLDAP32_LDAP,
    res: *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if ld.is_null() {
        return !0u32;
    }
    ret = ldap_count_entries(ld, res);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap_count_references"))]
pub unsafe extern "C" fn WLDAP32_ldap_count_references(
    ld: *mut WLDAP32_LDAP,
    res: *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap_count_references")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_count_references(
    ld: *mut WLDAP32_LDAP,
    res: *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if ld.is_null() {
        return 0;
    }
    ret = ldap_count_references(ld, res);

    ret
}
unsafe extern "C" fn get_escape_size(src: PCHAR, srclen: ULONG) -> ULONG {
    let mut i: ULONG;
    let mut size: ULONG = 0;

    if !src.is_null() {
        i = 0;
        while i < srclen {
            let c = *src.offset(i as isize);
            if (c >= (b'0') as i8 && c <= (b'9') as i8) ||
               (c >= (b'A') as i8 && c <= (b'Z') as i8) ||
               (c >= (b'a') as i8 && c <= (b'z') as i8) {
                size += 1;
            } else {
                size += 3;
            }
            i += 1;
        }
    }
    size + 1
}
unsafe extern "C" fn escape_filter_element(src: PCHAR, srclen: ULONG, dst: PCHAR) {
    let mut i: ULONG;
    static fmt: &[u8] = b"\\%02X\0";
    let mut d: *mut i8 = dst;

    i = 0;
    while i < srclen {
        if (*src.offset(i as isize) >= b'0' as i8 && *src.offset(i as isize) <= b'9' as i8) ||
           (*src.offset(i as isize) >= b'A' as i8 && *src.offset(i as isize) <= b'Z' as i8) ||
           (*src.offset(i as isize) >= b'a' as i8 && *src.offset(i as isize) <= b'z' as i8) {
            *d = *src.offset(i as isize);
            d = d.offset(1);
        } else {
            // `as u8 as c_int`, NOT `as c_int`: the C is `(unsigned char)src[i]`, which
            // zero-extends BEFORE the default int promotion. CHAR is i8 on all three Windows
            // targets, so a bare `as c_int` sign-extends and "%02X" prints 0xE9 as FFFFFFE9 --
            // 9 bytes where get_escape_size budgeted 3. rustc's E0617 fix-it suggests exactly
            // that bare cast; it is only value-preserving when the source is already unsigned.
            d = d.offset(sprintf(
                d,
                fmt.as_ptr() as LPCSTR,
                *src.offset(i as isize) as u8 as c_int,
            ) as isize);
        }
        i += 1;
    }
    d = d.offset(1);
    *d = 0;
}

#[no_mangle]
pub unsafe extern "C" fn ldap_escape_filter_elementA(
    src: PCHAR,
    srclen: ULONG,
    dst: PCHAR,
    dstlen: ULONG,
) -> ULONG {
    let len: ULONG;

    len = get_escape_size(src, srclen);
    if dst.is_null() {
        return len;
    }

    if src.is_null() || dstlen < len {
        return WLDAP32_LDAP_PARAM_ERROR;
    } else {
        escape_filter_element(src, srclen, dst);
        return WLDAP32_LDAP_SUCCESS;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_escape_filter_elementW(
    src: PCHAR,
    srclen: ULONG,
    dst: PWCHAR,
    dstlen: ULONG,
) -> ULONG {
    let len: ULONG;

    len = get_escape_size(src, srclen);
    if dst.is_null() {
        return len;
    }

    return WLDAP32_LDAP_PARAM_ERROR;
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_first_attributeA(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
    ptr: *mut *mut WLDAP32_BerElement,
) -> PCHAR {
    core::ptr::null_mut()
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_first_attributeA(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
    ptr: *mut *mut WLDAP32_BerElement,
) -> PCHAR {
    let mut ret: PCHAR = core::ptr::null_mut();

    if ld.is_null() || entry.is_null() {
        return core::ptr::null_mut();
    }

    let retW: PWCHAR = ldap_first_attributeW(ld, entry, ptr);
    ret = strWtoA(retW);
    ldap_memfreeW(retW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_first_attributeW(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
    ptr: *mut *mut WLDAP32_BerElement,
) -> PWCHAR {
    core::ptr::null_mut()
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_first_attributeW(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
    ptr: *mut *mut WLDAP32_BerElement,
) -> PWCHAR {
    let mut ret: PWCHAR = core::ptr::null_mut();

    if ld.is_null() || entry.is_null() {
        return core::ptr::null_mut();
    }

    let retU = ldap_first_attribute(ld, entry, ptr);
    ret = strUtoW(retU);
    ldap_memfree(retU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_memfreeA(block: PCHAR) {
    // TRACE!( "(%p)\n", block );
    strfreeA(block);
}

#[no_mangle]
pub unsafe extern "C" fn ldap_memfreeW(block: PWCHAR) {
    // TRACE!( "(%p)\n", block );
    strfreeW(block);
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ldap_msgfree(res: *mut WLDAP32_LDAPMessage) -> ULONG {
    WLDAP32_LDAP_SUCCESS
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_msgfree(res: *mut WLDAP32_LDAPMessage) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_SUCCESS;
    ldap_msgfree(res);
    return ret;
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_next_attributeA(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
    ptr: *mut WLDAP32_BerElement,
) -> PCHAR {
    core::ptr::null_mut()
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_next_attributeA(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
    ptr: *mut WLDAP32_BerElement,
) -> PCHAR {
    let mut ret: PCHAR = core::ptr::null_mut();
    let retW: LPWSTR;

    if ld.is_null() || entry.is_null() || ptr.is_null() {
        return core::ptr::null_mut();
    }
    retW = ldap_next_attributeW(ld, entry, ptr);

    ret = strWtoA(retW);
    ldap_memfreeW(retW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_next_attributeW(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
    ptr: *mut WLDAP32_BerElement,
) -> LPWSTR {
    core::ptr::null_mut()
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_next_attributeW(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
    ptr: *mut WLDAP32_BerElement,
) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();
    #[cfg(feature = "have_ldap")]
    {
        let retU: LPSTR;

        if ld.is_null() || entry.is_null() || ptr.is_null() {
            return core::ptr::null_mut();
        }
        retU = ldap_next_attribute(ld, entry, ptr);

        ret = strUtoW(retU);
        ldap_memfree(retU);
    }
    ret
}
#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ldap_result(
    ld: *mut WLDAP32_LDAP,
    msgid: ULONG,
    all: ULONG,
    timeout: *mut l_timeval,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_result(
    ld: *mut WLDAP32_LDAP,
    msgid: ULONG,
    all: ULONG,
    timeout: *mut l_timeval,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if ld.is_null() || res.is_null() || msgid == !0u32 {
        return !0u32;
    }
    ret = ldap_result(ld, msgid, all, timeout as *mut timeval, res);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn LdapUnicodeToUTF8(
    src: LPCWSTR,
    srclen: i32,
    dst: LPSTR,
    dstlen: i32,
) -> i32 {
    WideCharToMultiByte(
        CP_UTF8,
        0,
        src,
        srclen,
        dst,
        dstlen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn LdapUTF8ToUnicode(
    src: LPCSTR,
    srclen: i32,
    dst: LPWSTR,
    dstlen: i32,
) -> i32 {
    MultiByteToWideChar(CP_UTF8, 0, src, srclen, dst, dstlen)
}


#[no_mangle]
pub unsafe extern "C" fn ldap_conn_from_msg(
    ld: *mut WLDAP32_LDAP,
    res: *mut WLDAP32_LDAPMessage,
) -> *mut WLDAP32_LDAP {
    if ld.is_null() || res.is_null() {
        return core::ptr::null_mut();
    }
    ld
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_first_entry(
    ld: *mut WLDAP32_LDAP,
    res: *mut WLDAP32_LDAPMessage,
) -> *mut WLDAP32_LDAPMessage {
    #[cfg(feature = "have_ldap")]
    {
        if ld.is_null() || res.is_null() {
            return core::ptr::null_mut();
        }
        return ldap_first_entry(ld, res);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        return core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_first_reference(
    ld: *mut WLDAP32_LDAP,
    res: *mut WLDAP32_LDAPMessage,
) -> *mut WLDAP32_LDAPMessage {
    #[cfg(feature = "have_ldap_first_reference")]
    {
        if ld.is_null() {
            return core::ptr::null_mut();
        }
        return ldap_first_reference(ld, res);
    }
    #[cfg(not(feature = "have_ldap_first_reference"))]
    {
        return core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_next_entry(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
) -> *mut WLDAP32_LDAPMessage {
    // misc.c: `#ifdef HAVE_LDAP ... #else return NULL; #endif`. The translation dropped the
    // guard here while getting it right on the two `_reference` siblings beside it.
    #[cfg(feature = "have_ldap")]
    {
        if ld.is_null() || entry.is_null() {
            return core::ptr::null_mut();
        }
        return ldap_next_entry(ld, entry);
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_next_reference(
    ld: *mut WLDAP32_LDAP,
    entry: *mut WLDAP32_LDAPMessage,
) -> *mut WLDAP32_LDAPMessage {
    #[cfg(feature = "have_ldap_next_reference")]
    {
        if ld.is_null() || entry.is_null() {
            return core::ptr::null_mut();
        }
        return ldap_next_reference(ld, entry);
    }
    #[cfg(not(feature = "have_ldap_next_reference"))]
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
// #include "wine/port.h"
//
// #include <stdarg.h>
// #include <stdio.h>
// #ifdef HAVE_LDAP_H
// #include <ldap.h>
// #endif
//
// #include "windef.h"
// #include "winbase.h"
// #include "winnls.h"
//
// #include "winldap_private.h"
// #include "wldap32.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(wldap32);
//
// /***********************************************************************
//  *      ldap_abandon     (WLDAP32.@)
//  *
//  * Cancel an asynchronous operation.
//  *
//  * PARAMS
//  *  ld    [I] Pointer to an LDAP context.
//  *  msgid [I] ID of the operation to cancel.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL WLDAP32_ldap_abandon( WLDAP32_LDAP *ld, ULONG msgid )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, 0x%08x)\n", ld, msgid );
//
//     if (!ld) return ~0u;
//     ret = map_error( ldap_abandon_ext( ld, msgid, NULL, NULL ));
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_check_filterA     (WLDAP32.@)
//  *
//  * See ldap_check_filterW.
//  */
// ULONG CDECL ldap_check_filterA( WLDAP32_LDAP *ld, PCHAR filter )
// {
//     ULONG ret;
//     WCHAR *filterW = NULL;
//
//     TRACE( "(%p, %s)\n", ld, debugstr_a!(filter) );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (filter) {
//         filterW = strAtoW( filter );
//         if (!filterW) return WLDAP32_LDAP_NO_MEMORY;
//     }
//
//     ret = ldap_check_filterW( ld, filterW );
//
//     strfreeW( filterW );
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_check_filterW     (WLDAP32.@)
//  *
//  * Check filter syntax.
//  *
//  * PARAMS
//  *  ld     [I] Pointer to an LDAP context.
//  *  filter [I] Filter string.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_check_filterW( WLDAP32_LDAP *ld, PWCHAR filter )
// {
//     TRACE( "(%p, %s)\n", ld, debugstr_w!(filter) );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//     return WLDAP32_LDAP_SUCCESS; /* FIXME: do some checks */
// }
//
// /***********************************************************************
//  *      ldap_cleanup     (WLDAP32.@)
//  */
// ULONG CDECL ldap_cleanup( HANDLE instance )
// {
//     TRACE( "(%p)\n", instance );
//     return WLDAP32_LDAP_SUCCESS;
// }
//
// /***********************************************************************
//  *      ldap_conn_from_msg     (WLDAP32.@)
//  *
//  * Get the LDAP context for a given message.
//  *
//  * PARAMS
//  *  ld  [I] Pointer to an LDAP context.
//  *  res [I] LDAP message.
//  *
//  * RETURNS
//  *  Success: Pointer to an LDAP context.
//  *  Failure: NULL
//  */
// WLDAP32_LDAP * CDECL ldap_conn_from_msg( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *res )
// {
//     TRACE( "(%p, %p)\n", ld, res );
//
//     if (!ld || !res) return NULL;
//     return ld; /* FIXME: not always correct */
// }
//
// /***********************************************************************
//  *      ldap_count_entries     (WLDAP32.@)
//  *
//  * Count the number of entries returned from a search.
//  *
//  * PARAMS
//  *  ld  [I] Pointer to an LDAP context.
//  *  res [I] LDAP message.
//  *
//  * RETURNS
//  *  Success: The number of entries.
//  *  Failure: ~0u
//  */
// ULONG CDECL WLDAP32_ldap_count_entries( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, %p)\n", ld, res );
//
//     if (!ld) return ~0u;
//     ret = ldap_count_entries( ld, res );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_count_references     (WLDAP32.@)
//  *
//  * Count the number of references returned from a search.
//  *
//  * PARAMS
//  *  ld  [I] Pointer to an LDAP context.
//  *  res [I] LDAP message.
//  *
//  * RETURNS
//  *  Success: The number of references.
//  *  Failure: ~0u
//  */
// ULONG CDECL WLDAP32_ldap_count_references( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP_COUNT_REFERENCES
//
//     TRACE( "(%p, %p)\n", ld, res );
//
//     if (!ld) return 0;
//     ret = ldap_count_references( ld, res );
//
// #endif
//     return ret;
// }
//
// static ULONG get_escape_size( PCHAR src, ULONG srclen )
// {
//     ULONG i, size = 0;
//
//     if (src)
//     {
//         for (i = 0; i < srclen; i++)
//         {
//             if ((src[i] >= '0' && src[i] <= '9') ||
//                 (src[i] >= 'A' && src[i] <= 'Z') ||
//                 (src[i] >= 'a' && src[i] <= 'z'))
//                 size++;
//             else
//                 size += 3;
//         }
//     }
//     return size + 1;
// }
//
// static void escape_filter_element( PCHAR src, ULONG srclen, PCHAR dst )
// {
//     ULONG i;
//     static const char fmt[] = "\\%02X";
//     char *d = dst;
//
//     for (i = 0; i < srclen; i++)
//     {
//         if ((src[i] >= '0' && src[i] <= '9') ||
//             (src[i] >= 'A' && src[i] <= 'Z') ||
//             (src[i] >= 'a' && src[i] <= 'z'))
//             *d++ = src[i];
//         else
//             d += sprintf( d, fmt, (unsigned char)src[i] );
//     }
//     *++d = 0;
// }
//
// /***********************************************************************
//  *      ldap_escape_filter_elementA     (WLDAP32.@)
//  *
//  * See ldap_escape_filter_elementW.
//  */
// ULONG CDECL ldap_escape_filter_elementA( PCHAR src, ULONG srclen, PCHAR dst, ULONG dstlen )
// {
//     ULONG len;
//
//     TRACE( "(%p, 0x%08x, %p, 0x%08x)\n", src, srclen, dst, dstlen );
//
//     len = get_escape_size( src, srclen );
//     if (!dst) return len;
//
//     if (!src || dstlen < len)
//         return WLDAP32_LDAP_PARAM_ERROR;
//     else
//     {
//         escape_filter_element( src, srclen, dst );
//         return WLDAP32_LDAP_SUCCESS;
//     }
// }
//
// /***********************************************************************
//  *      ldap_escape_filter_elementW     (WLDAP32.@)
//  *
//  * Escape binary data for safe passing in filters.
//  *
//  * PARAMS
//  *  src    [I] Filter element to be escaped.
//  *  srclen [I] Length in bytes of the filter element.
//  *  dst    [O] Destination buffer for the escaped filter element.
//  *  dstlen [I] Length in bytes of the destination buffer.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_escape_filter_elementW( PCHAR src, ULONG srclen, PWCHAR dst, ULONG dstlen )
// {
//     ULONG len;
//
//     TRACE( "(%p, 0x%08x, %p, 0x%08x)\n", src, srclen, dst, dstlen );
//
//     len = get_escape_size( src, srclen );
//     if (!dst) return len;
//
//     /* no matter what you throw at it, this is what native returns */
//     return WLDAP32_LDAP_PARAM_ERROR;
// }
//
// /***********************************************************************
//  *      ldap_first_attributeA     (WLDAP32.@)
//  *
//  * See ldap_first_attributeW.
//  */
// PCHAR CDECL ldap_first_attributeA( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *entry,
//     WLDAP32_BerElement** ptr )
// {
//     PCHAR ret = NULL;
// #ifdef HAVE_LDAP
//     WCHAR *retW;
//
//     TRACE( "(%p, %p, %p)\n", ld, entry, ptr );
//
//     if (!ld || !entry) return NULL;
//     retW = ldap_first_attributeW( ld, entry, ptr );
//
//     ret = strWtoA( retW );
//     ldap_memfreeW( retW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_first_attributeW     (WLDAP32.@)
//  *
//  * Get the first attribute for a given entry.
//  *
//  * PARAMS
//  *  ld    [I] Pointer to an LDAP context.
//  *  entry [I] Entry to retrieve attribute for.
//  *  ptr   [O] Position pointer.
//  *
//  * RETURNS
//  *  Success: Name of the first attribute.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  Use ldap_memfree to free the returned string.
//  */
// PWCHAR CDECL ldap_first_attributeW( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *entry,
//     WLDAP32_BerElement** ptr )
// {
//     PWCHAR ret = NULL;
// #ifdef HAVE_LDAP
//     char *retU;
//
//     TRACE( "(%p, %p, %p)\n", ld, entry, ptr );
//
//     if (!ld || !entry) return NULL;
//     retU = ldap_first_attribute( ld, entry, ptr );
//
//     ret = strUtoW( retU );
//     ldap_memfree( retU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_first_entry     (WLDAP32.@)
//  *
//  * Get the first entry from a result message.
//  *
//  * PARAMS
//  *  ld  [I] Pointer to an LDAP context.
//  *  res [I] Search result message.
//  *
//  * RETURNS
//  *  Success: The first entry.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  The returned entry will be freed when the message is freed.
//  */
// WLDAP32_LDAPMessage * CDECL WLDAP32_ldap_first_entry( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *res )
// {
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, %p)\n", ld, res );
//
//     if (!ld || !res) return NULL;
//     return ldap_first_entry( ld, res );
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_first_reference     (WLDAP32.@)
//  *
//  * Get the first reference from a result message.
//  *
//  * PARAMS
//  *  ld  [I] Pointer to an LDAP context.
//  *  res [I] Search result message.
//  *
//  * RETURNS
//  *  Success: The first reference.
//  *  Failure: NULL
//  */
// WLDAP32_LDAPMessage * CDECL WLDAP32_ldap_first_reference( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *res )
// {
// #ifdef HAVE_LDAP_FIRST_REFERENCE
//
//     TRACE( "(%p, %p)\n", ld, res );
//
//     if (!ld) return NULL;
//     return ldap_first_reference( ld, res );
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_memfreeA     (WLDAP32.@)
//  *
//  * See ldap_memfreeW.
//  */
// void CDECL ldap_memfreeA( PCHAR block )
// {
//     TRACE( "(%p)\n", block );
//     strfreeA( block );
// }
//
// /***********************************************************************
//  *      ldap_memfreeW     (WLDAP32.@)
//  *
//  * Free a block of memory.
//  *
//  * PARAMS
//  *  block [I] Pointer to memory block to be freed.
//  */
// void CDECL ldap_memfreeW( PWCHAR block )
// {
//     TRACE( "(%p)\n", block );
//     strfreeW( block );
// }
//
// /***********************************************************************
//  *      ldap_msgfree     (WLDAP32.@)
//  *
//  * Free a message.
//  *
//  * PARAMS
//  *  res [I] Message to be freed.
//  */
// ULONG CDECL WLDAP32_ldap_msgfree( WLDAP32_LDAPMessage *res )
// {
//     ULONG ret = WLDAP32_LDAP_SUCCESS;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p)\n", res );
//     ldap_msgfree( res );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_next_attributeA     (WLDAP32.@)
//  *
//  * See ldap_next_attributeW.
//  */
// PCHAR CDECL ldap_next_attributeA( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *entry,
//     WLDAP32_BerElement *ptr )
// {
//     PCHAR ret = NULL;
// #ifdef HAVE_LDAP
//     WCHAR *retW;
//
//     TRACE( "(%p, %p, %p)\n", ld, entry, ptr );
//
//     if (!ld || !entry || !ptr) return NULL;
//     retW = ldap_next_attributeW( ld, entry, ptr );
//
//     ret = strWtoA( retW );
//     ldap_memfreeW( retW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_next_attributeW     (WLDAP32.@)
//  *
//  * Get the next attribute for a given entry.
//  *
//  * PARAMS
//  *  ld    [I]   Pointer to an LDAP context.
//  *  entry [I]   Entry to retrieve attribute for.
//  *  ptr   [I/O] Position pointer.
//  *
//  * RETURNS
//  *  Success: The name of the next attribute.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  Free the returned string after each iteration with ldap_memfree.
//  *  When done iterating and when ptr != NULL, call ber_free( ptr, 0 ).
//  */
// PWCHAR CDECL ldap_next_attributeW( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *entry,
//     WLDAP32_BerElement *ptr )
// {
//     PWCHAR ret = NULL;
// #ifdef HAVE_LDAP
//     char *retU;
//
//     TRACE( "(%p, %p, %p)\n", ld, entry, ptr );
//
//     if (!ld || !entry || !ptr) return NULL;
//     retU = ldap_next_attribute( ld, entry, ptr );
//
//     ret = strUtoW( retU );
//     ldap_memfree( retU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_next_entry     (WLDAP32.@)
//  *
//  * Get the next entry from a result message.
//  *
//  * PARAMS
//  *  ld    [I] Pointer to an LDAP context.
//  *  entry [I] Entry returned by a previous call.
//  *
//  * RETURNS
//  *  Success: The next entry.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  The returned entry will be freed when the message is freed.
//  */
// WLDAP32_LDAPMessage * CDECL WLDAP32_ldap_next_entry( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *entry )
// {
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, %p)\n", ld, entry );
//
//     if (!ld || !entry) return NULL;
//     return ldap_next_entry( ld, entry );
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_next_reference     (WLDAP32.@)
//  *
//  * Get the next reference from a result message.
//  *
//  * PARAMS
//  *  ld    [I] Pointer to an LDAP context.
//  *  entry [I] Entry returned by a previous call.
//  *
//  * RETURNS
//  *  Success: The next reference.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  The returned entry will be freed when the message is freed.
//  */
// WLDAP32_LDAPMessage * CDECL WLDAP32_ldap_next_reference( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *entry )
// {
// #ifdef HAVE_LDAP_NEXT_REFERENCE
//
//     TRACE( "(%p, %p)\n", ld, entry );
//
//     if (!ld || !entry) return NULL;
//     return ldap_next_reference( ld, entry );
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_result     (WLDAP32.@)
//  *
//  * Get the result of an asynchronous operation.
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  msgid   [I] Message ID of the operation.
//  *  all     [I] How many results should be returned?
//  *  timeout [I] How long to wait for the results?
//  *  res     [O] Result message for the operation.
//  *
//  * RETURNS
//  *  Success: One of the following values:
//  *
//  *   LDAP_RES_ADD
//  *   LDAP_RES_BIND
//  *   LDAP_RES_COMPARE
//  *   LDAP_RES_DELETE
//  *   LDAP_RES_EXTENDED
//  *   LDAP_RES_MODIFY
//  *   LDAP_RES_MODRDN
//  *   LDAP_RES_REFERRAL
//  *   LDAP_RES_SEARCH_ENTRY
//  *   LDAP_RES_SEARCH_RESULT
//  *
//  *  Failure: ~0u
//  *
//  *  This function returns 0 when the timeout has expired.
//  *
//  * NOTES
//  *  A NULL timeout pointer causes the function to block waiting
//  *  for results to arrive. A timeout value of 0 causes the function
//  *  to immediately return any available results. Free returned results
//  *  with ldap_msgfree.
//  */
// ULONG CDECL WLDAP32_ldap_result( WLDAP32_LDAP *ld, ULONG msgid, ULONG all,
//     struct l_timeval *timeout, WLDAP32_LDAPMessage **res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, 0x%08x, 0x%08x, %p, %p)\n", ld, msgid, all, timeout, res );
//
//     if (!ld || !res || msgid == ~0u) return ~0u;
//     ret = ldap_result( ld, msgid, all, (struct timeval *)timeout, res );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      LdapUnicodeToUTF8     (WLDAP32.@)
//  *
//  * Convert a wide character string to a UTF8 string.
//  *
//  * PARAMS
//  *  src    [I] Wide character string to convert.
//  *  srclen [I] Size of string to convert, in characters.
//  *  dst    [O] Pointer to a buffer that receives the converted string.
//  *  dstlen [I] Size of the destination buffer in characters.
//  *
//  * RETURNS
//  *  The number of characters written into the destination buffer.
//  *
//  * NOTES
//  *  Set dstlen to zero to ask for the required buffer size.
//  */
// int CDECL LdapUnicodeToUTF8( LPCWSTR src, int srclen, LPSTR dst, int dstlen )
// {
//     return WideCharToMultiByte( CP_UTF8, 0, src, srclen, dst, dstlen, NULL, NULL );
// }
//
// /***********************************************************************
//  *      LdapUTF8ToUnicode     (WLDAP32.@)
//  *
//  * Convert a UTF8 string to a wide character string.
//  *
//  * PARAMS
//  *  src    [I] UTF8 string to convert.
//  *  srclen [I] Size of string to convert, in characters.
//  *  dst    [O] Pointer to a buffer that receives the converted string.
//  *  dstlen [I] Size of the destination buffer in characters.
//  *
//  * RETURNS
//  *  The number of characters written into the destination buffer.
//  *
//  * NOTES
//  *  Set dstlen to zero to ask for the required buffer size.
//  */
// int CDECL LdapUTF8ToUnicode( LPCSTR src, int srclen, LPWSTR dst, int dstlen )
// {
//     return MultiByteToWideChar( CP_UTF8, 0, src, srclen, dst, dstlen );
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
