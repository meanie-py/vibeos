use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(secur32) - debug channel macro, no Rust equivalent

unsafe fn SECUR32_makeSecHandle(
    phSec: PSecHandle,
    package: *mut SecurePackage,
    realHandle: PSecHandle,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%p %p %p\n\0".as_ptr() as PSTR,
        phSec,
        package,
        realHandle,
    );

    if !phSec.is_null() && !package.is_null() && !realHandle.is_null() {
        let newSec = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<SecHandle>() as _,
        ) as PSecHandle;

        if !newSec.is_null() {
            *newSec = *realHandle;
            (*phSec).dwUpper = package as ULONG_PTR;
            (*phSec).dwLower = newSec as ULONG_PTR;
            ret = SEC_E_OK;
        } else {
            ret = SEC_E_INSUFFICIENT_MEMORY;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn AcquireCredentialsHandleA(
    pszPrincipal: *mut SEC_CHAR,
    pszPackage: *mut SEC_CHAR,
    fCredentialsUse: ULONG,
    pvLogonID: PLUID,
    pAuthData: PVOID,
    pGetKeyFn: SEC_GET_KEY_FN,
    pvGetKeyArgument: PVOID,
    phCredential: PCredHandle,
    ptsExpiry: PTimeStamp,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%s %s %d %p %p %p %p %p %p\n\0".as_ptr() as PSTR,
        debugstr_a(pszPrincipal),
        debugstr_a(pszPackage),
        fCredentialsUse,
        pvLogonID as _,
        pAuthData,
        pGetKeyFn,
        pvGetKeyArgument,
        phCredential,
        ptsExpiry,
    );

    if !pszPackage.is_null() {
        let package: *mut SecurePackage = SECUR32_findPackageA(pszPackage);

        if !package.is_null() && !(*package).provider.is_null() {
            if (*(*package).provider).fnTableA.AcquireCredentialsHandleA.is_some() {
                let mut myCred: CredHandle = core::mem::zeroed();

                ret = ((*(*package).provider).fnTableA.AcquireCredentialsHandleA.unwrap())(
                    pszPrincipal,
                    pszPackage,
                    fCredentialsUse,
                    pvLogonID as _,
                    pAuthData,
                    pGetKeyFn,
                    pvGetKeyArgument,
                    &mut myCred,
                    ptsExpiry,
                );
                let mut ret = ret;
                if ret == SEC_E_OK {
                    ret = SECUR32_makeSecHandle(phCredential, package, &mut myCred);
                    if ret != SEC_E_OK {
                        ((*(*package).provider).fnTableW.FreeCredentialsHandle.unwrap())(
                            &mut myCred,
                        );
                    }
                }
                return ret;
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_SECPKG_NOT_FOUND;
        }
    } else {
        ret = SEC_E_SECPKG_NOT_FOUND;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn AcquireCredentialsHandleW(
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

    TRACE!(
        b"%s %s %d %p %p %p %p %p %p\n\0".as_ptr() as PSTR,
        debugstr_w(pszPrincipal),
        debugstr_w(pszPackage),
        fCredentialsUse,
        pvLogonID as _,
        pAuthData,
        pGetKeyFn,
        pvGetKeyArgument,
        phCredential,
        ptsExpiry,
    );

    if !pszPackage.is_null() {
        let package: *mut SecurePackage = SECUR32_findPackageW(pszPackage);

        if !package.is_null() && !(*package).provider.is_null() {
            if (*(*package).provider).fnTableW.AcquireCredentialsHandleW.is_some() {
                let mut myCred: CredHandle = core::mem::zeroed();

                ret = ((*(*package).provider).fnTableW.AcquireCredentialsHandleW.unwrap())(
                    pszPrincipal,
                    pszPackage,
                    fCredentialsUse,
                    pvLogonID as _,
                    pAuthData,
                    pGetKeyFn,
                    pvGetKeyArgument,
                    core::ptr::addr_of_mut!(myCred),
                    ptsExpiry,
                );
                let mut ret = ret;
                if ret == SEC_E_OK {
                    ret = SECUR32_makeSecHandle(phCredential, package, core::ptr::addr_of_mut!(myCred));
                    if ret != SEC_E_OK {
                        ((*(*package).provider).fnTableW.FreeCredentialsHandle.unwrap())(
                            core::ptr::addr_of_mut!(myCred),
                        );
                    }
                }
                return ret;
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION;
            }
        } else {
            ret = SEC_E_SECPKG_NOT_FOUND;
        }
    } else {
        ret = SEC_E_SECPKG_NOT_FOUND;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn FreeCredentialsHandle(
    phCredential: PCredHandle,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p\n\0".as_ptr() as PSTR, phCredential);
    if !phCredential.is_null() {
        let package = (*phCredential).dwUpper as *mut SecurePackage;
        let cred = (*phCredential).dwLower as PCredHandle;

        if !package.is_null()
            && !(*package).provider.is_null()
            && (*(*package).provider).fnTableW.FreeCredentialsHandle.is_some()
        {
            ret = ((*(*package).provider).fnTableW.FreeCredentialsHandle.unwrap())(cred);
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
        HeapFree(GetProcessHeap(), 0, cred as *mut core::ffi::c_void);
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn QueryCredentialsAttributesA(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phCredential.is_null() {
        let package = (*phCredential).dwUpper as *mut SecurePackage;
        let cred = (*phCredential).dwLower as PCredHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableA.QueryCredentialsAttributesA {
                ret = func(cred, ulAttribute, pBuffer);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn QueryCredentialsAttributesW(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phCredential.is_null() {
        let package = (*phCredential).dwUpper as *mut SecurePackage;
        let cred = (*phCredential).dwLower as PCredHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.QueryCredentialsAttributesW {
                ret = func(cred, ulAttribute, pBuffer);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn InitializeSecurityContextA(
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
    let mut package: *mut SecurePackage = core::ptr::null_mut();
    let mut cred: PCredHandle = core::ptr::null_mut();
    let mut ctxt: PCredHandle = core::ptr::null_mut();

    TRACE!(
        b"%p %p %s 0x%08x %d %d %p %d %p %p %p %p\n\0".as_ptr() as PSTR,
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

    if !phContext.is_null() {
        package = (*phContext).dwUpper as *mut SecurePackage;
        ctxt = (*phContext).dwLower as PCtxtHandle;
    }
    if !phCredential.is_null() {
        package = (*phCredential).dwUpper as *mut SecurePackage;
        cred = (*phCredential).dwLower as PCredHandle;
    }

    if !package.is_null() && !(*package).provider.is_null() {
        if (*(*package).provider).fnTableA.InitializeSecurityContextA.is_some() {
            let mut myCtxt: CtxtHandle = core::mem::zeroed();

            if !phContext.is_null() {
                let realCtxt = (*phContext).dwLower as PCtxtHandle;
                myCtxt.dwUpper = (*realCtxt).dwUpper;
                myCtxt.dwLower = (*realCtxt).dwLower;
            }

            ret = ((*(*package).provider).fnTableA.InitializeSecurityContextA.unwrap())(
                cred,
                ctxt,
                pszTargetName,
                fContextReq,
                Reserved1,
                TargetDataRep,
                pInput,
                Reserved2,
                if !phNewContext.is_null() {
                    core::ptr::addr_of_mut!(myCtxt)
                } else {
                    core::ptr::null_mut()
                },
                pOutput,
                pfContextAttr,
                ptsExpiry,
            );
            if (ret == SEC_E_OK || ret == SEC_I_CONTINUE_NEEDED)
                && !phNewContext.is_null()
                && phNewContext != phContext
            {
                let ret2 = SECUR32_makeSecHandle(phNewContext, package, core::ptr::addr_of_mut!(myCtxt));
                if ret2 != SEC_E_OK {
                    ((*(*package).provider).fnTableA.DeleteSecurityContext.unwrap())(
                        core::ptr::addr_of_mut!(myCtxt),
                    );
                }
            }
        } else {
            ret = SEC_E_UNSUPPORTED_FUNCTION;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn InitializeSecurityContextW(
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
    let mut package: *mut SecurePackage = core::ptr::null_mut();
    let mut cred: PCredHandle = core::ptr::null_mut();
    let mut ctxt: PCredHandle = core::ptr::null_mut();

    if !phContext.is_null() {
        package = (*phContext).dwUpper as *mut SecurePackage;
        ctxt = (*phContext).dwLower as PCtxtHandle;
    }
    if !phCredential.is_null() {
        package = (*phCredential).dwUpper as *mut SecurePackage;
        cred = (*phCredential).dwLower as PCredHandle;
    }

    if !package.is_null() && !(*package).provider.is_null() {
        if let Some(initFn) = (*(*package).provider).fnTableW.InitializeSecurityContextW {
            let mut myCtxt: CtxtHandle = core::mem::zeroed();

            if !phContext.is_null() {
                let realCtxt = (*phContext).dwLower as PCtxtHandle;
                myCtxt.dwUpper = (*realCtxt).dwUpper;
                myCtxt.dwLower = (*realCtxt).dwLower;
            }

            ret = initFn(
                cred,
                ctxt,
                pszTargetName,
                fContextReq,
                Reserved1,
                TargetDataRep,
                pInput,
                Reserved2,
                if !phNewContext.is_null() {
                    &mut myCtxt
                } else {
                    core::ptr::null_mut()
                },
                pOutput,
                pfContextAttr,
                ptsExpiry,
            );
            if (ret == SEC_E_OK || ret == SEC_I_CONTINUE_NEEDED)
                && !phNewContext.is_null()
                && phNewContext != phContext
            {
                let ret2 = SECUR32_makeSecHandle(phNewContext, package, &mut myCtxt);
                if ret2 != SEC_E_OK {
                    if let Some(delFn) = (*(*package).provider).fnTableW.DeleteSecurityContext {
                        delFn(&mut myCtxt);
                    }
                }
            }
        } else {
            ret = SEC_E_UNSUPPORTED_FUNCTION;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn AcceptSecurityContext(
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
    let ret: SECURITY_STATUS;

    if !phCredential.is_null() {
        let package = (*phCredential).dwUpper as *mut SecurePackage;
        let cred = (*phCredential).dwLower as PCredHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(accept_fn) = (*(*package).provider).fnTableW.AcceptSecurityContext {
                let mut myCtxt: CtxtHandle = core::mem::zeroed();

                if !phContext.is_null() {
                    let realCtxt = (*phContext).dwLower as PCtxtHandle;
                    myCtxt.dwUpper = (*realCtxt).dwUpper;
                    myCtxt.dwLower = (*realCtxt).dwLower;
                }

                ret = accept_fn(
                    cred,
                    if !phContext.is_null() {
                        &mut myCtxt
                    } else {
                        core::ptr::null_mut()
                    },
                    pInput,
                    fContextReq,
                    TargetDataRep,
                    &mut myCtxt,
                    pOutput,
                    pfContextAttr,
                    ptsExpiry,
                );
                if ret == SEC_E_OK || ret == SEC_I_CONTINUE_NEEDED {
                    let ret2 = SECUR32_makeSecHandle(phNewContext, package, &mut myCtxt);
                    if ret2 != SEC_E_OK {
                        if let Some(delete_fn) =
                            (*(*package).provider).fnTableW.DeleteSecurityContext
                        {
                            delete_fn(&mut myCtxt);
                        }
                    }
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
pub unsafe extern "system" fn CompleteAuthToken(
    phContext: PCtxtHandle,
    pToken: PSecBufferDesc,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p %p\n\0".as_ptr() as PSTR, phContext, pToken);
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.CompleteAuthToken {
                ret = func(ctxt, pToken);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn DeleteSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p\n\0".as_ptr() as PSTR, phContext);
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null()
            && !(*package).provider.is_null()
            && (*(*package).provider).fnTableW.DeleteSecurityContext.is_some()
        {
            ret = ((*(*package).provider).fnTableW.DeleteSecurityContext.unwrap())(ctxt);
        } else {
            ret = SEC_E_INVALID_HANDLE;
        }
        HeapFree(GetProcessHeap(), 0, ctxt as *mut core::ffi::c_void);
    } else {
        ret = SEC_E_INVALID_HANDLE;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ApplyControlToken(
    phContext: PCtxtHandle,
    pInput: PSecBufferDesc,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p %p\n\0".as_ptr() as *const i8, phContext, pInput);
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.ApplyControlToken {
                ret = func(ctxt, pInput);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn QueryContextAttributesA(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%p %d %p\n\0".as_ptr() as *const i8,
        phContext,
        ulAttribute,
        pBuffer,
    );
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if (*(*package).provider).fnTableA.QueryContextAttributesA.is_some() {
                ret = ((*(*package).provider).fnTableA.QueryContextAttributesA.unwrap())(
                    ctxt, ulAttribute, pBuffer,
                );
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn QueryContextAttributesW(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if (*(*package).provider).fnTableW.QueryContextAttributesW.is_some() {
                ret = ((*(*package).provider).fnTableW.QueryContextAttributesW.unwrap())(
                    ctxt,
                    ulAttribute,
                    pBuffer,
                );
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
pub unsafe extern "system" fn ImpersonateSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p\n\0".as_ptr() as PSTR, phContext);
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.ImpersonateSecurityContext {
                ret = func(ctxt);
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
pub unsafe extern "system" fn RevertSecurityContext(phContext: PCtxtHandle) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p\n\0".as_ptr() as PSTR, phContext);
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.RevertSecurityContext {
                ret = func(ctxt);
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
pub unsafe extern "system" fn MakeSignature(
    phContext: PCtxtHandle,
    fQOP: ULONG,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%p %d %p %d\n\0".as_ptr() as PSTR,
        phContext,
        fQOP,
        pMessage,
        MessageSeqNo,
    );
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.MakeSignature {
                ret = func(ctxt, fQOP, pMessage, MessageSeqNo);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn VerifySignature(
    phContext: PCtxtHandle,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
    pfQOP: PULONG,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%p %p %d %p\n\0".as_ptr() as PSTR,
        phContext,
        pMessage,
        MessageSeqNo,
        pfQOP,
    );
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.VerifySignature {
                ret = func(ctxt, pMessage, MessageSeqNo, pfQOP);
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
pub unsafe extern "system" fn QuerySecurityPackageInfoA(
    pszPackageName: *mut SEC_CHAR,
    ppPackageInfo: *mut PSecPkgInfoA,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%s %p\n\0".as_ptr() as *const i8,
        debugstr_a(pszPackageName),
        ppPackageInfo,
    );

    if !pszPackageName.is_null() {
        let package: *mut SecurePackage = SECUR32_findPackageA(pszPackageName);

        if !package.is_null() {
            let mut bytesNeeded: usize = core::mem::size_of::<SecPkgInfoA>();
            let mut nameLen: i32 = 0;
            let mut commentLen: i32 = 0;

            if !(*package).infoW.Name.is_null() {
                nameLen = WideCharToMultiByte(
                    CP_ACP,
                    0,
                    (*package).infoW.Name,
                    -1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                bytesNeeded += nameLen as usize;
            }
            if !(*package).infoW.Comment.is_null() {
                commentLen = WideCharToMultiByte(
                    CP_ACP,
                    0,
                    (*package).infoW.Comment,
                    -1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                bytesNeeded += commentLen as usize;
            }
            *ppPackageInfo = HeapAlloc(GetProcessHeap(), 0, bytesNeeded as _) as PSecPkgInfoA;
            if !(*ppPackageInfo).is_null() {
                let mut nextString: PSTR =
                    (*ppPackageInfo as PBYTE).add(core::mem::size_of::<SecPkgInfoA>()) as PSTR;

                memcpy(
                    *ppPackageInfo as *mut core::ffi::c_void,
                    core::ptr::addr_of!((*package).infoW) as *const core::ffi::c_void,
                    core::mem::size_of_val(&(*package).infoW),
                );
                if !(*package).infoW.Name.is_null() {
                    (**ppPackageInfo).Name = nextString;
                    nextString = nextString.offset(WideCharToMultiByte(
                        CP_ACP,
                        0,
                        (*package).infoW.Name,
                        -1,
                        nextString,
                        nameLen,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) as isize);
                } else {
                    (**ppPackageInfo).Name = core::ptr::null_mut();
                }
                if !(*package).infoW.Comment.is_null() {
                    (**ppPackageInfo).Comment = nextString;
                    nextString = nextString.offset(WideCharToMultiByte(
                        CP_ACP,
                        0,
                        (*package).infoW.Comment,
                        -1,
                        nextString,
                        commentLen,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) as isize);
                } else {
                    (**ppPackageInfo).Comment = core::ptr::null_mut();
                }
                let _ = nextString;
                ret = SEC_E_OK;
            } else {
                ret = SEC_E_INSUFFICIENT_MEMORY;
            }
        } else {
            ret = SEC_E_SECPKG_NOT_FOUND;
        }
    } else {
        ret = SEC_E_SECPKG_NOT_FOUND;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn QuerySecurityPackageInfoW(
    pszPackageName: *mut SEC_WCHAR,
    ppPackageInfo: *mut PSecPkgInfoW,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;
    let package: *mut SecurePackage = SECUR32_findPackageW(pszPackageName);

    TRACE!(b"%s %p\n\0".as_ptr() as *const i8, debugstr_w(pszPackageName), ppPackageInfo);
    if !package.is_null() {
        let mut bytesNeeded: usize = core::mem::size_of::<SecPkgInfoW>();
        let mut nameLen: i32 = 0;
        let mut commentLen: i32 = 0;

        if !(*package).infoW.Name.is_null() {
            nameLen = lstrlenW((*package).infoW.Name) + 1;
            bytesNeeded += nameLen as usize * core::mem::size_of::<WCHAR>();
        }
        if !(*package).infoW.Comment.is_null() {
            commentLen = lstrlenW((*package).infoW.Comment) + 1;
            bytesNeeded += commentLen as usize * core::mem::size_of::<WCHAR>();
        }
        *ppPackageInfo = HeapAlloc(GetProcessHeap(), 0, bytesNeeded as _) as PSecPkgInfoW;
        if !(*ppPackageInfo).is_null() {
            let mut nextString: PWSTR =
                ((*ppPackageInfo as PBYTE).add(core::mem::size_of::<SecPkgInfoW>())) as PWSTR;

            **ppPackageInfo = (*package).infoW;
            if !(*package).infoW.Name.is_null() {
                (**ppPackageInfo).Name = nextString;
                lstrcpynW(nextString, (*package).infoW.Name, nameLen);
                nextString = nextString.add(nameLen as usize);
            } else {
                (**ppPackageInfo).Name = core::ptr::null_mut();
            }
            if !(*package).infoW.Comment.is_null() {
                (**ppPackageInfo).Comment = nextString;
                lstrcpynW(nextString, (*package).infoW.Comment, commentLen);
            } else {
                (**ppPackageInfo).Comment = core::ptr::null_mut();
            }
            ret = SEC_E_OK as SECURITY_STATUS;
        } else {
            ret = SEC_E_INSUFFICIENT_MEMORY as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_SECPKG_NOT_FOUND as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ExportSecurityContext(
    phContext: PCtxtHandle,
    fFlags: ULONG,
    pPackedContext: PSecBuffer,
    pToken: *mut *mut core::ffi::c_void,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if (*(*package).provider).fnTableW.ExportSecurityContext.is_some() {
                ret = ((*(*package).provider).fnTableW.ExportSecurityContext.unwrap())(
                    ctxt,
                    fFlags,
                    pPackedContext,
                    pToken,
                );
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
pub unsafe extern "system" fn ImportSecurityContextA(
    pszPackage: *mut SEC_CHAR,
    pPackedContext: PSecBuffer,
    Token: *mut core::ffi::c_void,
    phContext: PCtxtHandle,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;
    let package: *mut SecurePackage = SECUR32_findPackageA(pszPackage);

    TRACE!(
        b"%s %p %p %p\n\0".as_ptr() as PSTR,
        debugstr_a(pszPackage),
        pPackedContext,
        Token,
        phContext,
    );
    if !package.is_null() && !(*package).provider.is_null() {
        if let Some(ImportSecurityContextA) =
            (*(*package).provider).fnTableA.ImportSecurityContextA
        {
            let mut myCtxt: CtxtHandle = core::mem::zeroed();

            ret = ImportSecurityContextA(pszPackage, pPackedContext, Token, &mut myCtxt);
            if ret == SEC_E_OK {
                let ret2 = SECUR32_makeSecHandle(phContext, package, &mut myCtxt);
                if ret2 != SEC_E_OK {
                    (*(*package).provider).fnTableW.DeleteSecurityContext.unwrap()(&mut myCtxt);
                }
                return ret2;
            }
        } else {
            ret = SEC_E_UNSUPPORTED_FUNCTION;
        }
    } else {
        ret = SEC_E_SECPKG_NOT_FOUND;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ImportSecurityContextW(
    pszPackage: *mut SEC_WCHAR,
    pPackedContext: PSecBuffer,
    Token: *mut core::ffi::c_void,
    phContext: PCtxtHandle,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;
    let package: *mut SecurePackage = SECUR32_findPackageW(pszPackage);

    TRACE!(
        b"%s %p %p %p\n\0".as_ptr() as *const i8,
        debugstr_w(pszPackage),
        pPackedContext,
        Token,
        phContext,
    );

    if !package.is_null() && !(*package).provider.is_null() {
        if (*(*package).provider).fnTableW.ImportSecurityContextW.is_some() {
            let mut myCtxt: CtxtHandle = core::mem::zeroed();

            let mut r = (*(*package).provider).fnTableW.ImportSecurityContextW.unwrap()(
                pszPackage,
                pPackedContext,
                Token,
                &mut myCtxt,
            );
            if r == SEC_E_OK {
                r = SECUR32_makeSecHandle(phContext, package, &mut myCtxt);
                if r != SEC_E_OK {
                    (*(*package).provider).fnTableW.DeleteSecurityContext.unwrap()(&mut myCtxt);
                }
            }
            ret = r;
        } else {
            ret = SEC_E_UNSUPPORTED_FUNCTION;
        }
    } else {
        ret = SEC_E_SECPKG_NOT_FOUND;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn AddCredentialsA(
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

    TRACE!(
        b"%p %s %s %d %p %p %p %p\n\0".as_ptr() as PSTR,
        hCredentials,
        debugstr_a(pszPrincipal),
        debugstr_a(pszPackage),
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
            if let Some(func) = (*(*package).provider).fnTableA.AddCredentialsA {
                ret = func(
                    cred,
                    pszPrincipal,
                    pszPackage,
                    fCredentialUse,
                    pAuthData,
                    pGetKeyFn,
                    pvGetKeyArgument,
                    ptsExpiry,
                );
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn AddCredentialsW(
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

    if !hCredentials.is_null() {
        let package = (*hCredentials).dwUpper as *mut SecurePackage;
        let cred = (*hCredentials).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.AddCredentialsW {
                ret = func(
                    cred,
                    pszPrincipal,
                    pszPackage,
                    fCredentialUse,
                    pAuthData,
                    pGetKeyFn,
                    pvGetKeyArgument,
                    ptsExpiry,
                );
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
pub unsafe extern "system" fn QuerySecurityContextToken(
    phContext: PCtxtHandle,
    phToken: *mut HANDLE,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(b"%p %p\n\0".as_ptr() as *const i8, phContext, phToken);
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.QuerySecurityContextToken {
                ret = func(ctxt, phToken);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EncryptMessage(
    phContext: PCtxtHandle,
    fQOP: ULONG,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.EncryptMessage {
                ret = func(ctxt, fQOP, pMessage, MessageSeqNo);
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
pub unsafe extern "system" fn DecryptMessage(
    phContext: PCtxtHandle,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
    pfQOP: PULONG,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

    TRACE!(
        b"%p %p %d %p\n\0".as_ptr() as *const i8,
        phContext,
        pMessage,
        MessageSeqNo,
        pfQOP,
    );
    if !phContext.is_null() {
        let package = (*phContext).dwUpper as *mut SecurePackage;
        let ctxt = (*phContext).dwLower as PCtxtHandle;

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.DecryptMessage {
                ret = func(ctxt, pMessage, MessageSeqNo, pfQOP);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SetContextAttributesA(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
    cbBuffer: ULONG,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

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

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableA.SetContextAttributesA {
                ret = func(ctxt, ulAttribute, pBuffer, cbBuffer);
            } else {
                ret = SEC_E_UNSUPPORTED_FUNCTION as SECURITY_STATUS;
            }
        } else {
            ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
        }
    } else {
        ret = SEC_E_INVALID_HANDLE as SECURITY_STATUS;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SetContextAttributesW(
    phContext: PCtxtHandle,
    ulAttribute: ULONG,
    pBuffer: *mut core::ffi::c_void,
    cbBuffer: ULONG,
) -> SECURITY_STATUS {
    let ret: SECURITY_STATUS;

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

        if !package.is_null() && !(*package).provider.is_null() {
            if let Some(func) = (*(*package).provider).fnTableW.SetContextAttributesW {
                ret = func(ctxt, ulAttribute, pBuffer, cbBuffer);
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


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright (C) 2004 Juan Lang
//  *
//  * Implements secur32 functions that forward to (wrap) an SSP's implementation.
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
// /* Tries to allocate a new SecHandle, into which it stores package (in
//  * phSec->dwUpper) and a copy of realHandle (allocated with SECUR32_ALLOC,
//  * and stored in phSec->dwLower).  SecHandle is equivalent to both a
//  * CredHandle and a CtxtHandle.
//  */
// static SECURITY_STATUS SECUR32_makeSecHandle(PSecHandle phSec,
//  SecurePackage *package, PSecHandle realHandle)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p %p\n", phSec, package, realHandle);
//
//     if (phSec && package && realHandle)
//     {
//         PSecHandle newSec = HeapAlloc(GetProcessHeap(), 0, sizeof(SecHandle));
//
//         if (newSec)
//         {
//             *newSec = *realHandle;
//             phSec->dwUpper = (ULONG_PTR)package;
//             phSec->dwLower = (ULONG_PTR)newSec;
//             ret = SEC_E_OK;
//         }
//         else
//             ret = SEC_E_INSUFFICIENT_MEMORY;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// /***********************************************************************
//  *		AcquireCredentialsHandleA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI AcquireCredentialsHandleA(
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
//         SecurePackage *package = SECUR32_findPackageA(pszPackage);
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableA.AcquireCredentialsHandleA)
//             {
//                 CredHandle myCred;
//
//                 ret = package->provider->fnTableA.AcquireCredentialsHandleA(
//                  pszPrincipal, pszPackage, fCredentialsUse, pvLogonID,
//                  pAuthData, pGetKeyFn, pvGetKeyArgument, &myCred,
//                  ptsExpiry);
//                 if (ret == SEC_E_OK)
//                 {
//                     ret = SECUR32_makeSecHandle(phCredential, package, &myCred);
//                     if (ret != SEC_E_OK)
//                         package->provider->fnTableW.FreeCredentialsHandle(
//                          &myCred);
//                 }
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_SECPKG_NOT_FOUND;
//     }
//     else
//         ret = SEC_E_SECPKG_NOT_FOUND;
//     return ret;
// }
//
// /***********************************************************************
//  *		AcquireCredentialsHandleW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI AcquireCredentialsHandleW(
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
//         SecurePackage *package = SECUR32_findPackageW(pszPackage);
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.AcquireCredentialsHandleW)
//             {
//                 CredHandle myCred;
//
//                 ret = package->provider->fnTableW.AcquireCredentialsHandleW(
//                  pszPrincipal, pszPackage, fCredentialsUse, pvLogonID,
//                  pAuthData, pGetKeyFn, pvGetKeyArgument, &myCred,
//                  ptsExpiry);
//                 if (ret == SEC_E_OK)
//                 {
//                     ret = SECUR32_makeSecHandle(phCredential, package, &myCred);
//                     if (ret != SEC_E_OK)
//                         package->provider->fnTableW.FreeCredentialsHandle(
//                          &myCred);
//                 }
//             }
//             else
//                 ret = SEC_E_UNSUPPORTED_FUNCTION;
//         }
//         else
//             ret = SEC_E_SECPKG_NOT_FOUND;
//     }
//     else
//         ret = SEC_E_SECPKG_NOT_FOUND;
//     return ret;
// }
//
// /***********************************************************************
//  *		FreeCredentialsHandle (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI FreeCredentialsHandle(
//  PCredHandle phCredential)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p\n", phCredential);
//     if (phCredential)
//     {
//         SecurePackage *package = (SecurePackage *)phCredential->dwUpper;
//         PCredHandle cred = (PCredHandle)phCredential->dwLower;
//
//         if (package && package->provider &&
//          package->provider->fnTableW.FreeCredentialsHandle)
//             ret = package->provider->fnTableW.FreeCredentialsHandle(cred);
//         else
//             ret = SEC_E_INVALID_HANDLE;
//         HeapFree(GetProcessHeap(), 0, cred);
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// /***********************************************************************
//  *		QueryCredentialsAttributesA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI QueryCredentialsAttributesA(
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
//                 ret = package->provider->fnTableA.QueryCredentialsAttributesA(
//                  cred, ulAttribute, pBuffer);
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
// /***********************************************************************
//  *		QueryCredentialsAttributesW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI QueryCredentialsAttributesW(
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
//                 ret = package->provider->fnTableW.QueryCredentialsAttributesW(
//                  cred, ulAttribute, pBuffer);
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
// /***********************************************************************
//  *		InitializeSecurityContextA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI InitializeSecurityContextA(
//  PCredHandle phCredential, PCtxtHandle phContext,
//  SEC_CHAR *pszTargetName, ULONG fContextReq,
//  ULONG Reserved1, ULONG TargetDataRep, PSecBufferDesc pInput,
//  ULONG Reserved2, PCtxtHandle phNewContext, PSecBufferDesc pOutput,
//  ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//     SecurePackage *package = NULL;
//     PCredHandle cred = NULL;
//     PCredHandle ctxt = NULL;
//
//     TRACE("%p %p %s 0x%08x %d %d %p %d %p %p %p %p\n", phCredential, phContext,
//      debugstr_a(pszTargetName), fContextReq, Reserved1, TargetDataRep, pInput,
//      Reserved1, phNewContext, pOutput, pfContextAttr, ptsExpiry);
//
//     if (phContext)
//     {
//         package = (SecurePackage *)phContext->dwUpper;
//         ctxt = (PCtxtHandle)phContext->dwLower;
//     }
//     if (phCredential)
//     {
//         package = (SecurePackage *)phCredential->dwUpper;
//         cred = (PCredHandle)phCredential->dwLower;
//     }
//
//     if (package && package->provider)
//     {
//         if (package->provider->fnTableA.InitializeSecurityContextA)
//         {
//             CtxtHandle myCtxt;
//
//             if (phContext)
//             {
//                 PCtxtHandle realCtxt = (PCtxtHandle)phContext->dwLower;
//                 myCtxt.dwUpper = realCtxt->dwUpper;
//                 myCtxt.dwLower = realCtxt->dwLower;
//             }
//
//             ret = package->provider->fnTableA.InitializeSecurityContextA(
//                  cred, ctxt, pszTargetName, fContextReq,
//                  Reserved1, TargetDataRep, pInput, Reserved2, phNewContext ? &myCtxt : NULL,
//                  pOutput, pfContextAttr, ptsExpiry);
//             if ((ret == SEC_E_OK || ret == SEC_I_CONTINUE_NEEDED) &&
//                 phNewContext && phNewContext != phContext)
//             {
//                 SECURITY_STATUS ret2;
//                 ret2 = SECUR32_makeSecHandle(phNewContext, package, &myCtxt);
//                 if (ret2 != SEC_E_OK)
//                     package->provider->fnTableA.DeleteSecurityContext(&myCtxt);
//             }
//         }
//         else
//             ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// /***********************************************************************
//  *		InitializeSecurityContextW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI InitializeSecurityContextW(
//  PCredHandle phCredential, PCtxtHandle phContext,
//  SEC_WCHAR *pszTargetName, ULONG fContextReq,
//  ULONG Reserved1, ULONG TargetDataRep, PSecBufferDesc pInput,
//  ULONG Reserved2, PCtxtHandle phNewContext, PSecBufferDesc pOutput,
//  ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//     SecurePackage *package = NULL;
//     PCredHandle cred = NULL;
//     PCredHandle ctxt = NULL;
//
//     TRACE("%p %p %s %d %d %d %p %d %p %p %p %p\n", phCredential, phContext,
//      debugstr_w(pszTargetName), fContextReq, Reserved1, TargetDataRep, pInput,
//      Reserved1, phNewContext, pOutput, pfContextAttr, ptsExpiry);
//
//     if (phContext)
//     {
//         package = (SecurePackage *)phContext->dwUpper;
//         ctxt = (PCtxtHandle)phContext->dwLower;
//     }
//     if (phCredential)
//     {
//         package = (SecurePackage *)phCredential->dwUpper;
//         cred = (PCredHandle)phCredential->dwLower;
//     }
//
//     if (package && package->provider)
//     {
//         if (package->provider->fnTableW.InitializeSecurityContextW)
//         {
//             CtxtHandle myCtxt;
//
//             if (phContext)
//             {
//                 PCtxtHandle realCtxt = (PCtxtHandle)phContext->dwLower;
//                 myCtxt.dwUpper = realCtxt->dwUpper;
//                 myCtxt.dwLower = realCtxt->dwLower;
//             }
//
//             ret = package->provider->fnTableW.InitializeSecurityContextW(
//                  cred, ctxt, pszTargetName, fContextReq,
//                  Reserved1, TargetDataRep, pInput, Reserved2, phNewContext ? &myCtxt : NULL,
//                  pOutput, pfContextAttr, ptsExpiry);
//             if ((ret == SEC_E_OK || ret == SEC_I_CONTINUE_NEEDED) &&
//                 phNewContext && phNewContext != phContext)
//             {
//                 SECURITY_STATUS ret2;
//                 ret2 = SECUR32_makeSecHandle(phNewContext, package, &myCtxt);
//                 if (ret2 != SEC_E_OK)
//                     package->provider->fnTableW.DeleteSecurityContext(&myCtxt);
//             }
//         }
//         else
//             ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// /***********************************************************************
//  *		AcceptSecurityContext (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI AcceptSecurityContext(
//  PCredHandle phCredential, PCtxtHandle phContext, PSecBufferDesc pInput,
//  ULONG fContextReq, ULONG TargetDataRep, PCtxtHandle phNewContext,
//  PSecBufferDesc pOutput, ULONG *pfContextAttr, PTimeStamp ptsExpiry)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p %p %d %d %p %p %p %p\n", phCredential, phContext, pInput,
//      fContextReq, TargetDataRep, phNewContext, pOutput, pfContextAttr,
//      ptsExpiry);
//     if (phCredential)
//     {
//         SecurePackage *package = (SecurePackage *)phCredential->dwUpper;
//         PCredHandle cred = (PCredHandle)phCredential->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.AcceptSecurityContext)
//             {
//                 CtxtHandle myCtxt;
//
//                 if(phContext)
//                 {
//                     PCtxtHandle realCtxt = (PCtxtHandle)phContext->dwLower;
//                     TRACE("realCtx: %p\n", realCtxt);
//                     myCtxt.dwUpper = realCtxt->dwUpper;
//                     myCtxt.dwLower = realCtxt->dwLower;
//                 }
//                 
//                 ret = package->provider->fnTableW.AcceptSecurityContext(
//                  cred, phContext ? &myCtxt : NULL, pInput, fContextReq,
//                  TargetDataRep, &myCtxt, pOutput, pfContextAttr, ptsExpiry);
//                 if (ret == SEC_E_OK || ret == SEC_I_CONTINUE_NEEDED)
//                 {
//                     SECURITY_STATUS ret2;
//                     ret2 = SECUR32_makeSecHandle(phNewContext, package, &myCtxt);
//                     if (ret2 != SEC_E_OK)
//                         package->provider->fnTableW.DeleteSecurityContext(
//                          &myCtxt);
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
// /***********************************************************************
//  *		CompleteAuthToken (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI CompleteAuthToken(PCtxtHandle phContext,
//  PSecBufferDesc pToken)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p\n", phContext, pToken);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.CompleteAuthToken)
//                 ret = package->provider->fnTableW.CompleteAuthToken(ctxt,
//                  pToken);
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
// /***********************************************************************
//  *		DeleteSecurityContext (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI DeleteSecurityContext(PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p\n", phContext);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider &&
//          package->provider->fnTableW.DeleteSecurityContext)
//             ret = package->provider->fnTableW.DeleteSecurityContext(ctxt);
//         else
//             ret = SEC_E_INVALID_HANDLE;
//         HeapFree(GetProcessHeap(), 0, ctxt);
//     }
//     else
//         ret = SEC_E_INVALID_HANDLE;
//     return ret;
// }
//
// /***********************************************************************
//  *		ApplyControlToken (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI ApplyControlToken(PCtxtHandle phContext,
//  PSecBufferDesc pInput)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p\n", phContext, pInput);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.ApplyControlToken)
//                 ret = package->provider->fnTableW.ApplyControlToken(
//                  ctxt, pInput);
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
// /***********************************************************************
//  *		QueryContextAttributesA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI QueryContextAttributesA(PCtxtHandle phContext,
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
//                 ret = package->provider->fnTableA.QueryContextAttributesA(
//                  ctxt, ulAttribute, pBuffer);
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
// /***********************************************************************
//  *		QueryContextAttributesW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI QueryContextAttributesW(PCtxtHandle phContext,
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
//                 ret = package->provider->fnTableW.QueryContextAttributesW(
//                  ctxt, ulAttribute, pBuffer);
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
// /***********************************************************************
//  *		ImpersonateSecurityContext (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI ImpersonateSecurityContext(PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p\n", phContext);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.ImpersonateSecurityContext)
//                 ret = package->provider->fnTableW.ImpersonateSecurityContext(
//                  ctxt);
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
// /***********************************************************************
//  *		RevertSecurityContext (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI RevertSecurityContext(PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p\n", phContext);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.RevertSecurityContext)
//                 ret = package->provider->fnTableW.RevertSecurityContext(
//                  ctxt);
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
// /***********************************************************************
//  *		MakeSignature (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI MakeSignature(PCtxtHandle phContext, ULONG fQOP,
//  PSecBufferDesc pMessage, ULONG MessageSeqNo)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p %d\n", phContext, fQOP, pMessage, MessageSeqNo);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.MakeSignature)
//                 ret = package->provider->fnTableW.MakeSignature(
//                  ctxt, fQOP, pMessage, MessageSeqNo);
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
// /***********************************************************************
//  *		VerifySignature (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI VerifySignature(PCtxtHandle phContext,
//  PSecBufferDesc pMessage, ULONG MessageSeqNo, PULONG pfQOP)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p %d %p\n", phContext, pMessage, MessageSeqNo, pfQOP);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.VerifySignature)
//                 ret = package->provider->fnTableW.VerifySignature(
//                  ctxt, pMessage, MessageSeqNo, pfQOP);
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
// /***********************************************************************
//  *		QuerySecurityPackageInfoA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI QuerySecurityPackageInfoA(SEC_CHAR *pszPackageName,
//  PSecPkgInfoA *ppPackageInfo)
// {
//     SECURITY_STATUS ret;
//    
//     TRACE("%s %p\n", debugstr_a(pszPackageName), ppPackageInfo);
//     if (pszPackageName)
//     {
//         SecurePackage *package = SECUR32_findPackageA(pszPackageName);
//
//         if (package)
//         {
//             size_t bytesNeeded = sizeof(SecPkgInfoA);
//             int nameLen = 0, commentLen = 0;
//
//             if (package->infoW.Name)
//             {
//                 nameLen = WideCharToMultiByte(CP_ACP, 0,
//                  package->infoW.Name, -1, NULL, 0, NULL, NULL);
//                 bytesNeeded += nameLen;
//             }
//             if (package->infoW.Comment)
//             {
//                 commentLen = WideCharToMultiByte(CP_ACP, 0,
//                  package->infoW.Comment, -1, NULL, 0, NULL, NULL);
//                 bytesNeeded += commentLen;
//             }
//             *ppPackageInfo = HeapAlloc(GetProcessHeap(), 0, bytesNeeded as _);
//             if (*ppPackageInfo)
//             {
//                 PSTR nextString = (PSTR)((PBYTE)*ppPackageInfo +
//                  sizeof(SecPkgInfoA));
//
//                 memcpy(*ppPackageInfo, &package->infoW, sizeof(package->infoW));
//                 if (package->infoW.Name)
//                 {
//                     (*ppPackageInfo)->Name = nextString;
//                     nextString += WideCharToMultiByte(CP_ACP, 0,
//                      package->infoW.Name, -1, nextString, nameLen, NULL, NULL);
//                 }
//                 else
//                     (*ppPackageInfo)->Name = NULL;
//                 if (package->infoW.Comment)
//                 {
//                     (*ppPackageInfo)->Comment = nextString;
//                     nextString += WideCharToMultiByte(CP_ACP, 0,
//                      package->infoW.Comment, -1, nextString, commentLen, NULL,
//                      NULL);
//                 }
//                 else
//                     (*ppPackageInfo)->Comment = NULL;
//                 ret = SEC_E_OK;
//             }
//             else
//                 ret = SEC_E_INSUFFICIENT_MEMORY;
//         }
//         else
//             ret = SEC_E_SECPKG_NOT_FOUND;
//     }
//     else
//         ret = SEC_E_SECPKG_NOT_FOUND;
//     return ret;
// }
//
// /***********************************************************************
//  *		QuerySecurityPackageInfoW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI QuerySecurityPackageInfoW(SEC_WCHAR *pszPackageName,
//  PSecPkgInfoW *ppPackageInfo)
// {
//     SECURITY_STATUS ret;
//     SecurePackage *package = SECUR32_findPackageW(pszPackageName);
//
//     TRACE("%s %p\n", debugstr_w(pszPackageName), ppPackageInfo);
//     if (package)
//     {
//         size_t bytesNeeded = sizeof(SecPkgInfoW);
//         int nameLen = 0, commentLen = 0;
//
//         if (package->infoW.Name)
//         {
//             nameLen = lstrlenW(package->infoW.Name) + 1;
//             bytesNeeded += nameLen * sizeof(WCHAR);
//         }
//         if (package->infoW.Comment)
//         {
//             commentLen = lstrlenW(package->infoW.Comment) + 1;
//             bytesNeeded += commentLen * sizeof(WCHAR);
//         }
//         *ppPackageInfo = HeapAlloc(GetProcessHeap(), 0, bytesNeeded as _);
//         if (*ppPackageInfo)
//         {
//             PWSTR nextString = (PWSTR)((PBYTE)*ppPackageInfo +
//              sizeof(SecPkgInfoW));
//
//             **ppPackageInfo = package->infoW;
//             if (package->infoW.Name)
//             {
//                 (*ppPackageInfo)->Name = nextString;
//                 lstrcpynW(nextString, package->infoW.Name, nameLen);
//                 nextString += nameLen;
//             }
//             else
//                 (*ppPackageInfo)->Name = NULL;
//             if (package->infoW.Comment)
//             {
//                 (*ppPackageInfo)->Comment = nextString;
//                 lstrcpynW(nextString, package->infoW.Comment, commentLen);
//             }
//             else
//                 (*ppPackageInfo)->Comment = NULL;
//             ret = SEC_E_OK;
//         }
//         else
//             ret = SEC_E_INSUFFICIENT_MEMORY;
//     }
//     else
//         ret = SEC_E_SECPKG_NOT_FOUND;
//     return ret;
// }
//
// /***********************************************************************
//  *		ExportSecurityContext (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI ExportSecurityContext(PCtxtHandle phContext,
//  ULONG fFlags, PSecBuffer pPackedContext, void **pToken)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p %p\n", phContext, fFlags, pPackedContext, pToken);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.ExportSecurityContext)
//                 ret = package->provider->fnTableW.ExportSecurityContext(
//                  ctxt, fFlags, pPackedContext, pToken);
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
// /***********************************************************************
//  *		ImportSecurityContextA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI ImportSecurityContextA(SEC_CHAR *pszPackage,
//  PSecBuffer pPackedContext, void *Token, PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//     SecurePackage *package = SECUR32_findPackageA(pszPackage);
//  
//     TRACE("%s %p %p %p\n", debugstr_a(pszPackage), pPackedContext, Token,
//      phContext);
//     if (package && package->provider)
//     {
//         if (package->provider->fnTableA.ImportSecurityContextA)
//         {
//             CtxtHandle myCtxt;
//
//             ret = package->provider->fnTableA.ImportSecurityContextA(
//              pszPackage, pPackedContext, Token, &myCtxt);
//             if (ret == SEC_E_OK)
//             {
//                 ret = SECUR32_makeSecHandle(phContext, package, &myCtxt);
//                 if (ret != SEC_E_OK)
//                     package->provider->fnTableW.DeleteSecurityContext(&myCtxt);
//             }
//         }
//         else
//             ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//         ret = SEC_E_SECPKG_NOT_FOUND;
//     return ret;
//
// }
//
// /***********************************************************************
//  *		ImportSecurityContextW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI ImportSecurityContextW(SEC_WCHAR *pszPackage,
//  PSecBuffer pPackedContext, void *Token, PCtxtHandle phContext)
// {
//     SECURITY_STATUS ret;
//     SecurePackage *package = SECUR32_findPackageW(pszPackage);
//
//     TRACE("%s %p %p %p\n", debugstr_w(pszPackage), pPackedContext, Token,
//      phContext);
//     if (package && package->provider)
//     {
//         if (package->provider->fnTableW.ImportSecurityContextW)
//         {
//             CtxtHandle myCtxt;
//
//             ret = package->provider->fnTableW.ImportSecurityContextW(
//              pszPackage, pPackedContext, Token, &myCtxt);
//             if (ret == SEC_E_OK)
//             {
//                 ret = SECUR32_makeSecHandle(phContext, package, &myCtxt);
//                 if (ret != SEC_E_OK)
//                     package->provider->fnTableW.DeleteSecurityContext(&myCtxt);
//             }
//         }
//         else
//             ret = SEC_E_UNSUPPORTED_FUNCTION;
//     }
//     else
//         ret = SEC_E_SECPKG_NOT_FOUND;
//     return ret;
// }
//
// /***********************************************************************
//  *		AddCredentialsA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI AddCredentialsA(PCredHandle hCredentials,
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
//             if (package->provider->fnTableA.AddCredentialsA)
//                 ret = package->provider->fnTableA.AddCredentialsA(
//                  cred, pszPrincipal, pszPackage, fCredentialUse, pAuthData,
//                  pGetKeyFn, pvGetKeyArgument, ptsExpiry);
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
// /***********************************************************************
//  *		AddCredentialsW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI AddCredentialsW(PCredHandle hCredentials,
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
//             if (package->provider->fnTableW.AddCredentialsW)
//                 ret = package->provider->fnTableW.AddCredentialsW(
//                  cred, pszPrincipal, pszPackage, fCredentialUse, pAuthData,
//                  pGetKeyFn, pvGetKeyArgument, ptsExpiry);
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
// /***********************************************************************
//  *		QuerySecurityContextToken (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI QuerySecurityContextToken(PCtxtHandle phContext,
//  HANDLE *phToken)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p\n", phContext, phToken);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.QuerySecurityContextToken)
//                 ret = package->provider->fnTableW.QuerySecurityContextToken(
//                  ctxt, phToken);
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
// /***********************************************************************
//  *		EncryptMessage (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI EncryptMessage(PCtxtHandle phContext, ULONG fQOP,
//  PSecBufferDesc pMessage, ULONG MessageSeqNo)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %d %p %d\n", phContext, fQOP, pMessage, MessageSeqNo);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.EncryptMessage)
//                 ret = package->provider->fnTableW.EncryptMessage(
//                  ctxt, fQOP, pMessage, MessageSeqNo);
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
// /***********************************************************************
//  *		DecryptMessage (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI DecryptMessage(PCtxtHandle phContext,
//  PSecBufferDesc pMessage, ULONG MessageSeqNo, PULONG pfQOP)
// {
//     SECURITY_STATUS ret;
//
//     TRACE("%p %p %d %p\n", phContext, pMessage, MessageSeqNo, pfQOP);
//     if (phContext)
//     {
//         SecurePackage *package = (SecurePackage *)phContext->dwUpper;
//         PCtxtHandle ctxt = (PCtxtHandle)phContext->dwLower;
//
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.DecryptMessage)
//                 ret = package->provider->fnTableW.DecryptMessage(
//                  ctxt, pMessage, MessageSeqNo, pfQOP);
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
// /***********************************************************************
//  *		SetContextAttributesA (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI SetContextAttributesA(PCtxtHandle phContext,
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
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableA.SetContextAttributesA)
//                 ret = package->provider->fnTableA.SetContextAttributesA(
//                  ctxt, ulAttribute, pBuffer, cbBuffer);
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
// /***********************************************************************
//  *		SetContextAttributesW (SECUR32.@)
//  */
// SECURITY_STATUS WINAPI SetContextAttributesW(PCtxtHandle phContext,
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
//         if (package && package->provider)
//         {
//             if (package->provider->fnTableW.SetContextAttributesW)
//                 ret = package->provider->fnTableW.SetContextAttributesW(
//                  ctxt, ulAttribute, pBuffer, cbBuffer);
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
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
