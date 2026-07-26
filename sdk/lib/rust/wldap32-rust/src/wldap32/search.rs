use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from this preamble.
// The C code only contains preprocessor directives and a debug channel macro,
// neither of which produce Rust declarations.

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_searchA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_searchA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut baseW: PWCHAR = core::ptr::null_mut();
    let mut filterW: PWCHAR = core::ptr::null_mut();
    let mut attrsW: *mut PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return !0u32;
    }

    'exit: {
        if !base.is_null() {
            baseW = strAtoW(base);
            if baseW.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterW = strAtoW(filter);
            if filterW.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsW = strarrayAtoW(attrs);
            if attrsW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_searchW(ld, baseW, scope, filterW, attrsW, attrsonly);
    }

    strfreeW(baseW);
    strfreeW(filterW);
    strarrayfreeW(attrsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_searchW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_searchW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
) -> ULONG {
    let mut ret = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut baseU: *mut i8 = core::ptr::null_mut();
    let mut filterU: *mut i8 = core::ptr::null_mut();
    let mut attrsU: *mut *mut i8 = core::ptr::null_mut();
    let mut msg: i32 = 0;

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return !0u32;
    }

    'exit: {
        if !base.is_null() {
            baseU = strWtoU(base);
            if baseU.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterU = strWtoU(filter);
            if filterU.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsU = strarrayWtoU(attrs);
            if attrsU.is_null() {
                break 'exit;
            }
        }

        ret = ldap_search_ext(
            ld,
            baseU,
            scope,
            filterU,
            attrsU,
            attrsonly,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            &mut msg,
        );

        if ret == LDAP_SUCCESS {
            ret = msg as ULONG;
        } else {
            ret = !0u32;
        }
    }

    strfreeU(baseU);
    strfreeU(filterU);
    strarrayfreeU(attrsU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_search_extA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    timelimit: ULONG,
    sizelimit: ULONG,
    message: *mut ULONG,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_search_extA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    timelimit: ULONG,
    sizelimit: ULONG,
    message: *mut ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut baseW: PWCHAR = core::ptr::null_mut();
    let mut filterW: PWCHAR = core::ptr::null_mut();
    let mut attrsW: *mut PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !base.is_null() {
            baseW = strAtoW(base);
            if baseW.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterW = strAtoW(filter);
            if filterW.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsW = strarrayAtoW(attrs);
            if attrsW.is_null() {
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

        ret = ldap_search_extW(
            ld,
            baseW,
            scope,
            filterW,
            attrsW,
            attrsonly,
            serverctrlsW,
            clientctrlsW,
            timelimit,
            sizelimit,
            message,
        );
    }

    strfreeW(baseW);
    strfreeW(filterW);
    strarrayfreeW(attrsW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_search_extW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    timelimit: ULONG,
    sizelimit: ULONG,
    message: *mut ULONG,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_search_extW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    timelimit: ULONG,
    sizelimit: ULONG,
    message: *mut ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut baseU: *mut i8 = core::ptr::null_mut();
    let mut filterU: *mut i8 = core::ptr::null_mut();
    let mut attrsU: *mut *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut tv: timeval = core::mem::zeroed();
    let mut tvp: *mut timeval = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() {
            return !0u32;
        }

        if !base.is_null() {
            baseU = strWtoU(base);
            if baseU.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterU = strWtoU(filter);
            if filterU.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsU = strarrayWtoU(attrs);
            if attrsU.is_null() {
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

        if timelimit != 0 {
            tv.tv_sec = timelimit as i64;
            tv.tv_usec = 0;
            tvp = &mut tv;
        }

        ret = map_error(ldap_search_ext(
            ld,
            baseU,
            scope,
            filterU,
            attrsU,
            attrsonly,
            serverctrlsU,
            clientctrlsU,
            tvp,
            sizelimit,
            message as *mut i32,
        ));
    }

    strfreeU(baseU);
    strfreeU(filterU);
    strarrayfreeU(attrsU);
    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_search_ext_sA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    timeout: *mut l_timeval,
    sizelimit: ULONG,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_search_ext_sA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    timeout: *mut l_timeval,
    sizelimit: ULONG,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut baseW: PWCHAR = core::ptr::null_mut();
    let mut filterW: PWCHAR = core::ptr::null_mut();
    let mut attrsW: *mut PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || res.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !base.is_null() {
            baseW = strAtoW(base);
            if baseW.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterW = strAtoW(filter);
            if filterW.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsW = strarrayAtoW(attrs);
            if attrsW.is_null() {
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

        ret = ldap_search_ext_sW(
            ld,
            baseW,
            scope,
            filterW,
            attrsW,
            attrsonly,
            serverctrlsW,
            clientctrlsW,
            timeout,
            sizelimit,
            res,
        );
    }

    strfreeW(baseW);
    strfreeW(filterW);
    strarrayfreeW(attrsW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_search_ext_sW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    timeout: *mut l_timeval,
    sizelimit: ULONG,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_search_ext_sW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    timeout: *mut l_timeval,
    sizelimit: ULONG,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut baseU: *mut i8 = core::ptr::null_mut();
    let mut filterU: *mut i8 = core::ptr::null_mut();
    let mut attrsU: *mut *mut i8 = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || res.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !base.is_null() {
            baseU = strWtoU(base);
            if baseU.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterU = strWtoU(filter);
            if filterU.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsU = strarrayWtoU(attrs);
            if attrsU.is_null() {
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

        ret = map_error(ldap_search_ext_s(
            ld,
            baseU,
            scope,
            filterU,
            attrsU,
            attrsonly,
            serverctrlsU,
            clientctrlsU,
            timeout as *mut timeval,
            sizelimit,
            res,
        ));
    }

    strfreeU(baseU);
    strfreeU(filterU);
    strarrayfreeU(attrsU);
    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_search_sA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_search_sA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut baseW: PWCHAR = core::ptr::null_mut();
    let mut filterW: PWCHAR = core::ptr::null_mut();
    let mut attrsW: *mut PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() || res.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !base.is_null() {
            baseW = strAtoW(base);
            if baseW.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterW = strAtoW(filter);
            if filterW.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsW = strarrayAtoW(attrs);
            if attrsW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_search_sW(ld, baseW, scope, filterW, attrsW, attrsonly, res);
    }

    strfreeW(baseW);
    strfreeW(filterW);
    strarrayfreeW(attrsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_search_sW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_search_sW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut baseU: *mut i8 = core::ptr::null_mut();
    let mut filterU: *mut i8 = core::ptr::null_mut();
    let mut attrsU: *mut *mut i8 = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() || res.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !base.is_null() {
            baseU = strWtoU(base);
            if baseU.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterU = strWtoU(filter);
            if filterU.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsU = strarrayWtoU(attrs);
            if attrsU.is_null() {
                break 'exit;
            }
        }

        ret = map_error(ldap_search_ext_s(
            ld,
            baseU,
            scope,
            filterU,
            attrsU,
            attrsonly,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            res,
        ));
    }

    strfreeU(baseU);
    strfreeU(filterU);
    strarrayfreeU(attrsU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_search_stA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
    timeout: *mut l_timeval,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_search_stA(
    ld: *mut WLDAP32_LDAP,
    base: PCHAR,
    scope: ULONG,
    filter: PCHAR,
    attrs: *mut PCHAR,
    attrsonly: ULONG,
    timeout: *mut l_timeval,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut baseW: PWCHAR = core::ptr::null_mut();
    let mut filterW: PWCHAR = core::ptr::null_mut();
    let mut attrsW: *mut PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() || res.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !base.is_null() {
            baseW = strAtoW(base);
            if baseW.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterW = strAtoW(filter);
            if filterW.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsW = strarrayAtoW(attrs);
            if attrsW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_search_stW(ld, baseW, scope, filterW, attrsW, attrsonly, timeout, res);
    }

    strfreeW(baseW);
    strfreeW(filterW);
    strarrayfreeW(attrsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_search_stW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
    timeout: *mut l_timeval,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_search_stW(
    ld: *mut WLDAP32_LDAP,
    base: PWCHAR,
    scope: ULONG,
    filter: PWCHAR,
    attrs: *mut PWCHAR,
    attrsonly: ULONG,
    timeout: *mut l_timeval,
    res: *mut *mut WLDAP32_LDAPMessage,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut baseU: *mut i8 = core::ptr::null_mut();
    let mut filterU: *mut i8 = core::ptr::null_mut();
    let mut attrsU: *mut *mut i8 = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || res.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !base.is_null() {
            baseU = strWtoU(base);
            if baseU.is_null() {
                break 'exit;
            }
        }
        if !filter.is_null() {
            filterU = strWtoU(filter);
            if filterU.is_null() {
                break 'exit;
            }
        }
        if !attrs.is_null() {
            attrsU = strarrayWtoU(attrs);
            if attrsU.is_null() {
                break 'exit;
            }
        }

        ret = map_error(ldap_search_ext_s(
            ld,
            baseU,
            scope,
            filterU,
            attrsU,
            attrsonly,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            timeout as *mut timeval,
            0,
            res,
        ));
    }

    strfreeU(baseU);
    strfreeU(filterU);
    strarrayfreeU(attrsU);

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
//  *      ldap_searchA     (WLDAP32.@)
//  *
//  * See ldap_searchW.
//  */
// ULONG CDECL ldap_searchA( WLDAP32_LDAP *ld, PCHAR base, ULONG scope, PCHAR filter,
//     PCHAR attrs[], ULONG attrsonly )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *baseW = NULL, *filterW = NULL, **attrsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x)\n", ld, debugstr_a!(base),
//            scope, debugstr_a!(filter), attrs, attrsonly );
//
//     if (!ld) return ~0u;
//
//     if (base) {
//         baseW = strAtoW( base );
//         if (!baseW) goto exit;
//     }
//     if (filter) {
//         filterW = strAtoW( filter );
//         if (!filterW) goto exit;
//     }
//     if (attrs) {
//         attrsW = strarrayAtoW( attrs );
//         if (!attrsW) goto exit;
//     }
//
//     ret = ldap_searchW( ld, baseW, scope, filterW, attrsW, attrsonly );
//
// exit:
//     strfreeW( baseW );
//     strfreeW( filterW );
//     strarrayfreeW( attrsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_searchW     (WLDAP32.@)
//  *
//  * Search a directory tree (asynchronous operation).
//  *
//  * PARAMS
//  *  ld        [I] Pointer to an LDAP context.
//  *  base      [I] Starting point for the search.
//  *  scope     [I] Search scope. One of LDAP_SCOPE_BASE,
//  *                LDAP_SCOPE_ONELEVEL and LDAP_SCOPE_SUBTREE.
//  *  filter    [I] Search filter.
//  *  attrs     [I] Attributes to return.
//  *  attrsonly [I] Return no values, only attributes.
//  *
//  * RETURNS
//  *  Success: Message ID of the search operation.
//  *  Failure: ~0u
//  *
//  * NOTES
//  *  Call ldap_result with the message ID to get the result of
//  *  the operation. Cancel the operation by calling ldap_abandon
//  *  with the message ID.
//  */
// ULONG CDECL ldap_searchW( WLDAP32_LDAP *ld, PWCHAR base, ULONG scope, PWCHAR filter,
//     PWCHAR attrs[], ULONG attrsonly )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *baseU = NULL, *filterU = NULL, **attrsU = NULL;
//     int msg;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x)\n", ld, debugstr_w!(base),
//            scope, debugstr_w!(filter), attrs, attrsonly );
//
//     if (!ld) return ~0u;
//
//     if (base) {
//         baseU = strWtoU( base );
//         if (!baseU) goto exit;
//     }
//     if (filter) {
//         filterU = strWtoU( filter );
//         if (!filterU) goto exit;
//     }
//     if (attrs) {
//         attrsU = strarrayWtoU( attrs );
//         if (!attrsU) goto exit;
//     }
//
//     ret = ldap_search_ext( ld, baseU, scope, filterU, attrsU, attrsonly,
//                            NULL, NULL, NULL, 0, &msg );
//
//     if (ret == LDAP_SUCCESS)
//         ret = msg;
//     else
//         ret = ~0u;
//
// exit:
//     strfreeU( baseU );
//     strfreeU( filterU );
//     strarrayfreeU( attrsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_search_extA     (WLDAP32.@)
//  *
//  * See ldap_search_extW.
//  */
// ULONG CDECL ldap_search_extA( WLDAP32_LDAP *ld, PCHAR base, ULONG scope,
//     PCHAR filter, PCHAR attrs[], ULONG attrsonly, PLDAPControlA *serverctrls,
//     PLDAPControlA *clientctrls, ULONG timelimit, ULONG sizelimit, ULONG *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *baseW = NULL, *filterW = NULL, **attrsW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x, %p, %p, 0x%08x, 0x%08x, %p)\n",
//            ld, debugstr_a!(base), scope, debugstr_a!(filter), attrs, attrsonly,
//            serverctrls, clientctrls, timelimit, sizelimit, message );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (base) {
//         baseW = strAtoW( base );
//         if (!baseW) goto exit;
//     }
//     if (filter)
//     {
//         filterW = strAtoW( filter );
//         if (!filterW) goto exit;
//     }
//     if (attrs) {
//         attrsW = strarrayAtoW( attrs );
//         if (!attrsW) goto exit;
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
//     ret = ldap_search_extW( ld, baseW, scope, filterW, attrsW, attrsonly,
//                             serverctrlsW, clientctrlsW, timelimit, sizelimit, message );
//
// exit:
//     strfreeW( baseW );
//     strfreeW( filterW );
//     strarrayfreeW( attrsW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_search_extW     (WLDAP32.@)
//  *
//  * Search a directory tree (asynchronous operation).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  base        [I] Starting point for the search.
//  *  scope       [I] Search scope. One of LDAP_SCOPE_BASE,
//  *                  LDAP_SCOPE_ONELEVEL and LDAP_SCOPE_SUBTREE.
//  *  filter      [I] Search filter.
//  *  attrs       [I] Attributes to return.
//  *  attrsonly   [I] Return no values, only attributes.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *  timelimit   [I] Timeout in seconds.
//  *  sizelimit   [I] Maximum number of entries to return. Zero means unlimited.
//  *  message     [O] Message ID of the search operation.
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
// ULONG CDECL ldap_search_extW( WLDAP32_LDAP *ld, PWCHAR base, ULONG scope,
//     PWCHAR filter, PWCHAR attrs[], ULONG attrsonly, PLDAPControlW *serverctrls,
//     PLDAPControlW *clientctrls, ULONG timelimit, ULONG sizelimit, ULONG *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *baseU = NULL, *filterU = NULL, **attrsU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//     struct timeval tv, *tvp = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x, %p, %p, 0x%08x, 0x%08x, %p)\n",
//            ld, debugstr_w!(base), scope, debugstr_w!(filter), attrs, attrsonly,
//            serverctrls, clientctrls, timelimit, sizelimit, message );
//
//     if (!ld) return ~0u;
//
//     if (base) {
//         baseU = strWtoU( base );
//         if (!baseU) goto exit;
//     }
//     if (filter) {
//         filterU = strWtoU( filter );
//         if (!filterU) goto exit;
//     }
//     if (attrs) {
//         attrsU = strarrayWtoU( attrs );
//         if (!attrsU) goto exit;
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
//     if (timelimit)
//     {
//         tv.tv_sec = timelimit;
//         tv.tv_usec = 0;
//         tvp = &tv;
//     }
//
//     ret = map_error( ldap_search_ext( ld, baseU, scope, filterU, attrsU, attrsonly,
//                                       serverctrlsU, clientctrlsU, tvp, sizelimit, (int *)message ));
//
// exit:
//     strfreeU( baseU );
//     strfreeU( filterU );
//     strarrayfreeU( attrsU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_search_ext_sA     (WLDAP32.@)
//  *
//  * See ldap_search_ext_sW.
//  */
// ULONG CDECL ldap_search_ext_sA( WLDAP32_LDAP *ld, PCHAR base, ULONG scope,
//     PCHAR filter, PCHAR attrs[], ULONG attrsonly, PLDAPControlA *serverctrls,
//     PLDAPControlA *clientctrls, struct l_timeval* timeout, ULONG sizelimit, WLDAP32_LDAPMessage **res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *baseW = NULL, *filterW = NULL, **attrsW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x, %p, %p, %p, 0x%08x, %p)\n",
//            ld, debugstr_a!(base), scope, debugstr_a!(filter), attrs, attrsonly,
//            serverctrls, clientctrls, timeout, sizelimit, res );
//
//     if (!ld || !res) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (base) {
//         baseW = strAtoW( base );
//         if (!baseW) goto exit;
//     }
//     if (filter) {
//         filterW = strAtoW( filter );
//         if (!filterW) goto exit;
//     }
//     if (attrs) {
//         attrsW = strarrayAtoW( attrs );
//         if (!attrsW) goto exit;
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
//     ret = ldap_search_ext_sW( ld, baseW, scope, filterW, attrsW, attrsonly,
//                               serverctrlsW, clientctrlsW, timeout, sizelimit, res );
//
// exit:
//     strfreeW( baseW );
//     strfreeW( filterW );
//     strarrayfreeW( attrsW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_search_ext_sW     (WLDAP32.@)
//  *
//  * Search a directory tree (synchronous operation).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  base        [I] Starting point for the search.
//  *  scope       [I] Search scope. One of LDAP_SCOPE_BASE,
//  *                  LDAP_SCOPE_ONELEVEL and LDAP_SCOPE_SUBTREE.
//  *  filter      [I] Search filter.
//  *  attrs       [I] Attributes to return.
//  *  attrsonly   [I] Return no values, only attributes.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *  timeout     [I] Timeout in seconds.
//  *  sizelimit   [I] Maximum number of entries to return. Zero means unlimited.
//  *  res         [O] Results of the search operation.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Call ldap_msgfree to free the results.
//  */
// ULONG CDECL ldap_search_ext_sW( WLDAP32_LDAP *ld, PWCHAR base, ULONG scope,
//     PWCHAR filter, PWCHAR attrs[], ULONG attrsonly, PLDAPControlW *serverctrls,
//     PLDAPControlW *clientctrls, struct l_timeval* timeout, ULONG sizelimit, WLDAP32_LDAPMessage **res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *baseU = NULL, *filterU = NULL, **attrsU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x, %p, %p, %p, 0x%08x, %p)\n",
//            ld, debugstr_w!(base), scope, debugstr_w!(filter), attrs, attrsonly,
//            serverctrls, clientctrls, timeout, sizelimit, res );
//
//     if (!ld || !res) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (base) {
//         baseU = strWtoU( base );
//         if (!baseU) goto exit;
//     }
//     if (filter) {
//         filterU = strWtoU( filter );
//         if (!filterU) goto exit;
//     }
//     if (attrs) {
//         attrsU = strarrayWtoU( attrs );
//         if (!attrsU) goto exit;
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
//     ret = map_error( ldap_search_ext_s( ld, baseU, scope, filterU, attrsU, attrsonly,
//                                         serverctrlsU, clientctrlsU, (struct timeval *)timeout,
//                                         sizelimit, res ));
//
// exit:
//     strfreeU( baseU );
//     strfreeU( filterU );
//     strarrayfreeU( attrsU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_search_sA     (WLDAP32.@)
//  *
//  * See ldap_search_sW.
//  */
// ULONG CDECL ldap_search_sA( WLDAP32_LDAP *ld, PCHAR base, ULONG scope, PCHAR filter,
//     PCHAR attrs[], ULONG attrsonly, WLDAP32_LDAPMessage **res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *baseW = NULL, *filterW = NULL, **attrsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x, %p)\n", ld, debugstr_a!(base),
//            scope, debugstr_a!(filter), attrs, attrsonly, res );
//
//     if (!ld || !res) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (base) {
//         baseW = strAtoW( base );
//         if (!baseW) goto exit;
//     }
//     if (filter) {
//         filterW = strAtoW( filter );
//         if (!filterW) goto exit;
//     }
//     if (attrs) {
//         attrsW = strarrayAtoW( attrs );
//         if (!attrsW) goto exit;
//     }
//
//     ret = ldap_search_sW( ld, baseW, scope, filterW, attrsW, attrsonly, res );
//
// exit:
//     strfreeW( baseW );
//     strfreeW( filterW );
//     strarrayfreeW( attrsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_search_sW     (WLDAP32.@)
//  *
//  * Search a directory tree (synchronous operation).
//  *
//  * PARAMS
//  *  ld        [I] Pointer to an LDAP context.
//  *  base      [I] Starting point for the search.
//  *  scope     [I] Search scope. One of LDAP_SCOPE_BASE,
//  *                LDAP_SCOPE_ONELEVEL and LDAP_SCOPE_SUBTREE.
//  *  filter    [I] Search filter.
//  *  attrs     [I] Attributes to return.
//  *  attrsonly [I] Return no values, only attributes.
//  *  res       [O] Results of the search operation.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Call ldap_msgfree to free the results.
//  */
// ULONG CDECL ldap_search_sW( WLDAP32_LDAP *ld, PWCHAR base, ULONG scope, PWCHAR filter,
//     PWCHAR attrs[], ULONG attrsonly, WLDAP32_LDAPMessage **res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *baseU = NULL, *filterU = NULL, **attrsU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x, %p)\n", ld, debugstr_w!(base),
//            scope, debugstr_w!(filter), attrs, attrsonly, res );
//
//     if (!ld || !res) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (base) {
//         baseU = strWtoU( base );
//         if (!baseU) goto exit;
//     }
//     if (filter) {
//         filterU = strWtoU( filter );
//         if (!filterU) goto exit;
//     }
//     if (attrs) {
//         attrsU = strarrayWtoU( attrs );
//         if (!attrsU) goto exit;
//     }
//
//     ret = map_error( ldap_search_ext_s( ld, baseU, scope, filterU, attrsU, attrsonly,
//                                         NULL, NULL, NULL, 0, res ));
//
// exit:
//     strfreeU( baseU );
//     strfreeU( filterU );
//     strarrayfreeU( attrsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_search_stA     (WLDAP32.@)
//  *
//  * See ldap_search_stW.
//  */
// ULONG CDECL ldap_search_stA( WLDAP32_LDAP *ld, const PCHAR base, ULONG scope,
//     const PCHAR filter, PCHAR attrs[], ULONG attrsonly,
//     struct l_timeval *timeout, WLDAP32_LDAPMessage **res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *baseW = NULL, *filterW = NULL, **attrsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x, %p, %p)\n", ld,
//            debugstr_a!(base), scope, debugstr_a!(filter), attrs,
//            attrsonly, timeout, res );
//
//     if (!ld || !res) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (base) {
//         baseW = strAtoW( base );
//         if (!baseW) goto exit;
//     }
//     if (filter) {
//         filterW = strAtoW( filter );
//         if (!filterW) goto exit;
//     }
//     if (attrs) {
//         attrsW = strarrayAtoW( attrs );
//         if (!attrsW) goto exit;
//     }
//
//     ret = ldap_search_stW( ld, baseW, scope, filterW, attrsW, attrsonly,
//                            timeout, res );
//
// exit:
//     strfreeW( baseW );
//     strfreeW( filterW );
//     strarrayfreeW( attrsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_search_stW     (WLDAP32.@)
//  *
//  * Search a directory tree (synchronous operation).
//  *
//  * PARAMS
//  *  ld        [I] Pointer to an LDAP context.
//  *  base      [I] Starting point for the search.
//  *  scope     [I] Search scope. One of LDAP_SCOPE_BASE,
//  *                LDAP_SCOPE_ONELEVEL and LDAP_SCOPE_SUBTREE.
//  *  filter    [I] Search filter.
//  *  attrs     [I] Attributes to return.
//  *  attrsonly [I] Return no values, only attributes.
//  *  timeout   [I] Timeout in seconds.
//  *  res       [O] Results of the search operation.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Call ldap_msgfree to free the results.
//  */
// ULONG CDECL ldap_search_stW( WLDAP32_LDAP *ld, const PWCHAR base, ULONG scope,
//     const PWCHAR filter, PWCHAR attrs[], ULONG attrsonly,
//     struct l_timeval *timeout, WLDAP32_LDAPMessage **res )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *baseU = NULL, *filterU = NULL, **attrsU = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, 0x%08x, %s, %p, 0x%08x, %p, %p)\n", ld,
//            debugstr_w!(base), scope, debugstr_w!(filter), attrs,
//            attrsonly, timeout, res );
//
//     if (!ld || !res) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (base) {
//         baseU = strWtoU( base );
//         if (!baseU) goto exit;
//     }
//     if (filter) {
//         filterU = strWtoU( filter );
//         if (!filterU) goto exit;
//     }
//     if (attrs) {
//         attrsU = strarrayWtoU( attrs );
//         if (!attrsU) goto exit;
//     }
//
//     ret = map_error( ldap_search_ext_s( ld, baseU, scope, filterU, attrsU, attrsonly,
//                                         NULL, NULL, (struct timeval *)timeout, 0, res ));
//
// exit:
//     strfreeU( baseU );
//     strfreeU( filterU );
//     strarrayfreeU( attrsU );
//
// #endif
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
