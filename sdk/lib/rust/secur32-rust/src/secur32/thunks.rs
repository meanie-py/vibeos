use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.
#[no_mangle]
pub unsafe extern "system" fn thunk_AcquireCredentialsHandleA(
    pszPrincipal: PSTR,
    pszPackage: PSTR,
    fCredentialsUse: ULONG,
    pvLogonID: PLUID,
    pAuthData: PVOID,
    pGetKeyFn: SEC_GET_KEY_FN,
    pvGetKeyArgument: PVOID,
    phCredential: PCredHandle,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !pszPackage.is_null() {
        let mut principal: _UNICODE_STRING = core::mem::zeroed();
        let mut package: _UNICODE_STRING = core::mem::zeroed();

        RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(principal), pszPrincipal);
        RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(package), pszPackage);
        ret = AcquireCredentialsHandleW(
            principal.Buffer,
            package.Buffer,
            fCredentialsUse,
            pvLogonID,
            pAuthData,
            pGetKeyFn,
            pvGetKeyArgument,
            phCredential,
            ptsExpiry,
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(principal));
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(package));
    } else {
        ret = SEC_E_SECPKG_NOT_FOUND;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_AcquireCredentialsHandleW(
    pszPrincipal: *mut SEC_WCHAR,
    pszPackage: *mut SEC_WCHAR,
    fCredentialsUse: ULONG,
    pvLogonID: PLUID,
    pAuthData: PVOID,
    pGetKeyFn: SEC_GET_KEY_FN,
    pvGetKeyArgument: PVOID,
    phCredential: PCredHandle,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !pszPackage.is_null() {
        let principal: PSTR = SECUR32_AllocMultiByteFromWide(pszPrincipal);
        let package: PSTR = SECUR32_AllocMultiByteFromWide(pszPackage);
        ret = AcquireCredentialsHandleA(
            principal,
            package,
            fCredentialsUse,
            pvLogonID,
            pAuthData,
            pGetKeyFn,
            pvGetKeyArgument,
            phCredential,
            ptsExpiry,
        );
        HeapFree(GetProcessHeap(), 0, principal as PVOID);
        HeapFree(GetProcessHeap(), 0, package as PVOID);
    } else {
        ret = SEC_E_SECPKG_NOT_FOUND;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_QueryCredentialsAttributesA(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p %d %p\n\0".as_ptr() as PSTR, phCredential, ulAttribute, pBuffer);
    if !phCredential.is_null() {
        let package = (*phCredential).dwUpper as *mut SecurePackage;
        let cred = (*phCredential).dwLower as PCredHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(query) = (*(*package).provider).fnTableW.QueryCredentialsAttributesW {
                let mut ret_local = query(cred, ulAttribute, pBuffer);
                if ret_local == SEC_E_OK {
                    match ulAttribute {
                        SECPKG_CRED_ATTR_NAMES => {
                            let names = pBuffer as PSecPkgCredentials_NamesW;
                            let oldUser = (*names).sUserName;

                            if !oldUser.is_null() {
                                (*names).sUserName =
                                    SECUR32_AllocMultiByteFromWide(oldUser) as PWSTR;
                                if let Some(free) =
                                    (*(*package).provider).fnTableW.FreeContextBuffer
                                {
                                    free(oldUser as *mut core::ffi::c_void);
                                }
                            }
                        }
                        _ => {
                            WARN!(
                                b"attribute type %d unknown\n\0".as_ptr() as PSTR,
                                ulAttribute,
                            );
                            ret_local = SEC_E_INTERNAL_ERROR;
                        }
                    }
                }
                ret = ret_local;
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_QueryCredentialsAttributesW(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p %d %p\n\0".as_ptr() as PSTR, phCredential, ulAttribute, pBuffer);
    if !phCredential.is_null() {
        let package = (*phCredential).dwUpper as *mut SecurePackage;
        let cred = (*phCredential).dwLower as PCredHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if (*(*package).provider).fnTableA.QueryCredentialsAttributesA.is_some() {
                let mut ret_inner = ((*(*package).provider).fnTableA.QueryCredentialsAttributesA.unwrap())(
                    cred,
                    ulAttribute,
                    pBuffer,
                );
                if ret_inner == SEC_E_OK {
                    match ulAttribute {
                        SECPKG_CRED_ATTR_NAMES => {
                            let names = pBuffer as PSecPkgCredentials_NamesA;
                            let oldUser = (*names).sUserName;

                            if !oldUser.is_null() {
                                (*names).sUserName =
                                    SECUR32_AllocWideFromMultiByte(oldUser) as PSTR;
                                ((*(*package).provider).fnTableA.FreeContextBuffer.unwrap())(
                                    oldUser as *mut core::ffi::c_void,
                                );
                            }
                        }
                        _ => {
                            WARN!(b"attribute type %d unknown\n\0".as_ptr() as PSTR, ulAttribute);
                            ret_inner = SEC_E_INTERNAL_ERROR;
                        }
                    }
                }
                ret = ret_inner;
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_InitializeSecurityContextA(
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

    TRACE!(
        b"%p %p %s %d %d %d %p %d %p %p %p %p\n\0".as_ptr() as PSTR,
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

    if !phCredential.is_null() {
        let package = (*phCredential).dwUpper as *mut SecurePackage;

        if !package.is_null() && !(*package).provider.is_null() {
            if (*(*package).provider).fnTableW.InitializeSecurityContextW.is_some() {
                let mut target: _UNICODE_STRING = core::mem::zeroed();

                RtlCreateUnicodeStringFromAsciiz(&mut target, pszTargetName);
                ret = ((*(*package).provider).fnTableW.InitializeSecurityContextW.unwrap())(
                    phCredential,
                    phContext,
                    target.Buffer,
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
                RtlFreeUnicodeString(&mut target);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_InitializeSecurityContextW(
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
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%p %p %s %d %d %d %p %d %p %p %p %p\n\0".as_ptr() as PSTR,
        phCredential,
        phContext,
        debugstr_w(pszTargetName),
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

    if !phCredential.is_null() {
        let package = (*phCredential).dwUpper as *mut SecurePackage;

        if !package.is_null() && !(*package).provider.is_null() {
            if (*(*package).provider).fnTableA.InitializeSecurityContextA.is_some() {
                let target = SECUR32_AllocMultiByteFromWide(pszTargetName);

                ret = (*(*package).provider)
                    .fnTableA
                    .InitializeSecurityContextA
                    .unwrap()(
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
                HeapFree(GetProcessHeap(), 0, target as *mut core::ffi::c_void);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_AddCredentialsA(
    hCredentials: PCredHandle,
    pszPrincipal: *mut SEC_CHAR,
    pszPackage: *mut SEC_CHAR,
    fCredentialUse: ULONG,
    pAuthData: *mut core::ffi::c_void,
    pGetKeyFn: SEC_GET_KEY_FN,
    pvGetKeyArgument: *mut core::ffi::c_void,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !hCredentials.is_null() {
        let package = (*hCredentials).dwUpper as *mut SecurePackage;
        let cred = (*hCredentials).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(AddCredentialsW) = (*(*package).provider).fnTableW.AddCredentialsW {
                let mut szPrincipal: _UNICODE_STRING = core::mem::zeroed();
                let mut szPackage: _UNICODE_STRING = core::mem::zeroed();

                RtlCreateUnicodeStringFromAsciiz(
                    core::ptr::addr_of_mut!(szPrincipal),
                    pszPrincipal,
                );
                RtlCreateUnicodeStringFromAsciiz(
                    core::ptr::addr_of_mut!(szPackage),
                    pszPackage,
                );
                ret = AddCredentialsW(
                    cred,
                    szPrincipal.Buffer,
                    szPackage.Buffer,
                    fCredentialUse,
                    pAuthData,
                    pGetKeyFn,
                    pvGetKeyArgument,
                    ptsExpiry,
                );
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(szPrincipal));
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(szPackage));
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_AddCredentialsW(
    hCredentials: PCredHandle,
    pszPrincipal: *mut SEC_WCHAR,
    pszPackage: *mut SEC_WCHAR,
    fCredentialUse: ULONG,
    pAuthData: *mut core::ffi::c_void,
    pGetKeyFn: SEC_GET_KEY_FN,
    pvGetKeyArgument: *mut core::ffi::c_void,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%p %s %s %d %p %p %p %p\n\0".as_ptr() as PSTR,
        hCredentials,
        debugstr_w(pszPrincipal),
        debugstr_w(pszPackage),
        fCredentialUse,
        pAuthData,
        pGetKeyFn,
        pvGetKeyArgument,
        ptsExpiry,
    );

    if !hCredentials.is_null() {
        let package = (*hCredentials).dwUpper as *mut SecurePackage;
        let cred = (*hCredentials).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(AddCredentialsA) = (*(*package).provider).fnTableA.AddCredentialsA {
                let szPrincipal = SECUR32_AllocMultiByteFromWide(pszPrincipal);
                let szPackage = SECUR32_AllocMultiByteFromWide(pszPackage);

                ret = AddCredentialsA(
                    cred,
                    szPrincipal,
                    szPackage,
                    fCredentialUse,
                    pAuthData,
                    pGetKeyFn,
                    pvGetKeyArgument,
                    ptsExpiry,
                );
                HeapFree(GetProcessHeap(), 0, szPrincipal as *mut core::ffi::c_void);
                HeapFree(GetProcessHeap(), 0, szPackage as *mut core::ffi::c_void);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}

unsafe fn _copyPackageInfoFlatWToA(infoW: *const SecPkgInfoW) -> PSecPkgInfoA {
    let ret: PSecPkgInfoA;

    if !infoW.is_null() {
        let mut bytesNeeded: usize = core::mem::size_of::<SecPkgInfoA>();
        let mut nameLen: i32 = 0;
        let mut commentLen: i32 = 0;

        if !(*infoW).Name.is_null() {
            nameLen = WideCharToMultiByte(CP_ACP, 0, (*infoW).Name, -1,
                core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
            bytesNeeded += nameLen as usize;
        }
        if !(*infoW).Comment.is_null() {
            commentLen = WideCharToMultiByte(CP_ACP, 0, (*infoW).Comment, -1,
                core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
            bytesNeeded += commentLen as usize;
        }
        ret = HeapAlloc(GetProcessHeap(), 0, bytesNeeded as _) as PSecPkgInfoA;
        if !ret.is_null() {
            let mut nextString: PSTR = (ret as PBYTE).add(core::mem::size_of::<SecPkgInfoA>()) as PSTR;

            memcpy(ret as *mut core::ffi::c_void, infoW as *const core::ffi::c_void,
                core::mem::size_of::<SecPkgInfoA>());
            if !(*infoW).Name.is_null() {
                (*ret).Name = nextString;
                WideCharToMultiByte(CP_ACP, 0, (*infoW).Name, -1, nextString,
                    nameLen, core::ptr::null_mut(), core::ptr::null_mut());
                nextString = nextString.add(nameLen as usize);
            } else {
                (*ret).Name = core::ptr::null_mut();
            }
            if !(*infoW).Comment.is_null() {
                (*ret).Comment = nextString;
                WideCharToMultiByte(CP_ACP, 0, (*infoW).Comment, -1, nextString,
                    nameLen, core::ptr::null_mut(), core::ptr::null_mut());
            } else {
                (*ret).Comment = core::ptr::null_mut();
            }
        }
    } else {
        ret = core::ptr::null_mut();
    }
    ret
}

unsafe fn thunk_ContextAttributesWToA(
    package: *mut SecurePackage,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let mut ret: SECURITY_STATUS = SEC_E_OK;

    if !package.is_null() && !pBuffer.is_null() {
        match ulAttribute {
            SECPKG_ATTR_NAMES => {
                let names = pBuffer as PSecPkgContext_NamesW;
                let oldUser = (*names).sUserName;
                if !oldUser.is_null() {
                    (*names).sUserName = SECUR32_AllocMultiByteFromWide(oldUser) as PWSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldUser as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_AUTHORITY => {
                let names = pBuffer as PSecPkgContext_AuthorityW;
                let oldAuth = (*names).sAuthorityName;
                if !oldAuth.is_null() {
                    (*names).sAuthorityName = SECUR32_AllocMultiByteFromWide(oldAuth) as PWSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldAuth as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_KEY_INFO => {
                let info = pBuffer as PSecPkgContext_KeyInfoW;
                let oldSigAlgName = (*info).sSignatureAlgorithmName;
                let oldEncAlgName = (*info).sEncryptAlgorithmName;
                if !oldSigAlgName.is_null() {
                    (*info).sSignatureAlgorithmName =
                        SECUR32_AllocMultiByteFromWide(oldSigAlgName) as PWSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldSigAlgName as *mut core::ffi::c_void,
                    );
                }
                if !oldEncAlgName.is_null() {
                    (*info).sEncryptAlgorithmName =
                        SECUR32_AllocMultiByteFromWide(oldEncAlgName) as PWSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldEncAlgName as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_PACKAGE_INFO => {
                let info = pBuffer as PSecPkgContext_PackageInfoW;
                let oldPkgInfo = (*info).PackageInfo;
                if !oldPkgInfo.is_null() {
                    (*info).PackageInfo = _copyPackageInfoFlatWToA(oldPkgInfo) as PSecPkgInfoW;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldPkgInfo as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_NEGOTIATION_INFO => {
                let info = pBuffer as PSecPkgContext_NegotiationInfoW;
                let oldPkgInfo = (*info).PackageInfo;
                if !oldPkgInfo.is_null() {
                    (*info).PackageInfo = _copyPackageInfoFlatWToA(oldPkgInfo) as PSecPkgInfoW;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldPkgInfo as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_NATIVE_NAMES => {
                let names = pBuffer as PSecPkgContext_NativeNamesW;
                let oldClient = (*names).sClientName;
                let oldServer = (*names).sServerName;
                if !oldClient.is_null() {
                    (*names).sClientName = SECUR32_AllocMultiByteFromWide(oldClient) as PWSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldClient as *mut core::ffi::c_void,
                    );
                }
                if !oldServer.is_null() {
                    (*names).sServerName = SECUR32_AllocMultiByteFromWide(oldServer) as PWSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldServer as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_CREDENTIAL_NAME => {
                let name = pBuffer as PSecPkgContext_CredentialNameW;
                let oldCred = (*name).sCredentialName;
                if !oldCred.is_null() {
                    (*name).sCredentialName = SECUR32_AllocMultiByteFromWide(oldCred) as PWSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldCred as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_ACCESS_TOKEN
            | SECPKG_ATTR_DCE_INFO
            | SECPKG_ATTR_FLAGS
            | SECPKG_ATTR_LIFESPAN
            | SECPKG_ATTR_PASSWORD_EXPIRY
            | SECPKG_ATTR_SESSION_KEY
            | SECPKG_ATTR_SIZES
            | SECPKG_ATTR_STREAM_SIZES
            | SECPKG_ATTR_TARGET_INFORMATION => {}
            _ => {
                ret = SEC_E_INTERNAL_ERROR;
            }
        }
    } else {
        ret = SEC_E_INVALID_TOKEN;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_QueryContextAttributesA(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p %d %p\n\0".as_ptr() as *const i8, phContext, ulAttribute, pBuffer);
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(QueryContextAttributesW) =
                (*(*package).provider).fnTableW.QueryContextAttributesW
            {
                ret = QueryContextAttributesW(ctxt, ulAttribute, pBuffer);
                if ret == SEC_E_OK {
                    return thunk_ContextAttributesWToA(package, ulAttribute, pBuffer);
                }
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}

unsafe fn _copyPackageInfoFlatAToW(infoA: *const SecPkgInfoA) -> PSecPkgInfoW {
    let ret: PSecPkgInfoW;

    if !infoA.is_null() {
        let mut bytesNeeded: usize = core::mem::size_of::<SecPkgInfoW>();
        let mut nameLen: i32 = 0;
        let mut commentLen: i32 = 0;

        if !(*infoA).Name.is_null() {
            nameLen = MultiByteToWideChar(CP_ACP, 0, (*infoA).Name, -1,
                core::ptr::null_mut(), 0);
            bytesNeeded += nameLen as usize * core::mem::size_of::<WCHAR>();
        }
        if !(*infoA).Comment.is_null() {
            commentLen = MultiByteToWideChar(CP_ACP, 0, (*infoA).Comment, -1,
                core::ptr::null_mut(), 0);
            bytesNeeded += commentLen as usize * core::mem::size_of::<WCHAR>();
        }
        ret = HeapAlloc(GetProcessHeap(), 0, bytesNeeded as _) as PSecPkgInfoW;
        if !ret.is_null() {
            let mut nextString: PWSTR =
                (ret as PBYTE).add(core::mem::size_of::<SecPkgInfoW>()) as PWSTR;

            memcpy(ret as *mut _, infoA as *const _, core::mem::size_of::<SecPkgInfoA>());
            if !(*infoA).Name.is_null() {
                (*ret).Name = nextString;
                MultiByteToWideChar(CP_ACP, 0, (*infoA).Name, -1, nextString, nameLen);
                nextString = nextString.add(nameLen as usize);
            } else {
                (*ret).Name = core::ptr::null_mut();
            }
            if !(*infoA).Comment.is_null() {
                (*ret).Comment = nextString;
                MultiByteToWideChar(CP_ACP, 0, (*infoA).Comment, -1, nextString, commentLen);
            } else {
                (*ret).Comment = core::ptr::null_mut();
            }
        }
    } else {
        ret = core::ptr::null_mut();
    }
    ret
}

unsafe fn thunk_ContextAttributesAToW(
    package: *mut SecurePackage,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let mut ret: SECURITY_STATUS = SEC_E_OK;

    if !package.is_null() && !pBuffer.is_null() {
        match ulAttribute {
            SECPKG_ATTR_NAMES => {
                let names = pBuffer as PSecPkgContext_NamesA;
                let oldUser = (*names).sUserName;

                if !oldUser.is_null() {
                    (*names).sUserName = SECUR32_AllocWideFromMultiByte(oldUser) as PSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldUser as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_AUTHORITY => {
                let names = pBuffer as PSecPkgContext_AuthorityA;
                let oldAuth = (*names).sAuthorityName;

                if !oldAuth.is_null() {
                    (*names).sAuthorityName = SECUR32_AllocWideFromMultiByte(oldAuth) as PSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldAuth as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_KEY_INFO => {
                let info = pBuffer as PSecPkgContext_KeyInfoA;
                let oldSigAlgName = (*info).sSignatureAlgorithmName;
                let oldEncAlgName = (*info).sEncryptAlgorithmName;

                if !oldSigAlgName.is_null() {
                    (*info).sSignatureAlgorithmName =
                        SECUR32_AllocWideFromMultiByte(oldSigAlgName) as PSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldSigAlgName as *mut core::ffi::c_void,
                    );
                }
                if !oldEncAlgName.is_null() {
                    (*info).sEncryptAlgorithmName =
                        SECUR32_AllocWideFromMultiByte(oldEncAlgName) as PSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldEncAlgName as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_PACKAGE_INFO => {
                let info = pBuffer as PSecPkgContext_PackageInfoA;
                let oldPkgInfo = (*info).PackageInfo;

                if !oldPkgInfo.is_null() {
                    (*info).PackageInfo = _copyPackageInfoFlatAToW(oldPkgInfo) as PSecPkgInfoA;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldPkgInfo as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_NEGOTIATION_INFO => {
                let info = pBuffer as PSecPkgContext_NegotiationInfoA;
                let oldPkgInfo = (*info).PackageInfo;

                if !oldPkgInfo.is_null() {
                    (*info).PackageInfo = _copyPackageInfoFlatAToW(oldPkgInfo) as PSecPkgInfoA;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldPkgInfo as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_NATIVE_NAMES => {
                let names = pBuffer as PSecPkgContext_NativeNamesA;
                let oldClient = (*names).sClientName;
                let oldServer = (*names).sServerName;

                if !oldClient.is_null() {
                    (*names).sClientName = SECUR32_AllocWideFromMultiByte(oldClient) as PSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldClient as *mut core::ffi::c_void,
                    );
                }
                if !oldServer.is_null() {
                    (*names).sServerName = SECUR32_AllocWideFromMultiByte(oldServer) as PSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldServer as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_CREDENTIAL_NAME => {
                let name = pBuffer as PSecPkgContext_CredentialNameA;
                let oldCred = (*name).sCredentialName;

                if !oldCred.is_null() {
                    (*name).sCredentialName = SECUR32_AllocWideFromMultiByte(oldCred) as PSTR;
                    (*(*package).provider).fnTableW.FreeContextBuffer.unwrap()(
                        oldCred as *mut core::ffi::c_void,
                    );
                }
            }
            SECPKG_ATTR_ACCESS_TOKEN
            | SECPKG_ATTR_DCE_INFO
            | SECPKG_ATTR_FLAGS
            | SECPKG_ATTR_LIFESPAN
            | SECPKG_ATTR_PASSWORD_EXPIRY
            | SECPKG_ATTR_SESSION_KEY
            | SECPKG_ATTR_SIZES
            | SECPKG_ATTR_STREAM_SIZES
            | SECPKG_ATTR_TARGET_INFORMATION => {}
            _ => {
                ret = SEC_E_INTERNAL_ERROR;
            }
        }
    } else {
        ret = SEC_E_INVALID_TOKEN;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_QueryContextAttributesW(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p %d %p\n\0".as_ptr() as PSTR, phContext, ulAttribute, pBuffer);
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(QueryContextAttributesA) =
                (*(*package).provider).fnTableA.QueryContextAttributesA
            {
                ret = QueryContextAttributesA(ctxt, ulAttribute, pBuffer);
                if ret == SEC_E_OK {
                    return thunk_ContextAttributesAToW(package, ulAttribute, pBuffer);
                }
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_SetContextAttributesA(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
    cbBuffer: ULONG,
) -> SECURITY_STATUS {
    let mut ret: SECURITY_STATUS;

    TRACE!(
        b"%p %d %p %d\n\0".as_ptr() as PSTR,
        phContext,
        ulAttribute,
        pBuffer,
        cbBuffer,
    );
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null()
            && !(*package).provider.is_null()
            && !pBuffer.is_null()
            && cbBuffer != 0
        {
            if (*(*package).provider).fnTableW.SetContextAttributesW.is_some() {
                ret = thunk_ContextAttributesWToA(package, ulAttribute, pBuffer);
                if ret == SEC_E_OK {
                    ret = ((*(*package).provider).fnTableW.SetContextAttributesW.unwrap())(
                        ctxt,
                        ulAttribute,
                        pBuffer,
                        cbBuffer,
                    );
                }
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_SetContextAttributesW(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
    cbBuffer: ULONG,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%p %d %p %d\n\0".as_ptr() as *const core::ffi::c_char,
        phContext,
        ulAttribute,
        pBuffer,
        cbBuffer,
    );
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null()
            && !(*package).provider.is_null()
            && !pBuffer.is_null()
            && cbBuffer != 0
        {
            if (*(*package).provider).fnTableA.SetContextAttributesA.is_some() {
                ret = thunk_ContextAttributesAToW(package, ulAttribute, pBuffer);
                if ret == SEC_E_OK {
                    let ret2 = ((*(*package).provider)
                        .fnTableA
                        .SetContextAttributesA
                        .unwrap())(ctxt, ulAttribute, pBuffer, cbBuffer);
                    return ret2;
                }
                return ret;
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_ImportSecurityContextA(
    pszPackage: *mut SEC_CHAR,
    pPackedContext: PSecBuffer,
    Token: *mut core::ffi::c_void,
    phContext: PCtxtHandle,
) -> SECURITY_STATUS {
    let mut package: _UNICODE_STRING = core::mem::zeroed();

    RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(package), pszPackage as PSTR);
    let ret = ImportSecurityContextW(package.Buffer, pPackedContext, Token, phContext);
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(package));
    ret
}
#[no_mangle]
pub unsafe extern "system" fn thunk_ImportSecurityContextW(
    pszPackage: *mut SEC_WCHAR,
    pPackedContext: PSecBuffer,
    Token: *mut core::ffi::c_void,
    phContext: PCtxtHandle,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;
    let package: PSTR = SECUR32_AllocMultiByteFromWide(pszPackage);

    TRACE!(
        b"%s %p %p %p\n\0".as_ptr() as PSTR,
        debugstr_w(pszPackage),
        pPackedContext,
        Token,
        phContext,
    );
    ret = ImportSecurityContextA(package, pPackedContext, Token, phContext);
    HeapFree(GetProcessHeap(), 0, package as *mut core::ffi::c_void);
    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright (C) 2004 Juan Lang
//  *
//  * This file implements thunks between wide char and multibyte functions for
//  * SSPs that only provide one or the other.
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
// #include "precomp.h"
//
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(secur32);
//
// SECURITY_STATUS SEC_ENTRY thunk_AcquireCredentialsHandleA(
//  SEC_CHAR *pszPrincipal, SEC_CHAR *pszPackage, ULONG fCredentialsUse,
//  PLUID pvLogonID, PVOID pAuthData, SEC_GET_KEY_FN pGetKeyFn,
//  PVOID pvGetKeyArgument, PCredHandle phCredential, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%s %s %d %p %p %p %p %p %p\n", debugstr_a(pszPrincipal),
//      debugstr_a(pszPackage), fCredentialsUse, pvLogonID, pAuthData, pGetKeyFn,
//      pvGetKeyArgument, phCredential, ptsExpiry);
//     if (pszPackage)
//     {
//         UNICODE_STRING principal, package;
//
//         RtlCreateUnicodeStringFromAsciiz(&principal, pszPrincipal);
//         RtlCreateUnicodeStringFromAsciiz(&package, pszPackage);
//         ret = AcquireCredentialsHandleW(principal.Buffer, package.Buffer,
//          fCredentialsUse, pvLogonID, pAuthData, pGetKeyFn, pvGetKeyArgument,
//          phCredential, ptsExpiry);
//         RtlFreeUnicodeString(&principal);
//         RtlFreeUnicodeString(&package);
//     }
//     else
//         ret = SEC_E_SECPKG_NOT_FOUND;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_AcquireCredentialsHandleW(
//  SEC_WCHAR *pszPrincipal, SEC_WCHAR *pszPackage, ULONG fCredentialsUse,
//  PLUID pvLogonID, PVOID pAuthData, SEC_GET_KEY_FN pGetKeyFn,
//  PVOID pvGetKeyArgument, PCredHandle phCredential, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%s %s %d %p %p %p %p %p %p\n", debugstr_w(pszPrincipal),
//      debugstr_w(pszPackage), fCredentialsUse, pvLogonID, pAuthData, pGetKeyFn,
//      pvGetKeyArgument, phCredential, ptsExpiry);
//     if (pszPackage)
//     {
//         PSTR principal, package;
//
//         principal = SECUR32_AllocMultiByteFromWide(pszPrincipal);
//         package = SECUR32_AllocMultiByteFromWide(pszPackage);
//         ret = AcquireCredentialsHandleA(principal, package, fCredentialsUse,
//          pvLogonID, pAuthData, pGetKeyFn, pvGetKeyArgument, phCredential,
//          ptsExpiry);
//         HeapFree(GetProcessHeap(), 0, principal);
//         HeapFree(GetProcessHeap(), 0, package);
//     }
//     else
//         ret = SEC_E_SECPKG_NOT_FOUND;
//     return ret;
// }
//
// /* thunking is pretty dicey for these--the output type depends on ulAttribute,
//  * so we have to know about every type the caller does
//  */
// SECURITY_STATUS SEC_ENTRY thunk_QueryCredentialsAttributesA(
//  PCredHandle phCredential, ULONG ulAttribute, void *pBuffer)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p\n", phCredential, ulAttribute, pBuffer);
//     if (phCredential)
//     {
//         SecurePackage *package = (SecurePackage *)phCredential->dwUpper;
//         PCredHandle cred = (PCredHandle)phCredential->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.QueryCredentialsAttributesW)
//             {
//                 ret = package->provider->fnTableW.QueryCredentialsAttributesW(
//                  cred, ulAttribute, pBuffer);
//                 if (ret == SEC_E_OK)
//                 {
//                     switch (ulAttribute)
//                     {
//                         case SECPKG_CRED_ATTR_NAMES:
//                         {
//                             PSecPkgCredentials_NamesW names =
//                              (PSecPkgCredentials_NamesW)pBuffer;
//                             SEC_WCHAR *oldUser = names->sUserName;
//
//                             if (oldUser)
//                             {
//                                 names->sUserName =
//                                  (PWSTR)SECUR32_AllocMultiByteFromWide(oldUser);
//                                 package->provider->fnTableW.FreeContextBuffer(
//                                  oldUser);
//                             }
//                             break;
//                         }
//                         default:
//                             WARN("attribute type %d unknown\n", ulAttribute);
//                             ret = SEC_E_INTERNAL_ERROR;
//                     }
//                 }
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_QueryCredentialsAttributesW(
//  PCredHandle phCredential, ULONG ulAttribute, void *pBuffer)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p\n", phCredential, ulAttribute, pBuffer);
//     if (phCredential)
//     {
//         SecurePackage *package = (SecurePackage *)phCredential->dwUpper;
//         PCredHandle cred = (PCredHandle)phCredential->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableA.QueryCredentialsAttributesA)
//             {
//                 ret = package->provider->fnTableA.QueryCredentialsAttributesA(
//                  cred, ulAttribute, pBuffer);
//                 if (ret == SEC_E_OK)
//                 {
//                     switch (ulAttribute)
//                     {
//                         case SECPKG_CRED_ATTR_NAMES:
//                         {
//                             PSecPkgCredentials_NamesA names =
//                              (PSecPkgCredentials_NamesA)pBuffer;
//                             SEC_CHAR *oldUser = names->sUserName;
//
//                             if (oldUser)
//                             {
//                                 names->sUserName =
//                                  (PSTR)SECUR32_AllocWideFromMultiByte(oldUser);
//                                 package->provider->fnTableA.FreeContextBuffer(
//                                  oldUser);
//                             }
//                             break;
//                         }
//                         default:
//                             WARN("attribute type %d unknown\n", ulAttribute);
//                             ret = SEC_E_INTERNAL_ERROR;
//                     }
//                 }
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_InitializeSecurityContextA(
//  PCredHandle phCredential, PCtxtHandle phContext,
//  SEC_CHAR *pszTargetName, ULONG fContextReq,
//  ULONG Reserved1, ULONG TargetDataRep, PSecBufferDesc pInput,
//  ULONG Reserved2, PCtxtHandle phNewContext, PSecBufferDesc pOutput,
//  ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p %s %d %d %d %p %d %p %p %p %p\n", phCredential, phContext,
//      debugstr_a(pszTargetName), fContextReq, Reserved1, TargetDataRep, pInput,
//      Reserved1, phNewContext, pOutput, pfContextAttr, ptsExpiry);
//     if (phCredential)
//     {
//         SecurePackage *package = (SecurePackage *)phCredential->dwUpper;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.InitializeSecurityContextW)
//             {
//                 UNICODE_STRING target;
//
//                 RtlCreateUnicodeStringFromAsciiz(&target, pszTargetName);
//                 ret = package->provider->fnTableW.InitializeSecurityContextW(
//                  phCredential, phContext, target.Buffer, fContextReq, Reserved1,
//                  TargetDataRep, pInput, Reserved2, phNewContext, pOutput,
//                  pfContextAttr, ptsExpiry);
//                 RtlFreeUnicodeString(&target);
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_InitializeSecurityContextW(
//  PCredHandle phCredential, PCtxtHandle phContext,
//  SEC_WCHAR *pszTargetName, ULONG fContextReq,
//  ULONG Reserved1, ULONG TargetDataRep, PSecBufferDesc pInput,
//  ULONG Reserved2, PCtxtHandle phNewContext, PSecBufferDesc pOutput,
//  ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p %s %d %d %d %p %d %p %p %p %p\n", phCredential, phContext,
//      debugstr_w(pszTargetName), fContextReq, Reserved1, TargetDataRep, pInput,
//      Reserved1, phNewContext, pOutput, pfContextAttr, ptsExpiry);
//     if (phCredential)
//     {
//         SecurePackage *package = (SecurePackage *)phCredential->dwUpper;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableA.InitializeSecurityContextA)
//             {
//                 PSTR target = SECUR32_AllocMultiByteFromWide(pszTargetName);
//
//                 ret = package->provider->fnTableA.InitializeSecurityContextA(
//                  phCredential, phContext, target, fContextReq, Reserved1,
//                  TargetDataRep, pInput, Reserved2, phNewContext, pOutput,
//                  pfContextAttr, ptsExpiry);
//                 HeapFree(GetProcessHeap(), 0, target);
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_AddCredentialsA(PCredHandle hCredentials,
//  SEC_CHAR *pszPrincipal, SEC_CHAR *pszPackage, ULONG fCredentialUse,
//  void *pAuthData, SEC_GET_KEY_FN pGetKeyFn, void *pvGetKeyArgument,
//  PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %s %s %d %p %p %p %p\n", hCredentials, debugstr_a(pszPrincipal),
//      debugstr_a(pszPackage), fCredentialUse, pAuthData, pGetKeyFn,
//      pvGetKeyArgument, ptsExpiry);
//     if (hCredentials)
//     {
//         SecurePackage *package = (SecurePackage *)hCredentials->dwUpper;
//         PCredHandle cred = (PCtxtHandle)hCredentials->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.AddCredentialsW)
//             {
//                 UNICODE_STRING szPrincipal, szPackage;
//
//                 RtlCreateUnicodeStringFromAsciiz(&szPrincipal, pszPrincipal);
//                 RtlCreateUnicodeStringFromAsciiz(&szPackage, pszPackage);
//                 ret = package->provider->fnTableW.AddCredentialsW(
//                  cred, szPrincipal.Buffer, szPackage.Buffer, fCredentialUse,
//                  pAuthData, pGetKeyFn, pvGetKeyArgument, ptsExpiry);
//                 RtlFreeUnicodeString(&szPrincipal);
//                 RtlFreeUnicodeString(&szPackage);
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_AddCredentialsW(PCredHandle hCredentials,
//  SEC_WCHAR *pszPrincipal, SEC_WCHAR *pszPackage, ULONG fCredentialUse,
//  void *pAuthData, SEC_GET_KEY_FN pGetKeyFn, void *pvGetKeyArgument,
//  PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %s %s %d %p %p %p %p\n", hCredentials, debugstr_w(pszPrincipal),
//      debugstr_w(pszPackage), fCredentialUse, pAuthData, pGetKeyFn,
//      pvGetKeyArgument, ptsExpiry);
//     if (hCredentials)
//     {
//         SecurePackage *package = (SecurePackage *)hCredentials->dwUpper;
//         PCredHandle cred = (PCtxtHandle)hCredentials->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableA.AddCredentialsA)
//             {
//                 PSTR szPrincipal = SECUR32_AllocMultiByteFromWide(pszPrincipal);
//                 PSTR szPackage = SECUR32_AllocMultiByteFromWide(pszPackage);
//
//                 ret = package->provider->fnTableA.AddCredentialsA(
//                  cred, szPrincipal, szPackage, fCredentialUse, pAuthData,
//                  pGetKeyFn, pvGetKeyArgument, ptsExpiry);
//                 HeapFree(GetProcessHeap(), 0, szPrincipal);
//                 HeapFree(GetProcessHeap(), 0, szPackage);
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// static PSecPkgInfoA _copyPackageInfoFlatWToA(const SecPkgInfoW *infoW)
// {
//     PSecPkgInfoA ret;
//
//     if (infoW)
//     {
//         size_t bytesNeeded = sizeof(SecPkgInfoA);
//         int nameLen = 0, commentLen = 0;
//
//         if (infoW->Name)
//         {
//             nameLen = WideCharToMultiByte(CP_ACP, 0, infoW->Name, -1,
//              NULL, 0, NULL, NULL);
//             bytesNeeded += nameLen;
//         }
//         if (infoW->Comment)
//         {
//             commentLen = WideCharToMultiByte(CP_ACP, 0, infoW->Comment, -1,
//              NULL, 0, NULL, NULL);
//             bytesNeeded += commentLen;
//         }
//         ret = HeapAlloc(GetProcessHeap(), 0, bytesNeeded as _);
//         if (ret)
//         {
//             PSTR nextString = (PSTR)((PBYTE)ret + sizeof(SecPkgInfoA));
//
//             memcpy(ret, infoW, sizeof(SecPkgInfoA));
//             if (infoW->Name)
//             {
//                 ret->Name = nextString;
//                 WideCharToMultiByte(CP_ACP, 0, infoW->Name, -1, nextString,
//                  nameLen, NULL, NULL);
//                 nextString += nameLen;
//             }
//             else
//                 ret->Name = NULL;
//             if (infoW->Comment)
//             {
//                 ret->Comment = nextString;
//                 WideCharToMultiByte(CP_ACP, 0, infoW->Comment, -1, nextString,
//                  nameLen, NULL, NULL);
//             }
//             else
//                 ret->Comment = NULL;
//         }
//     }
//     else
//         ret = NULL;
//     return ret;
// }
//
// static SECURITY_STATUS thunk_ContextAttributesWToA(SecurePackage *package,
//  ULONG ulAttribute, void *pBuffer)
// {
//     SECURITY_STATUS ret = SEC_E_OK;
//
//     if (package && pBuffer)
//     {
//         switch (ulAttribute)
//         {
//             case SECPKG_ATTR_NAMES:
//             {
//                 PSecPkgContext_NamesW names = (PSecPkgContext_NamesW)pBuffer;
//                 SEC_WCHAR *oldUser = names->sUserName;
//
//                 if (oldUser)
//                 {
//                     names->sUserName =
//                      (PWSTR)SECUR32_AllocMultiByteFromWide(oldUser);
//                     package->provider->fnTableW.FreeContextBuffer(oldUser);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_AUTHORITY:
//             {
//                 PSecPkgContext_AuthorityW names =
//                  (PSecPkgContext_AuthorityW)pBuffer;
//                 SEC_WCHAR *oldAuth = names->sAuthorityName;
//
//                 if (oldAuth)
//                 {
//                     names->sAuthorityName =
//                      (PWSTR)SECUR32_AllocMultiByteFromWide(oldAuth);
//                     package->provider->fnTableW.FreeContextBuffer(oldAuth);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_KEY_INFO:
//             {
//                 PSecPkgContext_KeyInfoW info = (PSecPkgContext_KeyInfoW)pBuffer;
//                 SEC_WCHAR *oldSigAlgName = info->sSignatureAlgorithmName;
//                 SEC_WCHAR *oldEncAlgName = info->sEncryptAlgorithmName;
//
//                 if (oldSigAlgName)
//                 {
//                     info->sSignatureAlgorithmName =
//                      (PWSTR)SECUR32_AllocMultiByteFromWide(oldSigAlgName);
//                     package->provider->fnTableW.FreeContextBuffer(
//                      oldSigAlgName);
//                 }
//                 if (oldEncAlgName)
//                 {
//                     info->sEncryptAlgorithmName =
//                      (PWSTR)SECUR32_AllocMultiByteFromWide(oldEncAlgName);
//                     package->provider->fnTableW.FreeContextBuffer(
//                      oldEncAlgName);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_PACKAGE_INFO:
//             {
//                 PSecPkgContext_PackageInfoW info =
//                  (PSecPkgContext_PackageInfoW)pBuffer;
//                 PSecPkgInfoW oldPkgInfo = info->PackageInfo;
//
//                 if (oldPkgInfo)
//                 {
//                     info->PackageInfo = (PSecPkgInfoW)
//                      _copyPackageInfoFlatWToA(oldPkgInfo);
//                     package->provider->fnTableW.FreeContextBuffer(oldPkgInfo);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_NEGOTIATION_INFO:
//             {
//                 PSecPkgContext_NegotiationInfoW info =
//                  (PSecPkgContext_NegotiationInfoW)pBuffer;
//                 PSecPkgInfoW oldPkgInfo = info->PackageInfo;
//
//                 if (oldPkgInfo)
//                 {
//                     info->PackageInfo = (PSecPkgInfoW)
//                      _copyPackageInfoFlatWToA(oldPkgInfo);
//                     package->provider->fnTableW.FreeContextBuffer(oldPkgInfo);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_NATIVE_NAMES:
//             {
//                 PSecPkgContext_NativeNamesW names =
//                  (PSecPkgContext_NativeNamesW)pBuffer;
//                 PWSTR oldClient = names->sClientName;
//                 PWSTR oldServer = names->sServerName;
//
//                 if (oldClient)
//                 {
//                     names->sClientName = (PWSTR)SECUR32_AllocMultiByteFromWide(
//                      oldClient);
//                     package->provider->fnTableW.FreeContextBuffer(oldClient);
//                 }
//                 if (oldServer)
//                 {
//                     names->sServerName = (PWSTR)SECUR32_AllocMultiByteFromWide(
//                      oldServer);
//                     package->provider->fnTableW.FreeContextBuffer(oldServer);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_CREDENTIAL_NAME:
//             {
//                 PSecPkgContext_CredentialNameW name =
//                  (PSecPkgContext_CredentialNameW)pBuffer;
//                 PWSTR oldCred = name->sCredentialName;
//
//                 if (oldCred)
//                 {
//                     name->sCredentialName =
//                      (PWSTR)SECUR32_AllocMultiByteFromWide(oldCred);
//                     package->provider->fnTableW.FreeContextBuffer(oldCred);
//                 }
//                 break;
//             }
//             /* no thunking needed: */
//             case SECPKG_ATTR_ACCESS_TOKEN:
//             case SECPKG_ATTR_DCE_INFO:
//             case SECPKG_ATTR_FLAGS:
//             case SECPKG_ATTR_LIFESPAN:
//             case SECPKG_ATTR_PASSWORD_EXPIRY:
//             case SECPKG_ATTR_SESSION_KEY:
//             case SECPKG_ATTR_SIZES:
//             case SECPKG_ATTR_STREAM_SIZES:
//             case SECPKG_ATTR_TARGET_INFORMATION:
//                 break;
//             default:
//                 WARN("attribute type %d unknown\n", ulAttribute);
//                 ret = SEC_E_INTERNAL_ERROR;
//         }
//     }
//     else
//         ret = SEC_E_INVALID_TOKEN;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_QueryContextAttributesA(PCtxtHandle phContext,
//  ULONG ulAttribute, void *pBuffer)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p\n", phContext, ulAttribute, pBuffer);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.QueryContextAttributesW)
//             {
//                 ret = package->provider->fnTableW.QueryContextAttributesW(
//                  ctxt, ulAttribute, pBuffer);
//                 if (ret == SEC_E_OK)
//                     ret = thunk_ContextAttributesWToA(package, ulAttribute,
//                      pBuffer);
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// static PSecPkgInfoW _copyPackageInfoFlatAToW(const SecPkgInfoA *infoA)
// {
//     PSecPkgInfoW ret;
//
//     if (infoA)
//     {
//         size_t bytesNeeded = sizeof(SecPkgInfoW);
//         int nameLen = 0, commentLen = 0;
//
//         if (infoA->Name)
//         {
//             nameLen = MultiByteToWideChar(CP_ACP, 0, infoA->Name, -1,
//              NULL, 0);
//             bytesNeeded += nameLen * sizeof(WCHAR);
//         }
//         if (infoA->Comment)
//         {
//             commentLen = MultiByteToWideChar(CP_ACP, 0, infoA->Comment, -1,
//              NULL, 0);
//             bytesNeeded += commentLen * sizeof(WCHAR);
//         }
//         ret = HeapAlloc(GetProcessHeap(), 0, bytesNeeded as _);
//         if (ret)
//         {
//             PWSTR nextString = (PWSTR)((PBYTE)ret + sizeof(SecPkgInfoW));
//
//             memcpy(ret, infoA, sizeof(SecPkgInfoA));
//             if (infoA->Name)
//             {
//                 ret->Name = nextString;
//                 MultiByteToWideChar(CP_ACP, 0, infoA->Name, -1, nextString,
//                  nameLen);
//                 nextString += nameLen;
//             }
//             else
//                 ret->Name = NULL;
//             if (infoA->Comment)
//             {
//                 ret->Comment = nextString;
//                 MultiByteToWideChar(CP_ACP, 0, infoA->Comment, -1, nextString,
//                  commentLen);
//             }
//             else
//                 ret->Comment = NULL;
//         }
//     }
//     else
//         ret = NULL;
//     return ret;
// }
//
// static SECURITY_STATUS thunk_ContextAttributesAToW(SecurePackage *package,
//  ULONG ulAttribute, void *pBuffer)
// {
//     SECURITY_STATUS ret = SEC_E_OK;
//
//     if (package && pBuffer)
//     {
//         switch (ulAttribute)
//         {
//             case SECPKG_ATTR_NAMES:
//             {
//                 PSecPkgContext_NamesA names = (PSecPkgContext_NamesA)pBuffer;
//                 SEC_CHAR *oldUser = names->sUserName;
//
//                 if (oldUser)
//                 {
//                     names->sUserName =
//                      (PSTR)SECUR32_AllocWideFromMultiByte(oldUser);
//                     package->provider->fnTableW.FreeContextBuffer(oldUser);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_AUTHORITY:
//             {
//                 PSecPkgContext_AuthorityA names =
//                  (PSecPkgContext_AuthorityA)pBuffer;
//                 SEC_CHAR *oldAuth = names->sAuthorityName;
//
//                 if (oldAuth)
//                 {
//                     names->sAuthorityName =
//                      (PSTR)SECUR32_AllocWideFromMultiByte(oldAuth);
//                     package->provider->fnTableW.FreeContextBuffer(oldAuth);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_KEY_INFO:
//             {
//                 PSecPkgContext_KeyInfoA info = (PSecPkgContext_KeyInfoA)pBuffer;
//                 SEC_CHAR *oldSigAlgName = info->sSignatureAlgorithmName;
//                 SEC_CHAR *oldEncAlgName = info->sEncryptAlgorithmName;
//
//                 if (oldSigAlgName)
//                 {
//                     info->sSignatureAlgorithmName =
//                      (PSTR)SECUR32_AllocWideFromMultiByte(oldSigAlgName);
//                     package->provider->fnTableW.FreeContextBuffer(
//                      oldSigAlgName);
//                 }
//                 if (oldEncAlgName)
//                 {
//                     info->sEncryptAlgorithmName =
//                      (PSTR)SECUR32_AllocWideFromMultiByte(
//                      oldEncAlgName);
//                     package->provider->fnTableW.FreeContextBuffer(
//                      oldEncAlgName);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_PACKAGE_INFO:
//             {
//                 PSecPkgContext_PackageInfoA info =
//                  (PSecPkgContext_PackageInfoA)pBuffer;
//                 PSecPkgInfoA oldPkgInfo = info->PackageInfo;
//
//                 if (oldPkgInfo)
//                 {
//                     info->PackageInfo = (PSecPkgInfoA)
//                      _copyPackageInfoFlatAToW(oldPkgInfo);
//                     package->provider->fnTableW.FreeContextBuffer(oldPkgInfo);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_NEGOTIATION_INFO:
//             {
//                 PSecPkgContext_NegotiationInfoA info =
//                  (PSecPkgContext_NegotiationInfoA)pBuffer;
//                 PSecPkgInfoA oldPkgInfo = info->PackageInfo;
//
//                 if (oldPkgInfo)
//                 {
//                     info->PackageInfo = (PSecPkgInfoA)
//                      _copyPackageInfoFlatAToW(oldPkgInfo);
//                     package->provider->fnTableW.FreeContextBuffer(oldPkgInfo);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_NATIVE_NAMES:
//             {
//                 PSecPkgContext_NativeNamesA names =
//                  (PSecPkgContext_NativeNamesA)pBuffer;
//                 PSTR oldClient = names->sClientName;
//                 PSTR oldServer = names->sServerName;
//
//                 if (oldClient)
//                 {
//                     names->sClientName = (PSTR)SECUR32_AllocWideFromMultiByte(
//                      oldClient);
//                     package->provider->fnTableW.FreeContextBuffer(oldClient);
//                 }
//                 if (oldServer)
//                 {
//                     names->sServerName = (PSTR)SECUR32_AllocWideFromMultiByte(
//                      oldServer);
//                     package->provider->fnTableW.FreeContextBuffer(oldServer);
//                 }
//                 break;
//             }
//             case SECPKG_ATTR_CREDENTIAL_NAME:
//             {
//                 PSecPkgContext_CredentialNameA name =
//                  (PSecPkgContext_CredentialNameA)pBuffer;
//                 PSTR oldCred = name->sCredentialName;
//
//                 if (oldCred)
//                 {
//                     name->sCredentialName =
//                      (PSTR)SECUR32_AllocWideFromMultiByte(oldCred);
//                     package->provider->fnTableW.FreeContextBuffer(oldCred);
//                 }
//                 break;
//             }
//             /* no thunking needed: */
//             case SECPKG_ATTR_ACCESS_TOKEN:
//             case SECPKG_ATTR_DCE_INFO:
//             case SECPKG_ATTR_FLAGS:
//             case SECPKG_ATTR_LIFESPAN:
//             case SECPKG_ATTR_PASSWORD_EXPIRY:
//             case SECPKG_ATTR_SESSION_KEY:
//             case SECPKG_ATTR_SIZES:
//             case SECPKG_ATTR_STREAM_SIZES:
//             case SECPKG_ATTR_TARGET_INFORMATION:
//                 break;
//             default:
//                 WARN("attribute type %d unknown\n", ulAttribute);
//                 ret = SEC_E_INTERNAL_ERROR;
//         }
//     }
//     else
//         ret = SEC_E_INVALID_TOKEN;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_QueryContextAttributesW(PCtxtHandle phContext,
//  ULONG ulAttribute, void *pBuffer)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p\n", phContext, ulAttribute, pBuffer);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableA.QueryContextAttributesA)
//             {
//                 ret = package->provider->fnTableA.QueryContextAttributesA(
//                  ctxt, ulAttribute, pBuffer);
//                 if (ret == SEC_E_OK)
//                     ret = thunk_ContextAttributesAToW(package, ulAttribute,
//                      pBuffer);
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_SetContextAttributesA(PCtxtHandle phContext,
//  ULONG ulAttribute, void *pBuffer, ULONG cbBuffer)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p %d\n", phContext, ulAttribute, pBuffer, cbBuffer);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider && pBuffer && cbBuffer)
//         {
//             if (package->provider->fnTableW.SetContextAttributesW)
//             {
//                 /* TODO: gotta validate size too! */
//                 ret = thunk_ContextAttributesWToA(package, ulAttribute,
//                  pBuffer);
//                 if (ret == SEC_E_OK)
//                     ret = package->provider->fnTableW.SetContextAttributesW(
//                      ctxt, ulAttribute, pBuffer, cbBuffer);
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_SetContextAttributesW(PCtxtHandle phContext,
//  ULONG ulAttribute, void *pBuffer, ULONG cbBuffer)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p %d\n", phContext, ulAttribute, pBuffer, cbBuffer);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider && pBuffer && cbBuffer)
//         {
//             if (package->provider->fnTableA.SetContextAttributesA)
//             {
//                 /* TODO: gotta validate size too! */
//                 ret = thunk_ContextAttributesAToW(package, ulAttribute,
//                  pBuffer);
//                 if (ret == SEC_E_OK)
//                     ret = package->provider->fnTableA.SetContextAttributesA(
//                      ctxt, ulAttribute, pBuffer, cbBuffer);
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_INVALID_HANDLE;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_ImportSecurityContextA(
//  SEC_CHAR *pszPackage, PSecBuffer pPackedContext, void *Token,
//  PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//     UNICODE_STRING package;
//
//     TRACE("%s %p %p %p\n", debugstr_a(pszPackage), pPackedContext, Token,
//      phContext);
//     RtlCreateUnicodeStringFromAsciiz(&package, pszPackage);
//     ret = ImportSecurityContextW(package.Buffer, pPackedContext, Token,
//      phContext);
//     RtlFreeUnicodeString(&package);
//     return ret;
// }
//
// SECURITY_STATUS SEC_ENTRY thunk_ImportSecurityContextW(
//  SEC_WCHAR *pszPackage, PSecBuffer pPackedContext, void *Token,
//  PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//     PSTR package = SECUR32_AllocMultiByteFromWide(pszPackage);
//
//     TRACE("%s %p %p %p\n", debugstr_w(pszPackage), pPackedContext, Token,
//      phContext);
//     ret = ImportSecurityContextA(package, pPackedContext, Token, phContext);
//     HeapFree(GetProcessHeap(), 0, package);
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
