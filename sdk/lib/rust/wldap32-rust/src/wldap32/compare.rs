use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the C preamble

#[no_mangle]
pub unsafe extern "C" fn ldap_compareA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    attr: PCHAR,
    value: PCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut attrW: PWCHAR = core::ptr::null_mut();
    let mut valueW: PWCHAR = core::ptr::null_mut();

    ret = !0u;

    if ld.is_null() || attr.is_null() {
        return !0u;
    }

    'exit: {
        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }

        attrW = strAtoW(attr);
        if attrW.is_null() {
            break 'exit;
        }

        if !value.is_null() {
            valueW = strAtoW(value);
            if valueW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_compareW(ld, dnW, attrW, valueW);
    }

    strfreeW(dnW);
    strfreeW(attrW);
    strfreeW(valueW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_compareW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    attr: PWCHAR,
    value: PWCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut attrU: *mut i8 = core::ptr::null_mut();
    let mut valueU: *mut i8 = core::ptr::null_mut();
    let mut val: berval = berval {
        bv_len: 0,
        bv_val: core::ptr::null_mut(),
    };
    let mut msg: i32 = 0;

    ret = !0u;

    if ld.is_null() || attr.is_null() {
        return !0u;
    }

    'exit: loop {
        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }

        attrU = strWtoU(attr);
        if attrU.is_null() {
            break 'exit;
        }

        if !value.is_null() {
            valueU = strWtoU(value);
            if valueU.is_null() {
                break 'exit;
            }

            val.bv_len = strlen(valueU);
            val.bv_val = valueU;
        }

        ret = ldap_compare_ext(
            ld,
            if !dn.is_null() {
                dnU
            } else {
                b"\0".as_ptr() as *const i8
            },
            attrU,
            &mut val,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut msg,
        );

        if ret == LDAP_SUCCESS {
            ret = msg as ULONG;
        } else {
            ret = !0u;
        }

        break 'exit;
    }

    strfreeU(dnU);
    strfreeU(attrU);
    strfreeU(valueU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_compare_extA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    attr: PCHAR,
    value: PCHAR,
    data: *mut WLDAP32_berval,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    message: *mut ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut attrW: PWCHAR = core::ptr::null_mut();
    let mut valueW: PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() || message.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }
        if !attr.is_null() {
            attrW = strAtoW(attr);
            if attrW.is_null() {
                break 'exit;
            }
        }
        if !value.is_null() {
            valueW = strAtoW(value);
            if valueW.is_null() {
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

        ret = ldap_compare_extW(
            ld,
            dnW,
            attrW,
            valueW,
            data,
            serverctrlsW,
            clientctrlsW,
            message,
        );
    }

    strfreeW(dnW);
    strfreeW(attrW);
    strfreeW(valueW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_compare_extW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    attr: PWCHAR,
    value: PWCHAR,
    data: *mut WLDAP32_berval,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    message: *mut ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut attrU: *mut i8 = core::ptr::null_mut();
    let mut valueU: *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut val: berval = berval {
        bv_len: 0,
        bv_val: core::ptr::null_mut(),
    };

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() || message.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }
        if attr.is_null() {
            ret = WLDAP32_LDAP_NO_MEMORY;
            break 'exit;
        }

        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }

        attrU = strWtoU(attr);
        if attrU.is_null() {
            break 'exit;
        }

        if data.is_null() {
            if !value.is_null() {
                valueU = strWtoU(value);
                if valueU.is_null() {
                    break 'exit;
                }

                val.bv_len = strlen(valueU);
                val.bv_val = valueU;
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

        ret = map_error(ldap_compare_ext(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *mut i8 },
            attrU,
            if !data.is_null() { data as *mut berval } else { &mut val as *mut berval },
            serverctrlsU,
            clientctrlsU,
            message as *mut i32,
        ));
    }

    strfreeU(dnU);
    strfreeU(attrU);
    strfreeU(valueU);
    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_compare_ext_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    attr: PCHAR,
    value: PCHAR,
    data: *mut WLDAP32_berval,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut attrW: PWCHAR = core::ptr::null_mut();
    let mut valueW: PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }
        if !attr.is_null() {
            attrW = strAtoW(attr);
            if attrW.is_null() {
                break 'exit;
            }
        }
        if !value.is_null() {
            valueW = strAtoW(value);
            if valueW.is_null() {
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

        ret = ldap_compare_ext_sW(ld, dnW, attrW, valueW, data, serverctrlsW, clientctrlsW);
    }

    strfreeW(dnW);
    strfreeW(attrW);
    strfreeW(valueW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_compare_ext_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    attr: PWCHAR,
    value: PWCHAR,
    data: *mut WLDAP32_berval,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut attrU: *mut i8 = core::ptr::null_mut();
    let mut valueU: *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut val = berval {
        bv_len: 0,
        bv_val: core::ptr::null_mut(),
    };

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

        if !attr.is_null() {
            attrU = strWtoU(attr);
            if attrU.is_null() {
                break 'exit;
            }
        }

        if data.is_null() {
            if !value.is_null() {
                valueU = strWtoU(value);
                if valueU.is_null() {
                    break 'exit;
                }

                val.bv_len = strlen(valueU);
                val.bv_val = valueU;
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

        ret = map_error(ldap_compare_ext_s(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *mut i8 },
            if !attr.is_null() { attrU } else { b"\0".as_ptr() as *mut i8 },
            if !data.is_null() { data as *mut berval } else { &mut val as *mut berval },
            serverctrlsU,
            clientctrlsU,
        ));
    }

    strfreeU(dnU);
    strfreeU(attrU);
    strfreeU(valueU);
    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_compare_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    attr: PCHAR,
    value: PCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut attrW: PWCHAR = core::ptr::null_mut();
    let mut valueW: PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }
        if !attr.is_null() {
            attrW = strAtoW(attr);
            if attrW.is_null() {
                break 'exit;
            }
        }
        if !value.is_null() {
            valueW = strAtoW(value);
            if valueW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_compare_sW(ld, dnW, attrW, valueW);
    }

    strfreeW(dnW);
    strfreeW(attrW);
    strfreeW(valueW);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn ldap_compare_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    attr: PWCHAR,
    value: PWCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut attrU: *mut i8 = core::ptr::null_mut();
    let mut valueU: *mut i8 = core::ptr::null_mut();
    let mut val = berval {
        bv_len: 0,
        bv_val: core::ptr::null_mut(),
    };

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: loop {
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
        if !attr.is_null() {
            attrU = strWtoU(attr);
            if attrU.is_null() {
                break 'exit;
            }
        }
        if !value.is_null() {
            valueU = strWtoU(value);
            if valueU.is_null() {
                break 'exit;
            }

            val.bv_len = strlen(valueU);
            val.bv_val = valueU;
        }

        ret = map_error(ldap_compare_ext_s(
            ld,
            if !dn.is_null() { dnU } else { b"\0".as_ptr() as *mut i8 },
            if !attr.is_null() { attrU } else { b"\0".as_ptr() as *mut i8 },
            &mut val,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ));

        break 'exit;
    }

    strfreeU(dnU);
    strfreeU(attrU);
    strfreeU(valueU);

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
//  *      ldap_compareA     (WLDAP32.@)
//  *
//  * See ldap_compareW.
//  */
// ULONG CDECL ldap_compareA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR attr, PCHAR value )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *attrW = NULL, *valueW = NULL;
//
//     ret = ~0u;
//
//     TRACE( "(%p, %s, %s, %s)\n", ld, debugstr_a(dn), debugstr_a(attr),
//            debugstr_a(value) );
//
//     if (!ld || !attr) return ~0u;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//
//     attrW = strAtoW( attr );
//     if (!attrW) goto exit;
//
//     if (value) {
//         valueW = strAtoW( value );
//         if (!valueW) goto exit;
//     }
//
//     ret = ldap_compareW( ld, dnW, attrW, valueW );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( attrW );
//     strfreeW( valueW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_compareW     (WLDAP32.@)
//  *
//  * Check if an attribute has a certain value (asynchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of entry to compare value for.
//  *  attr    [I] Attribute to compare value for.
//  *  value   [I] Value to compare.
//  *
//  * RETURNS
//  *  Success: Message ID of the compare operation.
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_compareW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR attr, PWCHAR value )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *attrU = NULL, *valueU = NULL;
//     struct berval val = { 0, NULL };
//     int msg;
//
//     ret = ~0u;
//
//     TRACE( "(%p, %s, %s, %s)\n", ld, debugstr_w(dn), debugstr_w(attr),
//            debugstr_w(value) );
//
//     if (!ld || !attr) return ~0u;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//
//     attrU = strWtoU( attr );
//     if (!attrU) goto exit;
//
//     if (value) {
//         valueU = strWtoU( value );
//         if (!valueU) goto exit;
//
//         val.bv_len = strlen( valueU );
//         val.bv_val = valueU;
//     }
//
//     ret = ldap_compare_ext( ld, dn ? dnU : "", attrU, &val, NULL, NULL, &msg );
//
//     if (ret == LDAP_SUCCESS)
//         ret = msg;
//     else
//         ret = ~0u;
//
// exit:
//     strfreeU( dnU );
//     strfreeU( attrU );
//     strfreeU( valueU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_compare_extA     (WLDAP32.@)
//  *
//  * See ldap_compare_extW.
//  */
// ULONG CDECL ldap_compare_extA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR attr, PCHAR value,
//     struct WLDAP32_berval *data, PLDAPControlA *serverctrls, PLDAPControlA *clientctrls,
//     ULONG *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *attrW = NULL, *valueW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s, %p, %p, %p, %p)\n", ld, debugstr_a(dn),
//            debugstr_a(attr), debugstr_a(value), data, serverctrls,
//            clientctrls, message );
//
//     if (!ld || !message) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (attr) {
//         attrW = strAtoW( attr );
//         if (!attrW) goto exit;
//     }
//     if (value) {
//         valueW = strAtoW( value );
//         if (!valueW) goto exit;
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
//     ret = ldap_compare_extW( ld, dnW, attrW, valueW, data,
//                              serverctrlsW, clientctrlsW, message );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( attrW );
//     strfreeW( valueW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_compare_extW     (WLDAP32.@)
//  *
//  * Check if an attribute has a certain value (asynchronous operation).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  dn          [I] DN of entry to compare value for.
//  *  attr        [I] Attribute to compare value for.
//  *  value       [I] string encoded value to compare.
//  *  data        [I] berval encoded value to compare.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *  message     [O] Message ID of the compare operation.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Set value to compare strings or data to compare binary values. If
//  *  both are non-NULL, data will be used. The serverctrls and clientctrls
//  *  parameters are optional and should be set to NULL if not used.
//  */
// ULONG CDECL ldap_compare_extW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR attr, PWCHAR value,
//     struct WLDAP32_berval *data, PLDAPControlW *serverctrls, PLDAPControlW *clientctrls,
//     ULONG *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *attrU = NULL, *valueU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//     struct berval val = { 0, NULL };
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s, %p, %p, %p, %p)\n", ld, debugstr_w(dn),
//            debugstr_w(attr), debugstr_w(value), data, serverctrls,
//            clientctrls, message );
//
//     if (!ld || !message) return WLDAP32_LDAP_PARAM_ERROR;
//     if (!attr) return WLDAP32_LDAP_NO_MEMORY;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//
//     attrU = strWtoU( attr );
//     if (!attrU) goto exit;
//
//     if (!data) {
//         if (value) {
//             valueU = strWtoU( value );
//             if (!valueU) goto exit;
//
//             val.bv_len = strlen( valueU );
//             val.bv_val = valueU;
//         }
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
//     ret = map_error( ldap_compare_ext( ld, dn ? dnU : "", attrU, data ? (struct berval *)data : &val,
//                                        serverctrlsU, clientctrlsU, (int *)message ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( attrU );
//     strfreeU( valueU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_compare_ext_sA     (WLDAP32.@)
//  *
//  * See ldap_compare_ext_sW.
//  */
// ULONG CDECL ldap_compare_ext_sA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR attr, PCHAR value,
//     struct WLDAP32_berval *data, PLDAPControlA *serverctrls, PLDAPControlA *clientctrls )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *attrW = NULL, *valueW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s, %p, %p, %p)\n", ld, debugstr_a(dn),
//            debugstr_a(attr), debugstr_a(value), data, serverctrls,
//            clientctrls );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (attr) {
//         attrW = strAtoW( attr );
//         if (!attrW) goto exit;
//     }
//     if (value) {
//         valueW = strAtoW( value );
//         if (!valueW) goto exit;
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
//     ret = ldap_compare_ext_sW( ld, dnW, attrW, valueW, data, serverctrlsW,
//                                clientctrlsW );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( attrW );
//     strfreeW( valueW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_compare_ext_sW     (WLDAP32.@)
//  *
//  * Check if an attribute has a certain value (synchronous operation).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  dn          [I] DN of entry to compare value for.
//  *  attr        [I] Attribute to compare value for.
//  *  value       [I] string encoded value to compare.
//  *  data        [I] berval encoded value to compare.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Set value to compare strings or data to compare binary values. If
//  *  both are non-NULL, data will be used. The serverctrls and clientctrls
//  *  parameters are optional and should be set to NULL if not used.
//  */
// ULONG CDECL ldap_compare_ext_sW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR attr, PWCHAR value,
//     struct WLDAP32_berval *data, PLDAPControlW *serverctrls, PLDAPControlW *clientctrls )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *attrU = NULL, *valueU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//     struct berval val = { 0, NULL };
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s, %p, %p, %p)\n", ld, debugstr_w(dn),
//            debugstr_w(attr), debugstr_w(value), data, serverctrls,
//            clientctrls );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//     if (attr) {
//         attrU = strWtoU( attr );
//         if (!attrU) goto exit;
//     }
//     if (!data) {
//         if (value) {
//             valueU = strWtoU( value );
//             if (!valueU) goto exit;
//
//             val.bv_len = strlen( valueU );
//             val.bv_val = valueU;
//         }
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
//     ret = map_error( ldap_compare_ext_s( ld, dn ? dnU : "", attr ? attrU : "",
//                                          data ? (struct berval *)data : &val,
//                                          serverctrlsU, clientctrlsU ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( attrU );
//     strfreeU( valueU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_compare_sA     (WLDAP32.@)
//  *
//  * See ldap_compare_sW.
//  */
// ULONG CDECL ldap_compare_sA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR attr, PCHAR value )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *attrW = NULL, *valueW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s)\n", ld, debugstr_a(dn), debugstr_a(attr),
//            debugstr_a(value) );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (attr) {
//         attrW = strAtoW( attr );
//         if (!attrW) goto exit;
//     }
//     if (value) {
//         valueW = strAtoW( value );
//         if (!valueW) goto exit;
//     }
//
//     ret = ldap_compare_sW( ld, dnW, attrW, valueW );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( attrW );
//     strfreeW( valueW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_compare_sW     (WLDAP32.@)
//  *
//  * Check if an attribute has a certain value (synchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of entry to compare value for.
//  *  attr    [I] Attribute to compare value for.
//  *  value   [I] Value to compare.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_compare_sW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR attr, PWCHAR value )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *attrU = NULL, *valueU = NULL;
//     struct berval val = { 0, NULL };
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %s)\n", ld, debugstr_w(dn), debugstr_w(attr),
//            debugstr_w(value) );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//     if (attr) {
//         attrU = strWtoU( attr );
//         if (!attrU) goto exit;
//     }
//     if (value) {
//         valueU = strWtoU( value );
//         if (!valueU) goto exit;
//
//         val.bv_len = strlen( valueU );
//         val.bv_val = valueU;
//     }
//
//     ret = map_error( ldap_compare_ext_s( ld, dn ? dnU : "", attr ? attrU : "", &val, NULL, NULL ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( attrU );
//     strfreeU( valueU );
//
// #endif
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
