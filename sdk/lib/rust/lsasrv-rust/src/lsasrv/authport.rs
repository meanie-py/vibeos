use crate::prelude::*;

// Cross-file LSA request handlers dispatched by AuthPortThreadRoutine.
// 3 are Rust siblings (#[no_mangle]): LsapEnumLogonSessions/LsapGetLogonSessionData
// (session.rs), LsapRegisterNotification (notify.rs). 3 are still C: Lsap{LogonUser,
// CallAuthenticationPackage,LookupAuthenticationPackage} (authpackage.c). All cdecl
// (no NTAPI in lsasrv.h prototypes) → extern "C".
extern "C" {
    fn LsapCallAuthenticationPackage(
        RequestMsg: PLSA_API_MSG,
        LogonContext: PLSAP_LOGON_CONTEXT,
    ) -> NTSTATUS;
    fn LsapLogonUser(RequestMsg: PLSA_API_MSG, LogonContext: PLSAP_LOGON_CONTEXT) -> NTSTATUS;
    fn LsapLookupAuthenticationPackage(
        RequestMsg: PLSA_API_MSG,
        LogonContext: PLSAP_LOGON_CONTEXT,
    ) -> NTSTATUS;
    fn LsapEnumLogonSessions(RequestMsg: PLSA_API_MSG) -> NTSTATUS;
    fn LsapGetLogonSessionData(RequestMsg: PLSA_API_MSG) -> NTSTATUS;
    fn LsapRegisterNotification(RequestMsg: PLSA_API_MSG) -> NTSTATUS;
}

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

static mut LsapLogonContextList: LIST_ENTRY = LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut PortThreadHandle: HANDLE = NULL;

static mut AuthPortHandle: HANDLE = NULL;

unsafe fn LsapDeregisterLogonProcess(
    RequestMsg: PLSA_API_MSG,
    LogonContext: PLSAP_LOGON_CONTEXT,
) -> NTSTATUS {
    // TRACE macro expansion
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"LsapDeregisterLogonProcess\0".as_ptr() as *const ::core::ffi::c_char,
            25 as ::core::ffi::c_int,
            b"LsapDeregisterLogonProcess(%p %p)\n\0".as_ptr() as *const ::core::ffi::c_char,
            RequestMsg,
            LogonContext,
        );
    }

    // The C calls RemoveHeadList(&LogonContext->Entry), which unlinks the entry AFTER this
    // context -- the list head itself when this is the only one -- and leaves the context
    // being freed on LsapLogonContextList. The next deregistration walked freed memory and
    // lsass died (arm64 modrun, two LogonUserW clients in a row). Unlink the context itself.
    RemoveEntryList(&raw mut (*LogonContext).Entry as *mut _);
    NtClose((*LogonContext).ClientProcessHandle);
    NtClose((*LogonContext).ConnectionHandle);
    RtlFreeHeap(RtlGetProcessHeap(), 0, LogonContext as PVOID);

    STATUS_SUCCESS
}

unsafe fn LsapIsTrustedClient(ProcessHandle: HANDLE) -> BOOL {
    let tcb_privilege = LUID { LowPart: SE_TCB_PRIVILEGE, HighPart: 0 };
    let mut token_handle: HANDLE = core::ptr::null_mut();
    let mut privileges: PTOKEN_PRIVILEGES = core::ptr::null_mut();
    let mut size: ULONG = 0;

    let trusted: BOOL = 'done: {
        let mut status = NtOpenProcessToken(ProcessHandle, TOKEN_QUERY, &mut token_handle);
        if status < 0 {
            break 'done 0;
        }

        status = NtQueryInformationToken(token_handle, TokenPrivileges, core::ptr::null_mut(), 0, &mut size);
        if status < 0 && status != STATUS_BUFFER_TOO_SMALL {
            break 'done 0;
        }

        privileges = RtlAllocateHeap(RtlGetProcessHeap(), 0, size as SIZE_T) as PTOKEN_PRIVILEGES;
        if privileges.is_null() {
            break 'done 0;
        }

        status = NtQueryInformationToken(token_handle, TokenPrivileges, privileges as *mut core::ffi::c_void, size, &mut size);
        if status < 0 {
            break 'done 0;
        }

        for i in 0..(*privileges).PrivilegeCount {
            // TOKEN_PRIVILEGES.Privileges is `LUID_AND_ATTRIBUTES Privileges[ANYSIZE_ARRAY]`;
            // the loop is bounded by PrivilegeCount, so bindgen's `[T; 1]` panics at i == 1.
            let priv_luid = &(*(&raw mut (*privileges).Privileges[0]).add(i as usize)).Luid;
            if priv_luid.LowPart == tcb_privilege.LowPart && priv_luid.HighPart == tcb_privilege.HighPart {
                break 'done 1;
            }
        }
        break 'done 0;
    };

    if !privileges.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, privileges as *mut core::ffi::c_void);
    }
    if !token_handle.is_null() {
        NtClose(token_handle);
    }
    trusted
}

unsafe fn LsapCheckLogonProcess(
    RequestMsg: *mut LSA_API_MSG,
    LogonContext: *mut PLSAP_LOGON_CONTEXT,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let Context: PLSAP_LOGON_CONTEXT;
    let Status: NTSTATUS;

    TRACE!(
        b"LsapCheckLogonProcess(%p)\n\0".as_ptr() as *const i8,
        RequestMsg,
    );

    TRACE!(
        b"Client ID: %p %p\n\0".as_ptr() as *const i8,
        (*RequestMsg).h.__bindgen_anon_1.ClientId.UniqueProcess,
        (*RequestMsg).h.__bindgen_anon_1.ClientId.UniqueThread,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = core::ptr::null_mut();
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenProcess(
        core::ptr::addr_of_mut!(ProcessHandle),
        (PROCESS_VM_READ
            | PROCESS_VM_WRITE
            | PROCESS_VM_OPERATION
            | PROCESS_DUP_HANDLE
            | PROCESS_QUERY_INFORMATION) as u32,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!((*RequestMsg).h.__bindgen_anon_1.ClientId) as *mut _,
    );

    if !NT_SUCCESS(Status) {
        TRACE!(
            b"NtOpenProcess() failed (Status %lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        return Status;
    }

    Context = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<LSAP_LOGON_CONTEXT>() as SIZE_T,
    ) as PLSAP_LOGON_CONTEXT;

    if Context.is_null() {
        NtClose(ProcessHandle);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TRACE!(
        b"New LogonContext: %p\n\0".as_ptr() as *const i8,
        Context,
    );

    (*Context).ClientProcessHandle = ProcessHandle;

    match (*RequestMsg).__bindgen_anon_1.ConnectInfo.TrustedCaller {
        0 => {
            (*Context).TrustedCaller = 0;
        }
        1 => {
            (*Context).TrustedCaller = 1;
        }
        _ => {
            (*Context).TrustedCaller = LsapIsTrustedClient(ProcessHandle);
        }
    }

    TRACE!(
        b"TrustedCaller: %u\n\0".as_ptr() as *const i8,
        (*Context).TrustedCaller,
    );

    *LogonContext = Context;
    STATUS_SUCCESS
}

unsafe fn LsapHandlePortConnection(RequestMsg: *mut _LSA_API_MSG) -> NTSTATUS {
    let mut LogonContext: *mut _LSAP_LOGON_CONTEXT = core::ptr::null_mut();
    let mut ConnectionHandle: HANDLE = core::ptr::null_mut();
    let Accept: BOOLEAN;
    let mut RemotePortView: REMOTE_PORT_VIEW = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*RequestMsg).__bindgen_anon_1.ConnectInfo.CreateContext != 0 {
        Status = LsapCheckLogonProcess(RequestMsg, &raw mut LogonContext);
        (*RequestMsg).__bindgen_anon_1.ConnectInfo.OperationalMode = 0x43218765;
        (*RequestMsg).__bindgen_anon_1.ConnectInfo.Status = Status;
    }

    if Status >= 0 {
        Accept = 1u8;
    } else {
        Accept = 0u8;
    }

    RemotePortView.Length = core::mem::size_of::<REMOTE_PORT_VIEW>() as u32;
    Status = NtAcceptConnectPort(
        &raw mut ConnectionHandle,
        LogonContext as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*RequestMsg).h) as *mut _,
        Accept,
        core::ptr::null_mut(),
        &raw mut RemotePortView,
    );
    if !(Status >= 0) {
        return Status;
    }

    if Accept != 0u8 {
        if !LogonContext.is_null() {
            (*LogonContext).ConnectionHandle = ConnectionHandle;
            InsertHeadList(
                &raw mut LsapLogonContextList,
                &raw mut (*LogonContext).Entry as *mut _,
            );
        }
        Status = NtCompleteConnectPort(ConnectionHandle);
        if !(Status >= 0) {
            return Status;
        }
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn AuthPortThreadRoutine(_Param: *mut core::ffi::c_void) -> DWORD {
    let mut LogonContext: PLSAP_LOGON_CONTEXT = core::ptr::null_mut::<_LSAP_LOGON_CONTEXT>();
    let mut ReplyMsg: PLSA_API_MSG = core::ptr::null_mut::<_LSA_API_MSG>();
    // C initializes RequestMsg to {0}; zeroed() is exactly equivalent and avoids
    // c2rust's anonymous-struct type names (C2RustUnnamed_N) / field-name shapes
    // that don't match the bindgen LSA_API_MSG/PORT_MESSAGE union layout.
    let mut RequestMsg: LSA_API_MSG = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"AuthPortThreadRoutine\0" as *const u8 as *const core::ffi::c_char,
            238 as core::ffi::c_int,
            b"AuthPortThreadRoutine() called\n\0" as *const u8 as *const core::ffi::c_char,
        );
    }

    loop {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"AuthPortThreadRoutine\0" as *const u8 as *const core::ffi::c_char,
                244 as core::ffi::c_int,
                b"Reply: %p\n\0" as *const u8 as *const core::ffi::c_char,
                ReplyMsg,
            );
        }

        Status = NtReplyWaitReceivePort(
            AuthPortHandle,
            &raw mut LogonContext as *mut PVOID,
            ReplyMsg as PPORT_MESSAGE,
            &raw mut RequestMsg as PPORT_MESSAGE,
        );

        if Status < 0 {
            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"AuthPortThreadRoutine\0" as *const u8 as *const core::ffi::c_char,
                    251 as core::ffi::c_int,
                    b"NtReplyWaitReceivePort() failed (Status %lx)\n\0" as *const u8
                        as *const core::ffi::c_char,
                    Status,
                );
            }
            break;
        } else {
            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"AuthPortThreadRoutine\0" as *const u8 as *const core::ffi::c_char,
                    255 as core::ffi::c_int,
                    b"Received message\n\0" as *const u8 as *const core::ffi::c_char,
                );
            }

            match RequestMsg.h.u2.s2.Type as u32 {
                LPC_CONNECTION_REQUEST => {
                    if (*__wine_dbch___default).flags as core::ffi::c_int
                        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_TRACE,
                            __wine_dbch___default,
                            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                                as *const u8 as *const core::ffi::c_char,
                            b"AuthPortThreadRoutine\0" as *const u8
                                as *const core::ffi::c_char,
                            260 as core::ffi::c_int,
                            b"Port connection request\n\0" as *const u8
                                as *const core::ffi::c_char,
                        );
                    }
                    LsapHandlePortConnection(&raw mut RequestMsg);
                    ReplyMsg = core::ptr::null_mut::<_LSA_API_MSG>();
                }
                LPC_PORT_CLOSED => {
                    if (*__wine_dbch___default).flags as core::ffi::c_int
                        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_TRACE,
                            __wine_dbch___default,
                            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                                as *const u8 as *const core::ffi::c_char,
                            b"AuthPortThreadRoutine\0" as *const u8
                                as *const core::ffi::c_char,
                            266 as core::ffi::c_int,
                            b"Port closed\n\0" as *const u8 as *const core::ffi::c_char,
                        );
                    }
                    ReplyMsg = core::ptr::null_mut::<_LSA_API_MSG>();
                }
                LPC_CLIENT_DIED => {
                    if (*__wine_dbch___default).flags as core::ffi::c_int
                        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_TRACE,
                            __wine_dbch___default,
                            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                                as *const u8 as *const core::ffi::c_char,
                            b"AuthPortThreadRoutine\0" as *const u8
                                as *const core::ffi::c_char,
                            271 as core::ffi::c_int,
                            b"Client died\n\0" as *const u8 as *const core::ffi::c_char,
                        );
                    }
                    ReplyMsg = core::ptr::null_mut::<_LSA_API_MSG>();
                }
                _ => {
                    if (*__wine_dbch___default).flags as core::ffi::c_int
                        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_TRACE,
                            __wine_dbch___default,
                            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authport.c\0"
                                as *const u8 as *const core::ffi::c_char,
                            b"AuthPortThreadRoutine\0" as *const u8
                                as *const core::ffi::c_char,
                            276 as core::ffi::c_int,
                            b"Received request (ApiNumber: %lu)\n\0" as *const u8
                                as *const core::ffi::c_char,
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber as core::ffi::c_uint,
                        );
                    }

                    match RequestMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber as core::ffi::c_uint {
                        0 => {
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.Status =
                                LsapCallAuthenticationPackage(&raw mut RequestMsg, LogonContext);
                            ReplyMsg = &raw mut RequestMsg as PLSA_API_MSG;
                        }
                        1 => {
                            ReplyMsg = &raw mut RequestMsg as PLSA_API_MSG;
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_SUCCESS;
                            NtReplyPort(AuthPortHandle, &raw mut (*ReplyMsg).h as *mut _);
                            LsapDeregisterLogonProcess(&raw mut RequestMsg, LogonContext);
                            ReplyMsg = core::ptr::null_mut::<_LSA_API_MSG>();
                        }
                        2 => {
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.Status =
                                LsapLogonUser(&raw mut RequestMsg, LogonContext);
                            ReplyMsg = &raw mut RequestMsg as PLSA_API_MSG;
                        }
                        3 => {
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.Status =
                                LsapLookupAuthenticationPackage(&raw mut RequestMsg, LogonContext);
                            ReplyMsg = &raw mut RequestMsg as PLSA_API_MSG;
                        }
                        4 => {
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.Status =
                                LsapEnumLogonSessions(&raw mut RequestMsg);
                            ReplyMsg = &raw mut RequestMsg as PLSA_API_MSG;
                        }
                        5 => {
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.Status =
                                LsapGetLogonSessionData(&raw mut RequestMsg);
                            ReplyMsg = &raw mut RequestMsg as PLSA_API_MSG;
                        }
                        6 => {
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.Status =
                                LsapRegisterNotification(&raw mut RequestMsg);
                            ReplyMsg = &raw mut RequestMsg as PLSA_API_MSG;
                        }
                        _ => {
                            RequestMsg.__bindgen_anon_1.__bindgen_anon_1.Status =
                                STATUS_INVALID_SYSTEM_SERVICE;
                            ReplyMsg = &raw mut RequestMsg as PLSA_API_MSG;
                        }
                    }
                }
            }
        }
    }

    return STATUS_SUCCESS as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn StartAuthenticationPort() -> NTSTATUS {
    unsafe {
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut PortName: _UNICODE_STRING = core::mem::zeroed();
        let mut EventName: _UNICODE_STRING = core::mem::zeroed();
        let mut EventHandle: HANDLE = core::ptr::null_mut();
        let mut ThreadId: DWORD = 0;
        let mut Status: NTSTATUS;

        TRACE!(
            c"StartAuthenticationPort()\n".as_ptr() as *const core::ffi::c_char,
        );

        InitializeListHead(&raw mut LsapLogonContextList);

        static PORT_NAME: [u16; 23] = [
            0x5c, 0x4c, 0x73, 0x61, 0x41, 0x75, 0x74, 0x68, 0x65, 0x6e, 0x74, 0x69, 0x63, 0x61,
            0x74, 0x69, 0x6f, 0x6e, 0x50, 0x6f, 0x72, 0x74, 0,
        ];
        RtlInitUnicodeString(
            &raw mut PortName,
            PORT_NAME.as_ptr() as *const u16,
        );

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = 0;
        ObjectAttributes.ObjectName = &raw mut PortName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtCreatePort(
            &raw mut AuthPortHandle,
            &raw mut ObjectAttributes,
            core::mem::size_of::<LSA_CONNECTION_INFO>() as ULONG,
            core::mem::size_of::<LSA_API_MSG>() as ULONG,
            (core::mem::size_of::<LSA_API_MSG>() as ULONG).wrapping_mul(32),
        );
        if !NT_SUCCESS(Status) {
            WARN!(
                c"NtCreatePort() failed (Status %lx)\n".as_ptr() as *const core::ffi::c_char,
                Status,
            );
            return Status;
        }

        static EVENT_NAME: [u16; 41] = [
            0x5c, 0x53, 0x45, 0x43, 0x55, 0x52, 0x49, 0x54, 0x59, 0x5c, 0x4c, 0x53, 0x41, 0x5f,
            0x41, 0x55, 0x54, 0x48, 0x45, 0x4e, 0x54, 0x49, 0x43, 0x41, 0x54, 0x49, 0x4f, 0x4e,
            0x5f, 0x49, 0x4e, 0x49, 0x54, 0x49, 0x41, 0x4c, 0x49, 0x5a, 0x45, 0x44, 0,
        ];
        RtlInitUnicodeString(
            &raw mut EventName,
            EVENT_NAME.as_ptr() as *const u16,
        );

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_PERMANENT) as ULONG;
        ObjectAttributes.ObjectName = &raw mut EventName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenEvent(
            &raw mut EventHandle,
            EVENT_MODIFY_STATE as ACCESS_MASK,
            &raw mut ObjectAttributes,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                c"NtOpenEvent failed (Status 0x%08lx)\n".as_ptr() as *const core::ffi::c_char,
                Status,
            );
            Status = NtCreateEvent(
                &raw mut EventHandle,
                EVENT_MODIFY_STATE as ACCESS_MASK,
                &raw mut ObjectAttributes,
                _EVENT_TYPE_NotificationEvent,
                0u8,
            );
            if !NT_SUCCESS(Status) {
                WARN!(
                    c"NtCreateEvent failed (Status 0x%08lx)\n".as_ptr() as *const core::ffi::c_char,
                    Status,
                );
                return Status;
            }
        }

        Status = NtSetEvent(EventHandle, core::ptr::null_mut());
        NtClose(EventHandle);
        if !NT_SUCCESS(Status) {
            WARN!(
                c"NtSetEvent failed (Status 0x%08lx)\n".as_ptr() as *const core::ffi::c_char,
                Status,
            );
            return Status;
        }

        PortThreadHandle = CreateThread(
            core::ptr::null_mut(),
            0x1000,
            Some(AuthPortThreadRoutine as unsafe extern "system" fn(*mut core::ffi::c_void) -> u32),
            core::ptr::null_mut(),
            0,
            &raw mut ThreadId,
        );

        STATUS_SUCCESS
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/lsasrv/authport.c
//  * PURPOSE:     LsaAuthenticationPort server routines
//  * COPYRIGHT:   Copyright 2009 Eric Kohl
//  */
//
// #include "lsasrv.h"
//
// #include <ndk/lpcfuncs.h>
//
// static LIST_ENTRY LsapLogonContextList;
//
// static HANDLE PortThreadHandle = NULL;
// static HANDLE AuthPortHandle = NULL;
//
//
// /* FUNCTIONS ***************************************************************/
//
// static NTSTATUS
// LsapDeregisterLogonProcess(PLSA_API_MSG RequestMsg,
//                            PLSAP_LOGON_CONTEXT LogonContext)
// {
//     TRACE("LsapDeregisterLogonProcess(%p %p)\n", RequestMsg, LogonContext);
//
//     RemoveHeadList(&LogonContext->Entry);
//
//     NtClose(LogonContext->ClientProcessHandle);
//     NtClose(LogonContext->ConnectionHandle);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, LogonContext);
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// BOOL
// LsapIsTrustedClient(
//      HANDLE ProcessHandle)
// {
//     LUID TcbPrivilege = {SE_TCB_PRIVILEGE, 0};
//     HANDLE TokenHandle = NULL;
//     PTOKEN_PRIVILEGES Privileges = NULL;
//     ULONG Size, i;
//     BOOL Trusted = FALSE;
//     NTSTATUS Status;
//
//     Status = NtOpenProcessToken(ProcessHandle,
//                                 TOKEN_QUERY,
//                                 &TokenHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenPrivileges,
//                                      NULL,
//                                      0,
//                                      &Size);
//     if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     Privileges = RtlAllocateHeap(RtlGetProcessHeap(), 0, Size);
//     if (Privileges == NULL)
//         goto done;
//
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenPrivileges,
//                                      Privileges,
//                                      Size,
//                                      &Size);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     for (i = 0; i < Privileges->PrivilegeCount; i++)
//     {
//         if (RtlEqualLuid(&Privileges->Privileges[i].Luid, &TcbPrivilege))
//         {
//             Trusted = TRUE;
//             break;
//         }
//     }
//
// done:
//     if (Privileges != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Privileges);
//
//     if (TokenHandle != NULL)
//         NtClose(TokenHandle);
//
//     return Trusted;
// }
//
//
// static NTSTATUS
// LsapCheckLogonProcess(PLSA_API_MSG RequestMsg,
//                       PLSAP_LOGON_CONTEXT *LogonContext)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE ProcessHandle = NULL;
//     PLSAP_LOGON_CONTEXT Context = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsapCheckLogonProcess(%p)\n", RequestMsg);
//
//     TRACE("Client ID: %p %p\n", RequestMsg->h.ClientId.UniqueProcess, RequestMsg->h.ClientId.UniqueThread);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                0,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenProcess(&ProcessHandle,
//                            PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_DUP_HANDLE | PROCESS_QUERY_INFORMATION,
//                            &ObjectAttributes,
//                            &RequestMsg->h.ClientId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtOpenProcess() failed (Status %lx)\n", Status);
//         return Status;
//     }
//
//     /* Allocate the logon context */
//     Context = RtlAllocateHeap(RtlGetProcessHeap(),
//                               HEAP_ZERO_MEMORY,
//                               sizeof(LSAP_LOGON_CONTEXT));
//     if (Context == NULL)
//     {
//         NtClose(ProcessHandle);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     TRACE("New LogonContext: %p\n", Context);
//
//     Context->ClientProcessHandle = ProcessHandle;
//
//     switch (RequestMsg->ConnectInfo.TrustedCaller)
//     {
//         case NO:
//             Context->TrustedCaller = FALSE;
//             break;
//
//         case YES:
//             Context->TrustedCaller = TRUE;
//             break;
//
//         case CHECK:
//         default:
//             Context->TrustedCaller = LsapIsTrustedClient(ProcessHandle);
//             break;
//     }
//
//     TRACE("TrustedCaller: %u\n", Context->TrustedCaller);
//
//     *LogonContext = Context;
//
//     return STATUS_SUCCESS;
// }
//
//
// static NTSTATUS
// LsapHandlePortConnection(PLSA_API_MSG RequestMsg)
// {
//     PLSAP_LOGON_CONTEXT LogonContext = NULL;
//     HANDLE ConnectionHandle = NULL;
//     BOOLEAN Accept;
//     REMOTE_PORT_VIEW RemotePortView;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("LsapHandlePortConnection(%p)\n", RequestMsg);
//
//     TRACE("Logon Process Name: %s\n", RequestMsg->ConnectInfo.LogonProcessNameBuffer);
//
//     if (RequestMsg->ConnectInfo.CreateContext != FALSE)
//     {
//         Status = LsapCheckLogonProcess(RequestMsg,
//                                        &LogonContext);
//
//         RequestMsg->ConnectInfo.OperationalMode = 0x43218765;
//
//         RequestMsg->ConnectInfo.Status = Status;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         Accept = TRUE;
//     }
//     else
//     {
//         Accept = FALSE;
//     }
//
//     RemotePortView.Length = sizeof(REMOTE_PORT_VIEW);
//     Status = NtAcceptConnectPort(&ConnectionHandle,
//                                  (PVOID*)LogonContext,
//                                  &RequestMsg->h,
//                                  Accept,
//                                  NULL,
//                                  &RemotePortView);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtAcceptConnectPort failed (Status 0x%lx)\n", Status);
//         return Status;
//     }
//
//     if (Accept != FALSE)
//     {
//         if (LogonContext != NULL)
//         {
//             LogonContext->ConnectionHandle = ConnectionHandle;
//
//             InsertHeadList(&LsapLogonContextList,
//                            &LogonContext->Entry);
//         }
//
//         Status = NtCompleteConnectPort(ConnectionHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("NtCompleteConnectPort failed (Status 0x%lx)\n", Status);
//             return Status;
//         }
//     }
//
//     return Status;
// }
//
//
// NTSTATUS WINAPI
// AuthPortThreadRoutine(PVOID Param)
// {
//     PLSAP_LOGON_CONTEXT LogonContext;
//     PLSA_API_MSG ReplyMsg = NULL;
//     LSA_API_MSG RequestMsg;
//     NTSTATUS Status;
//
//     TRACE("AuthPortThreadRoutine() called\n");
//
//     Status = STATUS_SUCCESS;
//
//     for (;;)
//     {
//         TRACE("Reply: %p\n", ReplyMsg);
//         Status = NtReplyWaitReceivePort(AuthPortHandle,
//                                         (PVOID*)&LogonContext,
//                                         (PPORT_MESSAGE)ReplyMsg,
//                                         (PPORT_MESSAGE)&RequestMsg);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("NtReplyWaitReceivePort() failed (Status %lx)\n", Status);
//             break;
//         }
//
//         TRACE("Received message\n");
//
//         switch (RequestMsg.h.u2.s2.Type)
//         {
//             case LPC_CONNECTION_REQUEST:
//                 TRACE("Port connection request\n");
//                 Status = LsapHandlePortConnection(&RequestMsg);
//                 ReplyMsg = NULL;
//                 break;
//
//             case LPC_PORT_CLOSED:
//                 TRACE("Port closed\n");
//                 ReplyMsg = NULL;
//                 break;
//
//             case LPC_CLIENT_DIED:
//                 TRACE("Client died\n");
//                 ReplyMsg = NULL;
//                 break;
//
//             default:
//                 TRACE("Received request (ApiNumber: %lu)\n", RequestMsg.ApiNumber);
//
//                 switch (RequestMsg.ApiNumber)
//                 {
//                     case LSASS_REQUEST_CALL_AUTHENTICATION_PACKAGE:
//                         RequestMsg.Status = LsapCallAuthenticationPackage(&RequestMsg,
//                                                                           LogonContext);
//                         ReplyMsg = &RequestMsg;
//                         break;
//
//                     case LSASS_REQUEST_DEREGISTER_LOGON_PROCESS:
//
//                         ReplyMsg = &RequestMsg;
//                         RequestMsg.Status = STATUS_SUCCESS;
//                         NtReplyPort(AuthPortHandle,
//                                     &ReplyMsg->h);
//
//                         LsapDeregisterLogonProcess(&RequestMsg,
//                                                    LogonContext);
//
//                         ReplyMsg = NULL;
//                         break;
//
//                     case LSASS_REQUEST_LOGON_USER:
//                         RequestMsg.Status = LsapLogonUser(&RequestMsg,
//                                                           LogonContext);
//                         ReplyMsg = &RequestMsg;
//                         break;
//
//                     case LSASS_REQUEST_LOOKUP_AUTHENTICATION_PACKAGE:
//                         RequestMsg.Status = LsapLookupAuthenticationPackage(&RequestMsg,
//                                                                             LogonContext);
//                         ReplyMsg = &RequestMsg;
//                         break;
//
//                     case LSASS_REQUEST_ENUM_LOGON_SESSIONS:
//                         RequestMsg.Status = LsapEnumLogonSessions(&RequestMsg);
//                         ReplyMsg = &RequestMsg;
//                         break;
//
//                     case LSASS_REQUEST_GET_LOGON_SESSION_DATA:
//                         RequestMsg.Status = LsapGetLogonSessionData(&RequestMsg);
//                         ReplyMsg = &RequestMsg;
//                         break;
//
//                     case LSASS_REQUEST_POLICY_CHANGE_NOTIFY:
//                         RequestMsg.Status = LsapRegisterNotification(&RequestMsg);
//                         ReplyMsg = &RequestMsg;
//                         break;
//
//                     default:
//                         RequestMsg.Status = STATUS_INVALID_SYSTEM_SERVICE;
//                         ReplyMsg = &RequestMsg;
//                         break;
//                 }
//
//                 break;
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// StartAuthenticationPort(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING PortName;
//     DWORD ThreadId;
//     UNICODE_STRING EventName;
//     HANDLE EventHandle;
//     NTSTATUS Status;
//
//     TRACE("StartAuthenticationPort()\n");
//
//     /* Initialize the logon context list */
//     InitializeListHead(&LsapLogonContextList);
//
//     RtlInitUnicodeString(&PortName,
//                          L"\\LsaAuthenticationPort");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &PortName,
//                                0,
//                                NULL,
//                                NULL);
//
//     Status = NtCreatePort(&AuthPortHandle,
//                           &ObjectAttributes,
//                           sizeof(LSA_CONNECTION_INFO),
//                           sizeof(LSA_API_MSG),
//                           sizeof(LSA_API_MSG) * 32);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("NtCreatePort() failed (Status %lx)\n", Status);
//         return Status;
//     }
//
//     RtlInitUnicodeString(&EventName,
//                          L"\\SECURITY\\LSA_AUTHENTICATION_INITIALIZED");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &EventName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status = NtOpenEvent(&EventHandle,
//                          EVENT_MODIFY_STATE,
//                          &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtOpenEvent failed (Status 0x%08lx)\n", Status);
//
//         Status = NtCreateEvent(&EventHandle,
//                                EVENT_MODIFY_STATE,
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
//     Status = NtSetEvent(EventHandle, NULL);
//     NtClose(EventHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("NtSetEvent failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     PortThreadHandle = CreateThread(NULL,
//                                     0x1000,
//                                     (LPTHREAD_START_ROUTINE)AuthPortThreadRoutine,
//                                     NULL,
//                                     0,
//                                     &ThreadId);
//
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
