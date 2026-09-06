use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn SpInitialize(
    PackageId: ULONG_PTR,
    Parameters: PSECPKG_PARAMETERS,
    FunctionTable: PLSA_SECPKG_FUNCTION_TABLE,
) -> NTSTATUS {
    // TRACE!("LsaSpInitialize (0x%p, 0x%p, 0x%p)\n", PackageId, Parameters, FunctionTable);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpShutDown() -> NTSTATUS {
    // TRACE!("LsaSpShutDown\n");
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SpAcceptCredentials(
    LogonType: SECURITY_LOGON_TYPE,
    AccountName: PUNICODE_STRING,
    PrimaryCredentials: PSECPKG_PRIMARY_CRED,
    SupplementalCredentials: PSECPKG_SUPPLEMENTAL_CRED,
) -> NTSTATUS {
    // TRACE!("LsaSpAcceptCredentials(%li %wZ 0x%p 0x%p)\n", LogonType, AccountName, PrimaryCredentials, SupplementalCredentials);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpAcquireCredentialsHandle(
    PrincipalName: PUNICODE_STRING,
    CredentialUseFlags: ULONG,
    LogonId: PLUID,
    AuthorizationData: PVOID,
    GetKeyFunciton: PVOID,
    GetKeyArgument: PVOID,
    CredentialHandle: PLSA_SEC_HANDLE,
    ExpirationTime: PTimeStamp,
) -> NTSTATUS {
    // TRACE!("LsaSpAcquireCredentialsHandle(%wZ 0x%lx 0x%p 0x%p 0x%p 0x%p 0x%p 0x%p)\n", PrincipalName, CredentialUseFlags, LogonId, AuthorizationData, GetKeyFunciton, GetKeyArgument, CredentialHandle, ExpirationTime);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpQueryCredentialsAttributes(
    CredentialHandle: LSA_SEC_HANDLE,
    CredentialAttribute: ULONG,
    Buffer: PVOID,
) -> NTSTATUS {
    // TRACE!("LsaSpQueryCredentialsAttributes(0x%p 0x%lx 0x%p)\n", CredentialHandle, CredentialAttribute, Buffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpFreeCredentialsHandle(
    CredentialHandle: LSA_SEC_HANDLE,
) -> NTSTATUS {
    // TRACE!("LsaSpFreeCredentialsHandle(0x%p)\n", CredentialHandle);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpSaveCredentials(
    CredentialHandle: LSA_SEC_HANDLE,
    Credentials: PSecBuffer,
) -> NTSTATUS {
    // TRACE!("LsaSpSaveCredentials(0x%p 0x%p)\n", CredentialHandle, Credentials);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpGetCredentials(
    CredentialHandle: LSA_SEC_HANDLE,
    Credentials: PSecBuffer,
) -> NTSTATUS {
    // TRACE!("LsaSpGetCredentials(0x%p 0x%p)\n", CredentialHandle, Credentials);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpDeleteCredentials(
    CredentialHandle: LSA_SEC_HANDLE,
    Key: PSecBuffer,
) -> NTSTATUS {
    // TRACE!("LsaSpDeleteCredentials(0x%p 0x%p)\n", CredentialHandle, Key);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpGetInfoW(PackageInfo: PSecPkgInfoW) -> NTSTATUS {
    // TRACE!("LsaGetInfo(0x%p)\n", PackageInfo);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpInitLsaModeContext(
    CredentialHandle: LSA_SEC_HANDLE,
    ContextHandle: LSA_SEC_HANDLE,
    TargetName: PUNICODE_STRING,
    ContextRequirements: ULONG,
    TargetDataRep: ULONG,
    InputBuffers: PSecBufferDesc,
    NewContextHandle: PLSA_SEC_HANDLE,
    OutputBuffers: PSecBufferDesc,
    ContextAttributes: PULONG,
    ExpirationTime: PTimeStamp,
    MappedContext: PBOOLEAN,
    ContextData: PSecBuffer,
) -> NTSTATUS {
    // TRACE!("LsaSpInitLsaModeContext(0x%p 0x%p %wZ 0x%lx %i 0x%p 0x%p 0x%p " "0x%p 0x%p 0x%p 0x%p 0x%p 0x%p 0x%p 0x%p)\n", CredentialHandle, ContextHandle, TargetName, ContextRequirements, TargetDataRep, InputBuffers, NewContextHandle, OutputBuffers, ContextAttributes, ExpirationTime, MappedContext, ContextData);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpAcceptLsaModeContext(
    CredentialHandle: LSA_SEC_HANDLE,
    ContextHandle: LSA_SEC_HANDLE,
    InputBuffer: PSecBufferDesc,
    ContextRequirements: ULONG,
    TargetDataRep: ULONG,
    NewContextHandle: PLSA_SEC_HANDLE,
    OutputBuffer: PSecBufferDesc,
    ContextAttributes: PULONG,
    ExpirationTime: PTimeStamp,
    MappedContext: PBOOLEAN,
    ContextData: PSecBuffer,
) -> NTSTATUS {
    // TRACE!("LsaSpAcceptLsaModeContext(0x%p 0x%p 0x%p %i %i 0x%p 0x%p 0x%p " "0x%p 0x%p 0x%p)\n", CredentialHandle, ContextHandle, InputBuffer, ContextRequirements, TargetDataRep, NewContextHandle, OutputBuffer, ContextAttributes, ExpirationTime, MappedContext, ContextData);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpDeleteContext(ContextHandle: LSA_SEC_HANDLE) -> NTSTATUS {
    // TRACE!("LsaSpDeleteContext(0x%p)\n", ContextHandle);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpApplyControlToken(
    ContextHandle: LSA_SEC_HANDLE,
    ControlToken: PSecBufferDesc,
) -> NTSTATUS {
    // TRACE!("LsaSpApplyControlToken(0x%p 0x%p)\n", ContextHandle, ControlToken);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpGetUserInfo(
    LogonId: PLUID,
    Flags: ULONG,
    UserData: *mut PSecurityUserData,
) -> NTSTATUS {
    // TRACE!("LsaSpGetUserInfo(0x%p 0x%lx 0x%p)\n", LogonId, Flags, UserData);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpGetExtendedInformation(
    Class: SECPKG_EXTENDED_INFORMATION_CLASS,
    ppInfo: *mut PSECPKG_EXTENDED_INFORMATION,
) -> NTSTATUS {
    // TRACE!("LsaSpGetExtendedInformation(0x%lx 0x%p)\n", Class, ppInfo);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LsaSpSetExtendedInformation(
    Class: SECPKG_EXTENDED_INFORMATION_CLASS,
    Info: PSECPKG_EXTENDED_INFORMATION,
) -> NTSTATUS {
    // TRACE!("LsaSpSetExtendedInformation(0x%lx 0x%p)\n", Class, Info);
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Authentication Package DLL
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     NTLM functions returned from SpLsaModeInitialize (PSECPKG_FUNCTION_TABLE)
//  * COPYRIGHT:   Copyright 2019-2020 Andreas Maier <staubim@quantentunnel.de>
//  */
//
// #include "precomp.h"
//
// #include "wine/debug.h"
// WINE_DEFAULT_DEBUG_CHANNEL(msv1_0);
//
//
// NTSTATUS
// NTAPI
// SpInitialize(
//      ULONG_PTR PackageId,
//      PSECPKG_PARAMETERS Parameters,
//      PLSA_SECPKG_FUNCTION_TABLE FunctionTable)
// {
//     TRACE("LsaSpInitialize (0x%p, 0x%p, 0x%p)\n",
//           PackageId, Parameters, FunctionTable);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpShutDown(VOID)
// {
//     TRACE("LsaSpShutDown\n");
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* MS doc says name must be SpAcceptCredentials! */
// NTSTATUS
// NTAPI
// SpAcceptCredentials(
//      SECURITY_LOGON_TYPE LogonType,
//      PUNICODE_STRING AccountName,
//      PSECPKG_PRIMARY_CRED PrimaryCredentials,
//      PSECPKG_SUPPLEMENTAL_CRED SupplementalCredentials)
// {
//     TRACE("LsaSpAcceptCredentials(%li %wZ 0x%p 0x%p)\n",
//           LogonType, AccountName, PrimaryCredentials, SupplementalCredentials);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpAcquireCredentialsHandle(
//      PUNICODE_STRING PrincipalName,
//      ULONG CredentialUseFlags,
//      PLUID LogonId,
//      PVOID AuthorizationData,
//      PVOID GetKeyFunciton,
//      PVOID GetKeyArgument,
//      PLSA_SEC_HANDLE CredentialHandle,
//      PTimeStamp ExpirationTime)
// {
//     TRACE("LsaSpAcquireCredentialsHandle(%wZ 0x%lx 0x%p 0x%p 0x%p 0x%p 0x%p 0x%p)\n",
//           PrincipalName, CredentialUseFlags, LogonId,
//           AuthorizationData, GetKeyFunciton, GetKeyArgument,
//           CredentialHandle, ExpirationTime);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpQueryCredentialsAttributes(
//      LSA_SEC_HANDLE CredentialHandle,
//      ULONG CredentialAttribute,
//      PVOID Buffer)
// {
//     TRACE("LsaSpQueryCredentialsAttributes(0x%p 0x%lx 0x%p)\n",
//           CredentialHandle, CredentialAttribute, Buffer);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpFreeCredentialsHandle(
//      LSA_SEC_HANDLE CredentialHandle)
// {
//     TRACE("LsaSpFreeCredentialsHandle(0x%p)\n", CredentialHandle);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpSaveCredentials(
//      LSA_SEC_HANDLE CredentialHandle,
//      PSecBuffer Credentials)
// {
//     TRACE("LsaSpSaveCredentials(0x%p 0x%p)\n", CredentialHandle, Credentials);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpGetCredentials(
//      LSA_SEC_HANDLE CredentialHandle,
//      PSecBuffer Credentials)
// {
//     TRACE("LsaSpGetCredentials(0x%p 0x%p)\n", CredentialHandle, Credentials);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpDeleteCredentials(
//      LSA_SEC_HANDLE CredentialHandle,
//      PSecBuffer Key)
// {
//     TRACE("LsaSpDeleteCredentials(0x%p 0x%p)\n", CredentialHandle, Key);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpGetInfoW(
//      PSecPkgInfoW PackageInfo)
// {
//     TRACE("LsaGetInfo(0x%p)\n", PackageInfo);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpInitLsaModeContext(
//      LSA_SEC_HANDLE CredentialHandle,
//      LSA_SEC_HANDLE ContextHandle,
//      PUNICODE_STRING TargetName,
//      ULONG ContextRequirements,
//      ULONG TargetDataRep,
//      PSecBufferDesc InputBuffers,
//      PLSA_SEC_HANDLE NewContextHandle,
//      PSecBufferDesc OutputBuffers,
//      PULONG ContextAttributes,
//      PTimeStamp ExpirationTime,
//      PBOOLEAN MappedContext,
//      PSecBuffer ContextData)
// {
//     TRACE("LsaSpInitLsaModeContext(0x%p 0x%p %wZ 0x%lx %i 0x%p 0x%p 0x%p "
//           "0x%p 0x%p 0x%p 0x%p 0x%p 0x%p 0x%p 0x%p)\n",
//           CredentialHandle, ContextHandle, TargetName,
//           ContextRequirements, TargetDataRep, InputBuffers,
//           NewContextHandle, OutputBuffers, ContextAttributes,
//           ExpirationTime, MappedContext, ContextData);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpAcceptLsaModeContext(
//      LSA_SEC_HANDLE CredentialHandle,
//      LSA_SEC_HANDLE ContextHandle,
//      PSecBufferDesc InputBuffer,
//      ULONG ContextRequirements,
//      ULONG TargetDataRep,
//      PLSA_SEC_HANDLE NewContextHandle,
//      PSecBufferDesc OutputBuffer,
//      PULONG ContextAttributes,
//      PTimeStamp ExpirationTime,
//      PBOOLEAN MappedContext,
//      PSecBuffer ContextData)
// {
//     TRACE("LsaSpAcceptLsaModeContext(0x%p 0x%p 0x%p %i %i 0x%p 0x%p 0x%p "
//           "0x%p 0x%p 0x%p)\n",
//           CredentialHandle, ContextHandle, InputBuffer, ContextRequirements,
//           TargetDataRep, NewContextHandle, OutputBuffer,
//           ContextAttributes, ExpirationTime, MappedContext, ContextData);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpDeleteContext(
//      LSA_SEC_HANDLE ContextHandle)
// {
//     TRACE("LsaSpDeleteContext(0x%p)\n", ContextHandle);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpApplyControlToken(
//      LSA_SEC_HANDLE ContextHandle,
//      PSecBufferDesc ControlToken)
// {
//     TRACE("LsaSpApplyControlToken(0x%p 0x%p)\n", ContextHandle, ControlToken);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpGetUserInfo(
//      PLUID LogonId,
//      ULONG Flags,
//      PSecurityUserData *UserData)
// {
//     TRACE("LsaSpGetUserInfo(0x%p 0x%lx 0x%p)\n", LogonId, Flags, UserData);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpGetExtendedInformation(
//      SECPKG_EXTENDED_INFORMATION_CLASS Class,
//      PSECPKG_EXTENDED_INFORMATION *ppInfo)
// {
//     TRACE("LsaSpGetExtendedInformation(0x%lx 0x%p)\n",
//           Class, ppInfo);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// LsaSpSetExtendedInformation(
//      SECPKG_EXTENDED_INFORMATION_CLASS Class,
//      PSECPKG_EXTENDED_INFORMATION Info)
// {
//     TRACE("LsaSpSetExtendedInformation(0x%lx 0x%p)\n",
//           Class, Info);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
