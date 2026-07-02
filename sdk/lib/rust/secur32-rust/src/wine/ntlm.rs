use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NTLM_MAX_BUF: u32 = 1904;

pub const MIN_NTLM_AUTH_MAJOR_VERSION: i32 = 3;

pub const MIN_NTLM_AUTH_MINOR_VERSION: i32 = 0;

pub const MIN_NTLM_AUTH_MICRO_VERSION: i32 = 23;

pub static mut ntlm_auth: [CHAR; 10] = [b'n' as CHAR, b't' as CHAR, b'l' as CHAR, b'm' as CHAR, b'_' as CHAR, b'a' as CHAR, b'u' as CHAR, b't' as CHAR, b'h' as CHAR, 0];

pub static mut ntlmTableA: SecurityFunctionTableA = SecurityFunctionTableA {
    dwVersion: 1,
    EnumerateSecurityPackagesA: None,
    QueryCredentialsAttributesA: sec_entry!(ntlm_QueryCredentialsAttributesA),
    AcquireCredentialsHandleA: sec_entry!(ntlm_AcquireCredentialsHandleA),
    FreeCredentialsHandle: sec_entry!(ntlm_FreeCredentialsHandle),
    Reserved2: core::ptr::null_mut(),
    InitializeSecurityContextA: sec_entry!(ntlm_InitializeSecurityContextA),
    AcceptSecurityContext: sec_entry!(ntlm_AcceptSecurityContext),
    CompleteAuthToken: sec_entry!(ntlm_CompleteAuthToken),
    DeleteSecurityContext: sec_entry!(ntlm_DeleteSecurityContext),
    ApplyControlToken: None,
    QueryContextAttributesA: sec_entry!(ntlm_QueryContextAttributesA),
    ImpersonateSecurityContext: sec_entry!(ntlm_ImpersonateSecurityContext),
    RevertSecurityContext: sec_entry!(ntlm_RevertSecurityContext),
    MakeSignature: sec_entry!(ntlm_MakeSignature),
    VerifySignature: sec_entry!(ntlm_VerifySignature),
    FreeContextBuffer: sec_entry!(FreeContextBuffer),
    QuerySecurityPackageInfoA: None,
    Reserved3: core::ptr::null_mut(),
    Reserved4: core::ptr::null_mut(),
    ExportSecurityContext: None,
    ImportSecurityContextA: None,
    AddCredentialsA: None,
    Reserved8: core::ptr::null_mut(),
    QuerySecurityContextToken: None,
    EncryptMessage: sec_entry!(ntlm_EncryptMessage),
    DecryptMessage: sec_entry!(ntlm_DecryptMessage),
    SetContextAttributesA: None,
};

pub static mut ntlmTableW: SecurityFunctionTableW = SecurityFunctionTableW {
    dwVersion: 1,
    EnumerateSecurityPackagesW: None,
    QueryCredentialsAttributesW: sec_entry!(ntlm_QueryCredentialsAttributesW),
    AcquireCredentialsHandleW: sec_entry!(ntlm_AcquireCredentialsHandleW),
    FreeCredentialsHandle: sec_entry!(ntlm_FreeCredentialsHandle),
    Reserved2: core::ptr::null_mut(),
    InitializeSecurityContextW: sec_entry!(ntlm_InitializeSecurityContextW),
    AcceptSecurityContext: sec_entry!(ntlm_AcceptSecurityContext),
    CompleteAuthToken: sec_entry!(ntlm_CompleteAuthToken),
    DeleteSecurityContext: sec_entry!(ntlm_DeleteSecurityContext),
    ApplyControlToken: None,
    QueryContextAttributesW: sec_entry!(ntlm_QueryContextAttributesW),
    ImpersonateSecurityContext: sec_entry!(ntlm_ImpersonateSecurityContext),
    RevertSecurityContext: sec_entry!(ntlm_RevertSecurityContext),
    MakeSignature: sec_entry!(ntlm_MakeSignature),
    VerifySignature: sec_entry!(ntlm_VerifySignature),
    FreeContextBuffer: sec_entry!(FreeContextBuffer),
    QuerySecurityPackageInfoW: None,
    Reserved3: core::ptr::null_mut(),
    Reserved4: core::ptr::null_mut(),
    ExportSecurityContext: None,
    ImportSecurityContextW: None,
    AddCredentialsW: None,
    Reserved8: core::ptr::null_mut(),
    QuerySecurityContextToken: None,
    EncryptMessage: sec_entry!(ntlm_EncryptMessage),
    DecryptMessage: sec_entry!(ntlm_DecryptMessage),
    SetContextAttributesW: None,
};

pub static mut ntlm_comment_A: [CHAR; 22] = [
    b'N' as CHAR, b'T' as CHAR, b'L' as CHAR, b'M' as CHAR, b' ' as CHAR,
    b'S' as CHAR, b'e' as CHAR, b'c' as CHAR, b'u' as CHAR, b'r' as CHAR, b'i' as CHAR, b't' as CHAR, b'y' as CHAR, b' ' as CHAR,
    b'P' as CHAR, b'a' as CHAR, b'c' as CHAR, b'k' as CHAR, b'a' as CHAR, b'g' as CHAR, b'e' as CHAR, 0,
];

pub static mut ntlm_comment_W: [WCHAR; 22] = [
    b'N' as WCHAR, b'T' as WCHAR, b'L' as WCHAR, b'M' as WCHAR, b' ' as WCHAR,
    b'S' as WCHAR, b'e' as WCHAR, b'c' as WCHAR, b'u' as WCHAR, b'r' as WCHAR, b'i' as WCHAR, b't' as WCHAR, b'y' as WCHAR, b' ' as WCHAR,
    b'P' as WCHAR, b'a' as WCHAR, b'c' as WCHAR, b'k' as WCHAR, b'a' as WCHAR, b'g' as WCHAR, b'e' as WCHAR, 0,
];

pub static mut ntlm_name_A: [CHAR; 5] = [b'N' as CHAR, b'T' as CHAR, b'L' as CHAR, b'M' as CHAR, 0];

pub static mut ntlm_name_W: [WCHAR; 5] = [b'N' as WCHAR, b'T' as WCHAR, b'L' as WCHAR, b'M' as WCHAR, 0];

pub const CAPS: u32 = SECPKG_FLAG_INTEGRITY
    | SECPKG_FLAG_PRIVACY
    | SECPKG_FLAG_TOKEN_ONLY
    | SECPKG_FLAG_CONNECTION
    | SECPKG_FLAG_MULTI_REQUIRED
    | SECPKG_FLAG_IMPERSONATION
    | SECPKG_FLAG_ACCEPT_WIN32_NAME
    | SECPKG_FLAG_NEGOTIABLE
    | SECPKG_FLAG_LOGON
    | SECPKG_FLAG_RESTRICTED_TOKENS;

pub static mut infoW: SecPkgInfoW = SecPkgInfoW {
    fCapabilities: CAPS,
    wVersion: 1,
    wRPCID: RPC_C_AUTHN_WINNT as u16,
    cbMaxToken: NTLM_MAX_BUF,
    Name: unsafe { ntlm_name_W.as_ptr() as LPWSTR },
    Comment: unsafe { ntlm_comment_W.as_ptr() as LPWSTR },
};

pub static mut infoA: SecPkgInfoA = SecPkgInfoA {
    fCapabilities: CAPS,
    wVersion: 1,
    wRPCID: RPC_C_AUTHN_WINNT as u16,
    cbMaxToken: NTLM_MAX_BUF,
    Name: unsafe { ntlm_name_A.as_ptr() as LPSTR },
    Comment: unsafe { ntlm_comment_A.as_ptr() as LPSTR },
};

pub static mut ntlm_package_infoA: *mut SecPkgInfoA = core::ptr::addr_of!(infoA) as *mut SecPkgInfoA;
pub static mut ntlm_package_infoW: *mut SecPkgInfoW = core::ptr::addr_of!(infoW) as *mut SecPkgInfoW;

unsafe extern "C" {
    fn fork_helper(helper: *mut PNegoHelper, prog: *mut SEC_CHAR, args: *mut *mut SEC_CHAR) -> SECURITY_STATUS;
    fn check_version(helper: PNegoHelper);
    fn SECUR32_addProvider(fnTableA: *const SecurityFunctionTableA, fnTableW: *const SecurityFunctionTableW, moduleName: PWSTR) -> *mut SecureProvider;
    fn SECUR32_addPackages(provider: *mut SecureProvider, toAdd: ULONG, infoA: *const SecPkgInfoA, infoW: *const SecPkgInfoW);
    fn cleanup_helper(helper: PNegoHelper);
}

unsafe extern "system" fn ntlm_QueryCredentialsAttributesA(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: PVOID,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if ulAttribute == SECPKG_ATTR_NAMES {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    } else {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    }

    ret
}
unsafe extern "system" fn ntlm_QueryCredentialsAttributesW(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: PVOID,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if ulAttribute == SECPKG_ATTR_NAMES {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    } else {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    }

    ret
}

unsafe fn ntlm_GetUsernameArg(userW: LPCWSTR, userW_length: INT) -> *mut core::ffi::c_char {
    static username_arg: [u8; 12] = *b"--username=\0";
    let arg_len = username_arg.len() as i32;

    let unixcp_size = WideCharToMultiByte(
        CP_UNIXCP,
        WC_NO_BEST_FIT_CHARS,
        userW,
        userW_length,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) + arg_len;

    let user = HeapAlloc(GetProcessHeap(), 0, unixcp_size as usize as _) as *mut core::ffi::c_char;
    if user.is_null() {
        return core::ptr::null_mut();
    }

    memcpy(
        user as *mut core::ffi::c_void,
        username_arg.as_ptr() as *const core::ffi::c_void,
        (arg_len - 1) as usize,
    );

    WideCharToMultiByte(
        CP_UNIXCP,
        WC_NO_BEST_FIT_CHARS,
        userW,
        userW_length,
        user.add((arg_len - 1) as usize),
        unixcp_size - arg_len + 1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    *user.add((unixcp_size - 1) as usize) = 0;
    user
}

unsafe fn ntlm_GetDomainArg(domainW: LPCWSTR, domainW_length: INT) -> *mut c_char {
    let domain_arg: [c_char; 10] = [
        b'-' as c_char, b'-' as c_char, b'd' as c_char, b'o' as c_char,
        b'm' as c_char, b'a' as c_char, b'i' as c_char, b'n' as c_char,
        b'=' as c_char, 0,
    ];

    let unixcp_size: c_int = WideCharToMultiByte(
        CP_UNIXCP,
        WC_NO_BEST_FIT_CHARS,
        domainW,
        domainW_length,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) + core::mem::size_of_val(&domain_arg) as c_int;

    let domain = HeapAlloc(GetProcessHeap(), 0, unixcp_size as usize as _) as *mut c_char;
    if domain.is_null() {
        return core::ptr::null_mut();
    }

    memcpy(
        domain as *mut c_void,
        domain_arg.as_ptr() as *const c_void,
        core::mem::size_of_val(&domain_arg) - 1,
    );

    WideCharToMultiByte(
        CP_UNIXCP,
        WC_NO_BEST_FIT_CHARS,
        domainW,
        domainW_length,
        domain.add(core::mem::size_of_val(&domain_arg) - 1),
        unixcp_size - core::mem::size_of::<*mut c_char>() as c_int + 1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    *domain.add((unixcp_size - 1) as usize) = 0;
    domain
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_AcquireCredentialsHandleW(
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
    let mut ret: SECURITY_STATUS;
    let ntlm_cred: PNtlmCredentials;
    let mut domain: LPWSTR = core::ptr::null_mut();
    let mut user: LPWSTR = core::ptr::null_mut();
    let mut password: LPWSTR = core::ptr::null_mut();
    let mut auth_data: PSEC_WINNT_AUTH_IDENTITY_W = core::ptr::null_mut();

    'done: loop {
        match fCredentialUse {
            SECPKG_CRED_INBOUND => {
                ntlm_cred = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    core::mem::size_of::<NtlmCredentials>() as _,
                ) as PNtlmCredentials;
                if ntlm_cred.is_null() {
                    ret = SEC_E_INSUFFICIENT_MEMORY;
                } else {
                    (*ntlm_cred).mode = NTLM_SERVER;
                    (*ntlm_cred).username_arg = core::ptr::null_mut();
                    (*ntlm_cred).domain_arg = core::ptr::null_mut();
                    (*ntlm_cred).password = core::ptr::null_mut();
                    (*ntlm_cred).pwlen = 0;
                    (*ntlm_cred).no_cached_credentials = 0;

                    (*phCredential).dwUpper = fCredentialUse as ULONG_PTR;
                    (*phCredential).dwLower = ntlm_cred as ULONG_PTR;
                    ret = SEC_E_OK;
                }
            }
            SECPKG_CRED_OUTBOUND => {
                auth_data = pAuthData as PSEC_WINNT_AUTH_IDENTITY_W;
                ntlm_cred = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    core::mem::size_of::<NtlmCredentials>() as _,
                ) as PNtlmCredentials;
                if ntlm_cred.is_null() {
                    ret = SEC_E_INSUFFICIENT_MEMORY;
                    break 'done;
                }
                (*ntlm_cred).mode = NTLM_CLIENT;
                (*ntlm_cred).username_arg = core::ptr::null_mut();
                (*ntlm_cred).domain_arg = core::ptr::null_mut();
                (*ntlm_cred).password = core::ptr::null_mut();
                (*ntlm_cred).pwlen = 0;
                (*ntlm_cred).no_cached_credentials = 0;

                if !pAuthData.is_null() {
                    let mut domain_len: INT = 0;
                    let mut user_len: INT = 0;
                    let mut password_len: INT = 0;

                    if (*auth_data).Flags & SEC_WINNT_AUTH_IDENTITY_ANSI != 0 {
                        if (*auth_data).DomainLength != 0 {
                            domain_len = MultiByteToWideChar(
                                CP_ACP,
                                0,
                                (*auth_data).Domain as *mut c_char,
                                (*auth_data).DomainLength as INT,
                                core::ptr::null_mut(),
                                0,
                            );
                            domain = HeapAlloc(
                                GetProcessHeap(),
                                0,
                                (core::mem::size_of::<WCHAR>() * domain_len as usize) as _,
                            ) as LPWSTR;
                            if domain.is_null() {
                                ret = SEC_E_INSUFFICIENT_MEMORY;
                                break 'done;
                            }
                            MultiByteToWideChar(
                                CP_ACP,
                                0,
                                (*auth_data).Domain as *mut c_char,
                                (*auth_data).DomainLength as INT,
                                domain,
                                domain_len,
                            );
                        }

                        if (*auth_data).UserLength != 0 {
                            user_len = MultiByteToWideChar(
                                CP_ACP,
                                0,
                                (*auth_data).User as *mut c_char,
                                (*auth_data).UserLength as INT,
                                core::ptr::null_mut(),
                                0,
                            );
                            user = HeapAlloc(
                                GetProcessHeap(),
                                0,
                                (core::mem::size_of::<WCHAR>() * user_len as usize) as _,
                            ) as LPWSTR;
                            if user.is_null() {
                                ret = SEC_E_INSUFFICIENT_MEMORY;
                                break 'done;
                            }
                            MultiByteToWideChar(
                                CP_ACP,
                                0,
                                (*auth_data).User as *mut c_char,
                                (*auth_data).UserLength as INT,
                                user,
                                user_len,
                            );
                        }

                        if (*auth_data).PasswordLength != 0 {
                            password_len = MultiByteToWideChar(
                                CP_ACP,
                                0,
                                (*auth_data).Password as *mut c_char,
                                (*auth_data).PasswordLength as INT,
                                core::ptr::null_mut(),
                                0,
                            );
                            password = HeapAlloc(
                                GetProcessHeap(),
                                0,
                                (core::mem::size_of::<WCHAR>() * password_len as usize) as _,
                            ) as LPWSTR;
                            if password.is_null() {
                                ret = SEC_E_INSUFFICIENT_MEMORY;
                                break 'done;
                            }
                            MultiByteToWideChar(
                                CP_ACP,
                                0,
                                (*auth_data).Password as *mut c_char,
                                (*auth_data).PasswordLength as INT,
                                password,
                                password_len,
                            );
                        }
                    } else {
                        domain = (*auth_data).Domain;
                        domain_len = (*auth_data).DomainLength as INT;

                        user = (*auth_data).User;
                        user_len = (*auth_data).UserLength as INT;

                        password = (*auth_data).Password;
                        password_len = (*auth_data).PasswordLength as INT;
                    }

                    (*ntlm_cred).username_arg = ntlm_GetUsernameArg(user, user_len);
                    (*ntlm_cred).domain_arg = ntlm_GetDomainArg(domain, domain_len);

                    if password_len != 0 {
                        (*ntlm_cred).pwlen = WideCharToMultiByte(
                            CP_UNIXCP,
                            WC_NO_BEST_FIT_CHARS,
                            password,
                            password_len,
                            core::ptr::null_mut(),
                            0,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                        );

                        (*ntlm_cred).password = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            (*ntlm_cred).pwlen as usize as _,
                        ) as *mut i8;
                        if (*ntlm_cred).password.is_null() {
                            ret = SEC_E_INSUFFICIENT_MEMORY;
                            break 'done;
                        }

                        WideCharToMultiByte(
                            CP_UNIXCP,
                            WC_NO_BEST_FIT_CHARS,
                            password,
                            password_len,
                            (*ntlm_cred).password as *mut c_char,
                            (*ntlm_cred).pwlen,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                        );
                    }
                }

                (*phCredential).dwUpper = fCredentialUse as ULONG_PTR;
                (*phCredential).dwLower = ntlm_cred as ULONG_PTR;
                ret = SEC_E_OK;
            }
            SECPKG_CRED_BOTH => {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
            _ => {
                ret = SEC_E_UNKNOWN_CREDENTIALS;
            }
        }
        break 'done;
    }

    if !auth_data.is_null() && (*auth_data).Flags & SEC_WINNT_AUTH_IDENTITY_ANSI != 0 {
        HeapFree(GetProcessHeap(), 0, domain as PVOID);
        HeapFree(GetProcessHeap(), 0, user as PVOID);
        HeapFree(GetProcessHeap(), 0, password as PVOID);
    }
    ret
}
unsafe extern "system" fn ntlm_AcquireCredentialsHandleA(
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
    let ret: SECURITY_STATUS;
    let mut user_sizeW: i32;
    let mut domain_sizeW: i32;
    let mut passwd_sizeW: i32;

    let mut user: *mut SEC_WCHAR = core::ptr::null_mut();
    let mut domain: *mut SEC_WCHAR = core::ptr::null_mut();
    let mut passwd: *mut SEC_WCHAR = core::ptr::null_mut();
    let mut package: *mut SEC_WCHAR = core::ptr::null_mut();

    let mut pAuthDataW: PSEC_WINNT_AUTH_IDENTITY_W = core::ptr::null_mut();
    let mut identity: PSEC_WINNT_AUTH_IDENTITY_A = core::ptr::null_mut();

    if pszPackage != core::ptr::null_mut() {
        let package_sizeW = MultiByteToWideChar(CP_ACP, 0, pszPackage, -1,
                core::ptr::null_mut(), 0);

        package = HeapAlloc(GetProcessHeap(), 0,
                ((package_sizeW as usize) * core::mem::size_of::<SEC_WCHAR>()) as _) as *mut SEC_WCHAR;
        MultiByteToWideChar(CP_ACP, 0, pszPackage, -1, package, package_sizeW);
    }

    if pAuthData != core::ptr::null_mut() {
        identity = pAuthData as PSEC_WINNT_AUTH_IDENTITY_A;

        if (*identity).Flags == SEC_WINNT_AUTH_IDENTITY_ANSI {
            pAuthDataW = HeapAlloc(GetProcessHeap(), 0,
                    core::mem::size_of::<SEC_WINNT_AUTH_IDENTITY_W>() as _) as PSEC_WINNT_AUTH_IDENTITY_W;

            if (*identity).UserLength != 0 {
                user_sizeW = MultiByteToWideChar(CP_ACP, 0,
                    (*identity).User as LPCSTR, (*identity).UserLength as i32, core::ptr::null_mut(), 0);
                user = HeapAlloc(GetProcessHeap(), 0,
                        ((user_sizeW as usize) * core::mem::size_of::<SEC_WCHAR>()) as _) as *mut SEC_WCHAR;
                MultiByteToWideChar(CP_ACP, 0, (*identity).User as LPCSTR,
                    (*identity).UserLength as i32, user, user_sizeW);
            } else {
                user_sizeW = 0;
            }

            if (*identity).DomainLength != 0 {
                domain_sizeW = MultiByteToWideChar(CP_ACP, 0,
                    (*identity).Domain as LPCSTR, (*identity).DomainLength as i32, core::ptr::null_mut(), 0);
                domain = HeapAlloc(GetProcessHeap(), 0,
                    ((domain_sizeW as usize) * core::mem::size_of::<SEC_WCHAR>()) as _) as *mut SEC_WCHAR;
                MultiByteToWideChar(CP_ACP, 0, (*identity).Domain as LPCSTR,
                    (*identity).DomainLength as i32, domain, domain_sizeW);
            } else {
                domain_sizeW = 0;
            }

            if (*identity).PasswordLength != 0 {
                passwd_sizeW = MultiByteToWideChar(CP_ACP, 0,
                    (*identity).Password as LPCSTR, (*identity).PasswordLength as i32,
                    core::ptr::null_mut(), 0);
                passwd = HeapAlloc(GetProcessHeap(), 0,
                    ((passwd_sizeW as usize) * core::mem::size_of::<SEC_WCHAR>()) as _) as *mut SEC_WCHAR;
                MultiByteToWideChar(CP_ACP, 0, (*identity).Password as LPCSTR,
                    (*identity).PasswordLength as i32, passwd, passwd_sizeW);
            } else {
                passwd_sizeW = 0;
            }

            (*pAuthDataW).Flags = SEC_WINNT_AUTH_IDENTITY_UNICODE;
            (*pAuthDataW).User = user as *mut u16;
            (*pAuthDataW).UserLength = user_sizeW as u32;
            (*pAuthDataW).Domain = domain as *mut u16;
            (*pAuthDataW).DomainLength = domain_sizeW as u32;
            (*pAuthDataW).Password = passwd as *mut u16;
            (*pAuthDataW).PasswordLength = passwd_sizeW as u32;
        } else {
            pAuthDataW = identity as PSEC_WINNT_AUTH_IDENTITY_W;
        }
    }

    ret = ntlm_AcquireCredentialsHandleW(core::ptr::null_mut(), package, fCredentialUse,
            pLogonID, pAuthDataW as PVOID, pGetKeyFn, pGetKeyArgument, phCredential,
            ptsExpiry);

    HeapFree(GetProcessHeap(), 0, package as PVOID);
    HeapFree(GetProcessHeap(), 0, user as PVOID);
    HeapFree(GetProcessHeap(), 0, domain as PVOID);
    HeapFree(GetProcessHeap(), 0, passwd as PVOID);
    if pAuthDataW != identity as PSEC_WINNT_AUTH_IDENTITY_W {
        HeapFree(GetProcessHeap(), 0, pAuthDataW as PVOID);
    }

    ret
}

unsafe fn ntlm_GetTokenBufferIndex(pMessage: PSecBufferDesc) -> core::ffi::c_int {
    let mut i: UINT;

    i = 0;
    while i < (*pMessage).cBuffers {
        if (*(*pMessage).pBuffers.offset(i as isize)).BufferType == SECBUFFER_TOKEN {
            return i as core::ffi::c_int;
        }
        i += 1;
    }

    -1
}

unsafe fn ntlm_GetDataBufferIndex(pMessage: PSecBufferDesc) -> core::ffi::c_int {
    let mut i: UINT;

    i = 0;
    while i < (*pMessage).cBuffers {
        if (*(*pMessage).pBuffers.offset(i as isize)).BufferType == SECBUFFER_DATA {
            return i as core::ffi::c_int;
        }
        i += 1;
    }

    -1
}

unsafe fn ntlm_GetCachedCredential(pszTargetName: *const SEC_WCHAR, cred: *mut PCREDENTIALW) -> BOOL {
    let mut p: LPCWSTR;
    let mut pszHost: LPCWSTR;
    let pszHostOnly: LPWSTR;
    let ret: BOOL;

    if pszTargetName.is_null() {
        return 0;
    }

    pszHost = strchrW(pszTargetName, '/' as i32);
    if !pszHost.is_null() {
        pszHost = pszHost.add(1);

        p = strchrW(pszHost, ':' as i32);
        if p.is_null() {
            p = strchrW(pszHost, '/' as i32);
        }
        if p.is_null() {
            p = pszHost.add(strlenW(pszHost) as usize);
        }
    } else {
        pszHost = pszTargetName;
        p = pszHost.add(strlenW(pszHost) as usize);
    }

    let count = p.offset_from(pszHost);

    pszHostOnly = HeapAlloc(
        GetProcessHeap(),
        0,
        ((count + 1) as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as LPWSTR;
    if pszHostOnly.is_null() {
        return 0;
    }

    memcpy(
        pszHostOnly as *mut core::ffi::c_void,
        pszHost as *const core::ffi::c_void,
        (count as usize * core::mem::size_of::<WCHAR>()) as _,
    );
    *pszHostOnly.offset(count) = 0;

    ret = CredReadW(pszHostOnly, CRED_TYPE_DOMAIN_PASSWORD, 0, cred);

    HeapFree(GetProcessHeap(), 0, pszHostOnly as *mut core::ffi::c_void);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_InitializeSecurityContextW(
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
    let mut ret: SECURITY_STATUS;
    let ntlm_cred: PNtlmCredentials;
    let mut helper: PNegoHelper = core::ptr::null_mut();
    let mut ctxt_attr: ULONG = 0;
    let buffer: *mut c_char;
    let mut want_flags: *mut c_char = core::ptr::null_mut();
    let bin: PBYTE;
    let mut buffer_len: c_int = 0;
    let mut bin_len: c_int = 0;
    let max_len: c_int = NTLM_MAX_BUF as c_int;
    let mut token_idx: c_int;
    let mut username: *mut SEC_CHAR = core::ptr::null_mut();
    let mut domain: *mut SEC_CHAR = core::ptr::null_mut();
    let mut password: *mut SEC_CHAR = core::ptr::null_mut();

    if TargetDataRep == SECURITY_NETWORK_DREP {
    }

    buffer = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<c_char>() * NTLM_MAX_BUF as usize) as _) as *mut c_char;
    bin = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<BYTE>() * NTLM_MAX_BUF as usize) as _) as PBYTE;

    'isc_end: loop {
        if phContext.is_null() && pInput.is_null() {
            let mut helper_protocol: [c_char; 35] = {
                let s = b"--helper-protocol=ntlmssp-client-1\0";
                let mut arr = [0 as c_char; 35];
                let mut i = 0;
                while i < 35 { arr[i] = s[i] as c_char; i += 1; }
                arr
            };
            let mut credentials_argv: [c_char; 19] = {
                let s = b"--use-cached-creds\0";
                let mut arr = [0 as c_char; 19];
                let mut i = 0;
                while i < 19 { arr[i] = s[i] as c_char; i += 1; }
                arr
            };
            let mut client_argv: [*mut SEC_CHAR; 5] = [core::ptr::null_mut(); 5];
            let mut pwlen: c_int = 0;

            if phCredential.is_null() {
                ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
                break 'isc_end;
            }

            ntlm_cred = (*phCredential).dwLower as PNtlmCredentials;
            if (*ntlm_cred).mode != NTLM_CLIENT {
                ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
                break 'isc_end;
            }

            client_argv[0] = ntlm_auth.as_mut_ptr() as *mut SEC_CHAR;
            client_argv[1] = helper_protocol.as_mut_ptr();
            if (*ntlm_cred).username_arg.is_null() && (*ntlm_cred).domain_arg.is_null() {
                let mut ui: LPWKSTA_USER_INFO_1 = core::ptr::null_mut();
                let status: NET_API_STATUS;
                let mut cred: PCREDENTIALW = core::ptr::null_mut();

                if ntlm_GetCachedCredential(pszTargetName, &mut cred) != 0 {
                    let mut p: LPWSTR;
                    p = strchrW((*cred).UserName, '\\' as i32) as LPWSTR;
                    if !p.is_null() {
                        domain = ntlm_GetDomainArg((*cred).UserName, p.offset_from((*cred).UserName) as INT);
                        p = p.add(1);
                    } else {
                        domain = ntlm_GetDomainArg(core::ptr::null(), 0);
                        p = (*cred).UserName;
                    }

                    username = ntlm_GetUsernameArg(p, -1);

                    if (*cred).CredentialBlobSize != 0 {
                        pwlen = WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS,
                            (*cred).CredentialBlob as LPWSTR,
                            (*cred).CredentialBlobSize as c_int / core::mem::size_of::<WCHAR>() as c_int,
                            core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());

                        password = HeapAlloc(GetProcessHeap(), 0, pwlen as _) as *mut SEC_CHAR;

                        WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS,
                            (*cred).CredentialBlob as LPWSTR,
                            (*cred).CredentialBlobSize as c_int / core::mem::size_of::<WCHAR>() as c_int,
                            password, pwlen, core::ptr::null(), core::ptr::null_mut());
                    }

                    CredFree(cred as _);

                    client_argv[2] = username;
                    client_argv[3] = domain;
                    client_argv[4] = core::ptr::null_mut();
                } else {
                    status = NetWkstaUserGetInfo(core::ptr::null_mut(), 1, &mut ui as *mut LPWKSTA_USER_INFO_1 as *mut LPBYTE);
                    if status != NERR_Success || ui.is_null() || (*ntlm_cred).no_cached_credentials != 0 {
                        ret = SEC_E_NO_CREDENTIALS as SECURITY_STATUS;
                        break 'isc_end;
                    }
                    username = ntlm_GetUsernameArg((*ui).wkui1_username, -1);
                    NetApiBufferFree(ui as _);

                    client_argv[2] = username;
                    client_argv[3] = credentials_argv.as_mut_ptr();
                    client_argv[4] = core::ptr::null_mut();
                }
            } else {
                client_argv[2] = (*ntlm_cred).username_arg;
                client_argv[3] = (*ntlm_cred).domain_arg;
                client_argv[4] = core::ptr::null_mut();
            }

            ret = fork_helper(&mut helper, ntlm_auth.as_mut_ptr() as *mut SEC_CHAR, client_argv.as_mut_ptr());
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'isc_end;
            }

            (*helper).mode = NTLM_CLIENT;
            (*helper).session_key = HeapAlloc(GetProcessHeap(), 0, 16) as PBYTE;
            if (*helper).session_key.is_null() {
                cleanup_helper(helper);
                ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
                break 'isc_end;
            }

            if !password.is_null() || !(*ntlm_cred).password.is_null() {
                let unicode_password: *mut SEC_WCHAR;
                let passwd_lenW: c_int;

                passwd_lenW = MultiByteToWideChar(CP_ACP, 0,
                    if !password.is_null() { password } else { (*ntlm_cred).password },
                    if !password.is_null() { pwlen } else { (*ntlm_cred).pwlen },
                    core::ptr::null_mut(), 0);
                unicode_password = HeapAlloc(GetProcessHeap(), 0,
                    (passwd_lenW as usize * core::mem::size_of::<SEC_WCHAR>()) as _) as *mut SEC_WCHAR;
                MultiByteToWideChar(CP_ACP, 0,
                    if !password.is_null() { password } else { (*ntlm_cred).password },
                    if !password.is_null() { pwlen } else { (*ntlm_cred).pwlen },
                    unicode_password, passwd_lenW);

                SECUR32_CreateNTLM1SessionKey(unicode_password as PBYTE,
                    passwd_lenW * core::mem::size_of::<SEC_WCHAR>() as c_int, (*helper).session_key);

                HeapFree(GetProcessHeap(), 0, unicode_password as _);
            } else {
                memset((*helper).session_key as _, 0, 16);
            }

            want_flags = HeapAlloc(GetProcessHeap(), 0, 73) as *mut c_char;
            if want_flags.is_null() {
                cleanup_helper(helper);
                ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
                break 'isc_end;
            }
            lstrcpyA(want_flags, b"SF\0".as_ptr() as LPCSTR);
            if fContextReq & ISC_REQ_CONFIDENTIALITY != 0 {
                if strstr(want_flags, b"NTLMSSP_FEATURE_SEAL\0".as_ptr() as *const c_char).is_null() {
                    lstrcatA(want_flags, b" NTLMSSP_FEATURE_SEAL\0".as_ptr() as LPCSTR);
                }
            }
            if fContextReq & ISC_REQ_CONNECTION != 0 {
                ctxt_attr |= ISC_RET_CONNECTION;
            }
            if fContextReq & ISC_REQ_EXTENDED_ERROR != 0 {
                ctxt_attr |= ISC_RET_EXTENDED_ERROR;
            }
            if fContextReq & ISC_REQ_INTEGRITY != 0 {
                if strstr(want_flags, b"NTLMSSP_FEATURE_SIGN\0".as_ptr() as *const c_char).is_null() {
                    lstrcatA(want_flags, b" NTLMSSP_FEATURE_SIGN\0".as_ptr() as LPCSTR);
                }
            }
            if fContextReq & ISC_REQ_MUTUAL_AUTH != 0 {
                ctxt_attr |= ISC_RET_MUTUAL_AUTH;
            }
            if fContextReq & ISC_REQ_REPLAY_DETECT != 0 {
                if strstr(want_flags, b"NTLMSSP_FEATURE_SIGN\0".as_ptr() as *const c_char).is_null() {
                    lstrcatA(want_flags, b" NTLMSSP_FEATURE_SIGN\0".as_ptr() as LPCSTR);
                }
            }
            if fContextReq & ISC_REQ_SEQUENCE_DETECT != 0 {
                if strstr(want_flags, b"NTLMSSP_FEATURE_SIGN\0".as_ptr() as *const c_char).is_null() {
                    lstrcatA(want_flags, b" NTLMSSP_FEATURE_SIGN\0".as_ptr() as LPCSTR);
                }
            }
            if fContextReq & ISC_REQ_STREAM != 0 {
            }
            if fContextReq & ISC_REQ_USE_DCE_STYLE != 0 {
                ctxt_attr |= ISC_RET_USED_DCE_STYLE;
            }
            if fContextReq & ISC_REQ_DELEGATE != 0 {
                ctxt_attr |= ISC_RET_DELEGATE;
            }

            if password.is_null() && (*ntlm_cred).password.is_null() {
                lstrcpynA(buffer, b"OK\0".as_ptr() as LPCSTR, max_len - 1);
                ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
                if ret != SEC_E_OK as SECURITY_STATUS {
                    cleanup_helper(helper);
                    break 'isc_end;
                }
                if strncmp(buffer, b"PW\0".as_ptr() as *const c_char, 2) == 0 {
                    lstrcpynA(buffer, b"PW AA==\0".as_ptr() as LPCSTR, max_len - 1);
                } else {
                    lstrcpynA(buffer, b"OK\0".as_ptr() as LPCSTR, max_len - 1);
                }
            } else {
                lstrcpynA(buffer, b"PW \0".as_ptr() as LPCSTR, max_len - 1);
                ret = encodeBase64(
                    if !password.is_null() { password as *mut c_uchar } else { (*ntlm_cred).password as *mut c_uchar },
                    if !password.is_null() { pwlen } else { (*ntlm_cred).pwlen },
                    buffer.add(3), max_len - 3, &mut buffer_len);
                if ret != SEC_E_OK as SECURITY_STATUS {
                    cleanup_helper(helper);
                    break 'isc_end;
                }
            }

            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                cleanup_helper(helper);
                break 'isc_end;
            }

            if lstrlenA(want_flags) > 2 {
                lstrcpynA(buffer, want_flags, max_len - 1);
                ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
                if ret != SEC_E_OK as SECURITY_STATUS {
                    cleanup_helper(helper);
                    break 'isc_end;
                }
                if strncmp(buffer, b"BH\0".as_ptr() as *const c_char, 2) == 0 {
                }
            }

            lstrcpynA(buffer, b"YR\0".as_ptr() as LPCSTR, max_len - 1);

            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                cleanup_helper(helper);
                break 'isc_end;
            }

            if strncmp(buffer, b"YR \0".as_ptr() as *const c_char, 3) != 0 {
                ret = SEC_E_INTERNAL_ERROR as SECURITY_STATUS;
                cleanup_helper(helper);
                break 'isc_end;
            }
            ret = decodeBase64(buffer.add(3), buffer_len - 3, bin, max_len - 1, &mut bin_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                cleanup_helper(helper);
                break 'isc_end;
            }

            (*phNewContext).dwUpper = ctxt_attr as ULONG_PTR;
            (*phNewContext).dwLower = helper as ULONG_PTR;

            ret = SEC_I_CONTINUE_NEEDED as SECURITY_STATUS;
        } else {
            let input_token_idx: c_int;

            if pInput.is_null() {
                ret = SEC_E_INVALID_TOKEN as SECURITY_STATUS;
                break 'isc_end;
            }
            input_token_idx = ntlm_GetTokenBufferIndex(pInput);
            if input_token_idx == -1 {
                ret = SEC_E_INVALID_TOKEN as SECURITY_STATUS;
                break 'isc_end;
            }

            if phContext.is_null() {
                ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
                break 'isc_end;
            }

            helper = (*phContext).dwLower as PNegoHelper;
            if (*helper).mode != NTLM_CLIENT {
                ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
                break 'isc_end;
            }

            if (*(*pInput).pBuffers.add(input_token_idx as usize)).pvBuffer.is_null() {
                ret = SEC_E_INTERNAL_ERROR as SECURITY_STATUS;
                break 'isc_end;
            }

            if (*(*pInput).pBuffers.add(input_token_idx as usize)).cbBuffer as c_int > max_len {
                ret = SEC_E_INVALID_TOKEN as SECURITY_STATUS;
                break 'isc_end;
            } else {
                bin_len = (*(*pInput).pBuffers.add(input_token_idx as usize)).cbBuffer as c_int;
            }

            memcpy(bin as _, (*(*pInput).pBuffers.add(input_token_idx as usize)).pvBuffer, bin_len as _);

            lstrcpynA(buffer, b"TT \0".as_ptr() as LPCSTR, max_len - 1);

            ret = encodeBase64(bin, bin_len, buffer.add(3), max_len - 3, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'isc_end;
            }

            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'isc_end;
            }

            if strncmp(buffer, b"KK \0".as_ptr() as *const c_char, 3) != 0
                && strncmp(buffer, b"AF \0".as_ptr() as *const c_char, 3) != 0 {
                ret = SEC_E_INVALID_TOKEN as SECURITY_STATUS;
                break 'isc_end;
            }

            ret = decodeBase64(buffer.add(3), buffer_len - 3, bin, max_len, &mut bin_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'isc_end;
            }

            (*phNewContext).dwUpper = ctxt_attr as ULONG_PTR;
            (*phNewContext).dwLower = helper as ULONG_PTR;

            ret = SEC_E_OK as SECURITY_STATUS;
        }

        if pOutput.is_null() {
            ret = SEC_E_BUFFER_TOO_SMALL as SECURITY_STATUS;
            if phContext.is_null() && pInput.is_null() {
                cleanup_helper(helper);
                (*phNewContext).dwUpper = 0;
                (*phNewContext).dwLower = 0;
            }
            break 'isc_end;
        }
        token_idx = ntlm_GetTokenBufferIndex(pOutput);
        if token_idx == -1 {
            ret = SEC_E_BUFFER_TOO_SMALL as SECURITY_STATUS;
            if phContext.is_null() && pInput.is_null() {
                cleanup_helper(helper);
                (*phNewContext).dwUpper = 0;
                (*phNewContext).dwLower = 0;
            }
            break 'isc_end;
        }

        if fContextReq & ISC_REQ_ALLOCATE_MEMORY != 0 {
            (*(*pOutput).pBuffers.add(token_idx as usize)).pvBuffer = HeapAlloc(GetProcessHeap(), 0, bin_len as _);
            (*(*pOutput).pBuffers.add(token_idx as usize)).cbBuffer = bin_len as ULONG;
        } else if ((*(*pOutput).pBuffers.add(token_idx as usize)).cbBuffer as c_int) < bin_len {
            ret = SEC_E_BUFFER_TOO_SMALL as SECURITY_STATUS;
            if phContext.is_null() && pInput.is_null() {
                cleanup_helper(helper);
                (*phNewContext).dwUpper = 0;
                (*phNewContext).dwLower = 0;
            }
            break 'isc_end;
        }

        if (*(*pOutput).pBuffers.add(token_idx as usize)).pvBuffer.is_null() {
            ret = SEC_E_INTERNAL_ERROR as SECURITY_STATUS;
            if phContext.is_null() && pInput.is_null() {
                cleanup_helper(helper);
                (*phNewContext).dwUpper = 0;
                (*phNewContext).dwLower = 0;
            }
            break 'isc_end;
        }

        (*(*pOutput).pBuffers.add(token_idx as usize)).cbBuffer = bin_len as ULONG;
        memcpy((*(*pOutput).pBuffers.add(token_idx as usize)).pvBuffer, bin as _, bin_len as _);

        if ret == SEC_E_OK as SECURITY_STATUS {
            lstrcpynA(buffer, b"GF\0".as_ptr() as LPCSTR, max_len - 1);
            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'isc_end;
            }

            if buffer_len < 3 {
                (*helper).neg_flags = 0;
            } else {
                sscanf(buffer.add(3), b"%lx\0".as_ptr() as *const c_char, &mut (*helper).neg_flags);
            }

            lstrcpynA(buffer, b"GK\0".as_ptr() as LPCSTR, max_len - 1);
            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'isc_end;
            }

            if strncmp(buffer, b"BH\0".as_ptr() as *const c_char, 2) == 0 {
            } else if strncmp(buffer, b"GK \0".as_ptr() as *const c_char, 3) == 0 {
                ret = decodeBase64(buffer.add(3), buffer_len - 3, bin, max_len, &mut bin_len);
                if ret != SEC_E_OK as SECURITY_STATUS {
                }
                HeapFree(GetProcessHeap(), 0, (*helper).session_key as _);
                (*helper).session_key = HeapAlloc(GetProcessHeap(), 0, bin_len as _) as PBYTE;
                if (*helper).session_key.is_null() {
                    ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
                    break 'isc_end;
                }
                memcpy((*helper).session_key as _, bin as _, bin_len as _);
            }

            (*helper).crypt.ntlm.a4i = SECUR32_arc4Alloc();
            SECUR32_arc4Init((*helper).crypt.ntlm.a4i, (*helper).session_key, 16);
            (*helper).crypt.ntlm.seq_num = 0;
            SECUR32_CreateNTLM2SubKeys(helper);
            (*helper).crypt.ntlm2.send_a4i = SECUR32_arc4Alloc();
            (*helper).crypt.ntlm2.recv_a4i = SECUR32_arc4Alloc();
            SECUR32_arc4Init((*helper).crypt.ntlm2.send_a4i,
                (*helper).crypt.ntlm2.send_seal_key, 16);
            SECUR32_arc4Init((*helper).crypt.ntlm2.recv_a4i,
                (*helper).crypt.ntlm2.recv_seal_key, 16);
            (*helper).crypt.ntlm2.send_seq_no = 0;
            (*helper).crypt.ntlm2.recv_seq_no = 0;
        }

        break 'isc_end;
    }

    HeapFree(GetProcessHeap(), 0, username as _);
    HeapFree(GetProcessHeap(), 0, domain as _);
    HeapFree(GetProcessHeap(), 0, password as _);
    HeapFree(GetProcessHeap(), 0, want_flags as _);
    HeapFree(GetProcessHeap(), 0, buffer as _);
    HeapFree(GetProcessHeap(), 0, bin as _);
    ret
}
unsafe extern "system" fn ntlm_InitializeSecurityContextA(
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

    if pszTargetName != core::ptr::null_mut() {
        let target_size = MultiByteToWideChar(
            CP_ACP,
            0,
            pszTargetName,
            (strlen(pszTargetName) + 1) as i32,
            core::ptr::null_mut(),
            0,
        );
        target = HeapAlloc(
            GetProcessHeap(),
            0,
            ((target_size as usize) * core::mem::size_of::<SEC_WCHAR>()) as _,
        ) as *mut SEC_WCHAR;
        MultiByteToWideChar(
            CP_ACP,
            0,
            pszTargetName,
            (strlen(pszTargetName) + 1) as i32,
            target,
            target_size,
        );
    }

    ret = ntlm_InitializeSecurityContextW(
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

    HeapFree(GetProcessHeap(), 0, target as *mut _);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_AcceptSecurityContext(
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
    let mut ret: SECURITY_STATUS = SEC_E_OK as SECURITY_STATUS;
    let buffer: *mut core::ffi::c_char;
    let mut want_flags: *mut core::ffi::c_char = core::ptr::null_mut();
    let bin: PBYTE;
    let mut buffer_len: i32 = 0;
    let mut bin_len: i32 = 0;
    let max_len: i32 = NTLM_MAX_BUF as i32;
    let mut ctxt_attr: ULONG = 0;
    let mut helper: PNegoHelper = core::ptr::null_mut();
    let ntlm_cred: PNtlmCredentials;

    buffer = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<core::ffi::c_char>() * NTLM_MAX_BUF as usize) as _) as *mut core::ffi::c_char;
    bin = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<u8>() * NTLM_MAX_BUF as usize) as _) as PBYTE;

    if TargetDataRep == SECURITY_NETWORK_DREP as ULONG {
    }

    'asc_end: {
        if phContext.is_null() {
            let mut server_helper_protocol: [core::ffi::c_char; 36] = {
                let s = b"--helper-protocol=squid-2.5-ntlmssp\0";
                let mut arr = [0 as core::ffi::c_char; 36];
                let mut i = 0;
                while i < 36 { arr[i] = s[i] as core::ffi::c_char; i += 1; }
                arr
            };
            let mut server_argv: [*mut core::ffi::c_char; 3] = [
                ntlm_auth.as_mut_ptr() as *mut core::ffi::c_char,
                server_helper_protocol.as_mut_ptr(),
                core::ptr::null_mut(),
            ];

            if phCredential.is_null() {
                ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
                break 'asc_end;
            }

            ntlm_cred = (*phCredential).dwLower as PNtlmCredentials;

            if (*ntlm_cred).mode != NTLM_SERVER {
                ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
                break 'asc_end;
            }

            if pInput.is_null() {
                ret = SEC_E_INCOMPLETE_MESSAGE as SECURITY_STATUS;
                break 'asc_end;
            }

            if (*pInput).cBuffers < 1 {
                ret = SEC_E_INCOMPLETE_MESSAGE as SECURITY_STATUS;
                break 'asc_end;
            }

            if (*(*pInput).pBuffers.offset(0)).cbBuffer as i32 > max_len {
                ret = SEC_E_INVALID_TOKEN as SECURITY_STATUS;
                break 'asc_end;
            } else {
                bin_len = (*(*pInput).pBuffers.offset(0)).cbBuffer as i32;
            }

            ret = fork_helper(&mut helper, ntlm_auth.as_mut_ptr() as *mut core::ffi::c_char, server_argv.as_mut_ptr());
            if ret != SEC_E_OK as SECURITY_STATUS {
                ret = SEC_E_INTERNAL_ERROR as SECURITY_STATUS;
                break 'asc_end;
            }
            (*helper).mode = NTLM_SERVER;

            want_flags = HeapAlloc(GetProcessHeap(), 0, 73) as *mut core::ffi::c_char;
            if want_flags.is_null() {
                ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
                cleanup_helper(helper);
                break 'asc_end;
            }
            lstrcpyA(want_flags, b"SF\0".as_ptr() as LPCSTR);
            if fContextReq & ASC_REQ_ALLOCATE_MEMORY != 0 {
            }
            if fContextReq & ASC_REQ_CONFIDENTIALITY != 0 {
                lstrcatA(want_flags, b" NTLMSSP_FEATURE_SEAL\0".as_ptr() as LPCSTR);
            }
            if fContextReq & ASC_REQ_CONNECTION != 0 {
                lstrcatA(want_flags, b" NTLMSSP_FEATURE_SESSION_KEY\0".as_ptr() as LPCSTR);
                ctxt_attr |= ASC_RET_CONNECTION;
            }
            if fContextReq & ASC_REQ_EXTENDED_ERROR != 0 {
            }
            if fContextReq & ASC_REQ_INTEGRITY != 0 {
                lstrcatA(want_flags, b" NTLMSSP_FEATURE_SIGN\0".as_ptr() as LPCSTR);
            }
            if fContextReq & ASC_REQ_MUTUAL_AUTH != 0 {
            }
            if fContextReq & ASC_REQ_REPLAY_DETECT != 0 {
            }
            if fContextReq & ISC_REQ_SEQUENCE_DETECT != 0 {
            }
            if fContextReq & ISC_REQ_STREAM != 0 {
            }

            if lstrlenA(want_flags) > 3 {
                lstrcpynA(buffer, want_flags, max_len - 1);
                ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
                if ret != SEC_E_OK as SECURITY_STATUS {
                    cleanup_helper(helper);
                    break 'asc_end;
                }
                if strncmp(buffer, b"BH\0".as_ptr() as LPCSTR, 2) == 0 {
                }
            }

            memcpy(bin as *mut _, (*(*pInput).pBuffers.offset(0)).pvBuffer, bin_len as _);

            lstrcpynA(buffer, b"YR \0".as_ptr() as LPCSTR, max_len - 1);

            ret = encodeBase64(bin, bin_len, buffer.offset(3), max_len - 3, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                cleanup_helper(helper);
                break 'asc_end;
            }

            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                cleanup_helper(helper);
                break 'asc_end;
            }

            if strncmp(buffer, b"TT \0".as_ptr() as LPCSTR, 3) != 0 {
                ret = SEC_E_INTERNAL_ERROR as SECURITY_STATUS;
                cleanup_helper(helper);
                break 'asc_end;
            }

            ret = decodeBase64(buffer.offset(3), buffer_len - 3, bin, max_len, &mut bin_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                cleanup_helper(helper);
                break 'asc_end;
            }

            if pOutput.is_null() {
                ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
                cleanup_helper(helper);
                break 'asc_end;
            }

            if (*pOutput).cBuffers < 1 {
                ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
                cleanup_helper(helper);
                break 'asc_end;
            }

            (*(*pOutput).pBuffers.offset(0)).cbBuffer = bin_len as ULONG;
            (*(*pOutput).pBuffers.offset(0)).BufferType = SECBUFFER_DATA;
            memcpy((*(*pOutput).pBuffers.offset(0)).pvBuffer, bin as *const _, bin_len as _);
            ret = SEC_I_CONTINUE_NEEDED as SECURITY_STATUS;
        } else {
            if pInput.is_null() {
                ret = SEC_E_INCOMPLETE_MESSAGE as SECURITY_STATUS;
                break 'asc_end;
            }

            if (*pInput).cBuffers < 1 {
                ret = SEC_E_INCOMPLETE_MESSAGE as SECURITY_STATUS;
                break 'asc_end;
            }

            helper = (*phContext).dwLower as PNegoHelper;

            if (*helper).mode != NTLM_SERVER {
                ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
                break 'asc_end;
            }

            if (*(*pInput).pBuffers.offset(0)).cbBuffer as i32 > max_len {
                ret = SEC_E_INVALID_TOKEN as SECURITY_STATUS;
                break 'asc_end;
            } else {
                bin_len = (*(*pInput).pBuffers.offset(0)).cbBuffer as i32;
            }

            memcpy(bin as *mut _, (*(*pInput).pBuffers.offset(0)).pvBuffer, bin_len as _);

            lstrcpynA(buffer, b"KK \0".as_ptr() as LPCSTR, max_len - 1);

            ret = encodeBase64(bin, bin_len, buffer.offset(3), max_len - 3, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'asc_end;
            }

            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'asc_end;
            }

            if strncmp(buffer, b"AF \0".as_ptr() as LPCSTR, 3) != 0 {
                if strncmp(buffer, b"NA \0".as_ptr() as LPCSTR, 3) == 0 {
                    ret = SEC_E_LOGON_DENIED as SECURITY_STATUS;
                    break 'asc_end;
                } else {
                    let ntlm_pipe_err_v3_len = strlen(b"BH NT_STATUS_ACCESS_DENIED\0".as_ptr() as LPCSTR);
                    let ntlm_pipe_err_v4_len = strlen(b"BH NT_STATUS_UNSUCCESSFUL\0".as_ptr() as LPCSTR);

                    if (buffer_len as usize >= ntlm_pipe_err_v3_len
                        && strncmp(buffer, b"BH NT_STATUS_ACCESS_DENIED\0".as_ptr() as LPCSTR, ntlm_pipe_err_v3_len) == 0)
                        || (buffer_len as usize >= ntlm_pipe_err_v4_len
                            && strncmp(buffer, b"BH NT_STATUS_UNSUCCESSFUL\0".as_ptr() as LPCSTR, ntlm_pipe_err_v4_len) == 0)
                    {
                        ret = SEC_E_LOGON_DENIED as SECURITY_STATUS;
                    } else {
                        ret = SEC_E_INTERNAL_ERROR as SECURITY_STATUS;
                    }

                    break 'asc_end;
                }
            }
            (*(*pOutput).pBuffers.offset(0)).cbBuffer = 0;

            lstrcpynA(buffer, b"GF\0".as_ptr() as LPCSTR, max_len - 1);
            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'asc_end;
            }

            if buffer_len < 3 {
            } else {
                sscanf(buffer.offset(3), b"%lx\0".as_ptr() as LPCSTR, core::ptr::addr_of_mut!((*helper).neg_flags));
            }

            lstrcpynA(buffer, b"GK\0".as_ptr() as LPCSTR, max_len - 1);
            ret = run_helper(helper, buffer, max_len as _, &mut buffer_len);
            if ret != SEC_E_OK as SECURITY_STATUS {
                break 'asc_end;
            }

            if buffer_len < 3 {
            } else {
                if strncmp(buffer, b"BH \0".as_ptr() as LPCSTR, 3) == 0 {
                    HeapFree(GetProcessHeap(), 0, (*helper).session_key as *mut _);
                    (*helper).session_key = HeapAlloc(GetProcessHeap(), 0, 16) as *mut _;
                    if (*helper).session_key.is_null() {
                        ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
                        break 'asc_end;
                    }
                    memset((*helper).session_key as *mut _, 0, 16);
                } else if strncmp(buffer, b"GK \0".as_ptr() as LPCSTR, 3) == 0 {
                    ret = decodeBase64(buffer.offset(3), buffer_len - 3, bin, max_len, &mut bin_len);
                    if ret != SEC_E_OK as SECURITY_STATUS {
                    }
                    HeapFree(GetProcessHeap(), 0, (*helper).session_key as *mut _);
                    (*helper).session_key = HeapAlloc(GetProcessHeap(), 0, 16) as *mut _;
                    if (*helper).session_key.is_null() {
                        ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
                        break 'asc_end;
                    }
                    memcpy((*helper).session_key as *mut _, bin as *const _, 16);
                }
            }
            (*helper).crypt.ntlm.a4i = SECUR32_arc4Alloc();
            SECUR32_arc4Init((*helper).crypt.ntlm.a4i, (*helper).session_key, 16);
            (*helper).crypt.ntlm.seq_num = 0;
        }

        (*phNewContext).dwUpper = ctxt_attr as ULONG_PTR;
        (*phNewContext).dwLower = helper as ULONG_PTR;
    }

    HeapFree(GetProcessHeap(), 0, want_flags as *mut _);
    HeapFree(GetProcessHeap(), 0, buffer as *mut _);
    HeapFree(GetProcessHeap(), 0, bin as *mut _);
    ret
}
unsafe extern "system" fn ntlm_CompleteAuthToken(phContext: PCtxtHandle, pToken: PSecBufferDesc) -> SECURITY_STATUS {
    TRACE!(b"%p %p\n\0".as_ptr() as *const i8, phContext, pToken);
    if phContext.is_null() {
        return SEC_E_INVALID_HANDLE;
    }
    SEC_E_OK
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_DeleteSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    let helper: PNegoHelper;

    if phContext.is_null() {
        return SEC_E_INVALID_HANDLE;
    }

    helper = (*phContext).dwLower as PNegoHelper;

    (*phContext).dwUpper = 0;
    (*phContext).dwLower = 0;

    SECUR32_arc4Cleanup((*helper).crypt.ntlm.a4i);
    SECUR32_arc4Cleanup((*helper).crypt.ntlm2.send_a4i);
    SECUR32_arc4Cleanup((*helper).crypt.ntlm2.recv_a4i);
    HeapFree(GetProcessHeap(), 0, (*helper).crypt.ntlm2.send_sign_key as _);
    HeapFree(GetProcessHeap(), 0, (*helper).crypt.ntlm2.send_seal_key as _);
    HeapFree(GetProcessHeap(), 0, (*helper).crypt.ntlm2.recv_sign_key as _);
    HeapFree(GetProcessHeap(), 0, (*helper).crypt.ntlm2.recv_seal_key as _);

    cleanup_helper(helper);

    SEC_E_OK
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_QueryContextAttributesW(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    if phContext.is_null() {
        return SEC_E_INVALID_HANDLE;
    }

    match ulAttribute {
        SECPKG_ATTR_FLAGS => {
            let spcf = pBuffer as PSecPkgContext_Flags;
            let helper = (*phContext).dwLower as PNegoHelper;

            (*spcf).Flags = 0;
            if (*helper).neg_flags & NTLMSSP_NEGOTIATE_SIGN != 0 {
                (*spcf).Flags |= ISC_RET_INTEGRITY;
            }
            if (*helper).neg_flags & NTLMSSP_NEGOTIATE_SEAL != 0 {
                (*spcf).Flags |= ISC_RET_CONFIDENTIALITY;
            }
            return SEC_E_OK;
        }
        SECPKG_ATTR_SIZES => {
            let spcs = pBuffer as PSecPkgContext_Sizes;
            (*spcs).cbMaxToken = NTLM_MAX_BUF;
            (*spcs).cbMaxSignature = 16;
            (*spcs).cbBlockSize = 0;
            (*spcs).cbSecurityTrailer = 16;
            return SEC_E_OK;
        }
        SECPKG_ATTR_ACCESS_TOKEN
        | SECPKG_ATTR_AUTHORITY
        | SECPKG_ATTR_DCE_INFO
        | SECPKG_ATTR_KEY_INFO
        | SECPKG_ATTR_LIFESPAN
        | SECPKG_ATTR_NAMES
        | SECPKG_ATTR_NATIVE_NAMES
        | SECPKG_ATTR_NEGOTIATION_INFO
        | SECPKG_ATTR_PACKAGE_INFO
        | SECPKG_ATTR_PASSWORD_EXPIRY
        | SECPKG_ATTR_SESSION_KEY
        | SECPKG_ATTR_STREAM_SIZES
        | SECPKG_ATTR_TARGET_INFORMATION => {}
        _ => {}
    }

    SEC_E_UNSUPPORTED_FUNCTION
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_QueryContextAttributesA(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    ntlm_QueryContextAttributesW(phContext, ulAttribute, pBuffer)
}
unsafe extern "system" fn ntlm_ImpersonateSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phContext.is_null() {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
unsafe extern "system" fn ntlm_RevertSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;
    if !phContext.is_null() {
        ret = SEC_E_UNSUPPORTED_FUNCTION;
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}

unsafe fn ntlm_CreateSignature(
    helper: PNegoHelper,
    pMessage: PSecBufferDesc,
    token_idx: core::ffi::c_int,
    direction: SignDirection,
    encrypt_sig: BOOL,
) -> SECURITY_STATUS {
    let sign_version: ULONG = 1;
    let mut i: UINT;
    let sig: PBYTE;

    sig = (*(*pMessage).pBuffers.offset(token_idx as isize)).pvBuffer as PBYTE;

    if (*helper).neg_flags & NTLMSSP_NEGOTIATE_NTLM2 != 0
        && (*helper).neg_flags & NTLMSSP_NEGOTIATE_SIGN != 0
    {
        let mut digest: [BYTE; 16] = [0; 16];
        let mut seq_no: [BYTE; 4] = [0; 4];
        let mut hmac_md5_ctx: HMAC_MD5_CTX = core::mem::zeroed();

        if direction == NTLM_SEND {
            seq_no[0] = (((*helper).crypt.ntlm2.send_seq_no >> 0) & 0xff) as BYTE;
            seq_no[1] = (((*helper).crypt.ntlm2.send_seq_no >> 8) & 0xff) as BYTE;
            seq_no[2] = (((*helper).crypt.ntlm2.send_seq_no >> 16) & 0xff) as BYTE;
            seq_no[3] = (((*helper).crypt.ntlm2.send_seq_no >> 24) & 0xff) as BYTE;

            (*helper).crypt.ntlm2.send_seq_no += 1;

            HMACMD5Init(
                &mut hmac_md5_ctx,
                (*helper).crypt.ntlm2.send_sign_key,
                16,
            );
        } else {
            seq_no[0] = (((*helper).crypt.ntlm2.recv_seq_no >> 0) & 0xff) as BYTE;
            seq_no[1] = (((*helper).crypt.ntlm2.recv_seq_no >> 8) & 0xff) as BYTE;
            seq_no[2] = (((*helper).crypt.ntlm2.recv_seq_no >> 16) & 0xff) as BYTE;
            seq_no[3] = (((*helper).crypt.ntlm2.recv_seq_no >> 24) & 0xff) as BYTE;

            (*helper).crypt.ntlm2.recv_seq_no += 1;

            HMACMD5Init(
                &mut hmac_md5_ctx,
                (*helper).crypt.ntlm2.recv_sign_key,
                16,
            );
        }

        HMACMD5Update(&mut hmac_md5_ctx, seq_no.as_ptr(), 4);
        i = 0;
        while i < (*pMessage).cBuffers {
            if (*(*pMessage).pBuffers.offset(i as isize)).BufferType & SECBUFFER_DATA != 0 {
                HMACMD5Update(
                    &mut hmac_md5_ctx,
                    (*(*pMessage).pBuffers.offset(i as isize)).pvBuffer as *const BYTE,
                    (*(*pMessage).pBuffers.offset(i as isize)).cbBuffer as _,
                );
            }
            i += 1;
        }

        HMACMD5Final(&mut hmac_md5_ctx, digest.as_mut_ptr());

        if encrypt_sig != 0 && (*helper).neg_flags & NTLMSSP_NEGOTIATE_KEY_EXCHANGE != 0 {
            if direction == NTLM_SEND {
                SECUR32_arc4Process((*helper).crypt.ntlm2.send_a4i, digest.as_mut_ptr(), 8);
            } else {
                SECUR32_arc4Process((*helper).crypt.ntlm2.recv_a4i, digest.as_mut_ptr(), 8);
            }
        }

        *sig.offset(0) = ((sign_version >> 0) & 0xff) as BYTE;
        *sig.offset(1) = ((sign_version >> 8) & 0xff) as BYTE;
        *sig.offset(2) = ((sign_version >> 16) & 0xff) as BYTE;
        *sig.offset(3) = ((sign_version >> 24) & 0xff) as BYTE;
        memcpy(
            sig.offset(4) as *mut core::ffi::c_void,
            digest.as_ptr() as *const core::ffi::c_void,
            8,
        );
        memcpy(
            sig.offset(12) as *mut core::ffi::c_void,
            seq_no.as_ptr() as *const core::ffi::c_void,
            4,
        );

        (*(*pMessage).pBuffers.offset(token_idx as isize)).cbBuffer = 16;

        return SEC_E_OK;
    }

    if (*helper).neg_flags & NTLMSSP_NEGOTIATE_SIGN != 0 {
        let mut crc: ULONG = 0;

        i = 0;
        while i < (*pMessage).cBuffers {
            if (*(*pMessage).pBuffers.offset(i as isize)).BufferType & SECBUFFER_DATA != 0 {
                crc = ComputeCrc32(
                    (*(*pMessage).pBuffers.offset(i as isize)).pvBuffer as _,
                    (*(*pMessage).pBuffers.offset(i as isize)).cbBuffer as _,
                    crc,
                );
            }
            i += 1;
        }

        *sig.offset(0) = ((sign_version >> 0) & 0xff) as BYTE;
        *sig.offset(1) = ((sign_version >> 8) & 0xff) as BYTE;
        *sig.offset(2) = ((sign_version >> 16) & 0xff) as BYTE;
        *sig.offset(3) = ((sign_version >> 24) & 0xff) as BYTE;
        memset(sig.offset(4) as *mut core::ffi::c_void, 0, 4);
        *sig.offset(8) = ((crc >> 0) & 0xff) as BYTE;
        *sig.offset(9) = ((crc >> 8) & 0xff) as BYTE;
        *sig.offset(10) = ((crc >> 16) & 0xff) as BYTE;
        *sig.offset(11) = ((crc >> 24) & 0xff) as BYTE;
        *sig.offset(12) = (((*helper).crypt.ntlm.seq_num >> 0) & 0xff) as BYTE;
        *sig.offset(13) = (((*helper).crypt.ntlm.seq_num >> 8) & 0xff) as BYTE;
        *sig.offset(14) = (((*helper).crypt.ntlm.seq_num >> 16) & 0xff) as BYTE;
        *sig.offset(15) = (((*helper).crypt.ntlm.seq_num >> 24) & 0xff) as BYTE;

        (*helper).crypt.ntlm.seq_num += 1;

        if encrypt_sig != 0 {
            SECUR32_arc4Process((*helper).crypt.ntlm.a4i, sig.offset(4), 12);
        }
        return SEC_E_OK;
    }

    if (*helper).neg_flags & NTLMSSP_NEGOTIATE_ALWAYS_SIGN != 0 || (*helper).neg_flags == 0 {
        memset(
            (*(*pMessage).pBuffers.offset(token_idx as isize)).pvBuffer as _,
            0,
            16,
        );
        memset(
            (*(*pMessage).pBuffers.offset(token_idx as isize)).pvBuffer as _,
            0x01,
            1,
        );
        (*(*pMessage).pBuffers.offset(token_idx as isize)).cbBuffer = 16;
        return SEC_E_OK;
    }

    SEC_E_UNSUPPORTED_FUNCTION
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_MakeSignature(
    phContext: PCtxtHandle,
    fQOP: ULONG,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
) -> SECURITY_STATUS {
    let helper: PNegoHelper;
    let token_idx: core::ffi::c_int;

    if phContext.is_null() {
        return SEC_E_INVALID_HANDLE;
    }

    if fQOP != 0 {
    }

    if MessageSeqNo != 0 {
    }

    if pMessage.is_null() || (*pMessage).pBuffers.is_null() || (*pMessage).cBuffers < 2 {
        return SEC_E_INVALID_TOKEN;
    }

    token_idx = ntlm_GetTokenBufferIndex(pMessage);
    if token_idx == -1 {
        return SEC_E_INVALID_TOKEN;
    }

    if (*(*pMessage).pBuffers.offset(token_idx as isize)).cbBuffer < 16 {
        return SEC_E_BUFFER_TOO_SMALL;
    }

    helper = (*phContext).dwLower as PNegoHelper;

    ntlm_CreateSignature(helper, pMessage, token_idx, SignDirection_NTLM_SEND, TRUE as BOOL)
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_VerifySignature(
    phContext: PCtxtHandle,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
    pfQOP: PULONG,
) -> SECURITY_STATUS {
    let helper: PNegoHelper;
    let mut i: UINT;
    let token_idx: core::ffi::c_int;
    let mut ret: SECURITY_STATUS;
    let mut local_desc: SecBufferDesc = core::mem::zeroed();
    let local_buff: PSecBuffer;
    let mut local_sig: [BYTE; 16] = [0; 16];

    if phContext.is_null() {
        return SEC_E_INVALID_HANDLE;
    }

    if pMessage.is_null() || (*pMessage).pBuffers.is_null() || (*pMessage).cBuffers < 2 {
        return SEC_E_INVALID_TOKEN;
    }

    token_idx = ntlm_GetTokenBufferIndex(pMessage);
    if token_idx == -1 {
        return SEC_E_INVALID_TOKEN;
    }

    if (*(*pMessage).pBuffers.offset(token_idx as isize)).cbBuffer < 16 {
        return SEC_E_BUFFER_TOO_SMALL;
    }

    if MessageSeqNo != 0 {
        FIXME!(b"Ignoring MessageSeqNo\n\0".as_ptr() as LPCSTR);
    }

    helper = (*phContext).dwLower as PNegoHelper;

    local_buff = HeapAlloc(
        GetProcessHeap(),
        0,
        ((*pMessage).cBuffers as usize * core::mem::size_of::<SecBuffer>()) as _,
    ) as PSecBuffer;

    local_desc.ulVersion = SECBUFFER_VERSION;
    local_desc.cBuffers = (*pMessage).cBuffers;
    local_desc.pBuffers = local_buff;

    i = 0;
    while i < (*pMessage).cBuffers {
        let src = (*pMessage).pBuffers.offset(i as isize);
        let dst = local_buff.offset(i as isize);
        if (*src).BufferType == SECBUFFER_TOKEN {
            (*dst).BufferType = SECBUFFER_TOKEN;
            (*dst).cbBuffer = 16;
            (*dst).pvBuffer = local_sig.as_mut_ptr() as *mut core::ffi::c_void;
        } else {
            (*dst).BufferType = (*src).BufferType;
            (*dst).cbBuffer = (*src).cbBuffer;
            (*dst).pvBuffer = (*src).pvBuffer;
        }
        i += 1;
    }

    ret = ntlm_CreateSignature(helper, &mut local_desc, token_idx, NTLM_RECV, TRUE as BOOL);
    if ret != SEC_E_OK {
        return ret;
    }

    if memcmp(
        ((*local_buff.offset(token_idx as isize)).pvBuffer as PBYTE).offset(8)
            as *const core::ffi::c_void,
        ((*(*pMessage).pBuffers.offset(token_idx as isize)).pvBuffer as PBYTE).offset(8)
            as *const core::ffi::c_void,
        8,
    ) != 0
    {
        ret = SEC_E_MESSAGE_ALTERED;
    } else {
        ret = SEC_E_OK;
    }

    HeapFree(GetProcessHeap(), 0, local_buff as *mut core::ffi::c_void);

    ret
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_FreeCredentialsHandle(phCredential: PCredHandle) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phCredential.is_null() {
        let ntlm_cred = (*phCredential).dwLower as PNtlmCredentials;
        (*phCredential).dwUpper = 0;
        (*phCredential).dwLower = 0;
        if !(*ntlm_cred).password.is_null() {
            memset(
                (*ntlm_cred).password as *mut core::ffi::c_void,
                0,
                (*ntlm_cred).pwlen as usize as _,
            );
        }
        HeapFree(GetProcessHeap(), 0, (*ntlm_cred).password as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, (*ntlm_cred).username_arg as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, (*ntlm_cred).domain_arg as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, ntlm_cred as *mut core::ffi::c_void);
        ret = SEC_E_OK as SECURITY_STATUS;
    } else {
        ret = SEC_E_OK as SECURITY_STATUS;
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_EncryptMessage(
    phContext: PCtxtHandle,
    fQOP: ULONG,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
) -> SECURITY_STATUS {
    let helper: PNegoHelper;
    let token_idx: core::ffi::c_int;
    let data_idx: core::ffi::c_int;

    TRACE!(b"(%p %d %p %d)\n\0".as_ptr() as *const i8, phContext, fQOP, pMessage, MessageSeqNo);

    if phContext.is_null() {
        return SEC_E_INVALID_HANDLE;
    }

    if fQOP != 0 {
        FIXME!(b"Ignoring fQOP\n\0".as_ptr() as *const i8);
    }

    if MessageSeqNo != 0 {
        FIXME!(b"Ignoring MessageSeqNo\n\0".as_ptr() as *const i8);
    }

    if pMessage.is_null() || (*pMessage).pBuffers.is_null() || (*pMessage).cBuffers < 2 {
        return SEC_E_INVALID_TOKEN;
    }

    token_idx = ntlm_GetTokenBufferIndex(pMessage);
    if token_idx == -1 {
        return SEC_E_INVALID_TOKEN;
    }

    data_idx = ntlm_GetDataBufferIndex(pMessage);
    if data_idx == -1 {
        return SEC_E_INVALID_TOKEN;
    }

    if (*(*pMessage).pBuffers.offset(token_idx as isize)).cbBuffer < 16 {
        return SEC_E_BUFFER_TOO_SMALL;
    }

    helper = (*phContext).dwLower as PNegoHelper;

    if (*helper).neg_flags & NTLMSSP_NEGOTIATE_NTLM2 != 0
        && (*helper).neg_flags & NTLMSSP_NEGOTIATE_SEAL != 0
    {
        ntlm_CreateSignature(helper, pMessage, token_idx, NTLM_SEND, FALSE as BOOL);
        SECUR32_arc4Process(
            (*helper).crypt.ntlm2.send_a4i,
            (*(*pMessage).pBuffers.offset(data_idx as isize)).pvBuffer as _,
            (*(*pMessage).pBuffers.offset(data_idx as isize)).cbBuffer,
        );

        if (*helper).neg_flags & NTLMSSP_NEGOTIATE_KEY_EXCHANGE != 0 {
            SECUR32_arc4Process(
                (*helper).crypt.ntlm2.send_a4i,
                ((*(*pMessage).pBuffers.offset(token_idx as isize)).pvBuffer as *mut BYTE).offset(4) as _,
                8,
            );
        }
    } else {
        let sig: PBYTE;
        let save_flags: ULONG;

        save_flags = (*helper).neg_flags;
        (*helper).neg_flags |= NTLMSSP_NEGOTIATE_SIGN;
        ntlm_CreateSignature(helper, pMessage, token_idx, NTLM_SEND, FALSE as BOOL);
        (*helper).neg_flags = save_flags;

        sig = (*(*pMessage).pBuffers.offset(token_idx as isize)).pvBuffer as PBYTE;

        SECUR32_arc4Process(
            (*helper).crypt.ntlm.a4i,
            (*(*pMessage).pBuffers.offset(data_idx as isize)).pvBuffer as _,
            (*(*pMessage).pBuffers.offset(data_idx as isize)).cbBuffer,
        );
        SECUR32_arc4Process((*helper).crypt.ntlm.a4i, sig.offset(4) as _, 12);

        if (*helper).neg_flags & NTLMSSP_NEGOTIATE_ALWAYS_SIGN != 0 || (*helper).neg_flags == 0 {
            memset(sig.offset(4) as *mut core::ffi::c_void, 0, 4);
        }
    }
    SEC_E_OK
}
#[no_mangle]
pub unsafe extern "system" fn ntlm_DecryptMessage(
    phContext: PCtxtHandle,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
    pfQOP: PULONG,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;
    let ntlmssp_flags_save: ULONG;
    let helper: PNegoHelper;
    let token_idx: core::ffi::c_int;
    let data_idx: core::ffi::c_int;

    if phContext.is_null() {
        return SEC_E_INVALID_HANDLE;
    }

    if MessageSeqNo != 0 {
        FIXME!(b"Ignoring MessageSeqNo\n\0".as_ptr() as PCHAR);
    }

    if pMessage.is_null() || (*pMessage).pBuffers.is_null() || (*pMessage).cBuffers < 2 {
        return SEC_E_INVALID_TOKEN;
    }

    token_idx = ntlm_GetTokenBufferIndex(pMessage);
    if token_idx == -1 {
        return SEC_E_INVALID_TOKEN;
    }

    data_idx = ntlm_GetDataBufferIndex(pMessage);
    if data_idx == -1 {
        return SEC_E_INVALID_TOKEN;
    }

    if (*(*pMessage).pBuffers.offset(token_idx as isize)).cbBuffer < 16 {
        return SEC_E_BUFFER_TOO_SMALL;
    }

    helper = (*phContext).dwLower as PNegoHelper;

    if (*helper).neg_flags & NTLMSSP_NEGOTIATE_NTLM2 != 0
        && (*helper).neg_flags & NTLMSSP_NEGOTIATE_SEAL != 0
    {
        SECUR32_arc4Process(
            (*helper).crypt.ntlm2.recv_a4i,
            (*(*pMessage).pBuffers.offset(data_idx as isize)).pvBuffer as _,
            (*(*pMessage).pBuffers.offset(data_idx as isize)).cbBuffer,
        );
    } else {
        SECUR32_arc4Process(
            (*helper).crypt.ntlm.a4i,
            (*(*pMessage).pBuffers.offset(data_idx as isize)).pvBuffer as _,
            (*(*pMessage).pBuffers.offset(data_idx as isize)).cbBuffer,
        );
    }

    ntlmssp_flags_save = (*helper).neg_flags;

    (*helper).neg_flags |= NTLMSSP_NEGOTIATE_SIGN;
    ret = ntlm_VerifySignature(phContext, pMessage, MessageSeqNo, pfQOP);

    (*helper).neg_flags = ntlmssp_flags_save;

    ret
}
#[no_mangle]
pub unsafe extern "C" fn SECUR32_initNTLMSP() {
    let mut helper: PNegoHelper = core::ptr::null_mut();
    static mut version: [CHAR; 10] = [
        b'-' as CHAR, b'-' as CHAR, b'v' as CHAR, b'e' as CHAR, b'r' as CHAR,
        b's' as CHAR, b'i' as CHAR, b'o' as CHAR, b'n' as CHAR, 0,
    ];

    let mut args: [*mut SEC_CHAR; 3] = [
        ntlm_auth.as_mut_ptr() as *mut SEC_CHAR,
        version.as_mut_ptr() as *mut SEC_CHAR,
        core::ptr::null_mut(),
    ];

    if fork_helper(&mut helper, ntlm_auth.as_mut_ptr() as *mut SEC_CHAR, args.as_mut_ptr()) != SEC_E_OK {
        helper = core::ptr::null_mut();
    } else {
        check_version(helper);
    }

    if !helper.is_null()
        && (((*helper).major > MIN_NTLM_AUTH_MAJOR_VERSION)
            || ((*helper).major == MIN_NTLM_AUTH_MAJOR_VERSION
                && (*helper).minor > MIN_NTLM_AUTH_MINOR_VERSION)
            || ((*helper).major == MIN_NTLM_AUTH_MAJOR_VERSION
                && (*helper).minor == MIN_NTLM_AUTH_MINOR_VERSION
                && (*helper).micro >= MIN_NTLM_AUTH_MICRO_VERSION))
    {
        let provider = SECUR32_addProvider(&ntlmTableA, &ntlmTableW, core::ptr::null_mut());
        SECUR32_addPackages(provider, 1, ntlm_package_infoA, ntlm_package_infoW);
    }

    cleanup_helper(helper);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2005, 2006 Kai Blin
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
//  * This file implements the NTLM security provider.
//  */
//
// #include "precomp.h"
//
// #include <assert.h>
// #include <stdio.h>
// #include <wincred.h>
// #include <lm.h>
//
// #include "hmac_md5.h"
//
// #include <wine/unicode.h>
// #include <wine/debug.h>
// WINE_DEFAULT_DEBUG_CHANNEL(ntlm);
// WINE_DECLARE_DEBUG_CHANNEL(winediag);
//
// #define NTLM_MAX_BUF 1904
// #define MIN_NTLM_AUTH_MAJOR_VERSION 3
// #define MIN_NTLM_AUTH_MINOR_VERSION 0
// #ifndef __REACTOS__
// #define MIN_NTLM_AUTH_MICRO_VERSION 25
// #else
// #define MIN_NTLM_AUTH_MICRO_VERSION 23
// #endif
//
// static CHAR ntlm_auth[] = "ntlm_auth";
//
// /***********************************************************************
//  *              QueryCredentialsAttributesA
//  */
// static SECURITY_STATUS SEC_ENTRY ntlm_QueryCredentialsAttributesA(
//         PCredHandle phCredential, ULONG ulAttribute, PVOID pBuffer)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("(%p, %d, %p)\n", phCredential, ulAttribute, pBuffer);
//
//     if(ulAttribute == SECPKG_ATTR_NAMES)
//     {
//         FIXME("SECPKG_CRED_ATTR_NAMES: stub\n");
//         ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//         ret = SEC_E_UNSUPPORTED_FUNCTION;
//     
//     return ret;
// }
//
// /***********************************************************************
//  *              QueryCredentialsAttributesW
//  */
// static SECURITY_STATUS SEC_ENTRY ntlm_QueryCredentialsAttributesW(
//         PCredHandle phCredential, ULONG ulAttribute, PVOID pBuffer)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("(%p, %d, %p)\n", phCredential, ulAttribute, pBuffer);
//
//     if(ulAttribute == SECPKG_ATTR_NAMES)
//     {
//         FIXME("SECPKG_CRED_ATTR_NAMES: stub\n");
//         ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//         ret = SEC_E_UNSUPPORTED_FUNCTION;
//     
//     return ret;
// }
//
// static char *ntlm_GetUsernameArg(LPCWSTR userW, INT userW_length)
// {
//     static const char username_arg[] = "--username=";
//     char *user;
//     int unixcp_size;
//
//     unixcp_size =  WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS,
//         userW, userW_length, NULL, 0, NULL, NULL) + sizeof(username_arg);
//     user = HeapAlloc(GetProcessHeap(), 0, unixcp_size);
//     if (!user) return NULL;
//     memcpy(user, username_arg, sizeof(username_arg) - 1);
//     WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS, userW, userW_length,
//         user + sizeof(username_arg) - 1,
//         unixcp_size - sizeof(username_arg) + 1, NULL, NULL);
//     user[unixcp_size - 1] = '\0';
//     return user;
// }
//
// static char *ntlm_GetDomainArg(LPCWSTR domainW, INT domainW_length)
// {
//     static const char domain_arg[] = "--domain=";
//     char *domain;
//     int unixcp_size;
//
//     unixcp_size = WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS,
//         domainW, domainW_length, NULL, 0,  NULL, NULL) + sizeof(domain_arg);
//     domain = HeapAlloc(GetProcessHeap(), 0, unixcp_size);
//     if (!domain) return NULL;
//     memcpy(domain, domain_arg, sizeof(domain_arg) - 1);
//     WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS, domainW,
//         domainW_length, domain + sizeof(domain_arg) - 1,
//         unixcp_size - sizeof(domain) + 1, NULL, NULL);
//     domain[unixcp_size - 1] = '\0';
//     return domain;
// }
//
// /***********************************************************************
//  *              AcquireCredentialsHandleW
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_AcquireCredentialsHandleW(
//  SEC_WCHAR *pszPrincipal, SEC_WCHAR *pszPackage, ULONG fCredentialUse,
//  PLUID pLogonID, PVOID pAuthData, SEC_GET_KEY_FN pGetKeyFn,
//  PVOID pGetKeyArgument, PCredHandle phCredential, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//     PNtlmCredentials ntlm_cred;
//     LPWSTR domain = NULL, user = NULL, password = NULL;
//     PSEC_WINNT_AUTH_IDENTITY_W auth_data = NULL;
//
//     TRACE("(%s, %s, 0x%08x, %p, %p, %p, %p, %p, %p)\n",
//      debugstr_w(pszPrincipal), debugstr_w(pszPackage), fCredentialUse,
//      pLogonID, pAuthData, pGetKeyFn, pGetKeyArgument, phCredential, ptsExpiry);
//
//     switch(fCredentialUse)
//     {
//         case SECPKG_CRED_INBOUND:
//             ntlm_cred = HeapAlloc(GetProcessHeap(), 0, sizeof(*ntlm_cred));
//             if (!ntlm_cred)
//                 ret = SEC_E_INSUFFICIENT_MEMORY;
//             else
//             {
//                 ntlm_cred->mode = NTLM_SERVER;
//                 ntlm_cred->username_arg = NULL;
//                 ntlm_cred->domain_arg = NULL;
//                 ntlm_cred->password = NULL;
//                 ntlm_cred->pwlen = 0;
//                 ntlm_cred->no_cached_credentials = 0;
//
//                 phCredential->dwUpper = fCredentialUse;
//                 phCredential->dwLower = (ULONG_PTR)ntlm_cred;
//                 ret = SEC_E_OK;
//             }
//             break;
//         case SECPKG_CRED_OUTBOUND:
//             {
//                 auth_data = pAuthData;
//                 ntlm_cred = HeapAlloc(GetProcessHeap(), 0, sizeof(*ntlm_cred));
//                 if (!ntlm_cred)
//                 {
//                     ret = SEC_E_INSUFFICIENT_MEMORY;
//                     break;
//                 }
//                 ntlm_cred->mode = NTLM_CLIENT;
//                 ntlm_cred->username_arg = NULL;
//                 ntlm_cred->domain_arg = NULL;
//                 ntlm_cred->password = NULL;
//                 ntlm_cred->pwlen = 0;
//                 ntlm_cred->no_cached_credentials = 0;
//
//                 if(pAuthData != NULL)
//                 {
//                     int domain_len = 0, user_len = 0, password_len = 0;
//
//                     if (auth_data->Flags & SEC_WINNT_AUTH_IDENTITY_ANSI)
//                     {
//                         if (auth_data->DomainLength)
//                         {
//                             domain_len = MultiByteToWideChar(CP_ACP, 0, (char *)auth_data->Domain,
//                                                              auth_data->DomainLength, NULL, 0);
//                             domain = HeapAlloc(GetProcessHeap(), 0, sizeof(WCHAR) * domain_len);
//                             if(!domain)
//                             {
//                                 ret = SEC_E_INSUFFICIENT_MEMORY;
//                                 break;
//                             }
//                             MultiByteToWideChar(CP_ACP, 0, (char *)auth_data->Domain, auth_data->DomainLength,
//                                                 domain, domain_len);
//                         }
//
//                         if (auth_data->UserLength)
//                         {
//                             user_len = MultiByteToWideChar(CP_ACP, 0, (char *)auth_data->User,
//                                                            auth_data->UserLength, NULL, 0);
//                             user = HeapAlloc(GetProcessHeap(), 0, sizeof(WCHAR) * user_len);
//                             if(!user)
//                             {
//                                 ret = SEC_E_INSUFFICIENT_MEMORY;
//                                 break;
//                             }
//                             MultiByteToWideChar(CP_ACP, 0, (char *)auth_data->User, auth_data->UserLength,
//                                                 user, user_len);
//                         }
//
//                         if (auth_data->PasswordLength)
//                         {
//                             password_len = MultiByteToWideChar(CP_ACP, 0,(char *)auth_data->Password,
//                                                                auth_data->PasswordLength, NULL, 0);
//                             password = HeapAlloc(GetProcessHeap(), 0, sizeof(WCHAR) * password_len);
//                             if(!password)
//                             {
//                                 ret = SEC_E_INSUFFICIENT_MEMORY;
//                                 break;
//                             }
//                             MultiByteToWideChar(CP_ACP, 0, (char *)auth_data->Password, auth_data->PasswordLength,
//                                                 password, password_len);
//                         }
//                     }
//                     else
//                     {
//                         domain = auth_data->Domain;
//                         domain_len = auth_data->DomainLength;
//
//                         user = auth_data->User;
//                         user_len = auth_data->UserLength;
//
//                         password = auth_data->Password;
//                         password_len = auth_data->PasswordLength;
//                     }
//
//                     TRACE("Username is %s\n", debugstr_wn(user, user_len));
//                     TRACE("Domain name is %s\n", debugstr_wn(domain, domain_len));
//
//                     ntlm_cred->username_arg = ntlm_GetUsernameArg(user, user_len);
//                     ntlm_cred->domain_arg = ntlm_GetDomainArg(domain, domain_len);
//
//                     if(password_len != 0)
//                     {
//                         ntlm_cred->pwlen = WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS, password,
//                                                                password_len, NULL, 0, NULL, NULL);
//
//                         ntlm_cred->password = HeapAlloc(GetProcessHeap(), 0,
//                                                         ntlm_cred->pwlen);
//                         if(!ntlm_cred->password)
//                         {
//                             ret = SEC_E_INSUFFICIENT_MEMORY;
//                             break;
//                         }
//
//                         WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS, password, password_len,
//                                             ntlm_cred->password, ntlm_cred->pwlen, NULL, NULL);
//                     }
//                 }
//
//                 phCredential->dwUpper = fCredentialUse;
//                 phCredential->dwLower = (ULONG_PTR)ntlm_cred;
//                 TRACE("ACH phCredential->dwUpper: 0x%08lx, dwLower: 0x%08lx\n",
//                       phCredential->dwUpper, phCredential->dwLower);
//                 ret = SEC_E_OK;
//                 break;
//             }
//         case SECPKG_CRED_BOTH:
//             FIXME("AcquireCredentialsHandle: SECPKG_CRED_BOTH stub\n");
//             ret = SEC_E_UNSUPPORTED_FUNCTION;
//             phCredential = NULL;
//             break;
//         default:
//             phCredential = NULL;
//             ret = SEC_E_UNKNOWN_CREDENTIALS;
//     }
//
//     if (auth_data && auth_data->Flags & SEC_WINNT_AUTH_IDENTITY_ANSI)
//     {
//         HeapFree(GetProcessHeap(), 0, domain);
//         HeapFree(GetProcessHeap(), 0, user);
//         HeapFree(GetProcessHeap(), 0, password);
//     }
//     return ret;
// }
//
// /***********************************************************************
//  *              AcquireCredentialsHandleA
//  */
// static SECURITY_STATUS SEC_ENTRY ntlm_AcquireCredentialsHandleA(
//  SEC_CHAR *pszPrincipal, SEC_CHAR *pszPackage, ULONG fCredentialUse,
//  PLUID pLogonID, PVOID pAuthData, SEC_GET_KEY_FN pGetKeyFn,
//  PVOID pGetKeyArgument, PCredHandle phCredential, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//     int user_sizeW, domain_sizeW, passwd_sizeW;
//     
//     SEC_WCHAR *user = NULL, *domain = NULL, *passwd = NULL, *package = NULL;
//     
//     PSEC_WINNT_AUTH_IDENTITY_W pAuthDataW = NULL;
//     PSEC_WINNT_AUTH_IDENTITY_A identity  = NULL;
//
//     TRACE("(%s, %s, 0x%08x, %p, %p, %p, %p, %p, %p)\n",
//      debugstr_a(pszPrincipal), debugstr_a(pszPackage), fCredentialUse,
//      pLogonID, pAuthData, pGetKeyFn, pGetKeyArgument, phCredential, ptsExpiry);
//     
//     if(pszPackage != NULL)
//     {
//         int package_sizeW = MultiByteToWideChar(CP_ACP, 0, pszPackage, -1,
//                 NULL, 0);
//
//         package = HeapAlloc(GetProcessHeap(), 0, package_sizeW * 
//                 sizeof(SEC_WCHAR));
//         MultiByteToWideChar(CP_ACP, 0, pszPackage, -1, package, package_sizeW);
//     }
//
//     
//     if(pAuthData != NULL)
//     {
//         identity = pAuthData;
//
//         if(identity->Flags == SEC_WINNT_AUTH_IDENTITY_ANSI)
//         {
//             pAuthDataW = HeapAlloc(GetProcessHeap(), 0, 
//                     sizeof(SEC_WINNT_AUTH_IDENTITY_W));
//
//             if(identity->UserLength != 0)
//             {
//                 user_sizeW = MultiByteToWideChar(CP_ACP, 0, 
//                     (LPCSTR)identity->User, identity->UserLength, NULL, 0);
//                 user = HeapAlloc(GetProcessHeap(), 0, user_sizeW * 
//                         sizeof(SEC_WCHAR));
//                 MultiByteToWideChar(CP_ACP, 0, (LPCSTR)identity->User, 
//                     identity->UserLength, user, user_sizeW);
//             }
//             else
//             {
//                 user_sizeW = 0;
//             }
//              
//             if(identity->DomainLength != 0)
//             {
//                 domain_sizeW = MultiByteToWideChar(CP_ACP, 0, 
//                     (LPCSTR)identity->Domain, identity->DomainLength, NULL, 0);
//                 domain = HeapAlloc(GetProcessHeap(), 0, domain_sizeW 
//                     * sizeof(SEC_WCHAR));
//                 MultiByteToWideChar(CP_ACP, 0, (LPCSTR)identity->Domain, 
//                     identity->DomainLength, domain, domain_sizeW);
//             }
//             else
//             {
//                 domain_sizeW = 0;
//             }
//
//             if(identity->PasswordLength != 0)
//             {
//                 passwd_sizeW = MultiByteToWideChar(CP_ACP, 0, 
//                     (LPCSTR)identity->Password, identity->PasswordLength,
//                     NULL, 0);
//                 passwd = HeapAlloc(GetProcessHeap(), 0, passwd_sizeW
//                     * sizeof(SEC_WCHAR));
//                 MultiByteToWideChar(CP_ACP, 0, (LPCSTR)identity->Password,
//                     identity->PasswordLength, passwd, passwd_sizeW);
//             }
//             else
//             {
//                 passwd_sizeW = 0;
//             }
//             
//             pAuthDataW->Flags = SEC_WINNT_AUTH_IDENTITY_UNICODE;
//             pAuthDataW->User = user;
//             pAuthDataW->UserLength = user_sizeW;
//             pAuthDataW->Domain = domain;
//             pAuthDataW->DomainLength = domain_sizeW;
//             pAuthDataW->Password = passwd;
//             pAuthDataW->PasswordLength = passwd_sizeW;
//         }
//         else
//         {
//             pAuthDataW = (PSEC_WINNT_AUTH_IDENTITY_W)identity;
//         }
//     }       
//     
//     ret = ntlm_AcquireCredentialsHandleW(NULL, package, fCredentialUse, 
//             pLogonID, pAuthDataW, pGetKeyFn, pGetKeyArgument, phCredential,
//             ptsExpiry);
//     
//     HeapFree(GetProcessHeap(), 0, package);
//     HeapFree(GetProcessHeap(), 0, user);
//     HeapFree(GetProcessHeap(), 0, domain);
//     HeapFree(GetProcessHeap(), 0, passwd);
//     if(pAuthDataW != (PSEC_WINNT_AUTH_IDENTITY_W)identity)
//         HeapFree(GetProcessHeap(), 0, pAuthDataW);
//     
//     return ret;
// }
//
// /*************************************************************************
//  *             ntlm_GetTokenBufferIndex
//  * Calculates the index of the secbuffer with BufferType == SECBUFFER_TOKEN
//  * Returns index if found or -1 if not found.
//  */
// static int ntlm_GetTokenBufferIndex(PSecBufferDesc pMessage)
// {
//     UINT i;
//
//     TRACE("%p\n", pMessage);
//
//     for( i = 0; i < pMessage->cBuffers; ++i )
//     {
//         if(pMessage->pBuffers[i].BufferType == SECBUFFER_TOKEN)
//             return i;
//     }
//
//     return -1;
// }
//
// /*************************************************************************
//  *             ntlm_GetDataBufferIndex
//  * Calculates the index of the first secbuffer with BufferType == SECBUFFER_DATA
//  * Returns index if found or -1 if not found.
//  */
// static int ntlm_GetDataBufferIndex(PSecBufferDesc pMessage)
// {
//     UINT i;
//
//     TRACE("%p\n", pMessage);
//
//     for( i = 0; i < pMessage->cBuffers; ++i )
//     {
//         if(pMessage->pBuffers[i].BufferType == SECBUFFER_DATA)
//             return i;
//     }
//
//     return -1;
// }
//
// static BOOL ntlm_GetCachedCredential(const SEC_WCHAR *pszTargetName, PCREDENTIALW *cred)
// {
//     LPCWSTR p;
//     LPCWSTR pszHost;
//     LPWSTR pszHostOnly;
//     BOOL ret;
//
//     if (!pszTargetName)
//         return FALSE;
//
//     /* try to get the start of the hostname from service principal name (SPN) */
//     pszHost = strchrW(pszTargetName, '/');
//     if (pszHost)
//     {
//         /* skip slash character */
//         pszHost++;
//
//         /* find end of host by detecting start of instance port or start of referrer */
//         p = strchrW(pszHost, ':');
//         if (!p)
//             p = strchrW(pszHost, '/');
//         if (!p)
//             p = pszHost + strlenW(pszHost);
//     }
//     else /* otherwise not an SPN, just a host */
//     {
//         pszHost = pszTargetName;
//         p = pszHost + strlenW(pszHost);
//     }
//
//     pszHostOnly = HeapAlloc(GetProcessHeap(), 0, (p - pszHost + 1) * sizeof(WCHAR));
//     if (!pszHostOnly)
//         return FALSE;
//
//     memcpy(pszHostOnly, pszHost, (p - pszHost) * sizeof(WCHAR));
//     pszHostOnly[p - pszHost] = '\0';
//
//     ret = CredReadW(pszHostOnly, CRED_TYPE_DOMAIN_PASSWORD, 0, cred);
//
//     HeapFree(GetProcessHeap(), 0, pszHostOnly);
//     return ret;
// }
//
// /***********************************************************************
//  *              InitializeSecurityContextW
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_InitializeSecurityContextW(
//  PCredHandle phCredential, PCtxtHandle phContext, SEC_WCHAR *pszTargetName, 
//  ULONG fContextReq, ULONG Reserved1, ULONG TargetDataRep, 
//  PSecBufferDesc pInput, ULONG Reserved2, PCtxtHandle phNewContext, 
//  PSecBufferDesc pOutput, ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//     PNtlmCredentials ntlm_cred;
//     PNegoHelper helper = NULL;
//     ULONG ctxt_attr = 0;
//     char* buffer, *want_flags = NULL;
//     PBYTE bin;
//     int buffer_len, bin_len, max_len = NTLM_MAX_BUF;
//     int token_idx;
//     SEC_CHAR *username = NULL;
//     SEC_CHAR *domain = NULL;
//     SEC_CHAR *password = NULL;
//
//     TRACE("%p %p %s 0x%08x %d %d %p %d %p %p %p %p\n", phCredential, phContext,
//      debugstr_w(pszTargetName), fContextReq, Reserved1, TargetDataRep, pInput,
//      Reserved1, phNewContext, pOutput, pfContextAttr, ptsExpiry);
//
//     /****************************************
//      * When communicating with the client, there can be the
//      * following reply packets:
//      * YR <base64 blob>         should be sent to the server
//      * PW                       should be sent back to helper with
//      *                          base64 encoded password
//      * AF <base64 blob>         client is done, blob should be
//      *                          sent to server with KK prefixed
//      * GF <string list>         A string list of negotiated flags
//      * GK <base64 blob>         base64 encoded session key
//      * BH <char reason>         something broke
//      */
//     /* The squid cache size is 2010 chars, and that's what ntlm_auth uses */
//
//     if(TargetDataRep == SECURITY_NETWORK_DREP){
//         TRACE("Setting SECURITY_NETWORK_DREP\n");
//     }
//
//     buffer = HeapAlloc(GetProcessHeap(), 0, sizeof(char) * NTLM_MAX_BUF);
//     bin = HeapAlloc(GetProcessHeap(), 0, sizeof(BYTE) * NTLM_MAX_BUF);
//
//     if((phContext == NULL) && (pInput == NULL))
//     {
//         static char helper_protocol[] = "--helper-protocol=ntlmssp-client-1";
//         static CHAR credentials_argv[] = "--use-cached-creds";
//         SEC_CHAR *client_argv[5];
//         int pwlen = 0;
//
//         TRACE("First time in ISC()\n");
//
//         if(!phCredential)
//         {
//             ret = SEC_E_INVALID_HANDLE;
//             goto isc_end;
//         }
//
//         /* As the server side of sspi never calls this, make sure that
//          * the handler is a client handler.
//          */
//         ntlm_cred = (PNtlmCredentials)phCredential->dwLower;
//         if(ntlm_cred->mode != NTLM_CLIENT)
//         {
//             TRACE("Cred mode = %d\n", ntlm_cred->mode);
//             ret = SEC_E_INVALID_HANDLE;
//             goto isc_end;
//         }
//
//         client_argv[0] = ntlm_auth;
//         client_argv[1] = helper_protocol;
//         if (!ntlm_cred->username_arg && !ntlm_cred->domain_arg)
//         {
//             LPWKSTA_USER_INFO_1 ui = NULL;
//             NET_API_STATUS status;
//             PCREDENTIALW cred;
//
//             if (ntlm_GetCachedCredential(pszTargetName, &cred))
//             {
//                 LPWSTR p;
//                 p = strchrW(cred->UserName, '\\');
//                 if (p)
//                 {
//                     domain = ntlm_GetDomainArg(cred->UserName, p - cred->UserName);
//                     p++;
//                 }
//                 else
//                 {
//                     domain = ntlm_GetDomainArg(NULL, 0);
//                     p = cred->UserName;
//                 }
//
//                 username = ntlm_GetUsernameArg(p, -1);
//
//                 if(cred->CredentialBlobSize != 0)
//                 {
//                     pwlen = WideCharToMultiByte(CP_UNIXCP,
//                         WC_NO_BEST_FIT_CHARS, (LPWSTR)cred->CredentialBlob,
//                         cred->CredentialBlobSize / sizeof(WCHAR), NULL, 0,
//                         NULL, NULL);
//
//                     password = HeapAlloc(GetProcessHeap(), 0, pwlen);
//
//                     WideCharToMultiByte(CP_UNIXCP, WC_NO_BEST_FIT_CHARS,
//                                         (LPWSTR)cred->CredentialBlob,
//                                         cred->CredentialBlobSize / sizeof(WCHAR),
//                                         password, pwlen, NULL, NULL);
//                 }
//
//                 CredFree(cred);
//
//                 client_argv[2] = username;
//                 client_argv[3] = domain;
//                 client_argv[4] = NULL;
//             }
//             else
//             {
//                 status = NetWkstaUserGetInfo(NULL, 1, (LPBYTE *)&ui);
//                 if (status != NERR_Success || ui == NULL || ntlm_cred->no_cached_credentials)
//                 {
//                     ret = SEC_E_NO_CREDENTIALS;
//                     goto isc_end;
//                 }
//                 username = ntlm_GetUsernameArg(ui->wkui1_username, -1);
//                 NetApiBufferFree(ui);
//
//                 TRACE("using cached credentials\n");
//
//                 client_argv[2] = username;
//                 client_argv[3] = credentials_argv;
//                 client_argv[4] = NULL;
//             }
//         }
//         else
//         {
//             client_argv[2] = ntlm_cred->username_arg;
//             client_argv[3] = ntlm_cred->domain_arg;
//             client_argv[4] = NULL;
//         }
//
//         if((ret = fork_helper(&helper, ntlm_auth, client_argv)) != SEC_E_OK)
//             goto isc_end;
//
//         helper->mode = NTLM_CLIENT;
//         helper->session_key = HeapAlloc(GetProcessHeap(), 0, 16);
//         if (!helper->session_key)
//         {
//             cleanup_helper(helper);
//             ret = SEC_E_INSUFFICIENT_MEMORY;
//             goto isc_end;
//         }
//
//         /* Generate the dummy session key = MD4(MD4(password))*/
//         if(password || ntlm_cred->password)
//         {
//             SEC_WCHAR *unicode_password;
//             int passwd_lenW;
//
//             TRACE("Converting password to unicode.\n");
//             passwd_lenW = MultiByteToWideChar(CP_ACP, 0,
//                                               password ? password : ntlm_cred->password,
//                                               password ? pwlen : ntlm_cred->pwlen,
//                                               NULL, 0);
//             unicode_password = HeapAlloc(GetProcessHeap(), 0,
//                                          passwd_lenW * sizeof(SEC_WCHAR));
//             MultiByteToWideChar(CP_ACP, 0, password ? password : ntlm_cred->password,
//                                 password ? pwlen : ntlm_cred->pwlen, unicode_password, passwd_lenW);
//
//             SECUR32_CreateNTLM1SessionKey((PBYTE)unicode_password,
//                                           passwd_lenW * sizeof(SEC_WCHAR), helper->session_key);
//
//             HeapFree(GetProcessHeap(), 0, unicode_password);
//         }
//         else
//             memset(helper->session_key, 0, 16);
//
//         /* Allocate space for a maximal string of
//          * "SF NTLMSSP_FEATURE_SIGN NTLMSSP_FEATURE_SEAL
//          * NTLMSSP_FEATURE_SESSION_KEY"
//          */
//         want_flags = HeapAlloc(GetProcessHeap(), 0, 73);
//         if(want_flags == NULL)
//         {
//             cleanup_helper(helper);
//             ret = SEC_E_INSUFFICIENT_MEMORY;
//             goto isc_end;
//         }
//         lstrcpyA(want_flags, "SF");
//         if(fContextReq & ISC_REQ_CONFIDENTIALITY)
//         {
//             if(strstr(want_flags, "NTLMSSP_FEATURE_SEAL") == NULL)
//                 lstrcatA(want_flags, " NTLMSSP_FEATURE_SEAL");
//         }
//         if(fContextReq & ISC_REQ_CONNECTION)
//             ctxt_attr |= ISC_RET_CONNECTION;
//         if(fContextReq & ISC_REQ_EXTENDED_ERROR)
//             ctxt_attr |= ISC_RET_EXTENDED_ERROR;
//         if(fContextReq & ISC_REQ_INTEGRITY)
//         {
//             if(strstr(want_flags, "NTLMSSP_FEATURE_SIGN") == NULL)
//                 lstrcatA(want_flags, " NTLMSSP_FEATURE_SIGN");
//         }
//         if(fContextReq & ISC_REQ_MUTUAL_AUTH)
//             ctxt_attr |= ISC_RET_MUTUAL_AUTH;
//         if(fContextReq & ISC_REQ_REPLAY_DETECT)
//         {
//             if(strstr(want_flags, "NTLMSSP_FEATURE_SIGN") == NULL)
//                 lstrcatA(want_flags, " NTLMSSP_FEATURE_SIGN");
//         }
//         if(fContextReq & ISC_REQ_SEQUENCE_DETECT)
//         {
//             if(strstr(want_flags, "NTLMSSP_FEATURE_SIGN") == NULL)
//                 lstrcatA(want_flags, " NTLMSSP_FEATURE_SIGN");
//         }
//         if(fContextReq & ISC_REQ_STREAM)
//             FIXME("ISC_REQ_STREAM\n");
//         if(fContextReq & ISC_REQ_USE_DCE_STYLE)
//             ctxt_attr |= ISC_RET_USED_DCE_STYLE;
//         if(fContextReq & ISC_REQ_DELEGATE)
//             ctxt_attr |= ISC_RET_DELEGATE;
//
//         /* If no password is given, try to use cached credentials. Fall back to an empty
//          * password if this failed. */
//         if(!password && !ntlm_cred->password)
//         {
//             lstrcpynA(buffer, "OK", max_len-1);
//             if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) != SEC_E_OK)
//             {
//                 cleanup_helper(helper);
//                 goto isc_end;
//             }
//             /* If the helper replied with "PW", using cached credentials failed */
//             if(!strncmp(buffer, "PW", 2))
//             {
//                 TRACE("Using cached credentials failed.\n");
//                 lstrcpynA(buffer, "PW AA==", max_len-1);
//             }
//             else /* Just do a noop on the next run */
//                 lstrcpynA(buffer, "OK", max_len-1);
//         }
//         else
//         {
//             lstrcpynA(buffer, "PW ", max_len-1);
//             if((ret = encodeBase64(password ? (unsigned char *)password : (unsigned char *)ntlm_cred->password,
//                         password ? pwlen : ntlm_cred->pwlen, buffer+3,
//                         max_len-3, &buffer_len)) != SEC_E_OK)
//             {
//                 cleanup_helper(helper);
//                 goto isc_end;
//             }
//
//         }
//
//         TRACE("Sending to helper: %s\n", debugstr_a(buffer));
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) != SEC_E_OK)
//         {
//             cleanup_helper(helper);
//             goto isc_end;
//         }
//
//         TRACE("Helper returned %s\n", debugstr_a(buffer));
//
//         if(lstrlenA(want_flags) > 2)
//         {
//             TRACE("Want flags are %s\n", debugstr_a(want_flags));
//             lstrcpynA(buffer, want_flags, max_len-1);
//             if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) 
//                     != SEC_E_OK)
//             {
//                 cleanup_helper(helper);
//                 goto isc_end;
//             }
//             if(!strncmp(buffer, "BH", 2))
//                 ERR("Helper doesn't understand new command set. Expect more things to fail.\n");
//         }
//
//         lstrcpynA(buffer, "YR", max_len-1);
//
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) != SEC_E_OK)
//         {
//             cleanup_helper(helper);
//             goto isc_end;
//         }
//
//         TRACE("%s\n", buffer);
//
//         if(strncmp(buffer, "YR ", 3) != 0)
//         {
//             /* Something borked */
//             TRACE("Helper returned %c%c\n", buffer[0], buffer[1]);
//             ret = SEC_E_INTERNAL_ERROR;
//             cleanup_helper(helper);
//             goto isc_end;
//         }
//         if((ret = decodeBase64(buffer+3, buffer_len-3, bin,
//                         max_len-1, &bin_len)) != SEC_E_OK)
//         {
//             cleanup_helper(helper);
//             goto isc_end;
//         }
//
//         /* put the decoded client blob into the out buffer */
//
//         phNewContext->dwUpper = ctxt_attr;
//         phNewContext->dwLower = (ULONG_PTR)helper;
//
//         ret = SEC_I_CONTINUE_NEEDED;
//     }
//     else
//     {
//         int input_token_idx;
//
//         /* handle second call here */
//         /* encode server data to base64 */
//         if (!pInput || ((input_token_idx = ntlm_GetTokenBufferIndex(pInput)) == -1))
//         {
//             ret = SEC_E_INVALID_TOKEN;
//             goto isc_end;
//         }
//
//         if(!phContext)
//         {
//             ret = SEC_E_INVALID_HANDLE;
//             goto isc_end;
//         }
//
//         /* As the server side of sspi never calls this, make sure that
//          * the handler is a client handler.
//          */
//         helper = (PNegoHelper)phContext->dwLower;
//         if(helper->mode != NTLM_CLIENT)
//         {
//             TRACE("Helper mode = %d\n", helper->mode);
//             ret = SEC_E_INVALID_HANDLE;
//             goto isc_end;
//         }
//
//         if (!pInput->pBuffers[input_token_idx].pvBuffer)
//         {
//             ret = SEC_E_INTERNAL_ERROR;
//             goto isc_end;
//         }
//
//         if(pInput->pBuffers[input_token_idx].cbBuffer > max_len)
//         {
//             TRACE("pInput->pBuffers[%d].cbBuffer is: %d\n",
//                     input_token_idx,
//                     pInput->pBuffers[input_token_idx].cbBuffer);
//             ret = SEC_E_INVALID_TOKEN;
//             goto isc_end;
//         }
//         else
//             bin_len = pInput->pBuffers[input_token_idx].cbBuffer;
//
//         memcpy(bin, pInput->pBuffers[input_token_idx].pvBuffer, bin_len);
//
//         lstrcpynA(buffer, "TT ", max_len-1);
//
//         if((ret = encodeBase64(bin, bin_len, buffer+3,
//                         max_len-3, &buffer_len)) != SEC_E_OK)
//             goto isc_end;
//
//         TRACE("Server sent: %s\n", debugstr_a(buffer));
//
//         /* send TT base64 blob to ntlm_auth */
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) != SEC_E_OK)
//             goto isc_end;
//
//         TRACE("Helper replied: %s\n", debugstr_a(buffer));
//
//         if( (strncmp(buffer, "KK ", 3) != 0) &&
//                 (strncmp(buffer, "AF ", 3) !=0))
//         {
//             TRACE("Helper returned %c%c\n", buffer[0], buffer[1]);
//             ret = SEC_E_INVALID_TOKEN;
//             goto isc_end;
//         }
//
//         /* decode the blob and send it to server */
//         if((ret = decodeBase64(buffer+3, buffer_len-3, bin, max_len,
//                         &bin_len)) != SEC_E_OK)
//         {
//             goto isc_end;
//         }
//
//         phNewContext->dwUpper = ctxt_attr;
//         phNewContext->dwLower = (ULONG_PTR)helper;
//
//         ret = SEC_E_OK;
//     }
//
//     /* put the decoded client blob into the out buffer */
//
//     if (!pOutput || ((token_idx = ntlm_GetTokenBufferIndex(pOutput)) == -1))
//     {
//         TRACE("no SECBUFFER_TOKEN buffer could be found\n");
//         ret = SEC_E_BUFFER_TOO_SMALL;
//         if ((phContext == NULL) && (pInput == NULL))
//         {
//             cleanup_helper(helper);
//             phNewContext->dwUpper = 0;
//             phNewContext->dwLower = 0;
//         }
//         goto isc_end;
//     }
//
//     if (fContextReq & ISC_REQ_ALLOCATE_MEMORY)
//     {
//         pOutput->pBuffers[token_idx].pvBuffer = HeapAlloc(GetProcessHeap(), 0, bin_len);
//         pOutput->pBuffers[token_idx].cbBuffer = bin_len;
//     }
//     else if (pOutput->pBuffers[token_idx].cbBuffer < bin_len)
//     {
//         TRACE("out buffer is NULL or has not enough space\n");
//         ret = SEC_E_BUFFER_TOO_SMALL;
//         if ((phContext == NULL) && (pInput == NULL))
//         {
//             cleanup_helper(helper);
//             phNewContext->dwUpper = 0;
//             phNewContext->dwLower = 0;
//         }
//         goto isc_end;
//     }
//
//     if (!pOutput->pBuffers[token_idx].pvBuffer)
//     {
//         TRACE("out buffer is NULL\n");
//         ret = SEC_E_INTERNAL_ERROR;
//         if ((phContext == NULL) && (pInput == NULL))
//         {
//             cleanup_helper(helper);
//             phNewContext->dwUpper = 0;
//             phNewContext->dwLower = 0;
//         }
//         goto isc_end;
//     }
//
//     pOutput->pBuffers[token_idx].cbBuffer = bin_len;
//     memcpy(pOutput->pBuffers[token_idx].pvBuffer, bin, bin_len);
//
//     if(ret == SEC_E_OK)
//     {
//         TRACE("Getting negotiated flags\n");
//         lstrcpynA(buffer, "GF", max_len - 1);
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) != SEC_E_OK)
//             goto isc_end;
//
//         if(buffer_len < 3)
//         {
//             TRACE("No flags negotiated.\n");
//             helper->neg_flags = 0l;
//         }
//         else
//         {
//             TRACE("Negotiated %s\n", debugstr_a(buffer));
//             sscanf(buffer + 3, "%lx", &(helper->neg_flags));
//             TRACE("Stored 0x%08x as flags\n", helper->neg_flags);
//         }
//
//         TRACE("Getting session key\n");
//         lstrcpynA(buffer, "GK", max_len - 1);
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) != SEC_E_OK)
//             goto isc_end;
//
//         if(strncmp(buffer, "BH", 2) == 0)
//             TRACE("No key negotiated.\n");
//         else if(strncmp(buffer, "GK ", 3) == 0)
//         {
//             if((ret = decodeBase64(buffer+3, buffer_len-3, bin, max_len, 
//                             &bin_len)) != SEC_E_OK)
//             {
//                 TRACE("Failed to decode session key\n");
//             }
//             TRACE("Session key is %s\n", debugstr_a(buffer+3));
//             HeapFree(GetProcessHeap(), 0, helper->session_key);
//             helper->session_key = HeapAlloc(GetProcessHeap(), 0, bin_len);
//             if(!helper->session_key)
//             {
//                 ret = SEC_E_INSUFFICIENT_MEMORY;
//                 goto isc_end;
//             }
//             memcpy(helper->session_key, bin, bin_len);
//         }
//
//         helper->crypt.ntlm.a4i = SECUR32_arc4Alloc();
//         SECUR32_arc4Init(helper->crypt.ntlm.a4i, helper->session_key, 16);
//         helper->crypt.ntlm.seq_num = 0l;
//         SECUR32_CreateNTLM2SubKeys(helper);
//         helper->crypt.ntlm2.send_a4i = SECUR32_arc4Alloc();
//         helper->crypt.ntlm2.recv_a4i = SECUR32_arc4Alloc();
//         SECUR32_arc4Init(helper->crypt.ntlm2.send_a4i,
//                          helper->crypt.ntlm2.send_seal_key, 16);
//         SECUR32_arc4Init(helper->crypt.ntlm2.recv_a4i,
//                          helper->crypt.ntlm2.recv_seal_key, 16);
//         helper->crypt.ntlm2.send_seq_no = 0l;
//         helper->crypt.ntlm2.recv_seq_no = 0l;
//     }
//
// isc_end:
//     HeapFree(GetProcessHeap(), 0, username);
//     HeapFree(GetProcessHeap(), 0, domain);
//     HeapFree(GetProcessHeap(), 0, password);
//     HeapFree(GetProcessHeap(), 0, want_flags);
//     HeapFree(GetProcessHeap(), 0, buffer);
//     HeapFree(GetProcessHeap(), 0, bin);
//     return ret;
// }
//
// /***********************************************************************
//  *              InitializeSecurityContextA
//  */
// static SECURITY_STATUS SEC_ENTRY ntlm_InitializeSecurityContextA(
//  PCredHandle phCredential, PCtxtHandle phContext, SEC_CHAR *pszTargetName,
//  ULONG fContextReq, ULONG Reserved1, ULONG TargetDataRep, 
//  PSecBufferDesc pInput,ULONG Reserved2, PCtxtHandle phNewContext, 
//  PSecBufferDesc pOutput, ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//     SEC_WCHAR *target = NULL;
//
//     TRACE("%p %p %s %d %d %d %p %d %p %p %p %p\n", phCredential, phContext,
//      debugstr_a(pszTargetName), fContextReq, Reserved1, TargetDataRep, pInput,
//      Reserved1, phNewContext, pOutput, pfContextAttr, ptsExpiry);
//
//     if(pszTargetName != NULL)
//     {
//         int target_size = MultiByteToWideChar(CP_ACP, 0, pszTargetName,
//             strlen(pszTargetName)+1, NULL, 0);
//         target = HeapAlloc(GetProcessHeap(), 0, target_size *
//                 sizeof(SEC_WCHAR));
//         MultiByteToWideChar(CP_ACP, 0, pszTargetName, strlen(pszTargetName)+1,
//             target, target_size);
//     }
//
//     ret = ntlm_InitializeSecurityContextW(phCredential, phContext, target,
//             fContextReq, Reserved1, TargetDataRep, pInput, Reserved2,
//             phNewContext, pOutput, pfContextAttr, ptsExpiry);
//
//     HeapFree(GetProcessHeap(), 0, target);
//     return ret;
// }
//
// /***********************************************************************
//  *              AcceptSecurityContext
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_AcceptSecurityContext(
//  PCredHandle phCredential, PCtxtHandle phContext, PSecBufferDesc pInput,
//  ULONG fContextReq, ULONG TargetDataRep, PCtxtHandle phNewContext, 
//  PSecBufferDesc pOutput, ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//     char *buffer, *want_flags = NULL;
//     PBYTE bin;
//     int buffer_len, bin_len, max_len = NTLM_MAX_BUF;
//     ULONG ctxt_attr = 0;
//     PNegoHelper helper;
//     PNtlmCredentials ntlm_cred;
//
//     TRACE("%p %p %p %d %d %p %p %p %p\n", phCredential, phContext, pInput,
//      fContextReq, TargetDataRep, phNewContext, pOutput, pfContextAttr,
//      ptsExpiry);
//
//     buffer = HeapAlloc(GetProcessHeap(), 0, sizeof(char) * NTLM_MAX_BUF);
//     bin    = HeapAlloc(GetProcessHeap(),0, sizeof(BYTE) * NTLM_MAX_BUF);
//
//     if(TargetDataRep == SECURITY_NETWORK_DREP){
//         TRACE("Using SECURITY_NETWORK_DREP\n");
//     }
//
//     if(phContext == NULL)
//     {
//         static CHAR server_helper_protocol[] = "--helper-protocol=squid-2.5-ntlmssp";
//         SEC_CHAR *server_argv[] = { ntlm_auth,
//             server_helper_protocol,
//             NULL };
//
//         if (!phCredential)
//         {
//             ret = SEC_E_INVALID_HANDLE;
//             goto asc_end;
//         }
//
//         ntlm_cred = (PNtlmCredentials)phCredential->dwLower;
//
//         if(ntlm_cred->mode != NTLM_SERVER)
//         {
//             ret = SEC_E_INVALID_HANDLE;
//             goto asc_end;
//         }
//
//         /* This is the first call to AcceptSecurityHandle */
//         if(pInput == NULL)
//         {
//             ret = SEC_E_INCOMPLETE_MESSAGE;
//             goto asc_end;
//         }
//
//         if(pInput->cBuffers < 1)
//         {
//             ret = SEC_E_INCOMPLETE_MESSAGE;
//             goto asc_end;
//         }
//
//         if(pInput->pBuffers[0].cbBuffer > max_len)
//         {
//             ret = SEC_E_INVALID_TOKEN;
//             goto asc_end;
//         }
//         else
//             bin_len = pInput->pBuffers[0].cbBuffer;
//
//         if( (ret = fork_helper(&helper, ntlm_auth, server_argv)) !=
//             SEC_E_OK)
//         {
//             ret = SEC_E_INTERNAL_ERROR;
//             goto asc_end;
//         }
//         helper->mode = NTLM_SERVER;
//
//         /* Handle all the flags */
//         want_flags = HeapAlloc(GetProcessHeap(), 0, 73);
//         if(want_flags == NULL)
//         {
//             TRACE("Failed to allocate memory for the want_flags!\n");
//             ret = SEC_E_INSUFFICIENT_MEMORY;
//             cleanup_helper(helper);
//             goto asc_end;
//         }
//         lstrcpyA(want_flags, "SF");
//         if(fContextReq & ASC_REQ_ALLOCATE_MEMORY)
//         {
//             FIXME("ASC_REQ_ALLOCATE_MEMORY stub\n");
//         }
//         if(fContextReq & ASC_REQ_CONFIDENTIALITY)
//         {
//             lstrcatA(want_flags, " NTLMSSP_FEATURE_SEAL");
//         }
//         if(fContextReq & ASC_REQ_CONNECTION)
//         {
//             /* This is default, so we'll enable it */
//             lstrcatA(want_flags, " NTLMSSP_FEATURE_SESSION_KEY");
//             ctxt_attr |= ASC_RET_CONNECTION;
//         }
//         if(fContextReq & ASC_REQ_EXTENDED_ERROR)
//         {
//             FIXME("ASC_REQ_EXTENDED_ERROR stub\n");
//         }
//         if(fContextReq & ASC_REQ_INTEGRITY)
//         {
//             lstrcatA(want_flags, " NTLMSSP_FEATURE_SIGN");
//         }
//         if(fContextReq & ASC_REQ_MUTUAL_AUTH)
//         {
//             FIXME("ASC_REQ_MUTUAL_AUTH stub\n");
//         }
//         if(fContextReq & ASC_REQ_REPLAY_DETECT)
//         {
//             FIXME("ASC_REQ_REPLAY_DETECT stub\n");
//         }
//         if(fContextReq & ISC_REQ_SEQUENCE_DETECT)
//         {
//             FIXME("ASC_REQ_SEQUENCE_DETECT stub\n");
//         }
//         if(fContextReq & ISC_REQ_STREAM)
//         {
//             FIXME("ASC_REQ_STREAM stub\n");
//         }
//         /* Done with the flags */
//
//         if(lstrlenA(want_flags) > 3)
//         {
//             TRACE("Server set want_flags: %s\n", debugstr_a(want_flags));
//             lstrcpynA(buffer, want_flags, max_len - 1);
//             if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) !=
//                     SEC_E_OK)
//             {
//                 cleanup_helper(helper);
//                 goto asc_end;
//             }
//             if(!strncmp(buffer, "BH", 2))
//                 TRACE("Helper doesn't understand new command set\n");
//         }
//
//         /* This is the YR request from the client, encode to base64 */
//
//         memcpy(bin, pInput->pBuffers[0].pvBuffer, bin_len);
//
//         lstrcpynA(buffer, "YR ", max_len-1);
//
//         if((ret = encodeBase64(bin, bin_len, buffer+3, max_len-3,
//                     &buffer_len)) != SEC_E_OK)
//         {
//             cleanup_helper(helper);
//             goto asc_end;
//         }
//
//         TRACE("Client sent: %s\n", debugstr_a(buffer));
//
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) !=
//                     SEC_E_OK)
//         {
//             cleanup_helper(helper);
//             goto asc_end;
//         }
//
//         TRACE("Reply from ntlm_auth: %s\n", debugstr_a(buffer));
//         /* The expected answer is TT <base64 blob> */
//
//         if(strncmp(buffer, "TT ", 3) != 0)
//         {
//             ret = SEC_E_INTERNAL_ERROR;
//             cleanup_helper(helper);
//             goto asc_end;
//         }
//
//         if((ret = decodeBase64(buffer+3, buffer_len-3, bin, max_len,
//                         &bin_len)) != SEC_E_OK)
//         {
//             cleanup_helper(helper);
//             goto asc_end;
//         }
//
//         /* send this to the client */
//         if(pOutput == NULL)
//         {
//             ret = SEC_E_INSUFFICIENT_MEMORY;
//             cleanup_helper(helper);
//             goto asc_end;
//         }
//
//         if(pOutput->cBuffers < 1)
//         {
//             ret = SEC_E_INSUFFICIENT_MEMORY;
//             cleanup_helper(helper);
//             goto asc_end;
//         }
//
//         pOutput->pBuffers[0].cbBuffer = bin_len;
//         pOutput->pBuffers[0].BufferType = SECBUFFER_DATA;
//         memcpy(pOutput->pBuffers[0].pvBuffer, bin, bin_len);
//         ret = SEC_I_CONTINUE_NEEDED;
//
//     }
//     else
//     {
//         /* we expect a KK request from client */
//         if(pInput == NULL)
//         {
//             ret = SEC_E_INCOMPLETE_MESSAGE;
//             goto asc_end;
//         }
//
//         if(pInput->cBuffers < 1)
//         {
//             ret = SEC_E_INCOMPLETE_MESSAGE;
//             goto asc_end;
//         }
//
//         helper = (PNegoHelper)phContext->dwLower;
//
//         if(helper->mode != NTLM_SERVER)
//         {
//             ret = SEC_E_INVALID_HANDLE;
//             goto asc_end;
//         }
//
//         if(pInput->pBuffers[0].cbBuffer > max_len)
//         {
//             ret = SEC_E_INVALID_TOKEN;
//             goto asc_end;
//         }
//         else
//             bin_len = pInput->pBuffers[0].cbBuffer;
//
//         memcpy(bin, pInput->pBuffers[0].pvBuffer, bin_len);
//
//         lstrcpynA(buffer, "KK ", max_len-1);
//
//         if((ret = encodeBase64(bin, bin_len, buffer+3, max_len-3,
//                     &buffer_len)) != SEC_E_OK)
//         {
//             goto asc_end;
//         }
//
//         TRACE("Client sent: %s\n", debugstr_a(buffer));
//
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) !=
//                     SEC_E_OK)
//         {
//             goto asc_end;
//         }
//
//         TRACE("Reply from ntlm_auth: %s\n", debugstr_a(buffer));
//
//         /* At this point, we get a NA if the user didn't authenticate, but a BH
//          * if ntlm_auth could not connect to winbindd. Apart from running Wine
//          * as root, there is no way to fix this for now, so just handle this as
//          * a failed login. */
//         if(strncmp(buffer, "AF ", 3) != 0)
//         {
//             if(strncmp(buffer, "NA ", 3) == 0)
//             {
//                 ret = SEC_E_LOGON_DENIED;
//                 goto asc_end;
//             }
//             else
//             {
//                 size_t ntlm_pipe_err_v3_len = strlen("BH NT_STATUS_ACCESS_DENIED");
//                 size_t ntlm_pipe_err_v4_len = strlen("BH NT_STATUS_UNSUCCESSFUL");
//
//                 if( (buffer_len >= ntlm_pipe_err_v3_len &&
//                      strncmp(buffer, "BH NT_STATUS_ACCESS_DENIED", ntlm_pipe_err_v3_len) == 0) ||
//                     (buffer_len >= ntlm_pipe_err_v4_len &&
//                      strncmp(buffer, "BH NT_STATUS_UNSUCCESSFUL", ntlm_pipe_err_v4_len) == 0) )
//                 {
//                     TRACE("Connection to winbindd failed\n");
//                     ret = SEC_E_LOGON_DENIED;
//                 }
//                 else
//                     ret = SEC_E_INTERNAL_ERROR;
//
//                 goto asc_end;
//             }
//         }
//         pOutput->pBuffers[0].cbBuffer = 0;
//
//         TRACE("Getting negotiated flags\n");
//         lstrcpynA(buffer, "GF", max_len - 1);
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) != SEC_E_OK)
//             goto asc_end;
//
//         if(buffer_len < 3)
//         {
//             TRACE("No flags negotiated, or helper does not support GF command\n");
//         }
//         else
//         {
//             TRACE("Negotiated %s\n", debugstr_a(buffer));
//             sscanf(buffer + 3, "%lx", &(helper->neg_flags));
//             TRACE("Stored 0x%08x as flags\n", helper->neg_flags);
//         }
//
//         TRACE("Getting session key\n");
//         lstrcpynA(buffer, "GK", max_len - 1);
//         if((ret = run_helper(helper, buffer, max_len as _, &buffer_len)) != SEC_E_OK)
//             goto asc_end;
//
//         if(buffer_len < 3)
//             TRACE("Helper does not support GK command\n");
//         else
//         {
//             if(strncmp(buffer, "BH ", 3) == 0)
//             {
//                 TRACE("Helper sent %s\n", debugstr_a(buffer+3));
//                 HeapFree(GetProcessHeap(), 0, helper->session_key);
//                 helper->session_key = HeapAlloc(GetProcessHeap(), 0, 16);
//                 if (!helper->session_key)
//                 {
//                     ret = SEC_E_INSUFFICIENT_MEMORY;
//                     goto asc_end;
//                 }
//                 /*FIXME: Generate the dummy session key = MD4(MD4(password))*/
//                 memset(helper->session_key, 0 , 16);
//             }
//             else if(strncmp(buffer, "GK ", 3) == 0)
//             {
//                 if((ret = decodeBase64(buffer+3, buffer_len-3, bin, max_len, 
//                                 &bin_len)) != SEC_E_OK)
//                 {
//                     TRACE("Failed to decode session key\n");
//                 }
//                 TRACE("Session key is %s\n", debugstr_a(buffer+3));
//                 HeapFree(GetProcessHeap(), 0, helper->session_key);
//                 helper->session_key = HeapAlloc(GetProcessHeap(), 0, 16);
//                 if(!helper->session_key)
//                 {
//                     ret = SEC_E_INSUFFICIENT_MEMORY;
//                     goto asc_end;
//                 }
//                 memcpy(helper->session_key, bin, 16);
//             }
//         }
//         helper->crypt.ntlm.a4i = SECUR32_arc4Alloc();
//         SECUR32_arc4Init(helper->crypt.ntlm.a4i, helper->session_key, 16);
//         helper->crypt.ntlm.seq_num = 0l;
//     }
//
//     phNewContext->dwUpper = ctxt_attr;
//     phNewContext->dwLower = (ULONG_PTR)helper;
//
// asc_end:
//     HeapFree(GetProcessHeap(), 0, want_flags);
//     HeapFree(GetProcessHeap(), 0, buffer);
//     HeapFree(GetProcessHeap(), 0, bin);
//     return ret;
// }
//
// /***********************************************************************
//  *              CompleteAuthToken
//  */
// static SECURITY_STATUS SEC_ENTRY ntlm_CompleteAuthToken(PCtxtHandle phContext,
//  PSecBufferDesc pToken)
// {
//     /* We never need to call CompleteAuthToken anyway */
//     TRACE("%p %p\n", phContext, pToken);
//     if (!phContext)
//         return SEC_E_INVALID_HANDLE;
//     
//     return SEC_E_OK;
// }
//
// /***********************************************************************
//  *              DeleteSecurityContext
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_DeleteSecurityContext(PCtxtHandle phContext)
// {
//     PNegoHelper helper;
//
//     TRACE("%p\n", phContext);
//     if (!phContext)
//         return SEC_E_INVALID_HANDLE;
//
//     helper = (PNegoHelper)phContext->dwLower;
//
//     phContext->dwUpper = 0;
//     phContext->dwLower = 0;
//
//     SECUR32_arc4Cleanup(helper->crypt.ntlm.a4i);
//     SECUR32_arc4Cleanup(helper->crypt.ntlm2.send_a4i);
//     SECUR32_arc4Cleanup(helper->crypt.ntlm2.recv_a4i);
//     HeapFree(GetProcessHeap(), 0, helper->crypt.ntlm2.send_sign_key);
//     HeapFree(GetProcessHeap(), 0, helper->crypt.ntlm2.send_seal_key);
//     HeapFree(GetProcessHeap(), 0, helper->crypt.ntlm2.recv_sign_key);
//     HeapFree(GetProcessHeap(), 0, helper->crypt.ntlm2.recv_seal_key);
//
//     cleanup_helper(helper);
//
//     return SEC_E_OK;
// }
//
// /***********************************************************************
//  *              QueryContextAttributesW
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_QueryContextAttributesW(PCtxtHandle phContext,
//  ULONG ulAttribute, void *pBuffer)
// {
//     TRACE("%p %d %p\n", phContext, ulAttribute, pBuffer);
//     if (!phContext)
//         return SEC_E_INVALID_HANDLE;
//
//     switch(ulAttribute)
//     {
// #define _x(x) case (x) : FIXME(#x" stub\n"); break
//         _x(SECPKG_ATTR_ACCESS_TOKEN);
//         _x(SECPKG_ATTR_AUTHORITY);
//         _x(SECPKG_ATTR_DCE_INFO);
//         case SECPKG_ATTR_FLAGS:
//         {
//             PSecPkgContext_Flags spcf = (PSecPkgContext_Flags)pBuffer;
//             PNegoHelper helper = (PNegoHelper)phContext->dwLower;
//
//             spcf->Flags = 0;
//             if(helper->neg_flags & NTLMSSP_NEGOTIATE_SIGN)
//                 spcf->Flags |= ISC_RET_INTEGRITY;
//             if(helper->neg_flags & NTLMSSP_NEGOTIATE_SEAL)
//                 spcf->Flags |= ISC_RET_CONFIDENTIALITY;
//             return SEC_E_OK;
//         }
//         _x(SECPKG_ATTR_KEY_INFO);
//         _x(SECPKG_ATTR_LIFESPAN);
//         _x(SECPKG_ATTR_NAMES);
//         _x(SECPKG_ATTR_NATIVE_NAMES);
//         _x(SECPKG_ATTR_NEGOTIATION_INFO);
//         _x(SECPKG_ATTR_PACKAGE_INFO);
//         _x(SECPKG_ATTR_PASSWORD_EXPIRY);
//         _x(SECPKG_ATTR_SESSION_KEY);
//         case SECPKG_ATTR_SIZES:
//         {
//             PSecPkgContext_Sizes spcs = (PSecPkgContext_Sizes)pBuffer;
//             spcs->cbMaxToken = NTLM_MAX_BUF;
//             spcs->cbMaxSignature = 16;
//             spcs->cbBlockSize = 0;
//             spcs->cbSecurityTrailer = 16;
//             return SEC_E_OK;
//         }
//         _x(SECPKG_ATTR_STREAM_SIZES);
//         _x(SECPKG_ATTR_TARGET_INFORMATION);
// #undef _x
//         default:
//             TRACE("Unknown value %d passed for ulAttribute\n", ulAttribute);
//     }
//
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              QueryContextAttributesA
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_QueryContextAttributesA(PCtxtHandle phContext,
//  ULONG ulAttribute, void *pBuffer)
// {
//     return ntlm_QueryContextAttributesW(phContext, ulAttribute, pBuffer);
// }
//
// /***********************************************************************
//  *              ImpersonateSecurityContext
//  */
// static SECURITY_STATUS SEC_ENTRY ntlm_ImpersonateSecurityContext(PCtxtHandle phContext)
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
// static SECURITY_STATUS SEC_ENTRY ntlm_RevertSecurityContext(PCtxtHandle phContext)
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
//  *             ntlm_CreateSignature
//  * As both MakeSignature and VerifySignature need this, but different keys
//  * are needed for NTLM2, the logic goes into a helper function.
//  * To ensure maximal reusability, we can specify the direction as NTLM_SEND for
//  * signing/encrypting and NTLM_RECV for verifying/decrypting. When encrypting,
//  * the signature is encrypted after the message was encrypted, so
//  * CreateSignature shouldn't do it. In this case, encrypt_sig can be set to
//  * false.
//  */
// static SECURITY_STATUS ntlm_CreateSignature(PNegoHelper helper, PSecBufferDesc pMessage,
//         int token_idx, SignDirection direction, BOOL encrypt_sig)
// {
//     ULONG sign_version = 1;
//     UINT i;
//     PBYTE sig;
//     TRACE("%p, %p, %d, %d, %d\n", helper, pMessage, token_idx, direction,
//             encrypt_sig);
//
//     sig = pMessage->pBuffers[token_idx].pvBuffer;
//
//     if(helper->neg_flags & NTLMSSP_NEGOTIATE_NTLM2 &&
//             helper->neg_flags & NTLMSSP_NEGOTIATE_SIGN)
//     {
//         BYTE digest[16];
//         BYTE seq_no[4];
//         HMAC_MD5_CTX hmac_md5_ctx;
//
//         TRACE("Signing NTLM2 style\n");
//
//         if(direction == NTLM_SEND)
//         {
//             seq_no[0] = (helper->crypt.ntlm2.send_seq_no >>  0) & 0xff;
//             seq_no[1] = (helper->crypt.ntlm2.send_seq_no >>  8) & 0xff;
//             seq_no[2] = (helper->crypt.ntlm2.send_seq_no >> 16) & 0xff;
//             seq_no[3] = (helper->crypt.ntlm2.send_seq_no >> 24) & 0xff;
//
//             ++(helper->crypt.ntlm2.send_seq_no);
//
//             HMACMD5Init(&hmac_md5_ctx, helper->crypt.ntlm2.send_sign_key, 16);
//         }
//         else
//         {
//             seq_no[0] = (helper->crypt.ntlm2.recv_seq_no >>  0) & 0xff;
//             seq_no[1] = (helper->crypt.ntlm2.recv_seq_no >>  8) & 0xff;
//             seq_no[2] = (helper->crypt.ntlm2.recv_seq_no >> 16) & 0xff;
//             seq_no[3] = (helper->crypt.ntlm2.recv_seq_no >> 24) & 0xff;
//
//             ++(helper->crypt.ntlm2.recv_seq_no);
//
//             HMACMD5Init(&hmac_md5_ctx, helper->crypt.ntlm2.recv_sign_key, 16);
//         }
//
//         HMACMD5Update(&hmac_md5_ctx, seq_no, 4);
//         for( i = 0; i < pMessage->cBuffers; ++i )
//         {
//             if(pMessage->pBuffers[i].BufferType & SECBUFFER_DATA)
//                 HMACMD5Update(&hmac_md5_ctx, pMessage->pBuffers[i].pvBuffer,
//                         pMessage->pBuffers[i].cbBuffer);
//         }
//
//         HMACMD5Final(&hmac_md5_ctx, digest);
//
//         if(encrypt_sig && helper->neg_flags & NTLMSSP_NEGOTIATE_KEY_EXCHANGE)
//         {
//             if(direction == NTLM_SEND)
//                 SECUR32_arc4Process(helper->crypt.ntlm2.send_a4i, digest, 8);
//             else
//                 SECUR32_arc4Process(helper->crypt.ntlm2.recv_a4i, digest, 8);
//         }
//
//         /* The NTLM2 signature is the sign version */
//         sig[ 0] = (sign_version >>  0) & 0xff;
//         sig[ 1] = (sign_version >>  8) & 0xff;
//         sig[ 2] = (sign_version >> 16) & 0xff;
//         sig[ 3] = (sign_version >> 24) & 0xff;
//         /* The first 8 bytes of the digest */
//         memcpy(sig+4, digest, 8);
//         /* And the sequence number */
//         memcpy(sig+12, seq_no, 4);
//
//         pMessage->pBuffers[token_idx].cbBuffer = 16;
//
//         return SEC_E_OK;
//     }
//     if(helper->neg_flags & NTLMSSP_NEGOTIATE_SIGN)
//     {
//         ULONG crc = 0U;
//         TRACE("Signing NTLM1 style\n");
//
//         for(i=0; i < pMessage->cBuffers; ++i)
//         {
//             if(pMessage->pBuffers[i].BufferType & SECBUFFER_DATA)
//             {
//                 crc = ComputeCrc32(pMessage->pBuffers[i].pvBuffer,
//                     pMessage->pBuffers[i].cbBuffer, crc);
//             }
//         }
//
//         sig[ 0] = (sign_version >>  0) & 0xff;
//         sig[ 1] = (sign_version >>  8) & 0xff;
//         sig[ 2] = (sign_version >> 16) & 0xff;
//         sig[ 3] = (sign_version >> 24) & 0xff;
//         memset(sig+4, 0, 4);
//         sig[ 8] = (crc >>  0) & 0xff;
//         sig[ 9] = (crc >>  8) & 0xff;
//         sig[10] = (crc >> 16) & 0xff;
//         sig[11] = (crc >> 24) & 0xff;
//         sig[12] = (helper->crypt.ntlm.seq_num >>  0) & 0xff;
//         sig[13] = (helper->crypt.ntlm.seq_num >>  8) & 0xff;
//         sig[14] = (helper->crypt.ntlm.seq_num >> 16) & 0xff;
//         sig[15] = (helper->crypt.ntlm.seq_num >> 24) & 0xff;
//
//         ++(helper->crypt.ntlm.seq_num);
//
//         if(encrypt_sig)
//             SECUR32_arc4Process(helper->crypt.ntlm.a4i, sig+4, 12);
//         return SEC_E_OK;
//     }
//
//     if(helper->neg_flags & NTLMSSP_NEGOTIATE_ALWAYS_SIGN || helper->neg_flags == 0)
//     {
//         TRACE("Creating a dummy signature.\n");
//         /* A dummy signature is 0x01 followed by 15 bytes of 0x00 */
//         memset(pMessage->pBuffers[token_idx].pvBuffer, 0, 16);
//         memset(pMessage->pBuffers[token_idx].pvBuffer, 0x01, 1);
//         pMessage->pBuffers[token_idx].cbBuffer = 16;
//         return SEC_E_OK;
//     }
//
//     return SEC_E_UNSUPPORTED_FUNCTION;
// }
//
// /***********************************************************************
//  *              MakeSignature
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_MakeSignature(PCtxtHandle phContext,
//     ULONG fQOP, PSecBufferDesc pMessage, ULONG MessageSeqNo)
// {
//     PNegoHelper helper;
//     int token_idx;
//
//     TRACE("%p %d %p %d\n", phContext, fQOP, pMessage, MessageSeqNo);
//     if (!phContext)
//         return SEC_E_INVALID_HANDLE;
//
//     if(fQOP)
//         FIXME("Ignoring fQOP 0x%08x\n", fQOP);
//
//     if(MessageSeqNo)
//         FIXME("Ignoring MessageSeqNo\n");
//
//     if(!pMessage || !pMessage->pBuffers || pMessage->cBuffers < 2)
//         return SEC_E_INVALID_TOKEN;
//
//     /* If we didn't find a SECBUFFER_TOKEN type buffer */
//     if((token_idx = ntlm_GetTokenBufferIndex(pMessage)) == -1)
//         return SEC_E_INVALID_TOKEN;
//
//     if(pMessage->pBuffers[token_idx].cbBuffer < 16)
//         return SEC_E_BUFFER_TOO_SMALL;
//
//     helper = (PNegoHelper)phContext->dwLower;
//     TRACE("Negotiated flags are: 0x%08x\n", helper->neg_flags);
//
//     return ntlm_CreateSignature(helper, pMessage, token_idx, NTLM_SEND, TRUE);
// }
//
// /***********************************************************************
//  *              VerifySignature
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_VerifySignature(PCtxtHandle phContext,
//     PSecBufferDesc pMessage, ULONG MessageSeqNo, PULONG pfQOP)
// {
//     PNegoHelper helper;
//     UINT i;
//     int token_idx;
//     SECURITY_STATUS ret;
//     SecBufferDesc local_desc;
//     PSecBuffer     local_buff;
//     BYTE          local_sig[16];
//
//     TRACE("%p %p %d %p\n", phContext, pMessage, MessageSeqNo, pfQOP);
//     if(!phContext)
//         return SEC_E_INVALID_HANDLE;
//
//     if(!pMessage || !pMessage->pBuffers || pMessage->cBuffers < 2)
//         return SEC_E_INVALID_TOKEN;
//
//     if((token_idx = ntlm_GetTokenBufferIndex(pMessage)) == -1)
//         return SEC_E_INVALID_TOKEN;
//
//     if(pMessage->pBuffers[token_idx].cbBuffer < 16)
//         return SEC_E_BUFFER_TOO_SMALL;
//
//     if(MessageSeqNo)
//         FIXME("Ignoring MessageSeqNo\n");
//
//     helper = (PNegoHelper)phContext->dwLower;
//     TRACE("Negotiated flags: 0x%08x\n", helper->neg_flags);
//
//     local_buff = HeapAlloc(GetProcessHeap(), 0, pMessage->cBuffers * sizeof(SecBuffer));
//
//     local_desc.ulVersion = SECBUFFER_VERSION;
//     local_desc.cBuffers = pMessage->cBuffers;
//     local_desc.pBuffers = local_buff;
//
//     for(i=0; i < pMessage->cBuffers; ++i)
//     {
//         if(pMessage->pBuffers[i].BufferType == SECBUFFER_TOKEN)
//         {
//             local_buff[i].BufferType = SECBUFFER_TOKEN;
//             local_buff[i].cbBuffer = 16;
//             local_buff[i].pvBuffer = local_sig;
//         }
//         else
//         {
//             local_buff[i].BufferType = pMessage->pBuffers[i].BufferType;
//             local_buff[i].cbBuffer = pMessage->pBuffers[i].cbBuffer;
//             local_buff[i].pvBuffer = pMessage->pBuffers[i].pvBuffer;
//         }
//     }
//
//     if((ret = ntlm_CreateSignature(helper, &local_desc, token_idx, NTLM_RECV, TRUE)) != SEC_E_OK)
//         return ret;
//
//     if(memcmp(((PBYTE)local_buff[token_idx].pvBuffer) + 8,
//                 ((PBYTE)pMessage->pBuffers[token_idx].pvBuffer) + 8, 8))
//         ret = SEC_E_MESSAGE_ALTERED;
//     else
//         ret = SEC_E_OK;
//
//     HeapFree(GetProcessHeap(), 0, local_buff);
//
//     return ret;
//
// }
//
// /***********************************************************************
//  *             FreeCredentialsHandle
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_FreeCredentialsHandle(PCredHandle phCredential)
// {
//     SECURITY_STATUS ret;
//
//     if(phCredential){
//         PNtlmCredentials ntlm_cred = (PNtlmCredentials) phCredential->dwLower;
//         phCredential->dwUpper = 0;
//         phCredential->dwLower = 0;
//         if (ntlm_cred->password)
//             memset(ntlm_cred->password, 0, ntlm_cred->pwlen);
//         HeapFree(GetProcessHeap(), 0, ntlm_cred->password);
//         HeapFree(GetProcessHeap(), 0, ntlm_cred->username_arg);
//         HeapFree(GetProcessHeap(), 0, ntlm_cred->domain_arg);
//         HeapFree(GetProcessHeap(), 0, ntlm_cred);
//         ret = SEC_E_OK;
//     }
//     else
//         ret = SEC_E_OK;
//     
//     return ret;
// }
//
// /***********************************************************************
//  *             EncryptMessage
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_EncryptMessage(PCtxtHandle phContext,
//     ULONG fQOP, PSecBufferDesc pMessage, ULONG MessageSeqNo)
// {
//     PNegoHelper helper;
//     int token_idx, data_idx;
//
//     TRACE("(%p %d %p %d)\n", phContext, fQOP, pMessage, MessageSeqNo);
//
//     if(!phContext)
//         return SEC_E_INVALID_HANDLE;
//
//     if(fQOP)
//         FIXME("Ignoring fQOP\n");
//
//     if(MessageSeqNo)
//         FIXME("Ignoring MessageSeqNo\n");
//
//     if(!pMessage || !pMessage->pBuffers || pMessage->cBuffers < 2)
//         return SEC_E_INVALID_TOKEN;
//
//     if((token_idx = ntlm_GetTokenBufferIndex(pMessage)) == -1)
//         return SEC_E_INVALID_TOKEN;
//
//     if((data_idx = ntlm_GetDataBufferIndex(pMessage)) ==-1 )
//         return SEC_E_INVALID_TOKEN;
//
//     if(pMessage->pBuffers[token_idx].cbBuffer < 16)
//         return SEC_E_BUFFER_TOO_SMALL;
//
//     helper = (PNegoHelper) phContext->dwLower;
//
//     if(helper->neg_flags & NTLMSSP_NEGOTIATE_NTLM2 && 
//             helper->neg_flags & NTLMSSP_NEGOTIATE_SEAL)
//     { 
//         ntlm_CreateSignature(helper, pMessage, token_idx, NTLM_SEND, FALSE);
//         SECUR32_arc4Process(helper->crypt.ntlm2.send_a4i,
//                 pMessage->pBuffers[data_idx].pvBuffer,
//                 pMessage->pBuffers[data_idx].cbBuffer);
//
//         if(helper->neg_flags & NTLMSSP_NEGOTIATE_KEY_EXCHANGE)
//             SECUR32_arc4Process(helper->crypt.ntlm2.send_a4i,
//                     ((BYTE *)pMessage->pBuffers[token_idx].pvBuffer)+4, 8);
//     }
//     else
//     {
//         PBYTE sig;
//         ULONG save_flags;
//
//         /* EncryptMessage always produces real signatures, so make sure
//          * NTLMSSP_NEGOTIATE_SIGN is set*/
//         save_flags = helper->neg_flags;
//         helper->neg_flags |= NTLMSSP_NEGOTIATE_SIGN;
//         ntlm_CreateSignature(helper, pMessage, token_idx, NTLM_SEND, FALSE);
//         helper->neg_flags = save_flags;
//
//         sig = pMessage->pBuffers[token_idx].pvBuffer;
//
//         SECUR32_arc4Process(helper->crypt.ntlm.a4i,
//                 pMessage->pBuffers[data_idx].pvBuffer,
//                 pMessage->pBuffers[data_idx].cbBuffer);
//         SECUR32_arc4Process(helper->crypt.ntlm.a4i, sig+4, 12);
//
//         if(helper->neg_flags & NTLMSSP_NEGOTIATE_ALWAYS_SIGN || helper->neg_flags == 0)
//             memset(sig+4, 0, 4);
//     }
//     return SEC_E_OK;
// }
//
// /***********************************************************************
//  *             DecryptMessage
//  */
// SECURITY_STATUS SEC_ENTRY ntlm_DecryptMessage(PCtxtHandle phContext,
//     PSecBufferDesc pMessage, ULONG MessageSeqNo, PULONG pfQOP)
// {
//     SECURITY_STATUS ret;
//     ULONG ntlmssp_flags_save;
//     PNegoHelper helper;
//     int token_idx, data_idx;
//     TRACE("(%p %p %d %p)\n", phContext, pMessage, MessageSeqNo, pfQOP);
//
//     if(!phContext)
//         return SEC_E_INVALID_HANDLE;
//
//     if(MessageSeqNo)
//         FIXME("Ignoring MessageSeqNo\n");
//
//     if(!pMessage || !pMessage->pBuffers || pMessage->cBuffers < 2)
//         return SEC_E_INVALID_TOKEN;
//
//     if((token_idx = ntlm_GetTokenBufferIndex(pMessage)) == -1)
//         return SEC_E_INVALID_TOKEN;
//
//     if((data_idx = ntlm_GetDataBufferIndex(pMessage)) ==-1)
//         return SEC_E_INVALID_TOKEN;
//
//     if(pMessage->pBuffers[token_idx].cbBuffer < 16)
//         return SEC_E_BUFFER_TOO_SMALL;
//
//     helper = (PNegoHelper) phContext->dwLower;
//
//     if(helper->neg_flags & NTLMSSP_NEGOTIATE_NTLM2 && helper->neg_flags & NTLMSSP_NEGOTIATE_SEAL)
//     {
//         SECUR32_arc4Process(helper->crypt.ntlm2.recv_a4i,
//                 pMessage->pBuffers[data_idx].pvBuffer,
//                 pMessage->pBuffers[data_idx].cbBuffer);
//     }
//     else
//     {
//         SECUR32_arc4Process(helper->crypt.ntlm.a4i,
//                 pMessage->pBuffers[data_idx].pvBuffer,
//                 pMessage->pBuffers[data_idx].cbBuffer);
//     }
//
//     /* Make sure we use a session key for the signature check, EncryptMessage
//      * always does that, even in the dummy case */
//     ntlmssp_flags_save = helper->neg_flags;
//
//     helper->neg_flags |= NTLMSSP_NEGOTIATE_SIGN;
//     ret = ntlm_VerifySignature(phContext, pMessage, MessageSeqNo, pfQOP);
//
//     helper->neg_flags = ntlmssp_flags_save;
//
//     return ret;
// }
//
// static const SecurityFunctionTableA ntlmTableA = {
//     1,
//     NULL,   /* EnumerateSecurityPackagesA */
//     ntlm_QueryCredentialsAttributesA,   /* QueryCredentialsAttributesA */
//     ntlm_AcquireCredentialsHandleA,     /* AcquireCredentialsHandleA */
//     ntlm_FreeCredentialsHandle,         /* FreeCredentialsHandle */
//     NULL,   /* Reserved2 */
//     ntlm_InitializeSecurityContextA,    /* InitializeSecurityContextA */
//     ntlm_AcceptSecurityContext,         /* AcceptSecurityContext */
//     ntlm_CompleteAuthToken,             /* CompleteAuthToken */
//     ntlm_DeleteSecurityContext,         /* DeleteSecurityContext */
//     NULL,  /* ApplyControlToken */
//     ntlm_QueryContextAttributesA,       /* QueryContextAttributesA */
//     ntlm_ImpersonateSecurityContext,    /* ImpersonateSecurityContext */
//     ntlm_RevertSecurityContext,         /* RevertSecurityContext */
//     ntlm_MakeSignature,                 /* MakeSignature */
//     ntlm_VerifySignature,               /* VerifySignature */
//     FreeContextBuffer,                  /* FreeContextBuffer */
//     NULL,   /* QuerySecurityPackageInfoA */
//     NULL,   /* Reserved3 */
//     NULL,   /* Reserved4 */
//     NULL,   /* ExportSecurityContext */
//     NULL,   /* ImportSecurityContextA */
//     NULL,   /* AddCredentialsA */
//     NULL,   /* Reserved8 */
//     NULL,   /* QuerySecurityContextToken */
//     ntlm_EncryptMessage,                /* EncryptMessage */
//     ntlm_DecryptMessage,                /* DecryptMessage */
//     NULL,   /* SetContextAttributesA */
// };
//
// static const SecurityFunctionTableW ntlmTableW = {
//     1,
//     NULL,   /* EnumerateSecurityPackagesW */
//     ntlm_QueryCredentialsAttributesW,   /* QueryCredentialsAttributesW */
//     ntlm_AcquireCredentialsHandleW,     /* AcquireCredentialsHandleW */
//     ntlm_FreeCredentialsHandle,         /* FreeCredentialsHandle */
//     NULL,   /* Reserved2 */
//     ntlm_InitializeSecurityContextW,    /* InitializeSecurityContextW */
//     ntlm_AcceptSecurityContext,         /* AcceptSecurityContext */
//     ntlm_CompleteAuthToken,             /* CompleteAuthToken */
//     ntlm_DeleteSecurityContext,         /* DeleteSecurityContext */
//     NULL,  /* ApplyControlToken */
//     ntlm_QueryContextAttributesW,       /* QueryContextAttributesW */
//     ntlm_ImpersonateSecurityContext,    /* ImpersonateSecurityContext */
//     ntlm_RevertSecurityContext,         /* RevertSecurityContext */
//     ntlm_MakeSignature,                 /* MakeSignature */
//     ntlm_VerifySignature,               /* VerifySignature */
//     FreeContextBuffer,                  /* FreeContextBuffer */
//     NULL,   /* QuerySecurityPackageInfoW */
//     NULL,   /* Reserved3 */
//     NULL,   /* Reserved4 */
//     NULL,   /* ExportSecurityContext */
//     NULL,   /* ImportSecurityContextW */
//     NULL,   /* AddCredentialsW */
//     NULL,   /* Reserved8 */
//     NULL,   /* QuerySecurityContextToken */
//     ntlm_EncryptMessage,                /* EncryptMessage */
//     ntlm_DecryptMessage,                /* DecryptMessage */
//     NULL,   /* SetContextAttributesW */
// };
//
// #define NTLM_COMMENT \
//    { 'N', 'T', 'L', 'M', ' ', \
//      'S', 'e', 'c', 'u', 'r', 'i', 't', 'y', ' ', \
//      'P', 'a', 'c', 'k', 'a', 'g', 'e', 0}
//
// static CHAR ntlm_comment_A[] = NTLM_COMMENT;
// static WCHAR ntlm_comment_W[] = NTLM_COMMENT;
//
// #define NTLM_NAME {'N', 'T', 'L', 'M', 0}
//
// static char ntlm_name_A[] = NTLM_NAME;
// static WCHAR ntlm_name_W[] = NTLM_NAME;
//
// /* According to Windows, NTLM has the following capabilities.  */
// #define CAPS ( \
//     SECPKG_FLAG_INTEGRITY  | \
//     SECPKG_FLAG_PRIVACY    | \
//     SECPKG_FLAG_TOKEN_ONLY | \
//     SECPKG_FLAG_CONNECTION | \
//     SECPKG_FLAG_MULTI_REQUIRED    | \
//     SECPKG_FLAG_IMPERSONATION     | \
//     SECPKG_FLAG_ACCEPT_WIN32_NAME | \
//     SECPKG_FLAG_NEGOTIABLE        | \
//     SECPKG_FLAG_LOGON             | \
//     SECPKG_FLAG_RESTRICTED_TOKENS )
//
// static const SecPkgInfoW infoW = {
//     CAPS,
//     1,
//     RPC_C_AUTHN_WINNT,
//     NTLM_MAX_BUF,
//     ntlm_name_W,
//     ntlm_comment_W
// };
//
// static const SecPkgInfoA infoA = {
//     CAPS,
//     1,
//     RPC_C_AUTHN_WINNT,
//     NTLM_MAX_BUF,
//     ntlm_name_A,
//     ntlm_comment_A
// };
//
// SecPkgInfoA *ntlm_package_infoA = (SecPkgInfoA *)&infoA;
// SecPkgInfoW *ntlm_package_infoW = (SecPkgInfoW *)&infoW;
//
// void SECUR32_initNTLMSP(void)
// {
//     PNegoHelper helper;
//     static CHAR version[] = "--version";
//
//     SEC_CHAR *args[] = {
//         ntlm_auth,
//         version,
//         NULL };
//
//     if(fork_helper(&helper, ntlm_auth, args) != SEC_E_OK)
//         helper = NULL;
//     else
//         check_version(helper);
//
//     if( helper &&
//         ((helper->major >  MIN_NTLM_AUTH_MAJOR_VERSION) ||
//          (helper->major == MIN_NTLM_AUTH_MAJOR_VERSION  &&
//           helper->minor >  MIN_NTLM_AUTH_MINOR_VERSION) ||
//          (helper->major == MIN_NTLM_AUTH_MAJOR_VERSION  &&
//           helper->minor == MIN_NTLM_AUTH_MINOR_VERSION  &&
//           helper->micro >= MIN_NTLM_AUTH_MICRO_VERSION)) )
//     {
//         SecureProvider *provider = SECUR32_addProvider(&ntlmTableA, &ntlmTableW, NULL);
//         SECUR32_addPackages(provider, 1L, ntlm_package_infoA, ntlm_package_infoW);
//     }
//     else
//     {
//         ERR_(winediag)("%s was not found or is outdated. "
//                        "Make sure that ntlm_auth >= %d.%d.%d is in your path. "
//                        "Usually, you can find it in the winbind package of your distribution.\n",
//                        ntlm_auth,
//                        MIN_NTLM_AUTH_MAJOR_VERSION,
//                        MIN_NTLM_AUTH_MINOR_VERSION,
//                        MIN_NTLM_AUTH_MICRO_VERSION);
//
//     }
//     cleanup_helper(helper);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
