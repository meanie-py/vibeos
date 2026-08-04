use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hDllInstance: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn AuthzAccessCheck(
    flags: DWORD,
    AuthzClientContext: AUTHZ_CLIENT_CONTEXT_HANDLE,
    pRequest: PAUTHZ_ACCESS_REQUEST,
    AuditInfo: AUTHZ_AUDIT_INFO_HANDLE,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    OptionalSecurityDescriptorArray: *mut PSECURITY_DESCRIPTOR,
    OptionalSecurityDescriptorCount: DWORD,
    pReply: PAUTHZ_ACCESS_REPLY,
    pAuthzHandle: PAUTHZ_ACCESS_CHECK_RESULTS_HANDLE,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzAddSidsToContext(
    OrigClientContext: AUTHZ_CLIENT_CONTEXT_HANDLE,
    Sids: PSID_AND_ATTRIBUTES,
    SidCount: DWORD,
    RestrictedSids: PSID_AND_ATTRIBUTES,
    RestrictedSidCount: DWORD,
    pNewClientContext: PAUTHZ_CLIENT_CONTEXT_HANDLE,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzCachedAccessCheck(
    Flags: DWORD,
    AuthzHandle: AUTHZ_ACCESS_CHECK_RESULTS_HANDLE,
    pRequest: PAUTHZ_ACCESS_REQUEST,
    AuditInfo: AUTHZ_AUDIT_EVENT_HANDLE,
    pReply: PAUTHZ_ACCESS_REPLY,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzEnumerateSecurityEventSources(
    dwFlags: DWORD,
    Buffer: PAUTHZ_SOURCE_SCHEMA_REGISTRATION,
    pdwCount: PDWORD,
    pdwLength: PDWORD,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzFreeAuditEvent(
    pAuditEventInfo: AUTHZ_AUDIT_EVENT_HANDLE,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzFreeHandle(
    AuthzHandle: AUTHZ_ACCESS_CHECK_RESULTS_HANDLE,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzInitializeContextFromAuthzContext(
    flags: DWORD,
    AuthzHandle: AUTHZ_CLIENT_CONTEXT_HANDLE,
    ExpirationTime: PLARGE_INTEGER,
    Identifier: LUID,
    DynamicGroupArgs: PVOID,
    phNewAuthzHandle: PAUTHZ_CLIENT_CONTEXT_HANDLE,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzInitializeContextFromToken(
    Flags: DWORD,
    TokenHandle: HANDLE,
    AuthzResourceManager: AUTHZ_RESOURCE_MANAGER_HANDLE,
    pExpirationTime: PLARGE_INTEGER,
    Identifier: LUID,
    DynamicGroupArgs: PVOID,
    pAuthzClientContext: PAUTHZ_CLIENT_CONTEXT_HANDLE,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

// C-VARIADIC, and stable Rust cannot DEFINE one (rust-lang/rust#44930). No shim is needed
// here and none is honest: the C body is `UNIMPLEMENTED; return FALSE;` and never touches the
// varargs, authz.spec declares the export `cdecl` with only the FIXED parameters, and under
// cdecl the CALLER cleans the stack — so a fixed-arity callee that ignores the extras is
// ABI-identical to the variadic one for every caller, on all three targets. If this is ever
// implemented it needs the varargs trampoline (sdk/lib/rust/varargs-rust), not this shape.
#[no_mangle]
pub unsafe extern "C" fn AuthzInitializeObjectAccessAuditEvent(
    Flags: DWORD,
    hAuditEventType: AUTHZ_AUDIT_EVENT_TYPE_HANDLE,
    szOperationType: PWSTR,
    szObjectType: PWSTR,
    szObjectName: PWSTR,
    szAdditionalInfo: PWSTR,
    phAuditEvent: PAUTHZ_AUDIT_EVENT_HANDLE,
    dwAdditionalParamCount: DWORD
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

// C-VARIADIC, and stable Rust cannot DEFINE one (rust-lang/rust#44930). No shim is needed
// here and none is honest: the C body is `UNIMPLEMENTED; return FALSE;` and never touches the
// varargs, authz.spec declares the export `cdecl` with only the FIXED parameters, and under
// cdecl the CALLER cleans the stack — so a fixed-arity callee that ignores the extras is
// ABI-identical to the variadic one for every caller, on all three targets. If this is ever
// implemented it needs the varargs trampoline (sdk/lib/rust/varargs-rust), not this shape.
#[no_mangle]
pub unsafe extern "C" fn AuthzInitializeObjectAccessAuditEvent2(
    Flags: DWORD,
    hAuditEventType: AUTHZ_AUDIT_EVENT_TYPE_HANDLE,
    szOperationType: PWSTR,
    szObjectType: PWSTR,
    szObjectName: PWSTR,
    szAdditionalInfo: PWSTR,
    szAdditionalInfo2: PWSTR,
    phAuditEvent: PAUTHZ_AUDIT_EVENT_HANDLE,
    dwAdditionalParameterCount: DWORD
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzInstallSecurityEventSource(
    dwFlags: DWORD,
    pRegistration: PAUTHZ_SOURCE_SCHEMA_REGISTRATION,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzOpenObjectAudit(
    Flags: DWORD,
    hAuthzClientContext: AUTHZ_CLIENT_CONTEXT_HANDLE,
    pRequest: PAUTHZ_ACCESS_REQUEST,
    hAuditEvent: AUTHZ_AUDIT_EVENT_HANDLE,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    SecurityDescriptorArray: *mut PSECURITY_DESCRIPTOR,
    SecurityDescriptorCount: DWORD,
    pReply: PAUTHZ_ACCESS_REPLY,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzRegisterSecurityEventSource(
    dwFlags: DWORD,
    szEventSourceName: PCWSTR,
    phEventProvider: PAUTHZ_SECURITY_EVENT_PROVIDER_HANDLE,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

// C-VARIADIC, and stable Rust cannot DEFINE one (rust-lang/rust#44930). No shim is needed
// here and none is honest: the C body is `UNIMPLEMENTED; return FALSE;` and never touches the
// varargs, authz.spec declares the export `cdecl` with only the FIXED parameters, and under
// cdecl the CALLER cleans the stack — so a fixed-arity callee that ignores the extras is
// ABI-identical to the variadic one for every caller, on all three targets. If this is ever
// implemented it needs the varargs trampoline (sdk/lib/rust/varargs-rust), not this shape.
#[no_mangle]
pub unsafe extern "C" fn AuthzReportSecurityEvent(
    dwFlags: DWORD,
    hEventProvider: AUTHZ_SECURITY_EVENT_PROVIDER_HANDLE,
    dwAuditId: DWORD,
    pUserSid: PSID,
    dwCount: DWORD
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzReportSecurityEventFromParams(
    dwFlags: DWORD,
    hEventProvider: AUTHZ_SECURITY_EVENT_PROVIDER_HANDLE,
    dwAuditId: DWORD,
    pUserSid: PSID,
    pParams: PAUDIT_PARAMS,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzUninstallSecurityEventSource(
    dwFlags: DWORD,
    szEventSourceName: PWSTR,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzUnregisterSecurityEventSource(
    dwFlags: DWORD,
    phEventProvider: PAUTHZ_SECURITY_EVENT_PROVIDER_HANDLE,
) -> BOOL {
    UNIMPLEMENTED!();
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: PDWORD,
    lpvReserved: LPVOID,
) -> i32 {
    match dwReason as u32 {
        DLL_PROCESS_ATTACH => {
            hDllInstance = hinstDLL;
        }
        DLL_THREAD_ATTACH | DLL_THREAD_DETACH | DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Authorization Framework
//  * Copyright (C) 2005 - 2006 ReactOS Team
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
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
// /*
//  * PROJECT:         ReactOS Authorization Framework
//  * FILE:            lib/authz/authz.c
//  * PURPOSE:         Authorization Framework
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  *
//  * UPDATE HISTORY:
//  *      09/30/2005  Created
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// HINSTANCE hDllInstance;
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzAccessCheck( DWORD flags,
//                   AUTHZ_CLIENT_CONTEXT_HANDLE AuthzClientContext,
//                   PAUTHZ_ACCESS_REQUEST pRequest,
//                   AUTHZ_AUDIT_INFO_HANDLE AuditInfo,
//                   PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                   PSECURITY_DESCRIPTOR* OptionalSecurityDescriptorArray,
//                   DWORD OptionalSecurityDescriptorCount  ,
//                    PAUTHZ_ACCESS_REPLY pReply,
//                   PAUTHZ_ACCESS_CHECK_RESULTS_HANDLE pAuthzHandle)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzAddSidsToContext( AUTHZ_CLIENT_CONTEXT_HANDLE OrigClientContext,
//                        PSID_AND_ATTRIBUTES Sids,
//                        DWORD SidCount,
//                        PSID_AND_ATTRIBUTES RestrictedSids,
//                        DWORD RestrictedSidCount,
//                        PAUTHZ_CLIENT_CONTEXT_HANDLE pNewClientContext)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzCachedAccessCheck( DWORD Flags,
//                         AUTHZ_ACCESS_CHECK_RESULTS_HANDLE AuthzHandle,
//                         PAUTHZ_ACCESS_REQUEST pRequest,
//                         AUTHZ_AUDIT_EVENT_HANDLE AuditInfo,
//                         PAUTHZ_ACCESS_REPLY pReply)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzEnumerateSecurityEventSources( DWORD dwFlags,
//                                     PAUTHZ_SOURCE_SCHEMA_REGISTRATION Buffer,
//                                     PDWORD pdwCount,
//                                      PDWORD pdwLength)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzFreeAuditEvent( AUTHZ_AUDIT_EVENT_HANDLE pAuditEventInfo)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzFreeHandle( AUTHZ_ACCESS_CHECK_RESULTS_HANDLE AuthzHandle)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzInitializeContextFromAuthzContext( DWORD flags,
//                                         AUTHZ_CLIENT_CONTEXT_HANDLE AuthzHandle,
//                                         PLARGE_INTEGER ExpirationTime,
//                                         LUID Identifier,
//                                         PVOID DynamicGroupArgs,
//                                         PAUTHZ_CLIENT_CONTEXT_HANDLE phNewAuthzHandle)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzInitializeContextFromToken( DWORD Flags,
//                                  HANDLE TokenHandle,
//                                  AUTHZ_RESOURCE_MANAGER_HANDLE AuthzResourceManager,
//                                  PLARGE_INTEGER pExpirationTime,
//                                  LUID Identifier,
//                                  PVOID DynamicGroupArgs,
//                                  PAUTHZ_CLIENT_CONTEXT_HANDLE pAuthzClientContext)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// __cdecl
// AuthzInitializeObjectAccessAuditEvent( DWORD Flags,
//                                        AUTHZ_AUDIT_EVENT_TYPE_HANDLE hAuditEventType,
//                                        PWSTR szOperationType,
//                                        PWSTR szObjectType,
//                                        PWSTR szObjectName,
//                                        PWSTR szAdditionalInfo,
//                                        PAUTHZ_AUDIT_EVENT_HANDLE phAuditEvent,
//                                        DWORD dwAdditionalParamCount,
//                                       ...)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// __cdecl
// AuthzInitializeObjectAccessAuditEvent2( DWORD Flags,
//                                         AUTHZ_AUDIT_EVENT_TYPE_HANDLE hAuditEventType,
//                                         PWSTR szOperationType,
//                                         PWSTR szObjectType,
//                                         PWSTR szObjectName,
//                                         PWSTR szAdditionalInfo,
//                                         PWSTR szAdditionalInfo2,
//                                         PAUTHZ_AUDIT_EVENT_HANDLE phAuditEvent,
//                                         DWORD dwAdditionalParameterCount,
//                                        ...)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzInstallSecurityEventSource( DWORD dwFlags,
//                                  PAUTHZ_SOURCE_SCHEMA_REGISTRATION pRegistration)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzOpenObjectAudit( DWORD Flags,
//                       AUTHZ_CLIENT_CONTEXT_HANDLE hAuthzClientContext,
//                       PAUTHZ_ACCESS_REQUEST pRequest,
//                       AUTHZ_AUDIT_EVENT_HANDLE hAuditEvent,
//                       PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                       PSECURITY_DESCRIPTOR* SecurityDescriptorArray,
//                       DWORD SecurityDescriptorCount,
//                       PAUTHZ_ACCESS_REPLY pReply)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzRegisterSecurityEventSource( DWORD dwFlags,
//                                   PCWSTR szEventSourceName,
//                                   PAUTHZ_SECURITY_EVENT_PROVIDER_HANDLE phEventProvider)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// __cdecl
// AuthzReportSecurityEvent( DWORD dwFlags,
//                           AUTHZ_SECURITY_EVENT_PROVIDER_HANDLE hEventProvider,
//                           DWORD dwAuditId,
//                           PSID pUserSid  ,
//                           DWORD dwCount,
//                          ...)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzReportSecurityEventFromParams( DWORD dwFlags,
//                                     AUTHZ_SECURITY_EVENT_PROVIDER_HANDLE hEventProvider,
//                                     DWORD dwAuditId,
//                                     PSID pUserSid  ,
//                                     PAUDIT_PARAMS pParams)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzUninstallSecurityEventSource( DWORD dwFlags,
//                                    PWSTR szEventSourceName)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzUnregisterSecurityEventSource( DWORD dwFlags,
//                                      PAUTHZ_SECURITY_EVENT_PROVIDER_HANDLE phEventProvider)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// BOOL WINAPI
// DllMain( HINSTANCE hinstDLL,
//          DWORD dwReason,
//          LPVOID lpvReserved)
// {
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             hDllInstance = hinstDLL;
//             break;
//
//         case DLL_THREAD_ATTACH:
//         case DLL_THREAD_DETACH:
//         case DLL_PROCESS_DETACH:
//             break;
//     }
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
