use core::ffi::c_ulong;
use crate::prelude::*;
use core::mem::size_of;
use core::ptr::{addr_of_mut, null_mut};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"lsasrv\0\0\0\0\0\0\0\0\0",
        ),
    }
};

pub static mut SeLsaCommandPort: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut SeRmCommandPort: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

unsafe fn LsapComponentTest(_Message: *mut LSAP_RM_API_MESSAGE) {
    if (*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_ERR as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_ERR,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/srm.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"LsapComponentTest\0".as_ptr() as *const ::core::ffi::c_char,
            27i32,
            b"Security: LSA Component Test Command Received\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
}

unsafe fn LsapAdtWriteLog(_Message: PLSAP_RM_API_MESSAGE) {
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & (1i32 << __WINE_DBCL_ERR as core::ffi::c_int))
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_ERR,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/srm.c\0".as_ptr() as *const core::ffi::c_char,
            b"LsapAdtWriteLog\0".as_ptr() as *const core::ffi::c_char,
            35i32,
            b"LsapAdtWriteLog\n\0".as_ptr() as *const core::ffi::c_char,
        );
    }
}
unsafe fn LsapAsync(_Message: PLSAP_RM_API_MESSAGE) {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/srm.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LsapAsync\0" as *const u8 as *const ::core::ffi::c_char,
            43 as ::core::ffi::c_int,
            b"LsapAsync\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
}

unsafe extern "system" fn LsapRmServerThread(_StartContext: *mut c_void) -> u32 {

    let mut Message: LSAP_RM_API_MESSAGE = core::mem::zeroed();
    Message.Header.u1.s1.TotalLength = size_of::<LSAP_RM_API_MESSAGE>() as i16;
    Message.Header.u1.s1.DataLength = 0;

    let mut ReplyMessage: *mut PORT_MESSAGE;
    let mut RemotePortView: REMOTE_PORT_VIEW = REMOTE_PORT_VIEW {
        Length: size_of::<REMOTE_PORT_VIEW>() as u32,
        ViewSize: 0,
        ViewBase: null_mut(),
    };
    let mut MessagePort: HANDLE = null_mut();
    let mut DummyPortHandle: HANDLE = null_mut();
    let mut Status: NTSTATUS;

    Status = NtListenPort(SeLsaCommandPort, addr_of_mut!(Message.Header) as *mut _PORT_MESSAGE);
    if Status < 0 {
        ERR!("LsapRmServerThread - Port Listen failed 0x%lx\n", Status as c_ulong);
        return Status as u32;
    }

    Status = NtAcceptConnectPort(
        addr_of_mut!(MessagePort),
        null_mut(),
        addr_of_mut!(Message.Header) as *mut _PORT_MESSAGE,
        1u8,
        null_mut(),
        addr_of_mut!(RemotePortView),
    );
    if Status < 0 {
        ERR!("LsapRmServerThread - Port Accept Connect failed 0x%lx\n", Status as c_ulong);
        return Status as u32;
    }

    Status = NtCompleteConnectPort(MessagePort);
    if Status < 0 {
        ERR!("LsapRmServerThread - Port Complete Connect failed 0x%lx\n", Status as c_ulong);
        return Status as u32;
    }

    ReplyMessage = null_mut();

    loop {
        Status = NtReplyWaitReceivePort(MessagePort, null_mut(), ReplyMessage, addr_of_mut!(Message.Header) as *mut _PORT_MESSAGE);
        if Status < 0 {
            ERR!("LsapRmServerThread - Failed to get message: 0x%lx\n", Status as c_ulong);
            ReplyMessage = null_mut();
            continue;
        }

        if Message.Header.u2.s2.Type as i32 == LPC_CONNECTION_REQUEST as i32 {
            NtAcceptConnectPort(
                addr_of_mut!(DummyPortHandle),
                null_mut(),
                addr_of_mut!(Message.Header) as *mut _PORT_MESSAGE,
                0u8,
                null_mut(),
                null_mut(),
            );
            ReplyMessage = null_mut();
        } else if Message.Header.u2.s2.Type as i32 == LPC_REQUEST as i32 {
            ReplyMessage = addr_of_mut!(Message.Header) as *mut PORT_MESSAGE;
            match Message.ApiNumber {
                1 => LsapAdtWriteLog(&mut Message),
                3 => LsapAsync(&mut Message),
                2 => LsapComponentTest(&mut Message),
                _ => {
                    ERR!("LsapRmServerThread - invalid API number: 0x%lx\n", Message.ApiNumber as c_ulong);
                    ReplyMessage = null_mut();
                }
            }
        } else {
            ERR!("LsapRmServerThread - unexpected message type: 0x%lx\n", Message.Header.u2.s2.Type as c_ulong);
            ReplyMessage = null_mut();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn LsapRmInitializeServer() -> NTSTATUS {
    let mut Name: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut SecurityQos: SECURITY_QUALITY_OF_SERVICE = SECURITY_QUALITY_OF_SERVICE {
        Length: 0,
        ImpersonationLevel: SecurityIdentification,
        ContextTrackingMode: 0,
        EffectiveOnly: 0,
    };
    let mut InitEvent: HANDLE = core::ptr::null_mut();
    let ThreadHandle: HANDLE;
    let mut ThreadId: DWORD = 0;
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(
        &mut Name,
        &core::mem::transmute::<[u8; 36], [u16; 18]>(*b"\\\0S\0e\0L\0s\0a\0C\0o\0m\0m\0a\0n\0d\0P\0o\0r\0t\0\0\0") as *const _ as PCWSTR,
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = &mut Name as *mut _ as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreatePort(
        core::ptr::addr_of_mut!(SeLsaCommandPort),
        core::ptr::addr_of_mut!(ObjectAttributes),
        0,
        PORT_MAXIMUM_MESSAGE_LENGTH as u32,
        (2 * PAGE_SIZE) as u32,
    );
    if Status < 0 {
        if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR as i32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/srm.c\0" as *const u8 as *const i8,
                b"LsapRmInitializeServer\0" as *const u8 as *const i8,
                187,
                b"LsapRmInitializeServer - Port Create failed 0x%lx\n\0" as *const u8 as *const i8,
                Status,
            );
        }
        return Status;
    }
    RtlInitUnicodeString(
        &mut Name,
        &core::mem::transmute::<[u8; 32], [u16; 16]>(*b"\\\0S\0e\0L\0s\0a\0I\0n\0i\0t\0E\0v\0e\0n\0t\0\0\0") as *const _ as PCWSTR,
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = &mut Name as *mut _ as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenEvent(
        core::ptr::addr_of_mut!(InitEvent),
        2 as u32,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR as i32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/srm.c\0" as *const u8 as *const i8,
                b"LsapRmInitializeServer\0" as *const u8 as *const i8,
                197,
                b"LsapRmInitializeServer - Lsa Init Event Open failed 0x%lx\n\0" as *const u8 as *const i8,
                Status,
            );
        }
        return Status;
    }
    Status = NtSetEvent(InitEvent, core::ptr::null_mut::<i32>());
    if Status < 0 {
        if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR as i32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/srm.c\0" as *const u8 as *const i8,
                b"LsapRmInitializeServer\0" as *const u8 as *const i8,
                205,
                b"LsapRmInitializeServer - Set Init Event failed 0x%lx\n\0" as *const u8 as *const i8,
                Status,
            );
        }
        return Status;
    }
    SecurityQos.ImpersonationLevel = SecurityIdentification;
    SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as u8;
    SecurityQos.EffectiveOnly = 1u8;
    RtlInitUnicodeString(
        &mut Name,
        &core::mem::transmute::<[u8; 34], [u16; 17]>(*b"\\\0S\0e\0R\0m\0C\0o\0m\0m\0a\0n\0d\0P\0o\0r\0t\0\0\0") as *const _ as PCWSTR,
    );
    Status = NtConnectPort(
        core::ptr::addr_of_mut!(SeRmCommandPort),
        &mut Name,
        &mut SecurityQos,
        core::ptr::null_mut::<_PORT_VIEW>(),
        core::ptr::null_mut::<_REMOTE_PORT_VIEW>(),
        core::ptr::null_mut::<u32>(),
        core::ptr::null_mut::<core::ffi::c_void>(),
        core::ptr::null_mut::<u32>(),
    );
    if Status < 0 {
        if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR as i32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/srm.c\0" as *const u8 as *const i8,
                b"LsapRmInitializeServer\0" as *const u8 as *const i8,
                226,
                b"LsapRmInitializeServer - Connect to Rm Command Port failed 0x%lx\n\0" as *const u8 as *const i8,
                Status,
            );
        }
        return Status;
    }
    ThreadHandle = CreateThread(
        core::ptr::null_mut(),
        0usize,
        Some(LsapRmServerThread as unsafe extern "system" fn(*mut core::ffi::c_void) -> u32),
        core::ptr::null_mut::<core::ffi::c_void>(),
        0 as u32,
        core::ptr::addr_of_mut!(ThreadId),
    );
    if ThreadHandle.is_null() {
        if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR as i32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/srm.c\0" as *const u8 as *const i8,
                b"LsapRmInitializeServer\0" as *const u8 as *const i8,
                234,
                b"LsapRmInitializeServer - Create Thread  failed 0x%lx\n\0" as *const u8 as *const i8,
                Status,
            );
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    CloseHandle(ThreadHandle);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapRmCreateLogonSession(
    LogonId: PLUID,
) -> NTSTATUS {
    let mut RequestMessage: SEP_RM_API_MESSAGE = core::mem::zeroed();
    let mut ReplyMessage: SEP_RM_API_MESSAGE = core::mem::zeroed();

    RequestMessage.Header.u2.ZeroInit = 0;
    RequestMessage.Header.u1.s1.TotalLength = (core::mem::size_of::<PORT_MESSAGE>()
        + core::mem::size_of::<ULONG>()
        + core::mem::size_of::<LUID>()) as CSHORT;
    RequestMessage.Header.u1.s1.DataLength =
        RequestMessage.Header.u1.s1.TotalLength - core::mem::size_of::<PORT_MESSAGE>() as CSHORT;

    RequestMessage.ApiNumber = RmCreateLogonSession as ULONG;
    RtlCopyLuid(&raw mut RequestMessage.u.LogonLuid as *mut _LUID, LogonId);

    ReplyMessage.Header.u2.ZeroInit = 0;
    ReplyMessage.Header.u1.s1.TotalLength = (core::mem::size_of::<PORT_MESSAGE>()
        + core::mem::size_of::<ULONG>()
        + core::mem::size_of::<NTSTATUS>()) as CSHORT;
    ReplyMessage.Header.u1.s1.DataLength =
        ReplyMessage.Header.u1.s1.TotalLength - core::mem::size_of::<PORT_MESSAGE>() as CSHORT;
    ReplyMessage.u.ResultStatus = STATUS_SUCCESS;

    let mut Status = NtRequestWaitReplyPort(
        SeRmCommandPort,
        &raw mut RequestMessage as PPORT_MESSAGE,
        &raw mut ReplyMessage as PPORT_MESSAGE,
    );
    if Status >= 0 {
        Status = ReplyMessage.u.ResultStatus;
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapRmDeleteLogonSession(LogonId: PLUID) -> NTSTATUS {
    #[repr(C)]
    struct SEP_RM_API_MESSAGE {
        Header: PORT_MESSAGE,
        ApiNumber: ULONG,
        u: SEP_RM_API_MESSAGE_U,
    }

    #[repr(C)]
    union SEP_RM_API_MESSAGE_U {
        ResultStatus: NTSTATUS,
        LogonLuid: LUID,
    }

    TRACE!(
        b"LsapRmDeleteLogonSession(%p)\n\0".as_ptr() as *const i8,
        LogonId as *const core::ffi::c_void,
    );

    let mut RequestMessage: SEP_RM_API_MESSAGE = unsafe { core::mem::zeroed() };
    RequestMessage.Header.u2.ZeroInit = 0u32;
    RequestMessage.Header.u1.s1.TotalLength =
        (core::mem::size_of::<PORT_MESSAGE>() + core::mem::size_of::<ULONG>() + core::mem::size_of::<LUID>()) as i16;
    RequestMessage.Header.u1.s1.DataLength =
        RequestMessage.Header.u1.s1.TotalLength - core::mem::size_of::<PORT_MESSAGE>() as i16;
    RequestMessage.ApiNumber = RmDeleteLogonSession as u32;
    RtlCopyLuid(&mut RequestMessage.u.LogonLuid, LogonId);

    let mut ReplyMessage: SEP_RM_API_MESSAGE = unsafe { core::mem::zeroed() };
    ReplyMessage.Header.u2.ZeroInit = 0u32;
    ReplyMessage.Header.u1.s1.TotalLength =
        (core::mem::size_of::<PORT_MESSAGE>() + core::mem::size_of::<ULONG>() + core::mem::size_of::<NTSTATUS>()) as i16;
    ReplyMessage.Header.u1.s1.DataLength =
        ReplyMessage.Header.u1.s1.TotalLength - core::mem::size_of::<PORT_MESSAGE>() as i16;
    ReplyMessage.u.ResultStatus = STATUS_SUCCESS;

    let mut Status = NtRequestWaitReplyPort(
        SeRmCommandPort,
        &mut RequestMessage as *mut SEP_RM_API_MESSAGE as PPORT_MESSAGE,
        &mut ReplyMessage as *mut SEP_RM_API_MESSAGE as PPORT_MESSAGE,
    );
    if Status >= 0 {
        Status = ReplyMessage.u.ResultStatus;
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     Local Security Authority Server DLL
//  * FILE:        dll/win32/lsasrv/srm.c
//  * PURPOSE:     Security Reference Monitor Server
//  *
//  * PROGRAMMERS: Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include "lsasrv.h"
// #include <ndk/ntndk.h>
//
// /* GLOBALS *****************************************************************/
//
// HANDLE SeLsaCommandPort;
// HANDLE SeRmCommandPort;
//
// /* FUNCTIONS ***************************************************************/
//
// static
// VOID
// LsapComponentTest(
//     PLSAP_RM_API_MESSAGE Message)
// {
//     ERR("Security: LSA Component Test Command Received\n");
// }
//
// static
// VOID
// LsapAdtWriteLog(
//     PLSAP_RM_API_MESSAGE Message)
// {
//     ERR("LsapAdtWriteLog\n");
// }
//
// static
// VOID
// LsapAsync(
//     PLSAP_RM_API_MESSAGE Message)
// {
//     ERR("LsapAsync\n");
// }
//
// static
// DWORD
// WINAPI
// LsapRmServerThread(
//     PVOID StartContext)
// {
//     LSAP_RM_API_MESSAGE Message;
//     PPORT_MESSAGE ReplyMessage;
//     REMOTE_PORT_VIEW RemotePortView;
//     HANDLE MessagePort, DummyPortHandle;
//     NTSTATUS Status;
//
//     /* Initialize the port message */
//     Message.Header.u1.s1.TotalLength = sizeof(Message);
//     Message.Header.u1.s1.DataLength = 0;
//
//     /* Listen on the LSA command port */
//     Status = NtListenPort(SeLsaCommandPort, &Message.Header);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRmServerThread - Port Listen failed 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Setup the Port View Structure */
//     RemotePortView.Length = sizeof(REMOTE_PORT_VIEW);
//     RemotePortView.ViewSize = 0;
//     RemotePortView.ViewBase = NULL;
//
//     /* Accept the connection */
//     Status = NtAcceptConnectPort(&MessagePort,
//                                  0,
//                                  &Message.Header,
//                                  TRUE,
//                                  NULL,
//                                  &RemotePortView);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRmServerThread - Port Accept Connect failed 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Complete the connection */
//     Status = NtCompleteConnectPort(MessagePort);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRmServerThread - Port Complete Connect failed 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* No reply yet */
//     ReplyMessage = NULL;
//
//     /* Start looping */
//     while (TRUE)
//     {
//         /* Wait for a message */
//         Status = NtReplyWaitReceivePort(MessagePort,
//                                         NULL,
//                                         ReplyMessage,
//                                         &Message.Header);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("LsapRmServerThread - Failed to get message: 0x%lx\n", Status);
//             ReplyMessage = NULL;
//             continue;
//         }
//
//         /* Check if this is a connection request */
//         if (Message.Header.u2.s2.Type == LPC_CONNECTION_REQUEST)
//         {
//             /* Reject connection request */
//             NtAcceptConnectPort(&DummyPortHandle,
//                                 NULL,
//                                 &Message.Header,
//                                 FALSE,
//                                 NULL,
//                                 NULL);
//
//             /* Start over */
//             ReplyMessage = NULL;
//             continue;
//         }
//
//         /* Check if this is an actual request */
//         if (Message.Header.u2.s2.Type == LPC_REQUEST)
//         {
//             ReplyMessage = &Message.Header;
//
//             switch (Message.ApiNumber)
//             {
//                 case LsapAdtWriteLogApi:
//                     LsapAdtWriteLog(&Message);
//                     break;
//
//                 case LsapAsyncApi:
//                     LsapAsync(&Message);
//                     break;
//
//                 case LsapComponentTestApi:
//                     LsapComponentTest(&Message);
//                     break;
//
//                 default:
//                     ERR("LsapRmServerThread - invalid API number: 0x%lx\n",
//                         Message.ApiNumber);
//                     ReplyMessage = NULL;
//             }
//
//             continue;
//         }
//
//         ERR("LsapRmServerThread - unexpected message type: 0x%lx\n",
//             Message.Header.u2.s2.Type);
//
//         /* Start over */
//         ReplyMessage = NULL;
//     }
// }
//
// NTSTATUS
// LsapRmInitializeServer(VOID)
// {
//     UNICODE_STRING Name;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     SECURITY_QUALITY_OF_SERVICE SecurityQos;
//     HANDLE InitEvent;
//     HANDLE ThreadHandle;
//     DWORD ThreadId;
//     NTSTATUS Status;
//
//     /* Create the LSA command port */
//     RtlInitUnicodeString(&Name, L"\\SeLsaCommandPort");
//     InitializeObjectAttributes(&ObjectAttributes, &Name, 0, NULL, NULL);
//     Status = NtCreatePort(&SeLsaCommandPort,
//                           &ObjectAttributes,
//                           0,
//                           PORT_MAXIMUM_MESSAGE_LENGTH,
//                           2 * PAGE_SIZE);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRmInitializeServer - Port Create failed 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Open the LSA init event */
//     RtlInitUnicodeString(&Name, L"\\SeLsaInitEvent");
//     InitializeObjectAttributes(&ObjectAttributes, &Name, 0, NULL, NULL);
//     Status = NtOpenEvent(&InitEvent, 2, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRmInitializeServer - Lsa Init Event Open failed 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Signal the kernel, that we are ready */
//     Status = NtSetEvent(InitEvent, 0);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRmInitializeServer - Set Init Event failed 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Setup the QoS structure */
//     SecurityQos.ImpersonationLevel = SecurityIdentification;
//     SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//     SecurityQos.EffectiveOnly = TRUE;
//
//     /* Connect to the kernel server */
//     RtlInitUnicodeString(&Name, L"\\SeRmCommandPort");
//     Status = NtConnectPort(&SeRmCommandPort,
//                            &Name,
//                            &SecurityQos,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRmInitializeServer - Connect to Rm Command Port failed 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Create the server thread */
//     ThreadHandle = CreateThread(NULL, 0, LsapRmServerThread, NULL, 0, &ThreadId);
//     if (ThreadHandle == NULL)
//     {
//         ERR("LsapRmInitializeServer - Create Thread  failed 0x%lx\n", Status);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Close the server thread handle */
//     CloseHandle(ThreadHandle);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// LsapRmCreateLogonSession(
//     PLUID LogonId)
// {
//     SEP_RM_API_MESSAGE RequestMessage;
//     SEP_RM_API_MESSAGE ReplyMessage;
//     NTSTATUS Status;
//
//     TRACE("LsapRmCreateLogonSession(%p)\n", LogonId);
//
//     RequestMessage.Header.u2.ZeroInit = 0;
//     RequestMessage.Header.u1.s1.TotalLength =
//         (CSHORT)(sizeof(PORT_MESSAGE) + sizeof(ULONG) + sizeof(LUID));
//     RequestMessage.Header.u1.s1.DataLength =
//         RequestMessage.Header.u1.s1.TotalLength -
//         (CSHORT)sizeof(PORT_MESSAGE);
//
//     RequestMessage.ApiNumber = (ULONG)RmCreateLogonSession;
//     RtlCopyLuid(&RequestMessage.u.LogonLuid, LogonId);
//
//     ReplyMessage.Header.u2.ZeroInit = 0;
//     ReplyMessage.Header.u1.s1.TotalLength =
//         (CSHORT)(sizeof(PORT_MESSAGE) + sizeof(ULONG) + sizeof(NTSTATUS));
//     ReplyMessage.Header.u1.s1.DataLength =
//         ReplyMessage.Header.u1.s1.TotalLength -
//         (CSHORT)sizeof(PORT_MESSAGE);
//
//     ReplyMessage.u.ResultStatus = STATUS_SUCCESS;
//
//     Status = NtRequestWaitReplyPort(SeRmCommandPort,
//                                     (PPORT_MESSAGE)&RequestMessage,
//                                     (PPORT_MESSAGE)&ReplyMessage);
//     if (NT_SUCCESS(Status))
//     {
//         Status = ReplyMessage.u.ResultStatus;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// LsapRmDeleteLogonSession(
//     PLUID LogonId)
// {
//     SEP_RM_API_MESSAGE RequestMessage;
//     SEP_RM_API_MESSAGE ReplyMessage;
//     NTSTATUS Status;
//
//     TRACE("LsapRmDeleteLogonSession(%p)\n", LogonId);
//
//     RequestMessage.Header.u2.ZeroInit = 0;
//     RequestMessage.Header.u1.s1.TotalLength =
//         (CSHORT)(sizeof(PORT_MESSAGE) + sizeof(ULONG) + sizeof(LUID));
//     RequestMessage.Header.u1.s1.DataLength =
//         RequestMessage.Header.u1.s1.TotalLength -
//         (CSHORT)sizeof(PORT_MESSAGE);
//
//     RequestMessage.ApiNumber = (ULONG)RmDeleteLogonSession;
//     RtlCopyLuid(&RequestMessage.u.LogonLuid, LogonId);
//
//     ReplyMessage.Header.u2.ZeroInit = 0;
//     ReplyMessage.Header.u1.s1.TotalLength =
//         (CSHORT)(sizeof(PORT_MESSAGE) + sizeof(ULONG) + sizeof(NTSTATUS));
//     ReplyMessage.Header.u1.s1.DataLength =
//         ReplyMessage.Header.u1.s1.TotalLength -
//         (CSHORT)sizeof(PORT_MESSAGE);
//
//     ReplyMessage.u.ResultStatus = STATUS_SUCCESS;
//
//     Status = NtRequestWaitReplyPort(SeRmCommandPort,
//                                     (PPORT_MESSAGE)&RequestMessage,
//                                     (PPORT_MESSAGE)&ReplyMessage);
//     if (NT_SUCCESS(Status))
//     {
//         Status = ReplyMessage.u.ResultStatus;
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
