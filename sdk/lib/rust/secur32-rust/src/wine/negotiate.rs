use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NEGO_MAX_TOKEN: u32 = 12000;

// negotiate.c: #define CAPS ( … )  — the Negotiate package capability flags.
pub const CAPS: u32 = SECPKG_FLAG_INTEGRITY
    | SECPKG_FLAG_PRIVACY
    | SECPKG_FLAG_CONNECTION
    | SECPKG_FLAG_MULTI_REQUIRED
    | SECPKG_FLAG_EXTENDED_ERROR
    | SECPKG_FLAG_IMPERSONATION
    | SECPKG_FLAG_ACCEPT_WIN32_NAME
    | SECPKG_FLAG_NEGOTIABLE
    | SECPKG_FLAG_GSS_COMPATIBLE
    | SECPKG_FLAG_LOGON
    | SECPKG_FLAG_RESTRICTED_TOKENS;

// C: static const SecurityFunctionTable{A,W} negoTable{A,W} — dropped by the
// translation, recovered from negotiate.c. Stored `static mut` because the
// bindgen table type embeds raw-pointer Reserved fields (not Sync); the data is
// never mutated. SEC_ENTRY members bridge via sec_entry! (see lib.rs).
pub static mut negoTableA: SecurityFunctionTableA = SecurityFunctionTableA {
    dwVersion: 1,
    EnumerateSecurityPackagesA: None,
    QueryCredentialsAttributesA: sec_entry!(nego_QueryCredentialsAttributesA),
    AcquireCredentialsHandleA: sec_entry!(nego_AcquireCredentialsHandleA),
    FreeCredentialsHandle: sec_entry!(nego_FreeCredentialsHandle),
    Reserved2: core::ptr::null_mut(),
    InitializeSecurityContextA: sec_entry!(nego_InitializeSecurityContextA),
    AcceptSecurityContext: sec_entry!(nego_AcceptSecurityContext),
    CompleteAuthToken: sec_entry!(nego_CompleteAuthToken),
    DeleteSecurityContext: sec_entry!(nego_DeleteSecurityContext),
    ApplyControlToken: sec_entry!(nego_ApplyControlToken),
    QueryContextAttributesA: sec_entry!(nego_QueryContextAttributesA),
    ImpersonateSecurityContext: sec_entry!(nego_ImpersonateSecurityContext),
    RevertSecurityContext: sec_entry!(nego_RevertSecurityContext),
    MakeSignature: sec_entry!(nego_MakeSignature),
    VerifySignature: sec_entry!(nego_VerifySignature),
    FreeContextBuffer: sec_entry!(FreeContextBuffer),
    QuerySecurityPackageInfoA: None,
    Reserved3: core::ptr::null_mut(),
    Reserved4: core::ptr::null_mut(),
    ExportSecurityContext: None,
    ImportSecurityContextA: None,
    AddCredentialsA: None,
    Reserved8: core::ptr::null_mut(),
    QuerySecurityContextToken: None,
    EncryptMessage: sec_entry!(nego_EncryptMessage),
    DecryptMessage: sec_entry!(nego_DecryptMessage),
    SetContextAttributesA: None,
};

pub static mut negoTableW: SecurityFunctionTableW = SecurityFunctionTableW {
    dwVersion: 1,
    EnumerateSecurityPackagesW: None,
    QueryCredentialsAttributesW: sec_entry!(nego_QueryCredentialsAttributesW),
    AcquireCredentialsHandleW: sec_entry!(nego_AcquireCredentialsHandleW),
    FreeCredentialsHandle: sec_entry!(nego_FreeCredentialsHandle),
    Reserved2: core::ptr::null_mut(),
    InitializeSecurityContextW: sec_entry!(nego_InitializeSecurityContextW),
    AcceptSecurityContext: sec_entry!(nego_AcceptSecurityContext),
    CompleteAuthToken: sec_entry!(nego_CompleteAuthToken),
    DeleteSecurityContext: sec_entry!(nego_DeleteSecurityContext),
    ApplyControlToken: sec_entry!(nego_ApplyControlToken),
    QueryContextAttributesW: sec_entry!(nego_QueryContextAttributesW),
    ImpersonateSecurityContext: sec_entry!(nego_ImpersonateSecurityContext),
    RevertSecurityContext: sec_entry!(nego_RevertSecurityContext),
    MakeSignature: sec_entry!(nego_MakeSignature),
    VerifySignature: sec_entry!(nego_VerifySignature),
    FreeContextBuffer: sec_entry!(FreeContextBuffer),
    QuerySecurityPackageInfoW: None,
    Reserved3: core::ptr::null_mut(),
    Reserved4: core::ptr::null_mut(),
    ExportSecurityContext: None,
    ImportSecurityContextW: None,
    AddCredentialsW: None,
    Reserved8: core::ptr::null_mut(),
    QuerySecurityContextToken: None,
    EncryptMessage: sec_entry!(nego_EncryptMessage),
    DecryptMessage: sec_entry!(nego_DecryptMessage),
    SetContextAttributesW: None,
};

pub static nego_name_W: [WCHAR; 10] = [
    'N' as WCHAR, 'e' as WCHAR, 'g' as WCHAR, 'o' as WCHAR, 't' as WCHAR,
    'i' as WCHAR, 'a' as WCHAR, 't' as WCHAR, 'e' as WCHAR, 0,
];

pub static nego_name_A: [u8; 10] = *b"Negotiate\0";

pub static negotiate_comment_W: [WCHAR; 29] = [
    'M' as WCHAR, 'i' as WCHAR, 'c' as WCHAR, 'r' as WCHAR, 'o' as WCHAR,
    's' as WCHAR, 'o' as WCHAR, 'f' as WCHAR, 't' as WCHAR, ' ' as WCHAR,
    'P' as WCHAR, 'a' as WCHAR, 'c' as WCHAR, 'k' as WCHAR, 'a' as WCHAR,
    'g' as WCHAR, 'e' as WCHAR, ' ' as WCHAR, 'N' as WCHAR, 'e' as WCHAR,
    'g' as WCHAR, 'o' as WCHAR, 't' as WCHAR, 'i' as WCHAR, 'a' as WCHAR,
    't' as WCHAR, 'o' as WCHAR, 'r' as WCHAR,
 0,];

pub static negotiate_comment_A: [CHAR; 29] = {
    let mut a: [CHAR; 29] = [0; 29];
    let s = b"Microsoft Package Negotiator";
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as CHAR;
        i += 1;
    }
    a
};
unsafe extern "system" fn nego_QueryCredentialsAttributesA(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: PVOID,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn nego_QueryCredentialsAttributesW(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: PVOID,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn nego_AcquireCredentialsHandleW(
    pszPrincipal: *mut SEC_WCHAR,
    pszPackage: *mut SEC_WCHAR,
    fCredentialUse: ULONG,
    pLogonID: PLUID,
    pAuthData: PVOID,
    pGetKeyFn: SEC_GET_KEY_FN,
    pGetKeyArgument: PVOID,
    phCredential: PCredHandle,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    static mut ntlmW: [SEC_WCHAR; 5] = [
        'N' as SEC_WCHAR,
        'T' as SEC_WCHAR,
        'L' as SEC_WCHAR,
        'M' as SEC_WCHAR,
        0,
    ];
    let ret: SECURITY_STATUS;

    ret = ntlm_AcquireCredentialsHandleW(
        pszPrincipal,
        ntlmW.as_mut_ptr(),
        fCredentialUse,
        pLogonID,
        pAuthData,
        pGetKeyFn,
        pGetKeyArgument,
        phCredential,
        ptsExpiry,
    );
    if ret == SEC_E_OK {
        let cred = (*phCredential).dwLower as *mut NtlmCredentials;
        (*cred).no_cached_credentials = (pAuthData == core::ptr::null_mut()) as _;
    }
    ret
}
unsafe extern "system" fn nego_AcquireCredentialsHandleA(
    pszPrincipal: *mut SEC_CHAR,
    pszPackage: *mut SEC_CHAR,
    fCredentialUse: ULONG,
    pLogonID: PLUID,
    pAuthData: PVOID,
    pGetKeyFn: SEC_GET_KEY_FN,
    pGetKeyArgument: PVOID,
    phCredential: PCredHandle,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    let mut ret: SECURITY_STATUS = SEC_E_INSUFFICIENT_MEMORY;
    let mut user: *mut SEC_WCHAR = core::ptr::null_mut();
    let mut domain: *mut SEC_WCHAR = core::ptr::null_mut();
    let mut passwd: *mut SEC_WCHAR = core::ptr::null_mut();
    let mut package: *mut SEC_WCHAR = core::ptr::null_mut();
    let mut identityW: *mut SEC_WINNT_AUTH_IDENTITY_W = core::ptr::null_mut();

    TRACE!(
        b"%s, %s, 0x%08x, %p, %p, %p, %p, %p, %p\n\0".as_ptr() as LPCSTR,
        debugstr_a(pszPrincipal),
        debugstr_a(pszPackage),
        fCredentialUse,
        pLogonID,
        pAuthData,
        pGetKeyFn,
        pGetKeyArgument,
        phCredential,
        ptsExpiry,
    );

    'done: {
        if !pszPackage.is_null() {
            let package_len = MultiByteToWideChar(CP_ACP, 0, pszPackage, -1, core::ptr::null_mut(), 0);
            package = HeapAlloc(
                GetProcessHeap(),
                0,
                (package_len as usize * core::mem::size_of::<SEC_WCHAR>()) as _,
            ) as *mut SEC_WCHAR;
            if package.is_null() {
                return SEC_E_INSUFFICIENT_MEMORY;
            }
            MultiByteToWideChar(CP_ACP, 0, pszPackage, -1, package, package_len);
        }
        if !pAuthData.is_null() {
            let identity = pAuthData as *mut SEC_WINNT_AUTH_IDENTITY_A;
            let mut user_len: i32;
            let mut domain_len: i32;
            let mut passwd_len: i32;

            if (*identity).Flags == SEC_WINNT_AUTH_IDENTITY_ANSI {
                identityW = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    core::mem::size_of::<SEC_WINNT_AUTH_IDENTITY_W>() as _,
                ) as *mut SEC_WINNT_AUTH_IDENTITY_W;
                if identityW.is_null() {
                    break 'done;
                }

                if (*identity).UserLength == 0 {
                    user_len = 0;
                } else {
                    user_len = MultiByteToWideChar(
                        CP_ACP,
                        0,
                        (*identity).User as LPCSTR,
                        (*identity).UserLength as i32,
                        core::ptr::null_mut(),
                        0,
                    );
                    user = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        (user_len as usize * core::mem::size_of::<SEC_WCHAR>()) as _,
                    ) as *mut SEC_WCHAR;
                    if user.is_null() {
                        break 'done;
                    }
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        (*identity).User as LPCSTR,
                        (*identity).UserLength as i32,
                        user,
                        user_len,
                    );
                }
                if (*identity).DomainLength == 0 {
                    domain_len = 0;
                } else {
                    domain_len = MultiByteToWideChar(
                        CP_ACP,
                        0,
                        (*identity).Domain as LPCSTR,
                        (*identity).DomainLength as i32,
                        core::ptr::null_mut(),
                        0,
                    );
                    domain = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        (domain_len as usize * core::mem::size_of::<SEC_WCHAR>()) as _,
                    ) as *mut SEC_WCHAR;
                    if domain.is_null() {
                        break 'done;
                    }
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        (*identity).Domain as LPCSTR,
                        (*identity).DomainLength as i32,
                        domain,
                        domain_len,
                    );
                }
                if (*identity).PasswordLength == 0 {
                    passwd_len = 0;
                } else {
                    passwd_len = MultiByteToWideChar(
                        CP_ACP,
                        0,
                        (*identity).Password as LPCSTR,
                        (*identity).PasswordLength as i32,
                        core::ptr::null_mut(),
                        0,
                    );
                    passwd = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        (passwd_len as usize * core::mem::size_of::<SEC_WCHAR>()) as _,
                    ) as *mut SEC_WCHAR;
                    if passwd.is_null() {
                        break 'done;
                    }
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        (*identity).Password as LPCSTR,
                        (*identity).PasswordLength as i32,
                        passwd,
                        passwd_len,
                    );
                }
                (*identityW).Flags = SEC_WINNT_AUTH_IDENTITY_UNICODE;
                (*identityW).User = user;
                (*identityW).UserLength = user_len as u32;
                (*identityW).Domain = domain;
                (*identityW).DomainLength = domain_len as u32;
                (*identityW).Password = passwd;
                (*identityW).PasswordLength = passwd_len as u32;
            } else {
                identityW = identity as *mut SEC_WINNT_AUTH_IDENTITY_W;
            }
        }
        ret = nego_AcquireCredentialsHandleW(
            core::ptr::null_mut(),
            package,
            fCredentialUse,
            pLogonID,
            identityW as PVOID,
            pGetKeyFn,
            pGetKeyArgument,
            phCredential,
            ptsExpiry,
        );
    }

    HeapFree(GetProcessHeap(), 0, package as PVOID);
    HeapFree(GetProcessHeap(), 0, user as PVOID);
    HeapFree(GetProcessHeap(), 0, domain as PVOID);
    HeapFree(GetProcessHeap(), 0, passwd as PVOID);
    HeapFree(GetProcessHeap(), 0, identityW as PVOID);
    ret
}
unsafe extern "system" fn nego_InitializeSecurityContextW(
    phCredential: PCredHandle,
    phContext: PCtxtHandle,
    pszTargetName: *mut SEC_WCHAR,
    fContextReq: ULONG,
    Reserved1: ULONG,
    TargetDataRep: ULONG,
    pInput: PSecBufferDesc,
    Reserved2: ULONG,
    phNewContext: PCtxtHandle,
    pOutput: PSecBufferDesc,
    pfContextAttr: *mut ULONG,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    ntlm_InitializeSecurityContextW(
        phCredential,
        phContext,
        pszTargetName,
        fContextReq,
        Reserved1,
        TargetDataRep,
        pInput,
        Reserved2,
        phNewContext,
        pOutput,
        pfContextAttr,
        ptsExpiry,
    )
}
unsafe extern "system" fn nego_InitializeSecurityContextA(
    phCredential: PCredHandle,
    phContext: PCtxtHandle,
    pszTargetName: *mut SEC_CHAR,
    fContextReq: ULONG,
    Reserved1: ULONG,
    TargetDataRep: ULONG,
    pInput: PSecBufferDesc,
    Reserved2: ULONG,
    phNewContext: PCtxtHandle,
    pOutput: PSecBufferDesc,
    pfContextAttr: *mut ULONG,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;
    let mut target: *mut SEC_WCHAR = core::ptr::null_mut();

    TRACE!(
        b"%p, %p, %s, 0x%08x, %u, %u, %p, %u, %p, %p, %p, %p\n\0".as_ptr() as LPCSTR,
        phCredential,
        phContext,
        debugstr_a(pszTargetName),
        fContextReq,
        Reserved1,
        TargetDataRep,
        pInput,
        Reserved1,
        phNewContext,
        pOutput,
        pfContextAttr,
        ptsExpiry,
    );

    if !pszTargetName.is_null() {
        let target_len = MultiByteToWideChar(CP_ACP, 0, pszTargetName, -1, core::ptr::null_mut(), 0);
        target = HeapAlloc(
            GetProcessHeap(),
            0,
            (target_len as usize * core::mem::size_of::<SEC_WCHAR>()) as _,
        ) as *mut SEC_WCHAR;
        if target.is_null() {
            return SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
        }
        MultiByteToWideChar(CP_ACP, 0, pszTargetName, -1, target, target_len);
    }

    ret = nego_InitializeSecurityContextW(
        phCredential,
        phContext,
        target,
        fContextReq,
        Reserved1,
        TargetDataRep,
        pInput,
        Reserved2,
        phNewContext,
        pOutput,
        pfContextAttr,
        ptsExpiry,
    );

    HeapFree(GetProcessHeap(), 0, target as _);
    ret
}
unsafe extern "system" fn nego_AcceptSecurityContext(
    phCredential: PCredHandle,
    phContext: PCtxtHandle,
    pInput: PSecBufferDesc,
    fContextReq: ULONG,
    TargetDataRep: ULONG,
    phNewContext: PCtxtHandle,
    pOutput: PSecBufferDesc,
    pfContextAttr: *mut ULONG,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    ntlm_AcceptSecurityContext(
        phCredential,
        phContext,
        pInput,
        fContextReq,
        TargetDataRep,
        phNewContext,
        pOutput,
        pfContextAttr,
        ptsExpiry,
    )
}
unsafe extern "system" fn nego_CompleteAuthToken(phContext: PCtxtHandle, pToken: PSecBufferDesc) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p %p\n\0".as_ptr() as LPCSTR, phContext, pToken);
    if !phContext.is_null() {
        ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}
unsafe extern "system" fn nego_DeleteSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    ntlm_DeleteSecurityContext(phContext)
}
unsafe extern "system" fn nego_ApplyControlToken(phContext: PCtxtHandle, pInput: PSecBufferDesc) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phContext.is_null() {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
unsafe extern "system" fn nego_QueryContextAttributesW(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    TRACE!(
        b"%p, %u, %p\n\0".as_ptr() as LPCSTR,
        phContext,
        ulAttribute,
        pBuffer,
    );

    match ulAttribute {
        SECPKG_ATTR_SIZES => {
            let sizes = pBuffer as *mut SecPkgContext_Sizes;
            (*sizes).cbMaxToken = 2888;
            (*sizes).cbMaxSignature = 16;
            (*sizes).cbSecurityTrailer = 16;
            (*sizes).cbBlockSize = 0;
            SEC_E_OK
        }
        SECPKG_ATTR_NEGOTIATION_INFO => {
            let info = pBuffer as *mut SecPkgContext_NegotiationInfoW;
            (*info).PackageInfo = ntlm_package_infoW;
            (*info).NegotiationState = SECPKG_NEGOTIATION_COMPLETE;
            SEC_E_OK
        }
        _ => ntlm_QueryContextAttributesW(phContext, ulAttribute, pBuffer),
    }
}
unsafe extern "system" fn nego_QueryContextAttributesA(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    TRACE!(b"%p, %u, %p\n\0".as_ptr() as LPCSTR, phContext, ulAttribute, pBuffer);

    match ulAttribute {
        SECPKG_ATTR_SIZES => {
            let sizes = pBuffer as *mut SecPkgContext_Sizes;
            (*sizes).cbMaxToken = 2888;
            (*sizes).cbMaxSignature = 16;
            (*sizes).cbSecurityTrailer = 16;
            (*sizes).cbBlockSize = 0;
            SEC_E_OK
        }
        SECPKG_ATTR_NEGOTIATION_INFO => {
            let info = pBuffer as *mut SecPkgContext_NegotiationInfoA;
            (*info).PackageInfo = ntlm_package_infoA;
            (*info).NegotiationState = SECPKG_NEGOTIATION_COMPLETE;
            SEC_E_OK
        }
        _ => ntlm_QueryContextAttributesA(phContext, ulAttribute, pBuffer),
    }
}
unsafe extern "system" fn nego_ImpersonateSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;
    if !phContext.is_null() {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
unsafe extern "system" fn nego_RevertSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phContext.is_null() {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
unsafe extern "system" fn nego_MakeSignature(
    phContext: PCtxtHandle,
    fQOP: ULONG,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
) -> SECURITY_STATUS {
    TRACE!(
        b"%p, 0x%08x, %p, %u\n\0".as_ptr() as LPCSTR,
        phContext,
        fQOP,
        pMessage,
        MessageSeqNo,
    );

    ntlm_MakeSignature(phContext, fQOP, pMessage, MessageSeqNo)
}
unsafe extern "system" fn nego_VerifySignature(
    phContext: PCtxtHandle,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
    pfQOP: PULONG,
) -> SECURITY_STATUS {
    TRACE!(
        b"%p, %p, %u, %p\n\0".as_ptr() as LPCSTR,
        phContext,
        pMessage,
        MessageSeqNo,
        pfQOP,
    );

    ntlm_VerifySignature(phContext, pMessage, MessageSeqNo, pfQOP)
}
unsafe extern "system" fn nego_FreeCredentialsHandle(phCredential: PCredHandle) -> SECURITY_STATUS {
    ntlm_FreeCredentialsHandle(phCredential)
}
unsafe extern "system" fn nego_EncryptMessage(
    phContext: PCtxtHandle,
    fQOP: ULONG,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
) -> SECURITY_STATUS {
    TRACE!(
        b"%p, 0x%08x, %p, %u\n\0".as_ptr() as LPCSTR,
        phContext,
        fQOP,
        pMessage,
        MessageSeqNo,
    );

    ntlm_EncryptMessage(phContext, fQOP, pMessage, MessageSeqNo)
}
unsafe extern "system" fn nego_DecryptMessage(
    phContext: PCtxtHandle,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
    pfQOP: PULONG,
) -> SECURITY_STATUS {
    TRACE!(
        b"%p, %p, %u, %p\n\0".as_ptr() as LPCSTR,
        phContext,
        pMessage,
        MessageSeqNo,
        pfQOP,
    );

    ntlm_DecryptMessage(phContext, pMessage, MessageSeqNo, pfQOP)
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_initNegotiateSP() {
    let provider: *mut SecureProvider = SECUR32_addProvider(
        core::ptr::addr_of!(negoTableA),
        core::ptr::addr_of!(negoTableW),
        core::ptr::null_mut(),
    );

    let infoW = SecPkgInfoW {
        fCapabilities: CAPS,
        wVersion: 1,
        wRPCID: RPC_C_AUTHN_GSS_NEGOTIATE as u16,
        cbMaxToken: NEGO_MAX_TOKEN,
        Name: nego_name_W.as_ptr() as *mut u16,
        Comment: negotiate_comment_W.as_ptr() as *mut u16,
    };
    let infoA = SecPkgInfoA {
        fCapabilities: CAPS,
        wVersion: 1,
        wRPCID: RPC_C_AUTHN_GSS_NEGOTIATE as u16,
        cbMaxToken: NEGO_MAX_TOKEN,
        Name: nego_name_A.as_ptr() as *mut i8,
        Comment: negotiate_comment_A.as_ptr() as *mut i8,
    };
    SECUR32_addPackages(provider, 1, core::ptr::addr_of!(infoA), core::ptr::addr_of!(infoW));
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2005 Kai Blin
//  * Copyright 2012 Hans Leidekker for CodeWeavers
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
//  *
//  * This file implements a Negotiate provider that simply forwards to
//  * the NTLM provider.
//  */
//
// #include "precomp.h"
//
// #include <wine/debug.h>
// WINE_DEFAULT_DEBUG_CHANNEL(secur32);
//
// /***********************************************************************
//  *              QueryCredentialsAttributesA
//  */
// static SECURITY_STATUS SEC_ENTRY nego_QueryCredentialsAttributesA(
//     PCredHandle phCredential, ULONG ulAttribute, PVOID pBuffer)
// {
//     FIXME("%p, %u, %p\n", phCredential, ulAttribute, pBuffer);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              QueryCredentialsAttributesW
//  */
// static SECURITY_STATUS SEC_ENTRY nego_QueryCredentialsAttributesW(
//     PCredHandle phCredential, ULONG ulAttribute, PVOID pBuffer)
// {
//     FIXME("%p, %u, %p\n", phCredential, ulAttribute, pBuffer);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              AcquireCredentialsHandleW
//  */
// static SECURITY_STATUS SEC_ENTRY nego_AcquireCredentialsHandleW(
//     SEC_WCHAR *pszPrincipal, SEC_WCHAR *pszPackage, ULONG fCredentialUse,
//     PLUID pLogonID, PVOID pAuthData, SEC_GET_KEY_FN pGetKeyFn,
//     PVOID pGetKeyArgument, PCredHandle phCredential, PTimeStamp ptsExpiry )
// {
//     static SEC_WCHAR ntlmW[] = {'N','T','L','M',0};
//     SECURITY_STATUS ret;
//
//     TRACE("%s, %s, 0x%08x, %p, %p, %p, %p, %p, %p\n",
//           debugstr_w(pszPrincipal), debugstr_w(pszPackage), fCredentialUse,
//           pLogonID, pAuthData, pGetKeyFn, pGetKeyArgument, phCredential, ptsExpiry);
//
//     FIXME("forwarding to NTLM\n");
//     ret = ntlm_AcquireCredentialsHandleW( pszPrincipal, ntlmW, fCredentialUse,
//                                           pLogonID, pAuthData, pGetKeyFn, pGetKeyArgument,
//                                           phCredential, ptsExpiry );
//     if (ret == SEC_E_OK)
//     {
//         NtlmCredentials *cred = (NtlmCredentials *)phCredential->dwLower;
//         cred->no_cached_credentials = (pAuthData == NULL);
//     }
//     return ret;
// }
//
// /***********************************************************************
//  *              AcquireCredentialsHandleA
//  */
// static SECURITY_STATUS SEC_ENTRY nego_AcquireCredentialsHandleA(
//     SEC_CHAR *pszPrincipal, SEC_CHAR *pszPackage, ULONG fCredentialUse,
//     PLUID pLogonID, PVOID pAuthData, SEC_GET_KEY_FN pGetKeyFn,
//     PVOID pGetKeyArgument, PCredHandle phCredential, PTimeStamp ptsExpiry )
// {
//     SECURITY_STATUS ret = SEC_E_INSUFFICIENT_MEMORY;
//     SEC_WCHAR *user = NULL, *domain = NULL, *passwd = NULL, *package = NULL;
//     SEC_WINNT_AUTH_IDENTITY_W *identityW = NULL;
//
//     TRACE("%s, %s, 0x%08x, %p, %p, %p, %p, %p, %p\n",
//           debugstr_a(pszPrincipal), debugstr_a(pszPackage), fCredentialUse,
//           pLogonID, pAuthData, pGetKeyFn, pGetKeyArgument, phCredential, ptsExpiry);
//
//     if (pszPackage)
//     {
//         int package_len = MultiByteToWideChar( CP_ACP, 0, pszPackage, -1, NULL, 0 );
//         package = HeapAlloc( GetProcessHeap(), 0, package_len * sizeof(SEC_WCHAR) );
//         if (!package) return SEC_E_INSUFFICIENT_MEMORY;
//         MultiByteToWideChar( CP_ACP, 0, pszPackage, -1, package, package_len );
//     }
//     if (pAuthData)
//     {
//         SEC_WINNT_AUTH_IDENTITY_A *identity = pAuthData;
//         int user_len, domain_len, passwd_len;
//
//         if (identity->Flags == SEC_WINNT_AUTH_IDENTITY_ANSI)
//         {
//             identityW = HeapAlloc( GetProcessHeap(), 0, sizeof(*identityW) );
//             if (!identityW) goto done;
//
//             if (!identity->UserLength) user_len = 0;
//             else
//             {
//                 user_len = MultiByteToWideChar( CP_ACP, 0, (LPCSTR)identity->User,
//                                                 identity->UserLength, NULL, 0 );
//                 user = HeapAlloc( GetProcessHeap(), 0, user_len * sizeof(SEC_WCHAR) );
//                 if (!user) goto done;
//                 MultiByteToWideChar( CP_ACP, 0, (LPCSTR)identity->User, identity->UserLength,
//                                      user, user_len );
//             }
//             if (!identity->DomainLength) domain_len = 0;
//             else
//             {
//                 domain_len = MultiByteToWideChar( CP_ACP, 0, (LPCSTR)identity->Domain,
//                                                   identity->DomainLength, NULL, 0 );
//                 domain = HeapAlloc( GetProcessHeap(), 0, domain_len * sizeof(SEC_WCHAR) );
//                 if (!domain) goto done;
//                 MultiByteToWideChar( CP_ACP, 0, (LPCSTR)identity->Domain, identity->DomainLength,
//                                      domain, domain_len );
//             }
//             if (!identity->PasswordLength) passwd_len = 0;
//             else
//             {
//                 passwd_len = MultiByteToWideChar( CP_ACP, 0, (LPCSTR)identity->Password,
//                                                   identity->PasswordLength, NULL, 0 );
//                 passwd = HeapAlloc( GetProcessHeap(), 0, passwd_len * sizeof(SEC_WCHAR) );
//                 if (!passwd) goto done;
//                 MultiByteToWideChar( CP_ACP, 0, (LPCSTR)identity->Password, identity->PasswordLength,
//                                      passwd, passwd_len );
//             }
//             identityW->Flags          = SEC_WINNT_AUTH_IDENTITY_UNICODE;
//             identityW->User           = user;
//             identityW->UserLength     = user_len;
//             identityW->Domain         = domain;
//             identityW->DomainLength   = domain_len;
//             identityW->Password       = passwd;
//             identityW->PasswordLength = passwd_len;
//         }
//         else identityW = (SEC_WINNT_AUTH_IDENTITY_W *)identity;
//     }
//     ret = nego_AcquireCredentialsHandleW( NULL, package, fCredentialUse, pLogonID, identityW,
//                                           pGetKeyFn, pGetKeyArgument, phCredential, ptsExpiry );
// done:
//     HeapFree( GetProcessHeap(), 0, package );
//     HeapFree( GetProcessHeap(), 0, user );
//     HeapFree( GetProcessHeap(), 0, domain );
//     HeapFree( GetProcessHeap(), 0, passwd );
//     HeapFree( GetProcessHeap(), 0, identityW );
//     return ret;
// }
//
// /***********************************************************************
//  *              InitializeSecurityContextW
//  */
// static SECURITY_STATUS SEC_ENTRY nego_InitializeSecurityContextW(
//     PCredHandle phCredential, PCtxtHandle phContext, SEC_WCHAR *pszTargetName,
//     ULONG fContextReq, ULONG Reserved1, ULONG TargetDataRep,
//     PSecBufferDesc pInput, ULONG Reserved2, PCtxtHandle phNewContext,
//     PSecBufferDesc pOutput, ULONG *pfContextAttr, PTimeStamp ptsExpiry )
// {
//     TRACE("%p, %p, %s, 0x%08x, %u, %u, %p, %u, %p, %p, %p, %p\n",
//           phCredential, phContext, debugstr_w(pszTargetName), fContextReq,
//           Reserved1, TargetDataRep, pInput, Reserved1, phNewContext, pOutput,
//           pfContextAttr, ptsExpiry);
//
//     return ntlm_InitializeSecurityContextW( phCredential, phContext, pszTargetName,
//                                             fContextReq, Reserved1, TargetDataRep,
//                                             pInput, Reserved2, phNewContext,
//                                             pOutput, pfContextAttr, ptsExpiry );
// }
//
// /***********************************************************************
//  *              InitializeSecurityContextA
//  */
// static SECURITY_STATUS SEC_ENTRY nego_InitializeSecurityContextA(
//     PCredHandle phCredential, PCtxtHandle phContext, SEC_CHAR *pszTargetName,
//     ULONG fContextReq, ULONG Reserved1, ULONG TargetDataRep,
//     PSecBufferDesc pInput, ULONG Reserved2, PCtxtHandle phNewContext,
//     PSecBufferDesc pOutput, ULONG *pfContextAttr, PTimeStamp ptsExpiry )
// {
//     SECURITY_STATUS ret;
//     SEC_WCHAR *target = NULL;
//
//     TRACE("%p, %p, %s, 0x%08x, %u, %u, %p, %u, %p, %p, %p, %p\n",
//           phCredential, phContext, debugstr_a(pszTargetName), fContextReq,
//           Reserved1, TargetDataRep, pInput, Reserved1, phNewContext, pOutput,
//           pfContextAttr, ptsExpiry);
//
//     if (pszTargetName)
//     {
//         int target_len = MultiByteToWideChar( CP_ACP, 0, pszTargetName, -1, NULL, 0 );
//         target = HeapAlloc(GetProcessHeap(), 0, target_len * sizeof(SEC_WCHAR) );
//         if (!target) return SEC_E_INSUFFICIENT_MEMORY;
//         MultiByteToWideChar( CP_ACP, 0, pszTargetName, -1, target, target_len );
//     }
//     ret = nego_InitializeSecurityContextW( phCredential, phContext, target, fContextReq,
//                                            Reserved1, TargetDataRep, pInput, Reserved2,
//                                            phNewContext, pOutput, pfContextAttr, ptsExpiry );
//     HeapFree( GetProcessHeap(), 0, target );
//     return ret;
// }
//
// /***********************************************************************
//  *              AcceptSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY nego_AcceptSecurityContext(
//     PCredHandle phCredential, PCtxtHandle phContext, PSecBufferDesc pInput,
//     ULONG fContextReq, ULONG TargetDataRep, PCtxtHandle phNewContext,
//     PSecBufferDesc pOutput, ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     TRACE("%p, %p, %p, 0x%08x, %u, %p, %p, %p, %p\n", phCredential, phContext,
//           pInput, fContextReq, TargetDataRep, phNewContext, pOutput, pfContextAttr,
//           ptsExpiry);
//
//     return ntlm_AcceptSecurityContext( phCredential, phContext, pInput,
//                                        fContextReq, TargetDataRep, phNewContext,
//                                        pOutput, pfContextAttr, ptsExpiry );
// }
//
// /***********************************************************************
//  *              CompleteAuthToken
//  */
// static SECURITY_STATUS SEC_ENTRY nego_CompleteAuthToken(PCtxtHandle phContext,
//  PSecBufferDesc pToken)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p\n", phContext, pToken);
//     if (phContext)
//     {
//         ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//     {
//         ret = SEC_E_INVALID_HANDLE;
//     }
//     return ret;
// }
//
// /***********************************************************************
//  *              DeleteSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY nego_DeleteSecurityContext(PCtxtHandle phContext)
// {
//     TRACE("%p\n", phContext);
//
//     return ntlm_DeleteSecurityContext( phContext );
// }
//
// /***********************************************************************
//  *              ApplyControlToken
//  */
// static SECURITY_STATUS SEC_ENTRY nego_ApplyControlToken(PCtxtHandle phContext,
//  PSecBufferDesc pInput)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p\n", phContext, pInput);
//     if (phContext)
//     {
//         ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//     {
//         ret = SEC_E_INVALID_HANDLE;
//     }
//     return ret;
// }
//
// /***********************************************************************
//  *              QueryContextAttributesW
//  */
// static SECURITY_STATUS SEC_ENTRY nego_QueryContextAttributesW(
//     PCtxtHandle phContext, ULONG ulAttribute, void *pBuffer)
// {
//     TRACE("%p, %u, %p\n", phContext, ulAttribute, pBuffer);
//
//     switch (ulAttribute)
//     {
//     case SECPKG_ATTR_SIZES:
//     {
//         SecPkgContext_Sizes *sizes = (SecPkgContext_Sizes *)pBuffer;
//         sizes->cbMaxToken        = 2888;
//         sizes->cbMaxSignature    = 16;
//         sizes->cbSecurityTrailer = 16;
//         sizes->cbBlockSize       = 0;
//         return SEC_E_OK;
//     }
//     case SECPKG_ATTR_NEGOTIATION_INFO:
//     {
//         SecPkgContext_NegotiationInfoW *info = (SecPkgContext_NegotiationInfoW *)pBuffer;
//         info->PackageInfo      = ntlm_package_infoW;
//         info->NegotiationState = SECPKG_NEGOTIATION_COMPLETE;
//         return SEC_E_OK;
//     }
//     default:
//         return ntlm_QueryContextAttributesW( phContext, ulAttribute, pBuffer );
//     }
// }
//
// /***********************************************************************
//  *              QueryContextAttributesA
//  */
// static SECURITY_STATUS SEC_ENTRY nego_QueryContextAttributesA(PCtxtHandle phContext,
//  ULONG ulAttribute, void *pBuffer)
// {
//     TRACE("%p, %u, %p\n", phContext, ulAttribute, pBuffer);
//
//     switch (ulAttribute)
//     {
//     case SECPKG_ATTR_SIZES:
//     {
//         SecPkgContext_Sizes *sizes = (SecPkgContext_Sizes *)pBuffer;
//         sizes->cbMaxToken        = 2888;
//         sizes->cbMaxSignature    = 16;
//         sizes->cbSecurityTrailer = 16;
//         sizes->cbBlockSize       = 0;
//         return SEC_E_OK;
//     }
//     case SECPKG_ATTR_NEGOTIATION_INFO:
//     {
//         SecPkgContext_NegotiationInfoA *info = (SecPkgContext_NegotiationInfoA *)pBuffer;
//         info->PackageInfo      = ntlm_package_infoA;
//         info->NegotiationState = SECPKG_NEGOTIATION_COMPLETE;
//         return SEC_E_OK;
//     }
//     default:
//         return ntlm_QueryContextAttributesA( phContext, ulAttribute, pBuffer );
//     }
// }
//
// /***********************************************************************
//  *              ImpersonateSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY nego_ImpersonateSecurityContext(PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p\n", phContext);
//     if (phContext)
//     {
//         ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//     {
//         ret = SEC_E_INVALID_HANDLE;
//     }
//     return ret;
// }
//
// /***********************************************************************
//  *              RevertSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY nego_RevertSecurityContext(PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p\n", phContext);
//     if (phContext)
//     {
//         ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//     {
//         ret = SEC_E_INVALID_HANDLE;
//     }
//     return ret;
// }
//
// /***********************************************************************
//  *              MakeSignature
//  */
// static SECURITY_STATUS SEC_ENTRY nego_MakeSignature(PCtxtHandle phContext,
//     ULONG fQOP, PSecBufferDesc pMessage, ULONG MessageSeqNo)
// {
//     TRACE("%p, 0x%08x, %p, %u\n", phContext, fQOP, pMessage, MessageSeqNo);
//
//     return ntlm_MakeSignature( phContext, fQOP, pMessage, MessageSeqNo );
// }
//
// /***********************************************************************
//  *              VerifySignature
//  */
// static SECURITY_STATUS SEC_ENTRY nego_VerifySignature(PCtxtHandle phContext,
//     PSecBufferDesc pMessage, ULONG MessageSeqNo, PULONG pfQOP)
// {
//     TRACE("%p, %p, %u, %p\n", phContext, pMessage, MessageSeqNo, pfQOP);
//
//     return ntlm_VerifySignature( phContext, pMessage, MessageSeqNo, pfQOP );
// }
//
// /***********************************************************************
//  *             FreeCredentialsHandle
//  */
// static SECURITY_STATUS SEC_ENTRY nego_FreeCredentialsHandle(PCredHandle phCredential)
// {
//     TRACE("%p\n", phCredential);
//
//     return ntlm_FreeCredentialsHandle( phCredential );
// }
//
// /***********************************************************************
//  *             EncryptMessage
//  */
// static SECURITY_STATUS SEC_ENTRY nego_EncryptMessage(PCtxtHandle phContext,
//     ULONG fQOP, PSecBufferDesc pMessage, ULONG MessageSeqNo)
// {
//     TRACE("%p, 0x%08x, %p, %u\n", phContext, fQOP, pMessage, MessageSeqNo);
//
//     return ntlm_EncryptMessage( phContext, fQOP, pMessage, MessageSeqNo );
// }
//
// /***********************************************************************
//  *             DecryptMessage
//  */
// static SECURITY_STATUS SEC_ENTRY nego_DecryptMessage(PCtxtHandle phContext,
//     PSecBufferDesc pMessage, ULONG MessageSeqNo, PULONG pfQOP)
// {
//     TRACE("%p, %p, %u, %p\n", phContext, pMessage, MessageSeqNo, pfQOP);
//
//     return ntlm_DecryptMessage( phContext, pMessage, MessageSeqNo, pfQOP );
// }
//
// static const SecurityFunctionTableA negoTableA = {
//     1,
//     NULL,                               /* EnumerateSecurityPackagesA */
//     nego_QueryCredentialsAttributesA,   /* QueryCredentialsAttributesA */
//     nego_AcquireCredentialsHandleA,     /* AcquireCredentialsHandleA */
//     nego_FreeCredentialsHandle,         /* FreeCredentialsHandle */
//     NULL,                               /* Reserved2 */
//     nego_InitializeSecurityContextA,    /* InitializeSecurityContextA */
//     nego_AcceptSecurityContext,         /* AcceptSecurityContext */
//     nego_CompleteAuthToken,             /* CompleteAuthToken */
//     nego_DeleteSecurityContext,         /* DeleteSecurityContext */
//     nego_ApplyControlToken,             /* ApplyControlToken */
//     nego_QueryContextAttributesA,       /* QueryContextAttributesA */
//     nego_ImpersonateSecurityContext,    /* ImpersonateSecurityContext */
//     nego_RevertSecurityContext,         /* RevertSecurityContext */
//     nego_MakeSignature,                 /* MakeSignature */
//     nego_VerifySignature,               /* VerifySignature */
//     FreeContextBuffer,                  /* FreeContextBuffer */
//     NULL,   /* QuerySecurityPackageInfoA */
//     NULL,   /* Reserved3 */
//     NULL,   /* Reserved4 */
//     NULL,   /* ExportSecurityContext */
//     NULL,   /* ImportSecurityContextA */
//     NULL,   /* AddCredentialsA */
//     NULL,   /* Reserved8 */
//     NULL,   /* QuerySecurityContextToken */
//     nego_EncryptMessage,                /* EncryptMessage */
//     nego_DecryptMessage,                /* DecryptMessage */
//     NULL,   /* SetContextAttributesA */
// };
//
// static const SecurityFunctionTableW negoTableW = {
//     1,
//     NULL,                               /* EnumerateSecurityPackagesW */
//     nego_QueryCredentialsAttributesW,   /* QueryCredentialsAttributesW */
//     nego_AcquireCredentialsHandleW,     /* AcquireCredentialsHandleW */
//     nego_FreeCredentialsHandle,         /* FreeCredentialsHandle */
//     NULL,                               /* Reserved2 */
//     nego_InitializeSecurityContextW,    /* InitializeSecurityContextW */
//     nego_AcceptSecurityContext,         /* AcceptSecurityContext */
//     nego_CompleteAuthToken,             /* CompleteAuthToken */
//     nego_DeleteSecurityContext,         /* DeleteSecurityContext */
//     nego_ApplyControlToken,             /* ApplyControlToken */
//     nego_QueryContextAttributesW,       /* QueryContextAttributesW */
//     nego_ImpersonateSecurityContext,    /* ImpersonateSecurityContext */
//     nego_RevertSecurityContext,         /* RevertSecurityContext */
//     nego_MakeSignature,                 /* MakeSignature */
//     nego_VerifySignature,               /* VerifySignature */
//     FreeContextBuffer,                  /* FreeContextBuffer */
//     NULL,   /* QuerySecurityPackageInfoW */
//     NULL,   /* Reserved3 */
//     NULL,   /* Reserved4 */
//     NULL,   /* ExportSecurityContext */
//     NULL,   /* ImportSecurityContextW */
//     NULL,   /* AddCredentialsW */
//     NULL,   /* Reserved8 */
//     NULL,   /* QuerySecurityContextToken */
//     nego_EncryptMessage,                /* EncryptMessage */
//     nego_DecryptMessage,                /* DecryptMessage */
//     NULL,   /* SetContextAttributesW */
// };
//
// #define NEGO_MAX_TOKEN 12000
//
// static WCHAR nego_name_W[] = {'N','e','g','o','t','i','a','t','e',0};
// static char nego_name_A[] = "Negotiate";
//
// static WCHAR negotiate_comment_W[] =
//     {'M','i','c','r','o','s','o','f','t',' ','P','a','c','k','a','g','e',' ',
//      'N','e','g','o','t','i','a','t','o','r',0};
// static CHAR negotiate_comment_A[] = "Microsoft Package Negotiator";
//
// #define CAPS ( \
//     SECPKG_FLAG_INTEGRITY  | \
//     SECPKG_FLAG_PRIVACY    | \
//     SECPKG_FLAG_CONNECTION | \
//     SECPKG_FLAG_MULTI_REQUIRED | \
//     SECPKG_FLAG_EXTENDED_ERROR | \
//     SECPKG_FLAG_IMPERSONATION  | \
//     SECPKG_FLAG_ACCEPT_WIN32_NAME | \
//     SECPKG_FLAG_NEGOTIABLE        | \
//     SECPKG_FLAG_GSS_COMPATIBLE    | \
//     SECPKG_FLAG_LOGON             | \
//     SECPKG_FLAG_RESTRICTED_TOKENS )
//
// void SECUR32_initNegotiateSP(void)
// {
//     SecureProvider *provider = SECUR32_addProvider(&negoTableA, &negoTableW, NULL);
//
//     const SecPkgInfoW infoW = {CAPS, 1, RPC_C_AUTHN_GSS_NEGOTIATE, NEGO_MAX_TOKEN,
//                                nego_name_W, negotiate_comment_W};
//     const SecPkgInfoA infoA = {CAPS, 1, RPC_C_AUTHN_GSS_NEGOTIATE, NEGO_MAX_TOKEN,
//                                nego_name_A, negotiate_comment_A};
//     SECUR32_addPackages(provider, 1L, &infoA, &infoW);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
