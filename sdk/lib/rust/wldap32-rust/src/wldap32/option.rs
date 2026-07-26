use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the C preamble.
// The C code only contains conditional includes and a debug channel macro,
// which do not generate Rust declarations.

#[no_mangle]
pub unsafe extern "C" fn ldap_get_optionA(
    ld: *mut WLDAP32_LDAP,
    option: i32,
    value: *mut core::ffi::c_void,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if ld.is_null() || value.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    match option {
        WLDAP32_LDAP_OPT_API_FEATURE_INFO => {
            let mut featureW: LDAPAPIFeatureInfoW = core::mem::zeroed();
            let featureA = value as *mut LDAPAPIFeatureInfoA;

            if (*featureA).ldapaif_name.is_null() {
                return WLDAP32_LDAP_PARAM_ERROR;
            }

            featureW.ldapaif_info_version = (*featureA).ldapaif_info_version;
            featureW.ldapaif_name = strAtoW((*featureA).ldapaif_name);
            featureW.ldapaif_version = 0;

            if featureW.ldapaif_name.is_null() {
                return WLDAP32_LDAP_NO_MEMORY;
            }

            ret = ldap_get_optionW(ld, option, &mut featureW as *mut _ as *mut core::ffi::c_void);

            (*featureA).ldapaif_version = featureW.ldapaif_version;
            strfreeW(featureW.ldapaif_name);
            return ret;
        }
        WLDAP32_LDAP_OPT_API_INFO => {
            let mut infoW: LDAPAPIInfoW = core::mem::zeroed();
            let infoA = value as *mut LDAPAPIInfoA;

            memset(&mut infoW as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<LDAPAPIInfoW>());
            infoW.ldapai_info_version = (*infoA).ldapai_info_version;

            ret = ldap_get_optionW(ld, option, &mut infoW as *mut _ as *mut core::ffi::c_void);

            (*infoA).ldapai_api_version = infoW.ldapai_api_version;
            (*infoA).ldapai_protocol_version = infoW.ldapai_protocol_version;

            if !infoW.ldapai_extensions.is_null() {
                (*infoA).ldapai_extensions = strarrayWtoA(infoW.ldapai_extensions);
                if (*infoA).ldapai_extensions.is_null() {
                    return WLDAP32_LDAP_NO_MEMORY;
                }
            }
            if !infoW.ldapai_vendor_name.is_null() {
                (*infoA).ldapai_vendor_name = strWtoA(infoW.ldapai_vendor_name);
                if (*infoA).ldapai_vendor_name.is_null() {
                    ldap_value_freeW(infoW.ldapai_extensions);
                    return WLDAP32_LDAP_NO_MEMORY;
                }
            }
            (*infoA).ldapai_vendor_version = infoW.ldapai_vendor_version;

            ldap_value_freeW(infoW.ldapai_extensions);
            ldap_memfreeW(infoW.ldapai_vendor_name);
            return ret;
        }

        WLDAP32_LDAP_OPT_DEREF
        | WLDAP32_LDAP_OPT_DESC
        | WLDAP32_LDAP_OPT_ERROR_NUMBER
        | WLDAP32_LDAP_OPT_PROTOCOL_VERSION
        | WLDAP32_LDAP_OPT_REFERRALS
        | WLDAP32_LDAP_OPT_SIZELIMIT
        | WLDAP32_LDAP_OPT_TIMELIMIT => {
            return ldap_get_optionW(ld, option, value);
        }

        WLDAP32_LDAP_OPT_CACHE_ENABLE
        | WLDAP32_LDAP_OPT_CACHE_FN_PTRS
        | WLDAP32_LDAP_OPT_CACHE_STRATEGY
        | WLDAP32_LDAP_OPT_IO_FN_PTRS
        | WLDAP32_LDAP_OPT_REBIND_ARG
        | WLDAP32_LDAP_OPT_REBIND_FN
        | WLDAP32_LDAP_OPT_RESTART
        | WLDAP32_LDAP_OPT_THREAD_FN_PTRS => {
            return LDAP_LOCAL_ERROR;
        }

        WLDAP32_LDAP_OPT_AREC_EXCLUSIVE
        | WLDAP32_LDAP_OPT_AUTO_RECONNECT
        | WLDAP32_LDAP_OPT_CLIENT_CERTIFICATE
        | WLDAP32_LDAP_OPT_DNSDOMAIN_NAME
        | WLDAP32_LDAP_OPT_ENCRYPT
        | WLDAP32_LDAP_OPT_ERROR_STRING
        | WLDAP32_LDAP_OPT_FAST_CONCURRENT_BIND
        | WLDAP32_LDAP_OPT_GETDSNAME_FLAGS
        | WLDAP32_LDAP_OPT_HOST_NAME
        | WLDAP32_LDAP_OPT_HOST_REACHABLE
        | WLDAP32_LDAP_OPT_PING_KEEP_ALIVE
        | WLDAP32_LDAP_OPT_PING_LIMIT
        | WLDAP32_LDAP_OPT_PING_WAIT_TIME
        | WLDAP32_LDAP_OPT_PROMPT_CREDENTIALS
        | WLDAP32_LDAP_OPT_REF_DEREF_CONN_PER_MSG
        | WLDAP32_LDAP_OPT_REFERRAL_CALLBACK
        | WLDAP32_LDAP_OPT_REFERRAL_HOP_LIMIT
        | WLDAP32_LDAP_OPT_ROOTDSE_CACHE
        | WLDAP32_LDAP_OPT_SASL_METHOD
        | WLDAP32_LDAP_OPT_SECURITY_CONTEXT
        | WLDAP32_LDAP_OPT_SEND_TIMEOUT
        | WLDAP32_LDAP_OPT_SERVER_CERTIFICATE
        | WLDAP32_LDAP_OPT_SERVER_CONTROLS
        | WLDAP32_LDAP_OPT_SERVER_ERROR
        | WLDAP32_LDAP_OPT_SERVER_EXT_ERROR
        | WLDAP32_LDAP_OPT_SIGN
        | WLDAP32_LDAP_OPT_SSL
        | WLDAP32_LDAP_OPT_SSL_INFO
        | WLDAP32_LDAP_OPT_SSPI_FLAGS
        | WLDAP32_LDAP_OPT_TCP_KEEPALIVE => {
            return WLDAP32_LDAP_NOT_SUPPORTED;
        }

        _ => {
            return WLDAP32_LDAP_LOCAL_ERROR;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_get_optionW(
    ld: *mut WLDAP32_LDAP,
    option: i32,
    value: *mut core::ffi::c_void,
) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if ld.is_null() || value.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    match option {
        WLDAP32_LDAP_OPT_API_FEATURE_INFO => {
            let mut featureU: LDAPAPIFeatureInfo = core::mem::zeroed();
            let featureW = value as *mut LDAPAPIFeatureInfoW;

            if (*featureW).ldapaif_name.is_null() {
                return WLDAP32_LDAP_PARAM_ERROR;
            }

            featureU.ldapaif_info_version = (*featureW).ldapaif_info_version;
            featureU.ldapaif_name = strWtoU((*featureW).ldapaif_name);
            featureU.ldapaif_version = 0;

            if featureU.ldapaif_name.is_null() {
                return WLDAP32_LDAP_NO_MEMORY;
            }

            ret = map_error(ldap_get_option(
                ld,
                option,
                &mut featureU as *mut _ as *mut core::ffi::c_void,
            ));

            (*featureW).ldapaif_version = featureU.ldapaif_version;
            strfreeU(featureU.ldapaif_name);
            return ret;
        }
        WLDAP32_LDAP_OPT_API_INFO => {
            let mut infoU: LDAPAPIInfo = core::mem::zeroed();
            let infoW = value as *mut LDAPAPIInfoW;

            memset(
                &mut infoU as *mut _ as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<LDAPAPIInfo>(),
            );
            infoU.ldapai_info_version = (*infoW).ldapai_info_version;

            ret = map_error(ldap_get_option(
                ld,
                option,
                &mut infoU as *mut _ as *mut core::ffi::c_void,
            ));

            (*infoW).ldapai_api_version = infoU.ldapai_api_version;
            (*infoW).ldapai_protocol_version = infoU.ldapai_protocol_version;

            if !infoU.ldapai_extensions.is_null() {
                (*infoW).ldapai_extensions = strarrayUtoW(infoU.ldapai_extensions);
                if (*infoW).ldapai_extensions.is_null() {
                    return WLDAP32_LDAP_NO_MEMORY;
                }
            }
            if !infoU.ldapai_vendor_name.is_null() {
                (*infoW).ldapai_vendor_name = strUtoW(infoU.ldapai_vendor_name);
                if (*infoW).ldapai_vendor_name.is_null() {
                    ldap_memvfree(infoU.ldapai_extensions as *mut *mut core::ffi::c_void);
                    return WLDAP32_LDAP_NO_MEMORY;
                }
            }
            (*infoW).ldapai_vendor_version = infoU.ldapai_vendor_version;

            ldap_memvfree(infoU.ldapai_extensions as *mut *mut core::ffi::c_void);
            ldap_memfree(infoU.ldapai_vendor_name);
            return ret;
        }

        WLDAP32_LDAP_OPT_DEREF
        | WLDAP32_LDAP_OPT_DESC
        | WLDAP32_LDAP_OPT_ERROR_NUMBER
        | WLDAP32_LDAP_OPT_PROTOCOL_VERSION
        | WLDAP32_LDAP_OPT_REFERRALS
        | WLDAP32_LDAP_OPT_SIZELIMIT
        | WLDAP32_LDAP_OPT_TIMELIMIT => {
            return map_error(ldap_get_option(ld, option, value));
        }

        WLDAP32_LDAP_OPT_CACHE_ENABLE
        | WLDAP32_LDAP_OPT_CACHE_FN_PTRS
        | WLDAP32_LDAP_OPT_CACHE_STRATEGY
        | WLDAP32_LDAP_OPT_IO_FN_PTRS
        | WLDAP32_LDAP_OPT_REBIND_ARG
        | WLDAP32_LDAP_OPT_REBIND_FN
        | WLDAP32_LDAP_OPT_RESTART
        | WLDAP32_LDAP_OPT_THREAD_FN_PTRS => {
            return WLDAP32_LDAP_LOCAL_ERROR;
        }

        WLDAP32_LDAP_OPT_AREC_EXCLUSIVE
        | WLDAP32_LDAP_OPT_AUTO_RECONNECT
        | WLDAP32_LDAP_OPT_CLIENT_CERTIFICATE
        | WLDAP32_LDAP_OPT_DNSDOMAIN_NAME
        | WLDAP32_LDAP_OPT_ENCRYPT
        | WLDAP32_LDAP_OPT_ERROR_STRING
        | WLDAP32_LDAP_OPT_FAST_CONCURRENT_BIND
        | WLDAP32_LDAP_OPT_GETDSNAME_FLAGS
        | WLDAP32_LDAP_OPT_HOST_NAME
        | WLDAP32_LDAP_OPT_HOST_REACHABLE
        | WLDAP32_LDAP_OPT_PING_KEEP_ALIVE
        | WLDAP32_LDAP_OPT_PING_LIMIT
        | WLDAP32_LDAP_OPT_PING_WAIT_TIME
        | WLDAP32_LDAP_OPT_PROMPT_CREDENTIALS
        | WLDAP32_LDAP_OPT_REF_DEREF_CONN_PER_MSG
        | WLDAP32_LDAP_OPT_REFERRAL_CALLBACK
        | WLDAP32_LDAP_OPT_REFERRAL_HOP_LIMIT
        | WLDAP32_LDAP_OPT_ROOTDSE_CACHE
        | WLDAP32_LDAP_OPT_SASL_METHOD
        | WLDAP32_LDAP_OPT_SECURITY_CONTEXT
        | WLDAP32_LDAP_OPT_SEND_TIMEOUT
        | WLDAP32_LDAP_OPT_SERVER_CERTIFICATE
        | WLDAP32_LDAP_OPT_SERVER_CONTROLS
        | WLDAP32_LDAP_OPT_SERVER_ERROR
        | WLDAP32_LDAP_OPT_SERVER_EXT_ERROR
        | WLDAP32_LDAP_OPT_SIGN
        | WLDAP32_LDAP_OPT_SSL
        | WLDAP32_LDAP_OPT_SSL_INFO
        | WLDAP32_LDAP_OPT_SSPI_FLAGS
        | WLDAP32_LDAP_OPT_TCP_KEEPALIVE => {
            return WLDAP32_LDAP_NOT_SUPPORTED;
        }

        _ => {
            return WLDAP32_LDAP_LOCAL_ERROR;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_set_optionA(ld: *mut WLDAP32_LDAP, option: i32, value: *mut core::ffi::c_void) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    match option {
        WLDAP32_LDAP_OPT_SERVER_CONTROLS => {
            let ctrlsW = controlarrayAtoW(value);
            if ctrlsW.is_null() {
                return WLDAP32_LDAP_NO_MEMORY;
            }
            ret = ldap_set_optionW(ld, option, ctrlsW as *mut core::ffi::c_void);
            controlarrayfreeW(ctrlsW);
            return ret;
        }
        WLDAP32_LDAP_OPT_DEREF | WLDAP32_LDAP_OPT_DESC | WLDAP32_LDAP_OPT_ERROR_NUMBER | WLDAP32_LDAP_OPT_PROTOCOL_VERSION |
        WLDAP32_LDAP_OPT_REFERRALS | WLDAP32_LDAP_OPT_SIZELIMIT | WLDAP32_LDAP_OPT_TIMELIMIT => {
            return ldap_set_optionW(ld, option, value);
        }
        WLDAP32_LDAP_OPT_CACHE_ENABLE | WLDAP32_LDAP_OPT_CACHE_FN_PTRS | WLDAP32_LDAP_OPT_CACHE_STRATEGY | WLDAP32_LDAP_OPT_IO_FN_PTRS |
        WLDAP32_LDAP_OPT_REBIND_ARG | WLDAP32_LDAP_OPT_REBIND_FN | WLDAP32_LDAP_OPT_RESTART | WLDAP32_LDAP_OPT_THREAD_FN_PTRS => {
            return WLDAP32_LDAP_LOCAL_ERROR;
        }
        WLDAP32_LDAP_OPT_API_FEATURE_INFO | WLDAP32_LDAP_OPT_API_INFO => {
            return WLDAP32_LDAP_UNWILLING_TO_PERFORM;
        }
        WLDAP32_LDAP_OPT_AREC_EXCLUSIVE | WLDAP32_LDAP_OPT_AUTO_RECONNECT | WLDAP32_LDAP_OPT_CLIENT_CERTIFICATE | WLDAP32_LDAP_OPT_DNSDOMAIN_NAME |
        WLDAP32_LDAP_OPT_ENCRYPT | WLDAP32_LDAP_OPT_ERROR_STRING | WLDAP32_LDAP_OPT_FAST_CONCURRENT_BIND | WLDAP32_LDAP_OPT_GETDSNAME_FLAGS |
        WLDAP32_LDAP_OPT_HOST_NAME | WLDAP32_LDAP_OPT_HOST_REACHABLE | WLDAP32_LDAP_OPT_PING_KEEP_ALIVE | WLDAP32_LDAP_OPT_PING_LIMIT |
        WLDAP32_LDAP_OPT_PING_WAIT_TIME | WLDAP32_LDAP_OPT_PROMPT_CREDENTIALS | WLDAP32_LDAP_OPT_REF_DEREF_CONN_PER_MSG | WLDAP32_LDAP_OPT_REFERRAL_CALLBACK |
        WLDAP32_LDAP_OPT_REFERRAL_HOP_LIMIT | WLDAP32_LDAP_OPT_ROOTDSE_CACHE | WLDAP32_LDAP_OPT_SASL_METHOD | WLDAP32_LDAP_OPT_SECURITY_CONTEXT |
        WLDAP32_LDAP_OPT_SEND_TIMEOUT | WLDAP32_LDAP_OPT_SERVER_CERTIFICATE | WLDAP32_LDAP_OPT_SERVER_ERROR | WLDAP32_LDAP_OPT_SERVER_EXT_ERROR |
        WLDAP32_LDAP_OPT_SIGN | WLDAP32_LDAP_OPT_SSL | WLDAP32_LDAP_OPT_SSL_INFO | WLDAP32_LDAP_OPT_SSPI_FLAGS |
        WLDAP32_LDAP_OPT_TCP_KEEPALIVE => {
            return WLDAP32_LDAP_NOT_SUPPORTED;
        }
        _ => {
            return WLDAP32_LDAP_LOCAL_ERROR;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ldap_set_optionW(ld: *mut WLDAP32_LDAP, option: i32, value: *mut core::ffi::c_void) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;

    if ld.is_null() {
        return WLDAP32_LDAP_PARAM_ERROR;
    }

    match option {
        WLDAP32_LDAP_OPT_SERVER_CONTROLS => {
            let ctrlsU = controlarrayWtoU(value);
            if ctrlsU.is_null() {
                return WLDAP32_LDAP_NO_MEMORY;
            }

            ret = map_error(ldap_set_option(ld, option, ctrlsU));
            controlarrayfreeU(ctrlsU);
            return ret;
        }
        WLDAP32_LDAP_OPT_DEREF | WLDAP32_LDAP_OPT_DESC | WLDAP32_LDAP_OPT_ERROR_NUMBER | WLDAP32_LDAP_OPT_PROTOCOL_VERSION | WLDAP32_LDAP_OPT_REFERRALS | WLDAP32_LDAP_OPT_SIZELIMIT | WLDAP32_LDAP_OPT_TIMELIMIT => {
            return map_error(ldap_set_option(ld, option, value));
        }
        WLDAP32_LDAP_OPT_CACHE_ENABLE | WLDAP32_LDAP_OPT_CACHE_FN_PTRS | WLDAP32_LDAP_OPT_CACHE_STRATEGY | WLDAP32_LDAP_OPT_IO_FN_PTRS | WLDAP32_LDAP_OPT_REBIND_ARG | WLDAP32_LDAP_OPT_REBIND_FN | WLDAP32_LDAP_OPT_RESTART | WLDAP32_LDAP_OPT_THREAD_FN_PTRS => {
            return WLDAP32_LDAP_LOCAL_ERROR;
        }
        WLDAP32_LDAP_OPT_API_FEATURE_INFO | WLDAP32_LDAP_OPT_API_INFO => {
            return WLDAP32_LDAP_UNWILLING_TO_PERFORM;
        }
        WLDAP32_LDAP_OPT_AREC_EXCLUSIVE | WLDAP32_LDAP_OPT_AUTO_RECONNECT | WLDAP32_LDAP_OPT_CLIENT_CERTIFICATE | WLDAP32_LDAP_OPT_DNSDOMAIN_NAME | WLDAP32_LDAP_OPT_ENCRYPT | WLDAP32_LDAP_OPT_ERROR_STRING | WLDAP32_LDAP_OPT_FAST_CONCURRENT_BIND | WLDAP32_LDAP_OPT_GETDSNAME_FLAGS | WLDAP32_LDAP_OPT_HOST_NAME | WLDAP32_LDAP_OPT_HOST_REACHABLE | WLDAP32_LDAP_OPT_PING_KEEP_ALIVE | WLDAP32_LDAP_OPT_PING_LIMIT | WLDAP32_LDAP_OPT_PING_WAIT_TIME | WLDAP32_LDAP_OPT_PROMPT_CREDENTIALS | WLDAP32_LDAP_OPT_REF_DEREF_CONN_PER_MSG | WLDAP32_LDAP_OPT_REFERRAL_CALLBACK | WLDAP32_LDAP_OPT_REFERRAL_HOP_LIMIT | WLDAP32_LDAP_OPT_ROOTDSE_CACHE | WLDAP32_LDAP_OPT_SASL_METHOD | WLDAP32_LDAP_OPT_SECURITY_CONTEXT | WLDAP32_LDAP_OPT_SEND_TIMEOUT | WLDAP32_LDAP_OPT_SERVER_CERTIFICATE | WLDAP32_LDAP_OPT_SERVER_ERROR | WLDAP32_LDAP_OPT_SERVER_EXT_ERROR | WLDAP32_LDAP_OPT_SIGN | WLDAP32_LDAP_OPT_SSL | WLDAP32_LDAP_OPT_SSL_INFO | WLDAP32_LDAP_OPT_SSPI_FLAGS | WLDAP32_LDAP_OPT_TCP_KEEPALIVE => {
            return WLDAP32_LDAP_NOT_SUPPORTED;
        }
        _ => {
            return WLDAP32_LDAP_LOCAL_ERROR;
        }
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
//  *      ldap_get_optionA     (WLDAP32.@)
//  *
//  * See ldap_get_optionW.
//  */
// ULONG CDECL ldap_get_optionA( WLDAP32_LDAP *ld, int option, void *value )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, 0x%08x, %p)\n", ld, option, value );
//
//     if (!ld || !value) return WLDAP32_LDAP_PARAM_ERROR;
//
//     switch (option)
//     {
//     case WLDAP32_LDAP_OPT_API_FEATURE_INFO:
//     {
//         LDAPAPIFeatureInfoW featureW;
//         LDAPAPIFeatureInfoA *featureA = value;
//
//         if (!featureA->ldapaif_name) return WLDAP32_LDAP_PARAM_ERROR;
//
//         featureW.ldapaif_info_version = featureA->ldapaif_info_version;
//         featureW.ldapaif_name = strAtoW( featureA->ldapaif_name );
//         featureW.ldapaif_version = 0;
//
//         if (!featureW.ldapaif_name) return WLDAP32_LDAP_NO_MEMORY;
//
//         ret = ldap_get_optionW( ld, option, &featureW );
//
//         featureA->ldapaif_version = featureW.ldapaif_version;
//         strfreeW( featureW.ldapaif_name );
//         return ret;
//     }
//     case WLDAP32_LDAP_OPT_API_INFO:
//     {
//         LDAPAPIInfoW infoW;
//         LDAPAPIInfoA *infoA = value;
//
//         memset( &infoW, 0, sizeof(LDAPAPIInfoW) );
//         infoW.ldapai_info_version = infoA->ldapai_info_version;
//
//         ret = ldap_get_optionW( ld, option, &infoW );
//
//         infoA->ldapai_api_version = infoW.ldapai_api_version;
//         infoA->ldapai_protocol_version = infoW.ldapai_protocol_version;
//
//         if (infoW.ldapai_extensions)
//         {
//             infoA->ldapai_extensions = strarrayWtoA( infoW.ldapai_extensions );
//             if (!infoA->ldapai_extensions) return WLDAP32_LDAP_NO_MEMORY;
//         }
//         if (infoW.ldapai_vendor_name)
//         {
//             infoA->ldapai_vendor_name = strWtoA( infoW.ldapai_vendor_name );
//             if (!infoA->ldapai_vendor_name)
//             {
//                 ldap_value_freeW( infoW.ldapai_extensions );
//                 return WLDAP32_LDAP_NO_MEMORY;
//             }
//         }
//         infoA->ldapai_vendor_version = infoW.ldapai_vendor_version;
//
//         ldap_value_freeW( infoW.ldapai_extensions );
//         ldap_memfreeW( infoW.ldapai_vendor_name );
//         return ret;
//     }
//
//     case WLDAP32_LDAP_OPT_DEREF:
//     case WLDAP32_LDAP_OPT_DESC:
//     case WLDAP32_LDAP_OPT_ERROR_NUMBER:
//     case WLDAP32_LDAP_OPT_PROTOCOL_VERSION:
//     case WLDAP32_LDAP_OPT_REFERRALS:
//     case WLDAP32_LDAP_OPT_SIZELIMIT:
//     case WLDAP32_LDAP_OPT_TIMELIMIT:
//         return ldap_get_optionW( ld, option, value );
//
//     case WLDAP32_LDAP_OPT_CACHE_ENABLE:
//     case WLDAP32_LDAP_OPT_CACHE_FN_PTRS:
//     case WLDAP32_LDAP_OPT_CACHE_STRATEGY:
//     case WLDAP32_LDAP_OPT_IO_FN_PTRS:
//     case WLDAP32_LDAP_OPT_REBIND_ARG:
//     case WLDAP32_LDAP_OPT_REBIND_FN:
//     case WLDAP32_LDAP_OPT_RESTART:
//     case WLDAP32_LDAP_OPT_THREAD_FN_PTRS:
//         return LDAP_LOCAL_ERROR;
//
//     case WLDAP32_LDAP_OPT_AREC_EXCLUSIVE:
//     case WLDAP32_LDAP_OPT_AUTO_RECONNECT:
//     case WLDAP32_LDAP_OPT_CLIENT_CERTIFICATE:
//     case WLDAP32_LDAP_OPT_DNSDOMAIN_NAME:
//     case WLDAP32_LDAP_OPT_ENCRYPT:
//     case WLDAP32_LDAP_OPT_ERROR_STRING:
//     case WLDAP32_LDAP_OPT_FAST_CONCURRENT_BIND:
//     case WLDAP32_LDAP_OPT_GETDSNAME_FLAGS:
//     case WLDAP32_LDAP_OPT_HOST_NAME:
//     case WLDAP32_LDAP_OPT_HOST_REACHABLE:
//     case WLDAP32_LDAP_OPT_PING_KEEP_ALIVE:
//     case WLDAP32_LDAP_OPT_PING_LIMIT:
//     case WLDAP32_LDAP_OPT_PING_WAIT_TIME:
//     case WLDAP32_LDAP_OPT_PROMPT_CREDENTIALS:
//     case WLDAP32_LDAP_OPT_REF_DEREF_CONN_PER_MSG:
//     case WLDAP32_LDAP_OPT_REFERRAL_CALLBACK:
//     case WLDAP32_LDAP_OPT_REFERRAL_HOP_LIMIT:
//     case WLDAP32_LDAP_OPT_ROOTDSE_CACHE:
//     case WLDAP32_LDAP_OPT_SASL_METHOD:
//     case WLDAP32_LDAP_OPT_SECURITY_CONTEXT:
//     case WLDAP32_LDAP_OPT_SEND_TIMEOUT:
//     case WLDAP32_LDAP_OPT_SERVER_CERTIFICATE:
//     case WLDAP32_LDAP_OPT_SERVER_CONTROLS:
//     case WLDAP32_LDAP_OPT_SERVER_ERROR:
//     case WLDAP32_LDAP_OPT_SERVER_EXT_ERROR:
//     case WLDAP32_LDAP_OPT_SIGN:
//     case WLDAP32_LDAP_OPT_SSL:
//     case WLDAP32_LDAP_OPT_SSL_INFO:
//     case WLDAP32_LDAP_OPT_SSPI_FLAGS:
//     case WLDAP32_LDAP_OPT_TCP_KEEPALIVE:
//         FIXME( "Unsupported option: 0x%02x\n", option );
//         return WLDAP32_LDAP_NOT_SUPPORTED;
//
//     default:
//         FIXME( "Unknown option: 0x%02x\n", option );
//         return WLDAP32_LDAP_LOCAL_ERROR;
//     }
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_get_optionW     (WLDAP32.@)
//  *
//  * Retrieve option values for a given LDAP context.
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  option  [I] Option to get values for.
//  *  value   [O] Pointer to option values.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  */
// ULONG CDECL ldap_get_optionW( WLDAP32_LDAP *ld, int option, void *value )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, 0x%08x, %p)\n", ld, option, value );
//
//     if (!ld || !value) return WLDAP32_LDAP_PARAM_ERROR;
//
//     switch (option)
//     {
//     case WLDAP32_LDAP_OPT_API_FEATURE_INFO:
//     {
//         LDAPAPIFeatureInfo featureU;
//         LDAPAPIFeatureInfoW *featureW = value;
//
//         if (!featureW->ldapaif_name) return WLDAP32_LDAP_PARAM_ERROR;
//
//         featureU.ldapaif_info_version = featureW->ldapaif_info_version;
//         featureU.ldapaif_name = strWtoU( featureW->ldapaif_name );
//         featureU.ldapaif_version = 0;
//
//         if (!featureU.ldapaif_name) return WLDAP32_LDAP_NO_MEMORY;
//
//         ret = map_error( ldap_get_option( ld, option, &featureU ));
//
//         featureW->ldapaif_version = featureU.ldapaif_version;
//         strfreeU( featureU.ldapaif_name );
//         return ret;
//     }
//     case WLDAP32_LDAP_OPT_API_INFO:
//     {
//         LDAPAPIInfo infoU;
//         LDAPAPIInfoW *infoW = value;
//
//         memset( &infoU, 0, sizeof(LDAPAPIInfo) );
//         infoU.ldapai_info_version = infoW->ldapai_info_version;
//
//         ret = map_error( ldap_get_option( ld, option, &infoU ));
//
//         infoW->ldapai_api_version = infoU.ldapai_api_version;
//         infoW->ldapai_protocol_version = infoU.ldapai_protocol_version;
//
//         if (infoU.ldapai_extensions)
//         {
//             infoW->ldapai_extensions = strarrayUtoW( infoU.ldapai_extensions );
//             if (!infoW->ldapai_extensions) return WLDAP32_LDAP_NO_MEMORY;
//         }
//         if (infoU.ldapai_vendor_name)
//         {
//             infoW->ldapai_vendor_name = strUtoW( infoU.ldapai_vendor_name );
//             if (!infoW->ldapai_vendor_name)
//             {
//                 ldap_memvfree( (void **)infoU.ldapai_extensions );
//                 return WLDAP32_LDAP_NO_MEMORY;
//             }
//         }
//         infoW->ldapai_vendor_version = infoU.ldapai_vendor_version;
//
//         ldap_memvfree( (void **)infoU.ldapai_extensions );
//         ldap_memfree( infoU.ldapai_vendor_name );
//         return ret;
//     }
//
//     case WLDAP32_LDAP_OPT_DEREF:
//     case WLDAP32_LDAP_OPT_DESC:
//     case WLDAP32_LDAP_OPT_ERROR_NUMBER:
//     case WLDAP32_LDAP_OPT_PROTOCOL_VERSION:
//     case WLDAP32_LDAP_OPT_REFERRALS:
//     case WLDAP32_LDAP_OPT_SIZELIMIT:
//     case WLDAP32_LDAP_OPT_TIMELIMIT:
//         return map_error( ldap_get_option( ld, option, value ));
//
//     case WLDAP32_LDAP_OPT_CACHE_ENABLE:
//     case WLDAP32_LDAP_OPT_CACHE_FN_PTRS:
//     case WLDAP32_LDAP_OPT_CACHE_STRATEGY:
//     case WLDAP32_LDAP_OPT_IO_FN_PTRS:
//     case WLDAP32_LDAP_OPT_REBIND_ARG:
//     case WLDAP32_LDAP_OPT_REBIND_FN:
//     case WLDAP32_LDAP_OPT_RESTART:
//     case WLDAP32_LDAP_OPT_THREAD_FN_PTRS:
//         return WLDAP32_LDAP_LOCAL_ERROR;
//
//     case WLDAP32_LDAP_OPT_AREC_EXCLUSIVE:
//     case WLDAP32_LDAP_OPT_AUTO_RECONNECT:
//     case WLDAP32_LDAP_OPT_CLIENT_CERTIFICATE:
//     case WLDAP32_LDAP_OPT_DNSDOMAIN_NAME:
//     case WLDAP32_LDAP_OPT_ENCRYPT:
//     case WLDAP32_LDAP_OPT_ERROR_STRING:
//     case WLDAP32_LDAP_OPT_FAST_CONCURRENT_BIND:
//     case WLDAP32_LDAP_OPT_GETDSNAME_FLAGS:
//     case WLDAP32_LDAP_OPT_HOST_NAME:
//     case WLDAP32_LDAP_OPT_HOST_REACHABLE:
//     case WLDAP32_LDAP_OPT_PING_KEEP_ALIVE:
//     case WLDAP32_LDAP_OPT_PING_LIMIT:
//     case WLDAP32_LDAP_OPT_PING_WAIT_TIME:
//     case WLDAP32_LDAP_OPT_PROMPT_CREDENTIALS:
//     case WLDAP32_LDAP_OPT_REF_DEREF_CONN_PER_MSG:
//     case WLDAP32_LDAP_OPT_REFERRAL_CALLBACK:
//     case WLDAP32_LDAP_OPT_REFERRAL_HOP_LIMIT:
//     case WLDAP32_LDAP_OPT_ROOTDSE_CACHE:
//     case WLDAP32_LDAP_OPT_SASL_METHOD:
//     case WLDAP32_LDAP_OPT_SECURITY_CONTEXT:
//     case WLDAP32_LDAP_OPT_SEND_TIMEOUT:
//     case WLDAP32_LDAP_OPT_SERVER_CERTIFICATE:
//     case WLDAP32_LDAP_OPT_SERVER_CONTROLS:
//     case WLDAP32_LDAP_OPT_SERVER_ERROR:
//     case WLDAP32_LDAP_OPT_SERVER_EXT_ERROR:
//     case WLDAP32_LDAP_OPT_SIGN:
//     case WLDAP32_LDAP_OPT_SSL:
//     case WLDAP32_LDAP_OPT_SSL_INFO:
//     case WLDAP32_LDAP_OPT_SSPI_FLAGS:
//     case WLDAP32_LDAP_OPT_TCP_KEEPALIVE:
//         FIXME( "Unsupported option: 0x%02x\n", option );
//         return WLDAP32_LDAP_NOT_SUPPORTED;
//
//     default:
//         FIXME( "Unknown option: 0x%02x\n", option );
//         return WLDAP32_LDAP_LOCAL_ERROR;
//     }
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_set_optionA     (WLDAP32.@)
//  *
//  * See ldap_set_optionW.
//  */
// ULONG CDECL ldap_set_optionA( WLDAP32_LDAP *ld, int option, void *value )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, 0x%08x, %p)\n", ld, option, value );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     switch (option)
//     {
//     case WLDAP32_LDAP_OPT_SERVER_CONTROLS:
//     {
//         LDAPControlW **ctrlsW;
//
//         ctrlsW = controlarrayAtoW( value );
//         if (!ctrlsW) return WLDAP32_LDAP_NO_MEMORY;
//
//         ret = ldap_set_optionW( ld, option, ctrlsW );
//         controlarrayfreeW( ctrlsW );
//         return ret;
//     }
//     case WLDAP32_LDAP_OPT_DEREF:
//     case WLDAP32_LDAP_OPT_DESC:
//     case WLDAP32_LDAP_OPT_ERROR_NUMBER:
//     case WLDAP32_LDAP_OPT_PROTOCOL_VERSION:
//     case WLDAP32_LDAP_OPT_REFERRALS:
//     case WLDAP32_LDAP_OPT_SIZELIMIT:
//     case WLDAP32_LDAP_OPT_TIMELIMIT:
//         return ldap_set_optionW( ld, option, value );
//
//     case WLDAP32_LDAP_OPT_CACHE_ENABLE:
//     case WLDAP32_LDAP_OPT_CACHE_FN_PTRS:
//     case WLDAP32_LDAP_OPT_CACHE_STRATEGY:
//     case WLDAP32_LDAP_OPT_IO_FN_PTRS:
//     case WLDAP32_LDAP_OPT_REBIND_ARG:
//     case WLDAP32_LDAP_OPT_REBIND_FN:
//     case WLDAP32_LDAP_OPT_RESTART:
//     case WLDAP32_LDAP_OPT_THREAD_FN_PTRS:
//         return WLDAP32_LDAP_LOCAL_ERROR;
//
//     case WLDAP32_LDAP_OPT_API_FEATURE_INFO:
//     case WLDAP32_LDAP_OPT_API_INFO:
//         return WLDAP32_LDAP_UNWILLING_TO_PERFORM;
//
//     case WLDAP32_LDAP_OPT_AREC_EXCLUSIVE:
//     case WLDAP32_LDAP_OPT_AUTO_RECONNECT:
//     case WLDAP32_LDAP_OPT_CLIENT_CERTIFICATE:
//     case WLDAP32_LDAP_OPT_DNSDOMAIN_NAME:
//     case WLDAP32_LDAP_OPT_ENCRYPT:
//     case WLDAP32_LDAP_OPT_ERROR_STRING:
//     case WLDAP32_LDAP_OPT_FAST_CONCURRENT_BIND:
//     case WLDAP32_LDAP_OPT_GETDSNAME_FLAGS:
//     case WLDAP32_LDAP_OPT_HOST_NAME:
//     case WLDAP32_LDAP_OPT_HOST_REACHABLE:
//     case WLDAP32_LDAP_OPT_PING_KEEP_ALIVE:
//     case WLDAP32_LDAP_OPT_PING_LIMIT:
//     case WLDAP32_LDAP_OPT_PING_WAIT_TIME:
//     case WLDAP32_LDAP_OPT_PROMPT_CREDENTIALS:
//     case WLDAP32_LDAP_OPT_REF_DEREF_CONN_PER_MSG:
//     case WLDAP32_LDAP_OPT_REFERRAL_CALLBACK:
//     case WLDAP32_LDAP_OPT_REFERRAL_HOP_LIMIT:
//     case WLDAP32_LDAP_OPT_ROOTDSE_CACHE:
//     case WLDAP32_LDAP_OPT_SASL_METHOD:
//     case WLDAP32_LDAP_OPT_SECURITY_CONTEXT:
//     case WLDAP32_LDAP_OPT_SEND_TIMEOUT:
//     case WLDAP32_LDAP_OPT_SERVER_CERTIFICATE:
//     case WLDAP32_LDAP_OPT_SERVER_ERROR:
//     case WLDAP32_LDAP_OPT_SERVER_EXT_ERROR:
//     case WLDAP32_LDAP_OPT_SIGN:
//     case WLDAP32_LDAP_OPT_SSL:
//     case WLDAP32_LDAP_OPT_SSL_INFO:
//     case WLDAP32_LDAP_OPT_SSPI_FLAGS:
//     case WLDAP32_LDAP_OPT_TCP_KEEPALIVE:
//         FIXME( "Unsupported option: 0x%02x\n", option );
//         return WLDAP32_LDAP_NOT_SUPPORTED;
//
//     default:
//         FIXME( "Unknown option: 0x%02x\n", option );
//         return WLDAP32_LDAP_LOCAL_ERROR;
//     }
//
// #endif
//     return ret;
// }
//
// /***********************************************************************
//  *      ldap_set_optionW     (WLDAP32.@)
//  *
//  * Set option values for a given LDAP context.
//  *
//  * PARAMS
//  *  ld      [I] Pointer to an LDAP context.
//  *  option  [I] Option to set values for.
//  *  value   [I] Pointer to option values.
//  *
//  * RETURNS
//  *  Success: LDAP_SUCCESS
//  *  Failure: An LDAP error code.
//  *
//  * NOTES
//  *  Set value to LDAP_OPT_ON or LDAP_OPT_OFF for on/off options.
//  */
// ULONG CDECL ldap_set_optionW( WLDAP32_LDAP *ld, int option, void *value )
// {
//     ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
// #ifdef HAVE_LDAP
//
//     TRACE( "(%p, 0x%08x, %p)\n", ld, option, value );
//
//     if (!ld) return WLDAP32_LDAP_PARAM_ERROR;
//
//     switch (option)
//     {
//     case WLDAP32_LDAP_OPT_SERVER_CONTROLS:
//     {
//         LDAPControl **ctrlsU;
//
//         ctrlsU = controlarrayWtoU( value );
//         if (!ctrlsU) return WLDAP32_LDAP_NO_MEMORY;
//
//         ret = map_error( ldap_set_option( ld, option, ctrlsU ));
//         controlarrayfreeU( ctrlsU );
//         return ret;
//     }
//     case WLDAP32_LDAP_OPT_DEREF:
//     case WLDAP32_LDAP_OPT_DESC:
//     case WLDAP32_LDAP_OPT_ERROR_NUMBER:
//     case WLDAP32_LDAP_OPT_PROTOCOL_VERSION:
//     case WLDAP32_LDAP_OPT_REFERRALS:
//     case WLDAP32_LDAP_OPT_SIZELIMIT:
//     case WLDAP32_LDAP_OPT_TIMELIMIT:
//         return map_error( ldap_set_option( ld, option, value ));
//
//     case WLDAP32_LDAP_OPT_CACHE_ENABLE:
//     case WLDAP32_LDAP_OPT_CACHE_FN_PTRS:
//     case WLDAP32_LDAP_OPT_CACHE_STRATEGY:
//     case WLDAP32_LDAP_OPT_IO_FN_PTRS:
//     case WLDAP32_LDAP_OPT_REBIND_ARG:
//     case WLDAP32_LDAP_OPT_REBIND_FN:
//     case WLDAP32_LDAP_OPT_RESTART:
//     case WLDAP32_LDAP_OPT_THREAD_FN_PTRS:
//         return WLDAP32_LDAP_LOCAL_ERROR;
//
//     case WLDAP32_LDAP_OPT_API_FEATURE_INFO:
//     case WLDAP32_LDAP_OPT_API_INFO:
//         return WLDAP32_LDAP_UNWILLING_TO_PERFORM;
//
//     case WLDAP32_LDAP_OPT_AREC_EXCLUSIVE:
//     case WLDAP32_LDAP_OPT_AUTO_RECONNECT:
//     case WLDAP32_LDAP_OPT_CLIENT_CERTIFICATE:
//     case WLDAP32_LDAP_OPT_DNSDOMAIN_NAME:
//     case WLDAP32_LDAP_OPT_ENCRYPT:
//     case WLDAP32_LDAP_OPT_ERROR_STRING:
//     case WLDAP32_LDAP_OPT_FAST_CONCURRENT_BIND:
//     case WLDAP32_LDAP_OPT_GETDSNAME_FLAGS:
//     case WLDAP32_LDAP_OPT_HOST_NAME:
//     case WLDAP32_LDAP_OPT_HOST_REACHABLE:
//     case WLDAP32_LDAP_OPT_PING_KEEP_ALIVE:
//     case WLDAP32_LDAP_OPT_PING_LIMIT:
//     case WLDAP32_LDAP_OPT_PING_WAIT_TIME:
//     case WLDAP32_LDAP_OPT_PROMPT_CREDENTIALS:
//     case WLDAP32_LDAP_OPT_REF_DEREF_CONN_PER_MSG:
//     case WLDAP32_LDAP_OPT_REFERRAL_CALLBACK:
//     case WLDAP32_LDAP_OPT_REFERRAL_HOP_LIMIT:
//     case WLDAP32_LDAP_OPT_ROOTDSE_CACHE:
//     case WLDAP32_LDAP_OPT_SASL_METHOD:
//     case WLDAP32_LDAP_OPT_SECURITY_CONTEXT:
//     case WLDAP32_LDAP_OPT_SEND_TIMEOUT:
//     case WLDAP32_LDAP_OPT_SERVER_CERTIFICATE:
//     case WLDAP32_LDAP_OPT_SERVER_ERROR:
//     case WLDAP32_LDAP_OPT_SERVER_EXT_ERROR:
//     case WLDAP32_LDAP_OPT_SIGN:
//     case WLDAP32_LDAP_OPT_SSL:
//     case WLDAP32_LDAP_OPT_SSL_INFO:
//     case WLDAP32_LDAP_OPT_SSPI_FLAGS:
//     case WLDAP32_LDAP_OPT_TCP_KEEPALIVE:
//         FIXME( "Unsupported option: 0x%02x\n", option );
//         return WLDAP32_LDAP_NOT_SUPPORTED;
//
//     default:
//         FIXME( "Unknown option: 0x%02x\n", option );
//         return WLDAP32_LDAP_LOCAL_ERROR;
//     }
//
// #endif
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
