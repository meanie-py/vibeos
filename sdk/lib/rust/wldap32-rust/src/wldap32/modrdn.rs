use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from this preamble

#[no_mangle]
pub unsafe extern "C" fn ldap_modrdnA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    newdn: PCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut newdnW: PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || newdn.is_null() {
        return !0u;
    }

    'exit: {
        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }

        newdnW = strAtoW(newdn);
        if newdnW.is_null() {
            break 'exit;
        }

        ret = ldap_modrdnW(ld, dnW, newdnW);
    }

    strfreeW(dnW);
    strfreeW(newdnW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_modrdnW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    newdn: PWCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut newdnU: *mut i8 = core::ptr::null_mut();
    let mut msg: i32 = 0;

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() || newdn.is_null() {
            ret = !0u;
            break 'exit;
        }

        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }

        newdnU = strWtoU(newdn);
        if newdnU.is_null() {
            break 'exit;
        }

        ret = ldap_rename(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *const i8 },
            newdnU,
            core::ptr::null_mut(),
            1,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut msg,
        );

        if ret == LDAP_SUCCESS {
            ret = msg as ULONG;
        } else {
            ret = !0u;
        }
    }

    strfreeU(dnU);
    strfreeU(newdnU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_modrdn2A(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    newdn: PCHAR,
    delete: INT,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut newdnW: PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || newdn.is_null() {
        return !0u;
    }

    'exit: loop {
        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }

        newdnW = strAtoW(newdn);
        if newdnW.is_null() {
            break 'exit;
        }

        ret = ldap_modrdn2W(ld, dnW, newdnW, delete);
        break 'exit;
    }

    strfreeW(dnW);
    strfreeW(newdnW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_modrdn2W(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    newdn: PWCHAR,
    delete: INT,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut newdnU: *mut i8 = core::ptr::null_mut();
    let mut msg: i32 = 0;

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || newdn.is_null() {
        return !0u;
    }

    'exit: {
        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }

        newdnU = strWtoU(newdn);
        if newdnU.is_null() {
            break 'exit;
        }

        ret = ldap_rename(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *mut i8 },
            newdnU,
            core::ptr::null_mut(),
            delete,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut msg,
        );

        if ret == LDAP_SUCCESS {
            ret = msg as ULONG;
        } else {
            ret = !0u;
        }
    }

    strfreeU(dnU);
    strfreeU(newdnU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_modrdn2_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    newdn: PCHAR,
    delete: INT,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut newdnW: PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || newdn.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }

        newdnW = strAtoW(newdn);
        if newdnW.is_null() {
            break 'exit;
        }

        ret = ldap_modrdn2_sW(ld, dnW, newdnW, delete);
    }

    strfreeW(dnW);
    strfreeW(newdnW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_modrdn2_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    newdn: PWCHAR,
    delete: INT,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut newdnU: *mut i8 = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || newdn.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }

        newdnU = strWtoU(newdn);
        if newdnU.is_null() {
            break 'exit;
        }

        ret = map_error(ldap_rename_s(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *mut i8 },
            newdnU,
            core::ptr::null_mut(),
            delete,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ));
    }

    strfreeU(dnU);
    strfreeU(newdnU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_modrdn_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    newdn: PCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut newdnW: PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || newdn.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }

        newdnW = strAtoW(newdn);
        if newdnW.is_null() {
            break 'exit;
        }

        ret = ldap_modrdn_sW(ld, dnW, newdnW);
    }

    strfreeW(dnW);
    strfreeW(newdnW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_modrdn_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    newdn: PWCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut newdnU: *mut i8 = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || newdn.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }

        newdnU = strWtoU(newdn);
        if newdnU.is_null() {
            break 'exit;
        }

        ret = map_error(ldap_rename_s(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *const i8 },
            newdnU,
            core::ptr::null(),
            1,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ));
    }

    strfreeU(dnU);
    strfreeU(newdnU);

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
//  *      ldap_modrdnA     (WLDAP32.@)
//  *
//  * See ldap_modrdnW.
//  */
// ULONG CDECL ldap_modrdnA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR newdn )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *newdnW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s)\n", ld, debugstr_a(dn), debugstr_a(newdn) );
//
//     if (!ld || !newdn) return ~0u;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//
//     newdnW = strAtoW( newdn );
//     if (!newdnW) goto exit;
//
//     ret = ldap_modrdnW( ld, dnW, newdnW );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( newdnW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_modrdnW     (WLDAP32.@)
//  *
//  * Change the RDN of a directory entry (asynchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of the entry to change.
//  *  newdn   [I] New DN for the entry.
//  *
//  * RETURNS
//  *  Success: Message ID of the modrdn operation.
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Call ldap_result with the message ID to get the result of
//  *  the operation. Cancel the operation by calling ldap_abandon
//  *  with the message ID.
//  */
// ULONG CDECL ldap_modrdnW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR newdn )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *newdnU = NULL;
//     int msg;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s)\n", ld, debugstr_w(dn), debugstr_w(newdn) );
//
//     if (!ld || !newdn) return ~0u;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//
//     newdnU = strWtoU( newdn );
//     if (!newdnU) goto exit;
//
//     ret = ldap_rename( ld, dn ? dnU : "", newdnU, NULL, 1, NULL, NULL, &msg );
//
//     if (ret == LDAP_SUCCESS)
//         ret = msg;
//     else
//         ret = ~0u;
//
// exit:
//     strfreeU( dnU );
//     strfreeU( newdnU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_modrdn2A     (WLDAP32.@)
//  *
//  * See ldap_modrdn2W.
//  */
// ULONG CDECL ldap_modrdn2A( WLDAP32_LDAP *ld, PCHAR dn, PCHAR newdn, INT delete )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *newdnW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, 0x%02x)\n", ld, debugstr_a(dn), newdn, delete );
//
//     if (!ld || !newdn) return ~0u;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//
//     newdnW = strAtoW( newdn );
//     if (!newdnW) goto exit;
//
//     ret = ldap_modrdn2W( ld, dnW, newdnW, delete );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( newdnW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_modrdn2W     (WLDAP32.@)
//  *
//  * Change the RDN of a directory entry (asynchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of the entry to change.
//  *  newdn   [I] New DN for the entry.
//  *  delete  [I] Delete old DN?
//  *
//  * RETURNS
//  *  Success: Message ID of the modrdn operation.
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Call ldap_result with the message ID to get the result of
//  *  the operation. Cancel the operation by calling ldap_abandon
//  *  with the message ID.
//  */
// ULONG CDECL ldap_modrdn2W( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR newdn, INT delete )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *newdnU = NULL;
//     int msg;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, 0x%02x)\n", ld, debugstr_w(dn), newdn, delete );
//
//     if (!ld || !newdn) return ~0u;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//
//     newdnU = strWtoU( newdn );
//     if (!newdnU) goto exit;
//
//     ret = ldap_rename( ld, dn ? dnU : "", newdnU, NULL, delete, NULL, NULL, &msg );
//
//     if (ret == LDAP_SUCCESS)
//         ret = msg;
//     else
//         ret = ~0u;
//
// exit:
//     strfreeU( dnU );
//     strfreeU( newdnU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_modrdn2_sA     (WLDAP32.@)
//  *
//  * See ldap_modrdn2_sW.
//  */
// ULONG CDECL ldap_modrdn2_sA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR newdn, INT delete )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *newdnW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, 0x%02x)\n", ld, debugstr_a(dn), newdn, delete );
//
//     if (!ld || !newdn) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//
//     newdnW = strAtoW( newdn );
//     if (!newdnW) goto exit;
//
//     ret = ldap_modrdn2_sW( ld, dnW, newdnW, delete );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( newdnW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_modrdn2_sW     (WLDAP32.@)
//  *
//  * Change the RDN of a directory entry (synchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of the entry to change.
//  *  newdn   [I] New DN for the entry.
//  *  delete  [I] Delete old DN?
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_modrdn2_sW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR newdn, INT delete )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *newdnU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, 0x%02x)\n", ld, debugstr_w(dn), newdn, delete );
//
//     if (!ld || !newdn) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//
//     newdnU = strWtoU( newdn );
//     if (!newdnU) goto exit;
//
//     ret = map_error( ldap_rename_s( ld, dn ? dnU : "", newdnU, NULL, delete, NULL, NULL ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( newdnU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_modrdn_sA     (WLDAP32.@)
//  *
//  * See ldap_modrdn_sW.
//  */
// ULONG CDECL ldap_modrdn_sA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR newdn )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *newdnW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p)\n", ld, debugstr_a(dn), newdn );
//
//     if (!ld || !newdn) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//
//     newdnW = strAtoW( newdn );
//     if (!newdnW) goto exit;
//
//     ret = ldap_modrdn_sW( ld, dnW, newdnW );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( newdnW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_modrdn_sW     (WLDAP32.@)
//  *
//  * Change the RDN of a directory entry (synchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of the entry to change.
//  *  newdn   [I] New DN for the entry.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_modrdn_sW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR newdn )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *newdnU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p)\n", ld, debugstr_w(dn), newdn );
//
//     if (!ld || !newdn) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//
//     newdnU = strWtoU( newdn );
//     if (!newdnU) goto exit;
//
//     ret = map_error( ldap_rename_s( ld, dn ? dnU : "", newdnU, NULL, 1, NULL, NULL ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( newdnU );
//
// #endif
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
