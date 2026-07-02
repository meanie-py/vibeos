use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn SECUR32_addProvider( fnTableA: *const SecurityFunctionTableA, fnTableW: *const SecurityFunctionTableW, moduleName: PWSTR, ) -> *mut SecureProvider;
    fn SECUR32_addPackages( provider: *mut SecureProvider, toAdd: ULONG, infoA: *const SecPkgInfoA, infoW: *const SecPkgInfoW, );
}

// ---- file-scope preamble (translated) ----
pub const KERBEROS_MAX_BUF: u32 = 12000;

pub static mut kerberosTableA: SecurityFunctionTableA = SecurityFunctionTableA {
    dwVersion: 1,
    EnumerateSecurityPackagesA: None,
    QueryCredentialsAttributesA: sec_entry!(kerberos_QueryCredentialsAttributesA),
    AcquireCredentialsHandleA: sec_entry!(kerberos_AcquireCredentialsHandleA),
    FreeCredentialsHandle: sec_entry!(kerberos_FreeCredentialsHandle),
    Reserved2: core::ptr::null_mut(),
    InitializeSecurityContextA: sec_entry!(kerberos_InitializeSecurityContextA),
    AcceptSecurityContext: sec_entry!(kerberos_AcceptSecurityContext),
    CompleteAuthToken: sec_entry!(kerberos_CompleteAuthToken),
    DeleteSecurityContext: sec_entry!(kerberos_DeleteSecurityContext),
    ApplyControlToken: None,
    QueryContextAttributesA: sec_entry!(kerberos_QueryContextAttributesA),
    ImpersonateSecurityContext: sec_entry!(kerberos_ImpersonateSecurityContext),
    RevertSecurityContext: sec_entry!(kerberos_RevertSecurityContext),
    MakeSignature: sec_entry!(kerberos_MakeSignature),
    VerifySignature: sec_entry!(kerberos_VerifySignature),
    FreeContextBuffer: sec_entry!(FreeContextBuffer),
    QuerySecurityPackageInfoA: None,
    Reserved3: core::ptr::null_mut(),
    Reserved4: core::ptr::null_mut(),
    ExportSecurityContext: None,
    ImportSecurityContextA: None,
    AddCredentialsA: None,
    Reserved8: core::ptr::null_mut(),
    QuerySecurityContextToken: None,
    EncryptMessage: sec_entry!(kerberos_EncryptMessage),
    DecryptMessage: sec_entry!(kerberos_DecryptMessage),
    SetContextAttributesA: None,
};

pub static mut kerberosTableW: SecurityFunctionTableW = SecurityFunctionTableW {
    dwVersion: 1,
    EnumerateSecurityPackagesW: None,
    QueryCredentialsAttributesW: sec_entry!(kerberos_QueryCredentialsAttributesW),
    AcquireCredentialsHandleW: sec_entry!(kerberos_AcquireCredentialsHandleW),
    FreeCredentialsHandle: sec_entry!(kerberos_FreeCredentialsHandle),
    Reserved2: core::ptr::null_mut(),
    InitializeSecurityContextW: sec_entry!(kerberos_InitializeSecurityContextW),
    AcceptSecurityContext: sec_entry!(kerberos_AcceptSecurityContext),
    CompleteAuthToken: sec_entry!(kerberos_CompleteAuthToken),
    DeleteSecurityContext: sec_entry!(kerberos_DeleteSecurityContext),
    ApplyControlToken: None,
    QueryContextAttributesW: sec_entry!(kerberos_QueryContextAttributesW),
    ImpersonateSecurityContext: sec_entry!(kerberos_ImpersonateSecurityContext),
    RevertSecurityContext: sec_entry!(kerberos_RevertSecurityContext),
    MakeSignature: sec_entry!(kerberos_MakeSignature),
    VerifySignature: sec_entry!(kerberos_VerifySignature),
    FreeContextBuffer: sec_entry!(FreeContextBuffer),
    QuerySecurityPackageInfoW: None,
    Reserved3: core::ptr::null_mut(),
    Reserved4: core::ptr::null_mut(),
    ExportSecurityContext: None,
    ImportSecurityContextW: None,
    AddCredentialsW: None,
    Reserved8: core::ptr::null_mut(),
    QuerySecurityContextToken: None,
    EncryptMessage: sec_entry!(kerberos_EncryptMessage),
    DecryptMessage: sec_entry!(kerberos_DecryptMessage),
    SetContextAttributesW: None,
};

pub static mut kerberos_comment_A: [CHAR; 24] = [
    b'M' as CHAR, b'i' as CHAR, b'c' as CHAR, b'r' as CHAR, b'o' as CHAR, b's' as CHAR,
    b'o' as CHAR, b'f' as CHAR, b't' as CHAR, b' ' as CHAR, b'K' as CHAR, b'e' as CHAR,
    b'r' as CHAR, b'b' as CHAR, b'e' as CHAR, b'r' as CHAR, b'o' as CHAR, b's' as CHAR,
    b' ' as CHAR, b'V' as CHAR, b'1' as CHAR, b'.' as CHAR, b'0' as CHAR, 0,
];

pub static mut kerberos_comment_W: [WCHAR; 24] = [
    b'M' as WCHAR, b'i' as WCHAR, b'c' as WCHAR, b'r' as WCHAR, b'o' as WCHAR, b's' as WCHAR,
    b'o' as WCHAR, b'f' as WCHAR, b't' as WCHAR, b' ' as WCHAR, b'K' as WCHAR, b'e' as WCHAR,
    b'r' as WCHAR, b'b' as WCHAR, b'e' as WCHAR, b'r' as WCHAR, b'o' as WCHAR, b's' as WCHAR,
    b' ' as WCHAR, b'V' as WCHAR, b'1' as WCHAR, b'.' as WCHAR, b'0' as WCHAR, 0,
];

pub static mut kerberos_name_A: [::core::ffi::c_char; 9] = [
    b'K' as ::core::ffi::c_char, b'e' as ::core::ffi::c_char, b'r' as ::core::ffi::c_char,
    b'b' as ::core::ffi::c_char, b'e' as ::core::ffi::c_char, b'r' as ::core::ffi::c_char,
    b'o' as ::core::ffi::c_char, b's' as ::core::ffi::c_char, 0,
];

pub static mut kerberos_name_W: [WCHAR; 9] = [
    b'K' as WCHAR, b'e' as WCHAR, b'r' as WCHAR, b'b' as WCHAR, b'e' as WCHAR,
    b'r' as WCHAR, b'o' as WCHAR, b's' as WCHAR, 0,
];

pub const CAPS: u32 = SECPKG_FLAG_INTEGRITY
    | SECPKG_FLAG_PRIVACY
    | SECPKG_FLAG_TOKEN_ONLY
    | SECPKG_FLAG_DATAGRAM
    | SECPKG_FLAG_CONNECTION
    | SECPKG_FLAG_MULTI_REQUIRED
    | SECPKG_FLAG_EXTENDED_ERROR
    | SECPKG_FLAG_IMPERSONATION
    | SECPKG_FLAG_ACCEPT_WIN32_NAME
    | SECPKG_FLAG_NEGOTIABLE
    | SECPKG_FLAG_GSS_COMPATIBLE
    | SECPKG_FLAG_LOGON
    | SECPKG_FLAG_MUTUAL_AUTH
    | SECPKG_FLAG_DELEGATION
    | SECPKG_FLAG_READONLY_WITH_CHECKSUM
    | SECPKG_FLAG_RESTRICTED_TOKENS
    | SECPKG_FLAG_APPCONTAINER_CHECKS;

pub static mut infoW: SecPkgInfoW = SecPkgInfoW {
    fCapabilities: CAPS,
    wVersion: 1,
    wRPCID: RPC_C_AUTHN_GSS_KERBEROS as u16,
    cbMaxToken: KERBEROS_MAX_BUF,
    Name: unsafe { kerberos_name_W.as_ptr() as *mut WCHAR },
    Comment: unsafe { kerberos_comment_W.as_ptr() as *mut WCHAR },
};

pub static mut infoA: SecPkgInfoA = SecPkgInfoA {
    fCapabilities: CAPS,
    wVersion: 1,
    wRPCID: RPC_C_AUTHN_GSS_KERBEROS as u16,
    cbMaxToken: KERBEROS_MAX_BUF,
    Name: unsafe { kerberos_name_A.as_ptr() as *mut ::core::ffi::c_char },
    Comment: unsafe { kerberos_comment_A.as_ptr() as *mut CHAR },
};
unsafe extern "system" fn kerberos_QueryCredentialsAttributesA(
    phCredential: *mut CredHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_QueryCredentialsAttributesW(
    phCredential: *mut CredHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_AcquireCredentialsHandleW(
    pszPrincipal: *mut SEC_WCHAR,
    pszPackage: *mut SEC_WCHAR,
    fCredentialUse: ULONG,
    pLogonID: *mut LUID,
    pAuthData: *mut core::ffi::c_void,
    pGetKeyFn: SEC_GET_KEY_FN,
    pGetKeyArgument: *mut core::ffi::c_void,
    phCredential: *mut CredHandle,
    ptsExpiry: *mut TimeStamp,
) -> SECURITY_STATUS {
    SEC_E_NO_CREDENTIALS
}
unsafe extern "system" fn kerberos_AcquireCredentialsHandleA(
    pszPrincipal: *mut SEC_CHAR,
    pszPackage: *mut SEC_CHAR,
    fCredentialUse: ULONG,
    pLogonID: *mut LUID,
    pAuthData: *mut core::ffi::c_void,
    pGetKeyFn: SEC_GET_KEY_FN,
    pGetKeyArgument: *mut core::ffi::c_void,
    phCredential: *mut CredHandle,
    ptsExpiry: *mut TimeStamp,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_InitializeSecurityContextW(
    phCredential: *mut CredHandle,
    phContext: *mut CtxtHandle,
    pszTargetName: *mut SEC_WCHAR,
    fContextReq: ULONG,
    Reserved1: ULONG,
    TargetDataRep: ULONG,
    pInput: *mut SecBufferDesc,
    Reserved2: ULONG,
    phNewContext: *mut CtxtHandle,
    pOutput: *mut SecBufferDesc,
    pfContextAttr: *mut ULONG,
    ptsExpiry: *mut TimeStamp,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_InitializeSecurityContextA(
    phCredential: *mut CredHandle,
    phContext: *mut CtxtHandle,
    pszTargetName: *mut SEC_CHAR,
    fContextReq: ULONG,
    Reserved1: ULONG,
    TargetDataRep: ULONG,
    pInput: *mut SecBufferDesc,
    Reserved2: ULONG,
    phNewContext: *mut CtxtHandle,
    pOutput: *mut SecBufferDesc,
    pfContextAttr: *mut ULONG,
    ptsExpiry: *mut TimeStamp,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_AcceptSecurityContext(
    phCredential: *mut CredHandle,
    phContext: *mut CtxtHandle,
    pInput: *mut SecBufferDesc,
    fContextReq: ULONG,
    TargetDataRep: ULONG,
    phNewContext: *mut CtxtHandle,
    pOutput: *mut SecBufferDesc,
    pfContextAttr: *mut ULONG,
    ptsExpiry: *mut TimeStamp,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_CompleteAuthToken(phContext: *mut CtxtHandle, pToken: *mut SecBufferDesc) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_DeleteSecurityContext(phContext: *mut CtxtHandle) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_QueryContextAttributesW(phContext: *mut CtxtHandle, ulAttribute: ULONG, pBuffer: *mut core::ffi::c_void) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_QueryContextAttributesA(phContext: *mut CtxtHandle, ulAttribute: ULONG, pBuffer: *mut core::ffi::c_void) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_ImpersonateSecurityContext(phContext: *mut CtxtHandle) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_RevertSecurityContext(phContext: *mut CtxtHandle) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_MakeSignature(phContext: *mut CtxtHandle, fQOP: ULONG, pMessage: *mut SecBufferDesc, MessageSeqNo: ULONG) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_VerifySignature(phContext: *mut CtxtHandle, pMessage: *mut SecBufferDesc, MessageSeqNo: ULONG, pfQOP: PULONG) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_FreeCredentialsHandle(phCredential: PCredHandle) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_EncryptMessage(phContext: *mut CtxtHandle, fQOP: ULONG, pMessage: *mut SecBufferDesc, MessageSeqNo: ULONG) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
unsafe extern "system" fn kerberos_DecryptMessage(phContext: *mut CtxtHandle, pMessage: *mut SecBufferDesc, MessageSeqNo: ULONG, pfQOP: PULONG) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_initKerberosSP() {
    let provider: *mut SecureProvider = SECUR32_addProvider(
        &kerberosTableA,
        &kerberosTableW,
        core::ptr::null_mut(),
    );
    SECUR32_addPackages(provider, 1, &infoA, &infoW);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2005, 2006 Kai Blin
//  * Copyright 2016 Jacek Caban for CodeWeavers
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
// #include "precomp.h"
//
// #include <assert.h>
//
// #include <wine/debug.h>
// WINE_DEFAULT_DEBUG_CHANNEL(secur32);
//
// #define KERBEROS_MAX_BUF 12000
//
// /***********************************************************************
//  *              QueryCredentialsAttributesA
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_QueryCredentialsAttributesA(CredHandle *phCredential, ULONG ulAttribute, void *pBuffer)
// {
//     FIXME("(%p %d %p)\n", phCredential, ulAttribute, pBuffer);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              QueryCredentialsAttributesW
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_QueryCredentialsAttributesW(CredHandle *phCredential, ULONG ulAttribute, void *pBuffer)
// {
//     FIXME("(%p, %d, %p)\n", phCredential, ulAttribute, pBuffer);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              AcquireCredentialsHandleW
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_AcquireCredentialsHandleW(SEC_WCHAR *pszPrincipal, SEC_WCHAR *pszPackage, ULONG fCredentialUse,
//         LUID *pLogonID, void *pAuthData, SEC_GET_KEY_FN pGetKeyFn, void *pGetKeyArgument, CredHandle *phCredential, TimeStamp *ptsExpiry)
// {
//     FIXME("(%s %s 0x%08x %p %p %p %p %p %p)\n", debugstr_w(pszPrincipal), debugstr_w(pszPackage), fCredentialUse,
//           pLogonID, pAuthData, pGetKeyFn, pGetKeyArgument, phCredential, ptsExpiry);
//     return SEC_E_NO_CREDENTIALS;
// }
//
// /***********************************************************************
//  *              AcquireCredentialsHandleA
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_AcquireCredentialsHandleA(SEC_CHAR *pszPrincipal, SEC_CHAR *pszPackage, ULONG fCredentialUse,
//         LUID *pLogonID, void *pAuthData, SEC_GET_KEY_FN pGetKeyFn, void *pGetKeyArgument, CredHandle *phCredential, TimeStamp *ptsExpiry)
// {
//     FIXME("(%s %s 0x%08x %p %p %p %p %p %p)\n", debugstr_a(pszPrincipal), debugstr_a(pszPackage), fCredentialUse,
//           pLogonID, pAuthData, pGetKeyFn, pGetKeyArgument, phCredential, ptsExpiry);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              InitializeSecurityContextW
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_InitializeSecurityContextW(CredHandle *phCredential, CtxtHandle *phContext, SEC_WCHAR *pszTargetName,
//         ULONG fContextReq, ULONG Reserved1, ULONG TargetDataRep, SecBufferDesc *pInput, ULONG Reserved2, CtxtHandle *phNewContext,
//         SecBufferDesc *pOutput, ULONG *pfContextAttr, TimeStamp *ptsExpiry)
// {
//     FIXME("(%p %p %s 0x%08x %d %d %p %d %p %p %p %p)\n", phCredential, phContext, debugstr_w(pszTargetName),
//           fContextReq, Reserved1, TargetDataRep, pInput, Reserved1, phNewContext, pOutput, pfContextAttr, ptsExpiry);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              InitializeSecurityContextA
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_InitializeSecurityContextA(CredHandle *phCredential, CtxtHandle *phContext, SEC_CHAR *pszTargetName,
//         ULONG fContextReq, ULONG Reserved1, ULONG TargetDataRep, SecBufferDesc *pInput, ULONG Reserved2, CtxtHandle *phNewContext,
//         SecBufferDesc *pOutput, ULONG *pfContextAttr, TimeStamp *ptsExpiry)
// {
//     FIXME("%p %p %s %d %d %d %p %d %p %p %p %p\n", phCredential, phContext, debugstr_a(pszTargetName), fContextReq,
//           Reserved1, TargetDataRep, pInput, Reserved1, phNewContext, pOutput, pfContextAttr, ptsExpiry);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              AcceptSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_AcceptSecurityContext(CredHandle *phCredential, CtxtHandle *phContext, SecBufferDesc *pInput,
//         ULONG fContextReq, ULONG TargetDataRep, CtxtHandle *phNewContext, SecBufferDesc *pOutput, ULONG *pfContextAttr, TimeStamp *ptsExpiry)
// {
//     FIXME("(%p %p %p %d %d %p %p %p %p)\n", phCredential, phContext, pInput, fContextReq, TargetDataRep, phNewContext, pOutput,
//           pfContextAttr, ptsExpiry);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              CompleteAuthToken
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_CompleteAuthToken(CtxtHandle *phContext, SecBufferDesc *pToken)
// {
//     FIXME("(%p %p)\n", phContext, pToken);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              DeleteSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_DeleteSecurityContext(CtxtHandle *phContext)
// {
//     FIXME("(%p)\n", phContext);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              QueryContextAttributesW
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_QueryContextAttributesW(CtxtHandle *phContext, ULONG ulAttribute, void *pBuffer)
// {
//     FIXME("(%p %d %p)\n", phContext, ulAttribute, pBuffer);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              QueryContextAttributesA
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_QueryContextAttributesA(CtxtHandle *phContext, ULONG ulAttribute, void *pBuffer)
// {
//     FIXME("(%p %d %p)\n", phContext, ulAttribute, pBuffer);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              ImpersonateSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_ImpersonateSecurityContext(CtxtHandle *phContext)
// {
//     FIXME("(%p)\n", phContext);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              RevertSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_RevertSecurityContext(CtxtHandle *phContext)
// {
//     FIXME("(%p)\n", phContext);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              MakeSignature
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_MakeSignature(CtxtHandle *phContext, ULONG fQOP, SecBufferDesc *pMessage, ULONG MessageSeqNo)
// {
//     FIXME("(%p %d %p %d)\n", phContext, fQOP, pMessage, MessageSeqNo);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              VerifySignature
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_VerifySignature(CtxtHandle *phContext, SecBufferDesc *pMessage, ULONG MessageSeqNo, PULONG pfQOP)
// {
//     FIXME("(%p %p %d %p)\n", phContext, pMessage, MessageSeqNo, pfQOP);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *             FreeCredentialsHandle
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_FreeCredentialsHandle(PCredHandle phCredential)
// {
//     FIXME("(%p)\n", phCredential);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *             EncryptMessage
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_EncryptMessage(CtxtHandle *phContext, ULONG fQOP, SecBufferDesc *pMessage, ULONG MessageSeqNo)
// {
//     FIXME("(%p %d %p %d)\n", phContext, fQOP, pMessage, MessageSeqNo);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *             DecryptMessage
//  */
// static SECURITY_STATUS SEC_ENTRY kerberos_DecryptMessage(CtxtHandle *phContext, SecBufferDesc *pMessage, ULONG MessageSeqNo, PULONG pfQOP)
// {
//     FIXME("(%p %p %d %p)\n", phContext, pMessage, MessageSeqNo, pfQOP);
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// static const SecurityFunctionTableA kerberosTableA = {
//     1,
//     NULL,   /* EnumerateSecurityPackagesA */
//     kerberos_QueryCredentialsAttributesA,   /* QueryCredentialsAttributesA */
//     kerberos_AcquireCredentialsHandleA,     /* AcquireCredentialsHandleA */
//     kerberos_FreeCredentialsHandle,         /* FreeCredentialsHandle */
//     NULL,   /* Reserved2 */
//     kerberos_InitializeSecurityContextA,    /* InitializeSecurityContextA */
//     kerberos_AcceptSecurityContext,         /* AcceptSecurityContext */
//     kerberos_CompleteAuthToken,             /* CompleteAuthToken */
//     kerberos_DeleteSecurityContext,         /* DeleteSecurityContext */
//     NULL,  /* ApplyControlToken */
//     kerberos_QueryContextAttributesA,       /* QueryContextAttributesA */
//     kerberos_ImpersonateSecurityContext,    /* ImpersonateSecurityContext */
//     kerberos_RevertSecurityContext,         /* RevertSecurityContext */
//     kerberos_MakeSignature,                 /* MakeSignature */
//     kerberos_VerifySignature,               /* VerifySignature */
//     FreeContextBuffer,                      /* FreeContextBuffer */
//     NULL,   /* QuerySecurityPackageInfoA */
//     NULL,   /* Reserved3 */
//     NULL,   /* Reserved4 */
//     NULL,   /* ExportSecurityContext */
//     NULL,   /* ImportSecurityContextA */
//     NULL,   /* AddCredentialsA */
//     NULL,   /* Reserved8 */
//     NULL,   /* QuerySecurityContextToken */
//     kerberos_EncryptMessage,                /* EncryptMessage */
//     kerberos_DecryptMessage,                /* DecryptMessage */
//     NULL,   /* SetContextAttributesA */
// };
//
// static const SecurityFunctionTableW kerberosTableW = {
//     1,
//     NULL,   /* EnumerateSecurityPackagesW */
//     kerberos_QueryCredentialsAttributesW,   /* QueryCredentialsAttributesW */
//     kerberos_AcquireCredentialsHandleW,     /* AcquireCredentialsHandleW */
//     kerberos_FreeCredentialsHandle,         /* FreeCredentialsHandle */
//     NULL,   /* Reserved2 */
//     kerberos_InitializeSecurityContextW,    /* InitializeSecurityContextW */
//     kerberos_AcceptSecurityContext,         /* AcceptSecurityContext */
//     kerberos_CompleteAuthToken,             /* CompleteAuthToken */
//     kerberos_DeleteSecurityContext,         /* DeleteSecurityContext */
//     NULL,  /* ApplyControlToken */
//     kerberos_QueryContextAttributesW,       /* QueryContextAttributesW */
//     kerberos_ImpersonateSecurityContext,    /* ImpersonateSecurityContext */
//     kerberos_RevertSecurityContext,         /* RevertSecurityContext */
//     kerberos_MakeSignature,                 /* MakeSignature */
//     kerberos_VerifySignature,               /* VerifySignature */
//     FreeContextBuffer,                      /* FreeContextBuffer */
//     NULL,   /* QuerySecurityPackageInfoW */
//     NULL,   /* Reserved3 */
//     NULL,   /* Reserved4 */
//     NULL,   /* ExportSecurityContext */
//     NULL,   /* ImportSecurityContextW */
//     NULL,   /* AddCredentialsW */
//     NULL,   /* Reserved8 */
//     NULL,   /* QuerySecurityContextToken */
//     kerberos_EncryptMessage,                /* EncryptMessage */
//     kerberos_DecryptMessage,                /* DecryptMessage */
//     NULL,   /* SetContextAttributesW */
// };
//
// #define KERBEROS_COMMENT \
//     {'M','i','c','r','o','s','o','f','t',' ','K','e','r','b','e','r','o','s',' ','V','1','.','0',0}
// static CHAR kerberos_comment_A[] = KERBEROS_COMMENT;
// static WCHAR kerberos_comment_W[] = KERBEROS_COMMENT;
//
// #define KERBEROS_NAME {'K','e','r','b','e','r','o','s',0}
// static char kerberos_name_A[] = KERBEROS_NAME;
// static WCHAR kerberos_name_W[] = KERBEROS_NAME;
//
// #define CAPS \
//     ( SECPKG_FLAG_INTEGRITY \
//     | SECPKG_FLAG_PRIVACY \
//     | SECPKG_FLAG_TOKEN_ONLY \
//     | SECPKG_FLAG_DATAGRAM \
//     | SECPKG_FLAG_CONNECTION \
//     | SECPKG_FLAG_MULTI_REQUIRED \
//     | SECPKG_FLAG_EXTENDED_ERROR \
//     | SECPKG_FLAG_IMPERSONATION \
//     | SECPKG_FLAG_ACCEPT_WIN32_NAME \
//     | SECPKG_FLAG_NEGOTIABLE \
//     | SECPKG_FLAG_GSS_COMPATIBLE \
//     | SECPKG_FLAG_LOGON \
//     | SECPKG_FLAG_MUTUAL_AUTH \
//     | SECPKG_FLAG_DELEGATION \
//     | SECPKG_FLAG_READONLY_WITH_CHECKSUM \
//     | SECPKG_FLAG_RESTRICTED_TOKENS \
//     | SECPKG_FLAG_APPCONTAINER_CHECKS)
//
// static const SecPkgInfoW infoW = {
//     CAPS,
//     1,
//     RPC_C_AUTHN_GSS_KERBEROS,
//     KERBEROS_MAX_BUF,
//     kerberos_name_W,
//     kerberos_comment_W
// };
//
// static const SecPkgInfoA infoA = {
//     CAPS,
//     1,
//     RPC_C_AUTHN_GSS_KERBEROS,
//     KERBEROS_MAX_BUF,
//     kerberos_name_A,
//     kerberos_comment_A
// };
//
// void SECUR32_initKerberosSP(void)
// {
//     SecureProvider *provider = SECUR32_addProvider(&kerberosTableA, &kerberosTableW, NULL);
//     SECUR32_addPackages(provider, 1, &infoA, &infoW);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
