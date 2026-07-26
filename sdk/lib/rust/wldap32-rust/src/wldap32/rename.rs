use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the given C preamble.
// The preamble contains only preprocessor directives and a Wine debug macro,
// which do not produce Rust declarations.

#[no_mangle]
pub unsafe extern "C" fn ldap_rename_extA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    newrdn: PCHAR,
    newparent: PCHAR,
    delete: INT,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    message: *mut ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut newrdnW: PWCHAR = core::ptr::null_mut();
    let mut newparentW: PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || message.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: loop {
        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }
        if !newrdn.is_null() {
            newrdnW = strAtoW(newrdn);
            if newrdnW.is_null() {
                break 'exit;
            }
        }
        if !newparent.is_null() {
            newparentW = strAtoW(newparent);
            if newparentW.is_null() {
                break 'exit;
            }
        }
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

        ret = ldap_rename_extW(
            ld,
            dnW,
            newrdnW,
            newparentW,
            delete,
            serverctrlsW,
            clientctrlsW,
            message,
        );

        break 'exit;
    }

    strfreeW(dnW);
    strfreeW(newrdnW);
    strfreeW(newparentW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_rename_extW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    newrdn: PWCHAR,
    newparent: PWCHAR,
    delete: INT,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    message: *mut ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut newrdnU: *mut i8 = core::ptr::null_mut();
    let mut newparentU: *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() || message.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }
        if !newrdn.is_null() {
            newrdnU = strWtoU(newrdn);
            if newrdnU.is_null() {
                break 'exit;
            }
        }
        if !newparent.is_null() {
            newparentU = strWtoU(newparent);
            if newparentU.is_null() {
                break 'exit;
            }
        }
        if !serverctrls.is_null() {
            serverctrlsU = controlarrayWtoU(serverctrls);
            if serverctrlsU.is_null() {
                break 'exit;
            }
        }
        if !clientctrls.is_null() {
            clientctrlsU = controlarrayWtoU(clientctrls);
            if clientctrlsU.is_null() {
                break 'exit;
            }
        }

        ret = map_error(ldap_rename(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *mut i8 },
            if !newrdn.is_null() { newrdnU } else { b"\0".as_ptr() as *mut i8 },
            newparentU,
            delete,
            serverctrlsU,
            clientctrlsU,
            message as *mut i32,
        ));
    }

    strfreeU(dnU);
    strfreeU(newrdnU);
    strfreeU(newparentU);
    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_rename_ext_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    newrdn: PCHAR,
    newparent: PCHAR,
    delete: INT,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut newrdnW: PWCHAR = core::ptr::null_mut();
    let mut newparentW: PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }
        if !newrdn.is_null() {
            newrdnW = strAtoW(newrdn);
            if newrdnW.is_null() {
                break 'exit;
            }
        }
        if !newparent.is_null() {
            newparentW = strAtoW(newparent);
            if newparentW.is_null() {
                break 'exit;
            }
        }
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

        ret = ldap_rename_ext_sW(ld, dnW, newrdnW, newparentW, delete, serverctrlsW, clientctrlsW);
    }

    strfreeW(dnW);
    strfreeW(newrdnW);
    strfreeW(newparentW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_rename_ext_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    newrdn: PWCHAR,
    newparent: PWCHAR,
    delete: INT,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut newrdnU: *mut i8 = core::ptr::null_mut();
    let mut newparentU: *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }
        if !newrdn.is_null() {
            newrdnU = strWtoU(newrdn);
            if newrdnU.is_null() {
                break 'exit;
            }
        }
        if !newparent.is_null() {
            newparentU = strWtoU(newparent);
            if newparentU.is_null() {
                break 'exit;
            }
        }
        if !serverctrls.is_null() {
            serverctrlsU = controlarrayWtoU(serverctrls);
            if serverctrlsU.is_null() {
                break 'exit;
            }
        }
        if !clientctrls.is_null() {
            clientctrlsU = controlarrayWtoU(clientctrls);
            if clientctrlsU.is_null() {
                break 'exit;
            }
        }

        ret = map_error(ldap_rename_s(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *mut i8 },
            if !newrdn.is_null() { newrdnU } else { b"\0".as_ptr() as *mut i8 },
            newparentU,
            delete,
            serverctrlsU,
            clientctrlsU,
        ));
    }

    strfreeU(dnU);
    strfreeU(newrdnU);
    strfreeU(newparentU);
    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
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
//  *      ldap_rename_extA     (WLDAP32.@)
//  *
//  *  See ldap_rename_extW.
//  */
// ULONG CDECL ldap_rename_extA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR newrdn,
//     PCHAR newparent, INT delete, PLDAPControlA *serverctrls,
//     PLDAPControlA *clientctrls, ULONG *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *newrdnW = NULL, *newparentW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s, 0x%02x, %p, %p, %p)\n", ld, debugstr_a(dn),
//            debugstr_a(newrdn), debugstr_a(newparent), delete,
//            serverctrls, clientctrls, message );
//
//     if (!ld || !message) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (newrdn) {
//         newrdnW = strAtoW( newrdn );
//         if (!newrdnW) goto exit;
//     }
//     if (newparent) {
//         newparentW = strAtoW( newparent );
//         if (!newparentW) goto exit;
//     }
//     if (serverctrls) {
//         serverctrlsW = controlarrayAtoW( serverctrls );
//         if (!serverctrlsW) goto exit;
//     }
//     if (clientctrls) {
//         clientctrlsW = controlarrayAtoW( clientctrls );
//         if (!clientctrlsW) goto exit;
//     }
//
//     ret = ldap_rename_extW( ld, dnW, newrdnW, newparentW, delete,
//                             serverctrlsW, clientctrlsW, message );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( newrdnW );
//     strfreeW( newparentW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_rename_extW     (WLDAP32.@)
//  *
//  * Change the DN of a directory entry (asynchronous operation).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  dn          [I] DN of the entry to change.
//  *  newrdn      [I] New RDN for the entry.
//  *  newparent   [I] New parent for the entry.
//  *  delete      [I] Delete old RDN?
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *  message     [O] Message ID of the operation.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Call ldap_result with the message ID to get the result of
//  *  the operation. Cancel the operation by calling ldap_abandon
//  *  with the message ID.
//  */
// ULONG CDECL ldap_rename_extW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR newrdn,
//     PWCHAR newparent, INT delete, PLDAPControlW *serverctrls,
//     PLDAPControlW *clientctrls, ULONG *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *newrdnU = NULL, *newparentU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s, 0x%02x, %p, %p, %p)\n", ld, debugstr_w(dn),
//            debugstr_w(newrdn), debugstr_w(newparent), delete,
//            serverctrls, clientctrls, message );
//
//     if (!ld || !message) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//     if (newrdn) {
//         newrdnU = strWtoU( newrdn );
//         if (!newrdnU) goto exit;
//     }
//     if (newparent) {
//         newparentU = strWtoU( newparent );
//         if (!newparentU) goto exit;
//     }
//     if (serverctrls) {
//         serverctrlsU = controlarrayWtoU( serverctrls );
//         if (!serverctrlsU) goto exit;
//     }
//     if (clientctrls) {
//         clientctrlsU = controlarrayWtoU( clientctrls );
//         if (!clientctrlsU) goto exit;
//     }
//
//     ret = map_error( ldap_rename( ld, dn ? dnU : "", newrdn ? newrdnU : "", newparentU,
//                                   delete, serverctrlsU, clientctrlsU, (int *)message ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( newrdnU );
//     strfreeU( newparentU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_rename_ext_sA     (WLDAP32.@)
//  *
//  *  See ldap_rename_ext_sW.
//  */
// ULONG CDECL ldap_rename_ext_sA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR newrdn,
//     PCHAR newparent, INT delete, PLDAPControlA *serverctrls,
//     PLDAPControlA *clientctrls )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *newrdnW = NULL, *newparentW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s, 0x%02x, %p, %p)\n", ld, debugstr_a(dn),
//            debugstr_a(newrdn), debugstr_a(newparent), delete,
//            serverctrls, clientctrls );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (newrdn) {
//         newrdnW = strAtoW( newrdn );
//         if (!newrdnW) goto exit;
//     }
//     if (newparent) {
//         newparentW = strAtoW( newparent );
//         if (!newparentW) goto exit;
//     }
//     if (serverctrls) {
//         serverctrlsW = controlarrayAtoW( serverctrls );
//         if (!serverctrlsW) goto exit;
//     }
//     if (clientctrls) {
//         clientctrlsW = controlarrayAtoW( clientctrls );
//         if (!clientctrlsW) goto exit;
//     }
//
//     ret = ldap_rename_ext_sW( ld, dnW, newrdnW, newparentW, delete,
//                               serverctrlsW, clientctrlsW );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( newrdnW );
//     strfreeW( newparentW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
// /***********************************************************************
//  *      ldap_rename_ext_sW     (WLDAP32.@)
//  *
//  * Change the DN of a directory entry (synchronous operation).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  dn          [I] DN of the entry to change.
//  *  newrdn      [I] New RDN for the entry.
//  *  newparent   [I] New parent for the entry.
//  *  delete      [I] Delete old RDN?
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_rename_ext_sW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR newrdn,
//     PWCHAR newparent, INT delete, PLDAPControlW *serverctrls,
//     PLDAPControlW *clientctrls )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *newrdnU = NULL, *newparentU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s, 0x%02x, %p, %p)\n", ld, debugstr_w(dn),
//            debugstr_w(newrdn), debugstr_w(newparent), delete,
//            serverctrls, clientctrls );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//     if (newrdn) {
//         newrdnU = strWtoU( newrdn );
//         if (!newrdnU) goto exit;
//     }
//     if (newparent) {
//         newparentU = strWtoU( newparent );
//         if (!newparentU) goto exit;
//     }
//     if (serverctrls) {
//         serverctrlsU = controlarrayWtoU( serverctrls );
//         if (!serverctrlsU) goto exit;
//     }
//     if (clientctrls) {
//         clientctrlsU = controlarrayWtoU( clientctrls );
//         if (!clientctrlsU) goto exit;
//     }
//
//     ret = map_error( ldap_rename_s( ld, dn ? dnU : "", newrdn ? newrdnU : "", newparentU,
//                                     delete, serverctrlsU, clientctrlsU ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( newrdnU );
//     strfreeU( newparentU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
