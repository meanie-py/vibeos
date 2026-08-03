use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static defaulthost: [u16; 10] = [
    b'l' as u16,
    b'o' as u16,
    b'c' as u16,
    b'a' as u16,
    b'l' as u16,
    b'h' as u16,
    b'o' as u16,
    b's' as u16,
    b't' as u16,
    0,
];
#[cfg(feature = "have_ldap")]
unsafe extern "C" fn has_ldap_scheme(url: *mut i8) -> u8 {
    if _strnicmp(url, b"ldap://\0".as_ptr() as *const i8, 7) == 0 {
        return 1u8;
    }
    if _strnicmp(url, b"ldaps://\0".as_ptr() as *const i8, 8) == 0 {
        return 1u8;
    }
    if _strnicmp(url, b"ldapi://\0".as_ptr() as *const i8, 8) == 0 {
        return 1u8;
    }
    if _strnicmp(url, b"cldap://\0".as_ptr() as *const i8, 8) == 0 {
        return 1u8;
    }
    0u8
}
#[cfg(feature = "have_ldap")]
unsafe extern "C" fn join_hostnames(
    scheme: *const i8,
    hostnames: *mut *mut i8,
    portnumber: u32,
) -> *mut i8 {
    let mut res: *mut i8;
    let mut p: *mut i8;
    let mut q: *mut i8;
    let mut v: *mut *mut i8;
    let mut i: u32 = 0;
    let mut size: u32 = 0;
    let sep: &[i8] = &[b' ' as i8, 0];
    let fmt: &[i8] = &[b':' as i8, b'%' as i8, b'd' as i8, 0];
    let mut port: [i8; 7] = [0; 7];

    sprintf(port.as_mut_ptr(), fmt.as_ptr(), portnumber);

    v = hostnames;
    while !(*v).is_null() {
        if has_ldap_scheme(*v) == 0 {
            size += strlen(scheme) as u32;
            q = *v;
        } else {
            q = strchr(*v, b'/' as i32);
        }

        size += strlen(*v) as u32;

        if strchr(q, b':' as i32).is_null() {
            size += strlen(port.as_ptr()) as u32;
        }

        i += 1;
        v = v.offset(1);
    }

    size += (i - 1) * strlen(sep.as_ptr()) as u32;
    res = heap_alloc((size + 1) as usize) as *mut i8;
    if res.is_null() {
        return core::ptr::null_mut();
    }

    p = res;
    v = hostnames;
    while !(*v).is_null() {
        if v != hostnames {
            strcpy(p, sep.as_ptr());
            p = p.offset(strlen(sep.as_ptr()) as isize);
        }

        if has_ldap_scheme(*v) == 0 {
            strcpy(p, scheme);
            p = p.offset(strlen(scheme) as isize);
            q = *v;
        } else {
            q = strchr(*v, b'/' as i32);
        }

        strcpy(p, *v);
        p = p.offset(strlen(*v) as isize);

        if strchr(q, b':' as i32).is_null() {
            strcpy(p, port.as_ptr());
            p = p.offset(strlen(port.as_ptr()) as isize);
        }

        v = v.offset(1);
    }
    res
}
#[cfg(feature = "have_ldap")]
unsafe extern "C" fn urlify_hostnames(
    scheme: *const i8,
    hostnames: *mut i8,
    port: u32,
) -> *mut i8 {
    let mut url: *mut i8 = core::ptr::null_mut();
    let mut strarray: *mut *mut i8;

    strarray = split_hostnames(hostnames);
    if !strarray.is_null() {
        url = join_hostnames(scheme, strarray, port);
    } else {
        return core::ptr::null_mut();
    }

    strarrayfreeU(strarray);
    return url;
}
#[cfg(feature = "have_ldap")]
unsafe extern "C" fn create_context(url: *const i8) -> *mut LDAP {
    let mut ld: *mut LDAP = core::ptr::null_mut();
    let mut version: i32 = LDAP_VERSION3;
    if ldap_initialize(core::ptr::addr_of_mut!(ld), url) != LDAP_SUCCESS {
        return core::ptr::null_mut();
    }
    ldap_set_option(ld, LDAP_OPT_PROTOCOL_VERSION, core::ptr::addr_of_mut!(version) as *const core::ffi::c_void);
    return ld;
}

#[no_mangle]
pub unsafe extern "C" fn ldap_connect(
    ld: *mut WLDAP32_LDAP,
    timeout: *mut l_timeval,
) -> ULONG {
    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }
    WLDAP32_LDAP_SUCCESS
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_start_tls_sA(
    ld: *mut WLDAP32_LDAP,
    retval: PULONG,
    result: *mut *mut WLDAP32_LDAPMessage,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_start_tls_sA(
    ld: *mut WLDAP32_LDAP,
    retval: PULONG,
    result: *mut *mut WLDAP32_LDAPMessage,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut serverctrlsW: *mut *mut LDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut *mut LDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return !0u32;
    }

    'exit: loop {
        if !serverctrls.is_null() {
            serverctrlsW = controlarrayAtoW(serverctrls);
            if serverctrlsW.is_null() {
                break 'exit;
            }
        }
        if !clientctrls.is_null() {
            clientctrlsW = controlarrayAtoW(clientctrls);
            if clientctrlsW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_start_tls_sW(ld, retval, result, serverctrlsW, clientctrlsW);
        break 'exit;
    }

    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_start_tls_sW(
    ld: *mut WLDAP32_LDAP,
    retval: PULONG,
    result: *mut *mut WLDAP32_LDAPMessage,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_start_tls_sW(
    ld: *mut WLDAP32_LDAP,
    retval: PULONG,
    result: *mut *mut WLDAP32_LDAPMessage,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return !0u32;
    }

    if !serverctrls.is_null() {
        serverctrlsU = controlarrayWtoU(serverctrls);
        if serverctrlsU.is_null() {
            controlarrayfreeU(serverctrlsU);
            controlarrayfreeU(clientctrlsU);
            return ret;
        }
    }
    if !clientctrls.is_null() {
        clientctrlsU = controlarrayWtoU(clientctrls);
        if clientctrlsU.is_null() {
            controlarrayfreeU(serverctrlsU);
            controlarrayfreeU(clientctrlsU);
            return ret;
        }
    }

    ret = map_error(ldap_start_tls_s(ld, serverctrlsU, clientctrlsU));

    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_startup(
    version: PLDAP_VERSION_INFO,
    instance: *mut HANDLE,
) -> ULONG {
    // TRACE!( "(%p, %p)\n", version, instance );
    WLDAP32_LDAP_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ldap_stop_tls_s(ld: *mut WLDAP32_LDAP) -> BOOLEAN {
    // TRACE!( "(%p)\n", ld );
    (TRUE) as u8
}


#[cfg(feature = "have_ldap")]
unsafe fn split_hostnames(hostnames: *const i8) -> *mut *mut i8 {
    let mut res: *mut *mut i8;
    let mut str: *mut i8;
    let mut p: *mut i8;
    let mut q: *mut i8;
    let mut i: u32 = 0;

    str = strdupU(hostnames);
    if str.is_null() {
        return core::ptr::null_mut();
    }

    p = str;
    while isspace(*p as u8 as c_int) != 0 {
        p = p.offset(1);
    }
    if *p != 0 {
        i += 1;
    }

    while *p != 0 {
        if isspace(*p as u8 as c_int) != 0 {
            while isspace(*p as u8 as c_int) != 0 {
                p = p.offset(1);
            }
            if *p != 0 {
                i += 1;
            }
        }
        p = p.offset(1);
    }

    res = heap_alloc(((i + 1) as usize) * core::mem::size_of::<*mut i8>()) as *mut *mut i8;
    if res.is_null() {
        heap_free(str as *mut c_void);
        return core::ptr::null_mut();
    }

    p = str;
    while isspace(*p as u8 as c_int) != 0 {
        p = p.offset(1);
    }

    q = p;
    i = 0;

    'outer: loop {
        if *p == 0 {
            break;
        }

        if *p.offset(1) != b'\0' as i8 {
            if isspace(*p as u8 as c_int) != 0 {
                *p = b'\0' as i8;
                p = p.offset(1);
                *res.offset(i as isize) = strdupU(q);
                if (*res.offset(i as isize)).is_null() {
                    break 'outer;
                }
                i += 1;

                while isspace(*p as u8 as c_int) != 0 {
                    p = p.offset(1);
                }
                q = p;
            }
        } else {
            *res.offset(i as isize) = strdupU(q);
            if (*res.offset(i as isize)).is_null() {
                break 'outer;
            }
            i += 1;
        }
        p = p.offset(1);
    }

    if *p == 0 {
        *res.offset(i as isize) = core::ptr::null_mut();
        heap_free(str as *mut c_void);
        return res;
    }

    while i > 0 {
        i -= 1;
        strfreeU(*res.offset(i as isize));
    }

    heap_free(res as *mut c_void);
    heap_free(str as *mut c_void);

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn cldap_openW(hostname: *mut u16, portnumber: u32) -> *mut WLDAP32_LDAP {
    #[cfg(feature = "have_ldap")]
    {
        let mut ld: *mut WLDAP32_LDAP = core::ptr::null_mut();
        let mut hostnameU: *mut i8 = core::ptr::null_mut();
        let mut url: *mut i8 = core::ptr::null_mut();

        'exit: {
            if !hostname.is_null() {
                hostnameU = strWtoU(hostname);
                if hostnameU.is_null() {
                    break 'exit;
                }
            } else {
                hostnameU = strWtoU(defaulthost.as_ptr());
                if hostnameU.is_null() {
                    break 'exit;
                }
            }

            url = urlify_hostnames(b"cldap://\0".as_ptr() as *const i8, hostnameU, portnumber);
            if url.is_null() {
                break 'exit;
            }

            ld = create_context(url) as *mut WLDAP32_LDAP;
        }

        strfreeU(hostnameU);
        strfreeU(url);
        return ld;
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_initW(hostname: *const u16, portnumber: u32) -> *mut WLDAP32_LDAP {
    #[cfg(feature = "have_ldap")]
    {
        let mut ld: *mut WLDAP32_LDAP = core::ptr::null_mut();
        let mut hostnameU: *mut i8 = core::ptr::null_mut();
        let mut url: *mut i8 = core::ptr::null_mut();

        'exit: loop {
            if !hostname.is_null() {
                hostnameU = strWtoU(hostname);
                if hostnameU.is_null() {
                    break 'exit;
                }
            } else {
                hostnameU = strWtoU(defaulthost.as_ptr());
                if hostnameU.is_null() {
                    break 'exit;
                }
            }

            url = urlify_hostnames(b"ldap://\0".as_ptr() as *const i8, hostnameU, portnumber);
            if url.is_null() {
                break 'exit;
            }

            ld = create_context(url) as *mut WLDAP32_LDAP;
            break 'exit;
        }

        strfreeU(hostnameU);
        strfreeU(url);
        ld
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_openW(hostname: *mut u16, portnumber: u32) -> *mut WLDAP32_LDAP {
    #[cfg(feature = "have_ldap")]
    {
        let mut ld: *mut WLDAP32_LDAP = core::ptr::null_mut();
        let mut hostnameU: *mut i8 = core::ptr::null_mut();
        let mut url: *mut i8 = core::ptr::null_mut();

        'exit: loop {
            if !hostname.is_null() {
                hostnameU = strWtoU(hostname);
                if hostnameU.is_null() {
                    break 'exit;
                }
            } else {
                hostnameU = strWtoU(defaulthost.as_ptr());
                if hostnameU.is_null() {
                    break 'exit;
                }
            }

            url = urlify_hostnames(b"ldap://\0".as_ptr() as *const i8, hostnameU, portnumber);
            if url.is_null() {
                break 'exit;
            }

            ld = create_context(url) as *mut WLDAP32_LDAP;
            break 'exit;
        }

        strfreeU(hostnameU);
        strfreeU(url);
        return ld;
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_sslinitW(
    hostname: *mut u16,
    portnumber: u32,
    secure: i32,
) -> *mut WLDAP32_LDAP {
    #[cfg(feature = "have_ldap")]
    {
        let mut ld: *mut WLDAP32_LDAP = core::ptr::null_mut();
        let mut hostnameU: *mut i8 = core::ptr::null_mut();
        let mut url: *mut i8 = core::ptr::null_mut();

        if !hostname.is_null() {
            hostnameU = strWtoU(hostname);
            if hostnameU.is_null() {
                strfreeU(hostnameU);
                strfreeU(url);
                return ld;
            }
        } else {
            hostnameU = strWtoU(defaulthost.as_ptr());
            if hostnameU.is_null() {
                strfreeU(hostnameU);
                strfreeU(url);
                return ld;
            }
        }

        if secure != 0 {
            url = urlify_hostnames(b"ldaps://\0".as_ptr() as *const i8, hostnameU, portnumber);
        } else {
            url = urlify_hostnames(b"ldap://\0".as_ptr() as *const i8, hostnameU, portnumber);
        }

        if url.is_null() {
            strfreeU(hostnameU);
            strfreeU(url);
            return ld;
        }

        ldap_initialize(&mut ld, url);

        strfreeU(hostnameU);
        strfreeU(url);
        ld
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn cldap_openA(hostname: *mut i8, portnumber: u32) -> *mut WLDAP32_LDAP {
    #[cfg(feature = "have_ldap")]
    {
        let mut ld: *mut WLDAP32_LDAP = core::ptr::null_mut();
        let mut hostnameW: *mut u16 = core::ptr::null_mut();

        'exit: {
            if !hostname.is_null() {
                hostnameW = strAtoW(hostname);
                if hostnameW.is_null() {
                    break 'exit;
                }
            }

            ld = cldap_openW(hostnameW, portnumber);

            break 'exit;
        }

        strfreeW(hostnameW);
        return ld;
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_initA(hostname: *const i8, portnumber: u32) -> *mut WLDAP32_LDAP {
    #[cfg(feature = "have_ldap")]
    {
        let mut ld: *mut WLDAP32_LDAP = core::ptr::null_mut();
        let mut hostnameW: *mut u16 = core::ptr::null_mut();

        'exit: {
            if !hostname.is_null() {
                hostnameW = strAtoW(hostname);
                if hostnameW.is_null() {
                    break 'exit;
                }
            }

            ld = ldap_initW(hostnameW, portnumber);
        }

        strfreeW(hostnameW);
        return ld;
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_openA(hostname: *mut i8, portnumber: u32) -> *mut WLDAP32_LDAP {
    #[cfg(feature = "have_ldap")]
    {
        let mut ld: *mut WLDAP32_LDAP = core::ptr::null_mut();
        let mut hostnameW: *mut u16 = core::ptr::null_mut();

        'exit: loop {
            if !hostname.is_null() {
                hostnameW = strAtoW(hostname);
                if hostnameW.is_null() {
                    break 'exit;
                }
            }

            ld = ldap_openW(hostnameW, portnumber);
            break 'exit;
        }

        strfreeW(hostnameW);
        return ld;
    }
    #[cfg(not(feature = "have_ldap"))]
    {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_sslinitA(
    hostname: PCHAR,
    portnumber: ULONG,
    secure: i32,
) -> *mut WLDAP32_LDAP {
    #[cfg(feature = "have_ldap")]
    {
        let mut ld: *mut WLDAP32_LDAP;
        let mut hostnameW: *mut u16 = core::ptr::null_mut();

        if !hostname.is_null() {
            hostnameW = strAtoW(hostname);
            if hostnameW.is_null() {
                return core::ptr::null_mut();
            }
        }

        ld = ldap_sslinitW(hostnameW, portnumber, secure);

        strfreeW(hostnameW);
        return ld;
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
// #include "wine/port.h"
//
// #include <stdio.h>
// #include <stdarg.h>
// #ifdef HAVE_LDAP_H
// #include <ldap.h>
// #endif
//
// #include "windef.h"
// #include "winbase.h"
// #include "winnls.h"
// #include "wine/winternl.h"
//
// #include "winldap_private.h"
// #include "wldap32.h"
// #include "wine/debug.h"
//
// #ifdef HAVE_LDAP
// /* Should eventually be determined by the algorithm documented on MSDN. */
// static const WCHAR defaulthost[] = { 'l','o','c','a','l','h','o','s','t',0 };
//
// /* Split a space separated string of hostnames into a string array */
// static char **split_hostnames( const char *hostnames )
// {
//     char **res, *str, *p, *q;
//     unsigned int i = 0;
//
//     str = strdupU( hostnames );
//     if (!str) return NULL;
//
//     p = str;
//     while (isspace( *p )) p++;
//     if (*p) i++;
//
//     while (*p)
//     {
//         if (isspace( *p ))
//         {
//             while (isspace( *p )) p++;
//             if (*p) i++;
//         }
//         p++;
//     }
//
//     if (!(res = heap_alloc( (i + 1) * sizeof(char *) )))
//     {
//         heap_free( str );
//         return NULL;
//     }
//
//     p = str;
//     while (isspace( *p )) p++;
//
//     q = p;
//     i = 0;
//
//     while (*p)
//     {
//         if (p[1] != '\0')
//         {
//             if (isspace( *p ))
//             {
//                 *p = '\0'; p++;
//                 res[i] = strdupU( q );
//                 if (!res[i]) goto oom;
//                 i++;
//
//                 while (isspace( *p )) p++;
//                 q = p;
//             }
//         }
//         else
//         {
//             res[i] = strdupU( q );
//             if (!res[i]) goto oom;
//             i++;
//         }
//         p++;
//     }
//     res[i] = NULL;
//
//     heap_free( str );
//     return res;
//
// oom:
//     while (i > 0) strfreeU( res[--i] );
//
//     heap_free( res );
//     heap_free( str );
//
//     return NULL;
// }
//
// /* Determine if a URL starts with a known LDAP scheme */
// static BOOL has_ldap_scheme( char *url )
// {
//     return !_strnicmp( url, "ldap://", 7 )  ||
//            !_strnicmp( url, "ldaps://", 8 ) ||
//            !_strnicmp( url, "ldapi://", 8 ) ||
//            !_strnicmp( url, "cldap://", 8 );
// }
//
// /* Flatten an array of hostnames into a space separated string of URLs.
//  * Prepend a given scheme and append a given port number to each hostname
//  * if necessary.
//  */
// static char *join_hostnames( const char *scheme, char **hostnames, ULONG portnumber )
// {
//     char *res, *p, *q, **v;
//     unsigned int i = 0, size = 0;
//     static const char sep[] = " ", fmt[] = ":%d";
//     char port[7];
//
//     sprintf( port, fmt, portnumber );
//
//     for (v = hostnames; *v; v++)
//     {
//         if (!has_ldap_scheme( *v ))
//         {
//             size += strlen( scheme );
//             q = *v;
//         }
//         else
//             /* skip past colon in scheme prefix */
//             q = strchr( *v, '/' );
//
//         size += strlen( *v );
//
//         if (!strchr( q, ':' ))
//             size += strlen( port );
//
//         i++;
//     }
//
//     size += (i - 1) * strlen( sep );
//     if (!(res = heap_alloc( size + 1 ))) return NULL;
//
//     p = res;
//     for (v = hostnames; *v; v++)
//     {
//         if (v != hostnames)
//         {
//             strcpy( p, sep );
//             p += strlen( sep );
//         }
//
//         if (!has_ldap_scheme( *v ))
//         {
//             strcpy( p, scheme );
//             p += strlen( scheme );
//             q = *v;
//         }
//         else
//             /* skip past colon in scheme prefix */
//             q = strchr( *v, '/' );
//
//         strcpy( p, *v );
//         p += strlen( *v );
//
//         if (!strchr( q, ':' ))
//         {
//             strcpy( p, port );
//             p += strlen( port );
//         }
//     }
//     return res;
// }
//
// static char *urlify_hostnames( const char *scheme, char *hostnames, ULONG port )
// {
//     char *url = NULL, **strarray;
//
//     strarray = split_hostnames( hostnames );
//     if (strarray)
//         url = join_hostnames( scheme, strarray, port );
//     else
//         return NULL;
//
//     strarrayfreeU( strarray );
//     return url;
// }
// #endif
//
// WINE_DEFAULT_DEBUG_CHANNEL(wldap32);
//
// #ifdef HAVE_LDAP
// static LDAP *create_context( const char *url )
// {
//     LDAP *ld;
//     int version = LDAP_VERSION3;
//     if (ldap_initialize( &ld, url ) != LDAP_SUCCESS) return NULL;
//     ldap_set_option( ld, LDAP_OPT_PROTOCOL_VERSION, &version );
//     return ld;
// }
// #endif
//
// /***********************************************************************
//  *      cldap_openA     (WLDAP32.@)
//  *
//  * See cldap_openW.
//  */
// WLDAP32_LDAP * CDECL cldap_openA( PCHAR hostname, ULONG portnumber )
// {
// #ifdef HAVE_LDAP
//     WLDAP32_LDAP *ld = NULL;
//     WCHAR *hostnameW = NULL;
//
//     TRACE( "(%s, %d)\n", debugstr_a!(hostname), portnumber );
//
//     if (hostname) {
//         hostnameW = strAtoW( hostname );
//         if (!hostnameW) goto exit;
//     }
//
//     ld = cldap_openW( hostnameW, portnumber );
//
// exit:
//     strfreeW( hostnameW );
//     return ld;
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      cldap_openW     (WLDAP32.@)
//  *
//  * Initialize an LDAP context and create a UDP connection.
//  *
//  * PARAMS
//  *  hostname   [I] Name of the host to connect to.
//  *  portnumber [I] Port number to use.
//  *
//  * RETURNS
//  *  Success: Pointer to an LDAP context.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  The hostname string can be a space separated string of hostnames,
//  *  in which case the LDAP runtime will try to connect to the hosts
//  *  in order, until a connection can be made. A hostname may have a
//  *  trailing port number (separated from the hostname by a ':'), which
//  *  will take precedence over the port number supplied as a parameter
//  *  to this function.
//  */
// WLDAP32_LDAP * CDECL cldap_openW( PWCHAR hostname, ULONG portnumber )
// {
// #ifdef HAVE_LDAP
//     LDAP *ld = NULL;
//     char *hostnameU = NULL, *url = NULL;
//
//     TRACE( "(%s, %d)\n", debugstr_w!(hostname), portnumber );
//
//     if (hostname) {
//         hostnameU = strWtoU( hostname );
//         if (!hostnameU) goto exit;
//     }
//     else {
//         hostnameU = strWtoU( defaulthost );
//         if (!hostnameU) goto exit;
//     }
//
//     url = urlify_hostnames( "cldap://", hostnameU, portnumber );
//     if (!url) goto exit;
//
//     ld = create_context( url );
//
// exit:
//     strfreeU( hostnameU );
//     strfreeU( url );
//     return ld;
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_connect     (WLDAP32.@)
//  *
//  * Connect to an LDAP server.
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  timeout [I] Pointer to an l_timeval structure specifying the
//  *              timeout in seconds.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  The timeout parameter may be NULL in which case a default timeout
//  *  value will be used.
//  */
// ULONG CDECL ldap_connect( WLDAP32_LDAP *ld, struct l_timeval *timeout )
// {
//     TRACE( "(%p, %p)\n", ld, timeout );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//     return WLDAP32_LDAP_SUCCESS; /* FIXME: do something, e.g. ping the host */
// }
//
// /***********************************************************************
//  *      ldap_initA     (WLDAP32.@)
//  *
//  * See ldap_initW.
//  */
// WLDAP32_LDAP *  CDECL ldap_initA( const PCHAR hostname, ULONG portnumber )
// {
// #ifdef HAVE_LDAP
//     WLDAP32_LDAP *ld = NULL;
//     WCHAR *hostnameW = NULL;
//
//     TRACE( "(%s, %d)\n", debugstr_a!(hostname), portnumber );
//
//     if (hostname) {
//         hostnameW = strAtoW( hostname );
//         if (!hostnameW) goto exit;
//     }
//
//     ld = ldap_initW( hostnameW, portnumber );
//
// exit:
//     strfreeW( hostnameW );
//     return ld;
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_initW     (WLDAP32.@)
//  *
//  * Initialize an LDAP context and create a TCP connection.
//  *
//  * PARAMS
//  *  hostname   [I] Name of the host to connect to.
//  *  portnumber [I] Port number to use.
//  *
//  * RETURNS
//  *  Success: Pointer to an LDAP context.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  The hostname string can be a space separated string of hostnames,
//  *  in which case the LDAP runtime will try to connect to the hosts
//  *  in order, until a connection can be made. A hostname may have a
//  *  trailing port number (separated from the hostname by a ':'), which
//  *  will take precedence over the port number supplied as a parameter
//  *  to this function. The connection will not be made until the first
//  *  LDAP function that needs it is called.
//  */
// WLDAP32_LDAP * CDECL ldap_initW( const PWCHAR hostname, ULONG portnumber )
// {
// #ifdef HAVE_LDAP
//     LDAP *ld = NULL;
//     char *hostnameU = NULL, *url = NULL;
//
//     TRACE( "(%s, %d)\n", debugstr_w!(hostname), portnumber );
//
//     if (hostname) {
//         hostnameU = strWtoU( hostname );
//         if (!hostnameU) goto exit;
//     }
//     else {
//         hostnameU = strWtoU( defaulthost );
//         if (!hostnameU) goto exit;
//     }
//
//     url = urlify_hostnames( "ldap://", hostnameU, portnumber );
//     if (!url) goto exit;
//
//     ld = create_context( url );
//
// exit:
//     strfreeU( hostnameU );
//     strfreeU( url );
//     return ld;
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_openA     (WLDAP32.@)
//  *
//  * See ldap_openW.
//  */
// WLDAP32_LDAP * CDECL ldap_openA( PCHAR hostname, ULONG portnumber )
// {
// #ifdef HAVE_LDAP
//     WLDAP32_LDAP *ld = NULL;
//     WCHAR *hostnameW = NULL;
//
//     TRACE( "(%s, %d)\n", debugstr_a!(hostname), portnumber );
//
//     if (hostname) {
//         hostnameW = strAtoW( hostname );
//         if (!hostnameW) goto exit;
//     }
//
//     ld = ldap_openW( hostnameW, portnumber );
//
// exit:
//     strfreeW( hostnameW );
//     return ld;
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_openW     (WLDAP32.@)
//  *
//  * Initialize an LDAP context and create a TCP connection.
//  *
//  * PARAMS
//  *  hostname   [I] Name of the host to connect to.
//  *  portnumber [I] Port number to use.
//  *
//  * RETURNS
//  *  Success: Pointer to an LDAP context.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  The hostname string can be a space separated string of hostnames,
//  *  in which case the LDAP runtime will try to connect to the hosts
//  *  in order, until a connection can be made. A hostname may have a
//  *  trailing port number (separated from the hostname by a ':'), which
//  *  will take precedence over the port number supplied as a parameter
//  *  to this function.
//  */
// WLDAP32_LDAP * CDECL ldap_openW( PWCHAR hostname, ULONG portnumber )
// {
// #ifdef HAVE_LDAP
//     LDAP *ld = NULL;
//     char *hostnameU = NULL, *url = NULL;
//
//     TRACE( "(%s, %d)\n", debugstr_w!(hostname), portnumber );
//
//     if (hostname) {
//         hostnameU = strWtoU( hostname );
//         if (!hostnameU) goto exit;
//     }
//     else {
//         hostnameU = strWtoU( defaulthost );
//         if (!hostnameU) goto exit;
//     }
//
//     url = urlify_hostnames( "ldap://", hostnameU, portnumber );
//     if (!url) goto exit;
//
//     ld = create_context( url );
//
// exit:
//     strfreeU( hostnameU );
//     strfreeU( url );
//     return ld;
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_sslinitA     (WLDAP32.@)
//  *
//  * See ldap_sslinitW.
//  */
// WLDAP32_LDAP * CDECL ldap_sslinitA( PCHAR hostname, ULONG portnumber, int secure )
// {
// #ifdef HAVE_LDAP
//     WLDAP32_LDAP *ld;
//     WCHAR *hostnameW = NULL;
//
//     TRACE( "(%s, %d, 0x%08x)\n", debugstr_a!(hostname), portnumber, secure );
//
//     if (hostname) {
//         hostnameW = strAtoW( hostname );
//         if (!hostnameW) return NULL;
//     }
//
//     ld  = ldap_sslinitW( hostnameW, portnumber, secure );
//
//     strfreeW( hostnameW );
//     return ld;
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_sslinitW     (WLDAP32.@)
//  *
//  * Initialize an LDAP context and create a secure TCP connection.
//  *
//  * PARAMS
//  *  hostname   [I] Name of the host to connect to.
//  *  portnumber [I] Port number to use.
//  *  secure     [I] Ask the server to create an SSL connection.
//  *
//  * RETURNS
//  *  Success: Pointer to an LDAP context.
//  *  Failure: NULL
//  *
//  * NOTES
//  *  The hostname string can be a space separated string of hostnames,
//  *  in which case the LDAP runtime will try to connect to the hosts
//  *  in order, until a connection can be made. A hostname may have a
//  *  trailing port number (separated from the hostname by a ':'), which
//  *  will take precedence over the port number supplied as a parameter
//  *  to this function. The connection will not be made until the first
//  *  LDAP function that needs it is called.
//  */
// WLDAP32_LDAP * CDECL ldap_sslinitW( PWCHAR hostname, ULONG portnumber, int secure )
// {
// #ifdef HAVE_LDAP
//     WLDAP32_LDAP *ld = NULL;
//     char *hostnameU = NULL, *url = NULL;
//
//     TRACE( "(%s, %d, 0x%08x)\n", debugstr_w!(hostname), portnumber, secure );
//
//     if (hostname) {
//         hostnameU = strWtoU( hostname );
//         if (!hostnameU) goto exit;
//     }
//     else {
//         hostnameU = strWtoU( defaulthost );
//         if (!hostnameU) goto exit;
//     }
//
//     if (secure)
//         url = urlify_hostnames( "ldaps://", hostnameU, portnumber );
//     else
//         url = urlify_hostnames( "ldap://", hostnameU, portnumber );
//
//     if (!url) goto exit;
//     ldap_initialize( &ld, url );
//
// exit:
//     strfreeU( hostnameU );
//     strfreeU( url );
//     return ld;
//
// #else
//     return NULL;
// #endif
// }
//
// /***********************************************************************
//  *      ldap_start_tls_sA     (WLDAP32.@)
//  *
//  * See ldap_start_tls_sW.
//  */
// ULONG CDECL ldap_start_tls_sA( WLDAP32_LDAP *ld, PULONG retval, WLDAP32_LDAPMessage **result,
//     PLDAPControlA *serverctrls, PLDAPControlA *clientctrls )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %p, %p, %p, %p)\n", ld, retval, result, serverctrls, clientctrls );
//
//     if (!ld) return ~0u;
//
//     if (serverctrls) {
//         serverctrlsW = controlarrayAtoW( serverctrls );
//         if (!serverctrlsW) goto exit;
//     }
//     if (clientctrls) {
//         clientctrlsW = controlarrayAtoW( clientctrls );
//         if (!clientctrlsW) goto exit;
//     }
//
//     ret = ldap_start_tls_sW( ld, retval, result, serverctrlsW, clientctrlsW );
//
// exit:
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_start_tls_s     (WLDAP32.@)
//  *
//  * Start TLS encryption on an LDAP connection.
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  retval      [I] Return value from the server.
//  *  result      [I] Response message from the server.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  LDAP function that needs it is called.
//  */
// ULONG CDECL ldap_start_tls_sW( WLDAP32_LDAP *ld, PULONG retval, WLDAP32_LDAPMessage **result,
//     PLDAPControlW *serverctrls, PLDAPControlW *clientctrls )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %p, %p, %p, %p)\n", ld, retval, result, serverctrls, clientctrls );
//
//     if (!ld) return ~0u;
//
//     if (serverctrls) {
//         serverctrlsU = controlarrayWtoU( serverctrls );
//         if (!serverctrlsU) goto exit;
//     }
//     if (clientctrls) {
//         clientctrlsU = controlarrayWtoU( clientctrls );
//         if (!clientctrlsU) goto exit;
//     }
//
//     ret = map_error( ldap_start_tls_s( ld, serverctrlsU, clientctrlsU ));
//
// exit:
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_startup     (WLDAP32.@)
//  */
// ULONG CDECL ldap_startup( PLDAP_VERSION_INFO version, HANDLE *instance )
// {
//     TRACE( "(%p, %p)\n", version, instance );
//     return WLDAP32_LDAP_SUCCESS;
// }
//
// /***********************************************************************
//  *      ldap_stop_tls_s     (WLDAP32.@)
//  *
//  * Stop TLS encryption on an LDAP connection.
//  *
//  * PARAMS
//  *  ld [I] Pointer to an LDAP context.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOLEAN CDECL ldap_stop_tls_s( WLDAP32_LDAP *ld )
// {
//     TRACE( "(%p)\n", ld );
//     return TRUE; /* FIXME: find a way to stop tls on a connection */
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
