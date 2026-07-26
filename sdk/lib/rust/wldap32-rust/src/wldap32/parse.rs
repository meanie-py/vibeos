use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the preamble

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_parse_extended_resultA(
    ld: *mut WLDAP32_LDAP,
    result: *mut WLDAP32_LDAPMessage,
    oid: *mut PCHAR,
    data: *mut *mut WLDAP32_berval,
    free: BOOLEAN,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_extended_resultA(
    ld: *mut WLDAP32_LDAP,
    result: *mut WLDAP32_LDAPMessage,
    oid: *mut PCHAR,
    data: *mut *mut WLDAP32_berval,
    free: BOOLEAN,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut oidW: PWCHAR = core::ptr::null_mut();

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }
    if result.is_null() {
        return WLDAP32_LDAP_NO_RESULTS_RETURNED;
    }

    ret = ldap_parse_extended_resultW(ld, result, &mut oidW, data, free);

    if !oid.is_null() {
        *oid = strWtoA(oidW);
        if (*oid).is_null() {
            ret = WLDAP32_LDAP_NO_MEMORY;
        }
        ldap_memfreeW(oidW);
    }

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_parse_extended_resultW(
    ld: *mut WLDAP32_LDAP,
    result: *mut WLDAP32_LDAPMessage,
    oid: *mut PWCHAR,
    data: *mut *mut WLDAP32_berval,
    free: BOOLEAN,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_extended_resultW(
    ld: *mut WLDAP32_LDAP,
    result: *mut WLDAP32_LDAPMessage,
    oid: *mut PWCHAR,
    data: *mut *mut WLDAP32_berval,
    free: BOOLEAN,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut oidU: PCHAR = core::ptr::null_mut();

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }
    if result.is_null() {
        return WLDAP32_LDAP_NO_RESULTS_RETURNED;
    }

    ret = map_error(
        ldap_parse_extended_result(
            ld,
            result,
            core::ptr::addr_of_mut!(oidU),
            data as *mut *mut berval,
            free,
        ),
    );

    if !oid.is_null() {
        *oid = strUtoW(oidU);
        if (*oid).is_null() {
            ret = WLDAP32_LDAP_NO_MEMORY;
        }
        ldap_memfree(oidU);
    }

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_parse_referenceA(
    ld: *mut WLDAP32_LDAP,
    message: *mut WLDAP32_LDAPMessage,
    referrals: *mut *mut PCHAR,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_referenceA(
    ld: *mut WLDAP32_LDAP,
    message: *mut WLDAP32_LDAPMessage,
    referrals: *mut *mut PCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut referralsW: *mut PWCHAR = core::ptr::null_mut();

    if ld.is_null() {
        return !0u32;
    }

    ret = ldap_parse_referenceW(ld, message, &mut referralsW);

    *referrals = strarrayWtoA(referralsW);
    ldap_value_freeW(referralsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap_parse_reference"))]
pub unsafe extern "C" fn ldap_parse_referenceW(
    ld: *mut WLDAP32_LDAP,
    message: *mut WLDAP32_LDAPMessage,
    referrals: *mut *mut PWCHAR,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap_parse_reference")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_referenceW(
    ld: *mut WLDAP32_LDAP,
    message: *mut WLDAP32_LDAPMessage,
    referrals: *mut *mut PWCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut referralsU: *mut *mut i8 = core::ptr::null_mut();

    if ld.is_null() {
        return !0u32;
    }

    ret = map_error(ldap_parse_reference(
        ld,
        message,
        &mut referralsU,
        core::ptr::null_mut(),
        0,
    ));

    *referrals = strarrayUtoW(referralsU);
    ldap_memfree(referralsU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_parse_resultA(
    ld: *mut WLDAP32_LDAP,
    result: *mut WLDAP32_LDAPMessage,
    retcode: *mut ULONG,
    matched: *mut PCHAR,
    error: *mut PCHAR,
    referrals: *mut *mut PCHAR,
    serverctrls: *mut *mut PLDAPControlA,
    free: BOOLEAN,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_resultA(
    ld: *mut WLDAP32_LDAP,
    result: *mut WLDAP32_LDAPMessage,
    retcode: *mut ULONG,
    matched: *mut PCHAR,
    error: *mut PCHAR,
    referrals: *mut *mut PCHAR,
    serverctrls: *mut *mut PLDAPControlA,
    free: BOOLEAN,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut matchedW: PWCHAR = core::ptr::null_mut();
    let mut errorW: PWCHAR = core::ptr::null_mut();
    let mut referralsW: *mut PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    ret = ldap_parse_resultW(
        ld,
        result,
        retcode,
        &mut matchedW,
        &mut errorW,
        &mut referralsW,
        &mut serverctrlsW,
        free,
    );

    if !matched.is_null() {
        *matched = strWtoA(matchedW);
    }
    if !error.is_null() {
        *error = strWtoA(errorW);
    }

    if !referrals.is_null() {
        *referrals = strarrayWtoA(referralsW);
    }
    if !serverctrls.is_null() {
        *serverctrls = controlarrayWtoA(serverctrlsW);
    }

    ldap_memfreeW(matchedW);
    ldap_memfreeW(errorW);
    ldap_value_freeW(referralsW);
    ldap_controls_freeW(serverctrlsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_parse_resultW(
    ld: *mut WLDAP32_LDAP,
    result: *mut WLDAP32_LDAPMessage,
    retcode: *mut ULONG,
    matched: *mut PWCHAR,
    error: *mut PWCHAR,
    referrals: *mut *mut PWCHAR,
    serverctrls: *mut *mut PLDAPControlW,
    free: BOOLEAN,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_resultW(
    ld: *mut WLDAP32_LDAP,
    result: *mut WLDAP32_LDAPMessage,
    retcode: *mut ULONG,
    matched: *mut PWCHAR,
    error: *mut PWCHAR,
    referrals: *mut *mut PWCHAR,
    serverctrls: *mut *mut PLDAPControlW,
    free: BOOLEAN,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut matchedU: *mut i8 = core::ptr::null_mut();
    let mut errorU: *mut i8 = core::ptr::null_mut();
    let mut referralsU: *mut *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    ret = map_error(ldap_parse_result(
        ld,
        result,
        retcode as *mut i32,
        &mut matchedU,
        &mut errorU,
        &mut referralsU,
        &mut serverctrlsU,
        free,
    ));

    if !matched.is_null() {
        *matched = strUtoW(matchedU);
    }
    if !error.is_null() {
        *error = strUtoW(errorU);
    }

    if !referrals.is_null() {
        *referrals = strarrayUtoW(referralsU);
    }
    if !serverctrls.is_null() {
        *serverctrls = controlarrayUtoW(serverctrlsU);
    }

    ldap_memfree(matchedU);
    ldap_memfree(errorU);
    ldap_memvfree(referralsU as *mut *mut core::ffi::c_void);
    ldap_controls_free(serverctrlsU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_parse_sort_controlA(
    ld: *mut WLDAP32_LDAP,
    control: *mut PLDAPControlA,
    result: *mut ULONG,
    attr: *mut PCHAR,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_sort_controlA(
    ld: *mut WLDAP32_LDAP,
    control: *mut PLDAPControlA,
    result: *mut ULONG,
    attr: *mut PCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut attrW: *mut WCHAR = core::ptr::null_mut();
    let mut controlW: *mut *mut LDAPControlW = core::ptr::null_mut();

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }
    if control.is_null() {
        return WLDAP32_LDAP_CONTROL_NOT_FOUND;
    }

    controlW = controlarrayAtoW(control);
    if controlW.is_null() {
        return WLDAP32_LDAP_NO_MEMORY;
    }

    ret = ldap_parse_sort_controlW(ld, controlW, result, &mut attrW);

    *attr = strWtoA(attrW);
    controlarrayfreeW(controlW);

    ret
}

// Two nested guards: the whole body is `#ifdef HAVE_LDAP`, and INSIDE it the C picks
// between `HAVE_LDAP_PARSE_SORT_CONTROL` and `HAVE_LDAP_PARSE_SORTRESPONSE_CONTROL`
// with an `#elif`. The translation rendered the SORTRESPONSE arm, so that arm carries
// its own feature -- with HAVE_LDAP on but neither probe, the C marshals the controls,
// frees them and returns NOT_SUPPORTED, which is exactly what is left here. cfg_gate
// refuses nested regions rather than guess which arm shipped.
#[cfg(not(feature = "have_ldap"))]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_sort_controlW(
    ld: *mut WLDAP32_LDAP,
    control: *mut PLDAPControlW,
    result: *mut ULONG,
    attr: *mut PWCHAR,
) -> ULONG {
    map_error(WLDAP32_LDAP_NOT_SUPPORTED as i32)
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_sort_controlW(
    ld: *mut WLDAP32_LDAP,
    control: *mut PLDAPControlW,
    result: *mut ULONG,
    attr: *mut PWCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut attrU: *mut i8 = core::ptr::null_mut();
    let mut controlU: *mut *mut LDAPControl = core::ptr::null_mut();

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }
    if control.is_null() {
        return WLDAP32_LDAP_CONTROL_NOT_FOUND;
    }

    controlU = controlarrayWtoU(control);
    if controlU.is_null() {
        return WLDAP32_LDAP_NO_MEMORY;
    }

    #[cfg(feature = "have_ldap_parse_sortresponse_control")]
    {
        let mut res: ber_int_t = 0;
        let mut sortcontrol: *mut LDAPControl = core::ptr::null_mut();
        let mut i: u32 = 0;
        while !(*controlU.offset(i as isize)).is_null() {
            if strcmp(LDAP_SERVER_RESP_SORT_OID, (**controlU.offset(i as isize)).ldctl_oid) == 0 {
                sortcontrol = *controlU.offset(i as isize);
            }
            i += 1;
        }

        if sortcontrol.is_null() {
            controlarrayfreeU(controlU);
            return WLDAP32_LDAP_CONTROL_NOT_FOUND;
        }

        ret = ldap_parse_sortresponse_control(ld, sortcontrol, &mut res, &mut attrU);
        if ret == 0 {
            *result = res as ULONG;
            *attr = strUtoW(attrU);
        }
    }

    controlarrayfreeU(controlU);

    map_error(ret)
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_parse_vlv_controlA(
    ld: *mut WLDAP32_LDAP,
    control: *mut PLDAPControlA,
    targetpos: PULONG,
    listcount: PULONG,
    context: *mut *mut WLDAP32_berval,
    errcode: PINT,
) -> INT {
    (WLDAP32_LDAP_NOT_SUPPORTED) as i32
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_vlv_controlA(
    ld: *mut WLDAP32_LDAP,
    control: *mut PLDAPControlA,
    targetpos: PULONG,
    listcount: PULONG,
    context: *mut *mut WLDAP32_berval,
    errcode: PINT,
) -> INT {
    let mut ret: INT = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut controlW: *mut *mut LDAPControlW = core::ptr::null_mut();

    if ld.is_null() {
        return !0u32;
    }

    if !control.is_null() {
        controlW = controlarrayAtoW(control);
        if controlW.is_null() {
            return WLDAP32_LDAP_NO_MEMORY;
        }
    }

    ret = ldap_parse_vlv_controlW(ld, controlW, targetpos, listcount, context, errcode);

    controlarrayfreeW(controlW);

    ret
}

// Two nested guards: the whole body is `#ifdef HAVE_LDAP`, and INSIDE it the C picks
// between `HAVE_LDAP_PARSE_SORT_CONTROL` and `HAVE_LDAP_PARSE_SORTRESPONSE_CONTROL`
// with an `#elif`. The translation rendered the SORTRESPONSE arm, so that arm carries
// its own feature -- with HAVE_LDAP on but neither probe, the C marshals the controls,
// frees them and returns NOT_SUPPORTED, which is exactly what is left here. cfg_gate
// refuses nested regions rather than guess which arm shipped.
#[cfg(not(feature = "have_ldap"))]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_vlv_controlW(
    ld: *mut WLDAP32_LDAP,
    control: *mut PLDAPControlW,
    targetpos: PULONG,
    listcount: PULONG,
    context: *mut *mut WLDAP32_berval,
    errcode: PINT,
) -> INT {
    map_error((WLDAP32_LDAP_NOT_SUPPORTED) as i32) as INT
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_parse_vlv_controlW(
    ld: *mut WLDAP32_LDAP,
    control: *mut PLDAPControlW,
    targetpos: PULONG,
    listcount: PULONG,
    context: *mut *mut WLDAP32_berval,
    errcode: PINT,
) -> INT {
    let mut ret: core::ffi::c_int = (WLDAP32_LDAP_NOT_SUPPORTED) as i32;
    let mut controlU: *mut *mut LDAPControl = core::ptr::null_mut();

    if ld.is_null() || control.is_null() {
        return !0u32;
    }

    controlU = controlarrayWtoU(control);
    if controlU.is_null() {
        return WLDAP32_LDAP_NO_MEMORY;
    }

    #[cfg(feature = "have_ldap_parse_vlvresponse_control")]
    {
        let mut pos: ber_int_t = 0;
        let mut count: ber_int_t = 0;
        let mut vlvcontrol: *mut LDAPControl = core::ptr::null_mut();
        let mut i: core::ffi::c_uint = 0;
        while !(*controlU.offset(i as isize)).is_null() {
            if strcmp(LDAP_CONTROL_VLVRESPONSE, (**controlU.offset(i as isize)).ldctl_oid) == 0 {
                vlvcontrol = *controlU.offset(i as isize);
            }
            i += 1;
        }

        if vlvcontrol.is_null() {
            controlarrayfreeU(controlU);
            return WLDAP32_LDAP_CONTROL_NOT_FOUND;
        }

        ret = ldap_parse_vlvresponse_control(
            ld,
            vlvcontrol,
            &mut pos,
            &mut count,
            context as *mut *mut berval,
            errcode,
        );

        if ret == 0 {
            *targetpos = pos as ULONG;
            *listcount = count as ULONG;
        }
    }

    controlarrayfreeU(controlU);

    map_error(ret)
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
//  *      ldap_parse_extended_resultA     (WLDAP32.@)
//  *
//  * See ldap_parse_extended_resultW.
//  */
// ULONG CDECL ldap_parse_extended_resultA( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *result,
//     PCHAR *oid, struct WLDAP32_berval **data, BOOLEAN free )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *oidW = NULL;
//
//     TRACE( "(%p, %p, %p, %p, 0x%02x)\n", ld, result, oid, data, free );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//     if (!result) return WLDAP32_LDAP_NO_RESULTS_RETURNED;
//
//     ret = ldap_parse_extended_resultW( ld, result, &oidW, data, free );
//
//     if (oid) {
//         *oid = strWtoA( oidW );
//         if (!*oid) ret = WLDAP32_LDAP_NO_MEMORY;
//         ldap_memfreeW( oidW );
//     }
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_parse_extended_resultW     (WLDAP32.@)
//  *
//  * Parse the result of an extended operation.
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  result  [I] Result message from an extended operation.
//  *  oid     [O] OID of the extended operation.
//  *  data    [O] Result data.
//  *  free    [I] Free the result message?
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Free the OID and result data with ldap_memfree. Pass a nonzero
//  *  value for 'free' or call ldap_msgfree to free the result message.
//  */
// ULONG CDECL ldap_parse_extended_resultW( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *result,
//     PWCHAR *oid, struct WLDAP32_berval **data, BOOLEAN free )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *oidU = NULL;
//
//     TRACE( "(%p, %p, %p, %p, 0x%02x)\n", ld, result, oid, data, free );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//     if (!result) return WLDAP32_LDAP_NO_RESULTS_RETURNED;
//
//     ret = map_error( ldap_parse_extended_result( ld, result, &oidU, (struct berval **)data, free ) );
//
//     if (oid) {
//         *oid = strUtoW( oidU );
//         if (!*oid) ret = WLDAP32_LDAP_NO_MEMORY;
//         ldap_memfree( oidU );
//     }
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_parse_referenceA     (WLDAP32.@)
//  *
//  * See ldap_parse_referenceW.
//  */
// ULONG CDECL ldap_parse_referenceA( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *message,
//     PCHAR **referrals )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR **referralsW = NULL;
//
//     TRACE( "(%p, %p, %p)\n", ld, message, referrals );
//
//     if (!ld) return ~0u;
//
//     ret = ldap_parse_referenceW( ld, message, &referralsW );
//
//     *referrals = strarrayWtoA( referralsW );
//     ldap_value_freeW( referralsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_parse_referenceW     (WLDAP32.@)
//  *
//  * Return any referrals from a result message.
//  *
//  * PARAMS
//  *  ld         [I] Pointer to an LDAP context.
//  *  result     [I] Result message.
//  *  referrals  [O] Array of referral URLs.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Free the referrals with ldap_value_free.
//  */
// ULONG CDECL ldap_parse_referenceW( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *message,
//     PWCHAR **referrals )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP_PARSE_REFERENCE
//     char **referralsU = NULL;
//
//     TRACE( "(%p, %p, %p)\n", ld, message, referrals );
//
//     if (!ld) return ~0u;
//
//     ret = map_error( ldap_parse_reference( ld, message, &referralsU, NULL, 0 ));
//
//     *referrals = strarrayUtoW( referralsU );
//     ldap_memfree( referralsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_parse_resultA     (WLDAP32.@)
//  *
//  * See ldap_parse_resultW.
//  */
// ULONG CDECL ldap_parse_resultA( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *result,
//     ULONG *retcode, PCHAR *matched, PCHAR *error, PCHAR **referrals,
//     PLDAPControlA **serverctrls, BOOLEAN free )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *matchedW = NULL, *errorW = NULL, **referralsW = NULL;
//     LDAPControlW **serverctrlsW = NULL;
//
//     TRACE( "(%p, %p, %p, %p, %p, %p, %p, 0x%02x)\n", ld, result, retcode,
//            matched, error, referrals, serverctrls, free );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     ret = ldap_parse_resultW( ld, result, retcode, &matchedW, &errorW,
//                               &referralsW, &serverctrlsW, free );
//
//     if (matched) *matched = strWtoA( matchedW );
//     if (error) *error = strWtoA( errorW );
//
//     if (referrals) *referrals = strarrayWtoA( referralsW );
//     if (serverctrls) *serverctrls = controlarrayWtoA( serverctrlsW );
//
//     ldap_memfreeW( matchedW );
//     ldap_memfreeW( errorW );
//     ldap_value_freeW( referralsW );
//     ldap_controls_freeW( serverctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_parse_resultW     (WLDAP32.@)
//  *
//  * Parse a result message.
//  *
//  * PARAMS
//  *  ld           [I] Pointer to an LDAP context.
//  *  result       [I] Result message.
//  *  retcode      [O] Return code for the server operation.
//  *  matched      [O] DNs matched in the operation.
//  *  error        [O] Error message for the operation.
//  *  referrals    [O] Referrals found in the result message.
//  *  serverctrls  [O] Controls used in the operation.
//  *  free         [I] Free the result message?
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Free the DNs and error message with ldap_memfree. Free
//  *  the referrals with ldap_value_free and the controls with
//  *  ldap_controls_free. Pass a nonzero value for 'free' or call
//  *  ldap_msgfree to free the result message.
//  */
// ULONG CDECL ldap_parse_resultW( WLDAP32_LDAP *ld, WLDAP32_LDAPMessage *result,
//     ULONG *retcode, PWCHAR *matched, PWCHAR *error, PWCHAR **referrals,
//     PLDAPControlW **serverctrls, BOOLEAN free )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *matchedU = NULL, *errorU = NULL, **referralsU = NULL;
//     LDAPControl **serverctrlsU = NULL;
//
//     TRACE( "(%p, %p, %p, %p, %p, %p, %p, 0x%02x)\n", ld, result, retcode,
//            matched, error, referrals, serverctrls, free );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     ret = map_error( ldap_parse_result( ld, result, (int *)retcode, &matchedU, &errorU,
//                                         &referralsU, &serverctrlsU, free ));
//
//     if (matched) *matched = strUtoW( matchedU );
//     if (error) *error = strUtoW( errorU );
//
//     if (referrals) *referrals = strarrayUtoW( referralsU );
//     if (serverctrls) *serverctrls = controlarrayUtoW( serverctrlsU );
//
//     ldap_memfree( matchedU );
//     ldap_memfree( errorU );
//     ldap_memvfree( (void **)referralsU );
//     ldap_controls_free( serverctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_parse_sort_controlA     (WLDAP32.@)
//  *
//  * See ldap_parse_sort_controlW.
//  */
// ULONG CDECL ldap_parse_sort_controlA( WLDAP32_LDAP *ld, PLDAPControlA *control,
//     ULONG *result, PCHAR *attr )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *attrW = NULL;
//     LDAPControlW **controlW = NULL;
//
//     TRACE( "(%p, %p, %p, %p)\n", ld, control, result, attr );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//     if (!control) return WLDAP32_LDAP_CONTROL_NOT_FOUND;
//
//     controlW = controlarrayAtoW( control );
//     if (!controlW) return WLDAP32_LDAP_NO_MEMORY;
//
//     ret = ldap_parse_sort_controlW( ld, controlW, result, &attrW );
//
//     *attr = strWtoA( attrW );
//     controlarrayfreeW( controlW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_parse_sort_controlW     (WLDAP32.@)
//  *
//  * Parse a sort control.
//  *
//  * PARAMS
//  *  ld       [I] Pointer to an LDAP context.
//  *  control  [I] Control obtained from a result message.
//  *  result   [O] Result code.
//  *  attr     [O] Failing attribute.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  If the function fails, free the failing attribute with ldap_memfree.
//  */
// ULONG CDECL ldap_parse_sort_controlW( WLDAP32_LDAP *ld, PLDAPControlW *control,
//     ULONG *result, PWCHAR *attr )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *attrU = NULL;
//     LDAPControl **controlU = NULL;
// #ifdef HAVE_LDAP_PARSE_SORT_CONTROL
//     unsigned long res;
// #elif defined(HAVE_LDAP_PARSE_SORTRESPONSE_CONTROL)
//     ber_int_t res;
//     LDAPControl *sortcontrol = NULL;
//     unsigned int i;
// #endif
//
//     TRACE( "(%p, %p, %p, %p)\n", ld, control, result, attr );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//     if (!control) return WLDAP32_LDAP_CONTROL_NOT_FOUND;
//
//     controlU = controlarrayWtoU( control );
//     if (!controlU) return WLDAP32_LDAP_NO_MEMORY;
//
// #ifdef HAVE_LDAP_PARSE_SORT_CONTROL
//     if (!(ret = ldap_parse_sort_control( ld, controlU, &res, &attrU )))
//     {
//         *result = res;
//         *attr = strUtoW( attrU );
//     }
// #elif defined(HAVE_LDAP_PARSE_SORTRESPONSE_CONTROL)
//     for (i = 0; controlU[i]; i++)
//     {
//         if (!strcmp( LDAP_SERVER_RESP_SORT_OID, controlU[i]->ldctl_oid ))
//             sortcontrol = controlU[i];
//     }
//     if (!sortcontrol)
//     {
//         controlarrayfreeU( controlU );
//         return WLDAP32_LDAP_CONTROL_NOT_FOUND;
//     }
//     if (!(ret = ldap_parse_sortresponse_control( ld, sortcontrol, &res, &attrU )))
//     {
//         *result = res;
//         *attr = strUtoW( attrU );
//     }
// #endif
//     controlarrayfreeU( controlU );
//
// #endif
//     return map_error( ret );
// }
//
// /***********************************************************************
//  *      ldap_parse_vlv_controlA     (WLDAP32.@)
//  *
//  * See ldap_parse_vlv_controlW.
//  */
// INT CDECL ldap_parse_vlv_controlA( WLDAP32_LDAP *ld, PLDAPControlA *control,
//     PULONG targetpos, PULONG listcount,
//     struct WLDAP32_berval **context, PINT errcode )
// {
//     int ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     LDAPControlW **controlW = NULL;
//
//     TRACE( "(%p, %p, %p, %p, %p, %p)\n", ld, control, targetpos,
//            listcount, context, errcode );
//
//     if (!ld) return ~0u;
//
//     if (control) {
//         controlW = controlarrayAtoW( control );
//         if (!controlW) return WLDAP32_LDAP_NO_MEMORY;
//     }
//
//     ret = ldap_parse_vlv_controlW( ld, controlW, targetpos, listcount,
//                                    context, errcode );
//
//     controlarrayfreeW( controlW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_parse_vlv_controlW     (WLDAP32.@)
//  *
//  * Parse a virtual list view control.
//  *
//  * PARAMS
//  *  ld         [I] Pointer to an LDAP context.
//  *  control    [I] Controls obtained from a result message.
//  *  targetpos  [O] Position of the target in the result list.
//  *  listcount  [O] Estimate of the number of results in the list.
//  *  context    [O] Server side context.
//  *  errcode    [O] Error code from the listview operation.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Free the server context with ber_bvfree.
//  */
// INT CDECL ldap_parse_vlv_controlW( WLDAP32_LDAP *ld, PLDAPControlW *control,
//     PULONG targetpos, PULONG listcount,
//     struct WLDAP32_berval **context, PINT errcode )
// {
//     int ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     LDAPControl **controlU = NULL;
// #ifdef HAVE_LDAP_PARSE_VLV_CONTROL
//     unsigned long pos, count;
// #elif defined(HAVE_LDAP_PARSE_VLVRESPONSE_CONTROL)
//     ber_int_t pos, count;
//     LDAPControl *vlvcontrol = NULL;
//     unsigned int i;
// #endif
//
//     TRACE( "(%p, %p, %p, %p, %p, %p)\n", ld, control, targetpos,
//            listcount, context, errcode );
//
//     if (!ld || !control) return ~0u;
//
//     controlU = controlarrayWtoU( control );
//     if (!controlU) return WLDAP32_LDAP_NO_MEMORY;
//
// #ifdef HAVE_LDAP_PARSE_VLV_CONTROL
//     if (!(ret = ldap_parse_vlv_control( ld, controlU, &pos, &count,
//                                         (struct berval **)context, errcode )))
//     {
//         *targetpos = pos;
//         *listcount = count;
//     }
// #elif defined(HAVE_LDAP_PARSE_VLVRESPONSE_CONTROL)
//     for (i = 0; controlU[i]; i++)
//     {
//         if (!strcmp( LDAP_CONTROL_VLVRESPONSE, controlU[i]->ldctl_oid ))
//             vlvcontrol = controlU[i];
//     }
//     if (!vlvcontrol)
//     {
//         controlarrayfreeU( controlU );
//         return WLDAP32_LDAP_CONTROL_NOT_FOUND;
//     }
//     if (!(ret = ldap_parse_vlvresponse_control( ld, vlvcontrol, &pos, &count,
//                                                 (struct berval **)context, errcode )))
//     {
//         *targetpos = pos;
//         *listcount = count;
//     }
// #endif
//     controlarrayfreeU( controlU );
//
// #endif
//     return map_error( ret );
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
