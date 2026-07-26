use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the provided preamble.
// The C code only contains conditional includes and debug channel macro,
// which do not produce Rust declarations.

#[no_mangle]
pub unsafe extern "C" fn ldap_close_extended_op(ld: *mut WLDAP32_LDAP, msgid: ULONG) -> ULONG {
    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }
    return WLDAP32_LDAP_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn ldap_extended_operationA(
    ld: *mut WLDAP32_LDAP,
    oid: PCHAR,
    data: *mut WLDAP32_berval,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    message: *mut ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut oidW: PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() || message.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !oid.is_null() {
            oidW = strAtoW(oid);
            if oidW.is_null() {
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

        ret = ldap_extended_operationW(ld, oidW, data, serverctrlsW, clientctrlsW, message);
    }

    strfreeW(oidW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_extended_operationW(
    ld: *mut WLDAP32_LDAP,
    oid: PWCHAR,
    data: *mut WLDAP32_berval,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    message: *mut ULONG,
) -> ULONG {
    let mut ret = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut oidU: *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || message.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !oid.is_null() {
            oidU = strWtoU(oid);
            if oidU.is_null() {
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

        ret = map_error(ldap_extended_operation(
            ld,
            if !oid.is_null() { oidU } else { b"\0".as_ptr() as *mut i8 },
            data as *mut berval,
            serverctrlsU,
            clientctrlsU,
            message as *mut i32,
        ));
    }

    strfreeU(oidU);
    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_extended_operation_sA(
    ld: *mut WLDAP32_LDAP,
    oid: PCHAR,
    data: *mut WLDAP32_berval,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    retoid: *mut PCHAR,
    retdata: *mut *mut WLDAP32_berval,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut oidW: PWCHAR = core::ptr::null_mut();
    let mut retoidW: PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !oid.is_null() {
            oidW = strAtoW(oid);
            if oidW.is_null() {
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

        ret = ldap_extended_operation_sW(
            ld,
            oidW,
            data,
            serverctrlsW,
            clientctrlsW,
            &mut retoidW,
            retdata,
        );

        if !retoid.is_null() && !retoidW.is_null() {
            *retoid = strWtoA(retoidW);
            if (*retoid).is_null() {
                ret = WLDAP32_LDAP_NO_MEMORY;
            }
            ldap_memfreeW(retoidW);
        }
    }

    strfreeW(oidW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_extended_operation_sW(
    ld: *mut WLDAP32_LDAP,
    oid: PWCHAR,
    data: *mut WLDAP32_berval,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    retoid: *mut PWCHAR,
    retdata: *mut *mut WLDAP32_berval,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut oidU: *mut i8 = core::ptr::null_mut();
    let mut retoidU: *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: loop {
        if !oid.is_null() {
            oidU = strWtoU(oid);
            if oidU.is_null() {
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

        ret = map_error(ldap_extended_operation_s(
            ld,
            if !oid.is_null() { oidU } else { b"\0".as_ptr() as *mut i8 },
            data as *mut berval,
            serverctrlsU,
            clientctrlsU,
            core::ptr::addr_of_mut!(retoidU),
            retdata as *mut *mut berval,
        ));

        if !retoid.is_null() && !retoidU.is_null() {
            *retoid = strUtoW(retoidU);
            if (*retoid).is_null() {
                ret = WLDAP32_LDAP_NO_MEMORY;
            }
            ldap_memfree(retoidU);
        }

        break 'exit;
    }

    strfreeU(oidU);
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
//  *      ldap_close_extended_op     (WLDAP32.@)
//  *
//  * Close an extended operation.
//  *
//  * PARAMS
//  *  ld    [I] Pointer to an LDAP context.
//  *  msgid [I] Message ID of the operation to be closed.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Contrary to native, OpenLDAP does not require us to close
//  *  extended operations, so this is a no-op.
//  */
// ULONG CDECL ldap_close_extended_op( WLDAP32_LDAP *ld, ULONG msgid )
// {
//     TRACE( "(%p, 0x%08x)\n", ld, msgid );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//     return WLDAP32_LDAP_SUCCESS;
// }
//
// /***********************************************************************
//  *      ldap_extended_operationA     (WLDAP32.@)
//  *
//  * See ldap_extended_operationW.
//  */
// ULONG CDECL ldap_extended_operationA( WLDAP32_LDAP *ld, PCHAR oid, struct WLDAP32_berval *data,
//     PLDAPControlA *serverctrls, PLDAPControlA *clientctrls, ULONG *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *oidW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, %p, %p, %p)\n", ld, debugstr_a(oid), data, serverctrls,
//            clientctrls, message );
//
//     if (!ld || !message) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (oid) {
//         oidW = strAtoW( oid );
//         if (!oidW) goto exit;
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
//     ret = ldap_extended_operationW( ld, oidW, data, serverctrlsW, clientctrlsW, message );
//
// exit:
//     strfreeW( oidW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_extended_operationW     (WLDAP32.@)
//  *
//  * Perform an extended operation (asynchronous mode).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  oid         [I] OID of the extended operation.
//  *  data        [I] Data needed by the operation.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *  message     [O] Message ID of the extended operation.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  The data parameter should be set to NULL if the operation
//  *  requires no data. Call ldap_result with the message ID to
//  *  get the result of the operation or ldap_abandon to cancel
//  *  the operation. The serverctrls and clientctrls parameters
//  *  are optional and should be set to NULL if not used. Call
//  *  ldap_close_extended_op to close the operation.
//  */
// ULONG CDECL ldap_extended_operationW( WLDAP32_LDAP *ld, PWCHAR oid, struct WLDAP32_berval *data,
//     PLDAPControlW *serverctrls, PLDAPControlW *clientctrls, ULONG *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *oidU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, %p, %p, %p)\n", ld, debugstr_w(oid), data, serverctrls,
//            clientctrls, message );
//
//     if (!ld || !message) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (oid) {
//         oidU = strWtoU( oid );
//         if (!oidU) goto exit;
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
//     ret = map_error( ldap_extended_operation( ld, oid ? oidU : "", (struct berval *)data,
//                                               serverctrlsU, clientctrlsU, (int *)message ));
//
// exit:
//     strfreeU( oidU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_extended_operation_sA     (WLDAP32.@)
//  *
//  * See ldap_extended_operation_sW.
//  */
// ULONG CDECL ldap_extended_operation_sA( WLDAP32_LDAP *ld, PCHAR oid, struct WLDAP32_berval *data,
//     PLDAPControlA *serverctrls, PLDAPControlA *clientctrls, PCHAR *retoid,
//     struct WLDAP32_berval **retdata )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *oidW = NULL, *retoidW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, %p, %p, %p, %p)\n", ld, debugstr_a(oid), data, serverctrls,
//            clientctrls, retoid, retdata );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (oid) {
//         oidW = strAtoW( oid );
//         if (!oidW) goto exit;
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
//     ret = ldap_extended_operation_sW( ld, oidW, data, serverctrlsW, clientctrlsW,
//                                       &retoidW, retdata );
//
//     if (retoid && retoidW) {
//         *retoid = strWtoA( retoidW );
//         if (!*retoid) ret = WLDAP32_LDAP_NO_MEMORY;
//         ldap_memfreeW( retoidW );
//     }
//
// exit:
//     strfreeW( oidW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_extended_operation_sW     (WLDAP32.@)
//  *
//  * Perform an extended operation (synchronous mode).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  oid         [I] OID of the extended operation.
//  *  data        [I] Data needed by the operation.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *  retoid      [O] OID of the server response message.
//  *  retdata     [O] Data returned by the server.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  The data parameter should be set to NULL if the operation
//  *  requires no data. The serverctrls, clientctrls, retoid and
//  *  and retdata parameters are also optional. Set to NULL if not
//  *  used. Free retoid and retdata after use with ldap_memfree.
//  */
// ULONG CDECL ldap_extended_operation_sW( WLDAP32_LDAP *ld, PWCHAR oid, struct WLDAP32_berval *data,
//     PLDAPControlW *serverctrls, PLDAPControlW *clientctrls, PWCHAR *retoid,
//     struct WLDAP32_berval **retdata )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *oidU = NULL, *retoidU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, %p, %p, %p, %p)\n", ld, debugstr_w(oid), data, serverctrls,
//            clientctrls, retoid, retdata );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (oid) {
//         oidU = strWtoU( oid );
//         if (!oidU) goto exit;
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
//     ret = map_error( ldap_extended_operation_s( ld, oid ? oidU : "", (struct berval *)data, serverctrlsU,
//                                                 clientctrlsU, &retoidU, (struct berval **)retdata ));
//
//     if (retoid && retoidU) {
//         *retoid = strUtoW( retoidU );
//         if (!*retoid) ret = WLDAP32_LDAP_NO_MEMORY;
//         ldap_memfree( retoidU );
//     }
//
// exit:
//     strfreeU( oidU );
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
