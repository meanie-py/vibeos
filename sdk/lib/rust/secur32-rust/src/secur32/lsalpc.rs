use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut LsaPortHandle: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "C" fn LsapInitLsaPort() {
    LsaPortHandle = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn LsapCloseLsaPort() {
    if LsaPortHandle != core::ptr::null_mut() {
        NtClose(LsaPortHandle);
        LsaPortHandle = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn LsapOpenLsaPort() -> NTSTATUS {
    let mut PortName: _UNICODE_STRING = core::mem::zeroed();
    let mut SecurityQos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();
    let mut ConnectInfo: LSA_CONNECTION_INFO = core::mem::zeroed();
    let mut ConnectInfoLength: ULONG;
    let Status: NTSTATUS;

    if LsaPortHandle != core::ptr::null_mut() {
        return STATUS_SUCCESS;
    }

    RtlInitUnicodeString(
        &mut PortName,
        widestring!("\\LsaAuthenticationPort").as_ptr() as LPWSTR,
    );

    SecurityQos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG;
    SecurityQos.ImpersonationLevel = SecurityIdentification;
    SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as u8;
    SecurityQos.EffectiveOnly = 1u8;

    RtlZeroMemory(
        &mut ConnectInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<LSA_CONNECTION_INFO>() as _,
    );

    ConnectInfo.CreateContext = 0u8 as _;
    ConnectInfo.TrustedCaller = 1u8 as _;

    ConnectInfoLength = core::mem::size_of::<LSA_CONNECTION_INFO>() as ULONG;
    Status = NtConnectPort(
        &mut LsaPortHandle,
        &mut PortName,
        &mut SecurityQos,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut ConnectInfo as *mut _ as *mut core::ffi::c_void,
        &mut ConnectInfoLength,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaConnectUntrusted(LsaHandle: PHANDLE) -> NTSTATUS {
    let mut PortName: _UNICODE_STRING = core::mem::zeroed();
    let mut SecurityQos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();
    let mut ConnectInfo: LSA_CONNECTION_INFO = core::mem::zeroed();
    let mut ConnectInfoLength: ULONG = core::mem::size_of::<LSA_CONNECTION_INFO>() as ULONG;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut EventName: _UNICODE_STRING = core::mem::zeroed();
    let mut EventHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    /* Wait for the LSA authentication thread */
    RtlInitUnicodeString(
        &mut EventName,
        crate::wide!("\\SECURITY\\LSA_AUTHENTICATION_INITIALIZED").as_ptr() as LPWSTR,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_PERMANENT) as ULONG;
    ObjectAttributes.ObjectName = &mut EventName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenEvent(&mut EventHandle, SYNCHRONIZE, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        Status = NtCreateEvent(
            &mut EventHandle,
            SYNCHRONIZE,
            &mut ObjectAttributes,
            _EVENT_TYPE_NotificationEvent,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    Status = NtWaitForSingleObject(EventHandle, 1u8, core::ptr::null_mut());
    NtClose(EventHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Connect to the authentication port */
    RtlInitUnicodeString(
        &mut PortName,
        crate::wide!("\\LsaAuthenticationPort").as_ptr() as LPWSTR,
    );

    SecurityQos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG;
    SecurityQos.ImpersonationLevel = SecurityIdentification;
    SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as u8;
    SecurityQos.EffectiveOnly = 1u8;

    RtlZeroMemory(
        &mut ConnectInfo as *mut _ as *mut core::ffi::c_void,
        ConnectInfoLength as _,
    );

    ConnectInfo.CreateContext = 1u8 as _;
    ConnectInfo.TrustedCaller = 0u8 as _;

    Status = NtConnectPort(
        LsaHandle,
        &mut PortName,
        &mut SecurityQos,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut ConnectInfo as *mut _ as *mut core::ffi::c_void,
        &mut ConnectInfoLength,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if !NT_SUCCESS(ConnectInfo.Status) {}

    ConnectInfo.Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaEnumerateLogonSessions(
    LogonSessionCount: PULONG,
    LogonSessionList: *mut PLUID,
) -> NTSTATUS {
    let mut ApiMessage: LSA_API_MSG = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = LsapOpenLsaPort();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = LSASS_REQUEST_ENUM_LOGON_SESSIONS;
    ApiMessage.h.u1.s1.DataLength =
        LSA_PORT_DATA_SIZE!(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.EnumLogonSessions) as _;
    ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE as _;
    ApiMessage.h.u2.ZeroInit = 0;

    Status = NtRequestWaitReplyPort(
        LsaPortHandle,
        core::ptr::addr_of_mut!(ApiMessage) as PPORT_MESSAGE,
        core::ptr::addr_of_mut!(ApiMessage) as PPORT_MESSAGE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status) {
        return ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status;
    }

    *LogonSessionCount = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.EnumLogonSessions.__bindgen_anon_1.Reply.LogonSessionCount;
    *LogonSessionList = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.EnumLogonSessions.__bindgen_anon_1.Reply.LogonSessionBuffer as _;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaGetLogonSessionData(
    LogonId: PLUID,
    ppLogonSessionData: *mut PSECURITY_LOGON_SESSION_DATA,
) -> NTSTATUS {
    let mut ApiMessage: LSA_API_MSG = core::mem::zeroed();
    let SessionData: PSECURITY_LOGON_SESSION_DATA;
    let mut Status: NTSTATUS;

    Status = LsapOpenLsaPort();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = LSASS_REQUEST_GET_LOGON_SESSION_DATA;
    ApiMessage.h.u1.s1.DataLength =
        LSA_PORT_DATA_SIZE!(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.GetLogonSessionData) as i16;
    ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE as i16;
    ApiMessage.h.u2.ZeroInit = 0;

    RtlCopyLuid(
        core::ptr::addr_of_mut!(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.GetLogonSessionData.__bindgen_anon_1.Request.LogonId) as _,
        LogonId,
    );

    Status = NtRequestWaitReplyPort(
        LsaPortHandle,
        core::ptr::addr_of_mut!(ApiMessage) as PPORT_MESSAGE,
        core::ptr::addr_of_mut!(ApiMessage) as PPORT_MESSAGE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status) {
        return ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status;
    }

    SessionData = ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.GetLogonSessionData.__bindgen_anon_1.Reply.SessionDataBuffer as _;

    if !(*SessionData).UserName.Buffer.is_null() {
        (*SessionData).UserName.Buffer =
            (SessionData as usize + (*SessionData).UserName.Buffer as usize) as LPWSTR;
    }

    if !(*SessionData).LogonDomain.Buffer.is_null() {
        (*SessionData).LogonDomain.Buffer =
            (SessionData as usize + (*SessionData).LogonDomain.Buffer as usize) as LPWSTR;
    }

    if !(*SessionData).AuthenticationPackage.Buffer.is_null() {
        (*SessionData).AuthenticationPackage.Buffer =
            (SessionData as usize + (*SessionData).AuthenticationPackage.Buffer as usize) as LPWSTR;
    }

    if !(*SessionData).Sid.is_null() {
        (*SessionData).Sid =
            (SessionData as usize + (*SessionData).Sid as usize) as _;
    }

    if !(*SessionData).LogonServer.Buffer.is_null() {
        (*SessionData).LogonServer.Buffer =
            (SessionData as usize + (*SessionData).LogonServer.Buffer as usize) as LPWSTR;
    }

    if !(*SessionData).DnsDomainName.Buffer.is_null() {
        (*SessionData).DnsDomainName.Buffer =
            (SessionData as usize + (*SessionData).DnsDomainName.Buffer as usize) as LPWSTR;
    }

    if !(*SessionData).Upn.Buffer.is_null() {
        (*SessionData).Upn.Buffer =
            (SessionData as usize + (*SessionData).Upn.Buffer as usize) as LPWSTR;
    }

    *ppLogonSessionData = SessionData;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaRegisterPolicyChangeNotification(
    InformationClass: POLICY_NOTIFICATION_INFORMATION_CLASS,
    NotificationEventHandle: HANDLE,
) -> NTSTATUS {
    let mut ApiMessage: LSA_API_MSG = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = LsapOpenLsaPort();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = LSASS_REQUEST_POLICY_CHANGE_NOTIFY;
    ApiMessage.h.u1.s1.DataLength =
        LSA_PORT_DATA_SIZE!(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify) as i16;
    ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE as i16;
    ApiMessage.h.u2.ZeroInit = 0;

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify.__bindgen_anon_1.Request.InformationClass = InformationClass;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify.__bindgen_anon_1.Request.NotificationEventHandle = NotificationEventHandle;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify.__bindgen_anon_1.Request.Register = 1u8;

    Status = NtRequestWaitReplyPort(
        LsaPortHandle,
        core::ptr::addr_of_mut!(ApiMessage) as PPORT_MESSAGE,
        core::ptr::addr_of_mut!(ApiMessage) as PPORT_MESSAGE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status) {
        return ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LsaUnregisterPolicyChangeNotification(
    InformationClass: POLICY_NOTIFICATION_INFORMATION_CLASS,
    NotificationEventHandle: HANDLE,
) -> NTSTATUS {
    let mut ApiMessage: LSA_API_MSG = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = LsapOpenLsaPort();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = LSASS_REQUEST_POLICY_CHANGE_NOTIFY;
    ApiMessage.h.u1.s1.DataLength =
        LSA_PORT_DATA_SIZE!(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify) as i16;
    ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE as i16;
    ApiMessage.h.u2.ZeroInit = 0;

    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify.__bindgen_anon_1.Request.InformationClass = InformationClass;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify.__bindgen_anon_1.Request.NotificationEventHandle = NotificationEventHandle;
    ApiMessage.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify.__bindgen_anon_1.Request.Register = 0u8;

    Status = NtRequestWaitReplyPort(
        LsaPortHandle,
        core::ptr::addr_of_mut!(ApiMessage) as PPORT_MESSAGE,
        core::ptr::addr_of_mut!(ApiMessage) as PPORT_MESSAGE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status) {
        return ApiMessage.__bindgen_anon_1.__bindgen_anon_1.Status;
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/secur32/lsalpc.c
//  * PURPOSE:         LSA LPC port functions
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "precomp.h"
//
// #include <ndk/lpctypes.h>
// #include <ndk/lpcfuncs.h>
// #include <ndk/mmfuncs.h>
// #include <ndk/rtlfuncs.h>
// #include <ndk/obfuncs.h>
// #include <psdk/ntsecapi.h>
// #include <lsass/lsass.h>
//
// #include <wine/debug.h>
// WINE_DEFAULT_DEBUG_CHANNEL(secur32);
//
//
// /* GLOBALS *******************************************************************/
//
// HANDLE LsaPortHandle;
//
// extern HANDLE Secur32Heap;
//
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// LsapInitLsaPort(VOID)
// {
//     LsaPortHandle = NULL;
// }
//
//
// VOID
// LsapCloseLsaPort(VOID)
// {
//     if (LsaPortHandle != NULL)
//     {
//         NtClose(LsaPortHandle);
//         LsaPortHandle = NULL;
//     }
// }
//
//
// NTSTATUS
// LsapOpenLsaPort(VOID)
// {
//     UNICODE_STRING PortName;
//     SECURITY_QUALITY_OF_SERVICE SecurityQos;
//     LSA_CONNECTION_INFO ConnectInfo;
//     ULONG ConnectInfoLength;
//     NTSTATUS Status;
//
//     TRACE("LsapOpenLsaPort()\n");
//
//     if (LsaPortHandle != NULL)
//         return STATUS_SUCCESS;
//
//     RtlInitUnicodeString(&PortName,
//                          L"\\LsaAuthenticationPort");
//
//     SecurityQos.Length              = sizeof(SecurityQos);
//     SecurityQos.ImpersonationLevel  = SecurityIdentification;
//     SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//     SecurityQos.EffectiveOnly       = TRUE;
//
//     RtlZeroMemory(&ConnectInfo,
//                   sizeof(ConnectInfo));
//
//     ConnectInfo.CreateContext = FALSE;
//     ConnectInfo.TrustedCaller = YES;
//
//     ConnectInfoLength = sizeof(LSA_CONNECTION_INFO);
//     Status = NtConnectPort(&LsaPortHandle,
//                            &PortName,
//                            &SecurityQos,
//                            NULL,
//                            NULL,
//                            NULL,
//                            &ConnectInfo,
//                            &ConnectInfoLength);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtConnectPort failed (Status 0x%08lx)\n", Status);
//     }
//
//     return Status;
// /*
//     if (!NT_SUCCESS(ConnectInfo.Status))
//     {
//         DPRINT1("ConnectInfo.Status: 0x%08lx\n", ConnectInfo.Status);
//     }
//
//     return ConnectInfo.Status;
// */
// }
//
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaConnectUntrusted(
//     OUT PHANDLE LsaHandle)
// {
//     UNICODE_STRING PortName;
//     SECURITY_QUALITY_OF_SERVICE SecurityQos;
//     LSA_CONNECTION_INFO ConnectInfo;
//     ULONG ConnectInfoLength = sizeof(ConnectInfo);
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING EventName;
//     HANDLE EventHandle;
//     NTSTATUS Status;
//
//     TRACE("LsaConnectUntrusted(%p)\n", LsaHandle);
//
//     // TODO: we may need to impersonate ourselves before, because we are untrusted!
//
//     /* Wait for the LSA authentication thread */
//     RtlInitUnicodeString(&EventName,
//                          L"\\SECURITY\\LSA_AUTHENTICATION_INITIALIZED");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &EventName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status = NtOpenEvent(&EventHandle,
//                          SYNCHRONIZE,
//                          &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("NtOpenEvent failed (Status 0x%08lx)\n", Status);
//
//         Status = NtCreateEvent(&EventHandle,
//                                SYNCHRONIZE,
//                                &ObjectAttributes,
//                                NotificationEvent,
//                                FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             WARN("NtCreateEvent failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//     }
//
//     Status = NtWaitForSingleObject(EventHandle,
//                                    TRUE,
//                                    NULL);
//     NtClose(EventHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtWaitForSingleObject failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Connect to the authentication port */
//     RtlInitUnicodeString(&PortName,
//                          L"\\LsaAuthenticationPort");
//
//     SecurityQos.Length              = sizeof(SecurityQos);
//     SecurityQos.ImpersonationLevel  = SecurityIdentification;
//     SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//     SecurityQos.EffectiveOnly       = TRUE;
//
//     RtlZeroMemory(&ConnectInfo,
//                   ConnectInfoLength);
//
//     ConnectInfo.CreateContext = TRUE;
//     ConnectInfo.TrustedCaller = NO;
//
//     Status = NtConnectPort(LsaHandle,
//                            &PortName,
//                            &SecurityQos,
//                            NULL,
//                            NULL,
//                            NULL,
//                            &ConnectInfo,
//                            &ConnectInfoLength);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtConnectPort failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     if (!NT_SUCCESS(ConnectInfo.Status))
//     {
//         ERR("ConnectInfo.Status: 0x%08lx\n", ConnectInfo.Status);
//     }
//
//     return ConnectInfo.Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaEnumerateLogonSessions(
//     PULONG LogonSessionCount,
//     PLUID *LogonSessionList)
// {
//     LSA_API_MSG ApiMessage;
//     NTSTATUS Status;
//
//     TRACE("LsaEnumerateLogonSessions(%p %p)\n", LogonSessionCount, LogonSessionList);
//
//     Status = LsapOpenLsaPort();
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     ApiMessage.ApiNumber = LSASS_REQUEST_ENUM_LOGON_SESSIONS;
//     ApiMessage.h.u1.s1.DataLength = LSA_PORT_DATA_SIZE(ApiMessage.EnumLogonSessions);
//     ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE;
//     ApiMessage.h.u2.ZeroInit = 0;
//
//     Status = NtRequestWaitReplyPort(LsaPortHandle,
//                                     (PPORT_MESSAGE)&ApiMessage,
//                                     (PPORT_MESSAGE)&ApiMessage);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtRequestWaitReplyPort() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         ERR("NtRequestWaitReplyPort() failed (ApiMessage.Status 0x%08lx)\n", ApiMessage.Status);
//         return ApiMessage.Status;
//     }
//
//     *LogonSessionCount = ApiMessage.EnumLogonSessions.Reply.LogonSessionCount;
//     *LogonSessionList = ApiMessage.EnumLogonSessions.Reply.LogonSessionBuffer as _;
//
//     return Status;
// }
//
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// LsaGetLogonSessionData(
//     PLUID LogonId,
//     PSECURITY_LOGON_SESSION_DATA *ppLogonSessionData)
// {
//     LSA_API_MSG ApiMessage;
//     PSECURITY_LOGON_SESSION_DATA SessionData;
//     NTSTATUS Status;
//
//     TRACE("LsaGetLogonSessionData(%p %p)\n", LogonId, ppLogonSessionData);
//
//     Status = LsapOpenLsaPort();
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     ApiMessage.ApiNumber = LSASS_REQUEST_GET_LOGON_SESSION_DATA;
//     ApiMessage.h.u1.s1.DataLength = LSA_PORT_DATA_SIZE(ApiMessage.GetLogonSessionData);
//     ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE;
//     ApiMessage.h.u2.ZeroInit = 0;
//
//     RtlCopyLuid(&ApiMessage.GetLogonSessionData.Request.LogonId,
//                 LogonId);
//
//     Status = NtRequestWaitReplyPort(LsaPortHandle,
//                                     (PPORT_MESSAGE)&ApiMessage,
//                                     (PPORT_MESSAGE)&ApiMessage);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtRequestWaitReplyPort() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         ERR("NtRequestWaitReplyPort() failed (ApiMessage.Status 0x%08lx)\n", ApiMessage.Status);
//         return ApiMessage.Status;
//     }
//
//     SessionData = ApiMessage.GetLogonSessionData.Reply.SessionDataBuffer as _;
//
//     TRACE("UserName: %p\n", SessionData->UserName.Buffer);
//     if (SessionData->UserName.Buffer != NULL)
//         SessionData->UserName.Buffer = (LPWSTR)((ULONG_PTR)SessionData + (ULONG_PTR)SessionData->UserName.Buffer);
//
//     TRACE("LogonDomain: %p\n", SessionData->LogonDomain.Buffer);
//     if (SessionData->LogonDomain.Buffer != NULL)
//         SessionData->LogonDomain.Buffer = (LPWSTR)((ULONG_PTR)SessionData + (ULONG_PTR)SessionData->LogonDomain.Buffer);
//
//     TRACE("AuthenticationPackage: %p\n", SessionData->AuthenticationPackage.Buffer);
//     if (SessionData->AuthenticationPackage.Buffer != NULL)
//         SessionData->AuthenticationPackage.Buffer = (LPWSTR)((ULONG_PTR)SessionData + (ULONG_PTR)SessionData->AuthenticationPackage.Buffer);
//
//     TRACE("Sid: %p\n", SessionData->Sid);
//     if (SessionData->Sid != NULL)
//         SessionData->Sid = (LPWSTR)((ULONG_PTR)SessionData + (ULONG_PTR)SessionData->Sid);
//
//     TRACE("LogonServer: %p\n", SessionData->LogonServer.Buffer);
//     if (SessionData->LogonServer.Buffer != NULL)
//         SessionData->LogonServer.Buffer = (LPWSTR)((ULONG_PTR)SessionData + (ULONG_PTR)SessionData->LogonServer.Buffer);
//
//     TRACE("DnsDomainName: %p\n", SessionData->DnsDomainName.Buffer);
//     if (SessionData->DnsDomainName.Buffer != NULL)
//         SessionData->DnsDomainName.Buffer = (LPWSTR)((ULONG_PTR)SessionData + (ULONG_PTR)SessionData->DnsDomainName.Buffer);
//
//     TRACE("Upn: %p\n", SessionData->Upn.Buffer);
//     if (SessionData->Upn.Buffer != NULL)
//         SessionData->Upn.Buffer = (LPWSTR)((ULONG_PTR)SessionData + (ULONG_PTR)SessionData->Upn.Buffer);
//
//     *ppLogonSessionData = SessionData;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaRegisterPolicyChangeNotification(
//     POLICY_NOTIFICATION_INFORMATION_CLASS InformationClass,
//     HANDLE NotificationEventHandle)
// {
//     LSA_API_MSG ApiMessage;
//     NTSTATUS Status;
//
//     TRACE("LsaRegisterPolicyChangeNotification(%lu %p)\n",
//           InformationClass, NotificationEventHandle);
//
//     Status = LsapOpenLsaPort();
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     ApiMessage.ApiNumber = LSASS_REQUEST_POLICY_CHANGE_NOTIFY;
//     ApiMessage.h.u1.s1.DataLength = LSA_PORT_DATA_SIZE(ApiMessage.PolicyChangeNotify);
//     ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE;
//     ApiMessage.h.u2.ZeroInit = 0;
//
//     ApiMessage.PolicyChangeNotify.Request.InformationClass = InformationClass;
//     ApiMessage.PolicyChangeNotify.Request.NotificationEventHandle = NotificationEventHandle;
//     ApiMessage.PolicyChangeNotify.Request.Register = TRUE;
//
//     Status = NtRequestWaitReplyPort(LsaPortHandle,
//                                     (PPORT_MESSAGE)&ApiMessage,
//                                     (PPORT_MESSAGE)&ApiMessage);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtRequestWaitReplyPort() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         ERR("NtRequestWaitReplyPort() failed (ApiMessage.Status 0x%08lx)\n", ApiMessage.Status);
//         return ApiMessage.Status;
//     }
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LsaUnregisterPolicyChangeNotification(
//     POLICY_NOTIFICATION_INFORMATION_CLASS InformationClass,
//     HANDLE NotificationEventHandle)
// {
//     LSA_API_MSG ApiMessage;
//     NTSTATUS Status;
//
//     TRACE("LsaUnregisterPolicyChangeNotification(%lu %p)\n",
//           InformationClass, NotificationEventHandle);
//
//     Status = LsapOpenLsaPort();
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     ApiMessage.ApiNumber = LSASS_REQUEST_POLICY_CHANGE_NOTIFY;
//     ApiMessage.h.u1.s1.DataLength = LSA_PORT_DATA_SIZE(ApiMessage.PolicyChangeNotify);
//     ApiMessage.h.u1.s1.TotalLength = LSA_PORT_MESSAGE_SIZE;
//     ApiMessage.h.u2.ZeroInit = 0;
//
//     ApiMessage.PolicyChangeNotify.Request.InformationClass = InformationClass;
//     ApiMessage.PolicyChangeNotify.Request.NotificationEventHandle = NotificationEventHandle;
//     ApiMessage.PolicyChangeNotify.Request.Register = FALSE;
//
//     Status = NtRequestWaitReplyPort(LsaPortHandle,
//                                     (PPORT_MESSAGE)&ApiMessage,
//                                     (PPORT_MESSAGE)&ApiMessage);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtRequestWaitReplyPort() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         ERR("NtRequestWaitReplyPort() failed (ApiMessage.Status 0x%08lx)\n", ApiMessage.Status);
//         return ApiMessage.Status;
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
