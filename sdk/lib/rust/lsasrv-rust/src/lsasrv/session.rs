use crate::prelude::*;
// srm.c (still C) — the SRM logon-session create/delete hooks.
extern "C" {
    fn LsapRmCreateLogonSession(LogonId: PLUID) -> NTSTATUS;
    fn LsapRmDeleteLogonSession(LogonId: PLUID) -> NTSTATUS;
    fn LsapTerminateLogon(LogonId: PLUID);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"lsasrv\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

pub static mut SessionListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

pub static mut SessionCount: ULONG = 0;

#[repr(C)]
pub struct _LSAP_LOGON_SESSION {
    pub Entry: LIST_ENTRY,
    pub LogonId: LUID,
    pub LogonType: u32,
    pub Session: u32,
    pub LogonTime: LARGE_INTEGER,
    pub Sid: PSID,
    pub UserName: _UNICODE_STRING,
    pub LogonDomain: _UNICODE_STRING,
    pub AuthenticationPackage: _UNICODE_STRING,
    pub LogonServer: _UNICODE_STRING,
    pub DnsDomainName: _UNICODE_STRING,
    pub Upn: _UNICODE_STRING,
}
pub type LSAP_LOGON_SESSION = _LSAP_LOGON_SESSION;
pub type PLSAP_LOGON_SESSION = *mut _LSAP_LOGON_SESSION;
#[no_mangle]
pub unsafe extern "C" fn LsapInitLogonSessions() {
    InitializeListHead(core::ptr::addr_of_mut!(SessionListHead));
    SessionCount = 0;
}

unsafe fn LsapGetLogonSession(logon_id: PLUID) -> PLSAP_LOGON_SESSION {
    let list_head = core::ptr::addr_of_mut!(SessionListHead);
    let mut session_entry: PLIST_ENTRY;
    let mut current_session: PLSAP_LOGON_SESSION;

    session_entry = (*list_head).Flink;
    while session_entry != list_head {
        let entry_offset = core::mem::offset_of!(LSAP_LOGON_SESSION, Entry);
        current_session = ((session_entry as usize).wrapping_sub(entry_offset)) as PLSAP_LOGON_SESSION;
        if (*current_session).LogonId.HighPart == (*logon_id).HighPart
            && (*current_session).LogonId.LowPart == (*logon_id).LowPart
        {
            return current_session;
        }
        session_entry = (*session_entry).Flink;
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn LsapSetLogonSessionData(
    LogonId: PLUID,
    LogonType: ULONG,
    UserName: PUNICODE_STRING,
    LogonDomain: PUNICODE_STRING,
    Sid: PSID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Session: PLSAP_LOGON_SESSION;
    let Length: ULONG;

    // TRACE("LsapSetLogonSessionData(%p)\n", LogonId);
    TRACE!(b"LsapSetLogonSessionData(%p)\n\0".as_ptr() as *const u8 as *const i8, LogonId);

    Session = LsapGetLogonSession(LogonId);
    if Session.is_null() {
        return STATUS_NO_SUCH_LOGON_SESSION;
    }

    // TRACE("LogonType %lu\n", LogonType);
    TRACE!(b"LogonType %lu\n\0".as_ptr() as *const u8 as *const i8, LogonType);

    (*Session).LogonType = LogonType;

    Status = RtlValidateUnicodeString(0u32, UserName as PCUNICODE_STRING);
    if !(Status >= 0) {
        return STATUS_INVALID_PARAMETER;
    }

    // TRACE("UserName %wZ\n", UserName);
    TRACE!(b"UserName %wZ\n\0".as_ptr() as *const u8 as *const i8, UserName);

    (*Session).UserName.Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        (*UserName).MaximumLength as SIZE_T,
    ) as PWSTR;
    if (*Session).UserName.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*Session).UserName.Length = (*UserName).Length;
    (*Session).UserName.MaximumLength = (*UserName).MaximumLength;
    core::ptr::copy_nonoverlapping(
        (*UserName).Buffer as *const u8,
        (*Session).UserName.Buffer as *mut u8,
        (*UserName).MaximumLength as usize,
    );

    Status = RtlValidateUnicodeString(0u32, LogonDomain as PCUNICODE_STRING);
    if !(Status >= 0) {
        // Cleanup and fail
        if !(*Session).UserName.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).UserName.Buffer as PVOID);
        }
        return STATUS_INVALID_PARAMETER;
    }

    // TRACE("LogonDomain %wZ\n", LogonDomain);
    TRACE!(b"LogonDomain %wZ\n\0".as_ptr() as *const u8 as *const i8, LogonDomain);

    if (*LogonDomain).Length != 0 {
        (*Session).LogonDomain.Buffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY as ULONG,
            (*LogonDomain).MaximumLength as SIZE_T,
        ) as PWSTR;
        if (*Session).LogonDomain.Buffer.is_null() {
            // Cleanup and fail
            if !(*Session).UserName.Buffer.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).UserName.Buffer as PVOID);
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        (*Session).LogonDomain.Length = (*LogonDomain).Length;
        (*Session).LogonDomain.MaximumLength = (*LogonDomain).MaximumLength;
        core::ptr::copy_nonoverlapping(
            (*LogonDomain).Buffer as *const u8,
            (*Session).LogonDomain.Buffer as *mut u8,
            (*LogonDomain).MaximumLength as usize,
        );
    } else {
        RtlInitEmptyUnicodeString(
            core::ptr::addr_of_mut!((*Session).LogonDomain),
            core::ptr::null_mut::<u16>(),
            0u16,
        );
    }

    Length = RtlLengthSid(Sid);
    (*Session).Sid = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        Length as SIZE_T,
    ) as PSID;
    if (*Session).Sid.is_null() {
        // Cleanup and fail
        if !(*Session).LogonDomain.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).LogonDomain.Buffer as PVOID);
        }
        if !(*Session).UserName.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).UserName.Buffer as PVOID);
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::copy_nonoverlapping(
        Sid as *const u8,
        (*Session).Sid as *mut u8,
        Length as usize,
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn LsapCreateLogonSession(LogonId: PLUID) -> NTSTATUS {
    TRACE!(
        c"LsapCreateLogonSession(%p)\n".as_ptr(),
        LogonId as *const core::ffi::c_void,
    );

    if !LsapGetLogonSession(LogonId).is_null() {
        return STATUS_LOGON_SESSION_COLLISION;
    }

    let session = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<_LSAP_LOGON_SESSION>() as SIZE_T,
    ) as PLSAP_LOGON_SESSION;

    if session.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyLuid(
        core::ptr::addr_of_mut!((*session).LogonId),
        LogonId,
    );

    TRACE!(
        c"LsapCreateLogonSession(<0x%lx,0x%lx>)\n".as_ptr(),
        (*LogonId).HighPart as u32,
        (*LogonId).LowPart as u32,
    );

    let status = LsapRmCreateLogonSession(LogonId);
    if !NT_SUCCESS(status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0u32, session as PVOID);
        return status;
    }

    InsertHeadList(
        &raw mut SessionListHead,
        &raw mut (*session).Entry,
    );
    SessionCount = SessionCount.wrapping_add(1);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsapDeleteLogonSession(LogonId: PLUID) -> NTSTATUS {
    let Session: *mut _LSAP_LOGON_SESSION = LsapGetLogonSession(LogonId);
    if Session.is_null() {
        return STATUS_NO_SUCH_LOGON_SESSION;
    }

    let Status: NTSTATUS = LsapRmDeleteLogonSession(LogonId);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    LsapTerminateLogon(LogonId);

    RemoveEntryList(core::ptr::addr_of_mut!((*Session).Entry));
    SessionCount -= 1;

    if !(*Session).Sid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).Sid as *mut core::ffi::c_void);
    }
    if !(*Session).UserName.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).UserName.Buffer as *mut core::ffi::c_void);
    }
    if !(*Session).LogonDomain.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).LogonDomain.Buffer as *mut core::ffi::c_void);
    }
    if !(*Session).AuthenticationPackage.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).AuthenticationPackage.Buffer as *mut core::ffi::c_void);
    }
    if !(*Session).LogonServer.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).LogonServer.Buffer as *mut core::ffi::c_void);
    }
    if !(*Session).DnsDomainName.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).DnsDomainName.Buffer as *mut core::ffi::c_void);
    }
    if !(*Session).Upn.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).Upn.Buffer as *mut core::ffi::c_void);
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, Session as *mut core::ffi::c_void);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn LsapAddCredential(
    _LogonId: PLUID,
    _AuthenticationPackage: ULONG,
    _PrimaryKeyValue: PLSA_STRING,
    _Credential: PLSA_STRING,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LsapGetCredentials(
    _LogonId: PLUID,
    _AuthenticationPackage: ULONG,
    _QueryContext: PULONG,
    _RetrieveAllCredentials: BOOLEAN,
    _PrimaryKeyValue: PLSA_STRING,
    _PrimaryKeyLength: PULONG,
    _Credentials: PLSA_STRING,
) -> NTSTATUS {
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn LsapDeleteCredential(
    _LogonId: PLUID,
    _AuthenticationPackage: u32,
    _PrimaryKeyValue: PLSA_STRING,
) -> NTSTATUS {
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapEnumLogonSessions(RequestMsg: PLSA_API_MSG) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let mut SessionEntry: PLIST_ENTRY;
    let mut CurrentSession: PLSAP_LOGON_SESSION;
    let SessionList: PLUID;
    let mut i: ULONG;
    let Length: ULONG;
    let mut MemSize: SIZE_T;
    let mut ClientBaseAddress: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0"
                .as_ptr() as *const i8,
            b"LsapEnumLogonSessions\0".as_ptr() as *const i8,
            311,
            b"LsapEnumLogonSessions(%p)\n\0".as_ptr() as *const i8,
            RequestMsg,
        );
    }

    Length = (SessionCount as usize).wrapping_mul(core::mem::size_of::<LUID>()) as ULONG;
    SessionList = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        Length as SIZE_T,
    ) as PLUID;
    if SessionList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    i = 0;
    SessionEntry = SessionListHead.Flink;
    while SessionEntry != core::ptr::addr_of_mut!(SessionListHead) {
        CurrentSession = (SessionEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(LSAP_LOGON_SESSION, Entry) as ULONG_PTR,
        ) as PLSAP_LOGON_SESSION;
        RtlCopyLuid(
            SessionList.offset(i as isize) as PLUID,
            core::ptr::addr_of_mut!((*CurrentSession).LogonId),
        );
        SessionEntry = (*SessionEntry).Flink;
        i = i.wrapping_add(1);
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = core::ptr::null_mut();
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenProcess(
        &mut ProcessHandle,
        (PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION) as u32,
        &mut ObjectAttributes,
        core::ptr::addr_of_mut!((*RequestMsg).h.__bindgen_anon_1.ClientId) as *mut _CLIENT_ID,
    );

    if Status >= 0 {
        if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0"
                    .as_ptr() as *const i8,
                b"LsapEnumLogonSessions\0".as_ptr() as *const i8,
                351,
                b"Length: %lu\n\0".as_ptr() as *const i8,
                Length,
            );
        }
        MemSize = Length as SIZE_T;
        Status = NtAllocateVirtualMemory(
            ProcessHandle,
            &mut ClientBaseAddress,
            0,
            &mut MemSize,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        if Status >= 0 {
            if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0"
                        .as_ptr() as *const i8,
                    b"LsapEnumLogonSessions\0".as_ptr() as *const i8,
                    366,
                    b"MemSize: %lu\n\0".as_ptr() as *const i8,
                    MemSize,
                );
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0"
                        .as_ptr() as *const i8,
                    b"LsapEnumLogonSessions\0".as_ptr() as *const i8,
                    367,
                    b"ClientBaseAddress: %p\n\0".as_ptr() as *const i8,
                    ClientBaseAddress,
                );
            }
            Status = NtWriteVirtualMemory(
                ProcessHandle,
                ClientBaseAddress,
                SessionList as PVOID,
                Length as SIZE_T,
                core::ptr::null_mut::<SIZE_T>(),
            );
            if Status >= 0 {
                (*RequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .EnumLogonSessions
                    .__bindgen_anon_1
                    .Reply
                    .LogonSessionCount = SessionCount;
                (*RequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .EnumLogonSessions
                    .__bindgen_anon_1
                    .Reply
                    .LogonSessionBuffer = ClientBaseAddress;
            } else {
                if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0"
                            .as_ptr() as *const i8,
                        b"LsapEnumLogonSessions\0".as_ptr() as *const i8,
                        376,
                        b"NtWriteVirtualMemory() failed (Status %lx)\n\0".as_ptr()
                            as *const i8,
                        Status,
                    );
                }
            }
        } else {
            if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0"
                        .as_ptr() as *const i8,
                    b"LsapEnumLogonSessions\0".as_ptr() as *const i8,
                    362,
                    b"NtAllocateVirtualMemory() failed (Status %lx)\n\0".as_ptr()
                        as *const i8,
                    Status,
                );
            }
        }
    } else {
        if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0"
                    .as_ptr() as *const i8,
                b"LsapEnumLogonSessions\0".as_ptr() as *const i8,
                347,
                b"NtOpenProcess() failed (Status %lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    }

    if !ProcessHandle.is_null() {
        NtClose(ProcessHandle);
    }
    if !SessionList.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, SessionList as PVOID);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapGetLogonSessionData(RequestMsg: PLSA_API_MSG) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut::<core::ffi::c_void>(),
        ObjectName: core::ptr::null_mut::<_UNICODE_STRING>(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
        SecurityQualityOfService: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let Session: PLSAP_LOGON_SESSION;
    let LocalSessionData: PSECURITY_LOGON_SESSION_DATA;
    let mut ClientBaseAddress: PVOID = core::ptr::null_mut();
    let mut TotalLength: ULONG;
    let mut SidLength: ULONG = 0;
    let mut MemSize: SIZE_T = 0;
    let mut Ptr: PUCHAR;
    let mut Status: NTSTATUS;

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
            b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
            407i32,
            b"LsapGetLogonSessionData(%p)\n\0".as_ptr() as *const i8,
            RequestMsg,
        );
    }

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
            b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
            409i32,
            b"LogonId: %lx\n\0".as_ptr() as *const i8,
            (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.GetLogonSessionData.__bindgen_anon_1.Request.LogonId.LowPart,
        );
    }

    Session = LsapGetLogonSession(
        core::ptr::addr_of_mut!((*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.GetLogonSessionData.__bindgen_anon_1.Request.LogonId) as *mut _LUID,
    );

    if Session.is_null() {
        return STATUS_NO_SUCH_LOGON_SESSION;
    }

    TotalLength = (core::mem::size_of::<SECURITY_LOGON_SESSION_DATA>() as usize)
        .wrapping_add((*Session).UserName.MaximumLength as usize)
        .wrapping_add((*Session).LogonDomain.MaximumLength as usize)
        .wrapping_add((*Session).AuthenticationPackage.MaximumLength as usize)
        .wrapping_add((*Session).LogonServer.MaximumLength as usize)
        .wrapping_add((*Session).DnsDomainName.MaximumLength as usize)
        .wrapping_add((*Session).Upn.MaximumLength as usize) as ULONG;

    if !(*Session).Sid.is_null() {
        SidLength = RtlLengthSid((*Session).Sid);
        TotalLength = (TotalLength as usize).wrapping_add(SidLength as usize) as ULONG;
    }

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
            b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
            427i32,
            b"TotalLength: %lu\n\0".as_ptr() as *const i8,
            TotalLength,
        );
    }

    LocalSessionData = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        TotalLength as SIZE_T,
    ) as PSECURITY_LOGON_SESSION_DATA;

    if LocalSessionData.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Ptr = (LocalSessionData as usize).wrapping_add(core::mem::size_of::<SECURITY_LOGON_SESSION_DATA>()) as PUCHAR;

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
            b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
            437i32,
            b"LocalSessionData: %p  Ptr: %p\n\0".as_ptr() as *const i8,
            LocalSessionData,
            Ptr,
        );
    }

    (*LocalSessionData).Size = core::mem::size_of::<SECURITY_LOGON_SESSION_DATA>() as ULONG;

    RtlCopyLuid(
        core::ptr::addr_of_mut!((*LocalSessionData).LogonId) as *mut _LUID,
        core::ptr::addr_of_mut!((*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.GetLogonSessionData.__bindgen_anon_1.Request.LogonId) as *mut _LUID,
    );

    (*LocalSessionData).UserName.Length = (*Session).UserName.Length;
    (*LocalSessionData).UserName.MaximumLength = (*Session).UserName.MaximumLength;
    if (*Session).UserName.MaximumLength as i32 != 0 {
        core::ptr::copy_nonoverlapping(
            (*Session).UserName.Buffer as *const u8,
            Ptr as *mut u8,
            (*Session).UserName.MaximumLength as usize,
        );
        (*LocalSessionData).UserName.Buffer = (Ptr as usize).wrapping_sub(LocalSessionData as usize) as PWSTR;
        Ptr = (Ptr as usize).wrapping_add((*Session).UserName.MaximumLength as usize) as PUCHAR;
    }

    (*LocalSessionData).LogonDomain.Length = (*Session).LogonDomain.Length;
    (*LocalSessionData).LogonDomain.MaximumLength = (*Session).LogonDomain.MaximumLength;
    if (*Session).LogonDomain.MaximumLength as i32 != 0 {
        core::ptr::copy_nonoverlapping(
            (*Session).LogonDomain.Buffer as *const u8,
            Ptr as *mut u8,
            (*Session).LogonDomain.MaximumLength as usize,
        );
        (*LocalSessionData).LogonDomain.Buffer = (Ptr as usize).wrapping_sub(LocalSessionData as usize) as PWSTR;
        Ptr = (Ptr as usize).wrapping_add((*Session).LogonDomain.MaximumLength as usize) as PUCHAR;
    }

    (*LocalSessionData).AuthenticationPackage.Length = (*Session).AuthenticationPackage.Length;
    (*LocalSessionData).AuthenticationPackage.MaximumLength = (*Session).AuthenticationPackage.MaximumLength;
    if (*Session).AuthenticationPackage.MaximumLength as i32 != 0 {
        core::ptr::copy_nonoverlapping(
            (*Session).AuthenticationPackage.Buffer as *const u8,
            Ptr as *mut u8,
            (*Session).AuthenticationPackage.MaximumLength as usize,
        );
        (*LocalSessionData).AuthenticationPackage.Buffer = (Ptr as usize).wrapping_sub(LocalSessionData as usize) as PWSTR;
        Ptr = (Ptr as usize).wrapping_add((*Session).AuthenticationPackage.MaximumLength as usize) as PUCHAR;
    }

    (*LocalSessionData).LogonType = (*Session).LogonType;
    (*LocalSessionData).Session = 0;

    if !(*Session).Sid.is_null() {
        core::ptr::copy_nonoverlapping(
            (*Session).Sid as *const u8,
            Ptr as *mut u8,
            SidLength as usize,
        );
        (*LocalSessionData).Sid = (Ptr as usize).wrapping_sub(LocalSessionData as usize) as PSID;
        Ptr = (Ptr as usize).wrapping_add(SidLength as usize) as PUCHAR;
    }

    (*LocalSessionData).LogonTime.QuadPart = (*Session).LogonTime.QuadPart;

    (*LocalSessionData).LogonServer.Length = (*Session).LogonServer.Length;
    (*LocalSessionData).LogonServer.MaximumLength = (*Session).LogonServer.MaximumLength;
    if (*Session).LogonServer.MaximumLength as i32 != 0 {
        core::ptr::copy_nonoverlapping(
            (*Session).LogonServer.Buffer as *const u8,
            Ptr as *mut u8,
            (*Session).LogonServer.MaximumLength as usize,
        );
        (*LocalSessionData).LogonServer.Buffer = (Ptr as usize).wrapping_sub(LocalSessionData as usize) as PWSTR;
        Ptr = (Ptr as usize).wrapping_add((*Session).LogonServer.MaximumLength as usize) as PUCHAR;
    }

    (*LocalSessionData).DnsDomainName.Length = (*Session).DnsDomainName.Length;
    (*LocalSessionData).DnsDomainName.MaximumLength = (*Session).DnsDomainName.MaximumLength;
    if (*Session).DnsDomainName.MaximumLength as i32 != 0 {
        core::ptr::copy_nonoverlapping(
            (*Session).DnsDomainName.Buffer as *const u8,
            Ptr as *mut u8,
            (*Session).DnsDomainName.MaximumLength as usize,
        );
        (*LocalSessionData).DnsDomainName.Buffer = (Ptr as usize).wrapping_sub(LocalSessionData as usize) as PWSTR;
        Ptr = (Ptr as usize).wrapping_add((*Session).DnsDomainName.MaximumLength as usize) as PUCHAR;
    }

    (*LocalSessionData).Upn.Length = (*Session).Upn.Length;
    (*LocalSessionData).Upn.MaximumLength = (*Session).Upn.MaximumLength;
    if (*Session).Upn.MaximumLength as i32 != 0 {
        core::ptr::copy_nonoverlapping(
            (*Session).Upn.Buffer as *const u8,
            Ptr as *mut u8,
            (*Session).Upn.MaximumLength as usize,
        );
        (*LocalSessionData).Upn.Buffer = (Ptr as usize).wrapping_sub(LocalSessionData as usize) as PWSTR;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>() as HANDLE;
    ObjectAttributes.Attributes = 0 as ULONG;
    ObjectAttributes.ObjectName = core::ptr::null_mut::<_UNICODE_STRING>();
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;

    Status = NtOpenProcess(
        core::ptr::addr_of_mut!(ProcessHandle),
        (PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION) as ACCESS_MASK,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!((*RequestMsg).h.__bindgen_anon_1.ClientId) as *mut _CLIENT_ID,
    );

    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
                b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
                538i32,
                b"NtOpenProcess() failed (Status %lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    } else {
        MemSize = TotalLength as SIZE_T;
        Status = NtAllocateVirtualMemory(
            ProcessHandle,
            core::ptr::addr_of_mut!(ClientBaseAddress),
            0 as ULONG_PTR,
            core::ptr::addr_of_mut!(MemSize),
            MEM_COMMIT as ULONG,
            PAGE_READWRITE as ULONG,
        );
        if !NT_SUCCESS(Status) {
            if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
                    b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
                    551i32,
                    b"NtAllocateVirtualMemory() failed (Status %lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
        } else {
            if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
                    b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
                    555i32,
                    b"MemSize: %lu\n\0".as_ptr() as *const i8,
                    MemSize,
                );
            }
            if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
                    b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
                    556i32,
                    b"ClientBaseAddress: %p\n\0".as_ptr() as *const i8,
                    ClientBaseAddress,
                );
            }
            Status = NtWriteVirtualMemory(
                ProcessHandle,
                ClientBaseAddress,
                LocalSessionData as PVOID,
                TotalLength as SIZE_T,
                core::ptr::null_mut::<ULONG_PTR>(),
            );
            if !NT_SUCCESS(Status) {
                if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/session.c\0".as_ptr() as *const i8,
                        b"LsapGetLogonSessionData\0".as_ptr() as *const i8,
                        565i32,
                        b"NtWriteVirtualMemory() failed (Status %lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                }
            } else {
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.GetLogonSessionData.__bindgen_anon_1.Reply.SessionDataBuffer = ClientBaseAddress;
            }
        }
    }

    if !ProcessHandle.is_null() {
        NtClose(ProcessHandle);
    }
    if !LocalSessionData.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap() as HANDLE,
            0 as ULONG,
            LocalSessionData as PVOID,
        );
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/lsasrv/session.c
//  * PURPOSE:     Logon session management routines
//  * COPYRIGHT:   Copyright 2013 Eric Kohl
//  */
//
// #include "lsasrv.h"
//
// typedef struct _LSAP_LOGON_SESSION
// {
//     LIST_ENTRY Entry;
//     LUID LogonId;
//     ULONG LogonType;
//     ULONG Session;
//     LARGE_INTEGER LogonTime;
//     PSID Sid;
//     UNICODE_STRING UserName;
//     UNICODE_STRING LogonDomain;
//     UNICODE_STRING AuthenticationPackage;
//     UNICODE_STRING LogonServer;
//     UNICODE_STRING DnsDomainName;
//     UNICODE_STRING Upn;
// } LSAP_LOGON_SESSION, *PLSAP_LOGON_SESSION;
//
//
// /* GLOBALS *****************************************************************/
//
// LIST_ENTRY SessionListHead;
// ULONG SessionCount;
//
// /* FUNCTIONS ***************************************************************/
//
// VOID
// LsapInitLogonSessions(VOID)
// {
//     InitializeListHead(&SessionListHead);
//     SessionCount = 0;
// }
//
//
// static
// PLSAP_LOGON_SESSION
// LsapGetLogonSession(IN PLUID LogonId)
// {
//     PLIST_ENTRY SessionEntry;
//     PLSAP_LOGON_SESSION CurrentSession;
//
//     SessionEntry = SessionListHead.Flink;
//     while (SessionEntry != &SessionListHead)
//     {
//         CurrentSession = CONTAINING_RECORD(SessionEntry,
//                                            LSAP_LOGON_SESSION,
//                                            Entry);
//         if (RtlEqualLuid(&CurrentSession->LogonId, LogonId))
//             return CurrentSession;
//
//         SessionEntry = SessionEntry->Flink;
//     }
//
//     return NULL;
// }
//
//
// NTSTATUS
// LsapSetLogonSessionData(
//      PLUID LogonId,
//      ULONG LogonType,
//      PUNICODE_STRING UserName,
//      PUNICODE_STRING LogonDomain,
//      PSID Sid)
// {
//     NTSTATUS Status;
//     PLSAP_LOGON_SESSION Session;
//     ULONG Length;
//
//     TRACE("LsapSetLogonSessionData(%p)\n", LogonId);
//
//     Session = LsapGetLogonSession(LogonId);
//     if (Session == NULL)
//         return STATUS_NO_SUCH_LOGON_SESSION;
//
//     TRACE("LogonType %lu\n", LogonType);
//     Session->LogonType = LogonType;
//
//     Status = RtlValidateUnicodeString(0, UserName);
//     if (!NT_SUCCESS(Status))
//         return STATUS_INVALID_PARAMETER;
//
//     /* UserName is mandatory and cannot be an empty string */
//     TRACE("UserName %wZ\n", UserName);
//     Session->UserName.Buffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                HEAP_ZERO_MEMORY,
//                                                UserName->MaximumLength);
//     if (Session->UserName.Buffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Session->UserName.Length = UserName->Length;
//     Session->UserName.MaximumLength = UserName->MaximumLength;
//     RtlCopyMemory(Session->UserName.Buffer, UserName->Buffer, UserName->MaximumLength);
//
//     Status = RtlValidateUnicodeString(0, LogonDomain);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup and fail */
//         if (Session->UserName.Buffer != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, Session->UserName.Buffer);
//
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* LogonDomain is optional and can be an empty string */
//     TRACE("LogonDomain %wZ\n", LogonDomain);
//     if (LogonDomain->Length)
//     {
//         Session->LogonDomain.Buffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                       HEAP_ZERO_MEMORY,
//                                                       LogonDomain->MaximumLength);
//         if (Session->LogonDomain.Buffer == NULL)
//         {
//             /* Cleanup and fail */
//             if (Session->UserName.Buffer != NULL)
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, Session->UserName.Buffer);
//
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         Session->LogonDomain.Length = LogonDomain->Length;
//         Session->LogonDomain.MaximumLength = LogonDomain->MaximumLength;
//         RtlCopyMemory(Session->LogonDomain.Buffer, LogonDomain->Buffer, LogonDomain->MaximumLength);
//     }
//     else
//     {
//         RtlInitEmptyUnicodeString(&Session->LogonDomain, NULL, 0);
//     }
//
//     Length = RtlLengthSid(Sid);
//     Session->Sid = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, Length);
//     if (Session->Sid == NULL)
//     {
//         /* Cleanup and fail */
//         if (Session->LogonDomain.Buffer != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, Session->LogonDomain.Buffer);
//         if (Session->UserName.Buffer != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, Session->UserName.Buffer);
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyMemory(Session->Sid, Sid, Length);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// LsapCreateLogonSession(IN PLUID LogonId)
// {
//     PLSAP_LOGON_SESSION Session;
//     NTSTATUS Status;
//
//     TRACE("LsapCreateLogonSession(%p)\n", LogonId);
//
//     /* Fail, if a session already exists */
//     if (LsapGetLogonSession(LogonId) != NULL)
//         return STATUS_LOGON_SESSION_COLLISION;
//
//     /* Allocate a new session entry */
//     Session = RtlAllocateHeap(RtlGetProcessHeap(),
//                               HEAP_ZERO_MEMORY,
//                               sizeof(LSAP_LOGON_SESSION));
//     if (Session == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Initialize the session entry */
//     RtlCopyLuid(&Session->LogonId, LogonId);
//
//     TRACE("LsapCreateLogonSession(<0x%lx,0x%lx>)\n",
//           LogonId->HighPart, LogonId->LowPart);
//
//     /* Tell ntoskrnl to create a new logon session */
//     Status = LsapRmCreateLogonSession(LogonId);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session);
//         return Status;
//     }
//
//     /* Insert the new session into the session list */
//     InsertHeadList(&SessionListHead, &Session->Entry);
//     SessionCount++;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// LsapDeleteLogonSession(IN PLUID LogonId)
// {
//     PLSAP_LOGON_SESSION Session;
//     NTSTATUS Status;
//
//     TRACE("LsapDeleteLogonSession(%p)\n", LogonId);
//
//     /* Fail, if the session does not exist */
//     Session = LsapGetLogonSession(LogonId);
//     if (Session == NULL)
//         return STATUS_NO_SUCH_LOGON_SESSION;
//
//     TRACE("LsapDeleteLogonSession(0x%08lx%08lx)\n",
//           LogonId->HighPart, LogonId->LowPart);
//
//     /* Tell ntoskrnl to delete the logon session */
//     Status = LsapRmDeleteLogonSession(LogonId);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Notify the authentication packages */
//     LsapTerminateLogon(LogonId);
//
//     /* Remove the session entry from the list */
//     RemoveEntryList(&Session->Entry);
//     SessionCount--;
//
//     /* Free the session data */
//     if (Session->Sid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->Sid);
//
//     if (Session->UserName.Buffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->UserName.Buffer);
//
//     if (Session->LogonDomain.Buffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->LogonDomain.Buffer);
//
//     if (Session->AuthenticationPackage.Buffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->AuthenticationPackage.Buffer);
//
//     if (Session->LogonServer.Buffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->LogonServer.Buffer);
//
//     if (Session->DnsDomainName.Buffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->DnsDomainName.Buffer);
//
//     if (Session->Upn.Buffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->Upn.Buffer);
//
//     /* Free the session entry */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Session);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// LsapAddCredential(
//      PLUID LogonId,
//      ULONG AuthenticationPackage,
//      PLSA_STRING PrimaryKeyValue,
//      PLSA_STRING Credential)
// {
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// LsapGetCredentials(
//      PLUID LogonId,
//      ULONG AuthenticationPackage,
//      PULONG QueryContext,
//      BOOLEAN RetrieveAllCredentials,
//      PLSA_STRING PrimaryKeyValue,
//      PULONG PrimaryKeyLength,
//      PLSA_STRING Credentials)
// {
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// LsapDeleteCredential(
//      PLUID LogonId,
//      ULONG AuthenticationPackage,
//      PLSA_STRING PrimaryKeyValue)
// {
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// LsapEnumLogonSessions(IN OUT PLSA_API_MSG RequestMsg)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE ProcessHandle = NULL;
//     PLIST_ENTRY SessionEntry;
//     PLSAP_LOGON_SESSION CurrentSession;
//     PLUID SessionList;
//     ULONG i, Length;
//     SIZE_T MemSize;
//     PVOID ClientBaseAddress = NULL;
//     NTSTATUS Status;
//
//     TRACE("LsapEnumLogonSessions(%p)\n", RequestMsg);
//
//     Length = SessionCount * sizeof(LUID);
//     SessionList = RtlAllocateHeap(RtlGetProcessHeap(),
//                                   HEAP_ZERO_MEMORY,
//                                   Length);
//     if (SessionList == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     i = 0;
//     SessionEntry = SessionListHead.Flink;
//     while (SessionEntry != &SessionListHead)
//     {
//         CurrentSession = CONTAINING_RECORD(SessionEntry,
//                                            LSAP_LOGON_SESSION,
//                                            Entry);
//
//         RtlCopyLuid(&SessionList[i],
//                     &CurrentSession->LogonId);
//
//         SessionEntry = SessionEntry->Flink;
//         i++;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                0,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenProcess(&ProcessHandle,
//                            PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
//                            &ObjectAttributes,
//                            &RequestMsg->h.ClientId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtOpenProcess() failed (Status %lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("Length: %lu\n", Length);
//
//     MemSize = Length;
//     Status = NtAllocateVirtualMemory(ProcessHandle,
//                                      &ClientBaseAddress,
//                                      0,
//                                      &MemSize,
//                                      MEM_COMMIT,
//                                      PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtAllocateVirtualMemory() failed (Status %lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("MemSize: %lu\n", MemSize);
//     TRACE("ClientBaseAddress: %p\n", ClientBaseAddress);
//
//     Status = NtWriteVirtualMemory(ProcessHandle,
//                                   ClientBaseAddress,
//                                   SessionList,
//                                   Length,
//                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtWriteVirtualMemory() failed (Status %lx)\n", Status);
//         goto done;
//     }
//
//     RequestMsg->EnumLogonSessions.Reply.LogonSessionCount = SessionCount;
//     RequestMsg->EnumLogonSessions.Reply.LogonSessionBuffer = ClientBaseAddress;
//
// done:
//     if (ProcessHandle != NULL)
//         NtClose(ProcessHandle);
//
//     if (SessionList != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, SessionList);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapGetLogonSessionData(IN OUT PLSA_API_MSG RequestMsg)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE ProcessHandle = NULL;
//     PLSAP_LOGON_SESSION Session;
//     PSECURITY_LOGON_SESSION_DATA LocalSessionData;
//     PVOID ClientBaseAddress = NULL;
//     ULONG TotalLength, SidLength = 0;
//     SIZE_T MemSize;
//     PUCHAR Ptr;
//     NTSTATUS Status;
//
//     TRACE("LsapGetLogonSessionData(%p)\n", RequestMsg);
//
//     TRACE("LogonId: %lx\n", RequestMsg->GetLogonSessionData.Request.LogonId.LowPart);
//     Session = LsapGetLogonSession(&RequestMsg->GetLogonSessionData.Request.LogonId);
//     if (Session == NULL)
//         return STATUS_NO_SUCH_LOGON_SESSION;
//
//     /* Calculate the required buffer size */
//     TotalLength = sizeof(SECURITY_LOGON_SESSION_DATA) +
//                   Session->UserName.MaximumLength +
//                   Session->LogonDomain.MaximumLength +
//                   Session->AuthenticationPackage.MaximumLength +
//                   Session->LogonServer.MaximumLength +
//                   Session->DnsDomainName.MaximumLength +
//                   Session->Upn.MaximumLength;
//     if (Session->Sid != NULL)
//     {
//         SidLength = RtlLengthSid(Session->Sid);
//         TotalLength += SidLength;
//     }
//     TRACE("TotalLength: %lu\n", TotalLength);
//
//     /* Allocate the buffer */
//     LocalSessionData = RtlAllocateHeap(RtlGetProcessHeap(),
//                                        HEAP_ZERO_MEMORY,
//                                        TotalLength);
//     if (LocalSessionData == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Ptr = (PUCHAR)((ULONG_PTR)LocalSessionData + sizeof(SECURITY_LOGON_SESSION_DATA));
//     TRACE("LocalSessionData: %p  Ptr: %p\n", LocalSessionData, Ptr);
//
//     LocalSessionData->Size = sizeof(SECURITY_LOGON_SESSION_DATA);
//
//     /* Copy the LogonId */
//     RtlCopyLuid(&LocalSessionData->LogonId,
//                 &RequestMsg->GetLogonSessionData.Request.LogonId);
//
//     /* Copy the UserName string */
//     LocalSessionData->UserName.Length = Session->UserName.Length;
//     LocalSessionData->UserName.MaximumLength = Session->UserName.MaximumLength;
//     if (Session->UserName.MaximumLength != 0)
//     {
//         RtlCopyMemory(Ptr, Session->UserName.Buffer, Session->UserName.MaximumLength);
//         LocalSessionData->UserName.Buffer = (PWSTR)((ULONG_PTR)Ptr - (ULONG_PTR)LocalSessionData);
//
//         Ptr = (PUCHAR)((ULONG_PTR)Ptr + Session->UserName.MaximumLength);
//     }
//
//     /* Copy the LogonDomain string */
//     LocalSessionData->LogonDomain.Length = Session->LogonDomain.Length;
//     LocalSessionData->LogonDomain.MaximumLength = Session->LogonDomain.MaximumLength;
//     if (Session->LogonDomain.MaximumLength != 0)
//     {
//         RtlCopyMemory(Ptr, Session->LogonDomain.Buffer, Session->LogonDomain.MaximumLength);
//         LocalSessionData->LogonDomain.Buffer = (PWSTR)((ULONG_PTR)Ptr - (ULONG_PTR)LocalSessionData);
//
//         Ptr = (PUCHAR)((ULONG_PTR)Ptr + Session->LogonDomain.MaximumLength);
//     }
//
//     /* Copy the AuthenticationPackage string */
//     LocalSessionData->AuthenticationPackage.Length = Session->AuthenticationPackage.Length;
//     LocalSessionData->AuthenticationPackage.MaximumLength = Session->AuthenticationPackage.MaximumLength;
//     if (Session->AuthenticationPackage.MaximumLength != 0)
//     {
//         RtlCopyMemory(Ptr, Session->AuthenticationPackage.Buffer, Session->AuthenticationPackage.MaximumLength);
//         LocalSessionData->AuthenticationPackage.Buffer = (PWSTR)((ULONG_PTR)Ptr - (ULONG_PTR)LocalSessionData);
//
//         Ptr = (PUCHAR)((ULONG_PTR)Ptr + Session->AuthenticationPackage.MaximumLength);
//     }
//
//     LocalSessionData->LogonType = Session->LogonType;
//     LocalSessionData->Session = 0;
//
//     /* Sid */
//     if (Session->Sid != NULL)
//     {
//         RtlCopyMemory(Ptr, Session->Sid, SidLength);
//         LocalSessionData->Sid = (PSID)((ULONG_PTR)Ptr - (ULONG_PTR)LocalSessionData);
//
//         Ptr = (PUCHAR)((ULONG_PTR)Ptr + SidLength);
//     }
//
//     /* LogonTime */
//     LocalSessionData->LogonTime.QuadPart = Session->LogonTime.QuadPart;
//
//     /* Copy the LogonServer string */
//     LocalSessionData->LogonServer.Length = Session->LogonServer.Length;
//     LocalSessionData->LogonServer.MaximumLength = Session->LogonServer.MaximumLength;
//     if (Session->LogonServer.MaximumLength != 0)
//     {
//         RtlCopyMemory(Ptr, Session->LogonServer.Buffer, Session->LogonServer.MaximumLength);
//         LocalSessionData->LogonServer.Buffer = (PWSTR)((ULONG_PTR)Ptr - (ULONG_PTR)LocalSessionData);
//
//         Ptr = (PUCHAR)((ULONG_PTR)Ptr + Session->LogonServer.MaximumLength);
//     }
//
//     /* Copy the DnsDomainName string */
//     LocalSessionData->DnsDomainName.Length = Session->DnsDomainName.Length;
//     LocalSessionData->DnsDomainName.MaximumLength = Session->DnsDomainName.MaximumLength;
//     if (Session->DnsDomainName.MaximumLength != 0)
//     {
//         RtlCopyMemory(Ptr, Session->DnsDomainName.Buffer, Session->DnsDomainName.MaximumLength);
//         LocalSessionData->DnsDomainName.Buffer = (PWSTR)((ULONG_PTR)Ptr - (ULONG_PTR)LocalSessionData);
//
//         Ptr = (PUCHAR)((ULONG_PTR)Ptr + Session->DnsDomainName.MaximumLength);
//     }
//
//     /* Copy the Upn string */
//     LocalSessionData->Upn.Length = Session->Upn.Length;
//     LocalSessionData->Upn.MaximumLength = Session->Upn.MaximumLength;
//     if (Session->Upn.MaximumLength != 0)
//     {
//         RtlCopyMemory(Ptr, Session->Upn.Buffer, Session->Upn.MaximumLength);
//         LocalSessionData->Upn.Buffer = (PWSTR)((ULONG_PTR)Ptr - (ULONG_PTR)LocalSessionData);
//
//         Ptr = (PUCHAR)((ULONG_PTR)Ptr + Session->Upn.MaximumLength);
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                0,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenProcess(&ProcessHandle,
//                            PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
//                            &ObjectAttributes,
//                            &RequestMsg->h.ClientId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtOpenProcess() failed (Status %lx)\n", Status);
//         goto done;
//     }
//
//     MemSize = TotalLength;
//     Status = NtAllocateVirtualMemory(ProcessHandle,
//                                      &ClientBaseAddress,
//                                      0,
//                                      &MemSize,
//                                      MEM_COMMIT,
//                                      PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtAllocateVirtualMemory() failed (Status %lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("MemSize: %lu\n", MemSize);
//     TRACE("ClientBaseAddress: %p\n", ClientBaseAddress);
//
//     Status = NtWriteVirtualMemory(ProcessHandle,
//                                   ClientBaseAddress,
//                                   LocalSessionData,
//                                   TotalLength,
//                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtWriteVirtualMemory() failed (Status %lx)\n", Status);
//         goto done;
//     }
//
//     RequestMsg->GetLogonSessionData.Reply.SessionDataBuffer = ClientBaseAddress;
//
// done:
//     if (ProcessHandle != NULL)
//         NtClose(ProcessHandle);
//
//     if (LocalSessionData != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, LocalSessionData);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
