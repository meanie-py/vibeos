use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from this preamble.
// The C code only contains preprocessor includes and a Wine debug channel macro,
// neither of which translate to Rust declarations.

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_bindA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    cred: PCHAR,
    method: ULONG,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_bindA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    cred: PCHAR,
    method: ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut credW: PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return !0u32;
    }

    'exit: {
        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }
        if !cred.is_null() {
            credW = strAtoW(cred);
            if credW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_bindW(ld, dnW, credW, method);
    }

    strfreeW(dnW);
    strfreeW(credW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_bindW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    cred: PWCHAR,
    method: ULONG,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_bindW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    cred: PWCHAR,
    method: ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut credU: *mut i8 = core::ptr::null_mut();
    let mut pwd: berval = berval {
        bv_len: 0,
        bv_val: core::ptr::null_mut(),
    };
    let mut msg: i32 = 0;

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return !0u32;
    }
    if method != LDAP_AUTH_SIMPLE {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }
        if !cred.is_null() {
            credU = strWtoU(cred);
            if credU.is_null() {
                break 'exit;
            }

            pwd.bv_len = strlen(credU);
            pwd.bv_val = credU;
        }

        ret = ldap_sasl_bind(
            ld,
            dnU,
            LDAP_SASL_SIMPLE,
            &mut pwd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut msg,
        );

        if ret == LDAP_SUCCESS {
            ret = msg as ULONG;
        } else {
            ret = !0u32;
        }
    }

    strfreeU(dnU);
    strfreeU(credU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_bind_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    cred: PCHAR,
    method: ULONG,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_bind_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    cred: PCHAR,
    method: ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut credW: PWCHAR = core::ptr::null_mut();

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
        if !cred.is_null() {
            credW = strAtoW(cred);
            if credW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_bind_sW(ld, dnW, credW, method);
    }

    strfreeW(dnW);
    strfreeW(credW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_bind_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    cred: PWCHAR,
    method: ULONG,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_bind_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    cred: PWCHAR,
    method: ULONG,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut credU: *mut i8 = core::ptr::null_mut();
    let mut pwd: berval = berval {
        bv_len: 0,
        bv_val: core::ptr::null_mut(),
    };

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }
        if method != LDAP_AUTH_SIMPLE {
            ret = WLDAP32_LDAP_PARAM_ERROR;
            break 'exit;
        }

        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }
        if !cred.is_null() {
            credU = strWtoU(cred);
            if credU.is_null() {
                break 'exit;
            }

            pwd.bv_len = strlen(credU);
            pwd.bv_val = credU;
        }

        ret = map_error(ldap_sasl_bind_s(
            ld,
            dnU,
            LDAP_SASL_SIMPLE,
            &mut pwd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ));
    }

    strfreeU(dnU);
    strfreeU(credU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_sasl_bindA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    mechanism: PCHAR,
    cred: *const BERVAL,
    serverctrls: *mut *mut LDAPControlA,
    clientctrls: *mut *mut LDAPControlA,
    message: *mut i32,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_sasl_bindA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    mechanism: PCHAR,
    cred: *const BERVAL,
    serverctrls: *mut *mut LDAPControlA,
    clientctrls: *mut *mut LDAPControlA,
    message: *mut i32,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    #[cfg(feature = "have_ldap")]
    {
        let mut dnW: *mut u16;
        let mut mechanismW: *mut u16 = core::ptr::null_mut();
        let mut serverctrlsW: *mut *mut LDAPControlW = core::ptr::null_mut();
        let mut clientctrlsW: *mut *mut LDAPControlW = core::ptr::null_mut();

        ret = WLDAP32_LDAP_NO_MEMORY;

        if ld.is_null() || dn.is_null() || mechanism.is_null() || cred.is_null() || message.is_null() {
            return WLDAP32_LDAP_PARAM_ERROR;
        }

        'exit: {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }

            mechanismW = strAtoW(mechanism);
            if mechanismW.is_null() {
                break 'exit;
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

            ret = ldap_sasl_bindW(ld, dnW, mechanismW, cred, serverctrlsW, clientctrlsW, message);
        }

        strfreeW(dnW);
        strfreeW(mechanismW);
        controlarrayfreeW(serverctrlsW);
        controlarrayfreeW(clientctrlsW);
    }

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_sasl_bindW(
    ld: *mut WLDAP32_LDAP,
    dn: *const u16,
    mechanism: *const u16,
    cred: *const BERVAL,
    serverctrls: *mut *mut LDAPControlW,
    clientctrls: *mut *mut LDAPControlW,
    message: *mut i32,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_sasl_bindW(
    ld: *mut WLDAP32_LDAP,
    dn: *const u16,
    mechanism: *const u16,
    cred: *const BERVAL,
    serverctrls: *mut *mut LDAPControlW,
    clientctrls: *mut *mut LDAPControlW,
    message: *mut i32,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    #[cfg(feature = "have_ldap")]
    {
        let mut dnU: *mut i8;
        let mut mechanismU: *mut i8 = core::ptr::null_mut();
        let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
        let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
        let mut credU: berval = core::mem::zeroed();

        ret = WLDAP32_LDAP_NO_MEMORY;

        if ld.is_null() || dn.is_null() || mechanism.is_null() || cred.is_null() || message.is_null() {
            return WLDAP32_LDAP_PARAM_ERROR;
        }

        'exit: loop {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }

            mechanismU = strWtoU(mechanism);
            if mechanismU.is_null() {
                break 'exit;
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

            credU.bv_len = (*cred).bv_len;
            credU.bv_val = (*cred).bv_val;

            ret = map_error(ldap_sasl_bind(
                ld,
                dnU,
                mechanismU,
                &mut credU,
                serverctrlsU,
                clientctrlsU,
                message,
            ));

            break 'exit;
        }

        strfreeU(dnU);
        strfreeU(mechanismU);
        controlarrayfreeU(serverctrlsU);
        controlarrayfreeU(clientctrlsU);
    }

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_sasl_bind_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    mechanism: PCHAR,
    cred: *const BERVAL,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    serverdata: *mut PBERVAL,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_sasl_bind_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    mechanism: PCHAR,
    cred: *const BERVAL,
    serverctrls: *mut PLDAPControlA,
    clientctrls: *mut PLDAPControlA,
    serverdata: *mut PBERVAL,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR;
    let mut mechanismW: PWCHAR = core::ptr::null_mut();
    let mut serverctrlsW: *mut PLDAPControlW = core::ptr::null_mut();
    let mut clientctrlsW: *mut PLDAPControlW = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || dn.is_null() || mechanism.is_null() || cred.is_null() || serverdata.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: {
        dnW = strAtoW(dn);
        if dnW.is_null() {
            break 'exit;
        }

        mechanismW = strAtoW(mechanism);
        if mechanismW.is_null() {
            break 'exit;
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

        ret = ldap_sasl_bind_sW(ld, dnW, mechanismW, cred, serverctrlsW, clientctrlsW, serverdata);
    }

    strfreeW(dnW);
    strfreeW(mechanismW);
    controlarrayfreeW(serverctrlsW);
    controlarrayfreeW(clientctrlsW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_sasl_bind_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    mechanism: PWCHAR,
    cred: *const BERVAL,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    serverdata: *mut PBERVAL,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_sasl_bind_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    mechanism: PWCHAR,
    cred: *const BERVAL,
    serverctrls: *mut PLDAPControlW,
    clientctrls: *mut PLDAPControlW,
    serverdata: *mut PBERVAL,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut dnU: PCHAR;
    let mut mechanismU: PCHAR = core::ptr::null_mut();
    let mut serverctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut clientctrlsU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut credU: berval = core::mem::zeroed();

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() || dn.is_null() || mechanism.is_null() || cred.is_null() || serverdata.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    dnU = strWtoU(dn);
    if dnU.is_null() {
        strfreeU(dnU);
        strfreeU(mechanismU);
        controlarrayfreeU(serverctrlsU);
        controlarrayfreeU(clientctrlsU);
        return ret;
    }

    mechanismU = strWtoU(mechanism);
    if mechanismU.is_null() {
        strfreeU(dnU);
        strfreeU(mechanismU);
        controlarrayfreeU(serverctrlsU);
        controlarrayfreeU(clientctrlsU);
        return ret;
    }

    if !serverctrls.is_null() {
        serverctrlsU = controlarrayWtoU(serverctrls);
        if serverctrlsU.is_null() {
            strfreeU(dnU);
            strfreeU(mechanismU);
            controlarrayfreeU(serverctrlsU);
            controlarrayfreeU(clientctrlsU);
            return ret;
        }
    }

    if !clientctrls.is_null() {
        clientctrlsU = controlarrayWtoU(clientctrls);
        if clientctrlsU.is_null() {
            strfreeU(dnU);
            strfreeU(mechanismU);
            controlarrayfreeU(serverctrlsU);
            controlarrayfreeU(clientctrlsU);
            return ret;
        }
    }

    credU.bv_len = (*cred).bv_len;
    credU.bv_val = (*cred).bv_val;

    ret = map_error(ldap_sasl_bind_s(
        ld,
        dnU,
        mechanismU,
        &mut credU,
        serverctrlsU,
        clientctrlsU,
        serverdata as *mut *mut berval,
    ));

    strfreeU(dnU);
    strfreeU(mechanismU);
    controlarrayfreeU(serverctrlsU);
    controlarrayfreeU(clientctrlsU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_simple_bindA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    passwd: PCHAR,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_simple_bindA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    passwd: PCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut passwdW: PWCHAR = core::ptr::null_mut();

    ret = WLDAP32_LDAP_NO_MEMORY;

    'exit: {
        if ld.is_null() {
            return !0u32;
        }

        if !dn.is_null() {
            dnW = strAtoW(dn);
            if dnW.is_null() {
                break 'exit;
            }
        }
        if !passwd.is_null() {
            passwdW = strAtoW(passwd);
            if passwdW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_simple_bindW(ld, dnW, passwdW);
    }

    strfreeW(dnW);
    strfreeW(passwdW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_simple_bindW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    passwd: PWCHAR,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_simple_bindW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    passwd: PWCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut passwdU: *mut i8 = core::ptr::null_mut();
    let mut pwd: berval = berval {
        bv_len: 0,
        bv_val: core::ptr::null_mut(),
    };
    let mut msg: i32 = 0;

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return !0u32;
    }

    'exit: {
        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }
        if !passwd.is_null() {
            passwdU = strWtoU(passwd);
            if passwdU.is_null() {
                break 'exit;
            }

            pwd.bv_len = strlen(passwdU);
            pwd.bv_val = passwdU;
        }

        ret = ldap_sasl_bind(
            ld,
            dnU,
            LDAP_SASL_SIMPLE,
            &mut pwd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut msg,
        );

        if ret == LDAP_SUCCESS {
            ret = msg as ULONG;
        } else {
            ret = !0u32;
        }
    }

    strfreeU(dnU);
    strfreeU(passwdU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_simple_bind_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    passwd: PCHAR,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_simple_bind_sA(
    ld: *mut WLDAP32_LDAP,
    dn: PCHAR,
    passwd: PCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnW: PWCHAR = core::ptr::null_mut();
    let mut passwdW: PWCHAR = core::ptr::null_mut();

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
        if !passwd.is_null() {
            passwdW = strAtoW(passwd);
            if passwdW.is_null() {
                break 'exit;
            }
        }

        ret = ldap_simple_bind_sW(ld, dnW, passwdW);
    }

    strfreeW(dnW);
    strfreeW(passwdW);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn ldap_simple_bind_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    passwd: PWCHAR,
) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn ldap_simple_bind_sW(
    ld: *mut WLDAP32_LDAP,
    dn: PWCHAR,
    passwd: PWCHAR,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    let mut dnU: *mut i8 = core::ptr::null_mut();
    let mut passwdU: *mut i8 = core::ptr::null_mut();
    let mut pwd: berval = berval {
        bv_len: 0,
        bv_val: core::ptr::null_mut(),
    };

    ret = WLDAP32_LDAP_NO_MEMORY;

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    'exit: loop {
        if !dn.is_null() {
            dnU = strWtoU(dn);
            if dnU.is_null() {
                break 'exit;
            }
        }
        if !passwd.is_null() {
            passwdU = strWtoU(passwd);
            if passwdU.is_null() {
                break 'exit;
            }

            pwd.bv_len = strlen(passwdU);
            pwd.bv_val = passwdU;
        }

        ret = map_error(ldap_sasl_bind_s(
            ld,
            dnU,
            LDAP_SASL_SIMPLE,
            &mut pwd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ));

        break 'exit;
    }

    strfreeU(dnU);
    strfreeU(passwdU);

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ldap_unbind(ld: *mut WLDAP32_LDAP) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_unbind(ld: *mut WLDAP32_LDAP) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if !ld.is_null() {
        ret = map_error(ldap_unbind_ext(ld, core::ptr::null_mut(), core::ptr::null_mut()));
    } else {
        ret = WLDAP32_LDAP_PARAM_ERROR;
    }

    ret
}

#[no_mangle]
#[cfg(not(feature = "have_ldap"))]
pub unsafe extern "C" fn WLDAP32_ldap_unbind_s(ld: *mut WLDAP32_LDAP) -> ULONG {
    WLDAP32_LDAP_NOT_SUPPORTED
}

#[cfg(feature = "have_ldap")]
#[no_mangle]
pub unsafe extern "C" fn WLDAP32_ldap_unbind_s(ld: *mut WLDAP32_LDAP) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    if !ld.is_null() {
        ret = map_error(ldap_unbind_ext_s(ld, core::ptr::null_mut(), core::ptr::null_mut()));
    } else {
        ret = WLDAP32_LDAP_PARAM_ERROR;
    }
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
//  *      ldap_bindA     (WLDAP32.@)
//  *
//  * See ldap_bindW.
//  */
// ULONG CDECL ldap_bindA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR cred, ULONG method )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *credW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, 0x%08x)\n", ld, debugstr_a!(dn), cred, method );
//
//     if (!ld) return ~0u;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (cred) {
//         credW = strAtoW( cred );
//         if (!credW) goto exit;
//     }
//
//     ret = ldap_bindW( ld, dnW, credW, method );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( credW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_bindW     (WLDAP32.@)
//  *
//  * Authenticate with an LDAP server (asynchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of entry to bind as.
//  *  cred    [I] Credentials (e.g. password string).
//  *  method  [I] Authentication method.
//  *
//  * RETURNS
//  *  Success: Message ID of the bind operation.
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Only LDAP_AUTH_SIMPLE is supported (just like native).
//  */
// ULONG CDECL ldap_bindW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR cred, ULONG method )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *credU = NULL;
//     struct berval pwd = { 0, NULL };
//     int msg;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, 0x%08x)\n", ld, debugstr_w!(dn), cred, method );
//
//     if (!ld) return ~0u;
//     if (method != LDAP_AUTH_SIMPLE) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//     if (cred) {
//         credU = strWtoU( cred );
//         if (!credU) goto exit;
//
//         pwd.bv_len = strlen( credU );
//         pwd.bv_val = credU;
//     }
//
//     ret = ldap_sasl_bind( ld, dnU, LDAP_SASL_SIMPLE, &pwd, NULL, NULL, &msg );
//
//     if (ret == LDAP_SUCCESS)
//         ret = msg;
//     else
//         ret = ~0u;
//
// exit:
//     strfreeU( dnU );
//     strfreeU( credU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_bind_sA     (WLDAP32.@)
//  *
//  * See ldap_bind_sW.
//  */
// ULONG CDECL ldap_bind_sA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR cred, ULONG method )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *credW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, 0x%08x)\n", ld, debugstr_a!(dn), cred, method );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (cred) {
//         credW = strAtoW( cred );
//         if (!credW) goto exit;
//     }
//
//     ret = ldap_bind_sW( ld, dnW, credW, method );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( credW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_bind_sW     (WLDAP32.@)
//  *
//  * Authenticate with an LDAP server (synchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of entry to bind as.
//  *  cred    [I] Credentials (e.g. password string).
//  *  method  [I] Authentication method.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_bind_sW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR cred, ULONG method )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *credU = NULL;
//     struct berval pwd = { 0, NULL };
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p, 0x%08x)\n", ld, debugstr_w!(dn), cred, method );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//     if (method != LDAP_AUTH_SIMPLE) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//     if (cred) {
//         credU = strWtoU( cred );
//         if (!credU) goto exit;
//
//         pwd.bv_len = strlen( credU );
//         pwd.bv_val = credU;
//     }
//
//     ret = map_error( ldap_sasl_bind_s( ld, dnU, LDAP_SASL_SIMPLE, &pwd, NULL, NULL, NULL ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( credU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_sasl_bindA     (WLDAP32.@)
//  *
//  * See ldap_sasl_bindW.
//  */
// ULONG CDECL ldap_sasl_bindA( WLDAP32_LDAP *ld, const PCHAR dn,
//     const PCHAR mechanism, const BERVAL *cred, PLDAPControlA *serverctrls,
//     PLDAPControlA *clientctrls, int *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW, *mechanismW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %p, %p, %p, %p)\n", ld, debugstr_a!(dn),
//            debugstr_a!(mechanism), cred, serverctrls, clientctrls, message );
//
//     if (!ld || !dn || !mechanism || !cred || !message)
//         return WLDAP32_LDAP_PARAM_ERROR;
//
//     dnW = strAtoW( dn );
//     if (!dnW) goto exit;
//
//     mechanismW = strAtoW( mechanism );
//     if (!mechanismW) goto exit;
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
//     ret = ldap_sasl_bindW( ld, dnW, mechanismW, cred, serverctrlsW, clientctrlsW, message );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( mechanismW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_sasl_bindW     (WLDAP32.@)
//  *
//  * Authenticate with an LDAP server using SASL (asynchronous operation).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  dn          [I] DN of entry to bind as.
//  *  mechanism   [I] Authentication method.
//  *  cred        [I] Credentials.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *  message     [O] Message ID of the bind operation.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  The serverctrls and clientctrls parameters are optional and should
//  *  be set to NULL if not used.
//  */
// ULONG CDECL ldap_sasl_bindW( WLDAP32_LDAP *ld, const PWCHAR dn,
//     const PWCHAR mechanism, const BERVAL *cred, PLDAPControlW *serverctrls,
//     PLDAPControlW *clientctrls, int *message )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU, *mechanismU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//     struct berval credU;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %p, %p, %p, %p)\n", ld, debugstr_w!(dn),
//            debugstr_w!(mechanism), cred, serverctrls, clientctrls, message );
//
//     if (!ld || !dn || !mechanism || !cred || !message)
//         return WLDAP32_LDAP_PARAM_ERROR;
//
//     dnU = strWtoU( dn );
//     if (!dnU) goto exit;
//
//     mechanismU = strWtoU( mechanism );
//     if (!mechanismU) goto exit;
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
//     credU.bv_len = cred->bv_len;
//     credU.bv_val = cred->bv_val;
//
//     ret = map_error( ldap_sasl_bind( ld, dnU, mechanismU, &credU,
//                                      serverctrlsU, clientctrlsU, message ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( mechanismU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_sasl_bind_sA     (WLDAP32.@)
//  *
//  * See ldap_sasl_bind_sW.
//  */
// ULONG CDECL ldap_sasl_bind_sA( WLDAP32_LDAP *ld, const PCHAR dn,
//     const PCHAR mechanism, const BERVAL *cred, PLDAPControlA *serverctrls,
//     PLDAPControlA *clientctrls, PBERVAL *serverdata )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW, *mechanismW = NULL;
//     LDAPControlW **serverctrlsW = NULL, **clientctrlsW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %p, %p, %p, %p)\n", ld, debugstr_a!(dn),
//            debugstr_a!(mechanism), cred, serverctrls, clientctrls, serverdata );
//
//     if (!ld || !dn || !mechanism || !cred || !serverdata)
//         return WLDAP32_LDAP_PARAM_ERROR;
//
//     dnW = strAtoW( dn );
//     if (!dnW) goto exit;
//
//     mechanismW = strAtoW( mechanism );
//     if (!mechanismW) goto exit;
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
//     ret = ldap_sasl_bind_sW( ld, dnW, mechanismW, cred, serverctrlsW, clientctrlsW, serverdata );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( mechanismW );
//     controlarrayfreeW( serverctrlsW );
//     controlarrayfreeW( clientctrlsW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_sasl_bind_sW     (WLDAP32.@)
//  *
//  * Authenticate with an LDAP server using SASL (synchronous operation).
//  *
//  * PARAMS
//  *  ld          [I] Pointer to an LDAP context.
//  *  dn          [I] DN of entry to bind as.
//  *  mechanism   [I] Authentication method.
//  *  cred        [I] Credentials.
//  *  serverctrls [I] Array of LDAP server controls.
//  *  clientctrls [I] Array of LDAP client controls.
//  *  serverdata  [O] Authentication response from the server.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  The serverctrls and clientctrls parameters are optional and should
//  *  be set to NULL if not used.
//  */
// ULONG CDECL ldap_sasl_bind_sW( WLDAP32_LDAP *ld, const PWCHAR dn,
//     const PWCHAR mechanism, const BERVAL *cred, PLDAPControlW *serverctrls,
//     PLDAPControlW *clientctrls, PBERVAL *serverdata )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU, *mechanismU = NULL;
//     LDAPControl **serverctrlsU = NULL, **clientctrlsU = NULL;
//     struct berval credU;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %s, %p, %p, %p, %p)\n", ld, debugstr_w!(dn),
//            debugstr_w!(mechanism), cred, serverctrls, clientctrls, serverdata );
//
//     if (!ld || !dn || !mechanism || !cred || !serverdata)
//         return WLDAP32_LDAP_PARAM_ERROR;
//
//     dnU = strWtoU( dn );
//     if (!dnU) goto exit;
//
//     mechanismU = strWtoU( mechanism );
//     if (!mechanismU) goto exit;
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
//     credU.bv_len = cred->bv_len;
//     credU.bv_val = cred->bv_val;
//
//     ret = map_error( ldap_sasl_bind_s( ld, dnU, mechanismU, &credU,
//                                        serverctrlsU, clientctrlsU, (struct berval **)serverdata ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( mechanismU );
//     controlarrayfreeU( serverctrlsU );
//     controlarrayfreeU( clientctrlsU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_simple_bindA     (WLDAP32.@)
//  *
//  * See ldap_simple_bindW.
//  */
// ULONG CDECL ldap_simple_bindA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR passwd )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *passwdW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p)\n", ld, debugstr_a!(dn), passwd );
//
//     if (!ld) return ~0u;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (passwd) {
//         passwdW = strAtoW( passwd );
//         if (!passwdW) goto exit;
//     }
//
//     ret = ldap_simple_bindW( ld, dnW, passwdW );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( passwdW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_simple_bindW     (WLDAP32.@)
//  *
//  * Authenticate with an LDAP server (asynchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of entry to bind as.
//  *  passwd  [I] Password string.
//  *
//  * RETURNS
//  *  Success: Message ID of the bind operation.
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Set dn and passwd to NULL to bind as an anonymous user.
//  */
// ULONG CDECL ldap_simple_bindW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR passwd )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *passwdU = NULL;
//     struct berval pwd = { 0, NULL };
//     int msg;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p)\n", ld, debugstr_w!(dn), passwd );
//
//     if (!ld) return ~0u;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//     if (passwd) {
//         passwdU = strWtoU( passwd );
//         if (!passwdU) goto exit;
//
//         pwd.bv_len = strlen( passwdU );
//         pwd.bv_val = passwdU;
//     }
//
//     ret = ldap_sasl_bind( ld, dnU, LDAP_SASL_SIMPLE, &pwd, NULL, NULL, &msg );
//
//     if (ret == LDAP_SUCCESS)
//         ret = msg;
//     else
//         ret = ~0u;
//
// exit:
//     strfreeU( dnU );
//     strfreeU( passwdU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_simple_bind_sA     (WLDAP32.@)
//  *
//  * See ldap_simple_bind_sW.
//  */
// ULONG CDECL ldap_simple_bind_sA( WLDAP32_LDAP *ld, PCHAR dn, PCHAR passwd )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     WCHAR *dnW = NULL, *passwdW = NULL;
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p)\n", ld, debugstr_a!(dn), passwd );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnW = strAtoW( dn );
//         if (!dnW) goto exit;
//     }
//     if (passwd) {
//         passwdW = strAtoW( passwd );
//         if (!passwdW) goto exit;
//     }
//
//     ret = ldap_simple_bind_sW( ld, dnW, passwdW );
//
// exit:
//     strfreeW( dnW );
//     strfreeW( passwdW );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_simple_bind_sW     (WLDAP32.@)
//  *
//  * Authenticate with an LDAP server (synchronous operation).
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  dn      [I] DN of entry to bind as.
//  *  passwd  [I] Password string.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Set dn and passwd to NULL to bind as an anonymous user.
//  */
// ULONG CDECL ldap_simple_bind_sW( WLDAP32_LDAP *ld, PWCHAR dn, PWCHAR passwd )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//     char *dnU = NULL, *passwdU = NULL;
//     struct berval pwd = { 0, NULL };
//
//     ret = WLDAP32_LDAP_NO_MEMORY;
//
//     TRACE( "(%p, %s, %p)\n", ld, debugstr_w!(dn), passwd );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     if (dn) {
//         dnU = strWtoU( dn );
//         if (!dnU) goto exit;
//     }
//     if (passwd) {
//         passwdU = strWtoU( passwd );
//         if (!passwdU) goto exit;
//
//         pwd.bv_len = strlen( passwdU );
//         pwd.bv_val = passwdU;
//     }
//
//     ret = map_error( ldap_sasl_bind_s( ld, dnU, LDAP_SASL_SIMPLE, &pwd, NULL, NULL, NULL ));
//
// exit:
//     strfreeU( dnU );
//     strfreeU( passwdU );
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_unbind     (WLDAP32.@)
//  *
//  * Close LDAP connection and free resources (asynchronous operation).
//  *
//  * PARAMS
//  *  ld  [I] Pointer to an LDAP context.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL WLDAP32_ldap_unbind( WLDAP32_LDAP *ld )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p)\n", ld );
//
//     if (ld)
//         ret = map_error( ldap_unbind_ext( ld, NULL, NULL ));
//     else
//         ret = WLDAP32_LDAP_PARAM_ERROR;
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_unbind_s     (WLDAP32.@)
//  *
//  * Close LDAP connection and free resources (synchronous operation).
//  *
//  * PARAMS
//  *  ld  [I] Pointer to an LDAP context.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL WLDAP32_ldap_unbind_s( WLDAP32_LDAP *ld )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p)\n", ld );
//
//     if (ld)
//         ret = map_error( ldap_unbind_ext_s( ld, NULL, NULL ));
//     else
//         ret = WLDAP32_LDAP_PARAM_ERROR;
//
// #endif
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
