use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn SpInstanceInit(
    Version: ULONG,
    FunctionTable: PSECPKG_DLL_FUNCTIONS,
    UserFunctions: *mut PVOID,
) -> NTSTATUS {
    // TRACE!("SpInstanceInit(Version 0x%lx, 0x%p, 0x%p)\n", Version, FunctionTable, UserFunctions);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpMakeSignature(
    ContextHandle: LSA_SEC_HANDLE,
    QualityOfProtection: ULONG,
    MessageBuffers: PSecBufferDesc,
    MessageSequenceNumber: ULONG,
) -> NTSTATUS {
    // TRACE!("UsrSpMakeSignature(0x%p 0x%x 0x%p 0x%x)\n", ContextHandle, QualityOfProtection, MessageBuffers, MessageSequenceNumber);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpVerifySignature(
    phContext: LSA_SEC_HANDLE,
    pMessage: PSecBufferDesc,
    MessageSeqNo: ULONG,
    pfQOP: PULONG,
) -> NTSTATUS {
    // TRACE!("UsrSpVerifySignature(0x%p 0x%x 0x%x 0x%p)\n", phContext, pMessage, MessageSeqNo, pfQOP);
    (ERROR_NOT_SUPPORTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpSealMessage(
    ContextHandle: LSA_SEC_HANDLE,
    QualityOfProtection: ULONG,
    MessageBuffers: PSecBufferDesc,
    MessageSequenceNumber: ULONG,
) -> NTSTATUS {
    // TRACE!("UsrSpSealMessage(0x%p 0x%x 0x%p 0x%x)\n", ContextHandle, QualityOfProtection, MessageBuffers, MessageSequenceNumber);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpUnsealMessage(
    ContextHandle: LSA_SEC_HANDLE,
    MessageBuffers: PSecBufferDesc,
    MessageSequenceNumber: ULONG,
    QualityOfProtection: PULONG,
) -> NTSTATUS {
    // TRACE!("UsrSpUnsealMessage(0x%p 0x%x 0x%p 0x%x)\n", ContextHandle, MessageBuffers, MessageSequenceNumber, QualityOfProtection);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpGetContextToken(
    ContextHandle: LSA_SEC_HANDLE,
    ImpersonationToken: PHANDLE,
) -> NTSTATUS {
    // TRACE!("UsrSpGetContextToken(0x%p 0x%p)\n", ContextHandle, ImpersonationToken);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpQueryContextAttributes(
    ContextHandle: LSA_SEC_HANDLE,
    ContextAttribute: ULONG,
    Buffer: PVOID,
) -> NTSTATUS {
    // TRACE!("UsrSpQueryContextAttributes(0x%p 0x%x 0x%p)\n", ContextHandle, ContextAttribute, Buffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpCompleteAuthToken(
    ContextHandle: LSA_SEC_HANDLE,
    InputBuffer: PSecBufferDesc,
) -> NTSTATUS {
    // TRACE!("UsrSpCompleteAuthToken(0x%p 0x%p)\n", ContextHandle, InputBuffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpDeleteUserModeContext(
    ContextHandle: LSA_SEC_HANDLE,
) -> NTSTATUS {
    // TRACE!("UsrSpDeleteUserModeContext(0x%p)\n", ContextHandle);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpFormatCredentials(
    Credentials: PSecBuffer,
    FormattedCredentials: PSecBuffer,
) -> NTSTATUS {
    // TRACE!("UsrSpFormatCredentials(0x%p 0x%p)\n", Credentials, FormattedCredentials);
    (ERROR_NOT_SUPPORTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpMarshallSupplementalCreds(
    CredentialSize: ULONG,
    Credentials: PUCHAR,
    MarshalledCredSize: PULONG,
    MarshalledCreds: *mut PVOID,
) -> NTSTATUS {
    // TRACE!("UsrSpMarshallSupplementalCreds(0x%x 0x%p 0x%p 0x%p)\n", CredentialSize, Credentials, MarshalledCredSize, MarshalledCreds);
    (ERROR_NOT_SUPPORTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpExportSecurityContext(
    phContext: LSA_SEC_HANDLE,
    fFlags: ULONG,
    pPackedContext: PSecBuffer,
    pToken: PHANDLE,
) -> NTSTATUS {
    // TRACE!("UsrSpExportSecurityContext(0x%p 0x%x 0x%p 0x%p)\n", phContext, fFlags, pPackedContext, pToken);
    (ERROR_NOT_SUPPORTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn UsrSpImportSecurityContext(
    pPackedContext: PSecBuffer,
    Token: HANDLE,
    phContext: PLSA_SEC_HANDLE,
) -> NTSTATUS {
    // TRACE!("UsrSpImportSecurityContext(0x%p 0x%x 0x%p)\n", pPackedContext, Token, phContext);
    (ERROR_NOT_SUPPORTED) as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Authentication Package DLL
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     NTLM Functions returned from SpUserModeInitialize (PSECPKG_USER_FUNCTION_TABLE)
//  * COPYRIGHT:   Copyright 2019-2020 Andreas Maier <staubim@quantentunnel.de>
//  */
//
// #include "precomp.h"
//
// #include "wine/debug.h"
// WINE_DEFAULT_DEBUG_CHANNEL(msv1_0);
//
// NTSTATUS
// NTAPI
// SpInstanceInit(
//      ULONG Version,
//      PSECPKG_DLL_FUNCTIONS FunctionTable,
//      PVOID *UserFunctions)
// {
//     TRACE("SpInstanceInit(Version 0x%lx, 0x%p, 0x%p)\n",
//           Version, FunctionTable, UserFunctions);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpMakeSignature(
//      LSA_SEC_HANDLE ContextHandle,
//      ULONG QualityOfProtection,
//      PSecBufferDesc MessageBuffers,
//      ULONG MessageSequenceNumber)
// {
//     TRACE("UsrSpMakeSignature(0x%p 0x%x 0x%p 0x%x)\n",
//           ContextHandle, QualityOfProtection,
//           MessageBuffers, MessageSequenceNumber);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpVerifySignature(
//      LSA_SEC_HANDLE phContext,
//      PSecBufferDesc pMessage,
//      ULONG MessageSeqNo,
//      PULONG pfQOP)
// {
//     TRACE("UsrSpVerifySignature(0x%p 0x%x 0x%x 0x%p)\n",
//           phContext, pMessage, MessageSeqNo, pfQOP);
//
//     return ERROR_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpSealMessage(
//      LSA_SEC_HANDLE ContextHandle,
//      ULONG QualityOfProtection,
//      PSecBufferDesc MessageBuffers,
//      ULONG MessageSequenceNumber)
// {
//     TRACE("UsrSpSealMessage(0x%p 0x%x 0x%p 0x%x)\n",
//           ContextHandle, QualityOfProtection,
//           MessageBuffers, MessageSequenceNumber);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpUnsealMessage(
//      LSA_SEC_HANDLE ContextHandle,
//      PSecBufferDesc MessageBuffers,
//      ULONG MessageSequenceNumber,
//      PULONG QualityOfProtection)
// {
//     TRACE("UsrSpUnsealMessage(0x%p 0x%x 0x%p 0x%x)\n",
//           ContextHandle, MessageBuffers,
//           MessageSequenceNumber, QualityOfProtection);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpGetContextToken(
//      LSA_SEC_HANDLE ContextHandle,
//      PHANDLE ImpersonationToken)
// {
//     TRACE("UsrSpGetContextToken(0x%p 0x%p)\n",
//           ContextHandle, ImpersonationToken);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpQueryContextAttributes(
//      LSA_SEC_HANDLE ContextHandle,
//      ULONG ContextAttribute,
//      PVOID Buffer)
// {
//     TRACE("UsrSpQueryContextAttributes(0x%p 0x%x 0x%p)\n",
//             ContextHandle, ContextAttribute, Buffer);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpCompleteAuthToken(
//      LSA_SEC_HANDLE ContextHandle,
//      PSecBufferDesc InputBuffer)
// {
//     TRACE("UsrSpCompleteAuthToken(0x%p 0x%p)\n",
//           ContextHandle, InputBuffer);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpDeleteUserModeContext(
//      LSA_SEC_HANDLE ContextHandle)
// {
//     TRACE("UsrSpDeleteUserModeContext(0x%p)\n",
//           ContextHandle);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpFormatCredentials(
//      PSecBuffer Credentials,
//      PSecBuffer FormattedCredentials)
// {
//     TRACE("UsrSpFormatCredentials(0x%p 0x%p)\n",
//           Credentials, FormattedCredentials);
//
//     return ERROR_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpMarshallSupplementalCreds(
//      ULONG CredentialSize,
//      PUCHAR Credentials,
//      PULONG MarshalledCredSize,
//      PVOID *MarshalledCreds)
// {
//     TRACE("UsrSpMarshallSupplementalCreds(0x%x 0x%p 0x%p 0x%p)\n",
//           CredentialSize, Credentials, MarshalledCredSize, MarshalledCreds);
//
//     return ERROR_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpExportSecurityContext(
//      LSA_SEC_HANDLE phContext,
//      ULONG fFlags,
//      PSecBuffer pPackedContext,
//      PHANDLE pToken)
// {
//     TRACE("UsrSpExportSecurityContext(0x%p 0x%x 0x%p 0x%p)\n",
//           phContext, fFlags, pPackedContext, pToken);
//
//     return ERROR_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// UsrSpImportSecurityContext(
//      PSecBuffer pPackedContext,
//      HANDLE Token,
//      PLSA_SEC_HANDLE phContext)
// {
//     TRACE("UsrSpImportSecurityContext(0x%p 0x%x 0x%p)\n",
//           pPackedContext, Token, phContext);
//
//     return ERROR_NOT_SUPPORTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
